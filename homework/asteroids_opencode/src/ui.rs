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
//! - 星空背景和阴影面板
//! - 文本居中辅助函数

use macroquad::prelude::*;
use macroquad::text::Font;

use crate::achievement::AchievementManager;
use crate::background::Starfield;
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
            WeaponType::Homing => "Homing",
            WeaponType::ChainIon => "Chain Ion",
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

pub fn draw_waiting_screen(message: &str, font: Option<&Font>, starfield: &Starfield, time: f32) {
    // 绘制星空背景
    starfield.draw(time);

    let panel_width = screen_width() * 0.6;
    let panel_height = 160.;
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = screen_height() / 2. - panel_height / 2.;
    // 深色半透明面板适配星空背景
    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.08, 0.1, 0.15, 0.9),
    );
    // 亮色边框增强可见性
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        2.0,
        Color::new(0.4, 0.6, 0.9, 0.7),
    );

    let font_size = 32.;
    draw_text_centered(
        message,
        screen_height() / 2. + font_size / 4.,
        font_size as u16,
        Color::new(0.9, 0.92, 0.95, 1.0), // 浅色文字适配深色背景
        font,
    );
}

pub fn draw_game_over_message(
    message: &str,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    // 绘制星空背景
    starfield.draw(time);

    let banner_width = screen_width() * 0.6;
    let banner_height = 130.;
    let banner_x = screen_width() / 2. - banner_width / 2.;
    let banner_y = screen_height() * 0.35; // 稍微上移到 35% 位置

    // 深色半透明面板
    draw_shadow_panel(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        Color::new(0.08, 0.1, 0.18, 0.92),
    );

    // 亮蓝色边框
    draw_rectangle_lines(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        3.0,
        Color::new(0.3, 0.6, 0.95, 0.85),
    );

    let font_size = 36.;
    // 使用亮色文字
    draw_text_centered(
        message,
        banner_y + banner_height / 2. + font_size / 4.,
        font_size as u16,
        Color::new(0.7, 0.85, 1.0, 1.0),
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

/// 绘制限时挑战模式的计时器 HUD
pub fn draw_time_attack_hud(time_left: f64, frenzy_active: bool, score: u32, font: Option<&Font>) {
    // 计时器显示在屏幕上方中央
    let minutes = (time_left / 60.0).floor() as i32;
    let seconds = (time_left % 60.0).floor() as i32;
    let millis = ((time_left * 100.0) % 100.0).floor() as i32;

    let timer_text = format!("{:02}:{:02}.{:02}", minutes, seconds, millis);

    // 计时器颜色：正常为白色，狂暴时段为红色闪烁
    let timer_color = if frenzy_active {
        let flash = (time_left * 4.0).sin().abs() as f32;
        Color::new(1.0, 0.3 + flash * 0.3, 0.2, 1.0)
    } else if time_left <= 30.0 {
        YELLOW
    } else {
        WHITE
    };

    let timer_size = if frenzy_active { 48 } else { 40 };
    let timer_width = measure_text(&timer_text, font, timer_size, 1.0).width;

    // 背景框
    let bg_color = if frenzy_active {
        Color::new(0.5, 0.1, 0.1, 0.7)
    } else {
        Color::new(0.1, 0.1, 0.2, 0.7)
    };
    draw_rectangle(
        screen_width() / 2. - timer_width / 2. - 20.,
        10.,
        timer_width + 40.,
        timer_size as f32 + 16.,
        bg_color,
    );

    // 计时器文字
    draw_text_ex(
        &timer_text,
        screen_width() / 2. - timer_width / 2.,
        timer_size as f32 + 18.,
        TextParams {
            font,
            font_size: timer_size,
            color: timer_color,
            ..Default::default()
        },
    );

    // 狂暴时段提示
    if frenzy_active {
        let frenzy_text = "⚡ FRENZY MODE ⚡";
        let frenzy_width = measure_text(frenzy_text, font, 24, 1.0).width;
        let flash = (time_left * 6.0).sin().abs() as f32;
        draw_text_ex(
            frenzy_text,
            screen_width() / 2. - frenzy_width / 2.,
            timer_size as f32 + 50.,
            TextParams {
                font,
                font_size: 24,
                color: Color::new(1.0, 0.5 + flash * 0.5, 0.0, 1.0),
                ..Default::default()
            },
        );
    }

    // 当前分数
    let score_text = format!("Score: {}", score);
    let score_width = measure_text(&score_text, font, 28, 1.0).width;
    draw_text_ex(
        &score_text,
        screen_width() / 2. - score_width / 2.,
        timer_size as f32 + 80.,
        TextParams {
            font,
            font_size: 28,
            color: Color::new(0.9, 0.9, 0.5, 1.0),
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

    // 深色半透明面板
    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.06, 0.08, 0.14, 0.9),
    );

    // 渐变蓝色边框
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        4.0,
        Color::new(0.3, 0.55, 0.9, 0.85),
    );

    // 标题使用亮色
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
            color: Color::new(0.0, 0.0, 0.0, 0.4),
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
            color: Color::new(0.6, 0.8, 1.0, 1.0), // 亮蓝色
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
                color: Color::new(0.0, 0.0, 0.0, 0.4),
                ..Default::default()
            },
        );

        // 使用更亮的玩家颜色（增强对比度）
        let bright_color = Color::new(
            (player.2.r * 1.2).min(1.0),
            (player.2.g * 1.2).min(1.0),
            (player.2.b * 1.2).min(1.0),
            1.0,
        );

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

