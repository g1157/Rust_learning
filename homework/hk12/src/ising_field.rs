//! 伊辛模型高温外场响应程序
//!
//! 本程序验证在高温条件下（T >> Tc），伊辛模型的磁化强度M(H)近似满足
//! M = tanh(H/T) 的理论预期，并分析随温度降低偏差增大的现象。
//!
//! # 物理背景
//! 在极高温下，自旋间的相互作用可以忽略，此时系统行为由单自旋在外场中的
//! 响应决定。根据玻尔兹曼分布：
//!   M = tanh(μH / k_B T)
//! 设 μ = k_B = 1，则 M = tanh(H/T)
//!
//! # 运行方式
//! ```bash
//! # 默认参数 (T = 100, 30, 10)
//! cargo run --release --bin ising_field
//!
//! # 自定义温度
//! cargo run --release --bin ising_field -- --temperatures 100,50,20,10
//!
//! # 禁用可视化
//! cargo run --release --bin ising_field -- --no-plot
//! ```

use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;

use clap::Parser;
use plotters::prelude::*;
use rand::prelude::*;

// ============================================================================
// 命令行参数定义
// ============================================================================

/// 伊辛模型高温外场响应程序
#[derive(Parser, Debug)]
#[command(author, version, about = "验证高温下M(H) ≈ tanh(H/T)")]
struct Args {
    /// 晶格尺寸 L (总自旋数 = L × L)
    #[arg(long, short = 'L', default_value = "20")]
    size: usize,

    /// 温度列表，逗号分隔 (推荐: 100,30,10)
    #[arg(long, short = 'T', default_value = "100,30,10")]
    temperatures: String,

    /// 外场 H 扫描起点
    #[arg(long, default_value = "0.0")]
    h_min: f64,

    /// 外场 H 扫描终点
    #[arg(long, default_value = "5.0")]
    h_max: f64,

    /// 外场 H 扫描点数
    #[arg(long, default_value = "25")]
    h_points: usize,

    /// 每个 (T, H) 点的蒙特卡洛扫描次数
    #[arg(long, default_value = "2000")]
    sweeps: usize,

    /// 平衡化扫描次数
    #[arg(long, default_value = "1000")]
    equilibration: usize,

    /// 随机数种子
    #[arg(long, default_value = "2024")]
    seed: u64,

    /// 输出CSV文件路径
    #[arg(long, default_value = "ising_field_results.csv")]
    output: String,

    /// 禁用可视化输出
    #[arg(long, default_value = "false")]
    no_plot: bool,
}

// ============================================================================
// 晶格结构 (正方形晶格)
// ============================================================================

/// 二维正方形伊辛晶格（含外场）
struct IsingLattice {
    /// 晶格线性尺寸
    size: usize,
    /// 自旋数组 (+1 或 -1)
    spins: Vec<i8>,
    /// 每个格点的 4 个近邻索引
    neighbors: Vec<[usize; 4]>,
}

impl IsingLattice {
    /// 创建新晶格，初始化为全 +1 (有利于正外场下更快达到平衡)
    fn new(size: usize) -> Self {
        let n = size * size;
        let spins = vec![1i8; n]; // 初始化为全 +1
        let neighbors = Self::build_neighbors(size);

        Self {
            size,
            spins,
            neighbors,
        }
    }

    /// 构建周期性边界条件下的近邻列表
    fn build_neighbors(size: usize) -> Vec<[usize; 4]> {
        let n = size * size;
        let mut neighbors = vec![[0usize; 4]; n];

        for row in 0..size {
            for col in 0..size {
                let idx = row * size + col;

                let up = ((row + size - 1) % size) * size + col;
                let down = ((row + 1) % size) * size + col;
                let left = row * size + (col + size - 1) % size;
                let right = row * size + (col + 1) % size;

                neighbors[idx] = [up, down, left, right];
            }
        }

        neighbors
    }

