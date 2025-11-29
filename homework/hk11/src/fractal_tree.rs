// 分形树 (Fractal Tree) 实现
// 使用递归算法生成几何分形图形


// **为什么用 `#[derive(Copy, Clone, Debug)]`？**
// - `Copy`: 点和线段是小型数据，可以直接复制而不是移动
// - `Clone`: 允许显式克隆
// - `Debug`: 方便调试打印
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

/// 分形树的配置参数
pub struct TreeConfig {
    pub max_depth: u32,       // 最大递归深度
    pub branch_angle: f64,    // 分支角度（弧度）
    pub length_ratio: f64,    // 每层长度缩短比例
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 20,
            branch_angle: std::f64::consts::PI / 6.0,  // 30度
            length_ratio: 0.7,
        }
    }
}

/// 递归生成分形树的所有线段
/// 
/// # 参数
/// - `start`: 当前分支的起点
/// - `length`: 当前分支的长度
/// - `angle`: 当前分支的角度（相对于 y 轴正方向，逆时针为正）
/// - `depth`: 当前递归深度
/// - `config`: 分形树配置
/// - `lines`: 存储生成的所有线段
pub fn generate_tree(
    start: Point,
    length: f64,
    angle: f64,
    depth: u32,
    config: &TreeConfig,
    lines: &mut Vec<Line>,
) {
    // 递归终止条件
    if depth >= config.max_depth {
        return;
    }

    // 计算当前分支的终点
    // angle 是相对于 y 轴正方向的角度
    // x 方向：sin(angle) * length
    // y 方向：cos(angle) * length（向上为正，但图像 y 轴向下，所以取负）
    let end = Point::new(
        start.x + length * angle.sin(),
        start.y - length * angle.cos(),  // 负号因为图像 y 轴向下
    );

    // 记录这条线段
    lines.push(Line::new(start, end));

    // 计算下一层的长度
    let new_length = length * config.length_ratio;

    // 递归生成左分支（角度减小）
    generate_tree(
        end,
        new_length,
        angle - config.branch_angle,
        depth + 1,
        config,
        lines,
    );

    // 递归生成右分支（角度增大）
    generate_tree(
        end,
        new_length,
        angle + config.branch_angle,
        depth + 1,
        config,
        lines,
    );
}

/// 在图像上画一条线段（使用 Bresenham 直线算法）
/// 
/// # 参数
/// - `img`: 图像缓冲区
/// - `line`: 要绘制的线段
/// - `color`: 线段颜色
fn draw_line(img: &mut RgbImage, line: &Line, color: Rgb<u8>) {
    // 获取图像尺寸
    let (width, height) = img.dimensions();

    // 将 f64 坐标转换为 i32（用于 Bresenham 算法）
    let mut x0 = line.start.x as i32;
    let mut y0 = line.start.y as i32;
    let x1 = line.end.x as i32;
    let y1 = line.end.y as i32;

    // 计算差值和步进方向
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    // Bresenham 直线算法
    loop {
        // 检查坐标是否在图像范围内，如果是则绘制像素
        if x0 >= 0 && x0 < width as i32 && y0 >= 0 && y0 < height as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }

        // 到达终点则退出
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

/// 根据递归深度返回渐变颜色（从棕色树干到绿色树叶）
fn get_branch_color(depth: u32, max_depth: u32) -> Rgb<u8> {
    // t 从 0.0（树干）到 1.0（树叶）
    let t = depth as f64 / max_depth as f64;
    
    // 树干颜色：棕色 (139, 69, 19)
    // 树叶颜色：绿色 (34, 139, 34)
    let r = (139.0 * (1.0 - t) + 34.0 * t) as u8;
    let g = (69.0 * (1.0 - t) + 139.0 * t) as u8;
    let b = (19.0 * (1.0 - t) + 34.0 * t) as u8;
    
    Rgb([r, g, b])
}

/// 生成分形树图像并保存
/// 
/// # 参数
/// - `width`: 图像宽度
/// - `height`: 图像高度
/// - `output_path`: 输出文件路径
pub fn render_tree(width: u32, height: u32, output_path: &str) {
    // 创建白色背景的图像
    let mut img = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
    
    // 分形树配置
    let config = TreeConfig::default();
    
    // 树的起点：图像底部中央
    let start = Point::new(width as f64 / 2.0, height as f64 - 20.0);
    
    // 初始树干长度（约为图像高度的 1/4）
    let initial_length = height as f64 / 4.0;
    
    // 为每一层深度分别生成并绘制线段（以便使用不同颜色）
    for depth in 0..config.max_depth {
        let mut lines = Vec::new();
        
        // 生成当前深度的所有线段
        generate_tree_at_depth(
            start,
            initial_length,
            0.0,  // 初始角度：向上
            0,
            depth,
            &config,
            &mut lines,
        );
        
        // 获取当前深度的颜色
        let color = get_branch_color(depth, config.max_depth);
        
        // 绘制所有线段
        for line in &lines {
            draw_line(&mut img, line, color);
        }
    }
    
    // 保存图片
    img.save(output_path).expect("保存分形树图片失败");
    println!("分形树已生成: {}", output_path);
}

/// 生成指定深度的线段（辅助函数，用于分层着色）
fn generate_tree_at_depth(
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

    let end = Point::new(
        start.x + length * angle.sin(),
        start.y - length * angle.cos(),
    );

    // 只在目标深度时记录线段
    if current_depth == target_depth {
        lines.push(Line::new(start, end));
    }

    let new_length = length * config.length_ratio;

    generate_tree_at_depth(
        end,
        new_length,
        angle - config.branch_angle,
        current_depth + 1,
        target_depth,
        config,
        lines,
    );

    generate_tree_at_depth(
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
// 代码解释笔记
// ============================================================================
//
// ## TreeConfig - 配置参数
//
// ```
// pub struct TreeConfig {
//     pub max_depth: u32,       // 最大递归深度（12 层 = 2^12 = 4096 个末端分支）
//     pub branch_angle: f64,    // 分支角度：30度 (π/6 弧度)
//     pub length_ratio: f64,    // 每层缩短到 0.7 倍
// }
// ```
//
// ## generate_tree - 递归核心算法流程
//
// ```
//            起点 (start)
//             │
//             │ length（当前分支长度）
//             │
//            终点 (end)
//           ╱    ╲
//       左分支   右分支
//      angle-30° angle+30°
//      长度×0.7  长度×0.7
// ```
//
// ## 关键数学公式
//
// 从起点计算终点位置：
//   end.x = start.x + length * sin(angle)  // 水平偏移
//   end.y = start.y - length * cos(angle)  // 垂直偏移（负号因为图像y轴向下）
//
// ## 递归调用说明
//
// - 左分支：角度 - 30°
// - 右分支：角度 + 30°
// - 长度缩短为 0.7 倍
// - 深度 + 1，直到达到 max_depth 终止
//
// ============================================================================
// Step 3: 绘图函数解释
// ============================================================================
//
// ## draw_line - Bresenham 直线算法
//
// Bresenham 算法是经典的光栅化直线算法，只用整数运算，效率高：
// 1. 计算起点到终点的 dx, dy
// 2. 每步根据误差值决定移动方向
// 3. 逐像素绘制
//
// ## get_branch_color - 渐变着色
//
// 根据深度做线性插值：
// - depth=0（树干）: 棕色 (139, 69, 19)
// - depth=max（树叶）: 绿色 (34, 139, 34)
//
// ## render_tree - 主渲染函数流程
//
// 1. 创建白色背景图像
// 2. 树根在底部中央
// 3. 按深度分层绘制（每层不同颜色）
// 4. 保存图片
