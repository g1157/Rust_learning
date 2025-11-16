/// 渲染系统
/// 负责绘制所有游戏元素
use crate::game_state::{Game, GameState};
use crate::resources::Resources;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

pub struct RenderingSystem;

impl RenderingSystem {
    /// 渲染游戏画面
    pub fn render(game: &mut Game, material: &Material, render_target: &RenderTarget) {
        // 绘制星空背景
        Self::render_background(game, material, render_target);

        // 根据游戏状态绘制不同内容
        match game.state {
            GameState::MainMenu => Self::render_main_menu(game),
            GameState::Playing => Self::render_game(game),
            GameState::Paused => Self::render_paused(game),
            GameState::GameOver => Self::render_game_over(game),
        }
    }

    /// 渲染星空背景
    fn render_background(game: &Game, material: &Material, render_target: &RenderTarget) {
        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", game.direction_modifier);
        gl_use_material(material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// 渲染主菜单
    fn render_main_menu(game: &mut Game) {
        use crate::config::{WINDOW_SIZE_X, WINDOW_SIZE_Y};

        let window_size = vec2(WINDOW_SIZE_X, WINDOW_SIZE_Y);

        root_ui().window(
            hash!(),
            vec2(
                screen_width() / 2.0 - window_size.x / 2.0,
                screen_height() / 2.0 - window_size.y / 2.0,
            ),
            window_size,
            |ui| {
                ui.label(vec2(80.0, -34.0), "Main Menu");
                if ui.button(vec2(65.0, 25.0), "Play") {
                    game.reset();
                }
                if ui.button(vec2(65.0, 125.0), "Quit") {
                    std::process::exit(0);
                }
            },
        );
    }

    /// 渲染游戏场景
    fn render_game(game: &mut Game) {
        let resources = storage::get::<Resources>();

        // 更新动画
        game.ship_sprite.update();
        game.bullet_sprite.update();
        game.enemy_small_sprite.update();
        game.enemy_medium_sprite.update();
        game.enemy_big_sprite.update();

        // 绘制子弹
        Self::render_bullets(game, &resources);

        // 绘制玩家飞船
        Self::render_player(game, &resources);

        // 绘制敌人
        Self::render_enemies(game, &resources);

        // 绘制道具
        Self::render_powerups(game);

        // 绘制爆炸效果
        Self::render_explosions(game);

        // 绘制 UI（分数、高分）
        Self::render_ui(game);
    }

    /// 渲染子弹
    fn render_bullets(game: &Game, resources: &Resources) {
        let bullet_frame = game.bullet_sprite.frame();
        for bullet in &game.bullets {
            draw_texture_ex(
                &resources.bullet_texture,
                bullet.x - bullet.size / 2.0,
                bullet.y - bullet.size / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(bullet.size, bullet.size)),
                    source: Some(bullet_frame.source_rect),
                    ..Default::default()
                },
            );
        }
    }

