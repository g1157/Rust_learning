//! Asteroids 在线多人游戏服务器
//!
//! 基于 WebSocket 的实时游戏服务器
//! 支持房间系统、玩家匹配和游戏状态同步

mod game_logic;
mod handlers;
mod messages;
mod room;
mod types;

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use handlers::{cleanup_player, handle_message};
use messages::ServerMessage;
use room::{PeerMap, RoomMap};
use types::Peer;

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

    let msg = ServerMessage::Connected {
        player_id: player_id.to_string(),
    };
    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap()));

    println!("✅ 玩家 {} 已连接", player_id);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

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

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    cleanup_player(player_id, &peer_map, &room_map).await;
    println!("👋 玩家 {} 已断开", player_id);
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
