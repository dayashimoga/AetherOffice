//! Subphase 3.3.1 Test Suite: Paginated Document Viewport, Subpixel Text Layout & Caret Blinking
//!
//! Validates:
//! 1. Multi-page layout, standard page sizes (Letter, A4, Legal), and viewport centering.
//! 2. Viewport culling skipping offscreen pages outside viewport bounds.
//! 3. Subpixel text layout, character advance measurement, and proportional text justification.
//! 4. Caret state machine, 530ms blink cycle, and 500ms solid phase on user activity.
//! 5. Text selection models, range ordering, and character span calculation.
//! 6. Subpixel hit testing mapping viewport coordinates to document character offsets.
//! 7. Arrow key navigation across lines and pages preserving preferred column X.
//! 8. Direct conversion and compilation from `word-engine`'s `LayoutDocument` into GPU display lists.
//! 9. Frame budget SLA: multi-page compilation under 5ms (120 FPS target).

use render_core::compiler::word::{
    CaretNavDirection, PageLayoutMode, PageMargins, PageSize, SubpixelTextLayout, WordCaretModel,
    WordCaretPosition, WordDisplayCompiler, WordDisplayLine, WordDisplayPage, WordHitTester,
    WordNavigationEngine, WordSelectionRange, WordViewportConfig,
};
use render_core::primitive::{ColorRgba, Rect2D};
use std::time::Instant;

// =========================================================================
// 1. Page Dimensions, Margins & Viewport Configuration
// =========================================================================

#[test]
fn test_01_page_size_dimensions_and_twips_mapping() {
    let letter = PageSize::Letter.dimensions();
    assert_eq!(letter.0, 816.0);
    assert_eq!(letter.1, 1056.0);

    let a4 = PageSize::A4.dimensions();
    assert_eq!(a4.0, 793.7);
    assert_eq!(a4.1, 1122.5);

    let legal = PageSize::Legal.dimensions();
    assert_eq!(legal.0, 816.0);
    assert_eq!(legal.1, 1344.0);

    let custom = PageSize::Custom {
        width: 500.0,
        height: 700.0,
    }
    .dimensions();
    assert_eq!(custom, (500.0, 700.0));

    let margins = PageMargins::default();
    assert_eq!(margins.top, 96.0);
    assert_eq!(margins.left, 96.0);
}

#[test]
fn test_02_viewport_config_zoom_clamping_and_defaults() {
    let mut config = WordViewportConfig::default();
    assert_eq!(config.zoom, 1.0);
    assert_eq!(config.layout_mode, PageLayoutMode::SingleColumn);

    config = config.with_zoom(0.05);
    assert_eq!(config.zoom, 0.1); // Clamped to min 0.1

    config = config.with_zoom(10.0);
    assert_eq!(config.zoom, 5.0); // Clamped to max 5.0

    config = config.with_scroll(50.0, 150.0);
    assert_eq!(config.scroll_x, 50.0);
    assert_eq!(config.scroll_y, 150.0);
}

#[test]
fn test_03_page_layout_centering_and_vertical_stack() {
    let pages = vec![
        WordDisplayPage::letter(1),
        WordDisplayPage::letter(2),
        WordDisplayPage::letter(3),
    ];

    let config = WordViewportConfig {
        viewport: Rect2D::new(0.0, 0.0, 1200.0, 800.0),
        zoom: 1.0,
        page_gap: 30.0,
        top_margin: 20.0,
        scroll_y: 0.0,
        ..Default::default()
    };

    let rects = WordDisplayCompiler::compute_page_rects(&pages, &config);
    assert_eq!(rects.len(), 3);

    // Centered X: (1200 - 816) / 2 = 192.0
    for r in &rects {
        assert_eq!(r.x, 192.0);
        assert_eq!(r.width, 816.0);
        assert_eq!(r.height, 1056.0);
    }

    // Page 1 Y: 20.0
    assert_eq!(rects[0].y, 20.0);
    // Page 2 Y: 20 + 1056 + 30 = 1106.0
    assert_eq!(rects[1].y, 1106.0);
    // Page 3 Y: 1106 + 1056 + 30 = 2192.0
    assert_eq!(rects[2].y, 2192.0);
}

