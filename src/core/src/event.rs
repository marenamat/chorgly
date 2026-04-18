// External events: named occurrences that users tick off manually.
// Chores can declare dependencies on external events (depends_on_events).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::UserId;

pub type EventId = Uuid;

/// A named external event that one of the users must observe and confirm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEvent {
  pub id: EventId,
  pub name: String,
  pub description: String,

  /// True once a user has ticked this event off.
  pub triggered: bool,
  pub triggered_at: Option<DateTime<Utc>>,
  pub triggered_by: Option<UserId>,

  pub created_at: DateTime<Utc>,
  pub created_by: UserId,
}
