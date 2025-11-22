//! Asteroids 在线多人游戏服务器
//!
//! 基于 WebSocket 的实时游戏服务器
//! 支持房间系统、玩家匹配和游戏状态同步

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

type Tx = mpsc::UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<Uuid, Peer>>>;
type RoomMap = Arc<RwLock<HashMap<Uuid, Room>>>;

/// 游戏模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum GameMode {
    Survival,
    Duel,
}

/// 玩家信息
#[derive(Debug, Clone)]
struct Peer {
    id: Uuid,
    tx: Tx,
    room_id: Option<Uuid>,
    nickname: String,
}

/// 游戏房间
#[derive(Debug, Clone)]
struct Room {
    id: Uuid,
    mode: GameMode,
    players: Vec<Uuid>,
    max_players: usize,
    started: bool,
}

/// 客户端 → 服务器消息
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    /// 加入匹配队列
    JoinQueue {
        mode: GameMode,
        nickname: String,
    },
    /// 离开队列
    LeaveQueue,
    /// 游戏输入
    GameInput {
        keys: Vec<String>,
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
enum ServerMessage {
    /// 连接成功
    Connected {
        player_id: String,
    },
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
    PlayerDisconnected {
        player_id: String,
    },
    /// 游戏结束
    GameOver {
        winner: Option<String>,
        scores: Vec<(String, u32)>,
    },
    /// 错误消息
    Error {
        message: String,
    },
    /// 心跳响应
    Pong,
}

#[derive(Debug, Serialize, Clone)]
struct PlayerState {
    id: String,
    x: f32,
    y: f32,
    angle: f32,
    lives: u32,
    score: u32,
}

#[derive(Debug, Serialize, Clone)]
struct AsteroidState {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: u32,
}

impl Room {
    fn new(mode: GameMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            players: Vec::new(),
            max_players: if mode == GameMode::Duel { 2 } else { 2 },
            started: false,
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
}

async fn handle_client(
    stream: TcpStream,
    peer_map: PeerMap,
    room_map: RoomMap,
) {
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
                    if let Err(e) = handle_message(
                        player_id,
                        text,
                        &peer_map_clone,
                        &room_map_clone,
                    ).await {
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
        ClientMessage::GameInput { keys } => {
            handle_game_input(player_id, keys, peer_map, room_map).await?;
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
    let available_room = rooms.values_mut().find(|room| {
        room.mode == mode && !room.is_full() && !room.started
    });

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
    if let Some(room) = rooms.get(&room_id) {
        if room.is_full() {
            // 通知所有玩家匹配成功
            let player_nicknames: Vec<String> = {
                let peers = peer_map.read().await;
                room.players.iter()
                    .filter_map(|id| peers.get(id))
                    .map(|p| p.nickname.clone())
                    .collect()
            };

            let msg = ServerMessage::MatchFound {
                room_id: room_id.to_string(),
                players: player_nicknames,
                mode: room.mode.clone(),
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
    }

    Ok(())
}

async fn handle_leave_queue(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let room_id = peer_map.read().await.get(&player_id)
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
    _player_id: Uuid,
    _keys: Vec<String>,
    _peer_map: &PeerMap,
    _room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: 实现游戏输入处理
    Ok(())
}

async fn handle_ready(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let room_id = peer_map.read().await.get(&player_id)
        .and_then(|p| p.room_id);

    if let Some(rid) = room_id {
        let mut rooms = room_map.write().await;
        if let Some(room) = rooms.get_mut(&rid) {
            if room.is_full() && !room.started {
                room.started = true;

                // 通知所有玩家游戏开始
                let msg = ServerMessage::GameStart;
                let msg_text = serde_json::to_string(&msg)?;
                let peers = peer_map.read().await;
                for &pid in &room.players {
                    if let Some(peer) = peers.get(&pid) {
                        let _ = peer.tx.send(Message::Text(msg_text.clone()));
                    }
                }

                println!("🚀 房间 {} 游戏开始！", rid);
            }
        }
    }

    Ok(())
}

async fn handle_leave_room(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    handle_leave_queue(player_id, peer_map, room_map).await
}

async fn cleanup_player(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) {
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
