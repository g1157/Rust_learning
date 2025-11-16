use std::env;
use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use std::ops::{Add, Mul, Sub};

// 二维向量结构体
#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

// 向量加法
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

// 向量减法
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

// 向量数乘
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f64) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

// 行星结构体
#[derive(Debug, Clone)]
struct Planet {
    position: Vec2,
    velocity: Vec2,
    mass: f64,
}

// 轨道模拟器
struct Simulation {
    sun_mass: f64,
    planet: Planet,
    beta: f64,
    dt: f64,
    trail: Vec<Vec2>,
    time: f64,
}

impl Simulation {
    fn new(beta: f64, initial_pos: Vec2, initial_vel: Vec2) -> Self {
        Simulation {
            sun_mass: 1.0,
            planet: Planet {
                position: initial_pos,
                velocity: initial_vel,
                mass: 1e-6,
            },
            beta,
            dt: 0.0001,  // 更小的时间步长提高精度
            trail: Vec::new(),
            time: 0.0,
        }
    }

    // 计算广义引力加速度: a = -GM / r^(2+β) * r̂
    fn gravity_acceleration(&self) -> Vec2 {
        let r_vec = self.planet.position;
        let r = r_vec.length();

        if r < 1e-10 {
            return Vec2::new(0.0, 0.0);
        }

        let r_hat = r_vec.normalize();
        let g = 4.0 * PI * PI; // 引力常数（天文单位制：AU, year, solar mass）

        let a_magnitude = -g * self.sun_mass / r.powf(2.0 + self.beta);

        r_hat * a_magnitude
    }

    // 欧拉-克罗默单步积分
    fn step(&mut self) {
        let accel = self.gravity_acceleration();

        // 关键：先更新速度
        self.planet.velocity = self.planet.velocity + accel * self.dt;

        // 再用新速度更新位置
        self.planet.position = self.planet.position + self.planet.velocity * self.dt;

        // 记录轨迹
        self.trail.push(self.planet.position);

        self.time += self.dt;
    }

    // 运行模拟
    fn run(&mut self, total_time: f64) {
        let steps = (total_time / self.dt) as usize;
        println!("Running simulation for {:.2} years ({} steps)...", total_time, steps);

        for i in 0..steps {
            self.step();
            // 只记录一部分轨迹点以节省内存（每10步记录一次）
            if i % 10 == 0 {
                if i % 10000 == 0 {
                    print!("Progress: {:.1}%\r", (i as f64 / steps as f64) * 100.0);
                    std::io::stdout().flush().unwrap();
                }
            } else if i < steps - 1 {
                // 不是采样点则移除
                self.trail.pop();
            }
        }
        println!("Progress: 100.0% - Complete!     ");
    }

    // 计算总能量（用于检查守恒）
    fn total_energy(&self) -> f64 {
        let v = self.planet.velocity.length();
        let r = self.planet.position.length();

        // 动能
        let ke = 0.5 * self.planet.mass * v * v;

        // 势能（广义形式）
        let g = 4.0 * PI * PI;
        let pe = if self.beta.abs() < 1e-10 {
            -g * self.sun_mass * self.planet.mass / r
        } else {
            -g * self.sun_mass * self.planet.mass / ((1.0 + self.beta) * r.powf(1.0 + self.beta))
        };

        ke + pe
    }

