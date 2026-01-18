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
use crate::theme::{colors, typography, spacing, draw_lives_icons, draw_arc_progress};
use crate::{GameMode, GameSettings, PauseSelection, SettingOption};

pub enum HudMode {
    Waiting,
    Active { time: f64 },
}

/// 极简 HUD：左上角生命+护盾，右上角分数，底部武器指示器
pub fn draw_players_hud(players: &[Player], mode: HudMode, font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        let hud_time = match mode {
            HudMode::Waiting => None,
            HudMode::Active { time } => Some(time),
        };

        let base_y = spacing::MD + idx as f32 * 90.0;
        let player_color = player.color;

        // === 左上角：玩家标签 + 生命图标 ===
        draw_text_ex(
            &player.label,
            spacing::MD,
            base_y + 16.0,
            TextParams {
                font_size: typography::BODY_SM,
                color: colors::TEXT_SECONDARY,
                font,
                ..Default::default()
            },
        );

        // 生命图标（三角形飞船）
        draw_lives_icons(
            spacing::MD,
            base_y + 24.0,
            player.lives,
            5,
            player_color,
        );

        // === 护盾弧形进度条（围绕生命区域）===
        if let Some(time) = hud_time {
            if player.shield_active(time) {
                let shield_progress = (player.shield_remaining(time) / 10.0) as f32;
                draw_arc_progress(
                    spacing::MD + 45.0,
                    base_y + 32.0,
                    28.0,
                    3.0,
                    shield_progress,
                    colors::BG_PANEL,
                    colors::SHIELD_ACTIVE,
                );
            }
        }

        // === 右上角：分数（大字体醒目显示）===
        let score_text = format!("{}", player.score.value());
        let score_width = measure_text(&score_text, font, typography::TITLE_MD, 1.0).width;
        draw_text_ex(
            &score_text,
            screen_width() - score_width - spacing::LG - idx as f32 * 200.0,
            base_y + 28.0,
            TextParams {
                font_size: typography::TITLE_MD,
                color: player_color,
                font,
                ..Default::default()
            },
        );

        // 分数标签
        draw_text_ex(
            "SCORE",
            screen_width() - score_width - spacing::LG - idx as f32 * 200.0,
            base_y + 8.0,
            TextParams {
                font_size: typography::CAPTION,
                color: colors::TEXT_MUTED,
                font,
                ..Default::default()
            },
        );

        // === 状态指示器（无敌/死亡）===
        if let Some(time) = hud_time {
            let status_text = if player.is_invulnerable(time) {
                Some(("SAFE", colors::INVULNERABLE))
            } else if !player.alive {
                Some(("DOWN", colors::DANGER))
            } else {
                None
            };

            if let Some((text, color)) = status_text {
                let pulse = 0.6 + 0.4 * (time as f32 * 4.0).sin().abs();
                draw_text_ex(
                    text,
                    spacing::MD + 100.0,
                    base_y + 16.0,
                    TextParams {
                        font_size: typography::BODY_SM,
                        color: Color::new(color.r, color.g, color.b, pulse),
                        font,
                        ..Default::default()
                    },
                );
            }
        }

        // === 武器指示器（底部中央）===
        let weapon_icon = match player.weapon_type {
            WeaponType::Normal => "●",
            WeaponType::Spread => "◆◆◆",
            WeaponType::Penetrating => "▶▶",
            WeaponType::Homing => "◎",
            WeaponType::ChainIon => "⚡",
        };
        let weapon_name = match player.weapon_type {
            WeaponType::Normal => "NORMAL",
            WeaponType::Spread => "SPREAD",
            WeaponType::Penetrating => "PIERCE",
            WeaponType::Homing => "HOMING",
            WeaponType::ChainIon => "CHAIN",
        };

        if idx == 0 {
            let weapon_y = screen_height() - spacing::LG;
            let icon_width = measure_text(weapon_icon, font, typography::BODY_LG, 1.0).width;
            let name_width = measure_text(weapon_name, font, typography::CAPTION, 1.0).width;

            draw_text_ex(
                weapon_icon,
                screen_width() / 2.0 - icon_width / 2.0,
                weapon_y - 12.0,
                TextParams {
                    font_size: typography::BODY_LG,
                    color: player_color,
                    font,
                    ..Default::default()
                },
            );
            draw_text_ex(
                weapon_name,
                screen_width() / 2.0 - name_width / 2.0,
                weapon_y + 4.0,
                TextParams {
                    font_size: typography::CAPTION,
                    color: colors::TEXT_MUTED,
                    font,
                    ..Default::default()
                },
            );
        }

        // === 生存时间（紧凑显示在分数下方）===
        if let Some(time) = hud_time {
            let survival = player.survival_time(time);
            let survival_text = format!("{:.1}s", survival);
            let survival_width = measure_text(&survival_text, font, typography::BODY_SM, 1.0).width;
            draw_text_ex(
                &survival_text,
                screen_width() - survival_width - spacing::LG - idx as f32 * 200.0,
                base_y + 48.0,
                TextParams {
                    font_size: typography::BODY_SM,
                    color: colors::TEXT_SECONDARY,
                    font,
                    ..Default::default()
                },
            );
        }
    }
}

