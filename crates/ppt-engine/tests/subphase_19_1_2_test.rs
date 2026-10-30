//! Subphase 19.1.2 Integration Tests: 3D Shader Transitions & Keyframe Motion Paths.

use ppt_engine::transition_3d::{
    EasingCurve, MotionPathCompiler, SlideTransition3DEngine, Transform3DMatrix,
    Transition3DDirection, Transition3DEffect,
};

#[test]
fn test_01_transform_matrix_identity_and_primitives() {
    let id = Transform3DMatrix::identity();
    assert_eq!(id.m[0], 1.0);
    assert_eq!(id.m[5], 1.0);
    assert_eq!(id.m[10], 1.0);
    assert_eq!(id.m[15], 1.0);
    assert_eq!(id.m[1], 0.0);

    let trans = Transform3DMatrix::translation(100.0, -50.0, 20.0);
    assert_eq!(trans.m[12], 100.0);
    assert_eq!(trans.m[13], -50.0);
    assert_eq!(trans.m[14], 20.0);

    let scale = Transform3DMatrix::scale(2.0, 3.0, 0.5);
    assert_eq!(scale.m[0], 2.0);
    assert_eq!(scale.m[5], 3.0);
    assert_eq!(scale.m[10], 0.5);

    let css = id.to_css_matrix3d();
    assert_eq!(
        css,
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)"
    );
}

#[test]
fn test_02_matrix_multiplication_composition() {
    let t = Transform3DMatrix::translation(10.0, 20.0, 30.0);
    let s = Transform3DMatrix::scale(2.0, 2.0, 2.0);

    // Compound transform
    let compound = t.multiply(&s);
    // When multiplying translation with scale, translation vector remains intact
    assert_eq!(compound.m[0], 2.0);
    assert_eq!(compound.m[5], 2.0);
    assert_eq!(compound.m[10], 2.0);
    assert_eq!(compound.m[12], 10.0);
    assert_eq!(compound.m[13], 20.0);
    assert_eq!(compound.m[14], 30.0);
}

#[test]
fn test_03_transition_3d_effect_enums() {
    let effects = [
        ("cube", Transition3DEffect::Cube, "Cube"),
        ("flip", Transition3DEffect::Flip, "Flip"),
        ("vortex", Transition3DEffect::Vortex, "Vortex"),
        ("honeycomb", Transition3DEffect::Honeycomb, "Honeycomb"),
        ("origami", Transition3DEffect::Origami, "Origami"),
        ("ripple", Transition3DEffect::Ripple, "Ripple"),
    ];

    for (name, expected_enum, display) in effects {
        assert_eq!(Transition3DEffect::from_str_name(name), Some(expected_enum));
        assert_eq!(expected_enum.as_str_name(), display);
    }

    assert_eq!(Transition3DEffect::from_str_name("non_existent"), None);
}

#[test]
fn test_04_cube_transition_transform_evaluation() {
    // Progress t = 0.0: outgoing at 0 deg, incoming at 90 deg
    let frame_start = SlideTransition3DEngine::evaluate_transition(
        Transition3DEffect::Cube,
        Transition3DDirection::Right,
        0.0,
        1920.0,
        1080.0,
    );
    assert_eq!(frame_start.outgoing_opacity, 1.0);
    assert_eq!(frame_start.incoming_opacity, 0.8);

    // Progress t = 1.0: outgoing rotated out, incoming fully entered (opacity 1.0)
    let frame_end = SlideTransition3DEngine::evaluate_transition(
        Transition3DEffect::Cube,
        Transition3DDirection::Right,
        1.0,
        1920.0,
        1080.0,
    );
    assert_eq!(frame_end.outgoing_opacity, 0.8);
    assert_eq!(frame_end.incoming_opacity, 1.0);
}

#[test]
fn test_05_flip_transition_card_flip() {
    let frame_early = SlideTransition3DEngine::evaluate_transition(
        Transition3DEffect::Flip,
        Transition3DDirection::Right,
        0.2,
        1000.0,
        600.0,
    );
    // Before halfway point (t < 0.5), outgoing is visible, incoming is invisible
    assert_eq!(frame_early.outgoing_opacity, 1.0);
    assert_eq!(frame_early.incoming_opacity, 0.0);

    let frame_late = SlideTransition3DEngine::evaluate_transition(
        Transition3DEffect::Flip,
        Transition3DDirection::Right,
        0.8,
        1000.0,
        600.0,
    );
    // After halfway point (t >= 0.5), outgoing is invisible, incoming is visible
    assert_eq!(frame_late.outgoing_opacity, 0.0);
    assert_eq!(frame_late.incoming_opacity, 1.0);
}

