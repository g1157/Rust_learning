//! UI 通用组件模块
//!
//! 共享的 UI 辅助函数和组件。
//!
//! ## 功能
//! - 文本居中辅助
//! - 阴影面板
//! - 渐变背景
//! - 通用按钮样式

use macroquad::prelude::*;
use macroquad::text::Font;

// ============================================================================
// 文本辅助
// ============================================================================

/// 绘制居中文本
pub fn draw_text_centered(
    text: &str,
    y: f32,
    font_size: u16,
    color: Color,
    font: Option<&Font>,
) {
    let dims = measure_text(text, font, font_size, 1.0);
    let x = screen_width() / 2. - dims.width / 2.;

    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size,
            color,
            font,
            ..Default::default()
        },
    );
}

/// 绘制右对齐文本
pub fn draw_text_right_aligned(
    text: &str,
    right_x: f32,
    y: f32,
    font_size: u16,
    color: Color,
    font: Option<&Font>,
) {
    let dims = measure_text(text, font, font_size, 1.0);
    let x = right_x - dims.width;

    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size,
            color,
            font,
            ..Default::default()
        },
    );
}

// ============================================================================
// 面板组件
// ============================================================================

/// 绘制带阴影的面板
pub fn draw_shadow_panel(x: f32, y: f32, width: f32, height: f32, color: Color) {
    // 阴影
    let shadow_offset = 4.0;
    draw_rectangle(
        x + shadow_offset,
        y + shadow_offset,
        width,
        height,
        Color::new(0.0, 0.0, 0.0, 0.3),
    );

    // 主面板
    draw_rectangle(x, y, width, height, color);
}

/// 绘制带边框的面板
pub fn draw_bordered_panel(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    bg_color: Color,
    border_color: Color,
    border_width: f32,
) {
    draw_shadow_panel(x, y, width, height, bg_color);
    draw_rectangle_lines(x, y, width, height, border_width, border_color);
}

/// 绘制圆角面板（使用多个矩形模拟）
pub fn draw_rounded_panel(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    color: Color,
) {
    // 简化实现：绘制主矩形
    draw_rectangle(x, y, width, height, color);

    // 四个角的圆形
    let r = radius.min(width / 2.).min(height / 2.);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + width - r, y + r, r, color);
    draw_circle(x + r, y + height - r, r, color);
    draw_circle(x + width - r, y + height - r, r, color);
}

// ============================================================================
// 进度条
// ============================================================================

/// 绘制水平进度条
pub fn draw_progress_bar(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    progress: f32,
    bg_color: Color,
    fill_color: Color,
) {
    // 背景
    draw_rectangle(x, y, width, height, bg_color);

    // 填充
    let fill_width = width * progress.clamp(0.0, 1.0);
    draw_rectangle(x, y, fill_width, height, fill_color);

    // 边框
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(1.0, 1.0, 1.0, 0.3));
}

/// 绘制圆形冷却指示器
pub fn draw_cooldown_circle(
    x: f32,
    y: f32,
    radius: f32,
    progress: f32,
    color: Color,
) {
    use std::f32::consts::PI;

    let segments = 32;
    let angle_per_segment = 2.0 * PI / segments as f32;
    let filled_segments = (progress * segments as f32) as i32;

    // 背景圆
    draw_circle_lines(x, y, radius, 2.0, Color::new(0.3, 0.3, 0.3, 0.5));

    // 填充弧
    for i in 0..filled_segments {
        let start_angle = -PI / 2.0 + i as f32 * angle_per_segment;
        let end_angle = start_angle + angle_per_segment;

        let x1 = x + radius * start_angle.cos();
        let y1 = y + radius * start_angle.sin();
        let x2 = x + radius * end_angle.cos();
        let y2 = y + radius * end_angle.sin();

        draw_line(x, y, x1, y1, 2.0, color);
        draw_line(x1, y1, x2, y2, 2.0, color);
    }
}

// ============================================================================
// 按钮样式
// ============================================================================

