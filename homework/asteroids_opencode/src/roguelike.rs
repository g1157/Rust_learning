//! Roguelike 模式核心模块
//!
//! 实现 Run-based 游戏循环：波次战斗 → 奖励选择 → 商店 → Boss 战
//! 每次游玩体验不同，通过遗物和卡牌构建独特 build
//!
//! 注意：部分功能正在开发中，暂时允许 dead_code

#![allow(dead_code)]

use std::collections::HashSet;

use macroquad::prelude::*;
use macroquad::text::{Font, TextParams};

use crate::asteroid::{Asteroid, AsteroidType};
use crate::battle_draft::{Card, generate_draft_options};
use crate::player::Player;
use crate::ufo::Ufo;
use crate::utils::wrap_around;

// ============================================================================
// 区域定义
// ============================================================================

/// 游戏区域 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoneId {
    /// 第一区域：小行星带
    Zone1,
    /// 第二区域：UFO 领域
    Zone2,
    /// 第三区域：虫洞深渊
    Zone3,
}

impl ZoneId {
    /// 获取区域名称
    pub fn name(&self) -> &'static str {
        match self {
            ZoneId::Zone1 => "小行星带",
            ZoneId::Zone2 => "UFO 领域",
            ZoneId::Zone3 => "虫洞深渊",
        }
    }

    /// 获取区域波次数量
    pub fn wave_count(&self) -> u32 {
        match self {
            ZoneId::Zone1 => 3,
            ZoneId::Zone2 => 4,
            ZoneId::Zone3 => 5,
        }
    }

    /// 获取区域基础难度倍率（影响小行星速度和数量）
    /// Zone1 从 0.6 开始，比普通 Survival 更简单
    pub fn difficulty_multiplier(&self) -> f32 {
        match self {
            ZoneId::Zone1 => 0.6, // 简单开局
            ZoneId::Zone2 => 1.0, // 正常难度
            ZoneId::Zone3 => 1.5, // 困难
        }
    }

    /// 获取区域内波次的难度递增（每波增加的难度）
    pub fn wave_difficulty_increment(&self) -> f32 {
        match self {
            ZoneId::Zone1 => 0.1,  // 每波 +10%
            ZoneId::Zone2 => 0.15, // 每波 +15%
            ZoneId::Zone3 => 0.2,  // 每波 +20%
        }
    }

    /// 获取区域基础小行星数量
    pub fn base_asteroid_count(&self) -> usize {
        match self {
            ZoneId::Zone1 => 5,  // 少量小行星
            ZoneId::Zone2 => 8,  // 中等数量
            ZoneId::Zone3 => 12, // 大量小行星
        }
    }

    /// 获取区域每波增加的小行星数量
    pub fn asteroid_increment(&self) -> usize {
        match self {
            ZoneId::Zone1 => 1,
            ZoneId::Zone2 => 2,
            ZoneId::Zone3 => 3,
        }
    }

    /// 获取区域背景颜色
    pub fn background_color(&self) -> Color {
        match self {
            ZoneId::Zone1 => Color::new(0.02, 0.02, 0.08, 1.0),
            ZoneId::Zone2 => Color::new(0.05, 0.02, 0.08, 1.0),
            ZoneId::Zone3 => Color::new(0.08, 0.02, 0.05, 1.0),
        }
    }

    /// 获取下一个区域
    pub fn next(&self) -> Option<ZoneId> {
        match self {
            ZoneId::Zone1 => Some(ZoneId::Zone2),
            ZoneId::Zone2 => Some(ZoneId::Zone3),
            ZoneId::Zone3 => None,
        }
    }
}

// ============================================================================
// Boss 定义
// ============================================================================

/// Boss 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossKind {
    /// 巨型分裂小行星 - Zone 1 Boss
    GiantSplitter,
    /// UFO 母舰召唤者 - Zone 2 Boss
    UfoMothership,
    /// 虫洞守卫 - Zone 3 Boss
    WormholeWarden,
}

impl BossKind {
    /// 获取 Boss 名称
    pub fn name(&self) -> &'static str {
        match self {
            BossKind::GiantSplitter => "巨型分裂者",
            BossKind::UfoMothership => "UFO 母舰",
            BossKind::WormholeWarden => "虫洞守卫",
        }
    }

    /// 获取 Boss 基础血量
    pub fn base_health(&self) -> f32 {
        match self {
            BossKind::GiantSplitter => 500.0,
            BossKind::UfoMothership => 800.0,
            BossKind::WormholeWarden => 1200.0,
        }
    }

    /// 获取对应区域
    pub fn zone(&self) -> ZoneId {
        match self {
            BossKind::GiantSplitter => ZoneId::Zone1,
            BossKind::UfoMothership => ZoneId::Zone2,
            BossKind::WormholeWarden => ZoneId::Zone3,
        }
    }
}

/// Boss 状态
#[derive(Debug, Clone)]
pub struct BossState {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
    pub position: Vec2,
    pub phase: u32,
    pub phase_timer: f32,
    pub is_enraged: bool,
    /// 绝望阶段 (< 15% HP)
    pub is_desperate: bool,
    /// 绝望阶段弹幕计时器
    pub desperate_burst_timer: f32,
}

impl BossState {
    pub fn new(kind: BossKind) -> Self {
        let max_health = kind.base_health();
        Self {
            kind,
            health: max_health,
            max_health,
            position: Vec2::ZERO, // 位置在游戏中动态设置
            phase: 1,
            phase_timer: 0.0,
            is_enraged: false,
            is_desperate: false,
            desperate_burst_timer: 0.0,
        }
    }

    /// 设置 Boss 位置（在游戏中调用）
    pub fn set_position(&mut self, pos: Vec2) {
        self.position = pos;
    }

    /// 计算血量百分比
    pub fn health_percent(&self) -> f32 {
        self.health / self.max_health
    }

    /// 检查是否进入狂暴阶段
    pub fn check_enrage(&mut self) {
        if self.health_percent() < 0.3 && !self.is_enraged {
            self.is_enraged = true;
        }
    }

    /// 检查是否进入绝望阶段 (< 15% HP)
    pub fn check_desperate(&mut self) -> bool {
        if self.health_percent() < 0.15 && !self.is_desperate {
            self.is_desperate = true;
            self.desperate_burst_timer = 0.0;
            return true; // 刚进入绝望阶段
        }
        false
    }

