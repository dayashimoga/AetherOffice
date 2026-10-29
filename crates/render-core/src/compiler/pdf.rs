//! PDF Multi-Page Virtualized Canvas Display Compiler, Viewport & Text Selection
//!
//! Subphase 5.3.1 Deliverable:
//! - Tile-virtualized multi-page PDF canvas with 120fps viewport culling (< 8.33 ms frame budget).
//! - Continuous vertical scrolling, facing pages (two-page spread), and single-page layout modes.
//! - Smooth zoom scaling (25% - 500%), zoom-to-fit-width, zoom-to-fit-page, and cursor-anchored zooming.
//! - Tile-based LOD (Level of Detail) decomposition and bounding box intersection culling.
//! - Bidirectional coordinate transformations (Screen <-> Document <-> PDF User Space with Y-inversion).
//! - Interactive text selection overlay with character-level hit-testing and clipboard text extraction.
//! - Integrated annotation, highlight, and form field display list generation.

use crate::primitive::{
    ColorRgba, DisplayCommand, DisplayList, GlyphInstance, GlyphRun, QuadInstance, Rect2D,
    TextScript,
};
use pdf_engine::acroform::{FormField, FormFieldType};
use pdf_engine::annotation::{AnnotationSubtype, PdfAnnotation};
use pdf_engine::text::ExtractedPageText;
use pdf_engine::types::{PdfObjectId, PdfRect};
use serde::{Deserialize, Serialize};

// =========================================================================
// 1. Viewport Configuration & Layout Modes
// =========================================================================

/// Multi-page layout modes for the PDF canvas viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PdfPageLayoutMode {
    /// Continuous vertical column of pages centered horizontally.
    #[default]
    Continuous,
    /// Facing pages (two-page spread / book view) with a central gutter.
    FacingPages,
    /// Single page view focusing on the active page.
    SinglePage,
}

/// Zoom level constants and limits.
pub const MIN_PDF_ZOOM: f64 = 0.25; // 25%
pub const MAX_PDF_ZOOM: f64 = 5.00; // 500%
pub const DEFAULT_PAGE_GAP: f64 = 16.0; // 16 pt / px gap between pages
pub const DEFAULT_TILE_SIZE: f64 = 512.0; // 512x512 tile grid for virtualization

/// Discrete zoom preset ladder.
pub const ZOOM_PRESETS: &[f64] = &[0.25, 0.50, 0.75, 1.00, 1.25, 1.50, 2.00, 3.00, 4.00, 5.00];

/// Configuration parameters for the PDF canvas viewport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfViewportConfig {
    /// Visible screen viewport rectangle (e.g. [0, 0, width, height]).
    pub viewport: Rect2D,
    /// Horizontal scroll offset in pixels.
    pub scroll_x: f64,
    /// Vertical scroll offset in pixels.
    pub scroll_y: f64,
    /// Zoom factor clamped between `[MIN_PDF_ZOOM, MAX_PDF_ZOOM]`.
    pub zoom: f64,
    /// Spacing gap between adjacent pages in pixels.
    pub page_gap: f64,
    /// Active layout mode.
    pub layout_mode: PdfPageLayoutMode,
    /// Currently focused or active page index (0-based).
    pub active_page: usize,
    /// Tile size in pixels for virtualization.
    pub tile_size: f64,
}

impl Default for PdfViewportConfig {
    fn default() -> Self {
        Self {
            viewport: Rect2D::new(0.0, 0.0, 1280.0, 800.0),
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
            page_gap: DEFAULT_PAGE_GAP,
            layout_mode: PdfPageLayoutMode::Continuous,
            active_page: 0,
            tile_size: DEFAULT_TILE_SIZE,
        }
    }
}

