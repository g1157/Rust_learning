// 奶滴扩散的随机漫步模拟
// 模拟粒子在容器中扩散，通过小孔逃逸

use crate::common::{create_image, draw_pixel};
use image::Rgb;
use plotters::prelude::*;
use rand::Rng;

/// 容器配置
pub struct ContainerConfig {
    pub width: usize,      // 容器宽度
    pub height: usize,     // 容器高度
    pub hole_size: usize,  // 孔洞大小
    pub hole_start: usize, // 孔洞起始位置（沿边缘）
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            hole_size: 10,
            hole_start: 20, // 孔洞在 y=[20, 30) 位置，居中
        }
    }
}

/// 粒子位置
#[derive(Clone, Copy)]
struct Particle {
    x: i32,
    y: i32,
    active: bool, // 是否还在容器内
}

impl Particle {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y, active: true }
    }
}

/// 扩散模拟器
pub struct DiffusionSimulator {
    config: ContainerConfig,
    particles: Vec<Particle>,
    time: u64,
    history: Vec<(u64, usize)>, // (时间, 剩余粒子数)
}

impl DiffusionSimulator {
    /// 创建模拟器，粒子初始分布在中心区域
    pub fn new(config: ContainerConfig, num_particles: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut particles = Vec::with_capacity(num_particles);

        // 粒子初始分布在中心 10x10 区域
        let center_x = config.width as i32 / 2;
        let center_y = config.height as i32 / 2;

        for _ in 0..num_particles {
            let x = center_x + rng.r#gen::<i32>().rem_euclid(10) - 5;
            let y = center_y + rng.r#gen::<i32>().rem_euclid(10) - 5;
            particles.push(Particle::new(x, y));
        }

        let initial_count = particles.len();

        Self {
            config,
            particles,
            time: 0,
            history: vec![(0, initial_count)],
        }
    }

    /// 执行一步随机漫步
    pub fn step(&mut self) {
        let mut rng = rand::thread_rng();

        // 提取配置参数避免借用冲突
        let width = self.config.width as i32;
        let height = self.config.height as i32;
        let hole_start = self.config.hole_start as i32;
        let hole_end = (self.config.hole_start + self.config.hole_size) as i32;

        for particle in &mut self.particles {
            if !particle.active {
                continue;
            }

            // 随机选择方向：上下左右
            let direction = rng.r#gen::<u8>() % 4;
            let (dx, dy) = match direction {
                0 => (1, 0),  // 右
                1 => (-1, 0), // 左
                2 => (0, 1),  // 下
                _ => (0, -1), // 上
            };

            let new_x = particle.x + dx;
            let new_y = particle.y + dy;

            // 检查是否通过孔洞逃逸（右边墙）
            if new_x >= width && new_y >= hole_start && new_y < hole_end {
                particle.active = false;
                continue;
            }

            // 检查是否在容器内（碰到墙则不移动）
            if new_x >= 0 && new_x < width && new_y >= 0 && new_y < height {
                particle.x = new_x;
                particle.y = new_y;
            }
        }

        self.time += 1;
    }

    /// 执行多步模拟
    pub fn run(&mut self, steps: u64, record_interval: u64) {
        for _ in 0..steps {
            self.step();

            if self.time.rem_euclid(record_interval) == 0 {
                let count = self.count_active();
                self.history.push((self.time, count));
            }
        }
    }

    /// 统计活跃粒子数
    pub fn count_active(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }

    /// 获取历史记录
    pub fn get_history(&self) -> &[(u64, usize)] {
        &self.history
    }