    /// 绝望阶段是否应该发射弹幕
    pub fn should_desperate_burst(&mut self, dt: f32) -> bool {
        if !self.is_desperate {
            return false;
        }
        self.desperate_burst_timer += dt;
        // 绝望阶段每 0.8 秒发射一次弹幕
        if self.desperate_burst_timer >= 0.8 {
            self.desperate_burst_timer -= 0.8;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Zone1 Boss: GiantSplitter（最小可玩实现）
// ============================================================================

/// GiantSplitter 的碰撞/渲染半径（约为普通大型小行星的 3-4 倍）
pub const GIANT_SPLITTER_RADIUS: f32 = 140.0;

const GIANT_SPLITTER_SPEED_NORMAL: f32 = 80.0;
const GIANT_SPLITTER_SPEED_ENRAGED: f32 = 140.0;

const GIANT_SPLITTER_SUMMON_INTERVAL_NORMAL: f32 = 2.5;
const GIANT_SPLITTER_SUMMON_INTERVAL_ENRAGED: f32 = 1.2;

const GIANT_SPLITTER_SUMMON_COUNT_NORMAL: usize = 2;
const GIANT_SPLITTER_SUMMON_COUNT_ENRAGED: usize = 3;

const GIANT_SPLITTER_MAX_SUMMONED_ASTEROIDS: usize = 40;

// UFO 母舰常量
const UFO_MOTHERSHIP_SPEED_NORMAL: f32 = 80.0;
const UFO_MOTHERSHIP_SPEED_ENRAGED: f32 = 120.0;
const UFO_MOTHERSHIP_SUMMON_INTERVAL_NORMAL: f32 = 4.0;
const UFO_MOTHERSHIP_SUMMON_INTERVAL_ENRAGED: f32 = 2.5;
const UFO_MOTHERSHIP_SUMMON_COUNT_NORMAL: usize = 1;
const UFO_MOTHERSHIP_SUMMON_COUNT_ENRAGED: usize = 2;
const UFO_MOTHERSHIP_RADIUS: f32 = 60.0;
const UFO_MOTHERSHIP_MAX_UFOS: usize = 6;

// 虫洞守卫常量
const WORMHOLE_WARDEN_SPEED_NORMAL: f32 = 100.0;
const WORMHOLE_WARDEN_SPEED_ENRAGED: f32 = 150.0;
const WORMHOLE_WARDEN_TELEPORT_INTERVAL_NORMAL: f32 = 6.0;
const WORMHOLE_WARDEN_TELEPORT_INTERVAL_ENRAGED: f32 = 3.5;
const WORMHOLE_WARDEN_PROJECTILE_INTERVAL: f32 = 2.0;
const WORMHOLE_WARDEN_RADIUS: f32 = 50.0;

// 绝望阶段常量
const DESPERATE_BURST_COUNT: usize = 8; // 弹幕数量
const DESPERATE_SPEED_MULTIPLIER: f32 = 1.5; // 移动速度加成

/// 获取 Boss 碰撞半径
pub fn boss_radius(boss: &BossState) -> f32 {
    match boss.kind {
        BossKind::GiantSplitter => GIANT_SPLITTER_RADIUS,
        BossKind::UfoMothership => UFO_MOTHERSHIP_RADIUS,
        BossKind::WormholeWarden => WORMHOLE_WARDEN_RADIUS,
    }
}

/// Boss 行为更新入口
pub fn update_boss(
    boss: &mut BossState,
    players: &[Player],
    asteroids: &mut Vec<Asteroid>,
    ufos: &mut Vec<Ufo>,
    dt: f32,
) {
    // 检查阶段转换
    boss.check_enrage();
    boss.check_desperate();

    // 绝望阶段弹幕爆发（所有 Boss 通用）
    if boss.should_desperate_burst(dt) {
        spawn_desperate_burst(boss, asteroids);
    }

    match boss.kind {
        BossKind::GiantSplitter => update_giant_splitter(boss, players, asteroids, dt),
        BossKind::UfoMothership => update_ufo_mothership(boss, players, ufos, dt),
        BossKind::WormholeWarden => update_wormhole_warden(boss, players, asteroids, dt),
    }
}

/// GiantSplitter：缓慢追踪玩家 + 周期性召唤小型小行星
fn update_giant_splitter(
    boss: &mut BossState,
    players: &[Player],
    asteroids: &mut Vec<Asteroid>,
    dt: f32,
) {
    // 找到最近的存活玩家
    let Some((_, target_pos)) = players
        .iter()
        .filter(|p| p.alive)
        .map(|p| ((p.ship.pos - boss.position).length_squared(), p.ship.pos))
        .min_by(|a, b| a.0.total_cmp(&b.0))
    else {
        return;
    };

    // 移动速度（狂暴时加速）
    let speed = if boss.is_enraged {
        GIANT_SPLITTER_SPEED_ENRAGED
    } else {
        GIANT_SPLITTER_SPEED_NORMAL
    };

    // 追踪玩家
    let to_target = target_pos - boss.position;
    let dir = to_target.normalize_or_zero();
    boss.position += dir * speed * dt;
    boss.position = wrap_around(&boss.position);

    // 召唤计时器
    boss.phase_timer += dt;
    let summon_interval = if boss.is_enraged {
        GIANT_SPLITTER_SUMMON_INTERVAL_ENRAGED
    } else {
        GIANT_SPLITTER_SUMMON_INTERVAL_NORMAL
    };

    // 周期性召唤小行星
    if boss.phase_timer >= summon_interval
        && asteroids.len() < GIANT_SPLITTER_MAX_SUMMONED_ASTEROIDS
    {
        boss.phase_timer -= summon_interval;

        let spawn_count = if boss.is_enraged {
            GIANT_SPLITTER_SUMMON_COUNT_ENRAGED
        } else {
            GIANT_SPLITTER_SUMMON_COUNT_NORMAL
        };

        for _ in 0..spawn_count {
            if asteroids.len() >= GIANT_SPLITTER_MAX_SUMMONED_ASTEROIDS {
                break;
            }
            asteroids.push(spawn_giant_splitter_minion(boss.position, boss.is_enraged));
        }
    }
}

/// 绝望阶段弹幕爆发：向四周发射环形弹幕
fn spawn_desperate_burst(boss: &BossState, asteroids: &mut Vec<Asteroid>) {
    let radius = boss_radius(boss);
    for i in 0..DESPERATE_BURST_COUNT {
        let angle = (i as f32) * std::f32::consts::TAU / (DESPERATE_BURST_COUNT as f32);
        let dir = Vec2::new(angle.cos(), angle.sin());
        let spawn_pos = boss.position + dir * (radius + 15.0);
        let speed = 350.0; // 高速弹幕

        asteroids.push(Asteroid {
            pos: wrap_around(&spawn_pos),
            vel: dir * speed,
            size: 10.0, // 小型弹幕
            sides: 6,
            rot: angle,
            rot_speed: rand::gen_range(-3.0, 3.0),
            collided: false,
            vertex_offsets: std::array::from_fn(|_| rand::gen_range(0.85, 1.0)),
            asteroid_type: AsteroidType::Normal,
        });
    }
}

/// 生成 Boss 召唤的小型小行星
fn spawn_giant_splitter_minion(boss_pos: Vec2, enraged: bool) -> Asteroid {
    let dir = Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)).normalize_or_zero();
    // Boss 召唤物不应触发分裂：Asteroid::split() 在 size < 15.0 时直接返回 None
    // 这里将尺寸上限压到 14.0，确保不会因为分裂导致数量膨胀
    let size = rand::gen_range(10.0, 14.0);
    let speed = if enraged { 320.0 } else { 220.0 };

    // 在 Boss 周围生成
    let spawn_pos = boss_pos + dir * (GIANT_SPLITTER_RADIUS + size + 6.0);

    Asteroid {
        pos: wrap_around(&spawn_pos),
        vel: dir * speed,
        size,
        sides: rand::gen_range(6, 10),
        rot: rand::gen_range(0.0, std::f32::consts::TAU),
        rot_speed: rand::gen_range(-2.0, 2.0),
        collided: false,
        vertex_offsets: std::array::from_fn(|_| rand::gen_range(0.7, 1.0)),
        // 使用 Normal 类型避免引入额外特殊效果；不分裂由 size < 15.0 保证
        asteroid_type: AsteroidType::Normal,
    }
}

/// UfoMothership：快速移动 + 召唤UFO
fn update_ufo_mothership(
    boss: &mut BossState,
    players: &[Player],
    ufos: &mut Vec<Ufo>,
    dt: f32,
) {
    // 找到最近的存活玩家
    let Some((_, target_pos)) = players
        .iter()
        .filter(|p| p.alive)
        .map(|p| ((p.ship.pos - boss.position).length_squared(), p.ship.pos))
        .min_by(|a, b| a.0.total_cmp(&b.0))
    else {
        return;
    };

    // 移动速度（狂暴时加速）
    let speed = if boss.is_enraged {
        UFO_MOTHERSHIP_SPEED_ENRAGED
    } else {
        UFO_MOTHERSHIP_SPEED_NORMAL
    };

    // 环绕移动模式
    let to_target = target_pos - boss.position;
    let distance = to_target.length();

    // 在远处时直线接近，在近处时环绕
    let dir = if distance > 200.0 {
        to_target.normalize_or_zero()
    } else {
        // 环绕运动
        let perpendicular = Vec2::new(-to_target.y, to_target.x).normalize_or_zero();
        let orbit_strength = 0.7;
        let approach_strength = 0.3;
        (perpendicular * orbit_strength + to_target.normalize_or_zero() * approach_strength).normalize_or_zero()
    };

    boss.position += dir * speed * dt;
    boss.position = wrap_around(&boss.position);

    // 召唤计时器
    boss.phase_timer += dt;
    let summon_interval = if boss.is_enraged {
        UFO_MOTHERSHIP_SUMMON_INTERVAL_ENRAGED
    } else {
        UFO_MOTHERSHIP_SUMMON_INTERVAL_NORMAL
    };

    // 召唤UFO
    if boss.phase_timer >= summon_interval && ufos.len() < UFO_MOTHERSHIP_MAX_UFOS {
        boss.phase_timer -= summon_interval;

        let spawn_count = if boss.is_enraged {
            UFO_MOTHERSHIP_SUMMON_COUNT_ENRAGED
        } else {
            UFO_MOTHERSHIP_SUMMON_COUNT_NORMAL
        };

        for _ in 0..spawn_count {
            if ufos.len() >= UFO_MOTHERSHIP_MAX_UFOS {
                break;
            }
            ufos.push(spawn_ufo_mothership_minion(boss.position, boss.is_enraged));
        }
    }
}

/// 生成UFO母舰召唤的UFO
fn spawn_ufo_mothership_minion(_boss_pos: Vec2, enraged: bool) -> Ufo {
    use crate::ufo::ufo_config_for_wave;

    // 使用第3波的UFO配置作为基础，狂暴时升级到第5波
    let wave = if enraged { 5 } else { 3 };
    let config = ufo_config_for_wave(wave);

    Ufo::spawn_from_edge(get_time(), true, config) // 强制掉落道具
}

/// WormholeWarden：传送移动 + 发射虫洞弹丸
fn update_wormhole_warden(
    boss: &mut BossState,
    players: &[Player],
    asteroids: &mut Vec<Asteroid>,
    dt: f32,
) {
    // 找到最近的存活玩家
    let Some((_, target_pos)) = players
        .iter()
        .filter(|p| p.alive)
        .map(|p| ((p.ship.pos - boss.position).length_squared(), p.ship.pos))
        .min_by(|a, b| a.0.total_cmp(&b.0))
    else {
        return;
    };

    // 传送计时器
    boss.phase_timer += dt;
    let teleport_interval = if boss.is_enraged {
        WORMHOLE_WARDEN_TELEPORT_INTERVAL_ENRAGED
    } else {
        WORMHOLE_WARDEN_TELEPORT_INTERVAL_NORMAL
    };

    // 周期性传送
    if boss.phase_timer >= teleport_interval {
        boss.phase_timer -= teleport_interval;

        // 传送到玩家附近但不直接重叠
        let teleport_distance = rand::gen_range(150.0, 300.0);
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let teleport_pos = target_pos + Vec2::new(angle.cos(), angle.sin()) * teleport_distance;
        boss.position = wrap_around(&teleport_pos);

        // 传送时召唤虫洞弹丸
        for _ in 0..3 {
            asteroids.push(spawn_wormhole_projectile(boss.position, target_pos, boss.is_enraged));
        }
    }

    // 持续发射弹丸
    boss.phase += 1;
    if boss.phase % 120 == 0 { // 每2秒
        asteroids.push(spawn_wormhole_projectile(boss.position, target_pos, boss.is_enraged));
    }
}

/// 生成虫洞守卫发射的虫洞弹丸（特殊小行星）
fn spawn_wormhole_projectile(boss_pos: Vec2, target_pos: Vec2, enraged: bool) -> Asteroid {
    let to_target = target_pos - boss_pos;
    let dir = to_target.normalize_or_zero();

    // 虫洞弹丸：高速、发光的小型物体
    let size = rand::gen_range(8.0, 12.0);
    let speed = if enraged { 400.0 } else { 280.0 };

    Asteroid {
        pos: wrap_around(&(boss_pos + dir * (WORMHOLE_WARDEN_RADIUS + size + 10.0))),
        vel: dir * speed,
        size,
        sides: 6,
        rot: rand::gen_range(0.0, std::f32::consts::TAU),
        rot_speed: rand::gen_range(-4.0, 4.0),
        collided: false,
        vertex_offsets: std::array::from_fn(|_| rand::gen_range(0.8, 1.0)),
        asteroid_type: AsteroidType::Normal,
    }
}

/// Boss 渲染入口
pub fn draw_boss(boss: &BossState, offset: Vec2, time: f32) {
    match boss.kind {
        BossKind::GiantSplitter => draw_giant_splitter(boss, offset, time),
        BossKind::UfoMothership => draw_ufo_mothership(boss, offset, time),
        BossKind::WormholeWarden => draw_wormhole_warden(boss, offset, time),
    }
}

/// GiantSplitter 渲染（规则多边形 + 脉动效果）
pub fn draw_giant_splitter(boss: &BossState, offset: Vec2, time: f32) {
    if boss.kind != BossKind::GiantSplitter {
        return;
    }

    let center = boss.position + offset;
    let radius = GIANT_SPLITTER_RADIUS * (0.98 + 0.02 * (time * 2.0).sin());
    let color = if boss.is_enraged {
        Color::new(1.0, 0.25, 0.25, 1.0) // 狂暴时红色
    } else {
        Color::new(0.3, 0.9, 0.4, 1.0) // 正常绿色
    };

    // 外层光晕
    draw_circle(
        center.x,
        center.y,
        radius * 0.45,
        Color::new(color.r, color.g, color.b, 0.12),
    );
    // 外层多边形
    draw_poly_lines(center.x, center.y, 14, radius, time * 0.25, 5.0, color);
    // 内层多边形
    draw_poly_lines(
        center.x,
        center.y,
        10,
        radius * 0.72,
        -time * 0.18,
        2.0,
        DARKGRAY,
    );
}

/// UfoMothership 渲染（UFO形状 + 引擎光效）
pub fn draw_ufo_mothership(boss: &BossState, offset: Vec2, time: f32) {
    if boss.kind != BossKind::UfoMothership {
        return;
    }

    let center = boss.position + offset;
    let radius = UFO_MOTHERSHIP_RADIUS;
    let color = if boss.is_enraged {
        Color::new(1.0, 0.4, 0.1, 1.0) // 狂暴时橙色
    } else {
        Color::new(0.6, 0.6, 1.0, 1.0) // 正常蓝色
    };

    // UFO主体（椭圆）
    draw_ellipse(center.x, center.y, radius * 0.8, radius * 0.4, 0.0, color);

    // 顶部圆顶
    draw_circle(center.x, center.y - radius * 0.2, radius * 0.3, Color::new(0.8, 0.8, 0.9, 1.0));

    // 引擎光效
    let engine_offset = Vec2::new(-radius * 0.4, radius * 0.2);
    let engine_pos = center + engine_offset;
    draw_circle(engine_pos.x, engine_pos.y, 8.0, Color::new(0.3, 0.8, 1.0, 0.8));

    let engine_offset = Vec2::new(radius * 0.4, radius * 0.2);
    let engine_pos = center + engine_offset;
    draw_circle(engine_pos.x, engine_pos.y, 8.0, Color::new(0.3, 0.8, 1.0, 0.8));

    // 脉动效果
    draw_circle(center.x, center.y, radius * 0.6 + 3.0 * (time * 3.0).sin(), Color::new(color.r, color.g, color.b, 0.2));
}

/// WormholeWarden 渲染（虫洞形状 + 扭曲效果）
pub fn draw_wormhole_warden(boss: &BossState, offset: Vec2, time: f32) {
    if boss.kind != BossKind::WormholeWarden {
        return;
    }

    let center = boss.position + offset;
    let radius = WORMHOLE_WARDEN_RADIUS;
    let color = if boss.is_enraged {
        Color::new(0.8, 0.2, 0.8, 1.0) // 狂暴时紫色
    } else {
        Color::new(0.4, 0.2, 0.9, 1.0) // 正常深蓝紫色
    };

    // 外层虫洞环
    let ring_count = 3;
    for i in 0..ring_count {
        let ring_radius = radius * (0.3 + 0.2 * (i as f32));
        let rotation = time * (0.5 + 0.3 * (i as f32)) * if i % 2 == 0 { 1.0 } else { -1.0 };
        draw_poly_lines(center.x, center.y, 8, ring_radius, rotation, 3.0, color);
    }

    // 中心漩涡
    draw_circle(center.x, center.y, radius * 0.15, BLACK);

    // 扭曲光效
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 6.0 + time;
        let dist = radius * 0.8;
        let x = center.x + angle.cos() * dist;
        let y = center.y + angle.sin() * dist;
        draw_circle(x, y, 4.0, Color::new(0.8, 0.4, 1.0, 0.6));
    }
}