/// 绘制 Flux 能量条
pub fn draw_flux_bar(players: &[Player], font: Option<&Font>) {
    use crate::constants::flux;

    for (idx, player) in players.iter().enumerate() {
        if !player.alive {
            continue;
        }

        // Flux 条位置（在玩家 HUD 下方）
        let bar_x = 20.0;
        let bar_y = 70.0 + idx as f32 * 80.0;
        let bar_width = 150.0;
        let bar_height = 12.0;

        // 背景
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.2, 0.2, 0.2, 0.8));

        // Flux 填充
        let fill_width = bar_width * player.flux_percent();
        let flux_color = if player.is_flux_high() {
            Color::new(0.2, 0.9, 1.0, 1.0) // 高能量：青色
        } else if player.is_flux_low() {
            Color::new(1.0, 0.3, 0.2, 1.0) // 低能量：红色
        } else {
            Color::new(0.4, 0.7, 1.0, 1.0) // 正常：蓝色
        };
        draw_rectangle(bar_x, bar_y, fill_width, bar_height, flux_color);

        // 边框
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.5, WHITE);

        // 阈值标记
        let high_x = bar_x + bar_width * (flux::HIGH_THRESHOLD / flux::MAX);
        let low_x = bar_x + bar_width * (flux::LOW_THRESHOLD / flux::MAX);
        draw_line(high_x, bar_y, high_x, bar_y + bar_height, 1.0, Color::new(0.2, 0.9, 1.0, 0.5));
        draw_line(low_x, bar_y, low_x, bar_y + bar_height, 1.0, Color::new(1.0, 0.3, 0.2, 0.5));

        // 标签
        draw_text_ex(
            "FLUX",
            bar_x,
            bar_y - 2.0,
            TextParams {
                font_size: 12,
                color: flux_color,
                font,
                ..Default::default()
            },
        );

        // 数值
        draw_text_ex(
            &format!("{:.0}", player.flux),
            bar_x + bar_width + 5.0,
            bar_y + 10.0,
            TextParams {
                font_size: 12,
                color: flux_color,
                font,
                ..Default::default()
            },
        );

        // 状态提示
        if player.is_flux_high() {
            draw_text_ex(
                "HIGH",
                bar_x + bar_width - 30.0,
                bar_y - 2.0,
                TextParams {
                    font_size: 10,
                    color: Color::new(0.2, 0.9, 1.0, 1.0),
                    font,
                    ..Default::default()
                },
            );
        } else if player.is_flux_low() {
            draw_text_ex(
                "LOW!",
                bar_x + bar_width - 25.0,
                bar_y - 2.0,
                TextParams {
                    font_size: 10,
                    color: Color::new(1.0, 0.3, 0.2, 1.0),
                    font,
                    ..Default::default()
                },
            );
        }
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

    // === 响应式布局计算 ===
    let sw = screen_width();
    let sh = screen_height();
    // 基准分辨率 1024x768，缩放因子范围 0.6~1.5
    let scale = (sw / 1024.0).min(sh / 768.0).clamp(0.6, 1.5);

    // 字体大小响应式
    let title_size = ((48.0 * scale) as u16).max(28);
    let subtitle_size = ((24.0 * scale) as u16).max(14);
    let hint_size = ((26.0 * scale) as u16).max(16);

    // 标题区域高度（确保小屏不遮挡）
    let title_area_height = 140.0 * scale;
    // 底部提示区域高度
    let hint_area_height = 60.0 * scale;
    // 可用于卡片的高度
    let available_height = sh - title_area_height - hint_area_height;

    // Section header 布局参数
    let section_header_height = 28.0 * scale;
    let section_gap = 12.0 * scale;
    let num_sections = 3.0;
    let total_section_overhead = section_header_height * num_sections + section_gap * (num_sections - 1.0);

    // 卡片尺寸响应式计算
    let num_cards = 7.0;
    let base_spacing = 8.0 * scale;
    let total_spacing = base_spacing * (num_cards - 1.0);
    // 卡片高度 = (可用高度 - 间距总和 - section overhead) / 卡片数
    let card_height = ((available_height - total_spacing - total_section_overhead) / num_cards).clamp(50.0, 100.0);
    // 卡片宽度：小屏用更多宽度，大屏限制最大宽度
    let card_width = (sw * 0.75).min(800.0).max(sw * 0.5);
    let spacing = base_spacing;

    // 重新计算实际总高度并居中
    let total_height = card_height * num_cards + spacing * (num_cards - 1.0) + total_section_overhead;
    let start_y = title_area_height + (available_height - total_height) / 2.0;
    let card_x = (sw - card_width) / 2.0;

    // Section Y 坐标计算
    let game_modes_header_y = start_y;
    let game_modes_cards_start_y = game_modes_header_y + section_header_height;

    let progress_header_y = game_modes_cards_start_y + (card_height + spacing) * 5.0 + section_gap;
    let progress_cards_start_y = progress_header_y + section_header_height;

    let system_header_y = progress_cards_start_y + (card_height + spacing) * 1.0 + section_gap;
    let system_cards_start_y = system_header_y + section_header_height;

    // Section 颜色定义
    let game_modes_color = Color::new(0.30, 0.60, 0.95, 1.0);  // 宇宙蓝
    let progress_color = Color::new(1.0, 0.84, 0.0, 1.0);       // 金色
    let system_color = Color::new(0.30, 0.85, 0.45, 1.0);       // 翡翠绿

    // 判断各 section 是否激活
    let game_modes_active = matches!(selection,
        GameMode::Survival | GameMode::Duel | GameMode::TimeAttack |
        GameMode::Roguelike | GameMode::Online);
    let progress_active = matches!(selection, GameMode::Achievements);
    let system_active = matches!(selection, GameMode::Settings);

    // === 边缘暗角效果（突出星空深度）===
    let vignette_alpha = 0.35;
    // 顶部渐变
    for i in 0..4 {
        let h = 25.0 - i as f32 * 5.0;
        let a = vignette_alpha * (1.0 - i as f32 * 0.25);
        draw_rectangle(0.0, i as f32 * 25.0, screen_width(), h, Color::new(0.0, 0.0, 0.02, a));
    }
    // 底部渐变
    for i in 0..4 {
        let h = 25.0 - i as f32 * 5.0;
        let a = vignette_alpha * (1.0 - i as f32 * 0.25);
        draw_rectangle(0.0, screen_height() - (i + 1) as f32 * 25.0, screen_width(), h, Color::new(0.0, 0.0, 0.02, a));
    }

    let title = "Choose Your Adventure";
    let subtitle = "Press [M] later to revisit this screen";
    // title_size, subtitle_size 已在上面响应式计算

    // 标题Y位置响应式
    let title_y = 50.0 * scale + title_size as f32;
    let subtitle_y = title_y + 45.0 * scale;

    // === 标题发光效果（与星空呼应）===
    let title_pulse = 0.85 + 0.15 * (time * 1.2).sin();

    if let Some(f) = font {
        let title_dims = measure_text(title, Some(f), title_size, 1.0);
        let title_x = sw / 2. - title_dims.width / 2.;

        // 标题发光层（模拟星光）
        draw_text_ex(
            title,
            title_x + 2.0,
            title_y + 2.0,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: Color::new(0.3, 0.5, 0.9, 0.25 * title_pulse),
                ..Default::default()
            },
        );
        // 标题阴影
        draw_text_ex(
            title,
            title_x + 1.0,
            title_y + 1.0,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: Color::new(0.0, 0.0, 0.0, 0.35),
                ..Default::default()
            },
        );
        // 标题主体
        draw_text_ex(
            title,
            title_x,
            title_y,
            TextParams {
                font: Some(f),
                font_size: title_size,
                color: Color::new(0.92 * title_pulse, 0.94 * title_pulse, 0.98, 1.0),
                ..Default::default()
            },
        );

        let subtitle_dims = measure_text(subtitle, Some(f), subtitle_size, 1.0);
        draw_text_ex(
            subtitle,
            sw / 2. - subtitle_dims.width / 2.,
            subtitle_y,
            TextParams {
                font: Some(f),
                font_size: subtitle_size,
                color: Color::new(0.55, 0.60, 0.72, 0.9),
                ..Default::default()
            },
        );
    } else {
        let title_width = measure_text(title, None, title_size, 1.0).width;
        let title_x = sw / 2. - title_width / 2.;

        // 发光层
        draw_text(
            title,
            title_x + 2.0,
            title_y + 2.0,
            title_size as f32,
            Color::new(0.3, 0.5, 0.9, 0.25 * title_pulse),
        );
        // 主体
        draw_text(
            title,
            title_x,
            title_y,
            title_size as f32,
            Color::new(0.92 * title_pulse, 0.94 * title_pulse, 0.98, 1.0),
        );

        let subtitle_width = measure_text(subtitle, None, subtitle_size, 1.0).width;
        draw_text(
            subtitle,
            sw / 2. - subtitle_width / 2.,
            subtitle_y,
            subtitle_size as f32,
            Color::new(0.55, 0.60, 0.72, 0.9),
        );
    }

    // 卡片布局已在上方响应式计算

    // ═══════════════════════════════════════════════════════════════
    // SECTION 1: GAME MODES
    // ═══════════════════════════════════════════════════════════════
    draw_section_header("GAME MODES", game_modes_header_y, card_width, card_x, game_modes_color, game_modes_active, scale, font);

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
        y: game_modes_cards_start_y,
        width: card_width,
        height: card_height,
        title: "Survival",
        desc: survival_desc,
        detail: &survival_detail,
        active: matches!(selection, GameMode::Survival),
        accent: BLUE,
        footer: "[Enter] Start",
        font,
        scale,
    });

    // Duel 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: game_modes_cards_start_y + (card_height + spacing),
        width: card_width,
        height: card_height,
        title: "Duel",
        desc: "Face each other in timed shoot-outs with streak bonuses.",
        detail: "Competitive mode with flag capture and killstreak rewards.",
        active: matches!(selection, GameMode::Duel),
        accent: RED,
        footer: "[Enter] Start",
        font,
        scale,
    });

    // TimeAttack 卡片
    let time_attack_detail = format!(
        "Score as much as possible! [←/→] to switch: {}",
        time_attack_duration.name()
    );
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: game_modes_cards_start_y + (card_height + spacing) * 2.,
        width: card_width,
        height: card_height,
        title: "Time Attack",
        desc: "Race against the clock - score as many points as possible!",
        detail: &time_attack_detail,
        active: matches!(selection, GameMode::TimeAttack),
        accent: Color::new(1.0, 0.5, 0.0, 1.0), // 橙色
        footer: "[Enter] Start",
        font,
        scale,
    });

    // Roguelike 卡片
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: game_modes_cards_start_y + (card_height + spacing) * 3.,
        width: card_width,
        height: card_height,
        title: "Roguelike",
        desc: "Run-based adventure with random builds and permanent upgrades!",
        detail: "3 zones, unique bosses, relics, and card synergies each run.",
        active: matches!(selection, GameMode::Roguelike),
        accent: Color::new(0.8, 0.2, 0.6, 1.0), // 品红色
        footer: "[Enter] Start Run",
        font,
        scale,
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
        y: game_modes_cards_start_y + (card_height + spacing) * 4.,
        width: card_width,
        height: card_height,
        title: online_title,
        desc: online_desc,
        detail: online_detail,
        active: online_active,
        accent: online_accent,
        footer: online_footer,
        font,
        scale,
    });

    // ═══════════════════════════════════════════════════════════════
    // SECTION 2: PROGRESS
    // ═══════════════════════════════════════════════════════════════
    draw_section_header("PROGRESS", progress_header_y, card_width, card_x, progress_color, progress_active, scale, font);

    // Achievements 卡片
    let (unlocked, total) = achievements.get_stats();
    let achievements_summary = format!("Unlocked: {} / {} achievements", unlocked, total);
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: progress_cards_start_y,
        width: card_width,
        height: card_height,
        title: "Achievements",
        desc: &achievements_summary,
        detail: "Track your progress and unlock rewards as you play.",
        active: matches!(selection, GameMode::Achievements),
        accent: Color::new(0.9, 0.7, 0.2, 1.0), // 金色
        footer: "[Enter] View",
        font,
        scale,
    });

    // ═══════════════════════════════════════════════════════════════
    // SECTION 3: SYSTEM
    // ═══════════════════════════════════════════════════════════════
    draw_section_header("SYSTEM", system_header_y, card_width, card_x, system_color, system_active, scale, font);

    // Settings 卡片
    let settings_summary = format!(
        "Lives: {} | Ship: {:.1}x | Asteroids: {:.1}x",
        settings.starting_lives, settings.ship_speed_multiplier, settings.asteroid_speed_multiplier,
    );
    draw_mode_card(ModeCardParams {
        x: card_x,
        y: system_cards_start_y,
        width: card_width,
        height: card_height,
        title: "Settings",
        desc: &settings_summary,
        detail: "Configure game difficulty, effects, and features.",
        active: matches!(selection, GameMode::Settings),
        accent: Color::new(0.3, 0.7, 0.3, 1.0), // 绿色
        footer: "[Enter] Configure",
        font,
        scale,
    });

    // 底部提示使用响应式字体（亮色）
    let hint = "[Up/Down W/S] Select  |  [Enter Space] Confirm";
    let hint_y = sh - 30.0 * scale;
    if let Some(f) = font {
        let hint_dims = measure_text(hint, Some(f), hint_size, 1.0);
        draw_text_ex(
            hint,
            sw / 2. - hint_dims.width / 2.,
            hint_y,
            TextParams {
                font: Some(f),
                font_size: hint_size,
                color: Color::new(0.65, 0.7, 0.8, 1.0),
                ..Default::default()
            },
        );
    } else {
        let hint_width = measure_text(hint, None, hint_size, 1.0).width;
        draw_text(
            hint,
            sw / 2. - hint_width / 2.,
            hint_y,
            hint_size as f32,
            Color::new(0.65, 0.7, 0.8, 1.0),
        );
    }
}

