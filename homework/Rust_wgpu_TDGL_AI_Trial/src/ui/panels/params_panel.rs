//! Left-side parameters panel

use egui::{Ui, CollapsingHeader, DragValue};
use crate::ui::theme::{SUCCESS, WARNING};

/// Simulation state enum
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimState {
    Idle,
    Running,
    Paused,
}

/// UI-editable parameters
#[derive(Clone, Debug)]
pub struct UiParams {
    pub flux_n: i32,
    pub kappa: f32,
    pub defect_mode_lattice: bool,
    pub defect_count: usize,
    pub defect_spacing: i32,
    pub alpha_defect: f32,
    pub defect_radius: i32,
    pub sim_state: SimState,
    pub reset_requested: bool,
}

impl Default for UiParams {
    fn default() -> Self {
        Self {
            flux_n: 209,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 50,
            defect_spacing: 32,
            alpha_defect: -0.5,
            defect_radius: 3,
            sim_state: SimState::Running,
            reset_requested: false,
        }
    }
}

/// Draw the parameters panel
pub fn draw_params_panel(
    ui: &mut Ui,
    params: &mut UiParams,
    step_count: u64,
    dt: f32,
    nx: u32,
    ny: u32,
    dx: f32,
    b_field: f32,
) {
    // Status display
    ui.heading("TDGL Simulator");
    ui.separator();
    ui.label(format!("Step: {}", step_count));
    ui.label(format!("Time: {:.2}", step_count as f32 * dt));
    ui.separator();

    // Simulation parameters
    CollapsingHeader::new("▼ 仿真参数")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(format!("Grid: {} × {}", nx, ny));
            ui.label(format!("dt: {:.4}  dx: {:.4}", dt, dx));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("flux_n:");
                let mut flux = params.flux_n;
                if ui.add(DragValue::new(&mut flux).range(0..=1000)).changed() {
                    params.flux_n = flux;
                }
            });
            ui.label(format!("≈ B = {:.6}", b_field));

            ui.horizontal(|ui| {
                ui.label("κ (drive):");
                ui.add(DragValue::new(&mut params.kappa).range(-0.1..=0.1).speed(0.001));
            });
        });

    // Defect configuration
    CollapsingHeader::new("▼ 缺陷配置")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("模式:");
                ui.radio_value(&mut params.defect_mode_lattice, false, "Random");
                ui.radio_value(&mut params.defect_mode_lattice, true, "Lattice");
            });

            if params.defect_mode_lattice {
                ui.horizontal(|ui| {
                    ui.label("间距:");
                    ui.add(DragValue::new(&mut params.defect_spacing).range(8..=128));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("数量:");
                    ui.add(DragValue::new(&mut params.defect_count).range(0..=500));
                });
            }

            ui.horizontal(|ui| {
                ui.label("α_defect:");
                ui.add(DragValue::new(&mut params.alpha_defect).range(-2.0..=0.0).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("半径:");
                ui.add(DragValue::new(&mut params.defect_radius).range(1..=10));
                ui.label("cells");
            });
        });

    ui.separator();

    // Run controls
    ui.horizontal(|ui| {
        match params.sim_state {
            SimState::Idle => {
                if ui.button("▶ 开始仿真").clicked() {
                    params.sim_state = SimState::Running;
                }
            }
            SimState::Running => {
                if ui.button("⏸ 暂停").clicked() {
                    params.sim_state = SimState::Paused;
                }
            }
            SimState::Paused => {
                if ui.button("▶ 继续").clicked() {
                    params.sim_state = SimState::Running;
                }
            }
        }

        if ui.button("🔄 重置").clicked() {
            params.reset_requested = true;
            params.sim_state = SimState::Idle;
        }
    });

    // State indicator
    ui.horizontal(|ui| {
        let (color, text) = match params.sim_state {
            SimState::Idle => (WARNING, "● 就绪"),
            SimState::Running => (SUCCESS, "● 运行中"),
            SimState::Paused => (WARNING, "● 已暂停"),
        };
        ui.colored_label(color, text);
    });
}
