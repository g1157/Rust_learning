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

mod asteroid;
mod bullet;
mod duel;
mod font;
mod particle;
mod player;
mod powerup;
mod quadtree;
mod score;
mod ship;
mod sound;
mod ui;
mod utils;

use asteroid::{Asteroid, spawn_initial_wave};
use bullet::{BULLET_RADIUS, BULLET_SPEED, WeaponType};
use duel::{DUEL_BULLET_RADIUS, DuelState};
use font::FontSystem;
use macroquad::prelude::*;
use particle::ParticleSystem;
use player::{Controls, Player, SHIELD_DURATION};
use powerup::PowerUp;
use quadtree::{Bounds, ObjectIndex, QuadTree};
use ship::{SHIP_DAMPING, SHIP_HEIGHT, SHIP_ROTATION_STEP, SHIP_THRUST};
use sound::{SoundEffect, SoundSystem};
use ui::{DebugStats, HudMode};
use utils::{circle_intersects_triangle, wrap_around};

const ASTEROID_COUNT: usize = 10;
const ASTEROID_WAVE_INCREMENT: usize = 2;
const VICTORY_PAUSE_DURATION: f64 = 2.0;

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
}

impl GameSettings {
    fn default() -> Self {
        Self {
            starting_lives: 3,
            ship_speed_multiplier: 1.0,
            asteroid_speed_multiplier: 1.0,
            sound_volume: 0.01,                  // 默认1%音量（相对倍数）
            font_choice: FontChoice::DejaVuSans, // 默认使用 DejaVu Sans
            enable_weapon_switch: true,
            enable_screen_shake: true,
            enable_slow_motion: true,
            enable_debug_panel: true,
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
    ResetDefaults,
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
            Self::DebugPanel => Self::ResetDefaults,
            Self::ResetDefaults => Self::Lives,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Lives => Self::ResetDefaults,
            Self::ShipSpeed => Self::Lives,
            Self::AsteroidSpeed => Self::ShipSpeed,
            Self::SoundVolume => Self::AsteroidSpeed,
            Self::FontChoice => Self::SoundVolume,
            Self::WeaponSwitch => Self::FontChoice,
            Self::ScreenShake => Self::WeaponSwitch,
            Self::SlowMotion => Self::ScreenShake,
            Self::DebugPanel => Self::SlowMotion,
            Self::ResetDefaults => Self::DebugPanel,
        }
    }
}

/// 慢动作系统
#[derive(Clone, Copy)]
struct SlowMotion {
    active: bool,
    start_time: f32,
    duration: f32,
    time_scale: f32, // 时间缩放 0.0-1.0，越小越慢
}

impl SlowMotion {
    fn new() -> Self {
        Self {
            active: false,
            start_time: 0.0,
            duration: 0.0,
            time_scale: 1.0,
        }
    }

    /// 激活慢动作
    fn activate(&mut self, now: f32, duration: f32, scale: f32) {
        self.active = true;
        self.start_time = now;
        self.duration = duration;
        self.time_scale = scale;
    }

    /// 更新慢动作状态
    fn update(&mut self, now: f32) -> f32 {
        if !self.active {
            return 1.0;
        }

        let elapsed = now - self.start_time;
        if elapsed >= self.duration {
            self.active = false;
            return 1.0;
        }

        // 渐入渐出效果
        let progress = elapsed / self.duration;
        if progress < 0.2 {
            // 前20%时间渐入
            let fade_in = progress / 0.2;
            1.0 - (1.0 - self.time_scale) * fade_in
        } else if progress > 0.8 {
            // 后20%时间渐出
            let fade_out = (1.0 - progress) / 0.2;
            1.0 - (1.0 - self.time_scale) * fade_out
        } else {
            self.time_scale
        }
    }
}

/// 屏幕震动系统
#[derive(Clone, Copy)]
struct ScreenShake {
    intensity: f32,
    duration: f32,
    started_at: f32,
}

