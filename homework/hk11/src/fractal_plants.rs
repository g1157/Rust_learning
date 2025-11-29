// 列维C曲线 (Lévy C Curve) 和 L-System 分形植物
// 使用递归和 L-System 方法生成分形

use crate::common::{Line, Point, create_image, draw_line, lerp_color};
use image::Rgb;
use std::f64::consts::PI;

// ============================================================================
// 列维C曲线 (Lévy C Curve)
// ============================================================================

/// 递归绘制列维C曲线
fn levy_c_recursive(
    img: &mut image::RgbImage,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    depth: u32,
    max_depth: u32,
) {
    if depth == 0 {
        // 绘制线段，颜色根据深度变化
        let t = 1.0 - (depth as f64 / max_depth as f64);
        let color = lerp_color(Rgb([255, 100, 150]), Rgb([100, 200, 255]), t);
        let line = Line::new(Point::new(x1, y1), Point::new(x2, y2));
        draw_line(img, &line, color);
        return;
    }

    // 计算中点，向一侧偏移形成等腰直角三角形
    let dx = x2 - x1;
    let dy = y2 - y1;

    // 新的顶点：在中点的垂直方向偏移 len/2
    let mx = (x1 + x2) / 2.0 + (dy) / 2.0;
    let my = (y1 + y2) / 2.0 - (dx) / 2.0;

    // 递归绘制两段
    levy_c_recursive(img, x1, y1, mx, my, depth - 1, max_depth);
    levy_c_recursive(img, mx, my, x2, y2, depth - 1, max_depth);
}

/// 绘制列维C曲线
pub fn draw_levy_c_curve(output_path: &str) {
    let img_size = 800u32;
    let depth = 16; // 递归深度

    let mut img = create_image(img_size, img_size, Rgb([10, 10, 30]));

    // 起点和终点
    let x1 = img_size as f64 * 0.25;
    let y1 = img_size as f64 * 0.6;
    let x2 = img_size as f64 * 0.75;
    let y2 = img_size as f64 * 0.6;

    levy_c_recursive(&mut img, x1, y1, x2, y2, depth, depth);

    img.save(output_path).expect("保存列维C曲线失败");
    println!("列维C曲线已生成: {}", output_path);
}

// ============================================================================
// L-System 分形植物
// ============================================================================

/// L-System 规则
struct LSystem {
    axiom: String,
    rules: Vec<(char, String)>,
}

impl LSystem {
    fn new(axiom: &str, rules: Vec<(char, &str)>) -> Self {
        Self {
            axiom: axiom.to_string(),
            rules: rules.into_iter().map(|(c, s)| (c, s.to_string())).collect(),
        }
    }

    /// 迭代生成字符串
    fn generate(&self, iterations: u32) -> String {
        let mut current = self.axiom.clone();

        for _ in 0..iterations {
            let mut next = String::new();
            for ch in current.chars() {
                let replacement = self
                    .rules
                    .iter()
                    .find(|(c, _)| *c == ch)
                    .map(|(_, s)| s.as_str());
                match replacement {
                    Some(s) => next.push_str(s),
                    None => next.push(ch),
                }
            }
            current = next;
        }

        current
    }
}

/// 解释 L-System 字符串并绘制
fn draw_lsystem(
    img: &mut image::RgbImage,
    commands: &str,
    start_x: f64,
    start_y: f64,
    start_angle: f64,
    step_length: f64,
    turn_angle: f64,
    color_start: Rgb<u8>,
    color_end: Rgb<u8>,
) {
    let mut x = start_x;
    let mut y = start_y;
    let mut angle = start_angle;
    let mut stack: Vec<(f64, f64, f64)> = Vec::new();

    let total_f = commands.chars().filter(|&c| c == 'F' || c == 'G').count();
    let mut f_count = 0;

    for ch in commands.chars() {
        match ch {
            'F' | 'G' => {
                // 向前移动并画线
                let new_x = x + angle.cos() * step_length;
                let new_y = y - angle.sin() * step_length; // y轴向下

                let t = f_count as f64 / total_f.max(1) as f64;
                let color = lerp_color(color_start, color_end, t);

                let line = Line::new(Point::new(x, y), Point::new(new_x, new_y));
                draw_line(img, &line, color);

                x = new_x;
                y = new_y;
                f_count += 1;
            }
            'f' => {
                // 向前移动但不画线
                x += angle.cos() * step_length;
                y -= angle.sin() * step_length;
            }
            '+' => {
                // 左转
                angle += turn_angle;
            }
            '-' => {
                // 右转
                angle -= turn_angle;
            }
            '[' => {
                // 保存状态
                stack.push((x, y, angle));
            }
            ']' => {
                // 恢复状态
                if let Some((sx, sy, sa)) = stack.pop() {
                    x = sx;
                    y = sy;
                    angle = sa;
                }
            }
            _ => {}
        }
    }
}

