//! UI 模块
//!
//! 所有用户界面和渲染组件。
//!
//! ## 功能
//! - 玩家 HUD（生命、分数、护盾、生存时间）
//! - 模式选择界面
//! - 等待/游戏结束画面
//! - 暂停菜单
//! - 对战模式：回合结束、连击显示
//! - 渐变背景和阴影面板
//! - 文本居中辅助函数

use macroquad::prelude::*;
use macroquad::text::Font;

use crate::achievement::{Achievement, AchievementCategory, AchievementId, AchievementManager};
use crate::bullet::WeaponType;
use crate::duel::DuelState;
use crate::player::Player;
use crate::{GameMode, GameSettings, PauseSelection, SettingOption};

pub enum HudMode {
    Waiting,
    Active { time: f64 },
}

pub fn draw_players_hud(players: &[Player], mode: HudMode, font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        let (status, timer, hud_time) = match mode {
            HudMode::Waiting => ("READY".to_string(), 0.0, None),
            HudMode::Active { time } => {
                let timer = player.survival_time(time);
                let status = if player.is_invulnerable(time) {
                    format!("SAFE {:.1}s", player.invulnerability_remaining(time))
                } else if player.alive {
                    "ALIVE".to_string()
                } else {
                    "DOWN".to_string()
                };
                (status, timer, Some(time))
            }
        };

        let shield_text = if let Some(time) = hud_time {
            if player.shield_active(time) {
                format!("Shield {:.1}s", player.shield_remaining(time))
            } else {
                "Shield --".to_string()
            }
        } else {
            "Shield --".to_string()
        };

        let weapon_text = match player.weapon_type {
            WeaponType::Normal => "Normal",
            WeaponType::Spread => "Spread",
            WeaponType::Penetrating => "Penetrating",
        };

        let text = format!(
            "{} | Lives: {} | {} | Weapon: {} | Score: {} | Survival: {:.1}s | Status: {}",
            player.label,
            player.lives,
            shield_text,
            weapon_text,
            player.score.value(),
            timer,
            status
        );
        let y = 32. + idx as f32 * 36.;
        let panel_width = screen_width() * 0.55;
        let panel_color = Color::new(1.0, 1.0, 1.0, 0.35);
        draw_rectangle(12., y - 26., panel_width, 34., panel_color);
        draw_rectangle_lines(
            12.,
            y - 26.,
            panel_width,
            34.,
            1.5,
            Color::new(1.0, 1.0, 1.0, 0.45),
        );
        draw_text_ex(
            &text,
            20.,
            y,
            TextParams {
                font_size: 24,
                color: player.color,
                font,
                ..Default::default()
            },
        );
    }
}

pub fn draw_waiting_screen(message: &str, font: Option<&Font>) {
    draw_gradient_background(
        Color::new(0.9, 0.92, 0.97, 1.0),
        Color::new(0.84, 0.86, 0.93, 1.0),
    );
    let panel_width = screen_width() * 0.6;
    let panel_height = 160.;
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = screen_height() / 2. - panel_height / 2.;
    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(1.0, 1.0, 1.0, 0.9),
    );

    let font_size = 32.;
    draw_text_centered(
        message,
        screen_height() / 2. + font_size / 4.,
        font_size as u16,
        Color::new(0.25, 0.3, 0.35, 1.0),
        font,
    );
}

pub fn draw_game_over_message(message: &str, font: Option<&Font>) {
    draw_gradient_background(
        Color::new(0.91, 0.93, 0.99, 1.0),
        Color::new(0.82, 0.86, 0.94, 1.0),
    );
    let banner_width = screen_width() * 0.6;
    let banner_height = 130.;
    let banner_x = screen_width() / 2. - banner_width / 2.;
    let banner_y = screen_height() * 0.35; // 稍微上移到 35% 位置

    // 使用渐变色背景面板
    draw_shadow_panel(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        Color::new(0.95, 0.97, 1.0, 0.98),
    );

    // 添加彩色边框
    draw_rectangle_lines(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        3.0,
        Color::new(0.2, 0.5, 0.9, 0.8),
    );

    let font_size = 36.;
    // 使用醒目的蓝色
    draw_text_centered(
        message,
        banner_y + banner_height / 2. + font_size / 4.,
        font_size as u16,
        Color::new(0.1, 0.3, 0.7, 1.0),
        font,
    );
}

pub fn draw_survival_record(record: u32, font: Option<&Font>) {
    let text = format!("Survival record: {}", record);
    let width = measure_text(&text, font, 22, 1.0).width;
    draw_rectangle(
        screen_width() - width - 80.,
        24.,
        width + 48.,
        34.,
        Color::new(1.0, 1.0, 1.0, 0.4),
    );
    draw_text_ex(
        &text,
        screen_width() - width - 60.,
        48.,
        TextParams {
            font_size: 22,
            color: Color::new(0.3, 0.35, 0.45, 1.0),
            font,
            ..Default::default()
        },
    );
}

