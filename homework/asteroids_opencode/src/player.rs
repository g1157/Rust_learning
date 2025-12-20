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
//! - 多种武器类型（普通、散弹、穿透、追踪导弹）

use macroquad::prelude::*;

use crate::battle_draft::{Card, PlayerModifiers};
use crate::bullet::{Bullet, WeaponType};
use crate::constants::{chain_ion, homing, killstreak, phase_dash};
use crate::dash_trail::{PhaseExplosion, PhaseTrail};
use crate::score::Score;
use crate::ship::Ship;
use crate::utils::wrap_around;

pub const INVULNERABLE_DURATION: f64 = 3.0; // 秒
pub const HIT_INVULNERABLE_DURATION: f64 = 1.0; // 秒
pub const SHIELD_DURATION: f64 = 5.0; // 秒
pub const SHOOT_COOLDOWN: f64 = 0.5; // 秒
pub const WEAPON_POWERUP_DURATION: f64 = 10.0; // 武器道具持续时间

// 额外生命奖励阈值
pub const EXTRA_LIFE_THRESHOLD: u32 = 10_000; // 每 10000 分获得一条额外生命

// 冲刺系统常量
pub const DASH_COOLDOWN: f64 = 2.0; // 冲刺冷却时间
pub const DASH_DURATION: f64 = 0.35; // 冲刺持续时间（增加到0.35秒）
pub const DASH_INVULN_DURATION: f64 = 0.4; // 冲刺无敌时间（略长于冲刺时间）
pub const DASH_SPEED_MULTIPLIER: f32 = 3.5; // 冲刺速度倍数（增加到3.5倍）

// 超空间跳跃常量
pub const HYPERSPACE_COOLDOWN: f64 = 5.0; // 超空间跳跃冷却时间
pub const HYPERSPACE_VANISH_DURATION: f64 = 0.3; // 消失持续时间
pub const HYPERSPACE_APPEAR_INVULN: f64 = 0.5; // 出现后无敌时间
pub const HYPERSPACE_RISK_CHANCE: f32 = 0.15; // 风险概率（15%）

// 连击系统常量 - 使用集中化配置
const KILLSTREAK_RESET_TIME: f64 = killstreak::RESET_TIME;
const KILLSTREAK_FIRE_RATE_BONUS: f64 = killstreak::FIRE_RATE_BONUS;
const KILLSTREAK_SPEED_BONUS: f32 = killstreak::SPEED_BONUS;

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
    pub dash: KeyCode,        // 冲刺键
    pub hyperspace: KeyCode,  // 超空间跳跃键
    pub phase_dash: KeyCode,  // 相位闪现键
}

impl Controls {
    pub fn shoot_pressed(&self, input: &crate::input::Input) -> bool {
        input.is_key_down(self.shoot_primary)
            || self
                .shoot_alt
                .map(|key| input.is_key_down(key))
                .unwrap_or(false)
    }