impl ScreenShake {
    fn new(intensity: f32, duration: f32, now: f32) -> Self {
        Self {
            intensity,
            duration,
            started_at: now,
        }
    }

    /// 获取当前震动偏移量
    fn get_offset(&self, now: f32) -> Vec2 {
        let elapsed = now - self.started_at;
        if elapsed >= self.duration {
            return Vec2::ZERO;
        }

        // 随着时间衰减的震动强度
        let decay = 1.0 - (elapsed / self.duration);
        let current_intensity = self.intensity * decay;

        // 随机方向的震动
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        Vec2::new(
            angle.cos() * current_intensity,
            angle.sin() * current_intensity,
        )
    }

    fn is_active(&self, now: f32) -> bool {
        now - self.started_at < self.duration
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Duel,
    Settings,
}

#[derive(Clone, Copy)]
enum GameState {
    ModeSelection { selection: GameMode },
    SettingsDetail { selection: SettingOption }, // 设置详细界面
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

#[macroquad::main("Asteroids")]
async fn main() {
    let start_time = get_time();
    let mut settings = GameSettings::default(); // 先初始化设置
    let mut players = init_players(start_time, settings.starting_lives);
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut powerups: Vec<PowerUp> = Vec::new();
    let mut particles = ParticleSystem::new();
    let sounds = SoundSystem::new().await;
    let fonts = FontSystem::new().await; // 加载字体
    let mut next_shield_spawn = powerup::schedule_next_spawn(start_time);
    let mut current_mode = GameMode::Survival;
    let mut duel_state = DuelState::new(start_time);
    let mut highest_survival_score: u32 = 0;
    let mut survival_wave: u32 = 0;
    let mut state = GameState::ModeSelection {
        selection: GameMode::Survival,
    };
    let mut screen_shake: Option<ScreenShake> = None;
    let mut slow_motion = SlowMotion::new(); // 慢动作系统
    let mut show_debug = settings.enable_debug_panel; // 从设置初始化

    loop {
        let frame_t = get_time();
        let raw_dt = get_frame_time();

        // 应用慢动作时间缩放
        let time_scale = slow_motion.update(frame_t as f32);
        let dt = raw_dt * time_scale;

        let esc_pressed = is_key_pressed(KeyCode::Escape);
        let pause_pressed = esc_pressed || is_key_pressed(KeyCode::P);

        // F3 切换性能监控
        if is_key_pressed(KeyCode::F3) {
            show_debug = !show_debug;
        }

        match state {
            GameState::SettingsDetail { selection } => {
                ui::draw_settings_screen(&settings, selection, fonts.get(settings.font_choice));

                let mut next_selection = selection;

                // 上下键切换选项
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    next_selection = selection.prev();
                } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    next_selection = selection.next();
                }

                // 左右键调整数值
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    match selection {
                        SettingOption::Lives => {
                            if settings.starting_lives > 1 {
                                settings.starting_lives -= 1;
                            }
                        }
                        SettingOption::ShipSpeed => {
                            settings.ship_speed_multiplier =
                                (settings.ship_speed_multiplier - 0.1).max(0.5);
                        }
                        SettingOption::AsteroidSpeed => {
                            settings.asteroid_speed_multiplier =
                                (settings.asteroid_speed_multiplier - 0.1).max(0.5);
                        }
                        SettingOption::SoundVolume => {
                            settings.sound_volume = (settings.sound_volume - 0.01).max(0.0);
                        }
                        SettingOption::FontChoice => {
                            settings.font_choice = settings.font_choice.prev();
                        }
                        SettingOption::WeaponSwitch => {
                            settings.enable_weapon_switch = !settings.enable_weapon_switch
                        }
                        SettingOption::ScreenShake => {
                            settings.enable_screen_shake = !settings.enable_screen_shake
                        }
                        SettingOption::SlowMotion => {
                            settings.enable_slow_motion = !settings.enable_slow_motion
                        }
                        SettingOption::DebugPanel => {
                            settings.enable_debug_panel = !settings.enable_debug_panel;
                            show_debug = settings.enable_debug_panel;
                        }
                        SettingOption::ResetDefaults => {
                            settings.reset_to_default();
                            show_debug = settings.enable_debug_panel;
                        }
                    }
                } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    match selection {
                        SettingOption::Lives => {
                            if settings.starting_lives < 9 {
                                settings.starting_lives += 1;
                            }
                        }
                        SettingOption::ShipSpeed => {
                            settings.ship_speed_multiplier =
                                (settings.ship_speed_multiplier + 0.1).min(2.0);
                        }
                        SettingOption::AsteroidSpeed => {
                            settings.asteroid_speed_multiplier =
                                (settings.asteroid_speed_multiplier + 0.1).min(2.0);
                        }
                        SettingOption::SoundVolume => {
                            settings.sound_volume = (settings.sound_volume + 0.01).min(1.0);
                        }
                        SettingOption::FontChoice => {
                            settings.font_choice = settings.font_choice.next();
                        }
                        SettingOption::WeaponSwitch => {
                            settings.enable_weapon_switch = !settings.enable_weapon_switch
                        }
                        SettingOption::ScreenShake => {
                            settings.enable_screen_shake = !settings.enable_screen_shake
                        }
                        SettingOption::SlowMotion => {
                            settings.enable_slow_motion = !settings.enable_slow_motion
                        }
                        SettingOption::DebugPanel => {
                            settings.enable_debug_panel = !settings.enable_debug_panel;
                            show_debug = settings.enable_debug_panel;
                        }
                        SettingOption::ResetDefaults => {
                            settings.reset_to_default();
                            show_debug = settings.enable_debug_panel;
                        }
                    }
                }

