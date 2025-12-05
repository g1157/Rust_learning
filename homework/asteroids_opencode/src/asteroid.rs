//! 小行星模块
//!
//! 管理小行星的生成、移动、分裂和计分。
//!
//! ## 功能
//! - 三种尺寸的小行星（大、中、小）
//! - 不规则多边形形状（锯齿状岩石外观）
//! - 小行星分裂机制（大 → 中 → 小）
//! - 基于尺寸的分数系统
//! - 随机生成安全区域外的小行星波次
//! - 屏幕边界环绕

use macroquad::prelude::*;

use crate::constants::gameplay;

pub const ASTEROID_BASE_SPEED: f32 = 160.0; // 像素/秒
pub const ASTEROID_ROT_SPEED: f32 = 350.0; // 度/秒
pub const ASTEROID_SPLIT_SPEED_MIN: f32 = 150.0;
pub const ASTEROID_SPLIT_SPEED_MAX: f32 = 350.0;

/// 小行星顶点的最大数量
pub const MAX_VERTICES: usize = 12;

pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub rot_speed: f32,
    pub size: f32,
    pub sides: u8,
    pub collided: bool,
    /// 不规则形状的顶点偏移（相对于标准圆的半径比例 0.6-1.0）
    pub vertex_offsets: [f32; MAX_VERTICES],
}

impl Asteroid {
    /// 生成随机顶点偏移，创造不规则岩石形状
    fn generate_vertex_offsets(sides: u8) -> [f32; MAX_VERTICES] {
        let mut offsets = [1.0; MAX_VERTICES];
        for offset in offsets.iter_mut().take(sides as usize) {
            // 每个顶点在 0.65 到 1.0 之间随机偏移
            *offset = rand::gen_range(0.65, 1.0);
        }
        offsets
    }

    /// 生成初始小行星（用于向后兼容）
    #[allow(dead_code)]
    pub fn spawn_initial(center: Vec2, playfield_extent: f32) -> Self {
        Self::spawn_with_speed_multiplier(center, playfield_extent, 1.0)
    }

    /// 生成带速度倍数的小行星（用于难度递增）
    pub fn spawn_with_speed_multiplier(
        center: Vec2,
        playfield_extent: f32,
        speed_multiplier: f32,
    ) -> Self {
        let direction = Vec2::new(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize();
        // 大小有一定随机变化
        let base_size = playfield_extent / 10.;
        let size = base_size * rand::gen_range(0.8, 1.2);
        let sides = rand::gen_range(6, 10); // 增加边数范围，更像岩石

        Self {
            pos: center + direction * playfield_extent / 2.,
            vel: Vec2::new(
                rand::gen_range(-ASTEROID_BASE_SPEED, ASTEROID_BASE_SPEED),
                rand::gen_range(-ASTEROID_BASE_SPEED, ASTEROID_BASE_SPEED),
            ) * speed_multiplier,
            rot: rand::gen_range(0., 360.),
            rot_speed: rand::gen_range(-ASTEROID_ROT_SPEED, ASTEROID_ROT_SPEED),
            size,
            sides,
            collided: false,
            vertex_offsets: Self::generate_vertex_offsets(sides),
        }
    }

    pub fn advance(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.rot += self.rot_speed * dt;
    }

    /// 获取小行星的实际顶点坐标（用于绘制和精确碰撞）
    pub fn get_vertices(&self) -> Vec<Vec2> {
        let mut vertices = Vec::with_capacity(self.sides as usize);
        let angle_step = std::f32::consts::TAU / self.sides as f32;
        let rot_rad = self.rot.to_radians();

        for i in 0..self.sides as usize {
            let angle = rot_rad + angle_step * i as f32;
            let radius = self.size * self.vertex_offsets[i];
            vertices.push(Vec2::new(
                self.pos.x + angle.cos() * radius,
                self.pos.y + angle.sin() * radius,
            ));
        }
        vertices
    }

    pub fn split(&self, bullet_vel: Vec2) -> Option<Vec<Asteroid>> {
        // 小于等于5边或尺寸太小则不分裂
        if self.sides <= 5 || self.size < 15.0 {
            return None;
        }

        let child_size = self.size * gameplay::ASTEROID_CHILD_SIZE_RATIO;
        // 子小行星边数减少 1-2
        let sides_reduction = rand::gen_range(1, 3);
        let child_sides = (self.sides - sides_reduction).max(5);

        let perp_a = Vec2::new(bullet_vel.y, -bullet_vel.x).normalize()
            * rand::gen_range(ASTEROID_SPLIT_SPEED_MIN, ASTEROID_SPLIT_SPEED_MAX);
        let perp_b = Vec2::new(-bullet_vel.y, bullet_vel.x).normalize()
            * rand::gen_range(ASTEROID_SPLIT_SPEED_MIN, ASTEROID_SPLIT_SPEED_MAX);

        Some(vec![
            Self::child_from(self, perp_a, child_size, child_sides),
            Self::child_from(self, perp_b, child_size, child_sides),
        ])
    }

    fn child_from(parent: &Asteroid, velocity: Vec2, size: f32, sides: u8) -> Asteroid {
        Asteroid {
            pos: parent.pos,
            vel: velocity,
            rot: rand::gen_range(0., 360.),
            rot_speed: rand::gen_range(-ASTEROID_ROT_SPEED, ASTEROID_ROT_SPEED),
            size,
            sides,
            collided: false,
            vertex_offsets: Self::generate_vertex_offsets(sides),
        }
    }

    pub fn score_value(&self) -> u32 {
        if self.size > gameplay::ASTEROID_SIZE_LARGE {
            gameplay::SCORE_ASTEROID_LARGE
        } else if self.size > gameplay::ASTEROID_SIZE_MEDIUM {
            gameplay::SCORE_ASTEROID_MEDIUM
        } else {
            gameplay::SCORE_ASTEROID_SMALL
        }
    }

    /// 碰撞检测用的有效半径（考虑不规则形状的平均值）
    #[allow(dead_code)]
    pub fn collision_radius(&self) -> f32 {
        // 使用平均顶点偏移作为碰撞半径
        let avg_offset: f32 = self.vertex_offsets[..self.sides as usize]
            .iter()
            .sum::<f32>()
            / self.sides as f32;
        self.size * avg_offset * 0.9 // 稍微缩小，让碰撞更宽容
    }
}

/// 绘制小行星（不规则多边形）
pub fn draw_asteroid(asteroid: &Asteroid, offset: Vec2, color: Color) {
    let vertices = asteroid.get_vertices();
    let n = vertices.len();

    if n < 3 {
        return;
    }

    // 绘制不规则多边形的边
    for i in 0..n {
        let v1 = vertices[i] + offset;
        let v2 = vertices[(i + 1) % n] + offset;
        draw_line(v1.x, v1.y, v2.x, v2.y, 2.0, color);
    }
}

/// 生成初始波次（用于向后兼容）
#[allow(dead_code)]
pub fn spawn_initial_wave(center: Vec2, playfield_extent: f32, count: usize) -> Vec<Asteroid> {
    spawn_wave_with_speed(center, playfield_extent, count, 1.0)
}

/// 生成带速度倍数的波次（用于难度递增）
pub fn spawn_wave_with_speed(
    center: Vec2,
    playfield_extent: f32,
    count: usize,
    speed_multiplier: f32,
) -> Vec<Asteroid> {
    (0..count)
        .map(|_| Asteroid::spawn_with_speed_multiplier(center, playfield_extent, speed_multiplier))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asteroid_score_value_large() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 50.0,
            sides: 7,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        assert_eq!(asteroid.score_value(), 20);
    }

