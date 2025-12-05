//! 伊辛模型临界指数计算程序
//!
//! 本程序使用蒙特卡洛方法模拟二维伊辛模型，计算磁化强度M并估算临界指数β。
//! 支持正方形晶格和三角形晶格两种结构。
//!
//! # 运行方式
//! ```bash
//! # 正方形晶格，默认参数
//! cargo run --release --bin ising_critical
//!
//! # 三角形晶格，自定义参数
//! cargo run --release --bin ising_critical -- --lattice triangular --size 32 --sweeps 5000
//!
//! # 禁用可视化
//! cargo run --release --bin ising_critical -- --no-plot
//! ```

use std::error::Error;
use std::f64;
use std::fs::{create_dir_all, File};
use std::io::Write;

use clap::Parser;
use plotters::prelude::*;
use rand::prelude::*;

// ============================================================================
// 命令行参数定义
// ============================================================================

/// 伊辛模型临界指数计算程序
#[derive(Parser, Debug)]
#[command(author, version, about = "计算伊辛模型磁化强度与临界指数β")]
struct Args {
    /// 晶格类型: square (正方形) 或 triangular (三角形)
    #[arg(long, default_value = "square")]
    lattice: String,

    /// 晶格尺寸 L (总自旋数 = L × L)
    #[arg(long, short = 'L', default_value = "32")]
    size: usize,

    /// 每个温度点的蒙特卡洛扫描次数
    #[arg(long, default_value = "3000")]
    sweeps: usize,

    /// 平衡化（热化）扫描次数
    #[arg(long, default_value = "1000")]
    equilibration: usize,

    /// 随机数种子
    #[arg(long, default_value = "42")]
    seed: u64,

    /// 临界温度 Tc 估计值 (用于log-log分析)
    #[arg(long)]
    tc: Option<f64>,

    /// 输出CSV文件路径
    #[arg(long, default_value = "ising_critical_results.csv")]
    output: String,

    /// 禁用可视化输出
    #[arg(long, default_value = "false")]
    no_plot: bool,
}

// ============================================================================
// 晶格类型定义
// ============================================================================

/// 晶格类型枚举
#[derive(Clone, Copy, Debug, PartialEq)]
enum LatticeType {
    /// 正方形晶格 (配位数 z=4)
    Square,
    /// 三角形晶格 (配位数 z=6)
    Triangular,
}

impl LatticeType {
    /// 从字符串解析晶格类型
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "triangular" | "tri" | "t" => LatticeType::Triangular,
            _ => LatticeType::Square,
        }
    }

    /// 获取理论临界温度 (J=kB=1)
    fn critical_temperature(self) -> f64 {
        match self {
            // Tc = 2 / ln(1 + √2) ≈ 2.269
            LatticeType::Square => 2.0 / (1.0 + f64::consts::SQRT_2).ln(),
            // Tc = 4 / ln(3) ≈ 3.641
            LatticeType::Triangular => 4.0 / 3.0_f64.ln(),
        }
    }

    /// 获取配位数（最近邻数目）
    fn coordination_number(self) -> usize {
        match self {
            LatticeType::Square => 4,
            LatticeType::Triangular => 6,
        }
    }
}

// ============================================================================
// 晶格结构
// ============================================================================

/// 二维伊辛晶格
struct IsingLattice {
    /// 晶格线性尺寸
    size: usize,
    /// 自旋数组 (+1 或 -1)
    spins: Vec<i8>,
    /// 每个格点的近邻索引列表
    neighbors: Vec<Vec<usize>>,
    /// 晶格类型
    lattice_type: LatticeType,
}

impl IsingLattice {
    /// 创建新晶格，随机初始化自旋
    fn new(size: usize, lattice_type: LatticeType, rng: &mut StdRng) -> Self {
        let n = size * size;

        // 随机初始化自旋
        let spins: Vec<i8> = (0..n)
            .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
            .collect();

        // 构建近邻列表
        let neighbors = Self::build_neighbors(size, lattice_type);

        Self {
            size,
            spins,
            neighbors,
            lattice_type,
        }
    }

