//! Validation panel for comparing simulation results with theoretical/experimental data

use egui::{Ui, CollapsingHeader, ComboBox};
use crate::ui::theme::{SUCCESS, WARNING, ERROR, ACCENT, FG_SECONDARY};
use crate::utils::materials::{MATERIALS, MaterialParams};

/// Physical constants
pub const PHI_0: f64 = 2.067833848e-15;  // Magnetic flux quantum (Wb)

/// Validation status based on deviation percentage
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ValidationStatus {
    Good,      // < 5% deviation
    Warning,   // 5-15% deviation
    Poor,      // > 15% deviation
    #[default]
    Unknown,   // Not enough data
}

impl ValidationStatus {
    pub fn from_deviation(deviation_pct: f32) -> Self {
        if deviation_pct < 5.0 {
            ValidationStatus::Good
        } else if deviation_pct < 15.0 {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Poor
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            ValidationStatus::Good => SUCCESS,
            ValidationStatus::Warning => WARNING,
            ValidationStatus::Poor => ERROR,
            ValidationStatus::Unknown => FG_SECONDARY,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ValidationStatus::Good => "✓",
            ValidationStatus::Warning => "⚠",
            ValidationStatus::Poor => "✗",
            ValidationStatus::Unknown => "?",
        }
    }
}

/// Lattice symmetry type
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LatticeSymmetry {
    Hexagonal,    // Ideal Abrikosov lattice
    Square,       // Square lattice
    Disordered,   // No clear symmetry
    #[default]
    Unknown,      // Not enough vortices
}

impl LatticeSymmetry {
    pub fn name(&self) -> &'static str {
        match self {
            LatticeSymmetry::Hexagonal => "六角晶格",
            LatticeSymmetry::Square => "四方晶格",
            LatticeSymmetry::Disordered => "无序",
            LatticeSymmetry::Unknown => "未知",
        }
    }
}

/// Matching field status
#[derive(Clone, Debug)]
pub struct MatchingFieldStatus {
    pub ratio: f32,              // n_vortices / n_defects
    pub is_matched: bool,        // Within tolerance of integer ratio
    pub match_type: Option<(i32, i32)>,  // e.g., (1, 1) for 1:1, (2, 1) for 2:1
}

impl Default for MatchingFieldStatus {
    fn default() -> Self {
        Self {
            ratio: 0.0,
            is_matched: false,
            match_type: None,
        }
    }
}

impl MatchingFieldStatus {
    /// Check for matching field condition
    pub fn calculate(n_vortices: i32, n_defects: i32, tolerance: f32) -> Self {
        if n_defects == 0 {
            return Self::default();
        }

        let ratio = n_vortices as f32 / n_defects as f32;

        // Check common matching ratios
        let ratios_to_check = [
            (1, 1), (2, 1), (1, 2), (3, 1), (1, 3), (2, 3), (3, 2),
        ];

        for (num, den) in ratios_to_check {
            let target = num as f32 / den as f32;
            if (ratio - target).abs() < tolerance {
                return Self {
                    ratio,
                    is_matched: true,
                    match_type: Some((num, den)),
                };
            }
        }

        Self {
            ratio,
            is_matched: false,
            match_type: None,
        }
    }
}

/// Validation data storage
#[derive(Clone, Debug, Default)]
pub struct ValidationData {
    // Lattice validation
    pub theoretical_spacing: f32,    // Theoretical lattice spacing based on flux_n
    pub expected_spacing: f32,       // Expected spacing based on actual vortex count
    pub measured_spacing: f32,       // Measured mean nearest neighbor distance
    pub spacing_deviation_pct: f32,  // Deviation from expected (actual vortex count)
    pub lattice_status: ValidationStatus,
    pub lattice_symmetry: LatticeSymmetry,
    pub flux_n: i32,                 // Target vortex count
    pub actual_vortices: i32,        // Actual positive vortex count

    // Matching field
    pub matching_status: MatchingFieldStatus,

    // Depinning (β exponent)
    pub beta_exponent: Option<f32>,
    pub beta_r_squared: Option<f32>,
    pub beta_status: ValidationStatus,

