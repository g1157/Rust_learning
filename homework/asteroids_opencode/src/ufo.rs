//! UFO 敌人模块
//!
//! 管理 UFO 敌人的生成、移动、射击和碰撞检测。
//!
//! ## 功能
//! - 从屏幕边缘进入，缓慢蛇形巡航
//! - 周期性对最近玩家做轻微追踪（小角度校正）
//! - 预判射击并带少量随机散布
//! - 基于波次的难度配置（Easy/Normal/Hard/Insane）
//! - HP 3-6，击毁得分 200-420，40-50% 掉落道具

use macroquad::prelude::*;

use crate::bullet::BULLET_SPEED;

// ============================================================================
// UFO 常量与难度配置
// ============================================================================

/// UFO 巡航速度（像素/秒）- 平衡调整：降低以给玩家更多反应时间
pub const UFO_CRUISE_SPEED: f32 = 100.0;

/// UFO 蛇形运动振幅
pub const UFO_WOBBLE_AMPLITUDE: f32 = 80.0;

/// UFO 蛇形运动频率
pub const UFO_WOBBLE_FREQUENCY: f32 = 1.0;

/// UFO 追踪最大偏转角（度）
pub const UFO_TRACK_MAX_DEGREES: f32 = 25.0;

/// UFO 追踪插值系数（0-1，越大越快对准玩家）
pub const UFO_TRACK_LERP: f32 = 0.15;

/// UFO 初始 HP（需要多少发子弹击毁）
pub const UFO_HP: i32 = 3;

/// UFO 射击间隔（秒）- 平衡调整：增加间隔给玩家更多躲避时间
pub const UFO_FIRE_INTERVAL: f64 = 2.5;

/// UFO 子弹散布角度（度）- 平衡调整：增大散布使子弹更易躲避
pub const UFO_FIRE_SPREAD_DEGREES: f32 = 15.0;

/// UFO 预测射击系数
pub const UFO_LEAD_FACTOR: f32 = 0.5;

/// UFO 碰撞半径
pub const UFO_RADIUS: f32 = 28.0;

/// UFO 击毁最小得分
pub const UFO_SCORE_MIN: u32 = 200;

/// UFO 击毁最大得分
pub const UFO_SCORE_MAX: u32 = 300;

/// UFO 道具掉落几率
pub const UFO_DROP_CHANCE: f32 = 0.45;

/// UFO 子弹速度（稍慢于玩家子弹）- 平衡调整：降低速度提升可躲避性
pub const UFO_BULLET_SPEED: f32 = BULLET_SPEED * 0.5;

/// UFO 出界清理边距
pub const UFO_DESPAWN_MARGIN: f32 = 100.0;

// ============================================================================
// UFO 难度配置系统
// ============================================================================

/// UFO 难度等级
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifficultyLevel {
    Easy,
    Normal,
    Hard,
    Insane,
}

/// UFO 可调参数配置
#[derive(Clone, Copy, Debug)]
pub struct UfoConfig {
    pub cruise_speed: f32,
    pub hp: i32,
    pub fire_interval: f64,
    pub fire_spread_degrees: f32,
    pub score_min: u32,
    pub score_max: u32,
}

impl UfoConfig {
    /// 创建新的 UFO 配置
    pub const fn new(
        cruise_speed: f32,
        hp: i32,
        fire_interval: f64,
        fire_spread_degrees: f32,
        score_min: u32,
        score_max: u32,
    ) -> Self {
        Self {
            cruise_speed,
            hp,
            fire_interval,
            fire_spread_degrees,
            score_min,
            score_max,
        }
    }

    /// 获取基础/默认配置
    pub const fn default_config() -> Self {
        Self::new(
            UFO_CRUISE_SPEED,
            UFO_HP,
            UFO_FIRE_INTERVAL,
            UFO_FIRE_SPREAD_DEGREES,
            UFO_SCORE_MIN,
            UFO_SCORE_MAX,
        )
    }
}

/// 根据波次返回对应的 UFO 难度等级
pub fn difficulty_for_wave(wave: u32) -> DifficultyLevel {
    match wave {
        0..=3 => DifficultyLevel::Easy,
        4..=6 => DifficultyLevel::Normal,
        7..=9 => DifficultyLevel::Hard,
        _ => DifficultyLevel::Insane,
    }
}

