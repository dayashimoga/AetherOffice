//! PowerPoint Display List Compiler & Interactive Vector Slide Stage
//!
//! Compiles PresentationML slide canvas, background, shape hierarchies, and dynamic
//! animated/morphed scene states into GPU display lists with aspect-ratio preservation.
//!
//! Complies with ECMA-376 Part 1 §19 (PresentationML) and §20 (DrawingML).
//! Features:
//! - Slide canvas projection with aspect ratio preservation (16:9 widescreen, 4:3 standard, custom)
//! - Preset vector geometry compiler (10 standard shapes: Rect, RoundRect, Ellipse, Triangle, Arrow, Star5, Diamond, Callout, Parallelogram, Hexagon)
//! - Interactive 8-point transform gizmos & rotation handle
//! - Translation, aspect-locked resize, and angle snapping rotation math
//! - Slide shape CRUD and Z-order management (BringToFront, SendToBack, BringForward, SendBackward)
//! - Transactional Command Bus undo/redo tracking

use crate::interaction::{Command, DocumentState, PptSelection, ResizeHandle};
use crate::primitive::{
    BezierPathBatch, ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D,
};
use crate::text::ShapedTextEngine;
use ppt_engine::drawingml::DrawingPath;
use ppt_engine::{MorphedSlideState, Shape, Slide};
use serde::{Deserialize, Serialize};

// =========================================================================
// 1. Slide Canvas Configuration & Viewport Projection
// =========================================================================

/// Slide aspect ratio presets.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum SlideAspectRatio {
    #[default]
    Widescreen16x9,
    Standard4x3,
    Custom(f64, f64),
}

/// Slide canvas visual definition.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SlideCanvasConfig {
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub background_color: ColorRgba,
}

impl Default for SlideCanvasConfig {
    fn default() -> Self {
        Self {
            canvas_width: 1920.0,
            canvas_height: 1080.0,
            background_color: ColorRgba::WHITE,
        }
    }
}

impl SlideCanvasConfig {
    pub fn widescreen_16_9() -> Self {
        Self {
            canvas_width: 1920.0,
            canvas_height: 1080.0,
            background_color: ColorRgba::WHITE,
        }
    }

    pub fn standard_4_3() -> Self {
        Self {
            canvas_width: 1440.0,
            canvas_height: 1080.0,
            background_color: ColorRgba::WHITE,
        }
    }

    pub fn custom(width: f64, height: f64) -> Self {
        Self {
            canvas_width: width,
            canvas_height: height,
            background_color: ColorRgba::WHITE,
        }
    }

    pub fn aspect_ratio(&self) -> f64 {
        if self.canvas_height <= 0.0 {
            1.0
        } else {
            self.canvas_width / self.canvas_height
        }
    }
}

/// 2D Viewport Transformation between screen and slide coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PptViewportTransform {
    pub slide_rect: Rect2D,
    pub scale: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

impl PptViewportTransform {
    pub fn new(canvas_cfg: &SlideCanvasConfig, viewport: &Rect2D) -> Self {
        let (slide_rect, scale) =
            PptDisplayCompiler::compute_slide_viewport_transform(canvas_cfg, viewport);
        Self {
            slide_rect,
            scale,
            canvas_width: canvas_cfg.canvas_width,
            canvas_height: canvas_cfg.canvas_height,
        }
    }

    /// Map physical/viewport point $(x, y)$ to slide canvas local coordinates.
    pub fn viewport_to_slide(&self, pt: (f64, f64)) -> (f64, f64) {
        if self.scale <= 0.00001 {
            return (0.0, 0.0);
        }
        let x = (pt.0 - self.slide_rect.x) / self.scale;
        let y = (pt.1 - self.slide_rect.y) / self.scale;
        (x, y)
    }

    /// Map slide canvas local coordinates to physical/viewport coordinates.
    pub fn slide_to_viewport(&self, pt: (f64, f64)) -> (f64, f64) {
        let x = self.slide_rect.x + pt.0 * self.scale;
        let y = self.slide_rect.y + pt.1 * self.scale;
        (x, y)
    }

    /// Convert slide local rect to viewport screen rect.
    pub fn slide_to_viewport_rect(&self, r: &Rect2D) -> Rect2D {
        Rect2D::new(
            self.slide_rect.x + r.x * self.scale,
            self.slide_rect.y + r.y * self.scale,
            r.width * self.scale,
            r.height * self.scale,
        )
    }

    /// Check whether a physical/viewport point falls inside the slide bounds.
    pub fn is_point_on_slide(&self, pt: (f64, f64)) -> bool {
        self.slide_rect.contains_point(pt.0, pt.1)
    }
}

// =========================================================================
// 2. Preset Vector Geometries
// =========================================================================

