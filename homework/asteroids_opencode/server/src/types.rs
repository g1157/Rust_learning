//! 服务器核心类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub type Tx = mpsc::UnboundedSender<Message>;

/// 游戏模式（与客户端 NetworkGameMode 一致）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Duel,
}

/// 道具类型（与客户端 PowerUpType 一致）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerupType {
    Shield,
    DualShot,
    TripleShot,
}

/// 玩家信息
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: Uuid,
    pub tx: Tx,
    pub room_id: Option<Uuid>,
    pub nickname: String,
}

/// 服务器端游戏状态
#[derive(Debug, Clone)]
pub struct GameState {
    pub players: HashMap<Uuid, ServerPlayerState>,
    pub asteroids: Vec<ServerAsteroidState>,
    pub bullets: Vec<ServerBulletState>,
    pub vortices: Vec<ServerVortexState>,
    pub powerups: Vec<ServerPowerupState>,
    pub next_vortex_spawn: f32,
    pub next_powerup_spawn: f32,
    pub start_time: Instant,
    pub last_update: Instant,
}

/// 服务器端玩家状态
#[derive(Debug, Clone)]
pub struct ServerPlayerState {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub angle: f32,
    pub lives: u32,
    pub score: u32,
    pub alive: bool,
    pub thrust: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub shoot: bool,
    pub shoot_cooldown: f32,
    pub invulnerable_until: f32,
    pub last_input_at: f32,
    pub last_input_seq: u32,
}

impl Default for ServerPlayerState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            angle: 0.0,
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
        }
    }
}

/// 服务器端小行星状态
#[derive(Debug, Clone)]
pub struct ServerAsteroidState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: u32,
    pub angle: f32,
}

/// 服务器端子弹状态
#[derive(Debug, Clone)]
pub struct ServerBulletState {
    pub id: u32,
    pub owner_id: Uuid,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
}

/// 服务器端漩涡状态
#[derive(Debug, Clone)]
pub struct ServerVortexState {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub radius: f32,
    pub created_at: f32,
    pub lifetime: f32,
}

/// 服务器端道具状态
#[derive(Debug, Clone)]
pub struct ServerPowerupState {
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
    fn test_server_player_state_default() {
        let state = ServerPlayerState::default();
        assert_eq!(state.lives, 3);
        assert!(state.alive);
        assert!(!state.thrust);
        assert!(!state.shoot);
    }

    #[test]
    fn test_game_mode_serialization() {
        let mode = GameMode::Survival;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"Survival\"");

        let parsed: GameMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, GameMode::Survival);
    }

    #[test]
    fn test_powerup_type_serialization() {
        let powerup = PowerupType::Shield;
        let json = serde_json::to_string(&powerup).unwrap();
        let parsed: PowerupType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PowerupType::Shield);
    }
}