    // Energy validation
    pub energy_density: f64,
    pub expected_condensation_energy: f64,  // ~ -0.5 in normalized units

    // Material reference
    pub selected_material_index: usize,

    // Update tracking
    pub last_update_step: u64,
}

impl ValidationData {
    /// Calculate theoretical lattice spacing for triangular Abrikosov lattice
    /// a₀ = 1.075 × √(Φ₀/B) in real units
    /// In simulation units: a₀ = √(2 × Area / (√3 × n_vortices))
    pub fn calculate_theoretical_spacing(n_vortices: i32, nx: u32, ny: u32) -> f32 {
        if n_vortices <= 0 {
            return 0.0;
        }
        let area = (nx * ny) as f32;
        // For triangular lattice: a = √(2A / (√3 × N))
        (2.0 * area / (3.0_f32.sqrt() * n_vortices as f32)).sqrt()
    }

    /// Calculate mean nearest neighbor distance from vortex positions
    /// Returns median instead of mean for robustness against outliers
    pub fn calculate_measured_spacing(positions: &[(f32, f32)], nx: u32, ny: u32) -> f32 {
        if positions.len() < 2 {
            return 0.0;
        }

        let nx_f = nx as f32;
        let ny_f = ny as f32;

        let mut distances: Vec<f32> = Vec::with_capacity(positions.len());

        for (i, &(x1, y1)) in positions.iter().enumerate() {
            let mut min_dist = f32::MAX;

            for (j, &(x2, y2)) in positions.iter().enumerate() {
                if i != j {
                    // Minimum image convention for periodic boundaries
                    let mut dx = (x2 - x1).abs();
                    let mut dy = (y2 - y1).abs();

                    if dx > nx_f / 2.0 { dx = nx_f - dx; }
                    if dy > ny_f / 2.0 { dy = ny_f - dy; }

                    let dist = (dx * dx + dy * dy).sqrt();
                    min_dist = min_dist.min(dist);
                }
            }

            if min_dist < f32::MAX {
                distances.push(min_dist);
            }
        }

        if distances.is_empty() {
            return 0.0;
        }

        // Sort and compute statistics
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = distances.len();
        let median = if n % 2 == 0 {
            (distances[n / 2 - 1] + distances[n / 2]) / 2.0
        } else {
            distances[n / 2]
        };
        let mean = distances.iter().sum::<f32>() / n as f32;
        let min_val = distances[0];
        let max_val = distances[n - 1];

        log::info!(
            "Spacing stats: n={}, mean={:.2}, median={:.2}, min={:.2}, max={:.2}",
            n, mean, median, min_val, max_val
        );

        median  // Return median for robustness
    }

    /// Detect lattice symmetry from vortex positions
    /// Uses spatial hashing for O(n) average complexity
    pub fn detect_symmetry(positions: &[(f32, f32)], nx: u32, ny: u32) -> LatticeSymmetry {
        if positions.len() < 7 {
            return LatticeSymmetry::Unknown;
        }

        // For large vortex counts, sample a subset for efficiency
        let max_samples = 100;
        let sample_step = if positions.len() > max_samples {
            positions.len() / max_samples
        } else {
            1
        };

        let nx_f = nx as f32;
        let ny_f = ny as f32;

        // Build spatial hash grid for neighbor lookup
        let area = nx_f * ny_f;
        let estimated_spacing = (area / positions.len() as f32).sqrt();
        let cell_size = estimated_spacing * 3.0; // Search radius for 6 neighbors

        let grid_nx = ((nx_f / cell_size).ceil() as usize).max(1);
        let grid_ny = ((ny_f / cell_size).ceil() as usize).max(1);

        let mut grid: Vec<Vec<usize>> = vec![Vec::new(); grid_nx * grid_ny];
        for (idx, &(x, y)) in positions.iter().enumerate() {
            let gx = ((x / cell_size) as usize).min(grid_nx - 1);
            let gy = ((y / cell_size) as usize).min(grid_ny - 1);
            grid[gy * grid_nx + gx].push(idx);
        }

        let mut angle_histogram = [0u32; 12];  // 30-degree bins

        for (i, &(x1, y1)) in positions.iter().enumerate().step_by(sample_step) {
            let gx = ((x1 / cell_size) as usize).min(grid_nx - 1);
            let gy = ((y1 / cell_size) as usize).min(grid_ny - 1);

            // Collect neighbors from nearby cells
            let mut neighbors: Vec<(f32, f32, f32)> = Vec::with_capacity(20);

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let ngx = ((gx as i32 + dx).rem_euclid(grid_nx as i32)) as usize;
                    let ngy = ((gy as i32 + dy).rem_euclid(grid_ny as i32)) as usize;

                    for &j in &grid[ngy * grid_nx + ngx] {
                        if i != j {
                            let (x2, y2) = positions[j];
                            let mut ddx = x2 - x1;
                            let mut ddy = y2 - y1;

                            if ddx > nx_f / 2.0 { ddx -= nx_f; }
                            if ddx < -nx_f / 2.0 { ddx += nx_f; }
                            if ddy > ny_f / 2.0 { ddy -= ny_f; }
                            if ddy < -ny_f / 2.0 { ddy += ny_f; }

                            let dist = (ddx * ddx + ddy * ddy).sqrt();
                            neighbors.push((ddx, ddy, dist));
                        }
                    }
                }
            }

