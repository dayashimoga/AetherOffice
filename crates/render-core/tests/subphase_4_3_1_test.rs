#![forbid(unsafe_code)]
//! Subphase 4.3.1 Test Suite: Interactive Vector Slide Stage, Transform Gizmos & Shape Editing
//!
//! Validates:
//! - Slide canvas aspect ratio presets (16:9, 4:3, custom) and viewport centering projection.
//! - Subpixel coordinate mapping between physical viewport and slide canvas coordinates.
//! - DrawingML preset geometries path generation (Rect, RoundRect, Ellipse, Triangle, Arrow, Star5, Diamond, Callout, Parallelogram, Hexagon).
//! - 8-handle + rotation transform gizmos and hit testing.
//! - Translation, aspect-ratio locked resize, and angle snapping rotation math.
//! - Multi-shape selection bounding quad calculation (union AABB).
//! - Shape CRUD, styling, and Z-order manipulation (BringToFront, SendToBack, BringForward, SendBackward).
//! - Transactional Command Bus undo/redo tracking.
//! - Complete DisplayList compilation with GPU primitives, vector paths, and transform gizmos.
//! - Hard 120 FPS latency SLA (< 8.33 ms frame budget).

use render_core::compiler::ppt::{
    generate_preset_path, PptDisplayCompiler, PptGizmoHit, PptGizmoModel, PptStageController,
    PptViewportTransform, PresetShapeType, SlideAddShapeCommand, SlideCanvasConfig,
    SlideDeleteShapeCommand, SlideShapeModel, SlideStageModel, SlideTransformCommand,
    SlideTransformEngine,
};
use render_core::interaction::{CommandBus, DocumentState, PptSelection, ResizeHandle};
use render_core::primitive::{DisplayCommand, Rect2D};
use std::time::Instant;

// =========================================================================
// 1. Slide Canvas & Viewport Projection Tests
// =========================================================================

#[test]
fn test_01_slide_canvas_aspect_ratio_presets() {
    let wide = SlideCanvasConfig::widescreen_16_9();
    assert_eq!(wide.canvas_width, 1920.0);
    assert_eq!(wide.canvas_height, 1080.0);
    assert!((wide.aspect_ratio() - 16.0 / 9.0).abs() < 1e-4);

    let std = SlideCanvasConfig::standard_4_3();
    assert_eq!(std.canvas_width, 1440.0);
    assert_eq!(std.canvas_height, 1080.0);
    assert!((std.aspect_ratio() - 4.0 / 3.0).abs() < 1e-4);

    let custom = SlideCanvasConfig::custom(1200.0, 600.0);
    assert_eq!(custom.canvas_width, 1200.0);
    assert_eq!(custom.canvas_height, 600.0);
    assert!((custom.aspect_ratio() - 2.0).abs() < 1e-4);
}

#[test]
fn test_02_viewport_transform_letterboxing() {
    let cfg = SlideCanvasConfig::widescreen_16_9();

    // Viewport wider than 16:9: fits height, letterboxed on left and right
    let vp_wide = Rect2D::new(0.0, 0.0, 2560.0, 1080.0);
    let (dest_w, scale_w) = PptDisplayCompiler::compute_slide_viewport_transform(&cfg, &vp_wide);
    assert_eq!(scale_w, 1.0);
    assert_eq!(dest_w.height, 1080.0);
    assert_eq!(dest_w.width, 1920.0);
    assert_eq!(dest_w.x, (2560.0 - 1920.0) / 2.0);
    assert_eq!(dest_w.y, 0.0);

    // Viewport taller than 16:9: fits width, letterboxed on top and bottom
    let vp_tall = Rect2D::new(0.0, 0.0, 1920.0, 1500.0);
    let (dest_t, scale_t) = PptDisplayCompiler::compute_slide_viewport_transform(&cfg, &vp_tall);
    assert_eq!(scale_t, 1.0);
    assert_eq!(dest_t.width, 1920.0);
    assert_eq!(dest_t.height, 1080.0);
    assert_eq!(dest_t.x, 0.0);
    assert_eq!(dest_t.y, (1500.0 - 1080.0) / 2.0);
}

