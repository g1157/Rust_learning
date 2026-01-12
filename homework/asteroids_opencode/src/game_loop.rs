//! 游戏循环核心模块
//!
//! 从 main.rs 拆分出来的游戏循环辅助结构和函数。
//!
//! ## 功能
//! - RoundState: 回合状态聚合结构
//! - 回合初始化和重置
//! - 游戏模式初始化

use macroquad::prelude::*;

use crate::asteroid::{Asteroid, spawn_wave_with_speed};
use crate::constants::{gameplay, timing};
use crate::duel::DuelState;
use crate::game::{ASTEROID_COUNT, init_players, reset_players};
use crate::game_state::{GameMode, PlayerCount};
use crate::player::Player;
use crate::powerup::{self, PowerUp};
use crate::ufo::{EnemyBullet, Ufo};
use crate::vortex::VortexManager;

// ============================================================================
// 回合状态
// ============================================================================

/// 回合状态聚合 - 用于传递可变引用
pub struct RoundState<'a> {
    pub players: &'a mut Vec<Player>,
    pub asteroids: &'a mut Vec<Asteroid>,
    pub ufos: &'a mut Vec<Ufo>,
    pub enemy_bullets: &'a mut Vec<EnemyBullet>,
    pub powerups: &'a mut Vec<PowerUp>,
    pub next_shield_spawn: &'a mut f64,
    pub next_weapon_spawn: &'a mut f64,
    pub duel_state: &'a mut DuelState,
    pub survival_wave: &'a mut u32,
    pub next_ufo_wave: &'a mut u32,
    pub first_ufo_spawned: &'a mut bool,
    pub vortex_manager: &'a mut VortexManager,
}

// ============================================================================
// 回合初始化
// ============================================================================

/// 初始化新回合
pub fn start_round(
    state: RoundState<'_>,
    now: f64,
    mode: GameMode,
    starting_lives: u32,
    player_count: PlayerCount,
) {
    // 重置玩家
    reset_players(state.players, now, starting_lives, player_count);

    // 清空实体
    state.asteroids.clear();
    state.ufos.clear();
    state.enemy_bullets.clear();
    state.powerups.clear();

    // 重置道具生成计时
    *state.next_shield_spawn = powerup::schedule_next_spawn(now, player_count);
    *state.next_weapon_spawn = powerup::schedule_next_weapon_spawn(now, player_count);

    // 重置漩涡
    state.vortex_manager.reset();

    // 重置 UFO 状态
    *state.next_ufo_wave = 3;
    *state.first_ufo_spawned = false;

    match mode {
        GameMode::Survival | GameMode::TimeAttack => {
            // 生成初始小行星波次
            *state.survival_wave = 1;
            let asteroid_count = match player_count {
                PlayerCount::One => ((ASTEROID_COUNT as f32) * 0.7) as usize,
                PlayerCount::Two => ASTEROID_COUNT,
            };
            state.asteroids.extend(spawn_wave_with_speed(
                Vec2::new(screen_width() / 2., screen_height() / 2.),
                screen_width().min(screen_height()),
                asteroid_count,
                1.0,
            ));
        }
        GameMode::Duel => {
            // 对战模式：初始化对战状态
            state.duel_state.reset(now);
            state.duel_state.start_new_round(now);
        }
        _ => {}
    }
}

/// 生成下一波次（用于 VictoryPause 后）
pub fn spawn_next_wave(
    asteroids: &mut Vec<Asteroid>,
    ufos: &mut Vec<Ufo>,
    survival_wave: &mut u32,
    next_ufo_wave: &mut u32,
    first_ufo_spawned: &mut bool,
    player_count: PlayerCount,
    asteroid_speed_multiplier: f32,
    now: f64,
) {
    *survival_wave += 1;
    let wave_index = survival_wave.saturating_sub(1) as usize;

    // 计算小行星数量
    let base_count = match player_count {
        PlayerCount::One => ((ASTEROID_COUNT as f32) * 0.7) as usize,
        PlayerCount::Two => ASTEROID_COUNT,
    };
    let increment = match player_count {
        PlayerCount::One => ((gameplay::ASTEROID_WAVE_INCREMENT as f32) * 0.8) as usize,
        PlayerCount::Two => gameplay::ASTEROID_WAVE_INCREMENT,
    };
    let asteroid_count = base_count + wave_index * increment;

    // 计算速度倍数
    let speed_mult = (1.0 + wave_index as f32 * gameplay::WAVE_SPEED_INCREMENT)
        .min(gameplay::WAVE_SPEED_MAX_MULTIPLIER)
        * asteroid_speed_multiplier;

    asteroids.extend(spawn_wave_with_speed(
        Vec2::new(screen_width() / 2., screen_height() / 2.),
        screen_width().min(screen_height()),
        asteroid_count,
        speed_mult,
    ));

    // UFO 生成逻辑
    if *survival_wave >= *next_ufo_wave {
        let ufo_count = if *survival_wave >= 7 { 2 } else { 1 };
        for _ in 0..ufo_count {
            // 确定保底掉落
            let guaranteed_drop = !*first_ufo_spawned;
            if guaranteed_drop {
                *first_ufo_spawned = true;
            }
            ufos.push(Ufo::spawn_random(now, guaranteed_drop));
        }
        // 下一次 UFO 出现间隔
        *next_ufo_wave = *survival_wave + rand::gen_range(2u32, 4);
    }
}

// ============================================================================
// 时间常量
// ============================================================================

/// 胜利暂停持续时间
pub const VICTORY_PAUSE_DURATION: f64 = timing::VICTORY_PAUSE;

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_victory_pause_duration() {
        assert!(VICTORY_PAUSE_DURATION > 0.0);
    }
}
