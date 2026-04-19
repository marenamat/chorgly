// WebSocket message types shared between backend and frontend.
// All messages are serialised as CBOR over the wire.

use serde::{Deserialize, Serialize};

use crate::{Chore, ChoreId, ChoreKind, ExternalEvent, User, UserId};
use crate::event::EventId;

// ---------- client → server ----------

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
  /// First message after the WebSocket handshake. Must succeed before anything else.
  Auth { token: String },

  /// Request a full snapshot of the chore and event lists.
  ListAll,

  /// Add a new chore. Server assigns the ID and timestamps.
  AddChore {
    title: String,
    kind: ChoreKind,
    /// Who can see this chore. None = everyone.
    visible_to: Option<Vec<UserId>>,
    /// Primary assignee. None = no specific assignee.
    assignee: Option<UserId>,
    /// Who may mark this chore done. None = everyone.
    can_complete: Option<Vec<UserId>>,
    depends_on: Vec<ChoreId>,
    depends_on_events: Vec<EventId>,
  },

  /// Mark a chore as done by the authenticated user.
  CompleteChore { chore_id: ChoreId },

  /// Delete a chore (only creator may do this).
  DeleteChore { chore_id: ChoreId },

  // --- external events (Q3) ---

  /// Declare a new external event that users must watch for.
  AddEvent { name: String, description: String },

  /// Mark an external event as having occurred.
  TriggerEvent { event_id: EventId },

  /// Remove an external event.
  DeleteEvent { event_id: EventId },
}

// ---------- server → client ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
  /// Response to Auth.
  AuthOk { user: User },
  AuthFail { reason: String },

  /// Full snapshot sent after ListAll or on initial auth.
  Snapshot { chores: Vec<Chore>, events: Vec<ExternalEvent> },

  /// Incremental updates broadcast to all connected clients.
  ChoreAdded(Chore),
  ChoreUpdated(Chore),
  ChoreDeleted { chore_id: ChoreId },

  EventAdded(ExternalEvent),
  EventUpdated(ExternalEvent),
  EventDeleted { event_id: EventId },

  /// Generic error in response to a bad client message.
  Error { reason: String },
}