// ============================================================================
// 遗物系统
// ============================================================================

/// 遗物触发时机
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelicTrigger {
    /// 波次开始时
    OnWaveStart,
    /// 波次清空时
    OnWaveClear,
    /// 击杀敌人时
    OnKill,
    /// 受到伤害时
    OnDamage,
    /// 拾取道具时
    OnPickup,
    /// Boss 击败时
    OnBossDefeat,
    /// 被动效果（始终生效）
    Passive,
}

/// 遗物 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelicId {
    /// 薪水芯片 - 每波结束获得金币
    PaycheckChip,
    /// 抽卡手套 - 奖励选择时多一个选项
    DraftingGloves,
    /// 完美封印 - Boss 战无伤额外金币
    FlawlessSeal,
    /// 打捞磁铁 - 拾取道具时有几率获得金币
    SalvageMagnet,
    /// 收藏家账本 - 跳过稀有卡牌时返还金币
    CollectorLedger,
    /// 连击护符 - 连击伤害加成
    ComboAmulet,
    /// 护盾电池 - 护盾持续时间延长
    ShieldBattery,
    /// 相位增幅器 - 相位闪现伤害提升
    PhaseAmplifier,
    /// 幸运骰子 - 暴击几率提升
    LuckyDice,
    /// 磁力核心 - 道具拾取范围扩大
    MagneticCore,
    // === 新增行为改变型遗物 ===
    /// 肾上腺素注射器 - 击杀后短时间内伤害提升
    AdrenalineInjector,
    /// 纳米虫群 - 近距离敌人持续受伤
    NanoSwarm,
    /// 赌徒筹码 - 商店价格随机波动
    GamblersChip,
    /// 虚空锚点 - 相位闪现后留下伤害区域
    VoidAnchor,
    /// 链式反应堆 - 连锁离子炮额外跳跃
    ChainReactor,
    /// 时间膨胀器 - 击杀时触发短暂慢动作
    TimeDilator,
    // === 腐化遗物（高风险高回报）===
    /// 重型枪管 - 伤害+200%，后坐力+100%，射速-30%
    HeavyBarrel,
    /// 玻璃大炮 - 伤害+500%，生命上限=1
    GlassCannon,
    /// 虫洞引擎 - Hyperspace无风险，但每次使用-50分
    WormholeEngine,
    /// 吸血弹药 - 击杀回血，但无法拾取道具
    VampireAmmo,
    /// 狂战士之心 - 低血量时伤害翻倍，但无法回血
    BerserkerHeart,
    /// 贪婪契约 - 金币获取+100%，但受伤时损失金币
    GreedPact,
}

