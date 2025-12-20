//! 网络模块 - 跨平台 WebSocket 客户端
//!
//! 使用 ewebsock 库实现跨平台 WebSocket 支持（原生和 WASM）。
//! NetworkGameMode 仅包含联网可用的游戏模式，确保与服务器协议一致。
//!
//! 注意：部分代码暂未使用，待在线功能完全实现后移除此 allow

#![allow(dead_code)]

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use macroquad::time::get_time;
use serde::{Deserialize, Serialize};

/// 消息队列最大容量，防止未消费消息无限堆积
const MAX_MESSAGE_QUEUE: usize = 256;
/// 客户端发送限流：每秒/每分钟最大消息数
const MAX_MESSAGES_PER_SECOND: usize = 10;
const MAX_MESSAGES_PER_MINUTE: usize = 50;
const RATE_LIMIT_WINDOW_SECONDS: f64 = 1.0;
const RATE_LIMIT_WINDOW_MINUTES: f64 = 60.0;
/// 待确认输入队列最大长度
const MAX_PENDING_INPUTS: usize = 120;

// ============================================================================
// 网络专用游戏模式（仅包含服务器支持的模式）
// ============================================================================

/// 网络游戏模式（仅 Survival 和 Duel 支持在线）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkGameMode {
    Survival,
    Duel,
}

impl NetworkGameMode {
    /// 从主菜单 GameMode 转换（不支持的模式返回 None）
    pub fn from_game_mode(mode: crate::GameMode) -> Option<Self> {
        match mode {
            crate::GameMode::Survival => Some(Self::Survival),
            crate::GameMode::Duel => Some(Self::Duel),
            _ => None,
        }
    }

    /// 转换为主菜单 GameMode
    pub fn to_game_mode(self) -> crate::GameMode {
        match self {
            Self::Survival => crate::GameMode::Survival,
            Self::Duel => crate::GameMode::Duel,
        }
    }
}

// ============================================================================
// 客户端 → 服务器消息
// ============================================================================

/// 客户端发送给服务器的消息
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 加入匹配队列
    JoinQueue {
        mode: NetworkGameMode,
        nickname: String,
        token: String,
    },
    /// 离开匹配队列
    LeaveQueue,
    /// 游戏输入（按键状态）
    GameInput {
        keys: Vec<String>,
        /// 输入序列号，用于服务器协调
        seq: u32,
    },
    /// 准备就绪
    Ready,
    /// 离开房间
    LeaveRoom,
    /// 心跳检测
    Ping,
}

// ============================================================================
// 服务器 → 客户端消息
// ============================================================================

/// 服务器发送给客户端的消息
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 连接成功，分配玩家 ID
    Connected { player_id: String },
    /// 匹配成功，进入房间
    MatchFound {
        room_id: String,
        players: Vec<String>,
        mode: NetworkGameMode,
    },
    /// 游戏开始
    GameStart,
    /// 游戏状态同步
    GameState {
        players: Vec<PlayerState>,
        asteroids: Vec<AsteroidState>,
        bullets: Vec<BulletState>,
        vortices: Vec<VortexState>,
        powerups: Vec<PowerupState>,
        timestamp: i64,
        /// 服务器已处理的各玩家最后输入序列号 (player_id -> seq)
        #[serde(default)]
        last_input_seqs: std::collections::HashMap<String, u32>,
    },
    /// 玩家断开连接
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
    /// 房间玩家列表更新
    RoomUpdate { players: Vec<String> },
}

/// 玩家状态（服务器同步）
#[derive(Debug, Deserialize, Clone)]
pub struct PlayerState {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub lives: u32,
    pub score: u32,
    pub alive: bool,
}

/// 小行星状态（服务器同步）
#[derive(Debug, Deserialize, Clone)]
pub struct AsteroidState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: u32,
    pub angle: f32,
}

/// 子弹状态（服务器同步）
#[derive(Debug, Deserialize, Clone)]
pub struct BulletState {
    pub id: u32,
    pub owner_id: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// 漩涡状态（服务器同步）
#[derive(Debug, Deserialize, Clone)]
pub struct VortexState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub radius: f32,
    pub created_at: f32,
    pub lifetime: f32,
}

