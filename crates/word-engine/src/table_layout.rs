#![forbid(unsafe_code)]
//! ECMA-376 / WordprocessingML Table Auto-Fit Layout & Floating Object Wrapping Solver
//!
//! Implements:
//! - Table column auto-fit solver (`w:tblLayout w:type="autofit"`):
//!   `minContentWidth` (unbreakable word bound) vs `maxContentWidth` (single-line un-wrapped paragraph bound)
//!   with proportional distribution across `w:gridSpan` merged columns and target table widths.
//! - Cell width unit conversion (`dxa`, `pct` in 50ths of a percent, `auto`).
//! - Cross-page table row breaking with `w:cantSplit` and `w:tblHeader` repeating headers.
//! - Floating DrawingML anchor (`wp:anchor`) text exclusion zones:
//!   `wrapSquare`, `wrapTight` / `wrapThrough` polygon ray-casting intersection, and `wrapTopAndBottom`.

use serde::{Deserialize, Serialize};

use crate::layout::{
    BoundingBox, FontMetricsProvider, LayoutLine, LayoutRun, LineBreaker, TokenKind,
};
use crate::model::{BodyElement, Table, TableCell, TableRow};
use crate::styles::StyleCascadeResolver;

// =========================================================================
// 1. Data Structures for Table & Float Layout
// =========================================================================

/// 2D Point coordinate in twips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

impl Point2D {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Text wrap style for floating DrawingML anchors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WrapType {
    #[default]
    None,
    Square,
    Tight,
    Through,
    TopAndBottom,
}

/// Floating DrawingML anchor obstacle configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingAnchor {
    pub bounds: BoundingBox,
    pub dist_top: u32,
    pub dist_bottom: u32,
    pub dist_left: u32,
    pub dist_right: u32,
    pub wrap_type: WrapType,
    pub wrap_polygon: Vec<Point2D>,
}

impl FloatingAnchor {
    pub fn new(bounds: BoundingBox, wrap_type: WrapType) -> Self {
        Self {
            bounds,
            dist_top: 0,
            dist_bottom: 0,
            dist_left: 180, // 0.125 in default text padding
            dist_right: 180,
            wrap_type,
            wrap_polygon: Vec::new(),
        }
    }

    /// Effective bounding box including text padding offsets.
    pub fn effective_bounds(&self) -> BoundingBox {
        let x = self.bounds.x.saturating_sub(self.dist_left);
        let y = self.bounds.y.saturating_sub(self.dist_top);
        let width = self
            .bounds
            .width
            .saturating_add(self.dist_left)
            .saturating_add(self.dist_right);
        let height = self
            .bounds
            .height
            .saturating_add(self.dist_top)
            .saturating_add(self.dist_bottom);
        BoundingBox::new(x, y, width, height)
    }
}

/// Fully laid-out table cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutTableCell {
    pub bounds: BoundingBox,
    pub grid_span: u32,
    pub shading_fill: Option<String>,
    pub lines: Vec<LayoutLine>,
}

/// Fully laid-out table row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutTableRow {
    pub bounds: BoundingBox,
    pub cells: Vec<LayoutTableCell>,
    pub is_header: bool,
    pub is_repeated_header: bool,
}

/// Fully laid-out table ready for page placement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutTable {
    pub bounds: BoundingBox,
    pub column_widths: Vec<u32>,
    pub rows: Vec<LayoutTableRow>,
}

// =========================================================================
// 2. Floating DrawingML Wrap Solver
// =========================================================================

pub struct FloatingWrapSolver;

impl FloatingWrapSolver {
    /// Determines whether a horizontal text line slice at [y_top, y_bottom] intersects the anchor.
    pub fn line_intersects(anchor: &FloatingAnchor, y_top: u32, y_bottom: u32) -> bool {
        if anchor.wrap_type == WrapType::None {
            return false;
        }
        let eff = anchor.effective_bounds();
        let eff_bottom = eff.bottom();
        !(y_bottom <= eff.y || y_top >= eff_bottom)
    }

