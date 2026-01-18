//! 统一主题系统
//!
//! 集中管理所有 UI 颜色、字号、间距和动画参数。

use macroquad::prelude::*;

/// 颜色调色板
pub mod colors {
    use macroquad::prelude::Color;

    // === 背景色 ===
    pub const BG_DARK: Color = Color::new(0.04, 0.05, 0.08, 0.98);
    pub const BG_PANEL: Color = Color::new(0.06, 0.08, 0.14, 0.95);
    pub const BG_CARD: Color = Color::new(0.08, 0.10, 0.18, 0.92);
    pub const BG_OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.55);

    // === 主色调 ===
    pub const PRIMARY: Color = Color::new(0.30, 0.60, 0.95, 1.0);
    pub const PRIMARY_LIGHT: Color = Color::new(0.45, 0.72, 1.0, 1.0);
    pub const PRIMARY_DARK: Color = Color::new(0.18, 0.40, 0.75, 1.0);

    // === 强调色 ===
    pub const ACCENT_CYAN: Color = Color::new(0.20, 0.90, 1.0, 1.0);
    pub const ACCENT_GOLD: Color = Color::new(1.0, 0.84, 0.0, 1.0);
    pub const ACCENT_PURPLE: Color = Color::new(0.65, 0.40, 0.95, 1.0);

    // === 语义色 ===
    pub const SUCCESS: Color = Color::new(0.30, 0.85, 0.45, 1.0);
    pub const WARNING: Color = Color::new(1.0, 0.75, 0.20, 1.0);
    pub const DANGER: Color = Color::new(1.0, 0.30, 0.25, 1.0);
    pub const INFO: Color = Color::new(0.40, 0.70, 1.0, 1.0);

    // === 文字色 ===
    pub const TEXT_PRIMARY: Color = Color::new(0.92, 0.94, 0.98, 1.0);
    pub const TEXT_SECONDARY: Color = Color::new(0.70, 0.75, 0.82, 1.0);
    pub const TEXT_MUTED: Color = Color::new(0.50, 0.55, 0.62, 1.0);
    pub const TEXT_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.40);

    // === 边框色 ===
    pub const BORDER_DEFAULT: Color = Color::new(0.30, 0.55, 0.90, 0.65);
    pub const BORDER_HOVER: Color = Color::new(0.40, 0.70, 1.0, 0.90);
    pub const BORDER_ACTIVE: Color = Color::new(0.50, 0.85, 1.0, 1.0);

    // === 玩家色 ===
    pub const PLAYER_1: Color = Color::new(0.30, 0.80, 1.0, 1.0);
    pub const PLAYER_2: Color = Color::new(1.0, 0.50, 0.30, 1.0);

    // === 状态色 ===
    pub const SHIELD_ACTIVE: Color = Color::new(0.40, 0.80, 1.0, 0.60);
    pub const INVULNERABLE: Color = Color::new(1.0, 1.0, 0.60, 0.50);
    pub const FLUX_HIGH: Color = Color::new(0.20, 0.90, 1.0, 1.0);
    pub const FLUX_LOW: Color = Color::new(1.0, 0.30, 0.20, 1.0);
    pub const FLUX_NORMAL: Color = Color::new(0.40, 0.70, 1.0, 1.0);
}

/// 字号规范
pub mod typography {
    pub const TITLE_XL: u16 = 64;
    pub const TITLE_LG: u16 = 48;
    pub const TITLE_MD: u16 = 36;
    pub const HEADING: u16 = 28;
    pub const BODY_LG: u16 = 24;
    pub const BODY: u16 = 20;
    pub const BODY_SM: u16 = 16;
    pub const CAPTION: u16 = 12;
}

/// 间距规范
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}

/// 圆角规范
pub mod radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
    pub const XL: f32 = 16.0;
    pub const PILL: f32 = 999.0;
}

/// 动画参数
pub mod animation {
    pub const DURATION_FAST: f32 = 0.15;
    pub const DURATION_NORMAL: f32 = 0.25;
    pub const DURATION_SLOW: f32 = 0.40;

    pub const PULSE_SPEED: f32 = 3.0;
    pub const GLOW_SPEED: f32 = 2.0;
    pub const FLOAT_SPEED: f32 = 1.5;
}

/// 缓动函数
pub mod easing {
    /// 弹性缓出
    pub fn ease_out_back(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
    }

    /// 平滑缓出
    pub fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    /// 平滑缓入缓出
    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// 弹跳效果
    pub fn ease_out_bounce(t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;
        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }
}

