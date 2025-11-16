use macroquad::audio::{play_sound, PlaySoundParams};
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use macroquad::ui::root_ui;

mod config;
mod entity;
mod game_state;
mod resources;
mod systems;

use game_state::{Game, GameState};
use resources::Resources;
use systems::{CollisionSystem, InputSystem, MovementSystem, RenderingSystem, SpawnerSystem};

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

#[macroquad::main("Space Shooter")]
async fn main() -> Result<(), macroquad::Error> {
    // 初始化随机数生成器
    rand::srand(miniquad::date::now() as u64);

    // 创建渲染目标和材质（星空背景）
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float2),
                UniformDesc::new("direction_modifier", UniformType::Float1),
            ],
            ..Default::default()
        },
    )?;

    // 加载资源
    set_pc_assets_folder("assets");
    Resources::load().await?;
    let resources = storage::get::<Resources>();

    // 创建动画精灵
    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "bolt".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    bullet_sprite.set_animation(1);

    let ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );

    let enemy_small_sprite = AnimatedSprite::new(
        17,
        16,
        &[Animation {
            name: "enemy_small".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let enemy_medium_sprite = AnimatedSprite::new(
        32,
        32,
        &[Animation {
            name: "enemy_medium".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let enemy_big_sprite = AnimatedSprite::new(
        32,
        32,
        &[Animation {
            name: "enemy_big".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    // 播放背景音乐
    play_sound(
        &resources.theme_music,
        PlaySoundParams {
            looped: true,
            volume: 1.,
        },
    );

    // 设置 UI 皮肤
    root_ui().push_skin(&resources.ui_skin);

    // 创建游戏实例
    let mut game = Game::new(
        ship_sprite,
        bullet_sprite,
        enemy_small_sprite,
        enemy_medium_sprite,
        enemy_big_sprite,
    );

    // 主游戏循环
    loop {
        clear_background(BLACK);

        let delta_time = get_frame_time();

        // 只在游戏进行中更新游戏逻辑
        if game.state == GameState::Playing {
            InputSystem::update(&mut game, delta_time);
            MovementSystem::update(&mut game, delta_time);
            SpawnerSystem::update(&mut game);
            CollisionSystem::update(&mut game);
        } else {
            // 其他状态也需要处理输入（菜单、暂停、游戏结束）
            InputSystem::update(&mut game, delta_time);
        }

        // 渲染
        RenderingSystem::render(&mut game, &material, &render_target);

        next_frame().await
    }
}