impl PdfViewportConfig {
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            viewport: Rect2D::new(0.0, 0.0, viewport_width, viewport_height),
            ..Default::default()
        }
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom.clamp(MIN_PDF_ZOOM, MAX_PDF_ZOOM);
        self
    }

    pub fn with_scroll(mut self, scroll_x: f64, scroll_y: f64) -> Self {
        self.scroll_x = scroll_x;
        self.scroll_y = scroll_y;
        self
    }

    pub fn with_layout_mode(mut self, mode: PdfPageLayoutMode) -> Self {
        self.layout_mode = mode;
        self
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(MIN_PDF_ZOOM, MAX_PDF_ZOOM);
    }

    /// Zoom in to the next preset ladder step.
    pub fn zoom_in(&mut self) {
        let current = self.zoom;
        for &preset in ZOOM_PRESETS {
            if preset > current + 0.01 {
                self.set_zoom(preset);
                return;
            }
        }
        self.set_zoom(MAX_PDF_ZOOM);
    }

    /// Zoom out to the previous preset ladder step.
    pub fn zoom_out(&mut self) {
        let current = self.zoom;
        for &preset in ZOOM_PRESETS.iter().rev() {
            if preset < current - 0.01 {
                self.set_zoom(preset);
                return;
            }
        }
        self.set_zoom(MIN_PDF_ZOOM);
    }

    /// Zooms around a specific screen anchor point, keeping the content under the anchor stationary.
    pub fn zoom_around_point(&mut self, new_zoom: f64, anchor_x: f64, anchor_y: f64) {
        let clamped_zoom = new_zoom.clamp(MIN_PDF_ZOOM, MAX_PDF_ZOOM);
        if (clamped_zoom - self.zoom).abs() < 1e-6 {
            return;
        }

        // Content point under cursor in document unscaled space:
        let doc_x = (anchor_x + self.scroll_x) / self.zoom;
        let doc_y = (anchor_y + self.scroll_y) / self.zoom;

        self.zoom = clamped_zoom;

        // New scroll offset so that doc_x is still at anchor_x:
        self.scroll_x = (doc_x * self.zoom - anchor_x).max(0.0);
        self.scroll_y = (doc_y * self.zoom - anchor_y).max(0.0);
    }

    /// Computes zoom factor that fits page width snugly into viewport with margins.
    pub fn calculate_zoom_to_fit_width(&self, page_width: f64, margin_px: f64) -> f64 {
        let available_w = (self.viewport.width - margin_px * 2.0).max(100.0);
        (available_w / page_width).clamp(MIN_PDF_ZOOM, MAX_PDF_ZOOM)
    }

    /// Computes zoom factor that fits entire page into viewport with margins.
    pub fn calculate_zoom_to_fit_page(&self, page_w: f64, page_h: f64, margin_px: f64) -> f64 {
        let available_w = (self.viewport.width - margin_px * 2.0).max(100.0);
        let available_h = (self.viewport.height - margin_px * 2.0).max(100.0);
        let zoom_w = available_w / page_w;
        let zoom_h = available_h / page_h;
        zoom_w.min(zoom_h).clamp(MIN_PDF_ZOOM, MAX_PDF_ZOOM)
    }
}

// =========================================================================
// 2. Multi-Page Layout Calculation & Document Model
// =========================================================================

/// Computed spatial layout for a single PDF page on canvas and screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPageLayout {
    /// 0-based page index.
    pub page_index: usize,
    /// Bounding rectangle in unscaled document canvas space (pt).
    pub unscaled_bounds: Rect2D,
    /// Bounding rectangle in screen pixels (scaled by zoom, translated by scroll and centering).
    pub screen_bounds: Rect2D,
    /// Original PDF media box.
    pub media_box: PdfRect,
    /// Page rotation in degrees (0, 90, 180, 270).
    pub rotation: u16,
}

/// Computed spatial layout for all pages in a PDF document.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PdfDocumentLayout {
    pub pages: Vec<PdfPageLayout>,
    /// Total bounding box of all pages in unscaled document space.
    pub total_document_bounds: Rect2D,
    /// Total bounding box of all pages in screen space (including gaps).
    pub total_screen_bounds: Rect2D,
}

impl PdfDocumentLayout {
    /// Calculates multi-page layout based on page media boxes and viewport config.
    pub fn calculate(pages_media_boxes: &[PdfRect], config: &PdfViewportConfig) -> Self {
        if pages_media_boxes.is_empty() {
            return Self::default();
        }

        match config.layout_mode {
            PdfPageLayoutMode::Continuous => Self::layout_continuous(pages_media_boxes, config),
            PdfPageLayoutMode::FacingPages => Self::layout_facing_pages(pages_media_boxes, config),
            PdfPageLayoutMode::SinglePage => Self::layout_single_page(pages_media_boxes, config),
        }
    }

