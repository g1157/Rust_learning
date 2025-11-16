//! 音效系统模块
//!
//! 提供可选的游戏音效支持。
//!
//! ## 功能
//! - 射击、爆炸、击中、道具、推进器音效
//! - 异步加载音频文件（WAV/OGG）
//! - 优雅降级（无音频文件时静默运行）
//! - 音效开关
//! - 音频淡出（避免结尾电流声）
//!
//! ## 使用方法
//! 1. 在项目根目录创建 `assets/sounds/` 文件夹
//! 2. 添加音效文件：shoot.wav, explosion.wav, thrust.wav 等
//! 3. 运行 `python3 add_fadeout.py` 为音效添加淡出效果
//! 4. 在游戏初始化时调用 `SoundSystem::new().await` 加载音效
//!
//! 如果没有音频文件，音效系统会静默运行（不播放声音）

use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound};

/// 音效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    Shoot,
    Explosion,
    #[allow(dead_code)]
    Thruster,
    PowerUp,
    Hit,
}

/// 音效系统
pub struct SoundSystem {
    shoot: Option<Sound>,
    explosion: Option<Sound>,
    thruster: Option<Sound>,
    powerup: Option<Sound>,
    hit: Option<Sound>,
    enabled: bool,
}

impl SoundSystem {
    /// 创建新的音效系统（尝试加载音频文件）
    pub async fn new() -> Self {
        println!("正在加载音效文件...");

        let shoot = Self::load_optional("assets/sounds/shoot.wav").await;
        let explosion = Self::load_optional("assets/sounds/explosion.wav").await;
        let thruster = Self::load_optional("assets/sounds/thrust.wav").await;
        let powerup = Self::load_optional("assets/sounds/powerup.wav").await;
        let hit = Self::load_optional("assets/sounds/hit.wav").await;

        // 打印加载状态
        println!("音效加载状态:");
        println!(
            "  射击 (shoot.wav):      {}",
            if shoot.is_some() { "✓" } else { "✗" }
        );
        println!(
            "  爆炸 (explosion.wav):  {}",
            if explosion.is_some() { "✓" } else { "✗" }
        );
        println!(
            "  推进 (thrust.wav):     {}",
            if thruster.is_some() { "✓" } else { "✗" }
        );
        println!(
            "  道具 (powerup.wav):    {}",
            if powerup.is_some() { "✓" } else { "✗" }
        );
        println!(
            "  碰撞 (hit.wav):        {}",
            if hit.is_some() { "✓" } else { "✗" }
        );

        let enabled = shoot.is_some()
            || explosion.is_some()
            || thruster.is_some()
            || powerup.is_some()
            || hit.is_some();

        if enabled {
            println!("✅ 音效系统已启用（带淡出效果）");
        } else {
            println!("❌ 未找到音效文件，游戏将静音运行");
        }

        Self {
            shoot,
            explosion,
            thruster,
            powerup,
            hit,
            enabled,
        }
    }

    /// 创建静默的音效系统（用于测试或无音频环境）
    pub fn silent() -> Self {
        Self {
            shoot: None,
            explosion: None,
            thruster: None,
            powerup: None,
            hit: None,
            enabled: false,
        }
    }

    /// 尝试加载音频文件，失败时返回 None
    async fn load_optional(path: &str) -> Option<Sound> {
        load_sound(path).await.ok()
    }

    /// 播放音效（音频文件已包含淡出效果）
    pub fn play(&self, effect: SoundEffect, volume: f32) {
        if !self.enabled {
            return;
        }

        let sound = match effect {
            SoundEffect::Shoot => self.shoot.as_ref(),
            SoundEffect::Explosion => self.explosion.as_ref(),
            SoundEffect::Thruster => self.thruster.as_ref(),
            SoundEffect::PowerUp => self.powerup.as_ref(),
            SoundEffect::Hit => self.hit.as_ref(),
        };

        if let Some(sound) = sound {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0), // 限制音量范围 0.0-1.0
                },
            );
        }
    }

    /// 播放循环音效
    #[allow(dead_code)]
    pub fn play_looping(&self, effect: SoundEffect, volume: f32) {
        if !self.enabled {
            return;
        }

        let sound = match effect {
            SoundEffect::Thruster => self.thruster.as_ref(),
            _ => return,
        };

        if let Some(sound) = sound {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume,
                },
            );
        }
    }

    /// 检查音效系统是否启用
    #[cfg(test)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 切换音效开关
    #[cfg(test)]
    pub fn toggle(&mut self) {
        // 注意：这只是切换启用状态，不会停止正在播放的声音
        self.enabled = !self.enabled;
    }
}

impl Default for SoundSystem {
    fn default() -> Self {
        Self::silent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silent_sound_system() {
        let system = SoundSystem::silent();
        assert!(!system.is_enabled());
    }

    #[test]
    fn test_sound_system_toggle() {
        let mut system = SoundSystem::silent();
        assert!(!system.is_enabled());
        system.toggle();
        assert!(system.is_enabled());
        system.toggle();
        assert!(!system.is_enabled());
    }

    #[test]
    fn test_sound_effect_types() {
        // 确保所有音效类型都可以创建
        let effects = [
            SoundEffect::Shoot,
            SoundEffect::Explosion,
            SoundEffect::Thruster,
            SoundEffect::PowerUp,
            SoundEffect::Hit,
        ];
        assert_eq!(effects.len(), 5);
    }
}
