//! Excel Display List Compiler
//!
//! Provides binary-search viewport indexing for multi-million cell worksheets (Rows 1..1048576, Cols A..XFD),
//! compiling visible cell background fills, instanced grid lines, cell text,
//! selection highlights, freeze panes, and row/column headers.

use crate::primitive::{ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D};
use crate::text::ShapedTextEngine;
use std::collections::BTreeMap;

pub const MAX_EXCEL_ROWS: u32 = 1_048_576;
pub const MAX_EXCEL_COLS: u32 = 16_384;
pub const DEFAULT_ROW_HEIGHT: f64 = 20.0;
pub const DEFAULT_COL_WIDTH: f64 = 64.0;
pub const ROW_HEADER_WIDTH: f64 = 50.0;
pub const COL_HEADER_HEIGHT: f64 = 26.0;

/// Converts a 0-based column index to Excel column name (e.g. 0 -> "A", 25 -> "Z", 26 -> "AA", 16383 -> "XFD").
pub fn col_to_name(col: u32) -> String {
    let mut name = String::new();
    let mut n = col + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        if let Some(ch) = char::from_u32(65 + rem) {
            name.insert(0, ch);
        }
        n = (n - 1) / 26;
    }
    name
}

/// Converts an Excel column name to 0-based index (e.g. "A" -> 0, "Z" -> 25, "AA" -> 26, "XFD" -> 16383).
pub fn name_to_col(name: &str) -> Option<u32> {
    if name.is_empty() {
        return None;
    }
    let mut col: u32 = 0;
    for ch in name.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        let val = (ch.to_ascii_uppercase() as u32).saturating_sub(64);
        col = col.checked_mul(26).and_then(|c| c.checked_add(val))?;
    }
    if col == 0 || col > MAX_EXCEL_COLS {
        None
    } else {
        Some(col - 1)
    }
}

/// Converts a cell coordinate like "A1" or "XFD1048576" to 0-based (row, col).
pub fn cell_coord_to_indices(coord: &str) -> Option<(u32, u32)> {
    let mut letter_end = 0;
    for (i, b) in coord.bytes().enumerate() {
        if b.is_ascii_alphabetic() {
            letter_end = i + 1;
        } else {
            break;
        }
    }
    if letter_end == 0 || letter_end >= coord.len() {
        return None;
    }
    let col_str = &coord[..letter_end];
    let row_str = &coord[letter_end..];

    let col = name_to_col(col_str)?;
    let row: u32 = row_str.parse().ok()?;
    if row == 0 || row > MAX_EXCEL_ROWS {
        return None;
    }
    Some((row - 1, col))
}

/// Converts 0-based (row, col) to coordinate string like "A1".
pub fn indices_to_cell_coord(row: u32, col: u32) -> String {
    format!("{}{}", col_to_name(col), row.saturating_add(1))
}

/// Dimension indexer supporting uniform or sparse variable row heights and column widths.
#[derive(Debug, Clone)]
pub struct GridDimensionMap {
    pub default_row_height: f64,
    pub default_col_width: f64,
    pub custom_row_heights: BTreeMap<u32, f64>,
    pub custom_col_widths: BTreeMap<u32, f64>,
}

impl GridDimensionMap {
    pub fn new(default_row_height: f64, default_col_width: f64) -> Self {
        Self {
            default_row_height: if default_row_height > 0.0 {
                default_row_height
            } else {
                DEFAULT_ROW_HEIGHT
            },
            default_col_width: if default_col_width > 0.0 {
                default_col_width
            } else {
                DEFAULT_COL_WIDTH
            },
            custom_row_heights: BTreeMap::new(),
            custom_col_widths: BTreeMap::new(),
        }
    }

    pub fn set_row_height(&mut self, row: u32, height: f64) {
        if height > 0.0 {
            self.custom_row_heights.insert(row, height);
        }
    }

    pub fn set_col_width(&mut self, col: u32, width: f64) {
        if width > 0.0 {
            self.custom_col_widths.insert(col, width);
        }
    }

    pub fn row_height(&self, row: u32) -> f64 {
        self.custom_row_heights
            .get(&row)
            .copied()
            .unwrap_or(self.default_row_height)
    }

    pub fn col_width(&self, col: u32) -> f64 {
        self.custom_col_widths
            .get(&col)
            .copied()
            .unwrap_or(self.default_col_width)
    }

    pub fn row_offset_y(&self, target_row: u32) -> f64 {
        if self.custom_row_heights.is_empty() {
            (target_row as f64) * self.default_row_height
        } else {
            let mut y = 0.0;
            let mut last_row = 0;
            for (&r, &h) in &self.custom_row_heights {
                if r >= target_row {
                    break;
                }
                y += ((r - last_row) as f64) * self.default_row_height + h;
                last_row = r + 1;
            }
            if target_row > last_row {
                y += ((target_row - last_row) as f64) * self.default_row_height;
            }
            y
        }
    }

