use std::ops::{Add, Div, Mul, Sub};

/// 三维向量结构
#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    /// 创建新向量
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// 零向量
    fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

    /// 计算向量模长
    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// 计算向量平方模长（避免开方，提高性能）
    fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 归一化向量
    #[allow(dead_code)]
    fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            *self / mag
        } else {
            Vec3::zero()
        }
    }

    /// 点积
    #[allow(dead_code)]
    fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 叉积
    #[allow(dead_code)]
    fn cross(&self, other: &Vec3) -> Self {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

// 实现向量加法
impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

// 实现向量减法
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// 实现向量与标量乘法
impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

// 实现向量与标量除法
impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, scalar: f64) -> Vec3 {
        Vec3::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

/// 天体结构
#[derive(Debug, Clone, Copy)]
struct Body {
    mass: f64,
    position: Vec3,
    velocity: Vec3,
}

impl Body {
    /// 创建新天体
    fn new(mass: f64, position: Vec3, velocity: Vec3) -> Self {
        Body {
            mass,
            position,
            velocity,
        }
    }

    /// 计算动能
    fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.magnitude_squared()
    }
}

/// 三体系统
struct ThreeBodySystem {
    bodies: [Body; 3],
    g: f64, // 引力常数
    time: f64,
    trajectory: Vec<[Vec3; 3]>, // 记录轨迹
}

impl ThreeBodySystem {
    /// 创建新的三体系统
    fn new(bodies: [Body; 3], g: f64) -> Self {
        ThreeBodySystem {
            bodies,
            g,
            time: 0.0,
            trajectory: vec![[bodies[0].position, bodies[1].position, bodies[2].position]],
        }
    }

    /// 计算天体i受到的总引力加速度
    fn compute_acceleration(&self, i: usize) -> Vec3 {
        let mut acc = Vec3::zero();
        for j in 0..3 {
            if i != j {
                let r = self.bodies[j].position - self.bodies[i].position;
                let r_mag = r.magnitude();
                // 避免除零和数值不稳定
                if r_mag > 1e-10 {
                    // F = G * m1 * m2 / r^2, a = F / m1 = G * m2 / r^2
                    let force_magnitude = self.g * self.bodies[j].mass / (r_mag * r_mag * r_mag);
                    acc = acc + r * force_magnitude;
                }
            }
        }
        acc
    }

    /// 使用Velocity Verlet算法进行时间步进
    fn step(&mut self, dt: f64) {
        // 1. 计算当前加速度
        let accelerations: Vec<Vec3> = (0..3).map(|i| self.compute_acceleration(i)).collect();

        // 2. 更新位置和速度的一半
        let mut new_bodies = self.bodies;
        for i in 0..3 {
            new_bodies[i].position = new_bodies[i].position
                + new_bodies[i].velocity * dt
                + accelerations[i] * (0.5 * dt * dt);
            new_bodies[i].velocity = new_bodies[i].velocity + accelerations[i] * (0.5 * dt);
        }

        // 3. 临时更新bodies以计算新加速度
        let old_bodies = self.bodies;
        self.bodies = new_bodies;

        // 4. 计算新加速度
        let new_accelerations: Vec<Vec3> = (0..3).map(|i| self.compute_acceleration(i)).collect();

        // 5. 完成速度更新
        for i in 0..3 {
            self.bodies[i].velocity =
                old_bodies[i].velocity + (accelerations[i] + new_accelerations[i]) * (0.5 * dt);
        }

        self.time += dt;
    }

    /// 记录当前位置到轨迹
    fn record_trajectory(&mut self) {
        self.trajectory.push([
            self.bodies[0].position,
            self.bodies[1].position,
            self.bodies[2].position,
        ]);
    }

    /// 计算系统总动能
    fn kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }

    /// 计算系统总势能
    fn potential_energy(&self) -> f64 {
        let mut pe = 0.0;
        for i in 0..3 {
            for j in (i + 1)..3 {
                let r = (self.bodies[i].position - self.bodies[j].position).magnitude();
                if r > 1e-10 {
                    pe -= self.g * self.bodies[i].mass * self.bodies[j].mass / r;
                }
            }
        }
        pe
    }

    /// 计算系统总能量
    fn total_energy(&self) -> f64 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// 运行模拟
    fn simulate(&mut self, total_time: f64, dt: f64, record_interval: usize) {
        let steps = (total_time / dt) as usize;
        for step in 0..steps {
            self.step(dt);
            if step % record_interval == 0 {
                self.record_trajectory();
            }
        }
    }
}

