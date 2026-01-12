//! 技能系统模块
//!
//! 从 player.rs 拆分出来的主动技能逻辑。
//!
//! ## 功能
//! - 冲刺系统 (Dash)
//! - 相位闪现 (Phase Dash)
//! - 超空间跳跃 (Hyperspace)
//! - 技能冷却管理

use macroquad::prelude::*;

use crate::constants::phase_dash;
use crate::dash_trail::{PhaseExplosion, PhaseTrail};
use crate::utils::wrap_around;

// ============================================================================
// 技能常量
// ============================================================================

/// 冲刺冷却时间
pub const DASH_COOLDOWN: f64 = 2.0;
/// 冲刺持续时间
pub const DASH_DURATION: f64 = 0.35;
/// 冲刺无敌时间
pub const DASH_INVULN_DURATION: f64 = 0.4;
/// 冲刺速度倍数
pub const DASH_SPEED_MULTIPLIER: f32 = 3.5;

/// 超空间跳跃冷却时间
pub const HYPERSPACE_COOLDOWN: f64 = 5.0;
/// 超空间消失持续时间
pub const HYPERSPACE_VANISH_DURATION: f64 = 0.3;
/// 超空间出现后无敌时间
pub const HYPERSPACE_APPEAR_INVULN: f64 = 0.5;
/// 超空间风险概率
pub const HYPERSPACE_RISK_CHANCE: f32 = 0.15;

// ============================================================================
// 冲刺系统
// ============================================================================

/// 冲刺状态
#[derive(Clone)]
pub struct DashState {
    /// 冲刺冷却结束时间
    pub cooldown_until: f64,
    /// 冲刺效果结束时间
    pub active_until: f64,
    /// 冲刺无敌结束时间
    pub invuln_until: f64,
    /// 冲刺方向
    pub direction: Vec2,
    /// 残影轨迹 (位置, 角度, 时间)
    pub trail: Vec<(Vec2, f32, f64)>,
}