#[allow(clippy::too_many_arguments)]
pub fn draw_mode_selection(
    selection: GameMode,
    settings: &GameSettings,
    achievements: &AchievementManager,
    time_attack_duration: crate::TimeAttackDuration,
    online_enabled: bool,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    // 绘制星空背景
    starfield.draw(time);

    let title = "Choose Your Adventure";
    let subtitle = "Press [M] later to revisit this screen";
    let title_size = 48;
    let subtitle_size = 24;

    // 使用自定义字体绘制标题（亮色适配深色背景）
    if let Some(f) = font {
        let title_dims = measure_text(title, Some(f), title_size, 1.0);
        draw_text_ex(
            title,
            screen_width() / 2. - title_dims.width / 2.,
            70.,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: Color::new(0.9, 0.92, 0.98, 1.0),
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
                color: Color::new(0.6, 0.65, 0.75, 1.0),
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
            Color::new(0.9, 0.92, 0.98, 1.0),
        );
        let subtitle_width = measure_text(subtitle, None, subtitle_size, 1.0).width;
        draw_text(
            subtitle,
            screen_width() / 2. - subtitle_width / 2.,
            115.,
            subtitle_size as f32,
            Color::new(0.6, 0.65, 0.75, 1.0),
        );
    }

    // 7个卡片纵向排列（含 Roguelike）
    let card_width = screen_width() * 0.65;
    let card_height = 100.; // 减小卡片高度以容纳7个
    let spacing = 10.;
    let total_height = card_height * 7. + spacing * 6.; // 7个卡片
    let start_y = (screen_height() - total_height) / 2. + 20.;
    let card_x = screen_width() / 2. - card_width / 2.;

    // Survival 卡片 - 显示玩家数量
    let survival_desc = match settings.player_count {
        crate::PlayerCount::One => "Solo challenge - survive as long as you can alone!",
        crate::PlayerCount::Two => "Team up to clear every asteroid and push your score higher.",
    };
    let survival_detail = match settings.player_count {
        crate::PlayerCount::One => format!(
            "Single player mode. [←/→] to switch: {}",
            settings.player_count.name()
        ),
        crate::PlayerCount::Two => format!(
            "Co-op mode for two players. [←/→] to switch: {}",
            settings.player_count.name()
        ),
    };
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y,
        width: card_width,
        height: card_height,
        title: "Survival",
        desc: survival_desc,
        detail: &survival_detail,
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

    // TimeAttack 卡片
    let time_attack_detail = format!(
        "Score as much as possible! [←/→] to switch: {}",
        time_attack_duration.name()
    );
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 2.,
        width: card_width,
        height: card_height,
        title: "Time Attack",
        desc: "Race against the clock - score as many points as possible!",
        detail: &time_attack_detail,
        active: matches!(selection, GameMode::TimeAttack),
        accent: Color::new(1.0, 0.5, 0.0, 1.0), // 橙色
        footer: "[Enter] Start",
        font,
    });

    // Roguelike 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 3.,
        width: card_width,
        height: card_height,
        title: "Roguelike",
        desc: "Run-based adventure with random builds and permanent upgrades!",
        detail: "3 zones, unique bosses, relics, and card synergies each run.",
        active: matches!(selection, GameMode::Roguelike),
        accent: Color::new(0.8, 0.2, 0.6, 1.0), // 品红色
        footer: "[Enter] Start Run",
        font,
    });

    // Online 卡片 (disabled on WASM)
    let (online_title, online_desc, online_detail, online_footer, online_accent, online_active) =
        if online_enabled {
            (
                "Online Multiplayer",
                "Challenge players online in real-time battles.",
                "Connect to server and compete with other pilots worldwide.",
                "[Enter] Connect",
                Color::new(0.6, 0.3, 0.9, 1.0), // 紫色
                matches!(selection, GameMode::Online),
            )
        } else {
            (
                "Online (Coming Soon)",
                "Multiplayer mode is not available on Web build.",
                "Try the native desktop version for early online features.",
                "Unavailable on Web",
                Color::new(0.35, 0.35, 0.42, 1.0), // 灰色
                false,
            )
        };
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 4.,
        width: card_width,
        height: card_height,
        title: online_title,
        desc: online_desc,
        detail: online_detail,
        active: online_active,
        accent: online_accent,
        footer: online_footer,
        font,
    });

    // Achievements 卡片
    let (unlocked, total) = achievements.get_stats();
    let achievements_summary = format!("Unlocked: {} / {} achievements", unlocked, total);
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: start_y + (card_height + spacing) * 5.,
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
        y: start_y + (card_height + spacing) * 6.,
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

    // 底部提示使用自定义字体（亮色）
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
                color: Color::new(0.65, 0.7, 0.8, 1.0),
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
            Color::new(0.65, 0.7, 0.8, 1.0),
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
    // 深色半透明面板适配星空背景
    let base = if params.active {
        Color::new(0.1, 0.12, 0.18, 0.92)
    } else {
        Color::new(0.08, 0.1, 0.14, 0.85)
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
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.2),
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
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.35),
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
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.2),
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

    // 绘制描述（支持自动换行）- 亮色文字
    draw_wrapped_text(
        params.desc,
        params.x + 24.,
        params.y + 85.,
        params.width - 48.,
        22, // 增大从 18 到 22
        Color::new(0.75, 0.78, 0.85, 1.0),
        params.font,
    );

    // Detail 也支持换行 - 稍暗的亮色
    draw_wrapped_text(
        params.detail,
        params.x + 24.,
        params.y + params.height - 42.,
        params.width - 48.,
        19, // 增大从 16 到 19
        Color::new(0.55, 0.58, 0.65, 1.0),
        params.font,
    );
}

