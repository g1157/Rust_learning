//! 漩涡模块
//!
//! 漩涡会对小行星和飞船施加旋转力，增加游戏难度和趣味性

use macroquad::prelude::*;

/// 漩涡强度 (影响旋转速度)
const VORTEX_STRENGTH: f32 = 220.0;

/// 漩涡向心吸引力强度（控制被拉向中心的力度）
const VORTEX_PULL_STRENGTH: f32 = 320.0;

/// 漩涡最大影响半径
const VORTEX_RADIUS: f32 = 200.0;

/// 漩涡持续时间（秒）
const VORTEX_DURATION: f32 = 15.0;

/// 漩涡淡出时间（秒）
const VORTEX_FADEOUT: f32 = 3.0;

/// 漩涡
#[derive(Clone, Copy)]
pub struct Vortex {
    pub pos: Vec2,
    pub strength: f32,   // 强度倍数 (1.0 = 正常, -1.0 = 反向)
    pub radius: f32,     // 影响半径
    pub created_at: f32, // 创建时间
    pub lifetime: f32,   // 生命周期
}

impl Vortex {
    /// 在随机位置创建漩涡
    pub fn spawn_random(screen_width: f32, screen_height: f32, now: f32) -> Self {
        // 避免在边缘生成
        let margin = VORTEX_RADIUS;
        let x = rand::gen_range(margin, screen_width - margin);
        let y = rand::gen_range(margin, screen_height - margin);

        // 50% 概率为顺时针或逆时针
        let strength = if rand::gen_range(0.0, 1.0) < 0.5 {
            1.0
        } else {
            -1.0
        };

        Self {
            pos: vec2(x, y),
            strength,
            radius: VORTEX_RADIUS,
            created_at: now,
            lifetime: VORTEX_DURATION,
        }
    }

    /// 检查漩涡是否仍然活跃
    pub fn is_active(&self, now: f32) -> bool {
        now - self.created_at < self.lifetime
    }

    /// 获取当前强度（考虑淡出效果）
    pub fn current_strength(&self, now: f32) -> f32 {
        let elapsed = now - self.created_at;
        if elapsed >= self.lifetime {
            return 0.0;
        }

        // 最后几秒淡出
        let remaining = self.lifetime - elapsed;
        if remaining < VORTEX_FADEOUT {
            return self.strength * (remaining / VORTEX_FADEOUT);
        }

        self.strength
    }

    /// 对物体施加漩涡力（返回速度增量）
    ///
    /// 力由两部分组成：
    /// 1. 切向力（旋转）：使物体围绕漩涡中心旋转
    /// 2. 向心力（吸引）：将物体拉向漩涡中心，越近越强
    pub fn apply_force(&self, object_pos: Vec2, now: f32) -> Vec2 {
        let diff = object_pos - self.pos;
        let distance = diff.length();

        // 超出影响范围
        if distance > self.radius || distance < 0.1 {
            return Vec2::ZERO;
        }

        // 计算归一化距离和接近度（越靠近中心越接近 1）
        let normalized_dist = (distance / self.radius).clamp(0.0, 1.0);
        let proximity = 1.0 - normalized_dist;

        // 计算切向力（垂直于半径方向）- 产生旋转效果
        let tangent = vec2(-diff.y, diff.x).normalize();
        let tangential_force =
            tangent * (VORTEX_STRENGTH * self.current_strength(now) * proximity.powf(1.5));

        // 向心吸引力：始终指向中心，越近越强（三次方增长，产生"吸入"感）
        let inward_dir = -diff.normalize();
        let pull_strength =
            VORTEX_PULL_STRENGTH * self.current_strength(now).abs() * proximity.powf(3.0);
        let radial_force = inward_dir * pull_strength;

        tangential_force + radial_force
    }

    /// 渲染漩涡可视化效果
    pub fn draw(&self, now: f32) {
        let strength = self.current_strength(now);
        if strength.abs() < 0.01 {
            return;
        }

        let alpha = strength.abs();

        // 漩涡颜色：顺时针为蓝色，逆时针为红色
        let color = if self.strength > 0.0 {
            Color::new(0.3, 0.5, 1.0, alpha * 0.3)
        } else {
            Color::new(1.0, 0.3, 0.3, alpha * 0.3)
        };

        // 绘制影响范围圆
        draw_circle(self.pos.x, self.pos.y, self.radius, color);

        // 绘制旋转箭头指示方向
        let num_arrows = 8;
        let rotation_offset = (now - self.created_at) * 2.0; // 旋转动画

        for i in 0..num_arrows {
            let angle = (i as f32 / num_arrows as f32) * std::f32::consts::TAU
                + rotation_offset * self.strength;

            let r = self.radius * 0.7;
            let start = self.pos + vec2(angle.cos(), angle.sin()) * r;

            // 切向箭头
            let arrow_angle = angle + std::f32::consts::FRAC_PI_2 * self.strength.signum();
            let end = start + vec2(arrow_angle.cos(), arrow_angle.sin()) * 20.0;

            let arrow_color = Color::new(color.r, color.g, color.b, alpha * 0.6);

            draw_line(start.x, start.y, end.x, end.y, 2.0, arrow_color);

            // 箭头头部
            let tip_angle = arrow_angle - std::f32::consts::FRAC_PI_6 * self.strength.signum();
            let tip = end - vec2(tip_angle.cos(), tip_angle.sin()) * 8.0;
            draw_line(end.x, end.y, tip.x, tip.y, 2.0, arrow_color);
        }

        // 中心点
        draw_circle(self.pos.x, self.pos.y, 5.0, WHITE);
    }
}