    fn layout_continuous(boxes: &[PdfRect], config: &PdfViewportConfig) -> Self {
        let mut pages = Vec::with_capacity(boxes.len());
        let mut max_unscaled_w: f64 = 0.0;
        let mut curr_y_unscaled: f64 = config.page_gap;

        // 1. Determine maximum page width to center columns
        for mbox in boxes {
            let w = mbox.width();
            if w > max_unscaled_w {
                max_unscaled_w = w;
            }
        }

        let scaled_max_w = max_unscaled_w * config.zoom;
        let horizontal_centering_offset = if config.viewport.width > scaled_max_w {
            (config.viewport.width - scaled_max_w) / 2.0
        } else {
            config.page_gap
        };

        let mut curr_y_screen: f64 = config.page_gap * config.zoom - config.scroll_y;

        for (idx, mbox) in boxes.iter().enumerate() {
            let w_pt = mbox.width();
            let h_pt = mbox.height();

            let x_unscaled = (max_unscaled_w - w_pt) / 2.0 + config.page_gap;
            let unscaled_bounds = Rect2D::new(x_unscaled, curr_y_unscaled, w_pt, h_pt);

            let w_screen = w_pt * config.zoom;
            let h_screen = h_pt * config.zoom;
            let x_screen =
                horizontal_centering_offset + (scaled_max_w - w_screen) / 2.0 - config.scroll_x;
            let screen_bounds = Rect2D::new(x_screen, curr_y_screen, w_screen, h_screen);

            pages.push(PdfPageLayout {
                page_index: idx,
                unscaled_bounds,
                screen_bounds,
                media_box: *mbox,
                rotation: 0,
            });

            curr_y_unscaled += h_pt + config.page_gap;
            curr_y_screen += h_screen + config.page_gap * config.zoom;
        }

        let total_unscaled = Rect2D::new(
            0.0,
            0.0,
            max_unscaled_w + config.page_gap * 2.0,
            curr_y_unscaled,
        );
        let total_screen = Rect2D::new(
            horizontal_centering_offset - config.scroll_x,
            config.page_gap * config.zoom - config.scroll_y,
            scaled_max_w,
            curr_y_screen + config.scroll_y,
        );

        Self {
            pages,
            total_document_bounds: total_unscaled,
            total_screen_bounds: total_screen,
        }
    }

    fn layout_facing_pages(boxes: &[PdfRect], config: &PdfViewportConfig) -> Self {
        let mut pages = Vec::with_capacity(boxes.len());
        let gap = config.page_gap;
        let zoom = config.zoom;

        // Cover page (page 0) stands alone on right side in book layouts
        let mut curr_row_y_unscaled = gap;
        let mut curr_row_y_screen = gap * zoom - config.scroll_y;

        let mut max_spread_unscaled_w: f64 = 0.0;
        for pair in boxes.chunks(2) {
            let row_w = pair.iter().map(|b| b.width()).sum::<f64>() + gap;
            if row_w > max_spread_unscaled_w {
                max_spread_unscaled_w = row_w;
            }
        }

        let scaled_spread_w = max_spread_unscaled_w * zoom;
        let centering_x = if config.viewport.width > scaled_spread_w {
            (config.viewport.width - scaled_spread_w) / 2.0
        } else {
            gap
        };

        let mut idx = 0;
        while idx < boxes.len() {
            if idx == 0 {
                // Page 0 alone on right column
                let mbox = boxes[0];
                let w = mbox.width();
                let h = mbox.height();
                let col2_x_unscaled = max_spread_unscaled_w / 2.0 + gap / 2.0;

                let unscaled_bounds = Rect2D::new(col2_x_unscaled, curr_row_y_unscaled, w, h);
                let screen_x = centering_x + col2_x_unscaled * zoom - config.scroll_x;
                let screen_bounds = Rect2D::new(screen_x, curr_row_y_screen, w * zoom, h * zoom);

                pages.push(PdfPageLayout {
                    page_index: 0,
                    unscaled_bounds,
                    screen_bounds,
                    media_box: mbox,
                    rotation: 0,
                });

                curr_row_y_unscaled += h + gap;
                curr_row_y_screen += h * zoom + gap * zoom;
                idx += 1;
            } else {
                let left_box = boxes[idx];
                let right_box = boxes.get(idx + 1);

                let max_h = match right_box {
                    Some(rb) => left_box.height().max(rb.height()),
                    None => left_box.height(),
                };

                // Left page
                let lw = left_box.width();
                let lh = left_box.height();
                let left_x_unscaled = (max_spread_unscaled_w / 2.0) - lw - gap / 2.0;
                let left_unscaled = Rect2D::new(left_x_unscaled, curr_row_y_unscaled, lw, lh);
                let left_screen_x = centering_x + left_x_unscaled * zoom - config.scroll_x;
                let left_screen =
                    Rect2D::new(left_screen_x, curr_row_y_screen, lw * zoom, lh * zoom);

                pages.push(PdfPageLayout {
                    page_index: idx,
                    unscaled_bounds: left_unscaled,
                    screen_bounds: left_screen,
                    media_box: left_box,
                    rotation: 0,
                });

                // Right page (if present)
                if let Some(rb) = right_box {
                    let rw = rb.width();
                    let rh = rb.height();
                    let right_x_unscaled = max_spread_unscaled_w / 2.0 + gap / 2.0;
                    let right_unscaled = Rect2D::new(right_x_unscaled, curr_row_y_unscaled, rw, rh);
                    let right_screen_x = centering_x + right_x_unscaled * zoom - config.scroll_x;
                    let right_screen =
                        Rect2D::new(right_screen_x, curr_row_y_screen, rw * zoom, rh * zoom);

                    pages.push(PdfPageLayout {
                        page_index: idx + 1,
                        unscaled_bounds: right_unscaled,
                        screen_bounds: right_screen,
                        media_box: *rb,
                        rotation: 0,
                    });
                }

                curr_row_y_unscaled += max_h + gap;
                curr_row_y_screen += max_h * zoom + gap * zoom;
                idx += 2;
            }
        }

        Self {
            pages,
            total_document_bounds: Rect2D::new(
                0.0,
                0.0,
                max_spread_unscaled_w,
                curr_row_y_unscaled,
            ),
            total_screen_bounds: Rect2D::new(
                centering_x - config.scroll_x,
                0.0,
                scaled_spread_w,
                curr_row_y_screen + config.scroll_y,
            ),
        }
    }