/// 文本换行辅助函数（支持中文和英文）
fn wrap_text(text: &str, max_width: f32, font_size: u16, font: Option<&Font>) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    // 检查是否包含空格（英文文本）
    let has_spaces = text.contains(' ');

    if has_spaces {
        // 英文文本：按空格分词
        let words: Vec<&str> = text.split(' ').collect();
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
    } else {
        // 中文文本：按字符逐个测量
        let mut current_line = String::new();

        for ch in text.chars() {
            let test_line = format!("{}{}", current_line, ch);
            let width = measure_text(&test_line, font, font_size, 1.0).width;

            if width <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = ch.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
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
    let lines = wrap_text(text, max_width, font_size, font);

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

/// 插值缓冲区调试信息
#[derive(Debug, Clone)]
pub struct InterpDebugStats {
    pub player_buffers: usize,
    pub asteroid_buffers: usize,
    pub bullet_buffers: usize,
    pub avg_player_snapshots: f32,
    pub avg_bullet_snapshots: f32,
    pub render_delay_ms: f64,
}

/// 网络诊断信息
#[derive(Debug, Clone)]
pub struct NetworkDebugStats {
    pub rtt_ms: f32,
    pub pending_inputs: usize,
    pub interp: Option<InterpDebugStats>,
}

/// 性能监控统计数据
pub struct DebugStats {
    pub fps: f32,
    pub entity_count: usize,
    pub quadtree_depth: usize,
    pub particle_count: usize,
    pub network: Option<NetworkDebugStats>,
}

/// 绘制性能监控面板（左上角）
pub fn draw_debug_panel(stats: &DebugStats, font: Option<&Font>) {
    let panel_x = 12.0;
    let line_height = 22.0;
    let base_height = 115.0;

    // 根据网络调试信息动态扩展面板高度
    let extra_lines = if let Some(net) = &stats.network {
        1 + net.interp.as_ref().map(|_| 2).unwrap_or(0)
    } else {
        0
    };
    let panel_height = base_height + line_height * extra_lines as f32;
    let panel_y = screen_height() - panel_height - 15.0;
    let panel_width = 300.0;

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

    // 网络调试信息
    if let Some(net) = &stats.network {
        text_y += line_height;
        let rtt_color = if net.rtt_ms <= 120.0 {
            Color::new(0.2, 1.0, 0.2, 1.0) // 绿色：延迟良好
        } else if net.rtt_ms <= 200.0 {
            Color::new(1.0, 0.9, 0.2, 1.0) // 黄色：延迟一般
        } else {
            Color::new(1.0, 0.4, 0.2, 1.0) // 红色：延迟过高
        };
        draw_text_ex(
            &format!("RTT: {:.1}ms | Pending: {}", net.rtt_ms, net.pending_inputs),
            text_x,
            text_y,
            TextParams {
                font,
                font_size,
                color: rtt_color,
                ..Default::default()
            },
        );

        if let Some(interp) = &net.interp {
            text_y += line_height;
            draw_text_ex(
                &format!(
                    "Interp buf P:{} A:{} B:{}",
                    interp.player_buffers, interp.asteroid_buffers, interp.bullet_buffers
                ),
                text_x,
                text_y,
                TextParams {
                    font,
                    font_size,
                    color: Color::new(0.6, 0.9, 1.0, 1.0),
                    ..Default::default()
                },
            );

            text_y += line_height;
            draw_text_ex(
                &format!(
                    "Snap P~{:.1} B~{:.1} | Delay:{:.0}ms",
                    interp.avg_player_snapshots, interp.avg_bullet_snapshots, interp.render_delay_ms
                ),
                text_x,
                text_y,
                TextParams {
                    font,
                    font_size,
                    color: Color::new(0.8, 0.8, 0.8, 1.0),
                    ..Default::default()
                },
            );
        }
    }

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
pub fn draw_settings_screen(
    settings: &GameSettings,
    selected: SettingOption,
    font: Option<&Font>,
    scroll_offset: f32,
    starfield: &Starfield,
    time: f32,
) {
    // 绘制星空背景
    starfield.draw(time);

    // 标题（固定位置）- 亮色适配深色背景
    let title = "Game Settings";
    let title_size = 42;
    let title_color = Color::new(0.7, 0.85, 1.0, 1.0);

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

    // 设置面板（应用滚动偏移）- 深色半透明
    let panel_width = screen_width() * 0.65;
    let panel_height = 880.; // 12个选项（每个60px高 + 68px间距）+ 顶部40px + 底部20px
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = 140. + scroll_offset; // 应用滚动

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.06, 0.08, 0.14, 0.9),
    );
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        3.0,
        Color::new(0.3, 0.55, 0.9, 0.8),
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

    // 底部提示（固定位置）- 亮色适配深色背景
    let hint_y = screen_height() - 40.;
    let hint =
        "[Up/Down W/S] Select  |  [Left/Right A/D] Change  |  [Mouse Wheel] Scroll  |  [ESC] Back";
    let hint_color = Color::new(0.6, 0.65, 0.75, 1.0);

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

/// 绘制单个设置选项 - 适配深色背景
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
            Color::new(0.3, 0.5, 0.8, 0.2),
        );
        draw_rectangle_lines(
            panel_x + padding - 10.,
            y - 10.,
            panel_width - padding * 2. + 20.,
            option_height,
            2.5,
            Color::new(0.4, 0.65, 0.95, 0.7),
        );
    }

    // 标签 - 亮色适配深色背景
    let label_color = if selected {
        Color::new(0.7, 0.85, 1.0, 1.0)
    } else {
        Color::new(0.6, 0.65, 0.75, 1.0)
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
        Color::new(0.5, 0.8, 1.0, 1.0)
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

/// 绘制恢复默认选项（特殊样式）- 适配深色背景
fn draw_reset_option(panel_x: f32, y: f32, panel_width: f32, selected: bool, font: Option<&Font>) {
    let option_height = 60.;
    let padding = 30.;

    // 按钮背景颜色
    let bg_color = if selected {
        Color::new(0.9, 0.3, 0.2, 0.25)
    } else {
        Color::new(0.3, 0.3, 0.3, 0.15)
    };

    let border_color = if selected {
        Color::new(1.0, 0.4, 0.3, 0.9)
    } else {
        Color::new(0.5, 0.5, 0.5, 0.5)
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
        Color::new(1.0, 0.5, 0.4, 1.0)
    } else {
        Color::new(0.6, 0.6, 0.6, 1.0)
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
                color: Color::new(1.0, 0.5, 0.4, 0.85),
                ..Default::default()
            },
        );
    }
}