    /// 生成当前状态的图像
    pub fn render_state(&self, scale: u32, output_path: &str) {
        let img_width = self.config.width as u32 * scale;
        let img_height = self.config.height as u32 * scale;

        // 浅灰色背景
        let mut img = create_image(img_width, img_height, Rgb([240, 240, 240]));

        // 绘制边界（深灰色）
        let wall_color = Rgb([100, 100, 100]);
        for i in 0..img_width {
            for s in 0..scale {
                draw_pixel(&mut img, i as f64, s as f64, wall_color);
                draw_pixel(&mut img, i as f64, (img_height - 1 - s) as f64, wall_color);
            }
        }
        for j in 0..img_height {
            for s in 0..scale {
                draw_pixel(&mut img, s as f64, j as f64, wall_color);
                // 右边墙：孔洞处不画
                let grid_y = j / scale;
                if grid_y < self.config.hole_start as u32
                    || grid_y >= (self.config.hole_start + self.config.hole_size) as u32
                {
                    draw_pixel(&mut img, (img_width - 1 - s) as f64, j as f64, wall_color);
                }
            }
        }

        // 绘制孔洞（白色）
        let hole_color = Rgb([255, 255, 255]);
        for y in self.config.hole_start..(self.config.hole_start + self.config.hole_size) {
            for s in 0..scale {
                for t in 0..scale {
                    draw_pixel(
                        &mut img,
                        (img_width - 1 - s) as f64,
                        (y as u32 * scale + t) as f64,
                        hole_color,
                    );
                }
            }
        }

        // 绘制粒子（蓝色）
        let particle_color = Rgb([30, 100, 200]);
        for particle in &self.particles {
            if particle.active {
                let px = particle.x as u32 * scale;
                let py = particle.y as u32 * scale;
                for dx in 0..scale {
                    for dy in 0..scale {
                        draw_pixel(&mut img, (px + dx) as f64, (py + dy) as f64, particle_color);
                    }
                }
            }
        }

        img.save(output_path).expect("保存扩散模拟图片失败");
    }
}

