// 分形树 (Fractal Tree) 实现
// 使用递归算法生成几何分形图形

use crate::common::{Line, Point, create_image, draw_line, lerp_color};
use image::Rgb;

/// 分形树的配置参数
pub struct TreeConfig {
    pub max_depth: u32,    // 最大递归深度
    pub branch_angle: f64, // 分支角度（弧度）
    pub length_ratio: f64, // 每层长度缩短比例
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 14,
            branch_angle: std::f64::consts::PI / 6.0, // 30度
            length_ratio: 0.7,
        }
    }
}

/// 根据递归深度返回渐变颜色（从棕色树干到绿色树叶）
fn get_branch_color(depth: u32, max_depth: u32) -> Rgb<u8> {
    let t = depth as f64 / max_depth as f64;
    lerp_color(
        Rgb([139, 69, 19]), // 树干：棕色
        Rgb([34, 139, 34]), // 树叶：绿色
        t,
    )
}

/// 生成分形树图像并保存
pub fn render(width: u32, height: u32, output_path: &str) {
    let mut img = create_image(width, height, Rgb([255, 255, 255]));
    let config = TreeConfig::default();

    // 树的起点：图像底部中央
    let start = Point::new(width as f64 / 2.0, height as f64 - 20.0);
    let initial_length = height as f64 / 4.0;

    // 按深度分层绘制（每层不同颜色）
    for depth in 0..config.max_depth {
        let mut lines = Vec::new();
        generate_at_depth(start, initial_length, 0.0, 0, depth, &config, &mut lines);

        let color = get_branch_color(depth, config.max_depth);
        for line in &lines {
            draw_line(&mut img, line, color);
        }
    }

    img.save(output_path).expect("保存分形树图片失败");
    println!("分形树已生成: {}", output_path);
}

/// 生成指定深度的线段（辅助函数，用于分层着色）
fn generate_at_depth(
    start: Point,
    length: f64,
    angle: f64,
    current_depth: u32,
    target_depth: u32,
    config: &TreeConfig,
    lines: &mut Vec<Line>,
) {
    if current_depth > target_depth || current_depth >= config.max_depth {
        return;
    }

    // 计算终点：angle 相对于 y 轴正方向
    let end = Point::new(
        start.x + length * angle.sin(),
        start.y - length * angle.cos(), // 负号因为图像 y 轴向下
    );

    // 只在目标深度时记录线段
    if current_depth == target_depth {
        lines.push(Line::new(start, end));
    }

    let new_length = length * config.length_ratio;

    // 左分支
    generate_at_depth(
        end,
        new_length,
        angle - config.branch_angle,
        current_depth + 1,
        target_depth,
        config,
        lines,
    );

    // 右分支
    generate_at_depth(
        end,
        new_length,
        angle + config.branch_angle,
        current_depth + 1,
        target_depth,
        config,
        lines,
    );
}

// ============================================================================
// 分形树算法说明
// ============================================================================
//
// 递归结构：
//            起点 (start)
//             │
//             │ length
//             │
//            终点 (end)
//           ╱    ╲
//       左分支   右分支
//      angle-30° angle+30°
//      长度×0.7  长度×0.7
//
// 数学公式：
//   end.x = start.x + length * sin(angle)
//   end.y = start.y - length * cos(angle)