/// 根据波次返回对应的 UFO 配置
///
/// - Wave 1-3: Easy（基础值）
/// - Wave 4-6: Normal（+10% 速度，+1 HP，+10% 射速）
/// - Wave 7-9: Hard（+20% 速度，+2 HP，+20% 射速，更高精度）
/// - Wave 10+: Insane（+30% 速度，+3 HP，+30% 射速，最高精度）
pub fn ufo_config_for_wave(wave: u32) -> UfoConfig {
    match difficulty_for_wave(wave) {
        DifficultyLevel::Easy => UfoConfig::default_config(),
        DifficultyLevel::Normal => UfoConfig::new(
            UFO_CRUISE_SPEED * 1.1,
            UFO_HP + 1,
            UFO_FIRE_INTERVAL * 0.9,
            UFO_FIRE_SPREAD_DEGREES,
            (UFO_SCORE_MIN as f32 * 1.1) as u32,
            (UFO_SCORE_MAX as f32 * 1.1) as u32,
        ),
        DifficultyLevel::Hard => UfoConfig::new(
            UFO_CRUISE_SPEED * 1.2,
            UFO_HP + 2,
            UFO_FIRE_INTERVAL * 0.8,
            UFO_FIRE_SPREAD_DEGREES * 0.7, // 更准
            (UFO_SCORE_MIN as f32 * 1.25) as u32,
            (UFO_SCORE_MAX as f32 * 1.25) as u32,
        ),
        DifficultyLevel::Insane => UfoConfig::new(
            UFO_CRUISE_SPEED * 1.3,
            UFO_HP + 3,
            UFO_FIRE_INTERVAL * 0.7,
            UFO_FIRE_SPREAD_DEGREES * 0.5, // 最高精度
            (UFO_SCORE_MIN as f32 * 1.4) as u32,
            (UFO_SCORE_MAX as f32 * 1.4) as u32,
        ),
    }
}

// ============================================================================
// UFO 生成边缘枚举
// ============================================================================

/// UFO 生成的屏幕边缘
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpawnSide {
    Left,
    Right,
    Top,
    Bottom,
}

// ============================================================================
// 敌人子弹数据
// ============================================================================

/// 敌人射击数据（供调用方转换为 Bullet）
#[derive(Clone, Copy, Debug)]
pub struct EnemyShot {
    pub position: Vec2,
    pub velocity: Vec2,
}

/// 敌人子弹实体
#[derive(Clone, Debug)]
pub struct EnemyBullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shot_at: f64,
    pub collided: bool,
}

/// 敌人子弹半径
pub const ENEMY_BULLET_RADIUS: f32 = 4.0;

/// 敌人子弹生命周期
pub const ENEMY_BULLET_LIFETIME: f64 = 3.0;

impl EnemyBullet {
    /// 从 EnemyShot 数据创建敌人子弹
    pub fn from_shot(shot: EnemyShot, now: f64) -> Self {
        Self {
            pos: shot.position,
            vel: shot.velocity,
            shot_at: now,
            collided: false,
        }
    }

    /// 更新敌人子弹位置
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }

    /// 检查子弹是否过期
    pub fn is_expired(&self, now: f64) -> bool {
        now - self.shot_at > ENEMY_BULLET_LIFETIME
    }

    /// 检查是否应该移除（碰撞或过期）
    pub fn should_remove(&self, now: f64) -> bool {
        self.collided || self.is_expired(now)
    }
}

/// 绘制敌人子弹（带尾迹效果）
pub fn draw_enemy_bullet(bullet: &EnemyBullet) {
    if bullet.collided {
        return;
    }

    // 子弹核心（红色，危险感）
    let core_color = Color::new(1.0, 0.3, 0.2, 1.0);
    // 外圈光晕
    let glow_color = Color::new(1.0, 0.5, 0.3, 0.4);

    // 尾迹效果：沿速度反方向绘制 4 个渐隐圆点，突出动感
    let dir = bullet.vel.normalize_or_zero();
    if dir.length_squared() > 0.0 {
        for i in 1..=4 {
            let t = i as f32 / 4.0;
            // 尾迹沿运动反方向延伸
            let offset = -dir * ENEMY_BULLET_RADIUS * 3.0 * t;
            // 透明度随距离递减
            let alpha = 0.4 * (1.0 - t * 0.5);
            // 尾迹圆点逐渐变小
            let radius = ENEMY_BULLET_RADIUS * (1.0 - 0.12 * i as f32);
            draw_circle(
                bullet.pos.x + offset.x,
                bullet.pos.y + offset.y,
                radius,
                Color::new(1.0, 0.45, 0.3, alpha),
            );
        }
    }

    // 绘制光晕
    draw_circle(
        bullet.pos.x,
        bullet.pos.y,
        ENEMY_BULLET_RADIUS * 2.0,
        glow_color,
    );
    // 绘制核心
    draw_circle(bullet.pos.x, bullet.pos.y, ENEMY_BULLET_RADIUS, core_color);
    // 绘制高光
    draw_circle(
        bullet.pos.x - 1.0,
        bullet.pos.y - 1.0,
        ENEMY_BULLET_RADIUS * 0.4,
        Color::new(1.0, 0.8, 0.6, 0.8),
    );
}