/// 道具类型（与服务器 PowerupType 一致）
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PowerupType {
    Shield,
    DualShot,
    TripleShot,
}

/// 道具状态（服务器同步）
#[derive(Debug, Deserialize, Clone)]
pub struct PowerupState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub expires_at: f64,
    pub collected: bool,
    pub powerup_type: PowerupType,
}

// ============================================================================
// 客户端预测支持
// ============================================================================

/// 客户端本地输入命令（用于预测与重播）
#[derive(Debug, Clone)]
pub struct InputCommand {
    /// 输入序列号
    pub seq: u32,
    /// 按键状态
    pub keys: Vec<String>,
    /// 本地时间戳
    pub timestamp: f64,
    /// 输入对应的模拟步长（用于重播）
    pub dt: f32,
}

/// 预测状态快照（用于服务器协调后的重播）
#[derive(Debug, Clone)]
pub struct PredictedSnapshot {
    /// 对应的输入序列号
    pub seq: u32,
    /// 玩家位置 x
    pub x: f32,
    /// 玩家位置 y
    pub y: f32,
    /// 玩家朝向角度
    pub angle: f32,
    /// 玩家速度 x
    pub vel_x: f32,
    /// 玩家速度 y
    pub vel_y: f32,
}

/// 预测输入状态（解析后的按键状态）
#[derive(Debug, Clone, Default)]
pub struct ParsedInput {
    pub thrust: bool,
    pub left: bool,
    pub right: bool,
    pub shoot: bool,
}

impl ParsedInput {
    /// 从按键字符串列表解析输入状态
    pub fn from_keys(keys: &[String]) -> Self {
        Self {
            thrust: keys.iter().any(|k| k == "thrust"),
            left: keys.iter().any(|k| k == "left"),
            right: keys.iter().any(|k| k == "right"),
            shoot: keys.iter().any(|k| k == "shoot"),
        }
    }
}

// ============================================================================
// 连接状态
// ============================================================================

/// WebSocket 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// 未连接
    Disconnected,
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 连接错误
    Error(String),
}

// ============================================================================
// 发送限流
// ============================================================================

/// 发送频率超限详情
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    /// 窗口内允许的最大消息数
    pub max_messages: usize,
    /// 窗口长度（秒）
    pub window_secs: f64,
    /// 建议等待的时间（秒）
    pub retry_after_secs: f64,
}

impl RateLimitExceeded {
    fn new(oldest_timestamp: f64, now: f64, window_secs: f64, max_messages: usize) -> Self {
        let elapsed = (now - oldest_timestamp).max(0.0);
        let retry_after_secs = if elapsed >= window_secs {
            0.0
        } else {
            window_secs - elapsed
        };

        Self {
            max_messages,
            window_secs,
            retry_after_secs,
        }
    }
}

impl fmt::Display for RateLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "超过发送频率限制: {:.2}s 内最多 {} 条，建议 {:.2}s 后重试",
            self.window_secs, self.max_messages, self.retry_after_secs
        )
    }
}

impl Error for RateLimitExceeded {}

/// 滑动窗口发送限流器
#[derive(Debug)]
pub struct RateLimiter {
    per_second: VecDeque<f64>,
    per_minute: VecDeque<f64>,
    max_per_second: usize,
    max_per_minute: usize,
    window_second: f64,
    window_minute: f64,
}

impl RateLimiter {
    /// 创建限流器（默认窗口为 1 秒/60 秒）
    pub fn new(max_per_second: usize, max_per_minute: usize) -> Self {
        Self {
            per_second: VecDeque::new(),
            per_minute: VecDeque::new(),
            max_per_second,
            max_per_minute,
            window_second: RATE_LIMIT_WINDOW_SECONDS,
            window_minute: RATE_LIMIT_WINDOW_MINUTES,
        }
    }

