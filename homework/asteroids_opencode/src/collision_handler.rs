//! 碰撞处理模块
//!
//! 从 main.rs 拆分出来的碰撞检测和响应逻辑。
//!
//! ## 功能
//! - 子弹与小行星碰撞
//! - 子弹与 UFO 碰撞
//! - 玩家与小行星碰撞
//! - 玩家与敌方子弹碰撞
//! - 玩家与漩涡碰撞
//! - 对战模式玩家互击

use macroquad::prelude::*;

use crate::asteroid::Asteroid;
use crate::bullet::{Bullet, WeaponType, BULLET_RADIUS};
use crate::chain_lightning::ChainLightningManager;
use crate::constants::{chain_ion, gameplay};
use crate::particle::ParticleSystem;
use crate::player::Player;
use crate::quadtree::{Bounds, ObjectIndex, QuadTree};
use crate::ship::SHIP_HEIGHT;
use crate::ufo::{EnemyBullet, Ufo, ENEMY_BULLET_RADIUS, UFO_RADIUS};
use crate::utils::circle_intersects_triangle;
use crate::vortex::VortexManager;

// ============================================================================
// 碰撞检测结果
// ============================================================================

/// 小行星击中信息
#[derive(Clone)]
pub struct AsteroidHit {
    pub player_idx: usize,
    pub asteroid_idx: usize,
    pub score_value: u32,
    pub asteroid_pos: Vec2,
    pub asteroid_size: f32,
    pub player_color: Color,
    pub bullet_vel: Vec2,
    pub is_chain_hit: bool,
}

/// UFO 击中信息
#[derive(Clone)]
pub struct UfoHit {
    pub player_idx: usize,
    pub ufo_idx: usize,
    pub score_value: u32,
    pub ufo_pos: Vec2,
    pub drop_chance: f32,
    pub player_color: Color,
}

/// 玩家伤害信息
#[derive(Clone)]
pub struct PlayerDamage {
    pub player_idx: usize,
    pub damage_type: DamageType,
    pub source_pos: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Asteroid,
    EnemyBullet,
    Vortex,
}

// ============================================================================
// 子弹与小行星碰撞
// ============================================================================

/// 检测子弹与小行星碰撞
pub fn detect_bullet_asteroid_collisions(
    players: &mut [Player],
    asteroids: &mut [Asteroid],
    quadtree: &QuadTree,
    bullet_query: &mut Vec<ObjectIndex>,
    chain_lightnings: &mut ChainLightningManager,
    frame_t: f64,
) -> (Vec<AsteroidHit>, Vec<usize>, Vec<Asteroid>) {
    let mut hits = Vec::new();
    let mut player_kills: Vec<usize> = vec![0; players.len()];
    let mut new_asteroids = Vec::new();

    for (player_idx, player) in players.iter_mut().enumerate() {
        for bullet in player.bullets.iter_mut() {
            if bullet.collided {
                continue;
            }

            bullet_query.clear();
            quadtree.query_circle(bullet.pos, BULLET_RADIUS + 60.0, bullet_query);

            for obj in bullet_query.iter() {
                if obj.index >= asteroids.len() {
                    continue;
                }

                let asteroid = &mut asteroids[obj.index];
                if asteroid.collided {
                    continue;
                }

                let dist = (asteroid.pos - bullet.pos).length();
                if dist < asteroid.size + BULLET_RADIUS {
                    // 命中小行星
                    let score_value = if asteroid.size >= gameplay::ASTEROID_SIZE_LARGE {
                        gameplay::SCORE_ASTEROID_LARGE
                    } else if asteroid.size >= gameplay::ASTEROID_SIZE_MEDIUM {
                        gameplay::SCORE_ASTEROID_MEDIUM
                    } else {
                        gameplay::SCORE_ASTEROID_SMALL
                    };

                    hits.push(AsteroidHit {
                        player_idx,
                        asteroid_idx: obj.index,
                        score_value,
                        asteroid_pos: asteroid.pos,
                        asteroid_size: asteroid.size,
                        player_color: player.color,
                        bullet_vel: bullet.vel,
                        is_chain_hit: false,
                    });

                    player_kills[player_idx] += 1;

                    // 链式离子炮效果
                    if bullet.weapon_type == WeaponType::ChainIon {
                        trigger_chain_lightning(
                            asteroid.pos,
                            obj.index,
                            asteroids,
                            chain_lightnings,
                            &mut hits,
                            &mut player_kills,
                            player_idx,
                            player.color,
                            bullet.vel,
                            frame_t,
                        );
                    }

                    // 分裂小行星
                    if asteroid.size > gameplay::ASTEROID_SIZE_MEDIUM {
                        let children = asteroid.split();
                        new_asteroids.extend(children);
                    }
                    asteroid.collided = true;

                    // 尝试穿透
                    if !bullet.try_penetrate() {
                        break;
                    }
                }
            }
        }
    }

    (hits, player_kills, new_asteroids)
}

