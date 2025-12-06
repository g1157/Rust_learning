//! Battle Draft 卡牌选择系统
//!
//! 在游戏开始和击杀 UFO 后触发卡牌选择界面，
//! 玩家可以从 3 张随机卡牌中选择 1 张来强化飞船属性。
//!
//! ## 功能
//! - 12 种卡牌类型（武器、机动、防御、特殊）
//! - 稀有度系统（普通、稀有、史诗）
//! - 属性叠加机制
//! - 最多 4 次选卡机会（开局 + 3 次 UFO 击杀）

#![allow(dead_code)] // 模块正在集成中

use macroquad::prelude::*;

// ============================================================================
// 常量定义
// ============================================================================

/// 每局游戏最大 UFO 触发次数
pub const MAX_UFO_TRIGGERS: u32 = 3;
/// 每次选卡提供的选项数量
pub const DRAFT_OPTIONS_COUNT: usize = 3;

// ============================================================================
// 卡牌稀有度
// ============================================================================

/// 卡牌稀有度
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardRarity {
    /// 普通卡牌（白色边框）
    Common,
    /// 稀有卡牌（蓝色边框）
    Rare,
    /// 史诗卡牌（紫色边框）
    Epic,
}

impl CardRarity {
    /// 获取稀有度对应的边框颜色
    pub fn border_color(&self) -> Color {
        match self {
            CardRarity::Common => Color::new(0.7, 0.7, 0.7, 1.0), // 灰白色
            CardRarity::Rare => Color::new(0.3, 0.5, 0.9, 1.0),   // 蓝色
            CardRarity::Epic => Color::new(0.6, 0.3, 0.9, 1.0),   // 紫色
        }
    }

    /// 获取稀有度对应的背景光晕颜色
    pub fn glow_color(&self) -> Color {
        match self {
            CardRarity::Common => Color::new(0.5, 0.5, 0.5, 0.15),
            CardRarity::Rare => Color::new(0.3, 0.5, 0.9, 0.2),
            CardRarity::Epic => Color::new(0.6, 0.3, 0.9, 0.25),
        }
    }

    /// 获取稀有度名称
    pub fn name(&self) -> &'static str {
        match self {
            CardRarity::Common => "Common",
            CardRarity::Rare => "Rare",
            CardRarity::Epic => "Epic",
        }
    }
}

// ============================================================================
// 卡牌类型
// ============================================================================

/// 卡牌类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    // === 武器类 ===
    /// 射速提升 +20%
    FireRateBoost,
    /// 子弹速度提升 +30%
    BulletSpeedBoost,
    /// 双发射击（前后同时发射）
    DoubleShot,

    // === 机动类 ===
    /// 转向速度提升 +25%
    TurnRateBoost,
    /// 最大速度提升 +20%
    MaxSpeedBoost,
    /// 加速度提升 +30%
    AccelerationBoost,

    // === 防御类 ===
    /// 额外生命 +1
    ExtraLife,
    /// 护盾持续时间提升 +50%
    ShieldDurationBoost,
    /// 无敌时间延长 +1 秒
    InvulnDurationBoost,

    // === 特殊类 ===
    /// 技能冷却减少 -30%
    CooldownReduction,
    /// 闪电链伤害 +1（如果有连锁系统）
    ChainDamage,
    /// 爆炸半径提升 +50%
    ExplosionRadiusBoost,
}