pub fn draw_center_scores(
    players: &[Player],
    time: f64,
    survival_record: u32,
    font: Option<&Font>,
) {
    let mut combined_scores: Vec<_> = players
        .iter()
        .map(|player| (player.label, player.score.value(), player.color))
        .collect();
    combined_scores.sort_by(|a, b| b.1.cmp(&a.1));
    let best_score = combined_scores.first().map(|entry| entry.1).unwrap_or(0);

    let line_height = 38.;
    let panel_padding = 50.;
    let panel_width = screen_width() * 0.75;
    let panel_height = panel_padding * 2. + line_height * combined_scores.len() as f32 + 60.;
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = screen_height() * 0.55; // 在 55% 位置（Game Over banner 下方）

    // 使用更亮的面板
    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.98, 0.99, 1.0, 0.95),
    );

    // 添加渐变边框
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        4.0,
        Color::new(0.3, 0.6, 0.95, 0.9),
    );

    // 标题使用醒目的颜色
    let header = format!(
        "Highest score this run: {}   |   Survival record: {}",
        best_score, survival_record
    );
    let header_width = measure_text(&header, font, 32, 1.0).width;

    // 标题阴影效果
    draw_text_ex(
        &header,
        screen_width() / 2. - header_width / 2. + 2.,
        panel_y + 52.,
        TextParams {
            font_size: 32,
            color: Color::new(0.0, 0.0, 0.0, 0.15),
            font,
            ..Default::default()
        },
    );
    draw_text_ex(
        &header,
        screen_width() / 2. - header_width / 2.,
        panel_y + 50.,
        TextParams {
            font_size: 32,
            color: Color::new(0.15, 0.35, 0.65, 1.0),
            font,
            ..Default::default()
        },
    );

    let base_y = panel_y + 100.;
    for (idx, player) in combined_scores.iter().enumerate() {
        let text = format!(
            "{}  |  Score: {}  |  Survival: {:.1}s",
            player.0,
            player.1,
            players
                .iter()
                .find(|p| p.label == player.0)
                .map(|p| p.survival_time(time))
                .unwrap_or(0.0)
        );
        let text_width = measure_text(&text, font, 32, 1.0).width;
        let y = base_y + idx as f32 * line_height;

        // 添加文字阴影
        draw_text_ex(
            &text,
            screen_width() / 2. - text_width / 2. + 2.,
            y + 2.,
            TextParams {
                font,
                font_size: 32,
                color: Color::new(0.0, 0.0, 0.0, 0.2),
                ..Default::default()
            },
        );

        // 使用更鲜艳的玩家颜色
        let bright_color = Color::new(player.2.r * 0.8, player.2.g * 0.8, player.2.b * 0.8, 1.0);

        draw_text_ex(
            &text,
            screen_width() / 2. - text_width / 2.,
            y,
            TextParams {
                font,
                font_size: 32,
                color: bright_color,
                ..Default::default()
            },
        );
    }
}

pub fn draw_mode_selection(
    selection: GameMode,
    settings: &GameSettings,
    achievements: &AchievementManager,
    font: Option<&Font>,
) {
    draw_gradient_background(
        Color::new(0.91, 0.93, 0.99, 1.0),
        Color::new(0.8, 0.86, 0.95, 1.0),
    );

    let title = "Choose Your Adventure";
    let subtitle = "Press [M] later to revisit this screen";
    let title_size = 48;
    let subtitle_size = 24;

    // 使用自定义字体绘制标题
    if let Some(f) = font {
        let title_dims = measure_text(title, Some(f), title_size, 1.0);
        draw_text_ex(
            title,
            screen_width() / 2. - title_dims.width / 2.,
            70.,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: BLACK,
                ..Default::default()
            },
        );
        let subtitle_dims = measure_text(subtitle, Some(f), subtitle_size, 1.0);
        draw_text_ex(
            subtitle,
            screen_width() / 2. - subtitle_dims.width / 2.,
            115.,
            TextParams {
                font: Some(f),
                font_size: subtitle_size,
                color: DARKGRAY,
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
            BLACK,
        );
        let subtitle_width = measure_text(subtitle, None, subtitle_size, 1.0).width;
        draw_text(
            subtitle,
            screen_width() / 2. - subtitle_width / 2.,
            115.,
            subtitle_size as f32,
            DARKGRAY,
        );
    }

    // 4个卡片纵向排列
    let card_width = screen_width() * 0.65;
    let card_height = 140.;
    let spacing = 18.;
    let total_height = card_height * 5. + spacing * 4.;  // 改为5个卡片
    let start_y = (screen_height() - total_height) / 2. + 20.;
    let card_x = screen_width() / 2. - card_width / 2.;

    // Survival 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y,
        width: card_width,
        height: card_height,
        title: "Survival",
        desc: "Team up to clear every asteroid and push your score higher.",
        detail: "Best for co-op pilots. Supports two players on one keyboard.",
        active: matches!(selection, GameMode::Survival),
        accent: BLUE,
        footer: "[Enter] Start",
        font,
    });

    // Duel 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing),
        width: card_width,
        height: card_height,
        title: "Duel",
        desc: "Face each other in timed shoot-outs with streak bonuses.",
        detail: "Competitive mode with flag capture and killstreak rewards.",
        active: matches!(selection, GameMode::Duel),
        accent: RED,
        footer: "[Enter] Start",
        font,
    });

    // Online 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 2.,
        width: card_width,
        height: card_height,
        title: "Online Multiplayer",
        desc: "Challenge players online in real-time battles.",
        detail: "Connect to server and compete with other pilots worldwide.",
        active: matches!(selection, GameMode::Online),
        accent: Color::new(0.6, 0.3, 0.9, 1.0), // 紫色
        footer: "[Enter] Connect",
        font,
    });

    // Achievements 卡片
    let (unlocked, total) = achievements.get_stats();
    let achievements_summary = format!("Unlocked: {} / {} achievements", unlocked, total);
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 3.,
        width: card_width,
        height: card_height,
        title: "Achievements",
        desc: &achievements_summary,
        detail: "Track your progress and unlock rewards as you play.",
        active: matches!(selection, GameMode::Achievements),
        accent: Color::new(0.9, 0.7, 0.2, 1.0), // 金色
        footer: "[Enter] View",
        font,
    });

    // Settings 卡片
    let settings_summary = format!(
        "Lives: {} | Ship: {:.1}x | Asteroids: {:.1}x",
        settings.starting_lives, settings.ship_speed_multiplier, settings.asteroid_speed_multiplier,
    );
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 4.,
        width: card_width,
        height: card_height,
        title: "Settings",
        desc: &settings_summary,
        detail: "Configure game difficulty, effects, and features.",
        active: matches!(selection, GameMode::Settings),
        accent: Color::new(0.3, 0.7, 0.3, 1.0), // 绿色
        footer: "[Enter] Configure",
        font,
    });

    // 底部提示使用自定义字体
    let hint = "[Up/Down W/S] Select  |  [Enter Space] Confirm";
    if let Some(f) = font {
        let hint_dims = measure_text(hint, Some(f), 26, 1.0);
        draw_text_ex(
            hint,
            screen_width() / 2. - hint_dims.width / 2.,
            screen_height() - 50.,
            TextParams {
                font: Some(f),
                font_size: 26,
                color: DARKGRAY,
                ..Default::default()
            },
        );
    } else {
        let hint_width = measure_text(hint, None, 26, 1.0).width;
        draw_text(
            hint,
            screen_width() / 2. - hint_width / 2.,
            screen_height() - 50.,
            26.,
            DARKGRAY,
        );
    }
}

