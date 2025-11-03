use plotters::prelude::*;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::pendulum::{PendulumState, DEFAULT_F_D, OMEGA_D};

pub fn generate_high_res_attractor() -> Result<(), Box<dyn Error>> {
    let dt = 0.01;
    let total_steps = 50_000_000usize;
    let warmup_steps = 500_000usize;

    let mut state = PendulumState::new(0.2, 0.0, 0.0, dt, DEFAULT_F_D);
    let drive_period = 2.0 * PI / OMEGA_D;

    let file = File::create("chaotic_attractor_theta_gt2.csv")?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "theta,omega")?;

    let mut prev_phase = state.t.rem_euclid(drive_period);
    let mut samples = Vec::new();
    let mut theta_min = f64::MAX;
    let mut theta_max = f64::MIN;
    let mut omega_min = f64::MAX;
    let mut omega_max = f64::MIN;

    for step in 0..total_steps {
        state = state.next();
        let current_phase = state.t.rem_euclid(drive_period);

        if step >= warmup_steps && prev_phase > current_phase {
            let theta_wrapped = (state.theta + PI).rem_euclid(2.0 * PI) - PI;
            if theta_wrapped > 2.0 {
                writeln!(writer, "{:.10},{:.10}", theta_wrapped, state.omega)?;
                samples.push((theta_wrapped, state.omega));
                theta_min = theta_min.min(theta_wrapped);
                theta_max = theta_max.max(theta_wrapped);
                omega_min = omega_min.min(state.omega);
                omega_max = omega_max.max(state.omega);
            }
        }

        prev_phase = current_phase;
    }

    writer.flush()?;

    if samples.is_empty() {
        println!("No attractor samples with theta > 2 rad were collected.");
        return Ok(());
    }

    let theta_margin = ((theta_max - theta_min) * 0.05).max(1e-3);
    let omega_margin = ((omega_max - omega_min) * 0.05).max(1e-3);

    plot_attractor(
        &samples,
        (theta_min - theta_margin, theta_max + theta_margin),
        (omega_min - omega_margin, omega_max + omega_margin),
    )?;

    println!(
        "Saved chaotic attractor data (theta > 2 rad) to chaotic_attractor_theta_gt2.csv and plot to chaotic_attractor_theta_gt2.png. Samples: {}",
        samples.len()
    );

    Ok(())
}

fn plot_attractor(
    points: &[(f64, f64)],
    theta_range: (f64, f64),
    omega_range: (f64, f64),
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("chaotic_attractor_theta_gt2.png", (1600, 1200)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .caption("Chaotic Attractor (theta > 2 rad)", ("sans-serif", 36).into_font())
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(theta_range.0..theta_range.1, omega_range.0..omega_range.1)?;

    chart
        .configure_mesh()
        .x_desc("theta (rad)")
        .y_desc("omega (rad/s)")
        .axis_style(BLACK.mix(0.7))
        .label_style(("sans-serif", 22).into_font())
        .draw()?;

    chart.draw_series(points.iter().map(|(theta, omega)| {
        Circle::new((*theta, *omega), 1, RGBColor(0, 90, 200).mix(0.8).filled())
    }))?;

    root.present()?;
    Ok(())
}