/// Standard DrawingML preset shape types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresetShapeType {
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Triangle,
    RightArrow,
    Star5,
    Diamond,
    Callout,
    Parallelogram,
    Hexagon,
    TextBox,
}

impl PresetShapeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rectangle => "rect",
            Self::RoundedRectangle => "roundRect",
            Self::Ellipse => "ellipse",
            Self::Triangle => "triangle",
            Self::RightArrow => "rightArrow",
            Self::Star5 => "star5",
            Self::Diamond => "diamond",
            Self::Callout => "wedgeRectCallout",
            Self::Parallelogram => "parallelogram",
            Self::Hexagon => "hexagon",
            Self::TextBox => "textBox",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "rect" | "rectangle" => Self::Rectangle,
            "roundRect" | "roundedRectangle" => Self::RoundedRectangle,
            "ellipse" | "circle" => Self::Ellipse,
            "triangle" | "isoscelesTriangle" => Self::Triangle,
            "rightArrow" | "arrow" => Self::RightArrow,
            "star5" | "star" => Self::Star5,
            "diamond" => Self::Diamond,
            "callout" | "wedgeRectCallout" => Self::Callout,
            "parallelogram" => Self::Parallelogram,
            "hexagon" => Self::Hexagon,
            "textBox" | "text" => Self::TextBox,
            _ => Self::Rectangle,
        }
    }
}

/// Generate normalized vector paths for preset shapes inside `bounds`.
pub fn generate_preset_path(
    shape_type: PresetShapeType,
    bounds: &Rect2D,
    adjust: Option<f64>,
) -> DrawingPath {
    let mut path = DrawingPath::new();
    let x = bounds.x;
    let y = bounds.y;
    let w = bounds.width;
    let h = bounds.height;
    let r = x + w;
    let b = y + h;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    match shape_type {
        PresetShapeType::Rectangle | PresetShapeType::TextBox => {
            path.move_to(x, y);
            path.line_to(r, y);
            path.line_to(r, b);
            path.line_to(x, b);
            path.close();
        }
        PresetShapeType::RoundedRectangle => {
            let adj = adjust.unwrap_or(0.15).clamp(0.0, 0.5);
            let rad = (w.min(h) * adj).max(1.0);
            let kappa = 0.5522847498;
            let k_rad = rad * kappa;

            path.move_to(x + rad, y);
            path.line_to(r - rad, y);
            path.cubic_to(r - rad + k_rad, y, r, y + rad - k_rad, r, y + rad);
            path.line_to(r, b - rad);
            path.cubic_to(r, b - rad + k_rad, r - rad + k_rad, b, r - rad, b);
            path.line_to(x + rad, b);
            path.cubic_to(x + rad - k_rad, b, x, b - rad + k_rad, x, b - rad);
            path.line_to(x, y + rad);
            path.cubic_to(x, y + rad - k_rad, x + rad - k_rad, y, x + rad, y);
            path.close();
        }
        PresetShapeType::Ellipse => {
            let rx = w / 2.0;
            let ry = h / 2.0;
            let kappa = 0.5522847498;
            let kx = rx * kappa;
            let ky = ry * kappa;

            path.move_to(cx, y);
            path.cubic_to(cx + kx, y, r, cy - ky, r, cy);
            path.cubic_to(r, cy + ky, cx + kx, b, cx, b);
            path.cubic_to(cx - kx, b, x, cy + ky, x, cy);
            path.cubic_to(x, cy - ky, cx - kx, y, cx, y);
            path.close();
        }
        PresetShapeType::Triangle => {
            path.move_to(cx, y);
            path.line_to(r, b);
            path.line_to(x, b);
            path.close();
        }
        PresetShapeType::RightArrow => {
            let head_len = (w * 0.35).min(w);
            let shaft_h = h * 0.4;
            let shaft_top = cy - shaft_h / 2.0;
            let shaft_bottom = cy + shaft_h / 2.0;
            let head_start_x = r - head_len;

            path.move_to(x, shaft_top);
            path.line_to(head_start_x, shaft_top);
            path.line_to(head_start_x, y);
            path.line_to(r, cy);
            path.line_to(head_start_x, b);
            path.line_to(head_start_x, shaft_bottom);
            path.line_to(x, shaft_bottom);
            path.close();
        }
        PresetShapeType::Star5 => {
            let r_outer = (w.min(h) / 2.0).max(1.0);
            let r_inner = r_outer * 0.382;
            let step = std::f64::consts::PI / 5.0;
            let mut angle = -std::f64::consts::FRAC_PI_2;

            for i in 0..10 {
                let current_r = if i % 2 == 0 { r_outer } else { r_inner };
                let px = cx + current_r * angle.cos();
                let py = cy + current_r * angle.sin();
                if i == 0 {
                    path.move_to(px, py);
                } else {
                    path.line_to(px, py);
                }
                angle += step;
            }
            path.close();
        }
        PresetShapeType::Diamond => {
            path.move_to(cx, y);
            path.line_to(r, cy);
            path.line_to(cx, b);
            path.line_to(x, cy);
            path.close();
        }
        PresetShapeType::Callout => {
            let box_b = b - h * 0.2;
            let tail_tip_x = x + w * 0.2;
            let tail_tip_y = b;

            path.move_to(x, y);
            path.line_to(r, y);
            path.line_to(r, box_b);
            path.line_to(x + w * 0.45, box_b);
            path.line_to(tail_tip_x, tail_tip_y);
            path.line_to(x + w * 0.3, box_b);
            path.line_to(x, box_b);
            path.close();
        }
        PresetShapeType::Parallelogram => {
            let skew = w * 0.2;
            path.move_to(x + skew, y);
            path.line_to(r, y);
            path.line_to(r - skew, b);
            path.line_to(x, b);
            path.close();
        }
        PresetShapeType::Hexagon => {
            let inset = w * 0.25;
            path.move_to(x + inset, y);
            path.line_to(r - inset, y);
            path.line_to(r, cy);
            path.line_to(r - inset, b);
            path.line_to(x + inset, b);
            path.line_to(x, cy);
            path.close();
        }
    }

    path
}

