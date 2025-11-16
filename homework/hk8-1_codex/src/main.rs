use anyhow::{Result, anyhow};
use clap::Parser;
use plotly::{
    Plot, Scatter,
    common::{Mode, Title},
    layout::{Axis, AxisType, Layout},
};
use std::{
    f64::consts::PI,
    fs,
    path::{Path, PathBuf},
};

const GM_SATURN: f64 = 4.0 * PI * PI;
const TWO_PI: f64 = 2.0 * PI;
const DEFAULT_SEMIMAJOR_AXIS: f64 = 1.0; // Hyperion units

#[derive(Parser, Debug)]
#[command(author, version, about = "Hyperion 模型模拟器", long_about = None)]
struct Cli {
    /// 单个偏心率。若提供 --eccentricities 列表则忽略本项
    #[arg(long, default_value_t = 0.0)]
    eccentricity: f64,
    /// 多个偏心率，逗号分隔或重复参数
    #[arg(long, value_delimiter = ',', num_args = 1.., conflicts_with = "eccentricity")]
    eccentricities: Option<Vec<f64>>,
    /// 模拟时长（Hyperion 年）
    #[arg(long, default_value_t = 2.0)]
    duration: f64,
    /// 时间步长
    #[arg(long, default_value_t = 1e-4)]
    dt: f64,
    /// 第二条轨迹初始角度的偏移量（弧度）
    #[arg(long, default_value_t = 0.01)]
    theta_offset: f64,
    /// 当 |Δθ| 超过 delta_max 时是否执行归一化（Benettin 法）
    #[arg(long)]
    renormalize: bool,
    /// 归一化后的目标 |Δθ|，默认等于 theta_offset
    #[arg(long)]
    renorm_target: Option<f64>,
    /// 估计 Lyapunov 指数时使用的 Δθ 下限
    #[arg(long, default_value_t = 1e-8)]
    delta_min: f64,
    /// 估计 Lyapunov 指数时使用的 Δθ 上限
    #[arg(long, default_value_t = 0.3)]
    delta_max: f64,
    /// 绘图输出目录
    #[arg(long, default_value = "plots")]
    output_dir: String,
    /// 仅运行数值模拟，跳过绘图
    #[arg(long)]
    no_plots: bool,
}

#[derive(Clone, Copy, Debug)]
struct State {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    theta: f64,
    omega: f64,
}

#[derive(Debug)]
struct SimulationConfig {
    eccentricity: f64,
    dt: f64,
    steps: usize,
    theta_offset: f64,
    renormalize: bool,
    renorm_target: f64,
    delta_max: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SimulationResult {
    times: Vec<f64>,
    theta_primary: Vec<f64>,
    theta_secondary: Vec<f64>,
    delta_theta: Vec<f64>,
    renorm_events: Vec<RenormEvent>,
    log_stretch_sum: f64,
}

#[derive(Debug, Clone)]
struct RenormEvent {
    time: f64,
    delta_before: f64,
    log_stretch: f64,
}

#[derive(Debug)]
struct EccentricityReport {
    eccentricity: f64,
    lyapunov: Option<f64>,
    sample_count: usize,
    result: SimulationResult,
    method: EstimatorKind,
}

#[derive(Debug, Clone, Copy)]
enum EstimatorKind {
    Regression,
    Benettin,
}

impl EstimatorKind {
    fn label(self) -> &'static str {
        match self {
            EstimatorKind::Regression => "线性回归",
            EstimatorKind::Benettin => "Benettin 归一化",
        }
    }
}

/// 程序入口。
///
/// 解析命令行参数，针对每个指定的偏心率运行模拟、估计 Lyapunov 指数，
/// 并（可选地）将结果绘制为 HTML 图表保存在指定目录。
///
/// 返回：若运行成功返回 Ok(())，否则返回 Err 包含错误信息。
fn main() -> Result<()> {
    let cli = Cli::parse();
    validate_cli(&cli)?;

    let eccs = cli
        .eccentricities
        .clone()
        .unwrap_or_else(|| vec![cli.eccentricity]);

    let steps = compute_steps(cli.duration, cli.dt)?;
    let renorm_target = cli
        .renorm_target
        .unwrap_or_else(|| cli.theta_offset.abs())
        .abs();

    let mut reports = Vec::with_capacity(eccs.len());
    for ecc in eccs {
        let config = SimulationConfig {
            eccentricity: ecc,
            dt: cli.dt,
            steps,
            theta_offset: cli.theta_offset,
            renormalize: cli.renormalize,
            renorm_target,
            delta_max: cli.delta_max,
        };
        let report = run_for_eccentricity(&config, cli.delta_min, cli.delta_max)?;
        print_report(&report);
        reports.push(report);
    }

    summarize_reports(&reports);

    if !cli.no_plots {
        write_plots(&reports, &cli.output_dir)?;
    }

    Ok(())
}

