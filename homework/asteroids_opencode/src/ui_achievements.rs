//! 成就界面 UI 模块
//!
//! 包含成就查看界面、成就卡片、解锁通知等渲染功能。

use macroquad::prelude::*;
use macroquad::text::Font;

use crate::achievement::{Achievement, AchievementCategory, AchievementId, AchievementManager};
use crate::background::Starfield;

/// 绘制成就查看界面
pub fn draw_achievements_screen(
    manager: &AchievementManager,
    font: Option<&Font>,
    _time: f64,
    scroll_offset: f32,
    starfield: &Starfield,
    time: f32,
) {
    // 绘制星空背景
    starfield.draw(time);

    // 标题（固定位置，不受滚动影响）- 亮色适配深色背景
    let title = "Achievements";
    let title_size = 48;
    let title_color = Color::new(0.8, 0.9, 1.0, 1.0);

    if let Some(f) = font {
        let title_dims = measure_text(title, Some(f), title_size, 1.0);
        draw_text_ex(
            title,
            screen_width() / 2. - title_dims.width / 2.,
            70.,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: title_color,
                ..Default::default()
            },
        );
    } else {
        let title_width = measure_text(title, None, title_size, 1.0).width;
        draw_text(
            title,
            screen_width() / 2. - title_width / 2.,
            70.,
            title_size as f32,
            title_color,
        );
    }

    // 统计信息（固定位置）
    let (unlocked, total) = manager.get_stats();
    let percentage = if total > 0 {
        (unlocked as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };
    let stats_text = format!("{} / {} ({}%)", unlocked, total, percentage);
    let stats_size = 24;

    if let Some(f) = font {
        let stats_dims = measure_text(&stats_text, Some(f), stats_size, 1.0);
        draw_text_ex(
            &stats_text,
            screen_width() / 2. - stats_dims.width / 2.,
            110.,
            TextParams {
                font: Some(f),
                font_size: stats_size,
                color: Color::new(0.6, 0.7, 0.8, 1.0),
                ..Default::default()
            },
        );
    } else {
        let stats_width = measure_text(&stats_text, None, stats_size, 1.0).width;
        draw_text(
            &stats_text,
            screen_width() / 2. - stats_width / 2.,
            110.,
            stats_size as f32,
            Color::new(0.6, 0.7, 0.8, 1.0),
        );
    }

    // ═══════════════════════════════════════════════════════════════
    // 进度条 (Progress Bar)
    // ═══════════════════════════════════════════════════════════════
    let bar_width = screen_width() * 0.5;
    let bar_height = 12.0;
    let bar_x = screen_width() / 2. - bar_width / 2.;
    let bar_y = 125.0;
    let fill_ratio = if total > 0 { unlocked as f32 / total as f32 } else { 0.0 };

    // 进度条背景
    draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.15, 0.18, 0.25, 0.8));
    draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.0, Color::new(0.3, 0.4, 0.5, 0.6));

    // 进度条填充（渐变效果）
    let fill_width = bar_width * fill_ratio;
    if fill_width > 0.0 {
        // 基础填充
        draw_rectangle(bar_x, bar_y, fill_width, bar_height, Color::new(0.9, 0.7, 0.2, 0.9));
        // 高光层
        draw_rectangle(bar_x, bar_y, fill_width, bar_height * 0.4, Color::new(1.0, 0.9, 0.5, 0.3));
        // 动态光效
        let pulse = 0.7 + 0.3 * (time * 2.0).sin();
        draw_rectangle(
            bar_x + fill_width - 3.0,
            bar_y,
            3.0,
            bar_height,
            Color::new(1.0, 1.0, 0.8, 0.5 * pulse),
        );
    }

    // 分类显示（应用滚动偏移）
    let categories = vec![
        AchievementCategory::Beginner,
        AchievementCategory::Combo,
        AchievementCategory::Survival,
        AchievementCategory::Duel,
        AchievementCategory::Perfectionist,
        AchievementCategory::Explorer,
        AchievementCategory::Veteran,
        AchievementCategory::Hidden,
    ];

    let mut y_offset = 160.0 + scroll_offset; // 应用滚动
    let panel_width = screen_width() * 0.85;
    let panel_x = screen_width() / 2. - panel_width / 2.;

    for category in categories {
        let achievements = manager.get_by_category(category);
        if achievements.is_empty() {
            continue;
        }

        // 计算该分类的解锁数量
        let category_unlocked = achievements.iter()
            .filter(|&&id| manager.get_progress(id).map(|p| p.unlocked).unwrap_or(false))
            .count();
        let category_total = achievements.len();

        // 分类标题（带解锁计数）
        draw_category_header_with_count(category, category_unlocked, category_total, panel_x, y_offset, panel_width, font);
        y_offset += 50.0;

        // 成就卡片（每行显示4个）
        let card_width = (panel_width - 60.0) / 4.0;
        let card_height = 120.0;
        let spacing = 20.0;

        for (i, &id) in achievements.iter().enumerate() {
            let row = i / 4;
            let col = i % 4;
            let x = panel_x + col as f32 * (card_width + spacing);
            let y = y_offset + row as f32 * (card_height + spacing);

            draw_achievement_card(manager, id, x, y, card_width, card_height, font);
        }

        let rows = achievements.len().div_ceil(4);
        y_offset += rows as f32 * (card_height + spacing) + 30.0;
    }

    // 底部提示（固定位置）- 亮色适配深色背景
    let hint = "[ESC] Back to Menu  |  [Mouse Wheel / ↑↓ W S] Scroll";
    let hint_size = 20;
    if let Some(f) = font {
        let hint_dims = measure_text(hint, Some(f), hint_size, 1.0);
        draw_text_ex(
            hint,
            screen_width() / 2. - hint_dims.width / 2.,
            screen_height() - 40.,
            TextParams {
                font: Some(f),
                font_size: hint_size,
                color: Color::new(0.6, 0.65, 0.75, 1.0),
                ..Default::default()
            },
        );
    } else {
        let hint_width = measure_text(hint, None, hint_size, 1.0).width;
        draw_text(
            hint,
            screen_width() / 2. - hint_width / 2.,
            screen_height() - 40.,
            hint_size as f32,
            Color::new(0.6, 0.65, 0.75, 1.0),
        );
    }
}