    pub fn col_offset_x(&self, target_col: u32) -> f64 {
        if self.custom_col_widths.is_empty() {
            (target_col as f64) * self.default_col_width
        } else {
            let mut x = 0.0;
            let mut last_col = 0;
            for (&c, &w) in &self.custom_col_widths {
                if c >= target_col {
                    break;
                }
                x += ((c - last_col) as f64) * self.default_col_width + w;
                last_col = c + 1;
            }
            if target_col > last_col {
                x += ((target_col - last_col) as f64) * self.default_col_width;
            }
            x
        }
    }

    pub fn find_row_at_y(&self, y: f64, max_rows: u32) -> u32 {
        if y <= 0.0 {
            return 0;
        }
        if self.custom_row_heights.is_empty() {
            let row = (y / self.default_row_height).floor() as u32;
            return row.min(max_rows.saturating_sub(1));
        }

        // Binary search over [0, max_rows)
        let mut low = 0;
        let mut high = max_rows.saturating_sub(1);
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if self.row_offset_y(mid) <= y {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
        low
    }

    pub fn find_col_at_x(&self, x: f64, max_cols: u32) -> u32 {
        if x <= 0.0 {
            return 0;
        }
        if self.custom_col_widths.is_empty() {
            let col = (x / self.default_col_width).floor() as u32;
            return col.min(max_cols.saturating_sub(1));
        }

        // Binary search over [0, max_cols)
        let mut low = 0;
        let mut high = max_cols.saturating_sub(1);
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if self.col_offset_x(mid) <= x {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
        low
    }

    pub fn find_visible_window(
        &self,
        viewport: &Rect2D,
        max_rows: u32,
        max_cols: u32,
    ) -> (u32, u32, u32, u32) {
        let min_row = self.find_row_at_y(viewport.y.max(0.0), max_rows);
        let max_row = self.find_row_at_y(viewport.bottom().max(0.0), max_rows);
        let min_col = self.find_col_at_x(viewport.x.max(0.0), max_cols);
        let max_col = self.find_col_at_x(viewport.right().max(0.0), max_cols);
        (
            min_row,
            max_row.min(max_rows.saturating_sub(1)),
            min_col,
            max_col.min(max_cols.saturating_sub(1)),
        )
    }
}

/// Configuration for rendering row and column headers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaderRenderConfig {
    pub min_row: u32,
    pub max_row: u32,
    pub min_col: u32,
    pub max_col: u32,
    pub active_row: Option<u32>,
    pub active_col: Option<u32>,
    pub header_width: f64,
    pub header_height: f64,
}

/// Freeze Panes configuration (split rows at top, split columns at left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreezePanesConfig {
    pub split_row: u32,
    pub split_col: u32,
}

/// Grid Viewport Configuration.
#[derive(Debug, Clone)]
pub struct GridViewportConfig {
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub freeze_panes: Option<FreezePanesConfig>,
    pub show_gridlines: bool,
    pub show_headers: bool,
    pub max_rows: u32,
    pub max_cols: u32,
}

impl Default for GridViewportConfig {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            freeze_panes: None,
            show_gridlines: true,
            show_headers: true,
            max_rows: MAX_EXCEL_ROWS,
            max_cols: MAX_EXCEL_COLS,
        }
    }
}

/// Formatted spreadsheet cell value and visual attributes.
#[derive(Debug, Clone)]
pub struct ExcelDisplayCell {
    pub row: u32,
    pub col: u32,
    pub formatted_text: String,
    pub fill_color: Option<ColorRgba>,
    pub text_color: ColorRgba,
    pub font_size_px: f32,
}

/// Active selection range in the worksheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExcelSelectionRange {
    pub min_row: u32,
    pub min_col: u32,
    pub max_row: u32,
    pub max_col: u32,
}

