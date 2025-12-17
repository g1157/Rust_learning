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

/// 道具稀有度
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerUpRarity {
    Common,   // 普通（60%权重）
    Advanced, // 进阶（40%权重）
}

/// 道具类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerUpType {
    // === 原有道具 ===
    Shield,     // 护盾（普通）
    DualShot,   // 前后弹（普通）
    TripleShot, // 三向弹（普通）
    // === 新增道具 ===
    RapidFire,      // 射速+50%，6秒（普通）
    PiercingRounds, // 3次穿透，8秒（进阶）
    TempShield,     // 抵挡3次伤害（普通）
    GhostMode,      // 50%闪避，5秒（进阶）
    Overdrive,      // 速度+80%,转向+60%，7秒（进阶）
    TeleportCharge, // 技能冷却-50%，10秒（进阶）
}

impl PowerUpType {
    /// 获取道具稀有度
    pub fn rarity(self) -> PowerUpRarity {
        match self {
            Self::Shield | Self::DualShot | Self::TripleShot |
            Self::RapidFire | Self::TempShield => PowerUpRarity::Common,
            Self::PiercingRounds | Self::GhostMode |
            Self::Overdrive | Self::TeleportCharge => PowerUpRarity::Advanced,
        }
    }

    /// 获取道具持续时间（秒）
    pub fn duration(self) -> f64 {
        match self {
            Self::Shield => SHIELD_POWERUP_DURATION,
            Self::DualShot | Self::TripleShot => WEAPON_POWERUP_DURATION,
            Self::RapidFire => 6.0,
            Self::PiercingRounds => 8.0,
            Self::TempShield => 30.0, // 长时间，但用完3次就消失
            Self::GhostMode => 5.0,
            Self::Overdrive => 7.0,
            Self::TeleportCharge => 10.0,
        }
    }

    /// 根据权重随机选择道具类型
    pub fn random_weighted() -> Self {
        let roll = rand::gen_range(0.0f32, 1.0);
        if roll < 0.6 {
            // 60% 普通道具
            Self::random_common()
        } else {
            // 40% 进阶道具
            Self::random_advanced()
        }
    }

    fn random_common() -> Self {
        match rand::gen_range(0, 5) {
            0 => Self::Shield,
            1 => Self::DualShot,
            2 => Self::TripleShot,
            3 => Self::RapidFire,
            _ => Self::TempShield,
        }
    }

    fn random_advanced() -> Self {
        match rand::gen_range(0, 4) {
            0 => Self::PiercingRounds,
            1 => Self::GhostMode,
            2 => Self::Overdrive,
            _ => Self::TeleportCharge,
        }
    }
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
        let duration = powerup_type.duration();
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
    // 定时生成道具（使用权重随机选择类型）
    if now >= *next_spawn {
        powerups.push(PowerUp::new(now, PowerUpType::random_weighted()));
        *next_spawn = schedule_next_spawn(now, player_count);
    }