/// 绘制分区标题（带渐变分隔线）
fn draw_section_header(
    text: &str,
    y: f32,
    card_width: f32,
    card_x: f32,
    color: Color,
    is_active: bool,
    scale: f32,
    font: Option<&Font>,
) {
    let header_size = ((16.0 * scale) as u16).max(12);
    let alpha = if is_active { 1.0 } else { 0.7 };
    let text_color = Color::new(color.r, color.g, color.b, alpha);

    // 测量文字宽度
    let text_width = if let Some(f) = font {
        measure_text(text, Some(f), header_size, 1.0).width
    } else {
        measure_text(text, None, header_size, 1.0).width
    };

    let center_x = card_x + card_width / 2.0;
    let text_x = center_x - text_width / 2.0;
    let line_y = y + 8.0 * scale;

    // 左侧渐变线
    let line_padding = 12.0 * scale;
    let left_line_end = text_x - line_padding;
    let left_line_start = card_x + 20.0 * scale;
    let line_length = left_line_end - left_line_start;

    if line_length > 10.0 {
        let segments = 8;
        let seg_width = line_length / segments as f32;
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let seg_alpha = t * 0.5 * alpha;
            draw_rectangle(
                left_line_start + i as f32 * seg_width,
                line_y,
                seg_width + 0.5,
                1.5,
                Color::new(color.r, color.g, color.b, seg_alpha),
            );
        }
    }

    // 右侧渐变线（镜像）
    let right_line_start = text_x + text_width + line_padding;
    let right_line_end = card_x + card_width - 20.0 * scale;
    let right_line_length = right_line_end - right_line_start;

    if right_line_length > 10.0 {
        let segments = 8;
        let seg_width = right_line_length / segments as f32;
        for i in 0..segments {
            let t = 1.0 - (i as f32 / segments as f32);
            let seg_alpha = t * 0.5 * alpha;
            draw_rectangle(
                right_line_start + i as f32 * seg_width,
                line_y,
                seg_width + 0.5,
                1.5,
                Color::new(color.r, color.g, color.b, seg_alpha),
            );
        }
    }

    // 菱形装饰符
    let diamond = "◆";
    let diamond_spacing = 6.0 * scale;
    let diamond_size = ((12.0 * scale) as u16).max(10);
    let diamond_color = Color::new(color.r, color.g, color.b, alpha * 0.6);

    // 绘制标题文字
    if let Some(f) = font {
        // 左菱形
        draw_text_ex(diamond, text_x - diamond_spacing - 10.0 * scale, y + 12.0 * scale, TextParams {
            font: Some(f), font_size: diamond_size, color: diamond_color, ..Default::default()
        });
        // 标题
        draw_text_ex(text, text_x, y + 12.0 * scale, TextParams {
            font: Some(f), font_size: header_size, color: text_color, ..Default::default()
        });
        // 右菱形
        draw_text_ex(diamond, text_x + text_width + diamond_spacing, y + 12.0 * scale, TextParams {
            font: Some(f), font_size: diamond_size, color: diamond_color, ..Default::default()
        });
    } else {
        draw_text(diamond, text_x - diamond_spacing - 10.0 * scale, y + 12.0 * scale, diamond_size as f32, diamond_color);
        draw_text(text, text_x, y + 12.0 * scale, header_size as f32, text_color);
        draw_text(diamond, text_x + text_width + diamond_spacing, y + 12.0 * scale, diamond_size as f32, diamond_color);
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
    scale: f32, // 响应式缩放因子
}