struct ModeCardParams<'a> {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    title: &'a str,
    desc: &'a str,
    detail: &'a str,
    active: bool,
    accent: Color,
    footer: &'a str,
    font: Option<&'a Font>,
}

fn draw_mode_card(params: ModeCardParams) {
    let base = if params.active {
        Color::new(1.0, 1.0, 1.0, 0.98)
    } else {
        Color::new(0.94, 0.94, 0.94, 0.95)
    };
    let border = if params.active { 4.0 } else { 2.0 };

    draw_shadow_panel(params.x, params.y, params.width, params.height, base);
    draw_rectangle_lines(
        params.x,
        params.y,
        params.width,
        params.height,
        border,
        params.accent,
    );

    let icon_x = params.x + params.width * 0.2;
    let icon_y = params.y + 70.;
    draw_circle(
        icon_x,
        icon_y,
        28.,
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.15),
    );
    draw_circle(icon_x, icon_y, 20., params.accent);
    draw_line(
        icon_x - 32.,
        icon_y,
        icon_x + 32.,
        icon_y,
        5.,
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.5),
    );
    draw_circle(
        icon_x + 60.,
        icon_y - 12.,
        12.,
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.3),
    );

    draw_text_ex(
        params.title,
        params.x + 24.,
        params.y + 40.,
        TextParams {
            font: params.font,
            font_size: 32, // 增大从 28 到 32
            color: params.accent,
            ..Default::default()
        },
    );

    let tag_width = if let Some(f) = params.font {
        measure_text(params.footer, Some(f), 20, 1.0).width + 24.
    } else {
        measure_text(params.footer, None, 20, 1.0).width + 24.
    };

    draw_rectangle(
        params.x + params.width - tag_width - 24.,
        params.y + 18.,
        tag_width,
        30.,
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.15),
    );
    draw_text_ex(
        params.footer,
        params.x + params.width - tag_width - 12.,
        params.y + 39.,
        TextParams {
            font: params.font,
            font_size: 20, // 增大从 18 到 20
            color: params.accent,
            ..Default::default()
        },
    );

    // 绘制描述（支持自动换行）
    draw_wrapped_text(
        params.desc,
        params.x + 24.,
        params.y + 85.,
        params.width - 48.,
        22, // 增大从 18 到 22
        DARKGRAY,
        params.font,
    );

    // Detail 也支持换行
    draw_wrapped_text(
        params.detail,
        params.x + 24.,
        params.y + params.height - 42.,
        params.width - 48.,
        19, // 增大从 16 到 19
        GRAY,
        params.font,
    );
}

/// 绘制自动换行文本
fn draw_wrapped_text(
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

        let width = if let Some(f) = font {
            measure_text(&test_line, Some(f), font_size, 1.0).width
        } else {
            measure_text(&test_line, None, font_size, 1.0).width
        };

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

    let line_height = font_size as f32 + 4.0; // 减小行距从 6.0 到 4.0
    for (i, line) in lines.iter().enumerate() {
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

pub fn draw_pause_menu(selection: PauseSelection, font: Option<&Font>) {
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.55),
    );

    let panel_width = screen_width() * 0.55;
    let panel_height = 480.;
    let x = screen_width() / 2. - panel_width / 2.;
    let y = screen_height() / 2. - panel_height / 2.;
    draw_shadow_panel(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(0.12, 0.14, 0.2, 0.95),
    );

    let title = "Paused";
    draw_text_ex(
        title,
        screen_width() / 2. - measure_text(title, font, 60, 1.0).width / 2.,
        y + 88.,
        TextParams {
            font_size: 60,
            color: WHITE,
            font,
            ..Default::default()
        },
    );

    draw_pause_option(
        screen_height() / 2. - 50.,
        "Resume game",
        "Return to the current run.",
        matches!(selection, PauseSelection::Resume),
        font,
    );
    draw_pause_option(
        screen_height() / 2. + 60.,
        "Back to mode select",
        "Abandon this run and choose another mode.",
        matches!(selection, PauseSelection::ModeSelect),
        font,
    );

    let hint = "[Enter] confirm  ·  [Esc] resume";
    let hint_width = measure_text(hint, font, 24, 1.0).width;
    draw_text_ex(
        hint,
        screen_width() / 2. - hint_width / 2.,
        y + panel_height - 24.,
        TextParams {
            font,
            font_size: 24,
            color: LIGHTGRAY,
            ..Default::default()
        },
    );
}

