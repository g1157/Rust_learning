use plotters::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::pendulum::{PendulumState, DEFAULT_F_D};

pub fn generate_delta_theta_plot() -> Result<(), Box<dyn Error>> {
    let dt: f64 = 0.04;
    let total_time: f64 = 150.0;
    let steps = (total_time / dt).round() as usize;

    let mut state_1 = PendulumState::new(0.2, 0.0, 0.0, dt, DEFAULT_F_D);
    let mut state_2 = PendulumState::new(0.2 + 0.001, 0.0, 0.0, dt, DEFAULT_F_D);

    let mut deltas = Vec::with_capacity(steps + 1);
    deltas.push((state_1.t, (state_2.theta - state_1.theta).abs()));

    for _ in 0..steps {
        state_1 = state_1.next();
        state_2 = state_2.next();
        let delta_theta = (state_2.theta - state_1.theta).abs();
        deltas.push((state_1.t, delta_theta));
    }

    write_delta_csv(&deltas)?;
    plot_delta_theta(&deltas, total_time)?;

    println!(
        "Saved delta-theta data to delta_theta_vs_time.csv and plot to delta_theta_vs_time.png (dt = {}, steps = {}).",
        dt, steps
    );

    Ok(())
}

fn write_delta_csv(data: &[(f64, f64)]) -> Result<(), Box<dyn Error>> {
    let file = File::create("delta_theta_vs_time.csv")?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "t,delta_theta")?;
    for (t, delta) in data {
        writeln!(writer, "{:.6},{:.6}", t, delta)?;
    }
    writer.flush()?;
    Ok(())
}

fn plot_delta_theta(data: &[(f64, f64)], total_time: f64) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("delta_theta_vs_time.png", (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .caption("|Δθ| vs t (log₁₀ scale)", ("sans-serif", 32).into_font())
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(0.0f64..total_time, -6.0f64..2.0f64)?;

    chart
        .configure_mesh()
        .x_desc("t (s)")
        .y_desc("|Δθ|")
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("10^{:.0}", v))
        .label_style(("sans-serif", 20).into_font())
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|(t, delta)| (*t, log10_clamped(*delta))),
            &RED,
        ))?
        .label("|Δθ|")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 20).into_font())
        .draw()?;

    root.present()?;
    Ok(())
}

fn log10_clamped(value: f64) -> f64 {
    let clamped = value.clamp(1e-6, 100.0);
    clamped.log10()
}