impl Card {
    /// 获取卡牌名称
    pub fn name(&self) -> &'static str {
        match self {
            Card::FireRateBoost => "Rapid Fire",
            Card::BulletSpeedBoost => "Velocity Rounds",
            Card::DoubleShot => "Double Shot",
            Card::TurnRateBoost => "Quick Turn",
            Card::MaxSpeedBoost => "Afterburner",
            Card::AccelerationBoost => "Thruster Boost",
            Card::ExtraLife => "Extra Life",
            Card::ShieldDurationBoost => "Fortified Shield",
            Card::InvulnDurationBoost => "Grace Period",
            Card::CooldownReduction => "Efficiency",
            Card::ChainDamage => "Chain Lightning",
            Card::ExplosionRadiusBoost => "Big Bang",
        }
    }

    /// 获取卡牌描述
    pub fn description(&self) -> &'static str {
        match self {
            Card::FireRateBoost => "+20% Fire Rate",
            Card::BulletSpeedBoost => "+30% Bullet Speed",
            Card::DoubleShot => "Fire front & back",
            Card::TurnRateBoost => "+25% Turn Speed",
            Card::MaxSpeedBoost => "+20% Max Speed",
            Card::AccelerationBoost => "+30% Acceleration",
            Card::ExtraLife => "+1 Life",
            Card::ShieldDurationBoost => "+50% Shield Time",
            Card::InvulnDurationBoost => "+1s Invulnerability",
            Card::CooldownReduction => "-30% Cooldowns",
            Card::ChainDamage => "+1 Chain Target",
            Card::ExplosionRadiusBoost => "+50% Explosion Size",
        }
    }

    /// 获取卡牌稀有度
    pub fn rarity(&self) -> CardRarity {
        match self {
            // 普通卡牌
            Card::FireRateBoost
            | Card::TurnRateBoost
            | Card::MaxSpeedBoost
            | Card::AccelerationBoost => CardRarity::Common,

            // 稀有卡牌
            Card::BulletSpeedBoost
            | Card::ShieldDurationBoost
            | Card::InvulnDurationBoost
            | Card::CooldownReduction => CardRarity::Rare,

            // 史诗卡牌
            Card::DoubleShot | Card::ExtraLife | Card::ChainDamage | Card::ExplosionRadiusBoost => {
                CardRarity::Epic
            }
        }
    }

    /// 获取卡牌类别名称
    pub fn category(&self) -> &'static str {
        match self {
            Card::FireRateBoost | Card::BulletSpeedBoost | Card::DoubleShot => "WEAPON",
            Card::TurnRateBoost | Card::MaxSpeedBoost | Card::AccelerationBoost => "MOBILITY",
            Card::ExtraLife | Card::ShieldDurationBoost | Card::InvulnDurationBoost => "DEFENSE",
            Card::CooldownReduction | Card::ChainDamage | Card::ExplosionRadiusBoost => "SPECIAL",
        }
    }

    /// 获取卡牌类别颜色
    pub fn category_color(&self) -> Color {
        match self {
            Card::FireRateBoost | Card::BulletSpeedBoost | Card::DoubleShot => {
                Color::new(1.0, 0.4, 0.3, 1.0) // 红色 - 武器
            }
            Card::TurnRateBoost | Card::MaxSpeedBoost | Card::AccelerationBoost => {
                Color::new(0.3, 0.8, 1.0, 1.0) // 青色 - 机动
            }
            Card::ExtraLife | Card::ShieldDurationBoost | Card::InvulnDurationBoost => {
                Color::new(0.3, 0.9, 0.4, 1.0) // 绿色 - 防御
            }
            Card::CooldownReduction | Card::ChainDamage | Card::ExplosionRadiusBoost => {
                Color::new(1.0, 0.8, 0.2, 1.0) // 金色 - 特殊
            }
        }
    }

    /// 获取所有卡牌类型
    pub fn all_cards() -> &'static [Card] {
        &[
            Card::FireRateBoost,
            Card::BulletSpeedBoost,
            Card::DoubleShot,
            Card::TurnRateBoost,
            Card::MaxSpeedBoost,
            Card::AccelerationBoost,
            Card::ExtraLife,
            Card::ShieldDurationBoost,
            Card::InvulnDurationBoost,
            Card::CooldownReduction,
            Card::ChainDamage,
            Card::ExplosionRadiusBoost,
        ]
    }
}

// ============================================================================
// 玩家属性修改器
// ============================================================================

/// 玩家属性修改器
///
/// 存储所有已选择卡牌累积的属性加成
#[derive(Clone, Debug)]
pub struct PlayerModifiers {
    /// 射速加成（乘数，1.0 = 无加成）
    pub fire_rate_mult: f32,
    /// 子弹速度加成（乘数）
    pub bullet_speed_mult: f32,
    /// 是否启用双发射击
    pub double_shot: bool,
    /// 双发射击叠加次数（用于多次选择时增强效果）
    pub double_shot_stacks: u32,

    /// 转向速度加成（乘数）
    pub turn_rate_mult: f32,
    /// 最大速度加成（乘数）
    pub max_speed_mult: f32,
    /// 加速度加成（乘数）
    pub acceleration_mult: f32,

