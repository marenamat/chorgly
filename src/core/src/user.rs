use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  pub id: UserId,
  pub name: String,
  /// Active session token (hex string). Rotated daily, expires after a week.
  pub token: String,
  pub token_issued_at: DateTime<Utc>,
  pub token_expires_at: DateTime<Utc>,
}

impl User {
  /// Checks whether the token is still valid at the given time.
  pub fn token_valid_at(&self, now: DateTime<Utc>) -> bool {
    now < self.token_expires_at
  }
}
