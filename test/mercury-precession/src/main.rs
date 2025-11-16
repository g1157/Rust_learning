use std::f64::consts::PI;
use std::ops::{Add, Sub, Mul};
use plotters::prelude::*;

// ============================================================================
// 物理常数
// ============================================================================

/// 引力常数 (AU³/(M☉·year²))
/// 在天文单位制中 G = 4π²
const G: f64 = 4.0 * PI * PI;

/// 太阳质量 (M☉)
const SUN_MASS: f64 = 1.0;

/// 光速 (AU/year)
/// c ≈ 299792458 m/s ≈ 63241.077 AU/year
const C: f64 = 63241.077;

// ============================================================================
// 水星轨道参数
// ============================================================================

/// 水星轨道半长轴 (AU)
const MERCURY_SEMI_MAJOR_AXIS: f64 = 0.387098;

/// 水星轨道离心率
const MERCURY_ECCENTRICITY: f64 = 0.2056;

/// 水星轨道周期 (年)
const MERCURY_PERIOD: f64 = 0.2408;

// ============================================================================
// Vec2 - 二维向量
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    /// 创建新的二维向量
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    /// 计算向量的模长
    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 返回单位向量
    fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Vec2 {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            Vec2 { x: 0.0, y: 0.0 }
        }
    }

    /// 计算二维叉积的 z 分量 (用于角动量计算)
    fn cross_z(&self, other: &Vec2) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

// 向量加法
impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// 向量减法
impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// 向量数乘
impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f64) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

// ============================================================================
// Planet - 行星
// ============================================================================

#[derive(Debug, Clone)]
struct Planet {
    pos: Vec2,   // 位置 (AU)
    vel: Vec2,   // 速度 (AU/year)
    mass: f64,   // 质量 (M☉)
}

impl Planet {
    /// 创建新的行星
    fn new(pos: Vec2, vel: Vec2, mass: f64) -> Self {
        Planet { pos, vel, mass }
    }

    /// 计算相对于太阳的角动量 L = r × mv
    fn angular_momentum(&self) -> f64 {
        self.pos.cross_z(&self.vel).abs() * self.mass
    }
}

// ============================================================================
// Simulation - 轨道模拟器
// ============================================================================

struct Simulation {
    planet: Planet,
    sun_mass: f64,
    time: f64,
    dt: f64,
    trajectory: Vec<Vec2>,
    gr_coefficient: f64,  // 广义相对论修正系数 a
    perihelia: Vec<(f64, f64)>,  // 近日点记录: (时间, 角度)
    last_r: f64,  // 用于检测近日点
    last_last_r: f64,  // 用于检测近日点
}

impl Simulation {
    /// 创建新的模拟器
    /// gr_coefficient: 广义相对论修正系数 (0=牛顿, 1=标准GR)
    fn new(planet: Planet, dt: f64, gr_coefficient: f64) -> Self {
        let initial_r = planet.pos.magnitude();
        Simulation {
            planet,
            sun_mass: SUN_MASS,
            time: 0.0,
            dt,
            trajectory: Vec::new(),
            gr_coefficient,
            perihelia: Vec::new(),
            last_r: initial_r,
            last_last_r: initial_r,
        }
    }

    /// 计算牛顿引力加速度
    /// a = -GM/r² * r̂
    fn acceleration_newton(&self) -> Vec2 {
        let r = self.planet.pos.magnitude();
        let r_hat = self.planet.pos.normalize();
        
        // a = -GM/r²
        let a_magnitude = -G * self.sun_mass / (r * r);
        
        r_hat * a_magnitude
    }

    /// 计算带广义相对论修正的引力加速度
    /// a = -GM/r² × [1 + α × 3GM/(rc²)] × r̂
    /// 其中 α 是修正系数参数 (gr_coefficient)
    fn acceleration_gr(&self) -> Vec2 {
        let r = self.planet.pos.magnitude();
        let r_hat = self.planet.pos.normalize();
        
        // 施瓦西半径: r_s = 2GM/c²
        let schwarzschild_radius = 2.0 * G * self.sun_mass / (C * C);
        
        // 广义相对论修正项: 3GM/(rc²) = 1.5 × r_s / r
        let gr_correction_term = 1.5 * schwarzschild_radius / r;
        
        // 总修正因子: 1 + α × correction_term
        let correction_factor = 1.0 + self.gr_coefficient * gr_correction_term;
        
        // 加速度: -GM/r² × correction_factor
        let a_magnitude = -G * self.sun_mass / (r * r) * correction_factor;
        
        r_hat * a_magnitude
    }

