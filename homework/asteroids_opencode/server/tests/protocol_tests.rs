//! 网络协议集成测试
//!
//! 测试客户端-服务器消息协议的完整性和兼容性

use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;

// 重新定义消息类型用于集成测试（模拟客户端）

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum GameMode {
    Survival,
    Duel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PowerupType {
    Shield,
    RapidFire,
    Piercing,
    GhostMode,
    TimeWarp,
    Overdrive,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinQueue { mode: GameMode, nickname: String },
    LeaveQueue,
    GameInput { keys: Vec<String>, seq: u32 },
    Ready,
    LeaveRoom,
    Ping,
}

#[derive(Debug, Deserialize)]
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
        bullets: Vec<BulletState>,
        vortices: Vec<VortexState>,
        powerups: Vec<PowerupState>,
        last_input_seqs: HashMap<String, u32>,
        timestamp: i64,
    },
    PlayerDisconnected { player_id: String },
    GameOver {
        winner: Option<String>,
        scores: Vec<(String, u32)>,
    },
    Error { message: String },
    Pong,
    RoomUpdate { players: Vec<String> },
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct AsteroidState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: u32,
    pub angle: f32,
}

#[derive(Debug, Deserialize)]
pub struct BulletState {
    pub id: u32,
    pub owner_id: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Deserialize)]
pub struct VortexState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub radius: f32,
    pub created_at: f32,
    pub lifetime: f32,
}

#[derive(Debug, Deserialize)]
pub struct PowerupState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub expires_at: f64,
    pub collected: bool,
    pub powerup_type: PowerupType,
}

// ============================================================================
// 消息序列化/反序列化测试
// ============================================================================

#[test]
fn test_join_queue_message_format() {
    let msg = ClientMessage::JoinQueue {
        mode: GameMode::Survival,
        nickname: "TestPlayer".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "JoinQueue");
    assert_eq!(parsed["mode"], "Survival");
    assert_eq!(parsed["nickname"], "TestPlayer");
}

#[test]
fn test_join_queue_duel_mode() {
    let msg = ClientMessage::JoinQueue {
        mode: GameMode::Duel,
        nickname: "DuelPlayer".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "JoinQueue");
    assert_eq!(parsed["mode"], "Duel");
}

#[test]
fn test_leave_queue_message_format() {
    let msg = ClientMessage::LeaveQueue;
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "LeaveQueue");
}

#[test]
fn test_game_input_message_format() {
    let msg = ClientMessage::GameInput {
        keys: vec!["thrust".to_string(), "shoot".to_string(), "left".to_string()],
        seq: 123,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "GameInput");
    assert_eq!(parsed["seq"], 123);
    assert!(parsed["keys"].as_array().unwrap().contains(&Value::String("thrust".to_string())));
    assert!(parsed["keys"].as_array().unwrap().contains(&Value::String("shoot".to_string())));
}

#[test]
fn test_game_input_empty_keys() {
    let msg = ClientMessage::GameInput {
        keys: vec![],
        seq: 0,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "GameInput");
    assert_eq!(parsed["keys"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["seq"], 0);
}

#[test]
fn test_ready_message_format() {
    let msg = ClientMessage::Ready;
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "Ready");
}

#[test]
fn test_leave_room_message_format() {
    let msg = ClientMessage::LeaveRoom;
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "LeaveRoom");
}

#[test]
fn test_ping_message_format() {
    let msg = ClientMessage::Ping;
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "Ping");
}

// ============================================================================
// 服务器消息解析测试
// ============================================================================

#[test]
fn test_parse_connected_message() {
    let json = r#"{"type":"Connected","player_id":"abc-123-def"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::Connected { player_id } => {
            assert_eq!(player_id, "abc-123-def");
        }
        _ => panic!("Expected Connected message"),
    }
}

#[test]
fn test_parse_match_found_message() {
    let json = r#"{
        "type": "MatchFound",
        "room_id": "room-456",
        "players": ["Player1", "Player2"],
        "mode": "Duel"
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::MatchFound { room_id, players, mode } => {
            assert_eq!(room_id, "room-456");
            assert_eq!(players.len(), 2);
            assert_eq!(mode, GameMode::Duel);
        }
        _ => panic!("Expected MatchFound message"),
    }
}

