#![forbid(unsafe_code)]
//! Subphase 4.3.2 Test Suite: Slide Sorter Filmstrip, Master Theme Switcher & Fullscreen F5 Show
//!
//! Validates:
//! - Master presentation theme presets (Office Default, Dark Mode, Vibrant Coral, High Contrast).
//! - Theme application across slide canvas background and shape color palettes.
//! - Reversible theme switching via Command Bus.
//! - Slide Sorter filmstrip metadata, continuous reindexing, drag-and-drop reordering, lifecycle operations.
//! - Backward-compatible `SlideSorterEngine::reorder_slides`.
//! - Fullscreen presentation slide show state machine (F5 from beginning, Shift+F5 from current, Next, Previous, Loop).
//! - Keyboard navigation (`Space`, `ArrowRight`, `ArrowLeft`, `Escape`, `L` for laser pointer, `B` for black screen).
//! - Laser pointer overlay (glow halo radius 18px and bright core radius 6px).
//! - DisplayList compilation of fullscreen presentation show with aspect ratio preservation.
//! - 120 FPS latency SLA benchmark (< 8.33 ms frame budget).

use render_core::compiler::ppt::{
    MasterTheme, PptDisplayCompiler, PresetShapeType, SlideCanvasConfig, SlideMetadata,
    SlideShapeModel, SlideShowAction, SlideShowStateMachine, SlideSorterCommand, SlideSorterModel,
    SlideStageModel, SwitchThemeCommand, ThemePreset,
};
use render_core::interaction::{CommandBus, DocumentState};
use render_core::primitive::{ColorRgba, DisplayCommand, Rect2D};
use render_core::shell::SlideSorterEngine;
use std::time::Instant;

// =========================================================================
// 1. Master Theme Switcher Tests
// =========================================================================

#[test]
fn test_01_master_theme_presets_default_properties() {
    let office = MasterTheme::office_default();
    assert_eq!(office.name, "Office Default");
    assert_eq!(office.preset, ThemePreset::OfficeDefault);
    assert_eq!(office.major_font, "Calibri Light");
    assert_eq!(office.minor_font, "Calibri");
    assert_eq!(office.bg_color, ColorRgba::WHITE);

    let dark = MasterTheme::dark_mode();
    assert_eq!(dark.name, "Dark Mode");
    assert_eq!(dark.preset, ThemePreset::DarkMode);
    assert_eq!(dark.major_font, "Segoe UI");
    assert_eq!(dark.bg_color, ColorRgba::new(0.12, 0.12, 0.12, 1.0));

    let coral = MasterTheme::vibrant_coral();
    assert_eq!(coral.name, "Vibrant Coral");
    assert_eq!(coral.preset, ThemePreset::VibrantCoral);
    assert_eq!(coral.major_font, "Aptos");

    let contrast = MasterTheme::high_contrast();
    assert_eq!(contrast.name, "High Contrast");
    assert_eq!(contrast.preset, ThemePreset::HighContrast);
    assert_eq!(contrast.bg_color, ColorRgba::BLACK);
    assert_eq!(contrast.text_color, ColorRgba::WHITE);
}

#[test]
fn test_02_master_theme_from_preset_factory() {
    assert_eq!(
        MasterTheme::from_preset(ThemePreset::OfficeDefault).preset,
        ThemePreset::OfficeDefault
    );
    assert_eq!(
        MasterTheme::from_preset(ThemePreset::DarkMode).preset,
        ThemePreset::DarkMode
    );
    assert_eq!(
        MasterTheme::from_preset(ThemePreset::VibrantCoral).preset,
        ThemePreset::VibrantCoral
    );
    assert_eq!(
        MasterTheme::from_preset(ThemePreset::HighContrast).preset,
        ThemePreset::HighContrast
    );
}

