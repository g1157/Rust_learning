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
//! - 特殊小行星类型（冰冻、磁性、爆炸、金色）

use macroquad::prelude::*;

use crate::constants::gameplay;

pub const ASTEROID_BASE_SPEED: f32 = 160.0; // 像素/秒
pub const ASTEROID_ROT_SPEED: f32 = 350.0; // 度/秒
pub const ASTEROID_SPLIT_SPEED_MIN: f32 = 150.0;
pub const ASTEROID_SPLIT_SPEED_MAX: f32 = 350.0;

/// 小行星顶点的最大数量
pub const MAX_VERTICES: usize = 12;

// ============================================================================
// 特殊小行星类型系统
// ============================================================================

/// 特殊小行星类型
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AsteroidType {
    /// 普通小行星（灰色）
    #[default]
    Normal,
    /// 冰冻小行星（青色）- 击中后减速玩家 3 秒
    Ice,
    /// 磁性小行星（紫色）- 吸引附近子弹
    Magnetic,
    /// 爆炸小行星（橙色）- 击毁时产生范围爆炸
    Explosive,
    /// 金色小行星（金色）- 3倍分数，稀有
    Golden,
    /// 分裂小行星（绿色）- 分裂成更多碎片
    Splitter,
}

impl AsteroidType {
    /// 获取小行星类型的主颜色
    pub fn color(&self) -> Color {
        match self {
            AsteroidType::Normal => Color::new(0.7, 0.7, 0.7, 1.0),
            AsteroidType::Ice => Color::new(0.4, 0.9, 1.0, 1.0),
            AsteroidType::Magnetic => Color::new(0.8, 0.3, 0.9, 1.0),
            AsteroidType::Explosive => Color::new(1.0, 0.5, 0.2, 1.0),
            AsteroidType::Golden => Color::new(1.0, 0.85, 0.2, 1.0),
            AsteroidType::Splitter => Color::new(0.3, 0.9, 0.4, 1.0),
        }
    }

    /// 获取小行星类型的发光颜色
    pub fn glow_color(&self) -> Color {
        match self {
            AsteroidType::Normal => Color::new(0.5, 0.5, 0.5, 0.0), // 无发光
            AsteroidType::Ice => Color::new(0.4, 0.9, 1.0, 0.3),
            AsteroidType::Magnetic => Color::new(0.8, 0.3, 0.9, 0.4),
            AsteroidType::Explosive => Color::new(1.0, 0.4, 0.1, 0.5),
            AsteroidType::Golden => Color::new(1.0, 0.9, 0.3, 0.6),
            AsteroidType::Splitter => Color::new(0.3, 0.9, 0.4, 0.3),
        }
    }

    /// 获取分数倍数
    pub fn score_multiplier(&self) -> f32 {
        match self {
            AsteroidType::Normal => 1.0,
            AsteroidType::Ice => 1.2,
            AsteroidType::Magnetic => 1.5,
            AsteroidType::Explosive => 1.3,
            AsteroidType::Golden => 3.0,
            AsteroidType::Splitter => 1.1,
        }
    }

    /// 随机生成特殊类型（基于波次）
    pub fn random_for_wave(wave: u32) -> Self {
        let roll = rand::gen_range(0.0f32, 1.0);

        // 波次越高，特殊小行星概率越高
        let special_chance = (0.05 + wave as f32 * 0.02).min(0.35);

        if roll > special_chance {
            return AsteroidType::Normal;
        }

        // 特殊类型分布
        let type_roll = rand::gen_range(0.0f32, 1.0);
        if type_roll < 0.25 {
            AsteroidType::Ice
        } else if type_roll < 0.45 {
            AsteroidType::Magnetic
        } else if type_roll < 0.65 {
            AsteroidType::Explosive
        } else if type_roll < 0.80 {
            AsteroidType::Splitter
        } else {
            // 金色最稀有
            AsteroidType::Golden
        }
    }

    /// 是否有特殊效果
    pub fn has_special_effect(&self) -> bool {
        !matches!(self, AsteroidType::Normal)
    }
}

