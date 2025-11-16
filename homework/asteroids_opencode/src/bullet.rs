//! 子弹模块
//!
//! 处理玩家射击的子弹物理和生命周期。
//!
//! ## 功能
//! - 固定速度的子弹移动
//! - 基于时间的生命周期（1.5 秒）
//! - 碰撞检测标记
//! - 特殊武器类型（散弹、穿透弹）

use macroquad::prelude::*;

pub const BULLET_SPEED: f32 = 1500.0; // 像素/秒
pub const BULLET_LIFETIME: f64 = 1.5; // 秒
pub const BULLET_RADIUS: f32 = 3.0; // 像素

/// 武器类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponType {
    Normal,      // 普通单发
    Spread,      // 散弹（3发扇形）
    Penetrating, // 穿透弹（可击穿多个目标）
}

pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shot_at: f64,
    pub collided: bool,
    pub weapon_type: WeaponType,
    pub penetration_count: u32, // 穿透次数（穿透弹专用）
}

impl Bullet {
    pub fn new(position: Vec2, velocity: Vec2, shot_at: f64) -> Self {
        Self {
            pos: position,
            vel: velocity,
            shot_at,
            collided: false,
            weapon_type: WeaponType::Normal,
            penetration_count: 0,
        }
    }

    pub fn with_weapon_type(
        position: Vec2,
        velocity: Vec2,
        shot_at: f64,
        weapon_type: WeaponType,
    ) -> Self {
        let penetration_count = if weapon_type == WeaponType::Penetrating {
            3
        } else {
            0
        };
        Self {
            pos: position,
            vel: velocity,
            shot_at,
            collided: false,
            weapon_type,
            penetration_count,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }

    pub fn is_alive(&self, current_time: f64) -> bool {
        self.shot_at + BULLET_LIFETIME > current_time && !self.collided
    }

    /// 尝试穿透（仅穿透弹有效）
    pub fn try_penetrate(&mut self) -> bool {
        if self.weapon_type == WeaponType::Penetrating && self.penetration_count > 0 {
            self.penetration_count -= 1;
            true
        } else {
            self.collided = true;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bullet_creation() {
        let pos = Vec2::new(100.0, 200.0);
        let vel = Vec2::new(10.0, 0.0);
        let bullet = Bullet::new(pos, vel, 0.0);
        assert_eq!(bullet.pos, pos);
        assert_eq!(bullet.vel, vel);
        assert_eq!(bullet.shot_at, 0.0);
        assert!(!bullet.collided);
    }

    #[test]
    fn test_bullet_update() {
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0), 0.0);
        bullet.update(0.1);
        assert!((bullet.pos.x - 10.0).abs() < 0.001);
        assert!((bullet.pos.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_bullet_is_alive_fresh() {
        let bullet = Bullet::new(Vec2::ZERO, Vec2::ZERO, 0.0);
        assert!(bullet.is_alive(0.5));
    }

    #[test]
    fn test_bullet_is_alive_expired() {
        let bullet = Bullet::new(Vec2::ZERO, Vec2::ZERO, 0.0);
        assert!(!bullet.is_alive(2.0));
    }

    #[test]
    fn test_bullet_is_alive_collided() {
        let mut bullet = Bullet::new(Vec2::ZERO, Vec2::ZERO, 0.0);
        bullet.collided = true;
        assert!(!bullet.is_alive(0.5));
    }
}
