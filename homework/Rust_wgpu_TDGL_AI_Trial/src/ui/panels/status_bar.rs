//! Bottom status bar

use egui::Ui;
use crate::ui::theme::FG_SECONDARY;

/// Draw the bottom status bar
pub fn draw_status_bar(
    ui: &mut Ui,
    step_count: u64,
    total_steps: Option<u64>,
    sim_time: f32,
    steps_per_sec: f32,
    gpu_name: &str,
) {
    ui.horizontal(|ui| {
        // Step count
        if let Some(total) = total_steps {
            let progress = step_count as f32 / total as f32;
            ui.label(format!("Step: {} / {}", step_count, total));
            ui.add(egui::ProgressBar::new(progress).desired_width(100.0));

            // ETA
            if steps_per_sec > 0.0 {
                let remaining = total.saturating_sub(step_count);
                let eta_secs = remaining as f32 / steps_per_sec;
                if eta_secs < 60.0 {
                    ui.label(format!("ETA: {:.0}s", eta_secs));
                } else {
                    ui.label(format!("ETA: {:.1}m", eta_secs / 60.0));
                }
            }
        } else {
            ui.label(format!("Step: {}", step_count));
        }

        ui.separator();
        ui.label(format!("Time: {:.2}", sim_time));

        ui.separator();
        ui.colored_label(FG_SECONDARY, gpu_name);

        ui.separator();
        ui.label(format!("{:.0} steps/s", steps_per_sec));
    });
}
