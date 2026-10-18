//! scsynth-osc for the browser: what the Vite example calls. A typed form
//! is the serde form of the library's `Command` or `Reply`, and a raw form
//! is its `RawMessage`. Bytes cross as a `Uint8Array`, and an error as a
//! JS `Error` with the library's message.

use scsynth_osc::{Command, RawMessage, Reply};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// The OSC bytes of a command in its typed form.
///
/// # Errors
///
/// When `command` is not the typed form of a command.
#[wasm_bindgen(js_name = encodeCommand)]
pub fn encode_command(command: JsValue) -> Result<Vec<u8>, JsError> {
    let command: Command = serde_wasm_bindgen::from_value(command)?;
    Ok(command.encode())
}

/// The OSC bytes of a reply in its typed form.
///
/// # Errors
///
/// When `reply` is not the typed form of a reply.
#[wasm_bindgen(js_name = encodeReply)]
pub fn encode_reply(reply: JsValue) -> Result<Vec<u8>, JsError> {
    let reply: Reply = serde_wasm_bindgen::from_value(reply)?;
    Ok(reply.encode())
}

/// The OSC bytes of a message in its raw form.
///
/// # Errors
///
/// When `raw` is not the raw form of a message.
#[wasm_bindgen(js_name = encodeRaw)]
pub fn encode_raw(raw: JsValue) -> Result<Vec<u8>, JsError> {
    let raw: RawMessage = serde_wasm_bindgen::from_value(raw)?;
    Ok(raw.encode())
}

/// The typed form of the command in `bytes`.
///
/// # Errors
///
/// When `bytes` do not hold a command, decoded strictly.
#[wasm_bindgen(js_name = decodeCommand)]
pub fn decode_command(bytes: &[u8]) -> Result<JsValue, JsError> {
    to_js(&Command::decode(bytes)?)
}

/// The typed form of the reply in `bytes`.
///
/// # Errors
///
/// When `bytes` do not hold a reply, decoded strictly.
#[wasm_bindgen(js_name = decodeReply)]
pub fn decode_reply(bytes: &[u8]) -> Result<JsValue, JsError> {
    to_js(&Reply::decode(bytes)?)
}

/// The raw form of the message in `bytes`.
///
/// # Errors
///
/// When `bytes` do not hold one OSC message of the types of a raw message.
#[wasm_bindgen(js_name = decodeRaw)]
pub fn decode_raw(bytes: &[u8]) -> Result<JsValue, JsError> {
    to_js(&RawMessage::decode(bytes)?)
}

/// `value` as a JS value, with `None` as `null`, as in the library's JSON.
fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
    Ok(value.serialize(&serializer)?)
}
