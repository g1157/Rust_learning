//! 武器系统模块
//!
//! 从 player.rs 拆分出来的武器相关逻辑。
//!
//! ## 功能
//! - 武器类型定义和管理
//! - 射击逻辑
//! - 武器道具效果
//! - 武器冷却计算

use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::bullet::{Bullet, WeaponType, BULLET_SPEED};
use crate::constants::{chain_ion, homing};

// ============================================================================
// 武器道具类型
// ============================================================================

/// 武器道具类型（临时增强）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponPowerUp {
    None,
    DualShot,   // 前后弹
    TripleShot, // 三向弹
}

impl WeaponPowerUp {
    /// 是否为有效道具
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }
}

// ============================================================================
// 武器状态
// ============================================================================

/// 武器系统状态
#[derive(Clone)]
pub struct WeaponState {
    /// 当前武器类型
    pub weapon_type: WeaponType,
    /// 上次射击时间
    pub last_shot: f64,
    /// 武器道具类型
    pub powerup: WeaponPowerUp,
    /// 武器道具结束时间
    pub powerup_until: f64,
}

impl WeaponState {
    pub fn new(now: f64) -> Self {
        Self {
            weapon_type: WeaponType::Normal,
            last_shot: now - 1.0,
            powerup: WeaponPowerUp::None,
            powerup_until: now,
        }
    }

    pub fn reset(&mut self, now: f64) {
        self.weapon_type = WeaponType::Normal;
        self.last_shot = 0.0;
        self.powerup = WeaponPowerUp::None;
        self.powerup_until = 0.0;
    }

    /// 检查武器道具是否激活
    pub fn is_powerup_active(&self, now: f64) -> bool {
        self.powerup.is_active() && now < self.powerup_until
    }

    /// 清理过期的武器道具
    pub fn update(&mut self, now: f64) {
        if now >= self.powerup_until {
            self.powerup = WeaponPowerUp::None;
        }
    }

    /// 激活武器道具
    pub fn activate_powerup(&mut self, powerup: WeaponPowerUp, duration: f64, now: f64) {
        self.powerup = powerup;
        self.powerup_until = now + duration;
    }

    /// 切换到下一个武器
    pub fn switch_weapon(&mut self) {
        self.weapon_type = match self.weapon_type {
            WeaponType::Normal => WeaponType::Spread,
            WeaponType::Spread => WeaponType::Penetrating,
            WeaponType::Penetrating => WeaponType::Homing,
            WeaponType::Homing => WeaponType::ChainIon,
            WeaponType::ChainIon => WeaponType::Normal,
        };
    }

    /// 获取当前武器的冷却时间
    pub fn get_cooldown(&self, base_cooldown: f64) -> f64 {
        match self.weapon_type {
            WeaponType::Homing => homing::COOLDOWN,
            WeaponType::ChainIon => chain_ion::COOLDOWN,
            _ => base_cooldown,
        }
    }
}

// ============================================================================
// 射击系统
// ============================================================================

/// 射击结果
pub struct ShotResult {
    pub bullets: Vec<Bullet>,
    pub bullet_count: u32,
}

/// 创建普通武器射击
pub fn shoot_normal(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    ShotResult {
        bullets: vec![Bullet::new(position, direction, now)],
        bullet_count: 1,
    }
}

/// 创建散弹射击
pub fn shoot_spread(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    let spread_angle = PI / 12.0; // 15度
    let mut bullets = Vec::with_capacity(3);

    for i in -1..=1 {
        let angle = (i as f32) * spread_angle;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let spread_dir = Vec2::new(
            direction.x * cos_a - direction.y * sin_a,
            direction.x * sin_a + direction.y * cos_a,
        );
        bullets.push(Bullet::new(position, spread_dir, now));
    }

    ShotResult {
        bullets,
        bullet_count: 3,
    }
}