impl DashState {
    pub fn new() -> Self {
        Self {
            cooldown_until: 0.0,
            active_until: 0.0,
            invuln_until: 0.0,
            direction: Vec2::ZERO,
            trail: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cooldown_until = 0.0;
        self.active_until = 0.0;
        self.invuln_until = 0.0;
        self.direction = Vec2::ZERO;
        self.trail.clear();
    }

    /// 检查是否可以冲刺
    pub fn can_dash(&self, time: f64) -> bool {
        time >= self.cooldown_until && time >= self.active_until
    }

    /// 检查是否正在冲刺
    pub fn is_dashing(&self, time: f64) -> bool {
        time < self.active_until
    }

    /// 检查是否在冲刺无敌中
    pub fn is_invulnerable(&self, time: f64) -> bool {
        time < self.invuln_until
    }

    /// 开始冲刺
    pub fn start(
        &mut self,
        time: f64,
        direction: Vec2,
        cooldown_modifier: f64,
        invuln_modifier: f64,
    ) {
        let cooldown = DASH_COOLDOWN * cooldown_modifier;
        let invuln_duration = DASH_INVULN_DURATION * invuln_modifier;

        self.cooldown_until = time + cooldown;
        self.active_until = time + DASH_DURATION;
        self.invuln_until = time + invuln_duration;
        self.direction = direction.normalize_or_zero();
        self.trail.clear();
    }

    /// 添加残影点
    pub fn add_trail_point(&mut self, pos: Vec2, rot: f32, time: f64) {
        self.trail.push((pos, rot, time));
    }

    /// 获取冷却剩余时间
    pub fn cooldown_remaining(&self, time: f64) -> f64 {
        (self.cooldown_until - time).max(0.0)
    }

    /// 获取冷却进度 (0.0 = 冷却中, 1.0 = 就绪)
    pub fn cooldown_progress(&self, time: f64) -> f32 {
        if time >= self.cooldown_until {
            1.0
        } else {
            let remaining = self.cooldown_until - time;
            (1.0 - remaining / DASH_COOLDOWN).max(0.0) as f32
        }
    }
}

impl Default for DashState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 相位闪现系统
// ============================================================================

/// 相位闪现状态
#[derive(Clone)]
pub struct PhaseDashState {
    /// 冷却结束时间
    pub cooldown_until: f64,
    /// 无敌结束时间
    pub invuln_until: f64,
    /// 视觉效果结束时间
    pub visual_until: f64,
    /// 延迟爆裂尾迹
    pub trail: PhaseTrail,
}

impl PhaseDashState {
    pub fn new() -> Self {
        Self {
            cooldown_until: 0.0,
            invuln_until: 0.0,
            visual_until: 0.0,
            trail: PhaseTrail::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cooldown_until = 0.0;
        self.invuln_until = 0.0;
        self.visual_until = 0.0;
        self.trail.clear();
    }

    /// 检查是否可以进行相位闪现
    pub fn can_phase_dash(&self, time: f64) -> bool {
        time >= self.cooldown_until
    }

    /// 检查是否在相位无敌中
    pub fn is_invulnerable(&self, time: f64) -> bool {
        time < self.invuln_until
    }

    /// 是否处于相位视觉效果期
    pub fn is_visual_active(&self, time: f64) -> bool {
        time < self.visual_until
    }

    /// 执行相位闪现
    pub fn start(
        &mut self,
        time: f64,
        start_pos: Vec2,
        end_pos: Vec2,
        rot: f32,
        cooldown_modifier: f64,
        invuln_modifier: f64,
    ) {
        let cooldown = phase_dash::COOLDOWN * cooldown_modifier;
        let invuln_duration = phase_dash::INVULNERABLE_WINDOW * invuln_modifier;

        self.cooldown_until = time + cooldown;
        self.invuln_until = time + invuln_duration;
        self.visual_until = time + phase_dash::TRAIL_LIFETIME;
        self.trail.seed_path(start_pos, end_pos, rot, time);
    }

    /// 更新尾迹状态
    pub fn update(&mut self, time: f64) {
        self.trail.cull_expired(time);
    }

    /// 收集已就绪的爆炸点
    pub fn drain_explosions(&mut self, time: f64) -> Vec<PhaseExplosion> {
        self.trail.take_ready_explosions(time)
    }

    /// 获取冷却剩余时间
    pub fn cooldown_remaining(&self, time: f64) -> f64 {
        (self.cooldown_until - time).max(0.0)
    }

    /// 获取冷却进度
    pub fn cooldown_progress(&self, time: f64) -> f32 {
        if time >= self.cooldown_until {
            1.0
        } else {
            let remaining = self.cooldown_until - time;
            (1.0 - remaining / phase_dash::COOLDOWN).max(0.0) as f32
        }
    }
}

impl Default for PhaseDashState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 超空间跳跃系统
// ============================================================================

/// 超空间跳跃状态
#[derive(Clone)]
pub struct HyperspaceState {
    /// 冷却结束时间
    pub cooldown_until: f64,
    /// 是否正在消失中
    pub active: bool,
    /// 出现时间点
    pub appear_at: f64,
    /// 出现后无敌结束时间
    pub invuln_until: f64,
}

impl HyperspaceState {
    pub fn new() -> Self {
        Self {
            cooldown_until: 0.0,
            active: false,
            appear_at: 0.0,
            invuln_until: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.cooldown_until = 0.0;
        self.active = false;
        self.appear_at = 0.0;
        self.invuln_until = 0.0;
    }

    /// 检查是否可以超空间跳跃
    pub fn can_hyperspace(&self, time: f64) -> bool {
        time >= self.cooldown_until && !self.active
    }

    /// 检查是否正在超空间跳跃中（消失状态）
    pub fn is_active(&self, time: f64) -> bool {
        self.active && time < self.appear_at
    }

    /// 检查是否在超空间无敌中
    pub fn is_invulnerable(&self, time: f64) -> bool {
        time < self.invuln_until
    }

    /// 开始超空间跳跃
    pub fn start(&mut self, time: f64, cooldown_modifier: f64) {
        let cooldown = HYPERSPACE_COOLDOWN * cooldown_modifier;

        self.cooldown_until = time + cooldown;
        self.active = true;
        self.appear_at = time + HYPERSPACE_VANISH_DURATION;
    }

    /// 完成超空间跳跃（出现）
    pub fn complete(&mut self, time: f64) -> bool {
        if self.active && time >= self.appear_at {
            self.active = false;
            self.invuln_until = time + HYPERSPACE_APPEAR_INVULN;

            // 风险判定
            rand::gen_range(0.0f32, 1.0) < HYPERSPACE_RISK_CHANCE
        } else {
            false
        }
    }

    /// 生成随机出现位置
    pub fn random_destination() -> Vec2 {
        Vec2::new(
            rand::gen_range(50.0, macroquad::window::screen_width() - 50.0),
            rand::gen_range(50.0, macroquad::window::screen_height() - 50.0),
        )
    }

    /// 获取冷却进度
    pub fn cooldown_progress(&self, time: f64) -> f32 {
        if time >= self.cooldown_until {
            1.0
        } else {
            let remaining = self.cooldown_until - time;
            (1.0 - remaining / HYPERSPACE_COOLDOWN).max(0.0) as f32
        }
    }
}

impl Default for HyperspaceState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 综合技能状态
// ============================================================================

/// 玩家所有技能的聚合状态
#[derive(Clone)]
pub struct SkillState {
    pub dash: DashState,
    pub phase_dash: PhaseDashState,
    pub hyperspace: HyperspaceState,
}

impl SkillState {
    pub fn new() -> Self {
        Self {
            dash: DashState::new(),
            phase_dash: PhaseDashState::new(),
            hyperspace: HyperspaceState::new(),
        }
    }