fn draw_pause_option(y: f32, title: &str, desc: &str, active: bool, font: Option<&Font>) {
    let width = 420.;
    let x = screen_width() / 2. - width / 2.;
    let color = if active {
        Color::new(0.4, 0.7, 1.0, 1.0)
    } else {
        LIGHTGRAY
    };
    draw_rectangle(
        x,
        y,
        width,
        60.,
        if active {
            Color::new(1.0, 1.0, 1.0, 0.15)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.08)
        },
    );
    draw_rectangle_lines(x, y, width, 60., if active { 3.0 } else { 1.5 }, color);
    draw_text_ex(
        title,
        x + 16.,
        y + 28.,
        TextParams {
            font,
            font_size: 28,
            color,
            ..Default::default()
        },
    );
    draw_text_ex(
        desc,
        x + 16.,
        y + 48.,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(0.8, 0.85, 0.95, 1.0),
            ..Default::default()
        },
    );
}

pub fn draw_pause_hint(font: Option<&Font>) {
    let text = "Press [Esc] to pause";
    let padding = 14.;
    let width = measure_text(text, font, 22, 1.0).width + padding * 2.;
    let x = screen_width() - width - 20.;
    let y = screen_height() - 60.;
    draw_rectangle(x, y, width, 36., Color::new(1.0, 1.0, 1.0, 0.5));
    draw_text_ex(
        text,
        x + padding,
        y + 24.,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.2, 0.25, 0.35, 1.0),
            ..Default::default()
        },
    );
}

pub fn draw_victory_pause_overlay(remaining: f64, font: Option<&Font>) {
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    let panel_width = screen_width() * 0.4;
    let panel_height = 150.;
    let x = screen_width() / 2. - panel_width / 2.;
    let y = screen_height() / 2. - panel_height / 2.;
    draw_shadow_panel(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(1.0, 1.0, 1.0, 0.92),
    );

    let title = "Wave cleared!";
    let title_width = measure_text(title, font, 36, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2. - title_width / 2.,
        y + 60.,
        TextParams {
            font,
            font_size: 36,
            color: Color::new(0.2, 0.35, 0.4, 1.0),
            ..Default::default()
        },
    );
    let subtitle = format!("Next screen in {:.1}s", remaining);
    let subtitle_width = measure_text(&subtitle, font, 22, 1.0).width;
    draw_text_ex(
        &subtitle,
        screen_width() / 2. - subtitle_width / 2.,
        y + 100.,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.3, 0.4, 0.5, 1.0),
            ..Default::default()
        },
    );
}

pub fn draw_gradient_background(top: Color, bottom: Color) {
    let steps = 32;
    let slice = screen_height() / steps as f32;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let color = Color::new(
            lerp(top.r, bottom.r, t),
            lerp(top.g, bottom.g, t),
            lerp(top.b, bottom.b, t),
            1.0,
        );
        draw_rectangle(0., i as f32 * slice, screen_width(), slice + 1., color);
    }
}

pub fn draw_shadow_panel(x: f32, y: f32, width: f32, height: f32, color: Color) {
    draw_rectangle(
        x + 6.,
        y + 8.,
        width,
        height,
        Color::new(0.0, 0.0, 0.0, 0.08),
    );
    draw_rectangle(x, y, width, height, color);
}

/// 在屏幕水平居中绘制文本
fn draw_text_centered(text: &str, y: f32, font_size: u16, color: Color, font: Option<&Font>) {
    let width = measure_text(text, font, font_size, 1.0).width;
    draw_text_ex(
        text,
        screen_width() / 2. - width / 2.,
        y,
        TextParams {
            font,
            font_size,
            color,
            ..Default::default()
        },
    );
}

/// 绘制 Duel 回合结束界面
pub fn draw_round_end(winner_idx: usize, duel_state: &DuelState, font: Option<&Font>) {
    // 半透明黑色背景
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.5),
    );

    // 主面板
    let panel_width = screen_width() * 0.5;
    let panel_height = 280.;
    let x = screen_width() / 2. - panel_width / 2.;
    let y = screen_height() / 2. - panel_height / 2.;
    draw_shadow_panel(
        x,
        y,
        panel_width,
        panel_height,
        Color::new(1.0, 1.0, 1.0, 0.95),
    );

    // 标题
    let title = format!(
        "Player {} wins Round {}!",
        winner_idx + 1,
        duel_state.current_round
    );
    draw_text_centered(&title, y + 60., 36, Color::new(0.2, 0.35, 0.4, 1.0), font);

    // 回合比分
    let score_text = format!(
        "Score: {} - {}",
        duel_state.round_wins[0], duel_state.round_wins[1]
    );
    draw_text_centered(
        &score_text,
        y + 110.,
        28,
        Color::new(0.3, 0.4, 0.5, 1.0),
        font,
    );

    // 赛制信息
    let rounds_to_win = duel_state.round_mode.rounds_to_win();
    let mode_text = format!("First to {} rounds", rounds_to_win);
    draw_text_centered(
        &mode_text,
        y + 150.,
        20,
        Color::new(0.4, 0.5, 0.6, 1.0),
        font,
    );

    // 提示
    let hint = "Press [Space] or [Enter] to continue";
    draw_text_centered(hint, y + 200., 18, Color::new(0.5, 0.6, 0.7, 1.0), font);
}

