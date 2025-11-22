// src/input.rs

use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::wasm_input::{self, WasmInputState};

/// An abstraction layer for input handling to manage differences
/// between native and WASM platforms.
pub struct Input {
    #[cfg(target_arch = "wasm32")]
    state: WasmInputState,
}

impl Input {
    /// Creates a new Input instance, fetching the current frame's input state on WASM.
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            state: wasm_input::get_input(),
        }
    }

    /// A wrapper for macroquad's `is_key_pressed` function.
    /// On WASM, it uses our custom JavaScript input handler.
    /// On native, it calls the original macroquad function.
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return is_key_pressed(key_code);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let key_str = keycode_to_str(key_code);
            return self.state.pressed.iter().any(|k| k == key_str);
        }
    }

    /// A wrapper for macroquad's `is_key_down` function.
    /// On WASM, it uses our custom JavaScript input handler.
    /// On native, it calls the original macroquad function.
    pub fn is_key_down(&self, key_code: KeyCode) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return is_key_down(key_code);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let key_str = keycode_to_str(key_code);
            // .get(key_str) returns an Option<&bool>, so we unwrap_or(&false) and then dereference.
            return *self.state.down.get(key_str).unwrap_or(&false);
        }
    }

    /// A wrapper for macroquad's `mouse_wheel` function.
    /// On WASM, it uses our custom JavaScript input handler.
    /// On native, it calls the original macroquad function.
    pub fn mouse_wheel(&self) -> (f32, f32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return mouse_wheel();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Our JS handler only tracks the Y-axis scroll.
            return (0.0, self.state.mouse_wheel_y);
        }
    }
}

/// Maps a `KeyCode` enum to the string representation used by JavaScript's `event.code`.
#[cfg(target_arch = "wasm32")]
fn keycode_to_str(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Backspace => "Backspace",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",

        KeyCode::Up => "ArrowUp",
        KeyCode::Down => "ArrowDown",
        KeyCode::Left => "ArrowLeft",
        KeyCode::Right => "ArrowRight",

        KeyCode::W => "KeyW",
        KeyCode::A => "KeyA",
        KeyCode::S => "KeyS",
        KeyCode::D => "KeyD",
        KeyCode::P => "KeyP",
        KeyCode::M => "KeyM",
        KeyCode::J => "KeyJ",
        KeyCode::U => "KeyU",
        KeyCode::F => "KeyF",
        
        KeyCode::Key1 => "Digit1",
        KeyCode::Kp1 => "Numpad1",
        KeyCode::Key4 => "Digit4",
        KeyCode::Kp4 => "Numpad4",

        // Add other keys as needed...
        _ => "Unknown", // Return a string that will never match
    }
}
