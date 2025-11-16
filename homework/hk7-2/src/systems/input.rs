/// 输入处理系统
/// 负责处理键盘输入和玩家控制
use crate::config::*;
use crate::entity::Shape;
use crate::game_state::{Game, GameState};
use macroquad::prelude::*;

pub struct InputSystem;

impl InputSystem {
    /// 更新输入处理
    /// 处理玩家移动、射击、暂停等操作
    pub fn update(game: &mut Game, delta_time: f32) {
        match game.state {
            GameState::Playing => Self::handle_playing_input(game, delta_time),
            GameState::Paused => Self::handle_paused_input(game),
            GameState::GameOver => Self::handle_game_over_input(game),
            GameState::MainMenu => {
                // 主菜单输入在 UI 系统中处理
            }
        }
    }

    /// 处理游戏进行中的输入
    fn handle_playing_input(game: &mut Game, delta_time: f32) {
        // 重置动画为默认状态
        game.ship_sprite.set_animation(0);

        // 左右移动
        if is_key_down(KeyCode::Right) {
            game.player.x += PLAYER_MOVEMENT_SPEED * delta_time;
            game.direction_modifier += DIRECTION_MODIFIER_STEP * delta_time;
            game.ship_sprite.set_animation(2);
        }
        if is_key_down(KeyCode::Left) {
            game.player.x -= PLAYER_MOVEMENT_SPEED * delta_time;
            game.direction_modifier -= DIRECTION_MODIFIER_STEP * delta_time;
            game.ship_sprite.set_animation(1);
        }

        // 上下移动
        if is_key_down(KeyCode::Down) {
            game.player.y += PLAYER_MOVEMENT_SPEED * delta_time;
        }
        if is_key_down(KeyCode::Up) {
            game.player.y -= PLAYER_MOVEMENT_SPEED * delta_time;
        }

        // 限制玩家在屏幕内
        game.player.x = game.player.x.clamp(0.0, screen_width());
        game.player.y = game.player.y.clamp(0.0, screen_height());

        // 射击
        if is_key_pressed(KeyCode::Space) && game.can_fire() {
            Self::shoot_bullets(game);
            game.reset_fire_cooldown();
        }

        // 暂停
        if is_key_pressed(KeyCode::Escape) {
            game.state = GameState::Paused;
        }
    }

    /// 处理暂停状态的输入
    fn handle_paused_input(game: &mut Game) {
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
            game.state = GameState::Playing;
        }
    }

    /// 处理游戏结束状态的输入
    fn handle_game_over_input(game: &mut Game) {
        if is_key_pressed(KeyCode::Space) {
            game.state = GameState::MainMenu;
        }
    }

    /// 发射子弹（根据武器类型）
    fn shoot_bullets(game: &mut Game) {
        use crate::resources::Resources;
        use macroquad::audio::play_sound_once;
        use macroquad::experimental::collections::storage;

        let bullet_speed = PLAYER_MOVEMENT_SPEED * BULLET_SPEED_MULTIPLIER;

        match game.weapon_type {
            0 => {
                // 普通单发
                game.bullets.push(Shape {
                    x: game.player.x,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
            }
            1 => {
                // 双倍火力
                let offset = 12.0;
                game.bullets.push(Shape {
                    x: game.player.x - offset,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
                game.bullets.push(Shape {
                    x: game.player.x + offset,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
            }
            2 => {
                // 三重火力（散射）
                let offset = 16.0;
                // 中间
                game.bullets.push(Shape {
                    x: game.player.x,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
                // 左边
                game.bullets.push(Shape {
                    x: game.player.x - offset,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
                // 右边
                game.bullets.push(Shape {
                    x: game.player.x + offset,
                    y: game.player.y - BULLET_Y_OFFSET,
                    speed: bullet_speed,
                    size: BULLET_SIZE,
                    collided: false,
                    enemy_type: None,
                });
            }
            _ => {}
        }

        // 播放射击音效
        let resources = storage::get::<Resources>();
        play_sound_once(&resources.sound_laser);
    }
}