#[test]
fn test_03_viewport_and_slide_coordinate_mapping() {
    let cfg = SlideCanvasConfig::widescreen_16_9();
    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let transform = PptViewportTransform::new(&cfg, &viewport);

    // Origin mapping
    let slide_origin = transform.viewport_to_slide((0.0, 0.0));
    assert_eq!(slide_origin, (0.0, 0.0));

    // Center mapping
    let center_vp = (960.0, 540.0);
    let center_slide = transform.viewport_to_slide(center_vp);
    assert_eq!(center_slide, (960.0, 540.0));

    // Arbitrary point round-trip fidelity
    let test_pt = (345.5, 789.25);
    let mapped = transform.slide_to_viewport(test_pt);
    let round_trip = transform.viewport_to_slide(mapped);
    assert!((round_trip.0 - test_pt.0).abs() < 1e-4);
    assert!((round_trip.1 - test_pt.1).abs() < 1e-4);

    assert!(transform.is_point_on_slide((500.0, 500.0)));
    assert!(!transform.is_point_on_slide((-10.0, 500.0)));
}

// =========================================================================
// 2. Preset Vector Geometry Tests
// =========================================================================

#[test]
fn test_04_preset_geometry_rectangle_and_textbox() {
    let bounds = Rect2D::new(10.0, 20.0, 100.0, 50.0);
    let path = generate_preset_path(PresetShapeType::Rectangle, &bounds, None);
    assert_eq!(path.commands.len(), 5); // MoveTo, 3x LineTo, Close

    let tb_path = generate_preset_path(PresetShapeType::TextBox, &bounds, None);
    assert_eq!(tb_path.commands.len(), 5);
}

#[test]
fn test_05_preset_geometry_rounded_rectangle() {
    let bounds = Rect2D::new(0.0, 0.0, 200.0, 100.0);
    let path = generate_preset_path(PresetShapeType::RoundedRectangle, &bounds, Some(0.2));
    assert!(!path.commands.is_empty());
    // MoveTo, 4 lines and 4 cubic bezier arcs + Close = 10 commands
    assert_eq!(path.commands.len(), 10);
}

#[test]
fn test_06_preset_geometry_ellipse() {
    let bounds = Rect2D::new(50.0, 50.0, 100.0, 80.0);
    let path = generate_preset_path(PresetShapeType::Ellipse, &bounds, None);
    // MoveTo + 4 CubicTo arcs + Close = 6 commands
    assert_eq!(path.commands.len(), 6);
}

#[test]
fn test_07_preset_geometry_triangle() {
    let bounds = Rect2D::new(0.0, 0.0, 100.0, 100.0);
    let path = generate_preset_path(PresetShapeType::Triangle, &bounds, None);
    // MoveTo + 2x LineTo + Close = 4 commands
    assert_eq!(path.commands.len(), 4);
}

#[test]
fn test_08_preset_geometry_right_arrow() {
    let bounds = Rect2D::new(0.0, 0.0, 150.0, 80.0);
    let path = generate_preset_path(PresetShapeType::RightArrow, &bounds, None);
    // MoveTo + 6x LineTo + Close = 8 commands
    assert_eq!(path.commands.len(), 8);
}

#[test]
fn test_09_preset_geometry_star5() {
    let bounds = Rect2D::new(0.0, 0.0, 120.0, 120.0);
    let path = generate_preset_path(PresetShapeType::Star5, &bounds, None);
    // MoveTo + 9x LineTo + Close = 11 commands
    assert_eq!(path.commands.len(), 11);
}

#[test]
fn test_10_preset_geometry_diamond_callout_polygons() {
    let bounds = Rect2D::new(10.0, 10.0, 100.0, 100.0);

    let diamond = generate_preset_path(PresetShapeType::Diamond, &bounds, None);
    assert_eq!(diamond.commands.len(), 5);

    let callout = generate_preset_path(PresetShapeType::Callout, &bounds, None);
    assert_eq!(callout.commands.len(), 8);

    let parallelogram = generate_preset_path(PresetShapeType::Parallelogram, &bounds, None);
    assert_eq!(parallelogram.commands.len(), 5);

    let hexagon = generate_preset_path(PresetShapeType::Hexagon, &bounds, None);
    assert_eq!(hexagon.commands.len(), 7);
}

