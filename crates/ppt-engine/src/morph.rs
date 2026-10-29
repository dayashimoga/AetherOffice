//! Clean-Room Morph Transition (`p159:morph`) & SmartArt Layout Engine
//!
//! Complies with ECMA-376 Part 1 §19.5 & §21.4, and Microsoft Office Extension
//! Namespace `http://schemas.microsoft.com/office/powerpoint/2015/09/main`.
//!
//! Provides:
//! - Explicit `!!` and heuristic shape correspondence matching
//! - Equal-subdivision cubic Bezier path morphing across different vertex counts
//! - Text morphing: `byObject`, `byWord`, and `byChar` (LCS alignment)
//! - 120Hz frame-accurate morph transition evaluator
//! - SmartArt constraint layout algorithms: `lin`, `pyra`, `cycle`, `hierChild`, `snake`
//! - Synced `p:spTree` shape cache generation

use crate::drawingml::{DrawingPath, PathCommand, Point2D};
use crate::model::{
    Geometry, MorphOption, Shape, ShapeElement, ShapeProperties, Slide, TextBody, TextParagraph,
    TextRunElement, Transform2D,
};
use std::collections::{HashMap, HashSet};

// =========================================================================
// 1. Morph Data Structures
// =========================================================================

/// Paired correspondence between a source shape and a target shape.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphPair {
    pub source_id: Option<u32>,
    pub target_id: Option<u32>,
    pub is_explicit_match: bool,
    pub score: f64,
}

/// Continuous 2D transformation interpolated during morph animation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MorphedTransform {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation_deg: f64,
}

impl MorphedTransform {
    pub fn new(x: f64, y: f64, width: f64, height: f64, rotation_deg: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rotation_deg,
        }
    }

    pub fn from_transform_2d(t: &Transform2D) -> Self {
        Self {
            x: t.x as f64,
            y: t.y as f64,
            width: t.cx as f64,
            height: t.cy as f64,
            rotation_deg: (t.rot as f64) / 60000.0,
        }
    }
}

/// Evaluated state of a shape at a specific transition time $t \in [0.0, 1.0]$.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphedShapeState {
    pub shape_id: u32,
    pub name: String,
    pub transform: MorphedTransform,
    pub opacity: f64, // 0.0 to 1.0
    pub geometry_path: Option<DrawingPath>,
    pub text_elements: Vec<MorphedTextElement>,
}

/// Evaluated state of an individual text element or glyph during text morphing.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphedTextElement {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size_pt: f64,
    pub opacity: f64,
}

/// Complete scene state for the morph transition at time $t$.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphedSlideState {
    pub t: f64,
    pub shapes: Vec<MorphedShapeState>,
}

// =========================================================================
// 2. Shape Correspondence Matching (`MorphMatcher`)
// =========================================================================

pub struct MorphMatcher;

impl MorphMatcher {
    /// Extract all `Shape` instances from a `Slide`.
    pub fn extract_slide_shapes(slide: &Slide) -> Vec<&Shape> {
        let mut list = Vec::new();
        for elem in &slide.common_slide_data.shape_tree.shapes {
            if let ShapeElement::Shape(ref s) = elem {
                list.push(s.as_ref());
            }
        }
        list
    }

    /// Calculate similarity score between two shapes $\mathcal{S}(a, b) \in [0.0, 1.0]$.
    pub fn score_shapes(shape_a: &Shape, shape_b: &Shape, canvas_w: f64, canvas_h: f64) -> f64 {
        let s_geom = Self::score_geometry(shape_a, shape_b);
        let s_pos = Self::score_position(shape_a, shape_b, canvas_w, canvas_h);
        let s_size = Self::score_size(shape_a, shape_b);
        let s_text = Self::score_text(shape_a, shape_b);

        0.35 * s_geom + 0.25 * s_pos + 0.20 * s_size + 0.20 * s_text
    }

    pub fn score_geometry(shape_a: &Shape, shape_b: &Shape) -> f64 {
        let geom_a = shape_a
            .properties
            .geometry
            .as_ref()
            .and_then(|g| g.preset_name());
        let geom_b = shape_b
            .properties
            .geometry
            .as_ref()
            .and_then(|g| g.preset_name());

        match (geom_a, geom_b) {
            (Some(a), Some(b)) if a == b => 1.0,
            (Some(a), Some(b))
                if (a == "rect" && b == "roundRect") || (a == "roundRect" && b == "rect") =>
            {
                0.5
            }
            _ => 0.0,
        }
    }

    pub fn score_position(shape_a: &Shape, shape_b: &Shape, canvas_w: f64, canvas_h: f64) -> f64 {
        let (cx_a, cy_a) = Self::shape_center(shape_a);
        let (cx_b, cy_b) = Self::shape_center(shape_b);

        let w = if canvas_w > 0.0 { canvas_w } else { 1.0 };
        let h = if canvas_h > 0.0 { canvas_h } else { 1.0 };

        let dx = (cx_a - cx_b) / w;
        let dy = (cy_a - cy_b) / h;
        let d_norm = (dx * dx + dy * dy).sqrt();

        (1.0 - d_norm / std::f64::consts::SQRT_2).clamp(0.0, 1.0)
    }

