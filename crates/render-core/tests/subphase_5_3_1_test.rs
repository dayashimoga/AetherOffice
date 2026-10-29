//! Subphase 5.3.1 Integration Test Suite:
//! Tile-Virtualized Multi-Page PDF Canvas with Zoom & Text Select
//!
//! Validates:
//! - Smooth zoom scaling (25% - 500%), zoom preset ladder, zoom-to-fit-width/page, cursor anchor
//! - Multi-page layout engines (Continuous vertical, Facing pages book spread, Single page)
//! - Tile virtualization, LOD levels, and aggressive viewport culling
//! - Bidirectional coordinate transformations (Screen <-> Document <-> PDF User Space with Y-inversion)
//! - Interactive text selection model with normalization and clipboard text extraction
//! - Precise character-level and annotation/form field hit testing
//! - Retained GPU DisplayList generation with page canvas, drop shadow, text, annotations, and form widgets
//! - 120fps frame pacing benchmark (< 8.33 ms budget) across multi-page documents

use pdf_engine::acroform::{FormField, FormFieldType};
use pdf_engine::annotation::{AnnotationSubtype, PdfAnnotation};
use pdf_engine::text::{ExtractedPageText, TextGlyph, TextRun};
use pdf_engine::types::{PdfObjectId, PdfRect};
use render_core::compiler::pdf::{
    PdfCoordinateTransformer, PdfDisplayCompiler, PdfDocumentLayout, PdfHitTarget, PdfHitTester,
    PdfPageLayoutMode, PdfSelectionHelper, PdfTextPoint, PdfTextSelection, PdfTileVirtualizer,
    PdfViewportConfig, MAX_PDF_ZOOM, MIN_PDF_ZOOM,
};
use render_core::primitive::{ColorRgba, DisplayCommand};
use std::time::Instant;

fn sample_letter_pages(count: usize) -> Vec<PdfRect> {
    vec![PdfRect::letter(); count]
}

fn sample_page_text(page_number: usize, line: &str, y_pt: f64) -> ExtractedPageText {
    let mut glyphs = Vec::new();
    let mut curr_x = 72.0; // 1-inch left margin
    let char_w = 7.2; // approx 12pt Helvetica width
    let char_h = 12.0;

    for ch in line.chars() {
        let bbox = PdfRect::new(curr_x, y_pt, curr_x + char_w, y_pt + char_h);
        glyphs.push(TextGlyph {
            char_code: ch as u32,
            unicode: ch.to_string(),
            width: char_w,
            bbox,
        });
        curr_x += char_w;
    }

    let run_bbox = PdfRect::new(72.0, y_pt, curr_x, y_pt + char_h);
    let run = TextRun {
        text: line.to_string(),
        font_name: "Helvetica".into(),
        font_size: 12.0,
        color: [0.0, 0.0, 0.0, 1.0],
        matrix: [1.0, 0.0, 0.0, 1.0, 72.0, y_pt],
        bbox: run_bbox,
        glyphs,
    };

    ExtractedPageText {
        page_number,
        runs: vec![run],
    }
}

// =========================================================================
// Tests
// =========================================================================

#[test]
fn test_01_viewport_config_zoom_presets_and_limits() {
    let mut config = PdfViewportConfig::new(1024.0, 768.0);
    assert_eq!(config.zoom, 1.0);

    // Zoom in ladder: 1.0 -> 1.25 -> 1.5 -> 2.0 -> 3.0 -> 4.0 -> 5.0
    config.zoom_in();
    assert!((config.zoom - 1.25).abs() < 1e-4);
    config.zoom_in();
    assert!((config.zoom - 1.50).abs() < 1e-4);
    config.zoom_in();
    assert!((config.zoom - 2.00).abs() < 1e-4);

    // Clamp at maximum 5.0 (500%)
    config.set_zoom(10.0);
    assert_eq!(config.zoom, MAX_PDF_ZOOM);

    // Zoom out ladder
    config.zoom_out();
    assert!((config.zoom - 4.00).abs() < 1e-4);

    // Clamp at minimum 0.25 (25%)
    config.set_zoom(0.05);
    assert_eq!(config.zoom, MIN_PDF_ZOOM);
    config.zoom_out();
    assert_eq!(config.zoom, MIN_PDF_ZOOM);
}

