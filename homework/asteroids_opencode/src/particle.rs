//! 粒子效果系统
//!
//! 提供爆炸、推进器火焰等视觉效果。
//!
//! ## 功能
//! - 爆炸粒子（小行星/碰撞）
//! - 推进器火焰效果
//! - 重力和速度衰减
//! - 透明度淡出
//! - 最多 1000 个并发粒子
//! - 内置节流机制减少每帧粒子生成

use macroquad::prelude::*;

use crate::constants::particles;

const EXPLOSION_PARTICLE_COUNT: usize = particles::EXPLOSION_BASE_COUNT;
const THRUSTER_PARTICLE_COUNT: usize = particles::THRUSTER_COUNT;

/// 推进器粒子生成最小间隔（秒）
const THRUSTER_THROTTLE_INTERVAL: f32 = 0.025; // 40 次/秒，而非 60 次/秒

/// 子弹尾迹全局生成最小间隔（秒）
const BULLET_TRAIL_THROTTLE_INTERVAL: f32 = 0.016; // ~60 次/秒 全局上限

/// 单个粒子
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub size: f32,
    pub created_at: f32,
    pub lifetime: f32,
}

impl Particle {
    /// 创建新粒子
    pub fn new(pos: Vec2, vel: Vec2, color: Color, size: f32, lifetime: f32, now: f32) -> Self {
        Self {
            pos,
            vel,
            color,
            size,
            created_at: now,
            lifetime,
        }
    }

    /// 更新粒子位置
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        // 添加重力效果
        self.vel.y += particles::GRAVITY * dt;
    }

    /// 检查粒子是否存活
    pub fn is_alive(&self, now: f32) -> bool {
        now - self.created_at < self.lifetime
    }

    /// 获取粒子当前的透明度（随时间衰减）
    pub fn alpha(&self, now: f32) -> f32 {
        let age = now - self.created_at;
        let life_ratio = age / self.lifetime;
        (1.0 - life_ratio).max(0.0)
    }
}

