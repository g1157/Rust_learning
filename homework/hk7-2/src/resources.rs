/// 资源管理模块
/// 负责加载和管理游戏资源（纹理、音频、UI皮肤）
use macroquad::audio::{load_sound, Sound};
use macroquad::experimental::collections::storage;
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};

use crate::config;

/// 游戏资源集合
/// 包含所有预加载的游戏资源
pub struct Resources {
    /// 玩家飞船纹理
    pub ship_texture: Texture2D,
    /// 子弹纹理（精灵表）
    pub bullet_texture: Texture2D,
    /// 爆炸效果纹理（精灵表）
    pub explosion_texture: Texture2D,
    /// 小型敌人纹理
    pub enemy_small_texture: Texture2D,
    /// 中型敌人纹理
    pub enemy_medium_texture: Texture2D,
    /// 大型敌人纹理
    pub enemy_big_texture: Texture2D,
    /// 背景音乐
    pub theme_music: Sound,
    /// 爆炸音效
    pub sound_explosion: Sound,
    /// 激光射击音效
    pub sound_laser: Sound,
    /// UI 皮肤（按钮、窗口样式）
    pub ui_skin: Skin,
}

impl Resources {
    /// 异步加载所有游戏资源
    /// 返回 Result，包含加载的资源或错误
    async fn new() -> Result<Resources, macroquad::Error> {
        // 加载纹理
        let ship_texture: Texture2D = load_texture("ship.png").await?;
        ship_texture.set_filter(FilterMode::Nearest);

        let bullet_texture: Texture2D = load_texture("laser-bolts.png").await?;
        bullet_texture.set_filter(FilterMode::Nearest);

        let explosion_texture: Texture2D = load_texture("explosion.png").await?;
        explosion_texture.set_filter(FilterMode::Nearest);

        let enemy_small_texture: Texture2D = load_texture("enemy-small.png").await?;
        enemy_small_texture.set_filter(FilterMode::Nearest);

        let enemy_medium_texture: Texture2D = load_texture("enemy-medium.png").await?;
        enemy_medium_texture.set_filter(FilterMode::Nearest);

        let enemy_big_texture: Texture2D = load_texture("enemy-big.png").await?;
        enemy_big_texture.set_filter(FilterMode::Nearest);

        // 构建纹理图集（用于粒子系统）
        build_textures_atlas();

        // 加载音频
        let theme_music = load_sound("8bit-spaceshooter.ogg").await?;
        let sound_explosion = load_sound("explosion.wav").await?;
        let sound_laser = load_sound("laser.wav").await?;

        // 加载 UI 资源
        let window_background = load_image("window_background.png").await?;
        let button_background = load_image("button_background.png").await?;
        let button_clicked_background = load_image("button_clicked_background.png").await?;
        let font = load_file("atari_games.ttf").await?;

        // 构建 UI 皮肤
        let window_style = root_ui()
            .style_builder()
            .background(window_background)
            .background_margin(RectOffset::new(32.0, 76.0, 44.0, 20.0))
            .margin(RectOffset::new(0.0, -40.0, 0.0, 0.0))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background(button_background)
            .background_clicked(button_clicked_background)
            .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
            .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
            .font(&font)?
            .text_color(WHITE)
            .font_size(64)
            .build();

        let label_style = root_ui()
            .style_builder()
            .font(&font)?
            .text_color(WHITE)
            .font_size(28)
            .build();

        let ui_skin = Skin {
            window_style,
            button_style,
            label_style,
            ..root_ui().default_skin()
        };

        Ok(Resources {
            ship_texture,
            bullet_texture,
            explosion_texture,
            enemy_small_texture,
            enemy_medium_texture,
            enemy_big_texture,
            theme_music,
            sound_explosion,
            sound_laser,
            ui_skin,
        })
    }

    /// 异步加载资源并显示加载画面
    /// 资源加载完成后会存储到全局 storage 中
    pub async fn load() -> Result<(), macroquad::Error> {
        let resources_loading = start_coroutine(async move {
            let resources = Resources::new().await.unwrap();
            storage::store(resources);
        });

        // 显示加载动画
        while !resources_loading.is_done() {
            clear_background(BLACK);

            // 动态点点点效果
            let dots = ".".repeat(((get_time() * 2.) as usize) % 4);
            let text = format!("Loading resources{}", dots);

            draw_text(
                &text,
                screen_width() / 2.0 - config::LOADING_TEXT_OFFSET_X,
                screen_height() / 2.0,
                40.0,
                WHITE,
            );

            next_frame().await;
        }

        Ok(())
    }
}
