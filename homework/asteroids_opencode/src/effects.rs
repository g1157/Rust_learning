//! 游戏视觉效果模块
//!
//! 包含慢动作、屏幕震动和乐观命中提示效果系统。

#![allow(dead_code)] // 乐观命中系统正在集成中

use macroquad::prelude::*;

// ============================================================================
// 乐观命中提示常量
// ============================================================================

/// 待确认命中的超时时间（秒）
pub const PENDING_TIMEOUT: f32 = 0.5;
/// 确认后显示时间（秒）
pub const CONFIRM_GRACE: f32 = 0.08;
/// 拒绝后闪烁时间（秒）
pub const DENIED_FLASH: f32 = 0.2;

/// 慢动作系统
#[derive(Clone, Copy)]
pub struct SlowMotion {
    active: bool,
    start_time: f32,
    duration: f32,
    time_scale: f32, // 时间缩放 0.0-1.0，越小越慢
}

impl SlowMotion {
    pub fn new() -> Self {
        Self {
            active: false,
            start_time: 0.0,
            duration: 0.0,
            time_scale: 1.0,
        }
    }

    /// 激活慢动作
    pub fn activate(&mut self, now: f32, duration: f32, scale: f32) {
        self.active = true;
        self.start_time = now;
        self.duration = duration;
        self.time_scale = scale;
    }

    /// 更新慢动作状态
    pub fn update(&mut self, now: f32) -> f32 {
        if !self.active {
            return 1.0;
        }

        let elapsed = now - self.start_time;
        if elapsed >= self.duration {
            self.active = false;
            return 1.0;
        }

        // 渐入渐出效果
        let progress = elapsed / self.duration;
        if progress < 0.2 {
            // 前20%时间渐入
            let fade_in = progress / 0.2;
            1.0 - (1.0 - self.time_scale) * fade_in
        } else if progress > 0.8 {
            // 后20%时间渐出
            let fade_out = (1.0 - progress) / 0.2;
            1.0 - (1.0 - self.time_scale) * fade_out
        } else {
            self.time_scale
        }
    }
}

impl Default for SlowMotion {
    fn default() -> Self {
        Self::new()
    }
}

/// 命中停顿系统 (Hit Stop / Freeze Frame)
/// 在重大打击时短暂冻结游戏，增加打击感
#[derive(Clone, Copy)]
pub struct HitStop {
    active: bool,
    end_time: f32,
}

impl HitStop {
    pub fn new() -> Self {
        Self {
            active: false,
            end_time: 0.0,
        }
    }

    /// 触发命中停顿
    pub fn trigger(&mut self, now: f32, duration: f32) {
        // 如果已有停顿且剩余时间更长，不覆盖
        if self.active && self.end_time > now + duration {
            return;
        }
        self.active = true;
        self.end_time = now + duration;
    }

    /// 检查是否处于停顿状态
    pub fn is_frozen(&self, now: f32) -> bool {
        self.active && now < self.end_time
    }

    /// 更新并返回时间缩放（0.0 = 完全冻结，1.0 = 正常）
    pub fn update(&mut self, now: f32) -> f32 {
        if !self.active {
            return 1.0;
        }

        if now >= self.end_time {
            self.active = false;
            return 1.0;
        }

        // 完全冻结
        0.0
    }
}

impl Default for HitStop {
    fn default() -> Self {
        Self::new()
    }
}

/// 屏幕震动系统
#[derive(Clone, Copy)]
pub struct ScreenShake {
    intensity: f32,
    duration: f32,
    started_at: f32,
}

impl ScreenShake {
    pub fn new(intensity: f32, duration: f32, now: f32) -> Self {
        Self {
            intensity,
            duration,
            started_at: now,
        }
    }

    /// 获取当前震动偏移量
    pub fn get_offset(&self, now: f32) -> Vec2 {
        let elapsed = now - self.started_at;
        if elapsed >= self.duration {
            return Vec2::ZERO;
        }

        // 随着时间衰减的震动强度
        let decay = 1.0 - (elapsed / self.duration);
        let current_intensity = self.intensity * decay;

        // 随机方向的震动
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        Vec2::new(
            angle.cos() * current_intensity,
            angle.sin() * current_intensity,
        )
    }

    pub fn is_active(&self, now: f32) -> bool {
        now - self.started_at < self.duration
    }
}

