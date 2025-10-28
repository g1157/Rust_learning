use plotters::prelude::*;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};

struct PendulumState{
    theta: f64,
    omega: f64,
    t: f64,
    dt: f64,
}

const G: f64 = 9.8;
const L: f64 = 9.8;
const Q: f64 = 0.5;
const F_D: f64 = 1.2;
const OMEGA_D: f64 = 2.0 / 3.0;

impl PendulumState{
    fn new(theta: f64, omega: f64, t: f64, dt: f64) -> Self{
        PendulumState{
            theta,
            omega,
            t,
            dt,
        }        
    }
    fn acceleration(theta: f64, omega: f64, t: f64) -> f64 {
        - (G / L) * theta.sin() - Q * omega + F_D * (OMEGA_D * t).sin()
    }

    fn next(&self) -> Self{
        let dt = self.dt;

        // Classical RK4 integration for the driven pendulum.
        let k1_theta = self.omega;
        let k1_omega = Self::acceleration(self.theta, self.omega, self.t);

        let theta_mid = self.theta + 0.5 * dt * k1_theta;
        let omega_mid = self.omega + 0.5 * dt * k1_omega;
        let t_mid = self.t + 0.5 * dt;

        let k2_theta = omega_mid;
        let k2_omega = Self::acceleration(theta_mid, omega_mid, t_mid);

        let theta_mid = self.theta + 0.5 * dt * k2_theta;
        let omega_mid = self.omega + 0.5 * dt * k2_omega;

        let k3_theta = omega_mid;
        let k3_omega = Self::acceleration(theta_mid, omega_mid, t_mid);

        let theta_end = self.theta + dt * k3_theta;
        let omega_end = self.omega + dt * k3_omega;
        let t_end = self.t + dt;

        let k4_theta = omega_end;
        let k4_omega = Self::acceleration(theta_end, omega_end, t_end);

        let theta = self.theta + (dt / 6.0) * (k1_theta + 2.0 * k2_theta + 2.0 * k3_theta + k4_theta);
        let omega = self.omega + (dt / 6.0) * (k1_omega + 2.0 * k2_omega + 2.0 * k3_omega + k4_omega);
        let t = self.t + dt;

        PendulumState::new(theta, omega, t, dt)
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    generate_delta_theta_plot()?;
    generate_high_res_attractor()?;
    Ok(())
}

fn generate_delta_theta_plot() -> Result<(), Box<dyn Error>> {
    let dt: f64 = 0.04;
    let total_time: f64 = 150.0;
    let steps = (total_time / dt).round() as usize;

    let mut state_1 = PendulumState::new(0.2, 0.0, 0.0, dt);
    let mut state_2 = PendulumState::new(0.2 + 0.001, 0.0, 0.0, dt);

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

fn generate_high_res_attractor() -> Result<(), Box<dyn Error>> {
    let dt = 0.01;
    let total_steps = 50_000_000usize;
    let warmup_steps = 500_000usize;

    let mut state = PendulumState::new(0.2, 0.0, 0.0, dt);
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
        .axis_style(&BLACK.mix(0.7))
        .label_style(("sans-serif", 22).into_font())
        .draw()?;

    chart.draw_series(points.iter().map(|(theta, omega)| {
        Circle::new((*theta, *omega), 1, RGBColor(0, 90, 200).mix(0.8).filled())
    }))?;

    root.present()?;
    Ok(())
}
