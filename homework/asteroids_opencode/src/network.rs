//! 网络模块 - 使用 JavaScript WebSocket (FFI)
//! 
//! 通过 JavaScript FFI 调用浏览器的 WebSocket API

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// 使用主模块的 GameMode
pub use crate::GameMode;

/// 客户端 → 服务器消息
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinQueue { mode: GameMode, nickname: String },
    LeaveQueue,
    GameInput { keys: Vec<String> },
    Ready,
    LeaveRoom,
    Ping,
}

/// 服务器 → 客户端消息
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Connected { player_id: String },
    MatchFound {
        room_id: String,
        players: Vec<String>,
        mode: GameMode,
    },
    GameStart,
    GameState {
        players: Vec<PlayerState>,
        asteroids: Vec<AsteroidState>,
        timestamp: i64,
    },
    PlayerDisconnected { player_id: String },
    GameOver {
        winner: Option<String>,
        scores: Vec<(String, u32)>,
    },
    Error { message: String },
    Pong,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerState {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub lives: u32,
    pub score: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AsteroidState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: u32,
}

/// WebSocket 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

// JavaScript FFI 函数
#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    // WebSocket 操作
    fn ws_connect(url_ptr: *const u8, url_len: usize) -> bool;
    fn ws_send(msg_ptr: *const u8, msg_len: usize) -> bool;
    fn ws_receive(buf_ptr: *mut u8, buf_len: usize) -> i32;
    fn ws_get_state(buf_ptr: *mut u8, buf_len: usize) -> i32;
    fn ws_is_connected() -> bool;
    fn ws_close();
    fn ws_message_count() -> i32;
}

/// 网络客户端
pub struct NetworkClient {
    pub state: ConnectionState,
    pub player_id: Option<String>,
    pub room_id: Option<String>,
    pub message_queue: VecDeque<ServerMessage>,
    server_url: String,
    pub latency_ms: f32,
    last_ping: f64,
}

impl NetworkClient {
    pub fn new(server_url: String) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            player_id: None,
            room_id: None,
            message_queue: VecDeque::new(),
            server_url,
            latency_ms: 0.0,
            last_ping: 0.0,
        }
    }

    /// 连接到服务器
    pub fn connect(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.state = ConnectionState::Connecting;
            
            // 调用 JavaScript
            let success = js_ws_connect(&self.server_url);
            if !success {
                self.state = ConnectionState::Error("Failed to initiate connection".to_string());
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = ConnectionState::Error("WebSocket only available in browser".to_string());
        }
    }

    /// 轮询网络事件（每帧调用）
    pub fn poll(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            // 更新连接状态
            let js_state = js_ws_get_state();
            self.state = match js_state.as_str() {
                "connected" => ConnectionState::Connected,
                "connecting" => ConnectionState::Connecting,
                "error" => ConnectionState::Error("Connection error".to_string()),
                _ => ConnectionState::Disconnected,
            };
            
            // 处理接收到的消息
            while let Some(msg_json) = js_ws_receive() {
                self.handle_raw_message(&msg_json);
            }
        }
    }

    /// 发送消息
    pub fn send(&mut self, message: ClientMessage) {
        if let Ok(json) = serde_json::to_string(&message) {
            #[cfg(target_arch = "wasm32")]
            {
                let success = js_ws_send(&json);
                if !success {
                    eprintln!("Failed to send message");
                }
            }
        }
    }

    /// 接收消息
    pub fn receive(&mut self) -> Option<ServerMessage> {
        self.message_queue.pop_front()
    }

    /// 处理原始 JSON 消息
    fn handle_raw_message(&mut self, json: &str) {
        match serde_json::from_str::<ServerMessage>(json) {
            Ok(message) => {
                // 更新内部状态
                match &message {
                    ServerMessage::Connected { player_id } => {
                        self.player_id = Some(player_id.clone());
                    }
                    ServerMessage::MatchFound { room_id, .. } => {
                        self.room_id = Some(room_id.clone());
                    }
                    ServerMessage::Pong => {
                        let now = macroquad::time::get_time();
                        self.latency_ms = ((now - self.last_ping) * 500.0) as f32;
                    }
                    _ => {}
                }
                self.message_queue.push_back(message);
            }
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
            }
        }
    }

    /// 发送心跳
    pub fn send_ping(&mut self, now: f64) {
        self.last_ping = now;
        self.send(ClientMessage::Ping);
    }

    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_ws_is_connected()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }

    /// 是否在房间中
    pub fn in_room(&self) -> bool {
        self.room_id.is_some()
    }

    /// 关闭连接
    pub fn close(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            js_ws_close();
        }
        
        self.disconnect();
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.player_id = None;
        self.room_id = None;
        self.message_queue.clear();
    }
}

// JavaScript 互操作辅助函数
#[cfg(target_arch = "wasm32")]
fn js_ws_connect(url: &str) -> bool {
    use std::ffi::CString;
    // 安全处理：移除 NUL 字符并处理错误
    let sanitized_url = url.replace('\0', "");
    match CString::new(sanitized_url.as_str()) {
        Ok(c_url) => unsafe { ws_connect(c_url.as_ptr() as *const u8, sanitized_url.len()) },
        Err(_) => {
            eprintln!("Invalid URL for WebSocket connection");
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn js_ws_send(msg: &str) -> bool {
    use std::ffi::CString;
    // 安全处理：移除 NUL 字符并处理错误
    let sanitized_msg = msg.replace('\0', "");
    match CString::new(sanitized_msg.as_str()) {
        Ok(c_msg) => unsafe { ws_send(c_msg.as_ptr() as *const u8, sanitized_msg.len()) },
        Err(_) => {
            eprintln!("Invalid message for WebSocket send");
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn js_ws_receive() -> Option<String> {
    // 增大缓冲区以处理较大消息
    const BUFFER_SIZE: usize = 8192;
    unsafe {
        let mut buf = vec![0u8; BUFFER_SIZE];
        let len = ws_receive(buf.as_mut_ptr(), buf.len());
        if len > 0 && (len as usize) <= BUFFER_SIZE {
            buf.truncate(len as usize);
            String::from_utf8(buf).ok()
        } else if len > BUFFER_SIZE as i32 {
            eprintln!("WebSocket message too large: {} bytes (max {})", len, BUFFER_SIZE);
            None
        } else {
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn js_ws_get_state() -> String {
    unsafe {
        let mut buf = vec![0u8; 64];
        let len = ws_get_state(buf.as_mut_ptr(), buf.len());
        if len > 0 {
            buf.truncate(len as usize);
            String::from_utf8(buf).unwrap_or_else(|_| "error".to_string())
        } else {
            "disconnected".to_string()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn js_ws_is_connected() -> bool {
    unsafe { ws_is_connected() }
}

#[cfg(target_arch = "wasm32")]
fn js_ws_close() {
    unsafe { ws_close() }
}