    // 额外的武器道具生成点（保持兼容性）
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

/// 道具拾取信息（用于生成粒子效果）
pub struct PickupInfo {
    pub pos: Vec2,
    pub color: Color,
}

/// 处理道具拾取，返回拾取的道具信息列表
pub fn handle_pickups(players: &mut [Player], powerups: &mut Vec<PowerUp>, now: f64) -> Vec<PickupInfo> {
    let mut pickups = Vec::new();
    for player in players.iter_mut() {
        if !player.alive {
            continue;
        }
        for powerup in powerups.iter_mut() {
            if powerup.collected || powerup.expires_at <= now {
                continue;
            }
            if (player.ship.pos - powerup.pos).length() <= POWERUP_PICKUP_RADIUS {
                // 获取道具颜色（用于粒子效果）
                let pickup_color = powerup_color(powerup.powerup_type);

                match powerup.powerup_type {
                    // === 原有道具 ===
                    PowerUpType::Shield => {
                        player.grant_shield(now);
                    }
                    PowerUpType::DualShot => {
                        player.grant_dual_shot(now);
                    }
                    PowerUpType::TripleShot => {
                        player.grant_triple_shot(now);
                    }
                    // === 新增道具 ===
                    PowerUpType::RapidFire => {
                        player.grant_rapid_fire(now);
                    }
                    PowerUpType::PiercingRounds => {
                        player.grant_piercing_rounds(now);
                    }
                    PowerUpType::TempShield => {
                        player.grant_temp_shield();
                    }
                    PowerUpType::GhostMode => {
                        player.grant_ghost_mode(now);
                    }
                    PowerUpType::Overdrive => {
                        player.grant_overdrive(now);
                    }
                    PowerUpType::TeleportCharge => {
                        player.grant_teleport_charge(now);
                    }
                }
                powerup.collected = true;
                pickups.push(PickupInfo {
                    pos: powerup.pos,
                    color: pickup_color,
                });
            }
        }
    }
    powerups.retain(|powerup| powerup.expires_at > now && !powerup.collected);
    pickups
}

/// 获取道具类型对应的颜色
fn powerup_color(powerup_type: PowerUpType) -> Color {
    match powerup_type {
        PowerUpType::Shield => Color::new(0.2, 0.6, 1.0, 1.0),
        PowerUpType::DualShot => Color::new(1.0, 0.6, 0.2, 1.0),
        PowerUpType::TripleShot => Color::new(0.6, 1.0, 0.2, 1.0),
        PowerUpType::RapidFire => Color::new(1.0, 0.9, 0.2, 1.0),
        PowerUpType::PiercingRounds => Color::new(0.8, 0.3, 1.0, 1.0),
        PowerUpType::TempShield => Color::new(0.3, 0.8, 1.0, 1.0),
        PowerUpType::GhostMode => Color::new(0.7, 0.7, 0.9, 1.0),
        PowerUpType::Overdrive => Color::new(1.0, 0.3, 0.3, 1.0),
        PowerUpType::TeleportCharge => Color::new(0.6, 0.2, 0.9, 1.0),
    }
}

pub fn draw(powerups: &[PowerUp], frame_t: f64) {
    for powerup in powerups.iter() {
        let remaining = (powerup.expires_at - frame_t).max(0.0);
        let duration = powerup.powerup_type.duration();
        let alpha = ((remaining / duration).clamp(0.25, 1.0)) as f32;

        // 进阶道具添加光晕效果
        if powerup.powerup_type.rarity() == PowerUpRarity::Advanced {
            draw_advanced_glow(powerup.pos, frame_t as f32, alpha);
        }

        match powerup.powerup_type {
            PowerUpType::Shield => draw_shield_icon(powerup.pos, alpha),
            PowerUpType::DualShot => draw_dual_shot_icon(powerup.pos, alpha),
            PowerUpType::TripleShot => draw_triple_shot_icon(powerup.pos, alpha),
            PowerUpType::RapidFire => draw_rapid_fire_icon(powerup.pos, alpha),
            PowerUpType::PiercingRounds => draw_piercing_icon(powerup.pos, alpha),
            PowerUpType::TempShield => draw_temp_shield_icon(powerup.pos, alpha),
            PowerUpType::GhostMode => draw_ghost_mode_icon(powerup.pos, alpha),
            PowerUpType::Overdrive => draw_overdrive_icon(powerup.pos, alpha),
            PowerUpType::TeleportCharge => draw_teleport_icon(powerup.pos, alpha),
        }
    }
}

/// 绘制进阶道具的光晕效果
fn draw_advanced_glow(center: Vec2, time: f32, alpha: f32) {
    // 脉动光晕
    let pulse = 0.7 + 0.3 * (time * 3.0).sin();
    let glow_radius = POWERUP_RADIUS * 1.5 * pulse;

    // 外层光晕（紫色）
    draw_circle(
        center.x,
        center.y,
        glow_radius,
        Color::new(0.6, 0.3, 0.9, 0.15 * alpha * pulse),
    );

    // 中层光晕
    draw_circle(
        center.x,
        center.y,
        glow_radius * 0.7,
        Color::new(0.7, 0.4, 1.0, 0.2 * alpha * pulse),
    );

    // 旋转光芒
    let rotation = time * 2.0;
    for i in 0..4 {
        let angle = rotation + (i as f32 * std::f32::consts::FRAC_PI_2);
        let ray_length = POWERUP_RADIUS * 1.8;
        let end_x = center.x + angle.cos() * ray_length;
        let end_y = center.y + angle.sin() * ray_length;

        draw_line(
            center.x,
            center.y,
            end_x,
            end_y,
            2.0,
            Color::new(0.8, 0.5, 1.0, 0.3 * alpha * pulse),
        );
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

// ============================================================================
// 新道具图标
// ============================================================================

/// 快速射击图标（黄色闪电）
fn draw_rapid_fire_icon(center: Vec2, alpha: f32) {
    let color = Color::new(1.0, 0.9, 0.2, alpha); // 亮黄色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 闪电形状
    let r = POWERUP_RADIUS * 0.5;
    let points = [
        Vec2::new(center.x - r * 0.3, center.y - r),
        Vec2::new(center.x + r * 0.2, center.y - r * 0.2),
        Vec2::new(center.x - r * 0.1, center.y),
        Vec2::new(center.x + r * 0.4, center.y + r),
    ];
    for i in 0..3 {
        draw_line(points[i].x, points[i].y, points[i + 1].x, points[i + 1].y, 3., color);
    }
}

/// 穿透弹图标（紫色箭头穿过两个圆）
fn draw_piercing_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.8, 0.3, 1.0, alpha); // 紫色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 穿透箭头
    let r = POWERUP_RADIUS * 0.5;
    draw_line(center.x - r, center.y, center.x + r, center.y, 3., color);
    draw_line(center.x + r, center.y, center.x + r * 0.5, center.y - r * 0.4, 2., color);
    draw_line(center.x + r, center.y, center.x + r * 0.5, center.y + r * 0.4, 2., color);

    // 两个被穿透的小圆
    draw_circle_lines(center.x - r * 0.3, center.y, r * 0.25, 2., color);
    draw_circle_lines(center.x + r * 0.3, center.y, r * 0.25, 2., color);
}

/// 临时护盾图标（蓝色盾牌带数字3）
fn draw_temp_shield_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.3, 0.8, 1.0, alpha); // 天蓝色

    // 盾牌形状（六边形）
    let r = POWERUP_RADIUS * 0.7;
    let vertices: [Vec2; 6] = [
        Vec2::new(center.x, center.y - r),
        Vec2::new(center.x + r * 0.8, center.y - r * 0.3),
        Vec2::new(center.x + r * 0.6, center.y + r * 0.6),
        Vec2::new(center.x, center.y + r),
        Vec2::new(center.x - r * 0.6, center.y + r * 0.6),
        Vec2::new(center.x - r * 0.8, center.y - r * 0.3),
    ];
    for i in 0..6 {
        let next = (i + 1) % 6;
        draw_line(vertices[i].x, vertices[i].y, vertices[next].x, vertices[next].y, 3., color);
    }

    // 中心数字 "3"
    draw_text("3", center.x - 5.0, center.y + 6.0, 20.0, color);
}

