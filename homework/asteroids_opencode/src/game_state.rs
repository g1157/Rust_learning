//! 游戏状态机和核心类型定义
//!
//! 从 main.rs 拆分出来的类型定义，包括：
//! - 游戏模式 (GameMode)
//! - 游戏状态机 (GameState)
//! - 游戏设置 (GameSettings)
//! - 各种辅助枚举类型
//!
//! 注意：部分字段为未来功能准备，暂时允许 dead_code

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::achievement::AchievementManager;
use crate::battle_draft::DraftState;
use crate::constants::defaults;
use crate::roguelike;

// ============================================================================
// 字体选项
// ============================================================================

/// 字体选项
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    Default,    // Macroquad 默认字体
    DejaVuSans, // DejaVu Sans (圆润)
    Ubuntu,     // Ubuntu Sans (更圆润)
    Custom,     // 自定义 font.ttf
}

impl FontChoice {
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::DejaVuSans,
            Self::DejaVuSans => Self::Ubuntu,
            Self::Ubuntu => Self::Custom,
            Self::Custom => Self::Default,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Default => Self::Custom,
            Self::DejaVuSans => Self::Default,
            Self::Ubuntu => Self::DejaVuSans,
            Self::Custom => Self::Ubuntu,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::DejaVuSans => "DejaVu Sans",
            Self::Ubuntu => "Ubuntu",
            Self::Custom => "Custom",
        }
    }
}

// ============================================================================
// 玩家数量
// ============================================================================

/// 玩家数量
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerCount {
    One,
    Two,
}

impl PlayerCount {
    pub fn value(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::One => "1 Player",
            Self::Two => "2 Players",
        }
    }
}

// ============================================================================
// 游戏设置
// ============================================================================

/// 游戏设置
#[derive(Clone, Copy)]
pub struct GameSettings {
    pub starting_lives: u32,
    pub ship_speed_multiplier: f32,
    pub asteroid_speed_multiplier: f32,
    pub sound_volume: f32,
    pub font_choice: FontChoice,
    pub enable_weapon_switch: bool,
    pub enable_screen_shake: bool,
    pub enable_slow_motion: bool,
    pub enable_hit_stop: bool,
    pub enable_debug_panel: bool,
    pub flag_radius: f32,
    pub player_count: PlayerCount,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            starting_lives: defaults::LIVES,
            ship_speed_multiplier: defaults::SHIP_SPEED,
            asteroid_speed_multiplier: defaults::ASTEROID_SPEED,
            sound_volume: defaults::SOUND_VOLUME,
            font_choice: FontChoice::DejaVuSans,
            enable_weapon_switch: true,
            enable_screen_shake: true,
            enable_slow_motion: true,
            enable_hit_stop: true,
            enable_debug_panel: true,
            flag_radius: defaults::FLAG_RADIUS,
            player_count: PlayerCount::Two,
        }
    }
}

impl GameSettings {
    /// 恢复默认设置
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}

// ============================================================================
// 设置选项
// ============================================================================

/// 设置选项枚举
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingOption {
    Lives,
    ShipSpeed,
    AsteroidSpeed,
    SoundVolume,
    FontChoice,
    WeaponSwitch,
    ScreenShake,
    SlowMotion,
    HitStop,
    DebugPanel,
    FlagRadius,
    ResetDefaults,
    ResetAchievements,
}

impl SettingOption {
    pub fn next(self) -> Self {
        match self {
            Self::Lives => Self::ShipSpeed,
            Self::ShipSpeed => Self::AsteroidSpeed,
            Self::AsteroidSpeed => Self::SoundVolume,
            Self::SoundVolume => Self::FontChoice,
            Self::FontChoice => Self::WeaponSwitch,
            Self::WeaponSwitch => Self::ScreenShake,
            Self::ScreenShake => Self::SlowMotion,
            Self::SlowMotion => Self::HitStop,
            Self::HitStop => Self::DebugPanel,
            Self::DebugPanel => Self::FlagRadius,
            Self::FlagRadius => Self::ResetDefaults,
            Self::ResetDefaults => Self::ResetAchievements,
            Self::ResetAchievements => Self::Lives,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Lives => Self::ResetAchievements,
            Self::ShipSpeed => Self::Lives,
            Self::AsteroidSpeed => Self::ShipSpeed,
            Self::SoundVolume => Self::AsteroidSpeed,
            Self::FontChoice => Self::SoundVolume,
            Self::WeaponSwitch => Self::FontChoice,
            Self::ScreenShake => Self::WeaponSwitch,
            Self::SlowMotion => Self::ScreenShake,
            Self::HitStop => Self::SlowMotion,
            Self::DebugPanel => Self::HitStop,
            Self::FlagRadius => Self::DebugPanel,
            Self::ResetDefaults => Self::FlagRadius,
            Self::ResetAchievements => Self::ResetDefaults,
        }
    }
}

