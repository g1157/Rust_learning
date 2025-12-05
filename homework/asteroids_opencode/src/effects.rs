//! 游戏视觉效果模块
//!
//! 包含慢动作和屏幕震动效果系统。

use macroquad::prelude::*;

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