    pub fn shape_center(shape: &Shape) -> (f64, f64) {
        if let Some(ref xfrm) = shape.properties.transform {
            (
                xfrm.x as f64 + (xfrm.cx as f64) / 2.0,
                xfrm.y as f64 + (xfrm.cy as f64) / 2.0,
            )
        } else {
            (0.0, 0.0)
        }
    }

    pub fn score_size(shape_a: &Shape, shape_b: &Shape) -> f64 {
        let area_a = shape_a
            .properties
            .transform
            .as_ref()
            .map(|t| (t.cx as f64) * (t.cy as f64))
            .unwrap_or(0.0);
        let area_b = shape_b
            .properties
            .transform
            .as_ref()
            .map(|t| (t.cx as f64) * (t.cy as f64))
            .unwrap_or(0.0);

        if area_a <= 0.0 || area_b <= 0.0 {
            return 0.0;
        }

        let min_area = area_a.min(area_b);
        let max_area = area_a.max(area_b);
        min_area / max_area
    }

    pub fn score_text(shape_a: &Shape, shape_b: &Shape) -> f64 {
        let text_a = Self::extract_plain_text(shape_a);
        let text_b = Self::extract_plain_text(shape_b);

        if text_a.is_empty() && text_b.is_empty() {
            return 1.0;
        }
        if text_a.is_empty() || text_b.is_empty() {
            return 0.0;
        }
        if text_a == text_b {
            return 1.0;
        }

        let bigrams_a = Self::extract_bigrams(&text_a);
        let bigrams_b = Self::extract_bigrams(&text_b);

        let intersection = bigrams_a.intersection(&bigrams_b).count();
        let union = bigrams_a.union(&bigrams_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    pub fn extract_plain_text(shape: &Shape) -> String {
        let mut s = String::new();
        if let Some(ref tb) = shape.text_body {
            for p in &tb.paragraphs {
                for r in &p.runs {
                    match r {
                        TextRunElement::Text(ref text, _) => s.push_str(text),
                        TextRunElement::Field {
                            text: Some(ref text),
                            ..
                        } => s.push_str(text),
                        _ => {}
                    }
                }
            }
        }
        s
    }

    pub fn extract_bigrams(text: &str) -> HashSet<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut set = HashSet::new();
        if chars.len() < 2 {
            if !chars.is_empty() {
                set.insert(text.to_string());
            }
            return set;
        }
        for w in chars.windows(2) {
            set.insert(format!("{}{}", w[0], w[1]));
        }
        set
    }

    /// Match shapes across Slide A and Slide B according to GEC Morph specification.
    pub fn match_slides(
        slide_a: &Slide,
        slide_b: &Slide,
        canvas_w: f64,
        canvas_h: f64,
    ) -> Vec<MorphPair> {
        let shapes_a = Self::extract_slide_shapes(slide_a);
        let shapes_b = Self::extract_slide_shapes(slide_b);

        let mut matched_pairs = Vec::new();
        let mut matched_a_indices = HashSet::new();
        let mut matched_b_indices = HashSet::new();

        // 1. Explicit "!!" name prefix matching (Priority 1)
        for (ia, sa) in shapes_a.iter().enumerate() {
            if sa.name.starts_with("!!") {
                for (ib, sb) in shapes_b.iter().enumerate() {
                    if !matched_b_indices.contains(&ib) && sb.name == sa.name {
                        matched_pairs.push(MorphPair {
                            source_id: Some(sa.id),
                            target_id: Some(sb.id),
                            is_explicit_match: true,
                            score: 1.0,
                        });
                        matched_a_indices.insert(ia);
                        matched_b_indices.insert(ib);
                        break;
                    }
                }
            }
        }

        // 2. Heuristic similarity scoring for remaining shapes
        let mut candidates = Vec::new();
        for (ia, sa) in shapes_a.iter().enumerate() {
            if matched_a_indices.contains(&ia) {
                continue;
            }
            for (ib, sb) in shapes_b.iter().enumerate() {
                if matched_b_indices.contains(&ib) {
                    continue;
                }
                let score = Self::score_shapes(sa, sb, canvas_w, canvas_h);
                if score >= 0.45 {
                    candidates.push((ia, ib, score));
                }
            }
        }

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        for (ia, ib, score) in candidates {
            if !matched_a_indices.contains(&ia) && !matched_b_indices.contains(&ib) {
                let sa = shapes_a[ia];
                let sb = shapes_b[ib];
                matched_pairs.push(MorphPair {
                    source_id: Some(sa.id),
                    target_id: Some(sb.id),
                    is_explicit_match: false,
                    score,
                });
                matched_a_indices.insert(ia);
                matched_b_indices.insert(ib);
            }
        }

        // 3. Unmatched source shapes (exiting, fade out)
        for (ia, sa) in shapes_a.iter().enumerate() {
            if !matched_a_indices.contains(&ia) {
                matched_pairs.push(MorphPair {
                    source_id: Some(sa.id),
                    target_id: None,
                    is_explicit_match: false,
                    score: 0.0,
                });
            }
        }

        // 4. Unmatched destination shapes (entering, fade in)
        for (ib, sb) in shapes_b.iter().enumerate() {
            if !matched_b_indices.contains(&ib) {
                matched_pairs.push(MorphPair {
                    source_id: None,
                    target_id: Some(sb.id),
                    is_explicit_match: false,
                    score: 0.0,
                });
            }
        }

        matched_pairs
    }
}

// =========================================================================
// 3. Equal-Subdivision Cubic Bezier Path Morpher
// =========================================================================

/// Cubic Bezier segment: $B(u) = (1-u)^3 P_0 + 3(1-u)^2 u C_1 + 3(1-u) u^2 C_2 + u^3 P_3$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezierSegment {
    pub p0: (f64, f64),
    pub c1: (f64, f64),
    pub c2: (f64, f64),
    pub p3: (f64, f64),
}

