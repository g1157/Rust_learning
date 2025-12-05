//! 增强渲染模块
//!
//! 提供飞船、子弹等游戏对象的高质量程序化渲染。
//!
//! ## 功能
//! - 多层次飞船绘制（机身、驾驶舱、引擎）
//! - 子弹发光效果
//! - 追踪导弹详细渲染
//! - 引擎火焰动画

use macroquad::prelude::*;

use crate::bullet::{BULLET_RADIUS, Bullet, WeaponType};
use crate::constants::homing;
use crate::ship::{SHIP_BASE, SHIP_HEIGHT};

/// 绘制增强版飞船
///
/// 包含：机身轮廓、内部细节、驾驶舱、引擎喷口
pub fn draw_ship(
    pos: Vec2,
    rot: f32,
    color: Color,
    is_invulnerable: bool,
    is_thrusting: bool,
    time: f32,
) {
    let rotation = rot.to_radians();
    let sin_r = rotation.sin();
    let cos_r = rotation.cos();

    // 前方向量
    let forward = Vec2::new(sin_r, -cos_r);
    // 右方向量
    let right = Vec2::new(cos_r, sin_r);

    // 透明度（无敌时闪烁）
    let alpha = if is_invulnerable {
        0.3 + 0.2 * (time * 10.0).sin()
    } else {
        1.0
    };

    // === 主机身 ===
    // 飞船顶点
    let nose = pos + forward * (SHIP_HEIGHT / 2.0); // 船头
    let left_wing = pos - forward * (SHIP_HEIGHT / 2.0) - right * (SHIP_BASE / 2.0);
    let right_wing = pos - forward * (SHIP_HEIGHT / 2.0) + right * (SHIP_BASE / 2.0);
    let back_center = pos - forward * (SHIP_HEIGHT / 3.0); // 后部中心（凹陷）

    // 绘制填充的机身（两个三角形组成的箭头形状）
    let fill_color = Color::new(color.r * 0.3, color.g * 0.3, color.b * 0.3, alpha * 0.6);
    draw_triangle(nose, left_wing, back_center, fill_color);
    draw_triangle(nose, back_center, right_wing, fill_color);

    // 绘制机身轮廓
    let outline_color = Color::new(color.r, color.g, color.b, alpha);
    let line_width = 2.5;

    // 左侧边
    draw_line(
        nose.x,
        nose.y,
        left_wing.x,
        left_wing.y,
        line_width,
        outline_color,
    );
    // 右侧边
    draw_line(
        nose.x,
        nose.y,
        right_wing.x,
        right_wing.y,
        line_width,
        outline_color,
    );
    // 后部凹陷
    draw_line(
        left_wing.x,
        left_wing.y,
        back_center.x,
        back_center.y,
        line_width * 0.7,
        outline_color,
    );
    draw_line(
        back_center.x,
        back_center.y,
        right_wing.x,
        right_wing.y,
        line_width * 0.7,
        outline_color,
    );

    // === 驾驶舱 ===
    let cockpit_pos = pos + forward * (SHIP_HEIGHT * 0.1);
    let cockpit_size = SHIP_BASE * 0.2;
    let cockpit_color = Color::new(0.4, 0.8, 1.0, alpha * 0.8);
    draw_circle(cockpit_pos.x, cockpit_pos.y, cockpit_size, cockpit_color);
    // 驾驶舱高光
    let highlight_pos = cockpit_pos + Vec2::new(-cockpit_size * 0.3, -cockpit_size * 0.3);
    draw_circle(
        highlight_pos.x,
        highlight_pos.y,
        cockpit_size * 0.3,
        Color::new(1.0, 1.0, 1.0, alpha * 0.5),
    );

    // === 引擎喷口 ===
    let engine_left = pos - forward * (SHIP_HEIGHT * 0.35) - right * (SHIP_BASE * 0.25);
    let engine_right = pos - forward * (SHIP_HEIGHT * 0.35) + right * (SHIP_BASE * 0.25);
    let engine_size = 3.0;
    let engine_color = Color::new(0.5, 0.5, 0.6, alpha);
    draw_circle(engine_left.x, engine_left.y, engine_size, engine_color);
    draw_circle(engine_right.x, engine_right.y, engine_size, engine_color);

    // === 引擎火焰（推进时） ===
    if is_thrusting {
        draw_engine_flame(engine_left, forward, time, alpha);
        draw_engine_flame(engine_right, forward, time, alpha);
    }

    // === 机翼装饰线 ===
    let wing_detail_color = Color::new(color.r * 0.7, color.g * 0.7, color.b * 0.7, alpha * 0.8);
    let left_detail = pos - right * (SHIP_BASE * 0.3);
    let right_detail = pos + right * (SHIP_BASE * 0.3);
    draw_line(
        left_detail.x,
        left_detail.y,
        (left_wing.x + back_center.x) * 0.5,
        (left_wing.y + back_center.y) * 0.5,
        1.5,
        wing_detail_color,
    );
    draw_line(
        right_detail.x,
        right_detail.y,
        (right_wing.x + back_center.x) * 0.5,
        (right_wing.y + back_center.y) * 0.5,
        1.5,
        wing_detail_color,
    );
}

