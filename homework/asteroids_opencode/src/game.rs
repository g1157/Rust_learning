//! 游戏核心逻辑模块
//!
//! 从 main.rs 拆分出来的游戏初始化、重置和辅助函数。
//!
//! ## 主要功能
//! - 玩家初始化和重置
//! - 波次生成
//! - 分数计算
//! - 成就更新
//!
//! 注意：部分函数为未来重构准备，暂时允许 dead_code

#![allow(dead_code)]

use macroquad::prelude::*;

use crate::achievement::{AchievementId, AchievementManager};
use crate::asteroid::{Asteroid, spawn_wave_with_speed_and_wave};
use crate::constants::{difficulty, gameplay};
use crate::game_state::{GameMode, PlayerCount};
use crate::player::{Controls, Player};

// ============================================================================
// 常量
// ============================================================================

/// 初始小行星数量
pub const ASTEROID_COUNT: usize = gameplay::INITIAL_ASTEROID_COUNT;
/// 每波增加的小行星数量
pub const ASTEROID_WAVE_INCREMENT: usize = gameplay::ASTEROID_WAVE_INCREMENT;

// ============================================================================
// 玩家初始化
// ============================================================================

/// 初始化玩家列表
pub fn init_players(now: f64, starting_lives: u32, player_count: PlayerCount) -> Vec<Player> {
    let positions = player_start_positions(player_count);
    let mut players = vec![Player::new(
        "Player 1",
        BLUE,
        positions[0],
        Controls {
            thrust: KeyCode::W,
            left: KeyCode::A,
            right: KeyCode::D,
            shoot_primary: KeyCode::J,
            shoot_alt: Some(KeyCode::F),
            weapon_switch: KeyCode::U,
            weapon_switch_alt: None,
            dash: KeyCode::Space,
            hyperspace: KeyCode::H,
            phase_dash: KeyCode::E,
        },
        now,
        starting_lives,
    )];

    if player_count == PlayerCount::Two {
        players.push(Player::new(
            "Player 2",
            RED,
            positions[1],
            Controls {
                thrust: KeyCode::Up,
                left: KeyCode::Left,
                right: KeyCode::Right,
                shoot_primary: KeyCode::Key1,
                shoot_alt: Some(KeyCode::Kp1),
                weapon_switch: KeyCode::Key4,
                weapon_switch_alt: Some(KeyCode::Kp4),
                dash: KeyCode::Kp0,
                hyperspace: KeyCode::KpEnter,
                phase_dash: KeyCode::KpDecimal,
            },
            now,
            starting_lives,
        ));
    }

    players
}

/// 重置玩家状态
pub fn reset_players(
    players: &mut [Player],
    now: f64,
    starting_lives: u32,
    player_count: PlayerCount,
) {
    let positions = player_start_positions(player_count);
    for (player, position) in players.iter_mut().zip(positions.iter()) {
        player.reset(*position, now, starting_lives);
    }
}

/// 获取玩家初始位置
pub fn player_start_positions(player_count: PlayerCount) -> [Vec2; 2] {
    let center_y = screen_height() / 2.;
    let width = screen_width();
    match player_count {
        PlayerCount::One => [
            Vec2::new(width / 2., center_y),
            Vec2::new(width / 2., center_y),
        ],
        PlayerCount::Two => [
            Vec2::new(width * 0.25, center_y),
            Vec2::new(width * 0.75, center_y),
        ],
    }
}

// ============================================================================
// 波次生成
// ============================================================================

/// 生成生存模式波次
pub fn spawn_survival_wave(asteroids: &mut Vec<Asteroid>, wave: u32, player_count: PlayerCount) {
    let screen_center = Vec2::new(screen_width() / 2., screen_height() / 2.);
    let wave_index = wave.saturating_sub(1) as usize;

    // 单人模式：小行星数量减少 30%
    let base_count = match player_count {
        PlayerCount::One => ((ASTEROID_COUNT as f32) * 0.7) as usize,
        PlayerCount::Two => ASTEROID_COUNT,
    };
    let increment = match player_count {
        PlayerCount::One => ((ASTEROID_WAVE_INCREMENT as f32) * 0.8) as usize,
        PlayerCount::Two => ASTEROID_WAVE_INCREMENT,
    };

    // === 波动式难度系统 ===
    let cycle = wave_index as u32 / difficulty::WAVE_CYCLE;
    let position_in_cycle = wave_index as u32 % difficulty::WAVE_CYCLE;

    // 基础难度随周期增长
    let base_difficulty = 1.0 + cycle as f32 * difficulty::BASE_DIFFICULTY_GROWTH;

    // 周期内的难度波动
    let cycle_modifier = difficulty::CYCLE_MULTIPLIERS[position_in_cycle as usize];

    // 最终难度倍数
    let difficulty_multiplier =
        (base_difficulty * cycle_modifier).min(difficulty::MAX_DIFFICULTY_MULTIPLIER);

    // 应用难度倍数到小行星数量
    let base_asteroid_count = base_count + wave_index * increment;
    let asteroid_count = ((base_asteroid_count as f32) * difficulty_multiplier) as usize;

    // 速度也受难度影响
    let speed_multiplier = (1.0 + wave_index as f32 * gameplay::WAVE_SPEED_INCREMENT * 0.7)
        .min(gameplay::WAVE_SPEED_MAX_MULTIPLIER)
        * (0.9 + cycle_modifier * 0.1);

    asteroids.extend(spawn_wave_with_speed_and_wave(
        screen_center,
        screen_width().min(screen_height()),
        asteroid_count,
        speed_multiplier,
        wave.max(1),
    ));
}

// ============================================================================
// 分数计算
// ============================================================================

