//! 能量道具模块
//!
//! 处理护盾道具和武器道具的生成、拾取和视觉效果。
//!
//! ## 功能
//! - 随机位置生成护盾道具和武器道具
//! - 定时刷新（10-20 秒间隔）
//! - 拾取碰撞检测
//! - 发光动画效果

use macroquad::prelude::*;

use crate::PlayerCount;
use crate::constants::powerup as powerup_config;
use crate::player::Player;

// 使用集中化常量
pub const SHIELD_SPAWN_MIN: f64 = powerup_config::SHIELD_SPAWN_INTERVAL.0 as f64;
pub const SHIELD_SPAWN_MAX: f64 = powerup_config::SHIELD_SPAWN_INTERVAL.1 as f64;
pub const WEAPON_SPAWN_MIN: f64 = powerup_config::WEAPON_SPAWN_INTERVAL.0 as f64;
pub const WEAPON_SPAWN_MAX: f64 = powerup_config::WEAPON_SPAWN_INTERVAL.1 as f64;
pub const SHIELD_POWERUP_DURATION: f64 = 5.0;
pub const WEAPON_POWERUP_DURATION: f64 = 8.0;
pub const POWERUP_RADIUS: f32 = powerup_config::RADIUS;
pub const POWERUP_PICKUP_RADIUS: f32 = powerup_config::PICKUP_RADIUS;

/// 道具类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerUpType {
    Shield,     // 护盾
    DualShot,   // 前后弹
    TripleShot, // 三向弹
}

#[derive(Clone)]
pub struct PowerUp {
    pub pos: Vec2,
    pub expires_at: f64,
    pub collected: bool,
    pub powerup_type: PowerUpType,
}

impl PowerUp {
    pub fn new(now: f64, powerup_type: PowerUpType) -> Self {
        let duration = match powerup_type {
            PowerUpType::Shield => SHIELD_POWERUP_DURATION,
            _ => WEAPON_POWERUP_DURATION,
        };
        Self {
            pos: random_powerup_position(),
            expires_at: now + duration,
            collected: false,
            powerup_type,
        }
    }
}

pub fn spawn(
    now: f64,
    powerups: &mut Vec<PowerUp>,
    next_spawn: &mut f64,
    next_weapon_spawn: &mut f64,
    player_count: PlayerCount,
) {
    // 生成护盾道具
    if now >= *next_spawn {
        powerups.push(PowerUp::new(now, PowerUpType::Shield));
        *next_spawn = schedule_next_spawn(now, player_count);
    }

    // 生成武器道具（随机选择前后弹或三向弹）
    if now >= *next_weapon_spawn {
        let weapon_type = if rand::gen_range(0.0, 1.0) < 0.5 {
            PowerUpType::DualShot
        } else {
            PowerUpType::TripleShot
        };
        powerups.push(PowerUp::new(now, weapon_type));
        *next_weapon_spawn = schedule_next_weapon_spawn(now, player_count);
    }

    powerups.retain(|powerup| powerup.expires_at > now && !powerup.collected);
}

/// 处理道具拾取，返回拾取的道具数量
pub fn handle_pickups(players: &mut [Player], powerups: &mut Vec<PowerUp>, now: f64) -> u32 {
    let mut collected_count = 0;
    for player in players.iter_mut() {
        if !player.alive {
            continue;
        }
        for powerup in powerups.iter_mut() {
            if powerup.collected || powerup.expires_at <= now {
                continue;
            }
            if (player.ship.pos - powerup.pos).length() <= POWERUP_PICKUP_RADIUS {
                match powerup.powerup_type {
                    PowerUpType::Shield => {
                        player.grant_shield(now);
                    }
                    PowerUpType::DualShot => {
                        player.grant_dual_shot(now);
                    }
                    PowerUpType::TripleShot => {
                        player.grant_triple_shot(now);
                    }
                }
                powerup.collected = true;
                collected_count += 1;
            }
        }
    }
    powerups.retain(|powerup| powerup.expires_at > now && !powerup.collected);
    collected_count
}