    /// 检查并记录一次发送
    pub fn allow(&mut self, now: f64) -> Result<(), RateLimitExceeded> {
        Self::prune(&mut self.per_second, now, self.window_second);
        Self::prune(&mut self.per_minute, now, self.window_minute);

        if let Some(err) = Self::limit_error(
            &self.per_second,
            now,
            self.window_second,
            self.max_per_second,
        ) {
            return Err(err);
        }
        if let Some(err) = Self::limit_error(
            &self.per_minute,
            now,
            self.window_minute,
            self.max_per_minute,
        ) {
            return Err(err);
        }

        self.per_second.push_back(now);
        self.per_minute.push_back(now);
        Ok(())
    }

    /// 重置限流状态（断线或重连时使用）
    pub fn reset(&mut self) {
        self.per_second.clear();
        self.per_minute.clear();
    }

    fn prune(queue: &mut VecDeque<f64>, now: f64, window_secs: f64) {
        while let Some(&front) = queue.front() {
            if now - front >= window_secs {
                queue.pop_front();
            } else {
                break;
            }
        }
    }

    fn limit_error(
        queue: &VecDeque<f64>,
        now: f64,
        window_secs: f64,
        max_messages: usize,
    ) -> Option<RateLimitExceeded> {
        if queue.len() >= max_messages {
            let oldest = queue.front().copied().unwrap_or(now);
            Some(RateLimitExceeded::new(
                oldest,
                now,
                window_secs,
                max_messages,
            ))
        } else {
            None
        }
    }
}

/// 网络发送错误
#[derive(Debug)]
pub enum NetworkSendError {
    /// 未建立连接
    NotConnected,
    /// 序列化失败
    Serialize(serde_json::Error),
    /// 触发限流
    RateLimited(RateLimitExceeded),
}

impl fmt::Display for NetworkSendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "未连接到服务器"),
            Self::Serialize(err) => write!(f, "消息序列化失败: {}", err),
            Self::RateLimited(err) => write!(f, "发送过于频繁: {}", err),
        }
    }
}

/// 网络解析错误
#[derive(Debug)]
pub enum NetworkParseError {
    /// JSON 格式无效
    InvalidJson(serde_json::Error),
    /// 缺少消息类型字段
    MissingTypeField,
    /// 未知消息类型
    UnknownMessageType(String),
    /// 已知类型但字段不完整
    InvalidPayload {
        message_type: String,
        source: serde_json::Error,
    },
}

impl NetworkParseError {
    fn is_fatal(&self) -> bool {
        match self {
            Self::UnknownMessageType(_) => false,
            Self::InvalidJson(_)
            | Self::MissingTypeField
            | Self::InvalidPayload { .. } => true,
        }
    }
}

impl fmt::Display for NetworkParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(err) => write!(f, "JSON 无效: {}", err),
            Self::MissingTypeField => write!(f, "缺少 type 字段"),
            Self::UnknownMessageType(message_type) => {
                write!(f, "未知消息类型: {}", message_type)
            }
            Self::InvalidPayload {
                message_type,
                source,
            } => write!(f, "消息字段不完整: {} ({})", message_type, source),
        }
    }
}

impl Error for NetworkParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidJson(err) => Some(err),
            Self::InvalidPayload { source, .. } => Some(source),
            Self::MissingTypeField | Self::UnknownMessageType(_) => None,
        }
    }
}

// ============================================================================
// 网络客户端
// ============================================================================

/// 网络客户端（跨平台 WebSocket）
pub struct NetworkClient {
    /// 当前连接状态
    pub state: ConnectionState,
    /// 本地玩家 ID（连接后由服务器分配）
    pub player_id: Option<String>,
    /// 当前房间 ID
    pub room_id: Option<String>,
    /// 接收到的消息队列
    pub message_queue: VecDeque<ServerMessage>,
    /// 服务器 URL
    server_url: String,
    /// 网络延迟（毫秒）
    pub latency_ms: f32,
    /// 上次发送 Ping 的时间
    last_ping: f64,
    /// WebSocket 发送端
    sender: Option<WsSender>,
    /// WebSocket 接收端
    receiver: Option<WsReceiver>,
    /// 发送限流器
    rate_limiter: RateLimiter,
    // ---- 客户端预测相关 ----
    /// 当前输入序列号（单调递增）
    input_seq: u32,
    /// 待服务器确认的输入队列
    pending_inputs: VecDeque<InputCommand>,
    /// 最后确认的服务器状态
    last_confirmed_state: Option<PlayerState>,
}