/// 计算所有玩家的总分
pub fn total_survival_score(players: &[Player]) -> u32 {
    players.iter().map(|player| player.score.value()).sum()
}

/// 结算生存模式
pub fn finalize_survival(players: &mut [Player], time: f64) {
    for player in players.iter_mut() {
        player.finalize_survival(time);
    }
}

// ============================================================================
// 成就更新
// ============================================================================

/// 更新成就进度
pub fn update_achievements(
    achievements: &mut AchievementManager,
    players: &[Player],
    current_mode: GameMode,
    frame_t: f64,
    survival_wave: u32,
) {
    // 检查连击成就
    for player in players {
        let streak = player.killstreak;

        if streak >= 2 {
            achievements.update_progress(AchievementId::DoubleTrouble, streak.max(2), frame_t);
        }
        if streak >= 3 {
            achievements.update_progress(AchievementId::TripleThreat, streak.max(3), frame_t);
        }
        if streak >= 5 {
            achievements.update_progress(AchievementId::MegaKiller, streak.max(5), frame_t);
            if streak == 5 {
                achievements.stats.five_streaks = achievements.stats.five_streaks.saturating_add(1);
            }
        }
        if streak >= 10 {
            achievements.update_progress(AchievementId::Unstoppable, streak.max(10), frame_t);
        }
        if streak >= 15 {
            achievements.update_progress(AchievementId::Godlike, streak.max(15), frame_t);
        }

        achievements.stats.max_killstreak = achievements.stats.max_killstreak.max(streak);

        let session_kills = player.score.value() / 10;
        if session_kills >= 1 {
            achievements.update_progress(AchievementId::FirstBlood, 1, frame_t);
        }
        if session_kills >= 10 {
            achievements.update_progress(
                AchievementId::Marksman,
                achievements.stats.total_kills,
                frame_t,
            );
        }
    }

    // 检查累计击杀数
    if achievements.stats.total_kills >= 500 {
        achievements.update_progress(
            AchievementId::Deadeye,
            achievements.stats.total_kills,
            frame_t,
        );
    }

    // 检查子弹发射数
    if achievements.stats.bullets_fired >= 100 {
        achievements.update_progress(
            AchievementId::Armed,
            achievements.stats.bullets_fired,
            frame_t,
        );
    }

    // 检查护盾拾取数
    if achievements.stats.shields_collected >= 1 {
        achievements.update_progress(AchievementId::Protected, 1, frame_t);
    }
    if achievements.stats.shields_collected >= 20 {
        achievements.update_progress(
            AchievementId::ShieldMaster,
            achievements.stats.shields_collected,
            frame_t,
        );
    }

    // 检查生存模式相关成就
    if matches!(current_mode, GameMode::Survival) {
        let survival_score = total_survival_score(players);

        if survival_score >= 1000 {
            achievements.update_progress(AchievementId::Century, survival_score, frame_t);
        }
        if survival_score >= 5000 {
            achievements.update_progress(AchievementId::Champion, survival_score, frame_t);
        }

        if survival_wave >= 3 {
            achievements.update_progress(AchievementId::WaveRider, survival_wave, frame_t);
        }
        if survival_wave >= 5 {
            achievements.update_progress(AchievementId::WaveMaster, survival_wave, frame_t);
        }
        if survival_wave >= 10 {
            achievements.update_progress(AchievementId::WaveGod, survival_wave, frame_t);
        }

        achievements.stats.max_wave = achievements.stats.max_wave.max(survival_wave);
    }

    // 检查累计时间成就
    if achievements.stats.total_playtime >= 1800.0 {
        achievements.update_progress(
            AchievementId::Veteran,
            achievements.stats.total_playtime as u32,
            frame_t,
        );
    }
    if achievements.stats.total_playtime >= 7200.0 {
        achievements.update_progress(
            AchievementId::Legend,
            achievements.stats.total_playtime as u32,
            frame_t,
        );
    }

    // 检查模式探索成就
    if achievements.stats.modes_played.len() >= 2 {
        achievements.update_progress(
            AchievementId::Adventurer,
            achievements.stats.modes_played.len() as u32,
            frame_t,
        );
    }

    // 检查武器使用成就
    if achievements.stats.weapons_used.len() >= 3 {
        achievements.update_progress(
            AchievementId::Arsenal,
            achievements.stats.weapons_used.len() as u32,
            frame_t,
        );
    }

    // 检查设置修改成就
    if achievements.stats.settings_changed >= 5 {
        achievements.update_progress(
            AchievementId::Tinkerer,
            achievements.stats.settings_changed,
            frame_t,
        );
    }

    // 检查对战模式成就
    if matches!(current_mode, GameMode::Duel) {
        if achievements.stats.duel_games >= 1 {
            achievements.update_progress(AchievementId::Warrior, 1, frame_t);
        }
        if achievements.stats.duel_wins >= 5 {
            achievements.update_progress(
                AchievementId::Duelist,
                achievements.stats.duel_wins,
                frame_t,
            );
        }
    }

    // 检查连击大师成就
    if achievements.stats.five_streaks >= 3 {
        achievements.unlock(AchievementId::ComboMaster, frame_t);
    }

    // 清理过期的解锁提示
    achievements.cleanup_recent_unlocks(6.0, frame_t);
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：player_start_positions 依赖 screen_width/screen_height，
    // 这些函数需要 macroquad 上下文，无法在单元测试中直接测试。
    // 这些测试已移至集成测试或手动测试。

    #[test]
    fn test_total_survival_score_empty() {
        let players: Vec<Player> = vec![];
        assert_eq!(total_survival_score(&players), 0);
    }

    #[test]
    fn test_asteroid_count_constants() {
        assert!(ASTEROID_COUNT > 0);
        assert!(ASTEROID_WAVE_INCREMENT > 0);
    }
}