    fn layout_single_page(boxes: &[PdfRect], config: &PdfViewportConfig) -> Self {
        let active = config.active_page.min(boxes.len().saturating_sub(1));
        let mbox = boxes[active];
        let w_pt = mbox.width();
        let h_pt = mbox.height();
        let w_screen = w_pt * config.zoom;
        let h_screen = h_pt * config.zoom;

        let x_screen = if config.viewport.width > w_screen {
            (config.viewport.width - w_screen) / 2.0 - config.scroll_x
        } else {
            config.page_gap - config.scroll_x
        };

        let y_screen = if config.viewport.height > h_screen {
            (config.viewport.height - h_screen) / 2.0 - config.scroll_y
        } else {
            config.page_gap - config.scroll_y
        };

        let page_layout = PdfPageLayout {
            page_index: active,
            unscaled_bounds: Rect2D::new(0.0, 0.0, w_pt, h_pt),
            screen_bounds: Rect2D::new(x_screen, y_screen, w_screen, h_screen),
            media_box: mbox,
            rotation: 0,
        };

        Self {
            pages: vec![page_layout],
            total_document_bounds: Rect2D::new(0.0, 0.0, w_pt, h_pt),
            total_screen_bounds: Rect2D::new(x_screen, y_screen, w_screen, h_screen),
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn get_page(&self, page_index: usize) -> Option<&PdfPageLayout> {
        self.pages.iter().find(|p| p.page_index == page_index)
    }
}

// =========================================================================
// 3. Tile Virtualization & Viewport Culling Engine
// =========================================================================

/// Sub-page tile coordinate for hardware-accelerated LOD caching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTile {
    pub page_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    pub screen_rect: Rect2D,
    pub page_user_rect: Rect2D,
    pub lod_level: u32,
}

pub struct PdfTileVirtualizer;

impl PdfTileVirtualizer {
    /// Identifies all page indices that intersect the visible viewport rectangle.
    pub fn get_visible_pages(layout: &PdfDocumentLayout, viewport: &Rect2D) -> Vec<usize> {
        let mut visible = Vec::new();
        for page in &layout.pages {
            if page.screen_bounds.intersects(viewport) {
                visible.push(page.page_index);
            }
        }
        visible
    }

    /// Decomposes visible pages into tiles and culls any tile completely outside the viewport.
    pub fn get_visible_tiles(
        layout: &PdfDocumentLayout,
        config: &PdfViewportConfig,
    ) -> Vec<PdfTile> {
        let mut visible_tiles = Vec::new();
        let tile_size = config.tile_size.max(128.0);

        let lod_level = if config.zoom <= 1.0 {
            0
        } else if config.zoom <= 2.0 {
            1
        } else {
            2
        };

        for page in &layout.pages {
            if !page.screen_bounds.intersects(&config.viewport) {
                continue;
            }

            let num_tiles_x = (page.screen_bounds.width / tile_size).ceil() as u32;
            let num_tiles_y = (page.screen_bounds.height / tile_size).ceil() as u32;

            for ty in 0..num_tiles_y {
                for tx in 0..num_tiles_x {
                    let tile_x_px = page.screen_bounds.x + (tx as f64) * tile_size;
                    let tile_y_px = page.screen_bounds.y + (ty as f64) * tile_size;
                    let tile_w_px = tile_size.min(page.screen_bounds.right() - tile_x_px);
                    let tile_h_px = tile_size.min(page.screen_bounds.bottom() - tile_y_px);

                    let screen_rect = Rect2D::new(tile_x_px, tile_y_px, tile_w_px, tile_h_px);

                    if screen_rect.intersects(&config.viewport) {
                        // Unscaled page space rect
                        let page_w_pt = tile_w_px / config.zoom;
                        let page_h_pt = tile_h_px / config.zoom;
                        let page_x_pt = (tile_x_px - page.screen_bounds.x) / config.zoom;
                        let page_y_pt = (tile_y_px - page.screen_bounds.y) / config.zoom;

                        visible_tiles.push(PdfTile {
                            page_index: page.page_index,
                            tile_x: tx,
                            tile_y: ty,
                            screen_rect,
                            page_user_rect: Rect2D::new(page_x_pt, page_y_pt, page_w_pt, page_h_pt),
                            lod_level,
                        });
                    }
                }
            }
        }

        visible_tiles
    }
}

// =========================================================================
// 4. Bidirectional Coordinate Transformation
// =========================================================================

pub struct PdfCoordinateTransformer;

impl PdfCoordinateTransformer {
    /// Translates screen viewport coordinate to PDF page user space point (with inverted Y-axis).
    pub fn screen_to_page_user(
        screen_x: f64,
        screen_y: f64,
        page: &PdfPageLayout,
        config: &PdfViewportConfig,
    ) -> Option<(f64, f64)> {
        if !page.screen_bounds.contains_point(screen_x, screen_y) {
            return None;
        }

        let rel_x = (screen_x - page.screen_bounds.x) / config.zoom;
        let rel_y = (screen_y - page.screen_bounds.y) / config.zoom;

        // PDF user space: (0, 0) at lower-left, Y points upwards
        let pdf_x = page.media_box.llx + rel_x;
        let pdf_y = page.media_box.ury - rel_y;

        Some((pdf_x, pdf_y))
    }