    pub fn reset(&mut self) {
        self.dash.reset();
        self.phase_dash.reset();
        self.hyperspace.reset();
    }

    /// 检查是否有任何技能提供的无敌效果
    pub fn is_skill_invulnerable(&self, time: f64) -> bool {
        self.dash.is_invulnerable(time)
            || self.phase_dash.is_invulnerable(time)
            || self.hyperspace.is_invulnerable(time)
    }

    /// 更新所有技能状态
    pub fn update(&mut self, time: f64) {
        self.phase_dash.update(time);
    }
}

impl Default for SkillState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dash_state() {
        let mut dash = DashState::new();
        assert!(dash.can_dash(0.0));

        dash.start(0.0, Vec2::new(1.0, 0.0), 1.0, 1.0);
        assert!(!dash.can_dash(0.0));
        assert!(dash.is_dashing(0.1));
        assert!(dash.is_invulnerable(0.2));
    }

    #[test]
    fn test_phase_dash_state() {
        let mut phase = PhaseDashState::new();
        assert!(phase.can_phase_dash(0.0));

        phase.start(0.0, Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0, 1.0, 1.0);
        assert!(!phase.can_phase_dash(0.0));
        assert!(phase.is_invulnerable(0.1));
    }

    #[test]
    fn test_hyperspace_state() {
        let mut hyper = HyperspaceState::new();
        assert!(hyper.can_hyperspace(0.0));

        hyper.start(0.0, 1.0);
        assert!(!hyper.can_hyperspace(0.0));
        assert!(hyper.is_active(0.1));
    }

    #[test]
    fn test_skill_state_invulnerable() {
        let mut skills = SkillState::new();
        assert!(!skills.is_skill_invulnerable(0.0));

        skills.dash.start(0.0, Vec2::new(1.0, 0.0), 1.0, 1.0);
        assert!(skills.is_skill_invulnerable(0.1));
    }
}
