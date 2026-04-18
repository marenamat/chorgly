// In-memory database backed by CBOR files.
// Flushed to disk (and then committed to a git repo) at most once per hour.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Chore, ChoreId, User, UserId};
use crate::event::{EventId, ExternalEvent};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
  pub users: HashMap<UserId, User>,
  pub chores: HashMap<ChoreId, Chore>,
  pub events: HashMap<EventId, ExternalEvent>,
}

impl Database {
  /// Serialise the whole DB to a CBOR byte vector.
  pub fn to_cbor(&self) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(self, &mut buf)?;
    Ok(buf)
  }

  /// Deserialise a DB from CBOR bytes.
  pub fn from_cbor(bytes: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>> {
    ciborium::de::from_reader(bytes)
  }

  /// Look up a user by their current token.
  pub fn user_by_token(&self, token: &str) -> Option<&User> {
    self.users.values().find(|u| u.token == token)
  }
}