impl ExcelSelectionRange {
    pub fn single(row: u32, col: u32) -> Self {
        Self {
            min_row: row,
            min_col: col,
            max_row: row,
            max_col: col,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min_row <= other.max_row
            && self.max_row >= other.min_row
            && self.min_col <= other.max_col
            && self.max_col >= other.min_col
    }

    pub fn to_a1_string(&self) -> String {
        if self.min_row == self.max_row && self.min_col == self.max_col {
            indices_to_cell_coord(self.min_row, self.min_col)
        } else {
            format!(
                "{}:{}",
                indices_to_cell_coord(self.min_row, self.min_col),
                indices_to_cell_coord(self.max_row, self.max_col)
            )
        }
    }
}

pub struct ExcelDisplayCompiler;

impl ExcelDisplayCompiler {
    /// Binary search or direct indexing of visible row and column ranges for a uniform grid.
    pub fn find_visible_window(
        row_height: f64,
        col_width: f64,
        viewport: &Rect2D,
        max_rows: u32,
        max_cols: u32,
    ) -> (u32, u32, u32, u32) {
        if row_height <= 0.0 || col_width <= 0.0 {
            return (0, 0, 0, 0);
        }

        let min_col = ((viewport.x / col_width).floor() as i64)
            .clamp(0, max_cols.saturating_sub(1) as i64) as u32;
        let max_col = (((viewport.right() / col_width).ceil() as i64)
            .clamp(0, max_cols.saturating_sub(1) as i64)) as u32;

        let min_row = ((viewport.y / row_height).floor() as i64)
            .clamp(0, max_rows.saturating_sub(1) as i64) as u32;
        let max_row = (((viewport.bottom() / row_height).ceil() as i64)
            .clamp(0, max_rows.saturating_sub(1) as i64)) as u32;

        (min_row, max_row, min_col, max_col)
    }

