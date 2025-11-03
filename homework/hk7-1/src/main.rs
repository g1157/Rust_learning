use plotly::common::{Mode, Title};
use plotly::layout::{Axis, Layout};
use plotly::{common::Line, Plot, Scatter};
use rand::prelude::*;
use std::env;
use std::f64::consts::PI;
use std::fmt;

const DT: f64 = 0.01;
const TOTAL_TIME: f64 = 500.0;
const MAX_STEP_COLLISIONS: usize = 4; // 限制单步碰撞次数，防止在墙面附近无限反弹
const COLLISION_EPS: f64 = 1e-9;
const N_SAMPLES_BOUNDARY: usize = 720; // 边界曲线采样数量，决定绘图分辨率

#[derive(Clone, Copy, Debug)]
enum ShapeKind {
    Circle,
    Ellipse,
    RoundedTriangle,
    Star,
    Bean,
}

impl ShapeKind {
    fn parse(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "circle" | "圆" => Some(Self::Circle),
            "ellipse" | "椭圆" => Some(Self::Ellipse),
            "triangle" | "rounded_triangle" | "三角形" => Some(Self::RoundedTriangle),
            "star" | "星形" => Some(Self::Star),
            "bean" | "肾形" | "心豆" => Some(Self::Bean),
            _ => None,
        }
    }

    fn radius(&self, theta: f64) -> f64 {
        match self {
            ShapeKind::Circle => 1.0,
            ShapeKind::Ellipse => {
                let a = 1.3;
                let b = 0.8;
                (a * b) / ((b * theta.cos()).powi(2) + (a * theta.sin()).powi(2)).sqrt()
            }
            ShapeKind::RoundedTriangle => {
                let k = 0.35;
                let base = 1.05;
                base * (1.0 + k * (3.0 * theta).cos())
            }
            ShapeKind::Star => {
                let base = 0.85;
                let amplitude = 0.3;
                base * (1.0 + amplitude * (5.0 * theta).cos())
            }
            ShapeKind::Bean => {
                let base = 0.9;
                let skew = 0.35;
                base + skew * theta.sin()
            }
        }
    }

    fn radius_derivative(&self, theta: f64) -> f64 {
        let h = 1e-4;
        let forward = self.radius(theta + h);
        let backward = self.radius(theta - h);
        (forward - backward) / (2.0 * h)
    }
}

impl fmt::Display for ShapeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ShapeKind::Circle => "circle",
            ShapeKind::Ellipse => "ellipse",
            ShapeKind::RoundedTriangle => "rounded_triangle",
            ShapeKind::Star => "star",
            ShapeKind::Bean => "bean",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone)]
struct SimulationParams {
    boundary: ShapeKind,
    dt: f64,
    steps: usize,
    initial_pos: (f64, f64),
    initial_vel: (f64, f64),
}

#[derive(Debug, Clone)]
struct CollisionRecord {
    theta: f64,
    sin_chi: f64,
}

#[derive(Debug, Default, Clone)]
struct SimulationResult {
    positions: Vec<(f64, f64)>,
    velocities: Vec<(f64, f64)>,
    collisions: Vec<CollisionRecord>,
}

