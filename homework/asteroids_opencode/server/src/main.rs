//! Asteroids 在线多人游戏服务器
//!
//! 基于 WebSocket 的实时游戏服务器
//! 支持房间系统、玩家匹配和游戏状态同步

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

type Tx = mpsc::UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<Uuid, Peer>>>;
type RoomMap = Arc<RwLock<HashMap<Uuid, Room>>>;

/// 游戏模式（与客户端 NetworkGameMode 一致）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum GameMode {
    Survival,
    Duel,
}

/// 道具类型（与客户端 PowerUpType 一致）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum PowerupType {
    Shield,
    DualShot,
    TripleShot,
}

/// 玩家信息
#[derive(Debug, Clone)]
#[allow(dead_code)] // id 用于将来的断线重连功能
struct Peer {
    id: Uuid,
    tx: Tx,
    room_id: Option<Uuid>,
    nickname: String,
}

/// 游戏房间
#[derive(Debug)]
struct Room {
    id: Uuid,
    mode: GameMode,
    players: Vec<Uuid>,
    max_players: usize,
    started: bool,
    game_state: Option<GameState>,
}

/// 服务器端游戏状态
#[derive(Debug, Clone)]
struct GameState {
    players: HashMap<Uuid, ServerPlayerState>,
    asteroids: Vec<ServerAsteroidState>,
    bullets: Vec<ServerBulletState>,
    vortices: Vec<ServerVortexState>,
    powerups: Vec<ServerPowerupState>,
    next_vortex_spawn: f32,
    next_powerup_spawn: f32,
    start_time: Instant,
    last_update: Instant,
}

/// 服务器端玩家状态
#[derive(Debug, Clone)]
struct ServerPlayerState {
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    angle: f32,
    lives: u32,
    score: u32,
    alive: bool,
    // 输入状态
    thrust: bool,
    turn_left: bool,
    turn_right: bool,
    shoot: bool,
    // 射击冷却（秒）
    shoot_cooldown: f32,
    // 无敌时间（秒，碰撞后重生保护）
    invulnerable_until: f32,
    // 上次收到输入的时间（秒，用于超时保护）
    last_input_at: f32,
    // 最后处理的输入序列号（用于客户端预测确认）
    last_input_seq: u32,
}

/// 服务器端小行星状态
#[derive(Debug, Clone)]
struct ServerAsteroidState {
    id: u32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: u32,
    angle: f32,
}

/// 服务器端子弹状态
#[derive(Debug, Clone)]
struct ServerBulletState {
    id: u32,
    owner_id: Uuid,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    lifetime: f32,
}

/// 服务器端漩涡状态
#[derive(Debug, Clone)]
struct ServerVortexState {
    id: u32,
    x: f32,
    y: f32,
    strength: f32,
    radius: f32,
    created_at: f32,
    lifetime: f32,
}

/// 服务器端道具状态
#[derive(Debug, Clone)]
struct ServerPowerupState {
    id: u32,
    x: f32,
    y: f32,
    expires_at: f64,
    collected: bool,
    powerup_type: PowerupType,
}

/// 客户端 → 服务器消息
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    /// 加入匹配队列
    JoinQueue { mode: GameMode, nickname: String },
    /// 离开队列
    LeaveQueue,
    /// 游戏输入
    GameInput {
        keys: Vec<String>,
        /// 输入序列号（用于客户端预测确认）
        #[serde(default)]
        seq: u32,
    },
    /// 玩家准备
    Ready,
    /// 离开房间
    LeaveRoom,
    /// 心跳
    Ping,
}

/// 服务器 → 客户端消息
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
#[allow(dead_code)] // 部分变体用于将来的功能（断线通知、错误处理、房间更新）
enum ServerMessage {
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
        bullets: Vec<BulletState>,
        vortices: Vec<VortexState>,
        powerups: Vec<PowerupState>,
        /// 服务器已处理的各玩家最后输入序列号 (player_id -> seq)
        last_input_seqs: HashMap<String, u32>,
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
    /// 房间玩家列表更新
    RoomUpdate { players: Vec<String> },
}

/// 网络传输用玩家状态（与客户端 PlayerState 匹配）
#[derive(Debug, Serialize, Clone)]
struct PlayerState {
    id: String,
    x: f32,
    y: f32,
    angle: f32,
    vel_x: f32,
    vel_y: f32,
    lives: u32,
    score: u32,
    alive: bool,
}

/// 网络传输用小行星状态（与客户端 AsteroidState 匹配）
#[derive(Debug, Serialize, Clone)]
struct AsteroidState {
    id: u32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: u32,
    angle: f32,
}

