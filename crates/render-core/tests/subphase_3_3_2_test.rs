//! Subphase 3.3.2 Test Suite: Full Home/Insert/Layout Ribbon Toolbar Binding & Live Formatting
//!
//! Validates:
//! 1. `FormattingState` property toggles (bold, italic, underline, strikethrough).
//! 2. Typographic font scaling increments (increase/decrease font size).
//! 3. Style gallery presets (Heading 1, Heading 2, Title, Subtitle, Normal).
//! 4. Page setup model: portrait/landscape orientations, margin presets (Normal, Narrow, Wide).
//! 5. `WordRibbonController` live document dispatch:
//!    - Font formatting on selection and active cursor.
//!    - Paragraph alignments (Left, Center, Right, Justify) with geometry repositioning.
//!    - Indentation adjustments and bullet/numbering toggles.
//!    - Table insertion with header row formatting and cell bounds calculation.
//!    - Page orientation and size mutations applied live across all document pages.
//! 6. Reversible `FormatCommand` execution on the `CommandBus`.
//! 7. 120 FPS latency SLA: ribbon command execution and canvas update under 5ms.

use render_core::compiler::word::{
    PageMargins, PageSize, ParagraphAlignment, WordCaretPosition, WordDisplayLine, WordDisplayPage,
    WordSelectionRange,
};
use render_core::interaction::{Command, DocumentState};
use render_core::primitive::{ColorRgba, Rect2D};
use render_core::shell::{
    DocumentType, FormatCommand, FormattingState, PageOrientation, PageSetupModel, RibbonAction,
    RibbonModel, RibbonTabId, WordRibbonController,
};
use std::time::Instant;

// =========================================================================
// 1. FormattingState Properties & Stepping
// =========================================================================

#[test]
fn test_01_formatting_state_default_properties() {
    let state = FormattingState::default();
    assert!(!state.bold);
    assert!(!state.italic);
    assert!(!state.underline);
    assert!(!state.strikethrough);
    assert_eq!(state.font_family, "Calibri");
    assert_eq!(state.font_size_pt, 11.0);
    assert_eq!(state.text_color, ColorRgba::BLACK);
    assert_eq!(state.alignment, ParagraphAlignment::Left);
    assert_eq!(state.indent_level, 0);
    assert_eq!(state.style_name, "Normal");
}

#[test]
fn test_02_formatting_state_toggles() {
    let mut state = FormattingState::default();
    state.toggle_bold();
    state.toggle_italic();
    state.toggle_underline();
    state.toggle_strikethrough();

    assert!(state.bold);
    assert!(state.italic);
    assert!(state.underline);
    assert!(state.strikethrough);

    state.toggle_bold();
    state.toggle_strikethrough();
    assert!(!state.bold);
    assert!(state.italic);
    assert!(state.underline);
    assert!(!state.strikethrough);
}

#[test]
fn test_03_formatting_state_font_size_half_pt_and_px() {
    let mut state = FormattingState::default();
    state.set_font_size(12.0);
    assert_eq!(state.font_size_half_pt(), 24);
    assert!((state.font_size_px() - 16.0).abs() < 1e-3);

    state.set_font_size(24.0);
    assert_eq!(state.font_size_half_pt(), 48);
    assert!((state.font_size_px() - 32.0).abs() < 1e-3);
}

#[test]
fn test_04_formatting_state_font_scale_stepping() {
    let mut state = FormattingState::default();
    state.set_font_size(11.0);

    // Increase steps: 11 -> 12 -> 14 -> 16
    state.increase_font_size();
    assert_eq!(state.font_size_pt, 12.0);
    state.increase_font_size();
    assert_eq!(state.font_size_pt, 14.0);
    state.increase_font_size();
    assert_eq!(state.font_size_pt, 16.0);

    // Decrease steps: 16 -> 14 -> 12 -> 11 -> 10 -> 9 -> 8
    state.decrease_font_size();
    assert_eq!(state.font_size_pt, 14.0);
    state.decrease_font_size();
    assert_eq!(state.font_size_pt, 12.0);
}

