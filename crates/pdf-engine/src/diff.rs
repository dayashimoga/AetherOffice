#![forbid(unsafe_code)]
//! Side-by-Side PDF Visual & Text Diff Engine (ISO 32000-2)
//!
//! Complies with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics, zero unhandled errors in non-test production code.
//! - Word-level text diffing with spatial bounding box preservation.
//! - Document structure diffing (page count, MediaBox geometry).
//! - Visual diff overlay SVG generation with precise PDF coordinate inversion.

use serde::{Deserialize, Serialize};

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::PdfRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    Inserted,
    Deleted,
    Modified,
    Moved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl From<&PdfRect> for DiffBoundingBox {
    fn from(r: &PdfRect) -> Self {
        Self {
            x: r.llx,
            y: r.lly,
            width: (r.urx - r.llx).abs(),
            height: (r.ury - r.lly).abs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffItemDto {
    pub id: String,
    pub kind: DiffKind,
    pub page_index_a: Option<usize>,
    pub page_index_b: Option<usize>,
    pub text_a: Option<String>,
    pub text_b: Option<String>,
    pub bbox_a: Option<DiffBoundingBox>,
    pub bbox_b: Option<DiffBoundingBox>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageDiffDto {
    pub page_index: usize,
    pub has_changes: bool,
    pub media_box_changed: bool,
    pub dimension_a: Option<(f64, f64)>,
    pub dimension_b: Option<(f64, f64)>,
    pub items: Vec<DiffItemDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDiffSummaryDto {
    pub total_diffs: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub page_count_a: usize,
    pub page_count_b: usize,
    pub pages_with_diffs: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDiffResultDto {
    pub summary: PdfDiffSummaryDto,
    pub pages: Vec<PageDiffDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDiffOptions {
    pub ignore_case: bool,
    pub ignore_whitespace: bool,
    pub detect_moves: bool,
}

impl Default for PdfDiffOptions {
    fn default() -> Self {
        Self {
            ignore_case: false,
            ignore_whitespace: true,
            detect_moves: false,
        }
    }
}

#[derive(Debug, Clone)]
struct WordToken {
    text: String,
    bbox: DiffBoundingBox,
}

pub struct PdfDiffEngine;

impl PdfDiffEngine {
    /// Compares two PDF documents and returns a structured comparison result.
    pub fn compare(
        doc_a_bytes: &[u8],
        doc_b_bytes: &[u8],
        options: Option<&PdfDiffOptions>,
    ) -> Result<PdfDiffResultDto, PdfError> {
        if doc_a_bytes.is_empty() || doc_b_bytes.is_empty() {
            return Err(PdfError::UnexpectedEof);
        }

        let default_opt = PdfDiffOptions::default();
        let opt = options.unwrap_or(&default_opt);

        let doc_a = PdfDocument::parse(doc_a_bytes.to_vec())?;
        let doc_b = PdfDocument::parse(doc_b_bytes.to_vec())?;

        let page_count_a = doc_a.page_count();
        let page_count_b = doc_b.page_count();
        let max_pages = page_count_a.max(page_count_b);

        let mut page_diffs = Vec::with_capacity(max_pages);
        let mut total_insertions = 0usize;
        let mut total_deletions = 0usize;
        let mut total_modifications = 0usize;
        let mut pages_with_diffs = 0usize;
        let mut global_item_counter = 1usize;

        for page_idx in 0..max_pages {
            let mut items = Vec::new();
            let mut media_box_changed = false;
            let mut dim_a = None;
            let mut dim_b = None;

            let in_a = page_idx < page_count_a;
            let in_b = page_idx < page_count_b;

            if in_a && in_b {
                let page_a = doc_a.page(page_idx)?;
                let page_b = doc_b.page(page_idx)?;

                dim_a = Some((page_a.width_pts(), page_a.height_pts()));
                dim_b = Some((page_b.width_pts(), page_b.height_pts()));

                if (page_a.width_pts() - page_b.width_pts()).abs() > 0.5
                    || (page_a.height_pts() - page_b.height_pts()).abs() > 0.5
                {
                    media_box_changed = true;
                    items.push(DiffItemDto {
                        id: format!("diff-mb-{page_idx}"),
                        kind: DiffKind::Modified,
                        page_index_a: Some(page_idx),
                        page_index_b: Some(page_idx),
                        text_a: None,
                        text_b: None,
                        bbox_a: Some(DiffBoundingBox {
                            x: 0.0,
                            y: 0.0,
                            width: page_a.width_pts(),
                            height: page_a.height_pts(),
                        }),
                        bbox_b: Some(DiffBoundingBox {
                            x: 0.0,
                            y: 0.0,
                            width: page_b.width_pts(),
                            height: page_b.height_pts(),
                        }),
                        description: format!(
                            "Page dimensions changed from {:.1}x{:.1} pt to {:.1}x{:.1} pt",
                            page_a.width_pts(),
                            page_a.height_pts(),
                            page_b.width_pts(),
                            page_b.height_pts()
                        ),
                    });
                    total_modifications += 1;
                }

                // Extract word tokens with spatial bounding boxes
                let words_a = Self::extract_words(page_a, &doc_a)?;
                let words_b = Self::extract_words(page_b, &doc_b)?;

                let (page_items, ins, del, modif) =
                    Self::diff_words(&words_a, &words_b, page_idx, opt, &mut global_item_counter);

                total_insertions += ins;
                total_deletions += del;
                total_modifications += modif;
                items.extend(page_items);
            } else if in_a {
                // Page deleted in B
                let page_a = doc_a.page(page_idx)?;
                dim_a = Some((page_a.width_pts(), page_a.height_pts()));
                items.push(DiffItemDto {
                    id: format!("diff-del-page-{page_idx}"),
                    kind: DiffKind::Deleted,
                    page_index_a: Some(page_idx),
                    page_index_b: None,
                    text_a: Some(format!("Page {}", page_idx + 1)),
                    text_b: None,
                    bbox_a: Some(DiffBoundingBox {
                        x: 0.0,
                        y: 0.0,
                        width: page_a.width_pts(),
                        height: page_a.height_pts(),
                    }),
                    bbox_b: None,
                    description: format!("Page {} deleted in revised document", page_idx + 1),
                });
                total_deletions += 1;
            } else {
                // Page inserted in B
                let page_b = doc_b.page(page_idx)?;
                dim_b = Some((page_b.width_pts(), page_b.height_pts()));
                items.push(DiffItemDto {
                    id: format!("diff-ins-page-{page_idx}"),
                    kind: DiffKind::Inserted,
                    page_index_a: None,
                    page_index_b: Some(page_idx),
                    text_a: None,
                    text_b: Some(format!("Page {}", page_idx + 1)),
                    bbox_a: None,
                    bbox_b: Some(DiffBoundingBox {
                        x: 0.0,
                        y: 0.0,
                        width: page_b.width_pts(),
                        height: page_b.height_pts(),
                    }),
                    description: format!("Page {} inserted in revised document", page_idx + 1),
                });
                total_insertions += 1;
            }

            let has_changes = !items.is_empty();
            if has_changes {
                pages_with_diffs += 1;
            }

            page_diffs.push(PageDiffDto {
                page_index: page_idx,
                has_changes,
                media_box_changed,
                dimension_a: dim_a,
                dimension_b: dim_b,
                items,
            });
        }

        let total_diffs = total_insertions + total_deletions + total_modifications;

        Ok(PdfDiffResultDto {
            summary: PdfDiffSummaryDto {
                total_diffs,
                insertions: total_insertions,
                deletions: total_deletions,
                modifications: total_modifications,
                page_count_a,
                page_count_b,
                pages_with_diffs,
            },
            pages: page_diffs,
        })
    }

    /// Generates standalone SVG diff overlay markup for a given page.
    pub fn generate_diff_overlay_svg(
        doc_bytes: &[u8],
        page_index: usize,
        items: &[DiffItemDto],
        is_revised: bool,
    ) -> Result<String, PdfError> {
        if doc_bytes.is_empty() {
            return Err(PdfError::UnexpectedEof);
        }

        let doc = PdfDocument::parse(doc_bytes.to_vec())?;
        if page_index >= doc.page_count() {
            return Err(PdfError::PageOutOfRange {
                index: page_index,
                count: doc.page_count(),
            });
        }

        let page = doc.page(page_index)?;
        let width = page.width_pts();
        let height = page.height_pts();

        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.2} {height:.2}" width="{width:.2}pt" height="{height:.2}pt" style="pointer-events: none;">"##
        );

        for item in items {
            let (bbox, fill_color, stroke_color) = if is_revised {
                match item.kind {
                    DiffKind::Inserted => {
                        (item.bbox_b.as_ref(), "rgba(34, 197, 94, 0.25)", "#16a34a")
                    }
                    DiffKind::Modified => {
                        (item.bbox_b.as_ref(), "rgba(245, 158, 11, 0.25)", "#d97706")
                    }
                    _ => (None, "", ""),
                }
            } else {
                match item.kind {
                    DiffKind::Deleted => {
                        (item.bbox_a.as_ref(), "rgba(239, 68, 68, 0.25)", "#dc2626")
                    }
                    DiffKind::Modified => {
                        (item.bbox_a.as_ref(), "rgba(245, 158, 11, 0.25)", "#d97706")
                    }
                    _ => (None, "", ""),
                }
            };

            if let Some(box_val) = bbox {
                let svg_y = height - box_val.y - box_val.height;
                svg.push_str(&format!(
                    r##"<rect id="{}" x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{}" stroke="{}" stroke-width="1.5" stroke-dasharray="{}" data-kind="{:?}" />"##,
                    item.id,
                    box_val.x,
                    svg_y.max(0.0),
                    box_val.width.max(2.0),
                    box_val.height.max(2.0),
                    fill_color,
                    stroke_color,
                    if item.kind == DiffKind::Modified { "3,3" } else { "none" },
                    item.kind
                ));
            }
        }

        svg.push_str("</svg>");
        Ok(svg)
    }

    fn extract_words(
        page: &crate::page::PdfPage,
        doc: &PdfDocument,
    ) -> Result<Vec<WordToken>, PdfError> {
        let extracted = page.extract_text(doc)?;
        let mut words = Vec::new();

        for run in &extracted.runs {
            let run_text = &run.text;
            if run_text.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = run_text.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // If glyphs are available, compute precise glyph-bounded words
            if !run.glyphs.is_empty() {
                let mut current_word = String::new();
                let mut word_glyphs = Vec::new();

                for g in &run.glyphs {
                    if g.unicode.chars().all(|c| c.is_whitespace()) {
                        if !current_word.is_empty() {
                            let bbox = Self::glyphs_bounding_box(&word_glyphs);
                            words.push(WordToken {
                                text: current_word.clone(),
                                bbox,
                            });
                            current_word.clear();
                            word_glyphs.clear();
                        }
                    } else {
                        current_word.push_str(&g.unicode);
                        word_glyphs.push(g.clone());
                    }
                }
                if !current_word.is_empty() {
                    let bbox = Self::glyphs_bounding_box(&word_glyphs);
                    words.push(WordToken {
                        text: current_word,
                        bbox,
                    });
                }
            } else {
                // Fallback to run bbox estimation
                let total_chars = run_text.len().max(1) as f64;
                let char_width = (run.bbox.urx - run.bbox.llx).abs() / total_chars;
                let mut current_offset = 0usize;

                for part in parts {
                    let part_len = part.len();
                    let start_x = run.bbox.llx + (current_offset as f64 * char_width);
                    let word_w = (part_len as f64 * char_width).max(4.0);
                    let word_h = (run.bbox.ury - run.bbox.lly).abs().max(4.0);

                    words.push(WordToken {
                        text: part.to_string(),
                        bbox: DiffBoundingBox {
                            x: start_x,
                            y: run.bbox.lly,
                            width: word_w,
                            height: word_h,
                        },
                    });

                    current_offset += part_len + 1;
                }
            }
        }

        Ok(words)
    }

    fn glyphs_bounding_box(glyphs: &[crate::text::TextGlyph]) -> DiffBoundingBox {
        if glyphs.is_empty() {
            return DiffBoundingBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for g in glyphs {
            if g.bbox.llx < min_x {
                min_x = g.bbox.llx;
            }
            if g.bbox.lly < min_y {
                min_y = g.bbox.lly;
            }
            if g.bbox.urx > max_x {
                max_x = g.bbox.urx;
            }
            if g.bbox.ury > max_y {
                max_y = g.bbox.ury;
            }
        }

        DiffBoundingBox {
            x: min_x,
            y: min_y,
            width: (max_x - min_x).max(1.0),
            height: (max_y - min_y).max(1.0),
        }
    }

    fn diff_words(
        words_a: &[WordToken],
        words_b: &[WordToken],
        page_idx: usize,
        opt: &PdfDiffOptions,
        counter: &mut usize,
    ) -> (Vec<DiffItemDto>, usize, usize, usize) {
        let n = words_a.len();
        let m = words_b.len();

        let normalize = |s: &str| -> String {
            if opt.ignore_case {
                s.to_lowercase()
            } else {
                s.to_string()
            }
        };

        // Longest Common Subsequence (LCS) matrix
        let mut dp = vec![vec![0usize; m + 1]; n + 1];
        for i in 0..n {
            for j in 0..m {
                if normalize(&words_a[i].text) == normalize(&words_b[j].text) {
                    dp[i + 1][j + 1] = dp[i][j] + 1;
                } else {
                    dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
                }
            }
        }

        // Backtrack to extract insertions, deletions, and modifications
        let mut i = n;
        let mut j = m;
        let mut raw_ops = Vec::new();

        while i > 0 || j > 0 {
            if i > 0 && j > 0 && normalize(&words_a[i - 1].text) == normalize(&words_b[j - 1].text)
            {
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
                raw_ops.push((DiffKind::Inserted, None, Some(j - 1)));
                j -= 1;
            } else if i > 0 {
                raw_ops.push((DiffKind::Deleted, Some(i - 1), None));
                i -= 1;
            }
        }

        raw_ops.reverse();

        // Coalesce adjacent Delete and Insert into Modified
        let mut items = Vec::new();
        let mut ins_count = 0usize;
        let mut del_count = 0usize;
        let mut mod_count = 0usize;

        let mut idx = 0usize;
        while idx < raw_ops.len() {
            let (kind, a_opt, b_opt) = raw_ops[idx];

            if kind == DiffKind::Deleted
                && idx + 1 < raw_ops.len()
                && raw_ops[idx + 1].0 == DiffKind::Inserted
            {
                // Pair deletion and insertion into Modification
                let wa = a_opt.and_then(|a_idx| words_a.get(a_idx));
                let wb = raw_ops[idx + 1].2.and_then(|b_idx| words_b.get(b_idx));

                items.push(DiffItemDto {
                    id: format!("diff-item-{}", *counter),
                    kind: DiffKind::Modified,
                    page_index_a: Some(page_idx),
                    page_index_b: Some(page_idx),
                    text_a: wa.map(|w| w.text.clone()),
                    text_b: wb.map(|w| w.text.clone()),
                    bbox_a: wa.map(|w| w.bbox.clone()),
                    bbox_b: wb.map(|w| w.bbox.clone()),
                    description: format!(
                        "Changed '{}' to '{}'",
                        wa.map(|w| w.text.as_str()).unwrap_or(""),
                        wb.map(|w| w.text.as_str()).unwrap_or("")
                    ),
                });
                *counter += 1;
                mod_count += 1;
                idx += 2;
            } else if kind == DiffKind::Inserted {
                let wb = b_opt.and_then(|b_idx| words_b.get(b_idx));
                items.push(DiffItemDto {
                    id: format!("diff-item-{}", *counter),
                    kind: DiffKind::Inserted,
                    page_index_a: None,
                    page_index_b: Some(page_idx),
                    text_a: None,
                    text_b: wb.map(|w| w.text.clone()),
                    bbox_a: None,
                    bbox_b: wb.map(|w| w.bbox.clone()),
                    description: format!(
                        "Inserted '{}'",
                        wb.map(|w| w.text.as_str()).unwrap_or("")
                    ),
                });
                *counter += 1;
                ins_count += 1;
                idx += 1;
            } else {
                let wa = a_opt.and_then(|a_idx| words_a.get(a_idx));
                items.push(DiffItemDto {
                    id: format!("diff-item-{}", *counter),
                    kind: DiffKind::Deleted,
                    page_index_a: Some(page_idx),
                    page_index_b: None,
                    text_a: wa.map(|w| w.text.clone()),
                    text_b: None,
                    bbox_a: wa.map(|w| w.bbox.clone()),
                    bbox_b: None,
                    description: format!("Deleted '{}'", wa.map(|w| w.text.as_str()).unwrap_or("")),
                });
                *counter += 1;
                del_count += 1;
                idx += 1;
            }
        }

        (items, ins_count, del_count, mod_count)
    }
}