#[test]
fn test_parse_game_start_message() {
    let json = r#"{"type":"GameStart"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(matches!(msg, ServerMessage::GameStart));
}

#[test]
fn test_parse_game_state_message() {
    let json = r#"{
        "type": "GameState",
        "players": [{
            "id": "p1",
            "x": 100.0,
            "y": 200.0,
            "angle": 1.57,
            "vel_x": 10.0,
            "vel_y": -5.0,
            "lives": 3,
            "score": 100,
            "alive": true
        }],
        "asteroids": [{
            "id": 1,
            "x": 300.0,
            "y": 400.0,
            "vx": 50.0,
            "vy": -30.0,
            "size": 2,
            "angle": 0.5
        }],
        "bullets": [],
        "vortices": [],
        "powerups": [],
        "last_input_seqs": {"p1": 42},
        "timestamp": 1234567890
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameState {
            players,
            asteroids,
            bullets,
            last_input_seqs,
            timestamp,
            ..
        } => {
            assert_eq!(players.len(), 1);
            assert_eq!(players[0].id, "p1");
            assert_eq!(players[0].lives, 3);
            assert!(players[0].alive);

            assert_eq!(asteroids.len(), 1);
            assert_eq!(asteroids[0].id, 1);
            assert_eq!(asteroids[0].size, 2);

            assert!(bullets.is_empty());

            assert_eq!(last_input_seqs.get("p1"), Some(&42));
            assert_eq!(timestamp, 1234567890);
        }
        _ => panic!("Expected GameState message"),
    }
}

#[test]
fn test_parse_player_disconnected_message() {
    let json = r#"{"type":"PlayerDisconnected","player_id":"player-xyz"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::PlayerDisconnected { player_id } => {
            assert_eq!(player_id, "player-xyz");
        }
        _ => panic!("Expected PlayerDisconnected message"),
    }
}

#[test]
fn test_parse_game_over_with_winner() {
    let json = r#"{
        "type": "GameOver",
        "winner": "player-1",
        "scores": [["player-1", 150], ["player-2", 100]]
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameOver { winner, scores } => {
            assert_eq!(winner, Some("player-1".to_string()));
            assert_eq!(scores.len(), 2);
            assert_eq!(scores[0].0, "player-1");
            assert_eq!(scores[0].1, 150);
        }
        _ => panic!("Expected GameOver message"),
    }
}

#[test]
fn test_parse_game_over_no_winner() {
    let json = r#"{"type":"GameOver","winner":null,"scores":[]}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameOver { winner, scores } => {
            assert!(winner.is_none());
            assert!(scores.is_empty());
        }
        _ => panic!("Expected GameOver message"),
    }
}

#[test]
fn test_parse_error_message() {
    let json = r#"{"type":"Error","message":"Room is full"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::Error { message } => {
            assert_eq!(message, "Room is full");
        }
        _ => panic!("Expected Error message"),
    }
}

#[test]
fn test_parse_pong_message() {
    let json = r#"{"type":"Pong"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(matches!(msg, ServerMessage::Pong));
}

#[test]
fn test_parse_room_update_message() {
    let json = r#"{"type":"RoomUpdate","players":["Alice","Bob","Charlie"]}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::RoomUpdate { players } => {
            assert_eq!(players.len(), 3);
            assert!(players.contains(&"Alice".to_string()));
        }
        _ => panic!("Expected RoomUpdate message"),
    }
}

// ============================================================================
// 边界情况测试
// ============================================================================

#[test]
fn test_game_input_high_sequence_number() {
    let msg = ClientMessage::GameInput {
        keys: vec!["shoot".to_string()],
        seq: u32::MAX,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["seq"], u32::MAX);
}

