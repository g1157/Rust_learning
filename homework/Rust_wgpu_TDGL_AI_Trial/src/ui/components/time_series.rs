//! Time series plot component with ring buffer

use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

const BUFFER_SIZE: usize = 1000;

/// Ring buffer for time series data
#[derive(Clone)]
pub struct RingBuffer {
    data: Vec<f64>,
    head: usize,
    len: usize,
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self {
            data: vec![0.0; BUFFER_SIZE],
            head: 0,
            len: 0,
        }
    }
}

impl RingBuffer {
    pub fn push(&mut self, value: f64) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % BUFFER_SIZE;
        if self.len < BUFFER_SIZE {
            self.len += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = [f64; 2]> + '_ {
        let start = if self.len < BUFFER_SIZE {
            0
        } else {
            self.head
        };
        (0..self.len).map(move |i| {
            let idx = (start + i) % BUFFER_SIZE;
            [i as f64, self.data[idx]]
        })
    }
}

/// Time series data storage
#[derive(Clone, Default)]
pub struct TimeSeriesData {
    pub vortices: RingBuffer,
    pub energy_density: RingBuffer,
}

impl TimeSeriesData {
    pub fn push(&mut self, vortices: i32, energy_density: f64) {
        self.vortices.push(vortices as f64);
        self.energy_density.push(energy_density);
    }
}

/// Draw time series plot
pub fn draw_time_series(ui: &mut Ui, data: &TimeSeriesData) {
    let vortex_points: PlotPoints = data.vortices.iter().collect();
    let vortex_line = Line::new(vortex_points)
        .name("涡旋数")
        .color(egui::Color32::from_rgb(0x4f, 0xc3, 0xf7));

    Plot::new("time_series")
        .height(120.0)
        .show_axes(true)
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .show(ui, |plot_ui| {
            plot_ui.line(vortex_line);
        });
}