    /// 构建周期性边界条件下的近邻列表
    /// 
    /// ## 正方形晶格
    /// 每个格点有4个近邻：上、下、左、右
    /// 
    /// ## 三角形晶格
    /// 采用偏移行（skewed row）表示法：
    /// - 偶数行：上、下、左、右 + 右上(row-1, col+1)、左下(row+1, col-1)
    /// - 奇数行：上、下、左、右 + 左上(row-1, col-1)、右下(row+1, col+1)
    /// 
    /// 这样每个格点有6个近邻，形成三角形拓扑。
    fn build_neighbors(size: usize, lattice_type: LatticeType) -> Vec<Vec<usize>> {
        let n = size * size;
        let mut neighbors = vec![Vec::new(); n];

        for row in 0..size {
            for col in 0..size {
                let idx = row * size + col;

                // 周期性边界条件下的近邻坐标
                let up = ((row + size - 1) % size) * size + col;
                let down = ((row + 1) % size) * size + col;
                let left = row * size + (col + size - 1) % size;
                let right = row * size + (col + 1) % size;

                match lattice_type {
                    LatticeType::Square => {
                        // 正方形晶格：上下左右 4 个近邻
                        neighbors[idx] = vec![up, down, left, right];
                    }
                    LatticeType::Triangular => {
                        // 三角形晶格：6 个近邻
                        // 使用交替的对角方向，形成三角形拓扑
                        // 
                        // 偶数行的格点连接：右上(row-1, col+1) 和 左下(row+1, col-1)
                        // 奇数行的格点连接：左上(row-1, col-1) 和 右下(row+1, col+1)
                        // 这样确保整体形成三角形晶格
                        let (diag1, diag2) = if row % 2 == 0 {
                            // 偶数行：右上、左下
                            let up_right = ((row + size - 1) % size) * size + (col + 1) % size;
                            let down_left = ((row + 1) % size) * size + (col + size - 1) % size;
                            (up_right, down_left)
                        } else {
                            // 奇数行：左上、右下
                            let up_left = ((row + size - 1) % size) * size + (col + size - 1) % size;
                            let down_right = ((row + 1) % size) * size + (col + 1) % size;
                            (up_left, down_right)
                        };
                        neighbors[idx] = vec![up, down, left, right, diag1, diag2];
                    }
                }
            }
        }

        neighbors
    }

    /// 计算翻转指定自旋导致的能量变化
    /// ΔE = 2 * s_i * Σ_j s_j (其中 j 是 i 的近邻)
    fn energy_delta(&self, idx: usize) -> f64 {
        let spin = self.spins[idx] as f64;
        let neighbor_sum: f64 = self.neighbors[idx]
            .iter()
            .map(|&j| self.spins[j] as f64)
            .sum();
        2.0 * spin * neighbor_sum
    }

    /// 执行一次 Metropolis 扫描 (N 次单自旋翻转尝试)
    fn metropolis_sweep(&mut self, beta: f64, rng: &mut StdRng) {
        let n = self.spins.len();

        for _ in 0..n {
            // 随机选择一个自旋
            let idx = rng.gen_range(0..n);

            // 计算能量变化
            let delta_e = self.energy_delta(idx);

            // Metropolis 接受准则
            // 若 ΔE ≤ 0，始终接受；否则以概率 exp(-βΔE) 接受
            if delta_e <= 0.0 || rng.gen::<f64>() < (-beta * delta_e).exp() {
                self.spins[idx] = -self.spins[idx]; // 翻转自旋
            }
        }
    }

    /// 计算当前磁化强度 M = (1/N) Σ s_i
    fn magnetization(&self) -> f64 {
        let total: i32 = self.spins.iter().map(|&s| s as i32).sum();
        total as f64 / self.spins.len() as f64
    }
}

// ============================================================================
// 数据收集与分析
// ============================================================================

/// 单个温度点的测量结果
#[derive(Clone, Debug)]
struct MeasurementResult {
    /// 温度
    temperature: f64,
    /// 平均磁化强度 (绝对值)
    magnetization: f64,
    /// 磁化强度标准差
    magnetization_std: f64,
}

/// 在指定温度下进行模拟，返回平均磁化强度
fn simulate_at_temperature(
    size: usize,
    lattice_type: LatticeType,
    temperature: f64,
    equilibration_sweeps: usize,
    measurement_sweeps: usize,
    rng: &mut StdRng,
) -> MeasurementResult {
    let beta = 1.0 / temperature;
    let mut lattice = IsingLattice::new(size, lattice_type, rng);

    // 平衡化阶段：让系统达到热平衡
    for _ in 0..equilibration_sweeps {
        lattice.metropolis_sweep(beta, rng);
    }

    // 测量阶段：收集磁化强度样本
    let mut m_samples = Vec::with_capacity(measurement_sweeps);

    for _ in 0..measurement_sweeps {
        lattice.metropolis_sweep(beta, rng);
        // 取绝对值，因为低温下系统可能处于 +M 或 -M 态
        m_samples.push(lattice.magnetization().abs());
    }

    // 计算统计量
    let m_mean = m_samples.iter().sum::<f64>() / measurement_sweeps as f64;
    let m_var =
        m_samples.iter().map(|&m| (m - m_mean).powi(2)).sum::<f64>() / measurement_sweeps as f64;
    let m_std = m_var.sqrt();

    MeasurementResult {
        temperature,
        magnetization: m_mean,
        magnetization_std: m_std,
    }
}

