//! 实体插值系统
//!
//! 用于平滑渲染远程实体（其他玩家、小行星、子弹），减少网络抖动。
//!
//! ## 工作原理
//!
//! 1. 服务器发送带时间戳的游戏状态
//! 2. 客户端将状态快照存入缓冲区
//! 3. 渲染时，在历史快照间进行插值，延迟约 100ms
//! 4. 这样即使网络有抖动，渲染也是平滑的

use macroquad::prelude::{Vec2, screen_height, screen_width};
use std::collections::{HashMap, VecDeque};

use crate::network::{AsteroidState, BulletState, PlayerState};

// ============================================================================
// 配置
// ============================================================================

/// 插值系统配置
#[derive(Debug, Clone)]
pub struct InterpConfig {
    /// 渲染延迟（毫秒），典型值 100ms
    pub render_delay_ms: f64,
    /// 历史快照保留时间（秒）
    pub history_secs: f64,
}

impl Default for InterpConfig {
    fn default() -> Self {
        Self {
            render_delay_ms: 100.0,
            history_secs: 1.5,
        }
    }
}

// ============================================================================
// 快照数据结构
// ============================================================================

/// 带时间戳的快照
#[derive(Debug, Clone)]
pub struct Snapshot<T> {
    /// 服务器时间戳（秒）
    pub server_time: f64,
    /// 状态数据
    pub state: T,
}

/// 插值缓冲区
#[derive(Debug, Clone)]
pub struct InterpBuffer<T> {
    /// 历史快照队列（按时间排序，旧的在前）
    pub history: VecDeque<Snapshot<T>>,
}

