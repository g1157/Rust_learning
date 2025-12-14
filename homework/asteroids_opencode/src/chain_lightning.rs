//! 链式离子炮效果模块
//!
//! 实现链式离子炮的视觉效果和目标查找逻辑。
//!
//! ## 功能
//! - 链式闪电/电弧视觉效果
//! - 目标查找（最近邻搜索）
//! - 伤害衰减计算
//! - 被击中目标的高亮闪烁效果

#![allow(dead_code)] // 模块正在集成中

use macroquad::prelude::*;

use crate::asteroid::Asteroid;
use crate::constants::chain_ion;
use crate::ufo::Ufo;

/// 单个电弧段（两点之间的闪电）
pub struct ChainSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub spawned_at: f32,
    pub lifetime: f32,
    pub color: Color,
}

/// 一次链式攻击事件（包含多个电弧段和端点闪烁）
pub struct ChainLightning {
    segments: Vec<ChainSegment>,
    /// 所有被链接的节点位置（用于闪烁效果）
    nodes: Vec<Vec2>,
    /// 闪烁效果结束时间
    flash_until: f32,
    /// 整个效果过期时间
    expires_at: f32,
    /// 基础颜色
    color: Color,
}

/// 链式闪电效果管理器
pub struct ChainLightningManager {
    effects: Vec<ChainLightning>,
}

impl ChainLightningManager {
    pub fn new() -> Self {
        Self {
            effects: Vec::with_capacity(16),
        }
    }

    /// 更新所有效果，清理过期的
    pub fn update(&mut self, now: f32) {
        self.effects.retain(|fx| fx.expires_at > now);
    }

    /// 生成一条链式闪电路径
    ///
    /// `nodes` 包含所有被链接的位置（按顺序）
    pub fn spawn_path(&mut self, nodes: Vec<Vec2>, now: f32, color: Color) {
        if nodes.len() < 2 {
            return;
        }

        let mut segments = Vec::with_capacity(nodes.len() - 1);
        for window in nodes.windows(2) {
            segments.push(ChainSegment {
                start: window[0],
                end: window[1],
                spawned_at: now,
                lifetime: chain_ion::ARC_LIFETIME,
                color,
            });
        }

        let flash_until = now + chain_ion::FLASH_DURATION;
        let expires_at = now + chain_ion::ARC_LIFETIME;

        self.effects.push(ChainLightning {
            segments,
            nodes,
            flash_until,
            expires_at,
            color,
        });
    }

    /// 绘制所有活跃的链式闪电效果
    pub fn draw(&self, offset: Vec2, now: f32) {
        for fx in self.effects.iter() {
            // 绘制电弧段
            for seg in fx.segments.iter() {
                draw_arc(seg, offset, now);
            }

            // 绘制端点闪烁效果
            if now < fx.flash_until {
                let flash_t = 1.0 - ((fx.flash_until - now) / chain_ion::FLASH_DURATION);
                let flash_alpha = (0.65 - flash_t * 0.5).max(0.0);
                let flash_radius = 12.0 + 8.0 * (flash_t * std::f32::consts::PI).sin().abs();
                let flash_color = Color::new(fx.color.r, fx.color.g, fx.color.b, flash_alpha);

                for node in fx.nodes.iter() {
                    let p = *node + offset;
                    // 外环发光
                    draw_circle(p.x, p.y, flash_radius * 1.5, Color::new(1.0, 1.0, 1.0, flash_alpha * 0.3));
                    // 主闪烁圈
                    draw_circle(p.x, p.y, flash_radius, flash_color);
                    // 核心高亮
                    draw_circle(p.x, p.y, flash_radius * 0.4, Color::new(1.0, 1.0, 1.0, flash_alpha * 0.8));
                }
            }
        }
    }

    /// 清空所有效果
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// 获取当前活跃效果数量
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.effects.len()
    }
}

impl Default for ChainLightningManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 链式目标类型（用于统一处理小行星和 UFO）
#[derive(Clone, Copy)]
pub enum ChainTarget {
    Asteroid(usize),
    Ufo(usize),
}

impl ChainTarget {
    pub fn index(&self) -> usize {
        match self {
            ChainTarget::Asteroid(i) => *i,
            ChainTarget::Ufo(i) => *i,
        }
    }
}

/// 可作为链式目标的实体信息
pub struct TargetInfo {
    pub pos: Vec2,
    pub target: ChainTarget,
    pub collided: bool,
}