    /// Computes horizontal exclusion interval `[x_start, x_end]` for a line intersecting the anchor.
    pub fn compute_exclusion_interval(
        anchor: &FloatingAnchor,
        y_mid: u32,
        column_width: u32,
    ) -> Option<(u32, u32)> {
        let eff = anchor.effective_bounds();
        if y_mid < eff.y || y_mid > eff.bottom() {
            return None;
        }

        match anchor.wrap_type {
            WrapType::None => None,
            WrapType::TopAndBottom => {
                // Entire column width is excluded
                Some((0, column_width))
            }
            WrapType::Square => Some((eff.x, eff.right())),
            WrapType::Tight | WrapType::Through => {
                if anchor.wrap_polygon.is_empty() {
                    Some((eff.x, eff.right()))
                } else {
                    let mut intersections = Vec::new();
                    let n = anchor.wrap_polygon.len();
                    let y_i = y_mid as i32;
                    for i in 0..n {
                        let p1 = anchor.wrap_polygon[i];
                        let p2 = anchor.wrap_polygon[(i + 1) % n];

                        if (p1.y <= y_i && p2.y > y_i) || (p2.y <= y_i && p1.y > y_i) {
                            let dy = p2.y - p1.y;
                            if dy != 0 {
                                let dx = p2.x - p1.x;
                                let x = p1.x + ((y_i - p1.y) * dx) / dy;
                                intersections.push(x.max(0) as u32);
                            }
                        }
                    }

                    if intersections.is_empty() {
                        Some((eff.x, eff.right()))
                    } else {
                        intersections.sort_unstable();
                        let min_x = *intersections.first().unwrap_or(&eff.x);
                        let max_x = *intersections.last().unwrap_or(&eff.right());
                        let left = min_x.saturating_sub(anchor.dist_left);
                        let right = max_x.saturating_add(anchor.dist_right);
                        Some((left, right))
                    }
                }
            }
        }
    }

    /// Computes available horizontal text line intervals for a line intersecting an anchor.
    pub fn available_text_intervals(
        anchor: &FloatingAnchor,
        y_mid: u32,
        col_x: u32,
        col_width: u32,
    ) -> Vec<(u32, u32)> {
        if let Some((ex_start, ex_end)) = Self::compute_exclusion_interval(anchor, y_mid, col_width)
        {
            let mut intervals = Vec::new();
            let col_end = col_x.saturating_add(col_width);

            // Left interval
            if ex_start > col_x {
                intervals.push((col_x, ex_start.min(col_end)));
            }
            // Right interval
            if ex_end < col_end {
                intervals.push((ex_end.max(col_x), col_end));
            }

            intervals
        } else {
            vec![(col_x, col_x.saturating_add(col_width))]
        }
    }
}

// =========================================================================
// 3. Table Column Auto-Fit Width Solver
// =========================================================================

pub struct TableAutoFitSolver<'a> {
    metrics: &'a dyn FontMetricsProvider,
    cascade: &'a StyleCascadeResolver<'a>,
}

impl<'a> TableAutoFitSolver<'a> {
    pub fn new(
        metrics: &'a dyn FontMetricsProvider,
        cascade: &'a StyleCascadeResolver<'a>,
    ) -> Self {
        Self { metrics, cascade }
    }

    /// Determines the number of columns in the table grid.
    pub fn resolve_column_count(&self, tbl: &Table) -> usize {
        if let Some(ref grid) = tbl.grid {
            if !grid.columns.is_empty() {
                return grid.columns.len();
            }
        }

        let mut max_cols = 0;
        for row in &tbl.rows {
            let mut row_cols = 0;
            for cell in &row.cells {
                let span = cell
                    .properties
                    .as_ref()
                    .map(|p| p.grid_span)
                    .unwrap_or(1)
                    .max(1);
                row_cols += span as usize;
            }
            max_cols = max_cols.max(row_cols);
        }

        max_cols.max(1)
    }

