//! 120Hz GPU Compositor & Frame Budget Pacer
//!
//! Enforces GEC-v1.0 Article III latency SLAs:
//! - Frame budget: 120 fps = 8.333 ms per frame
//! - Compilation budget: < 2.0 ms for 4K viewport

use crate::primitive::{DisplayList, Rect2D};
use std::time::Instant;

/// Active viewport configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportConfig {
    pub rect: Rect2D,
    pub zoom: f64,
    pub device_pixel_ratio: f64,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            rect: Rect2D::new(0.0, 0.0, 1920.0, 1080.0),
            zoom: 1.0,
            device_pixel_ratio: 1.0,
        }
    }
}

/// Statistics collected for a rendered frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStats {
    pub compile_time_ms: f64,
    pub command_count: usize,
    pub quad_count: usize,
    pub glyph_count: usize,
    pub path_count: usize,
    pub within_120fps_budget: bool,
}

pub struct Compositor {
    viewport: ViewportConfig,
    last_frame_stats: Option<FrameStats>,
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new(ViewportConfig::default())
    }
}

impl Compositor {
    pub fn new(viewport: ViewportConfig) -> Self {
        Self {
            viewport,
            last_frame_stats: None,
        }
    }

    pub fn viewport(&self) -> &ViewportConfig {
        &self.viewport
    }

    pub fn set_viewport(&mut self, viewport: ViewportConfig) {
        self.viewport = viewport;
    }

    /// Compile a display list with precise execution latency profiling.
    pub fn profile_compile<F>(&mut self, compile_fn: F) -> (DisplayList, FrameStats)
    where
        F: FnOnce(&Rect2D) -> DisplayList,
    {
        let start = Instant::now();
        let display_list = compile_fn(&self.viewport.rect);
        let elapsed = start.elapsed();

        let compile_time_ms = elapsed.as_secs_f64() * 1000.0;
        let within_120fps_budget = compile_time_ms < 2.0;

        let stats = FrameStats {
            compile_time_ms,
            command_count: display_list.len(),
            quad_count: display_list.quad_count,
            glyph_count: display_list.glyph_count,
            path_count: display_list.path_count,
            within_120fps_budget,
        };

        self.last_frame_stats = Some(stats);
        (display_list, stats)
    }

    pub fn last_frame_stats(&self) -> Option<FrameStats> {
        self.last_frame_stats
    }
}
