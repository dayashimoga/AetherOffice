//! GPU Draw Primitives & Display List Representation
//!
//! Provides hardware-accelerated 2D instancing primitives:
//! - `ColorRgba`: 32-bit float color with premultiplied alpha conversion
//! - `Rect2D`: 2D bounding boxes and geometric intersection/union logic
//! - `TransformMatrix`: $3 \times 3$ affine transformation matrix
//! - `QuadInstance`: Instanced GPU rectangle primitive for fills, borders, and images
//! - `GlyphInstance` & `GlyphRun`: Shaped typography runs with subpixel positioning and BiDi flags
//! - `BezierPathBatch`: Batched vector paths for DrawingML shapes
//! - `DisplayCommand` & `DisplayList`: Retained render commands with spatial bounds

use ppt_engine::DrawingPath;
use serde::{Deserialize, Serialize};

// =========================================================================
// 1. Color Representation
// =========================================================================

/// Normalized 32-bit floating-point RGBA color $[0.0, 1.0]$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for ColorRgba {
    fn default() -> Self {
        Self::BLACK
    }
}

impl ColorRgba {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parse hex string (`#RGB`, `#RRGGBB`, `#RRGGBBAA`).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self::from_u8(r, g, b, 255))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_u8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_u8(r, g, b, a))
            }
            _ => None,
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }

    /// Convert color to premultiplied alpha for direct GPU blending.
    pub fn to_premultiplied(&self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Convert to 8-bit RGBA integer array.
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }

    /// Convert to hex string format `#RRGGBB` or `#RRGGBBAA`.
    pub fn to_hex(&self) -> String {
        let [r, g, b, a] = self.to_rgba8();
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }
}

// =========================================================================
// 2. 2D Geometry & Rectangles
// =========================================================================

/// 2D axis-aligned bounding box in device-independent pixels (DIPs).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect2D {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect2D {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Check if two rectangles overlap (strict positive area intersection).
    pub fn intersects(&self, other: &Rect2D) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Compute the smallest bounding rectangle containing both rectangles.
    pub fn union(&self, other: &Rect2D) -> Rect2D {
        if self.width <= 0.0 || self.height <= 0.0 {
            return *other;
        }
        if other.width <= 0.0 || other.height <= 0.0 {
            return *self;
        }

        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());

        Rect2D {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    /// Check if a 2D point is contained within the rectangle bounds.
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    /// Uniform scale.
    pub fn scale(&self, factor: f64) -> Rect2D {
        Rect2D {
            x: self.x * factor,
            y: self.y * factor,
            width: self.width * factor,
            height: self.height * factor,
        }
    }

    /// Inset rectangle by horizontal and vertical amounts.
    pub fn inset(&self, dx: f64, dy: f64) -> Rect2D {
        Rect2D {
            x: self.x + dx,
            y: self.y + dy,
            width: (self.width - 2.0 * dx).max(0.0),
            height: (self.height - 2.0 * dy).max(0.0),
        }
    }
}

// =========================================================================
// 3. 2D Affine Transformation Matrix
// =========================================================================