/// 校验命令行参数的合法性。
///
/// - 检查时间步长 `dt`、模拟时长 `duration`、以及用于拟合的 `delta_min`/`delta_max` 是否为正，
///   并确保 `delta_min < delta_max`。
/// - 若参数非法，返回 Err 描述错误原因；否则返回 Ok(())。
fn validate_cli(cli: &Cli) -> Result<()> {
    if cli.dt <= 0.0 {
        return Err(anyhow!("dt 必须为正"));
    }
    if cli.duration <= 0.0 {
        return Err(anyhow!("duration 必须为正"));
    }
    if cli.delta_min <= 0.0 || cli.delta_max <= 0.0 {
        return Err(anyhow!("delta_min 和 delta_max 必须为正"));
    }
    if cli.delta_min >= cli.delta_max {
        return Err(anyhow!("delta_min 需要小于 delta_max"));
    }
    if cli.theta_offset == 0.0 {
        return Err(anyhow!("theta_offset 不能为 0"));
    }
    if let Some(target) = cli.renorm_target {
        if target == 0.0 {
            return Err(anyhow!("renorm_target 不能为 0"));
        }
    }
    if cli.renormalize {
        let target = cli
            .renorm_target
            .unwrap_or_else(|| cli.theta_offset.abs())
            .abs();
        if target == 0.0 {
            return Err(anyhow!(
                "开启 renormalize 时需要合法的 theta_offset/renorm_target"
            ));
        }
        if target >= cli.delta_max {
            return Err(anyhow!("renorm_target 需要小于 delta_max"));
        }
    }
    Ok(())
}

/// 根据总时长和时间步长计算积分步数。
///
/// 输入：
/// - `duration`：模拟总时长（年）
/// - `dt`：时间步长（年）
///
/// 输出：返回计算出的步数（向上取整为 usize）。若计算后步数为 0，返回 Err。
fn compute_steps(duration: f64, dt: f64) -> Result<usize> {
    let steps = (duration / dt).ceil() as usize;
    if steps == 0 {
        return Err(anyhow!("模拟步数为 0，请调整 duration 或 dt"));
    }
    Ok(steps)
}

/// 对单个偏心率执行完整流程：运行两条初始条件略有不同的轨迹并估计 Lyapunov 指数。
///
/// 输入：
/// - `config`：包含偏心率、时间步长、步数及 theta 初始偏移等模拟配置。
/// - `delta_min`/`delta_max`：在该 Δθ 范围内的数据将用于对 ln|Δθ| 关于时间做线性拟合来估计 λ。
///
/// 输出：返回 `EccentricityReport`，包含偏心率、估计的 λ（若无法估计则为 None）、样本数以及原始模拟数据。
fn run_for_eccentricity(
    config: &SimulationConfig,
    delta_min: f64,
    delta_max: f64,
) -> Result<EccentricityReport> {
    if !(0.0..1.0).contains(&config.eccentricity) {
        return Err(anyhow!("偏心率需在 [0, 1) 区间"));
    }

    let result = simulate_pair(config);
    let (lambda, sample_count, method) = estimate_lyapunov(&result, delta_min, delta_max);

    Ok(EccentricityReport {
        eccentricity: config.eccentricity,
        lyapunov: lambda,
        sample_count,
        result,
        method,
    })
}

