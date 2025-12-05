//! 子弹模块
//!
//! 处理玩家射击的子弹物理和生命周期。
//!
//! ## 功能
//! - 固定速度的子弹移动
//! - 基于时间的生命周期（1.5 秒）
//! - 碰撞检测标记
//! - 特殊武器类型（散弹、穿透弹、追踪导弹）

use macroquad::prelude::*;

use crate::constants::homing;

pub const BULLET_SPEED: f32 = 1500.0; // 像素/秒
pub const BULLET_LIFETIME: f64 = 1.5; // 秒
pub const BULLET_RADIUS: f32 = 3.0; // 像素

/// 武器类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponType {
    Normal,      // 普通单发
    Spread,      // 散弹（3发扇形）
    Penetrating, // 穿透弹（可击穿多个目标）
    Homing,      // 追踪导弹（自动追踪最近目标）
}

pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shot_at: f64,
    pub collided: bool,
    pub weapon_type: WeaponType,
    pub penetration_count: u32, // 穿透次数（穿透弹专用）
    /// 追踪导弹当前目标位置（每帧更新）
    pub target_pos: Option<Vec2>,
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
            target_pos: None,
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
            target_pos: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // 追踪导弹特殊更新逻辑
        if self.weapon_type == WeaponType::Homing
            && let Some(target) = self.target_pos
        {
            // 计算目标方向
            let to_target = target - self.pos;
            let distance = to_target.length();

            if distance > 0.1 {
                let target_dir = to_target.normalize();
                let current_dir = self.vel.normalize();

                // 计算需要转向的角度
                let cross = current_dir.x * target_dir.y - current_dir.y * target_dir.x;

                // 限制转向速率
                let max_turn = homing::TURN_RATE * dt;
                let turn_amount = cross.clamp(-max_turn, max_turn);

                // 旋转速度向量
                let cos_a = turn_amount.cos();
                let sin_a = turn_amount.sin();
                let new_dir = Vec2::new(
                    current_dir.x * cos_a - current_dir.y * sin_a,
                    current_dir.x * sin_a + current_dir.y * cos_a,
                );

                self.vel = new_dir * homing::SPEED;
            }
        }

        self.pos += self.vel * dt;
    }

    /// 获取子弹的有效生命周期（追踪导弹更长）
    pub fn lifetime(&self) -> f64 {
        match self.weapon_type {
            WeaponType::Homing => homing::LIFETIME,
            _ => BULLET_LIFETIME,
        }
    }

    pub fn is_alive(&self, current_time: f64) -> bool {
        self.shot_at + self.lifetime() > current_time && !self.collided
    }

    /// 获取子弹的碰撞半径（追踪导弹更大）
    #[allow(dead_code)]
    pub fn radius(&self) -> f32 {
        match self.weapon_type {
            WeaponType::Homing => homing::RADIUS,
            _ => BULLET_RADIUS,
        }
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

    /// 设置追踪目标位置（用于追踪导弹）
    pub fn set_target(&mut self, target: Option<Vec2>) {
        self.target_pos = target;
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

    #[test]
    fn test_homing_missile_creation() {
        let pos = Vec2::new(100.0, 100.0);
        let vel = Vec2::new(600.0, 0.0); // homing::SPEED
        let bullet = Bullet::with_weapon_type(pos, vel, 0.0, WeaponType::Homing);
        assert_eq!(bullet.weapon_type, WeaponType::Homing);
        assert!(bullet.target_pos.is_none());
        // 追踪导弹生命周期更长 (3.0秒)
        assert!(bullet.is_alive(2.5));
    }

    #[test]
    fn test_homing_missile_tracks_target() {
        let pos = Vec2::new(0.0, 0.0);
        let vel = Vec2::new(600.0, 0.0); // 初始向右
        let mut bullet = Bullet::with_weapon_type(pos, vel, 0.0, WeaponType::Homing);

        // 设置目标在右上方
        bullet.set_target(Some(Vec2::new(100.0, 100.0)));
        bullet.update(0.1);

        // 导弹应该转向目标，y分量应该变为正值
        assert!(bullet.vel.y > 0.0, "Missile should turn towards target");
    }
}