// ============================================================================
// 乐观命中提示系统 (Phase 4C)
// ============================================================================

/// 待确认命中的目标类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PendingHitKind {
    /// 小行星
    Asteroid,
    /// UFO 敌人
    Ufo,
}

/// 待确认命中的状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PendingHitState {
    /// 等待服务器确认
    Pending,
    /// 服务器已确认命中
    Confirmed,
    /// 服务器拒绝（目标仍存在或命中无效）
    Denied,
}

/// 单个待确认命中记录
#[derive(Clone, Copy)]
pub struct PendingHit {
    /// 唯一标识符
    pub id: u32,
    /// 命中位置
    pub pos: Vec2,
    /// 创建时间
    pub created_at: f32,
    /// 当前状态
    pub state: PendingHitState,
    /// 状态变更时间（用于动画计时）
    pub state_changed_at: f32,
    /// 目标类型
    pub kind: PendingHitKind,
}

/// 乐观命中提示管理器
///
/// 在客户端预测命中时注册，等待服务器确认或拒绝。
/// 提供视觉反馈，减少网络延迟带来的"卡顿感"。
pub struct PendingHitManager {
    hits: Vec<PendingHit>,
    next_id: u32,
}

impl PendingHitManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            hits: Vec::with_capacity(16), // 预分配容量
            next_id: 1,
        }
    }

    /// 注册一个预测命中
    ///
    /// 返回唯一 ID，用于后续确认或拒绝
    pub fn register(&mut self, pos: Vec2, kind: PendingHitKind, now: f32) -> u32 {
        let id = self.next_id;
        // 安全递增，避免溢出后变为 0
        self.next_id = self.next_id.wrapping_add(1).max(1);

        self.hits.push(PendingHit {
            id,
            pos,
            created_at: now,
            state: PendingHitState::Pending,
            state_changed_at: now,
            kind,
        });
        id
    }

    /// 服务器确认命中
    pub fn confirm(&mut self, id: u32, now: f32) {
        if let Some(hit) = self
            .hits
            .iter_mut()
            .find(|h| h.id == id && h.state == PendingHitState::Pending)
        {
            hit.state = PendingHitState::Confirmed;
            hit.state_changed_at = now;
        }
    }

    /// 服务器拒绝命中
    pub fn deny(&mut self, id: u32, now: f32) {
        if let Some(hit) = self
            .hits
            .iter_mut()
            .find(|h| h.id == id && h.state == PendingHitState::Pending)
        {
            hit.state = PendingHitState::Denied;
            hit.state_changed_at = now;
        }
    }

    /// 按位置确认命中（当目标从服务器状态中消失时）
    ///
    /// 用于简化的确认逻辑：如果预测命中位置附近的目标不再存在，视为确认
    pub fn confirm_by_position(&mut self, pos: Vec2, threshold: f32, now: f32) {
        for hit in self.hits.iter_mut() {
            if hit.state == PendingHitState::Pending && (hit.pos - pos).length() < threshold {
                hit.state = PendingHitState::Confirmed;
                hit.state_changed_at = now;
            }
        }
    }

    /// 更新状态，清理过期记录
    pub fn update(&mut self, now: f32) {
        // 超时的 Pending 状态自动转为 Denied
        for hit in self.hits.iter_mut() {
            if hit.state == PendingHitState::Pending && now - hit.created_at > PENDING_TIMEOUT {
                hit.state = PendingHitState::Denied;
                hit.state_changed_at = now;
            }
        }

        // 清理已完成动画的记录
        self.hits.retain(|hit| match hit.state {
            PendingHitState::Pending => now - hit.created_at <= PENDING_TIMEOUT,
            PendingHitState::Confirmed => now - hit.state_changed_at <= CONFIRM_GRACE,
            PendingHitState::Denied => now - hit.state_changed_at <= DENIED_FLASH,
        });
    }

    /// 清空所有记录（游戏重置时调用）
    pub fn clear(&mut self) {
        self.hits.clear();
    }

    /// 获取所有待绘制的命中记录
    pub fn get_all(&self) -> &[PendingHit] {
        &self.hits
    }

    /// 获取当前记录数量
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.hits.len()
    }
}

impl Default for PendingHitManager {
    fn default() -> Self {
        Self::new()
    }
}