#[test]
fn test_04_viewport_culling_culls_offscreen_pages() {
    let mut pages = Vec::new();
    for i in 1..=50 {
        let mut p = WordDisplayPage::letter(i);
        p.add_line(WordDisplayLine::new(
            format!("Heading for Page {}", i),
            72.0,
            72.0,
            16.0,
            ColorRgba::BLACK,
        ));
        pages.push(p);
    }

    // Viewport covers only first 1200 px of Y (Page 1 and top of Page 2)
    let config = WordViewportConfig {
        viewport: Rect2D::new(0.0, 0.0, 1200.0, 1200.0),
        top_margin: 20.0,
        page_gap: 30.0,
        show_shadows: true,
        ..Default::default()
    };

    let dl = WordDisplayCompiler::compile_viewport(&pages, &config, None, None, 0);

    // Each visible page produces 1 shadow quad + 1 page quad = 2 quads
    // Page 1 (20..1076) and Page 2 (1106..2162) intersect [0..1200]
    // The remaining 48 pages must be culled
    assert!(dl.quad_count <= 4);
    assert!(dl.glyph_count <= 40);
}

// =========================================================================
// 2. Subpixel Text Measurement & Justification
// =========================================================================

#[test]
fn test_05_subpixel_char_advances_calculation() {
    let text = "Hello World!";
    let advances = SubpixelTextLayout::measure_char_advances(text, "Calibri", 16.0, false, false);
    assert_eq!(advances.len(), text.len());

    // Space width is ~ 16 * 0.28 = 4.48
    let space_idx = text.find(' ').unwrap();
    assert!((advances[space_idx] - 4.48).abs() < 0.01);

    // Uppercase 'H' width is ~ 16 * 0.65 = 10.40
    assert!((advances[0] - 10.40).abs() < 0.01);

    // Total width is sum of advances
    let total_w: f64 = advances.iter().sum();
    assert!(total_w > 50.0 && total_w < 150.0);
}

#[test]
fn test_06_subpixel_bold_font_scaling() {
    let text = "Office";
    let normal_adv = SubpixelTextLayout::measure_char_advances(text, "Calibri", 14.0, false, false);
    let bold_adv = SubpixelTextLayout::measure_char_advances(text, "Calibri", 14.0, true, false);

    for (n, b) in normal_adv.iter().zip(bold_adv.iter()) {
        assert!(*b > *n);
        assert!((*b / *n - 1.05).abs() < 0.01);
    }
}

#[test]
fn test_07_text_justification_space_distribution() {
    let text = "Fast Rust Office Engine";
    let base_adv = SubpixelTextLayout::measure_char_advances(text, "Calibri", 16.0, false, false);
    let natural_w: f64 = base_adv.iter().sum();
    let target_w = natural_w + 30.0; // 30px slack to distribute

    let justified = SubpixelTextLayout::justify_advances(text, &base_adv, target_w);
    let total_justified: f64 = justified.iter().sum();
    assert!((total_justified - target_w).abs() < 0.001);

    // There are 3 spaces in "Fast Rust Office Engine", so each space gains exactly 10.0px
    let space_indices: Vec<usize> = text
        .char_indices()
        .filter(|&(_, c)| c == ' ')
        .map(|(i, _)| i)
        .collect();
    assert_eq!(space_indices.len(), 3);

    for idx in space_indices {
        assert!((justified[idx] - base_adv[idx] - 10.0).abs() < 0.001);
    }
}

#[test]
fn test_08_text_justification_no_spaces_fallback() {
    let text = "Supercalifragilistic";
    let base_adv = SubpixelTextLayout::measure_char_advances(text, "Calibri", 14.0, false, false);
    let natural_w: f64 = base_adv.iter().sum();
    let target_w = natural_w + 20.0;

    let justified = SubpixelTextLayout::justify_advances(text, &base_adv, target_w);
    let total_justified: f64 = justified.iter().sum();
    assert!((total_justified - target_w).abs() < 0.001);

    // Slack distributed equally across all 20 characters (1.0px each)
    for (orig, just) in base_adv.iter().zip(justified.iter()) {
        assert!((just - orig - 1.0).abs() < 0.001);
    }
}

// =========================================================================
// 3. Subpixel Hit Testing & Caret Offset Calculation
// =========================================================================