/// 生成温度扫描列表
///
/// 根据题目建议，在 2.0 < T < Tc 区域内 β ≈ 1/8 的幂律较好成立。
/// 因此我们在此区域密集采样，同时包含一些低温点用于对比。
fn generate_temperature_range(lattice_type: LatticeType) -> Vec<f64> {
    let tc = lattice_type.critical_temperature();

    let mut temps = Vec::new();

    // 低温区域 (稀疏采样)
    let t_low_start = tc * 0.6;
    let t_low_end = tc * 0.85;
    for i in 0..5 {
        let t = t_low_start + (t_low_end - t_low_start) * (i as f64) / 4.0;
        temps.push(t);
    }

    // 临界区域附近密集采样 (2.0 < T < Tc 对于正方形晶格)
    // 使用相对于 Tc 的比例，对不同晶格适用
    let t_critical_start = tc * 0.88; // 约 2.0 (对于正方形 Tc≈2.27)
    let t_critical_end = tc * 0.995; // 非常接近 Tc

    for i in 0..20 {
        let t = t_critical_start + (t_critical_end - t_critical_start) * (i as f64) / 19.0;
        temps.push(t);
    }

    temps
}

// ============================================================================
// 临界指数估算
// ============================================================================

/// 线性回归结果
#[derive(Clone, Debug)]
struct LinearFit {
    slope: f64,
    intercept: f64,
    r_squared: f64,
}

/// 最小二乘线性回归
fn linear_regression(x_data: &[f64], y_data: &[f64]) -> Option<LinearFit> {
    if x_data.len() != y_data.len() || x_data.is_empty() {
        return None;
    }

    let n = x_data.len() as f64;
    let x_mean = x_data.iter().sum::<f64>() / n;
    let y_mean = y_data.iter().sum::<f64>() / n;

    let mut ss_xx = 0.0; // Σ(x - x̄)²
    let mut ss_yy = 0.0; // Σ(y - ȳ)²
    let mut ss_xy = 0.0; // Σ(x - x̄)(y - ȳ)

    for (&x, &y) in x_data.iter().zip(y_data.iter()) {
        let dx = x - x_mean;
        let dy = y - y_mean;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
        ss_xy += dx * dy;
    }

    if ss_xx < 1e-15 {
        return None;
    }

    let slope = ss_xy / ss_xx;
    let intercept = y_mean - slope * x_mean;
    let r_squared = if ss_yy > 1e-15 {
        (ss_xy * ss_xy) / (ss_xx * ss_yy)
    } else {
        0.0
    };

    Some(LinearFit {
        slope,
        intercept,
        r_squared,
    })
}

/// 方法一：通过 M^(1/β*) vs T 的线性度分析估算 β
/// 返回 (β*, R²) 列表
fn analyze_beta_star_linearity(
    results: &[MeasurementResult],
    beta_star_values: &[f64],
) -> Vec<(f64, f64)> {
    let mut scores = Vec::new();

    for &beta_star in beta_star_values {
        // 计算 M^(1/β*) 作为 y 值
        let x_data: Vec<f64> = results.iter().map(|r| r.temperature).collect();
        let y_data: Vec<f64> = results
            .iter()
            .map(|r| {
                if r.magnetization > 1e-10 {
                    r.magnetization.powf(1.0 / beta_star)
                } else {
                    0.0
                }
            })
            .collect();

        // 线性回归，R² 越高说明线性度越好
        let r2 = linear_regression(&x_data, &y_data)
            .map(|fit| fit.r_squared)
            .unwrap_or(0.0);

        scores.push((beta_star, r2));
    }

    scores
}

