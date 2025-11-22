//! 玩家模块
//!
//! 管理玩家状态、控制、射击和生命系统。
//!
//! ## 功能
//! - 玩家控制映射（双人支持）
//! - 生命和无敌时间系统
//! - 护盾道具机制
//! - 射击冷却时间
//! - 击杀连击系统（连击加成射速和速度）
//! - 生存时间追踪

use macroquad::prelude::*;

use crate::bullet::{Bullet, WeaponType};
use crate::score::Score;
use crate::ship::Ship;

pub const INVULNERABLE_DURATION: f64 = 3.0; // 秒
pub const HIT_INVULNERABLE_DURATION: f64 = 1.0; // 秒
pub const SHIELD_DURATION: f64 = 5.0; // 秒
pub const SHOOT_COOLDOWN: f64 = 0.5; // 秒
pub const WEAPON_POWERUP_DURATION: f64 = 10.0; // 武器道具持续时间

// 连击系统常量
const KILLSTREAK_RESET_TIME: f64 = 5.0; // 秒内无击杀则重置连击
const KILLSTREAK_FIRE_RATE_BONUS: f64 = 0.15; // 每次连击减少 15% 冷却时间（最多 3 次）
const KILLSTREAK_SPEED_BONUS: f32 = 30.0; // 每次连击增加 30 像素/秒速度（最多 3 次）

/// 武器道具类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponPowerUp {
    None,
    DualShot,   // 前后弹
    TripleShot, // 三向弹
}

pub struct Controls {
    pub thrust: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub shoot_primary: KeyCode,
    pub shoot_alt: Option<KeyCode>,
    pub weapon_switch: KeyCode,
    pub weapon_switch_alt: Option<KeyCode>,
}

impl Controls {
        pub fn shoot_pressed(&self, input: &crate::input::Input) -> bool {
            input.is_key_down(self.shoot_primary)
                || self.shoot_alt.map(|key| input.is_key_down(key)).unwrap_or(false)
        }

    pub fn weapon_switch_pressed(&self, input: &crate::input::Input) -> bool {
        input.is_key_pressed(self.weapon_switch)
            || self.weapon_switch_alt.map(|key| input.is_key_pressed(key)).unwrap_or(false)
    }
}

pub struct Player {
    pub label: &'static str,
    pub color: Color,
    pub ship: Ship,
    pub bullets: Vec<Bullet>,
    pub last_shot: f64,
    pub controls: Controls,
    pub score: Score,
    pub alive: bool,
    pub lives: u32,
    survival_start: f64,
    survival_end: Option<f64>,
    invulnerable_until: f64,
    shield_until: f64,
    shield_ready: bool,
    // 击杀连击系统
    pub killstreak: u32,
    last_kill_time: f64,
    // 武器系统
    pub weapon_type: WeaponType,
    // 武器道具系统
    pub weapon_powerup: WeaponPowerUp,
    weapon_powerup_until: f64,
}

impl Player {
    pub fn new(
        label: &'static str,
        color: Color,
        position: Vec2,
        controls: Controls,
        now: f64,
        starting_lives: u32,
    ) -> Self {
        Self {
            label,
            color,
            ship: Ship::new(position),
            bullets: Vec::new(),
            last_shot: now - 1.0,
            controls,
            score: Score::new(),
            alive: true,
            lives: starting_lives,
            survival_start: now + INVULNERABLE_DURATION,
            survival_end: None,
            invulnerable_until: now + INVULNERABLE_DURATION,
            shield_until: now,
            shield_ready: false,
            killstreak: 0,
            last_kill_time: 0.0,
            weapon_type: WeaponType::Normal,
            weapon_powerup: WeaponPowerUp::None,
            weapon_powerup_until: now,
        }
    }

    pub fn reset(&mut self, position: Vec2, now: f64, starting_lives: u32) {
        self.ship = Ship::reset(position);
        self.bullets.clear();
        self.last_shot = now - 1.0;
        self.score.reset();
        self.alive = true;
        self.lives = starting_lives;
        self.survival_start = now + INVULNERABLE_DURATION;
        self.survival_end = None;
        self.invulnerable_until = now + INVULNERABLE_DURATION;
        self.shield_until = now;
        self.shield_ready = false;
        self.killstreak = 0;
        self.last_kill_time = 0.0;
        self.weapon_powerup = WeaponPowerUp::None;
        self.weapon_powerup_until = now;
    }

    pub fn can_shoot(&self, now: f64) -> bool {
        now - self.last_shot > self.shoot_cooldown()
    }

    pub fn record_shot(&mut self, position: Vec2, direction: Vec2, now: f64) -> u32 {
        self.last_shot = now;
        
        // 检查武器道具是否过期
        if now >= self.weapon_powerup_until {
            self.weapon_powerup = WeaponPowerUp::None;
        }

        // 如果有武器道具，使用道具射击
        if self.weapon_powerup != WeaponPowerUp::None {
            return self.record_powerup_shot(position, direction, now);
        }

        // 否则使用普通武器
        match self.weapon_type {
            WeaponType::Normal => {
                self.bullets.push(Bullet::new(position, direction, now));
                1
            }
            WeaponType::Spread => {
                // 散弹：3 发，扇形 30 度
                use std::f32::consts::PI;
                let spread_angle = PI / 6.0; // 30 度

                for i in -1..=1 {
                    let angle = (i as f32) * spread_angle;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    // 旋转向量
                    let spread_dir = Vec2::new(
                        direction.x * cos_a - direction.y * sin_a,
                        direction.x * sin_a + direction.y * cos_a,
                    );

                    self.bullets.push(Bullet::with_weapon_type(
                        position,
                        spread_dir,
                        now,
                        WeaponType::Spread,
                    ));
                }
                3
            }
            WeaponType::Penetrating => {
                self.bullets.push(Bullet::with_weapon_type(
                    position,
                    direction,
                    now,
                    WeaponType::Penetrating,
                ));
                1
            }
        }
    }
    