impl<T: Clone> InterpBuffer<T> {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
        }
    }

    /// 添加新快照
    pub fn push(&mut self, server_time: f64, state: T) {
        self.history.push_back(Snapshot { server_time, state });
    }

    /// 清理过期快照
    pub fn prune(&mut self, current_server_time: f64, history_secs: f64) {
        let cutoff = current_server_time - history_secs;
        while self.history.len() > 2 {
            if let Some(front) = self.history.front() {
                if front.server_time < cutoff {
                    self.history.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// 查找用于插值的两个快照（before, after）
    /// 返回 (before, after, t) 其中 t 是插值因子 [0, 1]
    pub fn find_interp_pair(&self, target_time: f64) -> Option<(&Snapshot<T>, &Snapshot<T>, f32)> {
        if self.history.len() < 2 {
            return None;
        }

        // 寻找 target_time 所在的区间
        for i in 0..self.history.len() - 1 {
            let before = &self.history[i];
            let after = &self.history[i + 1];

            if before.server_time <= target_time && target_time <= after.server_time {
                let duration = after.server_time - before.server_time;
                let t = if duration > 0.0 {
                    ((target_time - before.server_time) / duration) as f32
                } else {
                    0.0
                };
                return Some((before, after, t.clamp(0.0, 1.0)));
            }
        }

        // 如果 target_time 超过最新快照，使用最后两个进行外推
        if target_time > self.history.back()?.server_time {
            let len = self.history.len();
            let before = &self.history[len - 2];
            let after = &self.history[len - 1];
            let duration = after.server_time - before.server_time;
            let t = if duration > 0.0 {
                ((target_time - before.server_time) / duration) as f32
            } else {
                1.0
            };
            // 限制外推范围，避免过度预测
            return Some((before, after, t.clamp(0.0, 1.5)));
        }

        None
    }

    /// 获取最新快照
    pub fn latest(&self) -> Option<&Snapshot<T>> {
        self.history.back()
    }
}

impl<T: Clone> Default for InterpBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 远程实体状态（用于插值存储）
// ============================================================================

/// 远程玩家状态快照
#[derive(Debug, Clone)]
pub struct RemotePlayerState {
    pub id: String,
    pub pos: Vec2,
    pub rot: f32, // 度数（与 ship.rot 一致）
    pub vel: Vec2,
    pub lives: u32,
    pub score: u32,
    pub alive: bool,
}

impl From<&PlayerState> for RemotePlayerState {
    fn from(ps: &PlayerState) -> Self {
        Self {
            id: ps.id.clone(),
            pos: Vec2::new(ps.x, ps.y),
            rot: ps.angle,
            vel: Vec2::new(ps.vel_x, ps.vel_y),
            lives: ps.lives,
            score: ps.score,
            alive: ps.alive,
        }
    }
}

/// 远程小行星状态快照
#[derive(Debug, Clone)]
pub struct RemoteAsteroidState {
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: u32,
    pub angle: f32, // 弧度
}

impl From<&AsteroidState> for RemoteAsteroidState {
    fn from(ast: &AsteroidState) -> Self {
        Self {
            id: ast.id,
            pos: Vec2::new(ast.x, ast.y),
            vel: Vec2::new(ast.vx, ast.vy),
            size: ast.size,
            angle: ast.angle,
        }
    }
}

/// 远程子弹状态快照
#[derive(Debug, Clone)]
pub struct RemoteBulletState {
    pub id: u32,
    pub owner_id: String,
    pub pos: Vec2,
    pub vel: Vec2,
}

impl From<&BulletState> for RemoteBulletState {
    fn from(bs: &BulletState) -> Self {
        Self {
            id: bs.id,
            owner_id: bs.owner_id.clone(),
            pos: Vec2::new(bs.x, bs.y),
            vel: Vec2::new(bs.vx, bs.vy),
        }
    }
}

// ============================================================================
// 插值结果
// ============================================================================

/// 插值后的玩家状态（用于渲染）
#[derive(Debug, Clone)]
#[allow(dead_code)] // 保留字段供未来功能使用
pub struct InterpolatedPlayer {
    pub id: String,
    pub pos: Vec2,
    pub rot: f32,
    pub vel: Vec2,
    pub lives: u32,
    pub score: u32,
    pub alive: bool,
}

/// 插值后的小行星状态（用于渲染）
#[derive(Debug, Clone)]
#[allow(dead_code)] // 保留字段供未来功能使用
pub struct InterpolatedAsteroid {
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: u32,
    pub angle: f32,
}

/// 插值后的子弹状态（用于渲染）
#[derive(Debug, Clone)]
#[allow(dead_code)] // 保留字段供未来功能使用
pub struct InterpolatedBullet {
    pub id: u32,
    pub owner_id: String,
    pub pos: Vec2,
    pub vel: Vec2,
}

/// 插值后的世界状态
#[derive(Debug, Clone, Default)]
pub struct InterpolatedWorld {
    pub remote_players: Vec<InterpolatedPlayer>,
    pub asteroids: Vec<InterpolatedAsteroid>,
    pub bullets: Vec<InterpolatedBullet>,
}

/// 插值系统的调试信息（用于网络诊断 HUD）
#[derive(Debug, Clone)]
pub struct InterpDebugInfo {
    /// 远程玩家缓冲区数量
    pub player_buffers: usize,
    /// 小行星缓冲区数量
    pub asteroid_buffers: usize,
    /// 子弹缓冲区数量
    pub bullet_buffers: usize,
    /// 平均玩家快照数量
    pub avg_player_snapshots: f32,
    /// 平均子弹快照数量
    pub avg_bullet_snapshots: f32,
    /// 渲染延迟（毫秒）
    pub render_delay_ms: f64,
}

// ============================================================================
// 插值管理器
// ============================================================================

/// 插值系统管理器
///
/// 负责存储历史快照并提供插值采样
pub struct InterpolationManager {
    pub config: InterpConfig,
    /// 服务器时间偏移量 (local_time - server_time)
    server_offset: Option<f64>,
    /// 远程玩家缓冲区
    players: HashMap<String, InterpBuffer<RemotePlayerState>>,
    /// 小行星缓冲区
    asteroids: HashMap<u32, InterpBuffer<RemoteAsteroidState>>,
    /// 子弹缓冲区
    bullets: HashMap<u32, InterpBuffer<RemoteBulletState>>,
    /// 最后一次记录的服务器时间
    last_server_time: f64,
}

impl InterpolationManager {
    pub fn new(config: InterpConfig) -> Self {
        Self {
            config,
            server_offset: None,
            players: HashMap::new(),
            asteroids: HashMap::new(),
            bullets: HashMap::new(),
            last_server_time: 0.0,
        }
    }

    /// 校准时钟偏移
    ///
    /// 在首次收到服务器时间戳时调用，建立本地时间与服务器时间的对应关系
    pub fn align_clock(&mut self, server_ts_ms: i64, local_now: f64) {
        let server_time = server_ts_ms as f64 / 1000.0;
        if self.server_offset.is_none() {
            self.server_offset = Some(local_now - server_time);
        }
        self.last_server_time = server_time;
    }

    /// 获取服务器时间偏移
    #[allow(dead_code)] // 保留供延迟补偿功能使用
    pub fn server_offset(&self) -> Option<f64> {
        self.server_offset
    }

    /// 将本地时间转换为服务器时间
    #[allow(dead_code)] // 保留供延迟补偿功能使用
    pub fn local_to_server_time(&self, local_time: f64) -> Option<f64> {
        self.server_offset.map(|offset| local_time - offset)
    }

    /// 记录服务器快照
    ///
    /// 将服务器发送的游戏状态存入插值缓冲区
    pub fn record_server_snapshot(
        &mut self,
        server_ts_ms: i64,
        players: &[PlayerState],
        asteroids: &[AsteroidState],
        bullets: &[BulletState],
        local_player_id: Option<&str>,
    ) {
        let server_time = server_ts_ms as f64 / 1000.0;

        // 防止乱序包：如果时间戳比上次还早，跳过
        if server_time < self.last_server_time - 0.5 {
            // 允许 0.5 秒的容差（处理轻微乱序）
            return;
        }
        self.last_server_time = server_time;

        // 收集当前帧存在的实体 ID
        let current_player_ids: std::collections::HashSet<_> = players
            .iter()
            .filter(|p| Some(p.id.as_str()) != local_player_id)
            .map(|p| p.id.clone())
            .collect();
        let current_asteroid_ids: std::collections::HashSet<_> =
            asteroids.iter().map(|a| a.id).collect();
        let current_bullet_ids: std::collections::HashSet<_> =
            bullets.iter().map(|b| b.id).collect();

        // 记录远程玩家（排除本地玩家）
        for ps in players {
            if Some(ps.id.as_str()) == local_player_id {
                continue; // 跳过本地玩家，本地玩家使用预测
            }
            let state = RemotePlayerState::from(ps);
            self.players
                .entry(ps.id.clone())
                .or_default()
                .push(server_time, state);
        }

        // 记录小行星
        for ast in asteroids {
            let state = RemoteAsteroidState::from(ast);
            self.asteroids
                .entry(ast.id)
                .or_default()
                .push(server_time, state);
        }

        // 记录子弹
        for bs in bullets {
            let state = RemoteBulletState::from(bs);
            self.bullets
                .entry(bs.id)
                .or_default()
                .push(server_time, state);
        }

        // 清理过期数据和已销毁的实体
        self.prune_old_snapshots(
            server_time,
            &current_player_ids,
            &current_asteroid_ids,
            &current_bullet_ids,
        );
    }

    /// 清理过期快照和已销毁的实体
    fn prune_old_snapshots(
        &mut self,
        current_server_time: f64,
        current_player_ids: &std::collections::HashSet<String>,
        current_asteroid_ids: &std::collections::HashSet<u32>,
        current_bullet_ids: &std::collections::HashSet<u32>,
    ) {
        let history_secs = self.config.history_secs;
        let render_delay_secs = self.config.render_delay_ms / 1000.0;
        // 实体销毁后保留一个渲染延迟周期，确保插值平滑结束
        let grace_period = render_delay_secs + 0.1;

        // 清理玩家快照
        for buffer in self.players.values_mut() {
            buffer.prune(current_server_time, history_secs);
        }
        // 清理小行星快照
        for buffer in self.asteroids.values_mut() {
            buffer.prune(current_server_time, history_secs);
        }
        // 清理子弹快照
        for buffer in self.bullets.values_mut() {
            buffer.prune(current_server_time, history_secs);
        }

        // 移除空缓冲区
        self.players.retain(|_, buf| !buf.history.is_empty());
        self.asteroids.retain(|_, buf| !buf.history.is_empty());
        self.bullets.retain(|_, buf| !buf.history.is_empty());

        // 移除已销毁的实体（在宽限期后）
        self.players.retain(|id, buf| {
            if current_player_ids.contains(id) {
                return true;
            }
            // 检查最后快照是否在宽限期内
            buf.latest()
                .map(|s| current_server_time - s.server_time < grace_period)
                .unwrap_or(false)
        });
        self.asteroids.retain(|id, buf| {
            if current_asteroid_ids.contains(id) {
                return true;
            }
            buf.latest()
                .map(|s| current_server_time - s.server_time < grace_period)
                .unwrap_or(false)
        });
        self.bullets.retain(|id, buf| {
            if current_bullet_ids.contains(id) {
                return true;
            }
            buf.latest()
                .map(|s| current_server_time - s.server_time < grace_period)
                .unwrap_or(false)
        });
    }

    /// 采样插值后的世界状态
    ///
    /// 在渲染前调用，获取平滑的实体状态
    pub fn sample_world(&self, local_now: f64) -> InterpolatedWorld {
        let Some(offset) = self.server_offset else {
            return InterpolatedWorld::default();
        };

        // 计算目标服务器时间 = 当前本地时间 - 偏移 - 渲染延迟
        let render_delay_secs = self.config.render_delay_ms / 1000.0;
        let target_server_time = local_now - offset - render_delay_secs;

        let mut world = InterpolatedWorld::default();

        // 插值远程玩家
        for buffer in self.players.values() {
            if let Some(player) = self.sample_player(buffer, target_server_time) {
                world.remote_players.push(player);
            }
        }

        // 插值小行星
        for buffer in self.asteroids.values() {
            if let Some(asteroid) = self.sample_asteroid(buffer, target_server_time) {
                world.asteroids.push(asteroid);
            }
        }

        // 插值子弹
        for buffer in self.bullets.values() {
            if let Some(bullet) = self.sample_bullet(buffer, target_server_time) {
                world.bullets.push(bullet);
            }
        }

        world
    }

    /// 采样单个玩家
    fn sample_player(
        &self,
        buffer: &InterpBuffer<RemotePlayerState>,
        target_time: f64,
    ) -> Option<InterpolatedPlayer> {
        if let Some((before, after, t)) = buffer.find_interp_pair(target_time) {
            Some(InterpolatedPlayer {
                id: after.state.id.clone(),
                pos: lerp_vec2_wrap(before.state.pos, after.state.pos, t),
                rot: lerp_angle(before.state.rot, after.state.rot, t),
                vel: lerp_vec2(before.state.vel, after.state.vel, t),
                lives: after.state.lives,
                score: after.state.score,
                alive: after.state.alive,
            })
        } else {
            // 没有足够快照时，使用最新状态
            buffer.latest().map(|latest| InterpolatedPlayer {
                id: latest.state.id.clone(),
                pos: latest.state.pos,
                rot: latest.state.rot,
                vel: latest.state.vel,
                lives: latest.state.lives,
                score: latest.state.score,
                alive: latest.state.alive,
            })
        }
    }

    /// 采样单个小行星
    fn sample_asteroid(
        &self,
        buffer: &InterpBuffer<RemoteAsteroidState>,
        target_time: f64,
    ) -> Option<InterpolatedAsteroid> {
        if let Some((before, after, t)) = buffer.find_interp_pair(target_time) {
            Some(InterpolatedAsteroid {
                id: after.state.id,
                pos: lerp_vec2_wrap(before.state.pos, after.state.pos, t),
                vel: lerp_vec2(before.state.vel, after.state.vel, t),
                size: after.state.size,
                angle: lerp_angle(before.state.angle, after.state.angle, t),
            })
        } else {
            buffer.latest().map(|latest| InterpolatedAsteroid {
                id: latest.state.id,
                pos: latest.state.pos,
                vel: latest.state.vel,
                size: latest.state.size,
                angle: latest.state.angle,
            })
        }
    }

    /// 采样单个子弹
    fn sample_bullet(
        &self,
        buffer: &InterpBuffer<RemoteBulletState>,
        target_time: f64,
    ) -> Option<InterpolatedBullet> {
        if let Some((before, after, t)) = buffer.find_interp_pair(target_time) {
            Some(InterpolatedBullet {
                id: after.state.id,
                owner_id: after.state.owner_id.clone(),
                pos: lerp_vec2_wrap(before.state.pos, after.state.pos, t),
                vel: lerp_vec2(before.state.vel, after.state.vel, t),
            })
        } else {
            buffer.latest().map(|latest| InterpolatedBullet {
                id: latest.state.id,
                owner_id: latest.state.owner_id.clone(),
                pos: latest.state.pos,
                vel: latest.state.vel,
            })
        }
    }

    /// 延迟补偿：回溯玩家位置（用于命中判定）
    #[allow(dead_code)]
    pub fn rewind_player_at(&self, player_id: &str, server_time: f64) -> Option<RemotePlayerState> {
        let buffer = self.players.get(player_id)?;
        if let Some((before, after, t)) = buffer.find_interp_pair(server_time) {
            Some(RemotePlayerState {
                id: after.state.id.clone(),
                pos: lerp_vec2_wrap(before.state.pos, after.state.pos, t),
                rot: lerp_angle(before.state.rot, after.state.rot, t),
                vel: lerp_vec2(before.state.vel, after.state.vel, t),
                lives: after.state.lives,
                score: after.state.score,
                alive: after.state.alive,
            })
        } else {
            buffer.latest().map(|s| s.state.clone())
        }
    }

    /// 重置所有缓冲区（断线重连时调用）
    #[allow(dead_code)] // 保留供断线重连功能使用
    pub fn reset(&mut self) {
        self.server_offset = None;
        self.players.clear();
        self.asteroids.clear();
        self.bullets.clear();
        self.last_server_time = 0.0;
    }

    /// 获取插值缓冲区的调试信息（用于网络诊断 HUD）
    pub fn debug_info(&self) -> Option<InterpDebugInfo> {
        self.server_offset?;

        let player_buffers = self.players.len();
        let asteroid_buffers = self.asteroids.len();
        let bullet_buffers = self.bullets.len();

        // 计算平均玩家快照数量
        let avg_player_snapshots = if self.players.is_empty() {
            0.0
        } else {
            self.players
                .values()
                .map(|buf| buf.history.len())
                .sum::<usize>() as f32
                / self.players.len() as f32
        };

        // 计算平均子弹快照数量
        let avg_bullet_snapshots = if self.bullets.is_empty() {
            0.0
        } else {
            self.bullets
                .values()
                .map(|buf| buf.history.len())
                .sum::<usize>() as f32
                / self.bullets.len() as f32
        };

        Some(InterpDebugInfo {
            player_buffers,
            asteroid_buffers,
            bullet_buffers,
            avg_player_snapshots,
            avg_bullet_snapshots,
            render_delay_ms: self.config.render_delay_ms,
        })
    }
}

impl Default for InterpolationManager {
    fn default() -> Self {
        Self::new(InterpConfig::default())
    }
}

// ============================================================================
// 辅助函数：插值
// ============================================================================

/// 线性插值 Vec2
fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

/// 带屏幕环绕的 Vec2 插值
///
/// 处理从屏幕一边穿越到另一边的情况，选择最短路径
fn lerp_vec2_wrap(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    let sw = screen_width();
    let sh = screen_height();

    // X 轴处理
    let mut dx = b.x - a.x;
    if dx.abs() > sw / 2.0 {
        // 穿越边界，选择更短的路径
        if dx > 0.0 {
            dx -= sw;
        } else {
            dx += sw;
        }
    }

    // Y 轴处理
    let mut dy = b.y - a.y;
    if dy.abs() > sh / 2.0 {
        if dy > 0.0 {
            dy -= sh;
        } else {
            dy += sh;
        }
    }

    let mut result = Vec2::new(a.x + dx * t, a.y + dy * t);

    // 确保结果在屏幕范围内
    if result.x < 0.0 {
        result.x += sw;
    } else if result.x > sw {
        result.x -= sw;
    }
    if result.y < 0.0 {
        result.y += sh;
    } else if result.y > sh {
        result.y -= sh;
    }

    result
}

/// 角度插值（度数），处理环绕
///
/// 游戏中角度使用度数表示（0-360），此函数选择最短路径进行插值
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;

    // 规范化差值到 [-180, 180]
    while diff > 180.0 {
        diff -= 360.0;
    }
    while diff < -180.0 {
        diff += 360.0;
    }

    a + diff * t
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_vec2() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        let mid = lerp_vec2(a, b, 0.5);
        assert!((mid.x - 5.0).abs() < 0.001);
        assert!((mid.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_lerp_angle_simple() {
        let a = 0.0;
        let b = 1.0;
        let mid = lerp_angle(a, b, 0.5);
        assert!((mid - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_lerp_angle_wrap() {
        // 从 350° 到 10°（应该走短路，经过 0°）
        let a = 350.0;
        let b = 10.0;
        let mid = lerp_angle(a, b, 0.5);
        // 中点应该在 0° 附近（或 360°）
        assert!(
            mid < 10.0 || mid > 350.0,
            "mid={} should be near 0/360",
            mid
        );
    }

    #[test]
    fn test_lerp_angle_wrap_negative() {
        // 从 10° 到 350°（应该走短路，经过 0°）
        let a = 10.0;
        let b = 350.0;
        let mid = lerp_angle(a, b, 0.5);
        // 中点应该在 0° 附近（可能是负数，如 -10° 或 360°）
        assert!(
            mid < 10.0 || mid > 350.0 || mid < 0.0,
            "mid={} should be near 0/360",
            mid
        );
    }

    #[test]
    fn test_interp_buffer_push_and_latest() {
        let mut buffer = InterpBuffer::<f32>::new();
        buffer.push(1.0, 10.0);
        buffer.push(2.0, 20.0);

        let latest = buffer.latest().unwrap();
        assert!((latest.server_time - 2.0).abs() < 0.001);
        assert!((latest.state - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_interp_buffer_find_pair() {
        let mut buffer = InterpBuffer::<f32>::new();
        buffer.push(1.0, 10.0);
        buffer.push(2.0, 20.0);
        buffer.push(3.0, 30.0);

        // 目标时间 1.5，应该在第一和第二快照之间
        let (before, after, t) = buffer.find_interp_pair(1.5).unwrap();
        assert!((before.server_time - 1.0).abs() < 0.001);
        assert!((after.server_time - 2.0).abs() < 0.001);
        assert!((t - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_interp_buffer_prune() {
        let mut buffer = InterpBuffer::<f32>::new();
        buffer.push(1.0, 10.0);
        buffer.push(2.0, 20.0);
        buffer.push(3.0, 30.0);
        buffer.push(4.0, 40.0);

        // 清理 history_secs=1.5 之前的，current=4.0，cutoff=2.5
        buffer.prune(4.0, 1.5);

        // 应该保留 3.0 和 4.0（至少保留 2 个）
        assert!(buffer.history.len() >= 2);
        assert!(buffer.history.front().unwrap().server_time >= 2.5);
    }

    #[test]
    fn test_interpolation_manager_align_clock() {
        let mut manager = InterpolationManager::new(InterpConfig::default());

        // 服务器时间 1000ms，本地时间 1.5s
        manager.align_clock(1000, 1.5);

        assert!(manager.server_offset.is_some());
        let offset = manager.server_offset.unwrap();
        // offset = local_now - server_time = 1.5 - 1.0 = 0.5
        assert!((offset - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_interpolation_manager_local_to_server() {
        let mut manager = InterpolationManager::new(InterpConfig::default());
        manager.align_clock(1000, 1.5);

        // local_time=2.0 -> server_time = 2.0 - 0.5 = 1.5
        let server_time = manager.local_to_server_time(2.0).unwrap();
        assert!((server_time - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_config_default() {
        let config = InterpConfig::default();
        assert!((config.render_delay_ms - 100.0).abs() < 0.001);
        assert!((config.history_secs - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_remote_player_state_from() {
        let ps = PlayerState {
            id: "player1".to_string(),
            x: 100.0,
            y: 200.0,
            angle: 1.5,
            vel_x: 10.0,
            vel_y: 20.0,
            lives: 3,
            score: 1000,
            alive: true,
        };
        let remote = RemotePlayerState::from(&ps);
        assert_eq!(remote.id, "player1");
        assert!((remote.pos.x - 100.0).abs() < 0.001);
        assert!((remote.pos.y - 200.0).abs() < 0.001);
    }
}