/// 绘制分类标题（带解锁计数）- 适配深色背景
fn draw_category_header_with_count(
    category: AchievementCategory,
    unlocked: usize,
    total: usize,
    x: f32,
    y: f32,
    width: f32,
    font: Option<&Font>,
) {
    let name = category.name();
    let count_text = format!("({}/{})", unlocked, total);
    let is_complete = unlocked == total && total > 0;

    // 背景颜色根据完成状态变化
    let bg_color = if is_complete {
        Color::new(0.12, 0.18, 0.14, 0.85) // 完成时带绿色调
    } else {
        Color::new(0.1, 0.12, 0.18, 0.8)
    };

    let border_color = if is_complete {
        Color::new(0.4, 0.8, 0.5, 0.7) // 完成时绿色边框
    } else {
        Color::new(0.4, 0.5, 0.7, 0.6)
    };

    draw_rectangle(x, y, width, 40.0, bg_color);
    draw_rectangle_lines(x, y, width, 40.0, 1.5, border_color);

    // 分类图标
    let icon = category_icon(category);
    draw_text_ex(
        icon,
        x + 15.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 22,
            color: if is_complete {
                Color::new(0.5, 0.9, 0.6, 1.0)
            } else {
                Color::new(0.6, 0.7, 0.85, 1.0)
            },
            ..Default::default()
        },
    );

    // 分类名称
    draw_text_ex(
        name,
        x + 45.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 26,
            color: Color::new(0.7, 0.8, 0.95, 1.0),
            ..Default::default()
        },
    );

    // 解锁计数
    let count_color = if is_complete {
        Color::new(0.5, 0.9, 0.6, 1.0) // 完成时绿色
    } else if unlocked > 0 {
        Color::new(0.9, 0.8, 0.4, 1.0) // 部分完成时金色
    } else {
        Color::new(0.5, 0.55, 0.65, 0.8) // 未开始时灰色
    };

    let name_width = measure_text(name, font, 26, 1.0).width;
    draw_text_ex(
        &count_text,
        x + 50.0 + name_width,
        y + 28.0,
        TextParams {
            font,
            font_size: 20,
            color: count_color,
            ..Default::default()
        },
    );

    // 完成标记
    if is_complete {
        draw_text_ex(
            "✓",
            x + width - 35.0,
            y + 28.0,
            TextParams {
                font,
                font_size: 24,
                color: Color::new(0.5, 0.9, 0.6, 1.0),
                ..Default::default()
            },
        );
    }
}

