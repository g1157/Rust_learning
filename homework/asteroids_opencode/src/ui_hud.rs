//! HUD 组件模块
//!
//! 从 ui.rs 拆分出来的抬头显示组件。
//!
//! ## 功能
//! - 玩家状态 HUD
//! - 生存记录显示
//! - 限时挑战 HUD
//! - 连击计数显示
//! - 慢动作指示器
//! - 调试面板

use macroquad::prelude::*;
use macroquad::text::Font;

use crate::bullet::WeaponType;
use crate::constants::killstreak as ks_config;
use crate::interpolation::InterpolationManager;
use crate::network::NetworkClient;
use crate::player::Player;

// ============================================================================
// HUD 模式
// ============================================================================

/// HUD 显示模式
pub enum HudMode {
    Waiting,
    Active { time: f64 },
}

// ============================================================================
// 调试统计
// ============================================================================

/// 调试统计数据
pub struct DebugStats {
    pub fps: i32,
    pub entity_count: usize,
    pub quadtree_depth: usize,
    pub asteroid_count: usize,
    pub bullet_count: usize,
    pub particle_count: usize,
}

impl DebugStats {
    pub fn new() -> Self {
        Self {
            fps: 0,
            entity_count: 0,
            quadtree_depth: 0,
            asteroid_count: 0,
            bullet_count: 0,
            particle_count: 0,
        }
    }
}

impl Default for DebugStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 网络调试统计
pub struct NetworkDebugStats {
    pub rtt_ms: f64,
    pub pending_inputs: usize,
    pub buffer_size: usize,
}

/// 插值调试统计
pub struct InterpDebugStats {
    pub local_delay_ms: f64,
    pub remote_delay_ms: f64,
    pub buffer_entries: usize,
}

// ============================================================================
// HUD 渲染函数
// ============================================================================

/// 绘制玩家 HUD
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

/// 绘制生存记录
pub fn draw_survival_record(record: u32, font: Option<&Font>) {
    let text = format!("High Score: {}", record);
    let x = screen_width() - 200.;
    let y = 32.;
    draw_text_ex(
        &text,
        x,
        y,
        TextParams {
            font_size: 24,
            color: GOLD,
            font,
            ..Default::default()
        },
    );
}

/// 绘制限时挑战 HUD
pub fn draw_time_attack_hud(time_left: f64, frenzy_active: bool, score: u32, font: Option<&Font>) {
    let color = if frenzy_active { RED } else { WHITE };
    let time_text = format!("Time: {:.1}s", time_left);
    let score_text = format!("Score: {}", score);

    // 时间显示
    draw_text_ex(
        &time_text,
        screen_width() / 2. - 60.,
        60.,
        TextParams {
            font_size: 36,
            color,
            font,
            ..Default::default()
        },
    );

    // 分数显示
    draw_text_ex(
        &score_text,
        screen_width() / 2. - 60.,
        100.,
        TextParams {
            font_size: 28,
            color: WHITE,
            font,
            ..Default::default()
        },
    );

    // 狂暴提示
    if frenzy_active {
        draw_text_ex(
            "FRENZY!",
            screen_width() / 2. - 50.,
            140.,
            TextParams {
                font_size: 24,
                color: ORANGE,
                font,
                ..Default::default()
            },
        );
    }
}

/// 绘制连击计数（屏幕右侧）
pub fn draw_killstreak_counter(players: &[Player], time: f64, font: Option<&Font>) {
    for (idx, player) in players.iter().enumerate() {
        if player.killstreak < 2 {
            continue;
        }

        let streak = player.killstreak;
        let (level_name, level_color) = get_streak_level(streak);

        let text = format!("{} x{}", level_name, streak);
        let x = screen_width() - 150.;
        let y = 80. + idx as f32 * 50.;

        // 背景面板
        let text_width = 120.;
        draw_rectangle(x - 10., y - 25., text_width + 20., 35., Color::new(0.0, 0.0, 0.0, 0.5));

        draw_text_ex(
            &text,
            x,
            y,
            TextParams {
                font_size: 28,
                color: level_color,
                font,
                ..Default::default()
            },
        );

        // 倍数显示
        let multiplier = 1.0 + (streak as f32 - 1.0) * ks_config::SCORE_MULTIPLIER_PER_KILL;
        let mult_text = format!("x{:.1}", multiplier.min(ks_config::MAX_SCORE_MULTIPLIER));
        draw_text_ex(
            &mult_text,
            x + 80.,
            y,
            TextParams {
                font_size: 20,
                color: GOLD,
                font,
                ..Default::default()
            },
        );
    }
}

/// 获取连击等级名称和颜色
fn get_streak_level(streak: u32) -> (&'static str, Color) {
    if streak >= 15 {
        ("GODLIKE", Color::new(1.0, 0.2, 0.8, 1.0))
    } else if streak >= 10 {
        ("UNSTOPPABLE", Color::new(1.0, 0.5, 0.0, 1.0))
    } else if streak >= 5 {
        ("MEGA KILL", Color::new(1.0, 0.8, 0.0, 1.0))
    } else if streak >= 3 {
        ("TRIPLE", Color::new(0.0, 1.0, 0.5, 1.0))
    } else {
        ("DOUBLE", Color::new(0.5, 1.0, 0.5, 1.0))
    }
}

/// 绘制慢动作指示器
pub fn draw_slow_motion_indicator(time_scale: f32, font: Option<&Font>) {
    if time_scale >= 1.0 {
        return;
    }

    let text = format!("SLOW-MO x{:.1}", time_scale);
    let x = screen_width() / 2. - 60.;
    let y = screen_height() - 40.;

    // 脉冲效果
    let pulse = (get_time() * 4.0).sin() as f32 * 0.3 + 0.7;
    let color = Color::new(0.5, 0.8, 1.0, pulse);

    draw_text_ex(
        &text,
        x,
        y,
        TextParams {
            font_size: 24,
            color,
            font,
            ..Default::default()
        },
    );
}

/// 绘制调试面板
pub fn draw_debug_panel(stats: &DebugStats, font: Option<&Font>) {
    let x = 12.;
    let y = screen_height() - 130.;
    let width = 280.;
    let height = 115.;

    // 背景
    draw_rectangle(x, y, width, height, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.3, 0.5, 0.7, 0.8));

    let lines = [
        format!("FPS: {}", stats.fps),
        format!("Entities: {}", stats.entity_count),
        format!("Asteroids: {}", stats.asteroid_count),
        format!("Bullets: {}", stats.bullet_count),
        format!("Particles: {}", stats.particle_count),
    ];

    for (i, line) in lines.iter().enumerate() {
        draw_text_ex(
            line,
            x + 10.,
            y + 22. + i as f32 * 20.,
            TextParams {
                font_size: 16,
                color: Color::new(0.7, 0.9, 0.7, 1.0),
                font,
                ..Default::default()
            },
        );
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_stats_new() {
        let stats = DebugStats::new();
        assert_eq!(stats.fps, 0);
        assert_eq!(stats.entity_count, 0);
    }

    #[test]
    fn test_streak_level() {
        assert_eq!(get_streak_level(2).0, "DOUBLE");
        assert_eq!(get_streak_level(3).0, "TRIPLE");
        assert_eq!(get_streak_level(5).0, "MEGA KILL");
        assert_eq!(get_streak_level(10).0, "UNSTOPPABLE");
        assert_eq!(get_streak_level(15).0, "GODLIKE");
    }
}
