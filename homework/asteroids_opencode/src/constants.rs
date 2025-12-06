//! Centralized gameplay tuning constants extracted from `main.rs` and related systems.
//!
//! Grouped to make camera feedback, progression, timing, UI and particle tweaks easy to find.
//!
//! Note: Some constants (especially UI) are defined for future refactoring use.

#![allow(dead_code)]

/// Screen shake and camera feedback tuning.
pub mod shake {
    /// Player death shake (intensity, duration).
    pub const PLAYER_DEATH: (f32, f32) = (4.0, 0.25);
    /// Large asteroid explosion shake (intensity, duration).
    pub const ASTEROID_LARGE: (f32, f32) = (6.0, 0.2);
    /// Medium asteroid explosion shake (intensity, duration).
    pub const ASTEROID_MEDIUM: (f32, f32) = (3.0, 0.12);
    /// High killstreak shake (intensity, duration).
    pub const KILLSTREAK_HIGH: (f32, f32) = (7.0, 0.3);
    /// Mid killstreak shake (intensity, duration).
    pub const KILLSTREAK_MID: (f32, f32) = (5.0, 0.2);
}

/// Slow motion effect parameters.
pub mod slow_motion {
    /// Strong slow-motion (duration, time_scale).
    pub const STRONG: (f32, f32) = (2.0, 0.4);
    /// Light slow-motion (duration, time_scale).
    pub const LIGHT: (f32, f32) = (1.5, 0.6);
}

/// Core gameplay tuning: streak bonuses, wave pacing, scoring.
pub mod gameplay {
    /// Starting asteroid count for wave one.
    pub const INITIAL_ASTEROID_COUNT: usize = 10;
    /// Asteroids added each new wave.
    pub const ASTEROID_WAVE_INCREMENT: usize = 2;
    /// Speed increase per wave (10%).
    pub const WAVE_SPEED_INCREMENT: f32 = 0.1;
    /// Maximum asteroid speed multiplier.
    pub const WAVE_SPEED_MAX_MULTIPLIER: f32 = 2.0;

    /// Score for destroying large asteroid (size >= 40).
    pub const SCORE_ASTEROID_LARGE: u32 = 20;
    /// Score for destroying medium asteroid (size >= 20).
    pub const SCORE_ASTEROID_MEDIUM: u32 = 50;
    /// Score for destroying small asteroid.
    pub const SCORE_ASTEROID_SMALL: u32 = 100;
    /// Size threshold for large asteroid.
    pub const ASTEROID_SIZE_LARGE: f32 = 40.0;
    /// Size threshold for medium asteroid.
    pub const ASTEROID_SIZE_MEDIUM: f32 = 20.0;

    /// Child asteroid size ratio when splitting.
    pub const ASTEROID_CHILD_SIZE_RATIO: f32 = 0.8;
}

/// Kill streak system parameters.
pub mod killstreak {
    /// Seconds before a killstreak resets.
    pub const RESET_TIME: f64 = 5.0;
    /// Per-kill fire rate bonus (fractional cooldown reduction).
    pub const FIRE_RATE_BONUS: f64 = 0.15;
    /// Per-kill speed bonus in pixels/second.
    pub const SPEED_BONUS: f32 = 30.0;
}

/// 相位闪现（Phase Dash）参数
///
/// 与普通冲刺系统独立，是瞬移技能而非速度加速
pub mod phase_dash {
    /// 瞬移距离（像素，沿飞船朝向）
    pub const DISTANCE: f32 = 150.0;
    /// 冷却时间（秒）
    pub const COOLDOWN: f64 = 3.0;
    /// 无敌窗口时长（秒），用于穿透障碍
    pub const INVULNERABLE_WINDOW: f64 = 0.25;
    /// 残影可见时长（秒）
    pub const TRAIL_LIFETIME: f64 = 1.0;
    /// 延迟爆裂触发延时（秒）
    pub const EXPLOSION_DELAY: f64 = 1.0;
    /// 爆炸半径（像素）
    pub const EXPLOSION_RADIUS: f32 = 70.0;
    /// 爆炸伤害值（对小行星造成的伤害）
    pub const EXPLOSION_DAMAGE: u32 = 1;
    /// 残影采样步长（像素）
    pub const TRAIL_SAMPLE_STEP: f32 = 25.0;
    /// 相位半透明 alpha 值（闪现瞬间的透明度）
    pub const PHASE_ALPHA: f32 = 0.4;
}

