/// 碰撞检测系统
/// 负责检测和处理所有碰撞事件
use crate::config::*;
use crate::game_state::Game;
use crate::systems::particles::particle_explosion;
use macroquad::audio::play_sound_once;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use macroquad_particles::{Emitter, EmitterConfig};

pub struct CollisionSystem;

impl CollisionSystem {
    /// 更新碰撞检测
    pub fn update(game: &mut Game) {
        // 检测子弹与敌人的碰撞
        Self::check_bullet_enemy_collisions(game);

        // 检测玩家与敌人的碰撞
        Self::check_player_enemy_collisions(game);

        // 检测玩家与道具的碰撞
        Self::check_player_powerup_collisions(game);
    }

    /// 检测子弹与敌人的碰撞
    fn check_bullet_enemy_collisions(game: &mut Game) {
        use crate::resources::Resources;
        let resources = storage::get::<Resources>();

        // 先收集所有碰撞信息
        let mut explosions_to_add = Vec::new();
        let mut total_score_add = 0u32;

        for enemy in game.enemies.iter_mut() {
            for bullet in game.bullets.iter_mut() {
                if !bullet.collided && !enemy.collided && bullet.collides_with(enemy) {
                    // 标记为已碰撞
                    bullet.collided = true;
                    enemy.collided = true;

                    // 根据敌人类型增加分数
                    let score_add = if let Some(enemy_type) = enemy.enemy_type {
                        enemy_type.score_value()
                    } else {
                        enemy.size.round() as u32
                    };
                    total_score_add += score_add;

                    // 记录爆炸信息（稍后添加）
                    explosions_to_add.push((enemy.x, enemy.y, enemy.size));

                    // 播放爆炸音效
                    play_sound_once(&resources.sound_explosion);
                }
            }
        }

        // 循环结束后统一更新分数和难度
        if total_score_add > 0 {
            game.score += total_score_add;
            game.high_score = game.high_score.max(game.score);
            game.update_difficulty();
        }

        // 统一创建爆炸效果
        for (x, y, size) in explosions_to_add {
            Self::create_explosion(game, x, y, size);
        }
    }

    /// 检测玩家与敌人的碰撞
    fn check_player_enemy_collisions(game: &mut Game) {
        // 只在非无敌状态下检测碰撞
        if game.is_invincible() {
            return;
        }

        // 先收集碰撞信息
        let mut collision_data: Option<(f32, f32, f32)> = None;

        for enemy in game.enemies.iter_mut() {
            if !enemy.collided && game.player.collides_with(enemy) {
                // 标记敌人已碰撞（避免重复碰撞）
                enemy.collided = true;

                // 记录碰撞信息
                collision_data = Some((enemy.x, enemy.y, enemy.size));

                break;
            }
        }

        // 循环结束后处理碰撞
        if let Some((x, y, size)) = collision_data {
            // 玩家受伤
            game.take_damage();

            // 创建爆炸效果
            Self::create_explosion(game, x, y, size);

            // 播放爆炸音效
            use crate::resources::Resources;
            let resources = storage::get::<Resources>();
            play_sound_once(&resources.sound_explosion);
        }
    }

    /// 创建爆炸效果
    fn create_explosion(game: &mut Game, x: f32, y: f32, size: f32) {
        use crate::resources::Resources;
        let resources = storage::get::<Resources>();

        let explosion = Emitter::new(EmitterConfig {
            amount: size.round() as u32 * EXPLOSION_PARTICLE_MULTIPLIER,
            texture: Some(resources.explosion_texture.clone()),
            ..particle_explosion()
        });

        game.explosions.push((explosion, vec2(x, y)));
    }

    /// 检测玩家与道具的碰撞
    fn check_player_powerup_collisions(game: &mut Game) {
        use crate::resources::Resources;
        let resources = storage::get::<Resources>();

        // 先收集所有需要应用的道具类型
        let mut collected_powerups = Vec::new();

        for powerup in game.powerups.iter_mut() {
            if !powerup.collected && powerup.collides_with(&game.player) {
                // 标记道具为已收集
                powerup.collected = true;
                collected_powerups.push(powerup.powerup_type);
            }
        }

        // 循环结束后统一应用道具效果
        for powerup_type in collected_powerups {
            game.collect_powerup(powerup_type);
            // 播放收集音效（使用激光音效代替）
            play_sound_once(&resources.sound_laser);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Shape;

    #[test]
    fn test_bullet_enemy_collision_detection() {
        let bullet = Shape {
            x: 100.0,
            y: 100.0,
            size: 10.0,
            speed: 100.0,
            collided: false,
            enemy_type: None,
        };

        let enemy = Shape {
            x: 105.0,
            y: 105.0,
            size: 20.0,
            speed: 50.0,
            collided: false,
            enemy_type: None,
        };

        // 这两个实体应该碰撞
        assert!(bullet.collides_with(&enemy));
    }
}