fn draw_mode_card(params: ModeCardParams) {
    use crate::theme::colors;

    let is_active = params.active;
    let time = macroquad::time::get_time() as f32;
    let s = params.scale; // 缩放因子

    // 响应式字体大小
    let title_font = ((24.0 * s) as u16).max(16);
    let desc_font = ((15.0 * s) as u16).max(12);
    let detail_font = ((13.0 * s) as u16).max(10);
    let footer_font = ((15.0 * s) as u16).max(11);

    // 响应式位置偏移
    let icon_offset_x = 30.0 * s;
    let text_offset_x = 65.0 * s;
    let icon_radius = 16.0 * s;
    let icon_glow_radius = 22.0 * s;

    // 活跃卡片的浮动动画
    let float_offset = if is_active {
        (time * 2.0).sin() * 3.0 * s
    } else {
        0.0
    };
    let card_y = params.y + float_offset;

    // === 毛玻璃效果 (Glassmorphism) ===

    // 1. 外发光层（模拟玻璃边缘光散射）
    if is_active {
        for i in 1..=3 {
            let offset = i as f32 * 3.0;
            let glow_alpha = 0.08 / i as f32 * (0.8 + 0.2 * (time * 2.5).sin());
            draw_rectangle(
                params.x - offset,
                card_y - offset,
                params.width + offset * 2.0,
                params.height + offset * 2.0,
                Color::new(params.accent.r, params.accent.g, params.accent.b, glow_alpha),
            );
        }
    }

    // 2. 半透明玻璃背景（让星空透出）
    let bg_alpha = if is_active { 0.55 } else { 0.40 };
    let bg_color = if is_active {
        Color::new(0.06, 0.10, 0.18, bg_alpha)
    } else {
        Color::new(0.04, 0.05, 0.10, bg_alpha)
    };
    draw_rectangle(params.x, card_y, params.width, params.height, bg_color);

    // 3. 顶部高光条（玻璃反光效果）
    let highlight_alpha = if is_active { 0.12 } else { 0.06 };
    draw_rectangle(
        params.x,
        card_y,
        params.width,
        1.5,
        Color::new(1.0, 1.0, 1.0, highlight_alpha),
    );

    // 4. 底部渐变阴影（增加立体感）
    draw_rectangle(
        params.x,
        card_y + params.height - 2.0,
        params.width,
        2.0,
        Color::new(0.0, 0.0, 0.0, 0.15),
    );

    // 左侧强调色条（带发光）
    let accent_bar_width = if is_active { 5.0 } else { 3.0 };
    if is_active {
        // 强调条发光
        draw_rectangle(
            params.x - 2.0,
            card_y,
            accent_bar_width + 4.0,
            params.height,
            Color::new(params.accent.r, params.accent.g, params.accent.b, 0.25),
        );
    }
    draw_rectangle(
        params.x,
        card_y,
        accent_bar_width,
        params.height,
        params.accent,
    );

    // 边框（细腻的玻璃边缘）
    let border_color = if is_active {
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.7)
    } else {
        Color::new(0.4, 0.45, 0.55, 0.35)
    };
    draw_rectangle_lines(
        params.x,
        card_y,
        params.width,
        params.height,
        if is_active { 1.5 } else { 1.0 },
        border_color,
    );

    // 图标区域（带光晕）
    let icon_x = params.x + icon_offset_x;
    let icon_y = card_y + params.height / 2.0;
    let icon_alpha = if is_active { 1.0 } else { 0.5 };

    // 图标光晕
    if is_active {
        draw_circle(
            icon_x,
            icon_y,
            icon_glow_radius,
            Color::new(params.accent.r, params.accent.g, params.accent.b, 0.12),
        );
    }
    draw_circle(
        icon_x,
        icon_y,
        icon_radius,
        Color::new(params.accent.r, params.accent.g, params.accent.b, 0.18 * icon_alpha),
    );
    draw_circle_lines(
        icon_x,
        icon_y,
        icon_radius,
        if is_active { 2.0 } else { 1.5 },
        Color::new(params.accent.r, params.accent.g, params.accent.b, icon_alpha),
    );

    // === 文字区域（带阴影增强可读性）===
    let text_x = params.x + text_offset_x;

    // 计算文字垂直布局（根据卡片高度分布）
    let line_height = params.height / 4.0;
    let title_y = card_y + line_height * 1.2;
    let desc_y = card_y + line_height * 2.2;
    let detail_y = card_y + line_height * 3.0;

    // 标题阴影层
    draw_text_ex(
        params.title,
        text_x + 1.0,
        title_y + 1.0,
        TextParams {
            font: params.font,
            font_size: title_font,
            color: Color::new(0.0, 0.0, 0.0, 0.4),
            ..Default::default()
        },
    );

    // 标题发光层（活跃时）
    if is_active {
        draw_text_ex(
            params.title,
            text_x,
            title_y,
            TextParams {
                font: params.font,
                font_size: title_font,
                color: Color::new(params.accent.r, params.accent.g, params.accent.b, 0.35),
                ..Default::default()
            },
        );
    }

    // 标题主体
    let title_color = if is_active {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_SECONDARY
    };
    draw_text_ex(
        params.title,
        text_x,
        title_y,
        TextParams {
            font: params.font,
            font_size: title_font,
            color: title_color,
            ..Default::default()
        },
    );

    // 描述（带轻微阴影）
    draw_text_ex(
        params.desc,
        text_x + 0.5,
        desc_y + 0.5,
        TextParams {
            font: params.font,
            font_size: desc_font,
            color: Color::new(0.0, 0.0, 0.0, 0.3),
            ..Default::default()
        },
    );
    draw_text_ex(
        params.desc,
        text_x,
        desc_y,
        TextParams {
            font: params.font,
            font_size: desc_font,
            color: if is_active {
                Color::new(0.75, 0.78, 0.85, 1.0)
            } else {
                colors::TEXT_MUTED
            },
            ..Default::default()
        },
    );

    // 详情（第三行）
    draw_text_ex(
        params.detail,
        text_x,
        detail_y,
        TextParams {
            font: params.font,
            font_size: detail_font,
            color: Color::new(0.55, 0.60, 0.70, if is_active { 1.0 } else { 0.8 }),
            ..Default::default()
        },
    );

    // 右侧操作提示（玻璃按钮效果）
    if is_active {
        let footer_width = measure_text(params.footer, params.font, footer_font, 1.0).width;
        let btn_padding = 12.0 * s;
        let tag_x = params.x + params.width - footer_width - btn_padding * 2.0;
        let tag_h = 22.0 * s;
        let tag_y = card_y + params.height / 2.0 - tag_h / 2.0;
        let tag_w = footer_width + btn_padding;

        // 按钮发光
        draw_rectangle(
            tag_x - btn_padding,
            tag_y - 4.0 * s,
            tag_w + 4.0 * s,
            tag_h + 4.0 * s,
            Color::new(params.accent.r, params.accent.g, params.accent.b, 0.1),
        );
        // 按钮背景
        draw_rectangle(
            tag_x - btn_padding + 2.0 * s,
            tag_y - 2.0 * s,
            tag_w,
            tag_h,
            Color::new(params.accent.r * 0.2, params.accent.g * 0.2, params.accent.b * 0.2, 0.5),
        );
        // 按钮高光
        draw_rectangle(
            tag_x - btn_padding + 2.0 * s,
            tag_y - 2.0 * s,
            tag_w,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.15),
        );
        // 按钮边框
        draw_rectangle_lines(
            tag_x - btn_padding + 2.0 * s,
            tag_y - 2.0 * s,
            tag_w,
            tag_h,
            1.0,
            Color::new(params.accent.r, params.accent.g, params.accent.b, 0.6),
        );
        // 按钮文字
        draw_text_ex(
            params.footer,
            tag_x,
            tag_y + tag_h * 0.65,
            TextParams {
                font: params.font,
                font_size: footer_font,
                color: params.accent,
                ..Default::default()
            },
        );
    }
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
    use crate::theme::colors;

    // 半透明背景遮罩
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        Color::new(0.02, 0.03, 0.06, 0.75),
    );

    let time = macroquad::time::get_time() as f32;

    // 简洁的中央面板
    let panel_width = 400.0;
    let panel_height = 280.0;
    let x = screen_width() / 2. - panel_width / 2.;
    let y = screen_height() / 2. - panel_height / 2.;

    // 面板背景
    draw_rectangle(x, y, panel_width, panel_height, Color::new(0.06, 0.08, 0.12, 0.98));
    draw_rectangle_lines(x, y, panel_width, panel_height, 2.0, colors::PRIMARY);

    // 标题
    let title = "PAUSED";
    let title_width = measure_text(title, font, 42, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2. - title_width / 2.,
        y + 55.,
        TextParams {
            font_size: 42,
            color: colors::TEXT_PRIMARY,
            font,
            ..Default::default()
        },
    );

    // 分隔线
    draw_line(
        x + 40.0,
        y + 80.0,
        x + panel_width - 40.0,
        y + 80.0,
        1.0,
        Color::new(0.3, 0.35, 0.45, 0.5),
    );

    // Resume 选项
    draw_pause_option_modern(
        x + 30.0,
        y + 100.0,
        panel_width - 60.0,
        "Resume",
        "Continue playing",
        matches!(selection, PauseSelection::Resume),
        colors::SUCCESS,
        time,
        font,
    );

    // Mode Select 选项
    draw_pause_option_modern(
        x + 30.0,
        y + 170.0,
        panel_width - 60.0,
        "Exit to Menu",
        "Abandon current run",
        matches!(selection, PauseSelection::ModeSelect),
        colors::DANGER,
        time,
        font,
    );

    // 底部提示
    let hint = "↑↓ Select  •  Enter Confirm  •  Esc Resume";
    let hint_width = measure_text(hint, font, 14, 1.0).width;
    draw_text_ex(
        hint,
        screen_width() / 2. - hint_width / 2.,
        y + panel_height - 20.,
        TextParams {
            font,
            font_size: 14,
            color: colors::TEXT_MUTED,
            ..Default::default()
        },
    );
}