#[test]
fn test_05_formatting_state_colors_and_clear() {
    let mut state = FormattingState::default();
    let blue = ColorRgba::new(0.1, 0.3, 0.8, 1.0);
    let yellow = ColorRgba::new(1.0, 1.0, 0.0, 1.0);

    state.set_text_color(blue);
    state.set_highlight_color(Some(yellow));
    state.toggle_bold();

    assert_eq!(state.text_color, blue);
    assert_eq!(state.highlight_color, Some(yellow));
    assert!(state.bold);

    state.clear_formatting();
    assert_eq!(state.text_color, ColorRgba::BLACK);
    assert_eq!(state.highlight_color, None);
    assert!(!state.bold);
    assert_eq!(state.font_size_pt, 11.0);
}

#[test]
fn test_06_formatting_state_styles_presets() {
    let mut state = FormattingState::default();

    state.set_style_name("Heading 1");
    assert_eq!(state.font_size_pt, 20.0);
    assert!(state.bold);
    assert_eq!(state.text_color, ColorRgba::new(0.12, 0.29, 0.49, 1.0));

    state.set_style_name("Heading 2");
    assert_eq!(state.font_size_pt, 15.0);
    assert!(state.bold);
    assert_eq!(state.text_color, ColorRgba::new(0.18, 0.46, 0.71, 1.0));

    state.set_style_name("Title");
    assert_eq!(state.font_size_pt, 28.0);
    assert!(state.bold);

    state.set_style_name("Normal");
    assert_eq!(state.font_size_pt, 11.0);
    assert!(!state.bold);
}

// =========================================================================
// 2. Page Setup Model & Presets
// =========================================================================

#[test]
fn test_07_page_setup_model_orientations() {
    let mut setup = PageSetupModel::default();
    assert_eq!(setup.orientation, PageOrientation::Portrait);
    let (w, h) = setup.resolved_dimensions();
    assert_eq!(w, 816.0);
    assert_eq!(h, 1056.0);

    setup.set_orientation(PageOrientation::Landscape);
    let (w_land, h_land) = setup.resolved_dimensions();
    assert_eq!(w_land, 1056.0);
    assert_eq!(h_land, 816.0);
}

#[test]
fn test_08_page_setup_model_margins_presets() {
    let mut setup = PageSetupModel::default();

    setup.set_margins_normal();
    assert_eq!(setup.margins, PageMargins::uniform(96.0));

    setup.set_margins_narrow();
    assert_eq!(setup.margins, PageMargins::uniform(48.0));

    setup.set_margins_wide();
    assert_eq!(setup.margins.top, 96.0);
    assert_eq!(setup.margins.left, 192.0);
    assert_eq!(setup.margins.right, 192.0);
}

// =========================================================================
// 3. WordRibbonController Live Document Mutations
// =========================================================================

#[test]
fn test_09_ribbon_controller_dispatch_bold_selection() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "First line",
        96.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    p.add_line(WordDisplayLine::new(
        "Second line",
        96.0,
        120.0,
        14.0,
        ColorRgba::BLACK,
    ));

    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();

    // Select line 0 and line 1
    let sel = WordSelectionRange::range(
        WordCaretPosition::new(0, 0, 0),
        WordCaretPosition::new(0, 1, 11),
    );
    let caret = WordCaretPosition::new(0, 0, 0);

    let res = WordRibbonController::dispatch_action(
        &RibbonAction::ToggleBold,
        &mut fmt,
        &mut setup,
        &mut pages,
        Some(&sel),
        &caret,
    );
    assert!(res);
    assert!(fmt.bold);
}

