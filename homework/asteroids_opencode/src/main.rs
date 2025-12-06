//! 小行星游戏 - 主模块
//!
//! 现代化的经典小行星街机游戏重制版。
//! 提供生存模式（合作清除小行星波次）和对战模式（竞技夺旗战）。
//!
//! ## 主要功能
//! - 双人本地多人游戏
//! - 多回合对战系统（Best of 3/5）
//! - 击杀连击奖励系统
//! - QuadTree 空间分区优化碰撞检测
//! - 粒子效果和音效系统
//!
//! ## 游戏循环
//! 使用 Macroquad 引擎的异步主循环，处理：
//! - 用户输入
//! - 物理更新（飞船、小行星、子弹）
//! - 碰撞检测（使用 QuadTree 优化）
//! - 粒子和音效
//! - UI 渲染

mod achievement;
mod asteroid;
mod background;
mod bullet;
mod constants;
mod duel;
mod effects;
mod font;
mod input;
mod network;
mod particle;
mod performance;
mod player;
mod powerup;
mod quadtree;
mod render;
mod score;
mod ship;
mod sound;
mod storage;
mod ufo;
mod ui;
mod ui_achievements;
mod utils;
mod vortex;
mod wasm_input;

use achievement::{AchievementId, AchievementManager};
use asteroid::{Asteroid, spawn_wave_with_speed};
use bullet::{BULLET_RADIUS, BULLET_SPEED, WeaponType};
use clap::Parser;
use duel::{DUEL_BULLET_RADIUS, DuelState};
use effects::{ScreenShake, SlowMotion};
use font::FontSystem;
use macroquad::prelude::*;
use particle::ParticleSystem;
use player::{Controls, Player, SHIELD_DURATION};
use powerup::PowerUp;
use quadtree::{Bounds, ObjectIndex, QuadTree};
use ship::{SHIP_DAMPING, SHIP_HEIGHT, SHIP_ROTATION_STEP, SHIP_THRUST};
use sound::{SoundEffect, SoundSystem};
use ufo::{ENEMY_BULLET_RADIUS, EnemyBullet, UFO_RADIUS, Ufo, draw_enemy_bullet};
use ui::{DebugStats, HudMode};
use utils::{circle_intersects_triangle, wrap_around};
use vortex::VortexManager;

use crate::constants::{defaults, gameplay, shake, slow_motion, timing};

const ASTEROID_COUNT: usize = gameplay::INITIAL_ASTEROID_COUNT;
const ASTEROID_WAVE_INCREMENT: usize = gameplay::ASTEROID_WAVE_INCREMENT;
const VICTORY_PAUSE_DURATION: f64 = timing::VICTORY_PAUSE;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 运行指定帧数后退出（用于性能测试）
    #[arg(long)]
    frames: Option<u64>,

    /// 导出性能指标到文件
    #[arg(long)]
    dump_metrics: Option<String>,

    /// 生成指定数量的实体进行压力测试
    #[arg(long)]
    entities: Option<usize>,

    /// 启用网络测试模式
    #[arg(long)]
    network_test: bool,

    /// 禁用图形界面（仅用于CI）
    #[arg(long)]
    headless: bool,
}

/// 字体选项
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    Default,    // Macroquad 默认字体
    DejaVuSans, // DejaVu Sans (圆润)
    Ubuntu,     // Ubuntu Sans (更圆润)
    Custom,     // 自定义 font.ttf
}

impl FontChoice {
    fn next(self) -> Self {
        match self {
            Self::Default => Self::DejaVuSans,
            Self::DejaVuSans => Self::Ubuntu,
            Self::Ubuntu => Self::Custom,
            Self::Custom => Self::Default,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Default => Self::Custom,
            Self::DejaVuSans => Self::Default,
            Self::Ubuntu => Self::DejaVuSans,
            Self::Custom => Self::Ubuntu,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::DejaVuSans => "DejaVu Sans",
            Self::Ubuntu => "Ubuntu",
            Self::Custom => "Custom",
        }
    }
}

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

/// 游戏设置
#[derive(Clone, Copy)]
pub struct GameSettings {
    pub starting_lives: u32,            // 每回合生命值 (1-9)
    pub ship_speed_multiplier: f32,     // 飞船速度倍数 (0.5-2.0)
    pub asteroid_speed_multiplier: f32, // 小行星速度倍数 (0.5-2.0)
    pub sound_volume: f32,              // 音量 (0.0-1.0)
    pub font_choice: FontChoice,        // 字体选择
    pub enable_weapon_switch: bool,     // 是否允许切换武器
    pub enable_screen_shake: bool,      // 是否开启振动
    pub enable_slow_motion: bool,       // 是否开启慢动作
    pub enable_debug_panel: bool,       // 是否默认显示性能面板
    pub flag_radius: f32,               // Flag 半径 (50.0-150.0)
    pub player_count: PlayerCount,      // 玩家数量 (1 or 2)
}

impl GameSettings {
    fn default() -> Self {
        Self {
            starting_lives: defaults::LIVES,
            ship_speed_multiplier: defaults::SHIP_SPEED,
            asteroid_speed_multiplier: defaults::ASTEROID_SPEED,
            sound_volume: defaults::SOUND_VOLUME,
            font_choice: FontChoice::DejaVuSans, // 默认使用 DejaVu Sans
            enable_weapon_switch: true,
            enable_screen_shake: true,
            enable_slow_motion: true,
            enable_debug_panel: true,
            flag_radius: defaults::FLAG_RADIUS,
            player_count: PlayerCount::Two, // 默认双人
        }
    }

    /// 恢复默认设置
    fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}

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
    DebugPanel,
    FlagRadius,
    ResetDefaults,
    ResetAchievements,
}

impl SettingOption {
    fn next(self) -> Self {
        match self {
            Self::Lives => Self::ShipSpeed,
            Self::ShipSpeed => Self::AsteroidSpeed,
            Self::AsteroidSpeed => Self::SoundVolume,
            Self::SoundVolume => Self::FontChoice,
            Self::FontChoice => Self::WeaponSwitch,
            Self::WeaponSwitch => Self::ScreenShake,
            Self::ScreenShake => Self::SlowMotion,
            Self::SlowMotion => Self::DebugPanel,
            Self::DebugPanel => Self::FlagRadius,
            Self::FlagRadius => Self::ResetDefaults,
            Self::ResetDefaults => Self::ResetAchievements,
            Self::ResetAchievements => Self::Lives,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Lives => Self::ResetAchievements,
            Self::ShipSpeed => Self::Lives,
            Self::AsteroidSpeed => Self::ShipSpeed,
            Self::SoundVolume => Self::AsteroidSpeed,
            Self::FontChoice => Self::SoundVolume,
            Self::WeaponSwitch => Self::FontChoice,
            Self::ScreenShake => Self::WeaponSwitch,
            Self::SlowMotion => Self::ScreenShake,
            Self::DebugPanel => Self::SlowMotion,
            Self::FlagRadius => Self::DebugPanel,
            Self::ResetDefaults => Self::FlagRadius,
            Self::ResetAchievements => Self::ResetDefaults,
        }
    }
}

/// 统一的设置调整函数，减少重复代码
fn adjust_setting(
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
            // 注意：重置成就不算设置更改
            false
        }
    }
}

