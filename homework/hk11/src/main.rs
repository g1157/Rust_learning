// 作业十一：分形图形与扩散模拟
//
// 作业一：几何/递归类分形图形（分形树、Barnsley 蕨类）
// 作业二：奶滴扩散随机漫步模拟

mod barnsley_fern;
mod common;
mod diffusion;
mod fractal_tree;

fn main() {
    // ===== 作业一：分形图形 =====
    println!("=== 作业一：几何/递归类分形图形 ===\n");

    let width = 800;
    let height = 600;

    // 生成分形树
    fractal_tree::render(width, height, "fractal_tree.png");

    // 生成 Barnsley 蕨类
    barnsley_fern::render(width, height, "barnsley_fern.png");

    // ===== 作业二：扩散模拟 =====
    diffusion::run_simulation();
}

// ============================================================================
// 项目结构说明
// ============================================================================
//
// 模块：
//   common.rs        - 公共组件（Point, Line, 绘图函数）
//   fractal_tree.rs  - 分形树（递归算法）
//   barnsley_fern.rs - Barnsley 蕨类（IFS 迭代算法）
//   diffusion.rs     - 奶滴扩散模拟（随机漫步）
//
// 运行命令: cargo run
//
// 输出文件:
//   fractal_tree.png    - 分形树
//   barnsley_fern.png   - Barnsley 蕨类
//   diffusion_t0.png    - 扩散初始状态
//   diffusion_t1e4.png  - t=10,000 时刻
//   diffusion_t1e5.png  - t=100,000 时刻
//   diffusion_t5e5.png  - t=500,000 时刻