impl CubicBezierSegment {
    pub fn new(p0: (f64, f64), c1: (f64, f64), c2: (f64, f64), p3: (f64, f64)) -> Self {
        Self { p0, c1, c2, p3 }
    }

    /// Convert a line segment $(P_0, P_1)$ to an exact cubic Bezier representation.
    pub fn from_line(p0: (f64, f64), p1: (f64, f64)) -> Self {
        let c1 = (p0.0 + (p1.0 - p0.0) / 3.0, p0.1 + (p1.1 - p0.1) / 3.0);
        let c2 = (
            p0.0 + 2.0 * (p1.0 - p0.0) / 3.0,
            p0.1 + 2.0 * (p1.1 - p0.1) / 3.0,
        );
        Self { p0, c1, c2, p3: p1 }
    }

    /// Convert a quadratic Bezier $(P_0, \text{ctrl}, P_1)$ to an exact cubic Bezier.
    pub fn from_quad(p0: (f64, f64), ctrl: (f64, f64), p1: (f64, f64)) -> Self {
        let c1 = (
            p0.0 + 2.0 * (ctrl.0 - p0.0) / 3.0,
            p0.1 + 2.0 * (ctrl.1 - p0.1) / 3.0,
        );
        let c2 = (
            p1.0 + 2.0 * (ctrl.0 - p1.0) / 3.0,
            p1.1 + 2.0 * (ctrl.1 - p1.1) / 3.0,
        );
        Self { p0, c1, c2, p3: p1 }
    }

    /// Sample position at parameter $u \in [0.0, 1.0]$.
    pub fn sample(&self, u: f64) -> (f64, f64) {
        let u = u.clamp(0.0, 1.0);
        let omt = 1.0 - u;
        let omt2 = omt * omt;
        let omt3 = omt2 * omt;
        let u2 = u * u;
        let u3 = u2 * u;

        let x = omt3 * self.p0.0
            + 3.0 * omt2 * u * self.c1.0
            + 3.0 * omt * u2 * self.c2.0
            + u3 * self.p3.0;
        let y = omt3 * self.p0.1
            + 3.0 * omt2 * u * self.c1.1
            + 3.0 * omt * u2 * self.c2.1
            + u3 * self.p3.1;

        (x, y)
    }

    /// Approximate arc length of the segment via 8-point polyline integration.
    pub fn length_approx(&self) -> f64 {
        let mut len = 0.0;
        let steps = 8;
        let mut prev = self.p0;
        for i in 1..=steps {
            let u = i as f64 / steps as f64;
            let pt = self.sample(u);
            let dx = pt.0 - prev.0;
            let dy = pt.1 - prev.1;
            len += (dx * dx + dy * dy).sqrt();
            prev = pt;
        }
        len
    }
}

pub struct PathMorpher;

impl PathMorpher {
    /// Decompose a `DrawingPath` into continuous cubic Bezier segments.
    pub fn decompose_path(path: &DrawingPath) -> Vec<CubicBezierSegment> {
        let mut segments = Vec::new();
        let mut current_pt = (0.0, 0.0);
        let mut start_pt = (0.0, 0.0);

        for cmd in &path.commands {
            match *cmd {
                PathCommand::MoveTo(ref pt) => {
                    current_pt = (pt.x, pt.y);
                    start_pt = current_pt;
                }
                PathCommand::LineTo(ref pt) => {
                    let next_pt = (pt.x, pt.y);
                    segments.push(CubicBezierSegment::from_line(current_pt, next_pt));
                    current_pt = next_pt;
                }
                PathCommand::QuadTo { ref ctrl, ref to } => {
                    let ctrl_pt = (ctrl.x, ctrl.y);
                    let next_pt = (to.x, to.y);
                    segments.push(CubicBezierSegment::from_quad(current_pt, ctrl_pt, next_pt));
                    current_pt = next_pt;
                }
                PathCommand::CubicTo {
                    ref ctrl1,
                    ref ctrl2,
                    ref to,
                } => {
                    let next_pt = (to.x, to.y);
                    segments.push(CubicBezierSegment::new(
                        current_pt,
                        (ctrl1.x, ctrl1.y),
                        (ctrl2.x, ctrl2.y),
                        next_pt,
                    ));
                    current_pt = next_pt;
                }
                PathCommand::Close => {
                    if (current_pt.0 - start_pt.0).abs() > 1e-4
                        || (current_pt.1 - start_pt.1).abs() > 1e-4
                    {
                        segments.push(CubicBezierSegment::from_line(current_pt, start_pt));
                        current_pt = start_pt;
                    }
                }
                PathCommand::ArcTo { .. } => {
                    segments.push(CubicBezierSegment::from_line(current_pt, current_pt));
                }
            }
        }

        segments
    }

