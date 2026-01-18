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
mod battle_draft;
mod bullet;
mod chain_lightning;
mod collision;
mod constants;
mod dash_trail;
mod duel;
mod effects;
mod font;
mod game;
mod game_state;
mod input;
mod interpolation;
mod network;
mod particle;
mod performance;
mod player;
mod powerup;
mod quadtree;
mod render;
mod roguelike;
mod score;
mod ship;
mod sound;
mod storage;
mod theme;
mod tutorial;
mod ufo;
mod ui;
mod ui_achievements;
mod utils;
mod vortex;
mod wasm_input;

use achievement::{AchievementId, AchievementManager};
use asteroid::{Asteroid, spawn_wave_with_speed};
use battle_draft::{Card, DraftState, draw_draft_ui};
use bullet::{BULLET_RADIUS, BULLET_SPEED, WeaponType};
use clap::Parser;
use duel::{DUEL_BULLET_RADIUS, DuelState};
use effects::{HitStop, PendingHitKind, PendingHitManager, ScreenShake, SlowMotion};
use font::FontSystem;
use game::{
    ASTEROID_COUNT, ASTEROID_WAVE_INCREMENT, init_players, reset_players, spawn_survival_wave,
    total_survival_score, update_achievements,
};
use game_state::{
    FontChoice, GameMode, GameSettings, GameState, OnlineBullet, PauseSelection, PlayerCount,
    SettingOption, TimeAttackDuration, TimeAttackState, adjust_setting,
};
use macroquad::prelude::*;
use particle::ParticleSystem;
use player::{Controls, Player, SHIELD_DURATION};
use powerup::PowerUp;
use quadtree::{Bounds, ObjectIndex, QuadTree};
use ship::{SHIP_DAMPING, SHIP_HEIGHT, SHIP_ROTATION_STEP, SHIP_THRUST};
use sound::{SoundEffect, SoundSystem};
use ufo::{ENEMY_BULLET_RADIUS, EnemyBullet, UFO_RADIUS, Ufo, draw_enemy_bullet};
use ui::{DebugStats, HudMode, InterpDebugStats, NetworkDebugStats};
use utils::{circle_intersects_triangle, wrap_around};
use vortex::VortexManager;

