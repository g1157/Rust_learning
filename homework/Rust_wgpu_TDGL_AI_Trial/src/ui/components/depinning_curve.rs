//! Depinning curve component for κ sweep visualization

use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints, Points, Text, PlotPoint};
use crate::ui::theme::{ACCENT, SUCCESS, WARNING, ERROR, FG_SECONDARY};

/// Data point for depinning curve
#[derive(Clone, Debug)]
pub struct DepinningPoint {
    pub kappa: f32,
    pub mean_speed: f32,
}

/// Power law fit result: v = A * (κ - κ_c)^β
#[derive(Clone, Debug, Default)]
pub struct PowerLawFit {
    pub beta: f32,           // Critical exponent
    pub amplitude: f32,      // Prefactor A
    pub r_squared: f32,      // Goodness of fit
    pub valid: bool,         // Whether fit is valid
}

/// Depinning curve data storage
#[derive(Clone, Debug, Default)]
pub struct DepinningCurveData {
    pub points: Vec<DepinningPoint>,
    pub kappa_c: Option<f32>,
    pub power_law_fit: Option<PowerLawFit>,
}

impl DepinningCurveData {
    pub fn clear(&mut self) {
        self.points.clear();
        self.kappa_c = None;
        self.power_law_fit = None;
    }

    pub fn add_point(&mut self, kappa: f32, mean_speed: f32) {
        self.points.push(DepinningPoint { kappa, mean_speed });
        // Auto-detect κ_c after adding point
        self.detect_kappa_c();
        // Auto-fit power law if we have κ_c
        self.fit_power_law();
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

    /// Fit power law: v = A * (κ - κ_c)^β using log-log linear regression
    fn fit_power_law(&mut self) {
        let kappa_c = match self.kappa_c {
            Some(k) => k,
            None => {
                self.power_law_fit = None;
                return;
            }
        };

        // Filter points where κ > κ_c and v > 0
        let log_data: Vec<(f64, f64)> = self.points.iter()
            .filter(|p| p.kappa > kappa_c && p.mean_speed > 1e-6)
            .map(|p| {
                let x = ((p.kappa - kappa_c) as f64).ln();
                let y = (p.mean_speed as f64).ln();
                (x, y)
            })
            .collect();

        if log_data.len() < 3 {
            self.power_law_fit = None;
            return;
        }

        // Linear regression: y = β*x + ln(A)
        let n = log_data.len() as f64;
        let sum_x: f64 = log_data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = log_data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = log_data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = log_data.iter().map(|(x, _)| x * x).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-10 {
            self.power_law_fit = None;
            return;
        }

        let beta = (n * sum_xy - sum_x * sum_y) / denom;
        let ln_a = (sum_y - beta * sum_x) / n;
        let amplitude = ln_a.exp();

        // Calculate R²
        let mean_y = sum_y / n;
        let ss_tot: f64 = log_data.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = log_data.iter()
            .map(|(x, y)| {
                let y_pred = beta * x + ln_a;
                (y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 1e-10 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };

        self.power_law_fit = Some(PowerLawFit {
            beta: beta as f32,
            amplitude: amplitude as f32,
            r_squared: r_squared as f32,
            valid: r_squared > 0.5 && beta > 0.0 && beta < 2.0,
        });
    }

    /// Get fitted curve points for plotting
    pub fn get_fit_curve(&self) -> Option<Vec<[f64; 2]>> {
        let kappa_c = self.kappa_c?;
        let fit = self.power_law_fit.as_ref()?;

        if !fit.valid {
            return None;
        }

        let max_kappa = self.points.iter()
            .map(|p| p.kappa)
            .fold(0.0f32, f32::max);

        let mut curve = Vec::new();
        let steps = 50;
        for i in 0..=steps {
            let kappa = kappa_c + (max_kappa - kappa_c) * (i as f32 / steps as f32);
            let dk = kappa - kappa_c;
            if dk > 0.0 {
                let v = fit.amplitude * dk.powf(fit.beta);
                curve.push([kappa as f64, v as f64]);
            }
        }

        Some(curve)
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Draw depinning curve plot with power law fit
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

            // Draw power law fit curve
            if let Some(fit_points) = data.get_fit_curve() {
                let fit_line = Line::new(PlotPoints::from_iter(fit_points))
                    .name("幂律拟合")
                    .color(SUCCESS)
                    .width(1.5)
                    .style(egui_plot::LineStyle::dashed_loose());
                plot_ui.line(fit_line);
            }

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

    // Show κ_c and fit results below the plot
    ui.horizontal(|ui| {
        if let Some(kappa_c) = data.kappa_c {
            ui.label("κ_c:");
            ui.colored_label(SUCCESS, format!("{:.4}", kappa_c));
            ui.separator();
        }

        if let Some(fit) = &data.power_law_fit {
            ui.label("β:");
            let beta_color = if fit.beta >= 0.4 && fit.beta <= 0.75 {
                SUCCESS
            } else if fit.beta >= 0.3 && fit.beta <= 0.9 {
                WARNING
            } else {
                ERROR
            };
            ui.colored_label(beta_color, format!("{:.3}", fit.beta));

            ui.label("R²:");
            let r2_color = if fit.r_squared > 0.9 {
                SUCCESS
            } else if fit.r_squared > 0.7 {
                WARNING
            } else {
                FG_SECONDARY
            };
            ui.colored_label(r2_color, format!("{:.3}", fit.r_squared));
        }
    });

    // Show theoretical reference
    if data.power_law_fit.is_some() {
        ui.colored_label(FG_SECONDARY, "理论: β = 0.5-0.65 (mean-field)");
    }
}
