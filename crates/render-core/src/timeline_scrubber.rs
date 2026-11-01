#![forbid(unsafe_code)]
//! Hardware-Accelerated Animation & Morph Timeline Scrubber
//!
//! Provides:
//! - Deterministic frame pacing and scrubbing at 60fps and 120Hz frame rates
//! - 2D Affine & 4x4 GPU uniform transformation matrix compositor
//! - Playback state machine for presentation preview and slide show transitions

use serde::{Deserialize, Serialize};

/// Target framerate for playback pacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpsTarget {
    Fps60,
    Fps120,
}

impl FpsTarget {
    /// Nominal frame time delta in milliseconds.
    pub fn frame_delta_ms(self) -> f64 {
        match self {
            Self::Fps60 => 1000.0 / 60.0,
            Self::Fps120 => 1000.0 / 120.0,
        }
    }
}

/// Operational state of the playback timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// 2D affine transformation matrix for 2D GPU rendering pipelines:
/// `[a, b, c, d, tx, ty]` mapping $(x, y) \mapsto (a \cdot x + c \cdot y + tx, b \cdot x + d \cdot y + ty)$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AffineMatrix2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Default for AffineMatrix2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl AffineMatrix2D {
    /// Identity matrix.
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Composes translation, rotation, and scale into a composite 2D affine matrix:
    /// $M = T(tx, ty) \times R(\theta) \times S(sx, sy)$.
    pub fn from_transform(tx: f64, ty: f64, rotation_deg: f64, sx: f64, sy: f64) -> Self {
        let rad = rotation_deg * std::f64::consts::PI / 180.0;
        let cos = rad.cos();
        let sin = rad.sin();

        Self {
            a: cos * sx,
            b: sin * sx,
            c: -sin * sy,
            d: cos * sy,
            tx,
            ty,
        }
    }

    /// Transforms a 2D point using this matrix.
    pub fn transform_point(&self, px: f64, py: f64) -> (f64, f64) {
        (
            self.a * px + self.c * py + self.tx,
            self.b * px + self.d * py + self.ty,
        )
    }

    /// Converts to standard column-major 4x4 matrix for WebGPU / DirectX / Metal uniform buffers.
    pub fn to_gpu_matrix4(&self) -> [f32; 16] {
        [
            self.a as f32,
            self.b as f32,
            0.0,
            0.0,
            self.c as f32,
            self.d as f32,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            self.tx as f32,
            self.ty as f32,
            0.0,
            1.0,
        ]
    }
}

/// Hardware-accelerated timeline scrubber and playback state machine.
#[derive(Debug, Clone)]
pub struct TimelineScrubber {
    pub playback_state: PlaybackState,
    pub current_time_ms: f64,
    pub total_duration_ms: f64,
    pub fps_target: FpsTarget,
    pub loop_playback: bool,
}

impl TimelineScrubber {
    /// Creates a new timeline scrubber with specified total duration.
    pub fn new(total_duration_ms: f64, fps_target: FpsTarget) -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            current_time_ms: 0.0,
            total_duration_ms: if total_duration_ms > 0.0 {
                total_duration_ms
            } else {
                1000.0
            },
            fps_target,
            loop_playback: false,
        }
    }

    /// Starts playback.
    pub fn play(&mut self) {
        if self.current_time_ms >= self.total_duration_ms {
            self.current_time_ms = 0.0;
        }
        self.playback_state = PlaybackState::Playing;
    }

    /// Pauses playback.
    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Paused;
    }

    /// Stops playback and rewinds to 0.
    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.current_time_ms = 0.0;
    }

    /// Seeks directly to an absolute timestamp in milliseconds.
    pub fn seek_ms(&mut self, time_ms: f64) -> f64 {
        self.current_time_ms = time_ms.clamp(0.0, self.total_duration_ms);
        self.current_time_ms
    }

    /// Seeks to a normalized progress ratio $t \in [0.0, 1.0]$.
    pub fn seek_progress(&mut self, progress: f64) -> f64 {
        let t = progress.clamp(0.0, 1.0);
        self.current_time_ms = t * self.total_duration_ms;
        self.current_time_ms
    }

    /// Returns normalized playback progress $t \in [0.0, 1.0]$.
    pub fn progress(&self) -> f64 {
        if self.total_duration_ms <= 0.0 {
            0.0
        } else {
            (self.current_time_ms / self.total_duration_ms).clamp(0.0, 1.0)
        }
    }

    /// Toggles loop playback mode.
    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    /// Advances the timeline clock by delta milliseconds (or nominal frame delta).
    pub fn tick(&mut self, delta_ms: Option<f64>) -> f64 {
        if self.playback_state == PlaybackState::Playing {
            let dt = delta_ms.unwrap_or_else(|| self.fps_target.frame_delta_ms());
            self.current_time_ms += dt;

            if self.current_time_ms >= self.total_duration_ms {
                if self.loop_playback {
                    self.current_time_ms %= self.total_duration_ms;
                } else {
                    self.current_time_ms = self.total_duration_ms;
                    self.playback_state = PlaybackState::Paused;
                }
            }
        }
        self.current_time_ms
    }

    /// Convenience: composes 2D affine matrix for a shape.
    pub fn compose_affine_matrix(
        tx: f64,
        ty: f64,
        rotation_deg: f64,
        sx: f64,
        sy: f64,
    ) -> AffineMatrix2D {
        AffineMatrix2D::from_transform(tx, ty, rotation_deg, sx, sy)
    }

    /// Convenience: composes 4x4 GPU uniform matrix for a shape.
    pub fn compose_gpu_matrix4(tx: f64, ty: f64, rotation_deg: f64, sx: f64, sy: f64) -> [f32; 16] {
        AffineMatrix2D::from_transform(tx, ty, rotation_deg, sx, sy).to_gpu_matrix4()
    }
}