// ============================================================================
// UFO 结构体
// ============================================================================

/// UFO 敌人实体
pub struct Ufo {
    /// 位置
    pub pos: Vec2,
    /// 速度
    pub vel: Vec2,
    /// 朝向角度（弧度）
    pub angle: f32,
    /// 当前 HP
    pub hp: i32,
    /// 下次射击时间
    pub next_fire_at: f64,
    /// 蛇形运动相位
    pub wobble_phase: f32,
    /// 生成边缘（预留：后续可用于特殊事件触发）
    #[allow(dead_code)]
    pub spawn_side: SpawnSide,
    /// 击毁得分
    pub score_value: u32,
    /// 道具掉落几率
    pub drop_chance: f32,
    /// 是否为本局第一架 UFO（用于保底掉落）
    #[allow(dead_code)]
    pub is_first_ufo: bool,
    /// 是否已被标记为销毁
    pub destroyed: bool,
    /// 受击闪烁计时器
    pub hit_flash_until: f64,
    // 实例难度配置（从 UfoConfig 复制）
    /// 当前巡航速度
    pub cruise_speed: f32,
    /// 射击间隔
    pub fire_interval: f64,
    /// 子弹散布角度（度）
    pub fire_spread_degrees: f32,
}

impl Ufo {
    /// 从屏幕边缘生成 UFO
    ///
    /// # 参数
    /// - `now`: 当前游戏时间
    /// - `is_first`: 是否为本局首架 UFO（首架保证掉落道具）
    /// - `config`: UFO 难度配置（基于当前波次）
    pub fn spawn_from_edge(now: f64, is_first: bool, config: UfoConfig) -> Self {
        let w = screen_width();
        let h = screen_height();

        // 随机选择生成边缘
        let side = match rand::gen_range(0, 4) {
            0 => SpawnSide::Left,
            1 => SpawnSide::Right,
            2 => SpawnSide::Top,
            _ => SpawnSide::Bottom,
        };

        // 根据边缘确定初始位置和方向
        let (pos, dir) = match side {
            SpawnSide::Left => (
                Vec2::new(-UFO_RADIUS * 2.0, rand::gen_range(h * 0.2, h * 0.8)),
                Vec2::new(1.0, rand::gen_range(-0.3, 0.3)),
            ),
            SpawnSide::Right => (
                Vec2::new(w + UFO_RADIUS * 2.0, rand::gen_range(h * 0.2, h * 0.8)),
                Vec2::new(-1.0, rand::gen_range(-0.3, 0.3)),
            ),
            SpawnSide::Top => (
                Vec2::new(rand::gen_range(w * 0.2, w * 0.8), -UFO_RADIUS * 2.0),
                Vec2::new(rand::gen_range(-0.3, 0.3), 1.0),
            ),
            SpawnSide::Bottom => (
                Vec2::new(rand::gen_range(w * 0.2, w * 0.8), h + UFO_RADIUS * 2.0),
                Vec2::new(rand::gen_range(-0.3, 0.3), -1.0),
            ),
        };

        // 首架 UFO 保证掉落道具，后续 UFO 使用常规掉落几率
        let drop_chance = if is_first { 1.0 } else { UFO_DROP_CHANCE };

        Self {
            pos,
            vel: dir.normalize() * config.cruise_speed,
            angle: dir.y.atan2(dir.x),
            hp: config.hp,
            next_fire_at: now + config.fire_interval * 0.5, // 初次射击稍快
            wobble_phase: rand::gen_range(0.0, std::f32::consts::TAU),
            spawn_side: side,
            score_value: rand::gen_range(config.score_min, config.score_max + 1),
            drop_chance,
            is_first_ufo: is_first,
            destroyed: false,
            hit_flash_until: 0.0,
            cruise_speed: config.cruise_speed,
            fire_interval: config.fire_interval,
            fire_spread_degrees: config.fire_spread_degrees,
        }
    }

