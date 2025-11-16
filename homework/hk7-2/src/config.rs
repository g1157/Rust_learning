/// 游戏配置常量模块
/// 集中管理所有游戏参数，方便调整和平衡
/// 玩家移动速度 (像素/秒)
pub const PLAYER_MOVEMENT_SPEED: f32 = 200.0;

/// 子弹速度倍率（相对于玩家速度）
pub const BULLET_SPEED_MULTIPLIER: f32 = 2.0;

/// 玩家飞船大小（用于碰撞检测）
pub const PLAYER_SIZE: f32 = 32.0;

/// 子弹大小
pub const BULLET_SIZE: f32 = 32.0;

/// 子弹相对玩家的 Y 偏移
pub const BULLET_Y_OFFSET: f32 = 24.0;

/// 敌人生成概率阈值 (0-99)
/// 值越高，生成越频繁。当前设置：5% 的概率每帧生成一个敌人
pub const ENEMY_SPAWN_THRESHOLD: i32 = 95;

/// 敌人最小速度
pub const ENEMY_MIN_SPEED: f32 = 50.0;

/// 敌人最大速度
pub const ENEMY_MAX_SPEED: f32 = 150.0;

/// 方向修正因子（玩家移动时背景的倾斜效果）
pub const DIRECTION_MODIFIER_STEP: f32 = 0.05;

/// UI 窗口大小
pub const WINDOW_SIZE_X: f32 = 370.0;
pub const WINDOW_SIZE_Y: f32 = 320.0;

/// 加载画面文字位置偏移
pub const LOADING_TEXT_OFFSET_X: f32 = 160.0;

/// 爆炸粒子数量倍数（相对于敌人大小）
pub const EXPLOSION_PARTICLE_MULTIPLIER: u32 = 4;

/// 初始生命数
pub const INITIAL_LIVES: u32 = 3;

/// 碰撞后无敌时间（秒）
pub const INVINCIBLE_TIME: f32 = 2.0;

/// 难度提升所需分数
pub const DIFFICULTY_INCREASE_SCORE: u32 = 100;

/// 最大难度等级
pub const MAX_DIFFICULTY_LEVEL: u32 = 10;

/// 道具掉落概率阈值（0-10000）
/// 当前设置：每帧 0.3% 的概率掉落道具
pub const POWERUP_DROP_THRESHOLD: i32 = 9970;

/// 道具掉落速度
pub const POWERUP_FALL_SPEED: f32 = 50.0;

/// 护盾道具持续时间（秒）
pub const SHIELD_DURATION: f32 = 10.0;

/// 武器升级持续时间（秒）
pub const WEAPON_UPGRADE_DURATION: f32 = 15.0;

/// 快速射击冷却时间（秒）
pub const RAPID_FIRE_COOLDOWN: f32 = 0.15;

/// 普通射击冷却时间（秒）
pub const NORMAL_FIRE_COOLDOWN: f32 = 0.3;
