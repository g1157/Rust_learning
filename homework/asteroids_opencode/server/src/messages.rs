//! 网络消息类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{GameMode, PowerupType};

/// 客户端 → 服务器消息
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 加入匹配队列
    JoinQueue { mode: GameMode, nickname: String },
    /// 离开队列
    LeaveQueue,
    /// 游戏输入
    GameInput {
        keys: Vec<String>,
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
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
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
        bullets: Vec<BulletState>,
        vortices: Vec<VortexState>,
        powerups: Vec<PowerupState>,
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

/// 网络传输用玩家状态
#[derive(Debug, Serialize, Clone)]
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

/// 网络传输用小行星状态
#[derive(Debug, Serialize, Clone)]
pub struct AsteroidState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: u32,
    pub angle: f32,
}

/// 网络传输用子弹状态
#[derive(Debug, Serialize, Clone)]
pub struct BulletState {
    pub id: u32,
    pub owner_id: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// 网络传输用漩涡状态
#[derive(Debug, Serialize, Clone)]
pub struct VortexState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub radius: f32,
    pub created_at: f32,
    pub lifetime: f32,
}

/// 网络传输用道具状态
#[derive(Debug, Serialize, Clone)]
pub struct PowerupState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub expires_at: f64,
    pub collected: bool,
    pub powerup_type: PowerupType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_join_queue() {
        let json = r#"{"type":"JoinQueue","mode":"Survival","nickname":"Player1"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::JoinQueue { mode, nickname } => {
                assert_eq!(mode, GameMode::Survival);
                assert_eq!(nickname, "Player1");
            }
            _ => panic!("Expected JoinQueue"),
        }
    }

    #[test]
    fn test_client_message_game_input() {
        let json = r#"{"type":"GameInput","keys":["thrust","shoot"],"seq":42}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::GameInput { keys, seq } => {
                assert!(keys.contains(&"thrust".to_string()));
                assert!(keys.contains(&"shoot".to_string()));
                assert_eq!(seq, 42);
            }
            _ => panic!("Expected GameInput"),
        }
    }

    #[test]
    fn test_client_message_ping() {
        let json = r#"{"type":"Ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn test_server_message_connected() {
        let msg = ServerMessage::Connected {
            player_id: "test-id".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"Connected\""));
        assert!(json.contains("\"player_id\":\"test-id\""));
    }

    #[test]
    fn test_server_message_match_found() {
        let msg = ServerMessage::MatchFound {
            room_id: "room-123".to_string(),
            players: vec!["Player1".to_string(), "Player2".to_string()],
            mode: GameMode::Duel,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"MatchFound\""));
        assert!(json.contains("\"mode\":\"Duel\""));
    }

    #[test]
    fn test_server_message_game_over() {
        let msg = ServerMessage::GameOver {
            winner: Some("player-1".to_string()),
            scores: vec![
                ("player-1".to_string(), 100),
                ("player-2".to_string(), 50),
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"GameOver\""));
        assert!(json.contains("\"winner\":\"player-1\""));
    }

    #[test]
    fn test_player_state_serialization() {
        let state = PlayerState {
            id: "p1".to_string(),
            x: 100.0,
            y: 200.0,
            angle: 1.57,
            vel_x: 10.0,
            vel_y: -5.0,
            lives: 3,
            score: 150,
            alive: true,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["id"], "p1");
        assert_eq!(parsed["lives"], 3);
        assert_eq!(parsed["alive"], true);
    }
}