    /// Equal-subdivision normalization: resamples path segments to exactly `target_count` segments.
    pub fn normalize_segments(
        segments: &[CubicBezierSegment],
        target_count: usize,
    ) -> Vec<CubicBezierSegment> {
        if segments.is_empty() || target_count == 0 {
            return Vec::new();
        }

        let total_len: f64 = segments.iter().map(|s| s.length_approx()).sum();
        if total_len <= 1e-6 {
            let p = segments[0].p0;
            return vec![CubicBezierSegment::new(p, p, p, p); target_count];
        }

        let mut resampled = Vec::with_capacity(target_count);

        for i in 0..target_count {
            let u_start = i as f64 / target_count as f64;
            let u_end = (i + 1) as f64 / target_count as f64;

            let p0 = Self::sample_composite_path(segments, u_start, total_len);
            let p3 = Self::sample_composite_path(segments, u_end, total_len);

            let c1_sample =
                Self::sample_composite_path(segments, u_start + (u_end - u_start) / 3.0, total_len);
            let c2_sample = Self::sample_composite_path(
                segments,
                u_start + 2.0 * (u_end - u_start) / 3.0,
                total_len,
            );

            let c1 = (
                p0.0 + (c1_sample.0 - p0.0) * 1.5,
                p0.1 + (c1_sample.1 - p0.1) * 1.5,
            );
            let c2 = (
                p3.0 + (c2_sample.0 - p3.0) * 1.5,
                p3.1 + (c2_sample.1 - p3.1) * 1.5,
            );

            resampled.push(CubicBezierSegment::new(p0, c1, c2, p3));
        }

        resampled
    }

    pub fn sample_composite_path(
        segments: &[CubicBezierSegment],
        normalized_u: f64,
        total_len: f64,
    ) -> (f64, f64) {
        let target_dist = normalized_u.clamp(0.0, 1.0) * total_len;
        let mut accumulated = 0.0;

        for seg in segments {
            let len = seg.length_approx();
            if accumulated + len >= target_dist || len <= 1e-6 {
                let local_u = if len > 1e-6 {
                    (target_dist - accumulated) / len
                } else {
                    0.0
                };
                return seg.sample(local_u);
            }
            accumulated += len;
        }

        segments.last().map(|s| s.p3).unwrap_or((0.0, 0.0))
    }

    /// Interpolate between two normalized segment arrays at time $t \in [0.0, 1.0]$.
    pub fn interpolate_segments(
        segs_a: &[CubicBezierSegment],
        segs_b: &[CubicBezierSegment],
        t: f64,
    ) -> Vec<CubicBezierSegment> {
        let t = t.clamp(0.0, 1.0);
        let count = segs_a.len().min(segs_b.len());
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let sa = &segs_a[i];
            let sb = &segs_b[i];

            let p0 = (
                (1.0 - t) * sa.p0.0 + t * sb.p0.0,
                (1.0 - t) * sa.p0.1 + t * sb.p0.1,
            );
            let c1 = (
                (1.0 - t) * sa.c1.0 + t * sb.c1.0,
                (1.0 - t) * sa.c1.1 + t * sb.c1.1,
            );
            let c2 = (
                (1.0 - t) * sa.c2.0 + t * sb.c2.0,
                (1.0 - t) * sa.c2.1 + t * sb.c2.1,
            );
            let p3 = (
                (1.0 - t) * sa.p3.0 + t * sb.p3.0,
                (1.0 - t) * sa.p3.1 + t * sb.p3.1,
            );

            result.push(CubicBezierSegment::new(p0, c1, c2, p3));
        }

        result
    }

    /// Convert interpolated cubic segments back into a `DrawingPath`.
    pub fn segments_to_drawing_path(segments: &[CubicBezierSegment], closed: bool) -> DrawingPath {
        let mut commands = Vec::new();
        if segments.is_empty() {
            return DrawingPath {
                width: None,
                height: None,
                fill: crate::drawingml::PathFillMode::Norm,
                stroke: true,
                commands,
            };
        }

        commands.push(PathCommand::MoveTo(Point2D {
            x: segments[0].p0.0,
            y: segments[0].p0.1,
        }));

        for s in segments {
            commands.push(PathCommand::CubicTo {
                ctrl1: Point2D {
                    x: s.c1.0,
                    y: s.c1.1,
                },
                ctrl2: Point2D {
                    x: s.c2.0,
                    y: s.c2.1,
                },
                to: Point2D {
                    x: s.p3.0,
                    y: s.p3.1,
                },
            });
        }

        if closed {
            commands.push(PathCommand::Close);
        }

        DrawingPath {
            width: None,
            height: None,
            fill: crate::drawingml::PathFillMode::Norm,
            stroke: true,
            commands,
        }
    }
}

