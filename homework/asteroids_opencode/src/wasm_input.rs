// src/wasm_input.rs

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::Deserialize;

// This annotation tells Rust that the function is defined in JavaScript.
#[wasm_bindgen]
extern "C" {
    // The `get_input_state` function we defined in input_handler.js
    fn get_input_state() -> String;
}

// This struct matches the JSON structure returned by get_input_state()
#[derive(Deserialize, Debug, Default)]
pub struct WasmInputState {
    // Stores the state of keys that are currently held down.
    // e.g., {"KeyW": true, "Space": false, ...}
    #[serde(default)]
    pub down: HashMap<String, bool>,

    // Stores keys that were just pressed in this frame.
    // e.g., ["Enter", "KeyJ"]
    #[serde(default)]
    pub pressed: Vec<String>,

    #[serde(default)]
    pub mouse_wheel_y: f32,
}

/// Fetches the current input state from JavaScript.
/// This function should be called once per frame in the main game loop
/// when running under WASM.
pub fn get_input() -> WasmInputState {
    // Call the external JS function and deserialize its JSON response.
    // If deserialization fails, return a default, empty state.
    serde_json::from_str(&get_input_state()).unwrap_or_default()
}
