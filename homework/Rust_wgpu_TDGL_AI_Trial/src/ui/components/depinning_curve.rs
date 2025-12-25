//! Depinning curve component for κ sweep visualization

use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints, Points, Text, PlotPoint};
use crate::ui::theme::{ACCENT, SUCCESS, WARNING};

/// Data point for depinning curve
#[derive(Clone, Debug)]
pub struct DepinningPoint {
    pub kappa: f32,
    pub mean_speed: f32,
}

/// Depinning curve data storage
#[derive(Clone, Debug, Default)]
pub struct DepinningCurveData {
    pub points: Vec<DepinningPoint>,
    pub kappa_c: Option<f32>,
}

impl DepinningCurveData {
    pub fn clear(&mut self) {
        self.points.clear();
        self.kappa_c = None;
    }

    pub fn add_point(&mut self, kappa: f32, mean_speed: f32) {
        self.points.push(DepinningPoint { kappa, mean_speed });
        // Auto-detect κ_c after adding point
        self.detect_kappa_c();
    }

    /// Detect κ_c using adaptive threshold method
    /// κ_c is where speed first exceeds 10% of max speed observed
    fn detect_kappa_c(&mut self) {
        if self.points.len() < 2 {
            self.kappa_c = None;
            return;
        }

        // Find max speed
        let max_speed = self.points.iter()
            .map(|p| p.mean_speed)
            .fold(0.0f32, f32::max);

        // Use 10% of max speed as threshold, with minimum of 0.01
        let threshold = (max_speed * 0.1).max(0.01);

        // Find first point exceeding threshold
        self.kappa_c = self.points.iter()
            .find(|p| p.mean_speed > threshold)
            .map(|p| p.kappa);
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Draw depinning curve plot
pub fn draw_depinning_curve(ui: &mut Ui, data: &DepinningCurveData) {
    if data.is_empty() {
        ui.label("暂无数据，启动 κ Sweep 后显示");
        return;
    }

    // Convert data to plot points
    let plot_points: PlotPoints = data.points.iter()
        .map(|p| [p.kappa as f64, p.mean_speed as f64])
        .collect();

    let line = Line::new(plot_points)
        .name("Depinning 曲线")
        .color(ACCENT)
        .width(2.0);

    // Create scatter points
    let scatter_points: PlotPoints = data.points.iter()
        .map(|p| [p.kappa as f64, p.mean_speed as f64])
        .collect();
    let scatter = Points::new(scatter_points)
        .name("数据点")
        .color(ACCENT)
        .radius(4.0);

    Plot::new("depinning_curve")
        .height(150.0)
        .show_axes(true)
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_label("κ")
        .y_axis_label("|v|")
        .show(ui, |plot_ui| {
            plot_ui.line(line);
            plot_ui.points(scatter);

            // Mark κ_c if detected
            if let Some(kappa_c) = data.kappa_c {
                // Vertical line at κ_c
                let kappa_c_line = Line::new(PlotPoints::from_iter([
                    [kappa_c as f64, 0.0],
                    [kappa_c as f64, data.points.iter().map(|p| p.mean_speed).fold(0.0f32, f32::max) as f64 * 1.2],
                ]))
                .name(format!("κ_c = {:.4}", kappa_c))
                .color(WARNING)
                .width(1.5)
                .style(egui_plot::LineStyle::dashed_dense());
                plot_ui.line(kappa_c_line);

                // Text label for κ_c
                let max_speed = data.points.iter().map(|p| p.mean_speed).fold(0.0f32, f32::max);
                plot_ui.text(Text::new(
                    PlotPoint::new(kappa_c as f64, max_speed as f64 * 0.9),
                    format!("κ_c={:.4}", kappa_c),
                ).color(WARNING));
            }
        });

    // Show κ_c value below the plot
    if let Some(kappa_c) = data.kappa_c {
        ui.horizontal(|ui| {
            ui.label("临界驱动力:");
            ui.colored_label(SUCCESS, format!("κ_c = {:.4}", kappa_c));
        });
    }
}