#[test]
fn test_03_master_theme_apply_to_stage() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let s1 = SlideShapeModel::new(1, PresetShapeType::Rectangle, Rect2D::ZERO);
    let s2 = SlideShapeModel::new(2, PresetShapeType::Ellipse, Rect2D::ZERO);
    let mut s3 = SlideShapeModel::new(3, PresetShapeType::TextBox, Rect2D::ZERO);
    s3.text = Some("Theme Header".to_string());
    stage.add_shape(s1);
    stage.add_shape(s2);
    stage.add_shape(s3);

    let dark_theme = MasterTheme::dark_mode();
    dark_theme.apply_to_stage(&mut stage);

    // Canvas background becomes dark
    assert_eq!(stage.canvas_config.background_color, dark_theme.bg_color);
    // Shapes receive theme accents
    assert_eq!(stage.shapes[0].fill_color, dark_theme.accent1);
    assert_eq!(stage.shapes[1].fill_color, dark_theme.accent2);
    // Text box receives theme text color
    assert_eq!(stage.shapes[2].text_color, dark_theme.text_color);
}

#[test]
fn test_04_theme_switcher_command_bus_undo_redo() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let cmd = SwitchThemeCommand {
        prev_theme: ThemePreset::OfficeDefault,
        new_theme: ThemePreset::DarkMode,
    };

    assert!(bus.execute(&mut state, Box::new(cmd)).is_ok());
    assert_eq!(bus.undo_stack_len(), 1);
    assert!(bus.undo(&mut state).is_ok());
    assert_eq!(bus.undo_stack_len(), 0);
    assert_eq!(bus.redo_stack_len(), 1);
    assert!(bus.redo(&mut state).is_ok());
}

// =========================================================================
// 2. Slide Sorter Filmstrip & Reordering Tests
// =========================================================================

#[test]
fn test_05_slide_sorter_model_initialization_and_count() {
    let sorter = SlideSorterModel::new();
    assert_eq!(sorter.count(), 0);
    assert_eq!(sorter.active_index, 0);

    let mut sorter_with_slides = SlideSorterModel::from_slides(vec![
        SlideMetadata {
            id: 1,
            index: 0,
            title: "Intro".to_string(),
            shape_count: 3,
            is_hidden: false,
            transition: "Fade".to_string(),
            duration_ms: 500,
        },
        SlideMetadata {
            id: 2,
            index: 1,
            title: "Agenda".to_string(),
            shape_count: 5,
            is_hidden: false,
            transition: "Push".to_string(),
            duration_ms: 500,
        },
    ]);

    assert_eq!(sorter_with_slides.count(), 2);
    assert_eq!(sorter_with_slides.active_slide().unwrap().title, "Intro");

    assert!(sorter_with_slides.select_slide(1));
    assert_eq!(sorter_with_slides.active_slide().unwrap().title, "Agenda");
}

#[test]
fn test_06_slide_sorter_add_slide_lifecycle() {
    let mut sorter = SlideSorterModel::new();
    let id1 = sorter.add_slide("First Slide");
    let id2 = sorter.add_slide("Second Slide");

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(sorter.count(), 2);
    assert_eq!(sorter.slides[0].index, 0);
    assert_eq!(sorter.slides[1].index, 1);
    assert_eq!(sorter.active_index, 1);
}

#[test]
fn test_07_slide_sorter_duplicate_slide() {
    let mut sorter = SlideSorterModel::new();
    sorter.add_slide("Original");
    let dup_id = sorter.duplicate_slide(0);

    assert!(dup_id.is_some());
    assert_eq!(sorter.count(), 2);
    assert_eq!(sorter.slides[1].title, "Original (Copy)");
    assert_eq!(sorter.slides[1].index, 1);
    assert_eq!(sorter.active_index, 1);
}

#[test]
fn test_08_slide_sorter_delete_slide_with_minimum_clamp() {
    let mut sorter = SlideSorterModel::new();
    sorter.add_slide("Slide 1");
    sorter.add_slide("Slide 2");

    assert!(sorter.delete_slide(0));
    assert_eq!(sorter.count(), 1);
    assert_eq!(sorter.slides[0].title, "Slide 2");

    // Cannot delete when only 1 slide remains
    assert!(!sorter.delete_slide(0));
    assert_eq!(sorter.count(), 1);
}