    /// 额外生命数量
    pub extra_lives: u32,
    /// 护盾持续时间加成（乘数）
    pub shield_duration_mult: f32,
    /// 无敌时间延长（秒）
    pub invuln_bonus_secs: f32,

    /// 技能冷却减少（乘数，0.7 = 减少 30%）
    pub cooldown_mult: f32,
    /// 闪电链额外目标数
    pub chain_damage_bonus: u32,
    /// 爆炸半径加成（乘数）
    pub explosion_radius_mult: f32,

    /// 已选择的卡牌历史
    pub selected_cards: Vec<Card>,
}

impl PlayerModifiers {
    /// 创建默认修改器（无加成）
    pub fn new() -> Self {
        Self {
            fire_rate_mult: 1.0,
            bullet_speed_mult: 1.0,
            double_shot: false,
            double_shot_stacks: 0,

            turn_rate_mult: 1.0,
            max_speed_mult: 1.0,
            acceleration_mult: 1.0,

            extra_lives: 0,
            shield_duration_mult: 1.0,
            invuln_bonus_secs: 0.0,

            cooldown_mult: 1.0,
            chain_damage_bonus: 0,
            explosion_radius_mult: 1.0,

            selected_cards: Vec::new(),
        }
    }

    /// 重置所有修改器
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// 应用卡牌效果
    pub fn apply_card(&mut self, card: Card) {
        self.selected_cards.push(card);

        match card {
            Card::FireRateBoost => {
                // +20% 射速 = 冷却时间减少到 83.3%
                self.fire_rate_mult *= 1.0 / 0.833;
            }
            Card::BulletSpeedBoost => {
                self.bullet_speed_mult *= 1.3;
            }
            Card::DoubleShot => {
                self.double_shot = true;
                self.double_shot_stacks += 1;
            }
            Card::TurnRateBoost => {
                self.turn_rate_mult *= 1.25;
            }
            Card::MaxSpeedBoost => {
                self.max_speed_mult *= 1.2;
            }
            Card::AccelerationBoost => {
                self.acceleration_mult *= 1.3;
            }
            Card::ExtraLife => {
                self.extra_lives += 1;
            }
            Card::ShieldDurationBoost => {
                self.shield_duration_mult *= 1.5;
            }
            Card::InvulnDurationBoost => {
                self.invuln_bonus_secs += 1.0;
            }
            Card::CooldownReduction => {
                self.cooldown_mult *= 0.7;
            }
            Card::ChainDamage => {
                self.chain_damage_bonus += 1;
            }
            Card::ExplosionRadiusBoost => {
                self.explosion_radius_mult *= 1.5;
            }
        }
    }

    /// 获取修改后的射击冷却时间
    pub fn modified_shoot_cooldown(&self, base: f64) -> f64 {
        base / self.fire_rate_mult as f64 * self.cooldown_mult as f64
    }

    /// 获取修改后的子弹速度
    pub fn modified_bullet_speed(&self, base: f32) -> f32 {
        base * self.bullet_speed_mult
    }

    /// 获取修改后的转向速度
    pub fn modified_turn_rate(&self, base: f32) -> f32 {
        base * self.turn_rate_mult
    }

    /// 获取修改后的最大速度
    pub fn modified_max_speed(&self, base: f32) -> f32 {
        base * self.max_speed_mult
    }

    /// 获取修改后的加速度
    pub fn modified_acceleration(&self, base: f32) -> f32 {
        base * self.acceleration_mult
    }

    /// 获取修改后的护盾持续时间
    pub fn modified_shield_duration(&self, base: f64) -> f64 {
        base * self.shield_duration_mult as f64
    }

    /// 获取修改后的无敌时间
    pub fn modified_invuln_duration(&self, base: f64) -> f64 {
        base + self.invuln_bonus_secs as f64
    }

    /// 获取修改后的技能冷却时间
    pub fn modified_cooldown(&self, base: f64) -> f64 {
        base * self.cooldown_mult as f64
    }

    /// 获取修改后的爆炸半径
    pub fn modified_explosion_radius(&self, base: f32) -> f32 {
        base * self.explosion_radius_mult
    }
}

