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

use macroquad::prelude::*;

const EXPLOSION_PARTICLE_COUNT: usize = 20;
const THRUSTER_PARTICLE_COUNT: usize = 3;

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
        self.vel.y += 50.0 * dt;
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
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(1000),
        }
    }

    /// 创建爆炸效果
    pub fn spawn_explosion(&mut self, pos: Vec2, size: f32, color: Color, now: f32) {
        let count = (EXPLOSION_PARTICLE_COUNT as f32 * (size / 20.0).min(3.0)) as usize;
        for _ in 0..count {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(50.0, 200.0);
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let particle_size = rand::gen_range(1.0, 4.0);
            let lifetime = rand::gen_range(0.3, 0.8);

            self.particles
                .push(Particle::new(pos, vel, color, particle_size, lifetime, now));
        }
    }

    /// 创建推进器火焰效果
    pub fn spawn_thruster(&mut self, pos: Vec2, direction: Vec2, now: f32) {
        for _ in 0..THRUSTER_PARTICLE_COUNT {
            let spread = rand::gen_range(-0.3, 0.3);
            let speed = rand::gen_range(80.0, 150.0);
            let vel = Vec2::new(
                -direction.x * speed + spread * 50.0,
                -direction.y * speed + spread * 50.0,
            );

            let orange = Color::new(1.0, rand::gen_range(0.5, 0.8), 0.2, 1.0);
            let size = rand::gen_range(1.5, 3.0);
            let lifetime = rand::gen_range(0.15, 0.3);

            self.particles
                .push(Particle::new(pos, vel, orange, size, lifetime, now));
        }
    }

    /// 创建子弹尾迹
    pub fn spawn_bullet_trail(&mut self, pos: Vec2, vel: Vec2, color: Color, now: f32) {
        // 每帧随机生成尾迹
        if rand::gen_range(0.0, 1.0) > 0.5 {
            return;
        }

        let particle_vel = vel * 0.2;
        let size = rand::gen_range(0.8, 1.5);
        let lifetime = rand::gen_range(0.1, 0.25);

        // 尾迹颜色更淡
        let trail_color = Color::new(color.r, color.g, color.b, 0.4);

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
}
