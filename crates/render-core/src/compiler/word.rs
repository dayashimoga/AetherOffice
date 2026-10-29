//! Word Display List Compiler
//!
//! Compiles Word document pages, text runs, and tables into tile-virtualized GPU display lists,
//! culling offscreen pages outside the active viewport.

use crate::primitive::{ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D};
use crate::text::ShapedTextEngine;

/// Formatted text line on a Word document page.
#[derive(Debug, Clone)]
pub struct WordDisplayLine {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size_px: f32,
    pub color: ColorRgba,
}

/// Table cell on a Word document page.
#[derive(Debug, Clone)]
pub struct WordDisplayTableCell {
    pub bounds: Rect2D,
    pub fill_color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub text: Option<String>,
}

/// Virtualized Word page representation.
#[derive(Debug, Clone)]
pub struct WordDisplayPage {
    pub page_number: usize,
    pub bounds: Rect2D,
    pub lines: Vec<WordDisplayLine>,
    pub table_cells: Vec<WordDisplayTableCell>,
}

pub struct WordDisplayCompiler;

impl WordDisplayCompiler {
    /// Compile a multi-page Word document into a display list for the current viewport.
    pub fn compile_document(
        pages: &[WordDisplayPage],
        viewport: &Rect2D,
        page_gap: f64,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();

        let mut curr_y = 40.0; // Top margin
        for page in pages {
            let page_bounds =
                Rect2D::new(page.bounds.x, curr_y, page.bounds.width, page.bounds.height);

            // Viewport culling: only compile pages that intersect the viewport
            if page_bounds.intersects(viewport) {
                // 1. Page drop shadow (5px offset, semi-transparent black)
                let shadow_bounds = Rect2D::new(
                    page_bounds.x + 4.0,
                    page_bounds.y + 4.0,
                    page_bounds.width,
                    page_bounds.height,
                );
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    shadow_bounds,
                    ColorRgba::new(0.0, 0.0, 0.0, 0.15),
                )));

                // 2. White paper canvas background with subtle border
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                    page_bounds,
                    ColorRgba::WHITE,
                    ColorRgba::new(0.8, 0.8, 0.8, 1.0),
                    1.0,
                )));

                // 3. Table cells (fills, borders, text)
                for cell in &page.table_cells {
                    let cell_bounds = Rect2D::new(
                        page_bounds.x + cell.bounds.x,
                        page_bounds.y + cell.bounds.y,
                        cell.bounds.width,
                        cell.bounds.height,
                    );

                    if cell_bounds.intersects(viewport) {
                        let fill = cell.fill_color.unwrap_or(ColorRgba::WHITE);
                        let border = cell
                            .border_color
                            .unwrap_or(ColorRgba::new(0.85, 0.85, 0.85, 1.0));
                        display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                            cell_bounds,
                            fill,
                            border,
                            cell.border_width.max(1.0),
                        )));

                        if let Some(ref text) = cell.text {
                            let run = ShapedTextEngine::shape_text(
                                text,
                                "Calibri",
                                14.0,
                                ColorRgba::BLACK,
                                (cell_bounds.x + 6.0, cell_bounds.y + 6.0),
                            );
                            display_list.push(DisplayCommand::DrawText(run));
                        }
                    }
                }

                // 4. Text lines
                for line in &page.lines {
                    let line_origin = (page_bounds.x + line.x, page_bounds.y + line.y);
                    let line_rect = Rect2D::new(
                        line_origin.0,
                        line_origin.1,
                        (line.text.len() as f64) * (line.font_size_px as f64) * 0.55,
                        line.font_size_px as f64 * 1.25,
                    );

                    if line_rect.intersects(viewport) {
                        let run = ShapedTextEngine::shape_text(
                            &line.text,
                            "Calibri",
                            line.font_size_px,
                            line.color,
                            line_origin,
                        );
                        display_list.push(DisplayCommand::DrawText(run));
                    }
                }
            }

            curr_y += page.bounds.height + page_gap;
        }

        display_list
    }
}
