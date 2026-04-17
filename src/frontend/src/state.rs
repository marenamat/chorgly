// Client-side application state (mirrors the server's view).
// Kept in Rust so we can do typed operations; exposed to JS via wasm-bindgen.

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use chorgly_core::{Chore, ChoreId, User};

#[wasm_bindgen]
pub struct AppState {
  pub(crate) current_user: Option<User>,
  pub(crate) chores: HashMap<ChoreId, Chore>,
}

#[wasm_bindgen]
impl AppState {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self { current_user: None, chores: HashMap::new() }
  }

  /// Returns true if the user is authenticated.
  pub fn is_authed(&self) -> bool {
    self.current_user.is_some()
  }

  /// Apply a server message (already decoded via decode_server_msg).
  /// Returns the name of the event fired so JS can re-render selectively.
  pub fn apply(&mut self, msg: JsValue) -> Result<String, JsValue> {
    let msg: chorgly_core::ServerMsg = serde_wasm_bindgen::from_value(msg)
      .map_err(|e| JsValue::from_str(&e.to_string()))?;

    use chorgly_core::ServerMsg::*;
    let event = match msg {
      AuthOk { user } => {
        self.current_user = Some(user);
        "auth_ok"
      }
      AuthFail { .. } => "auth_fail",
      Snapshot { chores } => {
        self.chores.clear();
        for c in chores { self.chores.insert(c.id, c); }
        "snapshot"
      }
      ChoreAdded(c) | ChoreUpdated(c) => {
        self.chores.insert(c.id, c);
        "chore_changed"
      }
      ChoreDeleted { chore_id } => {
        self.chores.remove(&chore_id);
        "chore_deleted"
      }
      Error { .. } => "error",
    };
    Ok(event.to_string())
  }

  /// Return chores as a JS array of objects, sorted by due date.
  pub fn pending_chores_json(&self) -> Result<JsValue, JsValue> {
    let now = js_sys::Date::now(); // ms since epoch
    let now_dt = chrono::DateTime::from_timestamp_millis(now as i64)
      .unwrap_or_else(chrono::Utc::now);

    let mut pending: Vec<&Chore> = self.chores.values()
      .filter(|c| c.next_due(now_dt).is_some())
      .collect();
    pending.sort_by_key(|c| c.next_due(now_dt));

    serde_wasm_bindgen::to_value(&pending)
      .map_err(|e| JsValue::from_str(&e.to_string()))
  }
}