/// 网络传输用子弹状态（与客户端 BulletState 匹配）
#[derive(Debug, Serialize, Clone)]
struct BulletState {
    id: u32,
    owner_id: String,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

/// 网络传输用漩涡状态（与客户端 Vortex 匹配）
#[derive(Debug, Serialize, Clone)]
struct VortexState {
    id: u32,
    x: f32,
    y: f32,
    strength: f32,
    radius: f32,
    created_at: f32,
    lifetime: f32,
}

/// 网络传输用道具状态（与客户端 PowerUp 匹配）
#[derive(Debug, Serialize, Clone)]
struct PowerupState {
    id: u32,
    x: f32,
    y: f32,
    expires_at: f64,
    collected: bool,
    powerup_type: PowerupType,
}

impl Room {
    fn new(mode: GameMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            players: Vec::new(),
            max_players: 2, // 当前所有模式都是 2 人
            started: false,
            game_state: None,
        }
    }

    fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    fn add_player(&mut self, player_id: Uuid) -> bool {
        if !self.is_full() {
            self.players.push(player_id);
            true
        } else {
            false
        }
    }

    /// 初始化游戏状态
    fn init_game_state(&mut self, screen_width: f32, screen_height: f32) {
        let mut players = HashMap::new();
        let spawn_positions = [
            (screen_width * 0.25, screen_height * 0.5, 0.0), // 左侧
            (
                screen_width * 0.75,
                screen_height * 0.5,
                std::f32::consts::PI,
            ), // 右侧
        ];

        for (i, &player_id) in self.players.iter().enumerate() {
            let (x, y, angle) = spawn_positions.get(i).copied().unwrap_or((
                screen_width / 2.0,
                screen_height / 2.0,
                0.0,
            ));
            players.insert(
                player_id,
                ServerPlayerState {
                    x,
                    y,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    angle,
                    lives: 3,
                    score: 0,
                    alive: true,
                    thrust: false,
                    turn_left: false,
                    turn_right: false,
                    shoot: false,
                    shoot_cooldown: 0.0,
                    invulnerable_until: 0.0,
                    last_input_at: 0.0,
                    last_input_seq: 0,
                },
            );
        }

        // 生成初始小行星
        let asteroids = Self::spawn_initial_asteroids(screen_width, screen_height);

        // 调试：打印初始化的玩家状态
        println!("✅ Game state initialized:");
        for (pid, p) in &players {
            println!(
                "   Player {:?}: pos=({:.1}, {:.1}), angle={:.2}, shoot={}, turn_l={}, turn_r={}",
                pid, p.x, p.y, p.angle, p.shoot, p.turn_left, p.turn_right
            );
        }

        self.game_state = Some(GameState {
            players,
            asteroids,
            bullets: Vec::new(),
            vortices: Vec::new(),
            powerups: Vec::new(),
            next_vortex_spawn: game_constants::VORTEX_SPAWN_INTERVAL,
            next_powerup_spawn: game_constants::POWERUP_SPAWN_INTERVAL,
            start_time: Instant::now(),
            last_update: Instant::now(),
        });
    }

    /// 生成初始小行星
    fn spawn_initial_asteroids(screen_width: f32, screen_height: f32) -> Vec<ServerAsteroidState> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let count = 4; // 初始小行星数量

        (0..count)
            .map(|i| {
                // 在屏幕边缘生成
                let (x, y) = if rng.gen_bool(0.5) {
                    // 左右边缘
                    let x = if rng.gen_bool(0.5) { 0.0 } else { screen_width };
                    let y = rng.gen_range(0.0..screen_height);
                    (x, y)
                } else {
                    // 上下边缘
                    let x = rng.gen_range(0.0..screen_width);
                    let y = if rng.gen_bool(0.5) {
                        0.0
                    } else {
                        screen_height
                    };
                    (x, y)
                };

                let speed = rng.gen_range(30.0..80.0);
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);

                ServerAsteroidState {
                    id: i as u32,
                    x,
                    y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    size: 3, // 大型小行星
                    angle: rng.gen_range(0.0..std::f32::consts::TAU),
                }
            })
            .collect()
    }
}