/// 方法二：通过 log(M) vs log(Tc - T) 分析估算 β
/// 返回拟合结果，slope 即为 β
///
/// 注意：为避免有限尺寸效应和临界慢化，只使用满足以下条件的数据点：
/// - T < 0.98 * Tc （避免太接近Tc的数据点）
/// - T > 0.85 * Tc （使用临界区域附近的数据）
/// - M > 0.1 （排除磁化强度过小的点）
fn analyze_loglog_fit(results: &[MeasurementResult], tc: f64) -> Option<LinearFit> {
    let mut x_data = Vec::new();
    let mut y_data = Vec::new();

    // 定义有效的温度窗口
    let t_min = tc * 0.85;
    let t_max = tc * 0.98;

    for r in results {
        // 只使用临界区域附近但不太接近Tc的数据点
        if r.temperature > t_min && r.temperature < t_max {
            let delta_t = tc - r.temperature;
            // 排除 M 过小的点（可能是有限尺寸效应或采样不足）
            if delta_t > 0.01 && r.magnetization > 0.1 {
                x_data.push(delta_t.ln());
                y_data.push(r.magnetization.ln());
            }
        }
    }

    linear_regression(&x_data, &y_data)
}

// ============================================================================
// 输出与报告
// ============================================================================

/// 将结果写入 CSV 文件
fn write_results_csv(
    path: &str,
    results: &[MeasurementResult],
    lattice_type: LatticeType,
    beta_star_analysis: &[(f64, f64)],
    loglog_fit: Option<&LinearFit>,
    tc: f64,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    // 写入元信息
    writeln!(file, "# 伊辛模型临界指数计算结果")?;
    writeln!(file, "# 晶格类型: {:?}, 理论 Tc = {:.4}", lattice_type, tc)?;
    writeln!(file)?;

    // 磁化强度数据
    writeln!(file, "# 温度扫描数据")?;
    writeln!(file, "T,M,M_std")?;
    for r in results {
        writeln!(
            file,
            "{:.6},{:.6},{:.6}",
            r.temperature, r.magnetization, r.magnetization_std
        )?;
    }
    writeln!(file)?;

    // β* 线性度分析
    writeln!(file, "# 方法一: M^(1/beta*) vs T 线性度分析")?;
    writeln!(file, "beta_star,R_squared")?;
    for (beta_star, r2) in beta_star_analysis {
        writeln!(file, "{:.4},{:.6}", beta_star, r2)?;
    }
    writeln!(file)?;

    // log-log 拟合
    writeln!(file, "# 方法二: log(M) vs log(Tc-T) 拟合")?;
    if let Some(fit) = loglog_fit {
        writeln!(file, "# slope (beta) = {:.6}", fit.slope)?;
        writeln!(file, "# R_squared = {:.6}", fit.r_squared)?;
    } else {
        writeln!(file, "# 拟合失败")?;
    }

    Ok(())
}

/// 打印结果摘要到终端
fn print_summary(
    lattice_type: LatticeType,
    results: &[MeasurementResult],
    beta_star_analysis: &[(f64, f64)],
    loglog_fit: Option<&LinearFit>,
    tc: f64,
) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            伊辛模型临界指数计算结果                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("晶格类型: {:?}", lattice_type);
    println!("理论临界温度 Tc = {:.4}", tc);
    println!("理论临界指数 β = 0.125 (1/8)");
    println!();

    // 显示部分磁化强度数据
    println!("═══ 磁化强度数据 (部分) ═══");
    println!("{:>10} {:>12} {:>12}", "T", "M", "M_std");
    println!("{:-<36}", "");
    for r in results.iter().take(10) {
        println!(
            "{:>10.4} {:>12.6} {:>12.6}",
            r.temperature, r.magnetization, r.magnetization_std
        );
    }
    if results.len() > 10 {
        println!("... (共 {} 个数据点)", results.len());
    }
    println!();

    // β* 分析结果
    println!("═══ 方法一: M^(1/β*) vs T 线性度分析 ═══");
    println!("{:>10} {:>12}", "β*", "R²");
    println!("{:-<24}", "");
    let mut best_beta_star = 0.0;
    let mut best_r2 = 0.0;
    for (beta_star, r2) in beta_star_analysis {
        println!("{:>10.4} {:>12.6}", beta_star, r2);
        if *r2 > best_r2 {
            best_r2 = *r2;
            best_beta_star = *beta_star;
        }
    }
    println!();
    println!("最佳 β* = {:.4} (R² = {:.6})", best_beta_star, best_r2);
    println!();

    // log-log 拟合结果
    println!("═══ 方法二: log(M) vs log(Tc-T) 拟合 ═══");
    if let Some(fit) = loglog_fit {
        println!("斜率 (β) = {:.6}", fit.slope);
        println!("R² = {:.6}", fit.r_squared);
        println!();
        let error = (fit.slope - 0.125).abs() / 0.125 * 100.0;
        println!("与理论值 0.125 的相对误差: {:.2}%", error);
    } else {
        println!("拟合失败：数据点不足");
    }

    println!();
    println!("════════════════════════════════════════════════════════════════");
}

