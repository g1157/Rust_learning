mod attractor;
mod bifurcation;
mod delta_theta;
mod pendulum;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = env::args().nth(1).unwrap_or_else(|| "delta".to_string());

    match mode.as_str() {
        "delta" => delta_theta::generate_delta_theta_plot(),
        "attractor" => attractor::generate_high_res_attractor(),
        "bifurcation" => bifurcation::generate_bifurcation_diagram(),
        other => {
            eprintln!(
                "Unknown mode '{}'. Use 'delta', 'attractor', or 'bifurcation' (default).",
                other
            );
            Ok(())
        }
    }
}