// 特殊小行星常量（用于 Roguelike 模式，暂未完全集成）
#[allow(dead_code)]
/// 磁性小行星的吸引半径
pub const MAGNETIC_ATTRACT_RADIUS: f32 = 150.0;
#[allow(dead_code)]
/// 磁性小行星的吸引力强度
pub const MAGNETIC_ATTRACT_FORCE: f32 = 200.0;
#[allow(dead_code)]
/// 爆炸小行星的爆炸半径
pub const EXPLOSIVE_RADIUS: f32 = 80.0;
#[allow(dead_code)]
/// 冰冻效果持续时间
pub const ICE_SLOW_DURATION: f64 = 3.0;
#[allow(dead_code)]
/// 冰冻减速比例
pub const ICE_SLOW_FACTOR: f32 = 0.5;

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
    /// 小行星类型（普通、冰冻、磁性、爆炸、金色、分裂）
    pub asteroid_type: AsteroidType,
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
        Self::spawn_with_wave(center, playfield_extent, speed_multiplier, 1)
    }

    /// 生成带波次信息的小行星（用于特殊类型生成）
    pub fn spawn_with_wave(
        center: Vec2,
        playfield_extent: f32,
        speed_multiplier: f32,
        wave: u32,
    ) -> Self {
        let direction = Vec2::new(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize();
        // 大小有一定随机变化
        let base_size = playfield_extent / 10.;
        let size = base_size * rand::gen_range(0.8, 1.2);
        let sides = rand::gen_range(6, 10); // 增加边数范围，更像岩石
        let asteroid_type = AsteroidType::random_for_wave(wave);

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
            asteroid_type,
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

        // 分裂型小行星产生更多碎片
        let child_count = if self.asteroid_type == AsteroidType::Splitter {
            3
        } else {
            2
        };

        let mut children = vec![
            Self::child_from(self, perp_a, child_size, child_sides),
            Self::child_from(self, perp_b, child_size, child_sides),
        ];

        // 分裂型额外产生一个碎片
        if child_count > 2 {
            let extra_vel = Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0))
                .normalize()
                * rand::gen_range(ASTEROID_SPLIT_SPEED_MIN, ASTEROID_SPLIT_SPEED_MAX);
            children.push(Self::child_from(
                self,
                extra_vel,
                child_size * 0.8,
                child_sides,
            ));
        }

        Some(children)
    }

    fn child_from(parent: &Asteroid, velocity: Vec2, size: f32, sides: u8) -> Asteroid {
        // 子小行星有小概率继承特殊类型，否则变为普通
        let child_type = if rand::gen_range(0.0f32, 1.0) < 0.3 {
            parent.asteroid_type
        } else {
            AsteroidType::Normal
        };

        Asteroid {
            pos: parent.pos,
            vel: velocity,
            rot: rand::gen_range(0., 360.),
            rot_speed: rand::gen_range(-ASTEROID_ROT_SPEED, ASTEROID_ROT_SPEED),
            size,
            sides,
            collided: false,
            vertex_offsets: Self::generate_vertex_offsets(sides),
            asteroid_type: child_type,
        }
    }

    pub fn score_value(&self) -> u32 {
        let base_score = if self.size > gameplay::ASTEROID_SIZE_LARGE {
            gameplay::SCORE_ASTEROID_LARGE
        } else if self.size > gameplay::ASTEROID_SIZE_MEDIUM {
            gameplay::SCORE_ASTEROID_MEDIUM
        } else {
            gameplay::SCORE_ASTEROID_SMALL
        };
        // 应用类型分数倍数
        (base_score as f32 * self.asteroid_type.score_multiplier()) as u32
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
    draw_asteroid_with_time(asteroid, offset, color, 0.0);
}

/// 绘制小行星（带时间参数，用于动画效果）
pub fn draw_asteroid_with_time(asteroid: &Asteroid, offset: Vec2, _base_color: Color, time: f32) {
    let vertices = asteroid.get_vertices();
    let n = vertices.len();

    if n < 3 {
        return;
    }

    let center = asteroid.pos + offset;
    let asteroid_color = asteroid.asteroid_type.color();
    let glow_color = asteroid.asteroid_type.glow_color();

    // 特殊类型绘制发光效果
    if asteroid.asteroid_type.has_special_effect() {
        let pulse = 0.7 + 0.3 * (time * 3.0 + asteroid.rot * 0.01).sin();
        let glow_radius = asteroid.size * 1.3 * pulse;

        // 外层光晕
        draw_circle(
            center.x,
            center.y,
            glow_radius,
            Color::new(
                glow_color.r,
                glow_color.g,
                glow_color.b,
                glow_color.a * pulse,
            ),
        );

        // 磁性小行星：绘制磁力线
        if asteroid.asteroid_type == AsteroidType::Magnetic {
            draw_magnetic_field(center, asteroid.size, time);
        }

        // 爆炸小行星：绘制警告标记
        if asteroid.asteroid_type == AsteroidType::Explosive {
            draw_explosive_warning(center, asteroid.size, time);
        }

        // 金色小行星：绘制闪光
        if asteroid.asteroid_type == AsteroidType::Golden {
            draw_golden_sparkles(center, asteroid.size, time);
        }

        // 冰冻小行星：绘制冰晶
        if asteroid.asteroid_type == AsteroidType::Ice {
            draw_ice_crystals(center, asteroid.size, time);
        }
    }

    // 绘制不规则多边形的边
    let line_width = if asteroid.asteroid_type == AsteroidType::Golden {
        3.0
    } else {
        2.0
    };
    for i in 0..n {
        let v1 = vertices[i] + offset;
        let v2 = vertices[(i + 1) % n] + offset;
        draw_line(v1.x, v1.y, v2.x, v2.y, line_width, asteroid_color);
    }

    // 特殊类型绘制内部填充
    if asteroid.asteroid_type.has_special_effect() {
        let fill_color = Color::new(asteroid_color.r, asteroid_color.g, asteroid_color.b, 0.15);
        // 简化填充：绘制中心圆
        draw_circle(center.x, center.y, asteroid.size * 0.5, fill_color);
    }
}

