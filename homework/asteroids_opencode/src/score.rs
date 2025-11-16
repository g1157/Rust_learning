//! 分数模块
//!
//! 简单的分数追踪系统。
//!
//! ## 功能
//! - 分数累加
//! - 分数重置
//! - 分数查询

/// 玩家分数跟踪
#[derive(Default)]
pub struct Score {
    value: u32,
}

impl Score {
    /// 创建新的分数记录（初始为 0）
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置分数为 0
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// 增加分数
    pub fn add_points(&mut self, points: u32) {
        self.value += points;
    }

    /// 获取当前分数
    pub fn value(&self) -> u32 {
        self.value
    }
}