/// 绘制重置成就选项（特殊样式）- 适配深色背景
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
        Color::new(0.9, 0.5, 0.2, 0.25)
    } else {
        Color::new(0.3, 0.3, 0.3, 0.15)
    };

    let border_color = if selected {
        Color::new(1.0, 0.6, 0.3, 0.9)
    } else {
        Color::new(0.5, 0.5, 0.5, 0.5)
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
        Color::new(1.0, 0.65, 0.4, 1.0)
    } else {
        Color::new(0.6, 0.6, 0.6, 1.0)
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
                color: Color::new(1.0, 0.65, 0.4, 0.85),
                ..Default::default()
            },
        );
    }
}

// Achievement UI functions moved to ui_achievements.rs
// Re-exported via crate::ui_achievements

/// 绘制在线大厅界面
pub fn draw_online_lobby(
    nickname: &str,
    is_inputting: bool,
    network_client: &crate::network::NetworkClient,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    let _w = screen_width();
    let h = screen_height();

    // 绘制星空背景
    starfield.draw(time);

    // 标题
    draw_text_centered(
        "Online Multiplayer",
        h * 0.2,
        60,
        Color::from_rgba(100, 200, 255, 255),
        font,
    );

    // 连接状态
    let status_text = match &network_client.state {
        crate::network::ConnectionState::Disconnected => "Status: Disconnected",
        crate::network::ConnectionState::Connecting => "Status: Connecting...",
        crate::network::ConnectionState::Connected => "Status: Connected",
        crate::network::ConnectionState::Error(e) => &format!("Status: Error - {}", e),
    };

    let status_color = if network_client.is_connected() {
        GREEN
    } else {
        YELLOW
    };
    draw_text_centered(status_text, h * 0.3, 32, status_color, font);

    if is_inputting {
        // 昵称输入
        draw_text_centered("Enter your nickname:", h * 0.4, 28, WHITE, font);

        // 昵称输入框
        let input_text = if nickname.is_empty() { "_" } else { nickname };
        let cursor = if (get_time() * 2.0) as i32 % 2 == 0 {
            "|"
        } else {
            ""
        };
        let display_text = format!("{}{}", input_text, cursor);

        draw_text_centered(
            &display_text,
            h * 0.5,
            40,
            Color::from_rgba(100, 255, 100, 255),
            font,
        );

        // 提示
        draw_text_centered("Press [Enter] to continue", h * 0.65, 24, LIGHTGRAY, font);
    } else if network_client.is_connected() {
        // 已连接，显示模式选择
        let welcome_text = format!("Welcome, {}!", nickname);
        draw_text_centered(
            &welcome_text,
            h * 0.4,
            32,
            Color::from_rgba(100, 255, 100, 255),
            font,
        );

        draw_text_centered("Select game mode:", h * 0.5, 28, WHITE, font);
        draw_text_centered("[1] Survival Mode", h * 0.58, 24, LIGHTGRAY, font);
        draw_text_centered("[2] Duel Mode", h * 0.64, 24, LIGHTGRAY, font);
    } else {
        // 正在连接
        draw_text_centered("Connecting to server...", h * 0.5, 32, YELLOW, font);
    }

    // 底部提示
    draw_text_centered(
        "[ESC] Return to menu",
        h * 0.9,
        20,
        Color::new(0.5, 0.55, 0.65, 1.0),
        font,
    );
}

/// 绘制在线等待界面
pub fn draw_online_waiting(
    room_id: u32,
    network_client: &crate::network::NetworkClient,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    let _w = screen_width();
    let h = screen_height();

    // 绘制星空背景
    starfield.draw(time);

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
    draw_text_centered(
        &latency_text,
        h * 0.6,
        24,
        Color::new(0.5, 0.55, 0.65, 1.0),
        font,
    );

    // 玩家列表（如果在房间中）
    if network_client.in_room() {
        draw_text_centered("Players in room:", h * 0.7, 24, WHITE, font);
        // TODO: 显示房间中的玩家列表
    }

    // 底部提示
    draw_text_centered(
        "[ESC] Leave queue",
        h * 0.9,
        20,
        Color::new(0.5, 0.55, 0.65, 1.0),
        font,
    );
}

// ============================================================================
// 玩家状态效果图标栏
// ============================================================================

/// 激活的buff/道具效果信息
#[allow(dead_code)] // name 字段保留用于未来的 tooltip 功能
pub struct ActiveBuff {
    pub name: &'static str,
    pub icon_char: String,
    pub color: Color,
    pub remaining: f64,
    pub max_duration: f64,
}