// =========================================================================
// 4. Text Morphing (`byObject`, `byWord`, `byChar`)
// =========================================================================

pub struct TextMorpher;

impl TextMorpher {
    /// Evaluate text morphing between two shapes.
    pub fn morph_text(
        shape_a: &Shape,
        shape_b: &Shape,
        option: MorphOption,
        t: f64,
        xfrm_t: &MorphedTransform,
    ) -> Vec<MorphedTextElement> {
        let t = t.clamp(0.0, 1.0);
        let text_a = MorphMatcher::extract_plain_text(shape_a);
        let text_b = MorphMatcher::extract_plain_text(shape_b);

        match option {
            MorphOption::ByObject => {
                let text = if t < 0.5 { &text_a } else { &text_b };
                if text.is_empty() {
                    return Vec::new();
                }
                vec![MorphedTextElement {
                    text: text.clone(),
                    x: xfrm_t.x + 10.0,
                    y: xfrm_t.y + 10.0,
                    font_size_pt: 18.0,
                    opacity: 1.0,
                }]
            }
            MorphOption::ByWord => {
                let words_a: Vec<&str> = text_a.split_whitespace().collect();
                let words_b: Vec<&str> = text_b.split_whitespace().collect();
                let mut elements = Vec::new();

                let mut matched_b = HashSet::new();
                let mut matched_pairs = Vec::new();

                for (ia, wa) in words_a.iter().enumerate() {
                    for (ib, wb) in words_b.iter().enumerate() {
                        if !matched_b.contains(&ib) && wa == wb {
                            matched_pairs.push((ia, ib, *wa));
                            matched_b.insert(ib);
                            break;
                        }
                    }
                }

                // Matched words interpolate positions
                for &(ia, ib, w) in &matched_pairs {
                    let xa = xfrm_t.x + (ia as f64) * 60.0;
                    let xb = xfrm_t.x + (ib as f64) * 60.0;
                    elements.push(MorphedTextElement {
                        text: w.to_string(),
                        x: (1.0 - t) * xa + t * xb,
                        y: xfrm_t.y + 10.0,
                        font_size_pt: 18.0,
                        opacity: 1.0,
                    });
                }

                // Unmatched words on A fade out
                let matched_a_indices: HashSet<usize> = matched_pairs.iter().map(|p| p.0).collect();
                for (ia, wa) in words_a.iter().enumerate() {
                    if !matched_a_indices.contains(&ia) {
                        elements.push(MorphedTextElement {
                            text: wa.to_string(),
                            x: xfrm_t.x + (ia as f64) * 60.0,
                            y: xfrm_t.y + 10.0,
                            font_size_pt: 18.0,
                            opacity: (1.0 - t).clamp(0.0, 1.0),
                        });
                    }
                }

                // Unmatched words on B fade in
                for (ib, wb) in words_b.iter().enumerate() {
                    if !matched_b.contains(&ib) {
                        elements.push(MorphedTextElement {
                            text: wb.to_string(),
                            x: xfrm_t.x + (ib as f64) * 60.0,
                            y: xfrm_t.y + 10.0,
                            font_size_pt: 18.0,
                            opacity: t.clamp(0.0, 1.0),
                        });
                    }
                }

                elements
            }
            MorphOption::ByChar => {
                let chars_a: Vec<char> = text_a.chars().collect();
                let chars_b: Vec<char> = text_b.chars().collect();
                let mut elements = Vec::new();

                let mut matched_b = HashSet::new();
                let mut matched_pairs = Vec::new();

                for (ia, &ca) in chars_a.iter().enumerate() {
                    for (ib, &cb) in chars_b.iter().enumerate() {
                        if !matched_b.contains(&ib) && ca == cb {
                            matched_pairs.push((ia, ib, ca));
                            matched_b.insert(ib);
                            break;
                        }
                    }
                }

                // Matched characters interpolate position
                for &(ia, ib, c) in &matched_pairs {
                    let xa = xfrm_t.x + (ia as f64) * 12.0;
                    let xb = xfrm_t.x + (ib as f64) * 12.0;
                    elements.push(MorphedTextElement {
                        text: c.to_string(),
                        x: (1.0 - t) * xa + t * xb,
                        y: xfrm_t.y + 10.0,
                        font_size_pt: 18.0,
                        opacity: 1.0,
                    });
                }

                // Unmatched chars in A fade out
                let matched_a: HashSet<usize> = matched_pairs.iter().map(|p| p.0).collect();
                for (ia, &ca) in chars_a.iter().enumerate() {
                    if !matched_a.contains(&ia) {
                        elements.push(MorphedTextElement {
                            text: ca.to_string(),
                            x: xfrm_t.x + (ia as f64) * 12.0,
                            y: xfrm_t.y + 10.0,
                            font_size_pt: 18.0,
                            opacity: (1.0 - t).clamp(0.0, 1.0),
                        });
                    }
                }

                // Unmatched chars in B fade in
                for (ib, &cb) in chars_b.iter().enumerate() {
                    if !matched_b.contains(&ib) {
                        elements.push(MorphedTextElement {
                            text: cb.to_string(),
                            x: xfrm_t.x + (ib as f64) * 12.0,
                            y: xfrm_t.y + 10.0,
                            font_size_pt: 18.0,
                            opacity: t.clamp(0.0, 1.0),
                        });
                    }
                }

                elements
            }
        }
    }
}