impl Default for PlayerModifiers {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 选卡状态
// ============================================================================

/// 选卡界面状态
#[derive(Clone, Debug)]
pub struct DraftState {
    /// 是否正在选卡
    pub active: bool,
    /// 当前可选卡牌（3 张）
    pub options: Vec<Card>,
    /// 当前选中的卡牌索引（0-2）
    pub selection: usize,
    /// 已使用的 UFO 触发次数
    pub ufo_triggers_used: u32,
    /// 是否为开局选卡
    pub is_initial_draft: bool,
    /// 选卡动画计时器
    pub animation_timer: f32,
    /// 卡牌悬停动画计时器
    pub hover_timers: [f32; 3],
}

impl DraftState {
    /// 创建新的选卡状态
    pub fn new() -> Self {
        Self {
            active: false,
            options: Vec::new(),
            selection: 1, // 默认选中中间卡牌
            ufo_triggers_used: 0,
            is_initial_draft: false,
            animation_timer: 0.0,
            hover_timers: [0.0; 3],
        }
    }

    /// 重置选卡状态（新游戏时调用）
    pub fn reset(&mut self) {
        self.active = false;
        self.options.clear();
        self.selection = 1;
        self.ufo_triggers_used = 0;
        self.is_initial_draft = false;
        self.animation_timer = 0.0;
        self.hover_timers = [0.0; 3];
    }

    /// 检查是否可以触发 UFO 击杀选卡
    pub fn can_trigger_ufo_draft(&self) -> bool {
        self.ufo_triggers_used < MAX_UFO_TRIGGERS
    }

    /// 开始选卡（开局或 UFO 击杀后）
    pub fn start_draft(&mut self, is_initial: bool) {
        self.active = true;
        self.is_initial_draft = is_initial;
        self.selection = 1;
        self.animation_timer = 0.0;
        self.hover_timers = [0.0; 3];
        self.options = generate_draft_options();

        if !is_initial {
            self.ufo_triggers_used += 1;
        }
    }

    /// 完成选卡
    pub fn finish_draft(&mut self) -> Option<Card> {
        if !self.active || self.options.is_empty() {
            return None;
        }

        let selected = self.options.get(self.selection).copied();
        self.active = false;
        self.options.clear();
        selected
    }

    /// 移动选择（左/右）
    pub fn move_selection(&mut self, delta: i32) {
        let new_sel = self.selection as i32 + delta;
        self.selection = new_sel.clamp(0, (self.options.len() as i32 - 1).max(0)) as usize;
    }

    /// 更新动画计时器
    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.animation_timer += dt;

            // 更新悬停动画
            for (i, timer) in self.hover_timers.iter_mut().enumerate() {
                if i == self.selection {
                    *timer = (*timer + dt * 4.0).min(1.0);
                } else {
                    *timer = (*timer - dt * 4.0).max(0.0);
                }
            }
        }
    }
}

impl Default for DraftState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 卡牌生成
// ============================================================================

/// 生成 3 张随机卡牌选项
///
/// 使用加权随机，确保每次选卡有合理的稀有度分布：
/// - 至少 1 张普通卡
/// - 稀有度权重：Common 50%, Rare 35%, Epic 15%
pub fn generate_draft_options() -> Vec<Card> {
    use rand::ChooseRandom;

    let all_cards = Card::all_cards();
    let mut result = Vec::with_capacity(DRAFT_OPTIONS_COUNT);
    let mut used_indices = Vec::new();

    // 分类卡牌
    let common_cards: Vec<Card> = all_cards
        .iter()
        .filter(|c| c.rarity() == CardRarity::Common)
        .copied()
        .collect();
    let rare_cards: Vec<Card> = all_cards
        .iter()
        .filter(|c| c.rarity() == CardRarity::Rare)
        .copied()
        .collect();
    let epic_cards: Vec<Card> = all_cards
        .iter()
        .filter(|c| c.rarity() == CardRarity::Epic)
        .copied()
        .collect();

    // 确保至少有一张普通卡
    if let Some(card) = common_cards.choose() {
        result.push(*card);
        used_indices.push(all_cards.iter().position(|c| c == card).unwrap());
    }

    // 随机生成剩余卡牌
    while result.len() < DRAFT_OPTIONS_COUNT {
        let roll = rand::gen_range(0.0f32, 100.0);

        let pool: &[Card] = if roll < 50.0 {
            &common_cards
        } else if roll < 85.0 {
            &rare_cards
        } else {
            &epic_cards
        };

        if let Some(card) = pool.choose() {
            let idx = all_cards.iter().position(|c| c == card).unwrap();
            if !used_indices.contains(&idx) {
                result.push(*card);
                used_indices.push(idx);
            }
        }
    }

    // 打乱顺序
    let mut indices: Vec<usize> = (0..result.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rand::gen_range(0, i + 1);
        indices.swap(i, j);
    }

    indices.iter().map(|&i| result[i]).collect()
}