/// 创建穿透弹射击
pub fn shoot_penetrating(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    ShotResult {
        bullets: vec![Bullet::with_weapon_type(
            position,
            direction,
            now,
            WeaponType::Penetrating,
        )],
        bullet_count: 1,
    }
}

/// 创建追踪导弹射击
pub fn shoot_homing(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    let missile_vel = direction.normalize() * homing::SPEED;
    ShotResult {
        bullets: vec![Bullet::with_weapon_type(
            position,
            missile_vel,
            now,
            WeaponType::Homing,
        )],
        bullet_count: 1,
    }
}

/// 创建链式离子炮射击
pub fn shoot_chain_ion(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    ShotResult {
        bullets: vec![Bullet::with_weapon_type(
            position,
            direction,
            now,
            WeaponType::ChainIon,
        )],
        bullet_count: 1,
    }
}

/// 创建前后双弹射击
pub fn shoot_dual(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    ShotResult {
        bullets: vec![
            Bullet::new(position, direction, now),
            Bullet::new(position, -direction, now),
        ],
        bullet_count: 2,
    }
}

/// 创建三向弹射击
pub fn shoot_triple(position: Vec2, direction: Vec2, now: f64) -> ShotResult {
    let spread_angle = PI / 6.0; // 30度
    let mut bullets = Vec::with_capacity(3);

    for i in -1..=1 {
        let angle = (i as f32) * spread_angle;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let spread_dir = Vec2::new(
            direction.x * cos_a - direction.y * sin_a,
            direction.x * sin_a + direction.y * cos_a,
        );
        bullets.push(Bullet::new(position, spread_dir, now));
    }

    ShotResult {
        bullets,
        bullet_count: 3,
    }
}

/// 根据武器类型和道具状态执行射击
pub fn shoot(
    weapon_state: &WeaponState,
    position: Vec2,
    direction: Vec2,
    speed_multiplier: f32,
    now: f64,
) -> ShotResult {
    let modified_direction = direction.normalize() * BULLET_SPEED * speed_multiplier;

    // 优先检查武器道具
    if weapon_state.is_powerup_active(now) {
        return match weapon_state.powerup {
            WeaponPowerUp::DualShot => shoot_dual(position, modified_direction, now),
            WeaponPowerUp::TripleShot => shoot_triple(position, modified_direction, now),
            WeaponPowerUp::None => shoot_normal(position, modified_direction, now),
        };
    }

    // 使用基础武器
    match weapon_state.weapon_type {
        WeaponType::Normal => shoot_normal(position, modified_direction, now),
        WeaponType::Spread => shoot_spread(position, modified_direction, now),
        WeaponType::Penetrating => shoot_penetrating(position, modified_direction, now),
        WeaponType::Homing => shoot_homing(position, direction, now),
        WeaponType::ChainIon => shoot_chain_ion(position, modified_direction, now),
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_state_new() {
        let state = WeaponState::new(0.0);
        assert_eq!(state.weapon_type, WeaponType::Normal);
        assert_eq!(state.powerup, WeaponPowerUp::None);
    }

    #[test]
    fn test_weapon_switch() {
        let mut state = WeaponState::new(0.0);
        state.switch_weapon();
        assert_eq!(state.weapon_type, WeaponType::Spread);
        state.switch_weapon();
        assert_eq!(state.weapon_type, WeaponType::Penetrating);
    }

    #[test]
    fn test_powerup_activation() {
        let mut state = WeaponState::new(0.0);
        state.activate_powerup(WeaponPowerUp::DualShot, 10.0, 0.0);
        assert!(state.is_powerup_active(5.0));
        assert!(!state.is_powerup_active(15.0));
    }

    #[test]
    fn test_shoot_normal() {
        let result = shoot_normal(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0);
        assert_eq!(result.bullet_count, 1);
        assert_eq!(result.bullets.len(), 1);
    }

    #[test]
    fn test_shoot_spread() {
        let result = shoot_spread(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0);
        assert_eq!(result.bullet_count, 3);
        assert_eq!(result.bullets.len(), 3);
    }
}
