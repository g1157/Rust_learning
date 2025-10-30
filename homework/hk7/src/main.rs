mod simulation;
mod plotting;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 配置模拟参数
    let config = simulation::SimulationConfig {
        boundary_segments: simulation::example_boundary(),
        dt: 0.01,
        total_time: 200.0,
        initial_position: (0.2, 0.1),
        initial_velocity: (0.8, 0.6),
        damping: 0.0,
        record_interval: 1,
    };

    let result = simulation::run_simulation(&config)?;

    plotting::export_plots(&result)?;

    println!(
        "模拟结束: 轨迹点数 = {}, 碰撞次数 = {}\n生成文件: trajectory_bezier.html, phase_space.html, attractor_xy.html",
        result.positions.len(),
        result.collisions
    );

    Ok(())
}