    /// Compile visible cells, grid lines, and selection into a `DisplayList`.
    pub fn compile_grid(
        row_height: f64,
        col_width: f64,
        cells: &[ExcelDisplayCell],
        selection: Option<&ExcelSelectionRange>,
        viewport: &Rect2D,
        max_rows: u32,
        max_cols: u32,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();
        let (min_row, max_row, min_col, max_col) =
            Self::find_visible_window(row_height, col_width, viewport, max_rows, max_cols);

        // 1. Grid background (pure white)
        let visible_grid_bounds = Rect2D::new(
            (min_col as f64) * col_width,
            (min_row as f64) * row_height,
            ((max_col.saturating_sub(min_col).saturating_add(1)) as f64) * col_width,
            ((max_row.saturating_sub(min_row).saturating_add(1)) as f64) * row_height,
        );
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            visible_grid_bounds,
            ColorRgba::WHITE,
        )));

        // 2. Cell background fills & text
        for cell in cells {
            if cell.row >= min_row
                && cell.row <= max_row
                && cell.col >= min_col
                && cell.col <= max_col
            {
                let cell_rect = Rect2D::new(
                    (cell.col as f64) * col_width,
                    (cell.row as f64) * row_height,
                    col_width,
                    row_height,
                );

                if let Some(fill) = cell.fill_color {
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                        cell_rect, fill,
                    )));
                }

                if !cell.formatted_text.is_empty() {
                    let run = ShapedTextEngine::shape_text(
                        &cell.formatted_text,
                        "Segoe UI",
                        cell.font_size_px,
                        cell.text_color,
                        (cell_rect.x + 4.0, cell_rect.y + 3.0),
                    );
                    display_list.push(DisplayCommand::DrawText(run));
                }
            }
        }

        // 3. Instanced horizontal grid lines
        let grid_color = ColorRgba::new(0.85, 0.85, 0.85, 1.0); // #D4D4D4
        let grid_w = visible_grid_bounds.width;
        let start_x = visible_grid_bounds.x;

        for r in min_row..=max_row.saturating_add(1) {
            let y = (r as f64) * row_height;
            display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                Rect2D::new(start_x, y, grid_w, 1.0),
                grid_color,
            )));
        }

        // 4. Instanced vertical grid lines
        let grid_h = visible_grid_bounds.height;
        let start_y = visible_grid_bounds.y;

        for c in min_col..=max_col.saturating_add(1) {
            let x = (c as f64) * col_width;
            display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                Rect2D::new(x, start_y, 1.0, grid_h),
                grid_color,
            )));
        }

        // 5. Active selection highlight
        if let Some(sel) = selection {
            let sel_x = (sel.min_col as f64) * col_width;
            let sel_y = (sel.min_row as f64) * row_height;
            let sel_w =
                ((sel.max_col.saturating_sub(sel.min_col).saturating_add(1)) as f64) * col_width;
            let sel_h =
                ((sel.max_row.saturating_sub(sel.min_row).saturating_add(1)) as f64) * row_height;

            let sel_rect = Rect2D::new(sel_x, sel_y, sel_w, sel_h);
            if sel_rect.intersects(viewport) {
                let fill = ColorRgba::new(0.063, 0.486, 0.255, 0.18);
                let border = ColorRgba::new(0.063, 0.486, 0.255, 1.0);
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                    sel_rect, fill, border, 2.0,
                )));

                // Bottom-right Fill Handle (6x6 square)
                let handle_rect =
                    Rect2D::new(sel_rect.right() - 3.0, sel_rect.bottom() - 3.0, 6.0, 6.0);
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    handle_rect,
                    border,
                )));
            }
        }

        display_list
    }

    /// Compile row and column headers for the visible window.
    pub fn compile_headers(dims: &GridDimensionMap, config: &HeaderRenderConfig) -> DisplayList {
        let mut list = DisplayList::new();
        let header_bg = ColorRgba::new(0.953, 0.949, 0.945, 1.0); // #F3F2F1
        let header_border = ColorRgba::new(0.824, 0.816, 0.808, 1.0); // #D2D0CE
        let active_bg = ColorRgba::new(0.792, 0.918, 0.847, 1.0); // #CAEAD8
        let text_color = ColorRgba::new(0.286, 0.278, 0.271, 1.0);
        let active_text = ColorRgba::new(0.063, 0.486, 0.255, 1.0);

        // 1. Top-Left Select All Corner
        list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
            Rect2D::new(0.0, 0.0, config.header_width, config.header_height),
            header_bg,
            header_border,
            1.0,
        )));

        // 2. Column Headers
        for c in config.min_col..=config.max_col {
            let col_x = dims.col_offset_x(c);
            let col_w = dims.col_width(c);
            let is_active = config.active_col == Some(c);
            let bg = if is_active { active_bg } else { header_bg };
            let fg = if is_active { active_text } else { text_color };

            list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                Rect2D::new(
                    col_x + config.header_width,
                    0.0,
                    col_w,
                    config.header_height,
                ),
                bg,
                header_border,
                1.0,
            )));

            let name = col_to_name(c);
            let run = ShapedTextEngine::shape_text(
                &name,
                "Segoe UI",
                11.0,
                fg,
                (col_x + config.header_width + col_w / 2.0 - 4.0, 6.0),
            );
            list.push(DisplayCommand::DrawText(run));
        }

        // 3. Row Headers
        for r in config.min_row..=config.max_row {
            let row_y = dims.row_offset_y(r);
            let row_h = dims.row_height(r);
            let is_active = config.active_row == Some(r);
            let bg = if is_active { active_bg } else { header_bg };
            let fg = if is_active { active_text } else { text_color };

            list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                Rect2D::new(
                    0.0,
                    row_y + config.header_height,
                    config.header_width,
                    row_h,
                ),
                bg,
                header_border,
                1.0,
            )));

            let num = format!("{}", r.saturating_add(1));
            let run = ShapedTextEngine::shape_text(
                &num,
                "Segoe UI",
                11.0,
                fg,
                (
                    config.header_width / 2.0 - 6.0,
                    row_y + config.header_height + 4.0,
                ),
            );
            list.push(DisplayCommand::DrawText(run));
        }

        list
    }

    /// High-performance compilation of virtualized grid viewport supporting freeze panes and custom dimensions.
    pub fn compile_virtualized_viewport(
        config: &GridViewportConfig,
        dims: &GridDimensionMap,
        cells: &[ExcelDisplayCell],
        selection: Option<&ExcelSelectionRange>,
    ) -> DisplayList {
        let mut list = DisplayList::new();
        let header_w = if config.show_headers {
            ROW_HEADER_WIDTH
        } else {
            0.0
        };
        let header_h = if config.show_headers {
            COL_HEADER_HEIGHT
        } else {
            0.0
        };

        let content_w = (config.viewport_width - header_w).max(10.0);
        let content_h = (config.viewport_height - header_h).max(10.0);

        let viewport_rect = Rect2D::new(config.scroll_x, config.scroll_y, content_w, content_h);
        let (min_row, max_row, min_col, max_col) =
            dims.find_visible_window(&viewport_rect, config.max_rows, config.max_cols);

        // 1. Grid Background
        list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            Rect2D::new(header_w, header_h, content_w, content_h),
            ColorRgba::WHITE,
        )));

        // 2. Visible Cells
        for cell in cells {
            if cell.row >= min_row
                && cell.row <= max_row
                && cell.col >= min_col
                && cell.col <= max_col
            {
                let cell_x = dims.col_offset_x(cell.col) - config.scroll_x + header_w;
                let cell_y = dims.row_offset_y(cell.row) - config.scroll_y + header_h;
                let cell_w = dims.col_width(cell.col);
                let cell_h = dims.row_height(cell.row);

                let cell_rect = Rect2D::new(cell_x, cell_y, cell_w, cell_h);
                if let Some(fill) = cell.fill_color {
                    list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                        cell_rect, fill,
                    )));
                }

                if !cell.formatted_text.is_empty() {
                    let run = ShapedTextEngine::shape_text(
                        &cell.formatted_text,
                        "Segoe UI",
                        cell.font_size_px,
                        cell.text_color,
                        (cell_rect.x + 4.0, cell_rect.y + 3.0),
                    );
                    list.push(DisplayCommand::DrawText(run));
                }
            }
        }

        // 3. Instanced Gridlines
        if config.show_gridlines {
            let grid_color = ColorRgba::new(0.85, 0.85, 0.85, 1.0);
            for r in min_row..=max_row.saturating_add(1) {
                let y = dims.row_offset_y(r) - config.scroll_y + header_h;
                if y >= header_h && y <= config.viewport_height {
                    list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                        Rect2D::new(header_w, y, content_w, 1.0),
                        grid_color,
                    )));
                }
            }
            for c in min_col..=max_col.saturating_add(1) {
                let x = dims.col_offset_x(c) - config.scroll_x + header_w;
                if x >= header_w && x <= config.viewport_width {
                    list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                        Rect2D::new(x, header_h, 1.0, content_h),
                        grid_color,
                    )));
                }
            }
        }

        // 4. Selection Highlight
        if let Some(sel) = selection {
            let sel_x = dims.col_offset_x(sel.min_col) - config.scroll_x + header_w;
            let sel_y = dims.row_offset_y(sel.min_row) - config.scroll_y + header_h;
            let sel_w =
                dims.col_offset_x(sel.max_col.saturating_add(1)) - dims.col_offset_x(sel.min_col);
            let sel_h =
                dims.row_offset_y(sel.max_row.saturating_add(1)) - dims.row_offset_y(sel.min_row);

            let sel_rect = Rect2D::new(sel_x, sel_y, sel_w, sel_h);
            let fill = ColorRgba::new(0.063, 0.486, 0.255, 0.18);
            let border = ColorRgba::new(0.063, 0.486, 0.255, 1.0);
            list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                sel_rect, fill, border, 2.0,
            )));

            let handle_rect =
                Rect2D::new(sel_rect.right() - 3.0, sel_rect.bottom() - 3.0, 6.0, 6.0);
            list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                handle_rect,
                border,
            )));
        }

        // 5. Freeze Pane Dividers
        if let Some(fp) = config.freeze_panes {
            let freeze_line_color = ColorRgba::new(0.4, 0.4, 0.4, 1.0);
            if fp.split_row > 0 {
                let y = dims.row_offset_y(fp.split_row) + header_h;
                list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    Rect2D::new(0.0, y, config.viewport_width, 2.0),
                    freeze_line_color,
                )));
            }
            if fp.split_col > 0 {
                let x = dims.col_offset_x(fp.split_col) + header_w;
                list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    Rect2D::new(x, 0.0, 2.0, config.viewport_height),
                    freeze_line_color,
                )));
            }
        }

        // 6. Headers
        if config.show_headers {
            let header_cfg = HeaderRenderConfig {
                min_row,
                max_row,
                min_col,
                max_col,
                active_row: selection.map(|s| s.min_row),
                active_col: selection.map(|s| s.min_col),
                header_width: header_w,
                header_height: header_h,
            };
            let header_list = Self::compile_headers(dims, &header_cfg);
            for cmd in header_list.commands {
                list.push(cmd);
            }
        }

        list
    }

    /// Compile a dashed or bordered auto-fill preview rectangle around projected cells.
    pub fn compile_autofill_preview(
        target: &ExcelSelectionRange,
        dims: &GridDimensionMap,
        scroll_x: f64,
        scroll_y: f64,
        header_w: f64,
        header_h: f64,
    ) -> DisplayList {
        let mut list = DisplayList::new();
        let x = dims.col_offset_x(target.min_col) - scroll_x + header_w;
        let y = dims.row_offset_y(target.min_row) - scroll_y + header_h;
        let w =
            dims.col_offset_x(target.max_col.saturating_add(1)) - dims.col_offset_x(target.min_col);
        let h =
            dims.row_offset_y(target.max_row.saturating_add(1)) - dims.row_offset_y(target.min_row);

        let rect = Rect2D::new(x, y, w, h);
        let border = ColorRgba::new(0.4, 0.4, 0.4, 0.9);
        let fill = ColorRgba::new(0.0, 0.0, 0.0, 0.04);
        list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
            rect, fill, border, 1.5,
        )));
        list
    }

    /// Compile colored visual boundaries for cell/range references in an active formula.
    pub fn compile_formula_references(
        references: &[(ExcelSelectionRange, ColorRgba)],
        dims: &GridDimensionMap,
        scroll_x: f64,
        scroll_y: f64,
        header_w: f64,
        header_h: f64,
    ) -> DisplayList {
        let mut list = DisplayList::new();
        for (range, color) in references {
            let x = dims.col_offset_x(range.min_col) - scroll_x + header_w;
            let y = dims.row_offset_y(range.min_row) - scroll_y + header_h;
            let w = dims.col_offset_x(range.max_col.saturating_add(1))
                - dims.col_offset_x(range.min_col);
            let h = dims.row_offset_y(range.max_row.saturating_add(1))
                - dims.row_offset_y(range.min_row);

            let rect = Rect2D::new(x, y, w, h);
            let fill = ColorRgba::new(color.r, color.g, color.b, 0.12);
            list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                rect, fill, *color, 2.0,
            )));
        }
        list
    }

    /// Compile multi-range selections including non-contiguous ranges and the primary fill handle.
    pub fn compile_multi_selection(
        selection: &MultiRangeSelectionModel,
        dims: &GridDimensionMap,
        scroll_x: f64,
        scroll_y: f64,
        header_w: f64,
        header_h: f64,
    ) -> DisplayList {
        let mut list = DisplayList::new();
        let fill = ColorRgba::new(0.063, 0.486, 0.255, 0.18);
        let border = ColorRgba::new(0.063, 0.486, 0.255, 1.0);

        for (idx, sel) in selection.ranges.iter().enumerate() {
            let sel_x = dims.col_offset_x(sel.min_col) - scroll_x + header_w;
            let sel_y = dims.row_offset_y(sel.min_row) - scroll_y + header_h;
            let sel_w =
                dims.col_offset_x(sel.max_col.saturating_add(1)) - dims.col_offset_x(sel.min_col);
            let sel_h =
                dims.row_offset_y(sel.max_row.saturating_add(1)) - dims.row_offset_y(sel.min_row);

            let sel_rect = Rect2D::new(sel_x, sel_y, sel_w, sel_h);
            list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                sel_rect, fill, border, 2.0,
            )));

            // Fill handle only on primary (last) range
            if idx == selection.ranges.len().saturating_sub(1) {
                let handle_rect =
                    Rect2D::new(sel_rect.right() - 3.0, sel_rect.bottom() - 3.0, 6.0, 6.0);
                list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    handle_rect,
                    border,
                )));
            }
        }
        list
    }
}