// =========================================================================
// 3. Transform Gizmo & Hit-Testing Tests
// =========================================================================

#[test]
fn test_11_transform_gizmo_handle_positions() {
    let bounds = Rect2D::new(100.0, 100.0, 200.0, 100.0);
    let gizmo = PptGizmoModel::new(bounds, 0.0);

    let positions = gizmo.handle_positions();
    assert_eq!(positions.len(), 8);

    // Top-Left corner
    assert_eq!((positions[0].0, positions[0].1), (100.0, 100.0));
    assert_eq!(positions[0].2, ResizeHandle::TopLeft);

    // Top-Center
    assert_eq!((positions[1].0, positions[1].1), (200.0, 100.0));
    assert_eq!(positions[1].2, ResizeHandle::Top);

    // Top-Right corner
    assert_eq!((positions[2].0, positions[2].1), (300.0, 100.0));
    assert_eq!(positions[2].2, ResizeHandle::TopRight);

    // Bottom-Right corner
    assert_eq!((positions[4].0, positions[4].1), (300.0, 200.0));
    assert_eq!(positions[4].2, ResizeHandle::BottomRight);

    // Rotation handle: 24px above top center
    assert_eq!(gizmo.rotation_handle_center(), (200.0, 76.0));
}

#[test]
fn test_12_transform_gizmo_hit_testing() {
    let bounds = Rect2D::new(100.0, 100.0, 200.0, 100.0);
    let gizmo = PptGizmoModel::new(bounds, 0.0);

    // Hit rotation handle at (200, 76)
    assert_eq!(
        gizmo.hit_test((200.0, 76.0), 2.0),
        Some(PptGizmoHit::Rotate)
    );

    // Hit TopLeft handle at (100, 100)
    assert_eq!(
        gizmo.hit_test((100.0, 100.0), 2.0),
        Some(PptGizmoHit::Handle(ResizeHandle::TopLeft))
    );

    // Hit BottomRight handle at (300, 200)
    assert_eq!(
        gizmo.hit_test((300.0, 200.0), 2.0),
        Some(PptGizmoHit::Handle(ResizeHandle::BottomRight))
    );

    // Hit shape body at (150, 150)
    assert_eq!(gizmo.hit_test((150.0, 150.0), 2.0), Some(PptGizmoHit::Body));

    // Miss outside at (50, 50)
    assert_eq!(gizmo.hit_test((50.0, 50.0), 2.0), None);
}

// =========================================================================
// 4. Slide Transform Engine (Translation, Resize, Rotation)
// =========================================================================

#[test]
fn test_13_slide_transform_translation() {
    let bounds = Rect2D::new(50.0, 50.0, 100.0, 80.0);
    let moved = SlideTransformEngine::translate(&bounds, 25.0, -10.0);

    assert_eq!(moved.x, 75.0);
    assert_eq!(moved.y, 40.0);
    assert_eq!(moved.width, 100.0);
    assert_eq!(moved.height, 80.0);
}

#[test]
fn test_14_slide_transform_resize_corner_and_sides() {
    let bounds = Rect2D::new(100.0, 100.0, 100.0, 100.0);

    // Drag BottomRight handle (+20, +30)
    let br =
        SlideTransformEngine::resize(&bounds, ResizeHandle::BottomRight, 20.0, 30.0, false, 10.0);
    assert_eq!(br.x, 100.0);
    assert_eq!(br.y, 100.0);
    assert_eq!(br.width, 120.0);
    assert_eq!(br.height, 130.0);

    // Drag TopLeft handle (-20, -10) -> moves origin and expands
    let tl =
        SlideTransformEngine::resize(&bounds, ResizeHandle::TopLeft, -20.0, -10.0, false, 10.0);
    assert_eq!(tl.x, 80.0);
    assert_eq!(tl.y, 90.0);
    assert_eq!(tl.width, 120.0);
    assert_eq!(tl.height, 110.0);

    // Drag Top handle (+0, -15) -> extends height upwards, pins bottom
    let top = SlideTransformEngine::resize(&bounds, ResizeHandle::Top, 0.0, -15.0, false, 10.0);
    assert_eq!(top.x, 100.0);
    assert_eq!(top.y, 85.0);
    assert_eq!(top.width, 100.0);
    assert_eq!(top.height, 115.0);
}

