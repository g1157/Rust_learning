//! 房间管理模块

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::game_logic::game_constants;
use crate::types::{
    GameMode, GameState, Peer, ServerAsteroidState, ServerPlayerState, Tx,
};

pub type PeerMap = Arc<RwLock<HashMap<Uuid, Peer>>>;
pub type RoomMap = Arc<RwLock<HashMap<Uuid, Room>>>;

/// 游戏房间
#[derive(Debug)]
pub struct Room {
    pub id: Uuid,
    pub mode: GameMode,
    pub players: Vec<Uuid>,
    pub max_players: usize,
    pub started: bool,
    pub game_state: Option<GameState>,
}

impl Room {
    pub fn new(mode: GameMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            players: Vec::new(),
            max_players: 2,
            started: false,
            game_state: None,
        }
    }

    /// 使用指定 ID 创建房间（用于测试）
    #[cfg(test)]
    pub fn with_id(id: Uuid, mode: GameMode) -> Self {
        Self {
            id,
            mode,
            players: Vec::new(),
            max_players: 2,
            started: false,
            game_state: None,
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    pub fn add_player(&mut self, player_id: Uuid) -> bool {
        if !self.is_full() {
            self.players.push(player_id);
            true
        } else {
            false
        }
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> bool {
        let len_before = self.players.len();
        self.players.retain(|&id| id != player_id);
        self.players.len() < len_before
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// 初始化游戏状态
    pub fn init_game_state(&mut self, screen_width: f32, screen_height: f32) {
        let mut players = HashMap::new();
        let spawn_positions = [
            (screen_width * 0.25, screen_height * 0.5, 0.0),
            (
                screen_width * 0.75,
                screen_height * 0.5,
                std::f32::consts::PI,
            ),
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
                    angle,
                    ..Default::default()
                },
            );
        }

        let asteroids = Self::spawn_initial_asteroids(screen_width, screen_height);

        println!("✅ Game state initialized:");
        for (pid, p) in &players {
            println!(
                "   Player {:?}: pos=({:.1}, {:.1}), angle={:.2}",
                pid, p.x, p.y, p.angle
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
        let count = 4;

        (0..count)
            .map(|i| {
                let (x, y) = if rng.gen_bool(0.5) {
                    let x = if rng.gen_bool(0.5) { 0.0 } else { screen_width };
                    let y = rng.gen_range(0.0..screen_height);
                    (x, y)
                } else {
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
                    size: 3,
                    angle: rng.gen_range(0.0..std::f32::consts::TAU),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_new() {
        let room = Room::new(GameMode::Survival);
        assert_eq!(room.mode, GameMode::Survival);
        assert_eq!(room.max_players, 2);
        assert!(!room.started);
        assert!(room.players.is_empty());
        assert!(room.game_state.is_none());
    }

    #[test]
    fn test_room_is_full() {
        let mut room = Room::new(GameMode::Duel);
        assert!(!room.is_full());

        room.add_player(Uuid::new_v4());
        assert!(!room.is_full());

        room.add_player(Uuid::new_v4());
        assert!(room.is_full());
    }

    #[test]
    fn test_room_add_player() {
        let mut room = Room::new(GameMode::Survival);
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let p3 = Uuid::new_v4();

        assert!(room.add_player(p1));
        assert_eq!(room.player_count(), 1);

        assert!(room.add_player(p2));
        assert_eq!(room.player_count(), 2);

        // 房间已满，无法添加
        assert!(!room.add_player(p3));
        assert_eq!(room.player_count(), 2);
    }

    #[test]
    fn test_room_remove_player() {
        let mut room = Room::new(GameMode::Survival);
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        room.add_player(p1);
        room.add_player(p2);
        assert_eq!(room.player_count(), 2);

        assert!(room.remove_player(p1));
        assert_eq!(room.player_count(), 1);
        assert!(!room.players.contains(&p1));

        // 移除不存在的玩家
        assert!(!room.remove_player(p1));
    }

    #[test]
    fn test_room_is_empty() {
        let mut room = Room::new(GameMode::Duel);
        assert!(room.is_empty());

        let p1 = Uuid::new_v4();
        room.add_player(p1);
        assert!(!room.is_empty());

        room.remove_player(p1);
        assert!(room.is_empty());
    }

    #[test]
    fn test_room_init_game_state() {
        let mut room = Room::new(GameMode::Survival);
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        room.add_player(p1);
        room.add_player(p2);

        room.init_game_state(1024.0, 768.0);

        assert!(room.game_state.is_some());
        let state = room.game_state.as_ref().unwrap();
        assert_eq!(state.players.len(), 2);
        assert!(!state.asteroids.is_empty());
    }

    #[test]
    fn test_spawn_initial_asteroids() {
        let asteroids = Room::spawn_initial_asteroids(1024.0, 768.0);
        assert_eq!(asteroids.len(), 4);

        for asteroid in &asteroids {
            assert_eq!(asteroid.size, 3); // 大型小行星
            assert!(asteroid.x >= 0.0 && asteroid.x <= 1024.0);
            assert!(asteroid.y >= 0.0 && asteroid.y <= 768.0);
        }
    }
}