fn draw_pause_option_modern(
    x: f32,
    y: f32,
    width: f32,
    title: &str,
    desc: &str,
    active: bool,
    accent: Color,
    time: f32,
    font: Option<&Font>,
) {
    use crate::theme::colors;

    let height = 55.0;

    // 背景
    let bg = if active {
        Color::new(accent.r * 0.15, accent.g * 0.15, accent.b * 0.15, 0.8)
    } else {
        Color::new(0.08, 0.09, 0.12, 0.6)
    };
    draw_rectangle(x, y, width, height, bg);

    // 左侧强调条
    if active {
        let pulse = 0.8 + 0.2 * (time * 4.0).sin();
        draw_rectangle(
            x,
            y,
            4.0,
            height,
            Color::new(accent.r * pulse, accent.g * pulse, accent.b * pulse, 1.0),
        );
    }

    // 边框
    draw_rectangle_lines(
        x,
        y,
        width,
        height,
        if active { 1.5 } else { 1.0 },
        if active { accent } else { Color::new(0.3, 0.35, 0.4, 0.4) },
    );

    // 标题
    let title_color = if active { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
    draw_text_ex(
        title,
        x + 20.0,
        y + 25.0,
        TextParams {
            font,
            font_size: 22,
            color: title_color,
            ..Default::default()
        },
    );

    // 描述
    draw_text_ex(
        desc,
        x + 20.0,
        y + 45.0,
        TextParams {
            font,
            font_size: 14,
            color: colors::TEXT_MUTED,
            ..Default::default()
        },
    );

    // 右侧箭头指示器
    if active {
        draw_text_ex(
            "→",
            x + width - 30.0,
            y + 32.0,
            TextParams {
                font,
                font_size: 20,
                color: accent,
                ..Default::default()
            },
        );
    }
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
                    interp.avg_player_snapshots,
                    interp.avg_bullet_snapshots,
                    interp.render_delay_ms
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
    let panel_height = 950.; // 13个选项（每个60px高 + 68px间距）+ 顶部40px + 底部20px
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

    // Hit Stop (新增：控制击中大型小行星时的短暂冻结)
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 8.,
        panel_width,
        "Hit Stop (Freeze on impact)",
        if settings.enable_hit_stop {
            "ON"
        } else {
            "OFF"
        },
        selected == SettingOption::HitStop,
        font,
    );

    // Debug Panel
    draw_setting_option(
        panel_x,
        option_y_start + option_spacing * 9.,
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
        option_y_start + option_spacing * 10.,
        panel_width,
        "Flag Radius (Duel mode)",
        &format!("{:.0}px", settings.flag_radius),
        selected == SettingOption::FlagRadius,
        font,
    );

    // Reset Defaults (特殊样式)
    draw_reset_option(
        panel_x,
        option_y_start + option_spacing * 11.,
        panel_width,
        selected == SettingOption::ResetDefaults,
        font,
    );

    // Reset Achievements (特殊样式)
    draw_reset_achievements_option(
        panel_x,
        option_y_start + option_spacing * 12.,
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
    draw_circle(
        x + size / 2.0,
        y + size / 2.0,
        size / 2.0,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );

    // 进度环（顺时针从顶部开始）
    let segments = 32;
    let filled_segments = (progress * segments as f32) as i32;
    for i in 0..filled_segments {
        let angle1 =
            -std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = -std::f32::consts::FRAC_PI_2
            + ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
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
    draw_circle_lines(x + size / 2.0, y + size / 2.0, size / 2.0, 2.0, buff.color);

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

        draw_rectangle(
            bar_x,
            bar_y,
            bar_width,
            bar_height,
            Color::new(0.3, 0.3, 0.3, 0.5),
        );
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
    for (i, base) in rects.iter().enumerate().take(count) {
        let selected = reward_state.selected == Some(i);
        let scale = if selected { pulse } else { 1.0 };
        let cx = base.x + base.w / 2.0;
        let cy = base.y + base.h / 2.0;
        let rect = Rect::new(
            cx - base.w * scale / 2.0,
            cy - base.h * scale / 2.0,
            base.w * scale,
            base.h * scale,
        );
        draw_reward_card(
            rect,
            &reward_state.options[i],
            selected,
            &format!("{}", i + 1),
            font,
        );
    }

    // 处理输入（支持 1-4 键）
    let key_choice = if input.is_key_pressed(KeyCode::Key1) || input.is_key_pressed(KeyCode::Kp1) {
        Some(0)
    } else if input.is_key_pressed(KeyCode::Key2) || input.is_key_pressed(KeyCode::Kp2) {
        Some(1)
    } else if input.is_key_pressed(KeyCode::Key3) || input.is_key_pressed(KeyCode::Kp3) {
        Some(2)
    } else if count >= 4
        && (input.is_key_pressed(KeyCode::Key4) || input.is_key_pressed(KeyCode::Kp4))
    {
        Some(3)
    } else {
        None
    };

    if let Some(i) = key_choice
        && i < count
    {
        reward_state.selected = Some(i);
        return Some(i);
    }

    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(i) = hover
    {
        reward_state.selected = Some(i);
        return Some(i);
    }

    None
}

// ============================================================================
// Roguelike：挑战选择 UI
// ============================================================================

/// 挑战选择操作
pub enum ChallengeOfferAction {
    None,
    Accept,
    Skip,
}

/// 绘制挑战选择界面
pub fn draw_challenge_offer(
    challenge_state: &roguelike::ChallengeState,
    input: &Input,
    font: Option<&Font>,
) -> ChallengeOfferAction {
    // 半透明背景
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.04, 0.05, 0.07, 0.95),
    );

    // 标题
    let title = "精英挑战";
    let title_w = measure_text(title, font, 44, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2.0 - title_w / 2.0,
        90.0,
        TextParams {
            font,
            font_size: 44,
            color: Color::new(0.9, 0.85, 0.6, 1.0),
            ..Default::default()
        },
    );

    let subtitle = format!("波次 {} - 可选挑战", challenge_state.wave_in_zone);
    let subtitle_w = measure_text(&subtitle, font, 22, 1.0).width;
    draw_text_ex(
        &subtitle,
        screen_width() / 2.0 - subtitle_w / 2.0,
        122.0,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.7, 0.75, 0.85, 1.0),
            ..Default::default()
        },
    );

    // 条件说明
    let mut y = 170.0;
    for line in challenge_state.description_lines() {
        draw_text_ex(
            &format!("• {}", line),
            screen_width() / 2.0 - 240.0,
            y,
            TextParams {
                font,
                font_size: 22,
                color: Color::new(0.85, 0.9, 0.98, 1.0),
                ..Default::default()
            },
        );
        y += 28.0;
    }

    // 奖励与惩罚
    draw_text_ex(
        "成功奖励：稀有卡牌 / 强遗物 / 双倍金币",
        screen_width() / 2.0 - 240.0,
        y + 20.0,
        TextParams {
            font,
            font_size: 20,
            color: GOLD,
            ..Default::default()
        },
    );
    let penalty_pct = (challenge_state.penalty_gold_ratio * 100.0).round() as u32;
    draw_text_ex(
        &format!("失败惩罚：损失 {}% 金币（不影响生命）", penalty_pct),
        screen_width() / 2.0 - 240.0,
        y + 50.0,
        TextParams {
            font,
            font_size: 20,
            color: Color::new(0.95, 0.5, 0.5, 1.0),
            ..Default::default()
        },
    );

    // 操作提示
    let hint = "按 1 接受挑战 / 按 2 跳过";
    let hint_w = measure_text(hint, font, 22, 1.0).width;
    draw_text_ex(
        hint,
        screen_width() / 2.0 - hint_w / 2.0,
        screen_height() - 90.0,
        TextParams {
            font,
            font_size: 22,
            color: Color::new(0.6, 0.65, 0.78, 1.0),
            ..Default::default()
        },
    );

    if input.is_key_pressed(KeyCode::Key1) || input.is_key_pressed(KeyCode::Enter) {
        return ChallengeOfferAction::Accept;
    }
    if input.is_key_pressed(KeyCode::Key2) || input.is_key_pressed(KeyCode::Escape) {
        return ChallengeOfferAction::Skip;
    }

    ChallengeOfferAction::None
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
        draw_shadow_panel(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.08, 0.1, 0.14, 0.88),
        );
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
    if input.is_key_pressed(KeyCode::R)
        || (refresh_hover && is_mouse_button_pressed(MouseButton::Left))
    {
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

    if let Some(i) = key_select
        && i < shop_state.items.len()
        && !shop_state.items[i].sold
    {
        return ShopUiAction::BuyConfirmed(i);
    }

    // 点击商品购买
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(i) = hover
        && !shop_state.items[i].sold
    {
        return ShopUiAction::BuyConfirmed(i);
    }

    // Enter 退出商店
    if input.is_key_pressed(KeyCode::Enter) {
        return ShopUiAction::ExitShop;
    }

    ShopUiAction::None
}

