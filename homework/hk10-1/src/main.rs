use plotly::{Layout, Plot, Scatter};
use plotly::common::Mode;
use plotly::layout::Axis;
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::RGBPixel;
use std::fs::File;
use std::io::BufWriter;

/// 波动方程求解器
struct WaveSolver {
    nx: usize,           // 空间网格点数
    dx: f64,             // 空间步长
    r: f64,              // r = c * dt / dx
    y_prev: Vec<f64>,    // y^(n-1)
    y_curr: Vec<f64>,    // y^n
    y_next: Vec<f64>,    // y^(n+1)
}

impl WaveSolver {
    fn new(nx: usize, length: f64, _c: f64, r: f64) -> Self {
        let dx = length / (nx - 1) as f64;
        
        WaveSolver {
            nx,
            dx,
            r,
            y_prev: vec![0.0; nx],
            y_curr: vec![0.0; nx],
            y_next: vec![0.0; nx],
        }
    }
    
    /// 设置初始条件（双向传播，初速度为0）
    fn set_initial_condition<F>(&mut self, f: F)
    where
        F: Fn(f64) -> f64,
    {
        for i in 0..self.nx {
            let x = i as f64 * self.dx;
            self.y_curr[i] = f(x);
            self.y_prev[i] = f(x); // 初速度为0，所以 y^(-1) = y^0
        }
    }
    
    /// 设置单向传播的初始条件
    fn set_unidirectional_condition<F>(&mut self, f: F, direction: i32)
    where
        F: Fn(f64) -> f64,
    {
        // 设置 t=0 时刻的位移
        for i in 0..self.nx {
            let x = i as f64 * self.dx;
            self.y_curr[i] = f(x);
        }
        
        // 设置 t=dt 时刻的位移，使用数值导数和初速度条件
        for i in 0..self.nx {
            if i == 0 {
                // 左边界
                self.y_prev[i] = 0.0;
            } else if i == self.nx - 1 {
                // 右边界
                self.y_prev[i] = 0.0;
            } else {
                let x = i as f64 * self.dx;
                let x_plus = (i + 1) as f64 * self.dx;
                let x_minus = (i - 1) as f64 * self.dx;
                let df_dx = (f(x_plus) - f(x_minus)) / (2.0 * self.dx);
                
                // direction = 1 表示向右传播，direction = -1 表示向左传播
                self.y_prev[i] = f(x) - (direction as f64) * self.r * self.dx * df_dx;
            }
        }
    }
    
    /// 执行一步时间演化
    fn step(&mut self) {
        // 使用公式: y[i]^(n+1) = 2*(1 - r^2)*y[i]^n - y[i]^(n-1) + r^2*(y[i+1]^n + y[i-1]^n)
        for i in 1..self.nx - 1 {
            self.y_next[i] = 2.0 * (1.0 - self.r * self.r) * self.y_curr[i]
                - self.y_prev[i]
                + self.r * self.r * (self.y_curr[i + 1] + self.y_curr[i - 1]);
        }
        
        // 固定端边界条件
        self.y_next[0] = 0.0;
        self.y_next[self.nx - 1] = 0.0;
        
        // 更新时间层
        std::mem::swap(&mut self.y_prev, &mut self.y_curr);
        std::mem::swap(&mut self.y_curr, &mut self.y_next);
    }
    
    /// 运行模拟并记录所有时间步
    fn run_all_steps(&mut self, n_steps: usize) -> Vec<Vec<f64>> {
        let mut snapshots = Vec::new();
        snapshots.push(self.y_curr.clone());
        
        for _ in 0..n_steps {
            self.step();
            snapshots.push(self.y_curr.clone());
        }
        
        snapshots
    }
    
    fn get_x_coordinates(&self) -> Vec<f64> {
        (0..self.nx).map(|i| i as f64 * self.dx).collect()
    }
    
    fn get_dt(&self, c: f64) -> f64 {
        self.r * self.dx / c
    }
}

/// 高斯波包
fn gaussian(x: f64, x0: f64, sigma: f64) -> f64 {
    (-(x - x0).powi(2) / (2.0 * sigma * sigma)).exp()
}