pub fn draw(powerups: &[PowerUp], frame_t: f64) {
    for powerup in powerups.iter() {
        let remaining = (powerup.expires_at - frame_t).max(0.0);
        let duration = match powerup.powerup_type {
            PowerUpType::Shield => SHIELD_POWERUP_DURATION,
            _ => WEAPON_POWERUP_DURATION,
        };
        let alpha = ((remaining / duration).clamp(0.25, 1.0)) as f32;

        match powerup.powerup_type {
            PowerUpType::Shield => draw_shield_icon(powerup.pos, alpha),
            PowerUpType::DualShot => draw_dual_shot_icon(powerup.pos, alpha),
            PowerUpType::TripleShot => draw_triple_shot_icon(powerup.pos, alpha),
        }
    }
}

pub fn schedule_next_spawn(now: f64, player_count: PlayerCount) -> f64 {
    // 单人模式：道具刷新更快（减少25%等待时间）
    let (min, max) = match player_count {
        PlayerCount::One => (SHIELD_SPAWN_MIN * 0.75, SHIELD_SPAWN_MAX * 0.75),
        PlayerCount::Two => (SHIELD_SPAWN_MIN, SHIELD_SPAWN_MAX),
    };
    now + rand_seconds(min, max)
}

pub fn schedule_next_weapon_spawn(now: f64, player_count: PlayerCount) -> f64 {
    // 单人模式：武器道具刷新更快（减少25%等待时间）
    let (min, max) = match player_count {
        PlayerCount::One => (WEAPON_SPAWN_MIN * 0.75, WEAPON_SPAWN_MAX * 0.75),
        PlayerCount::Two => (WEAPON_SPAWN_MIN, WEAPON_SPAWN_MAX),
    };
    now + rand_seconds(min, max)
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

fn draw_dual_shot_icon(center: Vec2, alpha: f32) {
    let color = Color::new(1.0, 0.6, 0.2, alpha); // 橙色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 两个箭头（前后）
    let arrow_len = POWERUP_RADIUS * 0.6;

    // 向上箭头
    draw_line(
        center.x,
        center.y,
        center.x,
        center.y - arrow_len,
        3.,
        color,
    );
    draw_line(
        center.x,
        center.y - arrow_len,
        center.x - 5.0,
        center.y - arrow_len + 8.0,
        3.,
        color,
    );
    draw_line(
        center.x,
        center.y - arrow_len,
        center.x + 5.0,
        center.y - arrow_len + 8.0,
        3.,
        color,
    );

    // 向下箭头
    draw_line(
        center.x,
        center.y,
        center.x,
        center.y + arrow_len,
        3.,
        color,
    );
    draw_line(
        center.x,
        center.y + arrow_len,
        center.x - 5.0,
        center.y + arrow_len - 8.0,
        3.,
        color,
    );
    draw_line(
        center.x,
        center.y + arrow_len,
        center.x + 5.0,
        center.y + arrow_len - 8.0,
        3.,
        color,
    );
}

fn draw_triple_shot_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.6, 1.0, 0.2, alpha); // 绿色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 三个箭头（扇形）
    let arrow_len = POWERUP_RADIUS * 0.6;
    let spread_angle = std::f32::consts::PI / 6.0; // 30度

    for i in -1..=1 {
        let angle = -std::f32::consts::FRAC_PI_2 + spread_angle * i as f32; // 向上为基准
        let end_x = center.x + angle.cos() * arrow_len;
        let end_y = center.y + angle.sin() * arrow_len;

        // 箭头线
        draw_line(center.x, center.y, end_x, end_y, 3., color);

        // 箭头头部
        let tip_angle1 = angle + std::f32::consts::FRAC_PI_4;
        let tip_angle2 = angle - std::f32::consts::FRAC_PI_4;
        draw_line(
            end_x,
            end_y,
            end_x - tip_angle1.cos() * 8.0,
            end_y - tip_angle1.sin() * 8.0,
            3.,
            color,
        );
        draw_line(
            end_x,
            end_y,
            end_x - tip_angle2.cos() * 8.0,
            end_y - tip_angle2.sin() * 8.0,
            3.,
            color,
        );
    }
}