/// 绘制磁性小行星的磁力线
fn draw_magnetic_field(center: Vec2, size: f32, time: f32) {
    let line_count = 4;
    let rotation = time * 1.5;

    for i in 0..line_count {
        let base_angle = rotation + (i as f32 * std::f32::consts::FRAC_PI_2);
        let inner_radius = size * 0.8;
        let outer_radius = size * 1.5;

        // 弯曲的磁力线
        let segments = 8;
        for j in 0..segments {
            let t1 = j as f32 / segments as f32;
            let t2 = (j + 1) as f32 / segments as f32;

            let r1 = inner_radius + (outer_radius - inner_radius) * t1;
            let r2 = inner_radius + (outer_radius - inner_radius) * t2;

            let curve = (t1 * std::f32::consts::PI).sin() * 0.3;
            let a1 = base_angle + curve;
            let a2 = base_angle + (t2 * std::f32::consts::PI).sin() * 0.3;

            let p1 = center + Vec2::new(a1.cos() * r1, a1.sin() * r1);
            let p2 = center + Vec2::new(a2.cos() * r2, a2.sin() * r2);

            let alpha = 0.4 * (1.0 - t1);
            draw_line(
                p1.x,
                p1.y,
                p2.x,
                p2.y,
                1.5,
                Color::new(0.8, 0.3, 0.9, alpha),
            );
        }
    }
}

/// 绘制爆炸小行星的警告标记
fn draw_explosive_warning(center: Vec2, size: f32, time: f32) {
    let flash = ((time * 4.0).sin() + 1.0) * 0.5;

    // 警告三角形
    let tri_size = size * 0.4;
    let tri_color = Color::new(1.0, 0.3, 0.1, 0.6 * flash);

    let top = center + Vec2::new(0.0, -tri_size);
    let left = center + Vec2::new(-tri_size * 0.866, tri_size * 0.5);
    let right = center + Vec2::new(tri_size * 0.866, tri_size * 0.5);

    draw_line(top.x, top.y, left.x, left.y, 2.0, tri_color);
    draw_line(left.x, left.y, right.x, right.y, 2.0, tri_color);
    draw_line(right.x, right.y, top.x, top.y, 2.0, tri_color);

    // 中心感叹号
    draw_line(
        center.x,
        center.y - tri_size * 0.3,
        center.x,
        center.y + tri_size * 0.1,
        2.0,
        tri_color,
    );
    draw_circle(center.x, center.y + tri_size * 0.25, 2.0, tri_color);
}

/// 绘制金色小行星的闪光效果
fn draw_golden_sparkles(center: Vec2, size: f32, time: f32) {
    let sparkle_count = 6;

    for i in 0..sparkle_count {
        let angle = time * 2.0 + (i as f32 * std::f32::consts::TAU / sparkle_count as f32);
        let dist = size * (0.6 + 0.3 * ((time * 3.0 + i as f32).sin()));
        let sparkle_pos = center + Vec2::new(angle.cos() * dist, angle.sin() * dist);

        let sparkle_alpha = 0.5 + 0.5 * ((time * 5.0 + i as f32 * 0.7).sin());
        let sparkle_size = 2.0 + 1.5 * sparkle_alpha;

        draw_circle(
            sparkle_pos.x,
            sparkle_pos.y,
            sparkle_size,
            Color::new(1.0, 0.95, 0.5, sparkle_alpha),
        );
    }
}

/// 绘制冰冻小行星的冰晶效果
fn draw_ice_crystals(center: Vec2, size: f32, time: f32) {
    let crystal_count = 3;

    for i in 0..crystal_count {
        let base_angle = (i as f32 * std::f32::consts::TAU / crystal_count as f32) + time * 0.5;
        let dist = size * 0.7;
        let crystal_center = center + Vec2::new(base_angle.cos() * dist, base_angle.sin() * dist);

        // 六角冰晶
        let crystal_size = size * 0.15;
        let alpha = 0.4 + 0.2 * ((time * 2.0 + i as f32).sin());

        for j in 0..6 {
            let angle = (j as f32 * std::f32::consts::FRAC_PI_3) + time * 0.3;
            let end =
                crystal_center + Vec2::new(angle.cos() * crystal_size, angle.sin() * crystal_size);
            draw_line(
                crystal_center.x,
                crystal_center.y,
                end.x,
                end.y,
                1.5,
                Color::new(0.6, 0.95, 1.0, alpha),
            );
        }
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
    spawn_wave_with_speed_and_wave(center, playfield_extent, count, speed_multiplier, 1)
}

/// 生成带速度倍数和波次信息的波次（用于特殊类型生成）
pub fn spawn_wave_with_speed_and_wave(
    center: Vec2,
    playfield_extent: f32,
    count: usize,
    speed_multiplier: f32,
    wave: u32,
) -> Vec<Asteroid> {
    (0..count)
        .map(|_| Asteroid::spawn_with_wave(center, playfield_extent, speed_multiplier, wave))
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
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
            asteroid_type: AsteroidType::Normal,
        };
        let vertices = asteroid.get_vertices();
        assert_eq!(vertices.len(), 7);
    }
}