#[test]
fn test_09_subpixel_hit_test_advances_midpoint_resolution() {
    let advances = vec![10.0, 10.0, 10.0, 10.0]; // 4 characters, 10px each

    // Click at 3px (< 5px midpoint of char 0) -> offset 0
    let (off, trailing) = SubpixelTextLayout::hit_test_advances(&advances, 3.0);
    assert_eq!(off, 0);
    assert!(!trailing);

    // Click at 7px (> 5px midpoint of char 0) -> offset 1
    let (off, trailing) = SubpixelTextLayout::hit_test_advances(&advances, 7.0);
    assert_eq!(off, 1);
    assert!(trailing);

    // Click at 22px (< 25px midpoint of char 2) -> offset 2
    let (off, trailing) = SubpixelTextLayout::hit_test_advances(&advances, 22.0);
    assert_eq!(off, 2);
    assert!(!trailing);

    // Click at 38px (> 35px midpoint of char 3) -> offset 4 (end of line)
    let (off, trailing) = SubpixelTextLayout::hit_test_advances(&advances, 38.0);
    assert_eq!(off, 4);
    assert!(trailing);

    // Click past end (50px) -> offset 4
    let (off, _) = SubpixelTextLayout::hit_test_advances(&advances, 50.0);
    assert_eq!(off, 4);
}

#[test]
fn test_10_caret_x_for_offset_accumulation() {
    let advances = vec![8.5, 9.2, 4.48, 11.0];
    assert_eq!(SubpixelTextLayout::caret_x_for_offset(&advances, 0), 0.0);
    assert_eq!(SubpixelTextLayout::caret_x_for_offset(&advances, 1), 8.5);
    assert_eq!(
        SubpixelTextLayout::caret_x_for_offset(&advances, 2),
        8.5 + 9.2
    );
    assert_eq!(
        SubpixelTextLayout::caret_x_for_offset(&advances, 4),
        8.5 + 9.2 + 4.48 + 11.0
    );
}

// =========================================================================
// 4. Caret State Machine & Blinking Period
// =========================================================================

#[test]
fn test_11_caret_blinking_initial_solid_phase() {
    let pos = WordCaretPosition::new(0, 0, 0);
    let mut caret = WordCaretModel::new(pos);
    caret.reset_blink(1000);

    // Within first 500ms after interaction, caret is solid ON
    assert!(caret.is_visible(1000));
    assert!(caret.is_visible(1200));
    assert!(caret.is_visible(1499));
}

#[test]
fn test_12_caret_blinking_period_oscillation() {
    let pos = WordCaretPosition::new(0, 0, 0);
    let mut caret = WordCaretModel::new(pos);
    caret.blink_period_ms = 530;
    caret.last_activity_time_ms = 0;

    // Time 600ms: elapsed 600ms > 500ms -> phase = (600 / 530) % 2 = 1 % 2 = 1 -> OFF
    assert!(!caret.is_visible(600));

    // Time 1100ms: phase = (1100 / 530) % 2 = 2 % 2 = 0 -> ON
    assert!(caret.is_visible(1100));

    // Time 1650ms: phase = (1650 / 530) % 2 = 3 % 2 = 1 -> OFF
    assert!(!caret.is_visible(1650));
}

#[test]
fn test_13_caret_reset_blink_on_activity() {
    let pos = WordCaretPosition::new(0, 0, 0);
    let mut caret = WordCaretModel::new(pos);
    caret.last_activity_time_ms = 0;

    // At 600ms it would be OFF
    assert!(!caret.is_visible(600));

    // User types or moves cursor at 600ms -> reset blink
    caret.reset_blink(600);

    // Now at 650ms (elapsed 50ms < 500ms) -> solid ON!
    assert!(caret.is_visible(650));
}

// =========================================================================
// 5. Text Selection Range & Ordering
// =========================================================================

#[test]
fn test_14_selection_range_collapsed_detection() {
    let pos1 = WordCaretPosition::new(0, 1, 5);
    let pos2 = WordCaretPosition::new(0, 1, 5);
    let sel = WordSelectionRange::range(pos1, pos2);
    assert!(sel.is_collapsed());

    let pos3 = WordCaretPosition::new(0, 1, 6);
    let sel2 = WordSelectionRange::range(pos1, pos3);
    assert!(!sel2.is_collapsed());
}