// ============================================================================
// UI 绘制
// ============================================================================

/// 绘制选卡界面
pub fn draw_draft_ui(state: &DraftState, font: Option<&Font>) {
    if !state.active || state.options.is_empty() {
        return;
    }

    let sw = screen_width();
    let sh = screen_height();

    // 半透明背景遮罩
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

    // 标题
    let title = if state.is_initial_draft {
        "Choose Your Starting Bonus"
    } else {
        "UFO Destroyed! Choose Your Reward"
    };
    let title_width = measure_text(title, font, 42, 1.0).width;

    // 标题阴影
    draw_text_ex(
        title,
        sw / 2.0 - title_width / 2.0 + 2.0,
        sh * 0.15 + 2.0,
        TextParams {
            font,
            font_size: 42,
            color: Color::new(0.0, 0.0, 0.0, 0.5),
            ..Default::default()
        },
    );

    // 标题主体
    draw_text_ex(
        title,
        sw / 2.0 - title_width / 2.0,
        sh * 0.15,
        TextParams {
            font,
            font_size: 42,
            color: Color::new(0.9, 0.92, 0.98, 1.0),
            ..Default::default()
        },
    );

    // 剩余选卡次数提示
    let remaining = MAX_UFO_TRIGGERS - state.ufo_triggers_used;
    let remaining_text = if state.is_initial_draft {
        format!("Remaining UFO bonuses: {}", MAX_UFO_TRIGGERS)
    } else {
        format!("Remaining UFO bonuses: {}", remaining)
    };
    let remaining_width = measure_text(&remaining_text, font, 20, 1.0).width;
    draw_text_ex(
        &remaining_text,
        sw / 2.0 - remaining_width / 2.0,
        sh * 0.22,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(0.6, 0.65, 0.75, 1.0),
            ..Default::default()
        },
    );

    // 卡牌布局
    let card_width = 220.0;
    let card_height = 300.0;
    let card_spacing = 40.0;
    let total_width = card_width * 3.0 + card_spacing * 2.0;
    let start_x = sw / 2.0 - total_width / 2.0;
    let card_y = sh * 0.35;

    // 绘制 3 张卡牌
    for (i, card) in state.options.iter().enumerate() {
        let hover_progress = state.hover_timers[i];
        let is_selected = i == state.selection;

        // 卡牌位置（选中时略微上移）
        let x = start_x + i as f32 * (card_width + card_spacing);
        let y_offset = if is_selected {
            -15.0 * hover_progress
        } else {
            0.0
        };
        let y = card_y + y_offset;

        // 缩放效果
        let scale = 1.0 + 0.08 * hover_progress;
        let scaled_w = card_width * scale;
        let scaled_h = card_height * scale;
        let scaled_x = x - (scaled_w - card_width) / 2.0;
        let scaled_y = y - (scaled_h - card_height) / 2.0;

        draw_card(
            scaled_x,
            scaled_y,
            scaled_w,
            scaled_h,
            *card,
            is_selected,
            hover_progress,
            font,
        );
    }

    // 底部提示
    let hint = "[A/D or Left/Right] Select  |  [Space/Enter] Confirm";
    let hint_width = measure_text(hint, font, 24, 1.0).width;
    draw_text_ex(
        hint,
        sw / 2.0 - hint_width / 2.0,
        sh * 0.88,
        TextParams {
            font,
            font_size: 24,
            color: Color::new(0.6, 0.7, 0.8, 1.0),
            ..Default::default()
        },
    );
}