// ============================================================================
// 主程序
// ============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // 解析参数
    let lattice_type = LatticeType::from_str(&args.lattice);
    let tc = args
        .tc
        .unwrap_or_else(|| lattice_type.critical_temperature());

    println!("伊辛模型临界指数计算");
    println!(
        "晶格: {:?}, 尺寸: {}×{}",
        lattice_type, args.size, args.size
    );
    println!("Tc = {:.4}", tc);
    println!();

    // 初始化随机数生成器
    let mut rng = StdRng::seed_from_u64(args.seed);

    // 生成温度扫描列表
    let temperatures = generate_temperature_range(lattice_type);

    println!("开始温度扫描 ({} 个温度点)...", temperatures.len());

    // 进行模拟
    let mut results = Vec::new();
    for (i, &temp) in temperatures.iter().enumerate() {
        let result = simulate_at_temperature(
            args.size,
            lattice_type,
            temp,
            args.equilibration,
            args.sweeps,
            &mut rng,
        );
        println!(
            "[{:>2}/{}] T = {:.4}, M = {:.6}",
            i + 1,
            temperatures.len(),
            temp,
            result.magnetization
        );
        results.push(result);
    }

    // β* 线性度分析
    let beta_star_values: Vec<f64> = (5..=20).map(|i| i as f64 * 0.01).collect(); // 0.05 到 0.20
    let beta_star_analysis = analyze_beta_star_linearity(&results, &beta_star_values);

    // log-log 拟合
    let loglog_fit = analyze_loglog_fit(&results, tc);

    // 输出结果
    write_results_csv(
        &args.output,
        &results,
        lattice_type,
        &beta_star_analysis,
        loglog_fit.as_ref(),
        tc,
    )?;

    // 打印摘要
    print_summary(
        lattice_type,
        &results,
        &beta_star_analysis,
        loglog_fit.as_ref(),
        tc,
    );

    println!("\n结果已保存至: {}", args.output);

    // 生成可视化
    if !args.no_plot {
        generate_plots(
            &results,
            &beta_star_analysis,
            loglog_fit.as_ref(),
            tc,
            lattice_type,
        )?;
    }

    Ok(())
}

// ============================================================================
// 可视化模块 (plotters)
// ============================================================================

/// 生成所有图表并保存到 plots/ 目录
fn generate_plots(
    results: &[MeasurementResult],
    beta_star_analysis: &[(f64, f64)],
    loglog_fit: Option<&LinearFit>,
    tc: f64,
    lattice_type: LatticeType,
) -> Result<(), Box<dyn Error>> {
    create_dir_all("plots")?;

    let lattice_name = format!("{:?}", lattice_type).to_lowercase();

    plot_m_vs_t(results, tc, &lattice_name)?;
    plot_loglog(results, tc, loglog_fit, &lattice_name)?;
    plot_beta_star(results, beta_star_analysis, &lattice_name)?;

    println!("可视化图表已生成到 plots/ 目录");
    Ok(())
}

/// 绘制 M vs T 曲线图，并标注临界温度 Tc
fn plot_m_vs_t(
    results: &[MeasurementResult],
    tc: f64,
    lattice_name: &str,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("plots/m_vs_t_{}.png", lattice_name);
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算数据范围
    let min_t = results
        .iter()
        .map(|r| r.temperature)
        .fold(f64::INFINITY, f64::min);
    let max_t = results
        .iter()
        .map(|r| r.temperature)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_m = results
        .iter()
        .map(|r| r.magnetization)
        .fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Magnetization M vs Temperature T ({} lattice)", lattice_name),
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_t * 0.95..max_t * 1.02, 0.0..max_m * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Temperature T")
        .y_desc("Magnetization |M|")
        .draw()?;

    // 绘制数据点和连线
    chart
        .draw_series(LineSeries::new(
            results.iter().map(|r| (r.temperature, r.magnetization)),
            &BLUE,
        ))?
        .label("M(T)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // 绘制误差棒（简化版，只画点）
    chart.draw_series(results.iter().map(|r| {
        Circle::new((r.temperature, r.magnetization), 3, BLUE.filled())
    }))?;

    // 标注临界温度 Tc
    if tc > min_t && tc < max_t * 1.1 {
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(tc, 0.0), (tc, max_m * 1.05)],
            RED.stroke_width(2),
        )))?;

        chart.draw_series(std::iter::once(Text::new(
            format!("Tc = {:.3}", tc),
            (tc + 0.02, max_m * 0.95),
            ("sans-serif", 14).into_font().color(&RED),
        )))?;
    }

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;
    println!("  生成: {}", filename);
    Ok(())
}