/// 绘制发光边框面板
pub fn draw_glow_panel(x: f32, y: f32, w: f32, h: f32, bg: Color, glow_color: Color, glow_intensity: f32) {
    // 外发光层（多层叠加模拟）
    for i in 1..=3 {
        let offset = i as f32 * 2.0;
        let alpha = glow_intensity * 0.15 / i as f32;
        draw_rectangle(
            x - offset,
            y - offset,
            w + offset * 2.0,
            h + offset * 2.0,
            Color::new(glow_color.r, glow_color.g, glow_color.b, alpha),
        );
    }

    // 主面板背景
    draw_rectangle(x, y, w, h, bg);

    // 边框
    draw_rectangle_lines(x, y, w, h, 2.0, glow_color);
}

/// 绘制渐变面板（顶部亮底部暗）
pub fn draw_gradient_panel(x: f32, y: f32, w: f32, h: f32, top_color: Color, bottom_color: Color) {
    let steps = 16;
    let step_h = h / steps as f32;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let color = Color::new(
            top_color.r + (bottom_color.r - top_color.r) * t,
            top_color.g + (bottom_color.g - top_color.g) * t,
            top_color.b + (bottom_color.b - top_color.b) * t,
            top_color.a + (bottom_color.a - top_color.a) * t,
        );
        draw_rectangle(x, y + i as f32 * step_h, w, step_h + 1.0, color);
    }
}

/// 绘制脉冲发光文字
pub fn draw_pulsing_text(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    base_color: Color,
    time: f32,
    font: Option<&macroquad::text::Font>,
) {
    let pulse = 0.7 + 0.3 * (time * animation::PULSE_SPEED).sin();
    let glow_alpha = 0.3 * pulse;

    // 发光层
    draw_text_ex(
        text,
        x - 1.0,
        y + 1.0,
        TextParams {
            font_size,
            color: Color::new(base_color.r, base_color.g, base_color.b, glow_alpha),
            font,
            ..Default::default()
        },
    );

    // 主文字
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size,
            color: Color::new(base_color.r * pulse, base_color.g * pulse, base_color.b, base_color.a),
            font,
            ..Default::default()
        },
    );
}

/// 绘制圆形进度条（用于 Flux、护盾等）
pub fn draw_arc_progress(
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    progress: f32,
    bg_color: Color,
    fill_color: Color,
) {
    let segments = 32;

    // 背景圆弧
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

        let x1 = cx + angle1.cos() * radius;
        let y1 = cy + angle1.sin() * radius;
        let x2 = cx + angle2.cos() * radius;
        let y2 = cy + angle2.sin() * radius;

        draw_line(x1, y1, x2, y2, thickness, bg_color);
    }

    // 填充圆弧
    let fill_segments = (segments as f32 * progress.clamp(0.0, 1.0)) as i32;
    for i in 0..fill_segments {
        let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

        let x1 = cx + angle1.cos() * radius;
        let y1 = cy + angle1.sin() * radius;
        let x2 = cx + angle2.cos() * radius;
        let y2 = cy + angle2.sin() * radius;

        draw_line(x1, y1, x2, y2, thickness, fill_color);
    }
}

/// 绘制图标式生命显示
pub fn draw_lives_icons(x: f32, y: f32, lives: u32, max_lives: u32, color: Color) {
    let icon_size = 16.0;
    let spacing = 4.0;

    for i in 0..max_lives {
        let ix = x + i as f32 * (icon_size + spacing);
        let alpha = if i < lives { 1.0 } else { 0.25 };
        let c = Color::new(color.r, color.g, color.b, alpha);

        // 简单三角形代表飞船
        let p1 = Vec2::new(ix + icon_size / 2.0, y);
        let p2 = Vec2::new(ix, y + icon_size);
        let p3 = Vec2::new(ix + icon_size, y + icon_size);

        draw_triangle(p1, p2, p3, c);
    }
}

/// 响应式布局计算
pub struct Layout {
    pub screen_w: f32,
    pub screen_h: f32,
    pub scale: f32,
}

impl Layout {
    pub fn new() -> Self {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let scale = (screen_w / 1024.0).min(screen_h / 768.0).max(0.5);

        Self {
            screen_w,
            screen_h,
            scale,
        }
    }

    pub fn scaled(&self, value: f32) -> f32 {
        value * self.scale
    }

    pub fn center_x(&self, width: f32) -> f32 {
        (self.screen_w - width) / 2.0
    }

    pub fn center_y(&self, height: f32) -> f32 {
        (self.screen_h - height) / 2.0
    }

    pub fn font_size(&self, base: u16) -> u16 {
        ((base as f32 * self.scale).round() as u16).max(10)
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