    /// Translates PDF user space coordinate back to screen pixels.
    pub fn page_user_to_screen(
        pdf_x: f64,
        pdf_y: f64,
        page: &PdfPageLayout,
        config: &PdfViewportConfig,
    ) -> (f64, f64) {
        let rel_x = (pdf_x - page.media_box.llx) * config.zoom;
        let rel_y = (page.media_box.ury - pdf_y) * config.zoom;

        (page.screen_bounds.x + rel_x, page.screen_bounds.y + rel_y)
    }

    /// Converts a PDF user space bounding rectangle to screen pixel rectangle.
    pub fn page_rect_to_screen(
        rect: &PdfRect,
        page: &PdfPageLayout,
        config: &PdfViewportConfig,
    ) -> Rect2D {
        let (x1, y_top) = Self::page_user_to_screen(rect.llx, rect.ury, page, config);
        let (x2, y_bottom) = Self::page_user_to_screen(rect.urx, rect.lly, page, config);

        Rect2D::new(
            x1.min(x2),
            y_top.min(y_bottom),
            (x2 - x1).abs(),
            (y_bottom - y_top).abs(),
        )
    }
}

/// Helper function to check if a point lies within a `PdfRect`.
fn pdf_rect_contains(rect: &PdfRect, x: f64, y: f64) -> bool {
    let min_x = rect.llx.min(rect.urx);
    let max_x = rect.llx.max(rect.urx);
    let min_y = rect.lly.min(rect.ury);
    let max_y = rect.lly.max(rect.ury);
    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

// =========================================================================
// 5. Interactive Text Selection & Hit Testing Model
// =========================================================================

/// Pinpointed text coordinate in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PdfTextPoint {
    pub page_index: usize,
    pub run_index: usize,
    pub glyph_index: usize,
    pub char_offset: usize,
}

impl PdfTextPoint {
    pub fn new(
        page_index: usize,
        run_index: usize,
        glyph_index: usize,
        char_offset: usize,
    ) -> Self {
        Self {
            page_index,
            run_index,
            glyph_index,
            char_offset,
        }
    }
}

/// Continuous text selection span across one or multiple pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfTextSelection {
    pub start: PdfTextPoint,
    pub end: PdfTextPoint,
}

impl PdfTextSelection {
    pub fn new(start: PdfTextPoint, end: PdfTextPoint) -> Self {
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Normalizes selection so that `start <= end` in reading order.
    pub fn normalized(&self) -> (PdfTextPoint, PdfTextPoint) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Checks if a glyph point falls within this selection range.
    pub fn contains(&self, pt: &PdfTextPoint) -> bool {
        let (s, e) = self.normalized();
        pt >= &s && pt <= &e
    }
}

/// Hit testing target classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PdfHitTarget {
    Text(PdfTextPoint),
    FormField { id: PdfObjectId, name: String },
    Annotation { id: PdfObjectId, subtype: String },
    PageBackground(usize),
    WorkspaceBackground,
}

/// Complete hit test result with spatial and user-space coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfHitResult {
    pub target: PdfHitTarget,
    pub screen_pos: (f64, f64),
    pub page_index: Option<usize>,
    pub page_user_pos: Option<(f64, f64)>,
}

pub struct PdfHitTester;

