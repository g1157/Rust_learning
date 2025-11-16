//! 能量道具模块
//!
//! 处理护盾道具的生成、拾取和视觉效果。
//!
//! ## 功能
//! - 随机位置生成护盾道具
//! - 定时刷新（10-20 秒间隔）
//! - 拾取碰撞检测
//! - 发光动画效果

use macroquad::prelude::*;

use crate::player::Player;

pub const SHIELD_SPAWN_MIN: f64 = 3.0;
pub const SHIELD_SPAWN_MAX: f64 = 8.0;
pub const SHIELD_POWERUP_DURATION: f64 = 5.0;
pub const POWERUP_RADIUS: f32 = 24.0;
pub const POWERUP_PICKUP_RADIUS: f32 = 30.0;

#[derive(Clone)]
pub struct PowerUp {
    pub pos: Vec2,
    pub expires_at: f64,
    pub collected: bool,
}

impl PowerUp {
    pub fn new(now: f64) -> Self {
        Self {
            pos: random_powerup_position(),
            expires_at: now + SHIELD_POWERUP_DURATION,
            collected: false,
        }
    }
}

pub fn spawn(now: f64, powerups: &mut Vec<PowerUp>, next_spawn: &mut f64) {
    if now >= *next_spawn {
        powerups.push(PowerUp::new(now));
        *next_spawn = schedule_next_spawn(now);
    }
    powerups.retain(|powerup| powerup.expires_at > now && !powerup.collected);
}

/// 处理道具拾取，返回是否有道具被拾取
pub fn handle_pickups(players: &mut [Player], powerups: &mut Vec<PowerUp>, now: f64) -> bool {
    let mut collected = false;
    for player in players.iter_mut() {
        if !player.alive {
            continue;
        }
        for powerup in powerups.iter_mut() {
            if powerup.collected || powerup.expires_at <= now {
                continue;
            }
            if (player.ship.pos - powerup.pos).length() <= POWERUP_PICKUP_RADIUS {
                player.grant_shield(now);
                powerup.collected = true;
                collected = true;
            }
        }
    }
    powerups.retain(|powerup| powerup.expires_at > now && !powerup.collected);
    collected
}

pub fn draw(powerups: &[PowerUp], frame_t: f64) {
    for powerup in powerups.iter() {
        let remaining = (powerup.expires_at - frame_t).max(0.0);
        let alpha = ((remaining / SHIELD_POWERUP_DURATION).clamp(0.25, 1.0)) as f32;
        draw_shield_icon(powerup.pos, alpha);
    }
}

pub fn schedule_next_spawn(now: f64) -> f64 {
    now + rand_seconds(SHIELD_SPAWN_MIN, SHIELD_SPAWN_MAX)
}

fn rand_seconds(min: f64, max: f64) -> f64 {
    rand::gen_range(min as f32, max as f32) as f64
}

pub fn random_powerup_position() -> Vec2 {
    Vec2::new(
        rand::gen_range(POWERUP_RADIUS, screen_width() - POWERUP_RADIUS),
        rand::gen_range(POWERUP_RADIUS, screen_height() - POWERUP_RADIUS),
    )
}

fn draw_shield_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.2, 0.6, 1.0, alpha);
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);
    draw_circle(
        center.x,
        center.y,
        POWERUP_RADIUS * 0.5,
        Color::new(0.2, 0.6, 1.0, alpha * 0.35),
    );
    draw_line(
        center.x - POWERUP_RADIUS * 0.4,
        center.y,
        center.x + POWERUP_RADIUS * 0.4,
        center.y,
        3.,
        color,
    );
    draw_line(
        center.x,
        center.y - POWERUP_RADIUS * 0.4,
        center.x,
        center.y + POWERUP_RADIUS * 0.4,
        3.,
        color,
    );
}