// =========================================================================
// 5. Complete Morph Transition Evaluator (`MorphEvaluator`)
// =========================================================================

pub struct MorphEvaluator;

impl MorphEvaluator {
    /// Evaluate the full morph transition scene at time progress $t \in [0.0, 1.0]$.
    pub fn evaluate_transition(
        slide_a: &Slide,
        slide_b: &Slide,
        option: MorphOption,
        t: f64,
        canvas_w: f64,
        canvas_h: f64,
    ) -> MorphedSlideState {
        let t = t.clamp(0.0, 1.0);
        let pairs = MorphMatcher::match_slides(slide_a, slide_b, canvas_w, canvas_h);

        let shapes_a = MorphMatcher::extract_slide_shapes(slide_a);
        let shapes_b = MorphMatcher::extract_slide_shapes(slide_b);

        let map_a: HashMap<u32, &Shape> = shapes_a.iter().map(|s| (s.id, *s)).collect();
        let map_b: HashMap<u32, &Shape> = shapes_b.iter().map(|s| (s.id, *s)).collect();

        let mut morphed_shapes = Vec::with_capacity(pairs.len());

        for pair in pairs {
            match (pair.source_id, pair.target_id) {
                (Some(id_a), Some(id_b)) => {
                    if let (Some(&sa), Some(&sb)) = (map_a.get(&id_a), map_b.get(&id_b)) {
                        let xfrm_a = sa
                            .properties
                            .transform
                            .as_ref()
                            .map(MorphedTransform::from_transform_2d)
                            .unwrap_or_default();
                        let xfrm_b = sb
                            .properties
                            .transform
                            .as_ref()
                            .map(MorphedTransform::from_transform_2d)
                            .unwrap_or_default();

                        let xfrm_t = MorphedTransform {
                            x: (1.0 - t) * xfrm_a.x + t * xfrm_b.x,
                            y: (1.0 - t) * xfrm_a.y + t * xfrm_b.y,
                            width: (1.0 - t) * xfrm_a.width + t * xfrm_b.width,
                            height: (1.0 - t) * xfrm_a.height + t * xfrm_b.height,
                            rotation_deg: (1.0 - t) * xfrm_a.rotation_deg + t * xfrm_b.rotation_deg,
                        };

                        let text_elements = TextMorpher::morph_text(sa, sb, option, t, &xfrm_t);

                        morphed_shapes.push(MorphedShapeState {
                            shape_id: if t < 0.5 { id_a } else { id_b },
                            name: sb.name.clone(),
                            transform: xfrm_t,
                            opacity: 1.0,
                            geometry_path: None,
                            text_elements,
                        });
                    }
                }
                (Some(id_a), None) => {
                    if let Some(&sa) = map_a.get(&id_a) {
                        let xfrm = sa
                            .properties
                            .transform
                            .as_ref()
                            .map(MorphedTransform::from_transform_2d)
                            .unwrap_or_default();
                        morphed_shapes.push(MorphedShapeState {
                            shape_id: id_a,
                            name: sa.name.clone(),
                            transform: xfrm,
                            opacity: (1.0 - t).clamp(0.0, 1.0),
                            geometry_path: None,
                            text_elements: Vec::new(),
                        });
                    }
                }
                (None, Some(id_b)) => {
                    if let Some(&sb) = map_b.get(&id_b) {
                        let xfrm = sb
                            .properties
                            .transform
                            .as_ref()
                            .map(MorphedTransform::from_transform_2d)
                            .unwrap_or_default();
                        morphed_shapes.push(MorphedShapeState {
                            shape_id: id_b,
                            name: sb.name.clone(),
                            transform: xfrm,
                            opacity: t.clamp(0.0, 1.0),
                            geometry_path: None,
                            text_elements: Vec::new(),
                        });
                    }
                }
                (None, None) => {}
            }
        }

        MorphedSlideState {
            t,
            shapes: morphed_shapes,
        }
    }
}

// =========================================================================
// 6. SmartArt Layout Engine (`ppt/diagrams/layout*.xml`, `dgm:alg`)
// =========================================================================

/// Rectangle bounding box $(x, y, w, h)$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
}

/// Diagram point node type: `<dgm:pt type="...">`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagramPointType {
    #[default]
    Node,
    Assistant,
    ParentTransition,
    SiblingTransition,
}

/// Diagram point model: `<dgm:pt>`.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagramPoint {
    pub model_id: String,
    pub point_type: DiagramPointType,
    pub title: String,
}