impl PdfHitTester {
    /// Performs geometric hit testing from screen coordinates down to individual glyphs or form fields.
    pub fn hit_test(
        screen_x: f64,
        screen_y: f64,
        layout: &PdfDocumentLayout,
        pages_text: &[ExtractedPageText],
        annotations: &[PdfAnnotation],
        config: &PdfViewportConfig,
    ) -> PdfHitResult {
        // 1. Check if inside any page
        for page in &layout.pages {
            if !page.screen_bounds.contains_point(screen_x, screen_y) {
                continue;
            }

            let user_pos =
                PdfCoordinateTransformer::screen_to_page_user(screen_x, screen_y, page, config);

            // Check annotations & form field widgets on this page
            if let Some((pdf_x, pdf_y)) = user_pos {
                for annot in annotations {
                    if annot.page_index == page.page_index
                        && pdf_rect_contains(&annot.rect, pdf_x, pdf_y)
                    {
                        let target = match annot.subtype {
                            AnnotationSubtype::Widget => PdfHitTarget::FormField {
                                id: annot.id,
                                name: annot.contents.clone().unwrap_or_default(),
                            },
                            _ => PdfHitTarget::Annotation {
                                id: annot.id,
                                subtype: format!("{:?}", annot.subtype),
                            },
                        };
                        return PdfHitResult {
                            target,
                            screen_pos: (screen_x, screen_y),
                            page_index: Some(page.page_index),
                            page_user_pos: Some((pdf_x, pdf_y)),
                        };
                    }
                }

                // Check text glyphs on this page
                if let Some(page_text) = pages_text
                    .iter()
                    .find(|pt| pt.page_number == page.page_index)
                {
                    for (run_idx, run) in page_text.runs.iter().enumerate() {
                        let run_rect =
                            PdfCoordinateTransformer::page_rect_to_screen(&run.bbox, page, config);
                        // Expanded run bounding box for easier targeting
                        let expanded_run = Rect2D::new(
                            run_rect.x - 2.0,
                            run_rect.y - 2.0,
                            run_rect.width + 4.0,
                            run_rect.height + 4.0,
                        );

                        if expanded_run.contains_point(screen_x, screen_y) {
                            for (glyph_idx, glyph) in run.glyphs.iter().enumerate() {
                                let glyph_rect = PdfCoordinateTransformer::page_rect_to_screen(
                                    &glyph.bbox,
                                    page,
                                    config,
                                );
                                if glyph_rect.contains_point(screen_x, screen_y)
                                    || (screen_x >= glyph_rect.x
                                        && screen_x <= glyph_rect.right()
                                        && screen_y >= glyph_rect.y - 4.0
                                        && screen_y <= glyph_rect.bottom() + 4.0)
                                {
                                    return PdfHitResult {
                                        target: PdfHitTarget::Text(PdfTextPoint::new(
                                            page.page_index,
                                            run_idx,
                                            glyph_idx,
                                            glyph_idx,
                                        )),
                                        screen_pos: (screen_x, screen_y),
                                        page_index: Some(page.page_index),
                                        page_user_pos: Some((pdf_x, pdf_y)),
                                    };
                                }
                            }
                        }
                    }
                }
            }

            return PdfHitResult {
                target: PdfHitTarget::PageBackground(page.page_index),
                screen_pos: (screen_x, screen_y),
                page_index: Some(page.page_index),
                page_user_pos: user_pos,
            };
        }

        PdfHitResult {
            target: PdfHitTarget::WorkspaceBackground,
            screen_pos: (screen_x, screen_y),
            page_index: None,
            page_user_pos: None,
        }
    }
}

// =========================================================================
// 6. Text Selection Highlight Quads & Clipboard Extraction
// =========================================================================

pub struct PdfSelectionHelper;

impl PdfSelectionHelper {
    /// Extracts the plain string spanned by the text selection for clipboard copy.
    pub fn extract_selected_text(
        selection: &PdfTextSelection,
        pages_text: &[ExtractedPageText],
    ) -> String {
        if selection.is_empty() {
            return String::new();
        }

        let (start, end) = selection.normalized();
        let mut extracted = String::new();
        let mut last_page: Option<usize> = None;
        let mut last_run: Option<usize> = None;

        for page in pages_text {
            if page.page_number < start.page_index || page.page_number > end.page_index {
                continue;
            }

            if let Some(prev) = last_page {
                if prev != page.page_number {
                    extracted.push_str("\n\n");
                }
            }
            last_page = Some(page.page_number);

            for (run_idx, run) in page.runs.iter().enumerate() {
                if page.page_number == start.page_index && run_idx < start.run_index {
                    continue;
                }
                if page.page_number == end.page_index && run_idx > end.run_index {
                    continue;
                }

                if let Some(prev_r) = last_run {
                    if prev_r != run_idx {
                        extracted.push(' ');
                    }
                }
                last_run = Some(run_idx);

                for (glyph_idx, glyph) in run.glyphs.iter().enumerate() {
                    let pt = PdfTextPoint::new(page.page_number, run_idx, glyph_idx, glyph_idx);
                    if pt >= start && pt <= end {
                        extracted.push_str(&glyph.unicode);
                    }
                }
            }
        }

        extracted
    }

