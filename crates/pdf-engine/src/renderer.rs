#![forbid(unsafe_code)]
//! ISO 32000-2 Vector Path, Image XObject & Font CMap Canvas Renderer
//!
//! Complies with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in production library code.
//! - ISO 32000-2 Clause 8 (Graphics) & Clause 9 (Text).
//! - Frame-accurate vector paths, image XObject extraction, and font CMap text rendering.

use serde::{Deserialize, Serialize};

use crate::catalog::PdfDocument;
use crate::content::{ContentOperator, ContentParser};
use crate::error::PdfError;
use crate::page::PdfPage;
use crate::text::ExtractedPageText;
use crate::types::PdfObject;

/// A rendered 2D vector path with SVG path data and stroke/fill styling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderedVectorPathDto {
    pub d: String,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: f64,
    pub fill_rule: String,
    pub opacity: f64,
    pub line_cap: String,
    pub line_join: String,
    pub dash_array: Option<String>,
}

/// A rendered text run element with glyph typography, color, and bounding box.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderedTextElementDto {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_name: String,
    pub font_size: f64,
    pub color_hex: String,
    pub transform: Option<[f64; 6]>,
}

/// A rendered image XObject element placed on the page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderedImageElementDto {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub mime_type: String,
    pub data_base64: String,
    pub raw_bytes_len: usize,
}

/// Complete canvas description for a rendered PDF page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderedPageCanvasDto {
    pub page_index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub scale: f64,
    pub vector_paths: Vec<RenderedVectorPathDto>,
    pub text_elements: Vec<RenderedTextElementDto>,
    pub image_elements: Vec<RenderedImageElementDto>,
    pub svg_markup: String,
}

/// 2D affine transformation matrix `[a, b, c, d, e, f]` representing:
/// `x' = a*x + c*y + e`
/// `y' = b*x + d*y + f`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineMatrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for AffineMatrix {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

impl AffineMatrix {
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn from_array(arr: [f64; 6]) -> Self {
        Self {
            a: arr[0],
            b: arr[1],
            c: arr[2],
            d: arr[3],
            e: arr[4],
            f: arr[5],
        }
    }

    pub fn multiply(&self, rhs: &Self) -> Self {
        Self {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            e: self.a * rhs.e + self.c * rhs.f + self.e,
            f: self.b * rhs.e + self.d * rhs.f + self.f,
        }
    }

    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

#[derive(Clone)]
struct GraphicsState {
    ctm: AffineMatrix,
    stroke_color: [f32; 3], // RGB in 0.0..=1.0
    fill_color: [f32; 3],   // RGB in 0.0..=1.0
    line_width: f64,
    line_cap: String,
    line_join: String,
    miter_limit: f64,
    dash_array: Option<String>,
    opacity: f64,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: AffineMatrix::default(),
            stroke_color: [0.0, 0.0, 0.0],
            fill_color: [0.0, 0.0, 0.0],
            line_width: 1.0,
            line_cap: "butt".to_string(),
            line_join: "miter".to_string(),
            miter_limit: 10.0,
            dash_array: None,
            opacity: 1.0,
        }
    }
}

enum SubPathSegment {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    Close,
}

pub struct PdfPageRenderer;

