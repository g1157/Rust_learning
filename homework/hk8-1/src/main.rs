use plotly::{Plot, Scatter, Layout};
use plotly::common::{Mode, Title};

// 模型参数：根据 README 使用 GM 与转动惯量 I
struct Params {
    gm: f64,    // G M_sat = 4 π^2 (单位化)
    i_moi: f64, // 转动惯量 I（单位化取 1）
}

#[derive(Clone, Copy)]
struct State {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    theta: f64,
    omega: f64,
}

// Euler-Cromer 步进：先更新速度和角速度，再更新位置与角度
fn euler_cromer_step(mut s: State, dt: f64, p: &Params) -> State {
    let r2 = s.x * s.x + s.y * s.y;
    let r = r2.sqrt();
    let r3 = r2 * r;
    // 平动加速度（引力）
    let ax = -p.gm * s.x / r3;
    let ay = -p.gm * s.y / r3;
    // 角加速度 (公式 4.24)：dω/dt = -3 GM /(r^5 I) (x sinθ - y cosθ)(x cosθ + y sinθ)
    let r5 = r3 * r2;
    let sin_th = s.theta.sin();
    let cos_th = s.theta.cos();
    let torque_factor = -3.0 * p.gm / (r5 * p.i_moi);
    let domega = torque_factor * (s.x * sin_th - s.y * cos_th) * (s.x * cos_th + s.y * sin_th);

    // 先更新速度与角速度
    s.vx += dt * ax;
    s.vy += dt * ay;
    s.omega += dt * domega;

    // 再更新位置与角度
    s.x += dt * s.vx;
    s.y += dt * s.vy;
    s.theta += dt * s.omega;

    // 角度 wrap 到 [-π, π]
    while s.theta > std::f64::consts::PI { s.theta -= 2.0 * std::f64::consts::PI; }
    while s.theta <= -std::f64::consts::PI { s.theta += 2.0 * std::f64::consts::PI; }
    s
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > std::f64::consts::PI { d -= 2.0 * std::f64::consts::PI; }
    while d <= -std::f64::consts::PI { d += 2.0 * std::f64::consts::PI; }
    d
}

// 初始化：圆轨道 (e=0) r=1, v = sqrt(GM/r) = 2π
fn init_circular(gm: f64) -> State {
    let r0 = 1.0;
    let v = (gm / r0).sqrt();
    State { x: r0, y: 0.0, vx: 0.0, vy: v, theta: 0.0, omega: 0.0 }
}

// 初始化：椭圆轨道近拱点 (给定 a=1, e)。近拱点 r_p = a(1-e), v_p = sqrt( GM (1+e)/(a(1-e)) )
fn init_ellipse(gm: f64, e: f64) -> State {
    let a = 1.0;
    let rp = a * (1.0 - e);
    let vp = (gm * (1.0 + e) / (a * (1.0 - e))).sqrt();
    State { x: rp, y: 0.0, vx: 0.0, vy: vp, theta: 0.0, omega: 0.0 }
}

fn simulate_and_divergence(gm: f64, e: f64, t_max: f64, dt: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let params = Params { gm, i_moi: 1.0 };
    let mut t = 0.0;
    let base = if e.abs() < 1e-12 { init_circular(gm) } else { init_ellipse(gm, e) };
    let mut s1 = base;
    let mut s2 = State { theta: base.theta + 0.01, ..base };
    let mut times = Vec::new();
    let mut dtheta_abs = Vec::new();
    let mut log_dtheta = Vec::new();

    while t <= t_max {
        let dth = angle_diff(s2.theta, s1.theta).abs();
        let ld = (dth + 1e-16).ln();
        times.push(t);
        dtheta_abs.push(dth);
        log_dtheta.push(ld);
        s1 = euler_cromer_step(s1, dt, &params);
        s2 = euler_cromer_step(s2, dt, &params);
        t += dt;
    }

    (times, dtheta_abs, log_dtheta)
}

fn linear_regression_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (sx, sy, sxx, sxy) = x.iter().zip(y.iter()).fold((0.0, 0.0, 0.0, 0.0), |acc, (&xi, &yi)| {
        (acc.0 + xi, acc.1 + yi, acc.2 + xi*xi, acc.3 + xi*yi)
    });
    (n * sxy - sx * sy) / (n * sxx - sx * sx)
}