/// 绘制 log(M) vs log(Tc-T) 图，显示幂律行为和拟合直线
fn plot_loglog(
    results: &[MeasurementResult],
    tc: f64,
    loglog_fit: Option<&LinearFit>,
    lattice_name: &str,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("plots/loglog_beta_{}.png", lattice_name);
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // 准备 log-log 数据
    let pts: Vec<(f64, f64)> = results
        .iter()
        .filter_map(|r| {
            let dt = tc - r.temperature;
            if dt > 0.01 && r.magnetization > 0.01 {
                Some((dt.ln(), r.magnetization.ln()))
            } else {
                None
            }
        })
        .collect();

    if pts.is_empty() {
        return Ok(());
    }

    // 计算范围
    let (min_x, max_x) = pts
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |acc, &(x, _)| {
            (acc.0.min(x), acc.1.max(x))
        });
    let (min_y, max_y) = pts
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |acc, &(_, y)| {
            (acc.0.min(y), acc.1.max(y))
        });

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("log(M) vs log(Tc - T) ({} lattice)", lattice_name),
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(min_x - 0.2..max_x + 0.2, min_y - 0.1..max_y + 0.1)?;

    chart
        .configure_mesh()
        .x_desc("log(Tc - T)")
        .y_desc("log(M)")
        .draw()?;

    // 绘制数据点
    chart.draw_series(
        pts.iter()
            .map(|&(x, y)| Circle::new((x, y), 4, BLUE.filled())),
    )?;

    // 绘制拟合直线
    if let Some(fit) = loglog_fit {
        let line_pts: Vec<(f64, f64)> = vec![
            (min_x - 0.1, fit.slope * (min_x - 0.1) + fit.intercept),
            (max_x + 0.1, fit.slope * (max_x + 0.1) + fit.intercept),
        ];
        chart
            .draw_series(LineSeries::new(line_pts, RED.stroke_width(2)))?
            .label(format!("beta = {:.4}, R^2 = {:.4}", fit.slope, fit.r_squared))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .position(SeriesLabelPosition::LowerRight)
            .draw()?;
    }

    root.present()?;
    println!("  生成: {}", filename);
    Ok(())
}

/// 绘制最佳 β* 的 M^(1/β*) vs T 曲线
fn plot_beta_star(
    results: &[MeasurementResult],
    beta_star_analysis: &[(f64, f64)],
    lattice_name: &str,
) -> Result<(), Box<dyn Error>> {
    if beta_star_analysis.is_empty() {
        return Ok(());
    }

    // 找到 R² 最大的 β*
    let (best_beta, best_r2) = beta_star_analysis
        .iter()
        .cloned()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    let filename = format!("plots/m_beta_star_{}.png", lattice_name);
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算数据
    let data: Vec<(f64, f64)> = results
        .iter()
        .filter(|r| r.magnetization > 1e-6)
        .map(|r| (r.temperature, r.magnetization.powf(1.0 / best_beta)))
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let min_t = data.iter().map(|&(t, _)| t).fold(f64::INFINITY, f64::min);
    let max_t = data
        .iter()
        .map(|&(t, _)| t)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = data.iter().map(|&(_, y)| y).fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "M^(1/beta*) vs T (beta* = {:.3}, R^2 = {:.4})",
                best_beta, best_r2
            ),
            ("sans-serif", 22),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_t * 0.95..max_t * 1.02, 0.0..max_y * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Temperature T")
        .y_desc("M^(1/beta*)")
        .draw()?;

    // 绘制数据
    chart
        .draw_series(LineSeries::new(data.iter().cloned(), &BLUE))?
        .label(format!("M^(1/{:.3})", best_beta))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart.draw_series(data.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())))?;

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;
    println!("  生成: {}", filename);
    Ok(())
}