/// 限时挑战模式时长选项
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeAttackDuration {
    Sixty,     // 60秒
    OneTwenty, // 120秒
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
    pub duration: TimeAttackDuration, // 选定时长
    pub start_time: f64,              // 游戏开始时间
    pub time_left: f64,               // 剩余时间
    pub frenzy_active: bool,          // 狂暴时段是否激活
    pub frenzy_threshold: f64,        // 狂暴时段触发阈值（剩余15秒）
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
        // 进入狂暴时段
        if self.time_left <= self.frenzy_threshold && !self.frenzy_active {
            self.frenzy_active = true;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.time_left <= 0.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum GameMode {
    Survival,
    Duel,
    TimeAttack, // 限时挑战模式
    Online,
    Achievements,
    Settings,
}

impl GameMode {
    /// Returns true if online mode is available (disabled on WASM due to input issues)
    #[inline]
    pub fn online_available() -> bool {
        !cfg!(target_arch = "wasm32")
    }

    /// Get next mode in menu cycle (skips Online on WASM)
    fn next_in_menu(self) -> Self {
        let next = match self {
            GameMode::Survival => GameMode::Duel,
            GameMode::Duel => GameMode::TimeAttack,
            GameMode::TimeAttack => GameMode::Online,
            GameMode::Online => GameMode::Achievements,
            GameMode::Achievements => GameMode::Settings,
            GameMode::Settings => GameMode::Survival,
        };
        // Skip Online on WASM
        if next == GameMode::Online && !Self::online_available() {
            GameMode::Achievements
        } else {
            next
        }
    }

    /// Get previous mode in menu cycle (skips Online on WASM)
    fn prev_in_menu(self) -> Self {
        let prev = match self {
            GameMode::Survival => GameMode::Settings,
            GameMode::Duel => GameMode::Survival,
            GameMode::TimeAttack => GameMode::Duel,
            GameMode::Online => GameMode::TimeAttack,
            GameMode::Achievements => GameMode::Online,
            GameMode::Settings => GameMode::Achievements,
        };
        // Skip Online on WASM
        if prev == GameMode::Online && !Self::online_available() {
            GameMode::TimeAttack
        } else {
            prev
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum GameState {
    ModeSelection { selection: GameMode },
    SettingsDetail { selection: SettingOption }, // 设置详细界面
    AchievementsView,                            // 成就查看界面
    OnlineLobby { nickname_input: bool },        // 在线大厅
    OnlineWaiting { room_id: u32 },              // 等待房间（保留供在线模式使用）
    WaitingStart,
    Playing,
    Paused { selection: PauseSelection },
    VictoryPause { started_at: f64 },
    RoundEnd { winner_idx: usize }, // Duel 回合结束
    GameOver { victory: bool, end_time: f64 },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PauseSelection {
    Resume,
    ModeSelect,
}

/// 在线模式子弹（从服务器同步，仅用于渲染）
#[derive(Clone, Copy)]
#[allow(dead_code)] // vx/vy 用于未来的客户端预测
struct OnlineBullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_owned(),
        window_width: defaults::WINDOW_WIDTH,
        window_height: defaults::WINDOW_HEIGHT,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 解析命令行参数
    let args = Args::parse();

    let start_time = get_time();
    let mut settings = GameSettings::default(); // 先初始化设置
    let mut players = init_players(start_time, settings.starting_lives, settings.player_count);
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut ufos: Vec<Ufo> = Vec::new(); // UFO 敌人列表
    let mut enemy_bullets: Vec<EnemyBullet> = Vec::new(); // UFO 发射的子弹
    let mut powerups: Vec<PowerUp> = Vec::new();
    let mut particles = ParticleSystem::new();
    let sounds = SoundSystem::new().await;
    let fonts = FontSystem::new().await; // 加载字体
    let mut achievements = AchievementManager::new(); // 成就系统
    let mut next_shield_spawn = powerup::schedule_next_spawn(start_time, settings.player_count);
    let mut next_weapon_spawn =
        powerup::schedule_next_weapon_spawn(start_time, settings.player_count);
    let mut current_mode = GameMode::Survival;
    let mut duel_state = DuelState::new(start_time);
    let mut time_attack_state = TimeAttackState::new(TimeAttackDuration::Sixty, start_time);
    let mut time_attack_duration = TimeAttackDuration::Sixty; // 用户选择的时长
    let mut highest_survival_score: u32 = 0;
    let mut survival_wave: u32 = 0;
    let mut next_ufo_wave: u32 = 3; // UFO 从第 3 波开始出现
    let mut first_ufo_spawned: bool = false; // 本局是否已生成首架 UFO（保底掉落）
    let _game_start_time: f64 = 0.0; // 当前局游戏开始时间（保留供未来使用）
    // session_bullets_fired 已移除 - 改用 achievements.stats.bullets_fired
    let mut achievements_scroll: f32 = 0.0; // 成就界面滚动偏移
    let mut settings_scroll: f32 = 0.0; // 设置界面滚动偏移
    let mut vortex_manager = VortexManager::new(timing::VORTEX_SPAWN_INTERVAL); // 漩涡生成间隔

    // 在线多人网络客户端
    let mut network_client = network::NetworkClient::new("ws://127.0.0.1:9001".to_string());
    let mut online_nickname = String::new(); // 玩家昵称
    let mut is_online_mode = false; // 是否为在线模式
    let _last_key_frame = 0u64; // 防抖：上次按键的帧计数（保留供未来使用）

    // 在线模式的子弹（从服务器同步）
    let mut online_bullets: Vec<OnlineBullet> = Vec::new();

    // 性能监控器
    let mut performance_monitor = if let Some(path) = &args.dump_metrics {
        crate::performance::PerformanceMonitor::new().with_export_path(path.clone())
    } else {
        crate::performance::PerformanceMonitor::new()
    };

    let mut state = GameState::ModeSelection {
        selection: GameMode::Survival,
    };
    let mut screen_shake: Option<ScreenShake> = None;
    let mut slow_motion = SlowMotion::new(); // 慢动作系统
    let mut starfield = background::Starfield::new(); // 星空背景
    let mut show_debug = settings.enable_debug_panel; // 从设置初始化
    let mut toast_message: Option<(String, f64)> = None; // (消息文本, 显示开始时间)
    let mut _frame_count = 0u64; // 帧计数器（保留供未来使用）

    // 预分配缓冲区，避免每帧重复分配内存
    let mut vortex_forces: Vec<Vec2> = Vec::with_capacity(ASTEROID_COUNT * 2);
    let mut quadtree = QuadTree::new(Bounds::new(
        0.0,
        0.0,
        defaults::WINDOW_WIDTH as f32,
        defaults::WINDOW_HEIGHT as f32,
    )); // 初始边界，会在每帧更新
    let mut player_query: Vec<ObjectIndex> = Vec::with_capacity(16);
    let mut bullet_query: Vec<ObjectIndex> = Vec::with_capacity(16);
    // UFO 碰撞查询缓冲（预留：UFO 数量较少时直接遍历，未来大量敌人时可启用）
    let mut _ufo_query: Vec<ObjectIndex> = Vec::with_capacity(16);

    loop {
        let input_state = input::Input::new();
        let frame_t = get_time();
        let raw_dt = get_frame_time();
        _frame_count += 1;

        // 轮询网络事件
        network_client.poll();

        // 应用慢动作时间缩放
        let time_scale = slow_motion.update(frame_t as f32);
        let dt = raw_dt * time_scale;

        // ⚠️ 注意：不要在这里调用任何输入函数（input_state.is_key_pressed, mouse_wheel 等）
        // 因为会与状态处理内的输入调用冲突，导致 RefCell panic

        match state {
            GameState::SettingsDetail { selection } => {
                // ⚠️ 暂时禁用鼠标滚轮（避免 RefCell panic）
                let (_mouse_wheel_x, _mouse_wheel_y) = input_state.mouse_wheel();
                settings_scroll += _mouse_wheel_y * 20.0;

                // 键盘滚动（Page Up/Down 或 鼠标侧键）
                if input_state.is_key_down(KeyCode::PageDown) {
                    settings_scroll -= 10.0;
                }
                if input_state.is_key_down(KeyCode::PageUp) {
                    settings_scroll += 10.0;
                }

                // 限制滚动范围
                settings_scroll = settings_scroll.clamp(-300.0, 0.0);

                ui::draw_settings_screen(
                    &settings,
                    selection,
                    fonts.get_best(settings.font_choice),
                    settings_scroll,
                    &starfield,
                    frame_t as f32,
                );

                let mut next_selection = selection;

                // 上下键切换选项
                if input_state.is_key_pressed(KeyCode::Up) || input_state.is_key_pressed(KeyCode::W)
                {
                    next_selection = selection.prev();
                } else if input_state.is_key_pressed(KeyCode::Down)
                    || input_state.is_key_pressed(KeyCode::S)
                {
                    next_selection = selection.next();
                }

                // 左右键调整数值
                let mut setting_changed = false;
                if input_state.is_key_pressed(KeyCode::Left)
                    || input_state.is_key_pressed(KeyCode::A)
                {
                    setting_changed = adjust_setting(
                        selection,
                        false,
                        &mut settings,
                        &mut achievements,
                        &mut show_debug,
                        &mut toast_message,
                        frame_t,
                    );
                } else if input_state.is_key_pressed(KeyCode::Right)
                    || input_state.is_key_pressed(KeyCode::D)
                {
                    setting_changed = adjust_setting(
                        selection,
                        true,
                        &mut settings,
                        &mut achievements,
                        &mut show_debug,
                        &mut toast_message,
                        frame_t,
                    );
                }

                // 追踪设置更改统计
                if setting_changed {
                    achievements.stats.settings_changed += 1;
                }

                // Enter 触发恢复默认或重置成就
                if input_state.is_key_pressed(KeyCode::Enter)
                    && matches!(
                        selection,
                        SettingOption::ResetDefaults | SettingOption::ResetAchievements
                    )
                {
                    let changed = adjust_setting(
                        selection,
                        true,
                        &mut settings,
                        &mut achievements,
                        &mut show_debug,
                        &mut toast_message,
                        frame_t,
                    );
                    if changed {
                        achievements.stats.settings_changed += 1;
                    }
                }

                if next_selection != selection {
                    state = GameState::SettingsDetail {
                        selection: next_selection,
                    };
                }

                // ESC 返回模式选择
                if input_state.is_key_pressed(KeyCode::Escape) {
                    settings_scroll = 0.0; // 重置滚动
                    state = GameState::ModeSelection {
                        selection: GameMode::Settings,
                    };
                }

                // 绘制消息提示（如果有）
                if let Some((message, show_time)) = &toast_message {
                    let time_since = (frame_t - show_time) as f32;
                    ui_achievements::draw_message_toast(
                        message,
                        time_since,
                        fonts.get_best(settings.font_choice),
                    );

                    // 清理过期的消息
                    if time_since > 3.0 {
                        toast_message = None;
                    }
                }

                next_frame().await;
                continue;
            }
            GameState::AchievementsView => {
                // 处理键盘滚动（修正方向）
                let scroll_speed = 30.0;
                if input_state.is_key_down(KeyCode::Down) || input_state.is_key_down(KeyCode::S) {
                    achievements_scroll -= scroll_speed; // 向下滚动 = 内容向上移动
                }
                if input_state.is_key_down(KeyCode::Up) || input_state.is_key_down(KeyCode::W) {
                    achievements_scroll += scroll_speed; // 向上滚动 = 内容向下移动
                }

                // ⚠️ 暂时禁用鼠标滚轮（避免 RefCell panic）
                let (_mouse_wheel_x, _mouse_wheel_y) = input_state.mouse_wheel();
                // achievements_scroll += _mouse_wheel_y * 20.0;

                // 限制滚动范围（最大内容高度约 1200）
                achievements_scroll = achievements_scroll.clamp(-800.0, 0.0);

                // 绘制成就界面
                ui_achievements::draw_achievements_screen(
                    &achievements,
                    fonts.get_best(settings.font_choice),
                    frame_t,
                    achievements_scroll,
                    &starfield,
                    frame_t as f32,
                );

                // ESC 返回模式选择
                if input_state.is_key_pressed(KeyCode::Escape) {
                    achievements_scroll = 0.0; // 重置滚动
                    state = GameState::ModeSelection {
                        selection: GameMode::Achievements,
                    };
                }

                next_frame().await;
                continue;
            }
            GameState::ModeSelection { selection } => {
                let mut next_selection = selection;
                // 上下键切换（纵向布局）
                // 顺序: Survival → Duel → TimeAttack → Online → Achievements → Settings
                // (Online skipped on WASM)
                if input_state.is_key_pressed(KeyCode::Up) || input_state.is_key_pressed(KeyCode::W)
                {
                    next_selection = selection.prev_in_menu();
                } else if input_state.is_key_pressed(KeyCode::Down)
                    || input_state.is_key_pressed(KeyCode::S)
                {
                    next_selection = selection.next_in_menu();
                }

                // 左右键切换选项（Survival: 玩家数量, TimeAttack: 时长）
                if input_state.is_key_pressed(KeyCode::Left)
                    || input_state.is_key_pressed(KeyCode::A)
                    || input_state.is_key_pressed(KeyCode::Right)
                    || input_state.is_key_pressed(KeyCode::D)
                {
                    match selection {
                        GameMode::Survival => {
                            settings.player_count = settings.player_count.toggle();
                            // 重新初始化玩家
                            players = init_players(
                                frame_t,
                                settings.starting_lives,
                                settings.player_count,
                            );
                        }
                        GameMode::TimeAttack => {
                            time_attack_duration = time_attack_duration.toggle();
                        }
                        _ => {}
                    }
                }

                if next_selection != selection {
                    state = GameState::ModeSelection {
                        selection: next_selection,
                    };
                }

                if input_state.is_key_pressed(KeyCode::Enter)
                    || input_state.is_key_pressed(KeyCode::Space)
                {
                    match next_selection {
                        GameMode::Settings => {
                            // 进入设置详细界面
                            state = GameState::SettingsDetail {
                                selection: SettingOption::Lives,
                            };
                        }
                        GameMode::Achievements => {
                            // 进入成就查看界面
                            state = GameState::AchievementsView;
                        }
                        GameMode::Online => {
                            // 在线模式：原生版本进入大厅（WASM 版本通过菜单导航跳过此选项）
                            // 注意：原生版本的 connect() 会立即返回错误（无 WebSocket FFI）
                            online_nickname.clear();
                            network_client.connect();
                            state = GameState::OnlineLobby {
                                nickname_input: true,
                            };
                        }
                        GameMode::TimeAttack => {
                            // 限时挑战模式
                            current_mode = next_selection;
                            time_attack_state = TimeAttackState::new(time_attack_duration, frame_t);
                            let mode_name = format!("{:?}", next_selection);
                            achievements.stats.modes_played.insert(mode_name);
                            state = GameState::WaitingStart;
                        }
                        _ => {
                            // 进入游戏 (Survival/Duel)
                            current_mode = next_selection;
                            // 追踪模式切换统计
                            let mode_name = format!("{:?}", next_selection);
                            achievements.stats.modes_played.insert(mode_name);
                            state = GameState::WaitingStart;
                        }
                    }
                }

                ui::draw_mode_selection(
                    next_selection,
                    &settings,
                    &achievements,
                    time_attack_duration,
                    GameMode::online_available(),
                    fonts.get_best(settings.font_choice),
                    &starfield,
                    frame_t as f32,
                );
                next_frame().await;
                continue;
            }
            GameState::WaitingStart => {
                if input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::M)
                {
                    state = GameState::ModeSelection {
                        selection: current_mode,
                    };
                    continue;
                }

                ui::draw_waiting_screen(
                    match current_mode {
                        GameMode::Survival => "Survival: press [Enter] to start",
                        GameMode::Duel => "Duel: capture the flag!",
                        GameMode::TimeAttack => "Time Attack: press [Enter] to start the clock!",
                        GameMode::Settings => unreachable!("Settings is not a playable mode"),
                        GameMode::Achievements => {
                            unreachable!("Achievements is not a playable mode")
                        }
                        GameMode::Online => "Online: multiplayer mode (not yet implemented)",
                    },
                    fonts.get_best(settings.font_choice),
                    &starfield,
                    frame_t as f32,
                );
                ui::draw_players_hud(
                    &players,
                    HudMode::Waiting,
                    fonts.get_best(settings.font_choice),
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(
                        highest_survival_score,
                        fonts.get_best(settings.font_choice),
                    );
                } else {
                    let text = format!(
                        "First to {} captures wins. Press [Enter] to start.",
                        duel_state.target_score
                    );
                    let font_choice = fonts.get_best(settings.font_choice);
                    let width = measure_text(&text, font_choice, 24, 1.0).width;
                    draw_text_ex(
                        &text,
                        screen_width() / 2. - width / 2.,
                        screen_height() / 2. + 100.,
                        TextParams {
                            font: font_choice,
                            font_size: 24,
                            color: Color::new(0.65, 0.7, 0.8, 1.0), // 亮色适配深色背景
                            ..Default::default()
                        },
                    );
                }

                if input_state.is_key_pressed(KeyCode::Enter) {
                    // 限时模式：重置计时器
                    if matches!(current_mode, GameMode::TimeAttack) {
                        time_attack_state = TimeAttackState::new(time_attack_duration, frame_t);
                    }

                    start_round(
                        RoundState {
                            players: &mut players,
                            asteroids: &mut asteroids,
                            ufos: &mut ufos,
                            enemy_bullets: &mut enemy_bullets,
                            powerups: &mut powerups,
                            next_shield_spawn: &mut next_shield_spawn,
                            next_weapon_spawn: &mut next_weapon_spawn,
                            duel_state: &mut duel_state,
                            survival_wave: &mut survival_wave,
                            next_ufo_wave: &mut next_ufo_wave,
                            first_ufo_spawned: &mut first_ufo_spawned,
                            vortex_manager: &mut vortex_manager,
                        },
                        frame_t,
                        current_mode,
                        settings.starting_lives,
                        settings.player_count,
                    );

                    // 触发 FirstFlight 成就（完成第一次游戏）
                    achievements.unlock(achievement::AchievementId::FirstFlight, frame_t);

                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
            GameState::GameOver { victory, end_time } => {
                if input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::M)
                {
                    state = GameState::ModeSelection {
                        selection: current_mode,
                    };
                    continue;
                }

                if matches!(current_mode, GameMode::Survival | GameMode::TimeAttack) {
                    highest_survival_score =
                        highest_survival_score.max(total_survival_score(&players));
                }

                let text = match current_mode {
                    GameMode::Survival => {
                        if victory {
                            "All asteroids cleared! Press [Enter] to restart".to_string()
                        } else {
                            "All players eliminated. Press [Enter] to restart".to_string()
                        }
                    }
                    GameMode::Duel => {
                        if let Some(idx) = duel_state.last_winner {
                            let label = players.get(idx).map(|p| p.label).unwrap_or("Player");
                            format!("{label} captured the sector! Press [Enter] to restart")
                        } else {
                            "Duel finished! Press [Enter] to restart".to_string()
                        }
                    }
                    GameMode::TimeAttack => {
                        let score = total_survival_score(&players);
                        format!(
                            "Time's up! Final score: {} - Press [Enter] to restart",
                            score
                        )
                    }
                    GameMode::Settings => unreachable!("Settings is not a playable mode"),
                    GameMode::Achievements => unreachable!("Achievements is not a playable mode"),
                    GameMode::Online => {
                        "Online match finished! Press [Enter] to restart".to_string()
                    }
                };
                ui::draw_game_over_message(
                    &text,
                    fonts.get_best(settings.font_choice),
                    &starfield,
                    frame_t as f32,
                );
                ui::draw_center_scores(
                    &players,
                    end_time,
                    highest_survival_score,
                    fonts.get_best(settings.font_choice),
                );
                ui::draw_players_hud(
                    &players,
                    HudMode::Active { time: end_time },
                    fonts.get_best(settings.font_choice),
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(
                        highest_survival_score,
                        fonts.get_best(settings.font_choice),
                    );
                }

                if input_state.is_key_pressed(KeyCode::Enter) {
                    // TimeAttack 模式：重置计时器
                    if matches!(current_mode, GameMode::TimeAttack) {
                        time_attack_state = TimeAttackState::new(time_attack_duration, frame_t);
                    }

                    start_round(
                        RoundState {
                            players: &mut players,
                            asteroids: &mut asteroids,
                            ufos: &mut ufos,
                            enemy_bullets: &mut enemy_bullets,
                            powerups: &mut powerups,
                            next_shield_spawn: &mut next_shield_spawn,
                            next_weapon_spawn: &mut next_weapon_spawn,
                            duel_state: &mut duel_state,
                            survival_wave: &mut survival_wave,
                            next_ufo_wave: &mut next_ufo_wave,
                            first_ufo_spawned: &mut first_ufo_spawned,
                            vortex_manager: &mut vortex_manager,
                        },
                        frame_t,
                        current_mode,
                        settings.starting_lives,
                        settings.player_count,
                    );
                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
            GameState::Playing => {
                // 检测暂停键（在状态内部检测，避免 RefCell 冲突）
                let pause_pressed = input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::P);
                if pause_pressed {
                    state = GameState::Paused {
                        selection: PauseSelection::Resume,
                    };
                    next_frame().await;
                    continue;
                }
            }
            GameState::Paused { selection } => {
                let duel_view = matches!(current_mode, GameMode::Duel).then_some(&duel_state);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &particles,
                    duel_view,
                    frame_t,
                    false,
                    None,
                    None,
                    1.0,
                    &achievements,
                    fonts.get_best(settings.font_choice),
                    settings.flag_radius,
                    current_mode,
                    &vortex_manager,
                    &starfield,
                    is_online_mode,
                    &online_bullets,
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(
                        highest_survival_score,
                        fonts.get_best(settings.font_choice),
                    );
                }

                let mut next_selection = selection;
                if input_state.is_key_pressed(KeyCode::Up)
                    || input_state.is_key_pressed(KeyCode::Left)
                    || input_state.is_key_pressed(KeyCode::W)
                    || input_state.is_key_pressed(KeyCode::A)
                {
                    next_selection = PauseSelection::Resume;
                } else if input_state.is_key_pressed(KeyCode::Down)
                    || input_state.is_key_pressed(KeyCode::Right)
                    || input_state.is_key_pressed(KeyCode::S)
                    || input_state.is_key_pressed(KeyCode::D)
                {
                    next_selection = PauseSelection::ModeSelect;
                }

                ui::draw_pause_menu(next_selection, fonts.get_best(settings.font_choice));

                if input_state.is_key_pressed(KeyCode::Enter)
                    || input_state.is_key_pressed(KeyCode::Space)
                {
                    match next_selection {
                        PauseSelection::Resume => {
                            state = GameState::Playing;
                        }
                        PauseSelection::ModeSelect => {
                            state = GameState::ModeSelection {
                                selection: current_mode,
                            };
                        }
                    }
                    next_frame().await;
                    continue;
                }

                // 检测 ESC 键退出暂停（在状态内部检测）
                if input_state.is_key_pressed(KeyCode::Escape) {
                    state = GameState::Playing;
                    next_frame().await;
                    continue;
                }

                if next_selection != selection {
                    state = GameState::Paused {
                        selection: next_selection,
                    };
                }

                next_frame().await;
                continue;
            }
            GameState::VictoryPause { started_at } => {
                let duel_view = matches!(current_mode, GameMode::Duel).then_some(&duel_state);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &particles,
                    duel_view,
                    frame_t,
                    false,
                    None,
                    None,
                    1.0,
                    &achievements,
                    fonts.get_best(settings.font_choice),
                    settings.flag_radius,
                    current_mode,
                    &vortex_manager,
                    &starfield,
                    is_online_mode,
                    &online_bullets,
                );
                ui::draw_victory_pause_overlay(
                    (started_at + VICTORY_PAUSE_DURATION - frame_t).max(0.0),
                    fonts.get_best(settings.font_choice),
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(
                        highest_survival_score,
                        fonts.get_best(settings.font_choice),
                    );
                }
                if frame_t - started_at >= VICTORY_PAUSE_DURATION {
                    // 生成下一波小行星并继续游戏
                    survival_wave += 1;
                    spawn_survival_wave(&mut asteroids, survival_wave, settings.player_count);
                    state = GameState::Playing;
                }
                next_frame().await;
                continue;
            }
            GameState::RoundEnd { winner_idx } => {
                // 显示回合结束画面
                let duel_view = Some(&duel_state);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &particles,
                    duel_view,
                    frame_t,
                    false,
                    None,
                    None,
                    1.0,
                    &achievements,
                    fonts.get_best(settings.font_choice),
                    settings.flag_radius,
                    current_mode,
                    &vortex_manager,
                    &starfield,
                    is_online_mode,
                    &online_bullets,
                );
                ui::draw_round_end(
                    winner_idx,
                    &duel_state,
                    fonts.get_best(settings.font_choice),
                );

                // 按空格或回车开始下一回合
                if input_state.is_key_pressed(KeyCode::Space)
                    || input_state.is_key_pressed(KeyCode::Enter)
                {
                    reset_players(
                        &mut players,
                        frame_t,
                        settings.starting_lives,
                        settings.player_count,
                    );
                    duel_state.start_new_round(frame_t);
                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
            GameState::OnlineLobby { nickname_input } => {
                ui::draw_online_lobby(
                    &online_nickname,
                    nickname_input,
                    &network_client,
                    fonts.get_best(settings.font_choice),
                    &starfield,
                    frame_t as f32,
                );

                // 原生版本的输入处理（WASM 版本已禁用在线模式入口）
                if nickname_input {
                    // 处理昵称输入
                    // 简化：按任意字母键添加字符
                    for key in [
                        KeyCode::A,
                        KeyCode::B,
                        KeyCode::C,
                        KeyCode::D,
                        KeyCode::E,
                        KeyCode::F,
                        KeyCode::G,
                        KeyCode::H,
                        KeyCode::I,
                        KeyCode::J,
                        KeyCode::K,
                        KeyCode::L,
                        KeyCode::M,
                        KeyCode::N,
                        KeyCode::O,
                        KeyCode::P,
                        KeyCode::Q,
                        KeyCode::R,
                        KeyCode::S,
                        KeyCode::T,
                        KeyCode::U,
                        KeyCode::V,
                        KeyCode::W,
                        KeyCode::X,
                        KeyCode::Y,
                        KeyCode::Z,
                    ] {
                        if input_state.is_key_pressed(key) && online_nickname.len() < 12 {
                            let c = format!("{:?}", key).chars().last().unwrap_or('?');
                            online_nickname.push(c);
                        }
                    }
                    // 退格删除
                    if input_state.is_key_pressed(KeyCode::Backspace) {
                        online_nickname.pop();
                    }
                }

                // 按 Enter 加入队列
                if input_state.is_key_pressed(KeyCode::Enter) && network_client.is_connected() {
                    let nickname = if online_nickname.is_empty() {
                        format!("Player_{}", rand::gen_range(1000u32, 9999))
                    } else {
                        online_nickname.clone()
                    };
                    // 默认使用 Survival 模式
                    if let Some(net_mode) =
                        network::NetworkGameMode::from_game_mode(GameMode::Survival)
                    {
                        network_client.send(network::ClientMessage::JoinQueue {
                            mode: net_mode,
                            nickname,
                        });
                        current_mode = GameMode::Survival;
                        state = GameState::OnlineWaiting { room_id: 0 };
                    }
                }

                // ESC 返回主菜单
                if input_state.is_key_pressed(KeyCode::Escape) {
                    network_client.disconnect();
                    state = GameState::ModeSelection {
                        selection: GameMode::Online,
                    };
                }

                next_frame().await;
                continue;
            }
            GameState::OnlineWaiting { room_id } => {
                // 等待匹配
                ui::draw_online_waiting(
                    room_id,
                    &network_client,
                    fonts.get_best(settings.font_choice),
                    &starfield,
                    frame_t as f32,
                );

                // ESC 离开队列返回大厅
                if input_state.is_key_pressed(KeyCode::Escape) {
                    network_client.send(network::ClientMessage::LeaveQueue);
                    state = GameState::OnlineLobby {
                        nickname_input: false,
                    };
                    next_frame().await;
                    continue;
                }

                // 处理服务器消息
                while let Some(message) = network_client.receive() {
                    match message {
                        network::ServerMessage::MatchFound {
                            room_id: rid,
                            players,
                            mode,
                        } => {
                            // 匹配成功，发送 Ready 消息启动游戏
                            current_mode = mode.to_game_mode();
                            println!(
                                "匹配成功! 房间: {}, 玩家: {:?}, 模式: {:?}",
                                rid, players, mode
                            );
                            // 立即发送 Ready 消息
                            network_client.send(network::ClientMessage::Ready);
                        }
                        network::ServerMessage::GameStart => {
                            // 游戏开始 - 初始化游戏状态
                            println!("游戏开始!");
                            is_online_mode = true;
                            online_bullets.clear();

                            // 清空本地玩家的子弹（避免残留本地子弹渲染）
                            for player in players.iter_mut() {
                                player.bullets.clear();
                            }

                            // 确保有一个本地玩家用于输入处理
                            // 在线模式下，本地只需要一个玩家对象来处理输入
                            if players.is_empty() {
                                let now = get_time();
                                players.push(Player::new(
                                    "Online Player",
                                    BLUE,
                                    Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
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
                                    },
                                    now,
                                    3, // 默认生命值
                                ));
                            }

                            // 直接进入游戏状态
                            state = GameState::Playing;
                        }
                        network::ServerMessage::Error { message } => {
                            println!("服务器错误: {}", message);
                        }
                        _ => {}
                    }
                }

                next_frame().await;
                continue;
            }
        }

        // ========== 在线模式：输入同步 ==========
        if is_online_mode && matches!(state, GameState::Playing) {
            // 收集当前按键状态
            let mut keys_pressed = Vec::new();

            // 检查玩家1的控制键（假设在线模式只有一个本地玩家）
            if !players.is_empty() {
                let controls = &players[0].controls;

                if input_state.is_key_down(controls.thrust) {
                    keys_pressed.push("thrust".to_string());
                }
                if input_state.is_key_down(controls.left) {
                    keys_pressed.push("left".to_string());
                }
                if input_state.is_key_down(controls.right) {
                    keys_pressed.push("right".to_string());
                }
                if input_state.is_key_down(controls.shoot_primary) {
                    keys_pressed.push("shoot".to_string());
                }
            }

            // 每帧发送输入到服务器（包括空列表，以清除服务器端的输入状态）
            network_client.send(network::ClientMessage::GameInput { keys: keys_pressed });

            // 接收服务器的游戏状态更新
            while let Some(message) = network_client.receive() {
                match message {
                    network::ServerMessage::GameState {
                        players: server_players,
                        asteroids: server_asteroids,
                        bullets: server_bullets,
                        vortices: server_vortices,
                        powerups: server_powerups,
                        ..
                    } => {
                        // 按玩家ID更新本地玩家状态（避免HashMap顺序不稳定导致的错位）
                        // 在线模式下，players[0] 是本地玩家，需要用 player_id 找到对应的服务器数据
                        if let Some(my_id) = &network_client.player_id {
                            // 找到服务器返回的本玩家数据并更新本地玩家
                            if let Some(my_server_data) =
                                server_players.iter().find(|p| &p.id == my_id)
                                && !players.is_empty()
                            {
                                let local_player = &mut players[0];
                                // 更新玩家位置和状态（权威服务器）
                                local_player.ship.pos =
                                    Vec2::new(my_server_data.x, my_server_data.y);
                                local_player.ship.rot = my_server_data.angle;
                                local_player.ship.vel =
                                    Vec2::new(my_server_data.vel_x, my_server_data.vel_y);
                                local_player.lives = my_server_data.lives;

                                // 分数同步
                                if local_player.score.value() != my_server_data.score {
                                    local_player.score.reset();
                                    local_player.score.add_points(my_server_data.score);
                                }

                                // 检查玩家是否存活
                                if !my_server_data.alive && local_player.alive {
                                    local_player.mark_dead(frame_t);
                                } else if my_server_data.alive && !local_player.alive {
                                    local_player.alive = true;
                                }
                            }

                            // 同步其他玩家（对手）用于渲染
                            // 如果 players 只有一个本地玩家，需要添加对手
                            for server_player in server_players.iter() {
                                if &server_player.id != my_id {
                                    // 这是对手玩家，确保有对应的本地 Player 对象
                                    if players.len() < 2 {
                                        // 添加对手玩家用于渲染
                                        let now = get_time();
                                        players.push(Player::new(
                                            "Opponent",
                                            RED, // 对手用红色
                                            Vec2::new(server_player.x, server_player.y),
                                            Controls {
                                                // 对手不需要本地控制
                                                thrust: KeyCode::Unknown,
                                                left: KeyCode::Unknown,
                                                right: KeyCode::Unknown,
                                                shoot_primary: KeyCode::Unknown,
                                                shoot_alt: None,
                                                weapon_switch: KeyCode::Unknown,
                                                weapon_switch_alt: None,
                                                dash: KeyCode::Unknown,
                                                hyperspace: KeyCode::Unknown,
                                            },
                                            now,
                                            server_player.lives,
                                        ));
                                    }
                                    // 更新对手位置
                                    if players.len() > 1 {
                                        let opponent = &mut players[1];
                                        opponent.ship.pos =
                                            Vec2::new(server_player.x, server_player.y);
                                        opponent.ship.rot = server_player.angle;
                                        opponent.ship.vel =
                                            Vec2::new(server_player.vel_x, server_player.vel_y);
                                        opponent.lives = server_player.lives;
                                        if opponent.score.value() != server_player.score {
                                            opponent.score.reset();
                                            opponent.score.add_points(server_player.score);
                                        }
                                        if !server_player.alive && opponent.alive {
                                            opponent.mark_dead(frame_t);
                                        } else if server_player.alive && !opponent.alive {
                                            opponent.alive = true;
                                        }
                                    }
                                }
                            }
                        }

                        // 更新小行星状态（简单同步）
                        // 注意：完整实现需要更复杂的同步策略（ID匹配、插值等）
                        if server_asteroids.len() != asteroids.len() {
                            // 小行星数量变化 - 简单重建
                            asteroids.clear();
                            for server_ast in server_asteroids.iter() {
                                let sides = 6u8; // 默认6边形
                                let mut vertex_offsets = [1.0f32; asteroid::MAX_VERTICES];
                                // 生成随机顶点偏移以保持不规则外观
                                for offset in vertex_offsets.iter_mut().take(sides as usize) {
                                    *offset = rand::gen_range(0.65, 1.0);
                                }
                                // 服务器 size 是等级 (3/2/1)，转换为客户端像素大小
                                let client_size = match server_ast.size {
                                    3 => 45.0, // 大型
                                    2 => 25.0, // 中型
                                    _ => 12.0, // 小型
                                };
                                let asteroid = Asteroid {
                                    pos: Vec2::new(server_ast.x, server_ast.y),
                                    vel: Vec2::new(server_ast.vx, server_ast.vy),
                                    size: client_size,
                                    rot: 0.0,
                                    rot_speed: 0.0,
                                    sides,
                                    collided: false,
                                    vertex_offsets,
                                };
                                asteroids.push(asteroid);
                            }
                        } else {
                            // 数量相同 - 更新位置
                            for (i, server_ast) in server_asteroids.iter().enumerate() {
                                if i < asteroids.len() {
                                    asteroids[i].pos = Vec2::new(server_ast.x, server_ast.y);
                                    asteroids[i].vel = Vec2::new(server_ast.vx, server_ast.vy);
                                    // 服务器 size 是等级 (3/2/1)，转换为客户端像素大小
                                    asteroids[i].size = match server_ast.size {
                                        3 => 45.0,
                                        2 => 25.0,
                                        _ => 12.0,
                                    };
                                }
                            }
                        }

                        // 更新子弹状态（从服务器同步）
                        online_bullets.clear();
                        for server_bullet in server_bullets.iter() {
                            online_bullets.push(OnlineBullet {
                                x: server_bullet.x,
                                y: server_bullet.y,
                                vx: server_bullet.vx,
                                vy: server_bullet.vy,
                            });
                        }

                        // 更新漩涡状态（从服务器同步）
                        vortex_manager.vortices.clear();
                        for server_vortex in server_vortices.iter() {
                            vortex_manager.vortices.push(vortex::Vortex {
                                pos: Vec2::new(server_vortex.x, server_vortex.y),
                                strength: server_vortex.strength,
                                radius: server_vortex.radius,
                                created_at: server_vortex.created_at,
                                lifetime: server_vortex.lifetime,
                            });
                        }

                        // 更新道具状态（从服务器同步）
                        powerups.clear();
                        for server_powerup in server_powerups.iter() {
                            let powerup_type = match server_powerup.powerup_type {
                                network::PowerupType::Shield => powerup::PowerUpType::Shield,
                                network::PowerupType::DualShot => powerup::PowerUpType::DualShot,
                                network::PowerupType::TripleShot => {
                                    powerup::PowerUpType::TripleShot
                                }
                            };
                            powerups.push(powerup::PowerUp {
                                pos: Vec2::new(server_powerup.x, server_powerup.y),
                                expires_at: server_powerup.expires_at,
                                collected: server_powerup.collected,
                                powerup_type,
                            });
                        }
                    }
                    network::ServerMessage::GameOver { winner, scores } => {
                        println!("游戏结束! 胜者: {:?}, 分数: {:?}", winner, scores);
                        is_online_mode = false;
                        state = GameState::GameOver {
                            victory: winner.as_ref() == network_client.player_id.as_ref(),
                            end_time: frame_t,
                        };
                    }
                    network::ServerMessage::PlayerDisconnected { player_id } => {
                        println!("玩家断开连接: {}", player_id);
                    }
                    _ => {}
                }
            }
        }

        // 在线模式下跳过本地物理更新，由服务器权威控制
        // 本地只负责渲染服务器同步过来的状态
        let bullets_fired = if is_online_mode {
            0 // 在线模式不产生本地子弹
        } else {
            update_players(
                &mut players,
                &mut particles,
                &sounds,
                frame_t,
                dt,
                settings.ship_speed_multiplier,
                settings.sound_volume,
                &input_state,
                &vortex_manager,
                current_mode,
                &asteroids,
            )
        };

        // 更新追踪导弹的目标（寻找最近的小行星）
        update_homing_missiles(&mut players, &asteroids);

        // 更新星空背景视差（基于玩家平均速度）
        let avg_vel: Vec2 = players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.ship.vel)
            .fold(Vec2::ZERO, |acc, v| acc + v)
            / players.iter().filter(|p| p.alive).count().max(1) as f32;
        starfield.update_with_velocity(avg_vel.x, avg_vel.y, dt);

        // 更新漩涡系统（仅在 Survival 模式）
        if current_mode == GameMode::Survival {
            vortex_manager.update(frame_t as f32, screen_width(), screen_height());
        }

        // 计算漩涡力（重用预分配的 Vec）
        vortex_forces.clear();
        vortex_forces.reserve(asteroids.len());
        for asteroid in asteroids.iter() {
            vortex_forces.push(vortex_manager.apply_forces(asteroid.pos, frame_t as f32));
        }

        update_asteroids(
            &mut asteroids,
            dt,
            settings.asteroid_speed_multiplier,
            &vortex_forces,
        );

        // ========== UFO 系统更新 ==========
        // UFO 生成（Survival/TimeAttack 模式，第 3 波开始）
        if matches!(current_mode, GameMode::Survival | GameMode::TimeAttack) && survival_wave >= 3 {
            // 固定波次生成
            let force_spawn = survival_wave >= next_ufo_wave && ufos.is_empty();
            // 低概率随机生成（每帧 0.5% 概率，且场上无 UFO）
            let random_spawn = ufos.is_empty() && rand::gen_range(0.0f32, 1.0) < 0.005;

            if force_spawn || random_spawn {
                // 首架 UFO 保证掉落道具
                let is_first = !first_ufo_spawned;
                // 根据当前波次获取 UFO 难度配置
                let ufo_config = ufo::ufo_config_for_wave(survival_wave);
                ufos.push(Ufo::spawn_from_edge(frame_t, is_first, ufo_config));
                first_ufo_spawned = true;
                if force_spawn {
                    // 下次固定生成在 2-3 波后
                    next_ufo_wave = survival_wave + rand::gen_range(2, 4);
                }
            }
        }

        // 更新 UFO AI 和移动
        let player_positions: Vec<Vec2> = players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.ship.pos)
            .collect();

        for ufo in ufos.iter_mut() {
            ufo.update(dt, frame_t, &player_positions);
        }

        // UFO 射击逻辑
        let player_velocities: Vec<Vec2> = players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.ship.vel)
            .collect();

        for ufo in ufos.iter_mut() {
            if let Some(shot) = ufo.try_fire(frame_t, &player_positions, &player_velocities) {
                enemy_bullets.push(EnemyBullet::from_shot(shot, frame_t));
                sounds.play(SoundEffect::Hit, settings.sound_volume * 0.5); // 敌人射击音效
            }
        }

        // 更新敌人子弹
        for bullet in enemy_bullets.iter_mut() {
            bullet.update(dt);
        }

        // 清理过期或已碰撞的敌人子弹
        enemy_bullets.retain(|b| !b.should_remove(frame_t));

        // 清理出界或已销毁的 UFO
        ufos.retain(|u| !u.should_despawn());

        // 累积游戏时间和子弹发射统计
        achievements.stats.total_playtime += dt as f64;
        achievements.stats.bullets_fired += bullets_fired;
        // session_bullets_fired 已移除 - 改用 achievements.stats.bullets_fired

        // 武器切换 - 每个玩家独立控制（仅当设置允许时）
        if settings.enable_weapon_switch {
            for player in players.iter_mut() {
                if player.controls.weapon_switch_pressed(&input_state) {
                    player.weapon_type = match player.weapon_type {
                        WeaponType::Normal => WeaponType::Spread,
                        WeaponType::Spread => WeaponType::Penetrating,
                        WeaponType::Penetrating => WeaponType::Homing,
                        WeaponType::Homing => WeaponType::Normal,
                    };
                    // 追踪武器使用统计
                    let weapon_name = format!("{:?}", player.weapon_type);
                    achievements.stats.weapons_used.insert(weapon_name);
                }
            }
        }

        // 为子弹添加尾迹效果
        for player in players.iter() {
            for bullet in player.bullets.iter() {
                particles.spawn_bullet_trail(bullet.pos, bullet.vel, player.color, frame_t as f32);
            }
        }

        particles.update(dt, frame_t as f32);

        // 重置并重用 QuadTree（避免每帧分配新内存）
        quadtree.reset(Bounds::new(0.0, 0.0, screen_width(), screen_height()));

        // 插入所有小行星到 QuadTree
        for (idx, asteroid) in asteroids.iter().enumerate() {
            quadtree.insert(ObjectIndex {
                index: idx,
                pos: asteroid.pos,
                radius: asteroid.size,
            });
        }

        let mut new_asteroids = Vec::new();
        let mut player_kills = vec![0u32; players.len()]; // 记录每个玩家的击杀数
        let mut defeated_ufo_indices: Vec<usize> = Vec::new(); // 被击毁的 UFO 索引

        // 玩家与小行星碰撞检测（使用 QuadTree）
        for player in players.iter_mut() {
            if !player.alive || player.is_invulnerable(frame_t) {
                continue;
            }

            let (t1, t2, t3) = player.ship.triangle_vertices();
            let ship_center = player.ship.pos;
            let ship_radius = SHIP_HEIGHT; // 保守估计

            // 查询附近的小行星（重用预分配的 Vec）
            player_query.clear();
            quadtree.query(ship_center, ship_radius, &mut player_query);

            for obj in player_query.iter() {
                let asteroid = &asteroids[obj.index];
                if circle_intersects_triangle(asteroid.pos, asteroid.size, t1, t2, t3) {
                    player.mark_dead(frame_t);
                    // 添加碰撞爆炸效果 - 使用飞船位置而不是小行星位置
                    particles.spawn_explosion(ship_center, asteroid.size, GRAY, frame_t as f32);
                    sounds.play(SoundEffect::Hit, settings.sound_volume);
                    // 玩家死亡触发中等强度震动（仅当设置允许时）
                    if settings.enable_screen_shake {
                        let (intensity, duration) = shake::PLAYER_DEATH;
                        screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                    }
                    break;
                }
            }
        }

        // ========== 玩家与 UFO 碰撞检测 ==========
        for player in players.iter_mut() {
            if !player.alive || player.is_invulnerable(frame_t) {
                continue;
            }

            let (t1, t2, t3) = player.ship.triangle_vertices();
            let ship_center = player.ship.pos;

            for ufo in ufos.iter() {
                if ufo.destroyed {
                    continue;
                }

                // 简单的圆形-三角形碰撞检测
                if circle_intersects_triangle(ufo.pos, UFO_RADIUS, t1, t2, t3) {
                    player.mark_dead(frame_t);
                    particles.spawn_explosion(ship_center, UFO_RADIUS, GRAY, frame_t as f32);
                    sounds.play(SoundEffect::Hit, settings.sound_volume);
                    if settings.enable_screen_shake {
                        let (intensity, duration) = shake::PLAYER_DEATH;
                        screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                    }
                    break;
                }
            }
        }

        // ========== 敌人子弹与玩家碰撞检测 ==========
        for player in players.iter_mut() {
            if !player.alive || player.is_invulnerable(frame_t) {
                continue;
            }

            let (t1, t2, t3) = player.ship.triangle_vertices();
            let ship_center = player.ship.pos;

            for bullet in enemy_bullets.iter_mut() {
                if bullet.collided {
                    continue;
                }

                // 简单的圆形-三角形碰撞检测
                if circle_intersects_triangle(bullet.pos, ENEMY_BULLET_RADIUS, t1, t2, t3) {
                    bullet.collided = true;
                    player.mark_dead(frame_t);
                    particles.spawn_explosion(ship_center, 20.0, RED, frame_t as f32);
                    sounds.play(SoundEffect::Hit, settings.sound_volume);
                    if settings.enable_screen_shake {
                        let (intensity, duration) = shake::PLAYER_DEATH;
                        screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                    }
                    break;
                }
            }
        }

        // 子弹与小行星碰撞检测（使用 QuadTree）
        // 收集小行星击杀信息：(player_idx, asteroid_idx, score_value, asteroid_pos, asteroid_size, player_color, bullet_vel)
        let mut asteroid_hits: Vec<(usize, usize, u32, Vec2, f32, Color, Vec2)> = Vec::new();

        for (player_idx, player) in players.iter_mut().enumerate() {
            for bullet in player.bullets.iter_mut() {
                if bullet.collided {
                    continue;
                }

                // 查询附近的小行星（重用预分配的 Vec）
                bullet_query.clear();
                quadtree.query(bullet.pos, BULLET_RADIUS * 3.0, &mut bullet_query);

                for obj in bullet_query.iter() {
                    let asteroid = &mut asteroids[obj.index];
                    if asteroid.collided {
                        continue;
                    }

                    if (asteroid.pos - bullet.pos).length() < asteroid.size {
                        asteroid.collided = true;
                        asteroid_hits.push((
                            player_idx,
                            obj.index,
                            asteroid.score_value(),
                            asteroid.pos,
                            asteroid.size,
                            player.color,
                            bullet.vel,
                        ));

                        // 记录击杀（仅在 Duel 模式下）
                        if matches!(current_mode, GameMode::Duel) {
                            player_kills[player_idx] += 1;
                        }

                        if let Some(split) = asteroid.split(bullet.vel) {
                            new_asteroids.extend(split);
                        }

                        // 尝试穿透，如果失败则标记碰撞
                        if !bullet.try_penetrate() {
                            break;
                        }
                    }
                }
            }
        }

        // 应用小行星击杀奖励和特效
        for (
            player_idx,
            _asteroid_idx,
            score_value,
            asteroid_pos,
            asteroid_size,
            player_color,
            _bullet_vel,
        ) in asteroid_hits
        {
            if players[player_idx].add_score(score_value) {
                // 获得额外生命！播放音效
                sounds.play(SoundEffect::PowerUp, settings.sound_volume);
            }

            // 添加小行星爆炸效果
            particles.spawn_explosion(asteroid_pos, asteroid_size, player_color, frame_t as f32);
            sounds.play(SoundEffect::Explosion, settings.sound_volume);

            // 大型小行星爆炸触发震动（仅当设置允许时）
            if settings.enable_screen_shake {
                if asteroid_size >= gameplay::ASTEROID_SIZE_LARGE {
                    let (intensity, duration) = shake::ASTEROID_LARGE;
                    screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                } else if asteroid_size >= gameplay::ASTEROID_SIZE_MEDIUM {
                    let (intensity, duration) = shake::ASTEROID_MEDIUM;
                    screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                }
            }
        }

        // ========== 子弹与 UFO 碰撞检测 ==========
        // 收集 UFO 击杀信息：(player_idx, ufo_idx, score_value, ufo_pos, drop_chance, player_color)
        let mut ufo_hits: Vec<(usize, usize, u32, Vec2, f32, Color)> = Vec::new();

        for (player_idx, player) in players.iter_mut().enumerate() {
            for bullet in player.bullets.iter_mut() {
                if bullet.collided {
                    continue;
                }

                // 遍历所有 UFO（数量较少，无需 QuadTree）
                for (ufo_idx, ufo) in ufos.iter_mut().enumerate() {
                    if ufo.destroyed {
                        continue;
                    }

                    let dist = (ufo.pos - bullet.pos).length();
                    if dist < ufo.radius() + BULLET_RADIUS {
                        // 命中 UFO
                        let destroyed = ufo.take_hit(1, frame_t);

                        // 爆炸效果
                        particles.spawn_explosion(
                            ufo.pos,
                            ufo.radius() * 0.8,
                            player.color,
                            frame_t as f32,
                        );
                        sounds.play(SoundEffect::Hit, settings.sound_volume);

                        // 屏幕震动
                        if settings.enable_screen_shake {
                            let (intensity, duration) = shake::ASTEROID_MEDIUM;
                            screen_shake =
                                Some(ScreenShake::new(intensity, duration, frame_t as f32));
                        }

                        if destroyed {
                            // UFO 被击毁 - 收集信息，稍后处理
                            ufo_hits.push((
                                player_idx,
                                ufo_idx,
                                ufo.score_value,
                                ufo.pos,
                                ufo.drop_chance,
                                player.color,
                            ));
                        }

                        // 尝试穿透，如果失败则标记碰撞
                        if !bullet.try_penetrate() {
                            break;
                        }
                    }
                }
            }
        }

        // 应用 UFO 击杀奖励
        for (player_idx, ufo_idx, score_value, ufo_pos, drop_chance, player_color) in ufo_hits {
            defeated_ufo_indices.push(ufo_idx);
            if players[player_idx].add_score(score_value) {
                // 获得额外生命！播放音效
                sounds.play(SoundEffect::PowerUp, settings.sound_volume);
            }
            players[player_idx].record_kill(frame_t);
            achievements.stats.total_kills += 1;

            // 更新 UFO 击杀统计并检查成就
            achievements.stats.ufo_kills_total =
                achievements.stats.ufo_kills_total.saturating_add(1);

            // FirstContact: 首次击毁 UFO
            achievements.update_progress(
                AchievementId::FirstContact,
                achievements.stats.ufo_kills_total,
                frame_t,
            );

            // SkyHunter: 累计击毁 10 个 UFO
            achievements.update_progress(
                AchievementId::SkyHunter,
                achievements.stats.ufo_kills_total,
                frame_t,
            );

            // CleanSweep: 无伤击毁 UFO
            if !players[player_idx].took_damage_this_life {
                achievements.update_progress(AchievementId::CleanSweep, 1, frame_t);
            }

            // 更大的爆炸效果
            particles.spawn_explosion(ufo_pos, UFO_RADIUS * 1.5, player_color, frame_t as f32);
            sounds.play(SoundEffect::Explosion, settings.sound_volume);

            // 更强的屏幕震动
            if settings.enable_screen_shake {
                let (intensity, duration) = shake::ASTEROID_LARGE;
                screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
            }

            // 掉落道具（根据掉落几率）
            if rand::gen_range(0.0f32, 1.0) < drop_chance {
                let drop_type = match rand::gen_range(0, 3) {
                    0 => powerup::PowerUpType::Shield,
                    1 => powerup::PowerUpType::DualShot,
                    _ => powerup::PowerUpType::TripleShot,
                };
                // 在 UFO 位置生成道具
                let mut drop = PowerUp::new(frame_t, drop_type);
                drop.pos = ufo_pos;
                powerups.push(drop);
            }
        }

        // 应用击杀记录
        for (player_idx, kills) in player_kills.iter().enumerate() {
            for _ in 0..*kills {
                players[player_idx].record_kill(frame_t);
                // 更新总击杀统计
                achievements.stats.total_kills += 1;
            }

            // 高连击触发震动和慢动作（根据设置）
            if *kills > 0 {
                let streak = players[player_idx].killstreak;
                if streak >= 5 {
                    if settings.enable_screen_shake {
                        let (intensity, duration) = shake::KILLSTREAK_HIGH;
                        screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                    }
                    // 超高连击触发慢动作
                    if settings.enable_slow_motion {
                        let (duration, scale) = slow_motion::STRONG;
                        slow_motion.activate(frame_t as f32, duration, scale);
                    }
                } else if streak >= 3 {
                    if settings.enable_screen_shake {
                        let (intensity, duration) = shake::KILLSTREAK_MID;
                        screen_shake = Some(ScreenShake::new(intensity, duration, frame_t as f32));
                    }
                    // 高连击触发轻微慢动作
                    if settings.enable_slow_motion {
                        let (duration, scale) = slow_motion::LIGHT;
                        slow_motion.activate(frame_t as f32, duration, scale);
                    }
                }
            }
        }

        if matches!(current_mode, GameMode::Duel) {
            handle_duel_hits(&mut players, &mut particles, frame_t);
        }

        for player in players.iter_mut() {
            player.bullets.retain(|bullet| bullet.is_alive(frame_t));
        }
        asteroids.retain(|asteroid| !asteroid.collided);
        asteroids.append(&mut new_asteroids);

        // 清理被击毁的 UFO
        if !defeated_ufo_indices.is_empty() {
            defeated_ufo_indices.sort_unstable();
            defeated_ufo_indices.dedup();
            // 从后往前删除，避免索引错位
            for idx in defeated_ufo_indices.into_iter().rev() {
                if idx < ufos.len() {
                    ufos.swap_remove(idx);
                }
            }
        }

        match current_mode {
            GameMode::Survival => {
                powerup::spawn(
                    frame_t,
                    &mut powerups,
                    &mut next_shield_spawn,
                    &mut next_weapon_spawn,
                    settings.player_count,
                );
                let shields_collected =
                    powerup::handle_pickups(&mut players, &mut powerups, frame_t);
                if shields_collected > 0 {
                    achievements.stats.shields_collected += shields_collected;
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                }
                highest_survival_score = highest_survival_score.max(total_survival_score(&players));

                let all_dead = players.iter().all(|player| !player.alive);
                if all_dead {
                    finalize_survival(&mut players, frame_t);
                    // 保存成就和统计数据
                    achievements.save();
                    state = GameState::GameOver {
                        victory: false,
                        end_time: frame_t,
                    };
                    continue;
                }

                if asteroids.is_empty() {
                    state = GameState::VictoryPause {
                        started_at: frame_t,
                    };
                }
            }
            GameMode::TimeAttack => {
                // 更新计时器
                time_attack_state.update(frame_t);

                // 道具生成（狂暴时段加速掉落）
                if time_attack_state.frenzy_active {
                    // 狂暴时段：更频繁的道具掉落
                    if frame_t >= next_shield_spawn {
                        powerups.push(PowerUp::new(frame_t, powerup::PowerUpType::Shield));
                        next_shield_spawn = frame_t + 2.0; // 每2秒一个护盾
                    }
                } else {
                    powerup::spawn(
                        frame_t,
                        &mut powerups,
                        &mut next_shield_spawn,
                        &mut next_weapon_spawn,
                        settings.player_count,
                    );
                }

                let shields_collected =
                    powerup::handle_pickups(&mut players, &mut powerups, frame_t);
                if shields_collected > 0 {
                    achievements.stats.shields_collected += shields_collected;
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                }

                highest_survival_score = highest_survival_score.max(total_survival_score(&players));

                // 检查时间结束
                if time_attack_state.is_finished() {
                    finalize_survival(&mut players, frame_t);
                    achievements.save();
                    state = GameState::GameOver {
                        victory: true, // 时间到了算胜利
                        end_time: frame_t,
                    };
                    continue;
                }

                // 检查所有玩家死亡
                let all_dead = players.iter().all(|player| !player.alive);
                if all_dead {
                    finalize_survival(&mut players, frame_t);
                    achievements.save();
                    state = GameState::GameOver {
                        victory: false,
                        end_time: frame_t,
                    };
                    continue;
                }

                // 小行星清空后立即生成下一波（无暂停）
                if asteroids.is_empty() {
                    survival_wave += 1;
                    // 狂暴时段：更多小行星，更快速度
                    let speed_mult = if time_attack_state.frenzy_active {
                        1.3
                    } else {
                        1.0
                    };
                    let wave_index = survival_wave.saturating_sub(1) as usize;
                    let asteroid_count = match settings.player_count {
                        PlayerCount::One => ((ASTEROID_COUNT as f32) * 0.7) as usize,
                        PlayerCount::Two => ASTEROID_COUNT,
                    } + wave_index * ASTEROID_WAVE_INCREMENT;

                    let base_speed = (1.0 + wave_index as f32 * gameplay::WAVE_SPEED_INCREMENT)
                        .min(gameplay::WAVE_SPEED_MAX_MULTIPLIER);

                    asteroids.extend(spawn_wave_with_speed(
                        Vec2::new(screen_width() / 2., screen_height() / 2.),
                        screen_width().min(screen_height()),
                        if time_attack_state.frenzy_active {
                            (asteroid_count as f32 * 1.25) as usize
                        } else {
                            asteroid_count
                        },
                        base_speed * speed_mult,
                    ));
                }
            }
            GameMode::Duel => {
                powerups.clear();
                // 检查旗帜夺取胜利
                if let Some(winner_idx) = duel::update(
                    &mut duel_state,
                    &mut players,
                    frame_t,
                    dt,
                    settings.flag_radius,
                ) {
                    duel_state.record_round_winner(winner_idx);
                    duel_state.last_winner = Some(winner_idx);

                    // 检查是否赢得整场比赛
                    if duel_state.check_match_winner().is_some() {
                        // 保存成就和统计数据
                        achievements.save();
                        state = GameState::GameOver {
                            victory: true,
                            end_time: frame_t,
                        };
                    } else {
                        // 只是赢得了当前回合
                        state = GameState::RoundEnd { winner_idx };
                    }
                    continue;
                }

                // 检查击杀胜利（最后一人存活）
                let alive = players.iter().filter(|player| player.alive).count();
                if alive <= 1
                    && let Some(winner_idx) = players.iter().position(|player| player.alive)
                {
                    duel_state.record_round_winner(winner_idx);
                    duel_state.last_winner = Some(winner_idx);

                    // 检查是否赢得整场比赛
                    if duel_state.check_match_winner().is_some() {
                        // 保存成就和统计数据
                        achievements.save();
                        state = GameState::GameOver {
                            victory: true,
                            end_time: frame_t,
                        };
                    } else {
                        // 只是赢得了当前回合
                        state = GameState::RoundEnd { winner_idx };
                    }
                    continue;
                }
            }
            GameMode::Settings => unreachable!("Settings is not a playable mode"),
            GameMode::Achievements => unreachable!("Achievements is not a playable mode"),
            GameMode::Online => {
                // 在线模式暂未实现，跳过游戏逻辑
            }
        }

        // 更新成就进度
        update_achievements(&mut achievements, &players, current_mode, frame_t, 0);

        let duel_view = matches!(current_mode, GameMode::Duel).then_some(&duel_state);

        // 计算性能统计
        let entity_count = asteroids.len() + players.iter().map(|p| p.bullets.len()).sum::<usize>();
        let debug_stats = DebugStats {
            fps: 1.0 / raw_dt, // 使用原始 dt 计算真实 FPS
            entity_count,
            quadtree_depth: quadtree.max_depth(),
            particle_count: particles.count(),
        };

        render_scene(
            &players,
            &asteroids,
            &ufos,
            &enemy_bullets,
            &powerups,
            &particles,
            duel_view,
            frame_t,
            false,
            screen_shake,
            Some(&debug_stats),
            time_scale,
            &achievements,
            fonts.get_best(settings.font_choice),
            settings.flag_radius,
            current_mode,
            &vortex_manager,
            &starfield,
            is_online_mode,
            &online_bullets,
        );
        if matches!(current_mode, GameMode::Survival) {
            ui::draw_survival_record(highest_survival_score, fonts.get_best(settings.font_choice));
        }
        if matches!(current_mode, GameMode::TimeAttack) {
            ui::draw_time_attack_hud(
                time_attack_state.time_left,
                time_attack_state.frenzy_active,
                total_survival_score(&players),
                fonts.get_best(settings.font_choice),
            );
        }

        // 更新性能监控
        performance_monitor.update((
            players.len() + ufos.len(),
            players.iter().map(|p| p.bullets.len()).sum::<usize>()
                + enemy_bullets.len()
                + online_bullets.len(),
            asteroids.len(),
            particles.count(),
        ));

        // 绘制性能覆盖层
        if show_debug {
            performance_monitor.draw_overlay(fonts.get_best(settings.font_choice));
        }

        // 检查帧数限制（用于性能测试）
        if let Some(max_frames) = args.frames
            && performance_monitor.metrics.total_frames >= max_frames
        {
            // 导出性能指标
            if let Err(e) = performance_monitor.export_metrics() {
                eprintln!("Failed to export metrics: {}", e);
            }
            println!("Performance test completed after {} frames", max_frames);
            break;
        }

        next_frame().await;
    }
}

struct RoundState<'a> {
    players: &'a mut [Player],
    asteroids: &'a mut Vec<Asteroid>,
    ufos: &'a mut Vec<Ufo>,
    enemy_bullets: &'a mut Vec<EnemyBullet>,
    powerups: &'a mut Vec<PowerUp>,
    next_shield_spawn: &'a mut f64,
    next_weapon_spawn: &'a mut f64,
    duel_state: &'a mut DuelState,
    survival_wave: &'a mut u32,
    next_ufo_wave: &'a mut u32,
    first_ufo_spawned: &'a mut bool,
    vortex_manager: &'a mut VortexManager,
}