    /// 计算翻转指定自旋导致的能量变化（含外场）
    ///
    /// 哈密顿量: H = -J Σ s_i s_j - H Σ s_i
    /// 翻转 s_i 的能量变化: ΔE = 2 s_i (J Σ_nn s_j + H)
    fn energy_delta(&self, idx: usize, external_field: f64) -> f64 {
        let spin = self.spins[idx] as f64;

        // 近邻自旋求和
        let neighbor_sum: f64 = self.neighbors[idx]
            .iter()
            .map(|&j| self.spins[j] as f64)
            .sum();

        // J = 1, μ = 1
        2.0 * spin * (neighbor_sum + external_field)
    }

    /// 执行一次 Metropolis 扫描（含外场）
    fn metropolis_sweep(&mut self, beta: f64, external_field: f64, rng: &mut StdRng) {
        let n = self.spins.len();

        for _ in 0..n {
            let idx = rng.gen_range(0..n);
            let delta_e = self.energy_delta(idx, external_field);

            // Metropolis 接受准则
            if delta_e <= 0.0 || rng.gen::<f64>() < (-beta * delta_e).exp() {
                self.spins[idx] = -self.spins[idx];
            }
        }
    }

    /// 计算当前磁化强度 (不取绝对值，因为有外场会打破对称性)
    fn magnetization(&self) -> f64 {
        let total: i32 = self.spins.iter().map(|&s| s as i32).sum();
        total as f64 / self.spins.len() as f64
    }
}

// ============================================================================
// 数据结构
// ============================================================================

/// 单个 (T, H) 点的测量结果
#[derive(Clone, Debug)]
struct MeasurementRecord {
    /// 温度
    temperature: f64,
    /// 外场强度
    field: f64,
    /// 模拟得到的磁化强度
    m_simulation: f64,
    /// 理论预期值 tanh(H/T)
    m_theory: f64,
    /// 绝对误差
    abs_error: f64,
    /// 相对误差 (%)
    rel_error_percent: f64,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 解析逗号分隔的浮点数列表
fn parse_float_list(s: &str) -> Vec<f64> {
    s.split(',')
        .filter_map(|x| x.trim().parse::<f64>().ok())
        .collect()
}

/// 生成等间距数列
fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![start];
    }
    let step = (end - start) / (n - 1) as f64;
    (0..n).map(|i| start + step * i as f64).collect()
}

// ============================================================================
// 模拟核心
// ============================================================================

/// 在指定 (T, H) 下进行模拟
fn simulate_at_point(
    size: usize,
    temperature: f64,
    field: f64,
    equilibration_sweeps: usize,
    measurement_sweeps: usize,
    rng: &mut StdRng,
) -> MeasurementRecord {
    let beta = 1.0 / temperature;
    let mut lattice = IsingLattice::new(size);

    // 平衡化
    for _ in 0..equilibration_sweeps {
        lattice.metropolis_sweep(beta, field, rng);
    }

    // 测量
    let mut m_sum = 0.0;
    for _ in 0..measurement_sweeps {
        lattice.metropolis_sweep(beta, field, rng);
        m_sum += lattice.magnetization();
    }

    let m_simulation = m_sum / measurement_sweeps as f64;
    let m_theory = (field / temperature).tanh();
    let abs_error = (m_simulation - m_theory).abs();
    let rel_error_percent = if m_theory.abs() > 1e-10 {
        (abs_error / m_theory.abs()) * 100.0
    } else {
        0.0
    };

    MeasurementRecord {
        temperature,
        field,
        m_simulation,
        m_theory,
        abs_error,
        rel_error_percent,
    }
}