/// Timing and pacing constants.
pub mod timing {
    /// Post-victory pause before next wave (seconds).
    pub const VICTORY_PAUSE: f64 = 2.0;
    /// Vortex spawn interval in Survival mode (seconds).
    pub const VORTEX_SPAWN_INTERVAL: f32 = 20.0;
}

/// Default game settings.
pub mod defaults {
    /// Default player starting lives.
    pub const LIVES: u32 = 3;
    /// Default ship speed multiplier.
    pub const SHIP_SPEED: f32 = 1.0;
    /// Default asteroid speed multiplier.
    pub const ASTEROID_SPEED: f32 = 1.0;
    /// Default master sound volume (0.0 - 1.0).
    pub const SOUND_VOLUME: f32 = 0.01;
    /// Default CTF flag radius for Duel mode.
    pub const FLAG_RADIUS: f32 = 90.0;
    /// Default window width in pixels.
    pub const WINDOW_WIDTH: i32 = 1024;
    /// Default window height in pixels.
    pub const WINDOW_HEIGHT: i32 = 768;
}

/// UI layout, sizing, and spacing.
pub mod ui {
    /// HUD base Y position.
    pub const HUD_BASE_Y: f32 = 32.0;
    /// HUD line height spacing.
    pub const HUD_LINE_HEIGHT: f32 = 36.0;
    /// HUD panel width as screen ratio.
    pub const HUD_PANEL_WIDTH_RATIO: f32 = 0.55;

    /// Waiting panel width ratio.
    pub const WAITING_PANEL_WIDTH_RATIO: f32 = 0.6;
    /// Waiting panel height.
    pub const WAITING_PANEL_HEIGHT: f32 = 160.0;
    /// Waiting screen font size.
    pub const WAITING_FONT_SIZE: u16 = 32;

    /// Game over banner width ratio.
    pub const GAMEOVER_BANNER_WIDTH_RATIO: f32 = 0.6;
    /// Game over banner height.
    pub const GAMEOVER_BANNER_HEIGHT: f32 = 130.0;
    /// Game over message font size.
    pub const GAMEOVER_FONT_SIZE: u16 = 36;

    /// Score panel line height.
    pub const SCORES_LINE_HEIGHT: f32 = 38.0;
    /// Score panel padding.
    pub const SCORES_PADDING: f32 = 50.0;

    /// Mode selection title font size.
    pub const TITLE_FONT_SIZE: u16 = 48;
    /// Mode selection subtitle font size.
    pub const SUBTITLE_FONT_SIZE: u16 = 24;

    /// Mode card width ratio.
    pub const CARD_WIDTH_RATIO: f32 = 0.65;
    /// Mode card height.
    pub const CARD_HEIGHT: f32 = 140.0;
    /// Mode card spacing.
    pub const CARD_SPACING: f32 = 18.0;

    /// Pause menu panel width ratio.
    pub const PAUSE_PANEL_WIDTH_RATIO: f32 = 0.55;
    /// Pause menu panel height.
    pub const PAUSE_PANEL_HEIGHT: f32 = 480.0;

    /// Background gradient steps for quality.
    pub const GRADIENT_STEPS: u32 = 32;

    /// Debug panel width.
    pub const DEBUG_PANEL_WIDTH: f32 = 280.0;
    /// Debug panel height.
    pub const DEBUG_PANEL_HEIGHT: f32 = 115.0;
    /// Debug panel left margin.
    pub const DEBUG_PANEL_MARGIN_X: f32 = 12.0;
    /// Debug panel bottom offset.
    pub const DEBUG_PANEL_BOTTOM_OFFSET: f32 = 130.0;
    /// Panel text padding.
    pub const PANEL_TEXT_PADDING: f32 = 10.0;

    /// Achievement toast vertical spacing.
    pub const ACHIEVEMENT_TOAST_SPACING: f32 = 110.0;
}

/// Powerup spawn and pickup parameters.
pub mod powerup {
    /// Shield spawn interval range (min, max) seconds.
    pub const SHIELD_SPAWN_INTERVAL: (f32, f32) = (3.0, 8.0);
    /// Weapon spawn interval range (min, max) seconds.
    pub const WEAPON_SPAWN_INTERVAL: (f32, f32) = (10.0, 20.0);
    /// Powerup visual radius.
    pub const RADIUS: f32 = 24.0;
    /// Powerup pickup radius.
    pub const PICKUP_RADIUS: f32 = 30.0;
}