/// 绘制连击状态（仅在 Duel 模式下显示）
pub fn draw_killstreak(players: &[Player], font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        if player.killstreak < 2 {
            continue; // 只显示 2 连击以上
        }

        // 根据玩家索引决定显示位置（左右两侧）
        let x = if idx == 0 {
            screen_width() * 0.15
        } else {
            screen_width() * 0.85
        };
        let y = screen_height() * 0.3;

        // 连击文本和颜色
        let (text, color) = if let Some(level) = player.killstreak_level() {
            (level, Color::new(1.0, 0.6, 0.1, 1.0))
        } else {
            continue;
        };

        // 绘制连击提示
        let streak_count = format!("{}x", player.killstreak);
        let streak_width = measure_text(&streak_count, font, 48, 1.0).width;
        let text_width = measure_text(text, font, 28, 1.0).width;

        // 阴影效果
        draw_text_ex(
            &streak_count,
            x - streak_width / 2. + 2.,
            y + 2.,
            TextParams {
                font,
                font_size: 48,
                color: Color::new(0.0, 0.0, 0.0, 0.3),
                ..Default::default()
            },
        );
        draw_text_ex(
            &streak_count,
            x - streak_width / 2.,
            y,
            TextParams {
                font,
                font_size: 48,
                color,
                ..Default::default()
            },
        );

        draw_text_ex(
            text,
            x - text_width / 2. + 2.,
            y + 40. + 2.,
            TextParams {
                font,
                font_size: 28,
                color: Color::new(0.0, 0.0, 0.0, 0.3),
                ..Default::default()
            },
        );
        draw_text_ex(
            text,
            x - text_width / 2.,
            y + 40.,
            TextParams {
                font,
                font_size: 28,
                color: player.color,
                ..Default::default()
            },
        );
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 性能监控统计数据
pub struct DebugStats {
    pub fps: f32,
    pub entity_count: usize,
    pub quadtree_depth: usize,
    pub particle_count: usize,
}

/// 绘制性能监控面板（左上角）
pub fn draw_debug_panel(stats: &DebugStats, font: Option<&Font>) {
    let panel_x = 12.0;
    let panel_y = screen_height() - 130.0;
    let panel_width = 280.0;
    let panel_height = 115.0;

    // 半透明背景面板
    draw_rectangle(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        2.0,
        Color::new(0.0, 1.0, 0.5, 0.8),
    );

    let text_x = panel_x + 10.0;
    let mut text_y = panel_y + 25.0;
    let line_height = 22.0;
    let font_size = 18;

    // FPS（绿色表示性能良好）
    let fps_color = if stats.fps >= 55.0 {
        Color::new(0.0, 1.0, 0.3, 1.0)
    } else if stats.fps >= 30.0 {
        Color::new(1.0, 1.0, 0.0, 1.0)
    } else {
        Color::new(1.0, 0.2, 0.2, 1.0)
    };
    draw_text_ex(
        &format!("FPS: {:.1}", stats.fps),
        text_x,
        text_y,
        TextParams {
            font,
            font_size,
            color: fps_color,
            ..Default::default()
        },
    );

    text_y += line_height;
    draw_text_ex(
        &format!("Entities: {}", stats.entity_count),
        text_x,
        text_y,
        TextParams {
            font,
            font_size,
            color: WHITE,
            ..Default::default()
        },
    );

    text_y += line_height;
    draw_text_ex(
        &format!("Particles: {}", stats.particle_count),
        text_x,
        text_y,
        TextParams {
            font,
            font_size,
            color: WHITE,
            ..Default::default()
        },
    );

    text_y += line_height;
    draw_text_ex(
        &format!("QuadTree Depth: {}", stats.quadtree_depth),
        text_x,
        text_y,
        TextParams {
            font,
            font_size,
            color: Color::new(0.5, 0.8, 1.0, 1.0),
            ..Default::default()
        },
    );

    // 提示文本
    let hint = "[F3] Close Debug";
    let hint_width = measure_text(hint, font, 14, 1.0).width;
    draw_text_ex(
        hint,
        panel_x + panel_width - hint_width - 10.0,
        panel_y + panel_height - 8.0,
        TextParams {
            font,
            font_size: 14,
            color: Color::new(0.7, 0.7, 0.7, 0.8),
            ..Default::default()
        },
    );
}

/// 绘制慢动作指示器
pub fn draw_slow_motion_indicator(time_scale: f32, font: Option<&Font>) {
    if time_scale >= 0.99 {
        return; // 正常速度，不显示
    }

    let center_x = screen_width() / 2.0;
    let center_y = screen_height() - 80.0;

    // 半透明背景
    draw_rectangle(
        center_x - 150.0,
        center_y - 30.0,
        300.0,
        50.0,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );

    // 边框
    draw_rectangle_lines(
        center_x - 150.0,
        center_y - 30.0,
        300.0,
        50.0,
        2.0,
        Color::new(0.0, 0.8, 1.0, 0.9),
    );

    // 文字
    let text = format!("SLOW MOTION [{:.0}%]", time_scale * 100.0);
    let text_width = measure_text(&text, font, 28, 1.0).width;

    // 阴影
    draw_text_ex(
        &text,
        center_x - text_width / 2.0 + 2.0,
        center_y + 7.0,
        TextParams {
            font,
            font_size: 28,
            color: Color::new(0.0, 0.0, 0.0, 0.5),
            ..Default::default()
        },
    );

    // 主文字（青色）
    draw_text_ex(
        &text,
        center_x - text_width / 2.0,
        center_y + 5.0,
        TextParams {
            font,
            font_size: 28,
            color: Color::new(0.0, 0.9, 1.0, 1.0),
            ..Default::default()
        },
    );
}

/// 绘制设置界面
pub fn draw_settings_screen(settings: &GameSettings, selected: SettingOption, font: Option<&Font>, scroll_offset: f32) {
    // 渐变背景
    draw_gradient_background(
        Color::new(0.90, 0.92, 0.98, 1.0),
        Color::new(0.82, 0.86, 0.94, 1.0),
    );

    // 标题（固定位置）
    let title = "Game Settings";
    let title_size = 42;
    let title_color = Color::new(0.15, 0.3, 0.6, 1.0);

    if let Some(f) = font {
        let title_dims = measure_text(title, Some(f), title_size, 1.0);
        draw_text_ex(
            title,
            screen_width() / 2. - title_dims.width / 2.,
            80.,
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
            80.,
            title_size as f32,
            title_color,
        );
    }

    // 设置面板（应用滚动偏移）
    let panel_width = screen_width() * 0.65;
    let panel_height = 880.; // 12个选项（每个60px高 + 68px间距）+ 顶部40px + 底部20px
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = 140. + scroll_offset; // 应用滚动

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(1.0, 1.0, 1.0, 0.95),
    );
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        3.0,
        Color::new(0.3, 0.5, 0.9, 0.8),
    );

    // 设置选项
    let option_y_start = panel_y + 40.;
    let option_spacing = 68.;

    // Lives
    draw_setting_option(
        panel_x,
        option_y_start,
        panel_width,
        "Starting Lives",
        &format!("< {} >", settings.starting_lives),
        selected == SettingOption::Lives,
        font,
    );

    // Ship Speed
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing,
        panel_width,
        "Ship Speed (Thrust & Rotation)",
        &format!("< {:.1}x >", settings.ship_speed_multiplier),
        selected == SettingOption::ShipSpeed,
        font,
    );

    // Asteroid Speed
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 2.,
        panel_width,
        "Asteroid Speed",
        &format!("< {:.1}x >", settings.asteroid_speed_multiplier),
        selected == SettingOption::AsteroidSpeed,
        font,
    );

    // Sound Volume
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 3.,
        panel_width,
        "Sound Volume",
        &format!("< {:.0}% >", settings.sound_volume * 100.0),
        selected == SettingOption::SoundVolume,
        font,
    );

    // Font Choice
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 4.,
        panel_width,
        "UI Font",
        &format!("< {} >", settings.font_choice.name()),
        selected == SettingOption::FontChoice,
        font,
    );

    // Weapon Switch
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 5.,
        panel_width,
        "Weapon Switch (Q key)",
        if settings.enable_weapon_switch {
            "ON"
        } else {
            "OFF"
        },
        selected == SettingOption::WeaponSwitch,
        font,
    );

    // Screen Shake
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 6.,
        panel_width,
        "Screen Shake Effects",
        if settings.enable_screen_shake {
            "ON"
        } else {
            "OFF"
        },
        selected == SettingOption::ScreenShake,
        font,
    );

    // Slow Motion
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 7.,
        panel_width,
        "Slow Motion (Killstreak)",
        if settings.enable_slow_motion {
            "ON"
        } else {
            "OFF"
        },
        selected == SettingOption::SlowMotion,
        font,
    );

    // Debug Panel
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 8.,
        panel_width,
        "Debug Panel (F3 toggle)",
        if settings.enable_debug_panel {
            "ON"
        } else {
            "OFF"
        },
        selected == SettingOption::DebugPanel,
        font,
    );

    // Flag Radius
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 9.,
        panel_width,
        "Flag Radius (Duel mode)",
        &format!("{:.0}px", settings.flag_radius),
        selected == SettingOption::FlagRadius,
        font,
    );

    // Reset Defaults (特殊样式)
    draw_reset_option(
        panel_x,
        option_y_start + option_spacing * 10.,
        panel_width,
        selected == SettingOption::ResetDefaults,
        font,
    );

    // Reset Achievements (特殊样式)
    draw_reset_achievements_option(
        panel_x,
        option_y_start + option_spacing * 11.,
        panel_width,
        selected == SettingOption::ResetAchievements,
        font,
    );

    // 底部提示（固定位置）
    let hint_y = screen_height() - 40.;
    let hint = "[Up/Down W/S] Select  |  [Left/Right A/D] Change  |  [Mouse Wheel] Scroll  |  [ESC] Back";
    let hint_color = Color::new(0.4, 0.5, 0.6, 1.0);

    if let Some(f) = font {
        let hint_dims = measure_text(hint, Some(f), 20, 1.0);
        draw_text_ex(
            hint,
            screen_width() / 2. - hint_dims.width / 2.,
            hint_y,
            TextParams {
                font: Some(f),
                font_size: 20,
                color: hint_color,
                ..Default::default()
            },
        );
    } else {
        let hint_width = measure_text(hint, None, 20, 1.0).width;
        draw_text(
            hint,
            screen_width() / 2. - hint_width / 2.,
            hint_y,
            20.,
            hint_color,
        );
    }
}