/// 绘制分形植物（使用多种 L-System）
pub fn draw_fractal_plants(output_path: &str) {
    let img_width = 1200u32;
    let img_height = 800u32;

    let mut img = create_image(img_width, img_height, Rgb([240, 248, 255]));

    // 植物1：经典分形草
    // F -> F[+F]F[-F]F
    let plant1 = LSystem::new("F", vec![('F', "F[+F]F[-F]F")]);
    let commands1 = plant1.generate(5);
    draw_lsystem(
        &mut img,
        &commands1,
        150.0,
        img_height as f64 - 50.0,
        PI / 2.0, // 向上
        3.0,      // 步长
        25.7_f64.to_radians(),
        Rgb([34, 139, 34]),   // 深绿
        Rgb([144, 238, 144]), // 浅绿
    );

    // 植物2：灌木
    // F -> FF-[-F+F+F]+[+F-F-F]
    let plant2 = LSystem::new("F", vec![('F', "FF-[-F+F+F]+[+F-F-F]")]);
    let commands2 = plant2.generate(4);
    draw_lsystem(
        &mut img,
        &commands2,
        400.0,
        img_height as f64 - 50.0,
        PI / 2.0,
        4.0,
        22.5_f64.to_radians(),
        Rgb([139, 69, 19]), // 棕色
        Rgb([50, 205, 50]), // 绿色
    );

    // 植物3：蕨类植物
    // X -> F+[[X]-X]-F[-FX]+X
    // F -> FF
    let plant3 = LSystem::new("X", vec![('X', "F+[[X]-X]-F[-FX]+X"), ('F', "FF")]);
    let commands3 = plant3.generate(6);
    draw_lsystem(
        &mut img,
        &commands3,
        700.0,
        img_height as f64 - 50.0,
        PI / 2.0,
        2.0,
        25.0_f64.to_radians(),
        Rgb([0, 100, 0]),   // 深绿
        Rgb([124, 252, 0]), // 草绿
    );

    // 植物4：花朵状
    // F -> F[+F]F[-F][F]
    let plant4 = LSystem::new("F", vec![('F', "F[+F]F[-F][F]")]);
    let commands4 = plant4.generate(5);
    draw_lsystem(
        &mut img,
        &commands4,
        1000.0,
        img_height as f64 - 50.0,
        PI / 2.0,
        4.0,
        20.0_f64.to_radians(),
        Rgb([75, 0, 130]),    // 靛蓝
        Rgb([255, 105, 180]), // 粉红
    );

    img.save(output_path).expect("保存分形植物失败");
    println!("分形植物已生成: {}", output_path);
}

// ============================================================================
// 分形树变体
// ============================================================================

use rand::Rng;