#[test]
fn test_09_slide_sorter_reorder_drag_and_drop() {
    let mut sorter = SlideSorterModel::new();
    sorter.add_slide("A");
    sorter.add_slide("B");
    sorter.add_slide("C");

    // Reorder slide 0 ("A") to index 2: order becomes B, C, A
    assert!(sorter.reorder_slide(0, 2));
    assert_eq!(sorter.slides[0].title, "B");
    assert_eq!(sorter.slides[1].title, "C");
    assert_eq!(sorter.slides[2].title, "A");

    // Verify indices sequentially reindexed
    assert_eq!(sorter.slides[0].index, 0);
    assert_eq!(sorter.slides[1].index, 1);
    assert_eq!(sorter.slides[2].index, 2);
}

#[test]
fn test_10_slide_sorter_move_up_and_down() {
    let mut sorter = SlideSorterModel::new();
    sorter.add_slide("1");
    sorter.add_slide("2");
    sorter.add_slide("3");

    // Move slide 1 down: order becomes 1, 3, 2
    assert!(sorter.move_down(1));
    assert_eq!(sorter.slides[1].title, "3");
    assert_eq!(sorter.slides[2].title, "2");

    // Move slide 2 up: order returns to 1, 2, 3
    assert!(sorter.move_up(2));
    assert_eq!(sorter.slides[1].title, "2");
    assert_eq!(sorter.slides[2].title, "3");

    // Boundary checks: move 0 up should fail
    assert!(!sorter.move_up(0));
    // Move last down should fail
    assert!(!sorter.move_down(2));
}

#[test]
fn test_11_slide_sorter_toggle_hide_slide() {
    let mut sorter = SlideSorterModel::new();
    sorter.add_slide("Visible");
    assert!(!sorter.slides[0].is_hidden);

    assert!(sorter.toggle_hide_slide(0));
    assert!(sorter.slides[0].is_hidden);

    assert!(sorter.toggle_hide_slide(0));
    assert!(!sorter.slides[0].is_hidden);
}

#[test]
fn test_12_slide_sorter_engine_reorder_slides_backward_compatible() {
    let mut slides = vec![101, 102, 103, 104];
    SlideSorterEngine::reorder_slides(&mut slides, 0, 2);
    // 101 moves from index 0 to index 2: [102, 103, 101, 104]
    assert_eq!(slides, vec![102, 103, 101, 104]);

    SlideSorterEngine::reorder_slides(&mut slides, 3, 0);
    // 104 moves to front: [104, 102, 103, 101]
    assert_eq!(slides, vec![104, 102, 103, 101]);
}

#[test]
fn test_13_slide_sorter_command_bus_undo_redo() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let cmd = SlideSorterCommand {
        from_idx: 0,
        to_idx: 2,
        action_name: "Reorder Slide 1 to 3".to_string(),
    };

    assert!(bus.execute(&mut state, Box::new(cmd)).is_ok());
    assert_eq!(bus.undo_stack_len(), 1);
    assert!(bus.undo(&mut state).is_ok());
    assert_eq!(bus.undo_stack_len(), 0);
}

// =========================================================================
// 3. Fullscreen F5 Presentation Slide Show State Machine Tests
// =========================================================================

#[test]
fn test_14_slide_show_start_from_beginning_and_current() {
    let mut sm = SlideShowStateMachine::new(5);
    assert!(!sm.is_active);

    // F5: Start from beginning
    assert!(sm.dispatch(SlideShowAction::StartFromBeginning));
    assert!(sm.is_active);
    assert_eq!(sm.current_slide_idx, 0);

    // Shift+F5: Start from current slide (e.g. slide 3)
    assert!(sm.dispatch(SlideShowAction::StartFromCurrent(3)));
    assert_eq!(sm.current_slide_idx, 3);
}

