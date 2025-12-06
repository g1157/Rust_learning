//! 性能监控模块
//!
//! 提供帧率、内存使用、实体数量等性能指标的实时监控
//! 支持 puffin 性能分析器集成

use macroquad::prelude::*;
use std::collections::VecDeque;
use std::time::Instant;

#[cfg(all(not(target_arch = "wasm32"), feature = "profiling"))]
use puffin;

/// 性能指标结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    /// 当前帧率
    pub fps: f32,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// 实体总数
    pub entity_count: usize,
    /// 子弹数量
    pub bullet_count: usize,
    /// 小行星数量
    pub asteroid_count: usize,
    /// 粒子数量
    pub particle_count: usize,
    /// 内存使用（MB）
    pub memory_usage_mb: f32,
    /// 网络延迟（毫秒，仅在线模式）
    pub network_latency_ms: Option<f32>,
    /// 最大帧时间（毫秒）
    pub max_frame_time_ms: f32,
    /// Hitch计数（帧时间>50ms）
    pub hitch_count: u32,
    /// 总帧数
    pub total_frames: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            entity_count: 0,
            bullet_count: 0,
            asteroid_count: 0,
            particle_count: 0,
            memory_usage_mb: 0.0,
            network_latency_ms: None,
            max_frame_time_ms: 0.0,
            hitch_count: 0,
            total_frames: 0,
        }
    }
}

/// 性能历史记录（用于计算平均值）
pub struct PerformanceHistory {
    fps_history: VecDeque<f32>,
    frame_time_history: VecDeque<f32>,
    max_samples: usize,
}

