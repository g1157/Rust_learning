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
use crate::player::Player;
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
            ZoneId::Zone1 => 0.6,  // 简单开局
            ZoneId::Zone2 => 1.0,  // 正常难度
            ZoneId::Zone3 => 1.5,  // 困难
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
            ZoneId::Zone1 => 5,   // 少量小行星
            ZoneId::Zone2 => 8,   // 中等数量
            ZoneId::Zone3 => 12,  // 大量小行星
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

/// 获取 Boss 碰撞半径
pub fn boss_radius(boss: &BossState) -> f32 {
    match boss.kind {
        BossKind::GiantSplitter => GIANT_SPLITTER_RADIUS,
        _ => 80.0,
    }
}

/// Boss 行为更新入口
pub fn update_boss(boss: &mut BossState, players: &[Player], asteroids: &mut Vec<Asteroid>, dt: f32) {
    if boss.kind == BossKind::GiantSplitter { update_giant_splitter(boss, players, asteroids, dt) }
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
    if boss.phase_timer >= summon_interval && asteroids.len() < GIANT_SPLITTER_MAX_SUMMONED_ASTEROIDS
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
        }
    }

    /// 获取遗物稀有度颜色
    pub fn rarity_color(&self) -> Color {
        match self {
            RelicId::PaycheckChip | RelicId::SalvageMagnet | RelicId::MagneticCore => {
                Color::new(0.6, 0.6, 0.6, 1.0) // 普通 - 灰色
            }
            RelicId::DraftingGloves | RelicId::ComboAmulet | RelicId::ShieldBattery => {
                Color::new(0.2, 0.6, 1.0, 1.0) // 稀有 - 蓝色
            }
            RelicId::FlawlessSeal | RelicId::PhaseAmplifier | RelicId::LuckyDice => {
                Color::new(0.8, 0.4, 1.0, 1.0) // 史诗 - 紫色
            }
            RelicId::CollectorLedger => {
                Color::new(1.0, 0.8, 0.2, 1.0) // 传说 - 金色
            }
        }
    }
}

// ============================================================================
// Run 阶段状态机
// ============================================================================

/// 战斗阶段状态
#[derive(Debug, Clone)]
pub struct CombatPhaseState {
    pub wave_in_zone: u32,
    pub enemies_remaining: u32,
    pub spawn_timer: f32,
    pub wave_start_time: f32,
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
    Card(String), // 暂用 String，后续集成 battle_draft::Card
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
    /// 奖励选择阶段
    Reward(RewardPhaseState),
    /// 商店阶段
    Shop(ShopPhaseState),
    /// Boss 战阶段
    Boss(BossState),
    /// 休息阶段
    Rest(RestPhaseState),
    /// 区域过渡
    ZoneTransition { from: ZoneId, to: ZoneId, timer: f32 },
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
            }),
            relics: HashSet::new(),
            gold: 0,
            total_kills: 0,
            run_time: 0.0,
            boss_damage_taken: false,
            combo: 0,
            max_combo: 0,
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
            base + increment * (state.wave_in_zone - 1) as usize
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
        if self.combo > self.max_combo {
            self.max_combo = self.combo;
        }
    }

    /// 重置连击
    pub fn reset_combo(&mut self) {
        self.combo = 0;
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
        if self.has_relic(RelicId::SalvageMagnet)
            && rand::gen_range(0.0, 1.0) < 0.3 {
                self.add_gold(5);
            }
    }

    /// 进入下一波
    pub fn advance_wave(&mut self) {
        if let RunPhase::Combat(ref mut state) = self.phase {
            let max_waves = self.zone.wave_count();
            if state.wave_in_zone < max_waves {
                state.wave_in_zone += 1;
                state.spawn_timer = 0.0;
                state.wave_start_time = 0.0;
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
    pub fn enter_rest_phase(&mut self) {
        self.phase = RunPhase::Rest(RestPhaseState {
            options: vec![RestOption::Heal, RestOption::UpgradeCard, RestOption::RemoveCard],
            selected: None,
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
    for (i, relic) in run.relics.iter().enumerate() {
        let x = relic_start_x - (i as f32 * 35.0);
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
}

/// 绘制 Boss 血条
pub fn draw_boss_health_bar(boss: &BossState) {
    let bar_width = 400.0;
    let bar_height = 20.0;
    let bar_x = (screen_width() - bar_width) / 2.0;
    let bar_y = 50.0;

    // 背景
    draw_rectangle(bar_x - 2.0, bar_y - 2.0, bar_width + 4.0, bar_height + 4.0, DARKGRAY);

    // 血条
    let health_width = bar_width * boss.health_percent();
    let health_color = if boss.is_enraged { RED } else { Color::new(0.8, 0.2, 0.2, 1.0) };
    draw_rectangle(bar_x, bar_y, health_width, bar_height, health_color);

    // Boss 名称
    let name = boss.kind.name();
    let text_width = measure_text(name, None, 24, 1.0).width;
    draw_text(
        name,
        (screen_width() - text_width) / 2.0,
        bar_y - 10.0,
        24.0,
        if boss.is_enraged { RED } else { WHITE },
    );

    // 狂暴标记
    if boss.is_enraged {
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
const ALL_RELICS: [RelicId; 10] = [
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
            // 卡牌占位：后续接 battle_draft::Card
            let name = match rand::gen_range(0u32, 4u32) {
                0 => "超频模块",
                1 => "穿甲弹",
                2 => "相位爆发",
                _ => "离子涌流",
            };
            RewardOption::Card(name.to_string())
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
            // 卡牌
            (RewardOption::Card("随机卡牌".to_string()), 30)
        };

        items.push(ShopItem {
            reward,
            price,
            sold: false,
        });
    }

    items
}

/// 应用奖励选项
pub fn apply_reward_option(run: &mut RunState, players: &mut [Player], reward: &RewardOption) {
    match reward {
        RewardOption::Gold(amount) => run.add_gold(*amount),
        RewardOption::Relic(relic) => run.add_relic(*relic),
        RewardOption::Heal(amount) => {
            let add_lives = (*amount).round().max(1.0) as u32;
            for p in players.iter_mut() {
                p.lives += add_lives;
            }
        }
        RewardOption::Card(_name) => {
            // 占位：后续与 battle_draft 卡牌系统对接
        }
    }
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
        && let Some(item) = shop.items.get_mut(idx) {
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
        RewardOption::Relic(relic) => (
            "遗物",
            relic.name().to_string(),
            relic.description().to_string(),
            relic.rarity_color(),
        ),
        RewardOption::Card(name) => (
            "卡牌",
            name.clone(),
            "获得一张构筑卡牌".to_string(),
            Color::new(0.2, 0.6, 1.0, 1.0),
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
}