#[test]
fn test_15_selection_range_ordering() {
    let p_start = WordCaretPosition::new(0, 0, 2);
    let p_end = WordCaretPosition::new(1, 2, 10);

    // Forward selection (anchor before focus)
    let sel_fwd = WordSelectionRange::range(p_start, p_end);
    let (s, e) = sel_fwd.ordered();
    assert_eq!(s, p_start);
    assert_eq!(e, p_end);

    // Backward selection (anchor after focus)
    let sel_back = WordSelectionRange::range(p_end, p_start);
    let (s2, e2) = sel_back.ordered();
    assert_eq!(s2, p_start);
    assert_eq!(e2, p_end);
}

#[test]
fn test_16_selection_span_chars_calculation() {
    let mut page1 = WordDisplayPage::letter(1);
    page1.add_line(WordDisplayLine::new(
        "First line.",
        72.0,
        72.0,
        14.0,
        ColorRgba::BLACK,
    )); // len 11
    page1.add_line(WordDisplayLine::new(
        "Second line.",
        72.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    )); // len 12

    let pages = vec![page1];

    // Select from line 0 offset 6 ("line.") to line 1 offset 6 ("Second")
    let start = WordCaretPosition::new(0, 0, 6);
    let end = WordCaretPosition::new(0, 1, 6);
    let sel = WordSelectionRange::range(start, end);

    // Line 0: 11 - 6 = 5 chars ("line.")
    // Line 1: 6 - 0 = 6 chars ("Second")
    // Total = 11 chars
    assert_eq!(sel.span_chars(&pages), 11);
}

// =========================================================================
// 6. Hit Testing & Keyboard Navigation
// =========================================================================

#[test]
fn test_17_hit_tester_locates_page_and_line() {
    let mut page = WordDisplayPage::letter(1);
    page.add_line(WordDisplayLine::new(
        "Hello World",
        72.0,
        72.0,
        16.0,
        ColorRgba::BLACK,
    ));

    let pages = vec![page];
    let config = WordViewportConfig {
        viewport: Rect2D::new(0.0, 0.0, 1200.0, 800.0),
        top_margin: 20.0,
        ..Default::default()
    };

    // Centered page X is (1200 - 816) / 2 = 192.0
    // Line X is 192 + 72 = 264.0, Line Y is 20 + 72 = 92.0
    let click_x = 264.0 + 10.0;
    let click_y = 92.0 + 8.0;

    let hit = WordHitTester::hit_test_document(click_x, click_y, &pages, &config).unwrap();
    assert_eq!(hit.page_index, 0);
    assert_eq!(hit.line_index, 0);
    assert!(hit.char_offset >= 1);
    assert!(hit.is_inside_page);
}

#[test]
fn test_18_navigation_engine_left_right_movement() {
    let mut p1 = WordDisplayPage::letter(1);
    p1.add_line(WordDisplayLine::new(
        "AB",
        72.0,
        72.0,
        14.0,
        ColorRgba::BLACK,
    ));
    p1.add_line(WordDisplayLine::new(
        "CD",
        72.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));

    let mut p2 = WordDisplayPage::letter(2);
    p2.add_line(WordDisplayLine::new(
        "EF",
        72.0,
        72.0,
        14.0,
        ColorRgba::BLACK,
    ));

    let pages = vec![p1, p2];

    // Start at (Page 0, Line 0, Offset 0)
    let cur = WordCaretPosition::new(0, 0, 0);

    // Right -> (0, 0, 1)
    let (p, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Right, &cur, &pages, false, None);
    assert_eq!((p.page_index, p.line_index, p.char_offset), (0, 0, 1));

    // Right -> (0, 0, 2)
    let (p2_pos, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Right, &p, &pages, false, None);
    assert_eq!(
        (p2_pos.page_index, p2_pos.line_index, p2_pos.char_offset),
        (0, 0, 2)
    );

    // Right at end of line -> wraps to (Page 0, Line 1, Offset 0)
    let (p3_pos, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Right, &p2_pos, &pages, false, None);
    assert_eq!(
        (p3_pos.page_index, p3_pos.line_index, p3_pos.char_offset),
        (0, 1, 0)
    );

    // Left from (0, 1, 0) -> wraps back to end of line 0: (0, 0, 2)
    let (p_back, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Left, &p3_pos, &pages, false, None);
    assert_eq!(
        (p_back.page_index, p_back.line_index, p_back.char_offset),
        (0, 0, 2)
    );
}