#[test]
fn test_06_vortex_spiral_depth_transition() {
    let frame_mid = SlideTransition3DEngine::evaluate_transition(
        Transition3DEffect::Vortex,
        Transition3DDirection::Right,
        0.5,
        1920.0,
        1080.0,
    );
    assert_eq!(frame_mid.outgoing_opacity, 0.5);
    assert_eq!(frame_mid.incoming_opacity, 0.5);

    // Both transforms should contain non-zero matrix elements representing rotation & depth
    assert!(frame_mid.outgoing_transform.m[0].abs() > 0.0);
    assert!(frame_mid.incoming_transform.m[0].abs() > 0.0);
}

#[test]
fn test_07_easing_curve_evaluations() {
    let linear = EasingCurve::Linear;
    assert_eq!(linear.evaluate(0.0), 0.0);
    assert_eq!(linear.evaluate(0.5), 0.5);
    assert_eq!(linear.evaluate(1.0), 1.0);

    let ease_in_out = EasingCurve::EaseInOut;
    assert_eq!(ease_in_out.evaluate(0.0), 0.0);
    assert!((ease_in_out.evaluate(0.5) - 0.5).abs() < 0.05);
    assert_eq!(ease_in_out.evaluate(1.0), 1.0);

    let bounce = EasingCurve::Bounce;
    assert_eq!(bounce.evaluate(0.0), 0.0);
    assert!((bounce.evaluate(1.0) - 1.0).abs() < 1e-4);

    let spring = EasingCurve::Spring {
        stiffness: 100.0,
        damping: 10.0,
    };
    assert_eq!(spring.evaluate(0.0), 0.0);
    // At t = 1.0, spring settles near 1.0
    assert!((spring.evaluate(1.0) - 1.0).abs() < 0.05);
}

#[test]
fn test_08_motion_path_compiler_linear_and_bezier() {
    // Linear path from (0, 0) to (200, 100)
    let path = "M 0 0 L 200 100";
    let trajectory = MotionPathCompiler::compile_trajectory(path, &EasingCurve::Linear, 5, true);

    assert_eq!(trajectory.len(), 5);
    assert_eq!(trajectory[0].x, 0.0);
    assert_eq!(trajectory[0].y, 0.0);
    assert_eq!(trajectory[4].x, 200.0);
    assert_eq!(trajectory[4].y, 100.0);

    // Tangent angle should be consistent along straight line: atan2(100, 200) in degrees (~26.565)
    let expected_angle = (100.0_f64).atan2(200.0).to_degrees();
    assert!((trajectory[2].tangent_angle_deg - expected_angle).abs() < 0.1);
}

#[test]
fn test_09_motion_path_cubic_bezier_sampling() {
    // S-curve cubic bezier: M 0 0 C 100 0 100 200 200 200
    let path = "M 0 0 C 100 0 100 200 200 200";
    let trajectory =
        MotionPathCompiler::compile_trajectory(path, &EasingCurve::EaseInOut, 11, true);

    assert_eq!(trajectory.len(), 11);
    assert_eq!(trajectory[0].x, 0.0);
    assert_eq!(trajectory[0].y, 0.0);
    assert!((trajectory[10].x - 200.0).abs() < 1e-3);
    assert!((trajectory[10].y - 200.0).abs() < 1e-3);
}

#[test]
fn test_10_motion_path_css_keyframes_output() {
    let path = "M 0 0 L 50 50 L 100 0";
    let trajectory = MotionPathCompiler::compile_trajectory(path, &EasingCurve::Linear, 3, false);

    let css = MotionPathCompiler::to_css_keyframes("glideMotion", &trajectory);
    assert!(css.contains("@keyframes glideMotion {"));
    assert!(css.contains("0% { transform: translate(0.0px, 0.0px) rotate(0.0deg); }"));
    assert!(css.contains("100% { transform: translate(100.0px, 0.0px) rotate(0.0deg); }"));
}