impl RelicId {
    /// 获取遗物名称
    pub fn name(&self) -> &'static str {
        match self {
            RelicId::PaycheckChip => "薪水芯片",
            RelicId::DraftingGloves => "抽卡手套",
            RelicId::FlawlessSeal => "完美封印",
            RelicId::SalvageMagnet => "打捞磁铁",
            RelicId::CollectorLedger => "收藏家账本",
            RelicId::ComboAmulet => "连击护符",
            RelicId::ShieldBattery => "护盾电池",
            RelicId::PhaseAmplifier => "相位增幅器",
            RelicId::LuckyDice => "幸运骰子",
            RelicId::MagneticCore => "磁力核心",
            RelicId::AdrenalineInjector => "肾上腺素",
            RelicId::NanoSwarm => "纳米虫群",
            RelicId::GamblersChip => "赌徒筹码",
            RelicId::VoidAnchor => "虚空锚点",
            RelicId::ChainReactor => "链式反应堆",
            RelicId::TimeDilator => "时间膨胀器",
            // 腐化遗物
            RelicId::HeavyBarrel => "重型枪管",
            RelicId::GlassCannon => "玻璃大炮",
            RelicId::WormholeEngine => "虫洞引擎",
            RelicId::VampireAmmo => "吸血弹药",
            RelicId::BerserkerHeart => "狂战士之心",
            RelicId::GreedPact => "贪婪契约",
        }
    }

    /// 获取遗物描述
    pub fn description(&self) -> &'static str {
        match self {
            RelicId::PaycheckChip => "每波结束时获得 10 金币",
            RelicId::DraftingGloves => "奖励选择时多一个选项",
            RelicId::FlawlessSeal => "Boss 战无伤获得 50 额外金币",
            RelicId::SalvageMagnet => "拾取道具时 30% 几率获得 5 金币",
            RelicId::CollectorLedger => "跳过稀有卡牌时返还 15 金币",
            RelicId::ComboAmulet => "连击数每增加 5，伤害 +10%",
            RelicId::ShieldBattery => "护盾持续时间 +50%",
            RelicId::PhaseAmplifier => "相位闪现爆炸伤害 +30%",
            RelicId::LuckyDice => "暴击几率 +15%",
            RelicId::MagneticCore => "道具拾取范围 +50%",
            RelicId::AdrenalineInjector => "击杀后 2 秒内伤害 +25%",
            RelicId::NanoSwarm => "近距离敌人每秒受 5 伤害",
            RelicId::GamblersChip => "商店价格 ±30% 随机波动",
            RelicId::VoidAnchor => "相位闪现后留下 3 秒伤害区域",
            RelicId::ChainReactor => "连锁离子炮额外跳跃 +1",
            RelicId::TimeDilator => "击杀时 20% 几率触发 0.3 秒慢动作",
            // 腐化遗物（显示代价）
            RelicId::HeavyBarrel => "伤害+200% | 代价: 后坐力+100%, 射速-30%",
            RelicId::GlassCannon => "伤害+500% | 代价: 生命上限=1",
            RelicId::WormholeEngine => "Hyperspace无风险 | 代价: 每次-50分",
            RelicId::VampireAmmo => "击杀回血 | 代价: 无法拾取道具",
            RelicId::BerserkerHeart => "低血量伤害x2 | 代价: 无法回血",
            RelicId::GreedPact => "金币+100% | 代价: 受伤损失金币",
        }
    }

    /// 获取遗物触发时机
    pub fn trigger(&self) -> RelicTrigger {
        match self {
            RelicId::PaycheckChip => RelicTrigger::OnWaveClear,
            RelicId::DraftingGloves => RelicTrigger::Passive,
            RelicId::FlawlessSeal => RelicTrigger::OnBossDefeat,
            RelicId::SalvageMagnet => RelicTrigger::OnPickup,
            RelicId::CollectorLedger => RelicTrigger::Passive,
            RelicId::ComboAmulet => RelicTrigger::Passive,
            RelicId::ShieldBattery => RelicTrigger::Passive,
            RelicId::PhaseAmplifier => RelicTrigger::Passive,
            RelicId::LuckyDice => RelicTrigger::Passive,
            RelicId::MagneticCore => RelicTrigger::Passive,
            RelicId::AdrenalineInjector => RelicTrigger::OnKill,
            RelicId::NanoSwarm => RelicTrigger::Passive,
            RelicId::GamblersChip => RelicTrigger::Passive,
            RelicId::VoidAnchor => RelicTrigger::Passive,
            RelicId::ChainReactor => RelicTrigger::Passive,
            RelicId::TimeDilator => RelicTrigger::OnKill,
            // 腐化遗物
            RelicId::HeavyBarrel => RelicTrigger::Passive,
            RelicId::GlassCannon => RelicTrigger::Passive,
            RelicId::WormholeEngine => RelicTrigger::Passive,
            RelicId::VampireAmmo => RelicTrigger::OnKill,
            RelicId::BerserkerHeart => RelicTrigger::Passive,
            RelicId::GreedPact => RelicTrigger::Passive,
        }
    }

    /// 获取遗物稀有度颜色
    pub fn rarity_color(&self) -> Color {
        match self {
            // 普通 - 灰色
            RelicId::PaycheckChip | RelicId::SalvageMagnet | RelicId::MagneticCore => {
                Color::new(0.6, 0.6, 0.6, 1.0)
            }
            // 稀有 - 蓝色
            RelicId::DraftingGloves | RelicId::ComboAmulet | RelicId::ShieldBattery
            | RelicId::AdrenalineInjector | RelicId::GamblersChip => {
                Color::new(0.2, 0.6, 1.0, 1.0)
            }
            // 史诗 - 紫色
            RelicId::FlawlessSeal | RelicId::PhaseAmplifier | RelicId::LuckyDice
            | RelicId::NanoSwarm | RelicId::VoidAnchor | RelicId::ChainReactor => {
                Color::new(0.8, 0.4, 1.0, 1.0)
            }
            // 传说 - 金色
            RelicId::CollectorLedger | RelicId::TimeDilator => {
                Color::new(1.0, 0.8, 0.2, 1.0)
            }
            // 腐化 - 深红色（特殊标识）
            RelicId::HeavyBarrel | RelicId::GlassCannon | RelicId::WormholeEngine
            | RelicId::VampireAmmo | RelicId::BerserkerHeart | RelicId::GreedPact => {
                Color::new(0.8, 0.1, 0.2, 1.0)
            }
        }
    }

    /// 检查是否为腐化遗物
    pub fn is_corrupted(&self) -> bool {
        matches!(
            self,
            RelicId::HeavyBarrel
                | RelicId::GlassCannon
                | RelicId::WormholeEngine
                | RelicId::VampireAmmo
                | RelicId::BerserkerHeart
                | RelicId::GreedPact
        )
    }
}

// ============================================================================
// Run 阶段状态机
// ============================================================================

/// 挑战类型（可组合）
#[derive(Debug, Clone)]
pub enum ChallengeType {
    /// 敌人数量增加
    EnemyCountBoost { multiplier: f32 },
    /// 限时击杀
    TimeLimit { seconds: f32 },
    /// 无护盾
    NoShield,
}

impl ChallengeType {
    pub fn description(&self) -> String {
        match self {
            ChallengeType::EnemyCountBoost { multiplier } => {
                format!("敌人数量 +{}%", ((multiplier - 1.0) * 100.0).round() as u32)
            }
            ChallengeType::TimeLimit { seconds } => format!("限时击杀 {} 秒", *seconds as u32),
            ChallengeType::NoShield => "无护盾".to_string(),
        }
    }
}

/// 挑战状态
#[derive(Debug, Clone)]
pub struct ChallengeState {
    pub wave_in_zone: u32,
    pub modifiers: Vec<ChallengeType>,
    pub started_at: Option<f32>,
    pub penalty_gold_ratio: f32,
}

impl ChallengeState {
    /// 创建精英挑战（根据区域缩放难度和奖励）
    pub fn elite_offer(wave_in_zone: u32, zone: ZoneId) -> Self {
        // 区域缩放系数
        let zone_scale = match zone {
            ZoneId::Zone1 => 1.0,
            ZoneId::Zone2 => 1.2,
            ZoneId::Zone3 => 1.5,
        };

        // 时间限制随区域递减（更紧迫）
        let base_time: f32 = 35.0;
        let time_limit = (base_time / zone_scale).max(20.0);

        // 敌人倍率随区域递增
        let enemy_multiplier = 1.3 + (zone_scale - 1.0) * 0.5;

        Self {
            wave_in_zone,
            modifiers: vec![
                ChallengeType::EnemyCountBoost { multiplier: enemy_multiplier },
                ChallengeType::TimeLimit { seconds: time_limit },
                ChallengeType::NoShield,
            ],
            started_at: None,
            penalty_gold_ratio: 0.25,
        }
    }

    /// 旧版兼容（默认 Zone1）
    pub fn elite_offer_default(wave_in_zone: u32) -> Self {
        Self::elite_offer(wave_in_zone, ZoneId::Zone1)
    }

    pub fn start(&mut self, now: f32) {
        self.started_at = Some(now);
    }

    pub fn enemy_multiplier(&self) -> f32 {
        self.modifiers.iter().fold(1.0, |acc, m| {
            if let ChallengeType::EnemyCountBoost { multiplier } = m {
                acc * multiplier
            } else {
                acc
            }
        })
    }

    pub fn time_limit(&self) -> Option<f32> {
        self.modifiers.iter().find_map(|m| {
            if let ChallengeType::TimeLimit { seconds } = m {
                Some(*seconds)
            } else {
                None
            }
        })
    }

    pub fn no_shield(&self) -> bool {
        self.modifiers
            .iter()
            .any(|m| matches!(m, ChallengeType::NoShield))
    }

    pub fn time_remaining(&self, now: f32) -> Option<f32> {
        let limit = self.time_limit()?;
        let start = self.started_at.unwrap_or(now);
        Some((limit - (now - start)).max(0.0))
    }

    pub fn is_time_up(&self, now: f32) -> bool {
        self.time_remaining(now).map(|t| t <= 0.0).unwrap_or(false)
    }

    pub fn description_lines(&self) -> Vec<String> {
        self.modifiers.iter().map(|m| m.description()).collect()
    }
}

/// 战斗阶段状态
#[derive(Debug, Clone)]
pub struct CombatPhaseState {
    pub wave_in_zone: u32,
    pub enemies_remaining: u32,
    pub spawn_timer: f32,
    pub wave_start_time: f32,
    pub challenge: Option<ChallengeState>,
}

/// 奖励阶段状态
#[derive(Debug, Clone)]
pub struct RewardPhaseState {
    pub options: Vec<RewardOption>,
    pub selected: Option<usize>,
    pub timer: f32,
    /// 进入奖励阶段时的波次（用于后续商店退出判断）
    pub wave_at_enter: u32,
}

/// 奖励选项
#[derive(Debug, Clone)]
pub enum RewardOption {
    /// 卡牌奖励
    Card(Card), // 实际卡牌奖励
    /// 遗物奖励
    Relic(RelicId),
    /// 金币奖励
    Gold(u32),
    /// 生命恢复
    Heal(f32),
}

/// 商店阶段状态
#[derive(Debug, Clone)]
pub struct ShopPhaseState {
    pub items: Vec<ShopItem>,
    pub selected: Option<usize>,
    /// 进入商店时的波次（用于退出时判断是否进入 Boss）
    pub wave_at_enter: u32,
    /// 进入商店时的区域最大波次
    pub max_waves_at_enter: u32,
}

/// 商店物品
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub reward: RewardOption,
    pub price: u32,
    pub sold: bool,
}

/// 休息阶段状态
#[derive(Debug, Clone)]
pub struct RestPhaseState {
    pub options: Vec<RestOption>,
    pub selected: Option<usize>,
    pub card_selection: Option<Card>, // 当前选择的卡牌（用于升级/移除）
}

/// 休息选项
#[derive(Debug, Clone, Copy)]
pub enum RestOption {
    /// 恢复生命
    Heal,
    /// 升级卡牌
    UpgradeCard,
    /// 移除卡牌
    RemoveCard,
}

/// Run 阶段
#[derive(Debug, Clone)]
pub enum RunPhase {
    /// 战斗阶段
    Combat(CombatPhaseState),
    /// 挑战选择阶段
    ChallengeOffer(ChallengeState),
    /// 奖励选择阶段
    Reward(RewardPhaseState),
    /// 商店阶段
    Shop(ShopPhaseState),
    /// Boss 战阶段
    Boss(BossState),
    /// 休息阶段
    Rest(RestPhaseState),
    /// 区域过渡
    ZoneTransition {
        from: ZoneId,
        to: ZoneId,
        timer: f32,
    },
    /// 游戏胜利
    Victory,
    /// 游戏失败
    Defeat,
}

// ============================================================================
// Run 状态
// ============================================================================