/// 绘制单个设置选项
fn draw_setting_option(
    panel_x: f32,
    y: f32,
    panel_width: f32,
    label: &str,
    value: &str,
    selected: bool,
    font: Option<&Font>,
) {
    let option_height = 60.;
    let padding = 30.;

    // 选中时的高亮背景
    if selected {
        draw_rectangle(
            panel_x + padding - 10.,
            y - 10.,
            panel_width - padding * 2. + 20.,
            option_height,
            Color::new(0.3, 0.6, 0.95, 0.15),
        );
        draw_rectangle_lines(
            panel_x + padding - 10.,
            y - 10.,
            panel_width - padding * 2. + 20.,
            option_height,
            2.5,
            Color::new(0.3, 0.6, 0.95, 0.6),
        );
    }

    // 标签
    let label_color = if selected {
        Color::new(0.1, 0.3, 0.7, 1.0)
    } else {
        Color::new(0.3, 0.4, 0.5, 1.0)
    };

    draw_text_ex(
        label,
        panel_x + padding,
        y + 18.,
        TextParams {
            font,
            font_size: 26,
            color: label_color,
            ..Default::default()
        },
    );

    // 值
    let value_color = if selected {
        Color::new(0.2, 0.5, 0.9, 1.0)
    } else {
        Color::new(0.5, 0.6, 0.7, 1.0)
    };

    let value_width = measure_text(value, font, 28, 1.0).width;
    draw_text_ex(
        value,
        panel_x + panel_width - padding - value_width,
        y + 18.,
        TextParams {
            font,
            font_size: 28,
            color: value_color,
            ..Default::default()
        },
    );
}