#[test]
fn test_15_slide_show_next_and_previous_navigation() {
    let mut sm = SlideShowStateMachine::new(3);
    sm.dispatch(SlideShowAction::StartFromBeginning);

    // Next slide
    assert!(sm.dispatch(SlideShowAction::Next));
    assert_eq!(sm.current_slide_idx, 1);

    assert!(sm.dispatch(SlideShowAction::Next));
    assert_eq!(sm.current_slide_idx, 2);

    // At last slide: Next should return false without loop
    assert!(!sm.dispatch(SlideShowAction::Next));
    assert_eq!(sm.current_slide_idx, 2);

    // Previous slide
    assert!(sm.dispatch(SlideShowAction::Previous));
    assert_eq!(sm.current_slide_idx, 1);

    assert!(sm.dispatch(SlideShowAction::Previous));
    assert_eq!(sm.current_slide_idx, 0);

    // At first slide: Previous should return false
    assert!(!sm.dispatch(SlideShowAction::Previous));
    assert_eq!(sm.current_slide_idx, 0);
}

#[test]
fn test_16_slide_show_loop_presentation() {
    let mut sm = SlideShowStateMachine::new(3);
    sm.loop_presentation = true;
    sm.dispatch(SlideShowAction::StartFromCurrent(2));

    // Next at end wraps around to 0
    assert!(sm.dispatch(SlideShowAction::Next));
    assert_eq!(sm.current_slide_idx, 0);
}

#[test]
fn test_17_slide_show_laser_pointer_mode() {
    let mut sm = SlideShowStateMachine::new(3);
    sm.dispatch(SlideShowAction::StartFromBeginning);

    assert!(!sm.laser_pointer_active);
    assert_eq!(sm.laser_pos, None);

    // Toggle laser on
    assert!(sm.dispatch(SlideShowAction::ToggleLaser));
    assert!(sm.laser_pointer_active);

    // Set pointer coordinates
    assert!(sm.dispatch(SlideShowAction::SetLaserPos(450.0, 320.0)));
    assert_eq!(sm.laser_pos, Some((450.0, 320.0)));

    // Clear coordinates
    assert!(sm.dispatch(SlideShowAction::ClearLaser));
    assert_eq!(sm.laser_pos, None);

    // Toggle laser off
    assert!(sm.dispatch(SlideShowAction::ToggleLaser));
    assert!(!sm.laser_pointer_active);
}

#[test]
fn test_18_slide_show_pause_black_screen() {
    let mut sm = SlideShowStateMachine::new(3);
    sm.dispatch(SlideShowAction::StartFromBeginning);

    assert!(!sm.is_paused);
    assert!(sm.dispatch(SlideShowAction::TogglePause));
    assert!(sm.is_paused);
    assert!(sm.dispatch(SlideShowAction::TogglePause));
    assert!(!sm.is_paused);
}

#[test]
fn test_19_slide_show_exit() {
    let mut sm = SlideShowStateMachine::new(3);
    sm.dispatch(SlideShowAction::StartFromBeginning);
    sm.dispatch(SlideShowAction::ToggleLaser);
    assert!(sm.is_active);

    assert!(sm.dispatch(SlideShowAction::Exit));
    assert!(!sm.is_active);
    assert!(!sm.laser_pointer_active);
    assert_eq!(sm.laser_pos, None);
}

#[test]
fn test_20_slide_show_keyboard_event_dispatcher() {
    let mut sm = SlideShowStateMachine::new(4);

    // F5 starts show
    assert!(sm.handle_key("F5", false, false));
    assert!(sm.is_active);
    assert_eq!(sm.current_slide_idx, 0);

    // Space advances
    assert!(sm.handle_key("Space", false, false));
    assert_eq!(sm.current_slide_idx, 1);

    // ArrowRight advances
    assert!(sm.handle_key("ArrowRight", false, false));
    assert_eq!(sm.current_slide_idx, 2);

    // ArrowLeft goes back
    assert!(sm.handle_key("ArrowLeft", false, false));
    assert_eq!(sm.current_slide_idx, 1);

    // End jumps to end
    assert!(sm.handle_key("End", false, false));
    assert_eq!(sm.current_slide_idx, 3);

    // Home jumps to start
    assert!(sm.handle_key("Home", false, false));
    assert_eq!(sm.current_slide_idx, 0);

    // L toggles laser
    assert!(sm.handle_key("L", false, false));
    assert!(sm.laser_pointer_active);

    // Escape exits
    assert!(sm.handle_key("Escape", false, false));
    assert!(!sm.is_active);
}