#[test]
fn test_02_viewport_config_zoom_to_fit_width() {
    let config = PdfViewportConfig::new(1280.0, 800.0);
    let page_width = 612.0; // US Letter width
    let padding = 20.0;

    let zoom = config.calculate_zoom_to_fit_width(page_width, padding);
    // (1280 - 40) / 612 = 1240 / 612 = 2.0261
    assert!((zoom - (1240.0 / 612.0)).abs() < 1e-4);
}

#[test]
fn test_03_viewport_config_zoom_to_fit_page() {
    let config = PdfViewportConfig::new(1000.0, 800.0);
    let letter = PdfRect::letter();
    let padding = 16.0;

    let zoom = config.calculate_zoom_to_fit_page(letter.width(), letter.height(), padding);
    let avail_w = 1000.0 - 32.0; // 968.0
    let avail_h = 800.0 - 32.0; // 768.0
    let zw = avail_w / 612.0; // 1.5816
    let zh = avail_h / 792.0; // 0.9696

    assert!((zoom - zh).abs() < 1e-4);
    assert!(zoom < zw); // Height was the constraining factor
}

#[test]
fn test_04_viewport_config_zoom_around_point_anchor() {
    let mut config = PdfViewportConfig::new(1200.0, 900.0);
    config.zoom = 1.0;
    config.scroll_x = 100.0;
    config.scroll_y = 200.0;

    let anchor_x = 400.0;
    let anchor_y = 300.0;

    // Point in document coordinates before zoom:
    let doc_x_before = (anchor_x + config.scroll_x) / config.zoom;
    let doc_y_before = (anchor_y + config.scroll_y) / config.zoom;

    // Zoom to 2.0x anchored at (400, 300)
    config.zoom_around_point(2.0, anchor_x, anchor_y);
    assert_eq!(config.zoom, 2.0);

    // Verify document point under anchor is identical after zoom:
    let doc_x_after = (anchor_x + config.scroll_x) / config.zoom;
    let doc_y_after = (anchor_y + config.scroll_y) / config.zoom;

    assert!((doc_x_before - doc_x_after).abs() < 1e-4);
    assert!((doc_y_before - doc_y_after).abs() < 1e-4);
}

#[test]
fn test_05_multi_page_continuous_layout_geometry() {
    let pages = sample_letter_pages(3);
    let config = PdfViewportConfig::new(1000.0, 1200.0).with_zoom(1.0);

    let layout = PdfDocumentLayout::calculate(&pages, &config);
    assert_eq!(layout.page_count(), 3);

    let p0 = layout.get_page(0).unwrap();
    let p1 = layout.get_page(1).unwrap();
    let p2 = layout.get_page(2).unwrap();

    // Centered horizontally: (1000 - 612) / 2 = 194.0
    assert_eq!(p0.screen_bounds.x, 194.0);
    assert_eq!(p1.screen_bounds.x, 194.0);
    assert_eq!(p2.screen_bounds.x, 194.0);

    // Vertical spacing: gap = 16.0
    assert_eq!(p0.screen_bounds.y, 16.0);
    assert_eq!(p1.screen_bounds.y, 16.0 + 792.0 + 16.0);
    assert_eq!(p2.screen_bounds.y, 16.0 + (792.0 + 16.0) * 2.0);

    // Document total height
    assert!(layout.total_document_bounds.height > 792.0 * 3.0);
}

#[test]
fn test_06_multi_page_facing_pages_book_layout() {
    let pages = sample_letter_pages(4);
    let config = PdfViewportConfig::new(1600.0, 1000.0)
        .with_zoom(1.0)
        .with_layout_mode(PdfPageLayoutMode::FacingPages);

    let layout = PdfDocumentLayout::calculate(&pages, &config);
    assert_eq!(layout.page_count(), 4);

    let p0 = layout.get_page(0).unwrap();
    let p1 = layout.get_page(1).unwrap();
    let p2 = layout.get_page(2).unwrap();

    // Page 0 is on the right spread column
    // Page 1 is on the left column, Page 2 is on the right column
    assert!(p1.screen_bounds.x < p2.screen_bounds.x);
    // Page 1 and 2 share the same row vertical position
    assert_eq!(p1.screen_bounds.y, p2.screen_bounds.y);
    assert!(p0.screen_bounds.y < p1.screen_bounds.y);
}

