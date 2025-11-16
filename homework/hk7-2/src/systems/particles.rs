/// 粒子系统配置模块
/// 定义各种粒子效果的配置参数
use macroquad_particles::{AtlasConfig, EmitterConfig};

/// 创建爆炸粒子效果配置
/// 用于敌人被击毁时的视觉反馈
pub fn particle_explosion() -> EmitterConfig {
    EmitterConfig {
        // 粒子在世界坐标系中发射（而非局部坐标）
        local_coords: false,

        // 一次性发射（不持续生成）
        one_shot: true,

        // 开始时就发射
        emitting: true,

        // 粒子生命周期（秒）
        lifetime: 0.6,

        // 生命周期随机性（0-1）
        lifetime_randomness: 0.3,

        // 爆炸性（粒子同时发射的程度，0-1）
        explosiveness: 0.65,

        // 初始方向扩散角度（弧度）
        // 2π = 360度，表示向所有方向发射
        initial_direction_spread: 2.0 * std::f32::consts::PI,

        // 初始速度（像素/秒）
        initial_velocity: 400.0,

        // 速度随机性（0-1）
        initial_velocity_randomness: 0.8,

        // 粒子大小（像素）
        size: 16.0,

        // 大小随机性（0-1）
        size_randomness: 0.3,

        // 使用纹理图集（5列1行，从第0帧开始）
        atlas: Some(AtlasConfig::new(5, 1, 0..)),

        // 其他参数使用默认值
        ..Default::default()
    }
}
