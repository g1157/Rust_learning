//! 菜单 UI 组件模块
//!
//! 从 ui.rs 拆分出来的菜单相关渲染。
//!
//! ## 功能
//! - 模式选择菜单
//! - 暂停菜单
//! - 等待屏幕
//! - 游戏结束屏幕
//! - 在线大厅 UI

use macroquad::prelude::*;
use macroquad::text::Font;

use crate::achievement::AchievementManager;
use crate::background::Starfield;
use crate::game_state::{GameMode, GameSettings, PauseSelection, TimeAttackDuration};
use crate::network::NetworkClient;
use crate::ui_common::{draw_shadow_panel, draw_text_centered};

// ============================================================================
// 模式选择
// ============================================================================

/// 模式信息
struct ModeInfo {
    name: &'static str,
    description: &'static str,
    color: Color,
}

fn get_mode_info(mode: GameMode) -> ModeInfo {
    match mode {
        GameMode::Survival => ModeInfo {
            name: "SURVIVAL",
            description: "Destroy asteroids, survive as long as you can",
            color: Color::new(0.3, 0.8, 0.4, 1.0),
        },
        GameMode::Duel => ModeInfo {
            name: "DUEL",
            description: "Capture the flag to win rounds",
            color: Color::new(0.8, 0.3, 0.3, 1.0),
        },
        GameMode::TimeAttack => ModeInfo {
            name: "TIME ATTACK",
            description: "Score as many points as you can before time runs out",
            color: Color::new(0.9, 0.7, 0.2, 1.0),
        },
        GameMode::Roguelike => ModeInfo {
            name: "ROGUELIKE",
            description: "Procedural run with upgrades and challenges",
            color: Color::new(0.6, 0.4, 0.9, 1.0),
        },
        GameMode::Online => ModeInfo {
            name: "ONLINE",
            description: "Multiplayer mode via network",
            color: Color::new(0.4, 0.7, 0.9, 1.0),
        },
        GameMode::Achievements => ModeInfo {
            name: "ACHIEVEMENTS",
            description: "View your progress and unlocked achievements",
            color: Color::new(0.9, 0.8, 0.3, 1.0),
        },
        GameMode::Settings => ModeInfo {
            name: "SETTINGS",
            description: "Configure game options",
            color: Color::new(0.6, 0.6, 0.7, 1.0),
        },
    }
}

/// 绘制模式选择界面
pub fn draw_mode_selection(
    selection: GameMode,
    settings: &GameSettings,
    achievements: &AchievementManager,
    time_attack_duration: TimeAttackDuration,
    online_available: bool,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    starfield.draw(time);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // 标题
    draw_text_centered(
        "ASTEROIDS",
        screen_h * 0.15,
        48,
        Color::new(0.9, 0.92, 0.95, 1.0),
        font,
    );

    // 模式列表
    let modes = if online_available {
        vec![
            GameMode::Survival,
            GameMode::Duel,
            GameMode::TimeAttack,
            GameMode::Roguelike,
            GameMode::Online,
            GameMode::Achievements,
            GameMode::Settings,
        ]
    } else {
        vec![
            GameMode::Survival,
            GameMode::Duel,
            GameMode::TimeAttack,
            GameMode::Roguelike,
            GameMode::Achievements,
            GameMode::Settings,
        ]
    };

    let card_height = 60.;
    let card_spacing = 12.;
    let card_width = screen_w * 0.5;
    let start_y = screen_h * 0.28;

    for (i, mode) in modes.iter().enumerate() {
        let info = get_mode_info(*mode);
        let y = start_y + i as f32 * (card_height + card_spacing);
        let x = screen_w / 2. - card_width / 2.;

        let is_selected = *mode == selection;
        let bg_color = if is_selected {
            Color::new(info.color.r * 0.3, info.color.g * 0.3, info.color.b * 0.3, 0.9)
        } else {
            Color::new(0.1, 0.12, 0.15, 0.8)
        };

        // 卡片背景
        draw_shadow_panel(x, y, card_width, card_height, bg_color);

        // 选中边框
        if is_selected {
            draw_rectangle_lines(x, y, card_width, card_height, 2.0, info.color);
        }

        // 模式名称
        draw_text_ex(
            info.name,
            x + 20.,
            y + 28.,
            TextParams {
                font_size: 24,
                color: if is_selected { info.color } else { WHITE },
                font,
                ..Default::default()
            },
        );

        // 描述
        draw_text_ex(
            info.description,
            x + 20.,
            y + 48.,
            TextParams {
                font_size: 14,
                color: Color::new(0.6, 0.65, 0.7, 1.0),
                font,
                ..Default::default()
            },
        );

        // 额外信息（如玩家数量、时长）
        if is_selected {
            match *mode {
                GameMode::Survival => {
                    let player_text = format!("Players: {}", settings.player_count.name());
                    draw_text_ex(
                        &player_text,
                        x + card_width - 150.,
                        y + 35.,
                        TextParams {
                            font_size: 16,
                            color: YELLOW,
                            font,
                            ..Default::default()
                        },
                    );
                }
                GameMode::TimeAttack => {
                    let duration_text = format!("Duration: {}", time_attack_duration.name());
                    draw_text_ex(
                        &duration_text,
                        x + card_width - 180.,
                        y + 35.,
                        TextParams {
                            font_size: 16,
                            color: YELLOW,
                            font,
                            ..Default::default()
                        },
                    );
                }
                _ => {}
            }
        }
    }

    // 底部提示
    let hint = "↑↓ Select  ←→ Options  [Enter] Start  [Esc] Back";
    draw_text_centered(
        hint,
        screen_h - 40.,
        16,
        Color::new(0.5, 0.55, 0.6, 1.0),
        font,
    );
}