/// 触发链式闪电效果
fn trigger_chain_lightning(
    origin: Vec2,
    origin_idx: usize,
    asteroids: &mut [Asteroid],
    chain_lightnings: &mut ChainLightningManager,
    hits: &mut Vec<AsteroidHit>,
    player_kills: &mut [usize],
    player_idx: usize,
    player_color: Color,
    bullet_vel: Vec2,
    frame_t: f64,
) {
    let mut current_pos = origin;
    let mut hit_indices = vec![origin_idx];

    for jump in 0..chain_ion::MAX_JUMPS {
        // 找到最近的未击中小行星
        let mut nearest: Option<(usize, f32)> = None;

        for (idx, asteroid) in asteroids.iter().enumerate() {
            if asteroid.collided || hit_indices.contains(&idx) {
                continue;
            }

            let dist = (asteroid.pos - current_pos).length();
            if dist < chain_ion::RANGE {
                if nearest.is_none() || dist < nearest.unwrap().1 {
                    nearest = Some((idx, dist));
                }
            }
        }

        if let Some((target_idx, _)) = nearest {
            let target = &mut asteroids[target_idx];
            let target_pos = target.pos;

            // 添加链式闪电视觉效果
            chain_lightnings.add_arc(current_pos, target_pos, frame_t as f32);

            // 应用伤害衰减
            let damage_ratio = chain_ion::DAMAGE_RATIOS
                .get(jump + 1)
                .copied()
                .unwrap_or(0.5);

            let score_value = if target.size >= gameplay::ASTEROID_SIZE_LARGE {
                (gameplay::SCORE_ASTEROID_LARGE as f32 * damage_ratio) as u32
            } else if target.size >= gameplay::ASTEROID_SIZE_MEDIUM {
                (gameplay::SCORE_ASTEROID_MEDIUM as f32 * damage_ratio) as u32
            } else {
                (gameplay::SCORE_ASTEROID_SMALL as f32 * damage_ratio) as u32
            };

            hits.push(AsteroidHit {
                player_idx,
                asteroid_idx: target_idx,
                score_value,
                asteroid_pos: target_pos,
                asteroid_size: target.size,
                player_color,
                bullet_vel,
                is_chain_hit: true,
            });

            player_kills[player_idx] += 1;
            target.collided = true;
            hit_indices.push(target_idx);
            current_pos = target_pos;
        } else {
            break;
        }
    }
}

// ============================================================================
// 子弹与 UFO 碰撞
// ============================================================================