fn start_round(
    state: RoundState,
    now: f64,
    mode: GameMode,
    starting_lives: u32,
    player_count: PlayerCount,
) {
    reset_players(state.players, now, starting_lives, player_count);
    state.asteroids.clear();
    state.ufos.clear();
    state.enemy_bullets.clear();
    state.powerups.clear();
    state.vortex_manager.clear(); // 清空漩涡
    state.vortex_manager.reset_game_time(now as f32); // 重置游戏时间
    *state.next_ufo_wave = 3; // 重置 UFO 生成波次
    *state.first_ufo_spawned = false; // 重置首架 UFO 保底状态
    *state.next_shield_spawn = powerup::schedule_next_spawn(now, player_count);
    *state.next_weapon_spawn = powerup::schedule_next_weapon_spawn(now, player_count);
    if matches!(mode, GameMode::Survival | GameMode::TimeAttack) {
        *state.survival_wave = 1;
        spawn_survival_wave(state.asteroids, *state.survival_wave, player_count);
    } else {
        *state.survival_wave = 0;
        state.duel_state.reset(now);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_players(
    players: &mut [Player],
    particles: &mut ParticleSystem,
    sounds: &SoundSystem,
    frame_t: f64,
    dt: f32,
    ship_speed_multiplier: f32,
    sound_volume: f32,
    input: &crate::input::Input,
    vortex_manager: &VortexManager,
    current_mode: GameMode,
    asteroids: &[Asteroid],
) -> u32 {
    let mut total_bullets_fired = 0;
    for player in players.iter_mut() {
        if !player.alive {
            player.is_thrusting = false;
            continue;
        }

        // 冲刺输入检测
        if input.is_key_pressed(player.controls.dash) && player.can_dash(frame_t) {
            // 冲刺方向：当前面向方向
            let dash_dir = player.ship.forward_vector();
            player.start_dash(frame_t, dash_dir);
            sounds.play(SoundEffect::Shoot, sound_volume * 0.5); // 使用射击音效的低音量版本
        }

        // 超空间跳跃输入检测
        if input.is_key_pressed(player.controls.hyperspace) && player.can_hyperspace(frame_t) {
            player.start_hyperspace(frame_t);
            sounds.play(SoundEffect::PowerUp, sound_volume * 0.7);
        }

        // 超空间跳跃完成处理
        if player.hyperspace_active && !player.is_in_hyperspace(frame_t) {
            // 生成随机传送位置
            let new_pos = Vec2::new(
                rand::gen_range(50.0, screen_width() - 50.0),
                rand::gen_range(50.0, screen_height() - 50.0),
            );

            // 风险检测：15% 基础风险 + 靠近小行星额外风险
            let base_risk = rand::gen_range(0.0, 1.0) < crate::player::HYPERSPACE_RISK_CHANCE;
            let proximity_risk = asteroids.iter().any(|a| {
                new_pos.distance(a.pos) < a.size + 30.0 // 传送到小行星附近
            });

            if base_risk || proximity_risk {
                // 超空间跳跃失败！
                player.hyperspace_malfunction(frame_t);
                particles.spawn_explosion(new_pos, 30.0, player.color, frame_t as f32);
                sounds.play(SoundEffect::Explosion, sound_volume);
            } else {
                // 成功传送
                player.complete_hyperspace(new_pos, frame_t);
                // 传送出现特效
                particles.spawn_explosion(new_pos, 15.0, WHITE, frame_t as f32);
            }
        }

        // 更新冲刺残影
        player.update_dash_trail(frame_t);

        // 超空间跳跃中：跳过所有移动，只更新子弹
        if player.is_in_hyperspace(frame_t) {
            for bullet in player.bullets.iter_mut() {
                bullet.update(dt);
            }
            continue;
        }

        // 如果正在冲刺，应用冲刺移动
        if player.is_dashing(frame_t) {
            // 冲刺中：高速移动，忽略常规控制
            let dash_speed = crate::player::DASH_SPEED_MULTIPLIER
                * crate::ship::SHIP_MAX_SPEED
                * ship_speed_multiplier;
            player.ship.vel = player.dash_direction * dash_speed;
            player.ship.pos += player.ship.vel * dt;
            player.ship.pos = wrap_around(&player.ship.pos);

            // 冲刺时生成特殊粒子效果
            let thruster_pos = player.ship.pos - player.dash_direction * SHIP_HEIGHT / 2.;
            particles.spawn_thruster(thruster_pos, player.dash_direction, frame_t as f32);

            // 更新子弹（即使冲刺中也要更新）
            for bullet in player.bullets.iter_mut() {
                bullet.update(dt);
            }
            continue; // 冲刺中跳过常规移动
        }

        let mut acc = -player.ship.vel * SHIP_DAMPING;
        let thrusting = input.is_key_down(player.controls.thrust);
        player.is_thrusting = thrusting;

        if thrusting {
            acc += player.ship.forward_vector() * SHIP_THRUST * ship_speed_multiplier;
            // 添加推进器粒子效果
            let forward = player.ship.forward_vector();
            let thruster_pos = player.ship.pos - forward * SHIP_HEIGHT / 2.;
            particles.spawn_thruster(thruster_pos, forward, frame_t as f32);
        }

        // 应用漩涡力（仅在 Survival 模式）
        if current_mode == GameMode::Survival {
            let vortex_force = vortex_manager.apply_forces(player.ship.pos, frame_t as f32);
            acc += vortex_force;
        }

        if input.is_key_down(player.controls.right) {
            player.ship.rot += SHIP_ROTATION_STEP * dt * ship_speed_multiplier;
        } else if input.is_key_down(player.controls.left) {
            player.ship.rot -= SHIP_ROTATION_STEP * dt * ship_speed_multiplier;
        }

        // 更新连击状态（检查是否过期）
        player.update_killstreak(frame_t);

        if player.controls.shoot_pressed(input) && player.can_shoot(frame_t) {
            let rot_vec = player.ship.forward_vector();
            let bullet_pos = player.ship.pos + rot_vec * SHIP_HEIGHT / 2.;
            let bullet_vel = rot_vec * BULLET_SPEED;

            let bullets_fired = player.record_shot(bullet_pos, bullet_vel, frame_t);
            total_bullets_fired += bullets_fired;
            // 播放射击音效
            sounds.play(SoundEffect::Shoot, sound_volume);
        }

        player.ship.vel += acc * dt;
        // 应用连击速度加成
        let max_speed = player.max_speed();
        if player.ship.vel.length() > max_speed {
            player.ship.vel = player.ship.vel.normalize() * max_speed;
        }
        player.ship.pos += player.ship.vel * dt;
        player.ship.pos = wrap_around(&player.ship.pos);

        for bullet in player.bullets.iter_mut() {
            bullet.update(dt);
        }
    }
    total_bullets_fired
}

fn update_asteroids(
    asteroids: &mut [Asteroid],
    dt: f32,
    speed_multiplier: f32,
    vortex_forces: &[Vec2],
) {
    for (i, asteroid) in asteroids.iter_mut().enumerate() {
        // 应用漩涡力
        if let Some(force) = vortex_forces.get(i) {
            asteroid.vel += *force * dt;
        }

        asteroid.advance(dt * speed_multiplier);
        asteroid.pos = wrap_around(&asteroid.pos);
    }
}

/// 更新追踪导弹的目标位置
fn update_homing_missiles(players: &mut [Player], asteroids: &[Asteroid]) {
    use crate::constants::homing;

    for player in players.iter_mut() {
        for bullet in player.bullets.iter_mut() {
            if bullet.weapon_type != WeaponType::Homing {
                continue;
            }

            // 寻找最近的小行星作为目标
            let mut closest_target: Option<Vec2> = None;
            let mut closest_dist_sq = homing::TRACKING_RANGE * homing::TRACKING_RANGE;

            for asteroid in asteroids.iter() {
                if asteroid.collided {
                    continue;
                }

                let dist_sq = (asteroid.pos - bullet.pos).length_squared();
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_target = Some(asteroid.pos);
                }
            }

            bullet.set_target(closest_target);
        }
    }
}

fn handle_duel_hits(players: &mut [Player], particles: &mut ParticleSystem, frame_t: f64) {
    // 防御性检查：空数组直接返回
    if players.is_empty() {
        return;
    }

    for shooter_idx in 0..players.len() {
        let (before, rest) = players.split_at_mut(shooter_idx);
        let Some((shooter, after)) = rest.split_first_mut() else {
            // 理论上不可能到达这里，但安全起见直接跳过
            continue;
        };
        for target in before.iter_mut() {
            apply_bullet_hits(shooter, target, particles, frame_t, DUEL_BULLET_RADIUS);
        }
        for target in after.iter_mut() {
            apply_bullet_hits(shooter, target, particles, frame_t, DUEL_BULLET_RADIUS);
        }
    }
}

fn apply_bullet_hits(
    shooter: &mut Player,
    target: &mut Player,
    particles: &mut ParticleSystem,
    frame_t: f64,
    radius: f32,
) {
    if !target.alive {
        return;
    }
    let (t1, t2, t3) = target.ship.triangle_vertices();
    let ship_center = target.ship.pos;
    for bullet in shooter.bullets.iter_mut() {
        if bullet.collided {
            continue;
        }
        if circle_intersects_triangle(bullet.pos, radius, t1, t2, t3) {
            target.mark_dead(frame_t);
            bullet.collided = true;
            // 添加飞船爆炸粒子效果 - 使用飞船位置
            particles.spawn_explosion(ship_center, SHIP_HEIGHT * 1.5, GRAY, frame_t as f32);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_scene(
    players: &[Player],
    asteroids: &[Asteroid],
    ufos: &[Ufo],
    enemy_bullets: &[EnemyBullet],
    powerups: &[PowerUp],
    particles: &ParticleSystem,
    duel_state: Option<&DuelState>,
    frame_t: f64,
    show_pause_hint: bool,
    screen_shake: Option<ScreenShake>,
    debug_stats: Option<&DebugStats>,
    time_scale: f32,
    achievements: &AchievementManager,
    font: Option<&Font>,
    flag_radius: f32,
    current_mode: GameMode,
    vortex_manager: &VortexManager,
    starfield: &background::Starfield,
    is_online_mode: bool,
    online_bullets: &[OnlineBullet],
) {
    // 应用屏幕震动偏移
    let shake_offset = screen_shake
        .filter(|s| s.is_active(frame_t as f32))
        .map(|s| s.get_offset(frame_t as f32))
        .unwrap_or(Vec2::ZERO);

    // 设置摄像机偏移（通过平移所有绘制坐标实现）
    gl_use_default_material();

    // 绘制星空背景（替代纯色渐变）
    starfield.draw(frame_t as f32);

    // 绘制粒子（在背景之后，其他物体之前）
    particles.draw(frame_t as f32);

    // 绘制子弹（使用增强渲染）
    for player in players.iter() {
        for bullet in player.bullets.iter() {
            render::draw_bullet(bullet, shake_offset, player.color, frame_t as f32);
        }
    }

    // 绘制在线模式的子弹（服务器同步）
    if is_online_mode {
        for bullet in online_bullets.iter() {
            let bullet_pos = Vec2::new(bullet.x, bullet.y) + shake_offset;
            draw_circle(bullet_pos.x, bullet_pos.y, 3.0, YELLOW);
        }
    }

    for asteroid in asteroids.iter() {
        // 根据小行星大小变化颜色：大的偏暗灰，小的偏亮白
        let brightness = 0.5 + 0.5 * (1.0 - (asteroid.size / 60.0).min(1.0));
        let asteroid_color = Color::new(brightness, brightness, brightness * 0.95, 1.0);
        // 使用新的不规则多边形绘制函数
        asteroid::draw_asteroid(asteroid, shake_offset, asteroid_color);
    }

    // 绘制漩涡（仅在 Survival 模式）
    if current_mode == GameMode::Survival {
        vortex_manager.draw(frame_t as f32);
    }

    // 绘制 UFO 敌人
    for ufo_entity in ufos.iter() {
        ufo::draw_ufo(ufo_entity, frame_t);
        ufo::draw_ufo_warning(ufo_entity, frame_t);
    }

    // 绘制敌人子弹
    for bullet in enemy_bullets.iter() {
        draw_enemy_bullet(bullet);
    }

    powerup::draw(powerups, frame_t);

    // 绘制飞船（使用增强渲染）
    for player in players.iter() {
        if !player.alive {
            continue;
        }

        // 超空间跳跃中不绘制飞船
        if player.is_in_hyperspace(frame_t) {
            continue;
        }

        let ship_pos = player.ship.pos + shake_offset;
        let is_invulnerable = player.is_invulnerable(frame_t);

        // 绘制冲刺残影（在飞船之前绘制）
        if !player.dash_trail.is_empty() {
            render::draw_dash_trail(&player.dash_trail, player.color, frame_t);
        }

        // 使用增强渲染绘制飞船
        render::draw_ship(
            ship_pos,
            player.ship.rot,
            player.color,
            is_invulnerable,
            player.is_thrusting || player.is_dashing(frame_t), // 冲刺时也显示推进效果
            frame_t as f32,
        );

        // 护盾效果
        if player.shield_active(frame_t) {
            let remaining_ratio = (player.shield_remaining(frame_t) / SHIELD_DURATION) as f32;
            render::draw_shield(ship_pos, frame_t as f32, remaining_ratio);
        }

        // 冲刺冷却指示器
        let cooldown_remaining = player.dash_cooldown_remaining(frame_t);
        if cooldown_remaining > 0.0 {
            let cooldown_ratio = (cooldown_remaining / crate::player::DASH_COOLDOWN) as f32;
            render::draw_dash_indicator(ship_pos, cooldown_ratio, player.color);
        }

        // 超空间跳跃冷却指示器
        let hyperspace_cooldown = player.hyperspace_cooldown_remaining(frame_t);
        if hyperspace_cooldown > 0.0 {
            let cooldown_ratio = (hyperspace_cooldown / crate::player::HYPERSPACE_COOLDOWN) as f32;
            render::draw_hyperspace_indicator(ship_pos, cooldown_ratio);
        }
    }

    if let Some(state) = duel_state
        && let Some(flag) = &state.flag
    {
        duel::draw_flag(flag, flag_radius);
    }

    ui::draw_players_hud(players, HudMode::Active { time: frame_t }, font);

    // 在 Duel 模式下显示连击状态
    if duel_state.is_some() {
        ui::draw_killstreak(players, font);
    }

    // 显示慢动作指示器
    ui::draw_slow_motion_indicator(time_scale, font);

    // 显示性能监控面板
    if let Some(stats) = debug_stats {
        ui::draw_debug_panel(stats, font);
    }

    if show_pause_hint {
        ui::draw_pause_hint(font);
    }

    // 显示成就解锁提示
    let recent_unlocks = achievements.get_recent_unlocks(6.0, frame_t);
    for (i, id) in recent_unlocks.iter().enumerate() {
        if let Some(progress) = achievements.get_progress(*id)
            && let Some(unlock_time) = progress.unlock_time
        {
            let time_since = (frame_t - unlock_time) as f32;
            // 每个提示稍微偏移一点，避免重叠
            let offset_y = i as f32 * 110.0;
            ui_achievements::draw_achievement_unlock_toast_offset(*id, time_since, offset_y, font);
        }
    }
}

fn finalize_survival(players: &mut [Player], time: f64) {
    for player in players.iter_mut() {
        player.finalize_survival(time);
    }
}

fn spawn_survival_wave(asteroids: &mut Vec<Asteroid>, wave: u32, player_count: PlayerCount) {
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
    let asteroid_count = base_count + wave_index * increment;

    // 难度递增：每波速度增加，最多到最大倍数
    let speed_multiplier = (1.0 + wave_index as f32 * gameplay::WAVE_SPEED_INCREMENT)
        .min(gameplay::WAVE_SPEED_MAX_MULTIPLIER);

    asteroids.extend(spawn_wave_with_speed(
        screen_center,
        screen_width().min(screen_height()),
        asteroid_count,
        speed_multiplier,
    ));
}

fn init_players(now: f64, starting_lives: u32, player_count: PlayerCount) -> Vec<Player> {
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
            dash: KeyCode::Space,   // 冲刺键：空格
            hyperspace: KeyCode::H, // 超空间跳跃：H
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
                dash: KeyCode::Kp0,           // 冲刺键：小键盘0
                hyperspace: KeyCode::KpEnter, // 超空间跳跃：小键盘回车
            },
            now,
            starting_lives,
        ));
    }

    players
}