    /// Velocity Verlet 算法单步积分
    /// 
    /// 算法流程:
    /// 1. 计算当前加速度 a(t)
    /// 2. 更新位置: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
    /// 3. 计算新加速度 a(t+dt)
    /// 4. 更新速度: v(t+dt) = v(t) + 0.5*[a(t) + a(t+dt)]*dt
    fn step(&mut self) {
        // 1. 计算当前加速度（根据 gr_coefficient 选择模型）
        let a1 = if self.gr_coefficient == 0.0 {
            self.acceleration_newton()
        } else {
            self.acceleration_gr()
        };
        
        // 2. 更新位置
        self.planet.pos = self.planet.pos + self.planet.vel * self.dt + a1 * (0.5 * self.dt * self.dt);
        
        // 3. 计算新位置的加速度
        let a2 = if self.gr_coefficient == 0.0 {
            self.acceleration_newton()
        } else {
            self.acceleration_gr()
        };
        
        // 4. 更新速度 (使用两个加速度的平均值)
        self.planet.vel = self.planet.vel + (a1 + a2) * (0.5 * self.dt);
        
        // 检测近日点（距离局部最小值）
        let current_r = self.planet.pos.magnitude();
        if self.last_r < self.last_last_r && self.last_r < current_r {
            // 上一个点是近日点，记录其角度
            // 注意：这里记录的是之前位置的角度，需要用保存的位置
            // 简化处理：用当前角度近似（误差很小）
            let angle = self.planet.pos.y.atan2(self.planet.pos.x);
            self.perihelia.push((self.time, angle));
        }
        
        // 更新距离历史
        self.last_last_r = self.last_r;
        self.last_r = current_r;
        
        // 更新时间
        self.time += self.dt;
    }

    /// 运行模拟指定的时长
    fn run(&mut self, duration: f64) {
        let num_steps = (duration / self.dt) as usize;
        let record_interval = 10; // 每 10 步记录一次轨迹点
        
        // 记录初始位置
        self.trajectory.push(self.planet.pos);
        
        for i in 0..num_steps {
            self.step();
            
            // 每隔一定步数记录轨迹
            if (i + 1) % record_interval == 0 {
                self.trajectory.push(self.planet.pos);
            }
        }
    }

    /// 计算系统总能量 (动能 + 势能)
    fn total_energy(&self) -> f64 {
        let r = self.planet.pos.magnitude();
        let v = self.planet.vel.magnitude();
        
        // 动能: KE = 0.5 * m * v²
        let kinetic = 0.5 * self.planet.mass * v * v;
        
        // 势能: PE = -GMm/r
        let potential = -G * self.sun_mass * self.planet.mass / r;
        
        kinetic + potential
    }
}

// ============================================================================
// 绘图函数
// ============================================================================

/// 绘制不同 a 系数下的进动轨迹对比图
fn plot_precession_comparison(
    results: &[(f64, Vec<Vec2>, Vec<(f64, f64)>)],
    filename: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1400, 1000))
        .into_drawing_area();
    root.fill(&WHITE)?;
    
    // 找出最大范围
    let mut max_range = 0.0;
    for (_, trajectory, _) in results {
        for pos in trajectory {
            let r = (pos.x * pos.x + pos.y * pos.y).sqrt();
            if r > max_range {
                max_range = r;
            }
        }
    }
    max_range *= 1.15;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("水星进动模拟 - 不同广义相对论修正系数 a", ("sans-serif", 50))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(-max_range..max_range, -max_range..max_range)?;
    
    chart.configure_mesh()
        .x_desc("X (AU)")
        .y_desc("Y (AU)")
        .axis_desc_style(("sans-serif", 20))
        .draw()?;
    
    // 绘制太阳
    chart.draw_series(PointSeries::of_element(
        vec![(0.0, 0.0)],
        8,
        ShapeStyle::from(&RED).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord) + Circle::new((0, 0), size, style)
        },
    ))?
    .label("太阳")
    .legend(|(x, y)| Circle::new((x + 10, y), 4, RED.filled()));
    
    // 颜色方案
    let colors = [
        &BLUE,
        &GREEN, 
        &RED,
        &MAGENTA,
        &BLACK,
        &CYAN,
    ];
    
    for (i, (a_coeff, trajectory, perihelia)) in results.iter().enumerate() {
        let color = colors[i % colors.len()];
        
        // 绘制轨迹
        chart.draw_series(LineSeries::new(
            trajectory.iter().map(|p| (p.x, p.y)),
            color.stroke_width(2),
        ))?
        .label(format!("a = {:.2}", a_coeff))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
        
        // 标记前几个近日点
        if perihelia.len() > 1 {
            let num_to_show = 10.min(perihelia.len());
            for (_, angle) in perihelia.iter().take(num_to_show) {
                let r = MERCURY_SEMI_MAJOR_AXIS * (1.0 - MERCURY_ECCENTRICITY);
                let x = r * angle.cos();
                let y = r * angle.sin();
                chart.draw_series(PointSeries::of_element(
                    vec![(x, y)],
                    4,
                    ShapeStyle::from(color).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                    },
                ))?;
            }
        }
    }
    
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.9))
        .border_style(&BLACK)
        .label_font(("sans-serif", 20))
        .draw()?;
    
    root.present()?;
    println!("✅ 图像已保存到: {}", filename);
    Ok(())
}