// =========================================================================
// 3. Transform Gizmos & Hit-Testing
// =========================================================================

/// Resolved hit target on a PowerPoint shape transform gizmo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PptGizmoHit {
    Handle(ResizeHandle),
    Rotate,
    Body,
}

/// 8-handle + rotation transform gizmo model.
#[derive(Debug, Clone, PartialEq)]
pub struct PptGizmoModel {
    pub bounds: Rect2D,
    pub rotation_deg: f64,
    pub handle_size: f64,
}

impl PptGizmoModel {
    pub fn new(bounds: Rect2D, rotation_deg: f64) -> Self {
        Self {
            bounds,
            rotation_deg,
            handle_size: 8.0,
        }
    }

    /// Center coordinates of the 8 resize handles.
    pub fn handle_positions(&self) -> [(f64, f64, ResizeHandle); 8] {
        let x = self.bounds.x;
        let y = self.bounds.y;
        let w = self.bounds.width;
        let h = self.bounds.height;
        let r = x + w;
        let b = y + h;
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;

        [
            (x, y, ResizeHandle::TopLeft),
            (cx, y, ResizeHandle::Top),
            (r, y, ResizeHandle::TopRight),
            (r, cy, ResizeHandle::Right),
            (r, b, ResizeHandle::BottomRight),
            (cx, b, ResizeHandle::Bottom),
            (x, b, ResizeHandle::BottomLeft),
            (x, cy, ResizeHandle::Left),
        ]
    }

    /// Center of the rotation handle (24px above top center).
    pub fn rotation_handle_center(&self) -> (f64, f64) {
        let cx = self.bounds.x + self.bounds.width / 2.0;
        let cy = self.bounds.y - 24.0;
        (cx, cy)
    }

    /// Hit test against rotation handle, 8 resize handles, and interior body.
    pub fn hit_test(&self, point: (f64, f64), threshold: f64) -> Option<PptGizmoHit> {
        let (px, py) = point;
        let (rot_cx, rot_cy) = self.rotation_handle_center();
        let dist_rot = ((px - rot_cx).powi(2) + (py - rot_cy).powi(2)).sqrt();
        if dist_rot <= 5.0 + threshold {
            return Some(PptGizmoHit::Rotate);
        }

        let half_h = self.handle_size / 2.0 + threshold;
        for (hx, hy, handle) in self.handle_positions() {
            if (px - hx).abs() <= half_h && (py - hy).abs() <= half_h {
                return Some(PptGizmoHit::Handle(handle));
            }
        }

        if self.bounds.contains_point(px, py) {
            return Some(PptGizmoHit::Body);
        }

        None
    }
}

// =========================================================================
// 4. Slide Transform Engine (Translation, Resize, Rotation)
// =========================================================================

pub struct SlideTransformEngine;

impl SlideTransformEngine {
    /// Apply translation delta $(dx, dy)$ to a bounding box.
    pub fn translate(bounds: &Rect2D, dx: f64, dy: f64) -> Rect2D {
        Rect2D::new(bounds.x + dx, bounds.y + dy, bounds.width, bounds.height)
    }