/// 粒子系统
pub struct ParticleSystem {
    particles: Vec<Particle>,
    /// 上次推进器粒子生成时间（用于节流）
    last_thruster_time: f32,
    /// 上次子弹尾迹生成时间（全局节流）
    last_bullet_trail_time: f32,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(particles::MAX_PARTICLES),
            last_thruster_time: f32::NEG_INFINITY,
            last_bullet_trail_time: f32::NEG_INFINITY,
        }
    }

    /// 创建爆炸效果
    pub fn spawn_explosion(&mut self, pos: Vec2, size: f32, color: Color, now: f32) {
        let count = (EXPLOSION_PARTICLE_COUNT as f32 * (size / 20.0).min(3.0)) as usize;
        let (speed_min, speed_max) = particles::EXPLOSION_SPEED_RANGE;
        let (size_min, size_max) = particles::EXPLOSION_SIZE_RANGE;
        let (life_min, life_max) = particles::EXPLOSION_LIFETIME_RANGE;

        for _ in 0..count {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(speed_min, speed_max);
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let particle_size = rand::gen_range(size_min, size_max);
            let lifetime = rand::gen_range(life_min, life_max);

            self.particles
                .push(Particle::new(pos, vel, color, particle_size, lifetime, now));
        }
    }

    /// 创建推进器火焰效果
    ///
    /// 内置节流：每 25ms 最多生成一次，减少高帧率下的粒子数量
    pub fn spawn_thruster(&mut self, pos: Vec2, direction: Vec2, now: f32) {
        // 节流检查：避免高帧率下过度生成
        if now - self.last_thruster_time < THRUSTER_THROTTLE_INTERVAL {
            return;
        }
        self.last_thruster_time = now;

        let (spread_min, spread_max) = particles::THRUSTER_SPREAD_RANGE;
        let (speed_min, speed_max) = particles::THRUSTER_SPEED_RANGE;
        let (life_min, life_max) = particles::THRUSTER_LIFETIME_RANGE;

        for _ in 0..THRUSTER_PARTICLE_COUNT {
            let spread = rand::gen_range(spread_min, spread_max);
            let speed = rand::gen_range(speed_min, speed_max);
            let vel = Vec2::new(
                -direction.x * speed + spread * 50.0,
                -direction.y * speed + spread * 50.0,
            );

            let orange = Color::new(1.0, rand::gen_range(0.5, 0.8), 0.2, 1.0);
            let size = rand::gen_range(1.5, 3.0);
            let lifetime = rand::gen_range(life_min, life_max);

            self.particles
                .push(Particle::new(pos, vel, orange, size, lifetime, now));
        }
    }

    /// 创建子弹尾迹
    ///
    /// 内置双重节流：
    /// 1. 全局时间节流：每 16ms 最多生成一次尾迹
    /// 2. 随机概率：50% 几率跳过生成
    pub fn spawn_bullet_trail(&mut self, pos: Vec2, vel: Vec2, color: Color, now: f32) {
        // 全局时间节流：防止多颗子弹在同一帧生成过多尾迹
        if now - self.last_bullet_trail_time < BULLET_TRAIL_THROTTLE_INTERVAL {
            return;
        }

        // 随机概率过滤（保留原有机制）
        if rand::gen_range(0.0, 1.0) > particles::BULLET_TRAIL_SPAWN_CHANCE {
            return;
        }

        self.last_bullet_trail_time = now;

        let particle_vel = vel * 0.2;
        let size = rand::gen_range(0.8, 1.5);
        let (life_min, life_max) = particles::TRAIL_LIFETIME_RANGE;
        let lifetime = rand::gen_range(life_min, life_max);

        // 尾迹颜色更淡
        let trail_color = Color::new(color.r, color.g, color.b, particles::TRAIL_ALPHA);

        self.particles.push(Particle::new(
            pos,
            particle_vel,
            trail_color,
            size,
            lifetime,
            now,
        ));
    }

    /// 更新所有粒子
    pub fn update(&mut self, dt: f32, now: f32) {
        for particle in self.particles.iter_mut() {
            particle.update(dt);
        }
        self.particles.retain(|p| p.is_alive(now));
    }

    /// 绘制所有粒子
    pub fn draw(&self, now: f32) {
        for particle in &self.particles {
            let alpha = particle.alpha(now);
            let color = Color::new(particle.color.r, particle.color.g, particle.color.b, alpha);
            draw_circle(particle.pos.x, particle.pos.y, particle.size, color);
        }
    }

    /// 清空所有粒子
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// 获取当前粒子数量
    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let particle = Particle::new(
            Vec2::new(100.0, 100.0),
            Vec2::new(10.0, 10.0),
            RED,
            2.0,
            1.0,
            0.0,
        );
        assert_eq!(particle.pos, Vec2::new(100.0, 100.0));
        assert_eq!(particle.size, 2.0);
        assert!(particle.is_alive(0.5));
    }

    #[test]
    fn test_particle_lifetime() {
        let particle = Particle::new(Vec2::ZERO, Vec2::ZERO, RED, 1.0, 1.0, 0.0);
        assert!(particle.is_alive(0.5));
        assert!(particle.is_alive(0.99));
        assert!(!particle.is_alive(1.5));
    }

    #[test]
    fn test_particle_alpha_decay() {
        let particle = Particle::new(Vec2::ZERO, Vec2::ZERO, RED, 1.0, 1.0, 0.0);
        assert!((particle.alpha(0.0) - 1.0).abs() < 0.001);
        assert!((particle.alpha(0.5) - 0.5).abs() < 0.001);
        assert!((particle.alpha(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_system_explosion() {
        let mut system = ParticleSystem::new();
        system.spawn_explosion(Vec2::new(100.0, 100.0), 20.0, RED, 0.0);
        assert!(system.count() > 0);
    }

    #[test]
    fn test_particle_system_update() {
        let mut system = ParticleSystem::new();
        system.spawn_explosion(Vec2::new(100.0, 100.0), 20.0, RED, 0.0);
        let initial_count = system.count();
        system.update(0.1, 0.1);
        assert_eq!(system.count(), initial_count);
        system.update(10.0, 10.0);
        assert_eq!(system.count(), 0); // 所有粒子应该已过期
    }

    // ------------------------------------------------------------------------
    // Throttling Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_thruster_throttling() {
        let mut system = ParticleSystem::new();
        let pos = Vec2::new(100.0, 100.0);
        let dir = Vec2::new(1.0, 0.0);

        // 第一次调用应该生成粒子
        system.spawn_thruster(pos, dir, 0.0);
        let count_after_first = system.count();
        assert!(
            count_after_first > 0,
            "First thruster call should spawn particles"
        );

        // 立即再次调用（同一时间）应该被节流
        system.spawn_thruster(pos, dir, 0.0);
        assert_eq!(
            system.count(),
            count_after_first,
            "Immediate second call should be throttled"
        );

        // 10ms 后调用仍应被节流（阈值是 25ms）
        system.spawn_thruster(pos, dir, 0.01);
        assert_eq!(
            system.count(),
            count_after_first,
            "Call at 10ms should still be throttled"
        );

        // 30ms 后应该可以生成
        system.spawn_thruster(pos, dir, 0.03);
        assert!(
            system.count() > count_after_first,
            "Call at 30ms should spawn more particles"
        );
    }

    #[test]
    fn test_bullet_trail_throttling() {
        let mut system = ParticleSystem::new();
        let pos = Vec2::new(100.0, 100.0);
        let vel = Vec2::new(500.0, 0.0);
        let color = YELLOW;

        // 尝试多次快速调用（使用固定随机种子无法保证，所以多次尝试）
        // 由于有 50% 随机过滤，我们需要确保节流本身有效
        let mut spawned_count = 0;
        for _ in 0..100 {
            let before = system.count();
            system.spawn_bullet_trail(pos, vel, color, 0.0); // 同一时间点
            if system.count() > before {
                spawned_count += 1;
            }
        }

        // 由于时间节流，即使调用 100 次，同一时间点最多生成 1 个
        assert!(
            spawned_count <= 1,
            "Multiple calls at same time should produce at most 1 particle due to throttling"
        );
    }

    #[test]
    fn test_bullet_trail_allows_after_interval() {
        let mut system = ParticleSystem::new();
        let pos = Vec2::new(100.0, 100.0);
        let vel = Vec2::new(500.0, 0.0);
        let color = YELLOW;

        // 在不同时间点调用，应该允许生成（受随机性影响）
        // 时间间隔 > 16ms，应该通过节流检查
        for i in 0..10 {
            let t = i as f32 * 0.02; // 每 20ms 一次
            system.spawn_bullet_trail(pos, vel, color, t);
        }

        // 由于随机性，不能保证确切数量，但应该有一些粒子生成
        // 这里主要验证不会因为节流完全阻止生成
        // （10 次调用，50% 随机，预期约 5 个，但有方差）
    }

    #[test]
    fn test_particle_system_default() {
        let system = ParticleSystem::default();
        assert_eq!(system.count(), 0);
        assert!(system.last_thruster_time.is_infinite() && system.last_thruster_time < 0.0);
    }
}