    // 生成 SVG 图像
    fn save_svg(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        // 找到轨道范围
        let max_r = self.trail.iter()
            .map(|v| v.length())
            .fold(0.0, f64::max);

        let view_size = (max_r * 2.2).max(3.0);

        // SVG 头部
        writeln!(file, r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="800" height="800">"#,
                 -view_size / 2.0, -view_size / 2.0, view_size, view_size)?;

        // 背景
        writeln!(file, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#0a0a0a\"/>",
                 -view_size / 2.0, -view_size / 2.0, view_size, view_size)?;

        // 网格线（参考）
        writeln!(file, "<circle cx=\"0\" cy=\"0\" r=\"{}\" fill=\"none\" stroke=\"#333\" stroke-width=\"{}\"/>",
                 1.0, view_size * 0.002)?;
        writeln!(file, "<circle cx=\"0\" cy=\"0\" r=\"{}\" fill=\"none\" stroke=\"#333\" stroke-width=\"{}\"/>",
                 2.0, view_size * 0.002)?;

        // 太阳（中心黄色圆）
        writeln!(file, "<circle cx=\"0\" cy=\"0\" r=\"{}\" fill=\"#FDB813\"/>", view_size * 0.03)?;
        writeln!(file, "<circle cx=\"0\" cy=\"0\" r=\"{}\" fill=\"#FFD700\" opacity=\"0.6\"/>", view_size * 0.04)?;

        // 轨道路径
        if !self.trail.is_empty() {
            write!(file, "<polyline points=\"")?;
            for point in &self.trail {
                write!(file, "{},{} ", point.x, -point.y)?; // 翻转 y 轴
            }
            writeln!(file, "\" fill=\"none\" stroke=\"#4A9EFF\" stroke-width=\"{}\" opacity=\"0.8\"/>",
                     view_size * 0.004)?;

            // 行星当前位置
            let last = self.trail.last().unwrap();
            writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#4A9EFF\"/>",
                     last.x, -last.y, view_size * 0.02)?;
        }

        // 添加文字标签
        let text_size = view_size * 0.08;
        writeln!(file, r#"<text x="{}" y="{}" fill="white" font-size="{}" font-family="monospace">β = {:.2}</text>"#,
                 -view_size * 0.48, -view_size * 0.42, text_size, self.beta)?;