    #[test]
    fn test_asteroid_score_value_medium() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 30.0,
            sides: 6,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        assert_eq!(asteroid.score_value(), 50);
    }

    #[test]
    fn test_asteroid_score_value_small() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 15.0,
            sides: 5,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        assert_eq!(asteroid.score_value(), 100);
    }

    #[test]
    fn test_asteroid_split_creates_smaller() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 50.0,
            sides: 8,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        let bullet_vel = Vec2::new(100.0, 0.0);
        let children = asteroid.split(bullet_vel).expect("Should split");
        assert_eq!(children.len(), 2);
        assert!(children[0].size < asteroid.size);
        assert!(children[0].sides <= asteroid.sides);
    }

    #[test]
    fn test_asteroid_split_minimum_sides_none() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 10.0,
            sides: 5,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        let bullet_vel = Vec2::new(100.0, 0.0);
        assert!(asteroid.split(bullet_vel).is_none());
    }

    #[test]
    fn test_asteroid_advance() {
        let mut asteroid = Asteroid {
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(100.0, 50.0),
            rot: 0.0,
            rot_speed: 45.0,
            size: 20.0,
            sides: 6,
            collided: false,
            vertex_offsets: [1.0; MAX_VERTICES],
        };
        asteroid.advance(0.1);
        assert!((asteroid.pos.x - 10.0).abs() < 0.001);
        assert!((asteroid.pos.y - 5.0).abs() < 0.001);
        assert!((asteroid.rot - 4.5).abs() < 0.001);
    }

    #[test]
    fn test_spawn_initial_wave_count() {
        let center = Vec2::new(400.0, 300.0);
        let extent = 600.0;
        let count = 5;
        let wave = spawn_initial_wave(center, extent, count);
        assert_eq!(wave.len(), count);
    }

    #[test]
    fn test_vertex_offsets_in_range() {
        let offsets = Asteroid::generate_vertex_offsets(8);
        for i in 0..8 {
            assert!(offsets[i] >= 0.65 && offsets[i] <= 1.0);
        }
    }

    #[test]
    fn test_get_vertices_count() {
        let asteroid = Asteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            rot: 0.0,
            rot_speed: 0.0,
            size: 30.0,
            sides: 7,
            collided: false,
            vertex_offsets: [0.8; MAX_VERTICES],
        };
        let vertices = asteroid.get_vertices();
        assert_eq!(vertices.len(), 7);
    }
}