/// 单次 Run 的完整状态
#[derive(Debug, Clone)]
pub struct RunState {
    /// 当前区域
    pub zone: ZoneId,
    /// 当前阶段
    pub phase: RunPhase,
    /// 已收集的遗物
    pub relics: HashSet<RelicId>,
    /// 金币
    pub gold: u32,
    /// 总击杀数
    pub total_kills: u32,
    /// 本次 Run 时长
    pub run_time: f32,
    /// 是否在 Boss 战中受伤（用于完美封印）
    pub boss_damage_taken: bool,
    /// 当前连击数
    pub combo: u32,
    /// 最高连击数
    pub max_combo: u32,
    /// 上次击杀时间（用于连击衰减）
    pub last_kill_time: f32,
}

impl Default for RunState {
    fn default() -> Self {
        Self::new()
    }
}

impl RunState {
    /// 创建新的 Run
    pub fn new() -> Self {
        Self {
            zone: ZoneId::Zone1,
            phase: RunPhase::Combat(CombatPhaseState {
                wave_in_zone: 1,
                enemies_remaining: 0,
                spawn_timer: 0.0,
                wave_start_time: 0.0,
                challenge: None,
            }),
            relics: HashSet::new(),
            gold: 0,
            total_kills: 0,
            run_time: 0.0,
            boss_damage_taken: false,
            combo: 0,
            max_combo: 0,
            last_kill_time: 0.0,
        }
    }

    /// 检查是否拥有某个遗物
    pub fn has_relic(&self, relic: RelicId) -> bool {
        self.relics.contains(&relic)
    }

    /// 添加遗物
    pub fn add_relic(&mut self, relic: RelicId) {
        self.relics.insert(relic);
    }

    /// 添加金币
    pub fn add_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    /// 获取当前波次的难度倍率
    pub fn current_difficulty(&self) -> f32 {
        if let RunPhase::Combat(ref state) = self.phase {
            let base = self.zone.difficulty_multiplier();
            let increment = self.zone.wave_difficulty_increment();
            base + increment * (state.wave_in_zone - 1) as f32
        } else {
            self.zone.difficulty_multiplier()
        }
    }

    /// 获取当前波次应生成的小行星数量
    pub fn current_asteroid_count(&self) -> usize {
        if let RunPhase::Combat(ref state) = self.phase {
            let base = self.zone.base_asteroid_count();
            let increment = self.zone.asteroid_increment();
            let base_count = base + increment * (state.wave_in_zone - 1) as usize;
            let multiplier = state
                .challenge
                .as_ref()
                .map(|c| c.enemy_multiplier())
                .unwrap_or(1.0);
            ((base_count as f32) * multiplier).round() as usize
        } else {
            self.zone.base_asteroid_count()
        }
    }

    /// 消费金币
    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// 记录击杀
    pub fn record_kill(&mut self) {
        self.total_kills += 1;
        self.combo += 1;
        self.last_kill_time = self.run_time;
        if self.combo > self.max_combo {
            self.max_combo = self.combo;
        }
    }

    /// 重置连击
    pub fn reset_combo(&mut self) {
        self.combo = 0;
    }

    /// 连击超时时间（秒）
    const COMBO_TIMEOUT: f32 = 5.0;

    /// 检查并应用连击衰减
    ///
    /// 如果距离上次击杀超过 COMBO_TIMEOUT 秒，连击重置为 0
    pub fn check_combo_decay(&mut self) {
        if self.combo > 0 && self.run_time - self.last_kill_time > Self::COMBO_TIMEOUT {
            self.combo = 0;
        }
    }

    /// 玩家受伤时调用（重置连击）
    ///
    /// 注意：Boss 战受伤标记需要单独设置 `boss_damage_taken = true`
    pub fn on_player_damage(&mut self) {
        self.reset_combo();
    }

    /// 标记 Boss 战中受伤（用于完美封印遗物判定）
    pub fn mark_boss_damage(&mut self) {
        self.boss_damage_taken = true;
        self.reset_combo();
    }

    /// 获取遗物加成后的奖励选项数量
    pub fn reward_option_count(&self) -> usize {
        let base = 3;
        if self.has_relic(RelicId::DraftingGloves) {
            base + 1
        } else {
            base
        }
    }

    /// 获取连击伤害加成
    pub fn combo_damage_bonus(&self) -> f32 {
        if self.has_relic(RelicId::ComboAmulet) {
            1.0 + (self.combo / 5) as f32 * 0.1
        } else {
            1.0
        }
    }

    /// 获取护盾持续时间倍率
    pub fn shield_duration_multiplier(&self) -> f32 {
        if self.has_relic(RelicId::ShieldBattery) {
            1.5
        } else {
            1.0
        }
    }

    /// 获取相位闪现伤害倍率
    pub fn phase_damage_multiplier(&self) -> f32 {
        if self.has_relic(RelicId::PhaseAmplifier) {
            1.3
        } else {
            1.0
        }
    }

    /// 获取暴击几率加成
    pub fn crit_chance_bonus(&self) -> f32 {
        if self.has_relic(RelicId::LuckyDice) {
            0.15
        } else {
            0.0
        }
    }

    /// 获取道具拾取范围倍率
    pub fn pickup_range_multiplier(&self) -> f32 {
        if self.has_relic(RelicId::MagneticCore) {
            1.5
        } else {
            1.0
        }
    }

    // === 新遗物效果查询 ===

    /// 检查是否有肾上腺素注射器（击杀后伤害加成）
    pub fn has_adrenaline(&self) -> bool {
        self.has_relic(RelicId::AdrenalineInjector)
    }

    /// 获取肾上腺素伤害加成（击杀后 2 秒内有效）
    pub fn adrenaline_damage_bonus(&self) -> f32 {
        if self.has_adrenaline() && self.run_time - self.last_kill_time < 2.0 {
            1.25
        } else {
            1.0
        }
    }

    /// 检查是否有纳米虫群（近距离伤害光环）
    pub fn has_nano_swarm(&self) -> bool {
        self.has_relic(RelicId::NanoSwarm)
    }

    /// 纳米虫群伤害范围
    pub fn nano_swarm_range(&self) -> f32 {
        if self.has_nano_swarm() { 80.0 } else { 0.0 }
    }

    /// 纳米虫群每秒伤害
    pub fn nano_swarm_dps(&self) -> f32 {
        if self.has_nano_swarm() { 5.0 } else { 0.0 }
    }

    /// 检查是否有赌徒筹码（商店价格波动）
    pub fn has_gamblers_chip(&self) -> bool {
        self.has_relic(RelicId::GamblersChip)
    }

    /// 获取商店价格倍率（±30%）
    pub fn shop_price_multiplier(&self) -> f32 {
        if self.has_gamblers_chip() {
            rand::gen_range(0.7, 1.3)
        } else {
            1.0
        }
    }

    /// 检查是否有虚空锚点（相位闪现留下伤害区域）
    pub fn has_void_anchor(&self) -> bool {
        self.has_relic(RelicId::VoidAnchor)
    }

    /// 检查是否有链式反应堆（连锁离子炮额外跳跃）
    pub fn has_chain_reactor(&self) -> bool {
        self.has_relic(RelicId::ChainReactor)
    }

    /// 获取连锁离子炮额外跳跃次数
    pub fn chain_extra_jumps(&self) -> u32 {
        if self.has_chain_reactor() { 1 } else { 0 }
    }

    /// 检查是否有时间膨胀器（击杀触发慢动作）
    pub fn has_time_dilator(&self) -> bool {
        self.has_relic(RelicId::TimeDilator)
    }

    /// 检查击杀时是否触发慢动作（20% 几率）
    pub fn should_trigger_slow_motion(&self) -> bool {
        self.has_time_dilator() && rand::gen_range(0.0f32, 1.0) < 0.2
    }

    // ========== 腐化遗物效果 ==========

    /// 检查是否有重型枪管（伤害+200%，后坐力+100%，射速-30%）
    pub fn has_heavy_barrel(&self) -> bool {
        self.has_relic(RelicId::HeavyBarrel)
    }

    /// 获取重型枪管伤害倍率
    pub fn heavy_barrel_damage_mult(&self) -> f32 {
        if self.has_heavy_barrel() { 3.0 } else { 1.0 }
    }

    /// 获取重型枪管后坐力倍率
    pub fn heavy_barrel_recoil_mult(&self) -> f32 {
        if self.has_heavy_barrel() { 2.0 } else { 1.0 }
    }

    /// 获取重型枪管射速倍率（冷却时间倍率）
    pub fn heavy_barrel_cooldown_mult(&self) -> f32 {
        if self.has_heavy_barrel() { 1.3 } else { 1.0 }
    }

    /// 检查是否有玻璃大炮（伤害+500%，生命上限=1）
    pub fn has_glass_cannon(&self) -> bool {
        self.has_relic(RelicId::GlassCannon)
    }

    /// 获取玻璃大炮伤害倍率
    pub fn glass_cannon_damage_mult(&self) -> f32 {
        if self.has_glass_cannon() { 6.0 } else { 1.0 }
    }

    /// 检查是否有虫洞引擎（Hyperspace无风险，但每次-50分）
    pub fn has_wormhole_engine(&self) -> bool {
        self.has_relic(RelicId::WormholeEngine)
    }

    /// 检查是否有吸血弹药（击杀回血，但无法拾取道具）
    pub fn has_vampire_ammo(&self) -> bool {
        self.has_relic(RelicId::VampireAmmo)
    }

    /// 检查是否有狂战士之心（低血量伤害翻倍，但无法回血）
    pub fn has_berserker_heart(&self) -> bool {
        self.has_relic(RelicId::BerserkerHeart)
    }

    /// 获取狂战士之心伤害倍率（需要传入当前生命值和最大生命值）
    pub fn berserker_damage_mult(&self, current_lives: u32, max_lives: u32) -> f32 {
        if self.has_berserker_heart() && current_lives == 1 && max_lives > 1 {
            2.0
        } else {
            1.0
        }
    }

    /// 检查是否有贪婪契约（金币+100%，但受伤损失金币）
    pub fn has_greed_pact(&self) -> bool {
        self.has_relic(RelicId::GreedPact)
    }

    /// 获取贪婪契约金币倍率
    pub fn greed_pact_gold_mult(&self) -> f32 {
        if self.has_greed_pact() { 2.0 } else { 1.0 }
    }

