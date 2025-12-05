//! WASM 输入处理模块
//!
//! 通过 JavaScript FFI 获取输入状态，绕过 macroquad 的输入系统。
//! 仅在 WASM 构建时使用。

#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

// JavaScript FFI 声明（仅 WASM）
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    /// 从 JavaScript 获取输入状态 JSON
    fn get_input_state() -> *const u8;
    fn get_input_state_len() -> usize;
}

/// WASM 输入状态（从 JavaScript 获取）
#[derive(Deserialize, Debug, Default)]
pub struct WasmInputState {
    /// 当前按下的键 (e.g., {"KeyW": true, "Space": false})
    #[serde(default)]
    pub down: HashMap<String, bool>,

    /// 本帧刚按下的键 (e.g., ["Enter", "KeyJ"])
    #[serde(default)]
    pub pressed: Vec<String>,

    /// 鼠标滚轮 Y 轴变化
    #[serde(default)]
    pub mouse_wheel_y: f32,
}

/// 从 JavaScript 获取当前输入状态
///
/// 仅在 WASM 构建时有效，原生构建返回空状态。
#[cfg(target_arch = "wasm32")]
pub fn get_input() -> WasmInputState {
    unsafe {
        let ptr = get_input_state();
        let len = get_input_state_len();
        if ptr.is_null() || len == 0 {
            return WasmInputState::default();
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        if let Ok(s) = std::str::from_utf8(slice) {
            serde_json::from_str(s).unwrap_or_default()
        } else {
            WasmInputState::default()
        }
    }
}

/// 原生构建的占位实现
#[cfg(not(target_arch = "wasm32"))]
pub fn get_input() -> WasmInputState {
    WasmInputState::default()
}