/// 获取分类图标
fn category_icon(category: AchievementCategory) -> &'static str {
    match category {
        AchievementCategory::Beginner => "🎯",
        AchievementCategory::Combo => "🔥",
        AchievementCategory::Survival => "💪",
        AchievementCategory::Duel => "⚔️",
        AchievementCategory::Perfectionist => "✨",
        AchievementCategory::Explorer => "🔍",
        AchievementCategory::Veteran => "🏆",
        AchievementCategory::Hidden => "🔮",
    }
}

/// 绘制分类标题 - 适配深色背景 (保留原版供兼容)
fn draw_category_header(
    category: AchievementCategory,
    x: f32,
    y: f32,
    width: f32,
    font: Option<&Font>,
) {
    let name = category.name();
    draw_rectangle(x, y, width, 40.0, Color::new(0.1, 0.12, 0.18, 0.8));
    draw_rectangle_lines(x, y, width, 40.0, 1.5, Color::new(0.4, 0.5, 0.7, 0.6));
    draw_text_ex(
        name,
        x + 20.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 26,
            color: Color::new(0.7, 0.8, 0.95, 1.0),
            ..Default::default()
        },
    );
}

/// 绘制单个成就卡片 - 增强视觉效果
fn draw_achievement_card(
    manager: &AchievementManager,
    id: AchievementId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font: Option<&Font>,
) {
    let achievement = Achievement::get(id);
    let progress = manager.get_progress(id);
    let unlocked = progress.map(|p| p.unlocked).unwrap_or(false);
    let time = macroquad::time::get_time() as f32;

    // ═══════════════════════════════════════════════════════════════
    // 卡片深度效果
    // ═══════════════════════════════════════════════════════════════

    if unlocked {
        // 解锁卡片：外发光效果
        let tier_color = achievement.tier.color();
        let glow_pulse = 0.6 + 0.4 * (time * 1.5 + id as u8 as f32 * 0.5).sin();

        // 外发光层
        for i in 1..=2 {
            let offset = i as f32 * 2.0;
            let glow_alpha = 0.08 * glow_pulse / i as f32;
            draw_rectangle(
                x - offset,
                y - offset,
                width + offset * 2.0,
                height + offset * 2.0,
                Color::new(tier_color.r, tier_color.g, tier_color.b, glow_alpha),
            );
        }

        // 阴影
        draw_rectangle(x + 3.0, y + 3.0, width, height, Color::new(0.0, 0.0, 0.0, 0.25));
    } else {
        // 锁定卡片：轻微阴影
        draw_rectangle(x + 2.0, y + 2.0, width, height, Color::new(0.0, 0.0, 0.0, 0.15));
    }

    // 背景颜色 - 解锁卡片更亮
    let bg_color = if unlocked {
        Color::new(0.14, 0.16, 0.24, 0.95)
    } else {
        Color::new(0.06, 0.08, 0.12, 0.7)
    };

    draw_rectangle(x, y, width, height, bg_color);

    // 解锁卡片顶部高光
    if unlocked {
        draw_rectangle(x, y, width, 3.0, Color::new(1.0, 1.0, 1.0, 0.1));
    }

    // 边框（根据等级显示不同颜色）
    let border_color = if unlocked {
        achievement.tier.color()
    } else {
        Color::new(0.3, 0.35, 0.4, 0.5)
    };
    let border_width = if unlocked { 2.5 } else { 1.5 };

    draw_rectangle_lines(x, y, width, height, border_width, border_color);

    // ═══════════════════════════════════════════════════════════════
    // 锁定标识
    // ═══════════════════════════════════════════════════════════════

    if !unlocked {
        // 锁定图标覆盖层
        draw_rectangle(x, y, width, height, Color::new(0.0, 0.0, 0.0, 0.2));

        // 锁定图标
        let lock_icon = if achievement.hidden { "🔮" } else { "🔒" };
        draw_text_ex(
            lock_icon,
            x + width - 28.0,
            y + 24.0,
            TextParams {
                font,
                font_size: 18,
                color: Color::new(0.5, 0.55, 0.6, 0.8),
                ..Default::default()
            },
        );
    }

    // 图标 - 适配深色背景
    let icon = if achievement.hidden && !unlocked {
        "?" // 隐藏成就未解锁时显示问号
    } else {
        achievement.icon
    };

    draw_text_ex(
        icon,
        x + width / 2. - 15.0,
        y + 35.0,
        TextParams {
            font,
            font_size: 32,
            color: if unlocked {
                Color::new(0.9, 0.92, 0.95, 1.0)
            } else {
                Color::new(0.5, 0.55, 0.6, 1.0)
            },
            ..Default::default()
        },
    );

    // 等级图标
    draw_text_ex(
        achievement.tier.icon(),
        x + 8.0,
        y + 24.0,
        TextParams {
            font,
            font_size: 20,
            color: if unlocked {
                achievement.tier.color()
            } else {
                Color::new(0.45, 0.5, 0.55, 0.7)
            },
            ..Default::default()
        },
    );

    // 名称 - 亮色适配深色背景
    let name = if achievement.hidden && !unlocked {
        "???"
    } else {
        achievement.name
    };

    let name_size = 16;
    let name_width = measure_text(name, font, name_size, 1.0).width;
    draw_text_ex(
        name,
        x + width / 2. - name_width / 2.,
        y + 65.0,
        TextParams {
            font,
            font_size: name_size,
            color: if unlocked {
                Color::new(0.85, 0.9, 0.95, 1.0)
            } else {
                Color::new(0.55, 0.6, 0.65, 1.0)
            },
            ..Default::default()
        },
    );

    // 进度或鼓励文案
    if unlocked {
        // 显示鼓励文案
        let quote = achievement.quote;
        let quote_size = 13;
        draw_wrapped_text_in_card(
            quote,
            x + 8.0,
            y + 82.0,
            width - 16.0,
            quote_size,
            Color::new(0.6, 0.7, 0.8, 1.0),
            font,
        );
    } else if let Some(p) = progress {
        // 显示进度
        if achievement.target > 0 {
            let progress_text = format!("{} / {}", p.current, achievement.target);
            let progress_size = 13;
            let progress_width = measure_text(&progress_text, font, progress_size, 1.0).width;
            draw_text_ex(
                &progress_text,
                x + width / 2. - progress_width / 2.,
                y + 95.0,
                TextParams {
                    font,
                    font_size: progress_size,
                    color: Color::new(0.55, 0.65, 0.75, 1.0),
                    ..Default::default()
                },
            );

            // 进度条
            let bar_width = width - 20.0;
            let bar_x = x + 10.0;
            let bar_y = y + 102.0;
            draw_rectangle(
                bar_x,
                bar_y,
                bar_width,
                8.0,
                Color::new(0.2, 0.25, 0.3, 0.5),
            );
            let fill = (p.current as f32 / achievement.target as f32).min(1.0);
            draw_rectangle(
                bar_x,
                bar_y,
                bar_width * fill,
                8.0,
                Color::new(0.3, 0.6, 0.9, 0.85),
            );
        }
    }
}