/// 绘制单张卡牌
#[allow(clippy::too_many_arguments)]
fn draw_card(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    card: Card,
    selected: bool,
    hover_progress: f32,
    font: Option<&Font>,
) {
    let rarity = card.rarity();

    // 卡牌背景
    let bg_color = if selected {
        Color::new(0.12, 0.14, 0.2, 0.98)
    } else {
        Color::new(0.08, 0.1, 0.14, 0.95)
    };

    // 绘制光晕效果（选中时）
    if selected {
        let glow = rarity.glow_color();
        let glow_size = 8.0 + 4.0 * hover_progress;
        draw_rectangle(
            x - glow_size,
            y - glow_size,
            width + glow_size * 2.0,
            height + glow_size * 2.0,
            glow,
        );
    }

    // 卡牌主体
    draw_rectangle(x, y, width, height, bg_color);

    // 边框
    let border_color = rarity.border_color();
    let border_width = if selected { 3.0 } else { 2.0 };
    draw_rectangle_lines(x, y, width, height, border_width, border_color);

    // 稀有度标签（左上角）
    let rarity_text = rarity.name();
    let rarity_width = measure_text(rarity_text, font, 14, 1.0).width;
    let tag_padding = 6.0;
    draw_rectangle(
        x + 10.0,
        y + 10.0,
        rarity_width + tag_padding * 2.0,
        22.0,
        Color::new(border_color.r, border_color.g, border_color.b, 0.3),
    );
    draw_text_ex(
        rarity_text,
        x + 10.0 + tag_padding,
        y + 26.0,
        TextParams {
            font,
            font_size: 14,
            color: border_color,
            ..Default::default()
        },
    );

    // 类别标签（右上角）
    let category = card.category();
    let category_color = card.category_color();
    let category_width = measure_text(category, font, 12, 1.0).width;
    draw_rectangle(
        x + width - category_width - tag_padding * 2.0 - 10.0,
        y + 10.0,
        category_width + tag_padding * 2.0,
        20.0,
        Color::new(category_color.r, category_color.g, category_color.b, 0.2),
    );
    draw_text_ex(
        category,
        x + width - category_width - tag_padding - 10.0,
        y + 25.0,
        TextParams {
            font,
            font_size: 12,
            color: category_color,
            ..Default::default()
        },
    );

    // 卡牌图标区域（中央圆形）
    let icon_y = y + height * 0.35;
    let icon_radius = 35.0;

    // 图标背景圈
    draw_circle(
        x + width / 2.0,
        icon_y,
        icon_radius + 5.0,
        Color::new(category_color.r, category_color.g, category_color.b, 0.15),
    );
    draw_circle(
        x + width / 2.0,
        icon_y,
        icon_radius,
        Color::new(category_color.r, category_color.g, category_color.b, 0.3),
    );

    // 简单图标（根据类别绘制不同形状）
    draw_card_icon(x + width / 2.0, icon_y, &card, category_color);

    // 卡牌名称
    let name = card.name();
    let name_width = measure_text(name, font, 24, 1.0).width;
    let name_y = y + height * 0.58;

    // 名称阴影
    draw_text_ex(
        name,
        x + width / 2.0 - name_width / 2.0 + 1.0,
        name_y + 1.0,
        TextParams {
            font,
            font_size: 24,
            color: Color::new(0.0, 0.0, 0.0, 0.4),
            ..Default::default()
        },
    );
    draw_text_ex(
        name,
        x + width / 2.0 - name_width / 2.0,
        name_y,
        TextParams {
            font,
            font_size: 24,
            color: Color::new(0.9, 0.92, 0.98, 1.0),
            ..Default::default()
        },
    );

    // 卡牌描述
    let desc = card.description();
    let desc_width = measure_text(desc, font, 18, 1.0).width;
    let desc_y = y + height * 0.72;
    draw_text_ex(
        desc,
        x + width / 2.0 - desc_width / 2.0,
        desc_y,
        TextParams {
            font,
            font_size: 18,
            color: Color::new(0.7, 0.75, 0.85, 1.0),
            ..Default::default()
        },
    );

    // 选中指示器
    if selected {
        let indicator_y = y + height - 25.0;
        let indicator_text = ">>> SELECTED <<<";
        let indicator_width = measure_text(indicator_text, font, 16, 1.0).width;

        // 闪烁效果
        let alpha = 0.7 + 0.3 * (get_time() as f32 * 4.0).sin();
        draw_text_ex(
            indicator_text,
            x + width / 2.0 - indicator_width / 2.0,
            indicator_y,
            TextParams {
                font,
                font_size: 16,
                color: Color::new(0.4, 0.8, 1.0, alpha),
                ..Default::default()
            },
        );
    }
}