    /// Computes highlight quads for selected text in screen coordinates.
    pub fn compute_selection_quads(
        selection: &PdfTextSelection,
        pages_text: &[ExtractedPageText],
        layout: &PdfDocumentLayout,
        config: &PdfViewportConfig,
    ) -> Vec<QuadInstance> {
        if selection.is_empty() {
            return Vec::new();
        }

        let (start, end) = selection.normalized();
        let mut quads = Vec::new();
        let highlight_color = ColorRgba::new(0.2, 0.45, 0.9, 0.35); // Semi-transparent selection blue

        for page_layout in &layout.pages {
            if page_layout.page_index < start.page_index || page_layout.page_index > end.page_index
            {
                continue;
            }

            let Some(page_text) = pages_text
                .iter()
                .find(|pt| pt.page_number == page_layout.page_index)
            else {
                continue;
            };

            for (run_idx, run) in page_text.runs.iter().enumerate() {
                if page_layout.page_index == start.page_index && run_idx < start.run_index {
                    continue;
                }
                if page_layout.page_index == end.page_index && run_idx > end.run_index {
                    continue;
                }

                let mut run_start_x: Option<f64> = None;
                let mut run_end_x: f64 = 0.0;
                let mut run_y: f64 = 0.0;
                let mut run_h: f64 = 0.0;

                for (glyph_idx, glyph) in run.glyphs.iter().enumerate() {
                    let pt =
                        PdfTextPoint::new(page_layout.page_index, run_idx, glyph_idx, glyph_idx);
                    if pt >= start && pt <= end {
                        let g_rect = PdfCoordinateTransformer::page_rect_to_screen(
                            &glyph.bbox,
                            page_layout,
                            config,
                        );

                        if run_start_x.is_none() {
                            run_start_x = Some(g_rect.x);
                            run_y = g_rect.y;
                            run_h = g_rect.height;
                        }
                        run_end_x = g_rect.right();
                        run_y = run_y.min(g_rect.y);
                        run_h = run_h.max(g_rect.height);
                    }
                }

                if let Some(sx) = run_start_x {
                    let sel_rect = Rect2D::new(sx, run_y, (run_end_x - sx).max(2.0), run_h);
                    quads.push(QuadInstance::solid(sel_rect, highlight_color));
                }
            }
        }

        quads
    }
}

// =========================================================================
// 7. Retained Display List Compiler
// =========================================================================

/// Retained Display List Compiler generating hardware-accelerated draw calls for the PDF viewport.
pub struct PdfDisplayCompiler;