/// 程序入口：解析命令行参数并执行模拟与绘图输出。
fn main() {

    let args: Vec<String> = env::args().collect();
    // 终端调用示例：hk7-1 <shape>，若缺省或无法识别则回退到椭圆边界。
    let shape = args
        .get(1)
        .and_then(|s| ShapeKind::parse(s))
        .unwrap_or(ShapeKind::Ellipse);

    let mut rng = StdRng::seed_from_u64(2025);
    let direction_angle: f64 = rng.random_range(0.0..(2.0 * PI));
    let speed = 0.85;

    let initial_pos = (0.15, 0.0);
    let initial_vel = (speed * direction_angle.cos(), speed * direction_angle.sin());

    let params = SimulationParams {
        boundary: shape,
        dt: DT,
        steps: (TOTAL_TIME / DT) as usize,
        initial_pos,
        initial_vel,
    };

    println!("=== Billiard Simulation ===");
    println!("Boundary shape: {}", params.boundary);
    println!("Total steps: {}", params.steps);
    println!("Initial position: ({:.3}, {:.3})", params.initial_pos.0, params.initial_pos.1);
    println!(
        "Initial velocity: ({:.3}, {:.3}) |speed| = {:.3}",
        params.initial_vel.0,
        params.initial_vel.1,
        (params.initial_vel.0.powi(2) + params.initial_vel.1.powi(2)).sqrt()
    );

    let result = simulate(&params);

    if result.positions.is_empty() {
        eprintln!("Simulation produced no data");
        return;
    }

    if let Err(err) = render_trajectory_plot(&params, &result) {
        eprintln!("Failed to write trajectory plot: {err}");
    }
    if let Err(err) = render_phase_plot(&params, &result) {
        eprintln!("Failed to write phase-space plot: {err}");
    }
    if let Err(err) = render_poincare_plot(&params, &result) {
        eprintln!("Failed to write attractor plot: {err}");
    }
}

/// 运行台球模拟主循环，返回轨迹、速度及碰撞记录。
fn simulate(params: &SimulationParams) -> SimulationResult {
    let mut pos = params.initial_pos;
    if signed_distance(&params.boundary, pos) > 0.0 {
        eprintln!("Initial position is outside the boundary");
        return SimulationResult::default();
    }

    let mut vel = params.initial_vel;
    let mut result = SimulationResult::default();
    let mut time = 0.0;

    // 使用简单欧拉积分推进运动，遇到越界时在单步内迭代查找碰撞。
    for _ in 0..params.steps {
        let mut remaining = params.dt;
        let mut step_collisions = 0;

        while remaining > COLLISION_EPS {
            // 优先尝试完整推进子步长，如越界则进行碰撞处理。
            let next_pos = (pos.0 + vel.0 * remaining, pos.1 + vel.1 * remaining);
            if signed_distance(&params.boundary, next_pos) <= 0.0 {
                pos = next_pos;
                time += remaining;
                remaining = 0.0;
            } else if let Some((collision_point, time_fraction)) =
                find_collision(pos, vel, remaining, &params.boundary)
            {
                    let normal = boundary_normal(&params.boundary, collision_point);
                    let incoming_speed = (vel.0.powi(2) + vel.1.powi(2)).sqrt();

                    if incoming_speed > 0.0 {
                        let tangent = (-normal.1, normal.0);
                        let sin_chi = (vel.0 * tangent.0 + vel.1 * tangent.1) / incoming_speed;
                        let theta = normalize_angle(collision_point.1.atan2(collision_point.0));
                        result.collisions.push(CollisionRecord { theta, sin_chi });
                    }

                    let reflected_vel = reflect_velocity(vel, normal);
                    vel = reflected_vel;
                    pos = (
                        collision_point.0 + normal.0 * -1e-9,
                        collision_point.1 + normal.1 * -1e-9,
                    );

                    let consumed_time = remaining * time_fraction;
                    remaining -= consumed_time;
                    time += consumed_time;
                    step_collisions += 1;

                    if step_collisions >= MAX_STEP_COLLISIONS {
                        break;
                    }
            } else {
                pos = next_pos;
                time += remaining;
                remaining = 0.0;
            }
        }

        result.positions.push(pos);
        result.velocities.push(vel);
    }

    println!("Simulation finished at t = {:.2}s with {} collisions", time, result.collisions.len());
    result
}