/// 绘制引擎火焰效果
fn draw_engine_flame(engine_pos: Vec2, forward: Vec2, time: f32, alpha: f32) {
    // 火焰长度随时间波动
    let flame_length = 8.0 + 4.0 * (time * 20.0).sin().abs();
    let flame_width = 4.0 + 2.0 * (time * 15.0 + 1.0).cos().abs();

    let flame_tip = engine_pos - forward * flame_length;
    let right = Vec2::new(forward.y, -forward.x);

    // 外层火焰（橙色）
    let outer_color = Color::new(1.0, 0.5, 0.1, alpha * 0.8);
    let left_base = engine_pos - right * flame_width * 0.5;
    let right_base = engine_pos + right * flame_width * 0.5;
    draw_triangle(left_base, right_base, flame_tip, outer_color);

    // 内层火焰（黄色）
    let inner_length = flame_length * 0.6;
    let inner_width = flame_width * 0.5;
    let inner_tip = engine_pos - forward * inner_length;
    let inner_left = engine_pos - right * inner_width * 0.5;
    let inner_right = engine_pos + right * inner_width * 0.5;
    let inner_color = Color::new(1.0, 0.9, 0.3, alpha * 0.9);
    draw_triangle(inner_left, inner_right, inner_tip, inner_color);

    // 核心（白色）
    let core_length = flame_length * 0.3;
    let core_tip = engine_pos - forward * core_length;
    let core_color = Color::new(1.0, 1.0, 0.9, alpha);
    draw_circle(engine_pos.x, engine_pos.y, 2.0, core_color);
    draw_line(
        engine_pos.x,
        engine_pos.y,
        core_tip.x,
        core_tip.y,
        2.0,
        core_color,
    );
}

/// 绘制增强版子弹
pub fn draw_bullet(bullet: &Bullet, offset: Vec2, player_color: Color, time: f32) {
    let pos = bullet.pos + offset;

    match bullet.weapon_type {
        WeaponType::Normal => draw_normal_bullet(pos, player_color, time),
        WeaponType::Spread => draw_spread_bullet(pos, player_color, time),
        WeaponType::Penetrating => draw_penetrating_bullet(pos, bullet.vel, player_color, time),
        WeaponType::Homing => draw_homing_missile(pos, bullet.vel, player_color, time),
    }
}

/// 普通子弹：带发光效果的圆形
fn draw_normal_bullet(pos: Vec2, color: Color, time: f32) {
    let pulse = 1.0 + 0.2 * (time * 15.0).sin();
    let size = BULLET_RADIUS * pulse;

    // 外发光
    let glow_color = Color::new(color.r, color.g, color.b, 0.3);
    draw_circle(pos.x, pos.y, size * 2.0, glow_color);

    // 主体
    draw_circle(pos.x, pos.y, size, color);

    // 核心高光
    let core_color = Color::new(1.0, 1.0, 1.0, 0.8);
    draw_circle(
        pos.x - size * 0.2,
        pos.y - size * 0.2,
        size * 0.4,
        core_color,
    );
}

/// 散弹：较小的发光点
fn draw_spread_bullet(pos: Vec2, color: Color, time: f32) {
    let pulse = 1.0 + 0.15 * (time * 20.0).sin();
    let size = BULLET_RADIUS * 0.7 * pulse;

    // 淡蓝色调
    let spread_color = Color::new(color.r * 0.7 + 0.3, color.g * 0.7 + 0.3, 1.0, 1.0);

    // 外发光
    let glow_color = Color::new(spread_color.r, spread_color.g, spread_color.b, 0.25);
    draw_circle(pos.x, pos.y, size * 2.5, glow_color);

    // 主体
    draw_circle(pos.x, pos.y, size, spread_color);
}