    /// 更新 UFO 运动和追踪逻辑
    pub fn update(&mut self, dt: f32, now: f64, player_positions: &[Vec2]) {
        if self.destroyed {
            return;
        }

        // 蛇形巡航：在垂直于运动方向上添加正弦波动
        let time = now as f32;
        let wobble_offset =
            (time * UFO_WOBBLE_FREQUENCY + self.wobble_phase).sin() * UFO_WOBBLE_AMPLITUDE;

        // 计算垂直方向
        let forward = self.vel.normalize_or_zero();
        let perpendicular = Vec2::new(-forward.y, forward.x);

        // 微追踪最近玩家
        if let Some(target) = find_nearest_player(self.pos, player_positions) {
            let to_target = (target - self.pos).normalize_or_zero();
            let current_dir = forward;

            // 限制追踪角度
            let blended = lerp_direction_clamped(current_dir, to_target, UFO_TRACK_LERP);
            self.vel = blended * self.cruise_speed;
            self.angle = self.vel.y.atan2(self.vel.x);
        }

        // 应用蛇形偏移
        let wobble_force = perpendicular * wobble_offset * 0.5 * dt;
        self.pos += self.vel * dt + wobble_force;

        // 屏幕边界软约束：靠近边缘时轻微转向
        self.apply_boundary_steering(dt);
    }

    /// 边界转向：防止 UFO 过快离开屏幕
    fn apply_boundary_steering(&mut self, dt: f32) {
        let margin = 100.0;
        let steer_strength = 50.0;
        let w = screen_width();
        let h = screen_height();

        let mut steer = Vec2::ZERO;

        if self.pos.x < margin {
            steer.x += steer_strength;
        } else if self.pos.x > w - margin {
            steer.x -= steer_strength;
        }

        if self.pos.y < margin {
            steer.y += steer_strength;
        } else if self.pos.y > h - margin {
            steer.y -= steer_strength;
        }

        self.vel += steer * dt;

        // 限制速度
        let speed = self.vel.length();
        if speed > self.cruise_speed * 1.5 {
            self.vel = self.vel.normalize() * self.cruise_speed * 1.5;
        }
    }

    /// 碰撞半径
    pub fn radius(&self) -> f32 {
        UFO_RADIUS
    }

    /// 是否应该被清理（出界或已销毁）
    pub fn should_despawn(&self) -> bool {
        if self.destroyed {
            return true;
        }

        let margin = UFO_DESPAWN_MARGIN;
        let w = screen_width();
        let h = screen_height();

        self.pos.x < -margin
            || self.pos.x > w + margin
            || self.pos.y < -margin
            || self.pos.y > h + margin
    }

    /// 尝试射击：返回射击数据（如果可以射击）
    pub fn try_fire(
        &mut self,
        now: f64,
        player_positions: &[Vec2],
        player_velocities: &[Vec2],
    ) -> Option<EnemyShot> {
        if self.destroyed || now < self.next_fire_at {
            return None;
        }

        // 更新下次射击时间
        self.next_fire_at = now + self.fire_interval;

        // 找到最近玩家
        let (target_pos, target_vel) =
            find_nearest_player_with_vel(self.pos, player_positions, player_velocities)?;

        // 预判射击：根据距离和玩家速度计算提前量
        let distance = self.pos.distance(target_pos);
        let travel_time = distance / UFO_BULLET_SPEED;
        let lead_target = target_pos + target_vel * UFO_LEAD_FACTOR * travel_time;

        // 计算射击方向
        let mut direction = (lead_target - self.pos).normalize_or_zero();

        // 添加随机散布（使用实例配置的散布角度）
        let spread_rad = self.fire_spread_degrees.to_radians() * rand::gen_range(-1.0f32, 1.0f32);
        let (sin_a, cos_a) = spread_rad.sin_cos();
        direction = Vec2::new(
            direction.x * cos_a - direction.y * sin_a,
            direction.x * sin_a + direction.y * cos_a,
        );

        Some(EnemyShot {
            position: self.pos,
            velocity: direction.normalize_or_zero() * UFO_BULLET_SPEED,
        })
    }