    /// 使用武器道具射击
    fn record_powerup_shot(&mut self, position: Vec2, direction: Vec2, now: f64) -> u32 {
        match self.weapon_powerup {
            WeaponPowerUp::DualShot => {
                // 前后弹：向前和向后各一发
                self.bullets.push(Bullet::new(position, direction, now));
                self.bullets.push(Bullet::new(position, -direction, now));
                2
            }
            WeaponPowerUp::TripleShot => {
                // 三向弹：3 发扇形 60 度
                use std::f32::consts::PI;
                let spread_angle = PI / 6.0; // 30 度

                for i in -1..=1 {
                    let angle = (i as f32) * spread_angle;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    let spread_dir = Vec2::new(
                        direction.x * cos_a - direction.y * sin_a,
                        direction.x * sin_a + direction.y * cos_a,
                    );

                    self.bullets.push(Bullet::new(position, spread_dir, now));
                }
                3
            }
            WeaponPowerUp::None => unreachable!(),
        }
    }

    pub fn mark_dead(&mut self, time: f64) {
        if !self.alive || self.is_invulnerable(time) {
            return;
        }

        if self.consume_shield(time) {
            return;
        }

        if self.lives > 0 {
            self.lives -= 1;
        }

        if self.lives == 0 {
            self.alive = false;
            self.survival_end = Some(time);
        } else {
            self.invulnerable_until = time + HIT_INVULNERABLE_DURATION;
        }
    }

    pub fn survival_time(&self, current: f64) -> f64 {
        let end = self.survival_end.unwrap_or(current);
        (end - self.survival_start).max(0.0)
    }

    pub fn finalize_survival(&mut self, time: f64) {
        if self.survival_end.is_none() {
            self.survival_end = Some(time.max(self.survival_start));
        }
    }

    pub fn is_invulnerable(&self, time: f64) -> bool {
        time < self.invulnerable_until
    }

    pub fn invulnerability_remaining(&self, time: f64) -> f64 {
        if self.is_invulnerable(time) {
            self.invulnerable_until - time
        } else {
            0.0
        }
    }

    pub fn grant_shield(&mut self, time: f64) {
        self.shield_ready = true;
        self.shield_until = time + SHIELD_DURATION;
    }
    
    pub fn grant_dual_shot(&mut self, time: f64) {
        self.weapon_powerup = WeaponPowerUp::DualShot;
        self.weapon_powerup_until = time + WEAPON_POWERUP_DURATION;
    }
    
    pub fn grant_triple_shot(&mut self, time: f64) {
        self.weapon_powerup = WeaponPowerUp::TripleShot;
        self.weapon_powerup_until = time + WEAPON_POWERUP_DURATION;
    }

    pub fn shield_active(&self, time: f64) -> bool {
        self.shield_ready && time < self.shield_until
    }
    
    pub fn weapon_powerup_active(&self, time: f64) -> bool {
        self.weapon_powerup != WeaponPowerUp::None && time < self.weapon_powerup_until
    }
    
    pub fn weapon_powerup_remaining(&self, time: f64) -> f64 {
        if self.weapon_powerup_active(time) {
            self.weapon_powerup_until - time
        } else {
            0.0
        }
    }

    pub fn shield_remaining(&self, time: f64) -> f64 {
        if self.shield_active(time) {
            self.shield_until - time
        } else {
            0.0
        }
    }

    fn consume_shield(&mut self, time: f64) -> bool {
        if self.shield_active(time) {
            self.shield_ready = false;
            true
        } else {
            if time >= self.shield_until {
                self.shield_ready = false;
            }
            false
        }
    }

    /// 记录击杀并更新连击计数
    pub fn record_kill(&mut self, time: f64) {
        // 如果距离上次击杀超过 KILLSTREAK_RESET_TIME，重置连击
        if time - self.last_kill_time > KILLSTREAK_RESET_TIME {
            self.killstreak = 0;
        }

        self.killstreak += 1;
        self.last_kill_time = time;
    }

    /// 检查并重置过期的连击
    pub fn update_killstreak(&mut self, time: f64) {
        if self.killstreak > 0 && time - self.last_kill_time > KILLSTREAK_RESET_TIME {
            self.killstreak = 0;
        }
    }

    /// 获取当前射击冷却时间（考虑连击加成）
    pub fn shoot_cooldown(&self) -> f64 {
        let bonus_multiplier = 1.0 - (self.killstreak.min(3) as f64 * KILLSTREAK_FIRE_RATE_BONUS);
        SHOOT_COOLDOWN * bonus_multiplier
    }

    /// 获取当前最大速度（考虑连击加成）
    pub fn max_speed(&self) -> f32 {
        crate::ship::SHIP_MAX_SPEED + (self.killstreak.min(3) as f32 * KILLSTREAK_SPEED_BONUS)
    }

    /// 获取连击等级描述
    pub fn killstreak_level(&self) -> Option<&'static str> {
        match self.killstreak {
            0..=1 => None,
            2..=3 => Some("Double Kill!"),
            4..=5 => Some("Triple Kill!"),
            6..=9 => Some("Mega Kill!"),
            _ => Some("UNSTOPPABLE!"),
        }
    }
}