/// 绘制玩家状态效果图标栏（显示当前激活的道具/buff）
pub fn draw_player_buffs(players: &[Player], time: f64, font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        if !player.alive {
            continue;
        }

        let buffs = collect_active_buffs(player, time);
        if buffs.is_empty() {
            continue;
        }

        // 根据玩家索引决定显示位置
        let base_x = if idx == 0 {
            20.0
        } else {
            screen_width() - 20.0 - (buffs.len() as f32 * 50.0)
        };
        let base_y = 80.0 + idx as f32 * 40.0;

        // 绘制每个buff图标
        for (i, buff) in buffs.iter().enumerate() {
            let x = base_x + i as f32 * 50.0;
            draw_buff_icon(x, base_y, buff, font);
        }
    }
}

/// 收集玩家当前激活的所有buff
fn collect_active_buffs(player: &Player, time: f64) -> Vec<ActiveBuff> {
    let mut buffs = Vec::new();

    // 护盾
    if player.shield_active(time) {
        buffs.push(ActiveBuff {
            name: "Shield",
            icon_char: "S".to_string(),
            color: Color::new(0.2, 0.6, 1.0, 1.0),
            remaining: player.shield_remaining(time),
            max_duration: crate::player::SHIELD_DURATION,
        });
    }

    // 临时护盾（次数型）
    if player.temp_shield_hits > 0 {
        buffs.push(ActiveBuff {
            name: "Temp Shield",
            icon_char: format!("{}", player.temp_shield_hits),
            color: Color::new(0.3, 0.8, 1.0, 1.0),
            remaining: player.temp_shield_hits as f64,
            max_duration: 3.0, // 最大3次
        });
    }

    // 快速射击
    if player.rapid_fire_active(time) {
        buffs.push(ActiveBuff {
            name: "Rapid Fire",
            icon_char: "R".to_string(),
            color: Color::new(1.0, 0.9, 0.2, 1.0),
            remaining: player.rapid_fire_until - time,
            max_duration: 6.0,
        });
    }

    // 穿透弹
    if player.piercing_active(time) {
        buffs.push(ActiveBuff {
            name: "Piercing",
            icon_char: "P".to_string(),
            color: Color::new(0.8, 0.3, 1.0, 1.0),
            remaining: player.piercing_until - time,
            max_duration: 8.0,
        });
    }

    // 幽灵模式
    if player.ghost_mode_active(time) {
        buffs.push(ActiveBuff {
            name: "Ghost",
            icon_char: "G".to_string(),
            color: Color::new(0.7, 0.7, 0.9, 0.8),
            remaining: player.ghost_mode_until - time,
            max_duration: 5.0,
        });
    }

    // 超速模式
    if player.overdrive_active(time) {
        buffs.push(ActiveBuff {
            name: "Overdrive",
            icon_char: "O".to_string(),
            color: Color::new(1.0, 0.3, 0.3, 1.0),
            remaining: player.overdrive_until - time,
            max_duration: 7.0,
        });
    }

    // 传送充能
    if player.teleport_charge_active(time) {
        buffs.push(ActiveBuff {
            name: "Teleport",
            icon_char: "T".to_string(),
            color: Color::new(0.6, 0.2, 0.9, 1.0),
            remaining: player.teleport_charge_until - time,
            max_duration: 10.0,
        });
    }

    buffs
}

/// 绘制单个buff图标
fn draw_buff_icon(x: f32, y: f32, buff: &ActiveBuff, font: Option<&Font>) {
    let size = 40.0;
    let progress = (buff.remaining / buff.max_duration).clamp(0.0, 1.0) as f32;

    // 背景圆
    draw_circle(x + size / 2.0, y + size / 2.0, size / 2.0, Color::new(0.0, 0.0, 0.0, 0.6));

    // 进度环（顺时针从顶部开始）
    let segments = 32;
    let filled_segments = (progress * segments as f32) as i32;
    for i in 0..filled_segments {
        let angle1 = -std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = -std::f32::consts::FRAC_PI_2 + ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
        let cx = x + size / 2.0;
        let cy = y + size / 2.0;
        let r = size / 2.0 - 2.0;

        draw_triangle(
            Vec2::new(cx, cy),
            Vec2::new(cx + angle1.cos() * r, cy + angle1.sin() * r),
            Vec2::new(cx + angle2.cos() * r, cy + angle2.sin() * r),
            Color::new(buff.color.r, buff.color.g, buff.color.b, 0.4),
        );
    }

    // 边框
    draw_circle_lines(
        x + size / 2.0,
        y + size / 2.0,
        size / 2.0,
        2.0,
        buff.color,
    );

    // 图标字符
    let text_width = measure_text(&buff.icon_char, font, 20, 1.0).width;
    draw_text_ex(
        &buff.icon_char,
        x + size / 2.0 - text_width / 2.0,
        y + size / 2.0 + 6.0,
        TextParams {
            font,
            font_size: 20,
            color: buff.color,
            ..Default::default()
        },
    );

    // 剩余时间（小字）
    if buff.remaining > 0.0 && buff.remaining < 100.0 {
        let time_text = format!("{:.1}", buff.remaining);
        let time_width = measure_text(&time_text, font, 12, 1.0).width;
        draw_text_ex(
            &time_text,
            x + size / 2.0 - time_width / 2.0,
            y + size + 12.0,
            TextParams {
                font,
                font_size: 12,
                color: Color::new(0.8, 0.8, 0.8, 0.9),
                ..Default::default()
            },
        );
    }
}

// ============================================================================
// 连击系统增强UI
// ============================================================================