impl PdfPageRenderer {
    /// Renders a given PDF page into a structured canvas description with vector paths, text, images, and SVG.
    pub fn render_page_canvas(
        doc: &PdfDocument,
        page_index: usize,
        scale: Option<f64>,
    ) -> Result<RenderedPageCanvasDto, PdfError> {
        let page = doc.page(page_index)?;
        let scale_factor = scale.unwrap_or(1.0).max(0.01);
        let w_pt = page.width_pts();
        let h_pt = page.height_pts();

        let mut vector_paths = Vec::new();
        let mut image_elements = Vec::new();

        // Decode all content streams for this page
        let mut combined_content = Vec::new();
        for &cid in &page.contents {
            if let Ok(PdfObject::Stream(stm)) = doc.resolve_object(cid) {
                if let Ok(decomp) = stm.decode() {
                    combined_content.extend_from_slice(&decomp);
                    combined_content.push(b'\n');
                }
            }
        }

        let operators = ContentParser::new(&combined_content).parse()?;
        let mut gs_stack = vec![GraphicsState::default()];
        let mut current_subpath = Vec::new();

        for op in operators {
            let Some(gs) = gs_stack.last_mut() else {
                continue;
            };

            match op {
                ContentOperator::SaveGraphicsState => {
                    let cur = gs.clone();
                    gs_stack.push(cur);
                }
                ContentOperator::RestoreGraphicsState => {
                    if gs_stack.len() > 1 {
                        gs_stack.pop();
                    }
                }
                ContentOperator::ConcatMatrix(m) => {
                    let m_aff = AffineMatrix::from_array(m);
                    gs.ctm = gs.ctm.multiply(&m_aff);
                }
                ContentOperator::SetLineWidth(w) => {
                    gs.line_width = w.max(0.1);
                }
                ContentOperator::SetLineCap(cap) => {
                    gs.line_cap = match cap {
                        1 => "round".to_string(),
                        2 => "square".to_string(),
                        _ => "butt".to_string(),
                    };
                }
                ContentOperator::SetLineJoin(join) => {
                    gs.line_join = match join {
                        1 => "round".to_string(),
                        2 => "bevel".to_string(),
                        _ => "miter".to_string(),
                    };
                }
                ContentOperator::SetMiterLimit(ml) => {
                    gs.miter_limit = ml;
                }
                ContentOperator::SetDash { pattern, .. } => {
                    if pattern.is_empty() {
                        gs.dash_array = None;
                    } else {
                        let parts: Vec<String> =
                            pattern.iter().map(|p| format!("{:.2}", p)).collect();
                        gs.dash_array = Some(parts.join(","));
                    }
                }
                ContentOperator::SetGrayStroke(g) => {
                    let clamped = g.clamp(0.0, 1.0);
                    gs.stroke_color = [clamped, clamped, clamped];
                }
                ContentOperator::SetGrayFill(g) => {
                    let clamped = g.clamp(0.0, 1.0);
                    gs.fill_color = [clamped, clamped, clamped];
                }
                ContentOperator::SetRgbStroke { r, g, b } => {
                    gs.stroke_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
                }
                ContentOperator::SetRgbFill { r, g, b } => {
                    gs.fill_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
                }
                ContentOperator::SetCmykStroke { c, m, y, k } => {
                    gs.stroke_color = Self::cmyk_to_rgb(c, m, y, k);
                }
                ContentOperator::SetCmykFill { c, m, y, k } => {
                    gs.fill_color = Self::cmyk_to_rgb(c, m, y, k);
                }

                // Path Construction Operators
                ContentOperator::MoveTo { x, y } => {
                    let (tx, ty) = gs.ctm.transform_point(x, y);
                    current_subpath.push(SubPathSegment::MoveTo(tx, ty));
                }
                ContentOperator::LineTo { x, y } => {
                    let (tx, ty) = gs.ctm.transform_point(x, y);
                    current_subpath.push(SubPathSegment::LineTo(tx, ty));
                }
                ContentOperator::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    let (tx1, ty1) = gs.ctm.transform_point(x1, y1);
                    let (tx2, ty2) = gs.ctm.transform_point(x2, y2);
                    let (tx3, ty3) = gs.ctm.transform_point(x3, y3);
                    current_subpath.push(SubPathSegment::CurveTo(tx1, ty1, tx2, ty2, tx3, ty3));
                }
                ContentOperator::Rectangle { x, y, w, h } => {
                    let (p0_x, p0_y) = gs.ctm.transform_point(x, y);
                    let (p1_x, p1_y) = gs.ctm.transform_point(x + w, y);
                    let (p2_x, p2_y) = gs.ctm.transform_point(x + w, y + h);
                    let (p3_x, p3_y) = gs.ctm.transform_point(x, y + h);
                    current_subpath.push(SubPathSegment::MoveTo(p0_x, p0_y));
                    current_subpath.push(SubPathSegment::LineTo(p1_x, p1_y));
                    current_subpath.push(SubPathSegment::LineTo(p2_x, p2_y));
                    current_subpath.push(SubPathSegment::LineTo(p3_x, p3_y));
                    current_subpath.push(SubPathSegment::Close);
                }
                ContentOperator::ClosePath => {
                    current_subpath.push(SubPathSegment::Close);
                }

                // Path Painting Operators
                ContentOperator::Stroke => {
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        None,
                        Some(Self::rgb_to_hex(gs.stroke_color)),
                        gs,
                        "nonzero",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::CloseAndStroke => {
                    current_subpath.push(SubPathSegment::Close);
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        None,
                        Some(Self::rgb_to_hex(gs.stroke_color)),
                        gs,
                        "nonzero",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::Fill => {
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        Some(Self::rgb_to_hex(gs.fill_color)),
                        None,
                        gs,
                        "nonzero",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::FillEvenOdd => {
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        Some(Self::rgb_to_hex(gs.fill_color)),
                        None,
                        gs,
                        "evenodd",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::FillAndStroke => {
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        Some(Self::rgb_to_hex(gs.fill_color)),
                        Some(Self::rgb_to_hex(gs.stroke_color)),
                        gs,
                        "nonzero",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::FillAndStrokeEvenOdd => {
                    if let Some(dto) = Self::build_path_dto(
                        &current_subpath,
                        page,
                        Some(Self::rgb_to_hex(gs.fill_color)),
                        Some(Self::rgb_to_hex(gs.stroke_color)),
                        gs,
                        "evenodd",
                    ) {
                        vector_paths.push(dto);
                    }
                    current_subpath.clear();
                }
                ContentOperator::EndPathNoOp => {
                    current_subpath.clear();
                }

                // XObjects
                ContentOperator::Do(ref name) => {
                    if let Some(img_dto) = Self::extract_image_xobject(name, gs, page, doc) {
                        image_elements.push(img_dto);
                    }
                }
                _ => {}
            }
        }

        // Extract styled text elements with Font CMap resolution
        let text_elements = Self::extract_text_elements(page, doc)?;

        // Build standalone SVG markup
        let svg_markup =
            Self::generate_svg_markup(w_pt, h_pt, &vector_paths, &text_elements, &image_elements);

        Ok(RenderedPageCanvasDto {
            page_index,
            width_pt: w_pt,
            height_pt: h_pt,
            scale: scale_factor,
            vector_paths,
            text_elements,
            image_elements,
            svg_markup,
        })
    }

    /// Renders the page directly to a standalone, scalable vector SVG string.
    pub fn render_page_svg(doc: &PdfDocument, page_index: usize) -> Result<String, PdfError> {
        let canvas = Self::render_page_canvas(doc, page_index, Some(1.0))?;
        Ok(canvas.svg_markup)
    }

    /// Extracts all Image XObjects placed on the specified page.
    pub fn extract_page_images(
        doc: &PdfDocument,
        page_index: usize,
    ) -> Result<Vec<RenderedImageElementDto>, PdfError> {
        let canvas = Self::render_page_canvas(doc, page_index, Some(1.0))?;
        Ok(canvas.image_elements)
    }

    fn build_path_dto(
        segments: &[SubPathSegment],
        page: &PdfPage,
        fill: Option<String>,
        stroke: Option<String>,
        gs: &GraphicsState,
        fill_rule: &str,
    ) -> Option<RenderedVectorPathDto> {
        if segments.is_empty() {
            return None;
        }

        let h_pt = page.height_pts();
        let llx = page.crop_box.llx;
        let lly = page.crop_box.lly;

        let mut d = String::with_capacity(segments.len() * 24);
        for seg in segments {
            match *seg {
                SubPathSegment::MoveTo(x, y) => {
                    let svg_x = x - llx;
                    let svg_y = h_pt - (y - lly);
                    d.push_str(&format!("M {:.2} {:.2} ", svg_x, svg_y));
                }
                SubPathSegment::LineTo(x, y) => {
                    let svg_x = x - llx;
                    let svg_y = h_pt - (y - lly);
                    d.push_str(&format!("L {:.2} {:.2} ", svg_x, svg_y));
                }
                SubPathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    let svg_x1 = x1 - llx;
                    let svg_y1 = h_pt - (y1 - lly);
                    let svg_x2 = x2 - llx;
                    let svg_y2 = h_pt - (y2 - lly);
                    let svg_x3 = x3 - llx;
                    let svg_y3 = h_pt - (y3 - lly);
                    d.push_str(&format!(
                        "C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2} ",
                        svg_x1, svg_y1, svg_x2, svg_y2, svg_x3, svg_y3
                    ));
                }
                SubPathSegment::Close => {
                    d.push_str("Z ");
                }
            }
        }

        let d_trimmed = d.trim_end().to_string();
        if d_trimmed.is_empty() {
            return None;
        }

        Some(RenderedVectorPathDto {
            d: d_trimmed,
            fill_color: fill,
            stroke_color: stroke,
            stroke_width: gs.line_width,
            fill_rule: fill_rule.to_string(),
            opacity: gs.opacity,
            line_cap: gs.line_cap.clone(),
            line_join: gs.line_join.clone(),
            dash_array: gs.dash_array.clone(),
        })
    }

    fn extract_text_elements(
        page: &PdfPage,
        doc: &PdfDocument,
    ) -> Result<Vec<RenderedTextElementDto>, PdfError> {
        let extracted = ExtractedPageText::extract(page, doc)?;
        let h_pt = page.height_pts();
        let llx = page.crop_box.llx;
        let lly = page.crop_box.lly;

        let mut out = Vec::with_capacity(extracted.runs.len());
        for run in extracted.runs {
            if run.text.trim().is_empty() {
                continue;
            }

            let svg_x = run.bbox.llx - llx;
            // PDF origin is bottom-left, SVG is top-left
            let svg_y = h_pt - (run.bbox.ury - lly);
            let w = run.bbox.width();
            let h = run.bbox.height().max(run.font_size);

            let color_hex = Self::rgb_to_hex([run.color[0], run.color[1], run.color[2]]);

            out.push(RenderedTextElementDto {
                text: run.text,
                x: svg_x,
                y: svg_y,
                width: w,
                height: h,
                font_name: run.font_name,
                font_size: run.font_size,
                color_hex,
                transform: Some(run.matrix),
            });
        }

        Ok(out)
    }

    fn extract_image_xobject(
        name: &str,
        gs: &GraphicsState,
        page: &PdfPage,
        doc: &PdfDocument,
    ) -> Option<RenderedImageElementDto> {
        let xobj_dict = page.resources.get("XObject")?;
        let res_dict = match xobj_dict {
            PdfObject::Dictionary(d) => d,
            _ => return None,
        };

        let raw_name = name.strip_prefix('/').unwrap_or(name);
        let xobj_ref = res_dict.get(raw_name).or_else(|| res_dict.get(name))?;

        let stream = match xobj_ref {
            PdfObject::Stream(s) => s.clone(),
            PdfObject::Reference(ref_id) => {
                if let Ok(PdfObject::Stream(s)) = doc.resolve_object(*ref_id) {
                    s
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        let subtype = stream.dict.get("Subtype").and_then(|v| v.as_name())?;
        if subtype != "Image" {
            return None;
        }

        let width = stream
            .dict
            .get("Width")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as f64;
        let height = stream
            .dict
            .get("Height")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as f64;

        let filter = stream.filter().unwrap_or("");
        let (mime_type, payload) = if filter == "DCTDecode" {
            ("image/jpeg".to_string(), stream.content.clone())
        } else if let Ok(decomp) = stream.decode() {
            ("image/png".to_string(), decomp)
        } else {
            (
                "application/octet-stream".to_string(),
                stream.content.clone(),
            )
        };

        // In PDF, an image XObject maps to unit square [0,0,1,1] in user space
        let (x, y) = gs.ctm.transform_point(0.0, 0.0);
        let w = (gs.ctm.a * gs.ctm.a + gs.ctm.b * gs.ctm.b).sqrt();
        let h = (gs.ctm.c * gs.ctm.c + gs.ctm.d * gs.ctm.d).sqrt();

        let h_pt = page.height_pts();
        let llx = page.crop_box.llx;
        let lly = page.crop_box.lly;

        let svg_x = x - llx;
        let svg_y = h_pt - (y + h - lly);

        let data_b64 = Self::base64_encode(&payload);
        let raw_len = payload.len();

        Some(RenderedImageElementDto {
            name: name.to_string(),
            x: svg_x,
            y: svg_y,
            width: if w > 0.0 { w } else { width },
            height: if h > 0.0 { h } else { height },
            mime_type,
            data_base64: data_b64,
            raw_bytes_len: raw_len,
        })
    }

    fn generate_svg_markup(
        w_pt: f64,
        h_pt: f64,
        vector_paths: &[RenderedVectorPathDto],
        text_elements: &[RenderedTextElementDto],
        image_elements: &[RenderedImageElementDto],
    ) -> String {
        let mut svg = String::with_capacity(8192);
        svg.push_str(&format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}pt" height="{:.2}pt">
  <defs>
    <style>
      .pdf-text {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; }}
    </style>
  </defs>
  <rect width="100%" height="100%" fill="#FFFFFF"/>
"##,
            w_pt, h_pt, w_pt, h_pt
        ));

        // 1. Vector Paths
        for p in vector_paths {
            let fill_attr = match &p.fill_color {
                Some(f) => format!(r#"fill="{}" fill-rule="{}" "#, f, p.fill_rule),
                None => r#"fill="none" "#.to_string(),
            };
            let stroke_attr = match &p.stroke_color {
                Some(s) => format!(
                    r#"stroke="{}" stroke-width="{:.2}" stroke-linecap="{}" stroke-linejoin="{}" "#,
                    s, p.stroke_width, p.line_cap, p.line_join
                ),
                None => r#"stroke="none" "#.to_string(),
            };
            let dash_attr = match &p.dash_array {
                Some(d) => format!(r#"stroke-dasharray="{}" "#, d),
                None => String::new(),
            };
            let opacity_attr = if (p.opacity - 1.0).abs() > 0.001 {
                format!(r#"opacity="{:.2}" "#, p.opacity)
            } else {
                String::new()
            };

            svg.push_str(&format!(
                r##"  <path d="{}" {}{}{}{}/>
"##,
                p.d, fill_attr, stroke_attr, dash_attr, opacity_attr
            ));
        }

        // 2. Image XObjects
        for img in image_elements {
            if !img.data_base64.is_empty() {
                svg.push_str(&format!(
                    r##"  <image x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" href="data:{};base64,{}"/>
"##,
                    img.x, img.y, img.width, img.height, img.mime_type, img.data_base64
                ));
            }
        }

        // 3. Text Elements
        for t in text_elements {
            let escaped_text = quick_xml_escape(&t.text);
            svg.push_str(&format!(
                r##"  <text x="{:.2}" y="{:.2}" class="pdf-text" font-size="{:.2}pt" fill="{}">{}</text>
"##,
                t.x,
                t.y + t.font_size,
                t.font_size,
                t.color_hex,
                escaped_text
            ));
        }

        svg.push_str("</svg>");
        svg
    }

    fn cmyk_to_rgb(c: f32, m: f32, y: f32, k: f32) -> [f32; 3] {
        let r = (1.0 - c) * (1.0 - k);
        let g = (1.0 - m) * (1.0 - k);
        let b = (1.0 - y) * (1.0 - k);
        [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
    }

    fn rgb_to_hex(rgb: [f32; 3]) -> String {
        let r = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    fn base64_encode(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = if chunk.len() > 1 {
                chunk[1] as usize
            } else {
                0
            };
            let b2 = if chunk.len() > 2 {
                chunk[2] as usize
            } else {
                0
            };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(CHARS[(triple >> 18) & 0x3F] as char);
            out.push(CHARS[(triple >> 12) & 0x3F] as char);
            if chunk.len() > 1 {
                out.push(CHARS[(triple >> 6) & 0x3F] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(CHARS[triple & 0x3F] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}

fn quick_xml_escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}