impl PdfDisplayCompiler {
    /// Compiles the complete PDF viewport into a `DisplayList` within the 120fps frame budget.
    pub fn compile(
        layout: &PdfDocumentLayout,
        config: &PdfViewportConfig,
        pages_text: &[ExtractedPageText],
        annotations: &[PdfAnnotation],
        form_fields: &[FormField],
        selection: Option<&PdfTextSelection>,
    ) -> DisplayList {
        let mut dl = DisplayList::new();

        // 1. Workspace background canvas fill (#525659)
        let bg_color = ColorRgba::new(0.32, 0.34, 0.35, 1.0);
        dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            config.viewport,
            bg_color,
        )));

        // 2. Identify visible pages intersecting viewport
        let visible_pages = PdfTileVirtualizer::get_visible_pages(layout, &config.viewport);

        for page_idx in visible_pages {
            let Some(page) = layout.get_page(page_idx) else {
                continue;
            };

            // 2a. Drop shadow behind the page (4px offset)
            let shadow_rect = Rect2D::new(
                page.screen_bounds.x + 3.0,
                page.screen_bounds.y + 3.0,
                page.screen_bounds.width,
                page.screen_bounds.height,
            );
            dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                shadow_rect,
                ColorRgba::new(0.0, 0.0, 0.0, 0.20),
            )));

            // 2b. White page canvas with thin border
            dl.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                page.screen_bounds,
                ColorRgba::WHITE,
                ColorRgba::new(0.75, 0.75, 0.75, 1.0),
                1.0,
            )));

            // 2c. Render text runs & glyphs for this page
            if let Some(page_text) = pages_text.iter().find(|pt| pt.page_number == page_idx) {
                for run in &page_text.runs {
                    let run_rect =
                        PdfCoordinateTransformer::page_rect_to_screen(&run.bbox, page, config);
                    if !run_rect.intersects(&config.viewport) {
                        continue;
                    }

                    let mut glyphs = Vec::with_capacity(run.glyphs.len());
                    for glyph in &run.glyphs {
                        let g_rect = PdfCoordinateTransformer::page_rect_to_screen(
                            &glyph.bbox,
                            page,
                            config,
                        );

                        glyphs.push(GlyphInstance {
                            glyph_id: glyph.char_code,
                            x: g_rect.x as f32,
                            y: g_rect.y as f32,
                            width: g_rect.width as f32,
                            height: g_rect.height as f32,
                            uv_rect: [0.0, 0.0, 1.0, 1.0],
                            color: ColorRgba::new(
                                run.color[0],
                                run.color[1],
                                run.color[2],
                                run.color[3],
                            ),
                        });
                    }

                    let font_size_px = (run.font_size * config.zoom) as f32;
                    dl.push(DisplayCommand::DrawText(GlyphRun {
                        text: run.text.clone(),
                        glyphs,
                        font_size_px,
                        font_family: run.font_name.clone(),
                        color: ColorRgba::new(
                            run.color[0],
                            run.color[1],
                            run.color[2],
                            run.color[3],
                        ),
                        bounds: run_rect,
                        bidi_level: 0,
                        script: TextScript::Latin,
                    }));
                }
            }

            // 2d. Render interactive AcroForm fields
            for field in form_fields {
                if let Some(p_idx) = field.page_index {
                    if p_idx != page_idx {
                        continue;
                    }
                }

                let field_screen_rect =
                    PdfCoordinateTransformer::page_rect_to_screen(&field.rect, page, config);
                if !field_screen_rect.intersects(&config.viewport) {
                    continue;
                }

                match &field.field_type {
                    FormFieldType::Text { .. } => {
                        // Light blue tinted input box with border
                        let bg = ColorRgba::new(0.95, 0.97, 1.0, 1.0);
                        let border = ColorRgba::new(0.4, 0.6, 0.9, 1.0);
                        dl.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                            field_screen_rect,
                            bg,
                            border,
                            1.0,
                        )));

                        if let Some(ref text_val) = field.value {
                            let text_rect = Rect2D::new(
                                field_screen_rect.x + 3.0,
                                field_screen_rect.y + 2.0,
                                field_screen_rect.width - 6.0,
                                field_screen_rect.height - 4.0,
                            );
                            dl.push(DisplayCommand::DrawText(GlyphRun {
                                text: text_val.clone(),
                                glyphs: Vec::new(),
                                font_size_px: (12.0 * config.zoom) as f32,
                                font_family: "Helvetica".into(),
                                color: ColorRgba::BLACK,
                                bounds: text_rect,
                                bidi_level: 0,
                                script: TextScript::Latin,
                            }));
                        }
                    }
                    FormFieldType::Checkbox { .. } | FormFieldType::RadioButton { .. } => {
                        let bg = ColorRgba::WHITE;
                        let border = ColorRgba::BLACK;
                        dl.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                            field_screen_rect,
                            bg,
                            border,
                            1.0,
                        )));

                        let is_checked = field.value.as_deref() == Some("Yes");
                        if is_checked {
                            let check_rect = Rect2D::new(
                                field_screen_rect.x + field_screen_rect.width * 0.25,
                                field_screen_rect.y + field_screen_rect.height * 0.25,
                                field_screen_rect.width * 0.5,
                                field_screen_rect.height * 0.5,
                            );
                            dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                                check_rect,
                                ColorRgba::BLACK,
                            )));
                        }
                    }
                    _ => {}
                }
            }

            // 2e. Render annotations (Highlights, Links, Notes)
            for annot in annotations {
                if annot.page_index != page_idx {
                    continue;
                }

                let annot_rect =
                    PdfCoordinateTransformer::page_rect_to_screen(&annot.rect, page, config);
                if !annot_rect.intersects(&config.viewport) {
                    continue;
                }

                match &annot.subtype {
                    AnnotationSubtype::Highlight { .. } => {
                        let fill = annot
                            .color
                            .map(|[r, g, b]| ColorRgba::new(r, g, b, 0.40))
                            .unwrap_or(ColorRgba::new(1.0, 1.0, 0.0, 0.40));
                        dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                            annot_rect, fill,
                        )));
                    }
                    AnnotationSubtype::Text => {
                        // Yellow sticky note icon quad
                        let fill = ColorRgba::new(1.0, 0.85, 0.2, 0.9);
                        dl.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                            annot_rect,
                            fill,
                            ColorRgba::BLACK,
                            1.0,
                        )));
                    }
                    _ => {}
                }
            }

            // 2f. Render selection highlight overlay for this page
            if let Some(sel) = selection {
                let sel_quads =
                    PdfSelectionHelper::compute_selection_quads(sel, pages_text, layout, config);
                for quad in sel_quads {
                    dl.push(DisplayCommand::DrawQuad(quad));
                }
            }
        }

        dl
    }
}