async fn handle_client(stream: TcpStream, peer_map: PeerMap, room_map: RoomMap) {
    let addr = stream.peer_addr().expect("连接必须有地址");
    println!("🔗 新连接: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("❌ WebSocket 握手失败: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id = Uuid::new_v4();
    let peer = Peer {
        id: player_id,
        tx: tx.clone(),
        room_id: None,
        nickname: format!("Player_{}", &player_id.to_string()[..8]),
    };

    peer_map.write().await.insert(player_id, peer);

    // 发送连接成功消息
    let msg = ServerMessage::Connected {
        player_id: player_id.to_string(),
    };
    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap()));

    println!("✅ 玩家 {} 已连接", player_id);

    // 发送任务
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 接收任务
    let peer_map_clone = peer_map.clone();
    let room_map_clone = room_map.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) =
                        handle_message(player_id, text, &peer_map_clone, &room_map_clone).await
                    {
                        eprintln!("❌ 处理消息错误: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("🔌 玩家 {} 主动断开", player_id);
                    break;
                }
                Err(e) => {
                    eprintln!("❌ WebSocket 错误: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // 等待任务完成
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // 清理
    cleanup_player(player_id, &peer_map, &room_map).await;
    println!("👋 玩家 {} 已断开", player_id);
}

async fn handle_message(
    player_id: Uuid,
    text: String,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg: ClientMessage = serde_json::from_str(&text)?;

    match msg {
        ClientMessage::JoinQueue { mode, nickname } => {
            handle_join_queue(player_id, mode, nickname, peer_map, room_map).await?;
        }
        ClientMessage::LeaveQueue => {
            handle_leave_queue(player_id, peer_map, room_map).await?;
        }
        ClientMessage::GameInput { keys, seq } => {
            handle_game_input(player_id, keys, seq, peer_map, room_map).await?;
        }
        ClientMessage::Ready => {
            handle_ready(player_id, peer_map, room_map).await?;
        }
        ClientMessage::LeaveRoom => {
            handle_leave_room(player_id, peer_map, room_map).await?;
        }
        ClientMessage::Ping => {
            // 发送 Pong
            if let Some(peer) = peer_map.read().await.get(&player_id) {
                let msg = ServerMessage::Pong;
                let _ = peer.tx.send(Message::Text(serde_json::to_string(&msg)?));
            }
        }
    }

    Ok(())
}

async fn handle_join_queue(
    player_id: Uuid,
    mode: GameMode,
    nickname: String,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    // 更新昵称
    if let Some(peer) = peer_map.write().await.get_mut(&player_id) {
        peer.nickname = nickname;
    }

    // 查找可用房间
    let mut rooms = room_map.write().await;
    let available_room = rooms
        .values_mut()
        .find(|room| room.mode == mode && !room.is_full() && !room.started);

    let room_id = if let Some(room) = available_room {
        // 加入现有房间
        room.add_player(player_id);
        room.id
    } else {
        // 创建新房间
        let mut new_room = Room::new(mode);
        new_room.add_player(player_id);
        let room_id = new_room.id;
        rooms.insert(room_id, new_room);
        room_id
    };

    // 更新玩家房间
    if let Some(peer) = peer_map.write().await.get_mut(&player_id) {
        peer.room_id = Some(room_id);
    }

    // 检查房间是否满员
    if let Some(room) = rooms.get(&room_id).filter(|r| r.is_full()) {
        // 通知所有玩家匹配成功
        let player_nicknames: Vec<String> = {
            let peers = peer_map.read().await;
            room.players
                .iter()
                .filter_map(|id| peers.get(id))
                .map(|p| p.nickname.clone())
                .collect()
        };

        let msg = ServerMessage::MatchFound {
            room_id: room_id.to_string(),
            players: player_nicknames,
            mode: room.mode,
        };

        let msg_text = serde_json::to_string(&msg)?;
        let peers = peer_map.read().await;
        for &pid in &room.players {
            if let Some(peer) = peers.get(&pid) {
                let _ = peer.tx.send(Message::Text(msg_text.clone()));
            }
        }

        println!("🎮 房间 {} 匹配成功，模式: {:?}", room_id, room.mode);
    }

    Ok(())
}

async fn handle_leave_queue(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let room_id = peer_map
        .read()
        .await
        .get(&player_id)
        .and_then(|p| p.room_id);

    if let Some(rid) = room_id {
        let mut rooms = room_map.write().await;
        if let Some(room) = rooms.get_mut(&rid) {
            room.players.retain(|&id| id != player_id);
            if room.players.is_empty() {
                rooms.remove(&rid);
                println!("🗑️  房间 {} 已删除（无玩家）", rid);
            }
        }
    }

    if let Some(peer) = peer_map.write().await.get_mut(&player_id) {
        peer.room_id = None;
    }

    Ok(())
}

async fn handle_game_input(
    player_id: Uuid,
    keys: Vec<String>,
    seq: u32,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    // 获取玩家所在房间
    let room_id = peer_map
        .read()
        .await
        .get(&player_id)
        .and_then(|p| p.room_id);

    if let Some(rid) = room_id {
        let mut rooms = room_map.write().await;
        if let Some(room) = rooms.get_mut(&rid)
            && let Some(ref mut state) = room.game_state
            && let Some(player) = state.players.get_mut(&player_id)
        {
            // 调试：记录输入更新（仅当有变化时）
            let new_shoot = keys.contains(&"shoot".to_string());
            let new_turn_left = keys.contains(&"left".to_string());
            let new_turn_right = keys.contains(&"right".to_string());
            if new_shoot != player.shoot || new_turn_left != player.turn_left || new_turn_right != player.turn_right {
                println!(
                    "🎮 Input {:?}: shoot={}->{}, left={}->{}, right={}->{}",
                    player_id, player.shoot, new_shoot, player.turn_left, new_turn_left, player.turn_right, new_turn_right
                );
            }
            // 更新玩家输入状态
            player.thrust = keys.contains(&"thrust".to_string());
            player.turn_left = new_turn_left;
            player.turn_right = new_turn_right;
            player.shoot = new_shoot;
            // 更新最后输入时间
            player.last_input_at = state.start_time.elapsed().as_secs_f32();
            // 更新最后处理的输入序列号（用于客户端预测确认）
            player.last_input_seq = seq;
        }
    }

    Ok(())
}

async fn handle_ready(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let room_id = peer_map
        .read()
        .await
        .get(&player_id)
        .and_then(|p| p.room_id);

    if let Some(rid) = room_id {
        let should_start = {
            let mut rooms = room_map.write().await;
            if let Some(room) = rooms.get_mut(&rid) {
                if room.is_full() && !room.started {
                    room.started = true;
                    // 初始化游戏状态（使用客户端匹配的屏幕尺寸）
                    room.init_game_state(
                        game_constants::SCREEN_WIDTH,
                        game_constants::SCREEN_HEIGHT,
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_start {
            // 通知所有玩家游戏开始
            let msg = ServerMessage::GameStart;
            let msg_text = serde_json::to_string(&msg)?;

            let player_ids = {
                let rooms = room_map.read().await;
                rooms
                    .get(&rid)
                    .map(|r| r.players.clone())
                    .unwrap_or_default()
            };

            let peers = peer_map.read().await;
            for &pid in &player_ids {
                if let Some(peer) = peers.get(&pid) {
                    let _ = peer.tx.send(Message::Text(msg_text.clone()));
                }
            }
            drop(peers);

            println!("🚀 房间 {} 游戏开始！", rid);

            // 启动游戏循环
            let peer_map_clone = peer_map.clone();
            let room_map_clone = room_map.clone();
            tokio::spawn(async move {
                game_loop(rid, peer_map_clone, room_map_clone).await;
            });
        }
    }

    Ok(())
}

/// 游戏主循环（每个房间独立运行）
async fn game_loop(room_id: Uuid, peer_map: PeerMap, room_map: RoomMap) {
    const TICK_RATE: u64 = 30; // 30 FPS 网络同步
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_RATE));

    loop {
        interval.tick().await;

        // 检查房间是否还存在
        let room_exists = {
            let rooms = room_map.read().await;
            rooms.contains_key(&room_id)
        };

        if !room_exists {
            println!("🛑 房间 {} 游戏循环结束（房间已删除）", room_id);
            break;
        }

        // 更新游戏状态
        let game_over = {
            let mut rooms = room_map.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if !room.started {
                    true // 游戏已停止
                } else if let Some(ref mut state) = room.game_state {
                    let dt = state.last_update.elapsed().as_secs_f32();
                    state.last_update = Instant::now();

                    // 更新游戏物理（使用客户端匹配的屏幕尺寸）
                    update_game_physics(
                        state,
                        dt,
                        game_constants::SCREEN_WIDTH,
                        game_constants::SCREEN_HEIGHT,
                        room.mode,
                    );

                    // 更新漩涡和道具
                    update_world_events(
                        state,
                        game_constants::SCREEN_WIDTH,
                        game_constants::SCREEN_HEIGHT,
                    );

                    // 检查游戏结束条件
                    check_game_over(state, room.mode)
                } else {
                    true
                }
            } else {
                true
            }
        };

        if game_over {
            // 发送游戏结束消息
            send_game_over(room_id, &peer_map, &room_map).await;
            break;
        }

        // 广播游戏状态
        broadcast_game_state(room_id, &peer_map, &room_map).await;
    }
}

/// 游戏常量（与客户端匹配）
mod game_constants {
    // 屏幕尺寸（与客户端 WINDOW_WIDTH/WINDOW_HEIGHT 一致）
    pub const SCREEN_WIDTH: f32 = 1024.0;
    pub const SCREEN_HEIGHT: f32 = 768.0;

    pub const SHIP_ACCEL: f32 = 200.0;
    pub const SHIP_ROTATION_SPEED: f32 = 4.0;
    pub const MAX_SPEED: f32 = 300.0;
    pub const FRICTION: f32 = 0.99;
    pub const SHIP_RADIUS: f32 = 12.5; // 约 SHIP_HEIGHT / 2
    pub const BULLET_SPEED: f32 = 500.0; // 子弹速度
    pub const BULLET_RADIUS: f32 = 3.0;
    pub const BULLET_LIFETIME: f32 = 1.5; // 子弹存活时间（秒）
    pub const SHOOT_COOLDOWN: f32 = 0.15; // 射击冷却（秒）

    // 漩涡参数
    pub const VORTEX_STRENGTH: f32 = 220.0;
    pub const VORTEX_PULL_STRENGTH: f32 = 320.0;
    pub const VORTEX_RADIUS: f32 = 200.0;
    pub const VORTEX_LIFETIME: f32 = 15.0;
    pub const VORTEX_SPAWN_INTERVAL: f32 = 20.0; // 漩涡生成间隔

    // 道具参数
    pub const POWERUP_PICKUP_RADIUS: f32 = 30.0;
    pub const POWERUP_LIFETIME: f32 = 8.0;
    pub const POWERUP_SPAWN_INTERVAL: f32 = 12.0;

    /// 根据小行星大小获取碰撞半径
    pub fn asteroid_radius(size: u32) -> f32 {
        match size {
            3 => 40.0, // 大型
            2 => 25.0, // 中型
            _ => 15.0, // 小型
        }
    }
}

/// 更新游戏物理
fn update_game_physics(
    state: &mut GameState,
    dt: f32,
    screen_width: f32,
    screen_height: f32,
    mode: GameMode,
) {
    use game_constants::*;

    // 输入超时阈值（秒）- 超过此时间未收到输入则清空输入状态
    const INPUT_TIMEOUT: f32 = 0.3;
    let now_secs = state.start_time.elapsed().as_secs_f32();

    // 收集需要生成的子弹（避免借用冲突）
    let mut new_bullets: Vec<ServerBulletState> = Vec::new();
    let mut next_bullet_id = state.bullets.iter().map(|b| b.id).max().unwrap_or(0) + 1;

    // 更新玩家
    for (player_id, player) in state.players.iter_mut() {
        if !player.alive {
            continue;
        }

        // 输入超时保护：长时间未收到输入则清空，防止"卡键"持续射击/转向
        if now_secs - player.last_input_at > INPUT_TIMEOUT {
            player.thrust = false;
            player.turn_left = false;
            player.turn_right = false;
            player.shoot = false;
        }

        // 旋转
        if player.turn_left {
            player.angle -= SHIP_ROTATION_SPEED * dt;
        }
        if player.turn_right {
            player.angle += SHIP_ROTATION_SPEED * dt;
        }

        // 推进
        if player.thrust {
            player.vel_x += player.angle.cos() * SHIP_ACCEL * dt;
            player.vel_y += player.angle.sin() * SHIP_ACCEL * dt;
        }

        // 限速
        let speed = (player.vel_x * player.vel_x + player.vel_y * player.vel_y).sqrt();
        if speed > MAX_SPEED {
            let scale = MAX_SPEED / speed;
            player.vel_x *= scale;
            player.vel_y *= scale;
        }

        // 摩擦
        player.vel_x *= FRICTION;
        player.vel_y *= FRICTION;

        // 移动
        player.x += player.vel_x * dt;
        player.y += player.vel_y * dt;

        // 屏幕环绕
        if player.x < 0.0 {
            player.x += screen_width;
        }
        if player.x > screen_width {
            player.x -= screen_width;
        }
        if player.y < 0.0 {
            player.y += screen_height;
        }
        if player.y > screen_height {
            player.y -= screen_height;
        }

        // 射击（边缘触发 + 冷却）
        if player.shoot && player.shoot_cooldown <= 0.0 {
            // 调试：记录子弹生成
            println!(
                "🔫 Player {:?} shooting: pos=({:.1}, {:.1}), angle={:.2}, turn_l={}, turn_r={}",
                player_id, player.x, player.y, player.angle, player.turn_left, player.turn_right
            );
            let bullet = ServerBulletState {
                id: next_bullet_id,
                owner_id: *player_id,
                x: player.x + player.angle.cos() * SHIP_RADIUS,
                y: player.y + player.angle.sin() * SHIP_RADIUS,
                vx: player.angle.cos() * BULLET_SPEED,
                vy: player.angle.sin() * BULLET_SPEED,
                lifetime: BULLET_LIFETIME,
            };
            new_bullets.push(bullet);
            next_bullet_id += 1;
            player.shoot_cooldown = SHOOT_COOLDOWN;
        }

        // 更新射击冷却
        player.shoot_cooldown = (player.shoot_cooldown - dt).max(0.0);
    }

    // 添加新子弹
    state.bullets.extend(new_bullets);

    // 漩涡对玩家产生影响
    for player in state.players.values_mut() {
        if !player.alive {
            continue;
        }
        for vortex in &state.vortices {
            let dx = vortex.x - player.x;
            let dy = vortex.y - player.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < vortex.radius * vortex.radius && dist_sq > 1.0 {
                let dist = dist_sq.sqrt();
                // 切向力（旋转）
                let tangent_x = -dy / dist;
                let tangent_y = dx / dist;
                let tangent_force = VORTEX_STRENGTH * vortex.strength * dt / dist;
                player.vel_x += tangent_x * tangent_force;
                player.vel_y += tangent_y * tangent_force;
                // 向心力（拉向中心）
                let pull_force = VORTEX_PULL_STRENGTH * dt / dist;
                player.vel_x += dx / dist * pull_force;
                player.vel_y += dy / dist * pull_force;
            }
        }
    }

    // 更新小行星
    for asteroid in &mut state.asteroids {
        asteroid.x += asteroid.vx * dt;
        asteroid.y += asteroid.vy * dt;
        asteroid.angle += 0.5 * dt; // 缓慢旋转

        // 屏幕环绕
        if asteroid.x < 0.0 {
            asteroid.x += screen_width;
        }
        if asteroid.x > screen_width {
            asteroid.x -= screen_width;
        }
        if asteroid.y < 0.0 {
            asteroid.y += screen_height;
        }
        if asteroid.y > screen_height {
            asteroid.y -= screen_height;
        }
    }

    // 更新子弹位置
    for bullet in &mut state.bullets {
        bullet.x += bullet.vx * dt;
        bullet.y += bullet.vy * dt;
        bullet.lifetime -= dt;

        // 屏幕环绕
        if bullet.x < 0.0 {
            bullet.x += screen_width;
        }
        if bullet.x > screen_width {
            bullet.x -= screen_width;
        }
        if bullet.y < 0.0 {
            bullet.y += screen_height;
        }
        if bullet.y > screen_height {
            bullet.y -= screen_height;
        }
    }

    // 移除过期子弹
    state.bullets.retain(|b| b.lifetime > 0.0);

    // ========== 碰撞检测 ==========

    // 子弹-小行星碰撞
    let mut bullets_to_remove: Vec<u32> = Vec::new();
    let mut asteroids_to_remove: Vec<u32> = Vec::new();
    let mut new_asteroids: Vec<ServerAsteroidState> = Vec::new();
    let mut score_updates: Vec<(Uuid, u32)> = Vec::new();
    let mut next_asteroid_id = state.asteroids.iter().map(|a| a.id).max().unwrap_or(0) + 1;

    for bullet in &state.bullets {
        for asteroid in &state.asteroids {
            let dx = bullet.x - asteroid.x;
            let dy = bullet.y - asteroid.y;
            let dist_sq = dx * dx + dy * dy;
            let hit_dist = BULLET_RADIUS + asteroid_radius(asteroid.size);

            if dist_sq < hit_dist * hit_dist {
                bullets_to_remove.push(bullet.id);
                asteroids_to_remove.push(asteroid.id);

                // 加分（大10分，中20分，小50分）
                let points = match asteroid.size {
                    3 => 10,
                    2 => 20,
                    _ => 50,
                };
                score_updates.push((bullet.owner_id, points));

                // 分裂小行星
                if asteroid.size > 1 {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let new_size = asteroid.size - 1;

                    for _ in 0..2 {
                        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                        let speed = rng.gen_range(50.0..100.0);
                        new_asteroids.push(ServerAsteroidState {
                            id: next_asteroid_id,
                            x: asteroid.x,
                            y: asteroid.y,
                            vx: angle.cos() * speed,
                            vy: angle.sin() * speed,
                            size: new_size,
                            angle: rng.gen_range(0.0..std::f32::consts::TAU),
                        });
                        next_asteroid_id += 1;
                    }
                }
                break; // 一颗子弹只能击中一个小行星
            }
        }
    }

    // 应用子弹-小行星碰撞结果
    state.bullets.retain(|b| !bullets_to_remove.contains(&b.id));
    state
        .asteroids
        .retain(|a| !asteroids_to_remove.contains(&a.id));
    state.asteroids.extend(new_asteroids);

    // 更新分数
    for (player_id, points) in score_updates {
        if let Some(player) = state.players.get_mut(&player_id) {
            player.score += points;
        }
    }

    // 飞船碰撞检测（子弹 + 小行星）
    // 需要收集玩家ID列表，因为我们要检查子弹归属
    let player_ids: Vec<Uuid> = state.players.keys().copied().collect();
    let mut ship_bullets_to_remove: Vec<u32> = Vec::new();

    for &player_id in &player_ids {
        let player = match state.players.get(&player_id) {
            Some(p) => p.clone(),
            None => continue,
        };

        if !player.alive {
            continue;
        }

        // 更新无敌时间
        if player.invulnerable_until > 0.0 {
            if let Some(p) = state.players.get_mut(&player_id) {
                p.invulnerable_until = (p.invulnerable_until - dt).max(0.0);
            }
            continue;
        }

        let mut hit = false;

        // Duel 模式下，敌方子弹可以击中玩家
        if mode == GameMode::Duel {
            for bullet in &state.bullets {
                // 跳过自己的子弹和已标记移除的子弹
                if bullet.owner_id == player_id || ship_bullets_to_remove.contains(&bullet.id) {
                    continue;
                }

                let dx = player.x - bullet.x;
                let dy = player.y - bullet.y;
                let dist_sq = dx * dx + dy * dy;
                let hit_dist = SHIP_RADIUS + BULLET_RADIUS;

                if dist_sq < hit_dist * hit_dist {
                    ship_bullets_to_remove.push(bullet.id);
                    hit = true;
                    break;
                }
            }
        }

        // 小行星碰撞
        if !hit {
            for asteroid in &state.asteroids {
                let dx = player.x - asteroid.x;
                let dy = player.y - asteroid.y;
                let dist_sq = dx * dx + dy * dy;
                let hit_dist = SHIP_RADIUS + asteroid_radius(asteroid.size);

                if dist_sq < hit_dist * hit_dist {
                    hit = true;
                    break;
                }
            }
        }

        // 处理碰撞结果
        if hit && let Some(p) = state.players.get_mut(&player_id) {
            if p.lives > 0 {
                p.lives -= 1;
            }
            if p.lives == 0 {
                p.alive = false;
            } else {
                // 重生保护（2秒无敌）
                p.invulnerable_until = 2.0;
                // 重置到屏幕中心附近
                p.x = screen_width / 2.0;
                p.y = screen_height / 2.0;
                p.vel_x = 0.0;
                p.vel_y = 0.0;
            }
        }
    }

    // 移除击中玩家的子弹
    state
        .bullets
        .retain(|b| !ship_bullets_to_remove.contains(&b.id));
}

/// 生成、更新漩涡与道具（含拾取）
fn update_world_events(state: &mut GameState, screen_width: f32, screen_height: f32) {
    use game_constants::*;
    use rand::Rng;

    let now_secs = state.start_time.elapsed().as_secs_f32();
    let now_secs_f64 = state.start_time.elapsed().as_secs_f64();

    // 生成漩涡
    if state.next_vortex_spawn <= now_secs {
        let mut rng = rand::thread_rng();
        let id = state.vortices.iter().map(|v| v.id).max().unwrap_or(0) + 1;
        // 避免在边缘生成
        let margin = VORTEX_RADIUS;
        let x = rng.gen_range(margin..screen_width - margin);
        let y = rng.gen_range(margin..screen_height - margin);
        // 50% 概率为顺时针或逆时针
        let strength = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

        state.vortices.push(ServerVortexState {
            id,
            x,
            y,
            strength,
            radius: VORTEX_RADIUS,
            created_at: now_secs,
            lifetime: VORTEX_LIFETIME,
        });
        state.next_vortex_spawn = now_secs + VORTEX_SPAWN_INTERVAL;
    }

    // 生成道具
    if state.next_powerup_spawn <= now_secs {
        let mut rng = rand::thread_rng();
        let id = state.powerups.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let powerup_type = match rng.gen_range(0..3) {
            0 => PowerupType::Shield,
            1 => PowerupType::DualShot,
            _ => PowerupType::TripleShot,
        };
        let x = rng.gen_range(50.0..screen_width - 50.0);
        let y = rng.gen_range(50.0..screen_height - 50.0);

        state.powerups.push(ServerPowerupState {
            id,
            x,
            y,
            expires_at: now_secs_f64 + POWERUP_LIFETIME as f64,
            collected: false,
            powerup_type,
        });
        state.next_powerup_spawn = now_secs + POWERUP_SPAWN_INTERVAL;
    }

    // 清理过期漩涡
    state
        .vortices
        .retain(|v| now_secs - v.created_at < v.lifetime);

    // 道具拾取检测
    for powerup in &mut state.powerups {
        if powerup.collected {
            continue;
        }
        // 检查是否过期
        if now_secs_f64 >= powerup.expires_at {
            powerup.collected = true;
            continue;
        }

        // 检查玩家拾取
        for player in state.players.values() {
            if !player.alive {
                continue;
            }
            let dx = player.x - powerup.x;
            let dy = player.y - powerup.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < POWERUP_PICKUP_RADIUS * POWERUP_PICKUP_RADIUS {
                powerup.collected = true;
                // 注意：道具效果需要客户端处理，服务器只标记拾取
                break;
            }
        }
    }

    // 清理已拾取的道具
    state.powerups.retain(|p| !p.collected);
}

/// 检查游戏结束条件
fn check_game_over(state: &GameState, mode: GameMode) -> bool {
    match mode {
        GameMode::Duel => {
            // Duel 模式：只剩一个存活玩家时结束
            let alive_count = state.players.values().filter(|p| p.alive).count();
            alive_count <= 1
        }
        GameMode::Survival => {
            // Survival 模式：所有玩家死亡时结束
            !state.players.values().any(|p| p.alive)
        }
    }
}

/// 广播游戏状态
async fn broadcast_game_state(room_id: Uuid, peer_map: &PeerMap, room_map: &RoomMap) {
    let (player_ids, game_state_msg) = {
        let rooms = room_map.read().await;
        let Some(room) = rooms.get(&room_id) else {
            return;
        };
        let Some(ref state) = room.game_state else {
            return;
        };

        let players: Vec<PlayerState> = state
            .players
            .iter()
            .map(|(id, p)| PlayerState {
                id: id.to_string(),
                x: p.x,
                y: p.y,
                angle: p.angle,
                vel_x: p.vel_x,
                vel_y: p.vel_y,
                lives: p.lives,
                score: p.score,
                alive: p.alive,
            })
            .collect();

        let asteroids: Vec<AsteroidState> = state
            .asteroids
            .iter()
            .map(|a| AsteroidState {
                id: a.id,
                x: a.x,
                y: a.y,
                vx: a.vx,
                vy: a.vy,
                size: a.size,
                angle: a.angle,
            })
            .collect();

        let bullets: Vec<BulletState> = state
            .bullets
            .iter()
            .map(|b| BulletState {
                id: b.id,
                owner_id: b.owner_id.to_string(),
                x: b.x,
                y: b.y,
                vx: b.vx,
                vy: b.vy,
            })
            .collect();

        let vortices: Vec<VortexState> = state
            .vortices
            .iter()
            .map(|v| VortexState {
                id: v.id,
                x: v.x,
                y: v.y,
                strength: v.strength,
                radius: v.radius,
                created_at: v.created_at,
                lifetime: v.lifetime,
            })
            .collect();

        let powerups: Vec<PowerupState> = state
            .powerups
            .iter()
            .map(|p| PowerupState {
                id: p.id,
                x: p.x,
                y: p.y,
                expires_at: p.expires_at,
                collected: p.collected,
                powerup_type: p.powerup_type,
            })
            .collect();

        // 构建 last_input_seqs: player_id -> last_input_seq
        let last_input_seqs: HashMap<String, u32> = state
            .players
            .iter()
            .map(|(id, p)| (id.to_string(), p.last_input_seq))
            .collect();

        let timestamp = state.start_time.elapsed().as_millis() as i64;

        let msg = ServerMessage::GameState {
            players,
            asteroids,
            bullets,
            vortices,
            powerups,
            last_input_seqs,
            timestamp,
        };

        (room.players.clone(), serde_json::to_string(&msg).ok())
    };

    if let Some(msg_text) = game_state_msg {
        let peers = peer_map.read().await;
        for &pid in &player_ids {
            if let Some(peer) = peers.get(&pid) {
                let _ = peer.tx.send(Message::Text(msg_text.clone()));
            }
        }
    }
}

/// 发送游戏结束消息
async fn send_game_over(room_id: Uuid, peer_map: &PeerMap, room_map: &RoomMap) {
    let (player_ids, game_over_msg) = {
        let mut rooms = room_map.write().await;
        let Some(room) = rooms.get_mut(&room_id) else {
            return;
        };

        room.started = false;

        let (winner, scores) = if let Some(ref state) = room.game_state {
            let winner = state
                .players
                .iter()
                .filter(|(_, p)| p.alive)
                .max_by_key(|(_, p)| p.score)
                .map(|(id, _)| id.to_string());

            let scores: Vec<(String, u32)> = state
                .players
                .iter()
                .map(|(id, p)| (id.to_string(), p.score))
                .collect();

            (winner, scores)
        } else {
            (None, Vec::new())
        };

        let msg = ServerMessage::GameOver { winner, scores };

        (room.players.clone(), serde_json::to_string(&msg).ok())
    };

    if let Some(msg_text) = game_over_msg {
        let peers = peer_map.read().await;
        for &pid in &player_ids {
            if let Some(peer) = peers.get(&pid) {
                let _ = peer.tx.send(Message::Text(msg_text.clone()));
            }
        }
    }

    println!("🏁 房间 {} 游戏结束", room_id);
}

async fn handle_leave_room(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    handle_leave_queue(player_id, peer_map, room_map).await
}

async fn cleanup_player(player_id: Uuid, peer_map: &PeerMap, room_map: &RoomMap) {
    // 从房间移除
    let _ = handle_leave_queue(player_id, peer_map, room_map).await;

    // 从玩家列表移除
    peer_map.write().await.remove(&player_id);
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:9001";
    let listener = TcpListener::bind(addr).await.expect("无法绑定端口");
    println!("🚀 Asteroids 服务器启动！");
    println!("📡 监听地址: {}", addr);
    println!("🎮 支持模式: Survival, Duel");
    println!("---");

    let peer_map: PeerMap = Arc::new(RwLock::new(HashMap::new()));
    let room_map: RoomMap = Arc::new(RwLock::new(HashMap::new()));

    // 定期清理空房间
    let room_map_clone = room_map.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut rooms = room_map_clone.write().await;
            let before = rooms.len();
            rooms.retain(|_, room| !room.players.is_empty());
            let after = rooms.len();
            if before != after {
                println!("🧹 清理了 {} 个空房间", before - after);
            }
        }
    });

    // 接受连接
    while let Ok((stream, _)) = listener.accept().await {
        let peer_map = peer_map.clone();
        let room_map = room_map.clone();
        tokio::spawn(handle_client(stream, peer_map, room_map));
    }
}