                // Enter 触发恢复默认
                if is_key_pressed(KeyCode::Enter)
                    && matches!(selection, SettingOption::ResetDefaults)
                {
                    settings.reset_to_default();
                }

                if next_selection != selection {
                    state = GameState::SettingsDetail {
                        selection: next_selection,
                    };
                }

                // ESC 返回模式选择
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::ModeSelection {
                        selection: GameMode::Settings,
                    };
                }

                next_frame().await;
                continue;
            }
            GameState::ModeSelection { selection } => {
                let mut next_selection = selection;
                // 上下键切换（纵向布局）
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    next_selection = match selection {
                        GameMode::Survival => GameMode::Settings,
                        GameMode::Duel => GameMode::Survival,
                        GameMode::Settings => GameMode::Duel,
                    };
                } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    next_selection = match selection {
                        GameMode::Survival => GameMode::Duel,
                        GameMode::Duel => GameMode::Settings,
                        GameMode::Settings => GameMode::Survival,
                    };
                }

                if next_selection != selection {
                    state = GameState::ModeSelection {
                        selection: next_selection,
                    };
                }

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match next_selection {
                        GameMode::Settings => {
                            // 进入设置详细界面
                            state = GameState::SettingsDetail {
                                selection: SettingOption::Lives,
                            };
                        }
                        _ => {
                            // 进入游戏
                            current_mode = next_selection;
                            state = GameState::WaitingStart;
                        }
                    }
                }

                ui::draw_mode_selection(next_selection, &settings, fonts.get(settings.font_choice));
                next_frame().await;
                continue;
            }
            GameState::WaitingStart => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                    state = GameState::ModeSelection {
                        selection: current_mode,
                    };
                    continue;
                }

                ui::draw_waiting_screen(match current_mode {
                    GameMode::Survival => "Survival: press [Enter] to start",
                    GameMode::Duel => "Duel: capture the flag!",
                    GameMode::Settings => unreachable!("Settings is not a playable mode"),
                });
                ui::draw_players_hud(&players, HudMode::Waiting);
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(highest_survival_score);
                } else {
                    let text = format!(
                        "First to {} captures wins. Press [Enter] to start.",
                        duel_state.target_score
                    );
                    let width = measure_text(&text, None, 24, 1.0).width;
                    draw_text(
                        &text,
                        screen_width() / 2. - width / 2.,
                        screen_height() / 2. + 100.,
                        24.,
                        DARKGRAY,
                    );
                }

                if is_key_pressed(KeyCode::Enter) {
                    start_round(
                        RoundState {
                            players: &mut players,
                            asteroids: &mut asteroids,
                            powerups: &mut powerups,
                            next_shield_spawn: &mut next_shield_spawn,
                            duel_state: &mut duel_state,
                            survival_wave: &mut survival_wave,
                        },
                        frame_t,
                        current_mode,
                        settings.starting_lives,
                    );
                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
            GameState::GameOver { victory, end_time } => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                    state = GameState::ModeSelection {
                        selection: current_mode,
                    };
                    continue;
                }

                if matches!(current_mode, GameMode::Survival) {
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
                    GameMode::Settings => unreachable!("Settings is not a playable mode"),
                };
                ui::draw_game_over_message(&text);
                ui::draw_center_scores(&players, end_time, highest_survival_score);
                ui::draw_players_hud(&players, HudMode::Active { time: end_time });
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(highest_survival_score);
                }

                if is_key_pressed(KeyCode::Enter) {
                    start_round(
                        RoundState {
                            players: &mut players,
                            asteroids: &mut asteroids,
                            powerups: &mut powerups,
                            next_shield_spawn: &mut next_shield_spawn,
                            duel_state: &mut duel_state,
                            survival_wave: &mut survival_wave,
                        },
                        frame_t,
                        current_mode,
                        settings.starting_lives,
                    );
                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
            GameState::Playing => {
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
                    &players, &asteroids, &powerups, &particles, duel_view, frame_t, false, None,
                    None, 1.0,
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(highest_survival_score);
                }

                let mut next_selection = selection;
                if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::W)
                    || is_key_pressed(KeyCode::A)
                {
                    next_selection = PauseSelection::Resume;
                } else if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::S)
                    || is_key_pressed(KeyCode::D)
                {
                    next_selection = PauseSelection::ModeSelect;
                }

                ui::draw_pause_menu(next_selection);

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
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

                if esc_pressed {
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
                    &players, &asteroids, &powerups, &particles, duel_view, frame_t, false, None,
                    None, 1.0,
                );
                ui::draw_victory_pause_overlay(
                    (started_at + VICTORY_PAUSE_DURATION - frame_t).max(0.0),
                );
                if matches!(current_mode, GameMode::Survival) {
                    ui::draw_survival_record(highest_survival_score);
                }
                if frame_t - started_at >= VICTORY_PAUSE_DURATION {
                    // 生成下一波小行星并继续游戏
                    survival_wave += 1;
                    spawn_survival_wave(&mut asteroids, survival_wave);
                    state = GameState::Playing;
                }
                next_frame().await;
                continue;
            }
            GameState::RoundEnd { winner_idx } => {
                // 显示回合结束画面
                let duel_view = Some(&duel_state);
                render_scene(
                    &players, &asteroids, &powerups, &particles, duel_view, frame_t, false, None,
                    None, 1.0,
                );
                ui::draw_round_end(winner_idx, &duel_state);

                // 按空格或回车开始下一回合
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    reset_players(&mut players, frame_t, settings.starting_lives);
                    duel_state.start_new_round(frame_t);
                    state = GameState::Playing;
                }

                next_frame().await;
                continue;
            }
        }

        update_players(
            &mut players,
            &mut particles,
            &sounds,
            frame_t,
            dt,
            settings.ship_speed_multiplier,
            settings.sound_volume,
        );
        update_asteroids(&mut asteroids, dt, settings.asteroid_speed_multiplier);

        // 武器切换（Q键）- 仅当设置允许时
        if settings.enable_weapon_switch && is_key_pressed(KeyCode::Q) {
            for player in players.iter_mut() {
                player.weapon_type = match player.weapon_type {
                    WeaponType::Normal => WeaponType::Spread,
                    WeaponType::Spread => WeaponType::Penetrating,
                    WeaponType::Penetrating => WeaponType::Normal,
                };
            }
        }

        // 为子弹添加尾迹效果
        for player in players.iter() {
            for bullet in player.bullets.iter() {
                particles.spawn_bullet_trail(bullet.pos, bullet.vel, player.color, frame_t as f32);
            }
        }

        particles.update(dt, frame_t as f32);

        // 构建 QuadTree 用于加速碰撞检测
        let mut quadtree = QuadTree::new(Bounds::new(0.0, 0.0, screen_width(), screen_height()));

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

        // 玩家与小行星碰撞检测（使用 QuadTree）
        for player in players.iter_mut() {
            if !player.alive || player.is_invulnerable(frame_t) {
                continue;
            }

            let (t1, t2, t3) = player.ship.triangle_vertices();
            let ship_center = player.ship.pos;
            let ship_radius = SHIP_HEIGHT; // 保守估计

            // 查询附近的小行星
            let mut nearby = Vec::new();
            quadtree.query(ship_center, ship_radius, &mut nearby);

            for obj in nearby {
                let asteroid = &asteroids[obj.index];
                if circle_intersects_triangle(asteroid.pos, asteroid.size, t1, t2, t3) {
                    player.mark_dead(frame_t);
                    // 添加碰撞爆炸效果
                    particles.spawn_explosion(asteroid.pos, asteroid.size, GRAY, frame_t as f32);
                    sounds.play(SoundEffect::Hit, settings.sound_volume);
                    // 玩家死亡触发中等强度震动（仅当设置允许时）
                    if settings.enable_screen_shake {
                        screen_shake = Some(ScreenShake::new(4.0, 0.25, frame_t as f32));
                    }
                    break;
                }
            }
        }

        // 子弹与小行星碰撞检测（使用 QuadTree）
        for (player_idx, player) in players.iter_mut().enumerate() {
            for bullet in player.bullets.iter_mut() {
                if bullet.collided {
                    continue;
                }

                // 查询附近的小行星
                let mut nearby = Vec::new();
                quadtree.query(bullet.pos, BULLET_RADIUS * 3.0, &mut nearby);

                for obj in nearby {
                    let asteroid = &mut asteroids[obj.index];
                    if asteroid.collided {
                        continue;
                    }

                    if (asteroid.pos - bullet.pos).length() < asteroid.size {
                        asteroid.collided = true;
                        player.score.add_points(asteroid.score_value());

                        // 记录击杀（仅在 Duel 模式下）
                        if matches!(current_mode, GameMode::Duel) {
                            player_kills[player_idx] += 1;
                        }

                        // 添加小行星爆炸效果
                        particles.spawn_explosion(
                            asteroid.pos,
                            asteroid.size,
                            player.color,
                            frame_t as f32,
                        );
                        sounds.play(SoundEffect::Explosion, settings.sound_volume);

                        // 大型小行星爆炸触发震动（仅当设置允许时）
                        if settings.enable_screen_shake {
                            if asteroid.size >= 40.0 {
                                screen_shake = Some(ScreenShake::new(6.0, 0.2, frame_t as f32));
                            } else if asteroid.size >= 25.0 {
                                screen_shake = Some(ScreenShake::new(3.0, 0.12, frame_t as f32));
                            }
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

        // 应用击杀记录
        for (player_idx, kills) in player_kills.iter().enumerate() {
            for _ in 0..*kills {
                players[player_idx].record_kill(frame_t);
            }

            // 高连击触发震动和慢动作（根据设置）
            if *kills > 0 {
                let streak = players[player_idx].killstreak;
                if streak >= 5 {
                    if settings.enable_screen_shake {
                        screen_shake = Some(ScreenShake::new(7.0, 0.3, frame_t as f32));
                    }
                    // 超高连击触发慢动作：0.4倍速，持续2秒
                    if settings.enable_slow_motion {
                        slow_motion.activate(frame_t as f32, 2.0, 0.4);
                    }
                } else if streak >= 3 {
                    if settings.enable_screen_shake {
                        screen_shake = Some(ScreenShake::new(5.0, 0.2, frame_t as f32));
                    }
                    // 高连击触发轻微慢动作：0.6倍速，持续1.5秒
                    if settings.enable_slow_motion {
                        slow_motion.activate(frame_t as f32, 1.5, 0.6);
                    }
                }
            }
        }

        if matches!(current_mode, GameMode::Duel) {
            handle_duel_hits(&mut players, frame_t);
        }

        for player in players.iter_mut() {
            player.bullets.retain(|bullet| bullet.is_alive(frame_t));
        }
        asteroids.retain(|asteroid| !asteroid.collided);
        asteroids.append(&mut new_asteroids);

        match current_mode {
            GameMode::Survival => {
                powerup::spawn(frame_t, &mut powerups, &mut next_shield_spawn);
                if powerup::handle_pickups(&mut players, &mut powerups, frame_t) {
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                }
                highest_survival_score = highest_survival_score.max(total_survival_score(&players));

                let all_dead = players.iter().all(|player| !player.alive);
                if all_dead {
                    finalize_survival(&mut players, frame_t);
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
            GameMode::Duel => {
                powerups.clear();
                // 检查旗帜夺取胜利
                if let Some(winner_idx) = duel::update(&mut duel_state, &mut players, frame_t, dt) {
                    duel_state.record_round_winner(winner_idx);
                    duel_state.last_winner = Some(winner_idx);

                    // 检查是否赢得整场比赛
                    if duel_state.check_match_winner().is_some() {
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
        }

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
            &powerups,
            &particles,
            duel_view,
            frame_t,
            true,
            screen_shake,
            show_debug.then_some(&debug_stats),
            time_scale,
        );
        if matches!(current_mode, GameMode::Survival) {
            ui::draw_survival_record(highest_survival_score);
        }
        next_frame().await;
    }
}

struct RoundState<'a> {
    players: &'a mut [Player],
    asteroids: &'a mut Vec<Asteroid>,
    powerups: &'a mut Vec<PowerUp>,
    next_shield_spawn: &'a mut f64,
    duel_state: &'a mut DuelState,
    survival_wave: &'a mut u32,
}

fn start_round(state: RoundState, now: f64, mode: GameMode, starting_lives: u32) {
    reset_players(state.players, now, starting_lives);
    state.asteroids.clear();
    state.powerups.clear();
    *state.next_shield_spawn = powerup::schedule_next_spawn(now);
    if matches!(mode, GameMode::Survival) {
        *state.survival_wave = 1;
        spawn_survival_wave(state.asteroids, *state.survival_wave);
    } else {
        *state.survival_wave = 0;
        state.duel_state.reset(now);
    }
}

fn update_players(
    players: &mut [Player],
    particles: &mut ParticleSystem,
    sounds: &SoundSystem,
    frame_t: f64,
    dt: f32,
    ship_speed_multiplier: f32,
    sound_volume: f32,
) {
    for player in players.iter_mut() {
        if !player.alive {
            continue;
        }

        let mut acc = -player.ship.vel * SHIP_DAMPING;
        if is_key_down(player.controls.thrust) {
            acc += player.ship.forward_vector() * SHIP_THRUST * ship_speed_multiplier;
            // 添加推进器粒子效果
            let forward = player.ship.forward_vector();
            let thruster_pos = player.ship.pos - forward * SHIP_HEIGHT / 2.;
            particles.spawn_thruster(thruster_pos, forward, frame_t as f32);
        }

        if is_key_down(player.controls.right) {
            player.ship.rot += SHIP_ROTATION_STEP * dt * ship_speed_multiplier;
        } else if is_key_down(player.controls.left) {
            player.ship.rot -= SHIP_ROTATION_STEP * dt * ship_speed_multiplier;
        }

        // 更新连击状态（检查是否过期）
        player.update_killstreak(frame_t);

        if player.controls.shoot_pressed() && player.can_shoot(frame_t) {
            let rot_vec = player.ship.forward_vector();
            let bullet_pos = player.ship.pos + rot_vec * SHIP_HEIGHT / 2.;
            let bullet_vel = rot_vec * BULLET_SPEED;

            player.record_shot(bullet_pos, bullet_vel, frame_t);
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
}

fn update_asteroids(asteroids: &mut [Asteroid], dt: f32, speed_multiplier: f32) {
    for asteroid in asteroids.iter_mut() {
        asteroid.advance(dt * speed_multiplier);
        asteroid.pos = wrap_around(&asteroid.pos);
    }
}

fn handle_duel_hits(players: &mut [Player], frame_t: f64) {
    for shooter_idx in 0..players.len() {
        let (before, rest) = players.split_at_mut(shooter_idx);
        let (shooter, after) = rest
            .split_first_mut()
            .expect("Player array should not be empty");
        for target in before.iter_mut() {
            apply_bullet_hits(shooter, target, frame_t, DUEL_BULLET_RADIUS);
        }
        for target in after.iter_mut() {
            apply_bullet_hits(shooter, target, frame_t, DUEL_BULLET_RADIUS);
        }
    }
}

fn apply_bullet_hits(shooter: &mut Player, target: &mut Player, frame_t: f64, radius: f32) {
    if !target.alive {
        return;
    }
    let (t1, t2, t3) = target.ship.triangle_vertices();
    for bullet in shooter.bullets.iter_mut() {
        if bullet.collided {
            continue;
        }
        if circle_intersects_triangle(bullet.pos, radius, t1, t2, t3) {
            target.mark_dead(frame_t);
            bullet.collided = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_scene(
    players: &[Player],
    asteroids: &[Asteroid],
    powerups: &[PowerUp],
    particles: &ParticleSystem,
    duel_state: Option<&DuelState>,
    frame_t: f64,
    show_pause_hint: bool,
    screen_shake: Option<ScreenShake>,
    debug_stats: Option<&DebugStats>,
    time_scale: f32,
) {
    // 应用屏幕震动偏移
    let shake_offset = screen_shake
        .filter(|s| s.is_active(frame_t as f32))
        .map(|s| s.get_offset(frame_t as f32))
        .unwrap_or(Vec2::ZERO);

    // 设置摄像机偏移（通过平移所有绘制坐标实现）
    gl_use_default_material();

    ui::draw_gradient_background(
        Color::new(0.88, 0.9, 0.95, 1.0),
        Color::new(0.83, 0.86, 0.92, 1.0),
    );

    // 绘制粒子（在背景之后，其他物体之前）
    particles.draw(frame_t as f32);

    for player in players.iter() {
        for bullet in player.bullets.iter() {
            let pos = bullet.pos + shake_offset;

            // 根据武器类型改变子弹颜色和大小
            let (color, size) = match bullet.weapon_type {
                WeaponType::Normal => (player.color, BULLET_RADIUS),
                WeaponType::Spread => (
                    Color::new(
                        player.color.r * 0.8,
                        player.color.g * 0.8,
                        player.color.b,
                        1.0,
                    ),
                    BULLET_RADIUS * 0.8,
                ),
                WeaponType::Penetrating => (
                    Color::new(1.0, player.color.g * 0.5, 0.0, 1.0), // 橙色穿透弹
                    BULLET_RADIUS * 1.3,
                ),
            };

            draw_circle(pos.x, pos.y, size, color);
        }
    }

    for asteroid in asteroids.iter() {
        let pos = asteroid.pos + shake_offset;
        draw_poly_lines(
            pos.x,
            pos.y,
            asteroid.sides,
            asteroid.size,
            asteroid.rot,
            2.,
            BLACK,
        );
    }

    powerup::draw(powerups, frame_t);

    for player in players.iter() {
        if !player.alive {
            continue;
        }
        let (v1, v2, v3) = player.ship.triangle_vertices();
        let v1 = v1 + shake_offset;
        let v2 = v2 + shake_offset;
        let v3 = v3 + shake_offset;
        let mut color = player.color;
        if player.is_invulnerable(frame_t) {
            color = Color::new(player.color.r, player.color.g, player.color.b, 0.4);
        }
        draw_triangle_lines(v1, v2, v3, 3., color);

        if player.shield_active(frame_t) {
            let remaining = player.shield_remaining(frame_t);
            let intensity = ((remaining / SHIELD_DURATION).clamp(0.2, 1.0)) as f32;
            let ring_color = Color::new(0.2, 0.6, 1.0, 0.2 + 0.5 * intensity);
            let pos = player.ship.pos + shake_offset;
            draw_circle_lines(pos.x, pos.y, SHIP_HEIGHT, 4., ring_color);
        }
    }

    if let Some(state) = duel_state
        && let Some(flag) = &state.flag
    {
        duel::draw_flag(flag);
    }

    ui::draw_players_hud(players, HudMode::Active { time: frame_t });

    // 在 Duel 模式下显示连击状态
    if duel_state.is_some() {
        ui::draw_killstreak(players);
    }

    // 显示慢动作指示器
    ui::draw_slow_motion_indicator(time_scale);

    // 显示性能监控面板
    if let Some(stats) = debug_stats {
        ui::draw_debug_panel(stats);
    }

    if show_pause_hint {
        ui::draw_pause_hint();
    }
}

fn finalize_survival(players: &mut [Player], time: f64) {
    for player in players.iter_mut() {
        player.finalize_survival(time);
    }
}

fn spawn_survival_wave(asteroids: &mut Vec<Asteroid>, wave: u32) {
    let screen_center = Vec2::new(screen_width() / 2., screen_height() / 2.);
    let wave_index = wave.saturating_sub(1) as usize;
    let asteroid_count = ASTEROID_COUNT + wave_index * ASTEROID_WAVE_INCREMENT;
    asteroids.extend(spawn_initial_wave(
        screen_center,
        screen_width().min(screen_height()),
        asteroid_count,
    ));
}

fn init_players(now: f64, starting_lives: u32) -> Vec<Player> {
    let positions = player_start_positions();
    vec![
        Player::new(
            "Player 1",
            BLUE,
            positions[0],
            Controls {
                thrust: KeyCode::W,
                left: KeyCode::A,
                right: KeyCode::D,
                shoot_primary: KeyCode::J,
                shoot_alt: Some(KeyCode::F),
            },
            now,
            starting_lives,
        ),
        Player::new(
            "Player 2",
            RED,
            positions[1],
            Controls {
                thrust: KeyCode::Up,
                left: KeyCode::Left,
                right: KeyCode::Right,
                shoot_primary: KeyCode::Key1,
                shoot_alt: Some(KeyCode::Kp1),
            },
            now,
            starting_lives,
        ),
    ]
}

fn reset_players(players: &mut [Player], now: f64, starting_lives: u32) {
    let positions = player_start_positions();
    for (player, position) in players.iter_mut().zip(positions.iter()) {
        player.reset(*position, now, starting_lives);
    }
}

fn player_start_positions() -> [Vec2; 2] {
    let center_y = screen_height() / 2.;
    let width = screen_width();
    [
        Vec2::new(width * 0.25, center_y),
        Vec2::new(width * 0.75, center_y),
    ]
}

fn total_survival_score(players: &[Player]) -> u32 {
    players.iter().map(|player| player.score.value()).sum()
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
}
