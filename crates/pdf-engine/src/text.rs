//! PDF Text Extraction and Layout Model (ISO 32000-1 Clause 9.3 - 9.4).

use crate::catalog::PdfDocument;
use crate::content::{ContentOperator, ContentParser, TextChunk};
use crate::error::PdfError;
use crate::font::PdfFont;
use crate::page::PdfPage;
use crate::types::{PdfObject, PdfRect};
use std::collections::HashMap;

/// An individual positioned glyph extracted from a content stream.
#[derive(Debug, Clone, PartialEq)]
pub struct TextGlyph {
    pub char_code: u32,
    pub unicode: String,
    pub width: f64,
    pub bbox: PdfRect,
}

/// A contiguous run of text sharing font, size, matrix, and color styling.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub font_name: String,
    pub font_size: f64,
    pub color: [f32; 4],
    pub matrix: [f64; 6],
    pub bbox: PdfRect,
    pub glyphs: Vec<TextGlyph>,
}

/// Complete extracted text representation for a page.
#[derive(Debug, Clone, Default)]
pub struct ExtractedPageText {
    pub page_number: usize,
    pub runs: Vec<TextRun>,
}

#[derive(Clone)]
struct GraphicsState {
    ctm: [f64; 6],
    fill_color: [f32; 4],
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            fill_color: [0.0, 0.0, 0.0, 1.0], // default black
        }
    }
}

#[derive(Clone)]
struct TextState {
    tm: [f64; 6],
    tlm: [f64; 6],
    font_name: String,
    font_size: f64,
    char_spacing: f64,
    word_spacing: f64,
    h_scale: f64,
    leading: f64,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            tm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            tlm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            font_name: String::new(),
            font_size: 12.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            h_scale: 1.0, // 100%
            leading: 0.0,
        }
    }
}

