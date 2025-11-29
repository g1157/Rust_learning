// 龙形曲线 (Dragon Curve)
// 又称 Heighway Dragon，由折纸展开形成的分形曲线

use crate::common::{Line, Point, create_image, draw_line};
use image::Rgb;

/// 生成龙形曲线的转向序列
/// 规则：每次迭代，在序列中间插入 R，然后翻转前半部分的L/R
fn generate_dragon_turns(iterations: u32) -> Vec<bool> {
    // true = 右转, false = 左转
    let mut turns = Vec::new();

    for _ in 0..iterations {
        // 在中间插入右转
        let mut new_turns = turns.clone();
        new_turns.push(true); // 右转

        // 翻转并反转前半部分，追加到后面
        for &turn in turns.iter().rev() {
            new_turns.push(!turn);
        }
        turns = new_turns;
    }

    turns
}

/// 绘制龙形曲线
pub fn draw_dragon_curve(output_path: &str) {
    let img_size = 800u32;
    let iterations = 15; // 迭代次数
    let step_length = 3.0; // 每步长度

    // 深蓝色背景
    let mut img = create_image(img_size, img_size, Rgb([15, 15, 35]));

    // 生成转向序列
    let turns = generate_dragon_turns(iterations);

    // 计算路径点
    let mut points = Vec::new();
    let mut x = img_size as f64 * 0.3;
    let mut y = img_size as f64 * 0.5;
    let mut angle = 0.0_f64; // 初始方向：向右

    points.push(Point::new(x, y));

    for &turn_right in &turns {
        // 移动一步
        x += angle.cos() * step_length;
        y += angle.sin() * step_length;
        points.push(Point::new(x, y));

        // 转向
        if turn_right {
            angle += std::f64::consts::FRAC_PI_2; // 右转 90°
        } else {
            angle -= std::f64::consts::FRAC_PI_2; // 左转 90°
        }
    }
    // 最后一步
    x += angle.cos() * step_length;
    y += angle.sin() * step_length;
    points.push(Point::new(x, y));

    // 绘制曲线，使用渐变色
    let total = points.len() - 1;
    for i in 0..total {
        let t = i as f64 / total as f64;

        // 彩虹渐变色
        let r = ((t * 2.0 * std::f64::consts::PI).sin() * 127.0 + 128.0) as u8;
        let g = ((t * 2.0 * std::f64::consts::PI + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((t * 2.0 * std::f64::consts::PI + 4.0).sin() * 127.0 + 128.0) as u8;

        let line = Line::new(points[i], points[i + 1]);
        draw_line(&mut img, &line, Rgb([r, g, b]));
    }

    img.save(output_path).expect("保存龙形曲线失败");
    println!("龙形曲线已生成: {}", output_path);
}