use crate::constants::{defaults, gameplay, hit_stop, shake, slow_motion, timing};

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
    let mut pending_hits = PendingHitManager::new(); // 客户端乐观命中提示（Phase 4C）
    let mut chain_lightnings = chain_lightning::ChainLightningManager::new(); // 链式闪电效果管理器
    let mut global_draft_state = DraftState::new(); // 全局选卡状态（追踪 UFO 触发次数）
    let mut tutorial_state = tutorial::TutorialState::new(); // 新手引导系统

    // 在线多人网络客户端
    let server_url = std::env::var("ASTEROIDS_SERVER_URL")
        .unwrap_or_else(|_| "wss://localhost:9001".to_string());
    let mut network_client = network::NetworkClient::new(server_url);
    let mut online_nickname = String::new(); // 玩家昵称
    let mut is_online_mode = false; // 是否为在线模式
    let _last_key_frame = 0u64; // 防抖：上次按键的帧计数（保留供未来使用）

    // 在线模式的子弹（从服务器同步）
    let mut online_bullets: Vec<OnlineBullet> = Vec::new();

    // 实体插值管理器（平滑远程实体渲染）
    let mut interp_manager =
        interpolation::InterpolationManager::new(interpolation::InterpConfig {
            render_delay_ms: 100.0, // 100ms 渲染延迟，平衡平滑度和响应性
            history_secs: 1.5,      // 保留 1.5 秒历史用于插值和回溯
        });

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
    let mut hit_stop_effect = HitStop::new(); // 命中停顿系统
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

        // 应用慢动作和命中停顿时间缩放
        let slow_scale = slow_motion.update(frame_t as f32);
        let hit_stop_scale = hit_stop_effect.update(frame_t as f32);
        let time_scale = slow_scale * hit_stop_scale; // 两者叠加
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
                        GameMode::Roguelike => {
                            // Roguelike 模式：直接进入 Run
                            current_mode = next_selection;
                            let mode_name = format!("{:?}", next_selection);
                            achievements.stats.modes_played.insert(mode_name);
                            // 初始化玩家
                            players = init_players(
                                frame_t,
                                settings.starting_lives,
                                PlayerCount::One, // Roguelike 模式固定单人
                            );
                            // 创建新的 Run 状态
                            let new_run = roguelike::RunState::new();
                            // 使用 Run 状态的难度设置初始化小行星
                            let asteroid_count = new_run.current_asteroid_count();
                            let difficulty = new_run.current_difficulty();
                            asteroids = spawn_wave_with_speed(
                                Vec2::new(screen_width() / 2., screen_height() / 2.),
                                screen_width().min(screen_height()),
                                asteroid_count,
                                difficulty * settings.asteroid_speed_multiplier,
                            );
                            survival_wave = 1;
                            state = GameState::RoguelikeRun { run_state: new_run };
                        }
                        _ => {
                            // 进入游戏 (Survival/Duel)
                            current_mode = next_selection;
                            // 追踪模式切换统计
                            let mode_name = format!("{:?}", next_selection);
                            achievements.stats.modes_played.insert(mode_name);
                            // 确保玩家数量正确（可能从 Roguelike 单人模式返回）
                            players = init_players(
                                frame_t,
                                settings.starting_lives,
                                settings.player_count,
                            );
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
                        GameMode::Roguelike => "Roguelike: press [Enter] to start your run!",
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

                    // Survival/TimeAttack 模式触发开局选卡
                    if matches!(current_mode, GameMode::Survival | GameMode::TimeAttack) {
                        global_draft_state.reset(); // 重置选卡状态（新游戏）
                        global_draft_state.start_draft(true); // 开局选卡
                        state = GameState::DraftSelection {
                            draft_state: global_draft_state.clone(),
                        };
                    } else {
                        state = GameState::Playing;
                    }
                }

                next_frame().await;
                continue;
            }
            GameState::DraftSelection {
                ref mut draft_state,
            } => {
                // 更新动画计时器
                draft_state.update(raw_dt);

                // 输入处理：左右键切换卡牌
                if input_state.is_key_pressed(KeyCode::Left)
                    || input_state.is_key_pressed(KeyCode::A)
                {
                    draft_state.move_selection(-1);
                }
                if input_state.is_key_pressed(KeyCode::Right)
                    || input_state.is_key_pressed(KeyCode::D)
                {
                    draft_state.move_selection(1);
                }

                // Enter/Space 确认选择
                if input_state.is_key_pressed(KeyCode::Enter)
                    || input_state.is_key_pressed(KeyCode::Space)
                {
                    if let Some(card) = draft_state.finish_draft() {
                        // 同步到全局状态
                        global_draft_state.ufo_triggers_used = draft_state.ufo_triggers_used;

                        // 为所有玩家应用卡牌效果
                        for player in players.iter_mut() {
                            player.apply_draft_card(card);
                        }

                        // 如果是 ExtraLife 卡牌，播放音效
                        if matches!(card, Card::ExtraLife) {
                            sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                        }

                        // 进入游戏状态
                        state = GameState::Playing;
                    }
                    next_frame().await;
                    continue;
                }

                // 绘制背景（星空 + 游戏实体）
                starfield.draw(frame_t as f32);

                // 绘制选卡界面
                draw_draft_ui(draft_state, fonts.get_best(settings.font_choice));

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
                    GameMode::Roguelike => {
                        if victory {
                            "Run complete! You conquered all zones! Press [Enter] to restart"
                                .to_string()
                        } else {
                            "Run ended. Press [Enter] to try again".to_string()
                        }
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
                        roguelike_state: None,
                    };
                    next_frame().await;
                    continue;
                }
            }
            // Roguelike 模式状态处理
            GameState::RoguelikeRun { ref mut run_state } => {
                // 更新 Run 时间
                run_state.run_time += dt as f32;

                // 检查连击衰减
                run_state.check_combo_decay();

                // 检测暂停键
                let pause_pressed = input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::P);
                if pause_pressed {
                    state = GameState::Paused {
                        selection: PauseSelection::Resume,
                        roguelike_state: Some(run_state.clone()),
                    };
                    next_frame().await;
                    continue;
                }

                // 处理区域过渡
                if let roguelike::RunPhase::ZoneTransition {
                    from,
                    to,
                    ref mut timer,
                } = run_state.phase
                {
                    *timer -= dt as f32;
                    roguelike::draw_zone_transition(from, to, *timer);
                    if *timer <= 0.0 {
                        run_state.complete_zone_transition(to);
                        // 生成新区域的小行星
                        let asteroid_count = run_state.current_asteroid_count();
                        let difficulty = run_state.current_difficulty();
                        asteroids = spawn_wave_with_speed(
                            Vec2::new(screen_width() / 2., screen_height() / 2.),
                            screen_width().min(screen_height()),
                            asteroid_count,
                            difficulty * settings.asteroid_speed_multiplier,
                        );
                    }
                    next_frame().await;
                    continue;
                }

                // 处理胜利/失败
                if matches!(run_state.phase, roguelike::RunPhase::Victory) {
                    state = GameState::GameOver {
                        victory: true,
                        end_time: frame_t,
                    };
                    next_frame().await;
                    continue;
                }
                if matches!(run_state.phase, roguelike::RunPhase::Defeat) {
                    state = GameState::GameOver {
                        victory: false,
                        end_time: frame_t,
                    };
                    next_frame().await;
                    continue;
                }

                // 检测玩家死亡
                let all_dead = players.iter().all(|p| !p.alive);
                if all_dead {
                    run_state.defeat();
                    next_frame().await;
                    continue;
                }

                // 挑战超时判定
                if let Some(remaining) = run_state.challenge_time_remaining()
                    && remaining <= 0.0
                {
                    let challenge = run_state.take_active_challenge();
                    if let Some(ref c) = challenge {
                        run_state.apply_challenge_failure_penalty(c);
                    }
                    asteroids.clear();
                    let options = roguelike::generate_reward_options(run_state);
                    run_state.enter_reward_phase(options);
                    state = GameState::RoguelikeReward {
                        run_state: run_state.clone(),
                    };
                    next_frame().await;
                    continue;
                }

                // 检测波次清空
                if asteroids.is_empty() {
                    // 触发遗物效果
                    run_state.trigger_wave_clear();
                    // 每波奖励 5 金币
                    run_state.add_gold(5);

                    if let roguelike::RunPhase::Combat(_) = run_state.phase {
                        // 检查是否有挑战
                        if let Some(_challenge) = run_state.take_active_challenge() {
                            // 挑战成功
                            let options = roguelike::generate_challenge_reward_options(run_state);
                            run_state.enter_reward_phase(options);
                            state = GameState::RoguelikeReward {
                                run_state: run_state.clone(),
                            };
                        } else {
                            // 普通波次清空，进入挑战选择
                            run_state.enter_challenge_offer();
                            state = GameState::RoguelikeChallengeOffer {
                                run_state: run_state.clone(),
                            };
                        }
                        next_frame().await;
                        continue;
                    }
                }
                // HUD 在后面的 render_scene 之后统一绘制（第 3130 行）
            }
            GameState::RoguelikeChallengeOffer { ref mut run_state } => {
                // 确保处于挑战选择阶段
                if !matches!(run_state.phase, roguelike::RunPhase::ChallengeOffer(_)) {
                    run_state.enter_challenge_offer();
                }

                let action = if let roguelike::RunPhase::ChallengeOffer(ref challenge_state) =
                    run_state.phase
                {
                    ui::draw_challenge_offer(
                        challenge_state,
                        &input_state,
                        fonts.get_best(settings.font_choice),
                    )
                } else {
                    ui::ChallengeOfferAction::None
                };

                match action {
                    ui::ChallengeOfferAction::Accept => {
                        // 检查是否禁用护盾
                        let no_shield = if let roguelike::RunPhase::ChallengeOffer(ref c) =
                            run_state.phase
                        {
                            c.no_shield()
                        } else {
                            false
                        };

                        if no_shield {
                            for player in players.iter_mut() {
                                player.clear_shields();
                            }
                            powerups.retain(|p| {
                                !matches!(
                                    p.powerup_type,
                                    powerup::PowerUpType::Shield | powerup::PowerUpType::TempShield
                                )
                            });
                        }

                        run_state.start_challenge(run_state.run_time);
                        let asteroid_count = run_state.current_asteroid_count();
                        let difficulty = run_state.current_difficulty();
                        asteroids = spawn_wave_with_speed(
                            Vec2::new(screen_width() / 2., screen_height() / 2.),
                            screen_width().min(screen_height()),
                            asteroid_count,
                            difficulty * settings.asteroid_speed_multiplier,
                        );
                        state = GameState::RoguelikeRun {
                            run_state: run_state.clone(),
                        };
                    }
                    ui::ChallengeOfferAction::Skip => {
                        let options = roguelike::generate_reward_options(run_state);
                        run_state.enter_reward_phase(options);
                        state = GameState::RoguelikeReward {
                            run_state: run_state.clone(),
                        };
                    }
                    ui::ChallengeOfferAction::None => {}
                }

                next_frame().await;
                continue;
            }
            GameState::RoguelikeReward { ref mut run_state } => {
                // 设置引导屏幕
                tutorial_state.set_screen(tutorial::TutorialScreen::RoguelikeReward, frame_t);

                // 确保奖励选项已生成
                if !matches!(run_state.phase, roguelike::RunPhase::Reward(_)) {
                    let options = roguelike::generate_reward_options(run_state);
                    run_state.enter_reward_phase(options);
                }

                // 绘制奖励选择 UI
                if let roguelike::RunPhase::Reward(ref mut reward_state) = run_state.phase
                    && let Some(idx) = ui::draw_reward_selection(
                        reward_state,
                        &input_state,
                        fonts.get_best(settings.font_choice),
                    )
                {
                    // 应用选中的奖励
                    if let Some(reward) = reward_state.options.get(idx).cloned() {
                        // 播放选择音效
                        sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                        roguelike::apply_reward_option(run_state, &mut players, &reward);
                        // 生成商店物品并进入商店
                        let items = roguelike::generate_shop_items(run_state);
                        run_state.enter_shop_phase(items);
                        state = GameState::RoguelikeShop {
                            run_state: run_state.clone(),
                        };
                    }
                }

                // 绘制引导提示
                tutorial_state.draw(frame_t, fonts.get_best(settings.font_choice));
                next_frame().await;
                continue;
            }
            GameState::RoguelikeShop { ref mut run_state } => {
                // 设置引导屏幕
                tutorial_state.set_screen(tutorial::TutorialScreen::RoguelikeShop, frame_t);

                // 确保商店物品已生成
                if !matches!(run_state.phase, roguelike::RunPhase::Shop(_)) {
                    let items = roguelike::generate_shop_items(run_state);
                    run_state.enter_shop_phase(items);
                }

                // 获取当前金币（用于 UI 显示）
                let current_gold = run_state.gold;
                let max_waves = run_state.zone.wave_count();

                // 绘制商店 UI 并获取操作
                let action = if let roguelike::RunPhase::Shop(ref mut shop_state) = run_state.phase
                {
                    ui::draw_shop_ui(
                        shop_state,
                        current_gold,
                        roguelike::SHOP_REFRESH_COST,
                        &input_state,
                        fonts.get_best(settings.font_choice),
                    )
                } else {
                    ui::ShopUiAction::None
                };

                // 处理操作（在借用结束后）
                match action {
                    ui::ShopUiAction::BuyConfirmed(idx) => {
                        if roguelike::buy_shop_item(run_state, &mut players, idx) {
                            // 购买成功，播放音效
                            sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                        }
                    }
                    ui::ShopUiAction::RefreshRequested => {
                        if roguelike::refresh_shop(run_state) {
                            // 刷新成功，播放音效
                            sounds.play(SoundEffect::Hit, settings.sound_volume * 0.5);
                        }
                    }
                    ui::ShopUiAction::ExitShop => {
                        // 从 ShopPhaseState 获取波次信息
                        let (wave_at_enter, max_waves_at_enter) =
                            if let roguelike::RunPhase::Shop(ref shop) = run_state.phase {
                                (shop.wave_at_enter, shop.max_waves_at_enter)
                            } else {
                                (1, max_waves)
                            };
                        let is_last_wave = wave_at_enter >= max_waves_at_enter;

                        // 先将 phase 恢复为 Combat，否则 advance_wave() 不会生效
                        run_state.phase =
                            roguelike::RunPhase::Combat(roguelike::CombatPhaseState {
                                wave_in_zone: wave_at_enter,
                                enemies_remaining: 0,
                                spawn_timer: 0.0,
                                wave_start_time: 0.0,
                                challenge: None,
                            });

                        if is_last_wave {
                            // 最后一波完成，进入 Boss 战
                            run_state.advance_wave();
                            state = GameState::RoguelikeBoss {
                                run_state: run_state.clone(),
                            };
                        } else {
                            // 还有更多波次，进入下一波
                            run_state.advance_wave();
                            let asteroid_count = run_state.current_asteroid_count();
                            let difficulty = run_state.current_difficulty();
                            asteroids = spawn_wave_with_speed(
                                Vec2::new(screen_width() / 2., screen_height() / 2.),
                                screen_width().min(screen_height()),
                                asteroid_count,
                                difficulty * settings.asteroid_speed_multiplier,
                            );
                            state = GameState::RoguelikeRun {
                                run_state: run_state.clone(),
                            };
                        }
                    }
                    ui::ShopUiAction::None => {}
                }

                // 绘制引导提示
                tutorial_state.draw(frame_t, fonts.get_best(settings.font_choice));
                next_frame().await;
                continue;
            }
            GameState::RoguelikeRest { ref mut run_state } => {
                // 设置引导屏幕
                tutorial_state.set_screen(tutorial::TutorialScreen::RoguelikeRest, frame_t);

                // 确保休息选项已生成
                if !matches!(run_state.phase, roguelike::RunPhase::Rest(_)) {
                    let options = roguelike::generate_rest_options(&players);
                    run_state.enter_rest_phase(options);
                }

                // 绘制休息 UI 并获取操作
                let action = if let roguelike::RunPhase::Rest(ref mut rest_state) = run_state.phase
                {
                    ui::draw_rest_ui(
                        rest_state,
                        &players,
                        fonts.get_best(settings.font_choice),
                    )
                } else {
                    ui::RestUiAction::None
                };

                // 处理操作
                match action {
                    ui::RestUiAction::SelectOption(idx) => {
                        if let roguelike::RunPhase::Rest(ref mut rest_state) = run_state.phase
                            && idx < rest_state.options.len()
                        {
                            rest_state.selected = Some(idx);
                        }
                    }
                    ui::RestUiAction::ConfirmRest => {
                        if let roguelike::RunPhase::Rest(ref rest_state) = run_state.phase
                            && let Some(selected_idx) = rest_state.selected
                        {
                            let option = rest_state.options[selected_idx];
                            let selected_card = rest_state.card_selection;

                            if roguelike::apply_rest_option(run_state, &mut players, option, selected_card) {
                                // 应用成功，播放音效
                                sounds.play(SoundEffect::PowerUp, settings.sound_volume);

                                // 调用 advance_zone() 进入区域过渡
                                run_state.advance_zone();

                                // 根据新的 phase 切换 GameState
                                match &run_state.phase {
                                    roguelike::RunPhase::ZoneTransition { .. } => {
                                        // 进入区域过渡动画，由 RoguelikeRun 处理
                                        state = GameState::RoguelikeRun {
                                            run_state: run_state.clone(),
                                        };
                                    }
                                    roguelike::RunPhase::Victory => {
                                        // 已完成所有区域，进入胜利界面
                                        state = GameState::RoguelikeVictory {
                                            run_state: run_state.clone(),
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    ui::RestUiAction::None => {}
                    ui::RestUiAction::SelectCard(_) => {}
                }

                // 绘制引导提示
                tutorial_state.draw(frame_t, fonts.get_best(settings.font_choice));
                next_frame().await;
                continue;
            }
            GameState::RoguelikeBoss { ref mut run_state } => {
                // 更新 Run 时间
                run_state.run_time += dt as f32;

                // 检查连击衰减
                run_state.check_combo_decay();

                // 检测玩家全灭
                let all_dead = players.iter().all(|p| !p.alive);
                if all_dead {
                    run_state.defeat();
                    state = GameState::GameOver {
                        victory: false,
                        end_time: frame_t,
                    };
                    next_frame().await;
                    continue;
                }

                if let roguelike::RunPhase::Boss(ref mut boss) = run_state.phase {
                    // 初始化 Boss 位置（首次进入时）
                    if boss.position == Vec2::ZERO {
                        boss.set_position(Vec2::new(screen_width() * 0.5, screen_height() * 0.25));
                    }

                    // 检查狂暴状态（在 AI 更新前，使本帧立即生效）
                    boss.check_enrage();

                    // Boss AI：移动追踪 + 召唤（根据boss类型不同）
                    roguelike::update_boss(boss, &players, &mut asteroids, &mut ufos, dt as f32);

                    // 子弹与 Boss 碰撞检测
                    let boss_pos = boss.position;
                    let boss_r = roguelike::boss_radius(boss);
                    let damage_per_hit: f32 = 20.0;

                    for player in players.iter_mut() {
                        for bullet in player.bullets.iter_mut() {
                            if bullet.collided {
                                continue;
                            }
                            if (boss_pos - bullet.pos).length() < boss_r + BULLET_RADIUS {
                                boss.health = (boss.health - damage_per_hit).max(0.0);
                                bullet.collided = true;
                                particles.spawn_explosion(
                                    bullet.pos,
                                    10.0,
                                    player.color,
                                    frame_t as f32,
                                );
                                sounds.play(SoundEffect::Hit, settings.sound_volume);
                            }
                        }
                    }

                    // Boss 击败检测
                    if boss.health <= 0.0 {
                        // 触发遗物效果
                        run_state.trigger_boss_defeat();
                        // 清空召唤的小行星
                        asteroids.clear();
                        // 进入休息阶段或胜利
                        if run_state.zone.next().is_some() {
                            // 还有下一区域，进入休息阶段
                            let options = roguelike::generate_rest_options(&players);
                            run_state.enter_rest_phase(options);
                            state = GameState::RoguelikeRest {
                                run_state: run_state.clone(),
                            };
                        } else {
                            // 已完成所有区域，胜利
                            run_state.phase = roguelike::RunPhase::Victory;
                            state = GameState::RoguelikeVictory {
                                run_state: run_state.clone(),
                            };
                        }
                        next_frame().await;
                        continue;
                    }

                    // 调试：按 K 键模拟对 Boss 造成伤害（测试用）
                    #[cfg(debug_assertions)]
                    if input_state.is_key_pressed(KeyCode::K) {
                        boss.health = (boss.health - 100.0).max(0.0);
                    }
                }

                // 检测暂停键
                let pause_pressed = input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::P);
                if pause_pressed {
                    state = GameState::Paused {
                        selection: PauseSelection::Resume,
                        roguelike_state: Some(run_state.clone()),
                    };
                    next_frame().await;
                    continue;
                }
            }
            GameState::RoguelikeVictory { ref run_state } => {
                // 胜利界面：等待玩家按键返回主菜单

                // 先提取需要显示的数据
                let total_kills = run_state.total_kills;
                let max_combo = run_state.max_combo;
                let run_time = run_state.run_time;
                let gold = run_state.gold;

                // 检测返回主菜单的按键
                let should_return = input_state.is_key_pressed(KeyCode::Enter)
                    || input_state.is_key_pressed(KeyCode::Escape)
                    || input_state.is_key_pressed(KeyCode::Space);

                if should_return {
                    // 返回主菜单
                    state = GameState::ModeSelection {
                        selection: GameMode::Roguelike,
                    };
                    // 重置游戏状态
                    players.clear();
                    asteroids.clear();
                    ufos.clear();
                    powerups.clear();
                    enemy_bullets.clear();
                    next_frame().await;
                    continue;
                }

                // 绘制背景星空
                starfield.draw(frame_t as f32);

                // 直接在此处渲染胜利界面（因为 continue 会跳过后面的渲染）
                roguelike::draw_run_hud(run_state, fonts.get_best(settings.font_choice));

                // 显示胜利信息
                let victory_text = "胜利！你征服了所有区域！";
                let tw = measure_text(victory_text, None, 48, 1.0).width;
                draw_text_ex(
                    victory_text,
                    screen_width() / 2.0 - tw / 2.0,
                    screen_height() / 2.0 - 60.0,
                    TextParams {
                        font_size: 48,
                        color: GOLD,
                        ..Default::default()
                    },
                );

                // 显示统计信息
                let stats = format!(
                    "总击杀: {}  最高连击: {}  用时: {:.1}s  金币: {}",
                    total_kills,
                    max_combo,
                    run_time,
                    gold
                );
                let sw = measure_text(&stats, None, 24, 1.0).width;
                draw_text(&stats, screen_width() / 2.0 - sw / 2.0, screen_height() / 2.0, 24.0, WHITE);

                // 提示返回
                let hint = "按 [Enter] 或 [Escape] 返回主菜单";
                let hw = measure_text(hint, None, 20, 1.0).width;
                draw_text(hint, screen_width() / 2.0 - hw / 2.0, screen_height() / 2.0 + 60.0, 20.0, LIGHTGRAY);

                next_frame().await;
                continue;
            }
            GameState::Paused {
                selection,
                ref roguelike_state,
            } => {
                let duel_view = matches!(current_mode, GameMode::Duel).then_some(&duel_state);
                let active_particles = particles.update_and_get_active(dt, frame_t as f32);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &active_particles,
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
                    &chain_lightnings,
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
                            // 恢复到正确的状态
                            if let Some(run_state) = roguelike_state.clone() {
                                // 根据 RunPhase 恢复到对应的 GameState
                                state = match run_state.phase {
                                    roguelike::RunPhase::Boss(_) => {
                                        GameState::RoguelikeBoss { run_state }
                                    }
                                    _ => GameState::RoguelikeRun { run_state },
                                };
                            } else {
                                state = GameState::Playing;
                            }
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
                    // 恢复到正确的状态
                    if let Some(run_state) = roguelike_state.clone() {
                        // 根据 RunPhase 恢复到对应的 GameState
                        state = match run_state.phase {
                            roguelike::RunPhase::Boss(_) => GameState::RoguelikeBoss { run_state },
                            _ => GameState::RoguelikeRun { run_state },
                        };
                    } else {
                        state = GameState::Playing;
                    }
                    next_frame().await;
                    continue;
                }

                if next_selection != selection {
                    state = GameState::Paused {
                        selection: next_selection,
                        roguelike_state: roguelike_state.clone(),
                    };
                }

                next_frame().await;
                continue;
            }
            GameState::VictoryPause { started_at } => {
                let duel_view = matches!(current_mode, GameMode::Duel).then_some(&duel_state);
                let active_particles = particles.update_and_get_active(dt, frame_t as f32);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &active_particles,
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
                    &chain_lightnings,
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
                let active_particles = particles.update_and_get_active(dt, frame_t as f32);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &active_particles,
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
                    &chain_lightnings,
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
                    // 生成客户端令牌用于基本身份验证
                    let client_token = format!("client_{}_{}", frame_t as u64, rand::gen_range(100000u32, 999999));
                    // 默认使用 Survival 模式
                    if let Some(net_mode) =
                        network::NetworkGameMode::from_game_mode(GameMode::Survival)
                    {
                        if let Err(err) = network_client.send(network::ClientMessage::JoinQueue {
                            mode: net_mode,
                            nickname,
                            token: client_token,
                        }) {
                            eprintln!("[网络] 加入队列发送失败: {}", err);
                        } else {
                            current_mode = GameMode::Survival;
                            state = GameState::OnlineWaiting { room_id: 0 };
                        }
                    }
                }

                // ESC 返回主菜单
                if input_state.is_key_pressed(KeyCode::Escape) {
                    network_client.disconnect();
                    state = GameState::ModeSelection {
                        selection: GameMode::Online,
                    };
                    next_frame().await;
                    continue;
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
                    if let Err(err) = network_client.send(network::ClientMessage::LeaveQueue) {
                        eprintln!("[网络] 离开队列发送失败: {}", err);
                    }
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
                            if let Err(err) = network_client.send(network::ClientMessage::Ready) {
                                eprintln!("[网络] Ready 发送失败: {}", err);
                            }
                        }
                        network::ServerMessage::GameStart => {
                            // 游戏开始 - 初始化游戏状态
                            println!("游戏开始!");
                            is_online_mode = true;
                            online_bullets.clear();
                            pending_hits.clear(); // 重置乐观命中提示

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
                                        phase_dash: KeyCode::E, // 相位闪现：E
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

        // ========== 在线模式：输入同步与客户端预测 ==========
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

            // 发送输入到服务器（使用 send_input 支持客户端预测）
            // 记录当前帧的 dt，用于重播时保持物理一致性
            let input_timestamp = get_time();
            if let Err(err) = network_client.send_input(keys_pressed.clone(), input_timestamp, dt)
                && !matches!(err, network::NetworkSendError::RateLimited(_))
            {
                eprintln!("[网络] 输入发送失败: {}", err);
            }

            // === 客户端预测：本地立即应用输入 ===
            // 在等待服务器响应期间，先在本地应用输入，减少感知延迟
            if let Some(local_player) = players.get_mut(0)
                && local_player.alive
            {
                let parsed_input = network::ParsedInput::from_keys(&keys_pressed);
                apply_predicted_input(local_player, &parsed_input, dt, input_timestamp);
            }

            // 接收服务器的游戏状态更新
            while let Some(message) = network_client.receive() {
                match message {
                    network::ServerMessage::GameState {
                        players: server_players,
                        asteroids: server_asteroids,
                        bullets: server_bullets,
                        vortices: server_vortices,
                        powerups: server_powerups,
                        last_input_seqs,
                        timestamp,
                    } => {
                        // === 插值系统：记录服务器快照 ===
                        // 校准时钟（首次）并记录快照用于远程实体插值
                        interp_manager.align_clock(timestamp, frame_t);
                        interp_manager.record_server_snapshot(
                            timestamp,
                            &server_players,
                            &server_asteroids,
                            &server_bullets,
                            network_client.player_id.as_deref(),
                        );

                        // 按玩家ID更新本地玩家状态（避免HashMap顺序不稳定导致的错位）
                        // 在线模式下，players[0] 是本地玩家，需要用 player_id 找到对应的服务器数据
                        if let Some(my_id) = network_client.player_id.clone() {
                            // 找到服务器返回的本玩家数据
                            if let Some(my_server_data) =
                                server_players.iter().find(|p| p.id == my_id)
                            {
                                // === 服务器协调：获取未确认的输入 ===
                                // 如果服务器尚未开始处理输入（last_input_seqs 无本玩家），
                                // 则不进行 reconcile，直接使用服务器状态
                                let replay_inputs =
                                    if let Some(&server_seq) = last_input_seqs.get(&my_id) {
                                        network_client.reconcile(server_seq, my_server_data)
                                    } else {
                                        // 服务器尚未确认任何输入，清空待确认队列避免累积
                                        // 这通常发生在刚加入游戏或服务器重启时
                                        network_client.reset_prediction_state();
                                        Vec::new()
                                    };

                                if !players.is_empty() {
                                    let local_player = &mut players[0];

                                    // 应用服务器权威状态作为基准
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

                                    // === 重播未确认输入，生成新的预测状态 ===
                                    // 这确保了本地状态与服务器状态一致，同时应用了服务器尚未处理的输入
                                    // 使用输入记录时的 dt 来保持物理一致性
                                    if !replay_inputs.is_empty() && local_player.alive {
                                        for cmd in replay_inputs.iter() {
                                            let parsed = network::ParsedInput::from_keys(&cmd.keys);
                                            // 使用记录的 dt，如果无效则回退到当前帧 dt
                                            let replay_dt = if cmd.dt > 0.0 && cmd.dt < 0.1 {
                                                cmd.dt
                                            } else {
                                                dt
                                            };
                                            apply_predicted_input(
                                                local_player,
                                                &parsed,
                                                replay_dt,
                                                cmd.timestamp,
                                            );
                                        }
                                    }
                                }
                            }

                            // 同步其他玩家（对手）用于渲染
                            // 如果 players 只有一个本地玩家，需要添加对手
                            for server_player in server_players.iter() {
                                if server_player.id != my_id {
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
                                                phase_dash: KeyCode::Unknown,
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
                                    asteroid_type: asteroid::AsteroidType::Normal,
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

                        // === 乐观命中确认 (Phase 4C) ===
                        // 检查待确认的命中是否已被服务器确认（目标消失）
                        let server_asteroid_positions: Vec<Vec2> = server_asteroids
                            .iter()
                            .map(|a| Vec2::new(a.x, a.y))
                            .collect();
                        let server_ufo_positions: Vec<Vec2> = ufos.iter().map(|u| u.pos).collect();

                        // 遍历所有待确认命中，检查目标是否仍存在
                        for hit in pending_hits.get_all().to_vec() {
                            if hit.state != effects::PendingHitState::Pending {
                                continue;
                            }

                            let target_still_exists = match hit.kind {
                                PendingHitKind::Asteroid => server_asteroid_positions
                                    .iter()
                                    .any(|pos| (*pos - hit.pos).length() < 30.0),
                                PendingHitKind::Ufo => server_ufo_positions
                                    .iter()
                                    .any(|pos| (*pos - hit.pos).length() < 35.0),
                            };

                            // 如果目标已消失，确认命中
                            if !target_still_exists {
                                pending_hits.confirm(hit.id, frame_t as f32);
                            }
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
                        WeaponType::Homing => WeaponType::ChainIon,
                        WeaponType::ChainIon => WeaponType::Normal,
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

        particles.update_and_get_active(dt, frame_t as f32);

        // 更新链式闪电效果
        chain_lightnings.update(frame_t as f32);

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
                    let lives_before = player.lives;
                    player.mark_dead(frame_t);
                    // Roguelike：受伤处理（重置连击 + Boss 战标记）
                    if player.lives < lives_before {
                        match &mut state {
                            GameState::RoguelikeBoss { run_state } => {
                                run_state.mark_boss_damage();
                            }
                            GameState::RoguelikeRun { run_state } => {
                                run_state.on_player_damage();
                            }
                            _ => {}
                        }
                    }
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
                    let lives_before = player.lives;
                    player.mark_dead(frame_t);
                    // Roguelike：受伤处理（重置连击 + Boss 战标记）
                    if player.lives < lives_before {
                        match &mut state {
                            GameState::RoguelikeBoss { run_state } => {
                                run_state.mark_boss_damage();
                            }
                            GameState::RoguelikeRun { run_state } => {
                                run_state.on_player_damage();
                            }
                            _ => {}
                        }
                    }
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
                    let lives_before = player.lives;
                    player.mark_dead(frame_t);
                    // Roguelike：受伤处理（重置连击 + Boss 战标记）
                    if player.lives < lives_before {
                        match &mut state {
                            GameState::RoguelikeBoss { run_state } => {
                                run_state.mark_boss_damage();
                            }
                            GameState::RoguelikeRun { run_state } => {
                                run_state.on_player_damage();
                            }
                            _ => {}
                        }
                    }
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

        // 相位闪现爆炸伤害处理
        for player in players.iter_mut() {
            let phase_explosions = player.drain_phase_explosions(frame_t);
            for explosion in phase_explosions {
                // 检测爆炸范围内的小行星
                for asteroid in asteroids.iter_mut() {
                    if asteroid.collided {
                        continue;
                    }
                    let dist = (asteroid.pos - explosion.pos).length();
                    if dist <= explosion.radius + asteroid.size {
                        asteroid.collided = true;
                        // 生成爆炸粒子效果
                        particles.spawn_explosion(
                            asteroid.pos,
                            asteroid.size,
                            SKYBLUE,
                            frame_t as f32,
                        );
                    }
                }
                // 生成爆炸视觉效果
                particles.spawn_explosion(
                    explosion.pos,
                    explosion.radius * 0.5,
                    SKYBLUE,
                    frame_t as f32,
                );
            }
        }

        // 子弹与小行星碰撞检测（使用 QuadTree）
        // 收集小行星击杀信息：(player_idx, asteroid_idx, score_value, asteroid_pos, asteroid_size, player_color, bullet_vel, is_chain_hit)
        #[allow(clippy::type_complexity)]
        let mut asteroid_hits: Vec<(usize, usize, u32, Vec2, f32, Color, Vec2, bool)> = Vec::new();

        for (player_idx, player) in players.iter_mut().enumerate() {
            for bullet in player.bullets.iter_mut() {
                if bullet.collided {
                    continue;
                }

                // 查询附近的小行星（重用预分配的 Vec）
                bullet_query.clear();
                quadtree.query(bullet.pos, BULLET_RADIUS * 3.0, &mut bullet_query);

                for obj in bullet_query.iter() {
                    // 先检查碰撞条件（只读访问）
                    let hit_asteroid_idx = obj.index;
                    let asteroid_collided = asteroids[hit_asteroid_idx].collided;
                    let asteroid_pos = asteroids[hit_asteroid_idx].pos;
                    let asteroid_size = asteroids[hit_asteroid_idx].size;

                    if asteroid_collided {
                        continue;
                    }

                    if (asteroid_pos - bullet.pos).length() < asteroid_size {
                        // === 链式离子炮：先查找链式目标（需要不可变借用） ===
                        let chain_targets = if bullet.weapon_type == WeaponType::ChainIon {
                            chain_lightning::find_chain_asteroid_targets(
                                asteroid_pos,
                                hit_asteroid_idx,
                                &asteroids,
                                constants::chain_ion::MAX_JUMPS - 1,
                                constants::chain_ion::RANGE,
                            )
                        } else {
                            Vec::new()
                        };

                        // 现在可以安全地进行可变借用
                        let asteroid = &mut asteroids[hit_asteroid_idx];
                        asteroid.collided = true;
                        asteroid_hits.push((
                            player_idx,
                            hit_asteroid_idx,
                            asteroid.score_value(),
                            asteroid.pos,
                            asteroid.size,
                            player.color,
                            bullet.vel,
                            false, // 非链式命中
                        ));

                        // 记录击杀（仅在 Duel 模式下）
                        if matches!(current_mode, GameMode::Duel) {
                            player_kills[player_idx] += 1;
                        }

                        if let Some(split) = asteroid.split(bullet.vel) {
                            new_asteroids.extend(split);
                        }

                        // === 处理链式攻击目标 ===
                        if !chain_targets.is_empty() {
                            // 收集链式路径节点（用于视觉效果）
                            let mut chain_nodes = vec![asteroid_pos];

                            for (hop, &target_idx) in chain_targets.iter().enumerate() {
                                let target_asteroid = &mut asteroids[target_idx];
                                if target_asteroid.collided {
                                    continue;
                                }

                                // 计算链式伤害比例
                                let damage_ratio = chain_lightning::damage_ratio(hop);
                                let chain_score =
                                    (target_asteroid.score_value() as f32 * damage_ratio) as u32;

                                target_asteroid.collided = true;
                                chain_nodes.push(target_asteroid.pos);

                                // 记录链式命中
                                asteroid_hits.push((
                                    player_idx,
                                    target_idx,
                                    chain_score,
                                    target_asteroid.pos,
                                    target_asteroid.size,
                                    player.color,
                                    bullet.vel,
                                    true, // 链式命中
                                ));

                                // Duel 模式记录击杀
                                if matches!(current_mode, GameMode::Duel) {
                                    player_kills[player_idx] += 1;
                                }

                                // 分裂小行星
                                if let Some(split) = target_asteroid.split(bullet.vel) {
                                    new_asteroids.extend(split);
                                }
                            }

                            // 生成链式闪电视觉效果
                            if chain_nodes.len() > 1 {
                                chain_lightnings.spawn_path(
                                    chain_nodes,
                                    frame_t as f32,
                                    player.color,
                                );
                            }
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
            _is_chain_hit,
        ) in asteroid_hits
        {
            // Roguelike：击杀事件（用于连击/遗物效果）
            if let GameState::RoguelikeRun { run_state } | GameState::RoguelikeBoss { run_state } =
                &mut state
            {
                run_state.record_kill();
            }

            // 应用连击倍数计算最终得分
            let multiplier = players[player_idx].score_multiplier();
            let final_score = (score_value as f32 * multiplier) as u32;

            if players[player_idx].add_score(final_score) {
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
                    // 大型小行星爆炸触发命中停顿（可在设置中关闭）
                    if settings.enable_hit_stop {
                        hit_stop_effect.trigger(frame_t as f32, hit_stop::LARGE_ASTEROID);
                    }
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

            // 应用连击倍数计算最终得分
            let multiplier = players[player_idx].score_multiplier();
            let final_score = (score_value as f32 * multiplier) as u32;

            if players[player_idx].add_score(final_score) {
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

            // UFO 击杀触发选卡（仅限 Survival/TimeAttack 模式，且还有剩余选卡次数）
            if matches!(current_mode, GameMode::Survival | GameMode::TimeAttack)
                && global_draft_state.can_trigger_ufo_draft()
            {
                global_draft_state.start_draft(false); // UFO 选卡（非开局）
                state = GameState::DraftSelection {
                    draft_state: global_draft_state.clone(),
                };
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
                let pickups = powerup::handle_pickups(&mut players, &mut powerups, frame_t);
                if !pickups.is_empty() {
                    achievements.stats.shields_collected += pickups.len() as u32;
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                    // 为每个拾取的道具生成粒子效果
                    for pickup in pickups {
                        particles.spawn_powerup_pickup(pickup.pos, pickup.color, frame_t as f32);
                    }
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

                let pickups = powerup::handle_pickups(&mut players, &mut powerups, frame_t);
                if !pickups.is_empty() {
                    achievements.stats.shields_collected += pickups.len() as u32;
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
                    // 为每个拾取的道具生成粒子效果
                    for pickup in pickups {
                        particles.spawn_powerup_pickup(pickup.pos, pickup.color, frame_t as f32);
                    }
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
            GameMode::Roguelike => {
                // Roguelike 模式的游戏逻辑在 RoguelikeRun 状态中处理
                // 这里处理基础的道具生成和拾取
                powerup::spawn(
                    frame_t,
                    &mut powerups,
                    &mut next_shield_spawn,
                    &mut next_weapon_spawn,
                    settings.player_count,
                );
                let pickups = powerup::handle_pickups(&mut players, &mut powerups, frame_t);
                if !pickups.is_empty() {
                    // Roguelike：拾取事件（遗物效果）
                    if let GameState::RoguelikeRun { run_state }
                    | GameState::RoguelikeBoss { run_state } = &mut state
                    {
                        for _ in 0..pickups.len() {
                            run_state.trigger_pickup();
                        }
                    }
                    achievements.stats.shields_collected += pickups.len() as u32;
                    sounds.play(SoundEffect::PowerUp, settings.sound_volume);
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

        // === 在线模式：应用实体插值 ===
        // 在渲染前，用插值后的状态更新远程实体，实现平滑渲染
        if is_online_mode {
            let sampled = interp_manager.sample_world(frame_t);

            // 更新远程玩家（players[1..] 是远程玩家）
            if players.len() > 1 {
                for remote_player in sampled.remote_players.iter() {
                    // 查找对应的本地 Player 对象
                    for player in players.iter_mut().skip(1) {
                        // 注意：当前实现只支持一个对手，未来可以通过 ID 匹配
                        if player.alive || remote_player.alive {
                            player.ship.pos = remote_player.pos;
                            player.ship.rot = remote_player.rot;
                            player.ship.vel = remote_player.vel;
                            // 生死状态和分数仍从服务器直接同步（在 GameState 处理中）
                            break;
                        }
                    }
                }
            }

            // 使用插值后的子弹替换在线子弹列表（平滑子弹渲染）
            online_bullets.clear();
            for bullet in sampled.bullets.iter() {
                online_bullets.push(OnlineBullet {
                    x: bullet.pos.x,
                    y: bullet.pos.y,
                    vx: bullet.vel.x,
                    vy: bullet.vel.y,
                });
            }
        }

        // 计算性能统计
        let entity_count = asteroids.len() + players.iter().map(|p| p.bullets.len()).sum::<usize>();

        // 网络调试信息（仅在线模式）
        let network_debug = if is_online_mode {
            let interp_debug = interp_manager.debug_info().map(|info| InterpDebugStats {
                player_buffers: info.player_buffers,
                asteroid_buffers: info.asteroid_buffers,
                bullet_buffers: info.bullet_buffers,
                avg_player_snapshots: info.avg_player_snapshots,
                avg_bullet_snapshots: info.avg_bullet_snapshots,
                render_delay_ms: info.render_delay_ms,
            });

            Some(NetworkDebugStats {
                rtt_ms: network_client.latency_ms,
                pending_inputs: network_client.pending_input_count(),
                interp: interp_debug,
            })
        } else {
            None
        };

        let _debug_stats = DebugStats {
            fps: 1.0 / raw_dt, // 使用原始 dt 计算真实 FPS
            entity_count,
            quadtree_depth: quadtree.max_depth(),
            particle_count: particles.count(),
            network: network_debug,
        };

                let active_particles = particles.update_and_get_active(dt, frame_t as f32);
                render_scene(
                    &players,
                    &asteroids,
                    &ufos,
                    &enemy_bullets,
                    &powerups,
                    &active_particles,
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
                    &chain_lightnings,
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

        // Roguelike 模式 HUD 和 Boss 渲染（必须在 render_scene 之后）
        if matches!(current_mode, GameMode::Roguelike) {
            let shake_offset = screen_shake
                .filter(|s| s.is_active(frame_t as f32))
                .map(|s| s.get_offset(frame_t as f32))
                .unwrap_or(Vec2::ZERO);

            match &state {
                GameState::RoguelikeRun { run_state } => {
                    roguelike::draw_run_hud(run_state, fonts.get_best(settings.font_choice));
                }
GameState::RoguelikeBoss { run_state } => {
                    roguelike::draw_run_hud(run_state, fonts.get_best(settings.font_choice));
                    if let roguelike::RunPhase::Boss(boss) = &run_state.phase {
                        roguelike::draw_boss(boss, shake_offset, frame_t as f32);
                        roguelike::draw_boss_health_bar(boss);
                    }
                }
                GameState::RoguelikeVictory { run_state } => {
                    roguelike::draw_run_hud(run_state, fonts.get_best(settings.font_choice));
                    // 显示胜利信息
                    let victory_text = "胜利！你征服了所有区域！";
                    let tw = measure_text(victory_text, None, 48, 1.0).width;
                    draw_text_ex(
                        victory_text,
                        screen_width() / 2.0 - tw / 2.0,
                        screen_height() / 2.0 - 60.0,
                        TextParams {
                            font_size: 48,
                            color: GOLD,
                            ..Default::default()
                        },
                    );

                    // 显示统计信息
                    let stats = format!(
                        "总击杀: {}  最高连击: {}  用时: {:.1}s  金币: {}",
                        run_state.total_kills,
                        run_state.max_combo,
                        run_state.run_time,
                        run_state.gold
                    );
                    let sw = measure_text(&stats, None, 24, 1.0).width;
                    draw_text(&stats, screen_width() / 2.0 - sw / 2.0, screen_height() / 2.0, 24.0, WHITE);

                    // 提示返回
                    let hint = "按 [Enter] 或 [Escape] 返回主菜单";
                    let hw = measure_text(hint, None, 20, 1.0).width;
                    draw_text(hint, screen_width() / 2.0 - hw / 2.0, screen_height() / 2.0 + 60.0, 20.0, LIGHTGRAY);
                }
                _ => {}
            }
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

// ============================================================================
// 客户端预测辅助函数
// ============================================================================

/// 应用预测输入到玩家状态（用于本地预测和服务器协调后重播）
///
/// 此函数模拟单帧的物理更新，基于给定的输入状态。
/// 用于：
/// 1. 本地输入即时应用（减少感知延迟）
/// 2. 服务器状态协调后重播未确认的输入
fn apply_predicted_input(player: &mut Player, input: &network::ParsedInput, dt: f32, now: f64) {
    // 阻尼减速
    let mut acc = -player.ship.vel * SHIP_DAMPING;

    // 推进加速
    player.is_thrusting = input.thrust;
    if input.thrust {
        acc += player.ship.forward_vector() * SHIP_THRUST;
    }

    // 旋转（应用超速模式加成）
    let turn_rate = player.turn_rate(now);
    if input.right {
        player.ship.rot += turn_rate * dt;
    } else if input.left {
        player.ship.rot -= turn_rate * dt;
    }

    // 更新速度
    player.ship.vel += acc * dt;

    // 限制最大速度（应用超速模式加成）
    let max_speed = player.max_speed(now);
    if player.ship.vel.length() > max_speed {
        player.ship.vel = player.ship.vel.normalize() * max_speed;
    }

    // 更新位置
    player.ship.pos += player.ship.vel * dt;

    // 边界环绕
    player.ship.pos = wrap_around(&player.ship.pos);
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

        // 更新 Flux 能量（每帧自然回复）
        player.update_flux(dt);

        // 冲刺输入检测（需要冷却完成 + 足够 Flux）
        if input.is_key_pressed(player.controls.dash)
            && player.can_dash(frame_t)
            && player.can_flux_dash()
        {
            // 消耗 Flux
            player.spend_flux(crate::constants::flux::DASH_COST);
            // 冲刺方向：当前面向方向
            let dash_dir = player.ship.forward_vector();
            player.start_dash(frame_t, dash_dir);
            sounds.play(SoundEffect::Shoot, sound_volume * 0.5);
        }

        // 超空间跳跃输入检测（需要冷却完成 + 足够 Flux）
        if input.is_key_pressed(player.controls.hyperspace)
            && player.can_hyperspace(frame_t)
            && player.can_flux_hyperspace()
        {
            // 消耗 Flux
            player.spend_flux(crate::constants::flux::HYPERSPACE_COST);
            player.start_hyperspace(frame_t);
            sounds.play(SoundEffect::PowerUp, sound_volume * 0.7);
        }

        // 相位闪现输入检测（需要冷却完成 + 足够 Flux）
        if input.is_key_pressed(player.controls.phase_dash)
            && player.can_phase_dash(frame_t)
            && player.can_flux_phase_dash()
        {
            // 消耗 Flux
            player.spend_flux(crate::constants::flux::PHASE_DASH_COST);
            let (start_pos, end_pos) = player.start_phase_dash(frame_t);
            // 起点和终点特效
            particles.spawn_explosion(start_pos, 15.0, SKYBLUE, frame_t as f32);
            particles.spawn_explosion(end_pos, 20.0, SKYBLUE, frame_t as f32);
            sounds.play(SoundEffect::PowerUp, sound_volume * 0.6);
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

        // 更新相位闪现尾迹
        player.update_phase_trail(frame_t);

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
            // 应用加速度修改器
            let modified_thrust = player.modifiers.modified_acceleration(SHIP_THRUST);
            acc += player.ship.forward_vector() * modified_thrust * ship_speed_multiplier;
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

        // 应用转向速度修改器（卡牌加成 + 超速模式加成）
        let mut modified_turn_rate = player.modifiers.modified_turn_rate(SHIP_ROTATION_STEP);
        if player.overdrive_active(frame_t) {
            modified_turn_rate *= 1.6; // 超速模式：转向+60%
        }
        if input.is_key_down(player.controls.right) {
            player.ship.rot += modified_turn_rate * dt * ship_speed_multiplier;
        } else if input.is_key_down(player.controls.left) {
            player.ship.rot -= modified_turn_rate * dt * ship_speed_multiplier;
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
        // 应用连击速度加成和超速模式加成
        let max_speed = player.max_speed(frame_t);
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
    active_particles: &[(Vec2, f32, Color)],
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
    chain_lightnings: &chain_lightning::ChainLightningManager,
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
    for (pos, size, color) in active_particles.iter() {
        draw_circle(pos.x, pos.y, *size, *color);
    }

    // 绘制子弹（使用增强渲染）
    for player in players.iter() {
        for bullet in player.bullets.iter() {
            render::draw_bullet(bullet, shake_offset, player.color, frame_t as f32);
        }
    }

    // 绘制链式闪电效果
    chain_lightnings.draw(shake_offset, frame_t as f32);

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

        // 绘制相位闪现尾迹（使用天蓝色）
        let phase_trail = player.phase_trail_tuples(frame_t);
        if !phase_trail.is_empty() {
            render::draw_dash_trail(&phase_trail, SKYBLUE, frame_t);
        }

        // 连击发光效果（在飞船下层）
        let visual_level = player.killstreak_visual_level();
        if visual_level > 0 {
            render::draw_ship_glow(ship_pos, player.color, visual_level, frame_t as f32);
        }

        // 幽灵模式效果（在飞船下层）
        if player.ghost_mode_active(frame_t) {
            render::draw_ghost_mode_effect(ship_pos, player.ship.rot, player.color, frame_t as f32);
        }

        // 超速模式效果（在飞船下层）
        if player.overdrive_active(frame_t) {
            render::draw_overdrive_effect(ship_pos, player.ship.vel, frame_t as f32);
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

    // 连击屏幕边缘特效（取所有玩家中最高的连击等级）
    let max_visual_level = players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.killstreak_visual_level())
        .max()
        .unwrap_or(0);
    if max_visual_level >= constants::killstreak::VIGNETTE_THRESHOLD {
        // 计算强度：从阈值开始线性增加
        let level_above_threshold =
            (max_visual_level - constants::killstreak::VIGNETTE_THRESHOLD) as f32;
        let intensity =
            (level_above_threshold * 0.3 + 0.2).min(constants::killstreak::VIGNETTE_MAX_INTENSITY);
        // 使用第一个活着的玩家的颜色
        let vignette_color = players
            .iter()
            .find(|p| p.alive && p.killstreak_visual_level() == max_visual_level)
            .map(|p| p.color)
            .unwrap_or(WHITE);
        render::draw_killstreak_vignette(vignette_color, intensity, frame_t as f32);
    }

    if let Some(state) = duel_state
        && let Some(flag) = &state.flag
    {
        duel::draw_flag(flag, flag_radius);
    }

    ui::draw_players_hud(players, HudMode::Active { time: frame_t }, font);

    // 绘制 Flux 能量条
    ui::draw_flux_bar(players, font);

    // 绘制玩家状态效果图标栏（显示当前激活的道具/buff）
    ui::draw_player_buffs(players, frame_t, font);

    // 绘制连击计数器和分数倍率（所有模式通用）
    ui::draw_killstreak_counter(players, frame_t, font);

    // 在 Duel 模式下显示连击状态（额外的大字提示）
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