/// 穿透弹：拉长的能量弹
fn draw_penetrating_bullet(pos: Vec2, vel: Vec2, _color: Color, time: f32) {
    let dir = vel.normalize();
    let pulse = 1.0 + 0.1 * (time * 25.0).sin();

    // 橙色能量弹
    let core_color = Color::new(1.0, 0.6, 0.1, 1.0);
    let glow_color = Color::new(1.0, 0.4, 0.0, 0.4);

    let length = BULLET_RADIUS * 3.0 * pulse;
    let width = BULLET_RADIUS * 1.2;

    // 外发光（椭圆形）
    let tip = pos + dir * length;
    let tail = pos - dir * length * 0.5;
    draw_line(tail.x, tail.y, tip.x, tip.y, width * 3.0, glow_color);

    // 主体
    draw_line(tail.x, tail.y, tip.x, tip.y, width * 1.5, core_color);

    // 核心
    let white = Color::new(1.0, 0.9, 0.7, 1.0);
    draw_circle(pos.x, pos.y, width * 0.5, white);
}

/// 追踪导弹：详细的导弹形状
fn draw_homing_missile(pos: Vec2, vel: Vec2, player_color: Color, time: f32) {
    let dir = vel.normalize();
    let right = Vec2::new(-dir.y, dir.x);
    let size = homing::RADIUS;

    // 导弹主体颜色（绿色调）
    let body_color = Color::new(
        player_color.r * 0.3 + 0.2,
        0.7 + player_color.g * 0.2,
        player_color.b * 0.2 + 0.2,
        1.0,
    );
    let dark_color = Color::new(
        body_color.r * 0.5,
        body_color.g * 0.5,
        body_color.b * 0.5,
        1.0,
    );

    // === 导弹头部（尖锥） ===
    let nose = pos + dir * size * 2.5;
    let body_front = pos + dir * size * 0.5;
    let body_front_left = body_front - right * size * 0.6;
    let body_front_right = body_front + right * size * 0.6;

    // 头部锥形
    draw_triangle(nose, body_front_left, body_front_right, body_color);

    // === 导弹主体 ===
    let body_back = pos - dir * size * 1.5;
    let body_back_left = body_back - right * size * 0.6;
    let body_back_right = body_back + right * size * 0.6;

    // 主体矩形（两个三角形）
    draw_triangle(
        body_front_left,
        body_front_right,
        body_back_right,
        dark_color,
    );
    draw_triangle(body_front_left, body_back_right, body_back_left, dark_color);

    // === 尾翼 ===
    let fin_size = size * 1.2;
    let fin_back = body_back - dir * size * 0.3;

    // 左尾翼
    let fin_left_tip = fin_back - right * fin_size;
    draw_triangle(body_back_left, fin_back, fin_left_tip, body_color);

    // 右尾翼
    let fin_right_tip = fin_back + right * fin_size;
    draw_triangle(body_back_right, fin_back, fin_right_tip, body_color);

    // === 追踪指示灯（闪烁） ===
    let blink = ((time * 8.0).sin() + 1.0) * 0.5;
    let indicator_color = Color::new(1.0, 0.2, 0.2, blink);
    let indicator_pos = pos + dir * size * 0.8;
    draw_circle(
        indicator_pos.x,
        indicator_pos.y,
        size * 0.25,
        indicator_color,
    );

    // === 引擎火焰 ===
    let flame_length = size * 1.5 + size * 0.8 * (time * 30.0).sin().abs();
    let flame_tip = body_back - dir * flame_length;

    // 外焰（橙色）
    let outer_flame = Color::new(1.0, 0.5, 0.1, 0.9);
    let flame_left = body_back - right * size * 0.4;
    let flame_right = body_back + right * size * 0.4;
    draw_triangle(flame_left, flame_right, flame_tip, outer_flame);

    // 内焰（黄色）
    let inner_flame = Color::new(1.0, 0.9, 0.4, 1.0);
    let inner_tip = body_back - dir * flame_length * 0.5;
    let inner_left = body_back - right * size * 0.2;
    let inner_right = body_back + right * size * 0.2;
    draw_triangle(inner_left, inner_right, inner_tip, inner_flame);
}

/// 绘制护盾效果
pub fn draw_shield(pos: Vec2, time: f32, remaining_ratio: f32) {
    let radius = SHIP_HEIGHT * 1.2;
    let intensity = remaining_ratio.clamp(0.2, 1.0);

    // 多层护盾环
    let base_alpha = 0.15 + 0.25 * intensity;

    // 外环（波动）
    let outer_pulse = 1.0 + 0.1 * (time * 3.0).sin();
    let outer_color = Color::new(0.2, 0.6, 1.0, base_alpha * 0.5);
    draw_circle_lines(pos.x, pos.y, radius * outer_pulse, 2.0, outer_color);

    // 中环
    let mid_color = Color::new(0.3, 0.7, 1.0, base_alpha * 0.7);
    draw_circle_lines(pos.x, pos.y, radius * 0.9, 3.0, mid_color);

    // 内环（填充）
    let inner_color = Color::new(0.2, 0.5, 0.9, base_alpha * 0.2);
    draw_circle(pos.x, pos.y, radius * 0.85, inner_color);

    // 能量粒子效果
    let particle_count = 6;
    for i in 0..particle_count {
        let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU + time * 2.0;
        let particle_pos = pos + Vec2::new(angle.cos(), angle.sin()) * radius * 0.9;
        let particle_alpha = 0.3 + 0.4 * ((time * 5.0 + i as f32).sin() + 1.0) * 0.5;
        let particle_color = Color::new(0.5, 0.8, 1.0, particle_alpha * intensity);
        draw_circle(particle_pos.x, particle_pos.y, 3.0, particle_color);
    }
}