/// 绘制连击计数器和分数倍率（所有模式通用）
pub fn draw_killstreak_counter(players: &[Player], time: f64, font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        if !player.alive || player.killstreak == 0 {
            continue;
        }

        // 检查连击是否即将过期（闪烁警告）
        let time_since_kill = time - player.get_last_kill_time();
        let is_expiring = time_since_kill > crate::constants::killstreak::RESET_TIME * 0.7;

        // 根据玩家索引决定显示位置
        let x = if idx == 0 {
            screen_width() * 0.15
        } else {
            screen_width() * 0.85
        };
        let y = screen_height() - 120.0;

        // 连击数
        let streak_text = format!("{}x", player.killstreak);
        let multiplier_text = format!("{:.1}x Score", player.score_multiplier());

        // 闪烁效果（即将过期时）
        let alpha = if is_expiring {
            0.5 + 0.5 * ((time * 8.0).sin().abs() as f32)
        } else {
            1.0
        };

        // 连击颜色（根据等级变化）
        let streak_color = match player.killstreak_visual_level() {
            0 => Color::new(0.8, 0.8, 0.8, alpha),
            1 => Color::new(1.0, 1.0, 0.5, alpha),
            2 => Color::new(1.0, 0.8, 0.2, alpha),
            3 => Color::new(1.0, 0.5, 0.1, alpha),
            _ => Color::new(1.0, 0.2, 0.2, alpha),
        };

        // 背景框
        let bg_width = 100.0;
        let bg_height = 60.0;
        draw_rectangle(
            x - bg_width / 2.0,
            y - bg_height / 2.0,
            bg_width,
            bg_height,
            Color::new(0.0, 0.0, 0.0, 0.5 * alpha),
        );
        draw_rectangle_lines(
            x - bg_width / 2.0,
            y - bg_height / 2.0,
            bg_width,
            bg_height,
            2.0,
            Color::new(streak_color.r, streak_color.g, streak_color.b, 0.7 * alpha),
        );

        // 连击数（大字）
        let streak_width = measure_text(&streak_text, font, 32, 1.0).width;
        draw_text_ex(
            &streak_text,
            x - streak_width / 2.0,
            y - 5.0,
            TextParams {
                font,
                font_size: 32,
                color: streak_color,
                ..Default::default()
            },
        );

        // 分数倍率（小字）
        let mult_width = measure_text(&multiplier_text, font, 16, 1.0).width;
        draw_text_ex(
            &multiplier_text,
            x - mult_width / 2.0,
            y + 18.0,
            TextParams {
                font,
                font_size: 16,
                color: Color::new(0.9, 0.9, 0.5, alpha),
                ..Default::default()
            },
        );

        // 过期进度条
        let progress = 1.0 - (time_since_kill / crate::constants::killstreak::RESET_TIME) as f32;
        let bar_width = bg_width - 10.0;
        let bar_height = 4.0;
        let bar_x = x - bar_width / 2.0;
        let bar_y = y + bg_height / 2.0 - 8.0;

        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.3, 0.3, 0.3, 0.5));
        draw_rectangle(
            bar_x,
            bar_y,
            bar_width * progress.max(0.0),
            bar_height,
            streak_color,
        );
    }
}

// ============================================================================
// Roguelike：奖励选择 UI
// ============================================================================

use crate::input::Input;
use crate::roguelike;

/// 绘制奖励卡片
fn draw_reward_card(
    rect: Rect,
    option: &roguelike::RewardOption,
    selected: bool,
    hotkey: &str,
    font: Option<&Font>,
) {
    let (kind, name, desc, rarity) = roguelike::reward_display_info(option);

    let base = if selected {
        Color::new(0.12, 0.14, 0.2, 0.96)
    } else {
        Color::new(0.08, 0.1, 0.14, 0.9)
    };

    draw_shadow_panel(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected { 4.0 } else { 2.0 },
        if selected {
            rarity
        } else {
            Color::new(rarity.r, rarity.g, rarity.b, 0.65)
        },
    );

    // 稀有度条
    draw_rectangle(rect.x, rect.y, rect.w, 6.0, rarity);

    // 快捷键标签
    let tag = format!("[{}]", hotkey);
    let tag_w = measure_text(&tag, font, 18, 1.0).width + 16.0;
    draw_rectangle(
        rect.x + rect.w - tag_w - 14.0,
        rect.y + 14.0,
        tag_w,
        26.0,
        Color::new(rarity.r, rarity.g, rarity.b, 0.18),
    );
    draw_text_ex(
        &tag,
        rect.x + rect.w - tag_w - 6.0,
        rect.y + 33.0,
        TextParams {
            font,
            font_size: 18,
            color: rarity,
            ..Default::default()
        },
    );

    // 类型标签
    draw_text_ex(
        kind,
        rect.x + 18.0,
        rect.y + 34.0,
        TextParams {
            font,
            font_size: 18,
            color: Color::new(0.7, 0.75, 0.85, 1.0),
            ..Default::default()
        },
    );

    // 名称
    draw_text_ex(
        &name,
        rect.x + 18.0,
        rect.y + 66.0,
        TextParams {
            font,
            font_size: 28,
            color: rarity,
            ..Default::default()
        },
    );

    // 描述
    draw_wrapped_text(
        &desc,
        rect.x + 18.0,
        rect.y + 96.0,
        rect.w - 36.0,
        20,
        Color::new(0.8, 0.84, 0.92, 1.0),
        font,
    );
}