/// 创建拉格朗日三角形稳定配置
/// 三个质量相等的天体在等边三角形的顶点上做圆周运动
fn create_lagrange_triangle(mass: f64, radius: f64) -> [Body; 3] {
    // 计算圆周运动所需的速度
    // 对于拉格朗日三角形: v = sqrt(G * M / (2 * R))
    let g = 1.0;
    let v = (g * mass / (2.0 * radius)).sqrt();

    [
        Body::new(mass, Vec3::new(radius, 0.0, 0.0), Vec3::new(0.0, v, 0.0)),
        Body::new(
            mass,
            Vec3::new(radius * (-0.5), radius * (3.0_f64.sqrt() / 2.0), 0.0),
            Vec3::new(-v * (3.0_f64.sqrt() / 2.0), -v * 0.5, 0.0),
        ),
        Body::new(
            mass,
            Vec3::new(radius * (-0.5), radius * (-3.0_f64.sqrt() / 2.0), 0.0),
            Vec3::new(v * (3.0_f64.sqrt() / 2.0), -v * 0.5, 0.0),
        ),
    ]
}

/// 创建混沌配置（随机初始条件）
fn create_chaotic_config(seed: u64) -> [Body; 3] {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(seed);

    let mut bodies = [Body::new(1.0, Vec3::zero(), Vec3::zero()); 3];

    for body in &mut bodies {
        let mass = 0.8 + rng.gen::<f64>() * 0.4; // 质量在0.8-1.2之间
        let pos = Vec3::new(
            (rng.gen::<f64>() - 0.5) * 2.0,
            (rng.gen::<f64>() - 0.5) * 2.0,
            (rng.gen::<f64>() - 0.5) * 0.5, // z方向较小
        );
        let vel = Vec3::new(
            (rng.gen::<f64>() - 0.5) * 0.5,
            (rng.gen::<f64>() - 0.5) * 0.5,
            (rng.gen::<f64>() - 0.5) * 0.1,
        );
        *body = Body::new(mass, pos, vel);
    }

    bodies
}

/// 对配置施加微小扰动
fn perturb_config(bodies: [Body; 3], perturbation: f64) -> [Body; 3] {
    let mut perturbed = bodies;
    perturbed[0].position.x += perturbation;
    perturbed
}

fn main() {
    println!("=== 三体问题混沌演示 ===\n");

    let g = 1.0;
    let dt = 0.001;
    let total_time = 50.0;
    let record_interval = 10;

    // 1. 稳定配置：拉格朗日三角形
    println!("1. 模拟稳定配置（拉格朗日三角形）");
    let stable_bodies = create_lagrange_triangle(1.0, 1.0);
    let mut stable_system = ThreeBodySystem::new(stable_bodies, g);
    let initial_energy_stable = stable_system.total_energy();
    println!("   初始能量: {:.6}", initial_energy_stable);

    stable_system.simulate(total_time, dt, record_interval);

    let final_energy_stable = stable_system.total_energy();
    let energy_error_stable =
        ((final_energy_stable - initial_energy_stable) / initial_energy_stable).abs() * 100.0;
    println!("   最终能量: {:.6}", final_energy_stable);
    println!("   能量误差: {:.4}%", energy_error_stable);
    println!("   记录轨迹点数: {}", stable_system.trajectory.len());

    // 2. 混沌配置
    println!("\n2. 模拟混沌配置（随机初始条件）");
    let chaotic_bodies = create_chaotic_config(42);
    let mut chaotic_system = ThreeBodySystem::new(chaotic_bodies, g);
    let initial_energy_chaotic = chaotic_system.total_energy();
    println!("   初始能量: {:.6}", initial_energy_chaotic);

    chaotic_system.simulate(total_time, dt, record_interval);

    let final_energy_chaotic = chaotic_system.total_energy();
    let energy_error_chaotic =
        ((final_energy_chaotic - initial_energy_chaotic) / initial_energy_chaotic).abs() * 100.0;
    println!("   最终能量: {:.6}", final_energy_chaotic);
    println!("   能量误差: {:.4}%", energy_error_chaotic);
    println!("   记录轨迹点数: {}", chaotic_system.trajectory.len());

    // 3. 初始条件敏感性测试
    println!("\n3. 初始条件敏感性测试");
    let base_bodies = create_chaotic_config(123);
    let perturbed_bodies = perturb_config(base_bodies, 1e-6);

    let mut system1 = ThreeBodySystem::new(base_bodies, g);
    let mut system2 = ThreeBodySystem::new(perturbed_bodies, g);

    system1.simulate(total_time, dt, record_interval);
    system2.simulate(total_time, dt, record_interval);

    // 计算轨迹差异
    let n = system1.trajectory.len().min(system2.trajectory.len());
    let final_diff = (system1.trajectory[n - 1][0] - system2.trajectory[n - 1][0]).magnitude();

    println!("   初始扰动: 1e-6");
    println!("   最终位置差异: {:.6}", final_diff);
    println!("   放大倍数: {:.2e}", final_diff / 1e-6);

    // 4. 可视化轨道
    println!("\n4. 生成轨道图像");
    plot_trajectory(
        &stable_system,
        "stable_orbit.png",
        "稳定配置：拉格朗日三角形",
    )
    .expect("绘图失败");
    println!("   已保存: stable_orbit.png");

    plot_trajectory(
        &chaotic_system,
        "chaotic_orbit.png",
        "混沌配置：随机初始条件",
    )
    .expect("绘图失败");
    println!("   已保存: chaotic_orbit.png");

    plot_sensitivity_comparison(&system1, &system2, "sensitivity_test.png").expect("绘图失败");
    println!("   已保存: sensitivity_test.png");

    // 5. 导出CSV数据
    println!("\n5. 导出数据文件");
    export_to_csv(&stable_system, "stable_data.csv").expect("导出失败");
    println!("   已保存: stable_data.csv");

    export_to_csv(&chaotic_system, "chaotic_data.csv").expect("导出失败");
    println!("   已保存: chaotic_data.csv");

    println!("\n模拟完成！");
}