/// 绘制冲刺残影效果
///
/// 根据残影轨迹绘制渐变透明的飞船轮廓
pub fn draw_dash_trail(trail: &[(Vec2, f32, f64)], color: Color, current_time: f64) {
    for (pos, rot, spawn_time) in trail.iter() {
        // 计算透明度（随时间衰减）
        let age = (current_time - *spawn_time) as f32;
        let alpha = (1.0 - age / 0.3).clamp(0.0, 0.6); // 最大透明度 0.6，0.3秒内衰减

        if alpha <= 0.0 {
            continue;
        }

        let rotation = rot.to_radians();
        let sin_r = rotation.sin();
        let cos_r = rotation.cos();

        let forward = Vec2::new(sin_r, -cos_r);
        let right = Vec2::new(cos_r, sin_r);

        // 飞船顶点
        let nose = *pos + forward * (SHIP_HEIGHT / 2.0);
        let left_wing = *pos - forward * (SHIP_HEIGHT / 2.0) - right * (SHIP_BASE / 2.0);
        let right_wing = *pos - forward * (SHIP_HEIGHT / 2.0) + right * (SHIP_BASE / 2.0);

        // 绘制半透明轮廓
        let trail_color = Color::new(color.r, color.g, color.b, alpha);
        let line_width = 1.5;

        draw_line(
            nose.x,
            nose.y,
            left_wing.x,
            left_wing.y,
            line_width,
            trail_color,
        );
        draw_line(
            nose.x,
            nose.y,
            right_wing.x,
            right_wing.y,
            line_width,
            trail_color,
        );
        draw_line(
            left_wing.x,
            left_wing.y,
            right_wing.x,
            right_wing.y,
            line_width,
            trail_color,
        );
    }
}

/// 绘制冲刺指示器（冷却状态）
pub fn draw_dash_indicator(pos: Vec2, cooldown_ratio: f32, player_color: Color) {
    if cooldown_ratio <= 0.0 {
        return; // 冷却完成，不绘制
    }

    // 在飞船下方绘制冷却条
    let bar_width = 20.0;
    let bar_height = 3.0;
    let bar_pos = pos + Vec2::new(-bar_width / 2.0, SHIP_HEIGHT * 0.8);

    // 背景条
    let bg_color = Color::new(0.2, 0.2, 0.2, 0.5);
    draw_rectangle(bar_pos.x, bar_pos.y, bar_width, bar_height, bg_color);

    // 冷却进度条
    let progress = 1.0 - cooldown_ratio;
    let fill_color = Color::new(player_color.r, player_color.g, player_color.b, 0.7);
    draw_rectangle(
        bar_pos.x,
        bar_pos.y,
        bar_width * progress,
        bar_height,
        fill_color,
    );
}

/// 绘制超空间跳跃冷却指示器
///
/// 在飞船上方绘制一个紫色的冷却条，表示超空间跳跃的冷却进度
pub fn draw_hyperspace_indicator(pos: Vec2, cooldown_ratio: f32) {
    if cooldown_ratio <= 0.0 {
        return; // 冷却完成，不绘制
    }

    // 在飞船上方绘制冷却条（与冲刺指示器相对）
    let bar_width = 20.0;
    let bar_height = 3.0;
    let bar_pos = pos + Vec2::new(-bar_width / 2.0, -SHIP_HEIGHT * 1.2);

    // 背景条
    let bg_color = Color::new(0.2, 0.2, 0.2, 0.5);
    draw_rectangle(bar_pos.x, bar_pos.y, bar_width, bar_height, bg_color);

    // 冷却进度条（紫色，代表超空间）
    let progress = 1.0 - cooldown_ratio;
    let hyperspace_color = Color::new(0.6, 0.2, 0.9, 0.7);
    draw_rectangle(
        bar_pos.x,
        bar_pos.y,
        bar_width * progress,
        bar_height,
        hyperspace_color,
    );
}