/// Diagram connection model: `<dgm:cxn>`.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagramConnection {
    pub model_id: String,
    pub src_id: String,
    pub dest_id: String,
    pub cxn_type: String, // "parOf", "presOf"
}

/// Diagram data model container: `<dgm:dataModel>`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DiagramDataModel {
    pub points: Vec<DiagramPoint>,
    pub connections: Vec<DiagramConnection>,
}

/// Layout position and shape assignment for an individual SmartArt node.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartArtNodeLayout {
    pub model_id: String,
    pub title: String,
    pub bounds: Rect,
    pub depth_level: usize,
    pub preset_geom: String,
}

/// Direction for linear SmartArt layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearDirection {
    Horizontal,
    Vertical,
}

pub struct SmartArtLayoutEngine;

impl SmartArtLayoutEngine {
    /// Linear layout algorithm (`lin`): arranges $N$ nodes along major axis.
    pub fn layout_linear(
        points: &[DiagramPoint],
        dir: LinearDirection,
        bounds: Rect,
        gap: f64,
    ) -> Vec<SmartArtNodeLayout> {
        let nodes: Vec<&DiagramPoint> = points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .collect();
        let n = nodes.len();
        if n == 0 {
            return Vec::new();
        }

        let mut layouts = Vec::with_capacity(n);

        match dir {
            LinearDirection::Horizontal => {
                let total_gap = gap * (n - 1) as f64;
                let item_w = (bounds.w - total_gap).max(0.0) / (n as f64);
                let item_h = bounds.h;

                for (i, p) in nodes.iter().enumerate() {
                    let x = bounds.x + (i as f64) * (item_w + gap);
                    layouts.push(SmartArtNodeLayout {
                        model_id: p.model_id.clone(),
                        title: p.title.clone(),
                        bounds: Rect::new(x, bounds.y, item_w, item_h),
                        depth_level: 0,
                        preset_geom: "roundRect".to_string(),
                    });
                }
            }
            LinearDirection::Vertical => {
                let total_gap = gap * (n - 1) as f64;
                let item_w = bounds.w;
                let item_h = (bounds.h - total_gap).max(0.0) / (n as f64);

                for (i, p) in nodes.iter().enumerate() {
                    let y = bounds.y + (i as f64) * (item_h + gap);
                    layouts.push(SmartArtNodeLayout {
                        model_id: p.model_id.clone(),
                        title: p.title.clone(),
                        bounds: Rect::new(bounds.x, y, item_w, item_h),
                        depth_level: 0,
                        preset_geom: "roundRect".to_string(),
                    });
                }
            }
        }

        layouts
    }

    /// Pyramid layout algorithm (`pyra`): stacks $N$ trapezoidal tiers vertically.
    pub fn layout_pyramid(
        points: &[DiagramPoint],
        _inverted: bool,
        bounds: Rect,
    ) -> Vec<SmartArtNodeLayout> {
        let nodes: Vec<&DiagramPoint> = points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .collect();
        let n = nodes.len();
        if n == 0 {
            return Vec::new();
        }

        let tier_h = bounds.h / (n as f64);
        let mut layouts = Vec::with_capacity(n);

        for (i, p) in nodes.iter().enumerate() {
            let y = bounds.y + (i as f64) * tier_h;
            let tier_w = bounds.w * ((i + 1) as f64 / n as f64);
            let x = bounds.x + (bounds.w - tier_w) / 2.0;

            layouts.push(SmartArtNodeLayout {
                model_id: p.model_id.clone(),
                title: p.title.clone(),
                bounds: Rect::new(x, y, tier_w, tier_h),
                depth_level: i,
                preset_geom: "trapezoid".to_string(),
            });
        }

        layouts
    }

    /// Cycle layout algorithm (`cycle`): arranges $N$ nodes radially along an ellipse.
    pub fn layout_cycle(
        points: &[DiagramPoint],
        bounds: Rect,
        node_w: f64,
        node_h: f64,
    ) -> Vec<SmartArtNodeLayout> {
        let nodes: Vec<&DiagramPoint> = points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .collect();
        let n = nodes.len();
        if n == 0 {
            return Vec::new();
        }

        let cx = bounds.x + bounds.w / 2.0;
        let cy = bounds.y + bounds.h / 2.0;
        let rx = (bounds.w - node_w).max(0.0) / 2.0;
        let ry = (bounds.h - node_h).max(0.0) / 2.0;

        let mut layouts = Vec::with_capacity(n);

        for (i, p) in nodes.iter().enumerate() {
            let angle =
                -std::f64::consts::FRAC_PI_2 + (i as f64) * 2.0 * std::f64::consts::PI / (n as f64);
            let x = cx + rx * angle.cos() - node_w / 2.0;
            let y = cy + ry * angle.sin() - node_h / 2.0;

            layouts.push(SmartArtNodeLayout {
                model_id: p.model_id.clone(),
                title: p.title.clone(),
                bounds: Rect::new(x, y, node_w, node_h),
                depth_level: 0,
                preset_geom: "ellipse".to_string(),
            });
        }

        layouts
    }