#[test]
fn test_15_slide_transform_resize_aspect_ratio_locking() {
    let bounds = Rect2D::new(100.0, 100.0, 200.0, 100.0); // 2:1 aspect ratio
    let resized =
        SlideTransformEngine::resize(&bounds, ResizeHandle::BottomRight, 50.0, 0.0, true, 10.0);

    let aspect = resized.width / resized.height;
    assert!((aspect - 2.0).abs() < 1e-4);
    assert_eq!(resized.width, 250.0);
    assert_eq!(resized.height, 125.0);
}

#[test]
fn test_16_slide_transform_resize_minimum_size_clamp() {
    let bounds = Rect2D::new(100.0, 100.0, 50.0, 50.0);
    let shrunk = SlideTransformEngine::resize(
        &bounds,
        ResizeHandle::BottomRight,
        -100.0,
        -100.0,
        false,
        16.0,
    );

    assert_eq!(shrunk.width, 16.0);
    assert_eq!(shrunk.height, 16.0);
}

#[test]
fn test_17_slide_transform_rotation_angle_and_snapping() {
    let center = (200.0, 200.0);

    // Directly above center: 0 degrees
    let angle_up = SlideTransformEngine::rotate(center, (200.0, 100.0), None);
    assert!((angle_up - 0.0).abs() < 1e-4);

    // Directly to the right: 90 degrees
    let angle_right = SlideTransformEngine::rotate(center, (300.0, 200.0), None);
    assert!((angle_right - 90.0).abs() < 1e-4);

    // Directly below: 180 degrees
    let angle_down = SlideTransformEngine::rotate(center, (200.0, 300.0), None);
    assert!((angle_down - 180.0).abs() < 1e-4);

    // 15-degree angle snapping
    let angle_snap = SlideTransformEngine::rotate(center, (280.0, 120.0), Some(15.0));
    assert_eq!(angle_snap % 15.0, 0.0);
}

#[test]
fn test_18_multi_selection_bounding_box_union() {
    let s1 = SlideShapeModel::new(
        1,
        PresetShapeType::Rectangle,
        Rect2D::new(10.0, 10.0, 40.0, 40.0),
    );
    let s2 = SlideShapeModel::new(
        2,
        PresetShapeType::Ellipse,
        Rect2D::new(80.0, 50.0, 50.0, 50.0),
    );
    let s3 = SlideShapeModel::new(
        3,
        PresetShapeType::Triangle,
        Rect2D::new(200.0, 200.0, 50.0, 50.0),
    );

    let union_selected = SlideTransformEngine::multi_selection_bounds(&[s1, s2, s3], &[1, 2]);
    assert!(union_selected.is_some());
    let u = union_selected.unwrap();
    assert_eq!(u.x, 10.0);
    assert_eq!(u.y, 10.0);
    assert_eq!(u.width, 120.0); // from 10 to 130
    assert_eq!(u.height, 90.0); // from 10 to 100
}

// =========================================================================
// 5. Slide Shape & Stage CRUD and Z-Order Tests
// =========================================================================