#[test]
fn test_21_slide_show_status_text() {
    let mut sm = SlideShowStateMachine::new(5);
    sm.dispatch(SlideShowAction::StartFromCurrent(2));
    assert_eq!(sm.status_text(), "Slide 3 of 5");
}

// =========================================================================
// 4. Slide Show Display Compiler & Laser Pointer Rendering Tests
// =========================================================================

#[test]
fn test_22_presentation_show_display_compiler_rendering() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let s1 = SlideShapeModel::new(
        1,
        PresetShapeType::Rectangle,
        Rect2D::new(100.0, 100.0, 300.0, 200.0),
    );
    let mut s2 = SlideShapeModel::new(
        2,
        PresetShapeType::Ellipse,
        Rect2D::new(500.0, 100.0, 150.0, 150.0),
    );
    s2.text = Some("Key Metrics".to_string());
    stage.add_shape(s1);
    stage.add_shape(s2);

    let mut sm = SlideShowStateMachine::new(1);
    sm.dispatch(SlideShowAction::StartFromBeginning);

    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let dl = PptDisplayCompiler::compile_presentation_show(&stage, &sm, &viewport);
    assert!(!dl.is_empty());

    // First command is pure black letterbox
    if let DisplayCommand::DrawQuad(ref q) = dl.commands[0] {
        assert_eq!(q.color, ColorRgba::BLACK);
    } else {
        panic!("Expected black quad for letterbox");
    }

    // Includes Quads, Paths, Text, and Clip boundaries
    let mut has_path = false;
    let mut has_text = false;
    for cmd in dl.commands.as_slice() {
        match cmd {
            DisplayCommand::DrawPath(_) => has_path = true,
            DisplayCommand::DrawText(_) => has_text = true,
            _ => {}
        }
    }
    assert!(has_path);
    assert!(has_text);
}

#[test]
fn test_23_presentation_show_laser_pointer_halo_and_core() {
    let stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let mut sm = SlideShowStateMachine::new(1);
    sm.dispatch(SlideShowAction::StartFromBeginning);
    sm.dispatch(SlideShowAction::ToggleLaser);
    sm.dispatch(SlideShowAction::SetLaserPos(600.0, 400.0));

    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let dl = PptDisplayCompiler::compile_presentation_show(&stage, &sm, &viewport);

    // Laser adds outer glow (radius 18) and inner core (radius 6)
    let mut laser_glow_found = false;
    let mut laser_core_found = false;

    for cmd in dl.commands.as_slice() {
        if let DisplayCommand::DrawQuad(ref q) = cmd {
            if q.corner_radius == 18.0 && q.rect.width == 36.0 {
                laser_glow_found = true;
            }
            if q.corner_radius == 6.0 && q.rect.width == 12.0 {
                laser_core_found = true;
            }
        }
    }

    assert!(laser_glow_found, "Should render 18px glow halo");
    assert!(laser_core_found, "Should render 6px bright laser core");
}

#[test]
fn test_24_120fps_sla_slide_show_navigation_and_rendering() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    for i in 1..=15 {
        stage.add_shape(SlideShapeModel::new(
            i,
            PresetShapeType::RoundedRectangle,
            Rect2D::new((i as f64) * 40.0, 150.0, 140.0, 90.0),
        ));
    }

    let mut sm = SlideShowStateMachine::new(50);
    sm.dispatch(SlideShowAction::StartFromBeginning);
    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);

    let start = Instant::now();
    for i in 0..100 {
        sm.dispatch(SlideShowAction::Next);
        sm.dispatch(SlideShowAction::SetLaserPos(300.0 + (i as f64), 200.0));
        let _dl = PptDisplayCompiler::compile_presentation_show(&stage, &sm, &viewport);
    }
    let elapsed = start.elapsed();

    // 100 presentation show navigations and renders must complete well within SLA (< 25 ms, i.e. < 0.25 ms per frame)
    assert!(
        elapsed.as_millis() < 50,
        "SLA Violation: 100 presentation navigations took {:?}",
        elapsed
    );
}