/// 在给定时间步内寻找首次越界的碰撞点及其相对时间。
fn find_collision(
    start: (f64, f64),
    velocity: (f64, f64),
    dt: f64,
    boundary: &ShapeKind,
) -> Option<((f64, f64), f64)> {
    // 在当前时间步内用二分法寻找首次越界时刻，返回碰撞点与时间占比。
    let mut low = 0.0;
    let mut high = 1.0;
    let mut contains_collision = false;

    let start_dist = signed_distance(boundary, start);
    if start_dist > 0.0 {
        return None;
    }

    let mut end_point = (
        start.0 + velocity.0 * dt,
        start.1 + velocity.1 * dt,
    );

    if signed_distance(boundary, end_point) <= 0.0 {
        return None;
    }

    for _ in 0..52 {
        let mid = 0.5 * (low + high);
        let probe = (
            start.0 + velocity.0 * dt * mid,
            start.1 + velocity.1 * dt * mid,
        );
        let dist = signed_distance(boundary, probe);
        if dist > 0.0 {
            high = mid;
            end_point = probe;
            contains_collision = true;
        } else {
            low = mid;
        }
    }

    if contains_collision {
        let collision_fraction = high;
        Some((end_point, collision_fraction))
    } else {
        None
    }
}

/// 计算点到边界的符号距离，用于判断是否越界。
fn signed_distance(boundary: &ShapeKind, point: (f64, f64)) -> f64 {
    // 返回点到边界的符号距离：负值表示在边界内，正值表示越界。
    let r = (point.0.powi(2) + point.1.powi(2)).sqrt();
    if r == 0.0 {
        return -boundary.radius(0.0);
    }
    let theta = point.1.atan2(point.0);
    let boundary_radius = boundary.radius(theta);
    r - boundary_radius
}

/// 估计边界在碰撞点的外法向，用于镜面反射。
fn boundary_normal(boundary: &ShapeKind, point: (f64, f64)) -> (f64, f64) {
    // 计算隐式极坐标曲线的梯度，得到用于镜面反射的外法向。
    let r = (point.0.powi(2) + point.1.powi(2)).sqrt();
    let theta = point.1.atan2(point.0);

    if r <= 1e-9 {
        return (point.0.signum(), point.1.signum());
    }

    let dr_dtheta = boundary.radius_derivative(theta);

    let grad_x = point.0 / r + dr_dtheta * point.1 / (r * r);
    let grad_y = point.1 / r - dr_dtheta * point.0 / (r * r);

    let magnitude = (grad_x.powi(2) + grad_y.powi(2)).sqrt();
    if magnitude == 0.0 {
        let radial = (point.0 / r, point.1 / r);
        return radial;
    }
    (grad_x / magnitude, grad_y / magnitude)
}

/// 基于外法向反射速度，实现理想弹性碰撞。
fn reflect_velocity(velocity: (f64, f64), normal: (f64, f64)) -> (f64, f64) {
    let dot = velocity.0 * normal.0 + velocity.1 * normal.1;
    (
        velocity.0 - 2.0 * dot * normal.0,
        velocity.1 - 2.0 * dot * normal.1,
    )
}

/// 采样边界轮廓点，用于可视化绘制。
fn sample_boundary(boundary: &ShapeKind, samples: usize) -> Vec<(f64, f64)> {
    let mut points = Vec::with_capacity(samples + 1);
    for i in 0..=samples {
        let theta = 2.0 * PI * (i as f64) / (samples as f64);
        let radius = boundary.radius(theta);
        points.push((radius * theta.cos(), radius * theta.sin()));
    }
    points
}