// ============================================================================
// 暂停菜单
// ============================================================================

/// 绘制暂停菜单
pub fn draw_pause_menu(selection: PauseSelection, font: Option<&Font>) {
    // 全屏半透明遮罩
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    let panel_w = 400.;
    let panel_h = 200.;
    let panel_x = screen_width() / 2. - panel_w / 2.;
    let panel_y = screen_height() / 2. - panel_h / 2.;

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        Color::new(0.1, 0.12, 0.15, 0.95),
    );

    // 标题
    draw_text_centered(
        "PAUSED",
        panel_y + 50.,
        36,
        WHITE,
        font,
    );

    // 选项
    let options = [
        (PauseSelection::Resume, "Resume"),
        (PauseSelection::ModeSelect, "Mode Select"),
    ];

    for (i, (opt, text)) in options.iter().enumerate() {
        let y = panel_y + 100. + i as f32 * 40.;
        let is_selected = *opt == selection;

        let color = if is_selected { YELLOW } else { WHITE };
        let prefix = if is_selected { "> " } else { "  " };

        draw_text_centered(
            &format!("{}{}", prefix, text),
            y,
            24,
            color,
            font,
        );
    }

    // 提示
    draw_text_centered(
        "↑↓ Select  [Enter] Confirm",
        panel_y + panel_h - 20.,
        14,
        Color::new(0.5, 0.55, 0.6, 1.0),
        font,
    );
}

// ============================================================================
// 等待和游戏结束屏幕
// ============================================================================

/// 绘制等待屏幕
pub fn draw_waiting_screen(message: &str, font: Option<&Font>, starfield: &Starfield, time: f32) {
    starfield.draw(time);

    let panel_width = screen_width() * 0.6;
    let panel_height = 160.;
    let panel_x = screen_width() / 2. - panel_width / 2.;
    let panel_y = screen_height() / 2. - panel_height / 2.;

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.08, 0.1, 0.15, 0.9),
    );

    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        2.0,
        Color::new(0.4, 0.6, 0.9, 0.7),
    );

    draw_text_centered(
        message,
        screen_height() / 2. + 8.,
        32,
        Color::new(0.9, 0.92, 0.95, 1.0),
        font,
    );
}

/// 绘制游戏结束消息
pub fn draw_game_over_message(
    message: &str,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    starfield.draw(time);

    let banner_width = screen_width() * 0.6;
    let banner_height = 130.;
    let banner_x = screen_width() / 2. - banner_width / 2.;
    let banner_y = screen_height() * 0.35;

    draw_shadow_panel(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        Color::new(0.08, 0.1, 0.15, 0.9),
    );

    draw_rectangle_lines(
        banner_x,
        banner_y,
        banner_width,
        banner_height,
        2.0,
        Color::new(0.6, 0.3, 0.3, 0.8),
    );

    draw_text_centered(
        "GAME OVER",
        banner_y + 45.,
        36,
        Color::new(0.9, 0.3, 0.3, 1.0),
        font,
    );

    draw_text_centered(
        message,
        banner_y + 90.,
        20,
        Color::new(0.8, 0.82, 0.85, 1.0),
        font,
    );
}

