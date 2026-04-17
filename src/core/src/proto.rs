// WebSocket message types shared between backend and frontend.
// All messages are serialised as CBOR over the wire.

use serde::{Deserialize, Serialize};

use crate::{Chore, ChoreId, ChoreKind, User, UserId};

// ---------- client → server ----------

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
  /// First message after the WebSocket handshake. Must succeed before anything else.
  Auth { token: String },

  /// Request a full snapshot of the chore list.
  ListChores,

  /// Add a new chore. Server assigns the ID and timestamps.
  AddChore {
    title: String,
    kind: ChoreKind,
    /// None → common chore; Some([]) → same as common (allowed but odd).
    assigned_to: Option<Vec<UserId>>,
    depends_on: Vec<ChoreId>,
  },

  /// Mark a chore as done by the authenticated user.
  CompleteChore { chore_id: ChoreId },

  /// Delete a chore (only creator or admin can do this – enforcement is backend-side).
  DeleteChore { chore_id: ChoreId },
}

// ---------- server → client ----------

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
  /// Response to Auth.
  AuthOk { user: User },
  AuthFail { reason: String },

  /// Full snapshot sent after ListChores or on initial auth.
  Snapshot { chores: Vec<Chore> },

  /// Incremental update broadcast to all connected clients.
  ChoreAdded(Chore),
  ChoreUpdated(Chore),
  ChoreDeleted { chore_id: ChoreId },

  /// Generic error in response to a bad client message.
  Error { reason: String },
}