    /// 贪婪契约受伤惩罚（损失 20% 金币）
    pub fn apply_greed_pact_penalty(&mut self) {
        if self.has_greed_pact() {
            let penalty = (self.gold as f32 * 0.2).round() as u32;
            self.gold = self.gold.saturating_sub(penalty);
        }
    }

    /// 获取总伤害倍率（综合所有腐化遗物）
    pub fn total_damage_multiplier(&self, current_lives: u32, max_lives: u32) -> f32 {
        let mut mult = 1.0;
        mult *= self.heavy_barrel_damage_mult();
        mult *= self.glass_cannon_damage_mult();
        mult *= self.berserker_damage_mult(current_lives, max_lives);
        mult
    }

    /// 触发遗物效果（波次清空）
    pub fn trigger_wave_clear(&mut self) {
        if self.has_relic(RelicId::PaycheckChip) {
            self.add_gold(10);
        }
    }

    /// 触发遗物效果（Boss 击败）
    pub fn trigger_boss_defeat(&mut self) {
        if self.has_relic(RelicId::FlawlessSeal) && !self.boss_damage_taken {
            self.add_gold(50);
        }
        self.boss_damage_taken = false;
    }

    /// 触发遗物效果（拾取道具）
    pub fn trigger_pickup(&mut self) {
        if self.has_relic(RelicId::SalvageMagnet) && rand::gen_range(0.0, 1.0) < 0.3 {
            self.add_gold(5);
        }
    }

    /// 进入挑战选择阶段
    pub fn enter_challenge_offer(&mut self) {
        if let RunPhase::Combat(ref state) = self.phase {
            self.phase = RunPhase::ChallengeOffer(ChallengeState::elite_offer(state.wave_in_zone, self.zone));
        }
    }

    /// 接受挑战并开始挑战波
    pub fn start_challenge(&mut self, now: f32) -> bool {
        if let RunPhase::ChallengeOffer(mut offer) = std::mem::replace(
            &mut self.phase,
            RunPhase::Combat(CombatPhaseState {
                wave_in_zone: 1,
                enemies_remaining: 0,
                spawn_timer: 0.0,
                wave_start_time: 0.0,
                challenge: None,
            }),
        ) {
            offer.start(now);
            self.phase = RunPhase::Combat(CombatPhaseState {
                wave_in_zone: offer.wave_in_zone,
                enemies_remaining: 0,
                spawn_timer: 0.0,
                wave_start_time: now,
                challenge: Some(offer),
            });
            true
        } else {
            false
        }
    }

    /// 当前挑战（仅在 Combat 阶段有效）
    pub fn active_challenge(&self) -> Option<&ChallengeState> {
        if let RunPhase::Combat(ref state) = self.phase {
            state.challenge.as_ref()
        } else {
            None
        }
    }

    /// 取出当前挑战
    pub fn take_active_challenge(&mut self) -> Option<ChallengeState> {
        if let RunPhase::Combat(ref mut state) = self.phase {
            state.challenge.take()
        } else {
            None
        }
    }

    /// 挑战是否禁用护盾
    pub fn challenge_disables_shield(&self) -> bool {
        self.active_challenge().map(|c| c.no_shield()).unwrap_or(false)
    }

    /// 挑战剩余时间
    pub fn challenge_time_remaining(&self) -> Option<f32> {
        self.active_challenge()
            .and_then(|c| c.time_remaining(self.run_time))
    }

    /// 挑战失败惩罚：损失部分金币
    pub fn apply_challenge_failure_penalty(&mut self, challenge: &ChallengeState) {
        let penalty = (self.gold as f32 * challenge.penalty_gold_ratio).round() as u32;
        self.gold = self.gold.saturating_sub(penalty);
    }

    /// 进入下一波
    pub fn advance_wave(&mut self) {
        if let RunPhase::Combat(ref mut state) = self.phase {
            let max_waves = self.zone.wave_count();
            if state.wave_in_zone < max_waves {
                state.wave_in_zone += 1;
                state.spawn_timer = 0.0;
                state.wave_start_time = 0.0;
                state.challenge = None;
            } else {
                // 进入 Boss 战
                let boss_kind = match self.zone {
                    ZoneId::Zone1 => BossKind::GiantSplitter,
                    ZoneId::Zone2 => BossKind::UfoMothership,
                    ZoneId::Zone3 => BossKind::WormholeWarden,
                };
                self.phase = RunPhase::Boss(BossState::new(boss_kind));
            }
        }
    }

    /// 进入奖励阶段
    pub fn enter_reward_phase(&mut self, options: Vec<RewardOption>) {
        // 保存当前波次信息
        let wave_at_enter = if let RunPhase::Combat(ref cs) = self.phase {
            cs.wave_in_zone
        } else {
            1
        };

        self.phase = RunPhase::Reward(RewardPhaseState {
            options,
            selected: None,
            timer: 0.0,
            wave_at_enter,
        });
    }

    /// 进入商店阶段
    pub fn enter_shop_phase(&mut self, items: Vec<ShopItem>) {
        // 保存当前波次信息（用于退出商店时判断）
        let wave_at_enter = if let RunPhase::Combat(ref cs) = self.phase {
            cs.wave_in_zone
        } else if let RunPhase::Reward(ref rs) = self.phase {
            // 从奖励阶段进入商店时，使用奖励阶段保存的波次
            rs.wave_at_enter
        } else {
            1
        };
        let max_waves_at_enter = self.zone.wave_count();

        self.phase = RunPhase::Shop(ShopPhaseState {
            items,
            selected: None,
            wave_at_enter,
            max_waves_at_enter,
        });
    }

/// 进入休息阶段
    pub fn enter_rest_phase(&mut self, options: Vec<RestOption>) {
        self.phase = RunPhase::Rest(RestPhaseState {
            options,
            selected: None,
            card_selection: None,
        });
    }

    /// 进入下一区域
    pub fn advance_zone(&mut self) {
        if let Some(next_zone) = self.zone.next() {
            self.phase = RunPhase::ZoneTransition {
                from: self.zone,
                to: next_zone,
                timer: 3.0,
            };
        } else {
            self.phase = RunPhase::Victory;
        }
    }

    /// 完成区域过渡
    pub fn complete_zone_transition(&mut self, new_zone: ZoneId) {
        self.zone = new_zone;
        self.phase = RunPhase::Combat(CombatPhaseState {
            wave_in_zone: 1,
            enemies_remaining: 0,
            spawn_timer: 0.0,
            wave_start_time: 0.0,
            challenge: None,
        });
    }

    /// 游戏失败
    pub fn defeat(&mut self) {
        self.phase = RunPhase::Defeat;
    }
}

// ============================================================================
// UI 渲染辅助
// ============================================================================