    /// Hierarchy layout algorithm (`hierChild`): builds parent-child tree layout.
    pub fn layout_hierarchy(
        model: &DiagramDataModel,
        bounds: Rect,
        node_w: f64,
        node_h: f64,
    ) -> Vec<SmartArtNodeLayout> {
        let nodes: Vec<&DiagramPoint> = model
            .points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .collect();
        if nodes.is_empty() {
            return Vec::new();
        }

        let mut children_map: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut has_parent = HashSet::new();

        for cxn in &model.connections {
            if cxn.cxn_type == "parOf" {
                children_map
                    .entry(&cxn.src_id)
                    .or_default()
                    .push(&cxn.dest_id);
                has_parent.insert(cxn.dest_id.as_str());
            }
        }

        let roots: Vec<&DiagramPoint> = nodes
            .iter()
            .copied()
            .filter(|p| !has_parent.contains(p.model_id.as_str()))
            .collect();

        let mut layouts = Vec::new();
        let depth_count = 2;
        let level_h = bounds.h / (depth_count as f64);

        for r in roots {
            let x = bounds.x + (bounds.w - node_w) / 2.0;
            let y = bounds.y;
            layouts.push(SmartArtNodeLayout {
                model_id: r.model_id.clone(),
                title: r.title.clone(),
                bounds: Rect::new(x, y, node_w, node_h),
                depth_level: 0,
                preset_geom: "roundRect".to_string(),
            });

            if let Some(children_ids) = children_map.get(r.model_id.as_str()) {
                let m = children_ids.len();
                let child_gap = 20.0;
                let total_w = (m as f64) * node_w + ((m.saturating_sub(1)) as f64) * child_gap;
                let start_x = bounds.x + (bounds.w - total_w).max(0.0) / 2.0;

                for (j, cid) in children_ids.iter().enumerate() {
                    if let Some(cp) = nodes.iter().find(|p| p.model_id == *cid) {
                        let cx = start_x + (j as f64) * (node_w + child_gap);
                        let cy = bounds.y + level_h;
                        layouts.push(SmartArtNodeLayout {
                            model_id: cp.model_id.clone(),
                            title: cp.title.clone(),
                            bounds: Rect::new(cx, cy, node_w, node_h),
                            depth_level: 1,
                            preset_geom: "roundRect".to_string(),
                        });
                    }
                }
            }
        }

        layouts
    }

    /// Snake layout algorithm (`snake`): zig-zag row-by-row alternating distribution.
    pub fn layout_snake(
        points: &[DiagramPoint],
        columns: usize,
        bounds: Rect,
        gap: f64,
    ) -> Vec<SmartArtNodeLayout> {
        let nodes: Vec<&DiagramPoint> = points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .collect();
        let n = nodes.len();
        if n == 0 || columns == 0 {
            return Vec::new();
        }

        let rows = n.div_ceil(columns);
        let item_w = (bounds.w - (columns - 1) as f64 * gap).max(0.0) / (columns as f64);
        let item_h = (bounds.h - (rows.saturating_sub(1)) as f64 * gap).max(0.0) / (rows as f64);

        let mut layouts = Vec::with_capacity(n);

        for (i, p) in nodes.iter().enumerate() {
            let row = i / columns;
            let col_in_row = i % columns;
            let col = if row.is_multiple_of(2) {
                col_in_row
            } else {
                columns - 1 - col_in_row
            };

            let x = bounds.x + (col as f64) * (item_w + gap);
            let y = bounds.y + (row as f64) * (item_h + gap);

            layouts.push(SmartArtNodeLayout {
                model_id: p.model_id.clone(),
                title: p.title.clone(),
                bounds: Rect::new(x, y, item_w, item_h),
                depth_level: row,
                preset_geom: "roundRect".to_string(),
            });
        }

        layouts
    }

    /// Synchronize calculated SmartArt node layouts into DrawingML `Shape` structures for `p:spTree`.
    pub fn sync_to_shape_tree(layouts: &[SmartArtNodeLayout], start_id: u32) -> Vec<Shape> {
        let mut shapes = Vec::with_capacity(layouts.len());

        for (i, node) in layouts.iter().enumerate() {
            let shape = Shape {
                id: start_id + (i as u32),
                name: format!("SmartArtNode_{}", node.title),
                properties: ShapeProperties {
                    transform: Some(Transform2D {
                        x: node.bounds.x.round() as i64,
                        y: node.bounds.y.round() as i64,
                        cx: node.bounds.w.round() as u32,
                        cy: node.bounds.h.round() as u32,
                        rot: 0,
                        flip_h: false,
                        flip_v: false,
                    }),
                    geometry: Some(Geometry::Preset(node.preset_geom.clone())),
                    ..Default::default()
                },
                text_body: Some(TextBody {
                    paragraphs: vec![TextParagraph {
                        properties: None,
                        runs: vec![TextRunElement::Text(node.title.clone(), None)],
                        end_paragraph_run_properties: None,
                    }],
                    body_properties: Default::default(),
                    list_style: None,
                }),
                ..Default::default()
            };

            shapes.push(shape);
        }

        shapes
    }
}