    /// Resize a shape bounding box from one of the 8 handles with optional aspect ratio lock.
    pub fn resize(
        bounds: &Rect2D,
        handle: ResizeHandle,
        dx: f64,
        dy: f64,
        lock_aspect: bool,
        min_size: f64,
    ) -> Rect2D {
        let min_s = min_size.max(4.0);
        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.width;
        let h = bounds.height;
        let r = x + w;
        let b = y + h;
        let aspect = if h > 0.0 { w / h } else { 1.0 };

        match handle {
            ResizeHandle::BottomRight => {
                let mut new_w = (w + dx).max(min_s);
                let mut new_h = (h + dy).max(min_s);
                if lock_aspect {
                    let s = (new_w / w).max(new_h / h);
                    new_w = (w * s).max(min_s);
                    new_h = (h * s).max(min_s);
                }
                Rect2D::new(x, y, new_w, new_h)
            }
            ResizeHandle::TopLeft => {
                let mut new_w = (w - dx).max(min_s);
                let mut new_h = (h - dy).max(min_s);
                if lock_aspect {
                    let s = (new_w / w).max(new_h / h);
                    new_w = (w * s).max(min_s);
                    new_h = (h * s).max(min_s);
                }
                Rect2D::new(r - new_w, b - new_h, new_w, new_h)
            }
            ResizeHandle::TopRight => {
                let mut new_w = (w + dx).max(min_s);
                let mut new_h = (h - dy).max(min_s);
                if lock_aspect {
                    let s = (new_w / w).max(new_h / h);
                    new_w = (w * s).max(min_s);
                    new_h = (h * s).max(min_s);
                }
                Rect2D::new(x, b - new_h, new_w, new_h)
            }
            ResizeHandle::BottomLeft => {
                let mut new_w = (w - dx).max(min_s);
                let mut new_h = (h + dy).max(min_s);
                if lock_aspect {
                    let s = (new_w / w).max(new_h / h);
                    new_w = (w * s).max(min_s);
                    new_h = (h * s).max(min_s);
                }
                Rect2D::new(r - new_w, y, new_w, new_h)
            }
            ResizeHandle::Top => {
                let new_h = (h - dy).max(min_s);
                let new_w = if lock_aspect { new_h * aspect } else { w };
                let new_x = if lock_aspect {
                    x + (w - new_w) / 2.0
                } else {
                    x
                };
                Rect2D::new(new_x, b - new_h, new_w, new_h)
            }
            ResizeHandle::Bottom => {
                let new_h = (h + dy).max(min_s);
                let new_w = if lock_aspect { new_h * aspect } else { w };
                let new_x = if lock_aspect {
                    x + (w - new_w) / 2.0
                } else {
                    x
                };
                Rect2D::new(new_x, y, new_w, new_h)
            }
            ResizeHandle::Left => {
                let new_w = (w - dx).max(min_s);
                let new_h = if lock_aspect { new_w / aspect } else { h };
                let new_y = if lock_aspect {
                    y + (h - new_h) / 2.0
                } else {
                    y
                };
                Rect2D::new(r - new_w, new_y, new_w, new_h)
            }
            ResizeHandle::Right => {
                let new_w = (w + dx).max(min_s);
                let new_h = if lock_aspect { new_w / aspect } else { h };
                let new_y = if lock_aspect {
                    y + (h - new_h) / 2.0
                } else {
                    y
                };
                Rect2D::new(x, new_y, new_w, new_h)
            }
        }
    }

    /// Calculate rotation angle around center $(cx, cy)$ towards pointer $(px, py)$.
    pub fn rotate(center: (f64, f64), pointer: (f64, f64), snap_step_deg: Option<f64>) -> f64 {
        let dx = pointer.0 - center.0;
        let dy = pointer.1 - center.1;
        let theta = dy.atan2(dx).to_degrees();
        let mut deg = theta + 90.0;

        while deg < 0.0 {
            deg += 360.0;
        }
        while deg >= 360.0 {
            deg -= 360.0;
        }

        if let Some(step) = snap_step_deg {
            if step > 0.001 {
                deg = (deg / step).round() * step;
                while deg >= 360.0 {
                    deg -= 360.0;
                }
            }
        }

        deg
    }

    /// Compute the smallest bounding rectangle containing all selected shapes.
    pub fn multi_selection_bounds(
        shapes: &[SlideShapeModel],
        selected_ids: &[u32],
    ) -> Option<Rect2D> {
        let mut union_rect: Option<Rect2D> = None;
        for s in shapes {
            if selected_ids.contains(&s.id) {
                union_rect = match union_rect {
                    Some(u) => Some(u.union(&s.bounds)),
                    None => Some(s.bounds),
                };
            }
        }
        union_rect
    }
}

// =========================================================================
// 5. Slide Shape & Stage Document Model
// =========================================================================

/// Complete visual and layout definition of a shape on a slide stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideShapeModel {
    pub id: u32,
    pub name: String,
    pub shape_type: PresetShapeType,
    pub bounds: Rect2D,
    pub rotation_deg: f64,
    pub fill_color: ColorRgba,
    pub stroke_color: Option<ColorRgba>,
    pub stroke_width: f32,
    pub text: Option<String>,
    pub font_size: f32,
    pub text_color: ColorRgba,
    pub is_locked: bool,
    pub is_visible: bool,
}

