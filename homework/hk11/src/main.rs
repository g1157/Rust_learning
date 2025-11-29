// 作业一：几何/递归类分形图形 - 分形树 (Fractal Tree)

mod fractal_tree;

fn main() {
    // 图像尺寸
    let width = 800;
    let height = 600;
    
    // 输出文件路径
    let output_path = "fractal_tree.png";
    
    // 生成分形树
    fractal_tree::render_tree(width, height, output_path);
}

// ============================================================================
// Step 4: main.rs 解释
// ============================================================================
//
// main.rs 非常简洁，只做三件事：
// 1. 声明模块: mod fractal_tree
// 2. 设置参数: 图像尺寸 800x600
// 3. 调用渲染: fractal_tree::render_tree()
//
// 运行命令: cargo run
// 输出文件: fractal_tree.png