/// 按钮状态
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Selected,
    Disabled,
}

/// 获取按钮颜色
pub fn get_button_colors(state: ButtonState, base_color: Color) -> (Color, Color) {
    match state {
        ButtonState::Normal => (
            Color::new(base_color.r * 0.3, base_color.g * 0.3, base_color.b * 0.3, 0.8),
            Color::new(base_color.r * 0.6, base_color.g * 0.6, base_color.b * 0.6, 0.8),
        ),
        ButtonState::Hovered => (
            Color::new(base_color.r * 0.4, base_color.g * 0.4, base_color.b * 0.4, 0.9),
            base_color,
        ),
        ButtonState::Selected => (
            Color::new(base_color.r * 0.5, base_color.g * 0.5, base_color.b * 0.5, 0.95),
            base_color,
        ),
        ButtonState::Disabled => (
            Color::new(0.2, 0.2, 0.2, 0.5),
            Color::new(0.4, 0.4, 0.4, 0.5),
        ),
    }
}

/// 绘制按钮
pub fn draw_button(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: &str,
    state: ButtonState,
    base_color: Color,
    font: Option<&Font>,
) {
    let (bg_color, border_color) = get_button_colors(state, base_color);

    draw_bordered_panel(x, y, width, height, bg_color, border_color, 2.0);

    let text_color = match state {
        ButtonState::Disabled => Color::new(0.5, 0.5, 0.5, 0.7),
        ButtonState::Selected => base_color,
        _ => WHITE,
    };

    let dims = measure_text(text, font, 20, 1.0);
    draw_text_ex(
        text,
        x + width / 2. - dims.width / 2.,
        y + height / 2. + 6.,
        TextParams {
            font_size: 20,
            color: text_color,
            font,
            ..Default::default()
        },
    );
}

// ============================================================================
// 工具提示
// ============================================================================

/// 绘制工具提示
pub fn draw_tooltip(x: f32, y: f32, text: &str, font: Option<&Font>) {
    let padding = 8.0;
    let dims = measure_text(text, font, 14, 1.0);
    let width = dims.width + padding * 2.;
    let height = 24.;

    // 确保不超出屏幕
    let adjusted_x = x.min(screen_width() - width - 10.);
    let adjusted_y = y.min(screen_height() - height - 10.);

    draw_rectangle(
        adjusted_x,
        adjusted_y,
        width,
        height,
        Color::new(0.1, 0.1, 0.1, 0.95),
    );
    draw_rectangle_lines(
        adjusted_x,
        adjusted_y,
        width,
        height,
        1.0,
        Color::new(0.4, 0.4, 0.4, 1.0),
    );

    draw_text_ex(
        text,
        adjusted_x + padding,
        adjusted_y + 17.,
        TextParams {
            font_size: 14,
            color: WHITE,
            font,
            ..Default::default()
        },
    );
}

// ============================================================================
// 颜色辅助
// ============================================================================

/// 线性插值颜色
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// 根据值获取渐变颜色（红-黄-绿）
pub fn get_value_color(value: f32, min: f32, max: f32) -> Color {
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);

    if t < 0.5 {
        // 红到黄
        lerp_color(RED, YELLOW, t * 2.0)
    } else {
        // 黄到绿
        lerp_color(YELLOW, GREEN, (t - 0.5) * 2.0)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_color() {
        let a = Color::new(0.0, 0.0, 0.0, 1.0);
        let b = Color::new(1.0, 1.0, 1.0, 1.0);

        let mid = lerp_color(a, b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_get_value_color() {
        let red = get_value_color(0.0, 0.0, 1.0);
        assert!(red.r > 0.9);

        let green = get_value_color(1.0, 0.0, 1.0);
        assert!(green.g > 0.9);
    }

    #[test]
    fn test_button_colors() {
        let (bg, border) = get_button_colors(ButtonState::Normal, WHITE);
        assert!(bg.a > 0.0);
        assert!(border.a > 0.0);
    }
}