/// 执行完整的参数扫描
fn run_simulation(args: &Args) -> Vec<MeasurementRecord> {
    let mut rng = StdRng::seed_from_u64(args.seed);
    let temperatures = parse_float_list(&args.temperatures);
    let fields = linspace(args.h_min, args.h_max, args.h_points);

    let total_points = temperatures.len() * fields.len();
    let mut results = Vec::with_capacity(total_points);

    println!("开始模拟...");
    println!(
        "温度: {:?}",
        temperatures
            .iter()
            .map(|t| format!("{:.1}", t))
            .collect::<Vec<_>>()
    );
    println!(
        "外场 H: {:.2} 到 {:.2} ({} 个点)",
        args.h_min, args.h_max, args.h_points
    );
    println!();

    let mut count = 0;
    for &temp in &temperatures {
        print!("T = {:>6.1}: ", temp);
        std::io::stdout().flush().ok();

        for &h in &fields {
            let record = simulate_at_point(
                args.size,
                temp,
                h,
                args.equilibration,
                args.sweeps,
                &mut rng,
            );
            results.push(record);
            count += 1;

            // 进度指示
            if count % 5 == 0 {
                print!(".");
                std::io::stdout().flush().ok();
            }
        }
        println!(" 完成");
    }

    results
}

// ============================================================================
// 输出与分析
// ============================================================================

/// 将结果写入 CSV 文件
fn write_results_csv(path: &str, records: &[MeasurementRecord]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    // 文件头
    writeln!(file, "# 伊辛模型高温外场响应计算结果")?;
    writeln!(file, "# 理论预期: M = tanh(H/T) (J=1, μ=1, kB=1)")?;
    writeln!(file)?;

    // CSV 头
    writeln!(
        file,
        "T,H,M_simulation,M_theory,abs_error,rel_error_percent"
    )?;

    for r in records {
        writeln!(
            file,
            "{:.4},{:.4},{:.6},{:.6},{:.6},{:.4}",
            r.temperature, r.field, r.m_simulation, r.m_theory, r.abs_error, r.rel_error_percent
        )?;
    }

    Ok(())
}

/// 计算并打印每个温度的误差统计
fn analyze_temperature_dependence(records: &[MeasurementRecord], temperatures: &[f64]) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            误差分析: M_sim vs tanh(H/T)                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!(
        "{:>10} {:>15} {:>15} {:>15}",
        "温度 T", "平均绝对误差", "最大绝对误差", "平均相对误差(%)"
    );
    println!("{:-<58}", "");

    let mut analysis_results = Vec::new();

    for &temp in temperatures {
        let temp_records: Vec<_> = records
            .iter()
            .filter(|r| (r.temperature - temp).abs() < 1e-9)
            .collect();

        if temp_records.is_empty() {
            continue;
        }

        let mean_abs_error =
            temp_records.iter().map(|r| r.abs_error).sum::<f64>() / temp_records.len() as f64;
        let max_abs_error = temp_records.iter().map(|r| r.abs_error).fold(0.0, f64::max);
        let mean_rel_error = temp_records
            .iter()
            .filter(|r| r.m_theory.abs() > 1e-10)
            .map(|r| r.rel_error_percent)
            .sum::<f64>()
            / temp_records
                .iter()
                .filter(|r| r.m_theory.abs() > 1e-10)
                .count()
                .max(1) as f64;

        println!(
            "{:>10.1} {:>15.6} {:>15.6} {:>15.4}",
            temp, mean_abs_error, max_abs_error, mean_rel_error
        );

        analysis_results.push((temp, mean_abs_error));
    }

    // 分析趋势
    println!("\n═══ 趋势分析 ═══");
    if analysis_results.len() >= 2 {
        let first_error = analysis_results.first().unwrap().1;
        let last_error = analysis_results.last().unwrap().1;

        if last_error > first_error {
            let ratio = last_error / first_error.max(1e-10);
            println!("✓ 验证成功：随着温度降低，与 tanh(H/T) 的偏差增大");
            println!(
                "  从 T = {:.1} 到 T = {:.1}，平均绝对误差增大了 {:.1} 倍",
                analysis_results.first().unwrap().0,
                analysis_results.last().unwrap().0,
                ratio
            );
        }
    }

    println!("\n═══ 物理解释 ═══");
    println!("• tanh(H/T) 是单自旋近似的结果，忽略了自旋间相互作用");
    println!("• 高温时 (T >> Tc ≈ 2.27)，热涨落主导，相互作用可忽略");
    println!("• 温度降低时，自旋间相互作用逐渐显著，导致偏差增大");
    println!("• 当 T → Tc 时，临界涨落使得单自旋近似完全失效");
}