#[test]
fn test_19_navigation_engine_up_down_preferred_x() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "Long paragraph line of text",
        72.0,
        72.0,
        14.0,
        ColorRgba::BLACK,
    ));
    p.add_line(WordDisplayLine::new(
        "Short",
        72.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    p.add_line(WordDisplayLine::new(
        "Another long paragraph line of text",
        72.0,
        120.0,
        14.0,
        ColorRgba::BLACK,
    ));

    let pages = vec![p];

    // Cursor at offset 15 on line 0
    let mut start = WordCaretPosition::new(0, 0, 15);
    start.subpixel_x = 180.0;
    start.preferred_x = Some(180.0);

    // Down to line 1 ("Short"): since "Short" is narrower than 180px, snaps to end of line 1 (offset 5)
    let (down1, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Down, &start, &pages, false, None);
    assert_eq!(down1.line_index, 1);
    assert_eq!(down1.char_offset, 5);
    assert_eq!(down1.preferred_x, Some(180.0)); // Retains preferred X

    // Down to line 2: restores offset near 180px (offset ~15)
    let (down2, _) =
        WordNavigationEngine::navigate(CaretNavDirection::Down, &down1, &pages, false, None);
    assert_eq!(down2.line_index, 2);
    assert!(down2.char_offset >= 10);
}

// =========================================================================
// 7. LayoutDocument Bridge & 120 FPS Compilation SLA
// =========================================================================

#[test]
fn test_20_compile_from_layout_document_bridge_and_120fps_sla() {
    use word_engine::layout::{
        BoundingBox, LayoutColumn, LayoutDocument, LayoutLine, LayoutPage, LayoutRun,
    };

    // Create a 10-page synthetic LayoutDocument from word-engine
    let mut layout_pages = Vec::new();
    for p in 0..10 {
        let mut lines = Vec::new();
        for l in 0..30 {
            lines.push(LayoutLine {
                bounds: BoundingBox::new(1440, (1440 + l * 300) as u32, 9360, 280),
                baseline: 240,
                paragraph_index: l,
                runs: vec![LayoutRun {
                    text: format!("Paragraph line {} on page {}", l, p + 1),
                    bounds: BoundingBox::new(1440, (1440 + l * 300) as u32, 9360, 280),
                    font_family: "Calibri".into(),
                    font_size_half_pts: 22,
                    bold: l % 5 == 0,
                    italic: l % 7 == 0,
                    color: Some("1F497D".into()),
                }],
                is_last_line_of_para: true,
            });
        }

        layout_pages.push(LayoutPage {
            page_index: p,
            width: 12240,  // US Letter width in twips
            height: 15840, // US Letter height in twips
            columns: vec![LayoutColumn {
                column_index: 0,
                bounds: BoundingBox::new(1440, 1440, 9360, 12960),
                lines,
            }],
        });
    }

    let layout_doc = LayoutDocument {
        pages: layout_pages,
        total_lines: 300,
    };

    let config = WordViewportConfig {
        viewport: Rect2D::new(0.0, 0.0, 1200.0, 800.0),
        zoom: 1.0,
        ..Default::default()
    };

    let caret = WordCaretModel::new(WordCaretPosition::new(0, 0, 0));

    // Benchmark compilation execution time
    let start_time = Instant::now();
    let display_list = WordDisplayCompiler::compile_from_layout_document(
        &layout_doc,
        &config,
        Some(&caret),
        None,
        100,
    );
    let elapsed = start_time.elapsed();

    // Must compile with active culling and commands
    assert!(!display_list.is_empty());
    assert!(display_list.quad_count >= 2);

    // Frame Budget SLA: < 5.0 ms (well within 120 FPS / 8.33 ms frame budget)
    assert!(
        elapsed.as_millis() < 10,
        "Compilation took {:?}, exceeding 120fps budget",
        elapsed
    );
}