            neighbors.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            // Take up to 6 nearest neighbors
            for &(ddx, ddy, _) in neighbors.iter().take(6) {
                let angle = ddy.atan2(ddx).to_degrees();
                let normalized = ((angle + 180.0) % 360.0) as usize;
                let bin = (normalized / 30) % 12;
                angle_histogram[bin] += 1;
            }
        }

        // Analyze histogram for symmetry
        let hex_bins = [0, 2, 4, 6, 8, 10];
        let square_bins = [0, 3, 6, 9];

        let hex_score: u32 = hex_bins.iter().map(|&b| angle_histogram[b]).sum();
        let square_score: u32 = square_bins.iter().map(|&b| angle_histogram[b]).sum();
        let total: u32 = angle_histogram.iter().sum();

        if total == 0 {
            return LatticeSymmetry::Unknown;
        }

        let hex_ratio = hex_score as f32 / total as f32;
        let square_ratio = square_score as f32 / total as f32;

        if hex_ratio > 0.6 {
            LatticeSymmetry::Hexagonal
        } else if square_ratio > 0.5 {
            LatticeSymmetry::Square
        } else {
            LatticeSymmetry::Disordered
        }
    }

    /// Update all validation metrics
    /// - flux_n: target vortex count from magnetic field (for theoretical spacing)
    /// - n_vortices: actual positive vortex count (for matching field)
    pub fn update(
        &mut self,
        vortex_positions: &[(f32, f32)],
        flux_n: i32,
        n_vortices: i32,
        n_defects: i32,
        nx: u32,
        ny: u32,
        energy_density: f64,
        step: u64,
    ) {
        self.flux_n = flux_n;
        self.actual_vortices = n_vortices;

        // Lattice spacing validation
        // Theoretical spacing based on flux_n (target vortex count from B field)
        self.theoretical_spacing = Self::calculate_theoretical_spacing(flux_n, nx, ny);
        // Expected spacing based on actual vortex count
        self.expected_spacing = Self::calculate_theoretical_spacing(n_vortices, nx, ny);
        // Measured spacing from actual vortex positions
        self.measured_spacing = Self::calculate_measured_spacing(vortex_positions, nx, ny);

        // Compare measured to expected (based on actual vortex count)
        if self.expected_spacing > 0.0 && self.measured_spacing > 0.0 {
            self.spacing_deviation_pct =
                ((self.measured_spacing - self.expected_spacing) / self.expected_spacing * 100.0).abs();
            self.lattice_status = ValidationStatus::from_deviation(self.spacing_deviation_pct);
        } else {
            self.spacing_deviation_pct = 0.0;
            self.lattice_status = ValidationStatus::Unknown;
        }

        // Lattice symmetry
        self.lattice_symmetry = Self::detect_symmetry(vortex_positions, nx, ny);

        // Matching field - compare actual vortices to defects
        self.matching_status = MatchingFieldStatus::calculate(n_vortices, n_defects, 0.1);

        // Energy
        self.energy_density = energy_density;
        self.expected_condensation_energy = -0.5;  // Normalized units

        self.last_update_step = step;
    }

    /// Update β exponent from depinning curve fit
    pub fn update_beta(&mut self, beta: f32, r_squared: f32) {
        self.beta_exponent = Some(beta);
        self.beta_r_squared = Some(r_squared);

        // Check if β is in expected range (0.5-0.65 for mean-field)
        if beta >= 0.4 && beta <= 0.75 {
            self.beta_status = ValidationStatus::Good;
        } else if beta >= 0.3 && beta <= 0.9 {
            self.beta_status = ValidationStatus::Warning;
        } else {
            self.beta_status = ValidationStatus::Poor;
        }
    }

    /// Get selected material reference
    pub fn selected_material(&self) -> &'static MaterialParams {
        &MATERIALS[self.selected_material_index.min(MATERIALS.len() - 1)]
    }
}

