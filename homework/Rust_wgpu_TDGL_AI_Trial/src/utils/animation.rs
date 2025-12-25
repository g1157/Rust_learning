//! Animation utilities for UI effects

use egui::Color32;
use std::time::Instant;

/// Animation state for pulsing effects
#[derive(Clone, Debug)]
pub struct PulseAnimation {
    start_time: Instant,
    duration_secs: f32,
    min_brightness: f32,
    max_brightness: f32,
}

impl Default for PulseAnimation {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            duration_secs: 1.5,
            min_brightness: 0.7,
            max_brightness: 1.0,
        }
    }
}

impl PulseAnimation {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration_secs,
            ..Default::default()
        }
    }

    /// Get current brightness multiplier (0.0 to 1.0)
    pub fn brightness(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let phase = (elapsed / self.duration_secs) % 1.0;
        // Sine wave oscillation
        let t = (phase * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
        self.min_brightness + t * (self.max_brightness - self.min_brightness)
    }

    /// Apply brightness to a color
    pub fn apply_to_color(&self, color: Color32) -> Color32 {
        let brightness = self.brightness();
        Color32::from_rgba_unmultiplied(
            (color.r() as f32 * brightness) as u8,
            (color.g() as f32 * brightness) as u8,
            (color.b() as f32 * brightness) as u8,
            color.a(),
        )
    }
}

/// Animation state for value highlight effects
#[derive(Clone, Debug)]
pub struct HighlightAnimation {
    trigger_time: Option<Instant>,
    duration_ms: u32,
    highlight_color: Color32,
    normal_color: Color32,
}

impl HighlightAnimation {
    pub fn new(highlight_color: Color32, normal_color: Color32) -> Self {
        Self {
            trigger_time: None,
            duration_ms: 200,
            highlight_color,
            normal_color,
        }
    }

    /// Trigger the highlight animation
    pub fn trigger(&mut self) {
        self.trigger_time = Some(Instant::now());
    }

    /// Get current color based on animation state
    pub fn current_color(&self) -> Color32 {
        if let Some(trigger_time) = self.trigger_time {
            let elapsed_ms = trigger_time.elapsed().as_millis() as u32;
            if elapsed_ms < self.duration_ms {
                let t = elapsed_ms as f32 / self.duration_ms as f32;
                return lerp_color(self.highlight_color, self.normal_color, t);
            }
        }
        self.normal_color
    }

    /// Check if animation is active
    pub fn is_active(&self) -> bool {
        if let Some(trigger_time) = self.trigger_time {
            trigger_time.elapsed().as_millis() < self.duration_ms as u128
        } else {
            false
        }
    }
}

/// Linear interpolation between two colors
pub fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        lerp_u8(from.r(), to.r(), t),
        lerp_u8(from.g(), to.g(), t),
        lerp_u8(from.b(), to.b(), t),
        lerp_u8(from.a(), to.a(), t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

/// Animation state for fade-in effects
#[derive(Clone, Debug)]
pub struct FadeInAnimation {
    start_time: Instant,
    duration_ms: u32,
}

impl FadeInAnimation {
    pub fn new(duration_ms: u32) -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms,
        }
    }

    /// Get current opacity (0.0 to 1.0)
    pub fn opacity(&self) -> f32 {
        let elapsed_ms = self.start_time.elapsed().as_millis() as u32;
        if elapsed_ms >= self.duration_ms {
            1.0
        } else {
            elapsed_ms as f32 / self.duration_ms as f32
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed().as_millis() >= self.duration_ms as u128
    }
}

/// Animation state for scale effects (e.g., vortex appearance)
#[derive(Clone, Debug)]
pub struct ScaleAnimation {
    start_time: Instant,
    duration_ms: u32,
    start_scale: f32,
    end_scale: f32,
}

impl ScaleAnimation {
    pub fn new(duration_ms: u32, start_scale: f32, end_scale: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms,
            start_scale,
            end_scale,
        }
    }

    /// Get current scale factor
    pub fn scale(&self) -> f32 {
        let elapsed_ms = self.start_time.elapsed().as_millis() as u32;
        if elapsed_ms >= self.duration_ms {
            self.end_scale
        } else {
            let t = elapsed_ms as f32 / self.duration_ms as f32;
            // Ease-out cubic
            let t = 1.0 - (1.0 - t).powi(3);
            self.start_scale + (self.end_scale - self.start_scale) * t
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed().as_millis() >= self.duration_ms as u128
    }
}