    pub fn weapon_switch_pressed(&self, input: &crate::input::Input) -> bool {
        input.is_key_pressed(self.weapon_switch)
            || self
                .weapon_switch_alt
                .map(|key| input.is_key_pressed(key))
                .unwrap_or(false)
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
    pub modifiers: PlayerModifiers, // 选卡系统属性修改器
    next_extra_life_at: u32,        // 下一个额外生命的分数阈值
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
    // 渲染状态
    pub is_thrusting: bool,
    // 冲刺系统
    pub dash_cooldown_until: f64,          // 冲刺冷却结束时间
    pub dash_active_until: f64,            // 冲刺效果结束时间
    pub dash_invuln_until: f64,            // 冲刺无敌结束时间
    pub dash_direction: Vec2,              // 冲刺方向
    pub dash_trail: Vec<(Vec2, f32, f64)>, // 残影轨迹 (位置, 角度, 时间)
    // 相位闪现系统（瞬移+延迟爆裂尾迹）
    pub phase_cooldown_until: f64, // 相位闪现冷却结束时间
    pub phase_invuln_until: f64,   // 相位闪现无敌结束时间
    pub phase_visual_until: f64,   // 相位半透明结束时间
    pub phase_trail: PhaseTrail,   // 延迟爆裂尾迹
    // 成就追踪
    pub took_damage_this_life: bool, // 当前生命是否受伤（用于无伤成就判定）
    // 超空间跳跃系统
    pub hyperspace_cooldown_until: f64, // 超空间跳跃冷却结束时间
    pub hyperspace_active: bool,        // 是否正在超空间跳跃中（消失状态）
    pub hyperspace_appear_at: f64,      // 出现时间点
    // 新道具效果系统
    pub rapid_fire_until: f64,      // 快速射击结束时间
    pub piercing_until: f64,        // 穿透弹结束时间
    pub temp_shield_hits: u32,      // 临时护盾剩余次数
    pub ghost_mode_until: f64,      // 幽灵模式结束时间
    pub overdrive_until: f64,       // 超速模式结束时间
    pub teleport_charge_until: f64, // 传送充能结束时间
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
            modifiers: PlayerModifiers::new(),
            next_extra_life_at: EXTRA_LIFE_THRESHOLD,
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
            is_thrusting: false,
            dash_cooldown_until: now,
            dash_active_until: now,
            dash_invuln_until: now,
            dash_direction: Vec2::ZERO,
            dash_trail: Vec::new(),
            phase_cooldown_until: now,
            phase_invuln_until: now,
            phase_visual_until: now,
            phase_trail: PhaseTrail::new(),
            took_damage_this_life: false,
            hyperspace_cooldown_until: now,
            hyperspace_active: false,
            hyperspace_appear_at: now,
            // 新道具效果
            rapid_fire_until: now,
            piercing_until: now,
            temp_shield_hits: 0,
            ghost_mode_until: now,
            overdrive_until: now,
            teleport_charge_until: now,
        }
    }

    pub fn reset(&mut self, position: Vec2, now: f64, starting_lives: u32) {
        self.ship = Ship::reset(position);
        self.bullets.clear();
        self.last_shot = now - 1.0;
        self.score.reset();
        self.modifiers.reset();
        self.next_extra_life_at = EXTRA_LIFE_THRESHOLD;
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
        self.is_thrusting = false;
        self.dash_cooldown_until = now;
        self.dash_active_until = now;
        self.dash_invuln_until = now;
        self.dash_direction = Vec2::ZERO;
        self.dash_trail.clear();
        self.phase_cooldown_until = now;
        self.phase_invuln_until = now;
        self.phase_visual_until = now;
        self.phase_trail.clear();
        self.took_damage_this_life = false;
        self.hyperspace_cooldown_until = now;
        self.hyperspace_active = false;
        self.hyperspace_appear_at = now;
        // 重置新道具效果
        self.rapid_fire_until = now;
        self.piercing_until = now;
        self.temp_shield_hits = 0;
        self.ghost_mode_until = now;
        self.overdrive_until = now;
        self.teleport_charge_until = now;
    }

    /// 增加分数，并在达到阈值时奖励额外生命
    ///
    /// 每达到 10000 分的倍数时获得一条额外生命。
    /// 如果一次性获得大量分数，可能会获得多条生命。
    ///
    /// 返回是否获得了额外生命（用于播放音效）
    pub fn add_score(&mut self, points: u32) -> bool {
        self.score.add_points(points);
        let mut awarded = false;

        // 检查是否达到或超过下一个额外生命阈值
        while self.score.value() >= self.next_extra_life_at {
            self.lives += 1;
            awarded = true;

            // 计算下一个阈值，防止溢出
            let next = self.next_extra_life_at.saturating_add(EXTRA_LIFE_THRESHOLD);
            if next == self.next_extra_life_at {
                // 溢出了，不再增加阈值
                break;
            }
            self.next_extra_life_at = next;
        }

        awarded
    }

    /// 应用选卡系统的卡牌效果
    ///
    /// 如果是 ExtraLife 卡牌，除了记录到 modifiers 外，还会立即增加生命值
    pub fn apply_draft_card(&mut self, card: Card) {
        self.modifiers.apply_card(card);

        // ExtraLife 需要立即增加生命值
        if matches!(card, Card::ExtraLife) {
            self.lives = self.lives.saturating_add(1);
        }
    }

    pub fn can_shoot(&self, now: f64) -> bool {
        now - self.last_shot > self.shoot_cooldown(now)
    }

    pub fn record_shot(&mut self, position: Vec2, direction: Vec2, now: f64) -> u32 {
        self.last_shot = now;

        // 检查武器道具是否过期
        if now >= self.weapon_powerup_until {
            self.weapon_powerup = WeaponPowerUp::None;
        }

        // 应用子弹速度修改器
        let modified_direction = direction * self.modifiers.bullet_speed_mult;

        // 如果有武器道具，使用道具射击
        if self.weapon_powerup != WeaponPowerUp::None {
            return self.record_powerup_shot(position, modified_direction, now);
        }

        // 否则使用普通武器
        // 穿透弹道具：将普通子弹改为穿透类型
        let use_piercing = self.piercing_active(now);
        match self.weapon_type {
            WeaponType::Normal => {
                if use_piercing {
                    self.bullets.push(Bullet::with_weapon_type(
                        position,
                        modified_direction,
                        now,
                        WeaponType::Penetrating,
                    ));
                } else {
                    self.bullets.push(Bullet::new(position, modified_direction, now));
                }
                1
            }
            WeaponType::Spread => {
                // 散弹：3 发，扇形 45 度
                use std::f32::consts::PI;
                let spread_angle = PI / 4.0; // 45 度（增强）

                for i in -1..=1 {
                    let angle = (i as f32) * spread_angle;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    // 旋转向量（使用修改后的速度）
                    let spread_dir = Vec2::new(
                        modified_direction.x * cos_a - modified_direction.y * sin_a,
                        modified_direction.x * sin_a + modified_direction.y * cos_a,
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
                    modified_direction,
                    now,
                    WeaponType::Penetrating,
                ));
                1
            }
            WeaponType::Homing => {
                // 追踪导弹：速度较慢，但会追踪目标（也受子弹速度加成影响）
                let missile_vel = direction.normalize() * homing::SPEED * self.modifiers.bullet_speed_mult;
                self.bullets.push(Bullet::with_weapon_type(
                    position,
                    missile_vel,
                    now,
                    WeaponType::Homing,
                ));
                1
            }
            WeaponType::ChainIon => {
                // 链式离子炮：命中后链式传导攻击附近目标
                self.bullets.push(Bullet::with_weapon_type(
                    position,
                    modified_direction,
                    now,
                    WeaponType::ChainIon,
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

        // 检查护盾道具（原有护盾）
        if self.consume_shield(time) {
            return;
        }

        // 检查临时护盾（新道具：可抵挡3次伤害）
        if self.consume_temp_shield() {
            return;
        }

        // 检查幽灵模式（50%闪避几率）
        if self.ghost_mode_active(time)
            && rand::gen_range(0.0f32, 1.0) < 0.5
        {
            return; // 闪避成功
        }

        // 标记本条命受伤
        self.took_damage_this_life = true;

        // 扣除一条生命
        if self.lives > 0 {
            self.lives -= 1;
        }

        if self.lives == 0 {
            // 最后一条命：游戏结束
            self.alive = false;
            self.survival_end = Some(time);
            // 注意：took_damage_this_life 保持当前值（用于记录这条命是否受伤）
        } else {
            // 复活：开始新的一条命（应用无敌时间修改器）
            let invuln_duration = self.modifiers.modified_invuln_duration(HIT_INVULNERABLE_DURATION);
            self.invulnerable_until = time + invuln_duration;
            // 重置受伤标记，因为这是新的一条命
            self.took_damage_this_life = false;
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
            || time < self.dash_invuln_until
            || time < self.phase_invuln_until
    }

    /// 检查是否可以冲刺（冷却结束且没有在冲刺中）
    pub fn can_dash(&self, time: f64) -> bool {
        time >= self.dash_cooldown_until && time >= self.dash_active_until
    }

    /// 检查是否正在冲刺
    pub fn is_dashing(&self, time: f64) -> bool {
        time < self.dash_active_until
    }

    /// 开始冲刺
    pub fn start_dash(&mut self, time: f64, direction: Vec2) {
        // 应用技能冷却修改器，再应用传送充能道具（-50%）和连击加成
        let mut cooldown = self.modifiers.modified_cooldown(DASH_COOLDOWN);
        if self.teleport_charge_active(time) {
            cooldown *= 0.5;
        }
        cooldown *= self.killstreak_cooldown_multiplier(); // 连击冷却加成
        self.dash_cooldown_until = time + cooldown;
        self.dash_active_until = time + DASH_DURATION;
        // 应用连击无敌时间加成
        let invuln_duration = DASH_INVULN_DURATION * self.killstreak_invuln_multiplier();
        self.dash_invuln_until = time + invuln_duration;
        self.dash_direction = direction.normalize_or_zero();
        self.dash_trail.clear();
    }

    // -------------------------------------------------------------------------
    // 相位闪现（瞬移+尾迹爆裂）
    // -------------------------------------------------------------------------

    /// 检查是否可以进行相位闪现（Shift 触发，与速度冲刺独立）
    pub fn can_phase_dash(&self, time: f64) -> bool {
        self.alive && time >= self.phase_cooldown_until
    }

    /// 执行相位闪现：瞬移固定距离，期间无敌并留下延迟爆裂尾迹
    ///
    /// 返回值：(起点, 终点)，便于上层生成粒子/音效
    pub fn start_phase_dash(&mut self, time: f64) -> (Vec2, Vec2) {
        let start_pos = self.ship.pos;
        let target = self.ship.phase_destination(phase_dash::DISTANCE);

        // 应用技能冷却修改器，再应用传送充能道具（-50%）和连击加成
        let mut cooldown = self.modifiers.modified_cooldown(phase_dash::COOLDOWN);
        if self.teleport_charge_active(time) {
            cooldown *= 0.5;
        }
        cooldown *= self.killstreak_cooldown_multiplier(); // 连击冷却加成
        self.phase_cooldown_until = time + cooldown;
        // 应用连击无敌时间加成
        let invuln_duration = phase_dash::INVULNERABLE_WINDOW * self.killstreak_invuln_multiplier();
        self.phase_invuln_until = time + invuln_duration;
        self.phase_visual_until = time + phase_dash::TRAIL_LIFETIME;

        self.phase_trail
            .seed_path(start_pos, target, self.ship.rot, time);

        // 瞬移并清空速度，避免继承旧速度
        self.ship.teleport_to(wrap_around(&target));

        (start_pos, target)
    }

    /// 更新相位尾迹的可见与爆炸状态
    pub fn update_phase_trail(&mut self, time: f64) {
        self.phase_trail.cull_expired(time);
    }

    /// 收集已到爆炸时间的尾迹节点，用于上层造成范围伤害
    #[allow(dead_code)] // 相位闪现功能正在集成中
    pub fn drain_phase_explosions(&mut self, time: f64) -> Vec<PhaseExplosion> {
        self.phase_trail.take_ready_explosions(time)
    }

    /// 获取相位闪现冷却剩余时间
    #[allow(dead_code)] // 相位闪现功能正在集成中
    pub fn phase_cooldown_remaining(&self, time: f64) -> f64 {
        if time < self.phase_cooldown_until {
            self.phase_cooldown_until - time
        } else {
            0.0
        }
    }

    /// 是否处于相位半透明期（用于渲染层淡化）
    #[allow(dead_code)] // 相位闪现功能正在集成中
    pub fn is_phase_cloaked(&self, time: f64) -> bool {
        time < self.phase_visual_until
    }

    /// 转换相位尾迹为可直接复用现有 dash 残影绘制的三元组
    pub fn phase_trail_tuples(&self, time: f64) -> Vec<(Vec2, f32, f64)> {
        self.phase_trail
            .active_segments(time)
            .map(|seg| (seg.pos, seg.rot, seg.spawned_at))
            .collect()
    }

    /// 更新冲刺残影轨迹
    pub fn update_dash_trail(&mut self, time: f64) {
        // 添加当前位置到残影轨迹
        if self.is_dashing(time) {
            self.dash_trail.push((self.ship.pos, self.ship.rot, time));
        }
        // 清理过期的残影（超过0.3秒）
        self.dash_trail.retain(|(_, _, t)| time - *t < 0.3);
    }

    /// 获取冲刺冷却剩余时间
    pub fn dash_cooldown_remaining(&self, time: f64) -> f64 {
        if time < self.dash_cooldown_until {
            self.dash_cooldown_until - time
        } else {
            0.0
        }
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
        // 应用护盾持续时间修改器
        let duration = self.modifiers.modified_shield_duration(SHIELD_DURATION);
        self.shield_until = time + duration;
    }

    pub fn grant_dual_shot(&mut self, time: f64) {
        self.weapon_powerup = WeaponPowerUp::DualShot;
        self.weapon_powerup_until = time + WEAPON_POWERUP_DURATION;
    }

    pub fn grant_triple_shot(&mut self, time: f64) {
        self.weapon_powerup = WeaponPowerUp::TripleShot;
        self.weapon_powerup_until = time + WEAPON_POWERUP_DURATION;
    }

    // -------------------------------------------------------------------------
    // 新道具效果
    // -------------------------------------------------------------------------

    /// 快速射击：射速+50%，持续6秒
    pub fn grant_rapid_fire(&mut self, time: f64) {
        self.rapid_fire_until = time + 6.0;
    }

    /// 穿透弹：子弹获得3次穿透能力，持续8秒
    pub fn grant_piercing_rounds(&mut self, time: f64) {
        self.piercing_until = time + 8.0;
    }

    /// 临时护盾：可抵挡3次伤害
    pub fn grant_temp_shield(&mut self) {
        self.temp_shield_hits = 3;
    }

    /// 幽灵模式：50%闪避几率，持续5秒
    pub fn grant_ghost_mode(&mut self, time: f64) {
        self.ghost_mode_until = time + 5.0;
    }

    /// 超速模式：速度+80%，转向+60%，持续7秒
    pub fn grant_overdrive(&mut self, time: f64) {
        self.overdrive_until = time + 7.0;
    }

    /// 传送充能：技能冷却-50%，持续10秒
    pub fn grant_teleport_charge(&mut self, time: f64) {
        self.teleport_charge_until = time + 10.0;
    }

    /// 检查快速射击是否激活
    pub fn rapid_fire_active(&self, time: f64) -> bool {
        time < self.rapid_fire_until
    }

    /// 检查穿透弹是否激活
    pub fn piercing_active(&self, time: f64) -> bool {
        time < self.piercing_until
    }

    /// 检查幽灵模式是否激活
    pub fn ghost_mode_active(&self, time: f64) -> bool {
        time < self.ghost_mode_until
    }

    /// 检查超速模式是否激活
    pub fn overdrive_active(&self, time: f64) -> bool {
        time < self.overdrive_until
    }

    /// 检查传送充能是否激活
    pub fn teleport_charge_active(&self, time: f64) -> bool {
        time < self.teleport_charge_until
    }

    /// 消耗临时护盾（被击中时调用）
    /// 返回 true 表示护盾吸收了伤害
    pub fn consume_temp_shield(&mut self) -> bool {
        if self.temp_shield_hits > 0 {
            self.temp_shield_hits -= 1;
            true
        } else {
            false
        }
    }

    pub fn shield_active(&self, time: f64) -> bool {
        self.shield_ready && time < self.shield_until
    }

    #[allow(dead_code)]
    pub fn weapon_powerup_active(&self, time: f64) -> bool {
        self.weapon_powerup != WeaponPowerUp::None && time < self.weapon_powerup_until
    }

    #[allow(dead_code)]
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

    /// 获取上次击杀时间（用于UI显示连击剩余时间）
    pub fn get_last_kill_time(&self) -> f64 {
        self.last_kill_time
    }

    /// 获取当前射击冷却时间（考虑卡牌加成、连击加成、武器类型和道具效果）
    pub fn shoot_cooldown(&self, time: f64) -> f64 {
        let base_cooldown = match self.weapon_type {
            WeaponType::Homing => homing::COOLDOWN,
            WeaponType::ChainIon => chain_ion::COOLDOWN,
            _ => SHOOT_COOLDOWN,
        };
        // 先应用卡牌加成
        let modified_cooldown = self.modifiers.modified_shoot_cooldown(base_cooldown);
        // 再应用连击加成
        let bonus_multiplier = 1.0 - (self.killstreak.min(3) as f64 * KILLSTREAK_FIRE_RATE_BONUS);
        let killstreak_cooldown = modified_cooldown * bonus_multiplier;
        // 最后应用快速射击道具（射速+50% = 冷却-33%）
        if self.rapid_fire_active(time) {
            killstreak_cooldown * 0.5
        } else {
            killstreak_cooldown
        }
    }

    /// 获取当前最大速度（考虑卡牌加成、连击加成和道具加成）
    pub fn max_speed(&self, time: f64) -> f32 {
        // 先应用卡牌加成到基础速度
        let base_speed = self.modifiers.modified_max_speed(crate::ship::SHIP_MAX_SPEED);
        // 再叠加连击加成
        let killstreak_speed = base_speed + (self.killstreak.min(3) as f32 * KILLSTREAK_SPEED_BONUS);
        // 最后应用超速模式（速度+80%）
        if self.overdrive_active(time) {
            killstreak_speed * 1.8
        } else {
            killstreak_speed
        }
    }

    /// 获取当前转向速率（考虑超速模式加成）
    pub fn turn_rate(&self, time: f64) -> f32 {
        let base_turn = crate::ship::SHIP_ROTATION_STEP;
        // 超速模式：转向+60%
        if self.overdrive_active(time) {
            base_turn * 1.6
        } else {
            base_turn
        }
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

    /// 获取当前连击的分数倍数
    ///
    /// 倍数公式：1.0 + min(连击数 * 0.1, 最大倍数 - 1.0)
    /// 例如：3连击 = 1.3x，10连击 = 2.0x（假设最大3.0x）
    pub fn score_multiplier(&self) -> f32 {
        let bonus = self.killstreak as f32 * killstreak::SCORE_MULTIPLIER_PER_KILL;
        (1.0 + bonus).min(killstreak::MAX_SCORE_MULTIPLIER)
    }

    /// 获取连击视觉等级（用于渲染效果强度）
    ///
    /// 返回 0-4 的等级：0=无，1=微光，2=发光，3=强光，4=极光
    pub fn killstreak_visual_level(&self) -> u32 {
        if self.killstreak >= killstreak::STREAK_THRESHOLDS[4] {
            4
        } else if self.killstreak >= killstreak::STREAK_THRESHOLDS[3] {
            3
        } else if self.killstreak >= killstreak::STREAK_THRESHOLDS[2] {
            2
        } else if self.killstreak >= killstreak::STREAK_THRESHOLDS[1] {
            1
        } else {
            0
        }
    }

    // -------------------------------------------------------------------------
    // 连击技能联动系统
    // -------------------------------------------------------------------------

    /// 获取连击对技能冷却的加成倍数
    ///
    /// - 3连击: 冷却时间 -20%（返回 0.8）
    /// - 10连击: 冷却时间 -50%（翻倍效果，返回 0.5）
    fn killstreak_cooldown_multiplier(&self) -> f64 {
        if self.killstreak >= 10 {
            0.5 // 10连击：技能效果翻倍，冷却减半
        } else if self.killstreak >= 3 {
            0.8 // 3连击：冷却-20%
        } else {
            1.0
        }
    }

    /// 获取连击对技能无敌时间的加成倍数
    ///
    /// - 5连击: 无敌时间 +50%（返回 1.5）
    /// - 10连击: 无敌时间 +100%（翻倍效果，返回 2.0）
    fn killstreak_invuln_multiplier(&self) -> f64 {
        if self.killstreak >= 10 {
            2.0 // 10连击：技能效果翻倍
        } else if self.killstreak >= 5 {
            1.5 // 5连击：无敌+50%
        } else {
            1.0
        }
    }

    // -------------------------------------------------------------------------
    // 超空间跳跃系统
    // -------------------------------------------------------------------------

    /// 检查是否可以进行超空间跳跃
    pub fn can_hyperspace(&self, time: f64) -> bool {
        self.alive && !self.hyperspace_active && time >= self.hyperspace_cooldown_until
    }

    /// 开始超空间跳跃（进入消失状态）
    pub fn start_hyperspace(&mut self, time: f64) {
        self.hyperspace_active = true;
        // 应用技能冷却修改器，再应用传送充能道具（-50%）和连击加成
        let mut cooldown = self.modifiers.modified_cooldown(HYPERSPACE_COOLDOWN);
        if self.teleport_charge_active(time) {
            cooldown *= 0.5;
        }
        cooldown *= self.killstreak_cooldown_multiplier(); // 连击冷却加成
        self.hyperspace_cooldown_until = time + cooldown;
        self.hyperspace_appear_at = time + HYPERSPACE_VANISH_DURATION;
        // 消失期间无敌
        self.invulnerable_until = self.invulnerable_until.max(self.hyperspace_appear_at);
    }

    /// 检查是否正在超空间跳跃中（消失状态）
    pub fn is_in_hyperspace(&self, time: f64) -> bool {
        self.hyperspace_active && time < self.hyperspace_appear_at
    }

    /// 完成超空间跳跃（传送到新位置）
    pub fn complete_hyperspace(&mut self, new_pos: Vec2, time: f64) {
        self.ship.pos = new_pos;
        self.ship.vel = Vec2::ZERO; // 传送后速度归零
        self.hyperspace_active = false;
        // 出现后给予短暂无敌（应用连击加成）
        let invuln_duration = HYPERSPACE_APPEAR_INVULN * self.killstreak_invuln_multiplier();
        self.invulnerable_until = time + invuln_duration;
    }

    /// 超空间跳跃失败（传送到危险位置导致死亡）
    pub fn hyperspace_malfunction(&mut self, time: f64) {
        self.hyperspace_active = false;
        // 取消无敌，立即受到伤害
        self.invulnerable_until = time - 0.1;
        self.mark_dead(time);
    }

    /// 获取超空间跳跃冷却剩余时间
    pub fn hyperspace_cooldown_remaining(&self, time: f64) -> f64 {
        if time < self.hyperspace_cooldown_until {
            self.hyperspace_cooldown_until - time
        } else {
            0.0
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用控制映射
    fn test_controls() -> Controls {
        Controls {
            thrust: KeyCode::W,
            left: KeyCode::A,
            right: KeyCode::D,
            shoot_primary: KeyCode::Space,
            shoot_alt: None,
            weapon_switch: KeyCode::Tab,
            weapon_switch_alt: None,
            dash: KeyCode::LeftShift,
            hyperspace: KeyCode::H,
            phase_dash: KeyCode::LeftControl,
        }
    }

    /// 创建测试用玩家
    fn make_player(now: f64, lives: u32) -> Player {
        Player::new("P1", WHITE, Vec2::ZERO, test_controls(), now, lives)
    }

    #[test]
    fn player_creation_initial_state() {
        let now = 1.0;
        let player = make_player(now, 3);

        assert!(player.alive);
        assert_eq!(player.lives, 3);
        assert!(player.is_invulnerable(now));
        assert!(player.invulnerability_remaining(now) > 0.0);
        assert!(!player.shield_active(now));
        assert_eq!(player.killstreak, 0);
        assert_eq!(player.weapon_powerup, WeaponPowerUp::None);
        assert!(player.can_shoot(now + 0.6));
        assert!(player.can_dash(now));
    }

    #[test]
    fn shooting_cooldown_respects_timer_and_killstreak_bonus() {
        let mut player = make_player(0.0, 3);
        let pos = Vec2::ZERO;
        let dir = Vec2::new(1.0, 0.0);

        // 初始可以射击
        assert!(player.can_shoot(0.1));
        player.record_shot(pos, dir, 0.2);
        // 刚射击完不能立即再射
        assert!(!player.can_shoot(0.2));

        // 冷却结束后可以射击
        let ready_at = 0.2 + player.shoot_cooldown(0.2) + 0.01;
        assert!(player.can_shoot(ready_at));

        // 连击加成减少射击冷却
        player.record_kill(1.0);
        player.record_kill(1.1);
        player.record_kill(1.2);
        assert!(player.shoot_cooldown(1.2) < SHOOT_COOLDOWN);
    }

    #[test]
    fn killstreak_tracks_and_resets() {
        let mut player = make_player(0.0, 3);

        // 记录击杀并检查连击
        player.record_kill(1.0);
        player.record_kill(1.5);
        assert_eq!(player.killstreak, 2);
        assert_eq!(player.killstreak_level(), Some("Double Kill!"));

        // 超时后连击重置
        player.update_killstreak(1.5 + KILLSTREAK_RESET_TIME + 0.1);
        assert_eq!(player.killstreak, 0);
        assert_eq!(player.killstreak_level(), None);
    }

    #[test]
    fn killstreak_level_tiers() {
        let mut player = make_player(0.0, 3);

        // 测试各级别连击描述
        player.killstreak = 0;
        assert_eq!(player.killstreak_level(), None);

        player.killstreak = 3;
        assert_eq!(player.killstreak_level(), Some("Double Kill!"));

        player.killstreak = 5;
        assert_eq!(player.killstreak_level(), Some("Triple Kill!"));

        player.killstreak = 8;
        assert_eq!(player.killstreak_level(), Some("Mega Kill!"));

        player.killstreak = 15;
        assert_eq!(player.killstreak_level(), Some("UNSTOPPABLE!"));
    }

    #[test]
    fn dash_cooldown_and_state() {
        let mut player = make_player(0.0, 3);

        // 初始可以冲刺
        assert!(player.can_dash(0.0));

        // 执行冲刺
        player.start_dash(1.0, Vec2::new(1.0, 0.0));
        assert!(player.is_dashing(1.1));
        assert!(!player.can_dash(1.1));
        assert!(player.dash_cooldown_remaining(1.1) > 0.0);

        // 冷却结束后可以再次冲刺
        let ready_time = 1.0 + DASH_COOLDOWN + 0.1;
        assert!(player.can_dash(ready_time));
        assert!(!player.is_dashing(ready_time));
    }

    #[test]
    fn dash_invulnerability() {
        let mut player = make_player(0.0, 3);

        // 初始无敌结束后
        let after_invuln = INVULNERABLE_DURATION + 0.1;
        assert!(!player.is_invulnerable(after_invuln));

        // 冲刺提供无敌
        player.start_dash(5.0, Vec2::new(1.0, 0.0));
        assert!(player.is_invulnerable(5.1));
    }

    #[test]
    fn shield_grant_and_consume() {
        let mut player = make_player(0.0, 3);

        // 授予护盾
        player.grant_shield(2.0);
        assert!(player.shield_active(2.5));
        assert!(player.shield_remaining(2.5) > 0.0);

        // 护盾应该在受击时被消耗（保护生命）
        let before_lives = player.lives;
        let hit_time = INVULNERABLE_DURATION + 0.5;
        // 注意：需要先让初始无敌结束，护盾才会被使用
        player.invulnerable_until = 0.0; // 手动清除初始无敌
        player.grant_shield(hit_time); // 重新授予护盾
        player.mark_dead(hit_time + 0.1);
        assert_eq!(player.lives, before_lives); // 生命未减少
        assert!(!player.shield_active(hit_time + 0.1)); // 护盾已消耗
    }

    #[test]
    fn shield_expires_after_duration() {
        let mut player = make_player(0.0, 3);

        player.grant_shield(1.0);
        assert!(player.shield_active(1.0 + SHIELD_DURATION - 0.1));
        assert!(!player.shield_active(1.0 + SHIELD_DURATION + 0.1));
    }

    #[test]
    fn weapon_powerups_set_and_time() {
        let mut player = make_player(0.0, 3);

        // 双向弹
        player.grant_dual_shot(1.0);
        assert_eq!(player.weapon_powerup, WeaponPowerUp::DualShot);
        assert!(player.weapon_powerup_active(1.5));

        // 三向弹（覆盖双向弹）
        player.grant_triple_shot(2.0);
        assert_eq!(player.weapon_powerup, WeaponPowerUp::TripleShot);
        assert!(player.weapon_powerup_active(2.5));
        assert!(player.weapon_powerup_remaining(2.5) > 0.0);

        // 超时后失效
        assert!(!player.weapon_powerup_active(2.0 + WEAPON_POWERUP_DURATION + 0.1));
    }

    #[test]
    fn invulnerability_timing() {
        let player = make_player(0.0, 3);

        // 初始无敌期内
        assert!(player.is_invulnerable(INVULNERABLE_DURATION - 0.1));
        assert!(player.invulnerability_remaining(0.5) > 0.0);

        // 初始无敌结束后
        let after_invuln = INVULNERABLE_DURATION + 0.1;
        assert!(!player.is_invulnerable(after_invuln));
        assert_eq!(player.invulnerability_remaining(after_invuln), 0.0);
    }

    #[test]
    fn mark_dead_consumes_lives_and_sets_alive() {
        let mut player = make_player(0.0, 2);

        // 清除初始无敌
        player.invulnerable_until = 0.0;

        // 第一次死亡
        player.mark_dead(0.5);
        assert_eq!(player.lives, 1);
        assert!(player.alive);

        // 等待受击无敌结束
        let second_death = 0.5 + HIT_INVULNERABLE_DURATION + 0.2;
        player.mark_dead(second_death);
        assert_eq!(player.lives, 0);
        assert!(!player.alive);
    }

    #[test]
    fn mark_dead_respects_invulnerability() {
        let mut player = make_player(0.0, 3);
        let initial_lives = player.lives;

        // 无敌期内不会扣命
        player.mark_dead(INVULNERABLE_DURATION - 0.5);
        assert_eq!(player.lives, initial_lives);
        assert!(player.alive);
    }

    #[test]
    fn player_reset_clears_state() {
        let mut player = make_player(0.0, 3);

        // 修改一些状态
        player.lives = 1;
        player.killstreak = 5;
        player.weapon_powerup = WeaponPowerUp::DualShot;
        player.alive = false;

        // 重置
        player.reset(Vec2::new(100.0, 100.0), 10.0, 5);

        // 验证重置后的状态
        assert!(player.alive);
        assert_eq!(player.lives, 5);
        assert_eq!(player.killstreak, 0);
        assert_eq!(player.weapon_powerup, WeaponPowerUp::None);
        assert!(player.is_invulnerable(10.0));
    }

    #[test]
    fn max_speed_increases_with_killstreak() {
        let mut player = make_player(0.0, 3);
        let base_speed = player.max_speed(0.0);

        player.killstreak = 3;
        let boosted_speed = player.max_speed(0.0);

        assert!(boosted_speed > base_speed);
    }

    // ------------------------------------------------------------------------
    // Extra Life Tests
    // ------------------------------------------------------------------------

    #[test]
    fn extra_life_awarded_on_threshold_cross() {
        let mut player = make_player(0.0, 3);

        // 接近阈值但未达到，不应获得额外生命
        assert!(!player.add_score(EXTRA_LIFE_THRESHOLD - 100));
        assert_eq!(player.lives, 3);
        assert_eq!(player.score.value(), EXTRA_LIFE_THRESHOLD - 100);

        // 达到阈值，应获得额外生命
        assert!(player.add_score(150));
        assert_eq!(player.lives, 4);
        assert_eq!(player.score.value(), EXTRA_LIFE_THRESHOLD + 50);
    }

    #[test]
    fn extra_life_awards_multiple_at_once() {
        let mut player = make_player(0.0, 1);

        // 一次性获得超过两个阈值的分数，应获得两条生命
        assert!(player.add_score(EXTRA_LIFE_THRESHOLD * 2 + 500));
        assert_eq!(player.lives, 3);
        assert_eq!(player.score.value(), EXTRA_LIFE_THRESHOLD * 2 + 500);
    }

    #[test]
    fn extra_life_threshold_resets_on_player_reset() {
        let mut player = make_player(0.0, 3);

        // 获得一条额外生命
        assert!(player.add_score(EXTRA_LIFE_THRESHOLD));
        assert_eq!(player.lives, 4);

        // 重置玩家
        player.reset(Vec2::new(10.0, 10.0), 5.0, 2);
        assert_eq!(player.lives, 2);
        assert_eq!(player.score.value(), 0);

        // 再次达到 10000 分应再次获得额外生命
        assert!(player.add_score(EXTRA_LIFE_THRESHOLD));
        assert_eq!(player.lives, 3);
    }

    #[test]
    fn extra_life_no_award_below_threshold() {
        let mut player = make_player(0.0, 3);

        // 多次小额加分，未达阈值
        assert!(!player.add_score(1000));
        assert!(!player.add_score(2000));
        assert!(!player.add_score(3000));
        assert_eq!(player.lives, 3);
        assert_eq!(player.score.value(), 6000);
    }

    // ------------------------------------------------------------------------
    // Hyperspace Tests
    // ------------------------------------------------------------------------

    #[test]
    fn hyperspace_can_activate() {
        let player = make_player(0.0, 3);
        // 初始可以使用超空间跳跃
        assert!(player.can_hyperspace(0.0));
    }

    #[test]
    fn hyperspace_cooldown_prevents_reuse() {
        let mut player = make_player(0.0, 3);

        player.start_hyperspace(1.0);
        // 立即无法再次使用（还在超空间中）
        assert!(!player.can_hyperspace(1.1));

        // 完成超空间跳跃
        player.complete_hyperspace(Vec2::new(200.0, 200.0), 1.5);

        // 冷却期间无法使用
        assert!(!player.can_hyperspace(1.0 + HYPERSPACE_COOLDOWN - 0.1));
        // 冷却结束后可以使用
        assert!(player.can_hyperspace(1.0 + HYPERSPACE_COOLDOWN + 0.1));
    }

    #[test]
    fn hyperspace_vanish_state() {
        let mut player = make_player(0.0, 3);

        player.start_hyperspace(1.0);
        assert!(player.hyperspace_active);
        assert!(player.is_in_hyperspace(1.1));
        // 消失期间应该无敌
        assert!(player.is_invulnerable(1.1));

        // 消失时间结束后不再处于超空间中
        let after_vanish = 1.0 + HYPERSPACE_VANISH_DURATION + 0.1;
        assert!(!player.is_in_hyperspace(after_vanish));
    }

    #[test]
    fn hyperspace_complete_teleports() {
        let mut player = make_player(0.0, 3);
        player.ship.pos = Vec2::new(100.0, 100.0);
        player.ship.vel = Vec2::new(50.0, 50.0);

        player.start_hyperspace(1.0);
        let new_pos = Vec2::new(500.0, 500.0);
        player.complete_hyperspace(new_pos, 1.5);

        assert_eq!(player.ship.pos, new_pos);
        assert_eq!(player.ship.vel, Vec2::ZERO);
        assert!(!player.hyperspace_active);
        // 出现后有无敌时间
        assert!(player.is_invulnerable(1.5));
    }

    #[test]
    fn hyperspace_malfunction_kills() {
        let mut player = make_player(0.0, 3);
        // 清除初始无敌
        player.invulnerable_until = 0.0;

        player.start_hyperspace(1.0);
        let initial_lives = player.lives;
        player.hyperspace_malfunction(1.5);

        assert!(!player.hyperspace_active);
        assert_eq!(player.lives, initial_lives - 1);
    }

    #[test]
    fn hyperspace_cooldown_remaining() {
        let mut player = make_player(0.0, 3);

        player.start_hyperspace(1.0);
        let remaining = player.hyperspace_cooldown_remaining(2.0);
        assert!(remaining > 0.0);
        assert!(remaining < HYPERSPACE_COOLDOWN);

        // 冷却结束后
        let after_cooldown = player.hyperspace_cooldown_remaining(1.0 + HYPERSPACE_COOLDOWN + 1.0);
        assert_eq!(after_cooldown, 0.0);
    }

    #[test]
    fn hyperspace_reset_clears_state() {
        let mut player = make_player(0.0, 3);

        player.start_hyperspace(1.0);
        player.reset(Vec2::ZERO, 10.0, 3);

        assert!(!player.hyperspace_active);
        assert!(player.can_hyperspace(10.0));
    }
}
