//! 小行星模块
//!
//! 管理小行星的生成、移动、分裂和计分。
//!
//! ## 功能
//! - 三种尺寸的小行星（大、中、小）
//! - 小行星分裂机制（大 → 中 → 小）
//! - 基于尺寸的分数系统
//! - 随机生成安全区域外的小行星波次
//! - 屏幕边界环绕

use macroquad::prelude::*;

pub const ASTEROID_BASE_SPEED: f32 = 160.0; // 像素/秒
pub const ASTEROID_ROT_SPEED: f32 = 350.0; // 度/秒
pub const ASTEROID_SPLIT_SPEED_MIN: f32 = 150.0;
pub const ASTEROID_SPLIT_SPEED_MAX: f32 = 350.0;

pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub rot_speed: f32,
    pub size: f32,
    pub sides: u8,
    pub collided: bool,
}

impl Asteroid {
    pub fn spawn_initial(center: Vec2, playfield_extent: f32) -> Self {
        let direction = Vec2::new(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize();
        Self {
            pos: center + direction * playfield_extent / 2.,
            vel: Vec2::new(
                rand::gen_range(-ASTEROID_BASE_SPEED, ASTEROID_BASE_SPEED),
                rand::gen_range(-ASTEROID_BASE_SPEED, ASTEROID_BASE_SPEED),
            ),
            rot: 0.,
            rot_speed: rand::gen_range(-ASTEROID_ROT_SPEED, ASTEROID_ROT_SPEED),
            size: playfield_extent / 10.,
            sides: rand::gen_range(3, 8),
            collided: false,
        }
    }

    pub fn advance(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.rot += self.rot_speed * dt;
    }

    pub fn split(&self, bullet_vel: Vec2) -> Option<Vec<Asteroid>> {
        if self.sides <= 3 {
            return None;
        }

        let child_size = self.size * 0.8;
        let child_sides = self.sides - 1;

        let perp_a = Vec2::new(bullet_vel.y, -bullet_vel.x).normalize()
            * rand::gen_range(ASTEROID_SPLIT_SPEED_MIN, ASTEROID_SPLIT_SPEED_MAX);
        let perp_b = Vec2::new(-bullet_vel.y, bullet_vel.x).normalize()
            * rand::gen_range(ASTEROID_SPLIT_SPEED_MIN, ASTEROID_SPLIT_SPEED_MAX);

        Some(vec![
            Self::child_from(self, perp_a, child_size, child_sides),
            Self::child_from(self, perp_b, child_size, child_sides),
        ])
    }

    fn child_from(parent: &Asteroid, velocity: Vec2, size: f32, sides: u8) -> Asteroid {
        Asteroid {
            pos: parent.pos,
            vel: velocity,
            rot: rand::gen_range(0., 360.),
            rot_speed: rand::gen_range(-ASTEROID_ROT_SPEED, ASTEROID_ROT_SPEED),
            size,
            sides,
            collided: false,
        }
    }

    pub fn score_value(&self) -> u32 {
        if self.size > 40. {
            20
        } else if self.size > 20. {
            50
        } else {
            100
        }
    }
}

pub fn spawn_initial_wave(center: Vec2, playfield_extent: f32, count: usize) -> Vec<Asteroid> {
    (0..count)
        .map(|_| Asteroid::spawn_initial(center, playfield_extent))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asteroid_score_value_large() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 50.0,
            sides: 5,
            collided: false,
        };
        assert_eq!(asteroid.score_value(), 20);
    }

    #[test]
    fn test_asteroid_score_value_medium() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 30.0,
            sides: 4,
            collided: false,
        };
        assert_eq!(asteroid.score_value(), 50);
    }

    #[test]
    fn test_asteroid_score_value_small() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 15.0,
            sides: 3,
            collided: false,
        };
        assert_eq!(asteroid.score_value(), 100);
    }

    #[test]
    fn test_asteroid_split_creates_smaller() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 50.0,
            sides: 5,
            collided: false,
        };
        let bullet_vel = Vec2::new(100.0, 0.0);
        let children = asteroid.split(bullet_vel).expect("Should split");
        assert_eq!(children.len(), 2);
        assert!(children[0].size < asteroid.size);
        assert_eq!(children[0].sides, asteroid.sides - 1);
    }

    #[test]
    fn test_asteroid_split_minimum_sides_none() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 10.0,
            sides: 3,
            collided: false,
        };
        let bullet_vel = Vec2::new(100.0, 0.0);
        assert!(asteroid.split(bullet_vel).is_none());
    }

    #[test]
    fn test_asteroid_advance() {
        let mut asteroid = Asteroid {
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(100.0, 50.0),
            rot: 0.0,
            rot_speed: 45.0,
            size: 20.0,
            sides: 4,
            collided: false,
        };
        asteroid.advance(0.1);
        assert!((asteroid.pos.x - 10.0).abs() < 0.001);
        assert!((asteroid.pos.y - 5.0).abs() < 0.001);
        assert!((asteroid.rot - 4.5).abs() < 0.001);
    }

    #[test]
    fn test_spawn_initial_wave_count() {
        let center = Vec2::new(400.0, 300.0);
        let extent = 600.0;
        let count = 5;
        let wave = spawn_initial_wave(center, extent, count);
        assert_eq!(wave.len(), count);
    }
}