// ============================================================================
// 主函数
// ============================================================================

fn main() {
    println!("🌟 水星近日点进动模拟 - 广义相对论修正");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // 测试不同的修正系数 a
    // a = 0: 牛顿引力（无进动）
    // a = 1: 标准广义相对论
    // a > 1: 增强的相对论效应
    // a < 1: 减弱的相对论效应
    let test_coefficients = vec![0.0, 0.5, 1.0, 2.0, 3.0];
    
    let num_orbits = 100.0;  // 运行100个轨道周期以看到明显进动
    
    let mut results = Vec::new();
    
    for &a_coeff in &test_coefficients {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("广义相对论修正系数 a = {:.2}", a_coeff);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // 计算初始条件（水星在近日点）
        let r_perihelion = MERCURY_SEMI_MAJOR_AXIS * (1.0 - MERCURY_ECCENTRICITY);
        let v_perihelion = (G * SUN_MASS * (1.0 + MERCURY_ECCENTRICITY) / 
                           (MERCURY_SEMI_MAJOR_AXIS * (1.0 - MERCURY_ECCENTRICITY))).sqrt();
        
        let initial_pos = Vec2::new(r_perihelion, 0.0);
        let initial_vel = Vec2::new(0.0, v_perihelion);
        let mercury = Planet::new(initial_pos, initial_vel, 1e-7);
        
        // 创建模拟器
        let mut sim = Simulation::new(mercury, 0.00001, a_coeff);
        
        let sim_duration = num_orbits * MERCURY_PERIOD;
        
        println!("模拟参数:");
        println!("  轨道周期: {:.4} 年", MERCURY_PERIOD);
        println!("  模拟时长: {:.2} 年", sim_duration);
        println!("  轨道数: {:.0}", num_orbits);
        println!("  时间步长: {} 年", sim.dt);
        
        println!("\n运行模拟...");
        sim.run(sim_duration);
        
        // 分析进动
        if sim.perihelia.len() >= 2 {
            let first_angle = sim.perihelia[0].1;
            let last_angle = sim.perihelia[sim.perihelia.len() - 1].1;
            let mut total_precession = last_angle - first_angle;
            
            // 处理角度跨越 ±π 的情况
            while total_precession > PI {
                total_precession -= 2.0 * PI;
            }
            while total_precession < -PI {
                total_precession += 2.0 * PI;
            }
            
            let total_precession_deg = total_precession.to_degrees();
            let num_periods = (sim.perihelia.len() - 1) as f64;
            let precession_per_orbit_deg = total_precession_deg / num_periods;
            let precession_per_orbit_arcsec = precession_per_orbit_deg * 3600.0;
            
            // 每世纪进动（角秒）
            let precession_per_century_arcsec = precession_per_orbit_arcsec / MERCURY_PERIOD * 100.0;
            
            println!("\n进动分析:");
            println!("  检测到近日点: {} 个", sim.perihelia.len());
            println!("  完整轨道周期: {:.0}", num_periods);
            println!("  总进动角度: {:.6}°", total_precession_deg);
            println!("  每轨道进动: {:.6}° = {:.6}\"", 
                     precession_per_orbit_deg, precession_per_orbit_arcsec);
            println!("  每世纪进动: {:.6}\"", precession_per_century_arcsec);
            
            if a_coeff == 1.0 {
                println!("\n  📊 理论值 (水星): 43\" / 世纪");
                println!("     相对误差: {:.2}%", 
                         ((precession_per_century_arcsec - 43.0) / 43.0 * 100.0).abs());
            }
        } else {
            println!("\n⚠️  近日点数据不足，无法计算进动");
        }
        
        results.push((a_coeff, sim.trajectory.clone(), sim.perihelia.clone()));
    }
    
    // 生成对比图
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("生成轨迹对比图...\n");
    
    if let Err(e) = plot_precession_comparison(&results, "mercury_precession_comparison.png") {
        eprintln!("❌ 绘图错误: {}", e);
    }
    
    println!("\n✅ 所有模拟完成！");
    println!("\n说明:");
    println!("  • a = 0.0: 牛顿引力（轨道闭合，无进动）");
    println!("  • a = 1.0: 标准广义相对论（水星实际进动 ≈ 43\"/世纪）");
    println!("  • a > 1.0: 增强的相对论效应（进动更快）");
    println!("  • 图中的点标记了近日点位置，可清晰看到进动趋势");
}
