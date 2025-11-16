/// 移动系统
/// 负责更新所有实体的位置
use crate::game_state::Game;
use macroquad::prelude::*;

pub struct MovementSystem;

impl MovementSystem {
    /// 更新所有实体的移动
    pub fn update(game: &mut Game, delta_time: f32) {
        // 更新所有计时器
        game.update_timers(delta_time);

        // 移动敌人（向下）
        for enemy in &mut game.enemies {
            enemy.y += enemy.speed * delta_time;
        }

        // 移动子弹（向上）
        for bullet in &mut game.bullets {
            bullet.y -= bullet.speed * delta_time;
        }

        // 移动道具（向下）
        for powerup in &mut game.powerups {
            powerup.y += powerup.speed * delta_time;
        }

        // 清理屏幕外的实体
        Self::cleanup_offscreen_entities(game);

        // 清理已碰撞的实体
        Self::cleanup_collided_entities(game);

        // 清理完成的爆炸效果
        Self::cleanup_expired_explosions(game);
    }

    /// 清理屏幕外的实体
    fn cleanup_offscreen_entities(game: &mut Game) {
        let screen_h = screen_height();

        // 移除飞出屏幕下方的敌人
        game.enemies.retain(|enemy| enemy.y < screen_h + enemy.size);

        // 移除飞出屏幕上方的子弹
        game.bullets
            .retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

        // 移除飞出屏幕下方的道具
        game.powerups.retain(|powerup| {
            powerup.y < screen_h + powerup.powerup_type.size()
        });
    }

    /// 清理已碰撞的实体
    fn cleanup_collided_entities(game: &mut Game) {
        game.enemies.retain(|enemy| !enemy.collided);
        game.bullets.retain(|bullet| !bullet.collided);
        game.powerups.retain(|powerup| !powerup.collected);
    }

    /// 清理已完成的爆炸效果
    fn cleanup_expired_explosions(game: &mut Game) {
        game.explosions
            .retain(|(explosion, _)| explosion.config.emitting);
    }
}