/// 对给定偏心率和配置运行两条轨迹（初始角度相差 `theta_offset`）。
///
/// 使用 Euler-Cromer 风格的积分：先更新角速度/线速度，再更新角度/位置。
/// 返回 `SimulationResult`，包含时间序列、两条轨迹的角度以及两轨之间的角度差 |Δθ|。
fn simulate_pair(config: &SimulationConfig) -> SimulationResult {
    let mut primary = initial_state(config.eccentricity, 0.0);
    let mut secondary = initial_state(config.eccentricity, config.theta_offset);

    let mut times = Vec::with_capacity(config.steps + 1);
    let mut theta_primary = Vec::with_capacity(config.steps + 1);
    let mut theta_secondary = Vec::with_capacity(config.steps + 1);
    let mut delta_theta = Vec::with_capacity(config.steps + 1);
    let mut renorm_events = Vec::new();
    let mut log_stretch_sum = 0.0;
    let target_delta = config.renorm_target.max(1e-20);

    times.push(0.0);
    theta_primary.push(primary.theta);
    theta_secondary.push(secondary.theta);
    delta_theta.push(angle_difference(primary.theta, secondary.theta).abs());

    for step in 0..config.steps {
        integrate_step(&mut primary, config.dt);
        integrate_step(&mut secondary, config.dt);

        let t = (step + 1) as f64 * config.dt;
        times.push(t);
        theta_primary.push(primary.theta);
        theta_secondary.push(secondary.theta);
        let delta = angle_difference(primary.theta, secondary.theta);
        let delta_abs = delta.abs();
        delta_theta.push(delta_abs);

        if config.renormalize && delta_abs >= config.delta_max {
            if delta_abs > 0.0 && target_delta > 0.0 {
                let stretch = delta_abs / target_delta;
                if stretch.is_finite() && stretch > 0.0 {
                    let log_stretch = stretch.ln();
                    log_stretch_sum += log_stretch;
                    renorm_events.push(RenormEvent {
                        time: t,
                        delta_before: delta_abs,
                        log_stretch,
                    });
                }
                apply_renormalization(&primary, &mut secondary, target_delta, delta);
            }
        }
    }

    SimulationResult {
        times,
        theta_primary,
        theta_secondary,
        delta_theta,
        renorm_events,
        log_stretch_sum,
    }
}

/// 根据偏心率和角度偏移创建并返回一个初始状态。
///
/// 使用近拱点（近心点）位置作为初始位置，并根据开普勒能量守恒计算该位置的速度（简单近似）。
/// 返回的 `State` 包含质心位置 (x,y)、速度 (vx,vy)、角度 `theta` 及角速度 `omega`。
fn initial_state(eccentricity: f64, theta_offset: f64) -> State {
    let a = DEFAULT_SEMIMAJOR_AXIS;
    let r_periapsis = a * (1.0 - eccentricity);
    let position = r_periapsis;
    let velocity = (GM_SATURN * (2.0 / r_periapsis - 1.0 / a)).sqrt();

    State {
        x: position,
        y: 0.0,
        vx: 0.0,
        vy: velocity,
        theta: wrap_angle(theta_offset),
        omega: 0.0,
    }
}

/// 对单个状态执行一个时间步的数值积分。
///
/// - 计算质心的引力加速度并更新线速度/位置。
/// - 计算作用在不规则体上的扭矩并更新角加速度/角速度/角度。
///
/// 此处采用简单显式步进（Euler-Cromer 风格更新），并对 r 做小值限制以避免除零。
fn integrate_step(state: &mut State, dt: f64) {
    let r_squared = state.x * state.x + state.y * state.y;
    let r = r_squared.sqrt().max(1e-9);
    let r_cubed = r * r * r;
    let r_fifth = r_cubed * r_squared;

    let ax = -GM_SATURN * state.x / r_cubed;
    let ay = -GM_SATURN * state.y / r_cubed;

    let sin_theta = state.theta.sin();
    let cos_theta = state.theta.cos();
    let torque_term =
        (state.x * sin_theta - state.y * cos_theta) * (state.x * cos_theta + state.y * sin_theta);
    let angular_acc = -3.0 * GM_SATURN * torque_term / r_fifth;

    state.omega += angular_acc * dt;
    state.theta = wrap_angle(state.theta + state.omega * dt);

    state.vx += ax * dt;
    state.vy += ay * dt;
    state.x += state.vx * dt;
    state.y += state.vy * dt;
}

