#![forbid(unsafe_code)]
//! High-DPI Vector Slide Deck Exporters (SVG & ISO 32000-1 PDF)
//!
//! Complies with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors.
//! - Standalone vector SVG generation for canvas rendering, browser fallback, and vector asset extraction.
//! - Clean-room multi-page ISO 32000-1 PDF serialization with vector paths, fills, and typography.

use crate::model::{ColorChoice, Fill, Geometry, ShapeElement, Slide, TextRunElement};
use crate::package::PresentationPackage;

// =========================================================================
// 1. Standalone Vector SVG Slide Exporter
// =========================================================================

pub struct SlideSvgExporter;

impl SlideSvgExporter {
    /// Renders a single presentation slide into standalone, scalable SVG markup.
    pub fn export_slide_svg(pkg: &PresentationPackage, slide_idx: usize) -> Result<String, String> {
        if slide_idx >= pkg.slides.len() {
            return Err(format!("Slide index {slide_idx} out of bounds"));
        }

        let slide = &pkg.slides[slide_idx].1;
        let (w_pt, h_pt) = Self::get_canvas_dimensions_pt(pkg);

        let mut svg = String::with_capacity(4096);
        svg.push_str(&format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}pt" height="{:.2}pt">
  <defs>
    <style>
      .slide-text {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; }}
    </style>
  </defs>
  <rect width="100%" height="100%" fill="#FFFFFF"/>
"##,
            w_pt, h_pt, w_pt, h_pt
        ));

        for elem in &slide.common_slide_data.shape_tree.shapes {
            Self::render_element_to_svg(&mut svg, elem);
        }

        svg.push_str("</svg>");
        Ok(svg)
    }

    fn get_canvas_dimensions_pt(pkg: &PresentationPackage) -> (f64, f64) {
        let cx = pkg.presentation.slide_size.cx;
        let cy = pkg.presentation.slide_size.cy;
        let w = if cx > 0 { cx as f64 / 12700.0 } else { 960.0 };
        let h = if cy > 0 { cy as f64 / 12700.0 } else { 540.0 };
        (w, h)
    }

    fn render_element_to_svg(svg: &mut String, elem: &ShapeElement) {
        match elem {
            ShapeElement::Shape(s) => {
                let (x, y, w, h) = if let Some(ref t) = s.properties.transform {
                    (
                        t.x as f64 / 12700.0,
                        t.y as f64 / 12700.0,
                        t.cx as f64 / 12700.0,
                        t.cy as f64 / 12700.0,
                    )
                } else {
                    (0.0, 0.0, 100.0, 100.0)
                };

                let rot = s
                    .properties
                    .transform
                    .as_ref()
                    .map(|t| t.rot as f64 / 60000.0)
                    .unwrap_or(0.0);

                let fill_hex = match s.properties.fill {
                    Some(Fill::Solid(ColorChoice::Srgb(ref hex))) => format!("#{hex}"),
                    _ => "#4472C4".to_string(),
                };

                let rot_attr = if rot.abs() > 1e-4 {
                    format!(
                        r#" transform="rotate({:.2}, {:.2}, {:.2})""#,
                        rot,
                        x + w / 2.0,
                        y + h / 2.0
                    )
                } else {
                    String::new()
                };

                let is_ellipse = matches!(s.properties.geometry, Some(Geometry::Preset(ref p)) if p == "ellipse");

                if is_ellipse {
                    let rx = w / 2.0;
                    let ry = h / 2.0;
                    let cx = x + rx;
                    let cy = y + ry;
                    svg.push_str(&format!(
                        r#"  <ellipse cx="{:.2}" cy="{:.2}" rx="{:.2}" ry="{:.2}" fill="{}"{}/>
"#,
                        cx, cy, rx, ry, fill_hex, rot_attr
                    ));
                } else {
                    svg.push_str(&format!(
                        r#"  <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="3" ry="3" fill="{}"{}/>
"#,
                        x, y, w, h, fill_hex, rot_attr
                    ));
                }

                // Render text if present
                if let Some(ref tb) = s.text_body {
                    let mut text_y = y + 24.0;
                    for p in &tb.paragraphs {
                        let mut text_line = String::new();
                        for r in &p.runs {
                            if let TextRunElement::Text(ref txt, _) = r {
                                text_line.push_str(&quick_xml::escape::escape(txt));
                            }
                        }
                        if !text_line.is_empty() {
                            svg.push_str(&format!(
                                r##"  <text x="{:.2}" y="{:.2}" class="slide-text" font-size="14pt" fill="#FFFFFF"{}>{}</text>
"##,
                                x + 12.0,
                                text_y,
                                rot_attr,
                                text_line
                            ));
                            text_y += 20.0;
                        }
                    }
                }
            }
            ShapeElement::Group(g) => {
                for child in &g.children {
                    Self::render_element_to_svg(svg, child);
                }
            }
            ShapeElement::GraphicFrame(_) => {}
        }
    }
}

// =========================================================================
// 2. High-DPI Multi-Page PDF Deck Exporter
// =========================================================================