/// 运行完整模拟并输出结果
pub fn run_simulation() {
    println!("\n=== 作业二：奶滴扩散随机漫步模拟 ===\n");

    let config = ContainerConfig::default();
    let num_particles = 200;
    let mut sim = DiffusionSimulator::new(config, num_particles);

    println!("容器: {}x{}, 孔洞大小: {}", 50, 50, 10);
    println!("初始粒子数: {}\n", num_particles);

    // 保存初始状态
    sim.render_state(10, "diffusion_t0.png");
    println!(
        "t=0: 粒子数={} (已保存 diffusion_t0.png)",
        sim.count_active()
    );

    // 模拟到 t=10000，每 100 步记录一次
    sim.run(10_000, 100);
    sim.render_state(10, "diffusion_t1e4.png");
    println!(
        "t=10^4: 粒子数={} (已保存 diffusion_t1e4.png)",
        sim.count_active()
    );

    // 继续模拟到 t=100000
    sim.run(90_000, 1000);
    sim.render_state(10, "diffusion_t1e5.png");
    println!(
        "t=10^5: 粒子数={} (已保存 diffusion_t1e5.png)",
        sim.count_active()
    );

    // 继续模拟到 t=1000000 (10^6)
    sim.run(900_000, 10000);
    sim.render_state(10, "diffusion_t1e6.png");
    println!(
        "t=10^6: 粒子数={} (已保存 diffusion_t1e6.png)",
        sim.count_active()
    );

    // 输出数据用于验证 exp(-t/τ)
    println!("\n--- 粒子数随时间变化 (用于验证 exp(-t/τ)) ---");
    println!("{:<10} {:>10} {:>12}", "时间 t", "粒子数 N", "ln(N/N0)");
    println!("{}", "-".repeat(34));

    let n0 = num_particles as f64;
    let history = sim.get_history();

    // 选择有代表性的数据点输出
    for (t, n) in history.iter() {
        if *n > 0 && (*t <= 100000 && *t % 10000 == 0 || *t > 100000 && *t % 100000 == 0) {
            let ln_ratio = (*n as f64 / n0).ln();
            println!("{:<10} {:>10} {:>12.4}", t, n, ln_ratio);
        }
    }

    // 使用线性回归估算 τ
    // ln(N/N0) = -t/τ，所以 τ = -t / ln(N/N0)
    let valid_points: Vec<_> = history
        .iter()
        .filter(|(_, n)| *n > 5 && *n < num_particles) // 排除边界点
        .collect();

    let mut tau = 0.0;
    if valid_points.len() >= 2 {
        // 使用最小二乘法拟合 ln(N/N0) = -t/τ
        // 即 y = kx，其中 y = ln(N/N0), x = t, k = -1/τ
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (t, n) in &valid_points {
            let x = *t as f64;
            let y = (*n as f64 / n0).ln();
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let k = sum_xy / sum_xx; // 斜率
        tau = -1.0 / k;

        println!("\n根据 ln(N/N0) = -t/τ 最小二乘拟合:");
        println!("  斜率 k = {:.6}", k);
        println!("  τ = -1/k ≈ {:.0} 时间步", tau);

        // 计算半衰期
        let t_half = tau * 0.693;
        println!("  半衰期 t_1/2 = τ·ln(2) ≈ {:.0} 时间步", t_half);
    }

    // 绘制 ln(N/N0) vs t 图表
    if tau > 0.0 {
        plot_decay_curve(history, n0, tau, "diffusion_lnN_vs_t.png");
        println!("\n已生成 ln(N/N0) vs t 图表: diffusion_lnN_vs_t.png");
    }

    println!("\n验证结论：粒子数随时间呈指数衰减 N(t) = N0·exp(-t/τ)");
    println!("模拟完成！");
}

/// 绘制 ln(N/N0) vs t 图表，包含实验数据点和理论直线
fn plot_decay_curve(history: &[(u64, usize)], n0: f64, tau: f64, output_path: &str) {
    // 筛选有效数据点 (N > 0)
    let data_points: Vec<(f64, f64)> = history
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(t, n)| (*t as f64, (*n as f64 / n0).ln()))
        .collect();

    if data_points.is_empty() {
        return;
    }

    // 确定坐标范围
    let t_max = data_points.iter().map(|(t, _)| *t).fold(0.0, f64::max) * 1.1;
    let ln_min = data_points.iter().map(|(_, ln)| *ln).fold(0.0, f64::min) * 1.1;

    // 创建绘图区域
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("ln(N/N₀) vs t", ("sans-serif", 28).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..t_max, ln_min..0.5)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("t (time steps)")
        .y_desc("ln(N/N₀)")
        .x_label_style(("sans-serif", 16))
        .y_label_style(("sans-serif", 16))
        .axis_desc_style(("sans-serif", 18))
        .draw()
        .unwrap();

    // 绘制理论直线 ln(N/N0) = -t/τ
    let theory_line: Vec<(f64, f64)> = (0..100)
        .map(|i| {
            let t = t_max * (i as f64 / 100.0);
            (t, -t / tau)
        })
        .filter(|(_, ln)| *ln >= ln_min)
        .collect();

    chart
        .draw_series(LineSeries::new(theory_line, BLUE.stroke_width(2)))
        .unwrap()
        .label(format!("Theory: ln(N/N0) = -t/tau, tau={:.0}", tau))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    // 绘制实验数据点
    chart
        .draw_series(
            data_points
                .iter()
                .map(|(t, ln)| Circle::new((*t, *ln), 5, RED.filled())),
        )
        .unwrap()
        .label("Experiment Data")
        .legend(|(x, y)| Circle::new((x + 10, y), 5, RED.filled()));

    // 绘制图例
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("sans-serif", 14))
        .draw()
        .unwrap();

    root.present().unwrap();
}

// ============================================================================
// 奶滴扩散模拟算法说明
// ============================================================================
//
// 物理模型：
//   - 粒子在 50×50 格子中做随机漫步
//   - 每步随机选择上/下/左/右移动一格
//   - 碰到墙壁则不移动（反弹）
//   - 右边墙有一个 10 格的孔，粒子到达孔处会逃逸
//
// 数学预期：
//   - 粒子数 N(t) = N0 * exp(-t/τ)
//   - τ 是特征逃逸时间，与容器尺寸和孔洞大小有关
//   - ln(N/N0) vs t 应该是直线，斜率 = -1/τ