#[test]
fn test_long_nickname() {
    let long_name = "A".repeat(100);
    let msg = ClientMessage::JoinQueue {
        mode: GameMode::Survival,
        nickname: long_name.clone(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["nickname"].as_str().unwrap().len(), 100);
}

#[test]
fn test_unicode_nickname() {
    let msg = ClientMessage::JoinQueue {
        mode: GameMode::Duel,
        nickname: "玩家一号".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("玩家一号"));
}

#[test]
fn test_special_characters_in_nickname() {
    let msg = ClientMessage::JoinQueue {
        mode: GameMode::Survival,
        nickname: r#"Player"Test'<>&"#.to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    // JSON 应正确转义特殊字符
    assert!(json.contains("Player"));
}

#[test]
fn test_powerup_type_parsing() {
    let json = r#"{
        "type": "GameState",
        "players": [],
        "asteroids": [],
        "bullets": [],
        "vortices": [],
        "powerups": [{
            "id": 1,
            "x": 100.0,
            "y": 200.0,
            "expires_at": 10.0,
            "collected": false,
            "powerup_type": "Shield"
        }],
        "last_input_seqs": {},
        "timestamp": 0
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameState { powerups, .. } => {
            assert_eq!(powerups.len(), 1);
            assert_eq!(powerups[0].powerup_type, PowerupType::Shield);
            assert!(!powerups[0].collected);
        }
        _ => panic!("Expected GameState message"),
    }
}

#[test]
fn test_vortex_state_parsing() {
    let json = r#"{
        "type": "GameState",
        "players": [],
        "asteroids": [],
        "bullets": [],
        "vortices": [{
            "id": 5,
            "x": 500.0,
            "y": 300.0,
            "strength": 100.0,
            "radius": 80.0,
            "created_at": 5.0,
            "lifetime": 10.0
        }],
        "powerups": [],
        "last_input_seqs": {},
        "timestamp": 0
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameState { vortices, .. } => {
            assert_eq!(vortices.len(), 1);
            assert_eq!(vortices[0].id, 5);
            assert!((vortices[0].strength - 100.0).abs() < f32::EPSILON);
        }
        _ => panic!("Expected GameState message"),
    }
}

#[test]
fn test_bullet_state_parsing() {
    let json = r#"{
        "type": "GameState",
        "players": [],
        "asteroids": [],
        "bullets": [{
            "id": 10,
            "owner_id": "player-1",
            "x": 200.0,
            "y": 150.0,
            "vx": 500.0,
            "vy": 0.0
        }],
        "vortices": [],
        "powerups": [],
        "last_input_seqs": {},
        "timestamp": 0
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameState { bullets, .. } => {
            assert_eq!(bullets.len(), 1);
            assert_eq!(bullets[0].id, 10);
            assert_eq!(bullets[0].owner_id, "player-1");
        }
        _ => panic!("Expected GameState message"),
    }
}

// ============================================================================
// 协议兼容性测试
// ============================================================================

#[test]
fn test_all_input_keys_recognized() {
    let all_keys = vec![
        "thrust".to_string(),
        "left".to_string(),
        "right".to_string(),
        "shoot".to_string(),
        "hyperspace".to_string(),
        "dash".to_string(),
    ];
    let msg = ClientMessage::GameInput {
        keys: all_keys.clone(),
        seq: 1,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    let keys_arr = parsed["keys"].as_array().unwrap();
    assert_eq!(keys_arr.len(), 6);
}

#[test]
fn test_game_mode_serialization_consistency() {
    // 确保 GameMode 序列化格式一致
    let survival = serde_json::to_string(&GameMode::Survival).unwrap();
    let duel = serde_json::to_string(&GameMode::Duel).unwrap();

    assert_eq!(survival, r#""Survival""#);
    assert_eq!(duel, r#""Duel""#);
}

#[test]
fn test_multiple_players_in_game_state() {
    let json = r#"{
        "type": "GameState",
        "players": [
            {"id": "p1", "x": 100.0, "y": 100.0, "angle": 0.0, "vel_x": 0.0, "vel_y": 0.0, "lives": 3, "score": 50, "alive": true},
            {"id": "p2", "x": 200.0, "y": 200.0, "angle": 3.14, "vel_x": 10.0, "vel_y": -10.0, "lives": 2, "score": 75, "alive": true}
        ],
        "asteroids": [],
        "bullets": [],
        "vortices": [],
        "powerups": [],
        "last_input_seqs": {"p1": 10, "p2": 15},
        "timestamp": 123456
    }"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::GameState {
            players,
            last_input_seqs,
            ..
        } => {
            assert_eq!(players.len(), 2);
            assert_eq!(last_input_seqs.len(), 2);
            assert_eq!(last_input_seqs.get("p1"), Some(&10));
            assert_eq!(last_input_seqs.get("p2"), Some(&15));
        }
        _ => panic!("Expected GameState message"),
    }
}
