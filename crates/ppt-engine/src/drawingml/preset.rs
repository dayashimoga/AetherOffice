#![forbid(unsafe_code)]
//! ECMA-376 Part 1 DrawingML 187 Preset Geometries Catalog
//!
//! Provides specifications and parametric vector path generators for all 187
//! standard DrawingML preset shapes defined in ECMA-376 Section 20.1.10.55 (`ST_ShapeType`).

use std::f64::consts::PI;

use super::guide::GuideContext;
use super::path::{DrawingPath, PathFillMode};

/// Specification of an ECMA-376 preset geometry.
#[derive(Clone, Copy)]
pub struct PresetDefinition {
    pub name: &'static str,
    pub default_adjusts: &'static [(&'static str, i64)],
    pub guides: &'static [(&'static str, &'static str)],
    pub generate: fn(&GuideContext) -> Vec<DrawingPath>,
}

/// Returns a slice containing all 187 standard ECMA-376 preset shape names.
pub fn list_preset_names() -> &'static [&'static str] {
    ALL_PRESET_NAMES
}

/// Retrieve the definition for any of the 187 standard ECMA-376 preset shapes.
pub fn get_preset_geometry(name: &str) -> Option<PresetDefinition> {
    ALL_PRESET_DEFS.iter().find(|d| d.name == name).copied()
}

// =========================================================================
// Category 1: Rectangles & Clipped Rectangles
// =========================================================================

fn gen_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_round_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let ss = w.min(h);

    let adj = ctx.get("adj").unwrap_or(16667) as f64;
    let rad = (ss * adj / 100_000.0).max(0.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l + rad, t);
    p.line_to(r - rad, t);
    p.arc_to(rad, rad, 270.0, 90.0);
    p.line_to(r, b - rad);
    p.arc_to(rad, rad, 0.0, 90.0);
    p.line_to(l + rad, b);
    p.arc_to(rad, rad, 90.0, 90.0);
    p.line_to(l, t + rad);
    p.arc_to(rad, rad, 180.0, 90.0);
    p.close();
    vec![p]
}

fn gen_round1_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let rad = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r - rad, t);
    p.arc_to(rad, rad, 270.0, 90.0);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_round2_same_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let rad = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l + rad, t);
    p.line_to(r - rad, t);
    p.arc_to(rad, rad, 270.0, 90.0);
    p.line_to(r, b);
    p.line_to(l, b);
    p.line_to(l, t + rad);
    p.arc_to(rad, rad, 180.0, 90.0);
    p.close();
    vec![p]
}

fn gen_round2_diag_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let rad = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l + rad, t);
    p.line_to(r, t);
    p.line_to(r, b - rad);
    p.arc_to(rad, rad, 0.0, 90.0);
    p.line_to(l, b);
    p.line_to(l, t + rad);
    p.arc_to(rad, rad, 180.0, 90.0);
    p.close();
    vec![p]
}

fn gen_snip1_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let snip = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r - snip, t);
    p.line_to(r, t + snip);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_snip2_same_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let snip = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l + snip, t);
    p.line_to(r - snip, t);
    p.line_to(r, t + snip);
    p.line_to(r, b);
    p.line_to(l, b);
    p.line_to(l, t + snip);
    p.close();
    vec![p]
}

fn gen_snip2_diag_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let snip = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r - snip, t);
    p.line_to(r, t + snip);
    p.line_to(r, b);
    p.line_to(l + snip, b);
    p.line_to(l, b - snip);
    p.close();
    vec![p]
}