/// Particle system counts and lifetime ranges.
pub mod particles {
    /// Maximum particle pool capacity.
    pub const MAX_PARTICLES: usize = 1000;
    /// Base explosion particle count (scaled by asteroid size).
    pub const EXPLOSION_BASE_COUNT: usize = 20;
    /// Thruster particles spawned per frame.
    pub const THRUSTER_COUNT: usize = 3;

    /// Explosion particle size range (min, max).
    pub const EXPLOSION_SIZE_RANGE: (f32, f32) = (1.0, 4.0);
    /// Explosion particle lifetime range in seconds.
    pub const EXPLOSION_LIFETIME_RANGE: (f32, f32) = (0.3, 0.8);
    /// Explosion particle speed range in pixels/second.
    pub const EXPLOSION_SPEED_RANGE: (f32, f32) = (50.0, 200.0);

    /// Thruster particle speed range in pixels/second.
    pub const THRUSTER_SPEED_RANGE: (f32, f32) = (80.0, 150.0);
    /// Thruster particle lifetime range in seconds.
    pub const THRUSTER_LIFETIME_RANGE: (f32, f32) = (0.15, 0.3);
    /// Thruster spread angle range.
    pub const THRUSTER_SPREAD_RANGE: (f32, f32) = (-0.3, 0.3);

    /// Bullet trail spawn chance (0.0-1.0).
    pub const BULLET_TRAIL_SPAWN_CHANCE: f32 = 0.5;
    /// Trail particle lifetime range in seconds.
    pub const TRAIL_LIFETIME_RANGE: (f32, f32) = (0.1, 0.25);
    /// Trail particle alpha.
    pub const TRAIL_ALPHA: f32 = 0.4;

    /// Particle gravity effect.
    pub const GRAVITY: f32 = 50.0;
}

/// Quadtree spatial partitioning parameters.
pub mod quadtree {
    /// Maximum objects per node before splitting.
    pub const MAX_OBJECTS: usize = 4;
    /// Maximum tree depth.
    pub const MAX_DEPTH: usize = 5;
}

/// Homing missile parameters.
pub mod homing {
    /// Missile speed in pixels/second (slower than normal bullets).
    pub const SPEED: f32 = 600.0;
    /// Maximum turn rate in radians/second.
    pub const TURN_RATE: f32 = 4.0;
    /// Missile lifetime in seconds (longer than normal bullets).
    pub const LIFETIME: f64 = 3.0;
    /// Cooldown between homing missile shots.
    pub const COOLDOWN: f64 = 1.2;
    /// Maximum tracking range in pixels.
    pub const TRACKING_RANGE: f32 = 500.0;
    /// Missile visual radius.
    pub const RADIUS: f32 = 5.0;
}

/// Chain ion shot parameters (链式离子炮).
pub mod chain_ion {
    /// Maximum total hits including the initial impact.
    pub const MAX_JUMPS: usize = 3;
    /// Maximum search distance per hop (pixels).
    pub const RANGE: f32 = 260.0;
    /// Damage ratios per hop (1st/2nd/3rd hit).
    /// First hit is 100%, second is 70%, third is 50%.
    pub const DAMAGE_RATIOS: [f32; 3] = [1.0, 0.7, 0.5];
    /// Lifetime of a chain arc visual effect (seconds).
    pub const ARC_LIFETIME: f32 = 0.35;
    /// Flash duration on struck targets (seconds).
    pub const FLASH_DURATION: f32 = 0.28;
    /// Visual jitter steps for the lightning line.
    pub const JITTER_STEPS: usize = 10;
    /// Maximum perpendicular jitter amplitude (pixels).
    pub const JITTER_AMPLITUDE: f32 = 12.0;
    /// Base line width for the arc.
    pub const LINE_WIDTH: f32 = 3.0;
    /// Cooldown between chain ion shots (seconds).
    pub const COOLDOWN: f64 = 0.4;
    /// Bullet lifetime multiplier (slightly longer than normal).
    pub const LIFETIME_MULTIPLIER: f64 = 1.2;
}
