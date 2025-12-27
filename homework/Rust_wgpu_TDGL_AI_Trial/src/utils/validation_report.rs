//! Validation report generation for TDGL simulation

use std::path::Path;
use std::fs;
use crate::ui::panels::validation_panel::{ValidationData, ValidationStatus, LatticeSymmetry};
use crate::ui::components::depinning_curve::DepinningCurveData;

/// Generate a Markdown validation report
pub fn generate_validation_report(
    validation: &ValidationData,
    depinning: &DepinningCurveData,
    run_config: &RunConfigSummary,
) -> String {
    let mut report = String::new();

    // Header
    report.push_str("# TDGL 模拟验证报告\n\n");
    report.push_str(&format!("> 生成时间: {}\n\n", chrono_lite_now()));
    report.push_str("---\n\n");

    // Run configuration
    report.push_str("## 运行配置\n\n");
    report.push_str("| 参数 | 值 |\n");
    report.push_str("|------|----|\n");
    report.push_str(&format!("| 网格尺寸 | {} × {} |\n", run_config.nx, run_config.ny));
    report.push_str(&format!("| flux_n | {} |\n", run_config.flux_n));
    report.push_str(&format!("| 缺陷数量 | {} |\n", run_config.defect_count));
    report.push_str(&format!("| 缺陷模式 | {} |\n", run_config.defect_mode));
    report.push_str(&format!("| κ (驱动) | {:.4} |\n", run_config.kappa));
    report.push_str(&format!("| 步数 | {} |\n", run_config.steps));
    report.push_str("\n");

    // Material reference
    let material = validation.selected_material();
    report.push_str("## 参考材料\n\n");
    report.push_str(&format!("**{}**\n\n", material.name));
    report.push_str("| 参数 | 值 |\n");
    report.push_str("|------|----|\n");
    report.push_str(&format!("| κ 范围 | {:.1} - {:.1} |\n", material.kappa_min, material.kappa_max));
    report.push_str(&format!("| ξ (相干长度) | {:.1} nm |\n", material.xi_nm));
    report.push_str(&format!("| λ (穿透深度) | {:.1} nm |\n", material.lambda_nm));
    report.push_str(&format!("| Tc | {:.1} K |\n", material.tc_k));
    report.push_str(&format!("| Hc2 | {:.1} T |\n", material.hc2_t));
    report.push_str("\n");

    // Lattice validation
    report.push_str("## 涡旋晶格验证\n\n");
    report.push_str("| 指标 | 值 | 状态 |\n");
    report.push_str("|------|----|----- |\n");
    report.push_str(&format!(
        "| 理论间距 | {:.2} (网格单位) | - |\n",
        validation.theoretical_spacing
    ));
    report.push_str(&format!(
        "| 实测间距 | {:.2} (网格单位) | - |\n",
        validation.measured_spacing
    ));
    report.push_str(&format!(
        "| 偏差 | {:.1}% | {} |\n",
        validation.spacing_deviation_pct,
        status_emoji(validation.lattice_status)
    ));
    report.push_str(&format!(
        "| 晶格对称性 | {} | - |\n",
        validation.lattice_symmetry.name()
    ));
    report.push_str("\n");

    // Lattice spacing formula
    report.push_str("### 理论公式\n\n");
    report.push_str("```\n");
    report.push_str("Abrikosov 三角晶格间距:\n");
    report.push_str("a₀ = √(2A / (√3 × N))\n");
    report.push_str("其中 A = Nx × Ny, N = 涡旋数\n");
    report.push_str("```\n\n");

    // Matching field
    report.push_str("## Matching Field 分析\n\n");
    report.push_str("| 指标 | 值 |\n");
    report.push_str("|------|----|\n");
    report.push_str(&format!(
        "| 涡旋/缺陷比 | {:.3} |\n",
        validation.matching_status.ratio
    ));
    if validation.matching_status.is_matched {
        if let Some((num, den)) = validation.matching_status.match_type {
            report.push_str(&format!("| 匹配状态 | ✅ {}:{} 匹配 |\n", num, den));
        }
    } else {
        report.push_str("| 匹配状态 | ❌ 未匹配 |\n");
    }
    report.push_str("\n");

    // Depinning analysis
    report.push_str("## Depinning 临界行为\n\n");
    if let Some(kappa_c) = depinning.kappa_c {
        report.push_str("| 指标 | 值 | 理论范围 |\n");
        report.push_str("|------|----|---------|\n");
        report.push_str(&format!("| κ_c (临界驱动力) | {:.4} | - |\n", kappa_c));

        if let Some(beta) = validation.beta_exponent {
            report.push_str(&format!(
                "| β (临界指数) | {:.3} {} | 0.5-0.65 |\n",
                beta,
                status_emoji(validation.beta_status)
            ));
        }
        if let Some(r2) = validation.beta_r_squared {
            report.push_str(&format!("| R² (拟合优度) | {:.4} | >0.9 |\n", r2));
        }
    } else {
        report.push_str("*未运行 κ Sweep，无 depinning 数据*\n");
    }
    report.push_str("\n");

    // Depinning formula
    report.push_str("### 理论公式\n\n");
    report.push_str("```\n");
    report.push_str("Depinning 幂律行为:\n");
    report.push_str("v ∝ (κ - κ_c)^β  当 κ > κ_c\n");
    report.push_str("\n");
    report.push_str("理论预期:\n");
    report.push_str("- β = 0.5 (mean-field)\n");
    report.push_str("- β = 0.5-0.65 (实验观测范围)\n");
    report.push_str("```\n\n");

    // Energy validation
    report.push_str("## 能量验证\n\n");
    report.push_str("| 指标 | 值 | 预期 |\n");
    report.push_str("|------|----|----- |\n");
    report.push_str(&format!(
        "| 能量密度 | {:.6} | - |\n",
        validation.energy_density
    ));
    report.push_str(&format!(
        "| 超导凝聚能 | {:.1} | ~-0.5 (归一化) |\n",
        validation.expected_condensation_energy
    ));
    report.push_str("\n");

    // Summary
    report.push_str("## 验证总结\n\n");
    let overall_status = determine_overall_status(validation);
    report.push_str(&format!("**整体验证状态: {}**\n\n", overall_status_text(overall_status)));

    report.push_str("### 验证清单\n\n");
    report.push_str(&format!(
        "- [{}] 涡旋晶格间距偏差 < 10%\n",
        if validation.spacing_deviation_pct < 10.0 { "x" } else { " " }
    ));
    report.push_str(&format!(
        "- [{}] 晶格呈六角对称\n",
        if validation.lattice_symmetry == LatticeSymmetry::Hexagonal { "x" } else { " " }
    ));
    if let Some(beta) = validation.beta_exponent {
        report.push_str(&format!(
            "- [{}] β 指数在合理范围 (0.3-0.8)\n",
            if beta >= 0.3 && beta <= 0.8 { "x" } else { " " }
        ));
    }
    report.push_str("\n");

    // Footer
    report.push_str("---\n\n");
    report.push_str("*此报告由 TDGL Simulator 自动生成*\n");

    report
}

