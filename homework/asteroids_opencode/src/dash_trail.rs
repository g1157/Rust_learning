//! 相位闪现残影与延迟爆裂系统
//!
//! 设计要点：
//! - 由玩家按 Shift 键触发的"相位闪现"，是瞬移而非速度加速
//! - 每个残影节点记录生成时间与延迟爆炸时间
//! - 主循环可轮询 `take_ready_explosions` 来应用范围伤害
//! - 渲染层可读取 `active_segments` 以绘制半透明残影
//! - 清理策略：残影可见时长 + 爆炸延迟窗口之后即丢弃

#![allow(dead_code)] // 模块正在集成中

use macroquad::prelude::*;

use crate::constants::phase_dash;

/// 相位闪现残影片段
#[derive(Clone, Debug)]
pub struct PhaseTrailSegment {
    /// 残影位置
    pub pos: Vec2,
    /// 残影旋转角度
    pub rot: f32,
    /// 生成时间
    pub spawned_at: f64,
    /// 预定爆炸时间
    pub explode_at: f64,
    /// 是否已爆炸
    pub exploded: bool,
}

impl PhaseTrailSegment {
    /// 创建新的残影片段
    pub fn new(pos: Vec2, rot: f32, spawned_at: f64) -> Self {
        Self {
            pos,
            rot,
            spawned_at,
            explode_at: spawned_at + phase_dash::EXPLOSION_DELAY,
            exploded: false,
        }
    }

    /// 获取残影年龄（秒）
    pub fn age(&self, now: f64) -> f64 {
        now - self.spawned_at
    }
}

/// 已准备好的延迟爆炸事件
#[derive(Clone, Debug)]
pub struct PhaseExplosion {
    /// 爆炸位置
    pub pos: Vec2,
    /// 爆炸半径
    pub radius: f32,
    /// 爆炸伤害
    pub damage: u32,
}

/// 相位闪现尾迹管理器
///
/// 负责残影采样、延迟爆裂事件调度和过期清理
#[derive(Default)]
pub struct PhaseTrail {
    /// 残影片段列表
    pub segments: Vec<PhaseTrailSegment>,
}

impl PhaseTrail {
    /// 创建新的相位尾迹
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(16),
        }
    }

    /// 清空所有残影
    pub fn clear(&mut self) {
        self.segments.clear();
    }

    /// 根据起止点生成整条残影链，按步长采样
    ///
    /// # 参数
    /// - `start`: 闪现起点
    /// - `end`: 闪现终点
    /// - `rot`: 飞船旋转角度
    /// - `now`: 当前时间
    pub fn seed_path(&mut self, start: Vec2, end: Vec2, rot: f32, now: f64) {
        self.segments.clear();

        let delta = end - start;
        let length = delta.length();

        // 如果距离为零，只添加一个节点
        if length <= f32::EPSILON {
            self.segments.push(PhaseTrailSegment::new(start, rot, now));
            return;
        }

        let dir = delta / length;
        let step = phase_dash::TRAIL_SAMPLE_STEP.max(1.0);
        let mut traveled = 0.0;

        // 沿路径采样残影节点
        while traveled <= length {
            let pos = start + dir * traveled;
            self.segments.push(PhaseTrailSegment::new(pos, rot, now));
            traveled += step;
        }

        // 确保终点也有节点（避免舍入遗漏）
        let last_pos = self.segments.last().map(|s| s.pos).unwrap_or(start);
        if (last_pos - end).length() > step * 0.5 {
            self.segments.push(PhaseTrailSegment::new(end, rot, now));
        }
    }

    /// 清理超时节点
    ///
    /// 超过可见窗口+爆炸延迟窗口的节点被移除
    pub fn cull_expired(&mut self, now: f64) {
        // 额外保留一些时间给爆炸效果展示
        let max_age = phase_dash::TRAIL_LIFETIME + phase_dash::EXPLOSION_DELAY + 0.5;
        self.segments.retain(|seg| seg.age(now) <= max_age);
    }

    /// 抽取到点未爆炸的节点
    ///
    /// 返回准备好爆炸的节点列表，上层负责造成范围伤害和播放粒子/音效
    pub fn take_ready_explosions(&mut self, now: f64) -> Vec<PhaseExplosion> {
        let mut ready = Vec::new();
        for seg in self.segments.iter_mut() {
            if !seg.exploded && now >= seg.explode_at {
                seg.exploded = true;
                ready.push(PhaseExplosion {
                    pos: seg.pos,
                    radius: phase_dash::EXPLOSION_RADIUS,
                    damage: phase_dash::EXPLOSION_DAMAGE,
                });
            }
        }
        ready
    }

    /// 提供仍需绘制的残影迭代器（相位闪现半透明轨迹）
    pub fn active_segments(&self, now: f64) -> impl Iterator<Item = &PhaseTrailSegment> {
        self.segments
            .iter()
            .filter(move |seg| seg.age(now) <= phase_dash::TRAIL_LIFETIME)
    }

    /// 检查是否有活跃的残影需要绘制
    pub fn has_active_segments(&self, now: f64) -> bool {
        self.segments
            .iter()
            .any(|seg| seg.age(now) <= phase_dash::TRAIL_LIFETIME)
    }

    /// 检查是否有待爆炸的残影
    pub fn has_pending_explosions(&self, now: f64) -> bool {
        self.segments
            .iter()
            .any(|seg| !seg.exploded && now < seg.explode_at)
    }
}

