// chorgly-frontend: WASM module
// Exposes JS-callable functions for the UI glue layer.

use wasm_bindgen::prelude::*;
use chorgly_core::{ClientMsg, ServerMsg};

mod state;
mod render;

pub use state::AppState;

/// Decode a CBOR byte array from the server into a JSON string the JS can use.
#[wasm_bindgen]
pub fn decode_server_msg(bytes: &[u8]) -> Result<JsValue, JsValue> {
  let msg: ServerMsg = ciborium::de::from_reader(bytes)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  serde_wasm_bindgen::to_value(&msg)
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Encode a ClientMsg variant (provided as JSON) into CBOR bytes for sending.
#[wasm_bindgen]
pub fn encode_client_msg(json: &str) -> Result<Vec<u8>, JsValue> {
  let msg: ClientMsg = serde_json::from_str(json)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  let mut buf = Vec::new();
  ciborium::ser::into_writer(&msg, &mut buf)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  Ok(buf)
}