/// 绘制奖励选择界面
/// 返回选中的奖励索引（如果玩家做出选择）
pub fn draw_reward_selection(
    reward_state: &mut roguelike::RewardPhaseState,
    input: &Input,
    font: Option<&Font>,
) -> Option<usize> {
    // 绘制半透明背景覆盖层（不完全清除，保留 HUD 可见性）
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.04, 0.05, 0.07, 0.95),
    );

    reward_state.timer += get_frame_time();

    // 标题
    let title = "选择奖励";
    let title_w = measure_text(title, font, 44, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2.0 - title_w / 2.0,
        92.0,
        TextParams {
            font,
            font_size: 44,
            color: Color::new(0.85, 0.9, 0.98, 1.0),
            ..Default::default()
        },
    );

    // 提示（根据选项数量动态调整）
    let hint = if reward_state.options.len() >= 4 {
        "按 1/2/3/4 或点击卡片选择"
    } else {
        "按 1/2/3 或点击卡片选择"
    };
    let hint_w = measure_text(hint, font, 22, 1.0).width;
    draw_text_ex(
        hint,
        screen_width() / 2.0 - hint_w / 2.0,
        122.0,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.6, 0.65, 0.78, 1.0),
            ..Default::default()
        },
    );

    // 获取奖励选项（支持 3-4 个，适配抽卡手套遗物）
    let count = reward_state.options.len().clamp(3, 4);
    if reward_state.options.is_empty() {
        draw_text_ex(
            "奖励选项未就绪",
            24.0,
            screen_height() - 24.0,
            TextParams {
                font,
                font_size: 20,
                color: RED,
                ..Default::default()
            },
        );
        return None;
    }

    // 计算卡片布局（动态适配 3-4 个选项，确保小屏不溢出）
    let gap = 16.0;
    let layout_w = (screen_width() * 0.86).min(1120.0);
    let card_w = ((layout_w - gap * (count as f32 - 1.0)) / count as f32).clamp(160.0, 300.0);
    let card_h = 220.0;
    let total_w = card_w * count as f32 + gap * (count as f32 - 1.0);
    let start_x = screen_width() / 2.0 - total_w / 2.0;
    let y = screen_height() / 2.0 - card_h / 2.0 + 30.0;

    let mouse = vec2(mouse_position().0, mouse_position().1);
    let mut hover: Option<usize> = None;
    let mut rects: Vec<Rect> = Vec::with_capacity(count);

    for i in 0..count {
        let x = start_x + i as f32 * (card_w + gap);
        let rect = Rect::new(x, y, card_w, card_h);
        rects.push(rect);
        if rect.contains(mouse) {
            hover = Some(i);
        }
    }

    // 更新选中状态
    if let Some(h) = hover {
        reward_state.selected = Some(h);
    }

    // 绘制卡片（选中时有呼吸动画）
    let pulse = 1.0 + 0.03 * ((get_time() as f32) * 7.0).sin().abs();
    for i in 0..count {
        let selected = reward_state.selected == Some(i);
        let base = rects[i];
        let scale = if selected { pulse } else { 1.0 };
        let cx = base.x + base.w / 2.0;
        let cy = base.y + base.h / 2.0;
        let rect = Rect::new(
            cx - base.w * scale / 2.0,
            cy - base.h * scale / 2.0,
            base.w * scale,
            base.h * scale,
        );
        draw_reward_card(rect, &reward_state.options[i], selected, &format!("{}", i + 1), font);
    }

    // 处理输入（支持 1-4 键）
    let key_choice = if input.is_key_pressed(KeyCode::Key1) || input.is_key_pressed(KeyCode::Kp1) {
        Some(0)
    } else if input.is_key_pressed(KeyCode::Key2) || input.is_key_pressed(KeyCode::Kp2) {
        Some(1)
    } else if input.is_key_pressed(KeyCode::Key3) || input.is_key_pressed(KeyCode::Kp3) {
        Some(2)
    } else if count >= 4 && (input.is_key_pressed(KeyCode::Key4) || input.is_key_pressed(KeyCode::Kp4)) {
        Some(3)
    } else {
        None
    };

    if let Some(i) = key_choice {
        if i < count {
            reward_state.selected = Some(i);
            return Some(i);
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(i) = hover {
            reward_state.selected = Some(i);
            return Some(i);
        }
    }

    None
}

// ============================================================================
// Roguelike：商店 UI
// ============================================================================

/// 商店 UI 操作结果
pub enum ShopUiAction {
    /// 无操作
    None,
    /// 确认购买指定索引的商品
    BuyConfirmed(usize),
    /// 请求刷新商店
    RefreshRequested,
    /// 退出商店
    ExitShop,
}