/// 检测子弹与 UFO 碰撞
pub fn detect_bullet_ufo_collisions(
    players: &mut [Player],
    ufos: &mut [Ufo],
    particles: &mut ParticleSystem,
    frame_t: f64,
) -> Vec<UfoHit> {
    let mut hits = Vec::new();

    for (player_idx, player) in players.iter_mut().enumerate() {
        for bullet in player.bullets.iter_mut() {
            if bullet.collided {
                continue;
            }

            for (ufo_idx, ufo) in ufos.iter_mut().enumerate() {
                if ufo.destroyed {
                    continue;
                }

                let dist = (ufo.pos - bullet.pos).length();
                if dist < ufo.radius() + BULLET_RADIUS {
                    let destroyed = ufo.take_hit(1, frame_t);

                    // 爆炸效果
                    particles.spawn_explosion(
                        ufo.pos,
                        ufo.radius() * 0.8,
                        player.color,
                        frame_t as f32,
                    );

                    if destroyed {
                        hits.push(UfoHit {
                            player_idx,
                            ufo_idx,
                            score_value: ufo.score_value,
                            ufo_pos: ufo.pos,
                            drop_chance: ufo.drop_chance,
                            player_color: player.color,
                        });
                    }

                    if !bullet.try_penetrate() {
                        break;
                    }
                }
            }
        }
    }

    hits
}

// ============================================================================
// 玩家碰撞
// ============================================================================

/// 检测玩家与小行星碰撞
pub fn detect_player_asteroid_collisions(
    players: &[Player],
    asteroids: &[Asteroid],
    quadtree: &QuadTree,
    player_query: &mut Vec<ObjectIndex>,
) -> Vec<PlayerDamage> {
    let mut damages = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable() {
            continue;
        }

        player_query.clear();
        quadtree.query_circle(player.ship.pos, SHIP_HEIGHT + 30.0, player_query);

        for obj in player_query.iter() {
            if obj.index >= asteroids.len() {
                continue;
            }

            let asteroid = &asteroids[obj.index];
            if asteroid.collided {
                continue;
            }

            // 三角形与圆形碰撞检测
            let ship_points = player.ship.points();
            if circle_intersects_triangle(asteroid.pos, asteroid.size, ship_points) {
                damages.push(PlayerDamage {
                    player_idx,
                    damage_type: DamageType::Asteroid,
                    source_pos: asteroid.pos,
                });
                break;
            }
        }
    }

    damages
}

/// 检测玩家与敌方子弹碰撞
pub fn detect_player_enemy_bullet_collisions(
    players: &[Player],
    enemy_bullets: &[EnemyBullet],
) -> Vec<(usize, usize)> {
    let mut hits = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable() {
            continue;
        }

        let ship_points = player.ship.points();
        for (bullet_idx, bullet) in enemy_bullets.iter().enumerate() {
            if circle_intersects_triangle(bullet.pos, ENEMY_BULLET_RADIUS, ship_points) {
                hits.push((player_idx, bullet_idx));
            }
        }
    }

    hits
}

/// 检测玩家与漩涡碰撞
pub fn detect_player_vortex_collisions(
    players: &[Player],
    vortex_manager: &VortexManager,
) -> Vec<PlayerDamage> {
    let mut damages = Vec::new();

    for (player_idx, player) in players.iter().enumerate() {
        if !player.alive || player.is_invulnerable() {
            continue;
        }

        for vortex in vortex_manager.active_vortices() {
            let dist = (player.ship.pos - vortex.pos).length();
            // 只有进入漩涡核心才受伤
            if dist < vortex.core_radius() {
                damages.push(PlayerDamage {
                    player_idx,
                    damage_type: DamageType::Vortex,
                    source_pos: vortex.pos,
                });
                break;
            }
        }
    }

    damages
}

// ============================================================================
// QuadTree 构建
// ============================================================================

/// 更新 QuadTree 边界
pub fn update_quadtree_bounds(quadtree: &mut QuadTree) {
    quadtree.reset_bounds(Bounds::new(0.0, 0.0, screen_width(), screen_height()));
}

/// 将小行星插入 QuadTree
pub fn insert_asteroids_to_quadtree(quadtree: &mut QuadTree, asteroids: &[Asteroid]) {
    for (idx, asteroid) in asteroids.iter().enumerate() {
        if !asteroid.collided {
            quadtree.insert(ObjectIndex {
                index: idx,
                pos: asteroid.pos,
                radius: asteroid.size,
            });
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_type() {
        assert_eq!(DamageType::Asteroid, DamageType::Asteroid);
        assert_ne!(DamageType::Asteroid, DamageType::Vortex);
    }
}