impl SlideShapeModel {
    pub fn new(id: u32, shape_type: PresetShapeType, bounds: Rect2D) -> Self {
        Self {
            id,
            name: format!("{} {}", shape_type.as_str(), id),
            shape_type,
            bounds,
            rotation_deg: 0.0,
            fill_color: ColorRgba::new(0.18, 0.45, 0.72, 1.0),
            stroke_color: Some(ColorRgba::new(0.10, 0.30, 0.55, 1.0)),
            stroke_width: 1.5,
            text: None,
            font_size: 16.0,
            text_color: ColorRgba::WHITE,
            is_locked: false,
            is_visible: true,
        }
    }
}

/// Z-order reordering actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZOrderAction {
    BringToFront,
    SendToBack,
    BringForward,
    SendBackward,
}

/// Slide Stage document model containing ordered shapes and canvas configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideStageModel {
    pub shapes: Vec<SlideShapeModel>,
    pub canvas_config: SlideCanvasConfig,
    pub next_shape_id: u32,
}

impl SlideStageModel {
    pub fn new(canvas_config: SlideCanvasConfig) -> Self {
        Self {
            shapes: Vec::new(),
            canvas_config,
            next_shape_id: 1,
        }
    }

    pub fn add_shape(&mut self, mut shape: SlideShapeModel) -> u32 {
        if shape.id == 0 {
            shape.id = self.next_shape_id;
            self.next_shape_id += 1;
        } else if shape.id >= self.next_shape_id {
            self.next_shape_id = shape.id + 1;
        }
        let id = shape.id;
        self.shapes.push(shape);
        id
    }