/// 绘制 Run 状态 HUD
pub fn draw_run_hud(run: &RunState, font: Option<&Font>) {
    let hud_y = 10.0;

    // 区域信息
    draw_text_ex(
        &format!("区域: {}", run.zone.name()),
        10.0,
        hud_y + 20.0,
        TextParams {
            font,
            font_size: 24,
            color: WHITE,
            ..Default::default()
        },
    );

    // 波次信息（仅战斗阶段）
    if let RunPhase::Combat(ref state) = run.phase {
        draw_text_ex(
            &format!("波次: {}/{}", state.wave_in_zone, run.zone.wave_count()),
            10.0,
            hud_y + 45.0,
            TextParams {
                font,
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );

        // 挑战提示
        if let Some(challenge) = &state.challenge {
            if let Some(remaining) = challenge.time_remaining(run.run_time) {
                draw_text_ex(
                    &format!("挑战剩余: {:.1}s", remaining),
                    10.0,
                    hud_y + 120.0,
                    TextParams {
                        font,
                        font_size: 18,
                        color: ORANGE,
                        ..Default::default()
                    },
                );
            }
            if challenge.no_shield() {
                draw_text_ex(
                    "挑战：无护盾",
                    10.0,
                    hud_y + 140.0,
                    TextParams {
                        font,
                        font_size: 18,
                        color: RED,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // 金币
    draw_text_ex(
        &format!("金币: {}", run.gold),
        10.0,
        hud_y + 70.0,
        TextParams {
            font,
            font_size: 20,
            color: GOLD,
            ..Default::default()
        },
    );

    // 连击数
    if run.combo > 0 {
        let combo_color = if run.combo >= 10 {
            RED
        } else if run.combo >= 5 {
            ORANGE
        } else {
            YELLOW
        };
        draw_text_ex(
            &format!("连击: {}x", run.combo),
            10.0,
            hud_y + 95.0,
            TextParams {
                font,
                font_size: 20,
                color: combo_color,
                ..Default::default()
            },
        );
    }

    // 遗物图标（右上角）
    let relic_start_x = screen_width() - 40.0;
    let mouse_pos = mouse_position();
    let mut hovered_relic: Option<&RelicId> = None;
    let mut hovered_pos = (0.0f32, 0.0f32);

    for (i, relic) in run.relics.iter().enumerate() {
        let x = relic_start_x - (i as f32 * 35.0);
        let icon_rect = (x, hud_y, 30.0, 30.0);

        // 检测鼠标悬停
        if mouse_pos.0 >= icon_rect.0
            && mouse_pos.0 <= icon_rect.0 + icon_rect.2
            && mouse_pos.1 >= icon_rect.1
            && mouse_pos.1 <= icon_rect.1 + icon_rect.3
        {
            hovered_relic = Some(relic);
            hovered_pos = (x, hud_y + 35.0);
            // 悬停时高亮边框
            draw_rectangle_lines(x - 2.0, hud_y - 2.0, 34.0, 34.0, 2.0, WHITE);
        }

        draw_rectangle(x, hud_y, 30.0, 30.0, relic.rarity_color());
        // 简化显示：用首字母
        let initial = relic.name().chars().next().unwrap_or('?');
        draw_text_ex(
            &initial.to_string(),
            x + 8.0,
            hud_y + 22.0,
            TextParams {
                font,
                font_size: 20,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    // 绘制悬停提示框
    if let Some(relic) = hovered_relic {
        draw_relic_tooltip(relic, hovered_pos.0, hovered_pos.1, font);
    }
}

/// 绘制遗物提示框
fn draw_relic_tooltip(relic: &RelicId, x: f32, y: f32, font: Option<&Font>) {
    let name = relic.name();
    let desc = relic.description();

    // 计算提示框尺寸
    let name_width = measure_text(name, None, 18, 1.0).width;
    let desc_width = measure_text(desc, None, 14, 1.0).width;
    let box_width = name_width.max(desc_width) + 20.0;
    let box_height = 50.0;

    // 确保提示框不超出屏幕
    let tooltip_x = (x - box_width + 30.0).max(10.0);
    let tooltip_y = y;

    // 背景
    draw_rectangle(
        tooltip_x,
        tooltip_y,
        box_width,
        box_height,
        Color::new(0.1, 0.1, 0.15, 0.95),
    );
    // 边框
    draw_rectangle_lines(
        tooltip_x,
        tooltip_y,
        box_width,
        box_height,
        2.0,
        relic.rarity_color(),
    );

    // 名称
    draw_text_ex(
        name,
        tooltip_x + 10.0,
        tooltip_y + 20.0,
        TextParams {
            font,
            font_size: 18,
            color: relic.rarity_color(),
            ..Default::default()
        },
    );

    // 描述
    draw_text_ex(
        desc,
        tooltip_x + 10.0,
        tooltip_y + 40.0,
        TextParams {
            font,
            font_size: 14,
            color: LIGHTGRAY,
            ..Default::default()
        },
    );
}

/// 绘制 Boss 血条
pub fn draw_boss_health_bar(boss: &BossState) {
    let bar_width = 400.0;
    let bar_height = 20.0;
    let bar_x = (screen_width() - bar_width) / 2.0;
    let bar_y = 50.0;

    // 背景
    draw_rectangle(
        bar_x - 2.0,
        bar_y - 2.0,
        bar_width + 4.0,
        bar_height + 4.0,
        DARKGRAY,
    );

    // 血条
    let health_width = bar_width * boss.health_percent();
    let health_color = if boss.is_desperate {
        Color::new(1.0, 0.0, 0.5, 1.0) // 绝望阶段：粉红色
    } else if boss.is_enraged {
        RED
    } else {
        Color::new(0.8, 0.2, 0.2, 1.0)
    };
    draw_rectangle(bar_x, bar_y, health_width, bar_height, health_color);

    // Boss 名称
    let name = boss.kind.name();
    let text_width = measure_text(name, None, 24, 1.0).width;
    let name_color = if boss.is_desperate {
        Color::new(1.0, 0.0, 0.5, 1.0)
    } else if boss.is_enraged {
        RED
    } else {
        WHITE
    };
    draw_text(
        name,
        (screen_width() - text_width) / 2.0,
        bar_y - 10.0,
        24.0,
        name_color,
    );

    // 状态标记
    if boss.is_desperate {
        draw_text("绝望!", bar_x + bar_width + 10.0, bar_y + 15.0, 18.0, Color::new(1.0, 0.0, 0.5, 1.0));
    } else if boss.is_enraged {
        draw_text("狂暴!", bar_x + bar_width + 10.0, bar_y + 15.0, 18.0, RED);
    }
}

/// 绘制区域过渡动画
pub fn draw_zone_transition(_from: ZoneId, to: ZoneId, timer: f32) {
    let alpha = (timer / 3.0).min(1.0);

    // 渐变背景
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, alpha),
    );

    // 区域名称
    let text = format!("进入 {}", to.name());
    let text_size = 48.0;
    let text_width = measure_text(&text, None, text_size as u16, 1.0).width;
    draw_text(
        &text,
        (screen_width() - text_width) / 2.0,
        screen_height() / 2.0,
        text_size,
        Color::new(1.0, 1.0, 1.0, alpha),
    );

    // 区域描述
    let desc = match to {
        ZoneId::Zone1 => "小心分裂的小行星...",
        ZoneId::Zone2 => "UFO 正在巡逻...",
        ZoneId::Zone3 => "虫洞的力量在扭曲空间...",
    };
    let desc_width = measure_text(desc, None, 24, 1.0).width;
    draw_text(
        desc,
        (screen_width() - desc_width) / 2.0,
        screen_height() / 2.0 + 40.0,
        24.0,
        Color::new(0.7, 0.7, 0.7, alpha),
    );
}

// ============================================================================
// 奖励/商店生成与结算
// ============================================================================

/// 商店刷新费用
pub const SHOP_REFRESH_COST: u32 = 5;

/// 所有遗物列表（用于随机生成）
const ALL_RELICS: [RelicId; 16] = [
    RelicId::PaycheckChip,
    RelicId::DraftingGloves,
    RelicId::FlawlessSeal,
    RelicId::SalvageMagnet,
    RelicId::CollectorLedger,
    RelicId::ComboAmulet,
    RelicId::ShieldBattery,
    RelicId::PhaseAmplifier,
    RelicId::LuckyDice,
    RelicId::MagneticCore,
    // 新增遗物
    RelicId::AdrenalineInjector,
    RelicId::NanoSwarm,
    RelicId::GamblersChip,
    RelicId::VoidAnchor,
    RelicId::ChainReactor,
    RelicId::TimeDilator,
];

/// 挑战奖励遗物池（强力遗物）
const CHALLENGE_RELICS: [RelicId; 8] = [
    RelicId::PhaseAmplifier,
    RelicId::LuckyDice,
    RelicId::MagneticCore,
    RelicId::ShieldBattery,
    RelicId::ComboAmulet,
    // 新增强力遗物
    RelicId::NanoSwarm,
    RelicId::VoidAnchor,
    RelicId::ChainReactor,
];

/// 挑战成功金币奖励
const CHALLENGE_GOLD_REWARD: u32 = 40;

/// 腐化遗物池（高风险高回报）
const CORRUPTED_RELICS: [RelicId; 6] = [
    RelicId::HeavyBarrel,
    RelicId::GlassCannon,
    RelicId::WormholeEngine,
    RelicId::VampireAmmo,
    RelicId::BerserkerHeart,
    RelicId::GreedPact,
];

/// 生成奖励选项
pub fn generate_reward_options(run: &RunState) -> Vec<RewardOption> {
    let mut options: Vec<RewardOption> = Vec::new();
    let target = run.reward_option_count().max(3);

    while options.len() < target {
        let roll: f32 = rand::gen_range(0.0, 1.0);
        let option = if roll < 0.25 {
            // 金币奖励
            RewardOption::Gold(rand::gen_range(10u32, 26u32))
        } else if roll < 0.45 {
            // 生命恢复
            RewardOption::Heal(1.0)
        } else if roll < 0.75 {
            // 遗物：尽量避免已拥有
            let mut relic = ALL_RELICS[rand::gen_range(0usize, ALL_RELICS.len())];
            for _ in 0..8 {
                if !run.has_relic(relic) {
                    break;
                }
                relic = ALL_RELICS[rand::gen_range(0usize, ALL_RELICS.len())];
            }
            RewardOption::Relic(relic)
        } else {
            // 生成随机卡牌奖励
            let cards = generate_draft_options();
            let card = cards[rand::gen_range(0, cards.len())];
            RewardOption::Card(card)
        };

        // 去重：遗物不重复
        let is_dup_relic = matches!(&option, RewardOption::Relic(r) if
            options.iter().any(|o| matches!(o, RewardOption::Relic(rr) if rr == r)));
        if !is_dup_relic {
            options.push(option);
        }
    }

    options
}

/// 生成挑战成功奖励（稀有卡牌/强遗物/双倍金币）
pub fn generate_challenge_reward_options(run: &RunState) -> Vec<RewardOption> {
    use crate::battle_draft::CardRarity;

    // 稀有或史诗卡牌
    let cards = generate_draft_options();
    let rare_card = cards
        .iter()
        .find(|c| matches!(c.rarity(), CardRarity::Rare | CardRarity::Epic))
        .copied()
        .unwrap_or(cards[0]);

    // 强遗物（尽量避免已拥有）
    let mut relic = CHALLENGE_RELICS[rand::gen_range(0usize, CHALLENGE_RELICS.len())];
    for _ in 0..8 {
        if !run.has_relic(relic) {
            break;
        }
        relic = CHALLENGE_RELICS[rand::gen_range(0usize, CHALLENGE_RELICS.len())];
    }

    vec![
        RewardOption::Card(rare_card),
        RewardOption::Relic(relic),
        RewardOption::Gold(CHALLENGE_GOLD_REWARD),
    ]
}

/// 生成商店物品
pub fn generate_shop_items(run: &RunState) -> Vec<ShopItem> {
    let count = rand::gen_range(4usize, 7usize); // 4-6 个商品
    let mut items: Vec<ShopItem> = Vec::with_capacity(count);

    while items.len() < count {
        let roll: f32 = rand::gen_range(0.0, 1.0);
        let (reward, price) = if roll < 0.2 {
            // 金币（低价）
            (RewardOption::Gold(rand::gen_range(12u32, 26u32)), 8)
        } else if roll < 0.4 {
            // 生命恢复
            (RewardOption::Heal(1.0), 18)
        } else if roll < 0.75 {
            // 遗物
            let mut relic = ALL_RELICS[rand::gen_range(0usize, ALL_RELICS.len())];
            for _ in 0..10 {
                let already_owned = run.has_relic(relic);
                let already_in_shop = items
                    .iter()
                    .any(|it| matches!(&it.reward, RewardOption::Relic(r) if *r == relic));
                if !already_owned && !already_in_shop {
                    break;
                }
                relic = ALL_RELICS[rand::gen_range(0usize, ALL_RELICS.len())];
            }
            (RewardOption::Relic(relic), 45)
        } else {
            // 随机卡牌
            let cards = generate_draft_options();
            let card = cards[rand::gen_range(0, cards.len())];
            (RewardOption::Card(card), 30)
        };

        items.push(ShopItem {
            reward,
            price,
            sold: false,
        });
    }

    // Zone 2+ 时 20% 几率添加腐化遗物
    if !matches!(run.zone, ZoneId::Zone1) && rand::gen_range(0.0f32, 1.0) < 0.2 {
        let mut relic = CORRUPTED_RELICS[rand::gen_range(0usize, CORRUPTED_RELICS.len())];
        for _ in 0..6 {
            if !run.has_relic(relic) {
                break;
            }
            relic = CORRUPTED_RELICS[rand::gen_range(0usize, CORRUPTED_RELICS.len())];
        }
        if !run.has_relic(relic) {
            items.push(ShopItem {
                reward: RewardOption::Relic(relic),
                price: 60, // 腐化遗物价格更高
                sold: false,
            });
        }
    }

    items
}

/// 应用奖励选项
pub fn apply_reward_option(run: &mut RunState, players: &mut [Player], reward: &RewardOption) {
    match reward {
        RewardOption::Gold(amount) => run.add_gold(*amount),
        RewardOption::Relic(relic) => run.add_relic(*relic),
        RewardOption::Card(card) => {
            // 为所有玩家应用卡牌效果
            for player in players.iter_mut() {
                player.apply_draft_card(*card);
            }
        }
        RewardOption::Heal(amount) => {
            let add_lives = (*amount).round().max(1.0) as u32;
            for p in players.iter_mut() {
                p.lives += add_lives;
            }
        }
    }
}

/// 应用休息选项
pub fn apply_rest_option(run: &mut RunState, players: &mut [Player], option: RestOption, selected_card: Option<Card>) -> bool {
    match option {
        RestOption::Heal => {
            // 恢复生命（递减效果）
            for player in players.iter_mut() {
                if player.lives < 3 {
                    player.lives += 1;
                }
            }
            true
        }
        RestOption::UpgradeCard => {
            // 升级选中的卡牌
            if let Some(card) = selected_card {
                for player in players.iter_mut() {
                    if player.upgrade_card(card) {
                        return true;
                    }
                }
            }
            false
        }
        RestOption::RemoveCard => {
            // 移除选中的卡牌并获得奖励
            if let Some(card) = selected_card {
                for player in players.iter_mut() {
                    if player.remove_card(card) {
                        // 移除卡牌的奖励：获得金币
                        run.add_gold(25);
                        return true;
                    }
                }
            }
            false
        }
    }
}

/// 生成休息选项（根据玩家卡牌情况动态调整）
pub fn generate_rest_options(players: &[Player]) -> Vec<RestOption> {
    let mut options = vec![RestOption::Heal]; // 始终提供治疗选项

    // 检查是否有玩家拥有卡牌
    let has_cards = players.iter().any(|p| !p.cards.is_empty());
    
    if has_cards {
        options.push(RestOption::UpgradeCard);
        options.push(RestOption::RemoveCard);
    }

    options
}

/// 购买商店物品（直接操作 RunState）
pub fn buy_shop_item(run: &mut RunState, players: &mut [Player], idx: usize) -> bool {
    // 先获取商品信息
    let (price, reward) = if let RunPhase::Shop(ref shop) = run.phase {
        if let Some(item) = shop.items.get(idx) {
            if item.sold {
                return false;
            }
            (item.price, item.reward.clone())
        } else {
            return false;
        }
    } else {
        return false;
    };

    // 检查并扣除金币
    if !run.spend_gold(price) {
        return false;
    }

    // 应用奖励
    apply_reward_option(run, players, &reward);

    // 标记为已售
    if let RunPhase::Shop(ref mut shop) = run.phase
        && let Some(item) = shop.items.get_mut(idx)
    {
        item.sold = true;
    }
    true
}

/// 刷新商店（直接操作 RunState）
pub fn refresh_shop(run: &mut RunState) -> bool {
    if !run.spend_gold(SHOP_REFRESH_COST) {
        return false;
    }
    let new_items = generate_shop_items(run);
    if let RunPhase::Shop(ref mut shop) = run.phase {
        shop.items = new_items;
        shop.selected = None;
    }
    true
}

/// 获取奖励选项的显示信息
pub fn reward_display_info(option: &RewardOption) -> (&'static str, String, String, Color) {
    match option {
        RewardOption::Relic(relic) => {
            let kind = if relic.is_corrupted() { "⚠腐化" } else { "遗物" };
            (
                kind,
                relic.name().to_string(),
                relic.description().to_string(),
                relic.rarity_color(),
            )
        }
        RewardOption::Card(card) => (
            "卡牌",
            card.name().to_string(),
            card.description().to_string(),
            card.rarity().border_color(),
        ),
        RewardOption::Gold(amount) => (
            "金币",
            format!("+{} 金币", amount),
            "立刻获得金币".to_string(),
            GOLD,
        ),
        RewardOption::Heal(amount) => (
            "修复",
            format!("+{} 生命", (*amount).round().max(1.0) as u32),
            "恢复生命值".to_string(),
            Color::new(0.2, 0.9, 0.5, 1.0),
        ),
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_state_creation() {
        let run = RunState::new();
        assert_eq!(run.zone, ZoneId::Zone1);
        assert_eq!(run.gold, 0);
        assert!(run.relics.is_empty());
    }

    #[test]
    fn test_relic_effects() {
        let mut run = RunState::new();

        // 测试薪水芯片
        run.add_relic(RelicId::PaycheckChip);
        run.trigger_wave_clear();
        assert_eq!(run.gold, 10);

        // 测试抽卡手套
        run.add_relic(RelicId::DraftingGloves);
        assert_eq!(run.reward_option_count(), 4);
    }

    #[test]
    fn test_combo_system() {
        let mut run = RunState::new();
        run.add_relic(RelicId::ComboAmulet);

        for _ in 0..10 {
            run.record_kill();
        }
        assert_eq!(run.combo, 10);
        assert_eq!(run.max_combo, 10);

        // 10 连击 = 20% 伤害加成
        assert!((run.combo_damage_bonus() - 1.2).abs() < 0.01);

        run.reset_combo();
        assert_eq!(run.combo, 0);
        assert_eq!(run.max_combo, 10); // 最高连击保留
    }

    #[test]
    fn test_zone_progression() {
        assert_eq!(ZoneId::Zone1.next(), Some(ZoneId::Zone2));
        assert_eq!(ZoneId::Zone2.next(), Some(ZoneId::Zone3));
        assert_eq!(ZoneId::Zone3.next(), None);
    }

    #[test]
    fn test_boss_enrage() {
        let mut boss = BossState::new(BossKind::GiantSplitter);
        assert!(!boss.is_enraged);

        boss.health = boss.max_health * 0.25;
        boss.check_enrage();
        assert!(boss.is_enraged);
    }

    #[test]
    fn test_zone_wave_count() {
        assert_eq!(ZoneId::Zone1.wave_count(), 3);
        assert_eq!(ZoneId::Zone2.wave_count(), 4);
        assert_eq!(ZoneId::Zone3.wave_count(), 5);
    }

    #[test]
    fn test_zone_difficulty_multiplier() {
        assert!((ZoneId::Zone1.difficulty_multiplier() - 0.6).abs() < 0.01);
        assert!((ZoneId::Zone2.difficulty_multiplier() - 1.0).abs() < 0.01);
        assert!((ZoneId::Zone3.difficulty_multiplier() - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_zone_name() {
        assert_eq!(ZoneId::Zone1.name(), "小行星带");
        assert_eq!(ZoneId::Zone2.name(), "UFO 领域");
        assert_eq!(ZoneId::Zone3.name(), "虫洞深渊");
    }

    #[test]
    fn test_run_state_add_gold() {
        let mut run = RunState::new();
        assert_eq!(run.gold, 0);
        run.add_gold(50);
        assert_eq!(run.gold, 50);
        run.add_gold(30);
        assert_eq!(run.gold, 80);
    }

    #[test]
    fn test_run_state_add_relic() {
        let mut run = RunState::new();
        assert!(!run.has_relic(RelicId::PaycheckChip));
        run.add_relic(RelicId::PaycheckChip);
        assert!(run.has_relic(RelicId::PaycheckChip));
        // 重复添加不会报错
        run.add_relic(RelicId::PaycheckChip);
        assert!(run.has_relic(RelicId::PaycheckChip));
    }

    #[test]
    fn test_run_state_defeat() {
        let mut run = RunState::new();
        assert!(!matches!(run.phase, RunPhase::Defeat));
        run.defeat();
        assert!(matches!(run.phase, RunPhase::Defeat));
    }

    #[test]
    fn test_advance_zone_to_victory() {
        let mut run = RunState::new();
        run.zone = ZoneId::Zone3;
        run.advance_zone();
        assert!(matches!(run.phase, RunPhase::Victory));
    }

    #[test]
    fn test_advance_zone_transition() {
        let mut run = RunState::new();
        run.zone = ZoneId::Zone1;
        run.advance_zone();
        assert!(matches!(run.phase, RunPhase::ZoneTransition { from: ZoneId::Zone1, to: ZoneId::Zone2, .. }));
    }

    #[test]
    fn test_complete_zone_transition() {
        let mut run = RunState::new();
        run.complete_zone_transition(ZoneId::Zone2);
        assert_eq!(run.zone, ZoneId::Zone2);
        assert!(matches!(run.phase, RunPhase::Combat(_)));
    }

    #[test]
    fn test_relic_is_corrupted() {
        assert!(RelicId::HeavyBarrel.is_corrupted());
        assert!(RelicId::GlassCannon.is_corrupted());
        assert!(!RelicId::PaycheckChip.is_corrupted());
        assert!(!RelicId::DraftingGloves.is_corrupted());
    }

    #[test]
    fn test_boss_state_creation() {
        let boss = BossState::new(BossKind::GiantSplitter);
        assert!(boss.health > 0.0);
        assert!(!boss.is_enraged);
        assert!(boss.phase >= 0); // phase starts at 1
    }

    #[test]
    fn test_challenge_state_elite_offer() {
        let challenge = ChallengeState::elite_offer(2, ZoneId::Zone1);
        assert_eq!(challenge.wave_in_zone, 2);
        assert!(!challenge.modifiers.is_empty());
    }

    #[test]
    fn test_challenge_no_shield() {
        let challenge = ChallengeState::elite_offer(1, ZoneId::Zone1);
        assert!(challenge.no_shield());
    }
}