/// 2D affine transformation matrix:
/// $$[x', y', 1] = [x, y, 1] \begin{bmatrix} a & b & 0 \\ c & d & 0 \\ tx & ty & 1 \end{bmatrix}$$
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformMatrix {
    pub a: f64,  // sx
    pub b: f64,  // shy
    pub c: f64,  // shx
    pub d: f64,  // sy
    pub tx: f64, // translation x
    pub ty: f64, // translation y
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl TransformMatrix {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn translate(tx: f64, ty: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx,
            ty,
        }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn rotate_deg(deg: f64) -> Self {
        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Multiply this matrix by another: `self * other`.
    pub fn multiply(&self, other: &TransformMatrix) -> Self {
        Self {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            tx: self.tx * other.a + self.ty * other.c + other.tx,
            ty: self.tx * other.b + self.ty * other.d + other.ty,
        }
    }

    /// Apply transformation to a 2D point $(x, y)$.
    pub fn apply_point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        )
    }

    /// Apply transformation to a rectangle, returning the axis-aligned bounding box.
    pub fn apply_rect(&self, r: &Rect2D) -> Rect2D {
        let p1 = self.apply_point(r.x, r.y);
        let p2 = self.apply_point(r.right(), r.y);
        let p3 = self.apply_point(r.x, r.bottom());
        let p4 = self.apply_point(r.right(), r.bottom());

        let min_x = p1.0.min(p2.0).min(p3.0).min(p4.0);
        let max_x = p1.0.max(p2.0).max(p3.0).max(p4.0);
        let min_y = p1.1.min(p2.1).min(p3.1).min(p4.1);
        let max_y = p1.1.max(p2.1).max(p3.1).max(p4.1);

        Rect2D {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

// =========================================================================
// 4. Instanced GPU Draw Primitives
// =========================================================================

/// Instanced 2D Quad GPU primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadInstance {
    pub rect: Rect2D,
    pub color: ColorRgba,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub texture_id: Option<u32>,
}

impl QuadInstance {
    pub fn solid(rect: Rect2D, color: ColorRgba) -> Self {
        Self {
            rect,
            color,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            texture_id: None,
        }
    }

    pub fn bordered(rect: Rect2D, fill: ColorRgba, border: ColorRgba, width: f32) -> Self {
        Self {
            rect,
            color: fill,
            border_color: Some(border),
            border_width: width,
            corner_radius: 0.0,
            texture_id: None,
        }
    }

    pub fn rounded(rect: Rect2D, fill: ColorRgba, radius: f32) -> Self {
        Self {
            rect,
            color: fill,
            border_color: None,
            border_width: 0.0,
            corner_radius: radius,
            texture_id: None,
        }
    }
}

/// Script category for text shaping and directionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextScript {
    #[default]
    Latin,
    Cjk,
    Arabic,
    Hebrew,
    Devanagari,
    Other,
}

/// Individual positioned glyph in a shaped typography run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphInstance {
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub uv_rect: [f32; 4],
    pub color: ColorRgba,
}

/// Run of shaped, positioned typography.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRun {
    pub text: String,
    pub glyphs: Vec<GlyphInstance>,
    pub font_size_px: f32,
    pub font_family: String,
    pub color: ColorRgba,
    pub bounds: Rect2D,
    pub bidi_level: u8, // 0 = LTR, 1 = RTL
    pub script: TextScript,
}

/// Batched DrawingML vector path with stroke and fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierPathBatch {
    pub path: DrawingPath,
    pub fill_color: Option<ColorRgba>,
    pub stroke_color: Option<ColorRgba>,
    pub stroke_width: f32,
    pub bounds: Rect2D,
}

// =========================================================================
// 5. Display Commands & Display List
// =========================================================================

/// Discrete render command in the retained display list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayCommand {
    PushClip(Rect2D),
    PopClip,
    PushTransform(TransformMatrix),
    PopTransform,
    DrawQuad(QuadInstance),
    DrawText(GlyphRun),
    DrawPath(BezierPathBatch),
}

impl DisplayCommand {
    /// Return the bounding box affected by this command, if any.
    pub fn bounds(&self) -> Option<Rect2D> {
        match self {
            Self::DrawQuad(q) => Some(q.rect),
            Self::DrawText(g) => Some(g.bounds),
            Self::DrawPath(p) => Some(p.bounds),
            _ => None,
        }
    }
}

/// Immutable retained list of GPU draw commands generated each frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
    pub bounds: Rect2D,
    pub quad_count: usize,
    pub glyph_count: usize,
    pub path_count: usize,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, cmd: DisplayCommand) {
        if let Some(b) = cmd.bounds() {
            self.bounds = self.bounds.union(&b);
        }
        match cmd {
            DisplayCommand::DrawQuad(_) => self.quad_count += 1,
            DisplayCommand::DrawText(ref g) => self.glyph_count += g.glyphs.len(),
            DisplayCommand::DrawPath(_) => self.path_count += 1,
            _ => {}
        }
        self.commands.push(cmd);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }
}