impl NetworkClient {
    /// 创建新的网络客户端
    pub fn new(server_url: String) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            player_id: None,
            room_id: None,
            message_queue: VecDeque::new(),
            server_url,
            latency_ms: 0.0,
            last_ping: 0.0,
            sender: None,
            receiver: None,
            rate_limiter: RateLimiter::new(
                MAX_MESSAGES_PER_SECOND,
                MAX_MESSAGES_PER_MINUTE,
            ),
            // 客户端预测
            input_seq: 0,
            pending_inputs: VecDeque::new(),
            last_confirmed_state: None,
        }
    }

    /// 连接到服务器
    pub fn connect(&mut self) {
        // 如果已经有连接，先断开
        if self.sender.is_some() {
            self.disconnect();
        }

        self.state = ConnectionState::Connecting;

        // ewebsock::connect 返回 Result<(WsSender, WsReceiver), String>
        match ewebsock::connect(&self.server_url, ewebsock::Options::default()) {
            Ok((sender, receiver)) => {
                self.sender = Some(sender);
                self.receiver = Some(receiver);
                // 状态会在收到 Opened 事件后更新为 Connected
            }
            Err(err) => {
                self.state = ConnectionState::Error(format!("连接失败: {}", err));
            }
        }
    }

    /// 轮询网络事件（每帧调用）
    pub fn poll(&mut self) {
        // 先收集所有事件，避免借用冲突
        let events: Vec<WsEvent> = {
            let Some(receiver) = self.receiver.as_ref() else {
                return;
            };
            let mut events = Vec::new();
            while let Some(event) = receiver.try_recv() {
                events.push(event);
            }
            events
        };

        // 然后处理收集到的事件
        for event in events {
            match event {
                WsEvent::Opened => {
                    self.state = ConnectionState::Connected;
                    println!("[网络] WebSocket 连接已建立");
                }
                WsEvent::Closed => {
                    println!("[网络] WebSocket 连接已关闭");
                    self.state = ConnectionState::Disconnected;
                    self.sender = None;
                    self.receiver = None;
                    return; // receiver 已失效，退出循环
                }
                WsEvent::Error(err) => {
                    eprintln!("[网络] WebSocket 错误: {}", err);
                    self.state = ConnectionState::Error(err);
                    self.sender = None;
                    self.receiver = None;
                    return;
                }
                WsEvent::Message(msg) => match msg {
                    WsMessage::Text(text) => {
                        if let Err(err) = self.handle_raw_message(&text) {
                            if self.handle_parse_error(err) {
                                return;
                            }
                        }
                    }
                    WsMessage::Binary(data) => {
                        // 尝试将二进制数据解析为 UTF-8 文本
                        if let Ok(text) = String::from_utf8(data) {
                            if let Err(err) = self.handle_raw_message(&text) {
                                if self.handle_parse_error(err) {
                                    return;
                                }
                            }
                        }
                    }
                    WsMessage::Ping(_) => {
                        // ewebsock 会自动回复 Pong
                    }
                    WsMessage::Pong(_) => {
                        // 收到 Pong，计算延迟
                        let now = get_time();
                        self.latency_ms = ((now - self.last_ping) * 1000.0) as f32;
                    }
                    WsMessage::Unknown(_) => {
                        // 忽略未知消息类型
                    }
                },
            }
        }
    }

    /// 发送消息到服务器（带发送频率限制）
    ///
    /// # Errors
    /// - `NetworkSendError::NotConnected`：尚未建立连接
    /// - `NetworkSendError::RateLimited`：发送频率超限
    /// - `NetworkSendError::Serialize`：消息序列化失败
    pub fn send(&mut self, message: ClientMessage) -> Result<(), NetworkSendError> {
        let Some(sender) = self.sender.as_mut() else {
            return Err(NetworkSendError::NotConnected);
        };

        let json = serde_json::to_string(&message).map_err(NetworkSendError::Serialize)?;
        let now = get_time();
        if let Err(err) = self.rate_limiter.allow(now) {
            return Err(NetworkSendError::RateLimited(err));
        }

        sender.send(WsMessage::Text(json));
        Ok(())
    }

    /// 从队列中接收一条消息
    pub fn receive(&mut self) -> Option<ServerMessage> {
        self.message_queue.pop_front()
    }

    /// 处理原始 JSON 消息
    fn handle_raw_message(&mut self, json: &str) -> Result<(), NetworkParseError> {
        // 先解析为 Value 提取 type 字段，区分未知类型与字段缺失
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(NetworkParseError::InvalidJson)?;
        let message_type = value
            .get("type")
            .and_then(|value| value.as_str())
            .ok_or(NetworkParseError::MissingTypeField)?
            .to_string();
        if !matches!(
            message_type.as_str(),
            "Connected"
                | "MatchFound"
                | "GameStart"
                | "GameState"
                | "PlayerDisconnected"
                | "GameOver"
                | "Error"
                | "Pong"
                | "RoomUpdate"
        ) {
            return Err(NetworkParseError::UnknownMessageType(message_type));
        }

        let message: ServerMessage =
            serde_json::from_value(value).map_err(|err| NetworkParseError::InvalidPayload {
                message_type,
                source: err,
            })?;

        // 更新内部状态
        match &message {
            ServerMessage::Connected { player_id } => {
                self.player_id = Some(player_id.clone());
                println!("[网络] 已连接，玩家 ID: {}", player_id);
            }
            ServerMessage::MatchFound {
                room_id,
                players,
                mode,
            } => {
                self.room_id = Some(room_id.clone());
                println!(
                    "[网络] 匹配成功！房间: {}, 玩家: {:?}, 模式: {:?}",
                    room_id, players, mode
                );
            }
            ServerMessage::Pong => {
                let now = get_time();
                self.latency_ms = ((now - self.last_ping) * 1000.0) as f32;
            }
            ServerMessage::Error { message } => {
                eprintln!("[网络] 服务器错误: {}", message);
            }
            _ => {}
        }
        // 队列容量保护：超出上限时移除最老的消息
        while self.message_queue.len() >= MAX_MESSAGE_QUEUE {
            self.message_queue.pop_front();
        }
        self.message_queue.push_back(message);
        Ok(())
    }

    fn handle_parse_error(&mut self, err: NetworkParseError) -> bool {
        if err.is_fatal() {
            eprintln!("[网络] 消息解析失败，断开连接: {}", err);
            self.state = ConnectionState::Error(format!("消息解析失败: {}", err));
            self.sender = None;
            self.receiver = None;
            return true;
        }

        eprintln!("[网络] 忽略无法识别的消息: {}", err);
        false
    }
    pub fn send_ping(&mut self, now: f64) {
        // 限制发送频率（至少间隔 1 秒）
        if now - self.last_ping < 1.0 {
            return;
        }

        if let Some(sender) = self.sender.as_mut() {
            self.last_ping = now;
            // 发送 WebSocket Ping 帧
            sender.send(WsMessage::Ping(vec![]));
        }
    }

    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    /// 是否在房间中
    pub fn in_room(&self) -> bool {
        self.room_id.is_some()
    }

    // ========================================================================
    // 客户端预测 API
    // ========================================================================

    /// 发送游戏输入并记录到待确认队列（用于客户端预测）
    ///
    /// 调用方应在发送后立即在本地应用该输入进行预测模拟。
    /// 当收到服务器状态时，调用 `reconcile` 进行协调。
    ///
    /// # 参数
    /// - `keys`: 当前按下的按键列表
    /// - `timestamp`: 输入时间戳
    /// - `dt`: 当前帧的模拟步长（用于重播时保持一致）
    pub fn send_input(
        &mut self,
        keys: Vec<String>,
        timestamp: f64,
        dt: f32,
    ) -> Result<(), NetworkSendError> {
        let seq = self.input_seq;
        self.input_seq = self.input_seq.wrapping_add(1);

        // 记录输入到待确认队列（包含 dt 用于重播）
        let cmd = InputCommand {
            seq,
            keys: keys.clone(),
            timestamp,
            dt,
        };
        // 先发送到服务器，避免在发送失败时污染待确认队列
        self.send(ClientMessage::GameInput { keys, seq })?;
        self.pending_inputs.push_back(cmd);
        while self.pending_inputs.len() > MAX_PENDING_INPUTS {
            self.pending_inputs.pop_front();
        }

        Ok(())
    }

    /// 服务器状态协调
    ///
    /// 当收到 `GameState` 后调用此方法：
    /// 1. 更新 `last_confirmed_state` 为服务器的权威状态
    /// 2. 移除已被服务器确认的输入
    /// 3. 返回剩余未确认的输入，供调用方重播预测
    ///
    /// # 参数
    /// - `server_seq`: 服务器已处理的最后输入序列号
    /// - `server_state`: 服务器返回的玩家状态
    ///
    /// # 返回
    /// 未被服务器确认的输入列表（需要在服务器状态基础上重播）
    pub fn reconcile(&mut self, server_seq: u32, server_state: &PlayerState) -> Vec<InputCommand> {
        // 保存服务器权威状态
        self.last_confirmed_state = Some(server_state.clone());

        // 移除已确认的输入（seq <= server_seq）
        while let Some(front) = self.pending_inputs.front() {
            // 处理 wrapping: 如果 front.seq 在 server_seq 之前或等于，则移除
            let diff = server_seq.wrapping_sub(front.seq);
            if diff < 0x8000_0000 {
                // front.seq <= server_seq
                self.pending_inputs.pop_front();
            } else {
                break;
            }
        }

        // 返回剩余输入供重播
        self.pending_inputs.iter().cloned().collect()
    }

    /// 获取待确认输入数量（用于调试/UI显示）
    #[allow(dead_code)]
    pub fn pending_input_count(&self) -> usize {
        self.pending_inputs.len()
    }

    /// 获取最后确认的服务器状态
    #[allow(dead_code)]
    pub fn last_confirmed_state(&self) -> Option<&PlayerState> {
        self.last_confirmed_state.as_ref()
    }

    /// 清除预测状态（切换房间/断线重连时调用）
    pub fn reset_prediction_state(&mut self) {
        self.input_seq = 0;
        self.pending_inputs.clear();
        self.last_confirmed_state = None;
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        // 发送关闭消息（如果连接存在）
        if let Some(sender) = self.sender.take() {
            // ewebsock 的 sender 被 drop 时会自动关闭连接
            drop(sender);
        }
        self.receiver = None;
        self.state = ConnectionState::Disconnected;
        self.player_id = None;
        self.room_id = None;
        self.message_queue.clear();
        self.rate_limiter.reset();
        // 重置预测状态
        self.reset_prediction_state();
    }

    /// 获取服务器 URL
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// 设置服务器 URL（需要重新连接才能生效）
    pub fn set_server_url(&mut self, url: String) {
        self.server_url = url;
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_client_new() {
        let client = NetworkClient::new("wss://example.com/ws".to_string());
        assert!(matches!(client.state, ConnectionState::Disconnected));
        assert!(client.player_id.is_none());
        assert!(client.room_id.is_none());
        assert!(client.message_queue.is_empty());
        assert_eq!(client.latency_ms, 0.0);
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(
            ConnectionState::Disconnected,
            ConnectionState::Error("test".to_string())
        );
    }

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::LeaveQueue;
        let json = serde_json::to_string(&msg).expect("serialize");
        assert!(json.contains("LeaveQueue"));
    }

    #[test]
    fn test_client_message_join_queue_serialization() {
        let msg = ClientMessage::JoinQueue {
            mode: NetworkGameMode::Survival,
            nickname: "Player1".to_string(),
            token: "test-token".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("serialize");
        assert!(json.contains("JoinQueue"));
        assert!(json.contains("Survival"));
        assert!(json.contains("Player1"));
        assert!(json.contains("test-token"));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"Pong"}"#;
        let msg: ServerMessage = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(msg, ServerMessage::Pong));
    }

    #[test]
    fn test_server_message_connected_deserialization() {
        let json = r#"{"type":"Connected","player_id":"abc-123"}"#;
        let msg: ServerMessage = serde_json::from_str(json).expect("deserialize");
        if let ServerMessage::Connected { player_id } = msg {
            assert_eq!(player_id, "abc-123");
        } else {
            panic!("Expected Connected message");
        }
    }

    #[test]
    fn test_server_message_match_found_deserialization() {
        let json =
            r#"{"type":"MatchFound","room_id":"room-456","players":["p1","p2"],"mode":"Duel"}"#;
        let msg: ServerMessage = serde_json::from_str(json).expect("deserialize");
        if let ServerMessage::MatchFound {
            room_id,
            players,
            mode,
        } = msg
        {
            assert_eq!(room_id, "room-456");
            assert_eq!(players, vec!["p1", "p2"]);
            assert_eq!(mode, NetworkGameMode::Duel);
        } else {
            panic!("Expected MatchFound message");
        }
    }

    #[test]
    fn test_handle_raw_message_invalid_json_error() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());
        let err = client
            .handle_raw_message("not-json")
            .expect_err("should fail");
        assert!(matches!(&err, NetworkParseError::InvalidJson(_)));
        assert!(err.is_fatal());
    }

    #[test]
    fn test_handle_raw_message_unknown_type_error() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());
        let err = client
            .handle_raw_message(r#"{"type":"NewType"}"#)
            .expect_err("unknown type");
        assert!(matches!(
            &err,
            NetworkParseError::UnknownMessageType(_)
        ));
        assert!(!err.is_fatal());
    }

    #[test]
    fn test_handle_raw_message_invalid_payload_error() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());
        let err = client
            .handle_raw_message(r#"{"type":"Connected"}"#)
            .expect_err("missing fields");
        if let NetworkParseError::InvalidPayload { message_type, .. } = &err {
            assert_eq!(message_type, "Connected");
        } else {
            panic!("Expected InvalidPayload error");
        }
        assert!(err.is_fatal());
    }

    #[test]
    fn test_network_client_in_room() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());
        assert!(!client.in_room());

        client.room_id = Some("room-123".to_string());
        assert!(client.in_room());
    }

    #[test]
    fn test_network_game_mode_conversion() {
        assert_eq!(
            NetworkGameMode::from_game_mode(crate::GameMode::Survival),
            Some(NetworkGameMode::Survival)
        );
        assert_eq!(
            NetworkGameMode::from_game_mode(crate::GameMode::Duel),
            Some(NetworkGameMode::Duel)
        );
        assert_eq!(
            NetworkGameMode::from_game_mode(crate::GameMode::TimeAttack),
            None
        );
        assert_eq!(
            NetworkGameMode::from_game_mode(crate::GameMode::Online),
            None
        );
    }

    #[test]
    fn test_network_game_mode_to_game_mode() {
        assert_eq!(
            NetworkGameMode::Survival.to_game_mode(),
            crate::GameMode::Survival
        );
        assert_eq!(NetworkGameMode::Duel.to_game_mode(), crate::GameMode::Duel);
    }

    #[test]
    fn test_network_client_is_connected() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());
        assert!(!client.is_connected());

        client.state = ConnectionState::Connected;
        assert!(client.is_connected());
    }

    #[test]
    fn test_rate_limiter_per_second_limit() {
        let mut limiter = RateLimiter::new(
            MAX_MESSAGES_PER_SECOND,
            MAX_MESSAGES_PER_MINUTE,
        );
        let now = 0.0;
        for _ in 0..MAX_MESSAGES_PER_SECOND {
            limiter.allow(now).expect("within per-second limit");
        }
        let err = limiter.allow(now).expect_err("should hit per-second limit");
        assert_eq!(err.max_messages, MAX_MESSAGES_PER_SECOND);
        assert!((err.window_secs - RATE_LIMIT_WINDOW_SECONDS).abs() < 1e-9);
        assert!(err.retry_after_secs > 0.0);

        assert!(limiter.allow(RATE_LIMIT_WINDOW_SECONDS).is_ok());
    }

    #[test]
    fn test_rate_limiter_per_minute_limit() {
        let mut limiter = RateLimiter::new(
            MAX_MESSAGES_PER_SECOND,
            MAX_MESSAGES_PER_MINUTE,
        );
        for i in 0..MAX_MESSAGES_PER_MINUTE {
            limiter
                .allow(i as f64)
                .expect("within per-minute limit");
        }
        let err = limiter
            .allow((MAX_MESSAGES_PER_MINUTE - 1) as f64 + 0.5)
            .expect_err("should hit per-minute limit");
        assert_eq!(err.max_messages, MAX_MESSAGES_PER_MINUTE);
        assert!((err.window_secs - RATE_LIMIT_WINDOW_MINUTES).abs() < 1e-9);
        assert!(err.retry_after_secs > 0.0);

        assert!(limiter.allow(RATE_LIMIT_WINDOW_MINUTES + 0.1).is_ok());
    }

    // ========================================================================
    // 客户端预测测试
    // ========================================================================

    #[test]
    fn test_client_message_game_input_serialization() {
        let msg = ClientMessage::GameInput {
            keys: vec!["w".to_string(), "space".to_string()],
            seq: 42,
        };
        let json = serde_json::to_string(&msg).expect("serialize");
        assert!(json.contains("GameInput"));
        assert!(json.contains(r#""seq":42"#));
        assert!(json.contains("space"));
    }

    #[test]
    fn test_input_command_creation() {
        let cmd = InputCommand {
            seq: 10,
            keys: vec!["w".to_string()],
            timestamp: 1.5,
            dt: 0.016,
        };
        assert_eq!(cmd.seq, 10);
        assert_eq!(cmd.keys, vec!["w"]);
        assert!((cmd.timestamp - 1.5).abs() < f64::EPSILON);
        assert!((cmd.dt - 0.016).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reconcile_removes_confirmed_inputs() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());

        // 手动添加一些待确认输入
        client.pending_inputs.push_back(InputCommand {
            seq: 0,
            keys: vec!["w".to_string()],
            timestamp: 0.0,
            dt: 0.016,
        });
        client.pending_inputs.push_back(InputCommand {
            seq: 1,
            keys: vec!["a".to_string()],
            timestamp: 0.016,
            dt: 0.016,
        });
        client.pending_inputs.push_back(InputCommand {
            seq: 2,
            keys: vec!["d".to_string()],
            timestamp: 0.032,
            dt: 0.016,
        });

        // 模拟服务器确认到 seq=1
        let server_state = PlayerState {
            id: "player-1".to_string(),
            x: 100.0,
            y: 200.0,
            angle: 1.5,
            vel_x: 10.0,
            vel_y: 5.0,
            lives: 3,
            score: 100,
            alive: true,
        };

        let remaining = client.reconcile(1, &server_state);

        // seq 0 和 1 应该被移除，只剩 seq 2
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].seq, 2);
        assert_eq!(remaining[0].keys, vec!["d"]);

        // 确认状态应该被保存
        assert!(client.last_confirmed_state.is_some());
        let confirmed = client.last_confirmed_state.as_ref().unwrap();
        assert_eq!(confirmed.x, 100.0);
        assert_eq!(confirmed.y, 200.0);
    }

    #[test]
    fn test_reset_prediction_state() {
        let mut client = NetworkClient::new("wss://example.com/ws".to_string());

        // 模拟一些状态
        client.input_seq = 42;
        client.pending_inputs.push_back(InputCommand {
            seq: 41,
            keys: vec!["w".to_string()],
            timestamp: 0.0,
            dt: 0.016,
        });
        client.last_confirmed_state = Some(PlayerState {
            id: "test".to_string(),
            x: 0.0,
            y: 0.0,
            angle: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            lives: 3,
            score: 0,
            alive: true,
        });

        // 重置
        client.reset_prediction_state();

        assert_eq!(client.input_seq, 0);
        assert!(client.pending_inputs.is_empty());
        assert!(client.last_confirmed_state.is_none());
    }
}
