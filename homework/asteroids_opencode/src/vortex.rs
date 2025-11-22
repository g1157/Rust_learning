//! 漩涡模块
//!
//! 漩涡会对小行星和飞船施加旋转力，增加游戏难度和趣味性

use macroquad::prelude::*;

/// 漩涡强度 (影响旋转速度)
const VORTEX_STRENGTH: f32 = 150.0;

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
    pub strength: f32,      // 强度倍数 (1.0 = 正常, -1.0 = 反向)
    pub radius: f32,        // 影响半径
    pub created_at: f32,    // 创建时间
    pub lifetime: f32,      // 生命周期
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
    pub fn apply_force(&self, object_pos: Vec2, now: f32) -> Vec2 {
        let diff = object_pos - self.pos;
        let distance = diff.length();

        // 超出影响范围
        if distance > self.radius || distance < 0.1 {
            return Vec2::ZERO;
        }

        // 距离越近，力越强（非线性衰减）
        let distance_factor = 1.0 - (distance / self.radius).powf(2.0);
        
        // 计算切向力（垂直于半径方向）
        let tangent = vec2(-diff.y, diff.x).normalize();
        
        let force_magnitude = VORTEX_STRENGTH 
            * self.current_strength(now) 
            * distance_factor;

        tangent * force_magnitude
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
            
            let arrow_color = Color::new(
                color.r, 
                color.g, 
                color.b, 
                alpha * 0.6
            );
            
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
    pub spawn_interval: f32, // 生成间隔（秒）
    pub base_interval: f32,  // 基础间隔（用于难度计算）
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
            self.vortices.push(Vortex::spawn_random(screen_width, screen_height, now));
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
