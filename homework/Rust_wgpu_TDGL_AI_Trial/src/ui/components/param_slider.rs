//! Parameter slider component with label, value display, and optional unit

use egui::{Ui, Response, DragValue};

/// Slider with label and computed value display
pub fn param_slider<'a>(
    ui: &mut Ui,
    label: &str,
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            DragValue::new(value)
                .range(range)
                .speed(0.01)
        )
    }).response
}

/// Integer slider
pub fn param_slider_i32(
    ui: &mut Ui,
    label: &str,
    value: &mut i32,
    range: std::ops::RangeInclusive<i32>,
) -> Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(DragValue::new(value).range(range))
    }).response
}

/// Slider with associated computed value display
pub fn param_slider_with_info(
    ui: &mut Ui,
    label: &str,
    value: &mut i32,
    range: std::ops::RangeInclusive<i32>,
    info: &str,
) -> Response {
    let resp = ui.horizontal(|ui| {
        ui.label(label);
        ui.add(DragValue::new(value).range(range))
    }).response;
    ui.label(info);
    resp
}
