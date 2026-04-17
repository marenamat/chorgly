// Shared mutable state behind a tokio RwLock.
// All business logic lives here so sessions can call it with the lock held.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use chorgly_core::{
  Chore, ChoreId, ChoreKind, Completion, Database, ServerMsg, User, UserId,
};

pub type Tx = broadcast::Sender<ServerMsg>;

pub struct SharedState {
  pub db: RwLock<Database>,
  pub broadcast: Tx,
  pub data_dir: PathBuf,
}

impl SharedState {
  /// Load DB from `<data_dir>/db.cbor` or start with an empty one.
  pub async fn load_or_default(data_dir: impl AsRef<Path>) -> Result<Self> {
    let data_dir = data_dir.as_ref().to_path_buf();
    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("db.cbor");
    let db = if db_path.exists() {
      let bytes = std::fs::read(&db_path)?;
      Database::from_cbor(&bytes)
        .unwrap_or_else(|e| { eprintln!("Corrupt DB, starting fresh: {e}"); Database::default() })
    } else {
      Database::default()
    };

    let (broadcast, _) = broadcast::channel(256);
    Ok(Self { db: RwLock::new(db), broadcast, data_dir })
  }

  // ------ auth ------

  pub async fn auth_user(&self, token: &str) -> Option<User> {
    let db = self.db.read().await;
    let now = Utc::now();
    db.user_by_token(token)
      .filter(|u| u.token_valid_at(now))
      .cloned()
  }

  // ------ chores ------

  pub async fn list_chores(&self) -> Vec<Chore> {
    self.db.read().await.chores.values().cloned().collect()
  }

  pub async fn add_chore(
    &self,
    title: String,
    kind: ChoreKind,
    assigned_to: Option<Vec<UserId>>,
    depends_on: Vec<ChoreId>,
    created_by: UserId,
  ) -> Chore {
    let chore = Chore {
      id: Uuid::new_v4(),
      title,
      kind,
      assigned_to,
      depends_on,
      created_at: Utc::now(),
      created_by,
      completions: vec![],
    };
    {
      let mut db = self.db.write().await;
      db.chores.insert(chore.id, chore.clone());
    }
    let _ = self.broadcast.send(ServerMsg::ChoreAdded(chore.clone()));
    chore
  }

  pub async fn complete_chore(
    &self,
    chore_id: ChoreId,
    by: UserId,
  ) -> Result<Chore, String> {
    let mut db = self.db.write().await;
    let chore = db.chores.get_mut(&chore_id).ok_or("chore not found")?;

    // Check that all dependencies are satisfied.
    let all_chores: Vec<Chore> = db.chores.values().cloned().collect();
    let blocked: Vec<ChoreId> = chore.blocked_by(all_chores.iter());
    if !blocked.is_empty() {
      return Err(format!("blocked by: {:?}", blocked));
    }

    chore.completions.push(Completion {
      completed_at: Utc::now(),
      completed_by: by,
    });
    let updated = chore.clone();
    drop(db);
    let _ = self.broadcast.send(ServerMsg::ChoreUpdated(updated.clone()));
    Ok(updated)
  }

  pub async fn delete_chore(
    &self,
    chore_id: ChoreId,
    by: UserId,
  ) -> Result<(), String> {
    let mut db = self.db.write().await;
    let chore = db.chores.get(&chore_id).ok_or("chore not found")?;

    // Only the creator may delete (TODO: add admin flag to User).
    if chore.created_by != by {
      return Err("only the creator may delete this chore".into());
    }
    db.chores.remove(&chore_id);
    drop(db);
    let _ = self.broadcast.send(ServerMsg::ChoreDeleted { chore_id });
    Ok(())
  }
}