/// 使用 plotters 生成 GIF 动画
fn create_gif_animation(
    filename: &str,
    x: &[f64],
    snapshots: &[Vec<f64>],
    dt: f64,
    title: &str,
    y_range: (f64, f64),
    frame_skip: usize,
    total_frames_target: usize, // 目标总帧数，用于调整播放速度
) -> Result<(), Box<dyn std::error::Error>> {
    let width = 800;
    let height = 600;
    
    let x_range = (x[0], x[x.len() - 1]);
    
    // 计算实际帧数
    let actual_frames = (snapshots.len() + frame_skip - 1) / frame_skip;
    
    // 调整延迟以保持总播放时长一致（假设完整动画为5秒）
    let target_duration_ms = 5000; // 5秒
    let delay = (target_duration_ms / total_frames_target) / 10; // 单位：10ms
    
    // 创建 GIF 文件
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    let mut encoder = gif::Encoder::new(writer, width as u16, height as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;
    
    // 为每一帧生成图像
    for (frame_idx, step) in (0..snapshots.len()).step_by(frame_skip).enumerate() {
        let snapshot = &snapshots[step];
        let time = step as f64 * dt;
        
        // 创建内存缓冲区
        let mut buffer = vec![0u8; (width * height * 3) as usize];
        
        {
            let root = BitMapBackend::<RGBPixel>::with_buffer(&mut buffer, (width, height))
                .into_drawing_area();
            root.fill(&WHITE)?;
            
            let mut chart = ChartBuilder::on(&root)
                .caption(
                    &format!("{} (t = {:.3} s, step: {})", title, time, step),
                    ("sans-serif", 30).into_font(),
                )
                .margin(15)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .build_cartesian_2d(x_range.0..x_range.1, y_range.0..y_range.1)?;
            
            chart
                .configure_mesh()
                .x_desc("Position x (m)")
                .y_desc("Displacement y (m)")
                .draw()?;
            
            // 绘制波形
            let points: Vec<(f64, f64)> = x.iter().zip(snapshot.iter())
                .map(|(&xi, &yi)| (xi, yi))
                .collect();
            
            chart.draw_series(LineSeries::new(points.clone(), &BLUE.mix(0.8)))?;
            chart.draw_series(PointSeries::of_element(
                points,
                3,
                &BLUE,
                &|c, s, st| {
                    return EmptyElement::at(c) + Circle::new((0, 0), s, st.filled());
                },
            ))?;
            
            root.present()?;
        }
        
        // 转换为 GIF 帧
        let mut frame = gif::Frame::from_rgb_speed(width as u16, height as u16, &buffer, 10);
        frame.delay = delay as u16;
        encoder.write_frame(&frame)?;
        
        if frame_idx % 10 == 0 {
            print!("\r  Generating frame: {}/{}", frame_idx + 1, actual_frames);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    
    println!(); // 换行
    drop(encoder);
    
    Ok(())
}

/// 习题 6.2: 比较不同 r 值的稳定性
fn problem_6_2() {
    println!("=== Problem 6.2: Comparing stability for different r values ===\n");
    
    let nx = 200;
    let length = 10.0;
    let c = 1.0;
    let r_values = vec![0.5, 1.0, 1.5];
    
    let n_steps = 300;
    let frame_skip = 3; // 每3步记录一帧
    let total_frames = (n_steps + frame_skip) / frame_skip; // 预期总帧数
    
    for &r in &r_values {
        println!("Testing r = {}", r);
        
        let mut solver = WaveSolver::new(nx, length, c, r);
        let dt = solver.get_dt(c);
        
        // 在中心位置创建高斯波包
        solver.set_initial_condition(|x| gaussian(x, 5.0, 0.5));
        
        let snapshots = solver.run_all_steps(n_steps);
        let x = solver.get_x_coordinates();
        
        // 检查是否发散，调整 y 轴范围
        let max_val = snapshots.last().unwrap().iter().fold(0.0f64, |a, &b| a.max(b.abs()));
        let y_range = if max_val > 10.0 {
            println!("  WARNING: Numerical divergence! Max value = {:.2e}", max_val);
            println!("  Using logarithmic scale indication for divergence visualization");
            
            // 使用所有帧，但调整 y 轴范围
            // 找到一个合理的上限（不要太大以至于看不清前期波形）
            let mut max_reasonable = 0.0f64;
            for (i, snap) in snapshots.iter().enumerate() {
                let m = snap.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
                // 只考虑前半段来设置范围，这样可以看到发散过程
                if i < snapshots.len() / 2 && m > max_reasonable && m < 100.0 {
                    max_reasonable = m;
                }
            }
            
            if max_reasonable < 5.0 {
                max_reasonable = 5.0;
            }
            let range = max_reasonable * 1.5;
            (-range, range)
        } else {
            (-1.5, 1.5)
        };
        
        // 生成 GIF 动画
        let filename = format!("problem_6_2_r_{}.gif", r.to_string().replace('.', "_"));
        let title = format!("Wave Equation (r = {}, c = {} m/s)", r, c);
        
        match create_gif_animation(&filename, &x, &snapshots, dt, &title, y_range, frame_skip, total_frames) {
            Ok(_) => {
                println!("  ✓ GIF animation saved to: {}", filename);
            }
            Err(e) => {
                println!("  ✗ Failed to generate GIF: {}", e);
            }
        }
        
        // 分析结果
        if r < 1.0 {
            println!("  r < 1: Algorithm stable, but may have small ripples on reflection");
        } else if (r - 1.0).abs() < 0.01 {
            println!("  r = 1: Most accurate, best error cancellation");
        } else {
            println!("  r > 1: Algorithm unstable, numerical divergence");
        }
        println!();
    }
    
    // 同时生成 Plotly HTML（作为备份）
    println!("Also generating interactive HTML files (using Plotly)...\n");
    problem_6_2_plotly();
}

/// 习题 6.3: 构造单向传播的波包
fn problem_6_3() {
    println!("=== Problem 6.3: Unidirectional wave packets ===\n");
    
    let nx = 200;
    let length = 10.0;
    let c = 1.0;
    let r = 1.0; // 使用最稳定的 r 值
    
    let n_steps = 300;
    let frame_skip = 3;
    let total_frames = (n_steps + frame_skip) / frame_skip;
    
    // 测试向右传播的波
    println!("Testing rightward propagation (direction = +1)");
    let mut solver = WaveSolver::new(nx, length, c, r);
    let dt = solver.get_dt(c);
    solver.set_unidirectional_condition(|x| gaussian(x, 3.0, 0.5), 1);
    
    let snapshots = solver.run_all_steps(n_steps);
    let x = solver.get_x_coordinates();
    
    let filename = "problem_6_3_rightward.gif";
    let title = format!("Rightward Wave Packet (r = {}, c = {} m/s)", r, c);
    
    match create_gif_animation(&filename, &x, &snapshots, dt, &title, (-1.2, 1.2), frame_skip, total_frames) {
        Ok(_) => {
            println!("  ✓ GIF animation saved to: {}", filename);
        }
        Err(e) => {
            println!("  ✗ Failed to generate GIF: {}", e);
        }
    }
    println!("  Wave packet should propagate rightward only, without splitting\n");
    
    // 测试向左传播的波
    println!("Testing leftward propagation (direction = -1)");
    let mut solver = WaveSolver::new(nx, length, c, r);
    solver.set_unidirectional_condition(|x| gaussian(x, 7.0, 0.5), -1);
    
    let snapshots = solver.run_all_steps(n_steps);
    let x = solver.get_x_coordinates();
    
    let filename = "problem_6_3_leftward.gif";
    let title = format!("Leftward Wave Packet (r = {}, c = {} m/s)", r, c);
    
    match create_gif_animation(&filename, &x, &snapshots, dt, &title, (-1.2, 1.2), frame_skip, total_frames) {
        Ok(_) => {
            println!("  ✓ GIF animation saved to: {}", filename);
        }
        Err(e) => {
            println!("  ✗ Failed to generate GIF: {}", e);
        }
    }
    println!("  Wave packet should propagate leftward only, without splitting\n");
    
    // 同时生成 Plotly HTML（作为备份）
    println!("Also generating interactive HTML files (using Plotly)...\n");
    problem_6_3_plotly();
}

/// 使用 Plotly 生成交互式 HTML（习题 6.2）
fn problem_6_2_plotly() {
    let nx = 200;
    let length = 10.0;
    let c = 1.0;
    let r_values = vec![0.5, 1.0];
    
    let n_steps = 300;
    let frame_skip = 5;
    
    for &r in &r_values {
        let mut solver = WaveSolver::new(nx, length, c, r);
        let dt = solver.get_dt(c);
        solver.set_initial_condition(|x| gaussian(x, 5.0, 0.5));
        
        let snapshots = solver.run_all_steps(n_steps);
        let x = solver.get_x_coordinates();
        
        let mut plot = Plot::new();
        
        for (frame_idx, step) in (0..=n_steps).step_by(frame_skip).enumerate() {
            if step >= snapshots.len() {
                break;
            }
            let snapshot = &snapshots[step];
            let time = step as f64 * dt;
            
            let trace = Scatter::new(x.clone(), snapshot.clone())
                .mode(Mode::LinesMarkers)
                .name(format!("t = {:.3} s", time))
                .visible(if frame_idx == 0 { 
                    plotly::common::Visible::True 
                } else { 
                    plotly::common::Visible::LegendOnly 
                });
            
            plot.add_trace(trace);
        }
        
        let layout = Layout::new()
            .title(&format!("波动方程数值解 (r = {}, c = {} m/s)", r, c))
            .x_axis(Axis::new()
                .title("位置 x (m)")
                .range(vec![0.0, length]))
            .y_axis(Axis::new()
                .title("位移 y (m)")
                .range(vec![-1.5, 1.5]));
        
        plot.set_layout(layout);
        plot.use_local_plotly();
        
        let filename = format!("problem_6_2_r_{}.html", r.to_string().replace('.', "_"));
        plot.write_html(&filename);
        println!("  ✓ HTML 已保存到: {}", filename);
    }
}

/// 使用 Plotly 生成交互式 HTML（习题 6.3）
fn problem_6_3_plotly() {
    let nx = 200;
    let length = 10.0;
    let c = 1.0;
    let r = 1.0;
    let n_steps = 300;
    let frame_skip = 5;
    
    // 向右传播
    let mut solver = WaveSolver::new(nx, length, c, r);
    let dt = solver.get_dt(c);
    solver.set_unidirectional_condition(|x| gaussian(x, 3.0, 0.5), 1);
    
    let snapshots = solver.run_all_steps(n_steps);
    let x = solver.get_x_coordinates();
    
    let mut plot = Plot::new();
    
    for (frame_idx, step) in (0..=n_steps).step_by(frame_skip).enumerate() {
        if step >= snapshots.len() {
            break;
        }
        let snapshot = &snapshots[step];
        let time = step as f64 * dt;
        
        let trace = Scatter::new(x.clone(), snapshot.clone())
            .mode(Mode::LinesMarkers)
            .name(format!("t = {:.3} s", time))
            .visible(if frame_idx == 0 { 
                plotly::common::Visible::True 
            } else { 
                plotly::common::Visible::LegendOnly 
            });
        
        plot.add_trace(trace);
    }
    
    let layout = Layout::new()
        .title(&format!("单向传播波包 - 向右 (r = {}, c = {} m/s)", r, c))
        .x_axis(Axis::new()
            .title("位置 x (m)")
            .range(vec![0.0, length]))
        .y_axis(Axis::new()
            .title("位移 y (m)")
            .range(vec![-1.2, 1.2]));
    
    plot.set_layout(layout);
    plot.use_local_plotly();
    plot.write_html("problem_6_3_rightward.html");
    println!("  ✓ HTML 已保存到: problem_6_3_rightward.html");
    
    // 向左传播
    let mut solver = WaveSolver::new(nx, length, c, r);
    solver.set_unidirectional_condition(|x| gaussian(x, 7.0, 0.5), -1);
    
    let snapshots = solver.run_all_steps(n_steps);
    let x = solver.get_x_coordinates();
    
    let mut plot = Plot::new();
    
    for (frame_idx, step) in (0..=n_steps).step_by(frame_skip).enumerate() {
        if step >= snapshots.len() {
            break;
        }
        let snapshot = &snapshots[step];
        let time = step as f64 * dt;
        
        let trace = Scatter::new(x.clone(), snapshot.clone())
            .mode(Mode::LinesMarkers)
            .name(format!("t = {:.3} s", time))
            .visible(if frame_idx == 0 { 
                plotly::common::Visible::True 
            } else { 
                plotly::common::Visible::LegendOnly 
            });
        
        plot.add_trace(trace);
    }
    
    let layout = Layout::new()
        .title(&format!("单向传播波包 - 向左 (r = {}, c = {} m/s)", r, c))
        .x_axis(Axis::new()
            .title("位置 x (m)")
            .range(vec![0.0, length]))
        .y_axis(Axis::new()
            .title("位移 y (m)")
            .range(vec![-1.2, 1.2]));
    
    plot.set_layout(layout);
    plot.use_local_plotly();
    plot.write_html("problem_6_3_leftward.html");
    println!("  ✓ HTML 已保存到: problem_6_3_leftward.html");
}

fn main() {
    println!("Wave Equation Numerical Solution - Problems 6.2 and 6.3\n");
    println!("============================================\n");
    
    // 运行习题 6.2
    problem_6_2();
    
    println!("============================================\n");
    
    // 运行习题 6.3
    problem_6_3();
    
    println!("============================================");
    println!("\n✓ All results generated!");
    println!("\nFile descriptions:");
    println!("  GIF files: Auto-playing animations, viewable in browsers/image viewers");
    println!("  HTML files: Interactive plots, click legend to show/hide different time frames");
}
