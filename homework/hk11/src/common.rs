// 公共模块：数据结构和绘图工具
// 供所有分形算法共享使用

use image::{Rgb, RgbImage};

/// 二维平面上的点
#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// 线段：由起点和终点构成
#[derive(Copy, Clone, Debug)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}

/// 在图像上画一条线段（Bresenham 直线算法）
pub fn draw_line(img: &mut RgbImage, line: &Line, color: Rgb<u8>) {
    let (width, height) = img.dimensions();

    let mut x0 = line.start.x as i32;
    let mut y0 = line.start.y as i32;
    let x1 = line.end.x as i32;
    let y1 = line.end.y as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < width as i32 && y0 >= 0 && y0 < height as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// 在图像上画一个像素点（带边界检查）
pub fn draw_pixel(img: &mut RgbImage, x: f64, y: f64, color: Rgb<u8>) {
    let (width, height) = img.dimensions();
    let px = x as i32;
    let py = y as i32;

    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
        img.put_pixel(px as u32, py as u32, color);
    }
}

/// 颜色线性插值
pub fn lerp_color(c1: Rgb<u8>, c2: Rgb<u8>, t: f64) -> Rgb<u8> {
    let t = t.clamp(0.0, 1.0);
    Rgb([
        (c1[0] as f64 * (1.0 - t) + c2[0] as f64 * t) as u8,
        (c1[1] as f64 * (1.0 - t) + c2[1] as f64 * t) as u8,
        (c1[2] as f64 * (1.0 - t) + c2[2] as f64 * t) as u8,
    ])
}

/// 创建指定尺寸和背景色的图像
pub fn create_image(width: u32, height: u32, bg_color: Rgb<u8>) -> RgbImage {
    RgbImage::from_pixel(width, height, bg_color)
}

// ============================================================================
// common.rs 解释
// ============================================================================
//
// 抽取的公共组件：
//
// 1. Point, Line - 基础几何结构
// 2. draw_line   - Bresenham 画线算法
// 3. draw_pixel  - 单像素绘制（用于 IFS 类分形）
// 4. lerp_color  - 颜色插值（用于渐变效果）
// 5. create_image - 创建图像缓冲区