/// 幽灵模式图标（半透明幽灵）
fn draw_ghost_mode_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.7, 0.7, 0.9, alpha * 0.8); // 半透明灰蓝

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 幽灵身体（椭圆上半部 + 波浪下半部）
    let r = POWERUP_RADIUS * 0.45;
    draw_circle(center.x, center.y - r * 0.2, r, Color::new(0.7, 0.7, 0.9, alpha * 0.4));

    // 眼睛
    let eye_color = Color::new(0.2, 0.2, 0.3, alpha);
    draw_circle(center.x - r * 0.3, center.y - r * 0.3, r * 0.2, eye_color);
    draw_circle(center.x + r * 0.3, center.y - r * 0.3, r * 0.2, eye_color);
}

/// 超速模式图标（红色双箭头）
fn draw_overdrive_icon(center: Vec2, alpha: f32) {
    let color = Color::new(1.0, 0.3, 0.3, alpha); // 红色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 双向加速箭头
    let r = POWERUP_RADIUS * 0.5;

    // 右箭头
    draw_line(center.x, center.y, center.x + r * 0.8, center.y, 3., color);
    draw_line(center.x + r * 0.8, center.y, center.x + r * 0.4, center.y - r * 0.4, 2., color);
    draw_line(center.x + r * 0.8, center.y, center.x + r * 0.4, center.y + r * 0.4, 2., color);

    // 左箭头（较短，表示旋转加速）
    draw_line(center.x - r * 0.2, center.y - r * 0.5, center.x - r * 0.2, center.y + r * 0.5, 3., color);
    draw_line(center.x - r * 0.2, center.y - r * 0.5, center.x - r * 0.5, center.y - r * 0.2, 2., color);
    draw_line(center.x - r * 0.2, center.y + r * 0.5, center.x - r * 0.5, center.y + r * 0.2, 2., color);
}

/// 传送充能图标（紫色漩涡）
fn draw_teleport_icon(center: Vec2, alpha: f32) {
    let color = Color::new(0.6, 0.2, 0.9, alpha); // 深紫色

    // 外圈
    draw_circle_lines(center.x, center.y, POWERUP_RADIUS * 0.8, 3., color);

    // 漩涡效果（同心圆弧）
    let r = POWERUP_RADIUS * 0.5;
    for i in 0..3 {
        let radius = r * (0.3 + i as f32 * 0.25);
        let start_angle = i as f32 * 0.8;
        // 绘制弧线（简化为多段线）
        let segments = 6;
        for j in 0..segments {
            let a1 = start_angle + (j as f32 / segments as f32) * std::f32::consts::PI;
            let a2 = start_angle + ((j + 1) as f32 / segments as f32) * std::f32::consts::PI;
            let p1 = center + Vec2::new(a1.cos(), a1.sin()) * radius;
            let p2 = center + Vec2::new(a2.cos(), a2.sin()) * radius;
            draw_line(p1.x, p1.y, p2.x, p2.y, 2., color);
        }
    }

    // 中心点
    draw_circle(center.x, center.y, 3.0, color);
}