/// 绘制恢复默认选项（特殊样式）
fn draw_reset_option(panel_x: f32, y: f32, panel_width: f32, selected: bool, font: Option<&Font>) {
    let option_height = 60.;
    let padding = 30.;

    // 按钮背景颜色
    let bg_color = if selected {
        Color::new(0.9, 0.3, 0.2, 0.2)
    } else {
        Color::new(0.8, 0.8, 0.8, 0.1)
    };

    let border_color = if selected {
        Color::new(0.9, 0.3, 0.2, 0.8)
    } else {
        Color::new(0.6, 0.6, 0.6, 0.5)
    };

    draw_rectangle(
        panel_x + padding - 10.,
        y - 10.,
        panel_width - padding * 2. + 20.,
        option_height,
        bg_color,
    );
    draw_rectangle_lines(
        panel_x + padding - 10.,
        y - 10.,
        panel_width - padding * 2. + 20.,
        option_height,
        2.5,
        border_color,
    );

    // 居中文字
    let text = "Reset to Defaults";
    let text_width = measure_text(text, font, 26, 1.0).width;
    let text_color = if selected {
        Color::new(0.9, 0.3, 0.2, 1.0)
    } else {
        Color::new(0.5, 0.5, 0.5, 1.0)
    };

    draw_text_ex(
        text,
        panel_x + panel_width / 2. - text_width / 2.,
        y + 18.,
        TextParams {
            font,
            font_size: 26,
            color: text_color,
            ..Default::default()
        },
    );

    if selected {
        let hint = "Press [Left/Right A/D] or [Enter] to reset all";
        let hint_width = measure_text(hint, font, 17, 1.0).width;
        draw_text_ex(
            hint,
            panel_x + panel_width / 2. - hint_width / 2.,
            y + 42.,
            TextParams {
                font,
                font_size: 17,
                color: Color::new(0.7, 0.3, 0.2, 0.8),
                ..Default::default()
            },
        );
    }
}

/// 绘制重置成就选项（特殊样式）
fn draw_reset_achievements_option(
    panel_x: f32,
    y: f32,
    panel_width: f32,
    selected: bool,
    font: Option<&Font>,
) {
    let option_height = 60.;
    let padding = 30.;

    // 按钮背景颜色
    let bg_color = if selected {
        Color::new(0.9, 0.5, 0.2, 0.2)
    } else {
        Color::new(0.8, 0.8, 0.8, 0.1)
    };

    let border_color = if selected {
        Color::new(0.9, 0.5, 0.2, 0.8)
    } else {
        Color::new(0.6, 0.6, 0.6, 0.5)
    };

    draw_rectangle(
        panel_x + padding - 10.,
        y - 10.,
        panel_width - padding * 2. + 20.,
        option_height,
        bg_color,
    );
    draw_rectangle_lines(
        panel_x + padding - 10.,
        y - 10.,
        panel_width - padding * 2. + 20.,
        option_height,
        2.5,
        border_color,
    );

    // 居中文字
    let text = "Reset Achievements";
    let text_width = measure_text(text, font, 26, 1.0).width;
    let text_color = if selected {
        Color::new(0.9, 0.5, 0.2, 1.0)
    } else {
        Color::new(0.5, 0.5, 0.5, 1.0)
    };

    draw_text_ex(
        text,
        panel_x + panel_width / 2. - text_width / 2.,
        y + 18.,
        TextParams {
            font,
            font_size: 26,
            color: text_color,
            ..Default::default()
        },
    );

    if selected {
        let hint = "Press [Left/Right A/D] or [Enter] to reset achievements";
        let hint_width = measure_text(hint, font, 17, 1.0).width;
        draw_text_ex(
            hint,
            panel_x + panel_width / 2. - hint_width / 2.,
            y + 42.,
            TextParams {
                font,
                font_size: 17,
                color: Color::new(0.7, 0.5, 0.2, 0.8),
                ..Default::default()
            },
        );
    }
}

/// 绘制成就查看界面
pub fn draw_achievements_screen(manager: &AchievementManager, font: Option<&Font>, _time: f64, scroll_offset: f32) {
    // 渐变背景
    draw_gradient_background(
        Color::new(0.88, 0.90, 0.95, 1.0),
        Color::new(0.80, 0.84, 0.92, 1.0),
    );

    // 标题（固定位置，不受滚动影响）
    let title = "Achievements";
    let title_size = 48;
    let title_color = Color::new(0.15, 0.3, 0.6, 1.0);

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
                color: Color::new(0.4, 0.5, 0.6, 1.0),
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
            Color::new(0.4, 0.5, 0.6, 1.0),
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

        // 分类标题
        draw_category_header(category, panel_x, y_offset, panel_width, font);
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

    // 底部提示（固定位置）
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
                color: Color::new(0.4, 0.5, 0.6, 1.0),
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
            Color::new(0.4, 0.5, 0.6, 1.0),
        );
    }
}

/// 绘制分类标题
fn draw_category_header(
    category: AchievementCategory,
    x: f32,
    y: f32,
    width: f32,
    font: Option<&Font>,
) {
    let name = category.name();
    draw_rectangle(x, y, width, 40.0, Color::new(1.0, 1.0, 1.0, 0.5));
    draw_text_ex(
        name,
        x + 20.0,
        y + 28.0,
        TextParams {
            font,
            font_size: 26,
            color: Color::new(0.2, 0.3, 0.5, 1.0),
            ..Default::default()
        },
    );
}