/// 导出轨迹数据到CSV文件
fn export_to_csv(
    system: &ThreeBodySystem,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(filename)?;

    // 写入表头
    writeln!(
        file,
        "time,body1_x,body1_y,body1_z,body2_x,body2_y,body2_z,body3_x,body3_y,body3_z"
    )?;

    // 写入数据
    for (i, positions) in system.trajectory.iter().enumerate() {
        let time = i as f64 * 0.001 * 10.0; // dt * record_interval
        write!(file, "{:.6}", time)?;
        for pos in positions {
            write!(file, ",{:.6},{:.6},{:.6}", pos.x, pos.y, pos.z)?;
        }
        writeln!(file)?;
    }

    Ok(())
}

/// 绘制轨道图（XY平面投影）
fn plot_trajectory(
    system: &ThreeBodySystem,
    filename: &str,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    let root = BitMapBackend::new(filename, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // 计算坐标范围
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for positions in &system.trajectory {
        for pos in positions {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }
    }

    let margin = 0.1 * (max_x - min_x).max(max_y - min_y);
    min_x -= margin;
    max_x += margin;
    min_y -= margin;
    max_y += margin;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    // 绘制三个天体的轨迹
    let colors = [&RED, &BLUE, &GREEN];

    for body_idx in 0..3 {
        let trajectory: Vec<(f64, f64)> = system
            .trajectory
            .iter()
            .map(|positions| (positions[body_idx].x, positions[body_idx].y))
            .collect();

        chart
            .draw_series(LineSeries::new(trajectory.clone(), colors[body_idx]))?
            .label(format!("天体 {}", body_idx + 1))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], colors[body_idx]));

        // 标记起点
        if let Some(&(x, y)) = trajectory.first() {
            chart.draw_series(std::iter::once(Circle::new(
                (x, y),
                5,
                colors[body_idx].filled(),
            )))?;
        }
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

/// 绘制敏感性测试对比图
fn plot_sensitivity_comparison(
    system1: &ThreeBodySystem,
    system2: &ThreeBodySystem,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    let root = BitMapBackend::new(filename, (1200, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((1, 2));

    // 绘制两个系统的对比
    for (idx, (system, area)) in [(system1, &areas[0]), (system2, &areas[1])]
        .iter()
        .enumerate()
    {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for positions in &system.trajectory {
            for pos in positions {
                min_x = min_x.min(pos.x);
                max_x = max_x.max(pos.x);
                min_y = min_y.min(pos.y);
                max_y = max_y.max(pos.y);
            }
        }

        let margin = 0.1 * (max_x - min_x).max(max_y - min_y);
        min_x -= margin;
        max_x += margin;
        min_y -= margin;
        max_y += margin;

        let title = if idx == 0 {
            "原始系统"
        } else {
            "扰动系统 (+1e-6)"
        };

        let mut chart = ChartBuilder::on(area)
            .caption(title, ("sans-serif", 25))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

        chart.configure_mesh().draw()?;

        let colors = [&RED, &BLUE, &GREEN];
        for body_idx in 0..3 {
            let trajectory: Vec<(f64, f64)> = system
                .trajectory
                .iter()
                .map(|positions| (positions[body_idx].x, positions[body_idx].y))
                .collect();

            chart.draw_series(LineSeries::new(trajectory, colors[body_idx]))?;
        }
    }

    root.present()?;
    Ok(())
}