/// Multi-Range Cell Selection Model supporting contiguous and disjoint ranges.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiRangeSelectionModel {
    pub ranges: Vec<ExcelSelectionRange>,
    pub active_cell: (u32, u32),
    pub anchor_cell: (u32, u32),
}

impl Default for MultiRangeSelectionModel {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl MultiRangeSelectionModel {
    pub fn new(row: u32, col: u32) -> Self {
        Self {
            ranges: vec![ExcelSelectionRange::single(row, col)],
            active_cell: (row, col),
            anchor_cell: (row, col),
        }
    }

    pub fn select_single(&mut self, row: u32, col: u32) {
        self.ranges.clear();
        self.ranges.push(ExcelSelectionRange::single(row, col));
        self.active_cell = (row, col);
        self.anchor_cell = (row, col);
    }

    pub fn extend_selection(&mut self, row: u32, col: u32) {
        let (ar, ac) = self.anchor_cell;
        let extended = ExcelSelectionRange {
            min_row: ar.min(row),
            min_col: ac.min(col),
            max_row: ar.max(row),
            max_col: ac.max(col),
        };
        if let Some(last) = self.ranges.last_mut() {
            *last = extended;
        } else {
            self.ranges.push(extended);
        }
        self.active_cell = (row, col);
    }

    pub fn add_range(&mut self, range: ExcelSelectionRange) {
        self.ranges.push(range);
        self.active_cell = (range.min_row, range.min_col);
        self.anchor_cell = (range.min_row, range.min_col);
    }

