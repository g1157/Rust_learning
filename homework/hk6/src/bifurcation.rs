use plotters::prelude::*;
use std::env;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::pendulum::{PendulumState, OMEGA_D};

pub fn generate_bifurcation_diagram() -> Result<(), Box<dyn Error>> {
    let cfg = BifurcationConfig::from_env();
    let drive_period = 2.0 * PI / OMEGA_D;
    let total_params = cfg.param_steps + 1;

    println!(
        "Bifurcation sweep: F_D from {:.3} to {:.3} ({} samples), warmup {} cycles, sample {} cycles, dt = {:.4}",
        cfg.f_min,
        cfg.f_max,
        total_params,
        cfg.warmup_cycles,
        cfg.sample_cycles,
        cfg.dt
    );

    let file = File::create("bifurcation_fd.csv")?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "F_D,theta,omega")?;

    let mut points = Vec::new();
    let mut theta_min = f64::MAX;
    let mut theta_max = f64::MIN;

    let progress_interval = (total_params / 20).max(1);

    for i in 0..total_params {
        let f_drive = cfg.f_min + (cfg.f_max - cfg.f_min) * (i as f64) / (cfg.param_steps as f64);
        if i % progress_interval == 0 {
            println!(
                "  processing F_D = {:.4} ({}/{})",
                f_drive,
                i + 1,
                total_params
            );
        }

        let mut prev_state = PendulumState::new(
            cfg.initial_theta,
            cfg.initial_omega,
            0.0,
            cfg.dt,
            f_drive,
        );
        let total_cycles = cfg.warmup_cycles + cfg.sample_cycles;
        let mut cycles_seen = 0usize;

        let mut prev_phase = prev_state.t.rem_euclid(drive_period);

        while cycles_seen < total_cycles {
            let next_state = prev_state.next();
            let current_phase = next_state.t.rem_euclid(drive_period);

            if prev_phase > current_phase {
                cycles_seen += 1;
                if cycles_seen > cfg.warmup_cycles {
                    let phase_prev = prev_phase;
                    let phase_curr = current_phase + drive_period;
                    let denom = phase_curr - phase_prev;
                    let frac = if denom.abs() > f64::EPSILON {
                        (drive_period - phase_prev) / denom
                    } else {
                        0.0
                    };

                    let theta_cross = prev_state.theta
                        + frac * (next_state.theta - prev_state.theta);
                    let omega_cross = prev_state.omega
                        + frac * (next_state.omega - prev_state.omega);
                    let theta_wrapped = (theta_cross + PI).rem_euclid(2.0 * PI) - PI;

                    writeln!(writer, "{:.6},{:.10},{:.10}", f_drive, theta_wrapped, omega_cross)?;
                    points.push((f_drive, theta_wrapped));
                    theta_min = theta_min.min(theta_wrapped);
                    theta_max = theta_max.max(theta_wrapped);
                }
            }

            prev_state = next_state;
            prev_phase = current_phase;
        }
    }

    writer.flush()?;

    if points.is_empty() {
        println!("No bifurcation samples were collected.");
        return Ok(());
    }

    let theta_margin = ((theta_max - theta_min) * 0.05).max(1e-3);
    plot_bifurcation(
        &points,
        (cfg.f_min, cfg.f_max),
        (theta_min - theta_margin, theta_max + theta_margin),
    )?;

    println!(
        "Saved bifurcation samples to bifurcation_fd.csv and plot to bifurcation_fd.png (points = {}).",
        points.len()
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct BifurcationConfig {
    f_min: f64,
    f_max: f64,
    param_steps: usize,
    warmup_cycles: usize,
    sample_cycles: usize,
    dt: f64,
    initial_theta: f64,
    initial_omega: f64,
}

impl BifurcationConfig {
    fn from_env() -> Self {
        let default = Self {
            f_min: 0.5,
            f_max: 4.0,
            param_steps: 1200,
            warmup_cycles: 250,
            sample_cycles: 800,
            dt: 0.02,
            initial_theta: 0.2,
            initial_omega: 0.0,
        };

        Self {
            f_min: env_var_f64("HK6_BIF_F_MIN", default.f_min),
            f_max: env_var_f64("HK6_BIF_F_MAX", default.f_max),
            param_steps: env_var_usize("HK6_BIF_PARAM_STEPS", default.param_steps).max(1),
            warmup_cycles: env_var_usize("HK6_BIF_WARMUP", default.warmup_cycles),
            sample_cycles: env_var_usize("HK6_BIF_SAMPLE", default.sample_cycles),
            dt: env_var_f64("HK6_BIF_DT", default.dt),
            initial_theta: env_var_f64("HK6_BIF_THETA0", default.initial_theta),
            initial_omega: env_var_f64("HK6_BIF_OMEGA0", default.initial_omega),
        }
    }
}

fn env_var_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

fn env_var_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn plot_bifurcation(
    points: &[(f64, f64)],
    f_range: (f64, f64),
    theta_range: (f64, f64),
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("bifurcation_fd.png", (1800, 1200)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .caption("Bifurcation Diagram vs F_D", ("sans-serif", 36).into_font())
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(f_range.0..f_range.1, theta_range.0..theta_range.1)?;

    chart
        .configure_mesh()
        .x_desc("F_D")
        .y_desc("theta (rad)")
        .label_style(("sans-serif", 22).into_font())
        .axis_style(&BLACK.mix(0.7))
        .draw()?;

    chart.draw_series(points.iter().map(|(f_drive, theta)| {
        Circle::new((*f_drive, *theta), 1, RGBColor(0, 120, 200).mix(0.35).filled())
    }))?;

    root.present()?;
    Ok(())
}