fn estimate_lyapunov(times: &[f64], logdiff: &[f64]) -> f64 {
    // 丢弃初始 10% 作为瞬态，末尾 20% 可能饱和
    let n = times.len();
    let i0 = (n as f64 * 0.1) as usize;
    let i1 = (n as f64 * 0.8) as usize;
    if i1 <= i0 + 1 { return 0.0; }
    let (x, y) = (&times[i0..i1], &logdiff[i0..i1]);
    linear_regression_slope(x, y)
}

fn plot_divergence_linear_html(times: &[f64], dth_circ: &[f64], dth_ellip: &[f64], out: &str) {
    let mut plot = Plot::new();

    let trace1 = Scatter::new(times.to_vec(), dth_circ.to_vec())
        .mode(Mode::Lines)
        .name("圆轨道 e=0");
    let trace2 = Scatter::new(times.to_vec(), dth_ellip.to_vec())
        .mode(Mode::Lines)
        .name("椭圆轨道 e=0.3");
    plot.add_trace(trace1);
    plot.add_trace(trace2);

    let layout = Layout::new().title(Title::with_text("|Δθ(t)|: 圆轨道 vs 椭圆轨道"));
    plot.set_layout(layout);
    plot.write_html(out);
}

fn plot_lambda_vs_e_html(es: &[f64], lambdas: &[f64], out: &str) {
    let mut plot = Plot::new();
    let trace = Scatter::new(es.to_vec(), lambdas.to_vec()).mode(Mode::Markers).name("λ(e)");
    plot.add_trace(trace);
    let layout = Layout::new().title(Title::with_text("Lyapunov 指数 λ(e)"));
    plot.set_layout(layout);
    plot.write_html(out);
}

fn plot_divergence_linear_single(times: &[f64], dth: &[f64], title: &str, out: &str) {
    let mut plot = Plot::new();
    let trace = Scatter::new(times.to_vec(), dth.to_vec()).mode(Mode::Lines).name("|Δθ|");
    plot.add_trace(trace);
    let layout = Layout::new().title(Title::with_text(title));
    plot.set_layout(layout);
    plot.write_html(out);
}

fn plot_divergence_log_single(times: &[f64], logd: &[f64], title: &str, out: &str) {
    let mut plot = Plot::new();
    let trace = Scatter::new(times.to_vec(), logd.to_vec()).mode(Mode::Lines).name("ln|Δθ|");
    plot.add_trace(trace);
    let layout = Layout::new().title(Title::with_text(title));
    plot.set_layout(layout);
    plot.write_html(out);
}

fn plot_divergence_log_compare(times: &[f64], logs_circ: &[f64], logs_ellip: &[f64], out: &str) {
    let mut plot = Plot::new();

    let trace1 = Scatter::new(times.to_vec(), logs_circ.to_vec())
        .mode(Mode::Lines)
        .name("圆轨道 e=0");
    let trace2 = Scatter::new(times.to_vec(), logs_ellip.to_vec())
        .mode(Mode::Lines)
        .name("椭圆轨道 e=0.3");
    plot.add_trace(trace1);
    plot.add_trace(trace2);

    let layout = Layout::new().title(Title::with_text("ln|Δθ(t)|: 圆轨道 vs 椭圆轨道"));
    plot.set_layout(layout);
    plot.write_html(out);
}