/// 统一的设置调整函数
pub fn adjust_setting(
    selection: SettingOption,
    increase: bool,
    settings: &mut GameSettings,
    achievements: &mut AchievementManager,
    show_debug: &mut bool,
    toast_message: &mut Option<(String, f64)>,
    frame_t: f64,
) -> bool {
    match selection {
        SettingOption::Lives => {
            let old = settings.starting_lives;
            if increase && settings.starting_lives < 9 {
                settings.starting_lives += 1;
            } else if !increase && settings.starting_lives > 1 {
                settings.starting_lives -= 1;
            }
            settings.starting_lives != old
        }
        SettingOption::ShipSpeed => {
            let old = settings.ship_speed_multiplier;
            let delta = if increase { 0.1 } else { -0.1 };
            settings.ship_speed_multiplier =
                (settings.ship_speed_multiplier + delta).clamp(0.5, 2.0);
            (settings.ship_speed_multiplier - old).abs() > f32::EPSILON
        }
        SettingOption::AsteroidSpeed => {
            let old = settings.asteroid_speed_multiplier;
            let delta = if increase { 0.1 } else { -0.1 };
            settings.asteroid_speed_multiplier =
                (settings.asteroid_speed_multiplier + delta).clamp(0.5, 2.0);
            (settings.asteroid_speed_multiplier - old).abs() > f32::EPSILON
        }
        SettingOption::SoundVolume => {
            let old = settings.sound_volume;
            let delta = if increase { 0.01 } else { -0.01 };
            settings.sound_volume = (settings.sound_volume + delta).clamp(0.0, 1.0);
            (settings.sound_volume - old).abs() > f32::EPSILON
        }
        SettingOption::FontChoice => {
            let old = settings.font_choice;
            settings.font_choice = if increase {
                settings.font_choice.next()
            } else {
                settings.font_choice.prev()
            };
            settings.font_choice != old
        }
        SettingOption::WeaponSwitch => {
            settings.enable_weapon_switch = !settings.enable_weapon_switch;
            true
        }
        SettingOption::ScreenShake => {
            settings.enable_screen_shake = !settings.enable_screen_shake;
            true
        }
        SettingOption::SlowMotion => {
            settings.enable_slow_motion = !settings.enable_slow_motion;
            true
        }
        SettingOption::HitStop => {
            settings.enable_hit_stop = !settings.enable_hit_stop;
            true
        }
        SettingOption::DebugPanel => {
            settings.enable_debug_panel = !settings.enable_debug_panel;
            *show_debug = settings.enable_debug_panel;
            true
        }
        SettingOption::FlagRadius => {
            let old = settings.flag_radius;
            let delta = if increase { 5.0 } else { -5.0 };
            settings.flag_radius = (settings.flag_radius + delta).clamp(50.0, 150.0);
            (settings.flag_radius - old).abs() > f32::EPSILON
        }
        SettingOption::ResetDefaults => {
            settings.reset_to_default();
            *show_debug = settings.enable_debug_panel;
            *toast_message = Some(("Settings reset to defaults".to_string(), frame_t));
            true
        }
        SettingOption::ResetAchievements => {
            achievements.reset();
            *toast_message = Some(("Achievements reset successfully".to_string(), frame_t));
            false
        }
    }
}

// ============================================================================
// 限时挑战模式
// ============================================================================

/// 限时挑战模式时长选项
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeAttackDuration {
    Sixty,
    OneTwenty,
}

impl TimeAttackDuration {
    pub fn seconds(self) -> f64 {
        match self {
            Self::Sixty => 60.0,
            Self::OneTwenty => 120.0,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Sixty => Self::OneTwenty,
            Self::OneTwenty => Self::Sixty,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sixty => "60 seconds",
            Self::OneTwenty => "120 seconds",
        }
    }
}

/// 限时挑战模式状态
#[derive(Clone, Copy)]
pub struct TimeAttackState {
    pub duration: TimeAttackDuration,
    pub start_time: f64,
    pub time_left: f64,
    pub frenzy_active: bool,
    pub frenzy_threshold: f64,
}

impl TimeAttackState {
    pub fn new(duration: TimeAttackDuration, now: f64) -> Self {
        Self {
            duration,
            start_time: now,
            time_left: duration.seconds(),
            frenzy_active: false,
            frenzy_threshold: 15.0,
        }
    }

