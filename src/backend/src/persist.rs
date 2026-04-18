// Hourly persistence: write db.cbor and commit to a git repo.
// The data repo is initialised automatically if absent (Q2).

use std::sync::Arc;
use std::path::Path;
use tokio::time::{interval, Duration};

use crate::state::SharedState;

const FLUSH_INTERVAL: Duration = Duration::from_secs(3600);

pub async fn flush_loop(state: Arc<SharedState>) {
  let mut ticker = interval(FLUSH_INTERVAL);
  ticker.tick().await; // first tick fires immediately – skip it

  loop {
    ticker.tick().await;
    if let Err(e) = flush(&state).await {
      eprintln!("persist flush error: {e}");
    }
  }
}

async fn flush(state: &SharedState) -> anyhow::Result<()> {
  let cbor = {
    let db = state.db.read().await;
    db.to_cbor()?
  };

  let db_path = state.data_dir.join("db.cbor");
  std::fs::write(&db_path, &cbor)?;
  eprintln!("flushed {} bytes to {}", cbor.len(), db_path.display());

  // Commit to the data git repo, initialising it if absent (Q2).
  if let Err(e) = git_commit(&state.data_dir) {
    eprintln!("git commit error: {e}");
  }

  Ok(())
}

fn git_commit(data_dir: &Path) -> anyhow::Result<()> {
  // Open existing repo or initialise a new one (Q2).
  let repo = match git2::Repository::open(data_dir) {
    Ok(r) => r,
    Err(_) => {
      eprintln!("initialising new git repo at {}", data_dir.display());
      git2::Repository::init(data_dir)?
    }
  };

  let mut index = repo.index()?;
  index.add_path(Path::new("db.cbor"))?;
  index.write()?;

  let tree_id = index.write_tree()?;
  let tree = repo.find_tree(tree_id)?;

  let sig = git2::Signature::now("chorgly-backend", "chorgly@localhost")?;
  let head = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
  let parents: Vec<_> = head.iter().collect();

  repo.commit(Some("HEAD"), &sig, &sig, "chore: auto-flush db", &tree, &parents)?;
  eprintln!("committed data snapshot to git");
  Ok(())
}