// ============================================================================
// 相位闪现尾迹管理器（用于管理多个玩家的尾迹）
// ============================================================================

/// 相位尾迹全局管理器
///
/// 在游戏主循环中统一管理所有玩家的相位闪现尾迹和爆炸事件
pub struct PhaseTrailManager {
    /// 待处理的爆炸事件队列（玩家索引, 爆炸信息）
    pending_explosions: Vec<(usize, PhaseExplosion)>,
}

impl PhaseTrailManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            pending_explosions: Vec::with_capacity(32),
        }
    }

    /// 收集所有玩家的待爆炸事件
    ///
    /// 返回 `(player_index, explosion)` 元组列表
    pub fn collect_explosions(&mut self) -> Vec<(usize, PhaseExplosion)> {
        std::mem::take(&mut self.pending_explosions)
    }

    /// 添加爆炸事件到队列
    pub fn queue_explosion(&mut self, player_idx: usize, explosion: PhaseExplosion) {
        self.pending_explosions.push((player_idx, explosion));
    }

    /// 清空所有待处理事件
    pub fn clear(&mut self) {
        self.pending_explosions.clear();
    }
}

impl Default for PhaseTrailManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_trail_segment_creation() {
        let seg = PhaseTrailSegment::new(Vec2::new(100.0, 100.0), 45.0, 1.0);
        assert_eq!(seg.pos, Vec2::new(100.0, 100.0));
        assert_eq!(seg.rot, 45.0);
        assert_eq!(seg.spawned_at, 1.0);
        assert!(!seg.exploded);
        assert!((seg.explode_at - (1.0 + phase_dash::EXPLOSION_DELAY)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_phase_trail_segment_age() {
        let seg = PhaseTrailSegment::new(Vec2::ZERO, 0.0, 1.0);
        assert!((seg.age(1.5) - 0.5).abs() < f64::EPSILON);
        assert!((seg.age(2.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_phase_trail_seed_path() {
        let mut trail = PhaseTrail::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(150.0, 0.0);

        trail.seed_path(start, end, 0.0, 0.0);

        // 应该有多个采样点
        assert!(trail.segments.len() > 1);

        // 第一个点应该在起点
        assert_eq!(trail.segments[0].pos, start);

        // 最后一个点应该接近终点
        let last = trail.segments.last().unwrap();
        assert!((last.pos - end).length() < phase_dash::TRAIL_SAMPLE_STEP);
    }

    #[test]
    fn test_phase_trail_seed_path_zero_distance() {
        let mut trail = PhaseTrail::new();
        let pos = Vec2::new(100.0, 100.0);

        trail.seed_path(pos, pos, 45.0, 0.0);

        // 零距离应该只有一个节点
        assert_eq!(trail.segments.len(), 1);
        assert_eq!(trail.segments[0].pos, pos);
    }

    #[test]
    fn test_phase_trail_take_ready_explosions() {
        let mut trail = PhaseTrail::new();
        trail.seed_path(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0, 0.0);

        // 在爆炸时间之前，不应有爆炸
        let explosions = trail.take_ready_explosions(0.5);
        assert!(explosions.is_empty());

        // 爆炸时间之后，应该有爆炸
        let delay = phase_dash::EXPLOSION_DELAY;
        let explosions = trail.take_ready_explosions(delay + 0.1);
        assert!(!explosions.is_empty());

        // 再次调用不应重复返回
        let explosions_again = trail.take_ready_explosions(delay + 0.2);
        assert!(explosions_again.is_empty());
    }

    #[test]
    fn test_phase_trail_active_segments() {
        let mut trail = PhaseTrail::new();
        trail.seed_path(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0, 0.0);

        // 在可见时间内应该有活跃节点
        let active: Vec<_> = trail.active_segments(0.5).collect();
        assert!(!active.is_empty());

        // 超过可见时间应该没有活跃节点
        let lifetime = phase_dash::TRAIL_LIFETIME;
        let active: Vec<_> = trail.active_segments(lifetime + 0.1).collect();
        assert!(active.is_empty());
    }

    #[test]
    fn test_phase_trail_cull_expired() {
        let mut trail = PhaseTrail::new();
        trail.seed_path(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.0, 0.0);
        let initial_count = trail.segments.len();

        // 在有效期内不应清除
        trail.cull_expired(0.5);
        assert_eq!(trail.segments.len(), initial_count);

        // 超过最大保留时间后应该清除
        let max_age = phase_dash::TRAIL_LIFETIME + phase_dash::EXPLOSION_DELAY + 1.0;
        trail.cull_expired(max_age);
        assert!(trail.segments.is_empty());
    }

    #[test]
    fn test_phase_trail_manager() {
        let mut manager = PhaseTrailManager::new();

        let explosion = PhaseExplosion {
            pos: Vec2::new(100.0, 100.0),
            radius: 50.0,
            damage: 1,
        };

        manager.queue_explosion(0, explosion.clone());
        manager.queue_explosion(1, explosion);

        let collected = manager.collect_explosions();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].0, 0);
        assert_eq!(collected[1].0, 1);

        // 收集后应该为空
        let empty = manager.collect_explosions();
        assert!(empty.is_empty());
    }
}
