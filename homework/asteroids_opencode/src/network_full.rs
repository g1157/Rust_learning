//! 网络模块 - WebSocket 客户端
//! 
//! 负责与服务器建立连接、发送和接收消息

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 游戏模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GameMode {
    Survival,
    Duel,
}

/// 客户端 → 服务器消息
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 加入匹配队列
    JoinQueue { mode: GameMode, nickname: String },
    /// 离开队列
    LeaveQueue,
    /// 游戏输入
    GameInput { keys: Vec<String> },
    /// 玩家准备
    Ready,
    /// 离开房间
    LeaveRoom,
    /// 心跳
    Ping,
}

/// 服务器 → 客户端消息
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 连接成功
    Connected { player_id: String },
    /// 匹配成功
    MatchFound {
        room_id: String,
        players: Vec<String>,
        mode: GameMode,
    },
    /// 游戏开始
    GameStart,
    /// 游戏状态更新
    GameState {
        players: Vec<PlayerState>,
        asteroids: Vec<AsteroidState>,
        timestamp: i64,
    },
    /// 玩家断线
    PlayerDisconnected { player_id: String },
    /// 游戏结束
    GameOver {
        winner: Option<String>,
        scores: Vec<(String, u32)>,
    },
    /// 错误消息
    Error { message: String },
    /// 心跳响应
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

/// 网络客户端
pub struct NetworkClient {
    pub state: ConnectionState,
    pub player_id: Option<String>,
    pub room_id: Option<String>,
    pub message_queue: VecDeque<ServerMessage>,
    pub pending_messages: VecDeque<String>, // 待发送的消息（JSON字符串）
    server_url: String,
    last_ping: f64,
    pub latency_ms: f32, // 网络延迟（毫秒）
}

