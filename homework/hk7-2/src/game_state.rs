/// 游戏状态管理模块
/// 定义游戏的各种状态和状态转换逻辑
use crate::entity::{PowerUp, Shape};
use macroquad::experimental::animation::AnimatedSprite;
use macroquad::prelude::*;
use macroquad_particles::Emitter;

/// 游戏状态枚举
/// 表示游戏的不同阶段
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// 主菜单状态
    /// 显示开始游戏和退出按钮
    MainMenu,

    /// 游戏进行中状态
    /// 玩家可以控制飞船，敌人生成和移动
    Playing,

    /// 暂停状态
    /// 游戏逻辑暂停，显示暂停提示
    Paused,

    /// 游戏结束状态
    /// 显示 Game Over 信息和最终分数
    GameOver,
}

impl GameState {
    /// 检查是否处于游戏中（非菜单、非暂停）
    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// 检查是否显示游戏场景（Playing 或 Paused）
    #[allow(dead_code)]
    pub fn should_render_game(&self) -> bool {
        matches!(self, GameState::Playing | GameState::Paused)
    }

    /// 获取状态名称（用于调试）
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            GameState::MainMenu => "Main Menu",
            GameState::Playing => "Playing",
            GameState::Paused => "Paused",
            GameState::GameOver => "Game Over",
        }
    }
}

/// 游戏核心数据结构
/// 包含所有游戏运行时需要的状态信息
pub struct Game {
    /// 当前游戏状态
    pub state: GameState,

    /// 玩家飞船
    pub player: Shape,

    /// 敌人列表
    pub enemies: Vec<Shape>,

    /// 子弹列表
    pub bullets: Vec<Shape>,

    /// 爆炸效果列表
    pub explosions: Vec<(Emitter, Vec2)>,

    /// 当前分数
    pub score: u32,

    /// 最高分
    pub high_score: u32,

    /// 方向修正因子（背景倾斜效果）
    pub direction_modifier: f32,

    /// 玩家飞船动画精灵
    pub ship_sprite: AnimatedSprite,

    /// 子弹动画精灵
    pub bullet_sprite: AnimatedSprite,

    /// 小型敌人动画精灵
    pub enemy_small_sprite: AnimatedSprite,

    /// 中型敌人动画精灵
    pub enemy_medium_sprite: AnimatedSprite,

    /// 大型敌人动画精灵
    pub enemy_big_sprite: AnimatedSprite,

    /// 剩余生命数
    pub lives: u32,

    /// 无敌时间计时器（碰撞后的无敌时间）
    pub invincible_timer: f32,

    /// 难度等级
    pub difficulty_level: u32,

    /// 道具列表
    pub powerups: Vec<PowerUp>,

    /// 当前武器类型（0=普通, 1=双倍火力, 2=三重火力）
    pub weapon_type: u32,

    /// 武器升级剩余时间
    pub weapon_timer: f32,

    /// 护盾剩余时间
    pub shield_timer: f32,

    /// 射击冷却计时器
    pub fire_cooldown: f32,

    /// 是否拥有快速射击
    pub rapid_fire: bool,

    /// 快速射击剩余时间
    pub rapid_fire_timer: f32,

    /// 最大生命值
    pub max_lives: u32,
}

impl Game {
    /// 创建新游戏实例
    pub fn new(
        ship_sprite: AnimatedSprite,
        bullet_sprite: AnimatedSprite,
        enemy_small_sprite: AnimatedSprite,
        enemy_medium_sprite: AnimatedSprite,
        enemy_big_sprite: AnimatedSprite,
    ) -> Self {
        use crate::config::{INITIAL_LIVES, PLAYER_MOVEMENT_SPEED, PLAYER_SIZE};
        use std::fs;

        let high_score = fs::read_to_string("highscore.dat")
            .ok()
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(0);

        Self {
            state: GameState::MainMenu,
            player: Shape {
                size: PLAYER_SIZE,
                speed: PLAYER_MOVEMENT_SPEED,
                x: screen_width() / 2.0,
                y: screen_height() / 2.0,
                collided: false,
                enemy_type: None,
            },
            enemies: vec![],
            bullets: vec![],
            explosions: vec![],
            score: 0,
            high_score,
            direction_modifier: 0.0,
            ship_sprite,
            bullet_sprite,
            enemy_small_sprite,
            enemy_medium_sprite,
            enemy_big_sprite,
            lives: INITIAL_LIVES,
            invincible_timer: 0.0,
            difficulty_level: 1,
            powerups: vec![],
            weapon_type: 0,
            weapon_timer: 0.0,
            shield_timer: 0.0,
            fire_cooldown: 0.0,
            rapid_fire: false,
            rapid_fire_timer: 0.0,
            max_lives: 5,
        }
    }

    /// 重置游戏状态（开始新游戏）
    pub fn reset(&mut self) {
        use crate::config::INITIAL_LIVES;

        self.enemies.clear();
        self.bullets.clear();
        self.explosions.clear();
        self.player.x = screen_width() / 2.0;
        self.player.y = screen_height() / 2.0;
        self.player.collided = false;
        self.score = 0;
        self.direction_modifier = 0.0;
        self.lives = INITIAL_LIVES;
        self.invincible_timer = 0.0;
        self.difficulty_level = 1;
        self.powerups.clear();
        self.weapon_type = 0;
        self.weapon_timer = 0.0;
        self.shield_timer = 0.0;
        self.fire_cooldown = 0.0;
        self.rapid_fire = false;
        self.rapid_fire_timer = 0.0;
        self.state = GameState::Playing;
    }