/// 贪婪最近邻搜索，查找链式攻击的目标
///
/// 从原点位置开始，逐跳查找最近的有效目标。
///
/// # 参数
/// - `origin_pos`: 起始位置
/// - `origin_asteroid_idx`: 原始命中的小行星索引（排除）
/// - `asteroids`: 所有小行星
/// - `ufos`: 所有 UFO
/// - `max_additional`: 最多额外命中的目标数
/// - `max_distance`: 每跳的最大搜索距离
///
/// # 返回
/// 按跳跃顺序排列的目标列表
pub fn find_chain_targets(
    origin_pos: Vec2,
    origin_asteroid_idx: usize,
    asteroids: &[Asteroid],
    ufos: &[Ufo],
    max_additional: usize,
    max_distance: f32,
) -> Vec<ChainTarget> {
    if max_additional == 0 {
        return Vec::new();
    }

    let mut targets = Vec::with_capacity(max_additional);
    let mut current_pos = origin_pos;
    let mut excluded_asteroids = vec![origin_asteroid_idx];
    let mut excluded_ufos: Vec<usize> = Vec::new();

    for _ in 0..max_additional {
        // 收集所有候选目标
        let mut candidates: Vec<(ChainTarget, f32, Vec2)> = Vec::new();

        // 小行星候选
        for (i, asteroid) in asteroids.iter().enumerate() {
            if asteroid.collided || excluded_asteroids.contains(&i) {
                continue;
            }
            let dist_sq = (asteroid.pos - current_pos).length_squared();
            if dist_sq <= max_distance * max_distance {
                candidates.push((ChainTarget::Asteroid(i), dist_sq, asteroid.pos));
            }
        }

        // UFO 候选
        for (i, ufo) in ufos.iter().enumerate() {
            if ufo.destroyed || excluded_ufos.contains(&i) {
                continue;
            }
            let dist_sq = (ufo.pos - current_pos).length_squared();
            if dist_sq <= max_distance * max_distance {
                candidates.push((ChainTarget::Ufo(i), dist_sq, ufo.pos));
            }
        }

        // 选择最近的目标
        if let Some((target, _dist_sq, pos)) = candidates
            .into_iter()
            .min_by(|(_, a, _), (_, b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            match target {
                ChainTarget::Asteroid(i) => excluded_asteroids.push(i),
                ChainTarget::Ufo(i) => excluded_ufos.push(i),
            }
            current_pos = pos;
            targets.push(target);
        } else {
            break;
        }
    }

    targets
}

/// 仅查找小行星目标的简化版本
pub fn find_chain_asteroid_targets(
    origin_pos: Vec2,
    origin_idx: usize,
    asteroids: &[Asteroid],
    max_additional: usize,
    max_distance: f32,
) -> Vec<usize> {
    if max_additional == 0 {
        return Vec::new();
    }

    let mut targets = Vec::with_capacity(max_additional);
    let mut current_pos = origin_pos;
    let mut excluded = vec![origin_idx];

    for _ in 0..max_additional {
        if let Some((idx, _dist_sq)) = asteroids
            .iter()
            .enumerate()
            .filter(|(i, a)| !a.collided && !excluded.contains(i))
            .filter_map(|(i, a)| {
                let dist_sq = (a.pos - current_pos).length_squared();
                if dist_sq <= max_distance * max_distance {
                    Some((i, dist_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            excluded.push(idx);
            current_pos = asteroids[idx].pos;
            targets.push(idx);
        } else {
            break;
        }
    }

    targets
}

/// 获取指定跳数的伤害比例
///
/// `hop` 是 0-based 的额外命中索引（0 表示第二个目标）
pub fn damage_ratio(hop: usize) -> f32 {
    chain_ion::DAMAGE_RATIOS
        .get(hop + 1) // +1 因为索引 0 是初始命中
        .copied()
        .unwrap_or_else(|| *chain_ion::DAMAGE_RATIOS.last().unwrap_or(&1.0))
}

/// 绘制单个电弧段
fn draw_arc(seg: &ChainSegment, offset: Vec2, now: f32) {
    let age = (now - seg.spawned_at).max(0.0);
    let t = (age / seg.lifetime).clamp(0.0, 1.0);
    let alpha = 1.0 - t;

    let dir = seg.end - seg.start;
    let normal = Vec2::new(-dir.y, dir.x).normalize_or_zero();

    // 基础颜色（蓝白色调）
    let base_color = Color::new(
        seg.color.r * 0.7 + 0.3,
        seg.color.g * 0.7 + 0.3,
        1.0,
        alpha * 0.9,
    );

    // 外发光颜色
    let glow_color = Color::new(base_color.r, base_color.g, base_color.b, alpha * 0.3);

    let steps = chain_ion::JITTER_STEPS.max(1);
    let mut last = seg.start + offset;

    // 绘制外发光层
    let mut glow_last = last;
    for i in 1..=steps {
        let progress = i as f32 / steps as f32;
        let wave = (progress * 12.0 + now * 15.0).sin();
        let amp = chain_ion::JITTER_AMPLITUDE * (1.0 - (progress - 0.5).abs() * 2.0);
        let base = seg.start.lerp(seg.end, progress);
        let jittered = base + normal * amp * wave;
        let next = jittered + offset;

        draw_line(
            glow_last.x,
            glow_last.y,
            next.x,
            next.y,
            chain_ion::LINE_WIDTH * 3.0,
            glow_color,
        );
        glow_last = next;
    }

    // 绘制主电弧
    for i in 1..=steps {
        let progress = i as f32 / steps as f32;
        // 使用不同的相位和频率创建动态效果
        let wave = (progress * 12.0 + now * 15.0).sin();
        let wave2 = (progress * 8.0 + now * 25.0 + 0.5).cos() * 0.5;
        let combined_wave = wave + wave2;

        // 中间部分抖动更大
        let amp = chain_ion::JITTER_AMPLITUDE * (1.0 - (progress - 0.5).abs() * 2.0);
        let base = seg.start.lerp(seg.end, progress);
        let jittered = base + normal * amp * combined_wave;
        let next = jittered + offset;

        draw_line(
            last.x,
            last.y,
            next.x,
            next.y,
            chain_ion::LINE_WIDTH,
            base_color,
        );
        last = next;
    }

    // 绘制核心高亮线
    let core_color = Color::new(1.0, 1.0, 1.0, alpha * 0.6);
    let mut core_last = seg.start + offset;
    for i in 1..=steps {
        let progress = i as f32 / steps as f32;
        let wave = (progress * 12.0 + now * 15.0).sin();
        let amp = chain_ion::JITTER_AMPLITUDE * 0.5 * (1.0 - (progress - 0.5).abs() * 2.0);
        let base = seg.start.lerp(seg.end, progress);
        let jittered = base + normal * amp * wave;
        let next = jittered + offset;

        draw_line(
            core_last.x,
            core_last.y,
            next.x,
            next.y,
            chain_ion::LINE_WIDTH * 0.4,
            core_color,
        );
        core_last = next;
    }

    // 在端点绘制小亮点
    let point_alpha = alpha * 0.8;
    let point_color = Color::new(1.0, 1.0, 1.0, point_alpha);
    let start_p = seg.start + offset;
    let end_p = seg.end + offset;
    draw_circle(start_p.x, start_p.y, 3.0, point_color);
    draw_circle(end_p.x, end_p.y, 3.0, point_color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asteroid::MAX_VERTICES;

    fn make_asteroid(pos: Vec2, collided: bool) -> Asteroid {
        Asteroid {
            pos,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 30.0,
            sides: 6,
            collided,
            vertex_offsets: [1.0; MAX_VERTICES],
            asteroid_type: crate::asteroid::AsteroidType::Normal,
        }
    }

    #[test]
    fn test_find_chain_targets_empty() {
        let asteroids: Vec<Asteroid> = vec![];
        let targets = find_chain_asteroid_targets(Vec2::ZERO, 999, &asteroids, 2, 100.0);
        assert!(targets.is_empty());
    }

    #[test]
    fn test_find_chain_targets_single() {
        let asteroids = vec![
            make_asteroid(Vec2::new(0.0, 0.0), true),   // 原始目标（已碰撞）
            make_asteroid(Vec2::new(50.0, 0.0), false), // 可链接目标
        ];
        let targets = find_chain_asteroid_targets(Vec2::ZERO, 0, &asteroids, 2, 100.0);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], 1);
    }

    #[test]
    fn test_find_chain_targets_nearest_first() {
        let asteroids = vec![
            make_asteroid(Vec2::new(0.0, 0.0), true),    // 原始目标
            make_asteroid(Vec2::new(100.0, 0.0), false), // 较远
            make_asteroid(Vec2::new(50.0, 0.0), false),  // 最近
        ];
        let targets = find_chain_asteroid_targets(Vec2::ZERO, 0, &asteroids, 2, 150.0);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0], 2); // 先选最近的
        assert_eq!(targets[1], 1); // 再选较远的
    }

    #[test]
    fn test_find_chain_targets_respects_range() {
        let asteroids = vec![
            make_asteroid(Vec2::new(0.0, 0.0), true),    // 原始目标
            make_asteroid(Vec2::new(200.0, 0.0), false), // 超出范围
            make_asteroid(Vec2::new(50.0, 0.0), false),  // 在范围内
        ];
        let targets = find_chain_asteroid_targets(Vec2::ZERO, 0, &asteroids, 2, 100.0);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], 2);
    }

    #[test]
    fn test_damage_ratio() {
        assert!((damage_ratio(0) - 0.7).abs() < 0.001); // 第二个目标
        assert!((damage_ratio(1) - 0.5).abs() < 0.001); // 第三个目标
        assert!((damage_ratio(2) - 0.5).abs() < 0.001); // 超出范围，使用最后一个
    }

    #[test]
    fn test_chain_lightning_manager_update() {
        let mut manager = ChainLightningManager::new();
        let nodes = vec![Vec2::ZERO, Vec2::new(100.0, 0.0)];
        manager.spawn_path(nodes, 0.0, WHITE);

        assert_eq!(manager.count(), 1);

        // 更新到过期时间之后
        manager.update(chain_ion::ARC_LIFETIME + 0.1);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_chain_lightning_manager_spawn_path_minimum() {
        let mut manager = ChainLightningManager::new();

        // 单点不应该创建效果
        manager.spawn_path(vec![Vec2::ZERO], 0.0, WHITE);
        assert_eq!(manager.count(), 0);

        // 两点应该创建效果
        manager.spawn_path(vec![Vec2::ZERO, Vec2::new(100.0, 0.0)], 0.0, WHITE);
        assert_eq!(manager.count(), 1);
    }
}