/// 绘制商店界面
pub fn draw_shop_ui(
    shop_state: &mut roguelike::ShopPhaseState,
    gold: u32,
    refresh_cost: u32,
    input: &Input,
    font: Option<&Font>,
) -> ShopUiAction {
    // 绘制半透明背景覆盖层（不完全清除，保留 HUD 可见性）
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.04, 0.05, 0.07, 0.95),
    );

    // 标题
    let title = "商店";
    let title_w = measure_text(title, font, 44, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2.0 - title_w / 2.0,
        72.0,
        TextParams {
            font,
            font_size: 44,
            color: Color::new(0.85, 0.9, 0.98, 1.0),
            ..Default::default()
        },
    );

    // 金币显示
    let gold_text = format!("金币: {}", gold);
    draw_text_ex(
        &gold_text,
        24.0,
        40.0,
        TextParams {
            font,
            font_size: 26,
            color: GOLD,
            ..Default::default()
        },
    );

    // 商品列表
    let list_w = (screen_width() * 0.72).min(860.0);
    let list_x = screen_width() / 2.0 - list_w / 2.0;
    let mut y = 120.0;
    let row_h = 72.0;

    let mouse = vec2(mouse_position().0, mouse_position().1);
    let mut hover: Option<usize> = None;

    for (i, item) in shop_state.items.iter().enumerate() {
        let rect = Rect::new(list_x, y, list_w, row_h);
        if rect.contains(mouse) {
            hover = Some(i);
        }

        let selected = shop_state.selected == Some(i);
        let (kind, name, desc, color) = roguelike::reward_display_info(&item.reward);
        let display_name = format!("{} · {}", kind, name);

        // 背景
        draw_shadow_panel(rect.x, rect.y, rect.w, rect.h, Color::new(0.08, 0.1, 0.14, 0.88));
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected { 3.5 } else { 2.0 },
            if selected {
                color
            } else {
                Color::new(color.r, color.g, color.b, 0.6)
            },
        );

        // 编号 + 名称
        draw_text_ex(
            &format!("{}.", i + 1),
            rect.x + 16.0,
            rect.y + 28.0,
            TextParams {
                font,
                font_size: 22,
                color: Color::new(0.7, 0.75, 0.85, 1.0),
                ..Default::default()
            },
        );
        draw_text_ex(
            &display_name,
            rect.x + 48.0,
            rect.y + 30.0,
            TextParams {
                font,
                font_size: 24,
                color,
                ..Default::default()
            },
        );
        draw_text_ex(
            &desc,
            rect.x + 48.0,
            rect.y + 54.0,
            TextParams {
                font,
                font_size: 18,
                color: Color::new(0.75, 0.78, 0.85, 1.0),
                ..Default::default()
            },
        );

        // 价格/售罄
        let right = rect.x + rect.w - 18.0;
        if item.sold {
            let sold = "已售";
            let w = measure_text(sold, font, 22, 1.0).width;
            draw_text_ex(
                sold,
                right - w,
                rect.y + 44.0,
                TextParams {
                    font,
                    font_size: 22,
                    color: Color::new(1.0, 0.4, 0.4, 1.0),
                    ..Default::default()
                },
            );
        } else {
            let price = format!("{}g", item.price);
            let w = measure_text(&price, font, 24, 1.0).width;
            let affordable = gold >= item.price;
            draw_text_ex(
                &price,
                right - w,
                rect.y + 44.0,
                TextParams {
                    font,
                    font_size: 24,
                    color: if affordable {
                        GOLD
                    } else {
                        Color::new(0.9, 0.5, 0.5, 1.0)
                    },
                    ..Default::default()
                },
            );
        }

        y += row_h + 10.0;
    }

    // 更新选中状态
    if let Some(h) = hover {
        shop_state.selected = Some(h);
    }

    // 刷新按钮
    let btn_w = 280.0;
    let btn_h = 44.0;
    let btn_x = screen_width() / 2.0 - btn_w / 2.0;
    let btn_y = screen_height() - 110.0;
    let refresh_rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
    let refresh_hover = refresh_rect.contains(mouse);

    draw_shadow_panel(btn_x, btn_y, btn_w, btn_h, Color::new(0.08, 0.1, 0.14, 0.9));
    draw_rectangle_lines(
        btn_x,
        btn_y,
        btn_w,
        btn_h,
        if refresh_hover { 3.0 } else { 2.0 },
        Color::new(0.3, 0.55, 0.9, if refresh_hover { 0.9 } else { 0.65 }),
    );
    let refresh_text = format!("刷新 [R] (-{}g)", refresh_cost);
    let rw = measure_text(&refresh_text, font, 22, 1.0).width;
    draw_text_ex(
        &refresh_text,
        screen_width() / 2.0 - rw / 2.0,
        btn_y + 30.0,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.85, 0.9, 0.98, 1.0),
            ..Default::default()
        },
    );

    // 继续按钮
    let cont_y = screen_height() - 60.0;
    let cont_text = "继续 [Enter]";
    let cont_w = measure_text(cont_text, font, 20, 1.0).width;
    draw_text_ex(
        cont_text,
        screen_width() / 2.0 - cont_w / 2.0,
        cont_y,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(0.6, 0.65, 0.75, 1.0),
            ..Default::default()
        },
    );

    // 处理输入
    if input.is_key_pressed(KeyCode::R) || (refresh_hover && is_mouse_button_pressed(MouseButton::Left)) {
        return ShopUiAction::RefreshRequested;
    }

    // 数字键选择
    let key_select = if input.is_key_pressed(KeyCode::Key1) || input.is_key_pressed(KeyCode::Kp1) {
        Some(0)
    } else if input.is_key_pressed(KeyCode::Key2) || input.is_key_pressed(KeyCode::Kp2) {
        Some(1)
    } else if input.is_key_pressed(KeyCode::Key3) || input.is_key_pressed(KeyCode::Kp3) {
        Some(2)
    } else if input.is_key_pressed(KeyCode::Key4) || input.is_key_pressed(KeyCode::Kp4) {
        Some(3)
    } else if input.is_key_pressed(KeyCode::Key5) || input.is_key_pressed(KeyCode::Kp5) {
        Some(4)
    } else if input.is_key_pressed(KeyCode::Key6) || input.is_key_pressed(KeyCode::Kp6) {
        Some(5)
    } else {
        None
    };

    if let Some(i) = key_select {
        if i < shop_state.items.len() && !shop_state.items[i].sold {
            return ShopUiAction::BuyConfirmed(i);
        }
    }

    // 点击商品购买
    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(i) = hover {
            if !shop_state.items[i].sold {
                return ShopUiAction::BuyConfirmed(i);
            }
        }
    }

    // Enter 退出商店
    if input.is_key_pressed(KeyCode::Enter) {
        return ShopUiAction::ExitShop;
    }

    ShopUiAction::None
}