    /// 保存高分到文件
    pub fn save_high_score(&self) {
        use std::fs;
        if self.score == self.high_score {
            if let Err(e) = fs::write("highscore.dat", self.high_score.to_string()) {
                eprintln!("Failed to save high score: {}", e);
            }
        }
    }

    /// 检查玩家是否处于无敌状态（受伤无敌或护盾）
    pub fn is_invincible(&self) -> bool {
        self.invincible_timer > 0.0 || self.shield_timer > 0.0
    }

    /// 检查是否有护盾
    pub fn has_shield(&self) -> bool {
        self.shield_timer > 0.0
    }

    /// 玩家受伤（失去一条命）
    pub fn take_damage(&mut self) {
        use crate::config::INVINCIBLE_TIME;

        if !self.is_invincible() && self.lives > 0 {
            self.lives -= 1;
            self.invincible_timer = INVINCIBLE_TIME;

            if self.lives == 0 {
                self.save_high_score();
                self.state = GameState::GameOver;
            }
        }
    }

    /// 更新无敌时间计时器
    pub fn update_invincibility(&mut self, delta_time: f32) {
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= delta_time;
            if self.invincible_timer < 0.0 {
                self.invincible_timer = 0.0;
            }
        }
    }

    /// 获取当前难度的敌人生成概率阈值
    pub fn get_enemy_spawn_threshold(&self) -> i32 {
        use crate::config::ENEMY_SPAWN_THRESHOLD;
        // 随难度增加生成速率（最低85，保证不会太疯狂）
        (ENEMY_SPAWN_THRESHOLD - (self.difficulty_level as i32 * 2)).max(85)
    }

    /// 获取当前难度的敌人速度倍率
    pub fn get_enemy_speed_multiplier(&self) -> f32 {
        // 每个难度等级增加10%速度
        1.0 + (self.difficulty_level as f32 * 0.1)
    }

    /// 更新难度等级（基于分数）
    pub fn update_difficulty(&mut self) {
        use crate::config::{DIFFICULTY_INCREASE_SCORE, MAX_DIFFICULTY_LEVEL};

        let new_level = (self.score / DIFFICULTY_INCREASE_SCORE + 1).min(MAX_DIFFICULTY_LEVEL);
        if new_level > self.difficulty_level {
            self.difficulty_level = new_level;
        }
    }

    /// 更新所有计时器
    pub fn update_timers(&mut self, delta_time: f32) {
        // 更新无敌时间
        self.update_invincibility(delta_time);

        // 更新护盾时间
        if self.shield_timer > 0.0 {
            self.shield_timer -= delta_time;
            if self.shield_timer < 0.0 {
                self.shield_timer = 0.0;
            }
        }

        // 更新武器升级时间
        if self.weapon_timer > 0.0 {
            self.weapon_timer -= delta_time;
            if self.weapon_timer <= 0.0 {
                self.weapon_timer = 0.0;
                self.weapon_type = 0; // 恢复普通武器
            }
        }

        // 更新快速射击时间
        if self.rapid_fire_timer > 0.0 {
            self.rapid_fire_timer -= delta_time;
            if self.rapid_fire_timer <= 0.0 {
                self.rapid_fire_timer = 0.0;
                self.rapid_fire = false;
            }
        }

        // 更新射击冷却
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= delta_time;
            if self.fire_cooldown < 0.0 {
                self.fire_cooldown = 0.0;
            }
        }
    }

    /// 检查是否可以射击
    pub fn can_fire(&self) -> bool {
        self.fire_cooldown <= 0.0
    }

    /// 重置射击冷却
    pub fn reset_fire_cooldown(&mut self) {
        use crate::config::{NORMAL_FIRE_COOLDOWN, RAPID_FIRE_COOLDOWN};
        self.fire_cooldown = if self.rapid_fire {
            RAPID_FIRE_COOLDOWN
        } else {
            NORMAL_FIRE_COOLDOWN
        };
    }

    /// 收集道具
    pub fn collect_powerup(&mut self, powerup_type: crate::entity::PowerUpType) {
        use crate::config::{SHIELD_DURATION, WEAPON_UPGRADE_DURATION};
        use crate::entity::PowerUpType;

        match powerup_type {
            PowerUpType::Health => {
                if self.lives < self.max_lives {
                    self.lives += 1;
                }
            }
            PowerUpType::Shield => {
                self.shield_timer = SHIELD_DURATION;
            }
            PowerUpType::DoubleFire => {
                self.weapon_type = 1;
                self.weapon_timer = WEAPON_UPGRADE_DURATION;
            }
            PowerUpType::TripleFire => {
                self.weapon_type = 2;
                self.weapon_timer = WEAPON_UPGRADE_DURATION;
            }
            PowerUpType::RapidFire => {
                self.rapid_fire = true;
                self.rapid_fire_timer = WEAPON_UPGRADE_DURATION;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_checks() {
        assert!(GameState::Playing.is_playing());
        assert!(!GameState::MainMenu.is_playing());
        assert!(!GameState::Paused.is_playing());
        assert!(!GameState::GameOver.is_playing());

        assert!(GameState::Playing.should_render_game());
        assert!(GameState::Paused.should_render_game());
        assert!(!GameState::MainMenu.should_render_game());
        assert!(!GameState::GameOver.should_render_game());
    }
}