impl ExtractedPageText {
    /// Extracts structured text runs from a page's content streams and resource fonts.
    pub fn extract(page: &PdfPage, doc: &PdfDocument) -> Result<Self, PdfError> {
        // Resolve fonts from page resources
        let mut fonts = HashMap::new();
        if let Some(PdfObject::Dictionary(res_fonts)) = page.resources.get("Font") {
            for (fname, f_obj) in res_fonts {
                let font_dict = match f_obj {
                    PdfObject::Dictionary(d) => Some(d.clone()),
                    PdfObject::Reference(ref_id) => {
                        if let Ok(PdfObject::Dictionary(d)) = doc.resolve_object(*ref_id) {
                            Some(d)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(ref d) = font_dict {
                    if let Ok(parsed_font) = PdfFont::from_dict(d, doc) {
                        fonts.insert(fname.clone(), parsed_font);
                    }
                }
            }
        }

        // Concatenate all content streams for this page
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
        let mut runs = Vec::new();

        let mut gs_stack = vec![GraphicsState::default()];
        let mut ts = TextState::default();

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
                    gs.ctm = multiply_matrices(m, gs.ctm);
                }
                ContentOperator::SetGrayFill(g) => {
                    gs.fill_color = [g, g, g, 1.0];
                }
                ContentOperator::SetRgbFill { r, g, b } => {
                    gs.fill_color = [r, g, b, 1.0];
                }
                ContentOperator::SetCmykFill { c, m, y, k } => {
                    // Convert CMYK to RGB approximation
                    let r = (1.0 - c) * (1.0 - k);
                    let g = (1.0 - m) * (1.0 - k);
                    let b = (1.0 - y) * (1.0 - k);
                    gs.fill_color = [r, g, b, 1.0];
                }
                ContentOperator::BeginText => {
                    ts.tm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                    ts.tlm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                }
                ContentOperator::EndText => {}
                ContentOperator::SetFont { name, size } => {
                    ts.font_name = name;
                    ts.font_size = size;
                }
                ContentOperator::SetCharSpacing(c) => ts.char_spacing = c,
                ContentOperator::SetWordSpacing(w) => ts.word_spacing = w,
                ContentOperator::SetHorizontalScaling(s) => ts.h_scale = s / 100.0,
                ContentOperator::SetLeading(l) => ts.leading = l,
                ContentOperator::MoveTextPosition { tx, ty } => {
                    let trans = [1.0, 0.0, 0.0, 1.0, tx, ty];
                    ts.tlm = multiply_matrices(trans, ts.tlm);
                    ts.tm = ts.tlm;
                }
                ContentOperator::MoveTextPositionSetLeading { tx, ty } => {
                    ts.leading = -ty;
                    let trans = [1.0, 0.0, 0.0, 1.0, tx, ty];
                    ts.tlm = multiply_matrices(trans, ts.tlm);
                    ts.tm = ts.tlm;
                }
                ContentOperator::SetTextMatrix(m) => {
                    ts.tm = m;
                    ts.tlm = m;
                }
                ContentOperator::MoveToNextLine => {
                    let trans = [1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading];
                    ts.tlm = multiply_matrices(trans, ts.tlm);
                    ts.tm = ts.tlm;
                }
                ContentOperator::ShowText(bytes) => {
                    Self::process_show_text(&bytes, &mut ts, gs, &fonts, &mut runs);
                }
                ContentOperator::MoveToNextLineShowText(bytes) => {
                    let trans = [1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading];
                    ts.tlm = multiply_matrices(trans, ts.tlm);
                    ts.tm = ts.tlm;
                    Self::process_show_text(&bytes, &mut ts, gs, &fonts, &mut runs);
                }
                ContentOperator::SetSpacingShowText { aw, ac, text } => {
                    ts.word_spacing = aw;
                    ts.char_spacing = ac;
                    let trans = [1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading];
                    ts.tlm = multiply_matrices(trans, ts.tlm);
                    ts.tm = ts.tlm;
                    Self::process_show_text(&text, &mut ts, gs, &fonts, &mut runs);
                }
                ContentOperator::ShowTextAdjusted(chunks) => {
                    for chunk in chunks {
                        match chunk {
                            TextChunk::Text(b) => {
                                Self::process_show_text(&b, &mut ts, gs, &fonts, &mut runs);
                            }
                            TextChunk::Spacing(adj) => {
                                // Spacing in 1/1000th of text unit (negative moves right)
                                let tx = (-adj / 1000.0) * ts.font_size * ts.h_scale;
                                let trans = [1.0, 0.0, 0.0, 1.0, tx, 0.0];
                                ts.tm = multiply_matrices(trans, ts.tm);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            page_number: page.page_number,
            runs,
        })
    }

    fn process_show_text(
        bytes: &[u8],
        ts: &mut TextState,
        gs: &GraphicsState,
        fonts: &HashMap<String, PdfFont>,
        runs: &mut Vec<TextRun>,
    ) {
        if bytes.is_empty() {
            return;
        }

        let default_font = PdfFont::new("Helvetica".into(), "Type1".into());
        let font = fonts.get(&ts.font_name).unwrap_or(&default_font);
        let decoded_str = font.decode_text(bytes);

        let eff_matrix = multiply_matrices(ts.tm, gs.ctm);
        let start_x = eff_matrix[4];
        let start_y = eff_matrix[5];

        let mut glyphs = Vec::with_capacity(bytes.len());
        let mut cur_x = start_x;
        let mut total_width = 0.0;

        for &b in bytes {
            let code = b as u32;
            let uni = font
                .cmap
                .as_ref()
                .and_then(|c| c.lookup(code))
                .map(String::from)
                .unwrap_or_else(|| (crate::font::win_ansi_to_unicode(b)).to_string());

            let glyph_w = font.glyph_width(code);
            let adv = (glyph_w / 1000.0) * ts.font_size * ts.h_scale + ts.char_spacing;
            let width_pts = adv * eff_matrix[0];

            let bbox = PdfRect::new(
                cur_x,
                start_y,
                cur_x + width_pts,
                start_y + ts.font_size * eff_matrix[3],
            );

            glyphs.push(TextGlyph {
                char_code: code,
                unicode: uni,
                width: width_pts,
                bbox,
            });

            cur_x += width_pts;
            total_width += width_pts;
        }

        let run_bbox = PdfRect::new(
            start_x,
            start_y,
            start_x + total_width,
            start_y + ts.font_size * eff_matrix[3],
        );

        runs.push(TextRun {
            text: decoded_str,
            font_name: ts.font_name.clone(),
            font_size: ts.font_size,
            color: gs.fill_color,
            matrix: eff_matrix,
            bbox: run_bbox,
            glyphs,
        });

        // Advance Tm by text width
        let tx = (total_width / eff_matrix[0]).max(0.0);
        let advance_matrix = [1.0, 0.0, 0.0, 1.0, tx, 0.0];
        ts.tm = multiply_matrices(advance_matrix, ts.tm);
    }

    /// Reconstructs readable plain text with words and paragraph line breaks.
    pub fn to_plain_text(&self) -> String {
        if self.runs.is_empty() {
            return String::new();
        }

        // Sort runs top-to-bottom (descending Y in PDF coordinates), then left-to-right (ascending X)
        let mut sorted = self.runs.clone();
        sorted.sort_by(|a, b| {
            // PDF origin is bottom-left, so larger Y is higher on page
            let y_diff = (b.bbox.lly - a.bbox.lly).abs();
            if y_diff > 4.0 {
                b.bbox
                    .lly
                    .partial_cmp(&a.bbox.lly)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.bbox
                    .llx
                    .partial_cmp(&b.bbox.llx)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let mut output = String::new();
        let mut last_y: Option<f64> = None;
        let mut last_x_end: Option<f64> = None;

        for run in sorted {
            if let Some(ly) = last_y {
                if (ly - run.bbox.lly).abs() > (run.font_size * 0.5) {
                    output.push('\n');
                } else if let Some(lx) = last_x_end {
                    let gap = run.bbox.llx - lx;
                    if gap > (run.font_size * 0.2) {
                        output.push(' ');
                    }
                }
            }

            output.push_str(&run.text);
            last_y = Some(run.bbox.lly);
            last_x_end = Some(run.bbox.urx);
        }

        output
    }
}

/// 2D Affine Matrix multiplication `A * B`:
/// `[a, b, c, d, e, f]` represents `[a b 0; c d 0; e f 1]`.
#[inline]
fn multiply_matrices(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}
