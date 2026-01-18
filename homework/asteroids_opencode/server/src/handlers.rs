//! 消息处理模块

use std::collections::HashMap;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::game_logic::{check_game_over, game_constants, update_game_physics, update_world_events};
use crate::messages::{
    AsteroidState, BulletState, ClientMessage, PlayerState, PowerupState, ServerMessage,
    VortexState,
};
use crate::room::{PeerMap, Room, RoomMap};
use crate::types::GameMode;

pub async fn handle_message(
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
            if let Some(peer) = peer_map.read().await.get(&player_id) {
                let msg = ServerMessage::Pong;
                let _ = peer.tx.send(Message::Text(serde_json::to_string(&msg)?));
            }
        }
    }

    Ok(())
}

pub async fn handle_join_queue(
    player_id: Uuid,
    mode: GameMode,
    nickname: String,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(peer) = peer_map.write().await.get_mut(&player_id) {
        peer.nickname = nickname;
    }

    let mut rooms = room_map.write().await;
    let available_room = rooms
        .values_mut()
        .find(|room| room.mode == mode && !room.is_full() && !room.started);

    let room_id = if let Some(room) = available_room {
        room.add_player(player_id);
        room.id
    } else {
        let mut new_room = Room::new(mode);
        new_room.add_player(player_id);
        let room_id = new_room.id;
        rooms.insert(room_id, new_room);
        room_id
    };

    if let Some(peer) = peer_map.write().await.get_mut(&player_id) {
        peer.room_id = Some(room_id);
    }

    if let Some(room) = rooms.get(&room_id).filter(|r| r.is_full()) {
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

pub async fn handle_leave_queue(
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

pub async fn handle_game_input(
    player_id: Uuid,
    keys: Vec<String>,
    seq: u32,
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
        if let Some(room) = rooms.get_mut(&rid)
            && let Some(ref mut state) = room.game_state
            && let Some(player) = state.players.get_mut(&player_id)
        {
            player.thrust = keys.contains(&"thrust".to_string());
            player.turn_left = keys.contains(&"left".to_string());
            player.turn_right = keys.contains(&"right".to_string());
            player.shoot = keys.contains(&"shoot".to_string());
            player.last_input_at = state.start_time.elapsed().as_secs_f32();
            player.last_input_seq = seq;
        }
    }

    Ok(())
}

pub async fn handle_ready(
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

            let peer_map_clone = peer_map.clone();
            let room_map_clone = room_map.clone();
            tokio::spawn(async move {
                game_loop(rid, peer_map_clone, room_map_clone).await;
            });
        }
    }

    Ok(())
}

pub async fn handle_leave_room(
    player_id: Uuid,
    peer_map: &PeerMap,
    room_map: &RoomMap,
) -> Result<(), Box<dyn std::error::Error>> {
    handle_leave_queue(player_id, peer_map, room_map).await
}

pub async fn cleanup_player(player_id: Uuid, peer_map: &PeerMap, room_map: &RoomMap) {
    let _ = handle_leave_queue(player_id, peer_map, room_map).await;
    peer_map.write().await.remove(&player_id);
}

/// 游戏主循环
async fn game_loop(room_id: Uuid, peer_map: PeerMap, room_map: RoomMap) {
    const TICK_RATE: u64 = 30;
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_RATE));

    loop {
        interval.tick().await;

        let room_exists = {
            let rooms = room_map.read().await;
            rooms.contains_key(&room_id)
        };

        if !room_exists {
            println!("🛑 房间 {} 游戏循环结束（房间已删除）", room_id);
            break;
        }

        let game_over = {
            let mut rooms = room_map.write().await;
            if let Some(room) = rooms.get_mut(&room_id) {
                if !room.started {
                    true
                } else if let Some(ref mut state) = room.game_state {
                    let dt = state.last_update.elapsed().as_secs_f32();
                    state.last_update = std::time::Instant::now();

                    update_game_physics(
                        state,
                        dt,
                        game_constants::SCREEN_WIDTH,
                        game_constants::SCREEN_HEIGHT,
                        room.mode,
                    );

                    update_world_events(
                        state,
                        game_constants::SCREEN_WIDTH,
                        game_constants::SCREEN_HEIGHT,
                    );

                    check_game_over(state, room.mode)
                } else {
                    true
                }
            } else {
                true
            }
        };

        if game_over {
            send_game_over(room_id, &peer_map, &room_map).await;
            break;
        }

        broadcast_game_state(room_id, &peer_map, &room_map).await;
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
