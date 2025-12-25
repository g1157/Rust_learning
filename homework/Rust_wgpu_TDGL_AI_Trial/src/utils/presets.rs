//! Configuration presets system

use crate::ui::panels::params_panel::UiParams;

/// Preset configuration
#[derive(Clone, Debug)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub flux_n: i32,
    pub kappa: f32,
    pub defect_mode_lattice: bool,
    pub defect_count: usize,
    pub defect_spacing: i32,
    pub alpha_defect: f32,
    pub defect_radius: i32,
}

impl Preset {
    /// Apply preset to UiParams
    pub fn apply_to(&self, params: &mut UiParams) {
        params.flux_n = self.flux_n;
        params.kappa = self.kappa;
        params.defect_mode_lattice = self.defect_mode_lattice;
        params.defect_count = self.defect_count;
        params.defect_spacing = self.defect_spacing;
        params.alpha_defect = self.alpha_defect;
        params.defect_radius = self.defect_radius;
    }
}

/// Built-in presets
pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "默认".to_string(),
            description: "标准配置，适合一般研究".to_string(),
            flux_n: 209,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 50,
            defect_spacing: 32,
            alpha_defect: -0.5,
            defect_radius: 3,
        },
        Preset {
            name: "高磁场".to_string(),
            description: "高磁通量子数，更多涡旋".to_string(),
            flux_n: 500,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 100,
            defect_spacing: 32,
            alpha_defect: -0.5,
            defect_radius: 3,
        },
        Preset {
            name: "无缺陷".to_string(),
            description: "无钉扎势，自由涡旋运动".to_string(),
            flux_n: 209,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 0,
            defect_spacing: 32,
            alpha_defect: 0.0,
            defect_radius: 0,
        },
        Preset {
            name: "周期阵列".to_string(),
            description: "规则缺陷阵列，研究匹配效应".to_string(),
            flux_n: 209,
            kappa: 0.0,
            defect_mode_lattice: true,
            defect_count: 50,
            defect_spacing: 32,
            alpha_defect: -0.5,
            defect_radius: 3,
        },
        Preset {
            name: "强钉扎".to_string(),
            description: "强钉扎势，高 depinning 阈值".to_string(),
            flux_n: 209,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 100,
            defect_spacing: 32,
            alpha_defect: -1.0,
            defect_radius: 4,
        },
        Preset {
            name: "低磁场".to_string(),
            description: "低磁通量子数，少量涡旋".to_string(),
            flux_n: 50,
            kappa: 0.0,
            defect_mode_lattice: false,
            defect_count: 20,
            defect_spacing: 32,
            alpha_defect: -0.5,
            defect_radius: 3,
        },
    ]
}

/// Presets state for UI
#[derive(Clone, Debug)]
pub struct PresetsState {
    pub presets: Vec<Preset>,
    pub selected_index: Option<usize>,
}

impl Default for PresetsState {
    fn default() -> Self {
        Self {
            presets: builtin_presets(),
            selected_index: None,
        }
    }
}

impl PresetsState {
    /// Get selected preset
    pub fn selected(&self) -> Option<&Preset> {
        self.selected_index.and_then(|i| self.presets.get(i))
    }
}