/// 绘制粒子在边界内的运动轨迹并输出 HTML。
fn render_trajectory_plot(
    params: &SimulationParams,
    result: &SimulationResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let boundary_points = sample_boundary(&params.boundary, N_SAMPLES_BOUNDARY);
    let boundary_trace = Scatter::new(
        boundary_points.iter().map(|p| p.0).collect(),
        boundary_points.iter().map(|p| p.1).collect(),
    )
    .mode(Mode::Lines)
    .line(Line::new().color("#222222").width(3.0))
    .name("Boundary");

    let trajectory = Scatter::new(
        result.positions.iter().map(|p| p.0).collect(),
        result.positions.iter().map(|p| p.1).collect(),
    )
    .mode(Mode::Lines)
    .line(Line::new().color("#1f77b4").width(1.5))
    .name("Trajectory");

    let layout = Layout::new()
        .title(Title::new("Billiard Trajectory"))
    .x_axis(Axis::new().title(Title::new("x")))
        .y_axis(Axis::new().title(Title::new("y")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(boundary_trace);
    plot.add_trace(trajectory);
    let file_name = format!("{}_trajectory.html", params.boundary);
    plot.write_html(&file_name);
    println!("Saved trajectory plot to {file_name}");
    Ok(())
}

/// 绘制相空间散点图 (位置-速度)，展示动力学结构。
fn render_phase_plot(
    params: &SimulationParams,
    result: &SimulationResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let sample_stride = 3;
    // 相空间点采取步长抽样，避免图像过大同时保留结构信息。
    let reduced_positions: Vec<(f64, f64)> = result
        .positions
        .iter()
        .zip(result.velocities.iter())
        .step_by(sample_stride)
        .map(|(p, v)| (p.0, v.0))
        .collect();

    let reduced_positions_y: Vec<(f64, f64)> = result
        .positions
        .iter()
        .zip(result.velocities.iter())
        .step_by(sample_stride)
        .map(|(p, v)| (p.1, v.1))
        .collect();

    let phase_x = Scatter::new(
        reduced_positions.iter().map(|p| p.0).collect(),
        reduced_positions.iter().map(|p| p.1).collect(),
    )
    .mode(Mode::Markers)
    .name("x vs v_x")
    .marker(plotly::common::Marker::new().size(4).color("#d62728"));

    let phase_y = Scatter::new(
        reduced_positions_y.iter().map(|p| p.0).collect(),
        reduced_positions_y.iter().map(|p| p.1).collect(),
    )
    .mode(Mode::Markers)
    .name("y vs v_y")
    .marker(plotly::common::Marker::new().size(4).color("#2ca02c"));

    let layout = Layout::new()
        .title(Title::new("Phase Space"))
        .x_axis(Axis::new().title(Title::new("Coordinate")))
        .y_axis(Axis::new().title(Title::new("Velocity")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(phase_x);
    plot.add_trace(phase_y);
    let file_name = format!("{}_phase_space.html", params.boundary);
    plot.write_html(&file_name);
    println!("Saved phase-space plot to {file_name}");
    Ok(())
}

/// 绘制庞加莱截面，揭示碰撞角与入射角关系。
fn render_poincare_plot(
    params: &SimulationParams,
    result: &SimulationResult,
) -> Result<(), Box<dyn std::error::Error>> {
    if result.collisions.is_empty() {
        println!("No collisions recorded; skipping attractor plot");
        return Ok(());
    }

    // 构建庞加莱截面：碰撞角 theta 与入射角 sin(χ) 可展现吸引子形态。
    let scatter = Scatter::new(
        result
            .collisions
            .iter()
            .map(|c| c.theta)
            .collect::<Vec<f64>>(),
        result
            .collisions
            .iter()
            .map(|c| c.sin_chi)
            .collect::<Vec<f64>>(),
    )
    .mode(Mode::Markers)
    .name("Poincaré Section")
    .marker(plotly::common::Marker::new().size(4).color("#ff7f0e"));

    let layout = Layout::new()
        .title(Title::new("Billiard Attractor (Poincaré Section)"))
        .x_axis(
            Axis::new()
                .title(Title::new("Boundary angle θ (rad)"))
                .range(vec![0.0, 2.0 * PI]),
        )
        .y_axis(Axis::new().title(Title::new("sin(χ)")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(scatter);
    let file_name = format!("{}_attractor.html", params.boundary);
    plot.write_html(&file_name);
    println!("Saved attractor plot to {file_name}");
    Ok(())
}

/// 将角度归一化到 [0, 2π) 区间，便于统计。
fn normalize_angle(theta: f64) -> f64 {
    let mut angle = theta % (2.0 * PI);
    if angle < 0.0 {
        angle += 2.0 * PI;
    }
    angle
}