impl NetworkClient {
    /// 创建新的网络客户端
    pub fn new(server_url: String) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            player_id: None,
            room_id: None,
            message_queue: VecDeque::new(),
            pending_messages: VecDeque::new(),
            server_url,
            last_ping: 0.0,
            latency_ms: 0.0,
        }
    }

    /// 标记为正在连接
    pub fn start_connecting(&mut self) {
        self.state = ConnectionState::Connecting;
    }

    /// 标记为已连接
    pub fn mark_connected(&mut self) {
        self.state = ConnectionState::Connected;
    }

    /// 标记为连接失败
    pub fn mark_error(&mut self, error: String) {
        self.state = ConnectionState::Error(error);
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.player_id = None;
        self.room_id = None;
        self.message_queue.clear();
        self.pending_messages.clear();
    }

    /// 发送消息到服务器（队列方式）
    pub fn send(&mut self, message: ClientMessage) {
        if let Ok(json) = serde_json::to_string(&message) {
            self.pending_messages.push_back(json);
        }
    }

    /// 获取待发送的消息
    pub fn pop_pending_message(&mut self) -> Option<String> {
        self.pending_messages.pop_front()
    }

    /// 接收来自服务器的消息
    pub fn receive(&mut self) -> Option<ServerMessage> {
        self.message_queue.pop_front()
    }

    /// 处理接收到的 JSON 消息
    pub fn handle_raw_message(&mut self, json: &str) {
        match serde_json::from_str::<ServerMessage>(json) {
            Ok(message) => {
                self.handle_message(message);
            }
            Err(e) => {
                eprintln!("Failed to parse server message: {}", e);
            }
        }
    }

    /// 处理接收到的消息
    pub fn handle_message(&mut self, message: ServerMessage) {
        match &message {
            ServerMessage::Connected { player_id } => {
                self.player_id = Some(player_id.clone());
                self.state = ConnectionState::Connected;
            }
            ServerMessage::MatchFound { room_id, .. } => {
                self.room_id = Some(room_id.clone());
            }
            ServerMessage::Error { message: err } => {
                self.state = ConnectionState::Error(err.clone());
            }
            ServerMessage::Pong => {
                // 计算延迟
                let now = crate::get_time();
                self.latency_ms = ((now - self.last_ping) * 500.0) as f32;
            }
            _ => {}
        }
        self.message_queue.push_back(message);
    }

    /// 发送心跳
    pub fn send_ping(&mut self, now: f64) {
        self.last_ping = now;
        self.send(ClientMessage::Ping);
    }

    /// 获取服务器 URL
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// 是否在房间中
    pub fn in_room(&self) -> bool {
        self.room_id.is_some()
    }
    
    /// 连接到服务器
    pub fn connect(&mut self) {
        self.start_connecting();
        
        #[cfg(target_arch = "wasm32")]
        {
            match wasm::connect(&self.server_url) {
                Ok(_) => {
                    // 连接成功会通过回调处理
                }
                Err(e) => {
                    self.mark_error(format!("连接失败: {:?}", e));
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match native::connect(&self.server_url) {
                Ok(_) => {
                    self.mark_connected();
                }
                Err(e) => {
                    self.mark_error(e);
                }
            }
        }
    }
    
    /// 轮询网络事件（应该每帧调用）
    pub fn poll(&mut self) {
        // 检查连接状态
        #[cfg(target_arch = "wasm32")]
        {
            if self.state == ConnectionState::Connecting && wasm::is_connected() {
                self.mark_connected();
            }
            
            // 处理接收到的消息
            while let Some(message) = wasm::receive_message() {
                self.handle_raw_message(&message);
            }
            
            // 发送待发送的消息
            while let Some(message) = self.pop_pending_message() {
                if let Err(e) = wasm::send_message(&message) {
                    eprintln!("发送消息失败: {:?}", e);
                    // 可以选择重新放回队列
                    break;
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 原生平台的轮询逻辑
            while let Some(message) = native::receive_message() {
                self.handle_raw_message(&message);
            }
        }
    }
    
    /// 关闭连接
    pub fn close(&mut self) {
        #[cfg(target_arch = "wasm32")]
        wasm::close();
        
        #[cfg(not(target_arch = "wasm32"))]
        native::close();
        
        self.disconnect();
    }
}

// WASM WebSocket 实现
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{ErrorEvent, MessageEvent, WebSocket};
    
    thread_local! {
        static WS_CONNECTION: RefCell<Option<WebSocket>> = RefCell::new(None);
        static MESSAGE_BUFFER: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
    }
    
    /// 连接到 WebSocket 服务器
    pub fn connect(url: &str) -> Result<(), JsValue> {
        let ws = WebSocket::new(url)?;
        
        // 设置二进制类型为文本
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // onopen 回调
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"WebSocket connected!".into());
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        // onmessage 回调
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let message = String::from(txt);
                MESSAGE_BUFFER.with(|buffer| {
                    buffer.borrow_mut().push_back(message);
                });
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // onerror 回调
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            web_sys::console::log_1(&format!("WebSocket error: {:?}", e.message()).into());
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        // onclose 回调
        let onclose_callback = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"WebSocket closed".into());
            WS_CONNECTION.with(|conn| {
                *conn.borrow_mut() = None;
            });
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        // 保存连接
        WS_CONNECTION.with(|conn| {
            *conn.borrow_mut() = Some(ws);
        });
        
        Ok(())
    }
    
    /// 发送消息
    pub fn send_message(message: &str) -> Result<(), JsValue> {
        WS_CONNECTION.with(|conn| {
            if let Some(ws) = conn.borrow().as_ref() {
                ws.send_with_str(message)
            } else {
                Err(JsValue::from_str("WebSocket not connected"))
            }
        })
    }
    
    /// 接收消息（从缓冲区）
    pub fn receive_message() -> Option<String> {
        MESSAGE_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop_front()
        })
    }
    
    /// 检查连接状态
    pub fn is_connected() -> bool {
        WS_CONNECTION.with(|conn| {
            conn.borrow().as_ref().map_or(false, |ws| {
                ws.ready_state() == WebSocket::OPEN
            })
        })
    }
    
    /// 关闭连接
    pub fn close() {
        WS_CONNECTION.with(|conn| {
            if let Some(ws) = conn.borrow().as_ref() {
                let _ = ws.close();
            }
            *conn.borrow_mut() = None;
        });
        MESSAGE_BUFFER.with(|buffer| {
            buffer.borrow_mut().clear();
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    // 原生平台暂不支持
    pub fn connect(_url: &str) -> Result<(), String> {
        Err("Native WebSocket not implemented".to_string())
    }
    
    pub fn send_message(_message: &str) -> Result<(), String> {
        Err("Native WebSocket not implemented".to_string())
    }
    
    pub fn receive_message() -> Option<String> {
        None
    }
    
    pub fn is_connected() -> bool {
        false
    }
    
    pub fn close() {}
}
