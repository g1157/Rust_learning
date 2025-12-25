//! History panel for browsing past runs

use egui::{Ui, ScrollArea};
use std::path::{Path, PathBuf};
use std::fs;

/// Run history entry
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub dir_name: String,
    pub path: PathBuf,
    pub mode: String,
    pub flux_n: Option<i32>,
    pub timestamp: String,
}

/// History panel state
#[derive(Clone, Debug, Default)]
pub struct HistoryState {
    pub entries: Vec<HistoryEntry>,
    pub selected_index: Option<usize>,
    pub last_scan_time: Option<std::time::Instant>,
    pub config_preview: Option<String>,
}

impl HistoryState {
    /// Scan runs directory for history entries
    pub fn scan_runs_dir(&mut self, runs_dir: &Path) {
        self.entries.clear();
        self.selected_index = None;
        self.config_preview = None;

        if !runs_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(runs_dir) {
            let mut dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            // Sort by name (timestamp) descending
            dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

            for entry in dirs.into_iter().take(50) {
                let path = entry.path();
                let dir_name = entry.file_name().to_string_lossy().to_string();

                // Parse directory name for mode and timestamp
                let (mode, timestamp) = parse_dir_name(&dir_name);

                // Try to read flux_n from config.toml
                let flux_n = read_flux_n_from_config(&path);

                self.entries.push(HistoryEntry {
                    dir_name,
                    path,
                    mode,
                    flux_n,
                    timestamp,
                });
            }
        }

        self.last_scan_time = Some(std::time::Instant::now());
    }

    /// Load config preview for selected entry
    pub fn load_config_preview(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(entry) = self.entries.get(idx) {
                let config_path = entry.path.join("config.toml");
                if config_path.exists() {
                    self.config_preview = fs::read_to_string(&config_path).ok();
                } else {
                    self.config_preview = Some("config.toml 不存在".to_string());
                }
            }
        }
    }
}

/// Parse directory name to extract mode and timestamp
fn parse_dir_name(name: &str) -> (String, String) {
    // Expected format: mode_timestamp (e.g., "interactive_1703500000000")
    if let Some(idx) = name.find('_') {
        let mode = &name[..idx];
        let timestamp = &name[idx + 1..];
        (mode.to_string(), format_timestamp(timestamp))
    } else {
        (name.to_string(), String::new())
    }
}

/// Format timestamp from milliseconds to readable date
fn format_timestamp(ts_str: &str) -> String {
    if let Ok(ts) = ts_str.parse::<u64>() {
        // Convert milliseconds to seconds
        let secs = ts / 1000;
        // Simple formatting (without chrono dependency)
        format!("{}", secs)
    } else {
        ts_str.to_string()
    }
}

/// Read flux_n from config.toml
fn read_flux_n_from_config(dir: &Path) -> Option<i32> {
    let config_path = dir.join("config.toml");
    if let Ok(content) = fs::read_to_string(&config_path) {
        // Simple parsing for flux_n
        for line in content.lines() {
            if line.starts_with("flux_n") {
                if let Some(value) = line.split('=').nth(1) {
                    return value.trim().parse().ok();
                }
            }
        }
    }
    None
}

/// Draw the history panel
pub fn draw_history_panel(ui: &mut Ui, state: &mut HistoryState, runs_dir: &Path) {
    ui.horizontal(|ui| {
        ui.heading("运行历史");
        if ui.button("🔄 刷新").clicked() {
            state.scan_runs_dir(runs_dir);
        }
    });

    ui.separator();

    if state.entries.is_empty() {
        ui.label("暂无历史运行记录");
        ui.label("运行 headless 或 κ sweep 后将在此显示");
        return;
    }

    // Entry list - collect clicked index first
    let mut clicked_idx: Option<usize> = None;
    ScrollArea::vertical()
        .id_salt("history_entries_scroll")
        .max_height(200.0)
        .show(ui, |ui| {
            for (idx, entry) in state.entries.iter().enumerate() {
                let is_selected = state.selected_index == Some(idx);
                let response = ui.selectable_label(
                    is_selected,
                    format!(
                        "{} | {} {}",
                        entry.mode,
                        entry.flux_n.map(|n| format!("n={}", n)).unwrap_or_default(),
                        if entry.timestamp.is_empty() { "" } else { &entry.timestamp }
                    ),
                );

                if response.clicked() {
                    clicked_idx = Some(idx);
                }
            }
        });

    // Handle selection after iteration
    if let Some(idx) = clicked_idx {
        state.selected_index = Some(idx);
        state.load_config_preview();
    }

    // Config preview
    if let Some(preview) = &state.config_preview {
        ui.separator();
        ui.label("配置预览:");
        ScrollArea::vertical()
            .id_salt("config_preview_scroll")
            .max_height(150.0)
            .show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut preview.as_str())
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY));
            });
    }

    // Action buttons
    if state.selected_index.is_some() {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("📂 打开目录").clicked() {
                if let Some(idx) = state.selected_index {
                    if let Some(entry) = state.entries.get(idx) {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("explorer")
                                .arg(&entry.path)
                                .spawn();
                        }
                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open")
                                .arg(&entry.path)
                                .spawn();
                        }
                        #[cfg(target_os = "linux")]
                        {
                            let _ = std::process::Command::new("xdg-open")
                                .arg(&entry.path)
                                .spawn();
                        }
                    }
                }
            }
        });
    }
}
