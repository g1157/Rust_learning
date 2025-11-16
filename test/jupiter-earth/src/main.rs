use plotters::prelude::*;
use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;

/// 行星结构体，包含位置和速度
#[derive(Debug, Clone)]
struct Planet {
    name: String,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    mass: f64, // 以太阳质量为单位
}

impl Planet {
    fn new(name: &str, x: f64, y: f64, vx: f64, vy: f64, mass: f64) -> Self {
        Planet {
            name: name.to_string(),
            x,
            y,
            vx,
            vy,
            mass,
        }
    }

    /// 计算行星到原点(太阳)的距离
    fn distance_from_sun(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 计算两个行星之间的距离
    fn distance_to(&self, other: &Planet) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// 太阳系统模拟器
struct SolarSystem {
    earth: Planet,
    jupiter: Planet,
    time: f64,
    dt: f64,
    omega_squared: f64, // 4π²
}

impl SolarSystem {
    fn new(earth: Planet, jupiter: Planet, dt: f64) -> Self {
        SolarSystem {
            earth,
            jupiter,
            time: 0.0,
            dt,
            omega_squared: 4.0 * PI * PI,
        }
    }

    /// 使用 Euler-Cromer 方法更新系统状态
    fn step(&mut self) {
        // 计算当前距离
        let r_earth = self.earth.distance_from_sun();
        let r_jupiter = self.jupiter.distance_from_sun();
        let r_ej = self.earth.distance_to(&self.jupiter);

        // 保存当前位置用于计算
        let xe = self.earth.x;
        let ye = self.earth.y;
        let xj = self.jupiter.x;
        let yj = self.jupiter.y;

        // 更新地球速度 (Euler-Cromer: 先更新速度)
        // v_e,x(i+1) = v_e,x(i) - 4π²*x_e(i)/r_e(i)³*Δt - 4π²*(M_j/M_s)*[x_e(i)-x_j(i)]/r_EJ(i)³*Δt
        let ax_earth_sun = -self.omega_squared * xe / r_earth.powi(3);
        let ax_earth_jupiter = -self.omega_squared * (self.jupiter.mass) * (xe - xj) / r_ej.powi(3);
        self.earth.vx += (ax_earth_sun + ax_earth_jupiter) * self.dt;

        let ay_earth_sun = -self.omega_squared * ye / r_earth.powi(3);
        let ay_earth_jupiter = -self.omega_squared * (self.jupiter.mass) * (ye - yj) / r_ej.powi(3);
        self.earth.vy += (ay_earth_sun + ay_earth_jupiter) * self.dt;

        // 更新木星速度
        // v_j,x(i+1) = v_j,x(i) - 4π²*x_j(i)/r_j(i)³*Δt - 4π²*(M_e/M_s)*[x_j(i)-x_e(i)]/r_EJ(i)³*Δt
        let ax_jupiter_sun = -self.omega_squared * xj / r_jupiter.powi(3);
        let ax_jupiter_earth = -self.omega_squared * (self.earth.mass) * (xj - xe) / r_ej.powi(3);
        self.jupiter.vx += (ax_jupiter_sun + ax_jupiter_earth) * self.dt;

        let ay_jupiter_sun = -self.omega_squared * yj / r_jupiter.powi(3);
        let ay_jupiter_earth = -self.omega_squared * (self.earth.mass) * (yj - ye) / r_ej.powi(3);
        self.jupiter.vy += (ay_jupiter_sun + ay_jupiter_earth) * self.dt;

        // 更新位置 (使用新的速度)
        self.earth.x += self.earth.vx * self.dt;
        self.earth.y += self.earth.vy * self.dt;
        self.jupiter.x += self.jupiter.vx * self.dt;
        self.jupiter.y += self.jupiter.vy * self.dt;

        // 更新时间
        self.time += self.dt;
    }

    /// 运行模拟
    fn simulate(&mut self, steps: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let mut earth_positions = Vec::with_capacity(steps);
        let mut jupiter_positions = Vec::with_capacity(steps);

        // 记录初始位置
        earth_positions.push((self.earth.x, self.earth.y));
        jupiter_positions.push((self.jupiter.x, self.jupiter.y));

        for i in 0..steps {
            self.step();
            earth_positions.push((self.earth.x, self.earth.y));
            jupiter_positions.push((self.jupiter.x, self.jupiter.y));

            // 每隔一定步数输出信息
            if (i + 1) % 1000 == 0 {
                println!(
                    "Step {}: Time = {:.2} years, Earth r = {:.4} AU, Jupiter r = {:.4} AU",
                    i + 1,
                    self.time,
                    self.earth.distance_from_sun(),
                    self.jupiter.distance_from_sun()
                );
            }
        }

        (earth_positions, jupiter_positions)
    }
}

/// 将轨道数据保存到 CSV 文件
fn save_to_csv(
    filename: &str,
    earth_positions: &[(f64, f64)],
    jupiter_positions: &[(f64, f64)],
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "earth_x,earth_y,jupiter_x,jupiter_y")?;

    for (e_pos, j_pos) in earth_positions.iter().zip(jupiter_positions.iter()) {
        writeln!(file, "{},{},{},{}", e_pos.0, e_pos.1, j_pos.0, j_pos.1)?;
    }

    println!("轨道数据已保存到: {}", filename);
    Ok(())
}

/// 绘制轨道图
fn plot_orbits(
    filename: &str,
    earth_positions: &[(f64, f64)],
    jupiter_positions: &[(f64, f64)],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // 找出最大范围用于设置坐标轴
    let max_range = jupiter_positions
        .iter()
        .map(|(x, y)| x.abs().max(y.abs()))
        .fold(0.0f64, f64::max)
        * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption("双行星太阳系统轨道模拟 (Euler-Cromer)", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-max_range..max_range, -max_range..max_range)?;

    chart
        .configure_mesh()
        .x_desc("x (AU)")
        .y_desc("y (AU)")
        .draw()?;

    // 绘制太阳 (在原点)
    chart.draw_series(std::iter::once(Circle::new(
        (0.0, 0.0),
        5,
        ShapeStyle::from(&YELLOW).filled(),
    )))?;

    // 绘制地球轨道
    chart
        .draw_series(LineSeries::new(
            earth_positions.iter().map(|(x, y)| (*x, *y)),
            &BLUE,
        ))?
        .label("地球")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // 绘制木星轨道
    chart
        .draw_series(LineSeries::new(
            jupiter_positions.iter().map(|(x, y)| (*x, *y)),
            &RED,
        ))?
        .label("木星")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("轨道图已保存到: {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 双行星太阳系统模拟 (Example 4.2) ===\n");

    // 初始条件设置
    // 地球: 距离 1 AU, 圆轨道速度 = 2π AU/year
    let earth = Planet::new(
        "Earth",
        1.0,             // x = 1 AU
        0.0,             // y = 0
        0.0,             // vx = 0
        2.0 * PI,        // vy = 2π AU/year (圆轨道)
        3.0e-6,          // 地球质量 (太阳质量的 3×10⁻⁶)
    );

    // 木星: 距离 5.2 AU, 圆轨道速度 = 2π/√5.2 AU/year
    let jupiter = Planet::new(
        "Jupiter",
        5.2,                      // x = 5.2 AU
        0.0,                      // y = 0
        0.0,                      // vx = 0
        2.0 * PI / 5.2_f64.sqrt(), // vy = 2π/√5.2 AU/year
        9.5e-4,                   // 木星质量 (太阳质量的 9.5×10⁻⁴)
    );

    // 模拟参数
    let dt = 0.001; // 时间步长: 0.001 年 (约 8.76 小时)
    let years = 200.0; // 模拟 200 年
    let steps = (years / dt) as usize;

    println!("模拟参数:");
    println!("  时间步长: {} 年 (约 {:.2} 小时)", dt, dt * 365.25 * 24.0);
    println!("  总时间: {} 年", years);
    println!("  总步数: {}\n", steps);

    println!("初始条件:");
    println!("  地球: 位置 ({:.2}, {:.2}) AU, 速度 ({:.2}, {:.2}) AU/year",
             earth.x, earth.y, earth.vx, earth.vy);
    println!("  木星: 位置 ({:.2}, {:.2}) AU, 速度 ({:.2}, {:.2}) AU/year\n",
             jupiter.x, jupiter.y, jupiter.vx, jupiter.vy);

    // 创建模拟系统并运行
    let mut system = SolarSystem::new(earth, jupiter, dt);
    println!("开始模拟...\n");
    let (earth_positions, jupiter_positions) = system.simulate(steps);

    println!("\n模拟完成!");
    println!("\n最终状态:");
    println!("  地球: 位置 ({:.4}, {:.4}) AU, 距离 {:.4} AU",
             system.earth.x, system.earth.y, system.earth.distance_from_sun());
    println!("  木星: 位置 ({:.4}, {:.4}) AU, 距离 {:.4} AU",
             system.jupiter.x, system.jupiter.y, system.jupiter.distance_from_sun());

    // 保存数据
    println!("\n正在保存结果...");
    save_to_csv("jupiter_earth_orbit.csv", &earth_positions, &jupiter_positions)?;

    // 绘制轨道图
    plot_orbits("jupiter_earth_orbit.png", &earth_positions, &jupiter_positions)?;

    println!("\n程序运行完成!");
    Ok(())
}