fn main() {
    // 默认参数（可按需调整）
    let dt = 1e-5;          // 年 (README 建议 1e-4，可根据性能调节)
    let t_max = 10.0;       // 年（演示用，可增大到 100）
    let gm = 4.0 * std::f64::consts::PI * std::f64::consts::PI; // G M_sat

    // 圆轨道 e=0 与 椭圆 e=0.3 基础对比
    let (t1, d1_abs, logd1) = simulate_and_divergence(gm, 0.0, t_max, dt);
    let (t2, d2_abs, logd2) = simulate_and_divergence(gm, 0.3, t_max, dt);

    // 使用对数发散估计 Lyapunov
    let lambda_circ = estimate_lyapunov(&t1, &logd1);
    let lambda_ellip = estimate_lyapunov(&t2, &logd2);
    println!("lambda(e=0)   ≈ {:.4}", lambda_circ);
    println!("lambda(e=0.3) ≈ {:.4}", lambda_ellip);

    // 单独线性与对数图（圆轨道）
    plot_divergence_linear_single(&t1, &d1_abs, "|Δθ|(t): e=0", "dtheta_linear_e0.html");
    plot_divergence_log_single(&t1, &logd1, "ln|Δθ|(t): e=0", "dtheta_log_e0.html");

    // 单独线性与对数图（椭圆轨道）
    plot_divergence_linear_single(&t2, &d2_abs, "|Δθ|(t): e=0.3", "dtheta_linear_e03.html");
    plot_divergence_log_single(&t2, &logd2, "ln|Δθ|(t): e=0.3", "dtheta_log_e03.html");

    // 基础两轨道对比图（线性与对数）
    plot_divergence_linear_html(&t1, &d1_abs, &d2_abs, "divergence_compare_linear.html");
    plot_divergence_log_compare(&t1, &logd1, &logd2, "divergence_compare_log.html");

    // 多个 e 扫描，收集发散曲线并分类
    let es = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    let mut lambdas = Vec::new();
    let mut linear_series: Vec<(f64, Vec<f64>)> = Vec::new();
    let mut log_series: Vec<(f64, Vec<f64>)> = Vec::new();
    let mut times_ref: Option<Vec<f64>> = None;
    for &e in &es {
        let (tt, dabs, llogs) = simulate_and_divergence(gm, e, t_max, dt);
        if times_ref.is_none() { times_ref = Some(tt.clone()); }
        let lam = estimate_lyapunov(&tt, &llogs);
        lambdas.push(lam);
        linear_series.push((e, dabs));
        log_series.push((e, llogs));
    }
    plot_lambda_vs_e_html(&es, &lambdas, "lambda_vs_e.html");

    // 设定混沌阈值（Lyapunov > 0.1 视为混沌）
    let chaos_threshold = 0.1;
    let times = times_ref.unwrap();
    // 构建多轨迹对比（混沌集）
    {
        let mut plot_lin = Plot::new();
        let mut plot_log = Plot::new();
        for (i, &e) in es.iter().enumerate() {
            let lam = lambdas[i];
            if lam > chaos_threshold {
                let lin_trace = Scatter::new(times.clone(), linear_series[i].1.clone()).mode(Mode::Lines).name(format!("e={:.1} λ={:.3}", e, lam));
                let log_trace = Scatter::new(times.clone(), log_series[i].1.clone()).mode(Mode::Lines).name(format!("e={:.1} λ={:.3}", e, lam));
                plot_lin.add_trace(lin_trace);
                plot_log.add_trace(log_trace);
            }
        }
        plot_lin.set_layout(Layout::new().title(Title::with_text(format!("|Δθ|(t) 混沌轨道对比 (阈值 {:.2})", chaos_threshold))));
        plot_log.set_layout(Layout::new().title(Title::with_text(format!("ln|Δθ|(t) 混沌轨道对比 (阈值 {:.2})", chaos_threshold))));
        plot_lin.write_html("chaotic_group_linear.html");
        plot_log.write_html("chaotic_group_log.html");
    }
    // 非混沌轨道组
    {
        let mut plot_lin = Plot::new();
        let mut plot_log = Plot::new();
        for (i, &e) in es.iter().enumerate() {
            let lam = lambdas[i];
            if lam <= chaos_threshold {
                let lin_trace = Scatter::new(times.clone(), linear_series[i].1.clone()).mode(Mode::Lines).name(format!("e={:.1} λ={:.3}", e, lam));
                let log_trace = Scatter::new(times.clone(), log_series[i].1.clone()).mode(Mode::Lines).name(format!("e={:.1} λ={:.3}", e, lam));
                plot_lin.add_trace(lin_trace);
                plot_log.add_trace(log_trace);
            }
        }
        plot_lin.set_layout(Layout::new().title(Title::with_text(format!("|Δθ|(t) 非混沌轨道对比 (阈值 {:.2})", chaos_threshold))));
        plot_log.set_layout(Layout::new().title(Title::with_text(format!("ln|Δθ|(t) 非混沌轨道对比 (阈值 {:.2})", chaos_threshold))));
        plot_lin.write_html("nonchaotic_group_linear.html");
        plot_log.write_html("nonchaotic_group_log.html");
    }

    println!("输出: 单轨道、基础对比、混沌/非混沌分组图已生成");
}