    /// 受到伤害，返回是否被击毁
    pub fn take_hit(&mut self, damage: i32, now: f64) -> bool {
        self.hp -= damage;
        self.hit_flash_until = now + 0.15; // 受击闪烁 150ms

        if self.hp <= 0 {
            self.destroyed = true;
            true
        } else {
            false
        }
    }

    /// 是否处于受击闪烁状态
    pub fn is_flashing(&self, now: f64) -> bool {
        now < self.hit_flash_until
    }
}

// ============================================================================
// 绘制函数
// ============================================================================

/// 绘制 UFO（飞碟形状）
pub fn draw_ufo(ufo: &Ufo, now: f64) {
    if ufo.destroyed {
        return;
    }

    let pos = ufo.pos;
    let r = UFO_RADIUS;

    // 受击闪烁效果
    let flash = ufo.is_flashing(now);
    let body_color = if flash {
        Color::new(1.0, 1.0, 1.0, 0.9)
    } else {
        Color::new(0.3, 0.4, 0.5, 0.85)
    };
    let outline_color = if flash {
        Color::new(1.0, 0.5, 0.5, 1.0)
    } else {
        Color::new(0.7, 0.85, 1.0, 0.95)
    };

    // 盘体（椭圆）
    draw_ellipse(pos.x, pos.y, r, r * 0.5, 0.0, body_color);
    draw_ellipse_lines(pos.x, pos.y, r, r * 0.5, 0.0, 2.5, outline_color);

    // 舱盖（圆顶）
    let dome_color = if flash {
        Color::new(1.0, 1.0, 1.0, 0.9)
    } else {
        Color::new(0.5, 0.7, 0.9, 0.8)
    };
    draw_circle(pos.x, pos.y - r * 0.2, r * 0.35, dome_color);
    draw_circle_lines(pos.x, pos.y - r * 0.2, r * 0.35, 1.5, outline_color);

    // 底部灯带
    let light_color = if flash {
        Color::new(1.0, 1.0, 0.5, 1.0)
    } else {
        Color::new(1.0, 0.8, 0.3, 0.9)
    };
    let light_positions = [-0.6, -0.3, 0.0, 0.3, 0.6];
    for &offset in &light_positions {
        let light_x = pos.x + offset * r;
        let light_y = pos.y + r * 0.15;
        draw_circle(light_x, light_y, 3.0, light_color);
    }
}