    /// Computes resolved column widths in twips for the given table and available width.
    pub fn compute_column_widths(&self, tbl: &Table, available_width: u32) -> Vec<u32> {
        let num_cols = self.resolve_column_count(tbl);
        let target_tbl_width = self.resolve_target_table_width(tbl, available_width);

        // Check if table layout is fixed and has explicit grid
        let is_fixed = tbl
            .properties
            .as_ref()
            .and_then(|p| p.layout_type.as_deref())
            == Some("fixed");

        if is_fixed {
            if let Some(ref grid) = tbl.grid {
                if grid.columns.len() == num_cols {
                    return grid.columns.clone();
                }
            }
        }

        // Auto-Fit Algorithm: Compute minContentWidth and maxContentWidth for each grid column
        let mut min_widths = vec![240u32; num_cols]; // minimum cell width floor (12pt = 240 twips)
        let mut max_widths = vec![240u32; num_cols];

        let breaker = LineBreaker::new(self.metrics);

        for row in &tbl.rows {
            let mut col_idx = 0;
            for cell in &row.cells {
                if col_idx >= num_cols {
                    break;
                }

                let span = cell
                    .properties
                    .as_ref()
                    .map(|p| p.grid_span)
                    .unwrap_or(1)
                    .max(1) as usize;

                let margins = cell
                    .properties
                    .as_ref()
                    .and_then(|p| p.cell_margins)
                    .or_else(|| tbl.properties.as_ref().and_then(|p| p.cell_margins))
                    .unwrap_or_default();

                let h_padding = margins.left.saturating_add(margins.right);

                // Measure cell content
                let (cell_min, cell_max) = self.measure_cell_content(cell, &breaker, h_padding);

                if span == 1 {
                    min_widths[col_idx] = min_widths[col_idx].max(cell_min);
                    max_widths[col_idx] = max_widths[col_idx].max(cell_max);
                } else {
                    // Distribute multi-column span excess proportionally
                    let end_col = (col_idx + span).min(num_cols);
                    let span_cols = end_col - col_idx;

                    let curr_span_min: u32 = min_widths[col_idx..end_col].iter().sum();
                    if cell_min > curr_span_min && span_cols > 0 {
                        let excess = cell_min - curr_span_min;
                        let extra_per_col = excess / span_cols as u32;
                        for w in &mut min_widths[col_idx..end_col] {
                            *w = w.saturating_add(extra_per_col);
                        }
                    }

                    let curr_span_max: u32 = max_widths[col_idx..end_col].iter().sum();
                    if cell_max > curr_span_max && span_cols > 0 {
                        let excess = cell_max - curr_span_max;
                        let extra_per_col = excess / span_cols as u32;
                        for w in &mut max_widths[col_idx..end_col] {
                            *w = w.saturating_add(extra_per_col);
                        }
                    }
                }

                col_idx += span;
            }
        }

        // Allocate final widths within target_tbl_width
        self.distribute_widths(&min_widths, &max_widths, target_tbl_width)
    }

    pub fn measure_cell_content(
        &self,
        cell: &TableCell,
        breaker: &LineBreaker,
        padding: u32,
    ) -> (u32, u32) {
        let mut max_token_w = 0u32;
        let mut max_para_w = 0u32;

        for elem in &cell.elements {
            if let BodyElement::Paragraph(ref p) = elem {
                let tokens = breaker.tokenize_paragraph(p, self.cascade);
                let mut para_w = 0u32;
                for t in &tokens {
                    let w = if t.kind == TokenKind::SoftHyphen {
                        0
                    } else {
                        t.width
                    };
                    if t.kind != TokenKind::Space {
                        max_token_w = max_token_w.max(w);
                    }
                    para_w = para_w.saturating_add(w);
                }
                max_para_w = max_para_w.max(para_w);
            }
        }

        let cell_min = max_token_w.saturating_add(padding).max(240);
        let cell_max = max_para_w.saturating_add(padding).max(cell_min);

        (cell_min, cell_max)
    }