impl PerformanceHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            fps_history: VecDeque::with_capacity(max_samples),
            frame_time_history: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn add_sample(&mut self, fps: f32, frame_time_ms: f32) {
        if self.fps_history.len() >= self.max_samples {
            self.fps_history.pop_front();
            self.frame_time_history.pop_front();
        }
        self.fps_history.push_back(fps);
        self.frame_time_history.push_back(frame_time_ms);
    }

    pub fn average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            0.0
        } else {
            self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
        }
    }

    pub fn average_frame_time(&self) -> f32 {
        if self.frame_time_history.is_empty() {
            0.0
        } else {
            self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32
        }
    }

    pub fn min_fps(&self) -> f32 {
        self.fps_history
            .iter()
            .fold(f32::INFINITY, |a, &b| a.min(b))
    }

    #[allow(dead_code)]
    pub fn max_fps(&self) -> f32 {
        self.fps_history.iter().fold(0.0, |a, &b| a.max(b))
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    pub metrics: PerformanceMetrics,
    pub history: PerformanceHistory,
    last_frame_time: Instant,
    frame_count: u64,
    show_overlay: bool,
    overlay_position: Vec2,
    export_path: Option<String>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::default(),
            history: PerformanceHistory::new(60), // 60帧历史
            last_frame_time: Instant::now(),
            frame_count: 0,
            show_overlay: false,
            overlay_position: Vec2::new(10.0, 10.0),
            export_path: None,
        }
    }

    pub fn with_export_path(mut self, path: String) -> Self {
        self.export_path = Some(path);
        self
    }

    /// 更新性能指标
    pub fn update(&mut self, entity_counts: (usize, usize, usize, usize)) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        let frame_time_ms = delta_time.as_secs_f32() * 1000.0;
        let fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };

        // Hitch检测（帧时间>50ms）
        if frame_time_ms > 50.0 {
            self.metrics.hitch_count += 1;
        }

        // 更新最大帧时间
        self.metrics.max_frame_time_ms = self.metrics.max_frame_time_ms.max(frame_time_ms);

        // 更新当前指标
        self.metrics.fps = fps;
        self.metrics.frame_time_ms = frame_time_ms;
        self.metrics.entity_count = entity_counts.0;
        self.metrics.bullet_count = entity_counts.1;
        self.metrics.asteroid_count = entity_counts.2;
        self.metrics.particle_count = entity_counts.3;
        self.metrics.total_frames = self.frame_count;

        // 估算内存使用（简化版）
        self.metrics.memory_usage_mb = (self.metrics.entity_count as f32 * 0.1).max(10.0);

        // 更新历史记录
        self.history.add_sample(fps, frame_time_ms);

        self.frame_count += 1;

        // 每60帧输出一次日志
        if self.frame_count.is_multiple_of(60) {
            println!(
                "Performance: FPS={:.1} (avg={:.1}, min={:.1}), Frame={:.2}ms (avg={:.2}ms), Entities={}, Hitches={}",
                fps,
                self.history.average_fps(),
                self.history.min_fps(),
                frame_time_ms,
                self.history.average_frame_time(),
                self.metrics.entity_count,
                self.metrics.hitch_count
            );
        }
    }

    /// 检查性能预算
    #[allow(dead_code)]
    pub fn check_performance_budget(&self) -> bool {
        // 目标：平均帧率 >= 55 FPS，最大帧时间 <= 33ms，Hitch < 3
        self.history.average_fps() >= 55.0 &&
        self.history.average_frame_time() <= 18.2 && // ~55 FPS
        self.metrics.hitch_count < 3
    }

    /// 导出性能指标到文件
    pub fn export_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.export_path {
            let json = serde_json::to_string_pretty(&self.metrics)?;
            std::fs::write(path, json)?;
            println!("Performance metrics exported to: {}", path);
        }
        Ok(())
    }

    /// 设置网络延迟
    #[allow(dead_code)]
    pub fn set_network_latency(&mut self, latency_ms: f32) {
        self.metrics.network_latency_ms = Some(latency_ms);
    }

    /// 切换性能覆盖层显示
    #[allow(dead_code)]
    pub fn toggle_overlay(&mut self) {
        self.show_overlay = !self.show_overlay;
    }

    /// 设置覆盖层位置
    #[allow(dead_code)]
    pub fn set_overlay_position(&mut self, position: Vec2) {
        self.overlay_position = position;
    }

    /// 绘制性能覆盖层
    pub fn draw_overlay(&self, font: Option<&Font>) {
        if !self.show_overlay {
            return;
        }

        let mut y_offset = self.overlay_position.y;
        let line_height = 20.0;

        // 背景框
        let overlay_width = 300.0;
        let overlay_height = 200.0;
        draw_rectangle(
            self.overlay_position.x - 5.0,
            self.overlay_position.y - 5.0,
            overlay_width,
            overlay_height,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );

        // 标题
        draw_text_ex(
            "Performance Monitor (F3)",
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 16,
                color: WHITE,
                ..Default::default()
            },
        );
        y_offset += line_height + 5.0;

        // FPS信息
        let fps_color = if self.metrics.fps >= 55.0 {
            GREEN
        } else if self.metrics.fps >= 30.0 {
            YELLOW
        } else {
            RED
        };

        draw_text_ex(
            &format!(
                "FPS: {:.1} (avg: {:.1}, min: {:.1})",
                self.metrics.fps,
                self.history.average_fps(),
                self.history.min_fps()
            ),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 14,
                color: fps_color,
                ..Default::default()
            },
        );
        y_offset += line_height;

        // 帧时间
        let frame_color = if self.metrics.frame_time_ms <= 16.7 {
            GREEN
        } else if self.metrics.frame_time_ms <= 33.3 {
            YELLOW
        } else {
            RED
        };

        draw_text_ex(
            &format!(
                "Frame: {:.2}ms (avg: {:.2}ms)",
                self.metrics.frame_time_ms,
                self.history.average_frame_time()
            ),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 14,
                color: frame_color,
                ..Default::default()
            },
        );
        y_offset += line_height;

        // 实体计数
        draw_text_ex(
            &format!("Entities: {}", self.metrics.entity_count),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 14,
                color: WHITE,
                ..Default::default()
            },
        );
        y_offset += line_height;

        draw_text_ex(
            &format!(
                "Bullets: {}, Asteroids: {}, Particles: {}",
                self.metrics.bullet_count, self.metrics.asteroid_count, self.metrics.particle_count
            ),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 12,
                color: GRAY,
                ..Default::default()
            },
        );
        y_offset += line_height;

        // 内存使用
        draw_text_ex(
            &format!("Memory: {:.1} MB", self.metrics.memory_usage_mb),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 14,
                color: WHITE,
                ..Default::default()
            },
        );
        y_offset += line_height;

        // Hitch检测
        let hitch_color = if self.metrics.hitch_count == 0 {
            GREEN
        } else if self.metrics.hitch_count < 3 {
            YELLOW
        } else {
            RED
        };

        draw_text_ex(
            &format!(
                "Hitches: {} (Max: {:.1}ms)",
                self.metrics.hitch_count, self.metrics.max_frame_time_ms
            ),
            self.overlay_position.x,
            y_offset,
            TextParams {
                font,
                font_size: 14,
                color: hitch_color,
                ..Default::default()
            },
        );
        y_offset += line_height;

        // 网络延迟（如果有）
        if let Some(latency) = self.metrics.network_latency_ms {
            let latency_color = if latency <= 50.0 {
                GREEN
            } else if latency <= 100.0 {
                YELLOW
            } else {
                RED
            };

            draw_text_ex(
                &format!("Network: {:.1}ms", latency),
                self.overlay_position.x,
                y_offset,
                TextParams {
                    font,
                    font_size: 14,
                    color: latency_color,
                    ..Default::default()
                },
            );
        }
    }

    /// 获取当前指标快照
    #[allow(dead_code)]
    pub fn snapshot(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// 检查是否显示覆盖层
    #[allow(dead_code)]
    pub fn is_overlay_visible(&self) -> bool {
        self.show_overlay
    }
}

/// 初始化 puffin 性能分析器（仅原生平台且启用profiling feature）
#[cfg(all(not(target_arch = "wasm32"), feature = "profiling"))]
pub fn init_puffin() {
    puffin::set_scopes_on(true);
}

/// puffin 作用域宏（仅原生平台且启用profiling feature）
#[cfg(all(not(target_arch = "wasm32"), feature = "profiling"))]
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        puffin::profile_scope!($name);
    };
}

/// puffin 作用域宏（其他情况空实现）
#[cfg(not(all(not(target_arch = "wasm32"), feature = "profiling")))]
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        // 性能分析未启用
    };
}