/// 绘制胜利暂停覆盖层
pub fn draw_victory_pause_overlay(remaining: f64, font: Option<&Font>) {
    let text = format!("Wave cleared! Next wave in {:.1}s", remaining);

    draw_text_centered(
        &text,
        screen_height() / 2.,
        28,
        Color::new(0.3, 0.9, 0.3, 1.0),
        font,
    );
}

// ============================================================================
// 在线模式 UI
// ============================================================================

/// 绘制在线大厅
pub fn draw_online_lobby(
    nickname: &str,
    nickname_input: bool,
    network_client: &NetworkClient,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    starfield.draw(time);

    let panel_w = 500.;
    let panel_h = 300.;
    let panel_x = screen_width() / 2. - panel_w / 2.;
    let panel_y = screen_height() / 2. - panel_h / 2.;

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        Color::new(0.08, 0.1, 0.15, 0.9),
    );

    // 标题
    draw_text_centered(
        "ONLINE LOBBY",
        panel_y + 50.,
        32,
        WHITE,
        font,
    );

    // 连接状态
    let status = if network_client.is_connected() {
        ("Connected", GREEN)
    } else {
        ("Disconnected", RED)
    };

    draw_text_ex(
        &format!("Status: {}", status.0),
        panel_x + 30.,
        panel_y + 100.,
        TextParams {
            font_size: 20,
            color: status.1,
            font,
            ..Default::default()
        },
    );

    // 昵称输入
    let nickname_display = if nickname.is_empty() {
        "Enter nickname..."
    } else {
        nickname
    };

    draw_text_ex(
        "Nickname:",
        panel_x + 30.,
        panel_y + 150.,
        TextParams {
            font_size: 18,
            color: WHITE,
            font,
            ..Default::default()
        },
    );

    let input_color = if nickname_input {
        Color::new(0.2, 0.3, 0.4, 1.0)
    } else {
        Color::new(0.15, 0.18, 0.22, 1.0)
    };

    draw_rectangle(panel_x + 120., panel_y + 130., 200., 30., input_color);
    draw_rectangle_lines(
        panel_x + 120.,
        panel_y + 130.,
        200.,
        30.,
        1.0,
        Color::new(0.4, 0.5, 0.6, 1.0),
    );

    draw_text_ex(
        nickname_display,
        panel_x + 130.,
        panel_y + 152.,
        TextParams {
            font_size: 18,
            color: if nickname.is_empty() {
                Color::new(0.5, 0.5, 0.5, 1.0)
            } else {
                WHITE
            },
            font,
            ..Default::default()
        },
    );

    // 提示
    draw_text_centered(
        "[Enter] Join Queue  [Esc] Back",
        panel_y + panel_h - 30.,
        16,
        Color::new(0.5, 0.55, 0.6, 1.0),
        font,
    );
}

/// 绘制在线等待
pub fn draw_online_waiting(
    room_id: u32,
    network_client: &NetworkClient,
    font: Option<&Font>,
    starfield: &Starfield,
    time: f32,
) {
    starfield.draw(time);

    let panel_w = 400.;
    let panel_h = 200.;
    let panel_x = screen_width() / 2. - panel_w / 2.;
    let panel_y = screen_height() / 2. - panel_h / 2.;

    draw_shadow_panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        Color::new(0.08, 0.1, 0.15, 0.9),
    );

    draw_text_centered(
        "SEARCHING FOR MATCH...",
        panel_y + 60.,
        28,
        WHITE,
        font,
    );

    // 动态点点点
    let dots = ".".repeat(((time * 2.0) as usize % 4) + 1);
    draw_text_centered(
        &dots,
        panel_y + 100.,
        24,
        YELLOW,
        font,
    );

    draw_text_centered(
        "[Esc] Cancel",
        panel_y + panel_h - 30.,
        16,
        Color::new(0.5, 0.55, 0.6, 1.0),
        font,
    );
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mode_info() {
        let info = get_mode_info(GameMode::Survival);
        assert_eq!(info.name, "SURVIVAL");
    }
}