/// 打印部分数据示例
fn print_sample_data(records: &[MeasurementRecord]) {
    println!("\n═══ 数据示例 (每个温度前5个点) ═══");
    println!(
        "{:>8} {:>8} {:>12} {:>12} {:>12}",
        "T", "H", "M_sim", "M_theory", "误差"
    );
    println!("{:-<56}", "");

    let mut current_temp = f64::NAN;
    let mut count_at_temp = 0;

    for r in records {
        if (r.temperature - current_temp).abs() > 1e-9 {
            current_temp = r.temperature;
            count_at_temp = 0;
            if !current_temp.is_nan() {
                println!(); // 温度变化时换行
            }
        }

        if count_at_temp < 5 {
            println!(
                "{:>8.1} {:>8.3} {:>12.6} {:>12.6} {:>12.6}",
                r.temperature, r.field, r.m_simulation, r.m_theory, r.abs_error
            );
            count_at_temp += 1;
        }
    }
}

// ============================================================================
// 主程序
// ============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         伊辛模型高温外场响应验证程序                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("参数设置:");
    println!("  晶格尺寸: {}×{}", args.size, args.size);
    println!("  温度列表: {}", args.temperatures);
    println!("  外场范围: {:.2} 到 {:.2}", args.h_min, args.h_max);
    println!("  平衡化步数: {}", args.equilibration);
    println!("  测量步数: {}", args.sweeps);
    println!();

    // 运行模拟
    let records = run_simulation(&args);

    // 解析温度列表用于分析
    let temperatures = parse_float_list(&args.temperatures);

    // 输出结果
    write_results_csv(&args.output, &records)?;
    println!("\n结果已保存至: {}", args.output);

    // 打印示例数据
    print_sample_data(&records);

    // 分析温度依赖性
    analyze_temperature_dependence(&records, &temperatures);

    // 生成可视化
    if !args.no_plot {
        generate_plots(&records, &temperatures)?;
    }

    println!("\n════════════════════════════════════════════════════════════════");

    Ok(())
}

// ============================================================================
// 可视化模块 (plotters)
// ============================================================================

/// 定义不同温度的颜色
fn get_color_for_index(idx: usize) -> RGBColor {
    const COLORS: [RGBColor; 6] = [
        RGBColor(31, 119, 180),   // 蓝色
        RGBColor(255, 127, 14),   // 橙色
        RGBColor(44, 160, 44),    // 绿色
        RGBColor(214, 39, 40),    // 红色
        RGBColor(148, 103, 189),  // 紫色
        RGBColor(140, 86, 75),    // 棕色
    ];
    COLORS[idx % COLORS.len()]
}

/// 生成所有图表
fn generate_plots(
    records: &[MeasurementRecord],
    temperatures: &[f64],
) -> Result<(), Box<dyn Error>> {
    create_dir_all("plots")?;

    plot_m_vs_h(records, temperatures)?;
    plot_error_vs_temperature(records, temperatures)?;

    println!("\n可视化图表已生成到 plots/ 目录");
    Ok(())
}

