//! 增强渲染模块
//!
//! 提供飞船、子弹等游戏对象的高质量程序化渲染。
//!
//! ## 功能
//! - 多层次飞船绘制（机身、驾驶舱、引擎）
//! - 子弹发光效果
//! - 追踪导弹详细渲染
//! - 引擎火焰动画
//! - 乐观命中提示效果

#![allow(dead_code)] // 乐观命中渲染正在集成中

use macroquad::prelude::*;

use crate::bullet::{BULLET_RADIUS, Bullet, WeaponType};
use crate::constants::homing;
use crate::effects::{CONFIRM_GRACE, DENIED_FLASH, PendingHit, PendingHitKind, PendingHitState};
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
        WeaponType::ChainIon => draw_chain_ion_bullet(pos, player_color, time),
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

/// 链式离子炮弹：蓝白色能量球，带电弧环
fn draw_chain_ion_bullet(pos: Vec2, player_color: Color, time: f32) {
    let pulse = 1.0 + 0.25 * (time * 18.0).sin();
    let size = BULLET_RADIUS * 1.4 * pulse;

    // 蓝白色调，混合玩家颜色
    let ion_color = Color::new(
        player_color.r * 0.3 + 0.4,
        player_color.g * 0.4 + 0.5,
        1.0, // 高蓝色分量
        1.0,
    );

    // 外发光层（大范围淡蓝）
    let glow_outer = Color::new(0.3, 0.6, 1.0, 0.2);
    draw_circle(pos.x, pos.y, size * 3.5, glow_outer);

    // 中发光层
    let glow_mid = Color::new(0.5, 0.8, 1.0, 0.35);
    draw_circle(pos.x, pos.y, size * 2.2, glow_mid);

    // 主体能量球
    draw_circle(pos.x, pos.y, size, ion_color);

    // 核心高亮（白色）
    let core_color = Color::new(1.0, 1.0, 1.0, 0.9);
    draw_circle(pos.x, pos.y, size * 0.4, core_color);

    // 绘制电弧环（旋转的小弧线）
    let arc_count = 4;
    let arc_radius = size * 2.0;
    for i in 0..arc_count {
        let base_angle = (i as f32 / arc_count as f32) * std::f32::consts::TAU;
        let rotation = time * 6.0; // 旋转速度
        let angle = base_angle + rotation;

        // 每个弧线由几个点组成
        let arc_len = 0.4; // 弧长（弧度）
        let segments = 4;
        for j in 0..segments {
            let t0 = j as f32 / segments as f32;
            let t1 = (j + 1) as f32 / segments as f32;
            let a0 = angle + t0 * arc_len;
            let a1 = angle + t1 * arc_len;

            let p0 = pos + Vec2::new(a0.cos(), a0.sin()) * arc_radius;
            let p1 = pos + Vec2::new(a1.cos(), a1.sin()) * arc_radius;

            // 电弧抖动
            let jitter = (time * 25.0 + i as f32 * 2.0 + j as f32).sin() * 2.0;
            let normal = Vec2::new(-(a0 + a1).sin() * 0.5, (a0 + a1).cos() * 0.5);
            let p0_j = p0 + normal * jitter;
            let p1_j = p1 + normal * jitter;

            let arc_alpha = 0.5 + 0.3 * (time * 12.0 + i as f32).sin().abs();
            let arc_color = Color::new(0.6, 0.9, 1.0, arc_alpha);
            draw_line(p0_j.x, p0_j.y, p1_j.x, p1_j.y, 1.8, arc_color);
        }
    }
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

// ============================================================================
// 乐观命中提示渲染 (Phase 4C)
// ============================================================================

/// 绘制乐观命中提示效果
///
/// 根据命中状态显示不同的视觉反馈：
/// - Pending: 黄色/橙色脉冲环，表示等待确认
/// - Confirmed: 绿色扩散淡出，表示命中成功
/// - Denied: 红色闪烁，表示命中无效
pub fn draw_pending_hit(hit: &PendingHit, now: f32) {
    let elapsed = (now - hit.created_at).max(0.0);

    // 根据目标类型设置基础半径
    let base_radius = match hit.kind {
        PendingHitKind::Asteroid => 18.0,
        PendingHitKind::Ufo => 22.0,
    };

    match hit.state {
        PendingHitState::Pending => {
            // 黄色/橙色脉冲环，表示等待确认
            let pulse = (elapsed * 8.0).sin().abs();
            let radius = base_radius + 6.0 * pulse;
            let alpha = 0.35 + 0.25 * pulse;

            // 外环（黄色）
            let ring_color = Color::new(1.0, 0.8, 0.2, alpha);
            draw_circle_lines(hit.pos.x, hit.pos.y, radius, 3.0, ring_color);

            // 内部发光（橙色）
            let glow_color = Color::new(1.0, 0.5, 0.1, alpha * 0.6);
            draw_circle(hit.pos.x, hit.pos.y, radius * 0.35, glow_color);
        }
        PendingHitState::Confirmed => {
            // 绿色扩散淡出，表示命中成功
            let t = ((now - hit.state_changed_at) / CONFIRM_GRACE).clamp(0.0, 1.0);
            let radius = base_radius + 10.0 * (1.0 - t);
            let alpha = (1.0 - t) * 0.9;

            // 外环（绿色）
            let ring_color = Color::new(0.3, 1.0, 0.5, alpha);
            draw_circle_lines(hit.pos.x, hit.pos.y, radius, 4.0, ring_color);

            // 内部发光（淡绿）
            let glow_color = Color::new(0.5, 1.0, 0.7, alpha * 0.6);
            draw_circle(hit.pos.x, hit.pos.y, radius * 0.4, glow_color);
        }
        PendingHitState::Denied => {
            // 红色闪烁，表示命中无效
            let phase = ((now - hit.state_changed_at) / DENIED_FLASH).clamp(0.0, 1.0);
            let blink = (now * 30.0).sin().abs();
            let alpha = (1.0 - phase) * (0.4 + 0.4 * blink);
            let radius = base_radius + 4.0;

            // 外环（红色）
            let ring_color = Color::new(1.0, 0.2, 0.2, alpha);
            draw_circle_lines(hit.pos.x, hit.pos.y, radius, 3.5, ring_color);

            // 内部发光（淡红）
            let glow_color = Color::new(1.0, 0.4, 0.3, alpha * 0.8);
            draw_circle(hit.pos.x, hit.pos.y, radius * 0.25, glow_color);
        }
    }
}

// ============================================================================
// 连击视觉反馈系统
// ============================================================================

/// 绘制飞船连击发光效果
///
/// 根据连击视觉等级（0-4）绘制不同强度的发光圈
/// - 0: 无效果
/// - 1: 微光（白色光晕）
/// - 2: 发光（彩色光晕）
/// - 3: 强光（多层光晕 + 脉冲）
/// - 4: 极光（最强光晕 + 强烈脉冲）
pub fn draw_ship_glow(pos: Vec2, color: Color, visual_level: u32, time: f32) {
    if visual_level == 0 {
        return;
    }

    // 脉冲效果强度随等级增加
    let pulse_speed = 4.0 + visual_level as f32 * 2.0;
    let pulse = 0.7 + 0.3 * (time * pulse_speed).sin();

    // 基础半径和透明度随等级增加
    let base_radius = 20.0 + visual_level as f32 * 8.0;
    let base_alpha = 0.15 + visual_level as f32 * 0.08;

    // 外层光晕（较大、较淡）
    let outer_radius = base_radius * 1.5 * pulse;
    let outer_alpha = base_alpha * 0.5;
    let outer_color = Color::new(color.r, color.g, color.b, outer_alpha);
    draw_circle(pos.x, pos.y, outer_radius, outer_color);

    // 中层光晕
    let mid_radius = base_radius * pulse;
    let mid_alpha = base_alpha * 0.7;
    let mid_color = Color::new(color.r, color.g, color.b, mid_alpha);
    draw_circle(pos.x, pos.y, mid_radius, mid_color);

    // 内层光晕（最亮）
    let inner_radius = base_radius * 0.6 * pulse;
    let inner_alpha = base_alpha;
    let inner_color = Color::new(
        (color.r + 0.3).min(1.0),
        (color.g + 0.3).min(1.0),
        (color.b + 0.3).min(1.0),
        inner_alpha,
    );
    draw_circle(pos.x, pos.y, inner_radius, inner_color);

    // 等级 3+ 额外添加光环线条
    if visual_level >= 3 {
        let ring_radius = base_radius * 1.2 * pulse;
        let ring_alpha = base_alpha * 1.5;
        let ring_color = Color::new(1.0, 1.0, 1.0, ring_alpha);
        draw_circle_lines(pos.x, pos.y, ring_radius, 2.0, ring_color);
    }

    // 等级 4 添加额外的外部光环
    if visual_level >= 4 {
        let outer_ring_radius = base_radius * 2.0 * pulse;
        let outer_ring_alpha = base_alpha * 0.8;
        let outer_ring_color = Color::new(color.r, color.g, color.b, outer_ring_alpha);
        draw_circle_lines(pos.x, pos.y, outer_ring_radius, 1.5, outer_ring_color);
    }
}

/// 绘制连击屏幕边缘特效（渐晕效果）
///
/// 在屏幕边缘绘制发光的渐变效果，表示高连击状态
/// intensity: 0.0 - 1.0 的强度值
pub fn draw_killstreak_vignette(color: Color, intensity: f32, time: f32) {
    if intensity <= 0.0 {
        return;
    }

    let sw = screen_width();
    let sh = screen_height();

    // 脉冲效果
    let pulse = 0.7 + 0.3 * (time * 3.0).sin();
    let alpha = intensity * 0.4 * pulse;

    // 渐变边缘宽度
    let edge_width = 60.0 + intensity * 40.0;

    // 创建半透明的边缘颜色
    let vignette_color = Color::new(color.r, color.g, color.b, alpha);

    // 绘制四条边的渐变矩形
    // 顶部
    draw_rectangle(0.0, 0.0, sw, edge_width, vignette_color);
    // 底部
    draw_rectangle(0.0, sh - edge_width, sw, edge_width, vignette_color);
    // 左侧
    draw_rectangle(0.0, 0.0, edge_width, sh, vignette_color);
    // 右侧
    draw_rectangle(sw - edge_width, 0.0, edge_width, sh, vignette_color);

    // 高强度时添加内层边缘
    if intensity > 0.5 {
        let inner_alpha = (intensity - 0.5) * 0.3 * pulse;
        let inner_color = Color::new(color.r, color.g, color.b, inner_alpha);
        let inner_edge = edge_width * 0.5;

        draw_rectangle(0.0, 0.0, sw, inner_edge, inner_color);
        draw_rectangle(0.0, sh - inner_edge, sw, inner_edge, inner_color);
        draw_rectangle(0.0, 0.0, inner_edge, sh, inner_color);
        draw_rectangle(sw - inner_edge, 0.0, inner_edge, sh, inner_color);
    }
}

/// 绘制幽灵模式效果（半透明飞船 + 残影）
///
/// 在飞船周围绘制幽灵般的视觉效果，表示50%闪避状态
pub fn draw_ghost_mode_effect(pos: Vec2, rot: f32, _color: Color, time: f32) {
    let rotation = rot.to_radians();
    let sin_r = rotation.sin();
    let cos_r = rotation.cos();

    let forward = Vec2::new(sin_r, -cos_r);
    let right = Vec2::new(cos_r, sin_r);

    // 幽灵残影（多个偏移的半透明轮廓）
    let ghost_offsets = [
        (Vec2::new(-8.0, -5.0), 0.15),
        (Vec2::new(6.0, -3.0), 0.12),
        (Vec2::new(-4.0, 7.0), 0.10),
        (Vec2::new(5.0, 5.0), 0.08),
    ];

    // 脉动效果
    let pulse = 0.6 + 0.4 * (time * 4.0).sin();

    for (offset, alpha_mult) in ghost_offsets {
        let ghost_pos = pos + offset * pulse;
        let ghost_alpha = alpha_mult * pulse;
        let ghost_color = Color::new(0.7, 0.7, 0.9, ghost_alpha);

        // 简化的飞船轮廓
        let nose = ghost_pos + forward * (SHIP_HEIGHT / 2.0);
        let left_wing = ghost_pos - forward * (SHIP_HEIGHT / 2.0) - right * (SHIP_BASE / 2.0);
        let right_wing = ghost_pos - forward * (SHIP_HEIGHT / 2.0) + right * (SHIP_BASE / 2.0);

        draw_line(nose.x, nose.y, left_wing.x, left_wing.y, 1.5, ghost_color);
        draw_line(nose.x, nose.y, right_wing.x, right_wing.y, 1.5, ghost_color);
        draw_line(
            left_wing.x,
            left_wing.y,
            right_wing.x,
            right_wing.y,
            1.5,
            ghost_color,
        );
    }

    // 中心光晕
    let glow_radius = SHIP_HEIGHT * 0.8 * pulse;
    draw_circle(
        pos.x,
        pos.y,
        glow_radius,
        Color::new(0.6, 0.6, 0.9, 0.1 * pulse),
    );

    // 闪烁的星点效果
    let star_count = 6;
    for i in 0..star_count {
        let angle = (time * 2.0 + i as f32 * std::f32::consts::TAU / star_count as f32)
            % std::f32::consts::TAU;
        let dist = SHIP_HEIGHT * 0.6 + 10.0 * (time * 3.0 + i as f32).sin();
        let star_pos = pos + Vec2::new(angle.cos(), angle.sin()) * dist;
        let star_alpha = 0.3 + 0.3 * ((time * 5.0 + i as f32 * 0.5).sin().abs());

        draw_circle(
            star_pos.x,
            star_pos.y,
            2.0,
            Color::new(0.8, 0.8, 1.0, star_alpha),
        );
    }
}

/// 绘制超速模式效果（速度线 + 红色光晕）
pub fn draw_overdrive_effect(pos: Vec2, vel: Vec2, time: f32) {
    let speed = vel.length();
    if speed < 50.0 {
        return;
    }

    let dir = vel.normalize();
    let pulse = 0.7 + 0.3 * (time * 8.0).sin();

    // 速度线
    let line_count = 4;
    for i in 0..line_count {
        let offset = (i as f32 - line_count as f32 / 2.0) * 8.0;
        let perp = Vec2::new(-dir.y, dir.x);
        let start = pos - dir * 20.0 + perp * offset;
        let end = start - dir * (30.0 + speed * 0.1) * pulse;

        let alpha = 0.3 * pulse * (1.0 - (i as f32 / line_count as f32).abs());
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            2.0,
            Color::new(1.0, 0.3, 0.2, alpha),
        );
    }

    // 红色光晕
    draw_circle(
        pos.x,
        pos.y,
        SHIP_HEIGHT * 0.6,
        Color::new(1.0, 0.2, 0.1, 0.15 * pulse),
    );
}