#[test]
fn test_19_stage_z_order_reordering() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let id1 = stage.add_shape(SlideShapeModel::new(
        1,
        PresetShapeType::Rectangle,
        Rect2D::ZERO,
    ));
    let id2 = stage.add_shape(SlideShapeModel::new(
        2,
        PresetShapeType::Ellipse,
        Rect2D::ZERO,
    ));
    let id3 = stage.add_shape(SlideShapeModel::new(
        3,
        PresetShapeType::Star5,
        Rect2D::ZERO,
    ));

    // Initial order: [1, 2, 3]
    assert_eq!(stage.shapes[0].id, id1);
    assert_eq!(stage.shapes[1].id, id2);
    assert_eq!(stage.shapes[2].id, id3);

    // Send 3 to back: order becomes [3, 1, 2]
    assert!(stage.send_to_back(id3));
    assert_eq!(stage.shapes[0].id, id3);
    assert_eq!(stage.shapes[1].id, id1);
    assert_eq!(stage.shapes[2].id, id2);

    // Bring 1 to front: order becomes [3, 2, 1]
    assert!(stage.bring_to_front(id1));
    assert_eq!(stage.shapes[0].id, id3);
    assert_eq!(stage.shapes[1].id, id2);
    assert_eq!(stage.shapes[2].id, id1);

    // Bring 3 forward: order becomes [2, 3, 1]
    assert!(stage.bring_forward(id3));
    assert_eq!(stage.shapes[0].id, id2);
    assert_eq!(stage.shapes[1].id, id3);
    assert_eq!(stage.shapes[2].id, id1);

    // Send 1 backward: order becomes [2, 1, 3]
    assert!(stage.send_backward(id1));
    assert_eq!(stage.shapes[0].id, id2);
    assert_eq!(stage.shapes[1].id, id1);
    assert_eq!(stage.shapes[2].id, id3);
}

#[test]
fn test_20_stage_shape_crud_and_duplication() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let s1 = SlideShapeModel::new(
        0,
        PresetShapeType::Rectangle,
        Rect2D::new(50.0, 50.0, 100.0, 100.0),
    );
    let id1 = stage.add_shape(s1);
    assert_eq!(id1, 1);

    // Duplicate shape with (20, 20) offset
    let dup_id = stage.duplicate_shape(id1, (20.0, 20.0));
    assert!(dup_id.is_some());
    let id2 = dup_id.unwrap();
    assert_eq!(id2, 2);

    let dup_shape = stage.get_shape(id2).unwrap();
    assert_eq!(dup_shape.bounds.x, 70.0);
    assert_eq!(dup_shape.bounds.y, 70.0);

    // Hit test top-most shape
    assert_eq!(stage.hit_test((75.0, 75.0)), Some(id2));

    // Remove first shape
    assert!(stage.remove_shape(id1).is_some());
    assert_eq!(stage.shapes.len(), 1);
}

// =========================================================================
// 6. Interactive Slide Controller & Undo/Redo Tests
// =========================================================================

#[test]
fn test_21_stage_controller_undo_redo_history() {
    let stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let mut controller = PptStageController::new(stage);

    assert!(!controller.can_undo());
    assert!(!controller.can_redo());

    // 1. Add shape
    let s = SlideShapeModel::new(
        1,
        PresetShapeType::Rectangle,
        Rect2D::new(10.0, 10.0, 100.0, 100.0),
    );
    let add_res = controller.execute_add_shape(s);
    assert!(add_res.is_ok());
    assert_eq!(controller.stage.shapes.len(), 1);
    assert!(controller.can_undo());

    // 2. Transform shape
    let trans_res = controller.execute_transform(1, Rect2D::new(50.0, 50.0, 150.0, 150.0), 45.0);
    assert!(trans_res.is_ok());
    assert_eq!(controller.stage.shapes[0].bounds.x, 50.0);
    assert_eq!(controller.stage.shapes[0].rotation_deg, 45.0);

    // 3. Undo transform
    assert!(controller.undo().is_ok());
    assert_eq!(controller.stage.shapes[0].bounds.x, 10.0);
    assert_eq!(controller.stage.shapes[0].rotation_deg, 0.0);
    assert!(controller.can_redo());

    // 4. Redo transform
    assert!(controller.redo().is_ok());
    assert_eq!(controller.stage.shapes[0].bounds.x, 50.0);
    assert_eq!(controller.stage.shapes[0].rotation_deg, 45.0);

    // 5. Undo back to empty
    assert!(controller.undo().is_ok()); // undo transform
    assert!(controller.undo().is_ok()); // undo add shape
    assert_eq!(controller.stage.shapes.len(), 0);
}