/// 在卡片内绘制自动换行文本（简化版）
fn draw_wrapped_text_in_card(
    text: &str,
    x: f32,
    y: f32,
    max_width: f32,
    font_size: u16,
    color: Color,
    font: Option<&Font>,
) {
    let words: Vec<&str> = text.split(' ').collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let width = measure_text(&test_line, font, font_size, 1.0).width;

        if width <= max_width {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let line_height = font_size as f32 + 2.0;
    for (i, line) in lines.iter().take(2).enumerate() {
        // 最多显示2行
        draw_text_ex(
            line,
            x,
            y + i as f32 * line_height,
            TextParams {
                font,
                font_size,
                color,
                ..Default::default()
            },
        );
    }
}

/// 绘制成就解锁提示（浮动通知）
#[allow(dead_code)]
pub fn draw_achievement_unlock_toast(
    id: AchievementId,
    time_since_unlock: f32,
    font: Option<&Font>,
) {
    let achievement = Achievement::get(id);

    // 动画：从右侧滑入，停留，然后淡出
    let duration = 5.0; // 总持续时间5秒
    let slide_in = 0.5; // 滑入0.5秒
    let fade_out = 1.0; // 淡出1秒

    if time_since_unlock > duration {
        return;
    }

    let panel_width = 350.0;
    let panel_height = 100.0;
    let target_x = screen_width() - panel_width - 20.0;
    let y = 20.0;

    // 计算动画位置
    let x = if time_since_unlock < slide_in {
        // 滑入动画
        let progress = time_since_unlock / slide_in;
        let eased = 1.0 - (1.0 - progress).powi(3); // ease-out cubic
        screen_width() + (target_x - screen_width()) * eased
    } else {
        target_x
    };

    // 计算透明度
    let alpha = if time_since_unlock > duration - fade_out {
        // 淡出动画

        (duration - time_since_unlock) / fade_out
    } else {
        1.0
    };

    // 绘制面板（带阴影）
    draw_rectangle(
        x + 4.0,
        y + 6.0,
        panel_width,
        panel_height,
        Color::new(0.0, 0.0, 0.0, 0.2 * alpha),
    );

    let bg_color = achievement.tier.color();
    draw_rectangle(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(bg_color.r, bg_color.g, bg_color.b, 0.95 * alpha),
    );

    draw_rectangle_lines(
        x,
        y,
        panel_width,
        panel_height,
        3.0,
        Color::new(1.0, 1.0, 1.0, 0.8 * alpha),
    );

    // 标题
    draw_text_ex(
        "Achievement Unlocked!",
        x + 20.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    // 图标和名称
    draw_text_ex(
        achievement.icon,
        x + 20.0,
        y + 60.0,
        TextParams {
            font,
            font_size: 32,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    draw_text_ex(
        achievement.name,
        x + 65.0,
        y + 58.0,
        TextParams {
            font,
            font_size: 24,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    // 鼓励文案
    draw_text_ex(
        achievement.quote,
        x + 65.0,
        y + 82.0,
        TextParams {
            font,
            font_size: 16,
            color: Color::new(1.0, 1.0, 1.0, 0.9 * alpha),
            ..Default::default()
        },
    );

    // 等级图标
    draw_text_ex(
        achievement.tier.icon(),
        x + panel_width - 40.0,
        y + 30.0,
        TextParams {
            font,
            font_size: 28,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );
}

/// 绘制成就解锁提示（带垂直偏移）
pub fn draw_achievement_unlock_toast_offset(
    id: AchievementId,
    time_since_unlock: f32,
    y_offset: f32,
    font: Option<&Font>,
) {
    let achievement = Achievement::get(id);

    // 动画：从右侧滑入，停留，然后淡出
    let duration = 5.0; // 总持续时间5秒
    let slide_in = 0.5; // 滑入0.5秒
    let fade_out = 1.0; // 淡出1秒

    if time_since_unlock > duration {
        return;
    }

    let panel_width = 350.0;
    let panel_height = 100.0;
    let target_x = screen_width() - panel_width - 20.0;
    let y = 20.0 + y_offset;

    // 计算动画位置
    let x = if time_since_unlock < slide_in {
        // 滑入动画
        let progress = time_since_unlock / slide_in;
        let eased = 1.0 - (1.0 - progress).powi(3); // ease-out cubic
        screen_width() + (target_x - screen_width()) * eased
    } else {
        target_x
    };

    // 计算透明度
    let alpha = if time_since_unlock > duration - fade_out {
        // 淡出动画

        (duration - time_since_unlock) / fade_out
    } else {
        1.0
    };

    // 绘制面板（带阴影）
    draw_rectangle(
        x + 4.0,
        y + 6.0,
        panel_width,
        panel_height,
        Color::new(0.0, 0.0, 0.0, 0.2 * alpha),
    );

    let bg_color = achievement.tier.color();
    draw_rectangle(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(bg_color.r, bg_color.g, bg_color.b, 0.95 * alpha),
    );

    draw_rectangle_lines(
        x,
        y,
        panel_width,
        panel_height,
        3.0,
        Color::new(1.0, 1.0, 1.0, 0.8 * alpha),
    );

    // 标题
    draw_text_ex(
        "Achievement Unlocked!",
        x + 20.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    // 图标和名称
    draw_text_ex(
        achievement.icon,
        x + 20.0,
        y + 60.0,
        TextParams {
            font,
            font_size: 32,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    draw_text_ex(
        achievement.name,
        x + 65.0,
        y + 58.0,
        TextParams {
            font,
            font_size: 24,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );

    // 鼓励文案
    draw_text_ex(
        achievement.quote,
        x + 65.0,
        y + 82.0,
        TextParams {
            font,
            font_size: 16,
            color: Color::new(1.0, 1.0, 1.0, 0.9 * alpha),
            ..Default::default()
        },
    );

    // 等级图标
    draw_text_ex(
        achievement.tier.icon(),
        x + panel_width - 40.0,
        y + 30.0,
        TextParams {
            font,
            font_size: 28,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );
}

/// 绘制消息提示（用于显示重置成功等通知）
pub fn draw_message_toast(message: &str, time_since_show: f32, font: Option<&Font>) {
    let display_duration = 3.0; // 显示3秒

    if time_since_show > display_duration {
        return;
    }

    // 计算透明度（淡入淡出效果）
    let alpha = if time_since_show < 0.3 {
        // 前0.3秒淡入
        time_since_show / 0.3
    } else if time_since_show > display_duration - 0.5 {
        // 最后0.5秒淡出
        (display_duration - time_since_show) / 0.5
    } else {
        1.0
    };

    let center_x = screen_width() / 2.0;
    let y = 150.0; // 屏幕上方

    // 面板尺寸
    let padding = 30.0;
    let text_size = 24;
    let text_width = measure_text(message, font, text_size, 1.0).width;
    let panel_width = text_width + padding * 2.0;
    let panel_height = 60.0;
    let x = center_x - panel_width / 2.0;

    // 绘制阴影
    draw_rectangle(
        x + 4.0,
        y + 4.0,
        panel_width,
        panel_height,
        Color::new(0.0, 0.0, 0.0, 0.3 * alpha),
    );

    // 绘制背景
    draw_rectangle(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(0.2, 0.8, 0.4, 0.95 * alpha), // 绿色背景表示成功
    );

    // 绘制边框
    draw_rectangle_lines(
        x,
        y,
        panel_width,
        panel_height,
        3.0,
        Color::new(1.0, 1.0, 1.0, 0.9 * alpha),
    );

    // 绘制消息文本
    draw_text_ex(
        message,
        x + padding,
        y + panel_height / 2.0 + 8.0,
        TextParams {
            font,
            font_size: text_size,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );
}
