//! Material parameters reference library for superconductors

/// Material parameters for common superconductors
#[derive(Clone, Debug)]
pub struct MaterialParams {
    pub name: &'static str,
    pub kappa_min: f32,      // GL parameter κ = λ/ξ (min)
    pub kappa_max: f32,      // GL parameter κ = λ/ξ (max)
    pub xi_nm: f32,          // Coherence length ξ (nm)
    pub lambda_nm: f32,      // Penetration depth λ (nm)
    pub tc_k: f32,           // Critical temperature Tc (K)
    pub hc2_t: f32,          // Upper critical field Hc2 (T)
}

impl MaterialParams {
    /// Check if a given κ value is within the material's range
    pub fn kappa_in_range(&self, kappa: f32) -> bool {
        kappa >= self.kappa_min && kappa <= self.kappa_max
    }

    /// Get the typical κ value (midpoint)
    pub fn typical_kappa(&self) -> f32 {
        (self.kappa_min + self.kappa_max) / 2.0
    }
}

/// Built-in material database
pub const MATERIALS: &[MaterialParams] = &[
    MaterialParams {
        name: "Nb (铌)",
        kappa_min: 0.7,
        kappa_max: 1.0,
        xi_nm: 38.0,
        lambda_nm: 39.0,
        tc_k: 9.2,
        hc2_t: 0.4,
    },
    MaterialParams {
        name: "NbSe₂",
        kappa_min: 9.0,
        kappa_max: 12.0,
        xi_nm: 7.7,
        lambda_nm: 73.0,  // average of 69-77
        tc_k: 7.2,
        hc2_t: 4.5,
    },
    MaterialParams {
        name: "YBCO",
        kappa_min: 50.0,
        kappa_max: 100.0,
        xi_nm: 1.75,  // average of 1.5-2
        lambda_nm: 150.0,
        tc_k: 92.0,
        hc2_t: 100.0,
    },
    MaterialParams {
        name: "MgB₂",
        kappa_min: 20.0,
        kappa_max: 32.0,
        xi_nm: 7.5,  // average of 5-10
        lambda_nm: 140.0,
        tc_k: 39.0,
        hc2_t: 16.0,
    },
    MaterialParams {
        name: "Pb (铅)",
        kappa_min: 0.4,
        kappa_max: 0.55,
        xi_nm: 83.0,
        lambda_nm: 39.0,
        tc_k: 7.2,
        hc2_t: 0.08,
    },
    MaterialParams {
        name: "NbTi",
        kappa_min: 50.0,
        kappa_max: 80.0,
        xi_nm: 4.0,
        lambda_nm: 300.0,
        tc_k: 9.8,
        hc2_t: 15.0,
    },
];

/// Get material by index
pub fn get_material(index: usize) -> Option<&'static MaterialParams> {
    MATERIALS.get(index)
}

/// Get all material names for UI dropdown
pub fn material_names() -> Vec<&'static str> {
    MATERIALS.iter().map(|m| m.name).collect()
}