pub struct DeckPdfExporter;

impl DeckPdfExporter {
    /// Exports the entire presentation package to a standard ISO 32000-1 PDF document.
    pub fn export_deck_pdf(pkg: &PresentationPackage) -> Result<Vec<u8>, String> {
        let (w_pt, h_pt) = SlideSvgExporter::get_canvas_dimensions_pt(pkg);
        let num_slides = pkg.slides.len().max(1);

        let mut out = Vec::with_capacity(16384);
        let mut xref_offsets = Vec::new();

        // 1. PDF Header
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        // Object ID 1: Catalog
        xref_offsets.push(out.len());
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Object ID 2: Pages root
        xref_offsets.push(out.len());
        let mut kids_str = String::new();
        for i in 0..num_slides {
            let page_obj_id = 3 + i * 3;
            kids_str.push_str(&format!("{page_obj_id} 0 R "));
        }
        let pages_obj = format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids_str.trim_end(),
            num_slides
        );
        out.extend_from_slice(pages_obj.as_bytes());

        // For each slide:
        // Object (3 + i*3): Page
        // Object (3 + i*3 + 1): Font
        // Object (3 + i*3 + 2): Content Stream
        for (i, (_part, slide)) in pkg.slides.iter().enumerate() {
            let page_id = 3 + i * 3;
            let font_id = page_id + 1;
            let content_id = page_id + 2;

            // Page Object
            xref_offsets.push(out.len());
            let page_def = format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>\nendobj\n",
                page_id, w_pt, h_pt, content_id, font_id
            );
            out.extend_from_slice(page_def.as_bytes());

            // Font Object
            xref_offsets.push(out.len());
            let font_def = format!(
                "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
                font_id
            );
            out.extend_from_slice(font_def.as_bytes());

            // Content Stream Object
            let stream_bytes = Self::build_slide_pdf_content(slide, h_pt);
            xref_offsets.push(out.len());
            let content_def = format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n",
                content_id,
                stream_bytes.len()
            );
            out.extend_from_slice(content_def.as_bytes());
            out.extend_from_slice(&stream_bytes);
            out.extend_from_slice(b"\nendstream\nendobj\n");
        }

        // Cross-reference table
        let xref_pos = out.len();
        let total_objects = xref_offsets.len() + 1;
        out.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", total_objects).as_bytes(),
        );
        for offset in xref_offsets {
            out.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // Trailer
        let trailer_str = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects, xref_pos
        );
        out.extend_from_slice(trailer_str.as_bytes());

        Ok(out)
    }

    fn build_slide_pdf_content(slide: &Slide, h_pt: f64) -> Vec<u8> {
        let mut stream = String::with_capacity(1024);

        // White background
        stream.push_str("q\n1 1 1 rg\n0 0 10000 10000 re\nf\nQ\n");

        for elem in &slide.common_slide_data.shape_tree.shapes {
            if let ShapeElement::Shape(ref s) = elem {
                if let Some(ref t) = s.properties.transform {
                    let x = t.x as f64 / 12700.0;
                    let w = t.cx as f64 / 12700.0;
                    let h = t.cy as f64 / 12700.0;
                    // PDF origin is bottom-left, PresentationML is top-left
                    let y = h_pt - (t.y as f64 / 12700.0) - h;

                    // Parse RGB fill
                    let (r, g, b) = match s.properties.fill {
                        Some(Fill::Solid(ColorChoice::Srgb(ref hex))) => Self::parse_hex_rgb(hex),
                        _ => (0.267, 0.447, 0.769), // Corporate blue
                    };

                    // Draw shape rectangle
                    stream.push_str(&format!(
                        "q\n{:.3} {:.3} {:.3} rg\n{:.2} {:.2} {:.2} {:.2} re\nf\nQ\n",
                        r, g, b, x, y, w, h
                    ));

                    // Draw text if present
                    if let Some(ref tb) = s.text_body {
                        let mut line_y = y + h - 20.0;
                        for p in &tb.paragraphs {
                            let mut text_line = String::new();
                            for r in &p.runs {
                                if let TextRunElement::Text(ref txt, _) = r {
                                    text_line.push_str(txt);
                                }
                            }
                            if !text_line.is_empty() {
                                let escaped = text_line.replace('(', "\\(").replace(')', "\\)");
                                stream.push_str(&format!(
                                    "BT\n/F1 12 Tf\n1 1 1 rg\n{:.2} {:.2} Td\n({}) Tj\nET\n",
                                    x + 8.0,
                                    line_y,
                                    escaped
                                ));
                                line_y -= 16.0;
                            }
                        }
                    }
                }
            }
        }

        stream.into_bytes()
    }

    fn parse_hex_rgb(hex: &str) -> (f64, f64, f64) {
        let clean = hex.trim_start_matches('#');
        if clean.len() >= 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(68) as f64 / 255.0;
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(114) as f64 / 255.0;
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(196) as f64 / 255.0;
            (r, g, b)
        } else {
            (0.267, 0.447, 0.769)
        }
    }
}