    pub fn select_row(&mut self, row: u32, max_cols: u32) {
        let range = ExcelSelectionRange {
            min_row: row,
            min_col: 0,
            max_row: row,
            max_col: max_cols.saturating_sub(1),
        };
        self.ranges.clear();
        self.ranges.push(range);
        self.active_cell = (row, 0);
        self.anchor_cell = (row, 0);
    }

    pub fn select_col(&mut self, col: u32, max_rows: u32) {
        let range = ExcelSelectionRange {
            min_row: 0,
            min_col: col,
            max_row: max_rows.saturating_sub(1),
            max_col: col,
        };
        self.ranges.clear();
        self.ranges.push(range);
        self.active_cell = (0, col);
        self.anchor_cell = (0, col);
    }

    pub fn select_all(&mut self, max_rows: u32, max_cols: u32) {
        let range = ExcelSelectionRange {
            min_row: 0,
            min_col: 0,
            max_row: max_rows.saturating_sub(1),
            max_col: max_cols.saturating_sub(1),
        };
        self.ranges.clear();
        self.ranges.push(range);
        self.active_cell = (0, 0);
        self.anchor_cell = (0, 0);
    }

    pub fn is_selected(&self, row: u32, col: u32) -> bool {
        self.ranges
            .iter()
            .any(|r| row >= r.min_row && row <= r.max_row && col >= r.min_col && col <= r.max_col)
    }

