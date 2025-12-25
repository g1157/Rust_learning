//! Dark theme configuration for TDGL Dashboard
//! Based on front-end-spec.md color scheme

use egui::{Color32, Visuals, Rounding, Stroke, FontFamily, FontId, TextStyle};

// Background colors
pub const BG_DARK: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x2e);
pub const BG_MID: Color32 = Color32::from_rgb(0x16, 0x21, 0x3e);
pub const BG_LIGHT: Color32 = Color32::from_rgb(0x1f, 0x34, 0x60);

// Foreground colors
pub const FG_PRIMARY: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xe8);
pub const FG_SECONDARY: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa0);
pub const FG_WEAK: Color32 = Color32::from_rgb(0x60, 0x60, 0x60);

// Accent colors
pub const ACCENT: Color32 = Color32::from_rgb(0x4f, 0xc3, 0xf7);
pub const SUCCESS: Color32 = Color32::from_rgb(0x66, 0xbb, 0x6a);
pub const WARNING: Color32 = Color32::from_rgb(0xff, 0xa7, 0x26);
pub const ERROR: Color32 = Color32::from_rgb(0xef, 0x53, 0x50);

// Vortex marker colors
pub const VORTEX_COLOR: Color32 = Color32::from_rgb(0xff, 0x6b, 0x6b);
pub const ANTIVORTEX_COLOR: Color32 = Color32::from_rgb(0x4e, 0xcd, 0xc4);
pub const PINNED_COLOR: Color32 = Color32::from_rgb(0xff, 0xe6, 0x6d);

/// Apply dark theme to egui context
pub fn apply_dark_theme(ctx: &egui::Context) {
    // 配置中文字体
    let mut fonts = egui::FontDefinitions::default();

    // 添加系统中文字体 (Windows: Microsoft YaHei)
    fonts.font_data.insert(
        "chinese".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "C:\\Windows\\Fonts\\msyh.ttc"
        ))),
    );

    // 将中文字体添加到 Proportional 和 Monospace 字体族
    fonts.families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("chinese".to_owned());
    fonts.families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("chinese".to_owned());

    ctx.set_fonts(fonts);

    let mut visuals = Visuals::dark();

    // Window and panel backgrounds
    visuals.window_fill = BG_MID;
    visuals.panel_fill = BG_MID;
    visuals.faint_bg_color = BG_DARK;
    visuals.extreme_bg_color = BG_DARK;

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = BG_LIGHT;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, FG_SECONDARY);

    visuals.widgets.inactive.bg_fill = BG_LIGHT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, FG_PRIMARY);

    visuals.widgets.hovered.bg_fill = ACCENT.linear_multiply(0.3);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, FG_PRIMARY);

    visuals.widgets.active.bg_fill = ACCENT.linear_multiply(0.5);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, FG_PRIMARY);

    // Selection
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    // Hyperlinks
    visuals.hyperlink_color = ACCENT;

    // Rounding
    let rounding = Rounding::same(4.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.active.rounding = rounding;
    visuals.window_rounding = Rounding::same(8.0);

    ctx.set_visuals(visuals);

    // Style adjustments
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}