/// 漩涡管理器
pub struct VortexManager {
    pub vortices: Vec<Vortex>,
    pub next_spawn_time: f32,
    pub spawn_interval: f32,  // 生成间隔（秒）
    pub base_interval: f32,   // 基础间隔（用于难度计算）
    pub game_start_time: f32, // 游戏开始时间
}

impl VortexManager {
    pub fn new(spawn_interval: f32) -> Self {
        Self {
            vortices: Vec::new(),
            next_spawn_time: spawn_interval,
            spawn_interval,
            base_interval: spawn_interval,
            game_start_time: 0.0,
        }
    }

    /// 更新漩涡状态，移除过期的漩涡，随时间增加难度
    pub fn update(&mut self, now: f32, screen_width: f32, screen_height: f32) {
        // 移除过期的漩涡
        self.vortices.retain(|v| v.is_active(now));

        // 计算难度增长：每分钟减少10%间隔，最少5秒
        let elapsed_minutes = (now - self.game_start_time) / 60.0;
        let difficulty_factor = (1.0 - elapsed_minutes * 0.1).max(0.25);
        self.spawn_interval = (self.base_interval * difficulty_factor).max(5.0);

        // 检查是否该生成新漩涡（最多同时3个）
        if now >= self.next_spawn_time && self.vortices.len() < 3 {
            self.vortices
                .push(Vortex::spawn_random(screen_width, screen_height, now));
            self.next_spawn_time = now + self.spawn_interval;
        }
    }

    /// 对物体施加所有漩涡的合力
    pub fn apply_forces(&self, object_pos: Vec2, now: f32) -> Vec2 {
        self.vortices
            .iter()
            .map(|v| v.apply_force(object_pos, now))
            .fold(Vec2::ZERO, |acc, force| acc + force)
    }

    /// 渲染所有漩涡
    pub fn draw(&self, now: f32) {
        for vortex in &self.vortices {
            vortex.draw(now);
        }
    }

    /// 清空所有漩涡并重置计时器
    pub fn clear(&mut self) {
        self.vortices.clear();
        self.next_spawn_time = self.base_interval;
        self.game_start_time = 0.0;
    }

    /// 重置游戏开始时间（用于新游戏）
    pub fn reset_game_time(&mut self, now: f32) {
        self.game_start_time = now;
        self.next_spawn_time = now + self.base_interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vortex(strength: f32) -> Vortex {
        Vortex {
            pos: vec2(100.0, 100.0),
            strength,
            radius: VORTEX_RADIUS,
            created_at: 0.0,
            lifetime: VORTEX_DURATION,
        }
    }

    #[test]
    fn test_vortex_no_force_outside_radius() {
        let vortex = create_test_vortex(1.0);
        // 物体在漩涡影响范围外
        let far_pos = vec2(100.0 + VORTEX_RADIUS + 50.0, 100.0);
        let force = vortex.apply_force(far_pos, 1.0);
        assert_eq!(force, Vec2::ZERO);
    }

    #[test]
    fn test_vortex_has_inward_component() {
        let vortex = create_test_vortex(1.0);
        // 物体在漩涡右侧中等距离
        let pos = vec2(100.0 + VORTEX_RADIUS * 0.5, 100.0);
        let force = vortex.apply_force(pos, 1.0);

        // 力应该有指向中心的分量（x 应该为负）
        assert!(force.x < 0.0, "Force should have inward x component");
    }

    #[test]
    fn test_vortex_force_increases_near_center() {
        let vortex = create_test_vortex(1.0);

        // 远距离位置
        let far_pos = vec2(100.0 + VORTEX_RADIUS * 0.8, 100.0);
        let far_force = vortex.apply_force(far_pos, 1.0);

        // 近距离位置
        let near_pos = vec2(100.0 + VORTEX_RADIUS * 0.3, 100.0);
        let near_force = vortex.apply_force(near_pos, 1.0);

        // 近距离的力应该更强
        assert!(
            near_force.length() > far_force.length(),
            "Force should be stronger near center"
        );
    }

    #[test]
    fn test_vortex_has_tangential_component() {
        let vortex = create_test_vortex(1.0);
        // 物体在漩涡右侧
        let pos = vec2(100.0 + VORTEX_RADIUS * 0.5, 100.0);
        let force = vortex.apply_force(pos, 1.0);

        // 对于顺时针漩涡（strength > 0），右侧物体应该有向下的切向分量
        // 因为切向向量是 (-diff.y, diff.x)，diff 是 (正, 0)，所以切向是 (0, 正)
        assert!(
            force.y.abs() > 0.01,
            "Force should have tangential y component"
        );
    }

    #[test]
    fn test_vortex_strength_fadeout() {
        let vortex = create_test_vortex(1.0);

        // 刚创建时，强度应该是满的
        let early_strength = vortex.current_strength(1.0);
        assert!((early_strength - 1.0).abs() < 0.01);

        // 接近结束时，强度应该减弱
        let late_time = VORTEX_DURATION - VORTEX_FADEOUT * 0.5;
        let late_strength = vortex.current_strength(late_time);
        assert!(late_strength < early_strength);
        assert!(late_strength > 0.0);

        // 结束后，强度应该是 0
        let expired_strength = vortex.current_strength(VORTEX_DURATION + 1.0);
        assert_eq!(expired_strength, 0.0);
    }

    #[test]
    fn test_vortex_is_active() {
        let vortex = create_test_vortex(1.0);

        assert!(vortex.is_active(1.0));
        assert!(vortex.is_active(VORTEX_DURATION - 0.1));
        assert!(!vortex.is_active(VORTEX_DURATION + 0.1));
    }
}