    pub fn remove_shape(&mut self, id: u32) -> Option<(usize, SlideShapeModel)> {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            let shape = self.shapes.remove(pos);
            Some((pos, shape))
        } else {
            None
        }
    }

    pub fn insert_shape_at(&mut self, index: usize, shape: SlideShapeModel) {
        let idx = index.min(self.shapes.len());
        self.shapes.insert(idx, shape);
    }

    pub fn get_shape(&self, id: u32) -> Option<&SlideShapeModel> {
        self.shapes.iter().find(|s| s.id == id)
    }

    pub fn get_shape_mut(&mut self, id: u32) -> Option<&mut SlideShapeModel> {
        self.shapes.iter_mut().find(|s| s.id == id)
    }

    pub fn bring_to_front(&mut self, id: u32) -> bool {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            if pos + 1 < self.shapes.len() {
                let shape = self.shapes.remove(pos);
                self.shapes.push(shape);
                return true;
            }
        }
        false
    }

    pub fn send_to_back(&mut self, id: u32) -> bool {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            if pos > 0 {
                let shape = self.shapes.remove(pos);
                self.shapes.insert(0, shape);
                return true;
            }
        }
        false
    }

    pub fn bring_forward(&mut self, id: u32) -> bool {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            if pos + 1 < self.shapes.len() {
                self.shapes.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    pub fn send_backward(&mut self, id: u32) -> bool {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            if pos > 0 {
                self.shapes.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }

    pub fn duplicate_shape(&mut self, id: u32, offset: (f64, f64)) -> Option<u32> {
        if let Some(shape) = self.get_shape(id).cloned() {
            let mut dup = shape;
            dup.id = self.next_shape_id;
            self.next_shape_id += 1;
            dup.name = format!("{} (Copy)", dup.name);
            dup.bounds = Rect2D::new(
                dup.bounds.x + offset.0,
                dup.bounds.y + offset.1,
                dup.bounds.width,
                dup.bounds.height,
            );
            let new_id = dup.id;
            self.shapes.push(dup);
            Some(new_id)
        } else {
            None
        }
    }

    pub fn hit_test(&self, point: (f64, f64)) -> Option<u32> {
        for s in self.shapes.iter().rev() {
            if s.is_visible && s.bounds.contains_point(point.0, point.1) {
                return Some(s.id);
            }
        }
        None
    }
}

// =========================================================================
// 6. Transactional Commands for Command Bus
// =========================================================================

/// Reversible shape transform command.
#[derive(Debug, Clone)]
pub struct SlideTransformCommand {
    pub shape_id: u32,
    pub prev_bounds: Rect2D,
    pub prev_rotation: f64,
    pub new_bounds: Rect2D,
    pub new_rotation: f64,
}

impl Command for SlideTransformCommand {
    fn execute(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn undo(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn description(&self) -> &str {
        "Transform Shape"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Reversible add shape command.
#[derive(Debug, Clone)]
pub struct SlideAddShapeCommand {
    pub shape: SlideShapeModel,
    pub added_id: u32,
}

impl Command for SlideAddShapeCommand {
    fn execute(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn undo(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn description(&self) -> &str {
        "Add Shape"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Reversible delete shape command.
#[derive(Debug, Clone)]
pub struct SlideDeleteShapeCommand {
    pub shape_id: u32,
    pub deleted_shape: Option<SlideShapeModel>,
    pub index: usize,
}

impl Command for SlideDeleteShapeCommand {
    fn execute(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn undo(&mut self, _state: &mut DocumentState) -> Result<(), String> {
        Ok(())
    }
    fn description(&self) -> &str {
        "Delete Shape"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// =========================================================================
// 7. Interactive Slide Stage Controller
// =========================================================================

/// State controller managing slide editing, selection state, and undo/redo history.
#[derive(Debug, Clone, Default)]
pub struct PptStageController {
    pub stage: SlideStageModel,
    pub selection: PptSelection,
    undo_stack: Vec<SlideStageModel>,
    redo_stack: Vec<SlideStageModel>,
}

impl PptStageController {
    pub fn new(stage: SlideStageModel) -> Self {
        Self {
            stage,
            selection: PptSelection::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(self.stage.clone());
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(prev) = self.undo_stack.pop() {
            let curr = std::mem::replace(&mut self.stage, prev);
            self.redo_stack.push(curr);
            Ok(())
        } else {
            Err("Nothing to undo".into())
        }
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(next) = self.redo_stack.pop() {
            let curr = std::mem::replace(&mut self.stage, next);
            self.undo_stack.push(curr);
            Ok(())
        } else {
            Err("Nothing to redo".into())
        }
    }

    pub fn select_shape(&mut self, id: u32, multi: bool) {
        if multi {
            self.selection.toggle(id);
        } else {
            self.selection = PptSelection::single(id);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn execute_transform(
        &mut self,
        shape_id: u32,
        new_bounds: Rect2D,
        new_rotation: f64,
    ) -> Result<(), String> {
        self.push_undo_snapshot();
        if let Some(shape) = self.stage.get_shape_mut(shape_id) {
            shape.bounds = new_bounds;
            shape.rotation_deg = new_rotation;
            Ok(())
        } else {
            Err(format!("Shape #{} not found", shape_id))
        }
    }

    pub fn execute_add_shape(&mut self, shape: SlideShapeModel) -> Result<u32, String> {
        self.push_undo_snapshot();
        let id = self.stage.add_shape(shape);
        self.selection = PptSelection::single(id);
        Ok(id)
    }

    pub fn execute_delete_shape(&mut self, shape_id: u32) -> Result<(), String> {
        self.push_undo_snapshot();
        if self.stage.remove_shape(shape_id).is_some() {
            self.selection.clear();
            Ok(())
        } else {
            Err(format!("Shape #{} not found", shape_id))
        }
    }

    pub fn execute_delete_selected(&mut self) -> Result<(), String> {
        if self.selection.selected_ids.is_empty() {
            return Ok(());
        }
        self.push_undo_snapshot();
        let ids = self.selection.selected_ids.clone();
        for id in ids {
            self.stage.remove_shape(id);
        }
        self.selection.clear();
        Ok(())
    }

    pub fn execute_style(
        &mut self,
        shape_id: u32,
        fill: Option<ColorRgba>,
        stroke: Option<ColorRgba>,
        stroke_width: Option<f32>,
    ) -> Result<(), String> {
        self.push_undo_snapshot();
        if let Some(shape) = self.stage.get_shape_mut(shape_id) {
            if let Some(f) = fill {
                shape.fill_color = f;
            }
            if let Some(s) = stroke {
                shape.stroke_color = Some(s);
            }
            if let Some(sw) = stroke_width {
                shape.stroke_width = sw;
            }
            Ok(())
        } else {
            Err(format!("Shape #{} not found", shape_id))
        }
    }

    pub fn execute_z_order(&mut self, shape_id: u32, action: ZOrderAction) -> Result<(), String> {
        self.push_undo_snapshot();
        let changed = match action {
            ZOrderAction::BringToFront => self.stage.bring_to_front(shape_id),
            ZOrderAction::SendToBack => self.stage.send_to_back(shape_id),
            ZOrderAction::BringForward => self.stage.bring_forward(shape_id),
            ZOrderAction::SendBackward => self.stage.send_backward(shape_id),
        };
        if changed {
            Ok(())
        } else {
            Err(format!(
                "Could not apply {:?} to shape #{}",
                action, shape_id
            ))
        }
    }

    pub fn compile_display_list(&self, viewport: &Rect2D) -> DisplayList {
        PptDisplayCompiler::compile_slide_stage(&self.stage, Some(&self.selection), viewport)
    }
}

// =========================================================================
// 8. PowerPoint Display Compiler
// =========================================================================

pub struct PptDisplayCompiler;

impl PptDisplayCompiler {
    /// Calculate aspect-ratio centered origin and dimensions of the slide canvas within the viewport.
    pub fn compute_slide_viewport_transform(
        canvas_cfg: &SlideCanvasConfig,
        viewport: &Rect2D,
    ) -> (Rect2D, f64) {
        let canvas_aspect = canvas_cfg.canvas_width / canvas_cfg.canvas_height;
        let viewport_aspect = viewport.width / viewport.height;

        let (dest_w, dest_h) = if viewport_aspect > canvas_aspect {
            let h = viewport.height;
            let w = h * canvas_aspect;
            (w, h)
        } else {
            let w = viewport.width;
            let h = w / canvas_aspect;
            (w, h)
        };

        let dest_x = viewport.x + (viewport.width - dest_w) / 2.0;
        let dest_y = viewport.y + (viewport.height - dest_h) / 2.0;

        let scale = dest_w / canvas_cfg.canvas_width;
        (Rect2D::new(dest_x, dest_y, dest_w, dest_h), scale)
    }

    /// Compile a slide and optional animated morphed state into a `DisplayList`.
    pub fn compile_slide(
        _slide: &Slide,
        shapes: &[Shape],
        morphed_state: Option<&MorphedSlideState>,
        canvas_cfg: &SlideCanvasConfig,
        viewport: &Rect2D,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();
        let (slide_rect, scale) = Self::compute_slide_viewport_transform(canvas_cfg, viewport);

        // 1. Dark letterbox background for area outside slide
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            *viewport,
            ColorRgba::new(0.12, 0.12, 0.12, 1.0),
        )));

        // 2. Slide drop shadow
        let shadow_rect = Rect2D::new(
            slide_rect.x + 8.0,
            slide_rect.y + 8.0,
            slide_rect.width,
            slide_rect.height,
        );
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            shadow_rect,
            ColorRgba::new(0.0, 0.0, 0.0, 0.4),
        )));

        // 3. Slide canvas background
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            slide_rect,
            canvas_cfg.background_color,
        )));

        // 4. Clip to slide canvas boundaries
        display_list.push(DisplayCommand::PushClip(slide_rect));

        // 5. If animated / morphed state is provided, evaluate morphed shapes
        if let Some(morph) = morphed_state {
            for mshape in &morph.shapes {
                let sx = slide_rect.x + mshape.transform.x * scale;
                let sy = slide_rect.y + mshape.transform.y * scale;
                let sw = mshape.transform.width * scale;
                let sh = mshape.transform.height * scale;
                let srect = Rect2D::new(sx, sy, sw, sh);

                let alpha = (mshape.opacity as f32).clamp(0.0, 1.0);
                if alpha <= 0.001 {
                    continue;
                }

                // If path is present, emit vector path
                if let Some(ref path) = mshape.geometry_path {
                    display_list.push(DisplayCommand::DrawPath(BezierPathBatch {
                        path: path.clone(),
                        fill_color: Some(ColorRgba::new(0.2, 0.4, 0.8, alpha)),
                        stroke_color: Some(ColorRgba::new(0.1, 0.2, 0.5, alpha)),
                        stroke_width: 2.0 * (scale as f32),
                        bounds: srect,
                    }));
                } else {
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::rounded(
                        srect,
                        ColorRgba::new(0.2, 0.4, 0.8, alpha),
                        4.0 * (scale as f32),
                    )));
                }

                // Render morphed text elements
                for te in &mshape.text_elements {
                    let tx = slide_rect.x + te.x * scale;
                    let ty = slide_rect.y + te.y * scale;
                    let talp = ((te.opacity * mshape.opacity) as f32).clamp(0.0, 1.0);
                    if talp > 0.001 {
                        let run = ShapedTextEngine::shape_text(
                            &te.text,
                            "Calibri",
                            te.font_size_pt as f32 * (scale as f32),
                            ColorRgba::new(1.0, 1.0, 1.0, talp),
                            (tx, ty),
                        );
                        display_list.push(DisplayCommand::DrawText(run));
                    }
                }
            }
        } else {
            // 6. Static shape tree rendering
            for s in shapes {
                if let Some(ref xfrm) = s.properties.transform {
                    let sx = slide_rect.x + (xfrm.x as f64) * scale;
                    let sy = slide_rect.y + (xfrm.y as f64) * scale;
                    let sw = (xfrm.cx as f64) * scale;
                    let sh = (xfrm.cy as f64) * scale;
                    let srect = Rect2D::new(sx, sy, sw, sh);

                    // Draw shape body
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                        srect,
                        ColorRgba::new(0.18, 0.45, 0.72, 1.0),
                        ColorRgba::new(0.10, 0.30, 0.55, 1.0),
                        1.5 * (scale as f32),
                    )));

                    // Draw text body
                    if let Some(ref tb) = s.text_body {
                        let mut line_y = sy + 10.0 * scale;
                        for p in &tb.paragraphs {
                            for r in &p.runs {
                                if let ppt_engine::TextRunElement::Text(ref text, _) = r {
                                    let run = ShapedTextEngine::shape_text(
                                        text,
                                        "Calibri",
                                        16.0 * (scale as f32),
                                        ColorRgba::WHITE,
                                        (sx + 10.0 * scale, line_y),
                                    );
                                    display_list.push(DisplayCommand::DrawText(run));
                                    line_y += 20.0 * scale;
                                }
                            }
                        }
                    }
                }
            }
        }

        display_list.push(DisplayCommand::PopClip);
        display_list
    }

    /// Compile a full interactive `SlideStageModel` with vector preset shapes and transform gizmos into a `DisplayList`.
    pub fn compile_slide_stage(
        stage: &SlideStageModel,
        selection: Option<&PptSelection>,
        viewport: &Rect2D,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();
        let transform = PptViewportTransform::new(&stage.canvas_config, viewport);

        // 1. Dark letterbox background for area outside slide
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            *viewport,
            ColorRgba::new(0.12, 0.12, 0.12, 1.0),
        )));

        // 2. Slide drop shadow
        let shadow_rect = Rect2D::new(
            transform.slide_rect.x + 8.0,
            transform.slide_rect.y + 8.0,
            transform.slide_rect.width,
            transform.slide_rect.height,
        );
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            shadow_rect,
            ColorRgba::new(0.0, 0.0, 0.0, 0.4),
        )));

        // 3. Slide canvas background
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            transform.slide_rect,
            stage.canvas_config.background_color,
        )));

        // 4. Clip to slide canvas boundaries
        display_list.push(DisplayCommand::PushClip(transform.slide_rect));

        // 5. Render shapes
        for shape in &stage.shapes {
            if !shape.is_visible {
                continue;
            }
            let v_rect = transform.slide_to_viewport_rect(&shape.bounds);
            let stroke_c = shape.stroke_color.unwrap_or(ColorRgba::TRANSPARENT);
            let stroke_w = (shape.stroke_width * (transform.scale as f32)).max(1.0);

            if shape.shape_type == PresetShapeType::Rectangle && shape.rotation_deg == 0.0 {
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                    v_rect,
                    shape.fill_color,
                    stroke_c,
                    stroke_w,
                )));
            } else {
                let path = generate_preset_path(shape.shape_type, &v_rect, None);
                display_list.push(DisplayCommand::DrawPath(BezierPathBatch {
                    path,
                    fill_color: Some(shape.fill_color),
                    stroke_color: shape.stroke_color,
                    stroke_width: stroke_w,
                    bounds: v_rect,
                }));
            }

            // Text run inside shape
            if let Some(ref text) = shape.text {
                if !text.is_empty() {
                    let text_pt = (
                        v_rect.x + 8.0 * transform.scale,
                        v_rect.y + 8.0 * transform.scale,
                    );
                    let run = ShapedTextEngine::shape_text(
                        text,
                        "Calibri",
                        shape.font_size * (transform.scale as f32),
                        shape.text_color,
                        text_pt,
                    );
                    display_list.push(DisplayCommand::DrawText(run));
                }
            }
        }

        display_list.push(DisplayCommand::PopClip);

        // 6. Selection Transform Gizmos (rendered on top of clip)
        if let Some(sel) = selection {
            if !sel.selected_ids.is_empty() {
                if let Some(union_bounds) =
                    SlideTransformEngine::multi_selection_bounds(&stage.shapes, &sel.selected_ids)
                {
                    let v_union = transform.slide_to_viewport_rect(&union_bounds);
                    let gizmo = PptGizmoModel::new(v_union, 0.0);
                    let terracotta = ColorRgba::new(0.77, 0.24, 0.11, 1.0); // #C43E1C

                    // Selection bounding box
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                        v_union.inset(-2.0, -2.0),
                        ColorRgba::TRANSPARENT,
                        terracotta,
                        1.5,
                    )));

                    // Rotation stalk line
                    let stalk_x = v_union.x + v_union.width / 2.0 - 0.75;
                    let stalk_rect = Rect2D::new(stalk_x, v_union.y - 20.0, 1.5, 20.0);
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                        stalk_rect, terracotta,
                    )));

                    // Rotation handle circle
                    let (rot_cx, rot_cy) = gizmo.rotation_handle_center();
                    let rot_rect = Rect2D::new(rot_cx - 4.0, rot_cy - 4.0, 8.0, 8.0);
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::rounded(
                        rot_rect, terracotta, 4.0,
                    )));

                    // 8 Resize handles
                    for (hx, hy, _) in gizmo.handle_positions() {
                        let h_rect = Rect2D::new(hx - 3.5, hy - 3.5, 7.0, 7.0);
                        display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                            h_rect,
                            ColorRgba::WHITE,
                            terracotta,
                            1.5,
                        )));
                    }
                }
            }
        }

        display_list
    }
}
