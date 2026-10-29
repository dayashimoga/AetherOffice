//! Excel Display List Compiler
//!
//! Provides binary-search viewport indexing for multi-million cell worksheets,
//! compiling only visible cell background fills, instanced grid lines, cell text,
//! and selection highlight quads.

use crate::primitive::{ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D};
use crate::text::ShapedTextEngine;

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

pub struct ExcelDisplayCompiler;

impl ExcelDisplayCompiler {
    /// Binary search or direct indexing of visible row and column ranges for a uniform or styled grid.
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

        let min_col = ((viewport.x / col_width).floor() as i64).clamp(0, max_cols as i64) as u32;
        let max_col =
            (((viewport.right() / col_width).ceil() as i64).clamp(0, max_cols as i64)) as u32;

        let min_row = ((viewport.y / row_height).floor() as i64).clamp(0, max_rows as i64) as u32;
        let max_row =
            (((viewport.bottom() / row_height).ceil() as i64).clamp(0, max_rows as i64)) as u32;

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
            ((max_col - min_col) as f64) * col_width,
            ((max_row - min_row) as f64) * row_height,
        );
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            visible_grid_bounds,
            ColorRgba::WHITE,
        )));

        // 2. Cell background fills
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

                // 3. Cell formatted text
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

        // 4. Instanced horizontal grid lines
        let grid_color = ColorRgba::new(0.85, 0.85, 0.85, 1.0); // Excel default gridline #D4D4D4
        let grid_w = visible_grid_bounds.width;
        let start_x = visible_grid_bounds.x;

        for r in min_row..=max_row {
            let y = (r as f64) * row_height;
            display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                Rect2D::new(start_x, y, grid_w, 1.0),
                grid_color,
            )));
        }

        // 5. Instanced vertical grid lines
        let grid_h = visible_grid_bounds.height;
        let start_y = visible_grid_bounds.y;

        for c in min_col..=max_col {
            let x = (c as f64) * col_width;
            display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                Rect2D::new(x, start_y, 1.0, grid_h),
                grid_color,
            )));
        }

        // 6. Active selection highlight
        if let Some(sel) = selection {
            let sel_x = (sel.min_col as f64) * col_width;
            let sel_y = (sel.min_row as f64) * row_height;
            let sel_w = ((sel.max_col - sel.min_col + 1) as f64) * col_width;
            let sel_h = ((sel.max_row - sel.min_row + 1) as f64) * row_height;

            let sel_rect = Rect2D::new(sel_x, sel_y, sel_w, sel_h);
            if sel_rect.intersects(viewport) {
                // Semi-transparent Excel green overlay (#107C4133) with 2px stroke
                let fill = ColorRgba::new(0.063, 0.486, 0.255, 0.20);
                let border = ColorRgba::new(0.063, 0.486, 0.255, 1.0);
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                    sel_rect, fill, border, 2.0,
                )));
            }
        }

        display_list
    }
}
