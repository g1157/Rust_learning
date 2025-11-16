/// 实体模块
/// 包含所有游戏实体的定义（玩家、敌人、子弹等）
use macroquad::prelude::*;

/// 道具类型枚举
/// 不同道具有不同的效果
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerUpType {
    /// 生命恢复（+1条命）
    Health,
    /// 护盾（10秒无敌时间）
    Shield,
    /// 双倍火力（同时发射2发子弹）
    DoubleFire,
    /// 三重火力（同时发射3发散射子弹）
    TripleFire,
    /// 快速射击（降低射击冷却）
    RapidFire,
}

impl PowerUpType {
    /// 随机生成一个道具类型（按概率分布）
    pub fn random() -> Self {
        match rand::gen_range(0, 100) {
            0..=30 => PowerUpType::Health,      // 30% 生命
            31..=50 => PowerUpType::Shield,     // 20% 护盾
            51..=70 => PowerUpType::DoubleFire, // 20% 双倍火力
            71..=85 => PowerUpType::TripleFire, // 15% 三重火力
            _ => PowerUpType::RapidFire,        // 15% 快速射击
        }
    }

    /// 获取道具的颜色（用于渲染）
    pub fn color(&self) -> Color {
        match self {
            PowerUpType::Health => GREEN,
            PowerUpType::Shield => SKYBLUE,
            PowerUpType::DoubleFire => ORANGE,
            PowerUpType::TripleFire => RED,
            PowerUpType::RapidFire => YELLOW,
        }
    }

    /// 获取道具名称
    pub fn name(&self) -> &'static str {
        match self {
            PowerUpType::Health => "生命+1",
            PowerUpType::Shield => "护盾",
            PowerUpType::DoubleFire => "双倍火力",
            PowerUpType::TripleFire => "三重火力",
            PowerUpType::RapidFire => "快速射击",
        }
    }

    /// 获取道具大小
    pub fn size(&self) -> f32 {
        24.0
    }
}

/// 道具实体
#[derive(Clone)]
pub struct PowerUp {
    /// 道具类型
    pub powerup_type: PowerUpType,
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// 移动速度
    pub speed: f32,
    /// 是否已被收集
    pub collected: bool,
}

impl PowerUp {
    /// 创建新的道具实例
    pub fn new(powerup_type: PowerUpType, x: f32, y: f32) -> Self {
        Self {
            powerup_type,
            x,
            y,
            speed: 50.0, // 道具下落速度较慢
            collected: false,
        }
    }

    /// 获取道具的边界矩形
    pub fn rect(&self) -> Rect {
        let size = self.powerup_type.size();
        Rect {
            x: self.x - size / 2.0,
            y: self.y - size / 2.0,
            w: size,
            h: size,
        }
    }

    /// 检查是否与玩家碰撞
    pub fn collides_with(&self, other: &Shape) -> bool {
        self.rect().overlaps(&other.rect())
    }
}

/// 敌人类型枚举
/// 不同类型的敌人有不同的大小、速度和分数
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    /// 小型敌人（快速、低分）
    Small,
    /// 中型敌人（中等速度、中等分数）
    Medium,
    /// 大型敌人（慢速、高分）
    Big,
}

impl EnemyType {
    /// 获取敌人类型的默认大小
    pub fn size(&self) -> f32 {
        match self {
            EnemyType::Small => 32.0,
            EnemyType::Medium => 48.0,
            EnemyType::Big => 64.0,
        }
    }

    /// 获取击败该类型敌人的分数
    pub fn score_value(&self) -> u32 {
        match self {
            EnemyType::Small => 10,
            EnemyType::Medium => 25,
            EnemyType::Big => 50,
        }
    }

    /// 获取敌人类型的速度倍率
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            EnemyType::Small => 1.5,  // 快速
            EnemyType::Medium => 1.0, // 正常
            EnemyType::Big => 0.7,    // 慢速
        }
    }

    /// 随机生成一个敌人类型（按概率分布）
    pub fn random() -> Self {
        match rand::gen_range(0, 100) {
            0..=60 => EnemyType::Small,   // 60% 小型
            61..=85 => EnemyType::Medium, // 25% 中型
            _ => EnemyType::Big,          // 15% 大型
        }
    }
}

/// 基础形状实体
/// 用于表示游戏中的可移动对象（玩家、敌人、子弹）
#[derive(Clone)]
pub struct Shape {
    /// 实体大小（宽高相同的正方形）
    pub size: f32,
    /// 移动速度（像素/秒）
    pub speed: f32,
    /// X 坐标（屏幕中心点）
    pub x: f32,
    /// Y 坐标（屏幕中心点）
    pub y: f32,
    /// 是否已碰撞（用于标记删除）
    pub collided: bool,
    /// 敌人类型（仅用于敌人实体）
    pub enemy_type: Option<EnemyType>,
}

impl Shape {
    /// 创建新的 Shape 实例（用于玩家和子弹）
    #[allow(dead_code)]
    pub fn new(size: f32, speed: f32, x: f32, y: f32) -> Self {
        Self {
            size,
            speed,
            x,
            y,
            collided: false,
            enemy_type: None,
        }
    }

    /// 创建新的敌人实例
    pub fn new_enemy(enemy_type: EnemyType, speed: f32, x: f32, y: f32) -> Self {
        Self {
            size: enemy_type.size(),
            speed: speed * enemy_type.speed_multiplier(),
            x,
            y,
            collided: false,
            enemy_type: Some(enemy_type),
        }
    }

    /// 检查是否与另一个 Shape 碰撞
    /// 使用矩形重叠检测算法
    pub fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    /// 获取此 Shape 的边界矩形
    /// 用于碰撞检测和渲染
    pub fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }

    /// 移动实体
    /// delta_time: 帧时间间隔（秒）
    /// dx, dy: 归一化的移动方向 (-1.0 到 1.0)
    #[allow(dead_code)]
    pub fn move_by(&mut self, delta_time: f32, dx: f32, dy: f32) {
        self.x += dx * self.speed * delta_time;
        self.y += dy * self.speed * delta_time;
    }

    /// 检查是否在屏幕外（用于清理）
    #[allow(dead_code)]
    pub fn is_offscreen(&self, screen_width: f32, screen_height: f32) -> bool {
        self.x < -self.size
            || self.x > screen_width + self.size
            || self.y < -self.size
            || self.y > screen_height + self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        let shape1 = Shape::new(32.0, 100.0, 100.0, 100.0);
        let shape2 = Shape::new(32.0, 100.0, 110.0, 110.0);
        let shape3 = Shape::new(32.0, 100.0, 200.0, 200.0);

        // 重叠的矩形应该碰撞
        assert!(shape1.collides_with(&shape2));

        // 不重叠的矩形不应该碰撞
        assert!(!shape1.collides_with(&shape3));
    }

    #[test]
    fn test_movement() {
        let mut shape = Shape::new(32.0, 100.0, 100.0, 100.0);

        // 向右移动 1 秒
        shape.move_by(1.0, 1.0, 0.0);
        assert_eq!(shape.x, 200.0);
        assert_eq!(shape.y, 100.0);
    }

    #[test]
    fn test_offscreen_detection() {
        let shape = Shape::new(32.0, 100.0, -100.0, 100.0);
        assert!(shape.is_offscreen(800.0, 600.0));

        let shape2 = Shape::new(32.0, 100.0, 400.0, 300.0);
        assert!(!shape2.is_offscreen(800.0, 600.0));
    }
}
