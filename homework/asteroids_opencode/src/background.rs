//! 星空背景系统
//!
//! 提供多层视差滚动的星空背景效果，增强游戏视觉体验。
//!
//! ## 特性
//! - 三层视差（远景/中景/近景）
//! - 星星闪烁效果
//! - 随玩家/镜头移动产生视差滚动
//! - 可配置星星密度和颜色

use macroquad::prelude::*;

/// 星空背景配置常量
pub mod config {
    /// 远景层星星数量
    pub const FAR_STAR_COUNT: usize = 80;
    /// 中景层星星数量
    pub const MID_STAR_COUNT: usize = 50;
    /// 近景层星星数量
    pub const NEAR_STAR_COUNT: usize = 25;

    /// 远景视差系数（移动最慢）
    pub const FAR_PARALLAX: f32 = 0.02;
    /// 中景视差系数
    pub const MID_PARALLAX: f32 = 0.05;
    /// 近景视差系数（移动最快）
    pub const NEAR_PARALLAX: f32 = 0.1;

    /// 闪烁速度范围
    pub const TWINKLE_SPEED_MIN: f32 = 0.5;
    pub const TWINKLE_SPEED_MAX: f32 = 2.0;

    /// 星星大小范围
    pub const STAR_SIZE_MIN: f32 = 0.5;
    pub const STAR_SIZE_FAR_MAX: f32 = 1.5;
    pub const STAR_SIZE_MID_MAX: f32 = 2.0;
    pub const STAR_SIZE_NEAR_MAX: f32 = 3.0;
}

/// 单个星星
#[derive(Clone)]
pub struct Star {
    /// 基础位置（相对于世界原点，0-1 范围归一化）
    pub base_x: f32,
    pub base_y: f32,
    /// 星星大小
    pub size: f32,
    /// 基础亮度 (0.0-1.0)
    pub base_brightness: f32,
    /// 闪烁相位（随机起始）
    pub twinkle_phase: f32,
    /// 闪烁速度
    pub twinkle_speed: f32,
    /// 颜色色调 (0: 白色, 1: 蓝色, 2: 黄色, 3: 红色)
    pub color_tint: u8,
}

impl Star {
    /// 创建随机星星
    pub fn random(size_min: f32, size_max: f32) -> Self {
        Self {
            base_x: rand::gen_range(0.0, 1.0),
            base_y: rand::gen_range(0.0, 1.0),
            size: rand::gen_range(size_min, size_max),
            base_brightness: rand::gen_range(0.3, 1.0),
            twinkle_phase: rand::gen_range(0.0, std::f32::consts::TAU),
            twinkle_speed: rand::gen_range(config::TWINKLE_SPEED_MIN, config::TWINKLE_SPEED_MAX),
            color_tint: rand::gen_range(0, 10) as u8, // 0-6: 白, 7: 蓝, 8: 黄, 9: 红
        }
    }

    /// 获取当前亮度（包含闪烁）
    pub fn current_brightness(&self, time: f32) -> f32 {
        let twinkle = (time * self.twinkle_speed + self.twinkle_phase).sin();
        let twinkle_factor = 0.7 + 0.3 * twinkle; // 0.4 到 1.0 范围
        (self.base_brightness * twinkle_factor).clamp(0.0, 1.0)
    }

    /// 获取星星颜色
    pub fn color(&self, brightness: f32) -> Color {
        match self.color_tint {
            7 => Color::new(0.7 * brightness, 0.8 * brightness, brightness, 1.0), // 蓝色
            8 => Color::new(brightness, brightness * 0.95, 0.7 * brightness, 1.0), // 黄色
            9 => Color::new(brightness, 0.7 * brightness, 0.7 * brightness, 1.0), // 红色
            _ => Color::new(brightness, brightness, brightness, 1.0),             // 白色
        }
    }
}

/// 星空层
pub struct StarLayer {
    /// 该层的星星
    pub stars: Vec<Star>,
    /// 视差系数（0-1，越小移动越慢）
    pub parallax: f32,
}