    pub fn update(&mut self, now: f64) {
        self.time_left = (self.duration.seconds() - (now - self.start_time)).max(0.0);
        if self.time_left <= self.frenzy_threshold && !self.frenzy_active {
            self.frenzy_active = true;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.time_left <= 0.0
    }
}

// ============================================================================
// 游戏模式
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Duel,
    TimeAttack,
    Roguelike,
    Online,
    Achievements,
    Settings,
}

impl GameMode {
    /// Returns true if online mode is available (disabled on WASM)
    #[inline]
    pub fn online_available() -> bool {
        !cfg!(target_arch = "wasm32")
    }

    /// Get next mode in menu cycle (skips Online on WASM)
    pub fn next_in_menu(self) -> Self {
        let next = match self {
            GameMode::Survival => GameMode::Duel,
            GameMode::Duel => GameMode::TimeAttack,
            GameMode::TimeAttack => GameMode::Roguelike,
            GameMode::Roguelike => GameMode::Online,
            GameMode::Online => GameMode::Achievements,
            GameMode::Achievements => GameMode::Settings,
            GameMode::Settings => GameMode::Survival,
        };
        if next == GameMode::Online && !Self::online_available() {
            GameMode::Achievements
        } else {
            next
        }
    }

    /// Get previous mode in menu cycle (skips Online on WASM)
    pub fn prev_in_menu(self) -> Self {
        let prev = match self {
            GameMode::Survival => GameMode::Settings,
            GameMode::Duel => GameMode::Survival,
            GameMode::TimeAttack => GameMode::Duel,
            GameMode::Roguelike => GameMode::TimeAttack,
            GameMode::Online => GameMode::Roguelike,
            GameMode::Achievements => GameMode::Online,
            GameMode::Settings => GameMode::Achievements,
        };
        if prev == GameMode::Online && !Self::online_available() {
            GameMode::Roguelike
        } else {
            prev
        }
    }
}

// ============================================================================
// 游戏状态机
// ============================================================================

/// 暂停菜单选项
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PauseSelection {
    Resume,
    ModeSelect,
}

/// 游戏状态机
#[derive(Clone)]
pub enum GameState {
    ModeSelection {
        selection: GameMode,
    },
    SettingsDetail {
        selection: SettingOption,
    },
    AchievementsView,
    OnlineLobby {
        nickname_input: bool,
    },
    OnlineWaiting {
        room_id: u32,
    },
    WaitingStart,
    DraftSelection {
        draft_state: DraftState,
    },
    Playing,
    Paused {
        selection: PauseSelection,
        roguelike_state: Option<roguelike::RunState>,
    },
    VictoryPause {
        started_at: f64,
    },
    RoundEnd {
        winner_idx: usize,
    },
    GameOver {
        victory: bool,
        end_time: f64,
    },
    RoguelikeRun {
        run_state: roguelike::RunState,
    },
    RoguelikeChallengeOffer {
        run_state: roguelike::RunState,
    },
    RoguelikeReward {
        run_state: roguelike::RunState,
    },
    RoguelikeShop {
        run_state: roguelike::RunState,
    },
    RoguelikeBoss {
        run_state: roguelike::RunState,
    },
    RoguelikeRest {
        run_state: roguelike::RunState,
    },
    RoguelikeVictory {
        run_state: roguelike::RunState,
    },
}

// ============================================================================
// 在线模式辅助类型
// ============================================================================

/// 在线模式子弹（从服务器同步，仅用于渲染）
#[derive(Clone, Copy)]
pub struct OnlineBullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.starting_lives, 3);
        assert_eq!(settings.ship_speed_multiplier, 1.0);
        assert!(settings.enable_weapon_switch);
    }

    #[test]
    fn test_game_mode_menu_cycle() {
        let mut mode = GameMode::Survival;
        let mut visited = vec![mode];

        for _ in 0..10 {
            mode = mode.next_in_menu();
            if mode == GameMode::Survival {
                break;
            }
            visited.push(mode);
        }

        assert_eq!(mode, GameMode::Survival);
        assert!(visited.len() >= 5);
    }

    #[test]
    fn test_time_attack_state() {
        let mut state = TimeAttackState::new(TimeAttackDuration::Sixty, 0.0);
        assert_eq!(state.time_left, 60.0);
        assert!(!state.frenzy_active);

        state.update(50.0);
        assert!(state.frenzy_active);
        assert!((state.time_left - 10.0).abs() < 0.01);
    }
}