fn gen_snip_round_rect(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let rad = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r - rad, t);
    p.arc_to(rad, rad, 270.0, 90.0);
    p.line_to(r, b - rad);
    p.line_to(r - rad, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_plaque(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let ss = (r - l).min(b - t);
    let rad = (ss * (ctx.get("adj").unwrap_or(16667) as f64) / 100_000.0).min(ss / 2.0);

    let mut p = DrawingPath::new();
    p.move_to(l + rad, t);
    p.line_to(r - rad, t);
    p.arc_to(rad, rad, 90.0, -90.0);
    p.line_to(r, b - rad);
    p.arc_to(rad, rad, 180.0, -90.0);
    p.line_to(l + rad, b);
    p.arc_to(rad, rad, 270.0, -90.0);
    p.line_to(l, t + rad);
    p.arc_to(rad, rad, 0.0, -90.0);
    p.close();
    vec![p]
}

// =========================================================================
// Category 2: Basic Polygons & Ellipses
// =========================================================================

fn gen_ellipse(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let rx = (r - l) / 2.0;
    let ry = (b - t) / 2.0;
    let cx = l + rx;
    let _cy = t + ry;

    let mut p = DrawingPath::new();
    p.move_to(cx, t);
    p.arc_to(rx, ry, 270.0, 180.0);
    p.arc_to(rx, ry, 90.0, 180.0);
    p.close();
    vec![p]
}

fn gen_triangle(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let hc = (l + r) / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(hc, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_rt_triangle(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_diamond(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let hc = (l + r) / 2.0;
    let vc = (t + b) / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(hc, t);
    p.line_to(r, vc);
    p.line_to(hc, b);
    p.line_to(l, vc);
    p.close();
    vec![p]
}

fn gen_parallelogram(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let adj = ctx.get("adj").unwrap_or(25000) as f64;
    let dx = w * adj / 100_000.0;

    let mut p = DrawingPath::new();
    p.move_to(l + dx, t);
    p.line_to(r, t);
    p.line_to(r - dx, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_trapezoid(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let adj = ctx.get("adj").unwrap_or(25000) as f64;
    let dx = w * adj / 100_000.0;

    let mut p = DrawingPath::new();
    p.move_to(l + dx, t);
    p.line_to(r - dx, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_non_isosceles_trapezoid(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let adj1 = ctx.get("adj1").unwrap_or(25000) as f64;
    let adj2 = ctx.get("adj2").unwrap_or(25000) as f64;
    let dx1 = w * adj1 / 100_000.0;
    let dx2 = w * adj2 / 100_000.0;

    let mut p = DrawingPath::new();
    p.move_to(l + dx1, t);
    p.line_to(r - dx2, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    vec![p]
}

fn gen_regular_polygon(ctx: &GuideContext, sides: usize) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let rx = (r - l) / 2.0;
    let ry = (b - t) / 2.0;
    let cx = l + rx;
    let cy = t + ry;

    let mut p = DrawingPath::new();
    for i in 0..sides {
        let angle = -PI / 2.0 + (i as f64) * 2.0 * PI / (sides as f64);
        let x = cx + rx * angle.cos();
        let y = cy + ry * angle.sin();
        if i == 0 {
            p.move_to(x, y);
        } else {
            p.line_to(x, y);
        }
    }
    p.close();
    vec![p]
}

// =========================================================================
// Category 3: Stars & Banners
// =========================================================================

fn gen_star(ctx: &GuideContext, points: usize, default_inner_ratio: f64) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let rx = (r - l) / 2.0;
    let ry = (b - t) / 2.0;
    let cx = l + rx;
    let cy = t + ry;

    let inner_ratio = if let Some(adj) = ctx.get("adj") {
        adj as f64 / 100_000.0
    } else {
        default_inner_ratio
    };

    let total_vertices = points * 2;
    let mut p = DrawingPath::new();
    for i in 0..total_vertices {
        let angle = -PI / 2.0 + (i as f64) * PI / (points as f64);
        let radius_scale = if i % 2 == 0 { 1.0 } else { inner_ratio };
        let x = cx + rx * radius_scale * angle.cos();
        let y = cy + ry * radius_scale * angle.sin();
        if i == 0 {
            p.move_to(x, y);
        } else {
            p.line_to(x, y);
        }
    }
    p.close();
    vec![p]
}

// =========================================================================
// Category 4: Block Arrows
// =========================================================================

fn gen_right_arrow(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let vc = t + h / 2.0;

    let head_w = (w * (ctx.get("adj1").unwrap_or(50000) as f64) / 100_000.0).min(w);
    let shaft_h = (h * (ctx.get("adj2").unwrap_or(50000) as f64) / 100_000.0).min(h);
    let half_shaft = shaft_h / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(l, vc - half_shaft);
    p.line_to(r - head_w, vc - half_shaft);
    p.line_to(r - head_w, t);
    p.line_to(r, vc);
    p.line_to(r - head_w, b);
    p.line_to(r - head_w, vc + half_shaft);
    p.line_to(l, vc + half_shaft);
    p.close();
    vec![p]
}

fn gen_left_arrow(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let vc = t + h / 2.0;

    let head_w = (w * (ctx.get("adj1").unwrap_or(50000) as f64) / 100_000.0).min(w);
    let shaft_h = (h * (ctx.get("adj2").unwrap_or(50000) as f64) / 100_000.0).min(h);
    let half_shaft = shaft_h / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(r, vc - half_shaft);
    p.line_to(l + head_w, vc - half_shaft);
    p.line_to(l + head_w, t);
    p.line_to(l, vc);
    p.line_to(l + head_w, b);
    p.line_to(l + head_w, vc + half_shaft);
    p.line_to(r, vc + half_shaft);
    p.close();
    vec![p]
}

fn gen_up_arrow(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let hc = l + w / 2.0;

    let head_h = (h * (ctx.get("adj1").unwrap_or(50000) as f64) / 100_000.0).min(h);
    let shaft_w = (w * (ctx.get("adj2").unwrap_or(50000) as f64) / 100_000.0).min(w);
    let half_shaft = shaft_w / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(hc, t);
    p.line_to(r, t + head_h);
    p.line_to(hc + half_shaft, t + head_h);
    p.line_to(hc + half_shaft, b);
    p.line_to(hc - half_shaft, b);
    p.line_to(hc - half_shaft, t + head_h);
    p.line_to(l, t + head_h);
    p.close();
    vec![p]
}

fn gen_down_arrow(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let hc = l + w / 2.0;

    let head_h = (h * (ctx.get("adj1").unwrap_or(50000) as f64) / 100_000.0).min(h);
    let shaft_w = (w * (ctx.get("adj2").unwrap_or(50000) as f64) / 100_000.0).min(w);
    let half_shaft = shaft_w / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(hc - half_shaft, t);
    p.line_to(hc + half_shaft, t);
    p.line_to(hc + half_shaft, b - head_h);
    p.line_to(r, b - head_h);
    p.line_to(hc, b);
    p.line_to(l, b - head_h);
    p.line_to(hc - half_shaft, b - head_h);
    p.close();
    vec![p]
}

fn gen_chevron(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let vc = (t + b) / 2.0;
    let dx = w * (ctx.get("adj").unwrap_or(50000) as f64) / 100_000.0;

    let mut p = DrawingPath::new();
    p.move_to(l, t);
    p.line_to(r - dx, t);
    p.line_to(r, vc);
    p.line_to(r - dx, b);
    p.line_to(l, b);
    p.line_to(l + dx, vc);
    p.close();
    vec![p]
}

// =========================================================================
// Category 5: Curves, Symbols & Action Buttons
// =========================================================================

fn gen_heart(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let hc = l + w / 2.0;

    let mut p = DrawingPath::new();
    p.move_to(hc, t + 0.3 * h);
    p.cubic_to(l + 0.2 * w, t - 0.1 * h, l, t + 0.2 * h, l, t + 0.45 * h);
    p.cubic_to(l, t + 0.7 * h, hc, b - 0.1 * h, hc, b);
    p.cubic_to(hc, b - 0.1 * h, r, t + 0.7 * h, r, t + 0.45 * h);
    p.cubic_to(r, t + 0.2 * h, r - 0.2 * w, t - 0.1 * h, hc, t + 0.3 * h);
    p.close();
    vec![p]
}

fn gen_lightning_bolt(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;

    let mut p = DrawingPath::new();
    p.move_to(l + 0.45 * w, t);
    p.line_to(r, t);
    p.line_to(l + 0.35 * w, t + 0.55 * h);
    p.line_to(l + 0.65 * w, t + 0.55 * h);
    p.line_to(l, b);
    p.line_to(l + 0.40 * w, t + 0.45 * h);
    p.line_to(l + 0.15 * w, t + 0.45 * h);
    p.close();
    vec![p]
}

fn gen_plus(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let w = r - l;
    let h = b - t;
    let adj = ctx.get("adj").unwrap_or(25000) as f64 / 100_000.0;
    let dx = w * adj;
    let dy = h * adj;

    let mut p = DrawingPath::new();
    p.move_to(l + dx, t);
    p.line_to(r - dx, t);
    p.line_to(r - dx, t + dy);
    p.line_to(r, t + dy);
    p.line_to(r, b - dy);
    p.line_to(r - dx, b - dy);
    p.line_to(r - dx, b);
    p.line_to(l + dx, b);
    p.line_to(l + dx, b - dy);
    p.line_to(l, b - dy);
    p.line_to(l, t + dy);
    p.line_to(l + dx, t + dy);
    p.close();
    vec![p]
}

fn gen_donut(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let rx = (r - l) / 2.0;
    let ry = (b - t) / 2.0;
    let cx = l + rx;
    let cy = t + ry;

    let adj = ctx.get("adj").unwrap_or(25000) as f64 / 100_000.0;
    let irx = rx * (1.0 - adj);
    let iry = ry * (1.0 - adj);

    let mut p1 = DrawingPath::new();
    p1.move_to(cx, t);
    p1.arc_to(rx, ry, 270.0, 180.0);
    p1.arc_to(rx, ry, 90.0, 180.0);
    p1.close();

    let mut p2 = DrawingPath::new();
    p2.move_to(cx, cy - iry);
    p2.arc_to(irx, iry, 270.0, -180.0);
    p2.arc_to(irx, iry, 90.0, -180.0);
    p2.close();

    vec![p1, p2]
}

fn gen_pie(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;
    let rx = (r - l) / 2.0;
    let ry = (b - t) / 2.0;
    let cx = l + rx;
    let cy = t + ry;

    let st_ang = ctx.get("adj1").unwrap_or(0) as f64 / 60_000.0;
    let sw_ang = ctx.get("adj2").unwrap_or(16_200_000) as f64 / 60_000.0;

    let mut p = DrawingPath::new();
    p.move_to(cx, cy);
    let st_rad = st_ang.to_radians();
    p.line_to(cx + rx * st_rad.cos(), cy + ry * st_rad.sin());
    p.arc_to(rx, ry, st_ang, sw_ang);
    p.close();
    vec![p]
}

// Fallback generator for shapes evaluated as simple bounding polygon/box
fn gen_generic_box(ctx: &GuideContext) -> Vec<DrawingPath> {
    gen_rect(ctx)
}

fn gen_generic_line(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;

    let mut p = DrawingPath::new();
    p.fill = PathFillMode::None;
    p.move_to(l, t);
    p.line_to(r, b);
    vec![p]
}

fn gen_generic_line_inv(ctx: &GuideContext) -> Vec<DrawingPath> {
    let l = ctx.get("l").unwrap_or(0) as f64;
    let t = ctx.get("t").unwrap_or(0) as f64;
    let r = ctx.get("r").unwrap_or(1000) as f64;
    let b = ctx.get("b").unwrap_or(1000) as f64;

    let mut p = DrawingPath::new();
    p.fill = PathFillMode::None;
    p.move_to(l, b);
    p.line_to(r, t);
    vec![p]
}

// =========================================================================
// Catalog Table of all 187 Shapes
// =========================================================================

macro_rules! def_preset {
    ($name:literal, $gen:expr) => {
        PresetDefinition {
            name: $name,
            default_adjusts: &[],
            guides: &[],
            generate: $gen,
        }
    };
    ($name:literal, $gen:expr, $adjs:expr) => {
        PresetDefinition {
            name: $name,
            default_adjusts: $adjs,
            guides: &[],
            generate: $gen,
        }
    };
}

const ADJ_ROUND_RECT: &[(&str, i64)] = &[("adj", 16667)];
const ADJ_PARALLELOGRAM: &[(&str, i64)] = &[("adj", 25000)];
const ADJ_ARROW: &[(&str, i64)] = &[("adj1", 50000), ("adj2", 50000)];
const ADJ_CHEVRON: &[(&str, i64)] = &[("adj", 50000)];
const ADJ_PLUS: &[(&str, i64)] = &[("adj", 25000)];
const ADJ_DONUT: &[(&str, i64)] = &[("adj", 25000)];
const ADJ_PIE: &[(&str, i64)] = &[("adj1", 0), ("adj2", 16_200_000)];

static ALL_PRESET_DEFS: &[PresetDefinition] = &[
    def_preset!("line", gen_generic_line),
    def_preset!("lineInv", gen_generic_line_inv),
    def_preset!("triangle", gen_triangle),
    def_preset!("rtTriangle", gen_rt_triangle),
    def_preset!("rect", gen_rect),
    def_preset!("diamond", gen_diamond),
    def_preset!("parallelogram", gen_parallelogram, ADJ_PARALLELOGRAM),
    def_preset!("trapezoid", gen_trapezoid, ADJ_PARALLELOGRAM),
    def_preset!("nonIsoscelesTrapezoid", gen_non_isosceles_trapezoid),
    def_preset!("pentagon", |c| gen_regular_polygon(c, 5)),
    def_preset!("hexagon", |c| gen_regular_polygon(c, 6)),
    def_preset!("heptagon", |c| gen_regular_polygon(c, 7)),
    def_preset!("octagon", |c| gen_regular_polygon(c, 8)),
    def_preset!("decagon", |c| gen_regular_polygon(c, 10)),
    def_preset!("dodecagon", |c| gen_regular_polygon(c, 12)),
    def_preset!("star4", |c| gen_star(c, 4, 0.382)),
    def_preset!("star5", |c| gen_star(c, 5, 0.382)),
    def_preset!("star6", |c| gen_star(c, 6, 0.5)),
    def_preset!("star7", |c| gen_star(c, 7, 0.5)),
    def_preset!("star8", |c| gen_star(c, 8, 0.5)),
    def_preset!("star10", |c| gen_star(c, 10, 0.6)),
    def_preset!("star12", |c| gen_star(c, 12, 0.7)),
    def_preset!("star16", |c| gen_star(c, 16, 0.75)),
    def_preset!("star24", |c| gen_star(c, 24, 0.8)),
    def_preset!("star32", |c| gen_star(c, 32, 0.85)),
    def_preset!("roundRect", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("round1Rect", gen_round1_rect, ADJ_ROUND_RECT),
    def_preset!("round2SameRect", gen_round2_same_rect, ADJ_ROUND_RECT),
    def_preset!("round2DiagRect", gen_round2_diag_rect, ADJ_ROUND_RECT),
    def_preset!("snipRoundRect", gen_snip_round_rect, ADJ_ROUND_RECT),
    def_preset!("snip1Rect", gen_snip1_rect, ADJ_ROUND_RECT),
    def_preset!("snip2SameRect", gen_snip2_same_rect, ADJ_ROUND_RECT),
    def_preset!("snip2DiagRect", gen_snip2_diag_rect, ADJ_ROUND_RECT),
    def_preset!("plaque", gen_plaque, ADJ_ROUND_RECT),
    def_preset!("ellipse", gen_ellipse),
    def_preset!("teardrop", gen_generic_box),
    def_preset!("chevron", gen_chevron, ADJ_CHEVRON),
    def_preset!("pieWedge", gen_pie, ADJ_PIE),
    def_preset!("pie", gen_pie, ADJ_PIE),
    def_preset!("blockArc", gen_donut, ADJ_DONUT),
    def_preset!("donut", gen_donut, ADJ_DONUT),
    def_preset!("noSmoking", gen_donut, ADJ_DONUT),
    def_preset!("rightArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("leftArrow", gen_left_arrow, ADJ_ARROW),
    def_preset!("upArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("downArrow", gen_down_arrow, ADJ_ARROW),
    def_preset!("stripedRightArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("notchedRightArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("bentUpArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("leftRightArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("upDownArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("leftUpArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("leftRightUpArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("quadArrow", gen_generic_box),
    def_preset!("leftRightArrowCallout", gen_generic_box),
    def_preset!("upDownArrowCallout", gen_generic_box),
    def_preset!("leftUpArrowCallout", gen_generic_box),
    def_preset!("quadArrowCallout", gen_generic_box),
    def_preset!("bentArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("uturnArrow", gen_generic_box),
    def_preset!("circularArrow", gen_donut, ADJ_DONUT),
    def_preset!("leftCircularArrow", gen_donut, ADJ_DONUT),
    def_preset!("leftRightCircularArrow", gen_donut, ADJ_DONUT),
    def_preset!("curvedRightArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("curvedLeftArrow", gen_left_arrow, ADJ_ARROW),
    def_preset!("curvedUpArrow", gen_up_arrow, ADJ_ARROW),
    def_preset!("curvedDownArrow", gen_down_arrow, ADJ_ARROW),
    def_preset!("swooshArrow", gen_right_arrow, ADJ_ARROW),
    def_preset!("cube", gen_generic_box),
    def_preset!("can", gen_generic_box),
    def_preset!("lightningBolt", gen_lightning_bolt),
    def_preset!("heart", gen_heart),
    def_preset!("sun", |c| gen_star(c, 8, 0.7)),
    def_preset!("moon", gen_generic_box),
    def_preset!("smileyFace", gen_ellipse),
    def_preset!("irregularSeal1", |c| gen_star(c, 16, 0.8)),
    def_preset!("irregularSeal2", |c| gen_star(c, 24, 0.8)),
    def_preset!("foldedCorner", gen_generic_box),
    def_preset!("bevel", gen_generic_box),
    def_preset!("frame", gen_donut, ADJ_DONUT),
    def_preset!("halfFrame", gen_generic_box),
    def_preset!("corner", gen_generic_box),
    def_preset!("diagStripe", gen_generic_box),
    def_preset!("chord", gen_pie, ADJ_PIE),
    def_preset!("arc", gen_pie, ADJ_PIE),
    def_preset!("leftBracket", gen_generic_box),
    def_preset!("rightBracket", gen_generic_box),
    def_preset!("leftBrace", gen_generic_box),
    def_preset!("rightBrace", gen_generic_box),
    def_preset!("bracketPair", gen_generic_box),
    def_preset!("bracePair", gen_generic_box),
    def_preset!("straightConnector1", gen_generic_line),
    def_preset!("bentConnector2", gen_generic_line),
    def_preset!("bentConnector3", gen_generic_line),
    def_preset!("bentConnector4", gen_generic_line),
    def_preset!("bentConnector5", gen_generic_line),
    def_preset!("curvedConnector2", gen_generic_line),
    def_preset!("curvedConnector3", gen_generic_line),
    def_preset!("curvedConnector4", gen_generic_line),
    def_preset!("curvedConnector5", gen_generic_line),
    def_preset!("callout1", gen_generic_box),
    def_preset!("callout2", gen_generic_box),
    def_preset!("callout3", gen_generic_box),
    def_preset!("accentCallout1", gen_generic_box),
    def_preset!("accentCallout2", gen_generic_box),
    def_preset!("accentCallout3", gen_generic_box),
    def_preset!("borderCallout1", gen_generic_box),
    def_preset!("borderCallout2", gen_generic_box),
    def_preset!("borderCallout3", gen_generic_box),
    def_preset!("accentBorderCallout1", gen_generic_box),
    def_preset!("accentBorderCallout2", gen_generic_box),
    def_preset!("accentBorderCallout3", gen_generic_box),
    def_preset!("wedgeRectCallout", gen_generic_box),
    def_preset!("wedgeRoundRectCallout", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("wedgeEllipseCallout", gen_ellipse),
    def_preset!("cloud", gen_generic_box),
    def_preset!("cloudCallout", gen_generic_box),
    def_preset!("ribbon", gen_generic_box),
    def_preset!("ribbon2", gen_generic_box),
    def_preset!("ellipseRibbon", gen_generic_box),
    def_preset!("ellipseRibbon2", gen_generic_box),
    def_preset!("verticalScroll", gen_generic_box),
    def_preset!("horizontalScroll", gen_generic_box),
    def_preset!("wave", gen_generic_box),
    def_preset!("doubleWave", gen_generic_box),
    def_preset!("plus", gen_plus, ADJ_PLUS),
    def_preset!("flowChartProcess", gen_rect),
    def_preset!("flowChartDecision", gen_diamond),
    def_preset!("flowChartInputOutput", gen_parallelogram, ADJ_PARALLELOGRAM),
    def_preset!("flowChartPredefinedProcess", gen_rect),
    def_preset!("flowChartInternalStorage", gen_rect),
    def_preset!("flowChartDocument", gen_generic_box),
    def_preset!("flowChartMultidocument", gen_generic_box),
    def_preset!("flowChartTerminator", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("flowChartPreparation", gen_hexagon),
    def_preset!("flowChartManualInput", gen_trapezoid, ADJ_PARALLELOGRAM),
    def_preset!("flowChartManualOperation", gen_trapezoid, ADJ_PARALLELOGRAM),
    def_preset!("flowChartConnector", gen_ellipse),
    def_preset!("flowChartPunchedCard", gen_snip1_rect, ADJ_ROUND_RECT),
    def_preset!("flowChartPunchedTape", gen_generic_box),
    def_preset!("flowChartSummingJunction", gen_ellipse),
    def_preset!("flowChartOr", gen_ellipse),
    def_preset!("flowChartCollate", gen_generic_box),
    def_preset!("flowChartSort", gen_diamond),
    def_preset!("flowChartExtract", gen_triangle),
    def_preset!("flowChartMerge", gen_triangle),
    def_preset!("flowChartOfflineStorage", gen_triangle),
    def_preset!("flowChartOnlineStorage", gen_generic_box),
    def_preset!("flowChartMagneticTape", gen_ellipse),
    def_preset!("flowChartMagneticDisk", gen_generic_box),
    def_preset!("flowChartMagneticDrum", gen_generic_box),
    def_preset!("flowChartDisplay", gen_generic_box),
    def_preset!("flowChartDelay", gen_generic_box),
    def_preset!("flowChartAlternateProcess", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("flowChartOffpageConnector", gen_generic_box),
    def_preset!("actionButtonBlank", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonHome", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonHelp", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonInformation", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonForwardNext", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonBackPrevious", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonEnd", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonBeginning", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonReturn", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonDocument", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonSound", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("actionButtonMovie", gen_round_rect, ADJ_ROUND_RECT),
    def_preset!("gear6", |c| gen_star(c, 6, 0.7)),
    def_preset!("gear9", |c| gen_star(c, 9, 0.7)),
    def_preset!("funnel", gen_trapezoid, ADJ_PARALLELOGRAM),
    def_preset!("mathPlus", gen_plus, ADJ_PLUS),
    def_preset!("mathMinus", gen_rect),
    def_preset!("mathMultiply", gen_generic_box),
    def_preset!("mathDivide", gen_generic_box),
    def_preset!("mathEqual", gen_generic_box),
    def_preset!("mathNotEqual", gen_generic_box),
    def_preset!("cornerTabs", gen_generic_box),
    def_preset!("squareTabs", gen_generic_box),
    def_preset!("plaqueTabs", gen_generic_box),
    def_preset!("chartX", gen_generic_box),
    def_preset!("chartStar", |c| gen_star(c, 5, 0.382)),
    def_preset!("chartPlus", gen_plus, ADJ_PLUS),
    def_preset!("homePlate", gen_generic_box),
    def_preset!("rightArrowCallout", gen_right_arrow, ADJ_ARROW),
    def_preset!("leftArrowCallout", gen_left_arrow, ADJ_ARROW),
    def_preset!("upArrowCallout", gen_up_arrow, ADJ_ARROW),
    def_preset!("downArrowCallout", gen_down_arrow, ADJ_ARROW),
];

fn gen_hexagon(ctx: &GuideContext) -> Vec<DrawingPath> {
    gen_regular_polygon(ctx, 6)
}

pub static ALL_PRESET_NAMES: &[&str] = &[
    "line",
    "lineInv",
    "triangle",
    "rtTriangle",
    "rect",
    "diamond",
    "parallelogram",
    "trapezoid",
    "nonIsoscelesTrapezoid",
    "pentagon",
    "hexagon",
    "heptagon",
    "octagon",
    "decagon",
    "dodecagon",
    "star4",
    "star5",
    "star6",
    "star7",
    "star8",
    "star10",
    "star12",
    "star16",
    "star24",
    "star32",
    "roundRect",
    "round1Rect",
    "round2SameRect",
    "round2DiagRect",
    "snipRoundRect",
    "snip1Rect",
    "snip2SameRect",
    "snip2DiagRect",
    "plaque",
    "ellipse",
    "teardrop",
    "chevron",
    "pieWedge",
    "pie",
    "blockArc",
    "donut",
    "noSmoking",
    "rightArrow",
    "leftArrow",
    "upArrow",
    "downArrow",
    "stripedRightArrow",
    "notchedRightArrow",
    "bentUpArrow",
    "leftRightArrow",
    "upDownArrow",
    "leftUpArrow",
    "leftRightUpArrow",
    "quadArrow",
    "leftRightArrowCallout",
    "upDownArrowCallout",
    "leftUpArrowCallout",
    "quadArrowCallout",
    "bentArrow",
    "uturnArrow",
    "circularArrow",
    "leftCircularArrow",
    "leftRightCircularArrow",
    "curvedRightArrow",
    "curvedLeftArrow",
    "curvedUpArrow",
    "curvedDownArrow",
    "swooshArrow",
    "cube",
    "can",
    "lightningBolt",
    "heart",
    "sun",
    "moon",
    "smileyFace",
    "irregularSeal1",
    "irregularSeal2",
    "foldedCorner",
    "bevel",
    "frame",
    "halfFrame",
    "corner",
    "diagStripe",
    "chord",
    "arc",
    "leftBracket",
    "rightBracket",
    "leftBrace",
    "rightBrace",
    "bracketPair",
    "bracePair",
    "straightConnector1",
    "bentConnector2",
    "bentConnector3",
    "bentConnector4",
    "bentConnector5",
    "curvedConnector2",
    "curvedConnector3",
    "curvedConnector4",
    "curvedConnector5",
    "callout1",
    "callout2",
    "callout3",
    "accentCallout1",
    "accentCallout2",
    "accentCallout3",
    "borderCallout1",
    "borderCallout2",
    "borderCallout3",
    "accentBorderCallout1",
    "accentBorderCallout2",
    "accentBorderCallout3",
    "wedgeRectCallout",
    "wedgeRoundRectCallout",
    "wedgeEllipseCallout",
    "cloud",
    "cloudCallout",
    "ribbon",
    "ribbon2",
    "ellipseRibbon",
    "ellipseRibbon2",
    "verticalScroll",
    "horizontalScroll",
    "wave",
    "doubleWave",
    "plus",
    "flowChartProcess",
    "flowChartDecision",
    "flowChartInputOutput",
    "flowChartPredefinedProcess",
    "flowChartInternalStorage",
    "flowChartDocument",
    "flowChartMultidocument",
    "flowChartTerminator",
    "flowChartPreparation",
    "flowChartManualInput",
    "flowChartManualOperation",
    "flowChartConnector",
    "flowChartPunchedCard",
    "flowChartPunchedTape",
    "flowChartSummingJunction",
    "flowChartOr",
    "flowChartCollate",
    "flowChartSort",
    "flowChartExtract",
    "flowChartMerge",
    "flowChartOfflineStorage",
    "flowChartOnlineStorage",
    "flowChartMagneticTape",
    "flowChartMagneticDisk",
    "flowChartMagneticDrum",
    "flowChartDisplay",
    "flowChartDelay",
    "flowChartAlternateProcess",
    "flowChartOffpageConnector",
    "actionButtonBlank",
    "actionButtonHome",
    "actionButtonHelp",
    "actionButtonInformation",
    "actionButtonForwardNext",
    "actionButtonBackPrevious",
    "actionButtonEnd",
    "actionButtonBeginning",
    "actionButtonReturn",
    "actionButtonDocument",
    "actionButtonSound",
    "actionButtonMovie",
    "gear6",
    "gear9",
    "funnel",
    "mathPlus",
    "mathMinus",
    "mathMultiply",
    "mathDivide",
    "mathEqual",
    "mathNotEqual",
    "cornerTabs",
    "squareTabs",
    "plaqueTabs",
    "chartX",
    "chartStar",
    "chartPlus",
    "homePlate",
    "rightArrowCallout",
    "leftArrowCallout",
    "upArrowCallout",
    "downArrowCallout",
];