impl StarLayer {
    /// 创建新的星空层
    pub fn new(count: usize, parallax: f32, size_min: f32, size_max: f32) -> Self {
        let stars = (0..count)
            .map(|_| Star::random(size_min, size_max))
            .collect();
        Self { stars, parallax }
    }

    /// 绘制该层星空
    pub fn draw(&self, time: f32, camera_x: f32, camera_y: f32) {
        let w = screen_width();
        let h = screen_height();

        for star in &self.stars {
            // 计算视差偏移
            let parallax_offset_x = camera_x * self.parallax;
            let parallax_offset_y = camera_y * self.parallax;

            // 计算屏幕位置（循环环绕）
            let mut screen_x = (star.base_x * w - parallax_offset_x) % w;
            let mut screen_y = (star.base_y * h - parallax_offset_y) % h;

            // 处理负数环绕
            if screen_x < 0.0 {
                screen_x += w;
            }
            if screen_y < 0.0 {
                screen_y += h;
            }

            // 计算亮度和颜色
            let brightness = star.current_brightness(time);
            let color = star.color(brightness);

            // 绘制星星
            if star.size <= 1.5 {
                // 小星星用点绘制
                draw_rectangle(screen_x, screen_y, star.size, star.size, color);
            } else {
                // 大星星用圆形
                draw_circle(screen_x, screen_y, star.size * 0.5, color);

                // 添加微弱光晕
                if brightness > 0.7 {
                    let glow_color = Color::new(color.r, color.g, color.b, 0.2 * brightness);
                    draw_circle(screen_x, screen_y, star.size, glow_color);
                }
            }
        }
    }
}

/// 星空背景管理器
pub struct Starfield {
    /// 远景层（最慢）
    pub far_layer: StarLayer,
    /// 中景层
    pub mid_layer: StarLayer,
    /// 近景层（最快）
    pub near_layer: StarLayer,
    /// 背景基础颜色
    pub bg_color: Color,
    /// 累计相机位置（用于视差计算）
    camera_x: f32,
    camera_y: f32,
}

impl Default for Starfield {
    fn default() -> Self {
        Self::new()
    }
}

impl Starfield {
    /// 创建默认星空
    pub fn new() -> Self {
        Self {
            far_layer: StarLayer::new(
                config::FAR_STAR_COUNT,
                config::FAR_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_FAR_MAX,
            ),
            mid_layer: StarLayer::new(
                config::MID_STAR_COUNT,
                config::MID_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_MID_MAX,
            ),
            near_layer: StarLayer::new(
                config::NEAR_STAR_COUNT,
                config::NEAR_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_NEAR_MAX,
            ),
            bg_color: Color::new(0.02, 0.02, 0.08, 1.0), // 深蓝黑色
            camera_x: 0.0,
            camera_y: 0.0,
        }
    }

    /// 创建自定义星空
    #[allow(dead_code)]
    pub fn with_density(far: usize, mid: usize, near: usize) -> Self {
        Self {
            far_layer: StarLayer::new(
                far,
                config::FAR_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_FAR_MAX,
            ),
            mid_layer: StarLayer::new(
                mid,
                config::MID_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_MID_MAX,
            ),
            near_layer: StarLayer::new(
                near,
                config::NEAR_PARALLAX,
                config::STAR_SIZE_MIN,
                config::STAR_SIZE_NEAR_MAX,
            ),
            bg_color: Color::new(0.02, 0.02, 0.08, 1.0),
            camera_x: 0.0,
            camera_y: 0.0,
        }
    }

    /// 更新相机位置（用于视差效果）
    ///
    /// 可以传入玩家平均位置或累计移动量
    #[allow(dead_code)]
    pub fn update_camera(&mut self, x: f32, y: f32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    /// 根据玩家速度更新相机（累加模式）
    pub fn update_with_velocity(&mut self, vx: f32, vy: f32, dt: f32) {
        self.camera_x += vx * dt;
        self.camera_y += vy * dt;
    }

    /// 设置背景颜色
    #[allow(dead_code)]
    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
    }