    pub fn primary_range(&self) -> Option<&ExcelSelectionRange> {
        self.ranges.last()
    }

    pub fn bounding_box(&self) -> Option<ExcelSelectionRange> {
        if self.ranges.is_empty() {
            return None;
        }
        let mut min_row = u32::MAX;
        let mut min_col = u32::MAX;
        let mut max_row = 0;
        let mut max_col = 0;
        for r in &self.ranges {
            min_row = min_row.min(r.min_row);
            min_col = min_col.min(r.min_col);
            max_row = max_row.max(r.max_row);
            max_col = max_col.max(r.max_col);
        }
        Some(ExcelSelectionRange {
            min_row,
            min_col,
            max_row,
            max_col,
        })
    }

    pub fn total_cell_count(&self) -> u64 {
        self.ranges
            .iter()
            .map(|r| {
                ((r.max_row.saturating_sub(r.min_row).saturating_add(1)) as u64)
                    * ((r.max_col.saturating_sub(r.min_col).saturating_add(1)) as u64)
            })
            .sum()
    }

    pub fn to_range_string(&self) -> String {
        self.ranges
            .iter()
            .map(|r| r.to_a1_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Auto-Fill and series prediction engine.
pub struct AutoFillEngine;

impl AutoFillEngine {
    /// Translates a single cell reference or range reference (e.g. "A1", "$A$1", "A1:B10")
    /// by row_delta and col_delta, keeping '$' fixed dimensions unchanged.
    pub fn translate_reference(ref_str: &str, row_delta: i32, col_delta: i32) -> String {
        if let Some((left, right)) = ref_str.split_once(':') {
            let new_left = Self::translate_reference(left, row_delta, col_delta);
            let new_right = Self::translate_reference(right, row_delta, col_delta);
            return format!("{}:{}", new_left, new_right);
        }

        let bytes = ref_str.as_bytes();
        if bytes.is_empty() {
            return ref_str.to_string();
        }

        let mut i = 0;
        let col_dollar = if bytes[i] == b'$' {
            i += 1;
            true
        } else {
            false
        };

        let col_start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        if i == col_start {
            return ref_str.to_string();
        }

        let col_name = &ref_str[col_start..i];
        let col_idx = match name_to_col(col_name) {
            Some(c) => c,
            None => return ref_str.to_string(),
        };

        if i >= bytes.len() {
            return ref_str.to_string();
        }

        let row_dollar = if bytes[i] == b'$' {
            i += 1;
            true
        } else {
            false
        };

        let row_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == row_start || i != bytes.len() {
            return ref_str.to_string();
        }

        let row_num: u32 = match ref_str[row_start..].parse() {
            Ok(n) => n,
            Err(_) => return ref_str.to_string(),
        };
        if row_num == 0 || row_num > MAX_EXCEL_ROWS {
            return ref_str.to_string();
        }
        let row_idx = row_num - 1;

        let new_col = if col_dollar {
            col_idx
        } else {
            let shifted = (col_idx as i32) + col_delta;
            shifted.clamp(0, (MAX_EXCEL_COLS - 1) as i32) as u32
        };

        let new_row = if row_dollar {
            row_idx
        } else {
            let shifted = (row_idx as i32) + row_delta;
            shifted.clamp(0, (MAX_EXCEL_ROWS - 1) as i32) as u32
        };

        let col_prefix = if col_dollar { "$" } else { "" };
        let row_prefix = if row_dollar { "$" } else { "" };
        format!(
            "{}{}{}{}",
            col_prefix,
            col_to_name(new_col),
            row_prefix,
            new_row.saturating_add(1)
        )
    }

    /// Adjusts all relative cell and range references in a formula string.
    pub fn translate_formula(formula: &str, row_delta: i32, col_delta: i32) -> String {
        if !formula.starts_with('=') || (row_delta == 0 && col_delta == 0) {
            return formula.to_string();
        }

        let tokens = crate::shell::FormulaBarEngine::tokenize_formula(formula);
        let mut result = String::with_capacity(formula.len() + 16);
        result.push('=');
        for token in tokens {
            if token.token_type == crate::shell::FormulaTokenType::Reference {
                result.push_str(&Self::translate_reference(
                    &token.value,
                    row_delta,
                    col_delta,
                ));
            } else {
                result.push_str(&token.value);
            }
        }
        result
    }

    /// Predicts the next `count` values in an auto-fill series.
    pub fn predict_series(samples: &[String], count: usize, is_horizontal: bool) -> Vec<String> {
        if samples.is_empty() || count == 0 {
            return Vec::new();
        }

        // 1. Check if all samples are formulas
        if samples.iter().all(|s| s.starts_with('=')) {
            let mut out = Vec::with_capacity(count);
            for k in 1..=count {
                let (row_delta, col_delta) = if is_horizontal {
                    (0, k as i32)
                } else {
                    (k as i32, 0)
                };
                let src_idx = (k - 1) % samples.len();
                out.push(Self::translate_formula(
                    &samples[src_idx],
                    row_delta,
                    col_delta,
                ));
            }
            return out;
        }

        // 2. Check if all samples are numbers
        let numbers: Option<Vec<f64>> = samples
            .iter()
            .map(|s| s.trim().parse::<f64>().ok())
            .collect();
        if let Some(nums) = numbers {
            let mut out = Vec::with_capacity(count);
            if nums.len() == 1 {
                let val = &samples[0];
                for _ in 0..count {
                    out.push(val.clone());
                }
            } else {
                let n = nums.len();
                let step = (nums[n - 1] - nums[0]) / ((n - 1) as f64);
                let is_integer = nums.iter().all(|x| x.fract() == 0.0) && step.fract() == 0.0;
                for k in 1..=count {
                    let next_val = nums[n - 1] + step * (k as f64);
                    if is_integer {
                        out.push(format!("{:.0}", next_val));
                    } else {
                        out.push(format!("{:.2}", next_val));
                    }
                }
            }
            return out;
        }

        // 3. Check alphanumeric pattern (e.g. "Day 1", "Q1", "Item 005")
        let alpha_num_pattern = samples.iter().all(|s| {
            let trimmed = s.trim();
            trimmed.chars().last().is_some_and(|c| c.is_ascii_digit())
        });

        if alpha_num_pattern {
            let last_s = samples.last().unwrap_or(&samples[0]);
            let trimmed = last_s.trim();
            let num_start = trimmed
                .rfind(|c: char| !c.is_ascii_digit())
                .map(|i| i + 1)
                .unwrap_or(0);
            let prefix = &trimmed[..num_start];
            let num_str = &trimmed[num_start..];
            let num_len = num_str.len();

            if let Ok(last_num) = num_str.parse::<i64>() {
                let step = if samples.len() >= 2 {
                    let first_s = &samples[0];
                    let first_start = first_s
                        .trim()
                        .rfind(|c: char| !c.is_ascii_digit())
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let first_num = first_s[first_start..].parse::<i64>().unwrap_or(last_num);
                    (last_num - first_num) / ((samples.len() - 1) as i64)
                } else {
                    1
                };

                let mut out = Vec::with_capacity(count);
                for k in 1..=count {
                    let next_num = last_num + step * (k as i64);
                    out.push(format!("{}{:0width$}", prefix, next_num, width = num_len));
                }
                return out;
            }
        }

        // 4. Default: cycle through samples
        let mut out = Vec::with_capacity(count);
        for k in 1..=count {
            let idx = (k - 1) % samples.len();
            out.push(samples[idx].clone());
        }
        out
    }

    /// Computes the auto-fill target range based on mouse drag coordinates.
    pub fn compute_autofill_target(
        source: &ExcelSelectionRange,
        drag_cell: (u32, u32),
    ) -> (ExcelSelectionRange, bool) {
        let (dr, dc) = drag_cell;
        let row_diff = if dr > source.max_row {
            dr - source.max_row
        } else {
            source.min_row.saturating_sub(dr)
        };

        let col_diff = if dc > source.max_col {
            dc - source.max_col
        } else {
            source.min_col.saturating_sub(dc)
        };

        if col_diff > row_diff {
            // Horizontal fill
            let max_c = dc.max(source.max_col);
            (
                ExcelSelectionRange {
                    min_row: source.min_row,
                    max_row: source.max_row,
                    min_col: source.min_col,
                    max_col: max_c,
                },
                true,
            )
        } else {
            // Vertical fill
            let max_r = dr.max(source.max_row);
            (
                ExcelSelectionRange {
                    min_row: source.min_row,
                    max_row: max_r,
                    min_col: source.min_col,
                    max_col: source.max_col,
                },
                false,
            )
        }
    }
}