    pub fn resolve_target_table_width(&self, tbl: &Table, available_width: u32) -> u32 {
        if let Some(ref tbl_pr) = tbl.properties {
            if let Some(ref w) = tbl_pr.width {
                match w.width_type.as_str() {
                    "dxa" if w.width > 0 => return w.width.min(available_width),
                    "pct" if w.width > 0 => {
                        // In ECMA-376, pct is in 50ths of a percent (5000 = 100%)
                        let pct_twips = (available_width as u64 * w.width as u64 / 5000) as u32;
                        return pct_twips.max(240);
                    }
                    _ => {}
                }
            }
        }
        available_width
    }

    pub fn distribute_widths(
        &self,
        min_widths: &[u32],
        max_widths: &[u32],
        target_width: u32,
    ) -> Vec<u32> {
        let n = min_widths.len();
        if n == 0 {
            return Vec::new();
        }

        let sum_min: u32 = min_widths.iter().sum();
        let sum_max: u32 = max_widths.iter().sum();

        if target_width >= sum_max {
            // Slack can be distributed proportionally to max widths
            let mut widths = Vec::with_capacity(n);
            let extra = target_width.saturating_sub(sum_max);
            for &m in max_widths {
                let share = if sum_max > 0 {
                    (extra as u64 * m as u64 / sum_max as u64) as u32
                } else {
                    extra / n as u32
                };
                widths.push(m.saturating_add(share));
            }
            widths
        } else if target_width >= sum_min {
            // Interpolate proportionally between minContentWidth and maxContentWidth
            let mut widths = Vec::with_capacity(n);
            let denom = sum_max.saturating_sub(sum_min).max(1);
            let numer = target_width.saturating_sub(sum_min);

            for i in 0..n {
                let diff = max_widths[i].saturating_sub(min_widths[i]);
                let add = (diff as u64 * numer as u64 / denom as u64) as u32;
                widths.push(min_widths[i].saturating_add(add));
            }
            widths
        } else {
            // Compress proportionally down to minimum
            let mut widths = Vec::with_capacity(n);
            for &m in min_widths {
                let scaled = if sum_min > 0 {
                    (m as u64 * target_width as u64 / sum_min as u64) as u32
                } else {
                    target_width / n as u32
                };
                widths.push(scaled.max(120)); // hard minimum column width floor 120 twips
            }
            widths
        }
    }
}

// =========================================================================
// 4. Table Pagination & Cross-Page Solver
// =========================================================================

pub struct TablePaginationSolver<'a> {
    metrics: &'a dyn FontMetricsProvider,
    cascade: &'a StyleCascadeResolver<'a>,
}

impl<'a> TablePaginationSolver<'a> {
    pub fn new(
        metrics: &'a dyn FontMetricsProvider,
        cascade: &'a StyleCascadeResolver<'a>,
    ) -> Self {
        Self { metrics, cascade }
    }