    /// 绘制星空背景
    ///
    /// `time` 用于闪烁动画
    pub fn draw(&self, time: f32) {
        // 绘制背景色
        clear_background(self.bg_color);

        // 按远到近顺序绘制各层
        self.far_layer.draw(time, self.camera_x, self.camera_y);
        self.mid_layer.draw(time, self.camera_x, self.camera_y);
        self.near_layer.draw(time, self.camera_x, self.camera_y);
    }

    /// 仅绘制星星（不清除背景）
    #[allow(dead_code)]
    pub fn draw_stars_only(&self, time: f32) {
        self.far_layer.draw(time, self.camera_x, self.camera_y);
        self.mid_layer.draw(time, self.camera_x, self.camera_y);
        self.near_layer.draw(time, self.camera_x, self.camera_y);
    }
}

/// 简化的静态星空（无视差，性能更好）
#[allow(dead_code)]
pub struct SimpleStarfield {
    stars: Vec<(f32, f32, f32, f32)>, // (x, y, size, brightness)
}

impl Default for SimpleStarfield {
    fn default() -> Self {
        Self::new(100)
    }
}

#[allow(dead_code)]
impl SimpleStarfield {
    pub fn new(count: usize) -> Self {
        let stars = (0..count)
            .map(|_| {
                (
                    rand::gen_range(0.0, 1.0),
                    rand::gen_range(0.0, 1.0),
                    rand::gen_range(0.5, 2.0),
                    rand::gen_range(0.3, 1.0),
                )
            })
            .collect();
        Self { stars }
    }

    pub fn draw(&self, time: f32) {
        let w = screen_width();
        let h = screen_height();

        // 深空背景
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        for &(x, y, size, base_bright) in &self.stars {
            // 简单闪烁
            let twinkle = (time * 1.5 + x * 10.0).sin() * 0.15 + 0.85;
            let brightness = base_bright * twinkle;
            let color = Color::new(brightness, brightness, brightness, 1.0);

            draw_rectangle(x * w, y * h, size, size, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_creation() {
        let star = Star::random(1.0, 2.0);
        assert!(star.base_x >= 0.0 && star.base_x <= 1.0);
        assert!(star.base_y >= 0.0 && star.base_y <= 1.0);
        assert!(star.size >= 1.0 && star.size <= 2.0);
    }

    #[test]
    fn test_star_brightness() {
        let star = Star {
            base_x: 0.5,
            base_y: 0.5,
            size: 1.0,
            base_brightness: 0.8,
            twinkle_phase: 0.0,
            twinkle_speed: 1.0,
            color_tint: 0,
        };

        let b1 = star.current_brightness(0.0);
        let b2 = star.current_brightness(std::f32::consts::FRAC_PI_2); // PI/2 gives max difference

        // 亮度应该在合理范围内波动
        assert!(b1 >= 0.0 && b1 <= 1.0);
        assert!(b2 >= 0.0 && b2 <= 1.0);
        // 不同时间点亮度应该不同
        assert!((b1 - b2).abs() > 0.01);
    }

    #[test]
    fn test_starfield_creation() {
        let starfield = Starfield::new();
        assert_eq!(starfield.far_layer.stars.len(), config::FAR_STAR_COUNT);
        assert_eq!(starfield.mid_layer.stars.len(), config::MID_STAR_COUNT);
        assert_eq!(starfield.near_layer.stars.len(), config::NEAR_STAR_COUNT);
    }

    #[test]
    fn test_camera_update() {
        let mut starfield = Starfield::new();
        starfield.update_camera(100.0, 200.0);
        assert_eq!(starfield.camera_x, 100.0);
        assert_eq!(starfield.camera_y, 200.0);
    }
}