#[test]
fn test_07_single_page_focused_layout() {
    let pages = sample_letter_pages(5);
    let mut config =
        PdfViewportConfig::new(1200.0, 900.0).with_layout_mode(PdfPageLayoutMode::SinglePage);
    config.active_page = 2;

    let layout = PdfDocumentLayout::calculate(&pages, &config);
    assert_eq!(layout.page_count(), 1);

    let active_page = layout.get_page(2).unwrap();
    assert_eq!(active_page.page_index, 2);
    // Centered in viewport
    assert_eq!(active_page.screen_bounds.x, (1200.0 - 612.0) / 2.0);
    assert_eq!(active_page.screen_bounds.y, (900.0 - 792.0) / 2.0);
}

#[test]
fn test_08_tile_virtualization_and_viewport_culling() {
    let pages = sample_letter_pages(10);
    // Viewport shows only the top 800 pixels
    let config = PdfViewportConfig::new(1000.0, 800.0).with_zoom(1.0);
    let layout = PdfDocumentLayout::calculate(&pages, &config);

    let visible_pages = PdfTileVirtualizer::get_visible_pages(&layout, &config.viewport);
    // Page 0 (y: 16..808) intersects viewport (y: 0..800).
    assert!(visible_pages.contains(&0));
    // Later pages (e.g. Page 5, 6, 7) must be culled!
    assert!(!visible_pages.contains(&5));
    assert!(!visible_pages.contains(&9));

    // Tile generation
    let tiles = PdfTileVirtualizer::get_visible_tiles(&layout, &config);
    assert!(!tiles.is_empty());
    for tile in &tiles {
        assert!(tile.screen_rect.intersects(&config.viewport));
        assert!(tile.page_index == 0 || tile.page_index == 1);
    }
}

#[test]
fn test_09_coordinate_transformation_bidirectional() {
    let pages = sample_letter_pages(2);
    let config = PdfViewportConfig::new(1200.0, 900.0)
        .with_zoom(1.5)
        .with_scroll(50.0, 100.0);
    let layout = PdfDocumentLayout::calculate(&pages, &config);
    let p0 = layout.get_page(0).unwrap();

    let orig_pdf_x = 150.0;
    let orig_pdf_y = 600.0;

    // Transform to screen
    let (screen_x, screen_y) =
        PdfCoordinateTransformer::page_user_to_screen(orig_pdf_x, orig_pdf_y, p0, &config);

    // Transform back to PDF user space
    let (back_x, back_y) =
        PdfCoordinateTransformer::screen_to_page_user(screen_x, screen_y, p0, &config).unwrap();

    assert!((orig_pdf_x - back_x).abs() < 1e-4);
    assert!((orig_pdf_y - back_y).abs() < 1e-4);
}

#[test]
fn test_10_interactive_hit_testing_text_glyphs() {
    let pages = sample_letter_pages(1);
    let config = PdfViewportConfig::new(1000.0, 800.0).with_zoom(1.0);
    let layout = PdfDocumentLayout::calculate(&pages, &config);
    let p0 = layout.get_page(0).unwrap();

    let text_line = "Antigravity PDF";
    let page_text = sample_page_text(0, text_line, 700.0);

    // Target the first character 'A' (x: 72..79.2, y: 700..712 in PDF space)
    let (click_x, click_y) =
        PdfCoordinateTransformer::page_user_to_screen(75.0, 706.0, p0, &config);

    let hit = PdfHitTester::hit_test(click_x, click_y, &layout, &[page_text], &[], &config);

    match hit.target {
        PdfHitTarget::Text(pt) => {
            assert_eq!(pt.page_index, 0);
            assert_eq!(pt.run_index, 0);
            assert_eq!(pt.glyph_index, 0);
        }
        other => panic!("Expected text hit on 'A', got {:?}", other),
    }
}

#[test]
fn test_11_interactive_hit_testing_form_fields_and_annotations() {
    let pages = sample_letter_pages(1);
    let config = PdfViewportConfig::new(1000.0, 800.0).with_zoom(1.0);
    let layout = PdfDocumentLayout::calculate(&pages, &config);
    let p0 = layout.get_page(0).unwrap();

    let annot = PdfAnnotation {
        id: PdfObjectId::new(12, 0),
        page_index: 0,
        subtype: AnnotationSubtype::Widget,
        rect: PdfRect::new(100.0, 500.0, 250.0, 530.0),
        contents: Some("User_Email".into()),
        author: None,
        color: None,
        flags: 0,
    };

    let (click_x, click_y) =
        PdfCoordinateTransformer::page_user_to_screen(120.0, 515.0, p0, &config);

    let hit = PdfHitTester::hit_test(click_x, click_y, &layout, &[], &[annot], &config);

    match hit.target {
        PdfHitTarget::FormField { id, name } => {
            assert_eq!(id, PdfObjectId::new(12, 0));
            assert_eq!(name, "User_Email");
        }
        other => panic!("Expected FormField hit, got {:?}", other),
    }
}