/// 将 secondary 轨迹重新拉回到 primary 附近，保持差值方向并缩放到 target_delta。
fn apply_renormalization(
    primary: &State,
    secondary: &mut State,
    target_delta: f64,
    current_delta: f64,
) {
    if target_delta <= 0.0 || current_delta == 0.0 {
        return;
    }
    let sign = current_delta.signum();
    let scale = (target_delta / current_delta.abs()).min(1.0);
    secondary.theta = wrap_angle(primary.theta + sign * target_delta);
    let omega_diff = secondary.omega - primary.omega;
    secondary.omega = primary.omega + omega_diff * scale;
}

/// 使用线性最小二乘对 ln|Δθ(t)| 关于 t 做拟合以估计 Lyapunov 指数 λ。
///
/// - `result`：从 `simulate_pair` 返回的模拟结果，包含时间序列与 |Δθ|。
/// - 仅使用满足 `delta_min <= |Δθ| <= delta_max` 的数据点来拟合，避免初期噪声与后期饱和影响。
///
/// 返回：(`Some(λ)`, 样本数) 或 (None, 样本数)（当样本点过少或计算不可行时）。
fn estimate_lyapunov(
    result: &SimulationResult,
    delta_min: f64,
    delta_max: f64,
) -> (Option<f64>, usize, EstimatorKind) {
    if !result.renorm_events.is_empty() {
        if let Some(&total_time) = result.times.last() {
            if total_time > 0.0 {
                let lambda = result.log_stretch_sum / total_time;
                return (
                    Some(lambda),
                    result.renorm_events.len(),
                    EstimatorKind::Benettin,
                );
            }
        }
    }

    let mut samples = Vec::new();
    for (t, delta) in result.times.iter().zip(result.delta_theta.iter()) {
        if *delta > 0.0 && *delta >= delta_min && *delta <= delta_max {
            samples.push((*t, delta.ln()));
        }
    }

    let sample_count = samples.len();
    if sample_count < 2 {
        return (None, sample_count, EstimatorKind::Regression);
    }

    let mean_t: f64 = samples.iter().map(|(t, _)| *t).sum::<f64>() / sample_count as f64;
    let mean_log: f64 = samples.iter().map(|(_, v)| *v).sum::<f64>() / sample_count as f64;

    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (t, log_delta) in samples {
        let dt = t - mean_t;
        numerator += dt * (log_delta - mean_log);
        denominator += dt * dt;
    }

    if denominator.abs() < f64::EPSILON {
        return (None, sample_count, EstimatorKind::Regression);
    }

    (
        Some(numerator / denominator),
        sample_count,
        EstimatorKind::Regression,
    )
}

/// 计算两个角度之间的差值并规范化到 [-π, π]。
fn angle_difference(a: f64, b: f64) -> f64 {
    wrap_angle(a - b)
}

/// 将任意角度映射到区间 [-π, π]，以方便角度差值计算和显示。
fn wrap_angle(theta: f64) -> f64 {
    let shifted = (theta + PI).rem_euclid(TWO_PI);
    shifted - PI
}

/// 在终端以可读格式打印单个偏心率的模拟报告（包括 λ 的估计结果）。
fn print_report(report: &EccentricityReport) {
    println!("====================");
    println!("偏心率 e = {:.4}", report.eccentricity);
    match report.lyapunov {
        Some(lambda) => println!(
            "估计的 Lyapunov 指数 λ ≈ {:.6} ({}，样本数 = {})",
            lambda,
            report.method.label(),
            report.sample_count
        ),
        None => println!(
            "未能在设定区间内估计 Lyapunov 指数 ({}，样本数 = {})",
            report.method.label(),
            report.sample_count
        ),
    }
    println!("轨迹样本点数量 = {}", report.result.times.len());
}

/// 对多组偏心率的计算结果进行汇总并在终端打印一个表格式摘要（偏心率 vs λ）。
fn summarize_reports(reports: &[EccentricityReport]) {
    if reports.len() <= 1 {
        return;
    }
    println!("====================");
    println!("偏心率与 Lyapunov 指数汇总：");
    for report in reports {
        match report.lyapunov {
            Some(lambda) => println!(
                "e = {:.3}: λ ≈ {:.5} ({})",
                report.eccentricity,
                lambda,
                report.method.label()
            ),
            None => println!(
                "e = {:.3}: 无法估计 λ ({}, 样本数 = {})",
                report.eccentricity,
                report.method.label(),
                report.sample_count
            ),
        }
    }
}

