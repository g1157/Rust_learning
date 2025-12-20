//! 碰撞检测系统
//!
//! 提供碰撞检测的核心逻辑和结果类型，从 main.rs 拆分出来以提高可维护性。
//!
//! ## 主要功能
//! - 玩家与小行星碰撞检测
//! - 玩家与 UFO 碰撞检测
//! - 子弹与小行星碰撞检测
//! - 子弹与 UFO 碰撞检测
//! - 敌人子弹与玩家碰撞检测
//!
//! 注意：部分函数为未来重构准备，暂时允许 dead_code

#![allow(dead_code)]

use macroquad::prelude::*;

use crate::asteroid::Asteroid;
use crate::bullet::BULLET_RADIUS;
use crate::player::Player;
use crate::quadtree::{ObjectIndex, QuadTree};
use crate::ship::SHIP_HEIGHT;
use crate::ufo::{EnemyBullet, ENEMY_BULLET_RADIUS, UFO_RADIUS, Ufo};
use crate::utils::circle_intersects_triangle;

// ============================================================================
// 碰撞结果类型
// ============================================================================

/// 玩家碰撞结果
#[derive(Debug, Clone)]
pub struct PlayerCollision {
    pub player_idx: usize,
    pub collision_pos: Vec2,
    pub damage_size: f32,
}

/// 小行星击杀信息
#[derive(Debug, Clone)]
pub struct AsteroidHit {
    pub player_idx: usize,
    pub asteroid_idx: usize,
    pub score_value: u32,
    pub pos: Vec2,
    pub size: f32,
    pub player_color: Color,
    pub bullet_vel: Vec2,
    pub is_chain_hit: bool,
}

/// UFO 击杀信息
#[derive(Debug, Clone)]
pub struct UfoHit {
    pub player_idx: usize,
    pub ufo_idx: usize,
    pub score_value: u32,
    pub pos: Vec2,
    pub drop_chance: f32,
    pub player_color: Color,
}

// ============================================================================
// 碰撞检测函数
// ============================================================================

/// 检测玩家与小行星的碰撞
///
/// 使用 QuadTree 进行空间分区优化，返回发生碰撞的玩家索引列表
pub fn check_player_asteroid_collisions(
    players: &[Player],
    asteroids: &[Asteroid],
    quadtree: &QuadTree,
    query_buffer: &mut Vec<ObjectIndex>,
    frame_t: f64,
) -> Vec<PlayerCollision> {
    let mut collisions = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable(frame_t) {
            continue;
        }

        let (t1, t2, t3) = player.ship.triangle_vertices();
        let ship_center = player.ship.pos;
        let ship_radius = SHIP_HEIGHT;

        query_buffer.clear();
        quadtree.query(ship_center, ship_radius, query_buffer);

        for obj in query_buffer.iter() {
            let asteroid = &asteroids[obj.index];
            if circle_intersects_triangle(asteroid.pos, asteroid.size, t1, t2, t3) {
                collisions.push(PlayerCollision {
                    player_idx,
                    collision_pos: ship_center,
                    damage_size: asteroid.size,
                });
                break; // 每个玩家每帧只处理一次碰撞
            }
        }
    }

    collisions
}

/// 检测玩家与 UFO 的碰撞
pub fn check_player_ufo_collisions(
    players: &[Player],
    ufos: &[Ufo],
    frame_t: f64,
) -> Vec<PlayerCollision> {
    let mut collisions = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable(frame_t) {
            continue;
        }

        let (t1, t2, t3) = player.ship.triangle_vertices();
        let ship_center = player.ship.pos;

        for ufo in ufos.iter() {
            if ufo.destroyed {
                continue;
            }

            if circle_intersects_triangle(ufo.pos, UFO_RADIUS, t1, t2, t3) {
                collisions.push(PlayerCollision {
                    player_idx,
                    collision_pos: ship_center,
                    damage_size: UFO_RADIUS,
                });
                break;
            }
        }
    }

    collisions
}

/// 检测敌人子弹与玩家的碰撞
pub fn check_enemy_bullet_player_collisions(
    players: &[Player],
    enemy_bullets: &[EnemyBullet],
    frame_t: f64,
) -> Vec<(usize, usize, Vec2)> {
    // 返回 (player_idx, bullet_idx, collision_pos)
    let mut collisions = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable(frame_t) {
            continue;
        }

        let (t1, t2, t3) = player.ship.triangle_vertices();
        let ship_center = player.ship.pos;

        for (bullet_idx, bullet) in enemy_bullets.iter().enumerate() {
            if bullet.collided {
                continue;
            }

            if circle_intersects_triangle(bullet.pos, ENEMY_BULLET_RADIUS, t1, t2, t3) {
                collisions.push((player_idx, bullet_idx, ship_center));
                break;
            }
        }
    }

    collisions
}

/// 检测子弹与 UFO 的碰撞（简单版本，不处理链式攻击）
pub fn check_bullet_ufo_collisions(
    players: &[Player],
    ufos: &[Ufo],
) -> Vec<(usize, usize, usize)> {
    // 返回 (player_idx, bullet_idx, ufo_idx)
    let mut collisions = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        for (bullet_idx, bullet) in player.bullets.iter().enumerate() {
            if bullet.collided {
                continue;
            }

            for (ufo_idx, ufo) in ufos.iter().enumerate() {
                if ufo.destroyed {
                    continue;
                }

                let dist = (ufo.pos - bullet.pos).length();
                if dist < ufo.radius() + BULLET_RADIUS {
                    collisions.push((player_idx, bullet_idx, ufo_idx));
                }
            }
        }
    }

    collisions
}

/// 检测圆形与圆形的碰撞
#[inline]
pub fn circles_collide(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    (pos1 - pos2).length_squared() < (radius1 + radius2).powi(2)
}

/// 检测点是否在圆内
#[inline]
pub fn point_in_circle(point: Vec2, center: Vec2, radius: f32) -> bool {
    (point - center).length_squared() < radius * radius
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circles_collide() {
        // 相交的圆
        assert!(circles_collide(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(15.0, 0.0),
            10.0
        ));

        // 不相交的圆
        assert!(!circles_collide(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(25.0, 0.0),
            10.0
        ));

        // 刚好相切
        assert!(!circles_collide(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(20.0, 0.0),
            10.0
        ));
    }

    #[test]
    fn test_point_in_circle() {
        let center = Vec2::new(100.0, 100.0);
        let radius = 50.0;

        // 圆心
        assert!(point_in_circle(center, center, radius));

        // 圆内
        assert!(point_in_circle(Vec2::new(120.0, 100.0), center, radius));

        // 圆外
        assert!(!point_in_circle(Vec2::new(200.0, 100.0), center, radius));
    }
}