/// 绘制 UFO 入场预警（闪烁箭头）
pub fn draw_ufo_warning(ufo: &Ufo, now: f64) {
    // 只在 UFO 尚未完全进入屏幕时显示预警
    let margin = UFO_RADIUS;
    let w = screen_width();
    let h = screen_height();

    let in_screen = ufo.pos.x > margin
        && ufo.pos.x < w - margin
        && ufo.pos.y > margin
        && ufo.pos.y < h - margin;

    if in_screen {
        return;
    }

    // 闪烁效果
    let blink = ((now * 8.0) as i32 % 2) == 0;
    if !blink {
        return;
    }

    let warning_color = Color::new(1.0, 0.3, 0.3, 0.8);

    // 计算预警位置（屏幕边缘）
    let warning_pos = Vec2::new(
        ufo.pos.x.clamp(20.0, w - 20.0),
        ufo.pos.y.clamp(20.0, h - 20.0),
    );

    // 绘制预警三角形
    let size = 12.0;
    let dir = (ufo.pos - warning_pos).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);

    let p1 = warning_pos + dir * size;
    let p2 = warning_pos - dir * size * 0.5 + perp * size * 0.6;
    let p3 = warning_pos - dir * size * 0.5 - perp * size * 0.6;

    draw_triangle(p1, p2, p3, warning_color);
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 查找最近的玩家位置
fn find_nearest_player(origin: Vec2, players: &[Vec2]) -> Option<Vec2> {
    players
        .iter()
        .min_by(|a, b| {
            origin
                .distance_squared(**a)
                .partial_cmp(&origin.distance_squared(**b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// 查找最近玩家的位置和速度
fn find_nearest_player_with_vel(
    origin: Vec2,
    positions: &[Vec2],
    velocities: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    if positions.is_empty() || positions.len() != velocities.len() {
        return None;
    }

    let mut best: Option<(Vec2, Vec2, f32)> = None;

    for (pos, vel) in positions.iter().zip(velocities.iter()) {
        let dist_sq = origin.distance_squared(*pos);
        if best.map(|(_, _, d)| dist_sq < d).unwrap_or(true) {
            best = Some((*pos, *vel, dist_sq));
        }
    }

    best.map(|(p, v, _)| (p, v))
}

/// 限制角度的方向插值
fn lerp_direction_clamped(from: Vec2, to: Vec2, t: f32) -> Vec2 {
    let from_norm = from.normalize_or_zero();
    let to_norm = to.normalize_or_zero();

    // 简单线性插值
    let blended = from_norm.lerp(to_norm, t);

    // 限制最大转向角度
    let max_rad = UFO_TRACK_MAX_DEGREES.to_radians();
    let angle = angle_between_vectors(from_norm, blended).abs();

    if angle > max_rad && angle > 0.001 {
        let clamped_t = max_rad / angle * t;
        from_norm.lerp(to_norm, clamped_t).normalize_or_zero()
    } else {
        blended.normalize_or_zero()
    }
}

/// 计算两个向量之间的角度（带符号）
fn angle_between_vectors(a: Vec2, b: Vec2) -> f32 {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let cross = a.x * b.y - a.y * b.x;
    angle * cross.signum()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试专用：创建 UFO 实例而不依赖 macroquad 图形上下文
    fn create_test_ufo(is_first: bool) -> Ufo {
        let drop_chance = if is_first { 1.0 } else { UFO_DROP_CHANCE };
        let config = UfoConfig::default_config();
        Ufo {
            pos: Vec2::new(100.0, 100.0),
            vel: Vec2::new(config.cruise_speed, 0.0),
            angle: 0.0,
            hp: config.hp,
            next_fire_at: config.fire_interval * 0.5,
            wobble_phase: 0.0,
            spawn_side: SpawnSide::Left,
            score_value: (config.score_min + config.score_max) / 2,
            drop_chance,
            is_first_ufo: is_first,
            destroyed: false,
            hit_flash_until: 0.0,
            cruise_speed: config.cruise_speed,
            fire_interval: config.fire_interval,
            fire_spread_degrees: config.fire_spread_degrees,
        }
    }

    #[test]
    fn test_ufo_creation_regular() {
        let ufo = create_test_ufo(false);
        assert_eq!(ufo.hp, UFO_HP);
        assert!(!ufo.destroyed);
        assert!(!ufo.is_first_ufo);
        // 非首架 UFO 使用常规掉落几率
        assert!((ufo.drop_chance - UFO_DROP_CHANCE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_first_ufo_guaranteed_drop() {
        let ufo = create_test_ufo(true);
        assert!(ufo.is_first_ufo);
        // 首架 UFO 保证掉落（100% 掉落几率）
        assert!((ufo.drop_chance - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ufo_take_hit() {
        let mut ufo = create_test_ufo(false);
        assert!(!ufo.take_hit(1, 0.0)); // 还剩 2 HP
        assert!(!ufo.take_hit(1, 0.0)); // 还剩 1 HP
        assert!(ufo.take_hit(1, 0.0)); // 击毁
        assert!(ufo.destroyed);
    }

    #[test]
    fn test_ufo_take_hit_with_overkill() {
        let mut ufo = create_test_ufo(false);
        // 一次性击杀（伤害超过 HP）
        assert!(ufo.take_hit(10, 0.0));
        assert!(ufo.destroyed);
        assert!(ufo.hp <= 0);
    }

    #[test]
    fn test_find_nearest_player() {
        let origin = Vec2::new(100.0, 100.0);
        let players = vec![
            Vec2::new(200.0, 100.0), // 距离 100
            Vec2::new(150.0, 100.0), // 距离 50 - 最近
            Vec2::new(300.0, 300.0), // 距离较远
        ];
        let nearest = find_nearest_player(origin, &players);
        assert_eq!(nearest, Some(Vec2::new(150.0, 100.0)));
    }

    #[test]
    fn test_find_nearest_player_empty() {
        let origin = Vec2::new(100.0, 100.0);
        let nearest = find_nearest_player(origin, &[]);
        assert!(nearest.is_none());
    }

    #[test]
    fn test_find_nearest_player_single() {
        let origin = Vec2::new(0.0, 0.0);
        let players = vec![Vec2::new(50.0, 50.0)];
        let nearest = find_nearest_player(origin, &players);
        assert_eq!(nearest, Some(Vec2::new(50.0, 50.0)));
    }

    #[test]
    fn test_angle_between_vectors() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        let angle = angle_between_vectors(a, b);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    }

    #[test]
    fn test_angle_between_same_direction() {
        let a = Vec2::new(1.0, 0.0);
        let angle = angle_between_vectors(a, a);
        assert!(angle.abs() < 0.01); // Should be ~0
    }

    #[test]
    fn test_angle_between_opposite_directions() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(-1.0, 0.0);
        let angle = angle_between_vectors(a, b);
        assert!((angle - std::f32::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_ufo_constants_are_sensible() {
        // 确保常量配置合理
        assert!(UFO_HP > 0);
        assert!(UFO_CRUISE_SPEED > 0.0);
        assert!(UFO_FIRE_INTERVAL > 0.0);
        assert!(UFO_DROP_CHANCE >= 0.0 && UFO_DROP_CHANCE <= 1.0);
        assert!(UFO_SCORE_MIN <= UFO_SCORE_MAX);
        assert!(UFO_RADIUS > 0.0);
    }

    // ------------------------------------------------------------------------
    // Difficulty Scaling Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_difficulty_for_wave_easy() {
        assert_eq!(difficulty_for_wave(0), DifficultyLevel::Easy);
        assert_eq!(difficulty_for_wave(1), DifficultyLevel::Easy);
        assert_eq!(difficulty_for_wave(3), DifficultyLevel::Easy);
    }

    #[test]
    fn test_difficulty_for_wave_normal() {
        assert_eq!(difficulty_for_wave(4), DifficultyLevel::Normal);
        assert_eq!(difficulty_for_wave(5), DifficultyLevel::Normal);
        assert_eq!(difficulty_for_wave(6), DifficultyLevel::Normal);
    }

    #[test]
    fn test_difficulty_for_wave_hard() {
        assert_eq!(difficulty_for_wave(7), DifficultyLevel::Hard);
        assert_eq!(difficulty_for_wave(8), DifficultyLevel::Hard);
        assert_eq!(difficulty_for_wave(9), DifficultyLevel::Hard);
    }

    #[test]
    fn test_difficulty_for_wave_insane() {
        assert_eq!(difficulty_for_wave(10), DifficultyLevel::Insane);
        assert_eq!(difficulty_for_wave(15), DifficultyLevel::Insane);
        assert_eq!(difficulty_for_wave(100), DifficultyLevel::Insane);
    }

    #[test]
    fn test_ufo_config_easy_is_default() {
        let easy = ufo_config_for_wave(1);
        let default_cfg = UfoConfig::default_config();

        assert_eq!(easy.cruise_speed, default_cfg.cruise_speed);
        assert_eq!(easy.hp, default_cfg.hp);
        assert_eq!(easy.fire_interval, default_cfg.fire_interval);
        assert_eq!(easy.fire_spread_degrees, default_cfg.fire_spread_degrees);
    }

    #[test]
    fn test_ufo_config_scaling() {
        let easy = ufo_config_for_wave(1);
        let normal = ufo_config_for_wave(5);
        let hard = ufo_config_for_wave(8);
        let insane = ufo_config_for_wave(10);

        // 速度应该逐渐增加
        assert!(normal.cruise_speed > easy.cruise_speed);
        assert!(hard.cruise_speed > normal.cruise_speed);
        assert!(insane.cruise_speed > hard.cruise_speed);

        // HP 应该逐渐增加
        assert!(normal.hp > easy.hp);
        assert!(hard.hp > normal.hp);
        assert!(insane.hp > hard.hp);

        // 射击间隔应该逐渐减少（更快）
        assert!(normal.fire_interval < easy.fire_interval);
        assert!(hard.fire_interval < normal.fire_interval);
        assert!(insane.fire_interval < hard.fire_interval);

        // 散布角度应该逐渐减少（更准）
        assert!(hard.fire_spread_degrees < easy.fire_spread_degrees);
        assert!(insane.fire_spread_degrees < hard.fire_spread_degrees);

        // 分数奖励应该逐渐增加
        assert!(normal.score_min > easy.score_min);
        assert!(hard.score_min > normal.score_min);
        assert!(insane.score_min > hard.score_min);
    }
}