/// 休息阶段UI动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestUiAction {
    None,
    SelectOption(usize),
    #[allow(dead_code)]
    SelectCard(usize),
    ConfirmRest,
}

/// 绘制休息阶段UI
pub fn draw_rest_ui(
    rest_state: &mut roguelike::RestPhaseState,
    players: &[Player],
    font: Option<&Font>,
) -> RestUiAction {
    let mouse = mouse_position();

    // 标题
    let title = "休息阶段";
    let tw = measure_text(title, font, 32, 1.0).width;
    draw_text_ex(
        title,
        screen_width() / 2.0 - tw / 2.0,
        80.0,
        TextParams {
            font,
            font_size: 32,
            color: WHITE,
            ..Default::default()
        },
    );

    // 说明文字
    let desc = "选择一个休息选项";
    let dw = measure_text(desc, font, 18, 1.0).width;
    draw_text_ex(
        desc,
        screen_width() / 2.0 - dw / 2.0,
        120.0,
        TextParams {
            font,
            font_size: 18,
            color: LIGHTGRAY,
            ..Default::default()
        },
    );

    let mut hover = None;
    let start_y = 180.0;
    let option_h = 60.0;
    let option_w = 400.0;
    let option_x = screen_width() / 2.0 - option_w / 2.0;

    // 绘制选项
    for (i, option) in rest_state.options.iter().enumerate() {
        let y = start_y + i as f32 * (option_h + 15.0);
        let rect = Rect::new(option_x, y, option_w, option_h);
        let is_hover = rect.contains(mouse.into());

        if is_hover {
            hover = Some(i);
        }

        // 选项背景
        let bg_color = if rest_state.selected == Some(i) {
            Color::new(0.2, 0.4, 0.7, 0.8)
        } else if is_hover {
            Color::new(0.15, 0.25, 0.4, 0.8)
        } else {
            Color::new(0.1, 0.15, 0.25, 0.8)
        };

        draw_shadow_panel(option_x, y, option_w, option_h, bg_color);
        draw_rectangle_lines(option_x, y, option_w, option_h, 2.0, Color::new(0.3, 0.5, 0.8, 0.8));

        // 选项名称
        let option_name = match option {
            roguelike::RestOption::Heal => "恢复生命",
            roguelike::RestOption::UpgradeCard => "升级卡牌",
            roguelike::RestOption::RemoveCard => "移除卡牌",
        };
        draw_text_ex(
            option_name,
            option_x + 20.0,
            y + 25.0,
            TextParams {
                font,
                font_size: 20,
                color: WHITE,
                ..Default::default()
            },
        );

        // 选项描述
        let option_desc = match option {
            roguelike::RestOption::Heal => "恢复1点生命值（最多3点）",
            roguelike::RestOption::UpgradeCard => "强化一张已拥有的卡牌",
            roguelike::RestOption::RemoveCard => "移除一张卡牌获得25金币",
        };
        draw_text_ex(
            option_desc,
            option_x + 20.0,
            y + 45.0,
            TextParams {
                font,
                font_size: 14,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
    }

    // 如果选择了卡牌相关选项，显示卡牌选择界面
    if let Some(selected_option) = rest_state.selected {
        let option = &rest_state.options[selected_option];
        if matches!(option, roguelike::RestOption::UpgradeCard | roguelike::RestOption::RemoveCard) {
            // 显示玩家卡牌列表
            let card_start_y = start_y + rest_state.options.len() as f32 * (option_h + 15.0) + 30.0;
            draw_text_ex(
                "选择一张卡牌:",
                option_x,
                card_start_y,
                TextParams {
                    font,
                    font_size: 18,
                    color: WHITE,
                    ..Default::default()
                },
            );

            let mut card_hover = None;
            for (i, player) in players.iter().enumerate() {
                for (j, (card, level)) in player.cards.iter().enumerate() {
                    let card_y = card_start_y + 30.0 + (i * player.cards.len() + j) as f32 * 35.0;
                    let card_rect = Rect::new(option_x, card_y, option_w, 30.0);
                    let is_card_hover = card_rect.contains(mouse.into());

                    if is_card_hover {
                        card_hover = Some((i, j));
                    }

                    let card_bg = if rest_state.card_selection == Some(*card) {
                        Color::new(0.3, 0.5, 0.8, 0.8)
                    } else if is_card_hover {
                        Color::new(0.2, 0.3, 0.5, 0.8)
                    } else {
                        Color::new(0.15, 0.2, 0.3, 0.8)
                    };

                    draw_rectangle(card_rect.x, card_rect.y, card_rect.w, card_rect.h, card_bg);
                    draw_rectangle_lines(card_rect.x, card_rect.y, card_rect.w, card_rect.h, 1.0, Color::new(0.4, 0.6, 0.9, 0.8));

                    // 显示卡牌信息，包括升级等级
                    let level_str = if *level > 0 { format!(" +{}", level) } else { String::new() };
                    draw_text_ex(
                        &format!("P{}: {}{} - {}", i + 1, card.name(), level_str, card.description()),
                        card_rect.x + 10.0,
                        card_rect.y + 20.0,
                        TextParams {
                            font,
                            font_size: 14,
                            color: WHITE,
                            ..Default::default()
                        },
                    );
                }
            }

            // 处理卡牌选择
            if let Some((player_idx, card_idx)) = card_hover
                && is_mouse_button_pressed(MouseButton::Left)
                && let Some((card, _)) = players.get(player_idx).and_then(|p| p.cards.get(card_idx))
            {
                rest_state.card_selection = Some(*card);
            }
        }
    }

    // 确认按钮
    let btn_y = screen_height() - 100.0;
    let btn_w = 200.0;
    let btn_h = 40.0;
    let btn_x = screen_width() / 2.0 - btn_w / 2.0;
    let btn_rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
    let btn_hover = btn_rect.contains(mouse.into());

    draw_shadow_panel(btn_x, btn_y, btn_w, btn_h, Color::new(0.1, 0.3, 0.6, 0.8));
    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, Color::new(0.4, 0.7, 0.9, 0.8));
    draw_text_ex(
        "确认 [Enter]",
        btn_x + btn_w / 2.0 - 40.0,
        btn_y + 25.0,
        TextParams {
            font,
            font_size: 18,
            color: WHITE,
            ..Default::default()
        },
    );

    // 处理输入
    if is_key_pressed(KeyCode::Enter) || (btn_hover && is_mouse_button_pressed(MouseButton::Left)) {
        return RestUiAction::ConfirmRest;
    }

    // 数字键选择选项
    if is_key_pressed(KeyCode::Key1) {
        return RestUiAction::SelectOption(0);
    } else if is_key_pressed(KeyCode::Key2) && rest_state.options.len() > 1 {
        return RestUiAction::SelectOption(1);
    } else if is_key_pressed(KeyCode::Key3) && rest_state.options.len() > 2 {
        return RestUiAction::SelectOption(2);
    }

    // 鼠标选择选项
    if let Some(i) = hover && is_mouse_button_pressed(MouseButton::Left) {
        return RestUiAction::SelectOption(i);
    }

    RestUiAction::None
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // === HudMode 测试 ===

    #[test]
    fn test_hud_mode_waiting_variant() {
        let mode = HudMode::Waiting;
        // HudMode::Waiting 无附加数据
        match mode {
            HudMode::Waiting => (),
            HudMode::Active { .. } => panic!("Expected Waiting variant"),
        }
    }

    #[test]
    fn test_hud_mode_active_variant() {
        let mode = HudMode::Active { time: 42.5 };
        match mode {
            HudMode::Active { time } => assert!((time - 42.5).abs() < f64::EPSILON),
            HudMode::Waiting => panic!("Expected Active variant"),
        }
    }

    // === InterpDebugStats 测试 ===

    #[test]
    fn test_interp_debug_stats_creation() {
        let stats = InterpDebugStats {
            player_buffers: 2,
            asteroid_buffers: 10,
            bullet_buffers: 5,
            avg_player_snapshots: 3.5,
            avg_bullet_snapshots: 2.0,
            render_delay_ms: 100.0,
        };
        assert_eq!(stats.player_buffers, 2);
        assert_eq!(stats.asteroid_buffers, 10);
        assert_eq!(stats.bullet_buffers, 5);
        assert!((stats.avg_player_snapshots - 3.5).abs() < f32::EPSILON);
        assert!((stats.render_delay_ms - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interp_debug_stats_clone() {
        let stats = InterpDebugStats {
            player_buffers: 1,
            asteroid_buffers: 2,
            bullet_buffers: 3,
            avg_player_snapshots: 1.0,
            avg_bullet_snapshots: 2.0,
            render_delay_ms: 50.0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.player_buffers, stats.player_buffers);
        assert_eq!(cloned.asteroid_buffers, stats.asteroid_buffers);
    }

    #[test]
    fn test_interp_debug_stats_debug_format() {
        let stats = InterpDebugStats {
            player_buffers: 1,
            asteroid_buffers: 2,
            bullet_buffers: 3,
            avg_player_snapshots: 1.0,
            avg_bullet_snapshots: 2.0,
            render_delay_ms: 50.0,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("InterpDebugStats"));
        assert!(debug_str.contains("player_buffers"));
    }

    // === NetworkDebugStats 测试 ===

    #[test]
    fn test_network_debug_stats_creation() {
        let net_stats = NetworkDebugStats {
            rtt_ms: 25.5,
            pending_inputs: 3,
            interp: None,
        };
        assert!((net_stats.rtt_ms - 25.5).abs() < f32::EPSILON);
        assert_eq!(net_stats.pending_inputs, 3);
        assert!(net_stats.interp.is_none());
    }

    #[test]
    fn test_network_debug_stats_with_interp() {
        let interp = InterpDebugStats {
            player_buffers: 2,
            asteroid_buffers: 8,
            bullet_buffers: 4,
            avg_player_snapshots: 2.5,
            avg_bullet_snapshots: 1.5,
            render_delay_ms: 100.0,
        };
        let net_stats = NetworkDebugStats {
            rtt_ms: 30.0,
            pending_inputs: 5,
            interp: Some(interp),
        };
        assert!(net_stats.interp.is_some());
        assert_eq!(net_stats.interp.as_ref().unwrap().player_buffers, 2);
    }

    #[test]
    fn test_network_debug_stats_clone() {
        let net_stats = NetworkDebugStats {
            rtt_ms: 20.0,
            pending_inputs: 2,
            interp: None,
        };
        let cloned = net_stats.clone();
        assert!((cloned.rtt_ms - 20.0).abs() < f32::EPSILON);
    }

    // === DebugStats 测试 ===

    #[test]
    fn test_debug_stats_creation() {
        let stats = DebugStats {
            fps: 60.0,
            entity_count: 150,
            quadtree_depth: 4,
            particle_count: 200,
            network: None,
        };
        assert!((stats.fps - 60.0).abs() < f32::EPSILON);
        assert_eq!(stats.entity_count, 150);
        assert_eq!(stats.quadtree_depth, 4);
        assert_eq!(stats.particle_count, 200);
        assert!(stats.network.is_none());
    }

    #[test]
    fn test_debug_stats_with_network() {
        let net = NetworkDebugStats {
            rtt_ms: 15.0,
            pending_inputs: 1,
            interp: None,
        };
        let stats = DebugStats {
            fps: 55.0,
            entity_count: 100,
            quadtree_depth: 3,
            particle_count: 50,
            network: Some(net),
        };
        assert!(stats.network.is_some());
    }

    // === ActiveBuff 测试 ===

    #[test]
    fn test_active_buff_creation() {
        let buff = ActiveBuff {
            name: "Shield",
            icon_char: "S".to_string(),
            color: Color::new(0.2, 0.6, 1.0, 1.0),
            remaining: 5.5,
            max_duration: 10.0,
        };
        assert_eq!(buff.name, "Shield");
        assert_eq!(buff.icon_char, "S");
        assert!((buff.remaining - 5.5).abs() < f64::EPSILON);
        assert!((buff.max_duration - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_active_buff_progress_calculation() {
        let buff = ActiveBuff {
            name: "Rapid Fire",
            icon_char: "R".to_string(),
            color: Color::new(1.0, 0.9, 0.2, 1.0),
            remaining: 3.0,
            max_duration: 6.0,
        };
        let progress = buff.remaining / buff.max_duration;
        assert!((progress - 0.5).abs() < f64::EPSILON);
    }

    // === ChallengeOfferAction 测试 ===

    #[test]
    fn test_challenge_offer_action_variants() {
        let action_none = ChallengeOfferAction::None;
        let action_accept = ChallengeOfferAction::Accept;
        let action_skip = ChallengeOfferAction::Skip;

        // 使用 match 确认变体正确
        match action_none {
            ChallengeOfferAction::None => (),
            _ => panic!("Expected None"),
        }
        match action_accept {
            ChallengeOfferAction::Accept => (),
            _ => panic!("Expected Accept"),
        }
        match action_skip {
            ChallengeOfferAction::Skip => (),
            _ => panic!("Expected Skip"),
        }
    }

    // === ShopUiAction 测试 ===

    #[test]
    fn test_shop_ui_action_none() {
        let action = ShopUiAction::None;
        match action {
            ShopUiAction::None => (),
            _ => panic!("Expected None"),
        }
    }

    #[test]
    fn test_shop_ui_action_buy_confirmed() {
        let action = ShopUiAction::BuyConfirmed(2);
        match action {
            ShopUiAction::BuyConfirmed(idx) => assert_eq!(idx, 2),
            _ => panic!("Expected BuyConfirmed"),
        }
    }

    #[test]
    fn test_shop_ui_action_refresh() {
        let action = ShopUiAction::RefreshRequested;
        match action {
            ShopUiAction::RefreshRequested => (),
            _ => panic!("Expected RefreshRequested"),
        }
    }

    #[test]
    fn test_shop_ui_action_exit() {
        let action = ShopUiAction::ExitShop;
        match action {
            ShopUiAction::ExitShop => (),
            _ => panic!("Expected ExitShop"),
        }
    }

    // === RestUiAction 测试 ===

    #[test]
    fn test_rest_ui_action_variants() {
        assert_eq!(RestUiAction::None, RestUiAction::None);
        assert_eq!(RestUiAction::SelectOption(1), RestUiAction::SelectOption(1));
        assert_ne!(RestUiAction::SelectOption(1), RestUiAction::SelectOption(2));
        assert_eq!(RestUiAction::ConfirmRest, RestUiAction::ConfirmRest);
    }

    #[test]
    fn test_rest_ui_action_clone() {
        let action = RestUiAction::SelectOption(3);
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_rest_ui_action_copy() {
        let action = RestUiAction::ConfirmRest;
        let copied: RestUiAction = action; // Copy trait
        assert_eq!(action, copied);
    }

    #[test]
    fn test_rest_ui_action_debug() {
        let action = RestUiAction::SelectCard(5);
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("SelectCard"));
        assert!(debug_str.contains("5"));
    }
}