    /// 渲染玩家飞船
    fn render_player(game: &Game, resources: &Resources) {
        // 护盾效果（蓝色光晕）
        if game.has_shield() {
            draw_circle(
                game.player.x,
                game.player.y,
                game.player.size / 1.5,
                Color::new(0.0, 0.5, 1.0, 0.3),
            );
        }

        // 受伤无敌时闪烁效果
        if !game.has_shield() && game.invincible_timer > 0.0 && (get_time() * 10.0) as i32 % 2 == 0
        {
            return; // 跳过渲染，产生闪烁效果
        }

        let ship_frame = game.ship_sprite.frame();
        draw_texture_ex(
            &resources.ship_texture,
            game.player.x - ship_frame.dest_size.x,
            game.player.y - ship_frame.dest_size.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(ship_frame.dest_size * 2.0),
                source: Some(ship_frame.source_rect),
                ..Default::default()
            },
        );
    }

    /// 渲染敌人
    fn render_enemies(game: &Game, resources: &Resources) {
        use crate::entity::EnemyType;

        for enemy in &game.enemies {
            // 根据敌人类型选择纹理和精灵
            let (texture, sprite_frame) = match enemy.enemy_type {
                Some(EnemyType::Small) => (
                    &resources.enemy_small_texture,
                    game.enemy_small_sprite.frame(),
                ),
                Some(EnemyType::Medium) => (
                    &resources.enemy_medium_texture,
                    game.enemy_medium_sprite.frame(),
                ),
                Some(EnemyType::Big) => {
                    (&resources.enemy_big_texture, game.enemy_big_sprite.frame())
                }
                None => (
                    &resources.enemy_small_texture,
                    game.enemy_small_sprite.frame(),
                ),
            };

            draw_texture_ex(
                texture,
                enemy.x - enemy.size / 2.0,
                enemy.y - enemy.size / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(enemy.size, enemy.size)),
                    source: Some(sprite_frame.source_rect),
                    ..Default::default()
                },
            );
        }
    }

    /// 渲染道具
    fn render_powerups(game: &Game) {
        for powerup in &game.powerups {
            let size = powerup.powerup_type.size();
            let color = powerup.powerup_type.color();

            // 绘制道具（使用星形或圆形）
            draw_circle(powerup.x, powerup.y, size / 2.0, color);

            // 添加脉动效果
            let pulse = ((get_time() * 5.0).sin() * 0.2 + 0.8) as f32;
            draw_circle_lines(
                powerup.x,
                powerup.y,
                size / 2.0 * pulse,
                2.0,
                WHITE,
            );

            // 绘制道具名称（小字）
            let name = powerup.powerup_type.name();
            let text_size = 12.0;
            let text_dimensions = measure_text(name, None, text_size as u16, 1.0);
            draw_text(
                name,
                powerup.x - text_dimensions.width / 2.0,
                powerup.y + size / 2.0 + 15.0,
                text_size,
                WHITE,
            );
        }
    }

    /// 渲染爆炸效果
    fn render_explosions(game: &mut Game) {
        for (explosion, coords) in game.explosions.iter_mut() {
            explosion.draw(*coords);
        }
    }

    /// 渲染 UI（分数显示）
    fn render_ui(game: &Game) {
        // 当前分数（左上角）
        draw_text(
            format!("Score: {}", game.score).as_str(),
            10.0,
            35.0,
            25.0,
            WHITE,
        );

        // 最高分（右上角）
        let highscore_text = format!("High score: {}", game.high_score);
        let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
        draw_text(
            highscore_text.as_str(),
            screen_width() - text_dimensions.width - 10.0,
            35.0,
            25.0,
            WHITE,
        );

        // 生命值显示（左上角第二行，使用爱心图标）
        let lives_color = if game.has_shield() {
            SKYBLUE
        } else if game.invincible_timer > 0.0 {
            YELLOW
        } else {
            WHITE
        };
        
        let lives_text = format!("Lives: {}", game.lives);
        draw_text(lives_text.as_str(), 10.0, 60.0, 25.0, lives_color);

        // 难度等级显示（左上角第三行）
        draw_text(
            format!("Level: {}", game.difficulty_level).as_str(),
            10.0,
            85.0,
            20.0,
            LIGHTGRAY,
        );

        // 武器状态显示（左上角第四行）
        if game.weapon_type > 0 {
            let weapon_name = match game.weapon_type {
                1 => "双倍火力",
                2 => "三重火力",
                _ => "",
            };
            draw_text(
                format!("{} [{:.1}s]", weapon_name, game.weapon_timer).as_str(),
                10.0,
                110.0,
                18.0,
                ORANGE,
            );
        }

        // 快速射击状态显示
        if game.rapid_fire {
            draw_text(
                format!("快速射击 [{:.1}s]", game.rapid_fire_timer).as_str(),
                10.0,
                if game.weapon_type > 0 { 130.0 } else { 110.0 },
                18.0,
                YELLOW,
            );
        }

        // 护盾状态显示
        if game.has_shield() {
            draw_text(
                format!("护盾 [{:.1}s]", game.shield_timer).as_str(),
                10.0,
                screen_height() - 20.0,
                20.0,
                SKYBLUE,
            );
        }
    }

    /// 渲染暂停画面
    fn render_paused(game: &mut Game) {
        // 先绘制游戏场景（静止）
        Self::render_game(game);

        // 绘制暂停文字
        let text = "Paused";
        let text_dimensions = measure_text(text, None, 50, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - text_dimensions.width / 2.0,
            screen_height() / 2.0,
            50.0,
            WHITE,
        );
    }

    /// 渲染游戏结束画面
    fn render_game_over(game: &mut Game) {
        // 先绘制游戏场景
        Self::render_game(game);

        // 绘制 Game Over 文字
        let text = "GAME OVER!";
        let text_dimensions = measure_text(text, None, 50, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - text_dimensions.width / 2.0,
            screen_height() / 2.0,
            50.0,
            RED,
        );
    }
}