/// 将每个偏心率的 Δθ(t) 曲线和偏心率-λ 汇总图保存为 Plotly 生成的 HTML 文件到 `output_dir`。
///
/// 若 `reports` 为空则直接返回 Ok。
fn write_plots(reports: &[EccentricityReport], output_dir: &str) -> Result<()> {
    if reports.is_empty() {
        return Ok(());
    }
    let dir = Path::new(output_dir);
    fs::create_dir_all(dir)?;

    for report in reports {
        let file_name = format!("delta_e{}.html", sanitize_ecc(report.eccentricity));
        let path = dir.join(file_name);
        write_delta_plot(report, &path)?;
        println!("Δθ - 时间 图已输出: {}", path.display());
    }

    match write_lyapunov_plot(reports, dir)? {
        Some(path) => println!("Lyapunov - 偏心率 散点图已输出: {}", path.display()),
        None => println!("缺少有效的 Lyapunov 数据，跳过汇总散点图输出。"),
    }

    Ok(())
}

/// 为单个偏心率绘制 |Δθ|(t) 的对数坐标图并保存为 HTML。
///
/// - 使用 Plotly 的对数 y 轴以便观察指数增长段。
/// - 若模拟结果为空则返回 Err。
fn write_delta_plot(report: &EccentricityReport, path: &Path) -> Result<()> {
    if report.result.times.is_empty() {
        return Err(anyhow!("模拟结果为空，无法绘制 Δθ 曲线"));
    }

    let y_values: Vec<f64> = report
        .result
        .delta_theta
        .iter()
        .map(|v| v.max(1e-12))
        .collect();
    let trace = Scatter::new(report.result.times.clone(), y_values)
        .mode(Mode::Lines)
        .name("|Δθ|");

    let mut plot = Plot::new();
    plot.add_trace(trace);

    let title = match report.lyapunov {
        Some(lambda) => format!("Δθ(t) e = {:.3}, λ ≈ {:.4}", report.eccentricity, lambda),
        None => format!("Δθ(t) e = {:.3}", report.eccentricity),
    };

    let layout = Layout::new()
        .title(Title::with_text(&title))
        .x_axis(Axis::new().title(Title::with_text("时间 (Hyperion 年)")))
        .y_axis(
            Axis::new()
                .title(Title::with_text("|Δθ|"))
                .type_(AxisType::Log),
        );
    plot.set_layout(layout);

    save_plot(&plot, path)
}

/// 绘制偏心率 vs Lyapunov 指数的散点图并保存为 HTML。
///
/// 仅使用那些成功估计到 λ 的结果；若没有有效数据返回 Ok(None)。
fn write_lyapunov_plot(reports: &[EccentricityReport], dir: &Path) -> Result<Option<PathBuf>> {
    let mut eccs = Vec::new();
    let mut lambdas = Vec::new();

    for report in reports {
        if let Some(lambda) = report.lyapunov {
            eccs.push(report.eccentricity);
            lambdas.push(lambda);
        }
    }

    if eccs.is_empty() {
        return Ok(None);
    }

    let scatter = Scatter::new(eccs, lambdas)
        .mode(Mode::Markers)
        .name("Lyapunov 指数");
    let mut plot = Plot::new();
    plot.add_trace(scatter);
    let layout = Layout::new()
        .title(Title::with_text("Lyapunov 指数随偏心率变化"))
        .x_axis(Axis::new().title(Title::with_text("偏心率 e")))
        .y_axis(Axis::new().title(Title::with_text("Lyapunov 指数 λ")));
    plot.set_layout(layout);

    let path = dir.join("lyapunov_vs_eccentricity.html");
    save_plot(&plot, &path)?;
    Ok(Some(path))
}

/// 将 Plotly `Plot` 对象序列化为 HTML 并写入磁盘。
///
/// 输入：`plot` 和目标文件路径；若写入失败返回 Err。
fn save_plot(plot: &Plot, path: &Path) -> Result<()> {
    let html = plot.to_html();
    fs::write(path, html)?;
    Ok(())
}

/// 将偏心率格式化为用于文件名的字符串（例如 0.200 -> "0p200"），避免文件名中的小数点问题。
fn sanitize_ecc(value: f64) -> String {
    let formatted = format!("{:.3}", value);
    formatted.replace('.', "p")
}
