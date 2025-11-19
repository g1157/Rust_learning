//! 对战模式模块
//!
//! 实现双人竞技夺旗战，包含多回合系统。
//!
//! ## 功能
//! - 旗帜夺取机制（5 秒占领时间）
//! - 多回合系统（Best of 3/5）
//! - 回合胜利计数
//! - 旗帜重生延迟
//! - 比赛胜利者判定

use macroquad::prelude::*;

use crate::player::Player;

// FLAG_RADIUS 现在从设置中获取
const CAPTURE_TIME: f32 = 5.0; // 秒
const FLAG_RESPAWN_DELAY: f64 = 1.5; // 秒
pub const DUEL_BULLET_RADIUS: f32 = 6.0; // 像素（用于击中检测）

pub struct FlagObjective {
    pub pos: Vec2,
    pub progress: f32,
    pub capturing: Option<usize>,
}

impl FlagObjective {
    pub fn new(flag_radius: f32) -> Self {
        Self {
            pos: Vec2::new(
                rand::gen_range(flag_radius, screen_width() - flag_radius),
                rand::gen_range(flag_radius, screen_height() - flag_radius),
            ),
            progress: 0.0,
            capturing: None,
        }
    }
}

/// 回合模式配置
#[derive(Clone, Copy)]
pub enum RoundMode {
    BestOf3, // 先赢 2 局
    #[allow(dead_code)]
    BestOf5, // 先赢 3 局
}

impl RoundMode {
    pub fn rounds_to_win(self) -> u32 {
        match self {
            RoundMode::BestOf3 => 2,
            RoundMode::BestOf5 => 3,
        }
    }
}

pub struct DuelState {
    pub flag: Option<FlagObjective>,
    pub next_flag_spawn: f64,
    pub target_score: u32,
    pub last_winner: Option<usize>,
    // 多回合系统
    pub round_mode: RoundMode,
    pub round_wins: Vec<u32>, // 每位玩家赢得的回合数
    pub current_round: u32,
}

impl DuelState {
    pub fn new(now: f64) -> Self {
        Self::new_with_mode(now, RoundMode::BestOf3)
    }

    pub fn new_with_mode(now: f64, mode: RoundMode) -> Self {
        Self {
            flag: None,
            next_flag_spawn: now,
            target_score: 3,
            last_winner: None,
            round_mode: mode,
            round_wins: vec![0, 0], // 支持 2 名玩家
            current_round: 1,
        }
    }

    pub fn reset(&mut self, now: f64) {
        self.flag = None;
        self.next_flag_spawn = now;
        self.last_winner = None;
    }

    /// 开始新的回合（重置当前回合的状态，但保留回合胜利计数）
    pub fn start_new_round(&mut self, now: f64) {
        self.flag = None;
        self.next_flag_spawn = now;
        self.current_round += 1;
    }

    /// 记录回合胜利者
    pub fn record_round_winner(&mut self, player_idx: usize) {
        if player_idx < self.round_wins.len() {
            self.round_wins[player_idx] += 1;
        }
    }

    /// 检查是否有玩家赢得了整场比赛
    pub fn check_match_winner(&self) -> Option<usize> {
        let rounds_needed = self.round_mode.rounds_to_win();
        self.round_wins
            .iter()
            .position(|&wins| wins >= rounds_needed)
    }

    /// 完全重置所有回合数据（用于重新开始游戏）
    #[allow(dead_code)]
    pub fn reset_all(&mut self, now: f64) {
        self.reset(now);
        self.round_wins.iter_mut().for_each(|w| *w = 0);
        self.current_round = 1;
    }
}

/// 返回值：
/// - Some(player_idx): 该玩家赢得了当前回合（达到 target_score）
/// - None: 回合继续进行
pub fn update(duel: &mut DuelState, players: &mut [Player], now: f64, dt: f32, flag_radius: f32) -> Option<usize> {
    if duel.flag.is_none() && now >= duel.next_flag_spawn {
        duel.flag = Some(FlagObjective::new(flag_radius));
    }

    if let Some(flag) = duel.flag.as_mut() {
        let mut capturer: Option<usize> = None;
        for (idx, player) in players.iter().enumerate() {
            if !player.alive {
                continue;
            }
            if (player.ship.pos - flag.pos).length() <= flag_radius {
                if capturer.is_some() {
                    capturer = None;
                    break;
                } else {
                    capturer = Some(idx);
                }
            }
        }

        match capturer {
            Some(idx) => {
                if flag.capturing != Some(idx) {
                    flag.capturing = Some(idx);
                    flag.progress = 0.0;
                }
                flag.progress += dt;
                if flag.progress >= CAPTURE_TIME {
                    players[idx].score.add_points(1);
                    duel.flag = None;
                    duel.next_flag_spawn = now + FLAG_RESPAWN_DELAY;
                    // 检查是否赢得当前回合
                    if players[idx].score.value() >= duel.target_score {
                        duel.last_winner = Some(idx);
                        return Some(idx);
                    }
                }
            }
            None => {
                flag.capturing = None;
                if flag.progress > 0.0 {
                    flag.progress = (flag.progress - dt).max(0.0);
                }
            }
        }
    }

    None
}

pub fn draw_flag(flag: &FlagObjective, flag_radius: f32) {
    let base_color = Color::new(0.95, 0.8, 0.2, 0.35);
    draw_circle(flag.pos.x, flag.pos.y, flag_radius, base_color);
    draw_circle_lines(
        flag.pos.x,
        flag.pos.y,
        flag_radius,
        3.,
        Color::new(1.0, 0.85, 0.3, 0.8),
    );

    let progress_ratio = (flag.progress / CAPTURE_TIME).clamp(0.0, 1.0);
    draw_circle(
        flag.pos.x,
        flag.pos.y,
        flag_radius * progress_ratio,
        Color::new(1.0, 0.9, 0.4, 0.5),
    );

    let text = if let Some(idx) = flag.capturing {
        format!("{} capturing...", idx + 1)
    } else {
        "Contest the flag!".to_string()
    };
    let size = measure_text(&text, None, 24, 1.0);
    draw_text(
        &text,
        flag.pos.x - size.width / 2.,
        flag.pos.y - flag_radius - 12.,
        24.,
        DARKGRAY,
    );
}
