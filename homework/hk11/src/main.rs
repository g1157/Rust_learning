// 作业十一：分形图形与扩散模拟
//
// 作业一：几何/递归类分形图形
// 作业二：奶滴扩散随机漫步模拟

mod barnsley_fern;
mod common;
mod diffusion;
mod dragon_curve;
mod fractal_plants;
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

    // 生成龙形曲线
    dragon_curve::draw_dragon_curve("dragon_curve.png");

    // 生成列维C曲线
    fractal_plants::draw_levy_c_curve("levy_c_curve.png");

    // 生成分形植物 (L-System)
    fractal_plants::draw_fractal_plants("fractal_plants.png");

    // 生成分形树变体（多分支/随机/风吹效果）
    fractal_plants::draw_fractal_tree_variants("fractal_tree_variants.png");

    // ===== 作业二：扩散模拟 =====
    diffusion::run_simulation();
}

// ============================================================================
// 项目结构说明
// ============================================================================
//
// 模块：
//   common.rs          - 公共组件（Point, Line, 绘图函数）
//   fractal_tree.rs    - 分形树（递归算法）
//   barnsley_fern.rs   - Barnsley 蕨类（IFS 迭代算法）
//   dragon_curve.rs    - 龙形曲线（折纸分形）
//   fractal_plants.rs  - 列维C曲线、L-System 植物、分形树变体
//   diffusion.rs       - 奶滴扩散模拟（随机漫步）
//
// 运行命令: cargo run --release
//
// 输出文件:
//   fractal_tree.png           - 分形树
//   barnsley_fern.png          - Barnsley 蕨类
//   dragon_curve.png           - 龙形曲线
//   levy_c_curve.png           - 列维C曲线
//   fractal_plants.png         - L-System 分形植物
//   fractal_tree_variants.png  - 分形树变体
//   diffusion_*.png            - 扩散模拟
//   diffusion_lnN_vs_t.png     - 衰减拟合图
