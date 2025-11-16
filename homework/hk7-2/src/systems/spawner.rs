/// 敌人生成系统
/// 负责生成敌人和道具
use crate::config::*;
use crate::entity::{EnemyType, PowerUp, PowerUpType, Shape};
use crate::game_state::Game;
use macroquad::prelude::*;

pub struct SpawnerSystem;

impl SpawnerSystem {
    /// 更新敌人和道具生成
    pub fn update(game: &mut Game) {
        // 根据难度调整的概率生成新敌人
        let spawn_threshold = game.get_enemy_spawn_threshold();
        if rand::gen_range(0, 99) >= spawn_threshold {
            Self::spawn_enemy(game);
        }

        // 随机生成道具
        if rand::gen_range(0, 10000) >= POWERUP_DROP_THRESHOLD {
            Self::spawn_powerup(game);
        }
    }

    /// 生成一个新敌人
    fn spawn_enemy(game: &mut Game) {
        let enemy_type = EnemyType::random();
        let size = enemy_type.size();
        let screen_w = screen_width();

        // 基础速度随机，再乘以难度倍率
        let base_speed = rand::gen_range(ENEMY_MIN_SPEED, ENEMY_MAX_SPEED);
        let speed = base_speed * game.get_enemy_speed_multiplier();

        game.enemies.push(Shape::new_enemy(
            enemy_type,
            speed,
            rand::gen_range(size / 2.0, screen_w - size / 2.0),
            -size,
        ));
    }

    /// 生成一个新道具
    fn spawn_powerup(game: &mut Game) {
        let powerup_type = PowerUpType::random();
        let size = powerup_type.size();
        let screen_w = screen_width();

        game.powerups.push(PowerUp::new(
            powerup_type,
            rand::gen_range(size / 2.0, screen_w - size / 2.0),
            -size,
        ));
    }
}