/// 绘制 M(H) vs H 曲线，多温度对比，包含理论曲线
fn plot_m_vs_h(
    records: &[MeasurementRecord],
    temperatures: &[f64],
) -> Result<(), Box<dyn Error>> {
    let filename = "plots/m_vs_h_field.png";
    let root = BitMapBackend::new(filename, (900, 650)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算数据范围
    let max_h = records
        .iter()
        .map(|r| r.field)
        .fold(0.0, f64::max);
    let max_m = records
        .iter()
        .map(|r| r.m_simulation.abs().max(r.m_theory.abs()))
        .fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("M(H) vs H - Simulation vs Theory", ("sans-serif", 26))
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(55)
        .build_cartesian_2d(0.0..max_h * 1.05, 0.0..max_m * 1.15)?;

    chart
        .configure_mesh()
        .x_desc("External Field H")
        .y_desc("Magnetization M")
        .draw()?;

    // 为每个温度绘制模拟曲线和理论曲线
    for (idx, &temp) in temperatures.iter().enumerate() {
        let color = get_color_for_index(idx);

        // 获取该温度的数据
        let temp_data: Vec<_> = records
            .iter()
            .filter(|r| (r.temperature - temp).abs() < 1e-6)
            .collect();

        if temp_data.is_empty() {
            continue;
        }

        // 模拟数据（实线 + 点）
        let sim_points: Vec<(f64, f64)> = temp_data
            .iter()
            .map(|r| (r.field, r.m_simulation))
            .collect();

        chart
            .draw_series(LineSeries::new(sim_points.clone(), color.stroke_width(2)))?
            .label(format!("T={:.0} (sim)", temp))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));

        chart.draw_series(
            sim_points
                .iter()
                .map(|&(x, y)| Circle::new((x, y), 3, color.filled())),
        )?;

        // 理论曲线（虚线）
        let theory_points: Vec<(f64, f64)> = temp_data
            .iter()
            .map(|r| (r.field, r.m_theory))
            .collect();

        chart
            .draw_series(DashedLineSeries::new(
                theory_points,
                5,
                3,
                color.stroke_width(1),
            ))?
            .label(format!("T={:.0} (tanh)", temp))
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 5, y), (x + 10, y), (x + 15, y), (x + 20, y)], color)
            });
    }

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .position(SeriesLabelPosition::LowerRight)
        .background_style(WHITE.mix(0.8))
        .draw()?;

    root.present()?;
    println!("  生成: {}", filename);
    Ok(())
}

/// 绘制误差随温度变化的柱状图
fn plot_error_vs_temperature(
    records: &[MeasurementRecord],
    temperatures: &[f64],
) -> Result<(), Box<dyn Error>> {
    let filename = "plots/error_vs_temp.png";
    let root = BitMapBackend::new(filename, (700, 500)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算每个温度的平均误差
    let mut error_data: Vec<(f64, f64)> = Vec::new();
    for &temp in temperatures {
        let temp_records: Vec<_> = records
            .iter()
            .filter(|r| (r.temperature - temp).abs() < 1e-6)
            .collect();

        if !temp_records.is_empty() {
            let mean_error =
                temp_records.iter().map(|r| r.abs_error).sum::<f64>() / temp_records.len() as f64;
            error_data.push((temp, mean_error));
        }
    }

    if error_data.is_empty() {
        return Ok(());
    }

    let max_error = error_data.iter().map(|&(_, e)| e).fold(0.0, f64::max);
    let max_temp = temperatures.iter().fold(0.0, |a, &b| f64::max(a, b));

    let mut chart = ChartBuilder::on(&root)
        .caption("Mean Absolute Error vs Temperature", ("sans-serif", 24))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(55)
        .build_cartesian_2d(0.0..max_temp * 1.1, 0.0..max_error * 1.2)?;

    chart
        .configure_mesh()
        .x_desc("Temperature T")
        .y_desc("Mean Absolute Error |M_sim - tanh(H/T)|")
        .draw()?;

    // 绘制柱状图
    let bar_width = max_temp * 0.08;
    chart.draw_series(error_data.iter().map(|&(t, e)| {
        Rectangle::new(
            [(t - bar_width / 2.0, 0.0), (t + bar_width / 2.0, e)],
            RGBColor(70, 130, 180).filled(),
        )
    }))?;

    // 在柱上方添加数值标签
    for &(t, e) in &error_data {
        chart.draw_series(std::iter::once(Text::new(
            format!("{:.4}", e),
            (t, e + max_error * 0.03),
            ("sans-serif", 12).into_font().color(&BLACK),
        )))?;
    }

    root.present()?;
    println!("  生成: {}", filename);
    Ok(())
}