fn reset_players(players: &mut [Player], now: f64, starting_lives: u32, player_count: PlayerCount) {
    let positions = player_start_positions(player_count);
    for (player, position) in players.iter_mut().zip(positions.iter()) {
        player.reset(*position, now, starting_lives);
    }
}

fn player_start_positions(player_count: PlayerCount) -> [Vec2; 2] {
    let center_y = screen_height() / 2.;
    let width = screen_width();
    match player_count {
        PlayerCount::One => [
            Vec2::new(width / 2., center_y), // 单人居中
            Vec2::new(width / 2., center_y), // 占位
        ],
        PlayerCount::Two => [
            Vec2::new(width * 0.25, center_y),
            Vec2::new(width * 0.75, center_y),
        ],
    }
}

fn total_survival_score(players: &[Player]) -> u32 {
    players.iter().map(|player| player.score.value()).sum()
}

/// 更新成就进度（在主循环中调用）
fn update_achievements(
    achievements: &mut AchievementManager,
    players: &[Player],
    current_mode: GameMode,
    frame_t: f64,
    survival_wave: u32,
) {
    use achievement::AchievementId;

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
            // 追踪5连击次数
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

        // 更新最高连击记录
        achievements.stats.max_killstreak = achievements.stats.max_killstreak.max(streak);

        // 检查击杀数成就（基于本局分数）
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

        // 分数成就
        if survival_score >= 1000 {
            achievements.update_progress(AchievementId::Century, survival_score, frame_t);
        }
        if survival_score >= 5000 {
            achievements.update_progress(AchievementId::Champion, survival_score, frame_t);
        }

        // 波次成就
        if survival_wave >= 3 {
            achievements.update_progress(AchievementId::WaveRider, survival_wave, frame_t);
        }
        if survival_wave >= 5 {
            achievements.update_progress(AchievementId::WaveMaster, survival_wave, frame_t);
        }
        if survival_wave >= 10 {
            achievements.update_progress(AchievementId::WaveGod, survival_wave, frame_t);
        }

        // 更新最高波次
        achievements.stats.max_wave = achievements.stats.max_wave.max(survival_wave);
    }

    // 检查累计时间成就
    if achievements.stats.total_playtime >= 1800.0 {
        // 30分钟
        achievements.update_progress(
            AchievementId::Veteran,
            achievements.stats.total_playtime as u32,
            frame_t,
        );
    }
    if achievements.stats.total_playtime >= 7200.0 {
        // 2小时
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

    // 检查连击大师成就（一局内3次以上5连击）
    if achievements.stats.five_streaks >= 3 {
        achievements.unlock(AchievementId::ComboMaster, frame_t);
    }

    // 清理过期的解锁提示
    achievements.cleanup_recent_unlocks(6.0, frame_t);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.starting_lives, 3);
        assert_eq!(settings.ship_speed_multiplier, 1.0);
        assert_eq!(settings.asteroid_speed_multiplier, 1.0);
        assert_eq!(settings.sound_volume, 0.01); // 默认1%音量
        assert!(settings.enable_weapon_switch);
        assert!(settings.enable_screen_shake);
        assert!(settings.enable_slow_motion);
        assert!(settings.enable_debug_panel);
    }

    #[test]
    fn test_sound_volume_range() {
        let settings = GameSettings::default();
        // 验证音量在有效范围内
        assert!(settings.sound_volume >= 0.0);
        assert!(settings.sound_volume <= 1.0);
    }

    #[test]
    fn test_game_mode_menu_navigation_cycle() {
        // 测试菜单导航循环（从 Survival 开始，回到 Survival）
        let mut mode = GameMode::Survival;
        let mut visited = vec![mode];

        for _ in 0..10 {
            // 最多遍历 10 次以防无限循环
            mode = mode.next_in_menu();
            if mode == GameMode::Survival {
                break;
            }
            visited.push(mode);
        }

        // 应该回到起点
        assert_eq!(mode, GameMode::Survival);
        // 至少访问了所有可用模式（不含 Online 在 WASM 上）
        assert!(visited.len() >= 5); // Survival, Duel, TimeAttack, Achievements, Settings
    }

    #[test]
    fn test_game_mode_menu_prev_is_inverse_of_next() {
        // 测试 prev_in_menu 是 next_in_menu 的逆操作
        for initial in [
            GameMode::Survival,
            GameMode::Duel,
            GameMode::TimeAttack,
            GameMode::Achievements,
            GameMode::Settings,
        ] {
            let next = initial.next_in_menu();
            let back = next.prev_in_menu();
            assert_eq!(
                back, initial,
                "prev(next({:?})) should be {:?}, got {:?}",
                initial, initial, back
            );
        }
    }

    #[test]
    fn test_game_mode_online_available_on_native() {
        // 在非 WASM 测试中，online_available 应该返回 true
        // 注意：此测试仅在原生构建时有意义
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(
                GameMode::online_available(),
                "Online should be available on native builds"
            );
        }
    }
}