#[test]
fn test_12_text_selection_range_and_normalization() {
    let pt_start = PdfTextPoint::new(0, 0, 5, 5);
    let pt_end = PdfTextPoint::new(0, 0, 1, 1);

    // Inverted selection (end before start)
    let sel = PdfTextSelection::new(pt_start, pt_end);
    let (norm_start, norm_end) = sel.normalized();

    assert_eq!(norm_start, pt_end);
    assert_eq!(norm_end, pt_start);

    // Point in between
    let mid_point = PdfTextPoint::new(0, 0, 3, 3);
    assert!(sel.contains(&mid_point));

    // Point outside
    let outside_point = PdfTextPoint::new(0, 0, 10, 10);
    assert!(!sel.contains(&outside_point));
}

#[test]
fn test_13_text_selection_clipboard_extraction() {
    let page_text = sample_page_text(0, "OpenOfficeRS PDF Canvas", 700.0);

    // Select "OpenOfficeRS" (glyphs 0..12)
    let sel = PdfTextSelection::new(
        PdfTextPoint::new(0, 0, 0, 0),
        PdfTextPoint::new(0, 0, 11, 11),
    );

    let extracted = PdfSelectionHelper::extract_selected_text(&sel, &[page_text]);
    assert_eq!(extracted, "OpenOfficeRS");
}

#[test]
fn test_14_pdf_display_compiler_retained_commands() {
    let pages = sample_letter_pages(2);
    let config = PdfViewportConfig::new(1280.0, 800.0).with_zoom(1.0);
    let layout = PdfDocumentLayout::calculate(&pages, &config);

    let page_text = sample_page_text(0, "Subphase 5.3.1 Display Compiler", 700.0);
    let field = FormField {
        id: PdfObjectId::new(5, 0),
        name: "Full_Name".into(),
        full_name: "Full_Name".into(),
        field_type: FormFieldType::Text {
            multiline: false,
            password: false,
        },
        value: Some("Linus Torvalds".into()),
        default_value: None,
        rect: PdfRect::new(100.0, 600.0, 250.0, 630.0),
        page_index: Some(0),
        read_only: false,
        required: true,
        default_appearance: None,
        appearance_stream: None,
    };

    let selection =
        PdfTextSelection::new(PdfTextPoint::new(0, 0, 0, 0), PdfTextPoint::new(0, 0, 8, 8));

    let dl = PdfDisplayCompiler::compile(
        &layout,
        &config,
        &[page_text],
        &[],
        &[field],
        Some(&selection),
    );

    assert!(!dl.is_empty());
    assert!(dl.quad_count >= 4); // Workspace bg + drop shadow + page bg + form field bg + selection
    assert!(dl.glyph_count > 0);

    // Verify workspace background is present
    let has_workspace_bg = dl.commands.iter().any(|cmd| {
        if let DisplayCommand::DrawQuad(q) = cmd {
            q.color == ColorRgba::new(0.32, 0.34, 0.35, 1.0)
        } else {
            false
        }
    });
    assert!(has_workspace_bg);
}

#[test]
fn test_15_performance_120fps_viewport_culling_and_compile_sla() {
    // 100-page document
    let pages = sample_letter_pages(100);
    let config = PdfViewportConfig::new(1920.0, 1080.0).with_zoom(1.25);

    let start = Instant::now();

    // 1. Layout calculation
    let layout = PdfDocumentLayout::calculate(&pages, &config);

    // 2. Tile virtualization and culling
    let tiles = PdfTileVirtualizer::get_visible_tiles(&layout, &config);

    // 3. Display list compilation
    let dl = PdfDisplayCompiler::compile(&layout, &config, &[], &[], &[], None);

    let elapsed = start.elapsed();

    assert!(!tiles.is_empty());
    assert!(!dl.is_empty());

    // SLA: Frame budget at 120fps is 8.33 ms
    assert!(
        elapsed.as_millis() < 8,
        "SLA Violation: Expected PDF compile < 8.33 ms, took {:?}",
        elapsed
    );
}