/// Run configuration summary for report
#[derive(Clone, Debug)]
pub struct RunConfigSummary {
    pub nx: u32,
    pub ny: u32,
    pub flux_n: i32,
    pub defect_count: usize,
    pub defect_mode: String,
    pub kappa: f32,
    pub steps: u64,
}

/// Save validation report to file
pub fn save_validation_report(
    report: &str,
    run_dir: &Path,
) -> std::io::Result<()> {
    let report_path = run_dir.join("validation_report.md");
    fs::write(&report_path, report)?;
    Ok(())
}

fn status_emoji(status: ValidationStatus) -> &'static str {
    match status {
        ValidationStatus::Good => "✅",
        ValidationStatus::Warning => "⚠️",
        ValidationStatus::Poor => "❌",
        ValidationStatus::Unknown => "❓",
    }
}

fn determine_overall_status(validation: &ValidationData) -> ValidationStatus {
    // Combine all validation statuses
    let statuses = [
        validation.lattice_status,
        validation.beta_status,
    ];

    if statuses.iter().any(|s| *s == ValidationStatus::Poor) {
        ValidationStatus::Poor
    } else if statuses.iter().any(|s| *s == ValidationStatus::Warning) {
        ValidationStatus::Warning
    } else if statuses.iter().all(|s| *s == ValidationStatus::Good) {
        ValidationStatus::Good
    } else {
        ValidationStatus::Unknown
    }
}

fn overall_status_text(status: ValidationStatus) -> &'static str {
    match status {
        ValidationStatus::Good => "✅ 通过 - 模拟结果与理论预期一致",
        ValidationStatus::Warning => "⚠️ 部分通过 - 存在轻微偏差",
        ValidationStatus::Poor => "❌ 未通过 - 存在显著偏差，请检查参数",
        ValidationStatus::Unknown => "❓ 数据不足 - 需要更多数据进行验证",
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple formatting: just return Unix timestamp
    format!("Unix timestamp: {}", secs)
}