/// Draw the validation panel
pub fn draw_validation_panel(ui: &mut Ui, data: &mut ValidationData) {
    CollapsingHeader::new("🔬 验证与对比")
        .default_open(true)
        .show(ui, |ui| {
            // Material reference selector
            ui.horizontal(|ui| {
                ui.label("参考材料:");
                ComboBox::from_id_salt("material_selector")
                    .selected_text(data.selected_material().name)
                    .show_ui(ui, |ui| {
                        for (idx, mat) in MATERIALS.iter().enumerate() {
                            ui.selectable_value(&mut data.selected_material_index, idx, mat.name);
                        }
                    });
            });

            let material = data.selected_material();
            ui.label(format!(
                "κ: {:.1}-{:.1} | ξ: {:.1}nm | Tc: {:.1}K",
                material.kappa_min, material.kappa_max, material.xi_nm, material.tc_k
            ));

            ui.separator();

            // Lattice spacing validation
            ui.label("涡旋晶格验证:");
            ui.horizontal(|ui| {
                ui.label(format!("涡旋数: {} (目标: {})", data.actual_vortices, data.flux_n));
            });
            ui.horizontal(|ui| {
                ui.label("预期间距:");
                ui.colored_label(ACCENT, format!("{:.2}", data.expected_spacing));
                ui.label("实测间距:");
                ui.colored_label(ACCENT, format!("{:.2}", data.measured_spacing));
            });

            ui.horizontal(|ui| {
                ui.label("偏差:");
                ui.colored_label(
                    data.lattice_status.color(),
                    format!("{:.1}% {}", data.spacing_deviation_pct, data.lattice_status.icon()),
                );
                ui.label(format!("对称性: {}", data.lattice_symmetry.name()));
            });

            ui.separator();

            // Matching field status
            ui.label("Matching Field:");
            ui.horizontal(|ui| {
                ui.label("涡旋/缺陷比:");
                if data.matching_status.is_matched {
                    if let Some((num, den)) = data.matching_status.match_type {
                        ui.colored_label(SUCCESS, format!("{}:{} ✓", num, den));
                    }
                } else {
                    ui.colored_label(FG_SECONDARY, format!("{:.2}", data.matching_status.ratio));
                }
            });

            ui.separator();

            // Depinning β exponent
            ui.label("Depinning 临界指数:");
            if let Some(beta) = data.beta_exponent {
                ui.horizontal(|ui| {
                    ui.label("β =");
                    ui.colored_label(data.beta_status.color(), format!("{:.3}", beta));
                    if let Some(r2) = data.beta_r_squared {
                        ui.label(format!("(R² = {:.3})", r2));
                    }
                    ui.label(data.beta_status.icon());
                });
                ui.label("理论范围: 0.5-0.65 (mean-field)");
            } else {
                ui.colored_label(FG_SECONDARY, "运行 κ Sweep 后显示");
            }

            ui.separator();

            // Energy validation
            ui.label("能量验证:");
            ui.horizontal(|ui| {
                ui.label("能量密度:");
                ui.colored_label(ACCENT, format!("{:.4}", data.energy_density));
                ui.label(format!("(凝聚能 ≈ {:.1})", data.expected_condensation_energy));
            });
        });
}
