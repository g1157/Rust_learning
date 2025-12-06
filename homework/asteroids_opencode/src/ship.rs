//! 飞船模块
//!
//! 定义玩家飞船的物理属性和渲染。
//!
//! ## 功能
//! - 基于动量的物理模拟
//! - 旋转和推进
//! - 速度阻尼
//! - 三角形顶点计算（用于碰撞检测和渲染）
//! - 相位闪现瞬移支持

use macroquad::prelude::*;

use crate::utils::wrap_around;

pub const SHIP_HEIGHT: f32 = 25.; // 像素
pub const SHIP_BASE: f32 = 20.; // 像素
pub const SHIP_THRUST: f32 = 800.0; // 像素/秒^2
pub const SHIP_ROTATION_STEP: f32 = 450.0; // 度/秒
pub const SHIP_MAX_SPEED: f32 = 1800.0; // 像素/秒
pub const SHIP_DAMPING: f32 = 2.0; // 阻尼系数

/// 玩家飞船的物理状态
pub struct Ship {
    pub pos: Vec2,
    pub rot: f32,
    pub vel: Vec2,
}

impl Ship {
    /// 在指定位置创建新飞船
    pub fn new(center: Vec2) -> Self {
        Self {
            pos: center,
            rot: 0.,
            vel: Vec2::new(0., 0.),
        }
    }

    /// 重置飞船到指定位置（用于重生）
    pub fn reset(position: Vec2) -> Self {
        Self::new(position)
    }

    /// 获取飞船旋转角度的弧度值
    pub fn rotation_radians(&self) -> f32 {
        self.rot.to_radians()
    }

    /// 获取飞船朝向的单位向量
    pub fn forward_vector(&self) -> Vec2 {
        let rot = self.rotation_radians();
        Vec2::new(rot.sin(), -rot.cos())
    }

    /// 计算飞船三角形的三个顶点位置（用于绘制和碰撞检测）
    pub fn triangle_vertices(&self) -> (Vec2, Vec2, Vec2) {
        let rotation = self.rotation_radians();
        let v1 = Vec2::new(
            self.pos.x + rotation.sin() * SHIP_HEIGHT / 2.,
            self.pos.y - rotation.cos() * SHIP_HEIGHT / 2.,
        );
        let v2 = Vec2::new(
            self.pos.x - rotation.cos() * SHIP_BASE / 2. - rotation.sin() * SHIP_HEIGHT / 2.,
            self.pos.y - rotation.sin() * SHIP_BASE / 2. + rotation.cos() * SHIP_HEIGHT / 2.,
        );
        let v3 = Vec2::new(
            self.pos.x + rotation.cos() * SHIP_BASE / 2. - rotation.sin() * SHIP_HEIGHT / 2.,
            self.pos.y + rotation.sin() * SHIP_BASE / 2. + rotation.cos() * SHIP_HEIGHT / 2.,
        );
        (v1, v2, v3)
    }

    /// 计算相位闪现目标位置
    ///
    /// 沿飞船朝向瞬移指定距离，并应用边界环绕
    #[allow(dead_code)] // 用于相位闪现功能
    pub fn phase_destination(&self, distance: f32) -> Vec2 {
        let dest = self.pos + self.forward_vector() * distance;
        wrap_around(&dest)
    }

    /// 瞬移到指定位置并清除速度
    ///
    /// 用于相位闪现等瞬移技能，避免继承旧速度
    #[allow(dead_code)] // 用于相位闪现功能
    pub fn teleport_to(&mut self, position: Vec2) {
        self.pos = position;
        self.vel = Vec2::ZERO;
    }
}