/// 绘制多分支随机分形树
fn draw_random_tree(
    img: &mut image::RgbImage,
    x: f64,
    y: f64,
    angle: f64,
    length: f64,
    depth: u32,
    max_depth: u32,
    rng: &mut impl Rng,
    wind_factor: f64, // 风力因子，正值向右吹
) {
    if depth == 0 || length < 2.0 {
        return;
    }

    // 计算终点
    let end_x = x + angle.cos() * length;
    let end_y = y - angle.sin() * length;

    // 根据深度计算颜色和线宽
    let t = depth as f64 / max_depth as f64;
    let color = if t > 0.3 {
        // 树干：棕色
        lerp_color(Rgb([101, 67, 33]), Rgb([139, 90, 43]), 1.0 - t)
    } else {
        // 树叶：绿色
        lerp_color(Rgb([34, 139, 34]), Rgb([144, 238, 144]), rng.r#gen::<f64>())
    };

    // 绘制分支（模拟线宽）
    let line = Line::new(Point::new(x, y), Point::new(end_x, end_y));
    draw_line(img, &line, color);

    // 额外的线来模拟粗线
    if depth > max_depth / 2 {
        for dx in -1..=1 {
            for dy in -1..=1 {
                let line = Line::new(
                    Point::new(x + dx as f64, y + dy as f64),
                    Point::new(end_x + dx as f64, end_y + dy as f64),
                );
                draw_line(img, &line, color);
            }
        }
    }

    // 随机生成 2-4 个分支
    let num_branches = rng.gen_range(2..=4);
    let base_angle_spread = 0.4 + rng.r#gen::<f64>() * 0.3; // 基础角度范围

    for i in 0..num_branches {
        // 计算分支角度
        let branch_offset =
            (i as f64 - (num_branches - 1) as f64 / 2.0) * base_angle_spread / num_branches as f64;
        let random_offset = (rng.r#gen::<f64>() - 0.5) * 0.2;
        let wind_offset = wind_factor * (1.0 - t); // 风力对细枝影响更大

        let new_angle = angle + branch_offset + random_offset + wind_offset;

        // 随机长度缩减
        let length_factor = 0.6 + rng.r#gen::<f64>() * 0.2;
        let new_length = length * length_factor;

        draw_random_tree(
            img,
            end_x,
            end_y,
            new_angle,
            new_length,
            depth - 1,
            max_depth,
            rng,
            wind_factor,
        );
    }
}

/// 绘制多棵不同风格的分形树
pub fn draw_fractal_tree_variants(output_path: &str) {
    let img_width = 1400u32;
    let img_height = 800u32;

    // 渐变天空背景
    let mut img = create_image(img_width, img_height, Rgb([135, 206, 235]));

    // 绘制渐变天空
    for y in 0..img_height {
        let t = y as f64 / img_height as f64;
        let sky_color = lerp_color(
            Rgb([135, 206, 250]), // 浅蓝
            Rgb([255, 218, 185]), // 桃色（地平线）
            t,
        );
        for x in 0..img_width {
            img.put_pixel(x, y, sky_color);
        }
    }

    // 绘制地面
    for y in (img_height - 80)..img_height {
        for x in 0..img_width {
            img.put_pixel(x, y, Rgb([34, 139, 34]));
        }
    }

    let mut rng = rand::thread_rng();
    let ground_y = img_height as f64 - 80.0;

    // 树1：正常树
    draw_random_tree(
        &mut img,
        200.0,
        ground_y,
        PI / 2.0,
        120.0,
        10,
        10,
        &mut rng,
        0.0, // 无风
    );

    // 树2：向右微风
    draw_random_tree(
        &mut img,
        500.0,
        ground_y,
        PI / 2.0,
        100.0,
        9,
        9,
        &mut rng,
        0.15, // 微风向右
    );

    // 树3：强风
    draw_random_tree(
        &mut img,
        800.0,
        ground_y,
        PI / 2.0 + 0.1, // 初始就有些倾斜
        110.0,
        9,
        9,
        &mut rng,
        0.3, // 强风
    );

    // 树4：小树
    draw_random_tree(
        &mut img,
        1050.0,
        ground_y,
        PI / 2.0,
        70.0,
        7,
        7,
        &mut rng,
        -0.1, // 向左微风
    );

    // 树5：大树
    draw_random_tree(
        &mut img,
        1250.0,
        ground_y,
        PI / 2.0,
        130.0,
        11,
        11,
        &mut rng,
        0.05,
    );

    img.save(output_path).expect("保存分形树变体失败");
    println!("分形树变体已生成: {}", output_path);
}