    /// Lays out a table and paginates its rows across available page heights.
    pub fn layout_table(
        &self,
        tbl: &Table,
        col_x: u32,
        mut curr_y: u32,
        page_content_bottom: u32,
        page_top_margin: u32,
        available_width: u32,
    ) -> (LayoutTable, u32, usize) {
        let autofit = TableAutoFitSolver::new(self.metrics, self.cascade);
        let col_widths = autofit.compute_column_widths(tbl, available_width);
        let breaker = LineBreaker::new(self.metrics);

        let total_tbl_w: u32 = col_widths.iter().sum();
        let tbl_x = match tbl
            .properties
            .as_ref()
            .and_then(|p| p.justification.as_deref())
        {
            Some("center") => col_x.saturating_add(available_width.saturating_sub(total_tbl_w) / 2),
            Some("right") => col_x.saturating_add(available_width.saturating_sub(total_tbl_w)),
            _ => col_x,
        };

        let mut laid_out_rows = Vec::new();
        let mut header_rows: Vec<LayoutTableRow> = Vec::new();
        let mut pages_spanned = 0;

        for (row_idx, row) in tbl.rows.iter().enumerate() {
            let is_header = row
                .properties
                .as_ref()
                .map(|p| p.is_header)
                .unwrap_or(false)
                || (row_idx == 0
                    && tbl
                        .properties
                        .as_ref()
                        .map(|p| p.style.as_deref() == Some("TableGrid"))
                        .unwrap_or(false));

            let cant_split = row
                .properties
                .as_ref()
                .map(|p| p.cant_split)
                .unwrap_or(false);

            // Lay out cells in row
            let (laid_out_row, row_height) =
                self.layout_row(row, &col_widths, tbl_x, curr_y, is_header, &breaker);

            // Pagination check
            let remaining_page_h = page_content_bottom.saturating_sub(curr_y);
            let should_advance = (row_height > remaining_page_h && !is_header)
                || (cant_split && row_height > remaining_page_h);

            if should_advance {
                // Advance page
                pages_spanned += 1;
                curr_y = page_top_margin;

                // Repeat header rows at top of next page if w:tblHeader is active
                for hdr in &header_rows {
                    let mut repeated_hdr: LayoutTableRow = hdr.clone();
                    repeated_hdr.is_repeated_header = true;
                    let hdr_h = repeated_hdr.bounds.height;
                    repeated_hdr.bounds.y = curr_y;
                    for cell in &mut repeated_hdr.cells {
                        cell.bounds.y = curr_y;
                    }
                    curr_y = curr_y.saturating_add(hdr_h);
                    laid_out_rows.push(repeated_hdr);
                }

                // Place the row at the new position
                let (relocated_row, h) =
                    self.layout_row(row, &col_widths, tbl_x, curr_y, false, &breaker);
                curr_y = curr_y.saturating_add(h);
                laid_out_rows.push(relocated_row);
            } else {
                if is_header {
                    header_rows.push(laid_out_row.clone());
                }
                curr_y = curr_y.saturating_add(row_height);
                laid_out_rows.push(laid_out_row);
            }
        }

        let total_w: u32 = col_widths.iter().sum();
        let total_h = laid_out_rows.iter().map(|r| r.bounds.height).sum::<u32>();

        let table = LayoutTable {
            bounds: BoundingBox::new(tbl_x, page_top_margin, total_w, total_h),
            column_widths: col_widths,
            rows: laid_out_rows,
        };

        (table, curr_y, pages_spanned)
    }