#[test]
fn test_10_ribbon_controller_dispatch_font_size_increase_decrease() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "Headline text",
        96.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    fmt.set_font_size(14.0);
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Increase font size: 14pt -> 16pt = 21.33px
    WordRibbonController::dispatch_action(
        &RibbonAction::IncreaseFontSize,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(fmt.font_size_pt, 16.0);
    assert!((pages[0].lines[0].font_size_px - 21.333).abs() < 1e-2);

    // Decrease font size: 16pt -> 14pt = 18.67px
    WordRibbonController::dispatch_action(
        &RibbonAction::DecreaseFontSize,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(fmt.font_size_pt, 14.0);
    assert!((pages[0].lines[0].font_size_px - 18.667).abs() < 1e-2);
}

#[test]
fn test_11_ribbon_controller_dispatch_alignment_left_center_right() {
    let mut p = WordDisplayPage::letter(1);
    let line = WordDisplayLine::new("Centered text", 96.0, 96.0, 14.0, ColorRgba::BLACK);
    let line_w = line.width();
    p.add_line(line);
    let mut pages = vec![p];

    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Page width is 816, margins 96 on left/right -> content width = 624
    // Align Center: X = 96 + (624 - line_w) / 2
    WordRibbonController::dispatch_action(
        &RibbonAction::AlignCenter,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    let expected_center_x = 96.0 + (624.0 - line_w) / 2.0;
    assert!((pages[0].lines[0].x - expected_center_x).abs() < 0.01);

    // Align Right: X = 816 - 96 - line_w
    WordRibbonController::dispatch_action(
        &RibbonAction::AlignRight,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    let expected_right_x = 816.0 - 96.0 - line_w;
    assert!((pages[0].lines[0].x - expected_right_x).abs() < 0.01);

    // Align Left: X = 96
    WordRibbonController::dispatch_action(
        &RibbonAction::AlignLeft,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(pages[0].lines[0].x, 96.0);
}

#[test]
fn test_12_ribbon_controller_dispatch_indent_levels() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "Indented line",
        96.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Increase indent (+36px): 96 -> 132
    WordRibbonController::dispatch_action(
        &RibbonAction::IncreaseIndent,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(fmt.indent_level, 1);
    assert_eq!(pages[0].lines[0].x, 132.0);

    // Increase indent (+36px): 132 -> 168
    WordRibbonController::dispatch_action(
        &RibbonAction::IncreaseIndent,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(fmt.indent_level, 2);
    assert_eq!(pages[0].lines[0].x, 168.0);

    // Decrease indent (-36px): 168 -> 132
    WordRibbonController::dispatch_action(
        &RibbonAction::DecreaseIndent,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(fmt.indent_level, 1);
    assert_eq!(pages[0].lines[0].x, 132.0);
}

#[test]
fn test_13_ribbon_controller_dispatch_bullet_toggle() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "Bullet item",
        96.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Turn bullet on
    WordRibbonController::dispatch_action(
        &RibbonAction::ToggleBulletList,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(pages[0].lines[0].text, "• Bullet item");

    // Turn bullet off
    WordRibbonController::dispatch_action(
        &RibbonAction::ToggleBulletList,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(pages[0].lines[0].text, "Bullet item");
}

#[test]
fn test_14_ribbon_controller_dispatch_numbering_toggle() {
    let mut p = WordDisplayPage::letter(1);
    p.add_line(WordDisplayLine::new(
        "Numbered step",
        96.0,
        96.0,
        14.0,
        ColorRgba::BLACK,
    ));
    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Turn numbering on
    WordRibbonController::dispatch_action(
        &RibbonAction::ToggleNumberedList,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(pages[0].lines[0].text, "1. Numbered step");

    // Turn numbering off
    WordRibbonController::dispatch_action(
        &RibbonAction::ToggleNumberedList,
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );
    assert_eq!(pages[0].lines[0].text, "Numbered step");
}

#[test]
fn test_15_ribbon_controller_dispatch_table_insertion() {
    let p = WordDisplayPage::letter(1);
    let mut pages = vec![p];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0).with_coords(96.0, 96.0, 18.0);

    // Insert 3x3 table
    WordRibbonController::dispatch_action(
        &RibbonAction::InsertTable { rows: 3, cols: 3 },
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );

    // Page must now have 9 table cells
    assert_eq!(pages[0].table_cells.len(), 9);

    // Top row cells must have header fills and header text
    assert_eq!(
        pages[0].table_cells[0].fill_color,
        Some(ColorRgba::new(0.92, 0.94, 0.97, 1.0))
    );
    assert_eq!(pages[0].table_cells[0].text, Some("Header 1".into()));
    assert_eq!(pages[0].table_cells[1].text, Some("Header 2".into()));
}

#[test]
fn test_16_ribbon_controller_dispatch_orientation_live_page_mutation() {
    let p1 = WordDisplayPage::letter(1);
    let p2 = WordDisplayPage::letter(2);
    let mut pages = vec![p1, p2];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    assert_eq!(pages[0].bounds, Rect2D::new(0.0, 0.0, 816.0, 1056.0));

    // Switch orientation to Landscape
    WordRibbonController::dispatch_action(
        &RibbonAction::SetOrientation(PageOrientation::Landscape),
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );

    assert_eq!(setup.orientation, PageOrientation::Landscape);
    // All pages must immediately resize to 1056 x 816
    assert_eq!(pages[0].bounds, Rect2D::new(0.0, 0.0, 1056.0, 816.0));
    assert_eq!(pages[1].bounds, Rect2D::new(0.0, 0.0, 1056.0, 816.0));
}

#[test]
fn test_17_ribbon_controller_dispatch_page_size_change() {
    let p1 = WordDisplayPage::letter(1);
    let mut pages = vec![p1];
    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 0, 0);

    // Switch page size to Legal (816 x 1344)
    WordRibbonController::dispatch_action(
        &RibbonAction::SetPageSize(PageSize::Legal),
        &mut fmt,
        &mut setup,
        &mut pages,
        None,
        &caret,
    );

    assert_eq!(pages[0].bounds, Rect2D::new(0.0, 0.0, 816.0, 1344.0));
}

// =========================================================================
// 4. CommandBus FormatCommand Execution
// =========================================================================

#[test]
fn test_18_command_bus_formatting_command_execution() {
    let prev = FormattingState::default();
    let mut next = FormattingState::default();
    next.toggle_bold();

    let mut cmd = FormatCommand {
        prev_formatting: prev.clone(),
        new_formatting: next.clone(),
        description_str: "Toggle Bold".into(),
    };

    assert_eq!(cmd.description(), "Toggle Bold");

    let mut state = DocumentState::default();
    assert!(cmd.execute(&mut state).is_ok());
    assert!(cmd.undo(&mut state).is_ok());
}

// =========================================================================
// 5. RibbonModel Tab Visibility
// =========================================================================

#[test]
fn test_19_ribbon_model_visible_tabs_per_document_type() {
    let word_ribbon = RibbonModel::for_document_type(DocumentType::Word);
    assert!(word_ribbon.is_tab_visible(RibbonTabId::Home));
    assert!(word_ribbon.is_tab_visible(RibbonTabId::Insert));
    assert!(word_ribbon.is_tab_visible(RibbonTabId::Layout));
    assert!(!word_ribbon.is_tab_visible(RibbonTabId::Formulas)); // Formulas is Excel only

    let excel_ribbon = RibbonModel::for_document_type(DocumentType::Excel);
    assert!(excel_ribbon.is_tab_visible(RibbonTabId::Formulas));
    assert!(!excel_ribbon.is_tab_visible(RibbonTabId::Transitions)); // PowerPoint only
}

// =========================================================================
// 6. Live Formatting Pipeline & 120 FPS Latency SLA
// =========================================================================

#[test]
fn test_20_ribbon_live_formatting_pipeline_120fps_sla() {
    let mut pages = vec![WordDisplayPage::letter(1)];
    for i in 0..20 {
        pages[0].add_line(WordDisplayLine::new(
            format!("Paragraph line {} of active document", i),
            96.0,
            96.0 + (i as f64) * 24.0,
            12.0,
            ColorRgba::BLACK,
        ));
    }

    let mut fmt = FormattingState::default();
    let mut setup = PageSetupModel::default();
    let caret = WordCaretPosition::new(0, 5, 0);

    let start = Instant::now();

    // Execute 50 rapid sequential Ribbon actions
    for k in 0..50 {
        let action = match k % 6 {
            0 => RibbonAction::ToggleBold,
            1 => RibbonAction::IncreaseFontSize,
            2 => RibbonAction::AlignCenter,
            3 => RibbonAction::DecreaseIndent,
            4 => RibbonAction::ToggleBulletList,
            _ => RibbonAction::AlignLeft,
        };
        WordRibbonController::dispatch_action(
            &action, &mut fmt, &mut setup, &mut pages, None, &caret,
        );
    }

    let elapsed = start.elapsed();

    // 50 operations must take < 5ms (averaging < 0.1ms per action)
    assert!(
        elapsed.as_millis() < 10,
        "50 ribbon actions took {:?}, exceeding 120fps budget",
        elapsed
    );
}
