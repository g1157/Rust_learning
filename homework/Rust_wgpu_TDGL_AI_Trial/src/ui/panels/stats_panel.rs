//! Right-side statistics panel

use egui::{Ui, CollapsingHeader};
use crate::ui::theme::{ACCENT, SUCCESS};

/// Statistics data for display
#[derive(Clone, Debug, Default)]
pub struct SimStats {
    pub vortices: i32,
    pub antivortices: i32,
    pub net: i32,
    pub pinned_v: i32,
    pub pinned_av: i32,
    pub pinned_net: i32,
    pub energy: f64,
    pub energy_density: f64,
    pub mean_vx: f32,
    pub mean_vy: f32,
    pub mean_speed: f32,
    pub steps_per_sec: f32,
}

/// Draw the statistics panel
pub fn draw_stats_panel(ui: &mut Ui, stats: &SimStats, target_flux_n: i32) {
    CollapsingHeader::new("▼ 实时统计")
        .default_open(true)
        .show(ui, |ui| {
            // Vortex counts
            ui.horizontal(|ui| {
                ui.label("涡旋:");
                ui.colored_label(ACCENT, format!("{}", stats.vortices));
                ui.label("反涡旋:");
                ui.colored_label(ACCENT, format!("{}", stats.antivortices));
            });

            ui.horizontal(|ui| {
                ui.label("净涡旋:");
                let net_color = if stats.net == target_flux_n { SUCCESS } else { ACCENT };
                ui.colored_label(net_color, format!("{}", stats.net));
                ui.label(format!("(目标: {})", target_flux_n));
                if stats.net == target_flux_n {
                    ui.label("✓");
                }
            });

            ui.separator();

            // Pinning stats
            let pinned_total = stats.vortices + stats.antivortices;
            let pinned_count = stats.pinned_v + stats.pinned_av;
            let pinned_pct = if pinned_total > 0 {
                100.0 * pinned_count as f32 / pinned_total as f32
            } else {
                0.0
            };
            ui.label(format!("钉扎: {} / {} ({:.1}%)", pinned_count, pinned_total, pinned_pct));

            ui.separator();

            // Energy
            ui.label(format!("能量: {:.4e}", stats.energy));
            ui.label(format!("能量密度: {:.6}", stats.energy_density));

            ui.separator();

            // Velocity
            ui.label(format!("速度: ({:.4}, {:.4})", stats.mean_vx, stats.mean_vy));
            ui.label(format!("|v|: {:.4}", stats.mean_speed));
        });
}