    pub fn layout_row(
        &self,
        row: &TableRow,
        col_widths: &[u32],
        start_x: u32,
        row_y: u32,
        is_header: bool,
        breaker: &LineBreaker,
    ) -> (LayoutTableRow, u32) {
        let mut laid_out_cells = Vec::new();
        let mut curr_x = start_x;
        let mut max_cell_h = 240u32; // minimum row height
        let mut col_idx = 0;

        for cell in &row.cells {
            if col_idx >= col_widths.len() {
                break;
            }

            let span = cell
                .properties
                .as_ref()
                .map(|p| p.grid_span)
                .unwrap_or(1)
                .max(1) as usize;

            let end_col = (col_idx + span).min(col_widths.len());
            let cell_width: u32 = col_widths[col_idx..end_col].iter().sum();

            let margins = cell
                .properties
                .as_ref()
                .and_then(|p| p.cell_margins)
                .unwrap_or_default();

            let inner_w = cell_width
                .saturating_sub(margins.left)
                .saturating_sub(margins.right)
                .max(120);

            let mut cell_lines = Vec::new();
            let mut line_y = row_y.saturating_add(margins.top);

            for elem in &cell.elements {
                if let BodyElement::Paragraph(ref p) = elem {
                    let p_props = self.cascade.resolve_paragraph(p);
                    let tokens = breaker.tokenize_paragraph(p, self.cascade);
                    let lines = breaker.break_lines(&tokens, &p_props, inner_w);

                    let line_h = (p_props.run_defaults.size_half_points.max(20) * 115 / 100)
                        .saturating_mul(10);

                    for line_tokens in lines {
                        let mut line_runs = Vec::new();
                        let mut token_x = curr_x.saturating_add(margins.left);
                        for tok in line_tokens {
                            let tw = tok.width;
                            line_runs.push(LayoutRun {
                                text: tok.text,
                                bounds: BoundingBox::new(token_x, line_y, tw, line_h),
                                font_family: tok.style.font_ascii.clone(),
                                font_size_half_pts: tok.style.size_half_points,
                                bold: tok.style.bold,
                                italic: tok.style.italic,
                                color: tok.style.color.clone(),
                            });
                            token_x = token_x.saturating_add(tw);
                        }

                        cell_lines.push(LayoutLine {
                            bounds: BoundingBox::new(
                                curr_x.saturating_add(margins.left),
                                line_y,
                                inner_w,
                                line_h,
                            ),
                            baseline: line_y.saturating_add(line_h * 8 / 10),
                            paragraph_index: 0,
                            runs: line_runs,
                            is_last_line_of_para: false,
                        });
                        line_y = line_y.saturating_add(line_h);
                    }
                }
            }

            let cell_h = line_y
                .saturating_sub(row_y)
                .saturating_add(margins.bottom)
                .max(240);
            max_cell_h = max_cell_h.max(cell_h);

            laid_out_cells.push(LayoutTableCell {
                bounds: BoundingBox::new(curr_x, row_y, cell_width, cell_h),
                grid_span: span as u32,
                shading_fill: cell
                    .properties
                    .as_ref()
                    .and_then(|p| p.shading_fill.clone()),
                lines: cell_lines,
            });

            curr_x = curr_x.saturating_add(cell_width);
            col_idx += span;
        }

        // Apply explicit trHeight rule if present
        if let Some(ref tr_pr) = row.properties {
            if let Some(h) = tr_pr.height {
                if tr_pr.height_rule.as_deref() == Some("exact") {
                    max_cell_h = h;
                } else {
                    max_cell_h = max_cell_h.max(h);
                }
            }
        }

        // Apply row height to all cells and adjust vertical alignment (w:vAlign)
        for (i, cell_layout) in laid_out_cells.iter_mut().enumerate() {
            cell_layout.bounds.height = max_cell_h;
            if let Some(cell_model) = row.cells.get(i) {
                let v_align = cell_model
                    .properties
                    .as_ref()
                    .and_then(|p| p.vertical_align.as_deref());
                if let Some(valign) = v_align {
                    if let (Some(first_line), Some(last_line)) =
                        (cell_layout.lines.first(), cell_layout.lines.last())
                    {
                        let margins = cell_model
                            .properties
                            .as_ref()
                            .and_then(|p| p.cell_margins)
                            .unwrap_or_default();
                        let first_y = first_line.bounds.y;
                        let last_bottom = last_line.bounds.bottom();
                        let content_h = last_bottom.saturating_sub(first_y);
                        let avail_cell_h = max_cell_h
                            .saturating_sub(margins.top)
                            .saturating_sub(margins.bottom);
                        let offset = match valign {
                            "center" if avail_cell_h > content_h => (avail_cell_h - content_h) / 2,
                            "bottom" if avail_cell_h > content_h => avail_cell_h - content_h,
                            _ => 0,
                        };
                        if offset > 0 {
                            for line in &mut cell_layout.lines {
                                line.bounds.y = line.bounds.y.saturating_add(offset);
                                line.baseline = line.baseline.saturating_add(offset);
                                for run in &mut line.runs {
                                    run.bounds.y = run.bounds.y.saturating_add(offset);
                                }
                            }
                        }
                    }
                }
            }
        }

        let row_w = curr_x.saturating_sub(start_x);
        let layout_row = LayoutTableRow {
            bounds: BoundingBox::new(start_x, row_y, row_w, max_cell_h),
            cells: laid_out_cells,
            is_header,
            is_repeated_header: false,
        };

        (layout_row, max_cell_h)
    }
}