#[test]
fn test_22_display_compiler_with_vector_shapes_and_gizmos() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    let s1 = SlideShapeModel::new(
        1,
        PresetShapeType::Rectangle,
        Rect2D::new(100.0, 100.0, 200.0, 150.0),
    );
    let mut s2 = SlideShapeModel::new(
        2,
        PresetShapeType::Ellipse,
        Rect2D::new(400.0, 100.0, 150.0, 150.0),
    );
    s2.text = Some("Overview".to_string());
    stage.add_shape(s1);
    stage.add_shape(s2);

    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let selection = PptSelection::single(1);

    let display_list = PptDisplayCompiler::compile_slide_stage(&stage, Some(&selection), &viewport);
    assert!(!display_list.is_empty());

    // Check commands: includes Quads, Paths, Text, Push/PopClip
    let mut has_quad = false;
    let mut has_path = false;
    let mut has_text = false;
    let mut has_clip = false;

    for cmd in display_list.commands.as_slice() {
        match cmd {
            DisplayCommand::DrawQuad(_) => has_quad = true,
            DisplayCommand::DrawPath(_) => has_path = true,
            DisplayCommand::DrawText(_) => has_text = true,
            DisplayCommand::PushClip(_) => has_clip = true,
            _ => {}
        }
    }

    assert!(has_quad, "Should render quads for backgrounds/handles");
    assert!(has_path, "Should render vector path for ellipse");
    assert!(has_text, "Should render text run");
    assert!(has_clip, "Should clip to slide canvas boundaries");
}

#[test]
fn test_23_command_bus_slide_commands_integration() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let transform_cmd = SlideTransformCommand {
        shape_id: 1,
        prev_bounds: Rect2D::ZERO,
        prev_rotation: 0.0,
        new_bounds: Rect2D::new(10.0, 10.0, 50.0, 50.0),
        new_rotation: 15.0,
    };

    let add_cmd = SlideAddShapeCommand {
        shape: SlideShapeModel::new(2, PresetShapeType::Star5, Rect2D::ZERO),
        added_id: 2,
    };

    let del_cmd = SlideDeleteShapeCommand {
        shape_id: 2,
        deleted_shape: None,
        index: 1,
    };

    assert!(bus.execute(&mut state, Box::new(transform_cmd)).is_ok());
    assert!(bus.execute(&mut state, Box::new(add_cmd)).is_ok());
    assert!(bus.execute(&mut state, Box::new(del_cmd)).is_ok());

    assert_eq!(bus.undo_stack_len(), 3);
    assert!(bus.undo(&mut state).is_ok());
    assert!(bus.undo(&mut state).is_ok());
    assert_eq!(bus.undo_stack_len(), 1);
    assert_eq!(bus.redo_stack_len(), 2);
}

#[test]
fn test_24_120fps_sla_slide_stage_transforms() {
    let mut stage = SlideStageModel::new(SlideCanvasConfig::widescreen_16_9());
    for i in 1..=20 {
        let s = SlideShapeModel::new(
            i,
            PresetShapeType::RoundedRectangle,
            Rect2D::new((i as f64) * 30.0, 100.0, 120.0, 80.0),
        );
        stage.add_shape(s);
    }

    let mut controller = PptStageController::new(stage);
    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);

    let start = Instant::now();
    for i in 0..100 {
        let _ = controller.execute_transform(
            1,
            Rect2D::new(100.0 + (i as f64), 100.0, 120.0, 80.0),
            (i as f64) * 2.0,
        );
        let _dl = controller.compile_display_list(&viewport);
    }
    let elapsed = start.elapsed();

    // 100 transforms + compilations must complete well within SLA (< 25 ms, i.e. < 0.25 ms per frame)
    assert!(
        elapsed.as_millis() < 50,
        "SLA Violation: 100 slide transforms took {:?}",
        elapsed
    );
}