/// 绘制卡牌图标
fn draw_card_icon(cx: f32, cy: f32, card: &Card, color: Color) {
    match card {
        // 武器类 - 子弹/火焰图案
        Card::FireRateBoost => {
            // 三条横线表示快速射击
            for i in -1..=1 {
                let offset = i as f32 * 10.0;
                draw_line(cx - 15.0, cy + offset, cx + 15.0, cy + offset, 3.0, color);
            }
            // 箭头
            draw_triangle(
                Vec2::new(cx + 20.0, cy),
                Vec2::new(cx + 12.0, cy - 8.0),
                Vec2::new(cx + 12.0, cy + 8.0),
                color,
            );
        }
        Card::BulletSpeedBoost => {
            // 快速移动的子弹
            draw_circle(cx, cy, 8.0, color);
            draw_line(cx - 25.0, cy, cx - 10.0, cy, 2.0, color);
            draw_line(cx - 22.0, cy - 5.0, cx - 12.0, cy - 5.0, 2.0, color);
            draw_line(cx - 22.0, cy + 5.0, cx - 12.0, cy + 5.0, 2.0, color);
        }
        Card::DoubleShot => {
            // 两个子弹
            draw_circle(cx - 10.0, cy - 8.0, 6.0, color);
            draw_circle(cx + 10.0, cy + 8.0, 6.0, color);
            draw_line(cx - 10.0, cy - 8.0, cx - 10.0, cy - 20.0, 2.0, color);
            draw_line(cx + 10.0, cy + 8.0, cx + 10.0, cy + 20.0, 2.0, color);
        }

        // 机动类 - 箭头/速度图案
        Card::TurnRateBoost => {
            // 旋转箭头
            let points = 8;
            for i in 0..points {
                let angle1 = i as f32 / points as f32 * std::f32::consts::TAU;
                let angle2 = (i + 1) as f32 / points as f32 * std::f32::consts::TAU;
                let r = 18.0;
                draw_line(
                    cx + angle1.cos() * r,
                    cy + angle1.sin() * r,
                    cx + angle2.cos() * r,
                    cy + angle2.sin() * r,
                    2.0,
                    color,
                );
            }
            // 旋转方向指示
            draw_triangle(
                Vec2::new(cx + 18.0, cy - 5.0),
                Vec2::new(cx + 12.0, cy - 10.0),
                Vec2::new(cx + 12.0, cy),
                color,
            );
        }
        Card::MaxSpeedBoost => {
            // 速度计
            draw_circle_lines(cx, cy, 18.0, 2.0, color);
            // 指针
            let angle = -std::f32::consts::PI * 0.25;
            draw_line(
                cx,
                cy,
                cx + angle.cos() * 14.0,
                cy + angle.sin() * 14.0,
                3.0,
                color,
            );
        }
        Card::AccelerationBoost => {
            // 火箭尾焰
            draw_triangle(
                Vec2::new(cx, cy - 20.0),
                Vec2::new(cx - 10.0, cy + 5.0),
                Vec2::new(cx + 10.0, cy + 5.0),
                color,
            );
            // 尾焰
            let flame_color = Color::new(1.0, 0.6, 0.2, 0.8);
            draw_triangle(
                Vec2::new(cx, cy + 20.0),
                Vec2::new(cx - 6.0, cy + 5.0),
                Vec2::new(cx + 6.0, cy + 5.0),
                flame_color,
            );
        }

        // 防御类 - 盾牌/心形图案
        Card::ExtraLife => {
            // 简化的心形
            draw_circle(cx - 8.0, cy - 5.0, 10.0, color);
            draw_circle(cx + 8.0, cy - 5.0, 10.0, color);
            draw_triangle(
                Vec2::new(cx, cy + 15.0),
                Vec2::new(cx - 18.0, cy - 2.0),
                Vec2::new(cx + 18.0, cy - 2.0),
                color,
            );
        }
        Card::ShieldDurationBoost => {
            // 盾牌
            draw_circle_lines(cx, cy, 18.0, 3.0, color);
            draw_circle_lines(cx, cy, 12.0, 2.0, color);
        }
        Card::InvulnDurationBoost => {
            // 星形（表示无敌）
            let points = 5;
            for i in 0..points {
                let angle1 = (i as f32 / points as f32 - 0.25) * std::f32::consts::TAU;
                let angle2 = ((i + 2) as f32 / points as f32 - 0.25) * std::f32::consts::TAU;
                let r = 18.0;
                draw_line(
                    cx + angle1.cos() * r,
                    cy + angle1.sin() * r,
                    cx + angle2.cos() * r,
                    cy + angle2.sin() * r,
                    2.0,
                    color,
                );
            }
        }

        // 特殊类 - 特效图案
        Card::CooldownReduction => {
            // 时钟
            draw_circle_lines(cx, cy, 16.0, 2.0, color);
            draw_line(cx, cy, cx, cy - 12.0, 2.0, color);
            draw_line(cx, cy, cx + 8.0, cy, 2.0, color);
        }
        Card::ChainDamage => {
            // 闪电
            draw_line(cx - 5.0, cy - 15.0, cx + 3.0, cy - 3.0, 3.0, color);
            draw_line(cx + 3.0, cy - 3.0, cx - 3.0, cy + 3.0, 3.0, color);
            draw_line(cx - 3.0, cy + 3.0, cx + 5.0, cy + 15.0, 3.0, color);
        }
        Card::ExplosionRadiusBoost => {
            // 爆炸
            let spokes = 8;
            for i in 0..spokes {
                let angle = i as f32 / spokes as f32 * std::f32::consts::TAU;
                let inner_r = 8.0;
                let outer_r = if i % 2 == 0 { 18.0 } else { 14.0 };
                draw_line(
                    cx + angle.cos() * inner_r,
                    cy + angle.sin() * inner_r,
                    cx + angle.cos() * outer_r,
                    cy + angle.sin() * outer_r,
                    2.0,
                    color,
                );
            }
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_modifiers_default() {
        let mods = PlayerModifiers::new();
        assert_eq!(mods.fire_rate_mult, 1.0);
        assert_eq!(mods.bullet_speed_mult, 1.0);
        assert!(!mods.double_shot);
        assert_eq!(mods.extra_lives, 0);
    }

    #[test]
    fn apply_fire_rate_boost() {
        let mut mods = PlayerModifiers::new();
        mods.apply_card(Card::FireRateBoost);

        // 20% 射速提升意味着冷却时间减少
        let base_cooldown = 0.5;
        let modified = mods.modified_shoot_cooldown(base_cooldown);
        assert!(modified < base_cooldown);
    }

    #[test]
    fn apply_multiple_cards_stack() {
        let mut mods = PlayerModifiers::new();

        // 多次应用同一卡牌应该叠加
        mods.apply_card(Card::MaxSpeedBoost);
        let after_one = mods.max_speed_mult;

        mods.apply_card(Card::MaxSpeedBoost);
        let after_two = mods.max_speed_mult;

        assert!(after_two > after_one);
    }

    #[test]
    fn extra_life_stacks() {
        let mut mods = PlayerModifiers::new();

        mods.apply_card(Card::ExtraLife);
        assert_eq!(mods.extra_lives, 1);

        mods.apply_card(Card::ExtraLife);
        assert_eq!(mods.extra_lives, 2);
    }

    #[test]
    fn draft_state_initial() {
        let state = DraftState::new();
        assert!(!state.active);
        assert_eq!(state.ufo_triggers_used, 0);
        assert!(state.can_trigger_ufo_draft());
    }

    #[test]
    fn draft_state_ufo_limit() {
        let mut state = DraftState::new();

        for _ in 0..MAX_UFO_TRIGGERS {
            assert!(state.can_trigger_ufo_draft());
            state.start_draft(false);
            state.finish_draft();
        }

        assert!(!state.can_trigger_ufo_draft());
    }

    #[test]
    fn generate_options_returns_three() {
        let options = generate_draft_options();
        assert_eq!(options.len(), DRAFT_OPTIONS_COUNT);
    }

    #[test]
    fn card_rarity_distribution() {
        // 多次生成，确保有不同稀有度的卡牌
        let mut has_common = false;
        let mut has_rare = false;
        let mut has_epic = false;

        for _ in 0..20 {
            let options = generate_draft_options();
            for card in options {
                match card.rarity() {
                    CardRarity::Common => has_common = true,
                    CardRarity::Rare => has_rare = true,
                    CardRarity::Epic => has_epic = true,
                }
            }
        }

        // 20 次生成后应该至少见到每种稀有度
        assert!(has_common, "Should have seen Common cards");
        assert!(has_rare, "Should have seen Rare cards");
        assert!(has_epic, "Should have seen Epic cards");
    }
}
