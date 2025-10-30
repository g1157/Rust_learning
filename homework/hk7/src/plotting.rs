use plotly::{
    common::{Line, Mode, Title},
    layout::{Axis, Layout},
    Plot, Scatter,
};
use std::error::Error;

use crate::simulation::SimulationResult;

pub fn export_plots(result: &SimulationResult) -> Result<(), Box<dyn Error>> {
    plot_trajectory(result)?;
    plot_phase_space(result)?;
    plot_attractor(result)?;
    Ok(())
}

fn plot_trajectory(result: &SimulationResult) -> Result<(), Box<dyn Error>> {
    let boundary_trace = Scatter::new(
        result.boundary_points.iter().map(|p| p.0).collect::<Vec<_>>(),
        result.boundary_points.iter().map(|p| p.1).collect::<Vec<_>>(),
    )
    .mode(Mode::Lines)
    .line(Line::new().color("rgba(80,80,80,0.8)").width(2.0))
    .name("边界");

    let trajectory_trace = Scatter::new(
        result.positions.iter().map(|p| p.0).collect::<Vec<_>>(),
        result.positions.iter().map(|p| p.1).collect::<Vec<_>>(),
    )
    .mode(Mode::Lines)
    .line(Line::new().color("rgba(220,20,60,0.8)").width(1.5))
    .name("轨迹");

    let layout = Layout::new()
        .title(Title::new("Bezier 边界内小球轨迹"))
        .x_axis(Axis::new().title(Title::new("x")))
        .y_axis(Axis::new().title(Title::new("y")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(boundary_trace);
    plot.add_trace(trajectory_trace);
    plot.write_html("trajectory_bezier.html");
    Ok(())
}

fn plot_phase_space(result: &SimulationResult) -> Result<(), Box<dyn Error>> {
    let velocities = &result.velocities;

    let phase_trace = Scatter::new(
        velocities.iter().map(|v| v.0).collect::<Vec<_>>(),
        velocities.iter().map(|v| v.1).collect::<Vec<_>>(),
    )
    .mode(Mode::Markers)
    .marker(plotly::common::Marker::new().size(4).color("rgba(30,144,255,0.5)"))
    .name("相空间");

    let layout = Layout::new()
        .title(Title::new("速度相空间 (v_x, v_y)"))
        .x_axis(Axis::new().title(Title::new("v_x")))
        .y_axis(Axis::new().title(Title::new("v_y")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(phase_trace);
    plot.write_html("phase_space.html");
    Ok(())
}

fn plot_attractor(result: &SimulationResult) -> Result<(), Box<dyn Error>> {
    let traj_trace = Scatter::new(
        result.positions.iter().map(|p| p.0).collect::<Vec<_>>(),
        result.velocities.iter().map(|v| v.0).collect::<Vec<_>>(),
    )
    .mode(Mode::Markers)
    .marker(plotly::common::Marker::new().size(3).opacity(0.5).color("rgba(34,139,34,0.5)"))
    .name("吸引子 (x vs v_x)");

    let layout = Layout::new()
        .title(Title::new("吸引子投影 (x, v_x)"))
        .x_axis(Axis::new().title(Title::new("x")))
        .y_axis(Axis::new().title(Title::new("v_x")));

    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(traj_trace);
    plot.write_html("attractor_xy.html");
    Ok(())
}