/// 绘制单个成就卡片
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

    // 背景颜色 - 简化渲染
    let bg_color = if unlocked {
        Color::new(1.0, 1.0, 1.0, 0.95)
    } else {
        Color::new(0.7, 0.7, 0.7, 0.6)
    };

    // 直接绘制矩形，不使用阴影面板
    draw_rectangle(x, y, width, height, bg_color);

    // 边框（根据等级显示不同颜色）
    let border_color = if unlocked {
        achievement.tier.color()
    } else {
        Color::new(0.5, 0.5, 0.5, 0.5)
    };

    draw_rectangle_lines(x, y, width, height, 2.0, border_color);

    // 图标
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
                BLACK
            } else {
                Color::new(0.4, 0.4, 0.4, 1.0)
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
                Color::new(0.5, 0.5, 0.5, 0.7)
            },
            ..Default::default()
        },
    );

    // 名称
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
                Color::new(0.2, 0.3, 0.4, 1.0)
            } else {
                Color::new(0.5, 0.5, 0.5, 1.0)
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
            Color::new(0.4, 0.5, 0.6, 1.0),
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
                    color: Color::new(0.5, 0.6, 0.7, 1.0),
                    ..Default::default()
                },
            );

            // 进度条
            let bar_width = width - 20.0;
            let bar_x = x + 10.0;
            let bar_y = y + 102.0;
            draw_rectangle(bar_x, bar_y, bar_width, 8.0, Color::new(0.3, 0.3, 0.3, 0.3));
            let fill = (p.current as f32 / achievement.target as f32).min(1.0);
            draw_rectangle(
                bar_x,
                bar_y,
                bar_width * fill,
                8.0,
                Color::new(0.3, 0.6, 0.9, 0.8),
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
        "🎉 Achievement Unlocked!",
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

/// 绘制在线大厅界面
pub fn draw_online_lobby(
    nickname: &str,
    is_inputting: bool,
    network_client: &crate::network::NetworkClient,
    font: Option<&Font>,
) {
    let w = screen_width();
    let h = screen_height();
    
    // 背景
    draw_gradient_background(
        Color::from_rgba(26, 38, 64, 255),
        Color::from_rgba(13, 26, 51, 255),
    );
    
    // 标题
    draw_text_centered("Online Multiplayer", h * 0.2, 60, Color::from_rgba(100, 200, 255, 255), font);
    
    // 连接状态
    let status_text = match &network_client.state {
        crate::network::ConnectionState::Disconnected => "Status: Disconnected",
        crate::network::ConnectionState::Connecting => "Status: Connecting...",
        crate::network::ConnectionState::Connected => "Status: Connected",
        crate::network::ConnectionState::Error(e) => {
            &format!("Status: Error - {}", e)
        }
    };
    
    let status_color = if network_client.is_connected() { GREEN } else { YELLOW };
    draw_text_centered(status_text, h * 0.3, 32, status_color, font);
    
    if is_inputting {
        // 昵称输入
        draw_text_centered("Enter your nickname:", h * 0.4, 28, WHITE, font);
        
        // 昵称输入框
        let input_text = if nickname.is_empty() { "_" } else { nickname };
        let cursor = if (get_time() * 2.0) as i32 % 2 == 0 { "|" } else { "" };
        let display_text = format!("{}{}", input_text, cursor);
        
        draw_text_centered(&display_text, h * 0.5, 40, Color::from_rgba(100, 255, 100, 255), font);
        
        // 提示
        draw_text_centered("Press [Enter] to continue", h * 0.65, 24, LIGHTGRAY, font);
    } else if network_client.is_connected() {
        // 已连接，显示模式选择
        let welcome_text = format!("Welcome, {}!", nickname);
        draw_text_centered(&welcome_text, h * 0.4, 32, Color::from_rgba(100, 255, 100, 255), font);
        
        draw_text_centered("Select game mode:", h * 0.5, 28, WHITE, font);
        draw_text_centered("[1] Survival Mode", h * 0.58, 24, LIGHTGRAY, font);
        draw_text_centered("[2] Duel Mode", h * 0.64, 24, LIGHTGRAY, font);
    } else {
        // 正在连接
        draw_text_centered("Connecting to server...", h * 0.5, 32, YELLOW, font);
    }
    
    // 底部提示
    draw_text_centered("[ESC] Return to menu", h * 0.9, 20, DARKGRAY, font);
}

/// 绘制在线等待界面
pub fn draw_online_waiting(
    room_id: u32,
    network_client: &crate::network::NetworkClient,
    font: Option<&Font>,
) {
    let w = screen_width();
    let h = screen_height();
    
    // 背景
    draw_gradient_background(
        Color::from_rgba(38, 26, 51, 255),
        Color::from_rgba(26, 13, 38, 255),
    );
    
    // 标题
    let title = if room_id == 0 {
        "Searching for match..."
    } else {
        "Match found!"
    };
    let title_color = if room_id == 0 { YELLOW } else { GREEN };
    draw_text_centered(title, h * 0.25, 50, title_color, font);
    
    // 房间信息
    if room_id != 0 {
        let room_text = format!("Room ID: {}", room_id);
        draw_text_centered(&room_text, h * 0.35, 32, WHITE, font);
    }
    
    // 加载动画
    if room_id == 0 {
        let dots = ".".repeat(((get_time() * 2.0) as usize % 4) + 1);
        let loading_text = format!("Please wait{}", dots);
        draw_text_centered(&loading_text, h * 0.45, 28, LIGHTGRAY, font);
    }
    
    // 连接状态
    let latency_text = if network_client.latency_ms > 0.0 {
        format!("Latency: {:.0} ms", network_client.latency_ms)
    } else {
        "Latency: -- ms".to_string()
    };
    draw_text_centered(&latency_text, h * 0.6, 24, DARKGRAY, font);
    
    // 玩家列表（如果在房间中）
    if network_client.in_room() {
        draw_text_centered("Players in room:", h * 0.7, 24, WHITE, font);
        // TODO: 显示房间中的玩家列表
    }
    
    // 底部提示
    draw_text_centered("[ESC] Leave queue", h * 0.9, 20, DARKGRAY, font);
}