        writeln!(file, "</svg>")?;
        println!("SVG saved to: {}", filename);
        Ok(())
    }

    // 生成 ASCII 艺术图
    fn print_ascii_orbit(&self) {
        const WIDTH: usize = 80;
        const HEIGHT: usize = 40;
        let mut grid = vec![vec![' '; WIDTH]; HEIGHT];

        if self.trail.is_empty() {
            println!("No orbit data to display!");
            return;
        }

        // 找到轨道范围
        let max_r = self.trail.iter()
            .map(|v| v.x.abs().max(v.y.abs()))
            .fold(0.0, f64::max) * 1.1;

        // 标记轨迹点
        for point in &self.trail {
            let x = ((point.x / max_r + 1.0) * WIDTH as f64 / 2.0) as usize;
            let y = ((point.y / max_r + 1.0) * HEIGHT as f64 / 2.0) as usize;
            if x < WIDTH && y < HEIGHT {
                grid[y][x] = '*';
            }
        }

        // 标记太阳
        let sun_x = WIDTH / 2;
        let sun_y = HEIGHT / 2;
        grid[sun_y][sun_x] = 'O';

        // 打印
        println!("\n╔{}╗", "═".repeat(WIDTH));
        for row in grid {
            print!("║");
            for ch in row {
                print!("{}", ch);
            }
            println!("║");
        }
        println!("╚{}╝", "═".repeat(WIDTH));
        println!("β = {:.2}, Time = {:.2} years", self.beta, self.time);
    }

    // 保存 CSV 数据
    fn save_csv(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "x,y,r,vx,vy,v")?;

        for (i, point) in self.trail.iter().enumerate() {
            let r = point.length();
            // 近似速度（使用相邻点差分）
            let (vx, vy) = if i + 1 < self.trail.len() {
                let next = self.trail[i + 1];
                let dt = self.dt * 10.0; // 因为每10步记录一次
                ((next.x - point.x) / dt, (next.y - point.y) / dt)
            } else {
                (0.0, 0.0)
            };
            let v = (vx * vx + vy * vy).sqrt();
            writeln!(file, "{},{},{},{},{},{}", point.x, point.y, r, vx, vy, v)?;
        }

        println!("CSV data saved to: {}", filename);
        Ok(())
    }

    // 生成交互式 HTML 文件
    fn save_html(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        // 找到轨道范围
        let max_r = self.trail.iter()
            .map(|v| v.length())
            .fold(0.0, f64::max);

        let view_size = (max_r * 2.2).max(3.0);

        // 生成轨道点的 JavaScript 数组
        let mut points_js = String::from("[");
        for (i, point) in self.trail.iter().enumerate() {
            if i > 0 {
                points_js.push(',');
            }
            points_js.push_str(&format!("[{:.6},{:.6}]", point.x, point.y));
        }
        points_js.push(']');

        // HTML 内容
        let html_content = format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>轨道模拟 - β = {:.2}</title>
    <style>
        body {{
            margin: 0;
            padding: 20px;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: #0a0a0a;
            color: #fff;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        h1 {{
            margin: 10px 0;
            font-size: 24px;
        }}
        .info {{
            margin: 10px 0;
            padding: 15px;
            background: #1a1a1a;
            border-radius: 8px;
            font-family: monospace;
            font-size: 14px;
        }}
        canvas {{
            border: 2px solid #333;
            border-radius: 8px;
            margin: 20px 0;
            cursor: crosshair;
        }}
        .controls {{
            display: flex;
            gap: 15px;
            margin: 10px 0;
        }}
        button {{
            padding: 10px 20px;
            background: #4A9EFF;
            border: none;
            border-radius: 5px;
            color: white;
            font-size: 14px;
            cursor: pointer;
            transition: background 0.3s;
        }}
        button:hover {{
            background: #357ABD;
        }}
        button:disabled {{
            background: #555;
            cursor: not-allowed;
        }}
    </style>
</head>
<body>
    <h1>🌌 广义引力轨道模拟器</h1>
    <div class="info">
        引力定律: F = -GMm / r^(2+β)<br>
        β = {:.2} | 模拟时间: {:.1} 年 | 轨迹点数: {}
    </div>
    
    <canvas id="canvas" width="800" height="800"></canvas>
    
    <div class="controls">
        <button id="resetBtn">重置视图</button>
        <button id="toggleTrailBtn">切换轨迹</button>
        <label style="color: #fff; display: flex; align-items: center; gap: 10px;">
            点大小: <input type="range" id="pointSize" min="1" max="5" value="2" style="width: 150px;">
            <span id="sizeLabel">2</span>
        </label>
    </div>
    
    <div class="info" id="stats">
        鼠标移动到画布上查看坐标
    </div>

    <script>
        const canvas = document.getElementById('canvas');
        const ctx = canvas.getContext('2d');
        const statsDiv = document.getElementById('stats');
        
        const points = {points_js};
        const viewSize = {view_size};
        const beta = {beta};
        
        let showTrail = true;
        let scale = canvas.width / viewSize;
        let offsetX = canvas.width / 2;
        let offsetY = canvas.height / 2;
        let pointSize = 2;
        
        // 坐标转换
        function worldToScreen(x, y) {{
            return {{
                x: x * scale + offsetX,
                y: -y * scale + offsetY
            }};
        }}
        
        function screenToWorld(sx, sy) {{
            return {{
                x: (sx - offsetX) / scale,
                y: -(sy - offsetY) / scale
            }};
        }}
        
        // 绘制函数
        function draw() {{
            ctx.fillStyle = '#0a0a0a';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            
            // 网格线
            ctx.strokeStyle = '#333';
            ctx.lineWidth = 1;
            for (let r = 1; r <= 3; r++) {{
                ctx.beginPath();
                let screenR = r * scale;
                ctx.arc(offsetX, offsetY, screenR, 0, Math.PI * 2);
                ctx.stroke();
            }}
            
            // 坐标轴
            ctx.strokeStyle = '#444';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(0, offsetY);
            ctx.lineTo(canvas.width, offsetY);
            ctx.moveTo(offsetX, 0);
            ctx.lineTo(offsetX, canvas.height);
            ctx.stroke();
            
            // 太阳
            ctx.fillStyle = '#FDB813';
            ctx.beginPath();
            ctx.arc(offsetX, offsetY, 12, 0, Math.PI * 2);
            ctx.fill();
            
            ctx.fillStyle = 'rgba(255, 215, 0, 0.4)';
            ctx.beginPath();
            ctx.arc(offsetX, offsetY, 18, 0, Math.PI * 2);
            ctx.fill();
            
            // 轨道散点图
            if (showTrail && points.length > 0) {{
                ctx.fillStyle = '#4A9EFF';
                
                for (let i = 0; i < points.length; i++) {{
                    let p = worldToScreen(points[i][0], points[i][1]);
                    ctx.beginPath();
                    ctx.arc(p.x, p.y, pointSize, 0, Math.PI * 2);
                    ctx.fill();
                }}
            }}
            
            // 行星当前位置（红色高亮）
            if (points.length > 0) {{
                let last = points[points.length - 1];
                let pos = worldToScreen(last[0], last[1]);
                
                ctx.fillStyle = '#FF4A4A';
                ctx.beginPath();
                ctx.arc(pos.x, pos.y, 6, 0, Math.PI * 2);
                ctx.fill();
                
                // 外圈高亮
                ctx.strokeStyle = '#FF4A4A';
                ctx.lineWidth = 2;
                ctx.beginPath();
                ctx.arc(pos.x, pos.y, 9, 0, Math.PI * 2);
                ctx.stroke();
            }}
        }}
        
        // 鼠标移动显示坐标
        canvas.addEventListener('mousemove', (e) => {{
            let rect = canvas.getBoundingClientRect();
            let sx = e.clientX - rect.left;
            let sy = e.clientY - rect.top;
            let world = screenToWorld(sx, sy);
            let r = Math.sqrt(world.x * world.x + world.y * world.y);
            statsDiv.innerHTML = `x = ${{world.x.toFixed(3)}} AU | y = ${{world.y.toFixed(3)}} AU | r = ${{r.toFixed(3)}} AU`;
        }});
        
        canvas.addEventListener('mouseleave', () => {{
            statsDiv.innerHTML = '鼠标移动到画布上查看坐标';
        }});
        
        // 按钮事件
        document.getElementById('resetBtn').addEventListener('click', () => {{
            scale = canvas.width / viewSize;
            offsetX = canvas.width / 2;
            offsetY = canvas.height / 2;
            draw();
        }});
        
        document.getElementById('toggleTrailBtn').addEventListener('click', () => {{
            showTrail = !showTrail;
            draw();
        }});
        
        document.getElementById('pointSize').addEventListener('input', (e) => {{
            pointSize = parseFloat(e.target.value);
            document.getElementById('sizeLabel').textContent = pointSize;
            draw();
        }});
        
        // 初始绘制
        draw();
    </script>
</body>
</html>"#, self.beta, self.beta, self.time, self.trail.len(), 
            points_js = points_js, view_size = view_size, beta = self.beta);

        file.write_all(html_content.as_bytes())?;
        println!("HTML visualization saved to: {}", filename);
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // 解析命令行参数
    let beta = if args.len() > 1 {
        args[1].parse::<f64>().unwrap_or(0.0)
    } else {
        0.0
    };

    println!("=== Orbital Simulation with Generalized Gravity ===");
    println!("Force law: F = -GMm / r^(2+β)");
    println!("β = {}", beta);
    println!();

    // 初始条件（椭圆轨道，离心率 e ≈ 0.5）
    let initial_pos = Vec2::new(1.0, 0.0); // 1 AU from sun（近日点）
    let initial_vel = Vec2::new(0.0, 2.0 * PI * 1.225); // 增加速度以产生椭圆轨道

    let mut sim = Simulation::new(beta, initial_pos, initial_vel);

    // 记录初始能量
    let initial_energy = sim.total_energy();
    println!("Initial energy: {:.6e}", initial_energy);

    // 运行模拟 50 个轨道周期（看到更多进动）
    sim.run(50.0);

    // 检查能量守恒
    let final_energy = sim.total_energy();
    let energy_drift = ((final_energy - initial_energy) / initial_energy).abs() * 100.0;
    println!("Final energy: {:.6e}", final_energy);
    println!("Energy drift: {:.4}%", energy_drift);
    println!();

    // 输出结果
    sim.print_ascii_orbit();

    // 生成文件名
    let beta_str = format!("{:.1}", beta).replace('.', "_");
    let svg_filename = format!("orbit_beta_{}.svg", beta_str);
    let csv_filename = format!("orbit_beta_{}.csv", beta_str);
    let html_filename = format!("orbit_beta_{}.html", beta_str);

    // 保存文件
    sim.save_svg(&svg_filename).expect("Failed to save SVG");
    sim.save_csv(&csv_filename).expect("Failed to save CSV");
    sim.save_html(&html_filename).expect("Failed to save HTML");

    println!();
    println!("Tip: Open {} in a web browser to view the interactive orbit!", html_filename);
}
