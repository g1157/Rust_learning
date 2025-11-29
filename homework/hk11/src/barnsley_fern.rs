// Barnsley 蕨类 (Barnsley Fern) 实现
// 使用迭代函数系统 (IFS) 生成类似真实蕨类植物的分形图形

use crate::common::{create_image, draw_pixel};
use image::Rgb;
use rand::Rng;

/// IFS 仿射变换：f(x,y) = (ax + by + e, cx + dy + f)
struct AffineTransform {
    a: f64,
    b: f64,
    c: f64,
    d: f64, // 变换矩阵
    e: f64,
    f: f64,    // 平移向量
    prob: f64, // 选择概率
}

impl AffineTransform {
    fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, prob: f64) -> Self {
        Self {
            a,
            b,
            c,
            d,
            e,
            f,
            prob,
        }
    }

    /// 应用变换
    fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.b * y + self.e,
            self.c * x + self.d * y + self.f,
        )
    }
}

/// Barnsley 蕨类的四个 IFS 变换
fn get_fern_transforms() -> Vec<AffineTransform> {
    vec![
        // f1: 茎干（概率 1%）
        AffineTransform::new(0.0, 0.0, 0.0, 0.16, 0.0, 0.0, 0.01),
        // f2: 主要叶片（概率 85%）- 产生大部分蕨类形状
        AffineTransform::new(0.85, 0.04, -0.04, 0.85, 0.0, 1.6, 0.85),
        // f3: 左侧小叶（概率 7%）
        AffineTransform::new(0.2, -0.26, 0.23, 0.22, 0.0, 1.6, 0.07),
        // f4: 右侧小叶（概率 7%）
        AffineTransform::new(-0.15, 0.28, 0.26, 0.24, 0.0, 0.44, 0.07),
    ]
}

/// 根据概率随机选择一个变换
fn select_transform<'a>(
    transforms: &'a [AffineTransform],
    rng: &mut impl Rng,
) -> &'a AffineTransform {
    let r: f64 = rng.r#gen();
    let mut cumulative = 0.0;

    for t in transforms {
        cumulative += t.prob;
        if r < cumulative {
            return t;
        }
    }

    // 默认返回最后一个
    transforms.last().unwrap()
}

/// 根据 y 坐标返回渐变颜色（从深绿到浅绿）
fn get_fern_color(y: f64, y_max: f64) -> Rgb<u8> {
    let t = (y / y_max).clamp(0.0, 1.0);

    // 底部深绿，顶部浅绿
    let r = (20.0 + 40.0 * t) as u8;
    let g = (80.0 + 100.0 * t) as u8;
    let b = (20.0 + 30.0 * t) as u8;

    Rgb([r, g, b])
}

/// 生成 Barnsley 蕨类图像并保存
pub fn render(width: u32, height: u32, output_path: &str) {
    let mut img = create_image(width, height, Rgb([255, 255, 255]));
    let transforms = get_fern_transforms();
    let mut rng = rand::thread_rng();

    // 迭代次数：越多越密集
    let iterations = 500_000;

    // 初始点
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;

    // 蕨类坐标范围（数学空间）
    // x: [-2.5, 2.5], y: [0, 10]
    let x_min = -2.5;
    let x_max = 2.5;
    let y_min = 0.0;
    let y_max = 10.0;

    // 跳过前几次迭代（让系统稳定）
    for _ in 0..20 {
        let t = select_transform(&transforms, &mut rng);
        (x, y) = t.apply(x, y);
    }

    // 主迭代
    for _ in 0..iterations {
        let t = select_transform(&transforms, &mut rng);
        (x, y) = t.apply(x, y);

        // 将数学坐标映射到图像坐标
        let px = ((x - x_min) / (x_max - x_min)) * (width as f64 - 1.0);
        let py = ((y_max - y) / (y_max - y_min)) * (height as f64 - 1.0); // y 翻转

        let color = get_fern_color(y, y_max);
        draw_pixel(&mut img, px, py, color);
    }

    img.save(output_path).expect("保存 Barnsley 蕨类图片失败");
    println!("Barnsley 蕨类已生成: {}", output_path);
}

// ============================================================================
// Barnsley 蕨类算法说明
// ============================================================================
//
// 迭代函数系统 (IFS) 原理：
//
// 使用 4 个仿射变换，每次迭代按概率选择一个：
//
// | 变换 | 概率 | 作用 |
// |------|------|------|
// | f1   | 1%   | 茎干 |
// | f2   | 85%  | 主叶片（自相似缩放+旋转）|
// | f3   | 7%   | 左侧小叶 |
// | f4   | 7%   | 右侧小叶 |
//
// 仿射变换公式：
//   x' = a*x + b*y + e
//   y' = c*x + d*y + f
//
// f2 是核心：它产生 85% 的点，通过轻微旋转和缩放
// 创造出蕨类的自相似结构。
