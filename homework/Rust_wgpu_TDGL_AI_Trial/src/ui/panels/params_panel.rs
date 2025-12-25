//! Left-side parameters panel

use egui::{Ui, CollapsingHeader, DragValue, ComboBox};
use crate::ui::theme::{SUCCESS, WARNING, ACCENT};
use crate::utils::presets::PresetsState;

/// Simulation state enum
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimState {
    Idle,
    Running,
    Paused,
    KappaSweep,
}

/// κ Sweep configuration
#[derive(Clone, Debug)]
pub struct KappaSweepParams {
    pub kappa_start: f32,
    pub kappa_end: f32,
    pub kappa_step: f32,
    pub relax_steps: u64,
    pub measure_steps: u64,
    pub current_kappa: f32,
    pub current_phase: String,
    pub progress: f32,
}

impl Default for KappaSweepParams {
    fn default() -> Self {
        Self {
            kappa_start: 0.0,
            kappa_end: 0.03,
            kappa_step: 0.002,
            relax_steps: 1000,
            measure_steps: 2000,
            current_kappa: 0.0,
            current_phase: String::new(),
            progress: 0.0,
        }
    }
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
    pub sweep_params: KappaSweepParams,
    pub start_sweep_requested: bool,
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
            sim_state: SimState::Idle,  // 默认就绪/重置状态
            reset_requested: false,
            sweep_params: KappaSweepParams::default(),
            start_sweep_requested: false,
        }
    }
}

/// Draw the parameters panel
pub fn draw_params_panel(
    ui: &mut Ui,
    params: &mut UiParams,
    presets_state: &mut PresetsState,
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

    // Presets - 放在仿真参数上面
    CollapsingHeader::new("配置预设")
        .default_open(true)
        .show(ui, |ui| {
            draw_presets_panel_inner(ui, params, presets_state);
        });

    // Simulation parameters
    CollapsingHeader::new("仿真参数")
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
    CollapsingHeader::new("缺陷配置")
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
            SimState::KappaSweep => {
                // κ Sweep 模式下显示停止按钮
                if ui.button("⏹ 停止").clicked() {
                    params.sim_state = SimState::Idle;
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
            SimState::KappaSweep => (ACCENT, "● κ Sweep"),
        };
        ui.colored_label(color, text);
    });

    ui.separator();

    // κ Sweep configuration
    CollapsingHeader::new("κ Sweep 扫参")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("κ_start:");
                ui.add(DragValue::new(&mut params.sweep_params.kappa_start)
                    .range(-0.1..=0.1)
                    .speed(0.001));
            });

            ui.horizontal(|ui| {
                ui.label("κ_end:");
                ui.add(DragValue::new(&mut params.sweep_params.kappa_end)
                    .range(-0.1..=0.1)
                    .speed(0.001));
            });

            ui.horizontal(|ui| {
                ui.label("κ_step:");
                ui.add(DragValue::new(&mut params.sweep_params.kappa_step)
                    .range(0.001..=0.05)
                    .speed(0.001));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("弛豫步数:");
                ui.add(DragValue::new(&mut params.sweep_params.relax_steps)
                    .range(100..=50000));
            });

            ui.horizontal(|ui| {
                ui.label("测量步数:");
                ui.add(DragValue::new(&mut params.sweep_params.measure_steps)
                    .range(100..=50000));
            });

            // Calculate total steps and ETA
            let num_kappa_values = ((params.sweep_params.kappa_end - params.sweep_params.kappa_start)
                / params.sweep_params.kappa_step).ceil() as u64 + 1;
            let total_steps = num_kappa_values * (params.sweep_params.relax_steps + params.sweep_params.measure_steps);
            ui.label(format!("κ 值数量: {}", num_kappa_values));
            ui.label(format!("总步数: {}", total_steps));

            ui.separator();

            // Sweep control buttons
            match params.sim_state {
                SimState::KappaSweep => {
                    // Show progress during sweep
                    ui.horizontal(|ui| {
                        ui.label("当前 κ:");
                        ui.colored_label(ACCENT, format!("{:.4}", params.sweep_params.current_kappa));
                    });
                    ui.label(format!("阶段: {}", params.sweep_params.current_phase));

                    // Progress bar
                    let progress = params.sweep_params.progress;
                    ui.add(egui::ProgressBar::new(progress)
                        .text(format!("{:.1}%", progress * 100.0)));

                    if ui.button("⏹ 停止 Sweep").clicked() {
                        params.sim_state = SimState::Idle;
                    }
                }
                _ => {
                    if ui.button("▶ 开始 κ Sweep").clicked() {
                        params.start_sweep_requested = true;
                        params.sim_state = SimState::KappaSweep;
                    }
                }
            }
        });
}

/// Draw presets selection (internal)
fn draw_presets_panel_inner(ui: &mut Ui, params: &mut UiParams, presets_state: &mut PresetsState) {
    ui.horizontal(|ui| {
        ui.label("预设:");

        let current_name = presets_state.selected_index
            .and_then(|i| presets_state.presets.get(i))
            .map(|p| p.name.as_str())
            .unwrap_or("选择预设...");

        ComboBox::from_id_salt("preset_selector")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for (idx, preset) in presets_state.presets.iter().enumerate() {
                    let is_selected = presets_state.selected_index == Some(idx);
                    if ui.selectable_label(is_selected, &preset.name).clicked() {
                        presets_state.selected_index = Some(idx);
                    }
                }
            });
    });

    // Show description and apply button
    if let Some(preset) = presets_state.selected() {
        ui.label(format!("📝 {}", preset.description));
        if ui.button("应用预设").clicked() {
            if let Some(preset) = presets_state.selected().cloned() {
                preset.apply_to(params);
            }
        }
    }
}
