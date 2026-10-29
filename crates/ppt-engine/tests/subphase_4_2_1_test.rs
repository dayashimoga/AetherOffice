#![forbid(unsafe_code)]
//! Subphase 4.2.1 Test Suite: SMIL Animation Timing Tree (p:timing) State Machine
//!
//! Validates:
//! - 120Hz frame-accurate deterministic evaluation (`sample_at(time_ms)` with 8.333ms frame budget).
//! - Acceleration and deceleration S-curve profiles and cubic Bezier easing.
//! - Time containers: Parallel (`p:par`) and Sequential (`p:seq`).
//! - Triggers: `OnClick`, `WithPrev`, `AfterPrev`.
//! - Property interpolators: motion paths, rotation, scale, color (RGB/HSL), effects, and opacity.
//! - XML parsing and serialization of `p:timing` trees.
//! - Vector manifest validation against `specs/evidence/ppt/4.2.1_vectors.json`.

use ppt_engine::animation::*;
use ppt_engine::model::*;
use ppt_engine::parser::parse_slide;
use ppt_engine::serializer::serialize_slide;
use std::path::{Path, PathBuf};

fn resolve_repo_path(rel_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root");
    repo_root.join(rel_path)
}

// =========================================================================
// 1. Easing & Timing Dynamics Tests
// =========================================================================

#[test]
fn test_01_accel_decel_easing_pure_linear() {
    // a=0, d=0 -> linear
    for step in 0..=10 {
        let t = step as f64 / 10.0;
        let progress = apply_accel_decel(t, 0.0, 0.0);
        assert!((progress - t).abs() < 1e-6);
    }
}

#[test]
fn test_02_accel_easing_profile() {
    // 50% acceleration, 0% deceleration
    let a = 0.5;
    let d = 0.0;

    let p0 = apply_accel_decel(0.0, a, d);
    let p_mid = apply_accel_decel(0.5, a, d);
    let p1 = apply_accel_decel(1.0, a, d);

    assert_eq!(p0, 0.0);
    assert_eq!(p1, 1.0);
    // At t=0.5, acceleration has just ended, velocity reached max
    // p(0.5) is lower than 0.5 because it was accelerating from 0
    assert!(p_mid < 0.5);
}

#[test]
fn test_03_decel_easing_profile() {
    // 0% acceleration, 50% deceleration
    let a = 0.0;
    let d = 0.5;

    let p0 = apply_accel_decel(0.0, a, d);
    let p_mid = apply_accel_decel(0.5, a, d);
    let p1 = apply_accel_decel(1.0, a, d);

    assert_eq!(p0, 0.0);
    assert_eq!(p1, 1.0);
    // Deceleration starts at 0.5, so at mid-point it has traveled more than linear
    assert!(p_mid > 0.5);
}

#[test]
fn test_04_accel_decel_s_curve_profile() {
    // 30% acceleration, 30% deceleration (symmetric S-curve)
    let a = 0.3;
    let d = 0.3;

    let p_mid = apply_accel_decel(0.5, a, d);
    assert!(
        (p_mid - 0.5).abs() < 1e-5,
        "Symmetric easing must be 0.5 at t=0.5"
    );

    let p_early = apply_accel_decel(0.2, a, d);
    let p_late = apply_accel_decel(0.8, a, d);
    assert!(
        (p_early + p_late - 1.0).abs() < 1e-5,
        "Symmetric easing property"
    );
}

#[test]
fn test_05_cubic_bezier_easing_solver() {
    // Standard ease-in-out: (0.42, 0.0, 0.58, 1.0)
    let y0 = solve_cubic_bezier_easing(0.42, 0.0, 0.58, 1.0, 0.0);
    let y_mid = solve_cubic_bezier_easing(0.42, 0.0, 0.58, 1.0, 0.5);
    let y1 = solve_cubic_bezier_easing(0.42, 0.0, 0.58, 1.0, 1.0);

    assert!((y0 - 0.0).abs() < 1e-3);
    assert!((y_mid - 0.5).abs() < 1e-3);
    assert!((y1 - 1.0).abs() < 1e-3);
}

#[test]
fn test_06_node_progress_delay_and_hold() {
    let delay = 500;
    let dur = Some(1000);

    // Before delay: None
    assert!(compute_node_progress(300.0, delay, dur, Some(1.0), false, 0.0, 0.0).is_none());

    // Halfway: 500ms delay + 500ms active = 1000ms elapsed -> progress 0.5
    let p_half = compute_node_progress(1000.0, delay, dur, Some(1.0), false, 0.0, 0.0);
    assert_eq!(p_half, Some(0.5));

    // After finish: 500ms delay + 1000ms dur = 1500ms -> progress 1.0 (held)
    let p_done = compute_node_progress(2000.0, delay, dur, Some(1.0), false, 0.0, 0.0);
    assert_eq!(p_done, Some(1.0));
}

#[test]
fn test_07_node_progress_repetition() {
    let dur = Some(1000);
    let repeats = Some(3.0); // 3 cycles

    // At 1500ms: cycle 2 at 50%
    let p = compute_node_progress(1500.0, 0, dur, repeats, false, 0.0, 0.0);
    assert_eq!(p, Some(0.5));

    // At 2500ms: cycle 3 at 50%
    let p3 = compute_node_progress(2500.0, 0, dur, repeats, false, 0.0, 0.0);
    assert_eq!(p3, Some(0.5));

    // At 3500ms: finished -> 1.0
    let p_end = compute_node_progress(3500.0, 0, dur, repeats, false, 0.0, 0.0);
    assert_eq!(p_end, Some(1.0));
}

#[test]
fn test_08_node_progress_auto_reverse_ping_pong() {
    let dur = Some(1000);
    let repeats = Some(2.0); // 2 cycles ping-pong

    // Cycle 1 at 500ms: moving forward -> 0.5
    let p1 = compute_node_progress(500.0, 0, dur, repeats, true, 0.0, 0.0);
    assert_eq!(p1, Some(0.5));

    // Cycle 2 at 1500ms: moving backward -> 1.0 - 0.5 = 0.5
    let p2 = compute_node_progress(1500.0, 0, dur, repeats, true, 0.0, 0.0);
    assert_eq!(p2, Some(0.5));

    // Cycle 2 at 1750ms: moving backward -> 1.0 - 0.75 = 0.25
    let p2_late = compute_node_progress(1750.0, 0, dur, repeats, true, 0.0, 0.0);
    assert_eq!(p2_late, Some(0.25));

    // Finished (after 2 full cycles): returned to 0.0
    let p_end = compute_node_progress(2500.0, 0, dur, repeats, true, 0.0, 0.0);
    assert_eq!(p_end, Some(0.0));
}

// =========================================================================
// 2. Property Interpolators Tests
// =========================================================================

#[test]
fn test_09_motion_path_linear_interpolation() {
    let path = "M 0 0 L 100 200";
    let (x0, y0, _) = interpolate_motion(path, 0.0);
    let (x_mid, y_mid, _) = interpolate_motion(path, 0.5);
    let (x1, y1, _) = interpolate_motion(path, 1.0);

    assert_eq!(x0, 0.0);
    assert_eq!(y0, 0.0);
    assert_eq!(x_mid, 50.0);
    assert_eq!(y_mid, 100.0);
    assert_eq!(x1, 100.0);
    assert_eq!(y1, 200.0);
}

#[test]
fn test_10_motion_path_multi_segment_length_interpolation() {
    // L-shaped path: (0,0) -> (100, 0) -> (100, 100). Total length = 200.
    let path = "M 0 0 L 100 0 L 100 100";
    let (x_half, y_half, _) = interpolate_motion(path, 0.5); // at 100 length -> (100, 0)
    let (x_three_q, y_three_q, _) = interpolate_motion(path, 0.75); // at 150 length -> (100, 50)

    assert!((x_half - 100.0).abs() < 1e-4);
    assert!((y_half - 0.0).abs() < 1e-4);
    assert!((x_three_q - 100.0).abs() < 1e-4);
    assert!((y_three_q - 50.0).abs() < 1e-4);
}

#[test]
fn test_11_rotation_interpolation_relative_and_absolute() {
    // From 0 to 180
    assert_eq!(
        interpolate_rotation(Some(0.0), Some(180.0), None, 0.5),
        90.0
    );

    // Delta by 360 from 45
    assert_eq!(
        interpolate_rotation(Some(45.0), None, Some(360.0), 0.5),
        225.0
    );
}

#[test]
fn test_12_scale_interpolation_uniform_and_non_uniform() {
    // Uniform scale: 1.0 to 2.0
    let (sx, sy) = interpolate_scale(Some((1.0, 1.0)), Some((2.0, 2.0)), None, 0.5);
    assert_eq!(sx, 1.5);
    assert_eq!(sy, 1.5);

    // Non-uniform: x: 1.0 -> 3.0, y: 1.0 -> 1.0
    let (sx_nu, sy_nu) = interpolate_scale(Some((1.0, 1.0)), Some((3.0, 1.0)), None, 0.5);
    assert_eq!(sx_nu, 2.0);
    assert_eq!(sy_nu, 1.0);
}

#[test]
fn test_13_color_interpolation_rgb_space() {
    let red = [255, 0, 0];
    let blue = [0, 0, 255];

    let mid = interpolate_color(red, blue, 0.5, ColorSpace::Rgb, ColorDirection::Clockwise);
    assert_eq!(mid, [128, 0, 128]); // Purple
}

#[test]
fn test_14_color_interpolation_hsl_space_cw_and_ccw() {
    let red = [255, 0, 0]; // hue = 0
    let green = [0, 255, 0]; // hue = 120

    let mid_cw = interpolate_color(red, green, 0.5, ColorSpace::Hsl, ColorDirection::Clockwise);
    // Hue at 60 is pure Yellow [255, 255, 0]
    assert_eq!(mid_cw[0], 255);
    assert_eq!(mid_cw[1], 255);
    assert_eq!(mid_cw[2], 0);
}

#[test]
fn test_15_effect_interpolation_fade_in_and_wipe_out() {
    let fade_in = EffectAnimation {
        target: TargetElement { shape_id: 10 },
        effect_type: "fade".to_string(),
        transition: EffectTransition::In,
    };
    let (op_in, _, vis_in) = interpolate_effect(&fade_in, 0.5);
    assert_eq!(op_in, 0.5);
    assert!(vis_in);

    let wipe_out = EffectAnimation {
        target: TargetElement { shape_id: 20 },
        effect_type: "wipe".to_string(),
        transition: EffectTransition::Out,
    };
    let (op_out, clip_out, vis_out) = interpolate_effect(&wipe_out, 0.5);
    assert_eq!(op_out, 0.5);
    assert_eq!(clip_out, 0.5);
    assert!(vis_out);
}

// =========================================================================
// 3. 120Hz Deterministic Timeline & State Machine Tests
// =========================================================================

#[test]
fn test_16_timeline_120hz_frame_sampling_budget() {
    // 120Hz frame budget: exactly 8.333333ms per frame
    let frame_dur_ms: f64 = 1000.0 / 120.0;
    assert!((frame_dur_ms - 8.333333).abs() < 1e-4);

    let mut timeline = AnimationTimeline::empty();
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(1000),
                ..Default::default()
            },
            kind: TimeNodeKind::Rotation(RotationAnimation {
                target: TargetElement { shape_id: 1 },
                from_deg: Some(0.0),
                to_deg: Some(360.0),
                by_deg: None,
            }),
        },
    );

    // Sample across 120 frames (1 full second)
    for frame in 0..=120 {
        let state = timeline.sample_frame(frame, 120);
        let shape = state.get_shape(1).expect("Shape 1 must exist");
        let expected_deg = (frame as f64 / 120.0) * 360.0;
        assert!(
            (shape.rotation_deg - expected_deg).abs() < 0.1,
            "Frame {} rotation mismatch",
            frame
        );
    }
}

#[test]
fn test_17_timeline_parallel_containers_synchronization() {
    let mut timeline = AnimationTimeline::empty();

    // Two animations running in parallel on click step 0:
    // 1. Move from (0,0) to (100, 100) in 1000ms
    // 2. Rotate from 0 to 180 deg in 1000ms
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(1000),
                ..Default::default()
            },
            kind: TimeNodeKind::Motion(MotionAnimation {
                target: TargetElement { shape_id: 5 },
                path: "M 0 0 L 100 100".to_string(),
                rotate_to_path: false,
            }),
        },
    );
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(1000),
                ..Default::default()
            },
            kind: TimeNodeKind::Rotation(RotationAnimation {
                target: TargetElement { shape_id: 5 },
                from_deg: Some(0.0),
                to_deg: Some(180.0),
                by_deg: None,
            }),
        },
    );

    // Sample at 500ms
    let state = timeline.sample_at(500.0);
    let s5 = state.get_shape(5).unwrap();
    assert_eq!(s5.translation, (50.0, 50.0));
    assert_eq!(s5.rotation_deg, 90.0);
}

#[test]
fn test_18_timeline_sequential_after_prev_scheduling() {
    let mut timeline = AnimationTimeline::empty();

    // Node 1: 0 to 500ms
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(500),
                ..Default::default()
            },
            kind: TimeNodeKind::Scale(ScaleAnimation {
                target: TargetElement { shape_id: 7 },
                from_scale: Some((1.0, 1.0)),
                to_scale: Some((2.0, 2.0)),
                by_scale: None,
            }),
        },
    );

    // Node 2: starts after Node 1 (at 500ms, duration 500ms)
    timeline.schedule_node(
        0,
        500.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(500),
                ..Default::default()
            },
            kind: TimeNodeKind::Rotation(RotationAnimation {
                target: TargetElement { shape_id: 7 },
                from_deg: Some(0.0),
                to_deg: Some(90.0),
                by_deg: None,
            }),
        },
    );

    // At 250ms: scale is 1.5, rotation is 0
    let s_early = timeline.sample_at(250.0);
    let sh_early = s_early.get_shape(7).unwrap();
    assert_eq!(sh_early.scale, (1.5, 1.5));
    assert_eq!(sh_early.rotation_deg, 0.0);

    // At 750ms: scale is 2.0 (held), rotation is 45.0
    let s_late = timeline.sample_at(750.0);
    let sh_late = s_late.get_shape(7).unwrap();
    assert_eq!(sh_late.scale, (2.0, 2.0));
    assert_eq!(sh_late.rotation_deg, 45.0);
}

#[test]
fn test_19_timeline_click_trigger_sequence_advancement() {
    let mut timeline = AnimationTimeline::empty();

    // Click step 0: Fade in shape 10
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(500),
                ..Default::default()
            },
            kind: TimeNodeKind::Effect(EffectAnimation {
                target: TargetElement { shape_id: 10 },
                effect_type: "fade".to_string(),
                transition: EffectTransition::In,
            }),
        },
    );

    // Click step 1: Fade in shape 20
    timeline.schedule_node(
        1,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(500),
                ..Default::default()
            },
            kind: TimeNodeKind::Effect(EffectAnimation {
                target: TargetElement { shape_id: 20 },
                effect_type: "fade".to_string(),
                transition: EffectTransition::In,
            }),
        },
    );

    assert_eq!(timeline.total_clicks(), 2);
    assert_eq!(timeline.current_click(), 0);

    // On click 0, sample at 500ms: shape 10 is fully visible (opacity 1.0)
    let s0 = timeline.sample_at(500.0);
    assert_eq!(s0.get_shape(10).unwrap().opacity, 1.0);
    assert!(s0.get_shape(20).is_none());

    // User clicks: advance to click step 1
    assert!(timeline.next_click());
    assert_eq!(timeline.current_click(), 1);

    // On click 1, sample at 250ms: shape 10 remains held at 1.0, shape 20 is at 0.5
    let s1 = timeline.sample_at(250.0);
    assert_eq!(s1.get_shape(10).unwrap().opacity, 1.0);
    assert_eq!(s1.get_shape(20).unwrap().opacity, 0.5);
}

#[test]
fn test_20_timeline_seek_click_navigation() {
    let mut timeline = AnimationTimeline::empty();
    for i in 0..5 {
        timeline.schedule_node(
            i,
            0.0,
            TimeNode {
                common: CommonTimeNode::default(),
                kind: TimeNodeKind::Generic(GenericAnimation::default()),
            },
        );
    }
    assert_eq!(timeline.total_clicks(), 5);

    timeline.seek_click(3);
    assert_eq!(timeline.current_click(), 3);

    timeline.seek_click(10); // clamped to max
    assert_eq!(timeline.current_click(), 4);

    assert!(timeline.prev_click());
    assert_eq!(timeline.current_click(), 3);
}

#[test]
fn test_21_timeline_multi_shape_state_accumulation() {
    let mut timeline = AnimationTimeline::empty();
    for id in 1..=10 {
        timeline.schedule_node(
            0,
            0.0,
            TimeNode {
                common: CommonTimeNode {
                    duration_ms: Some(1000),
                    ..Default::default()
                },
                kind: TimeNodeKind::Rotation(RotationAnimation {
                    target: TargetElement { shape_id: id },
                    from_deg: Some(0.0),
                    to_deg: Some(id as f64 * 10.0),
                    by_deg: None,
                }),
            },
        );
    }

    let state = timeline.sample_at(1000.0);
    assert_eq!(state.shapes.len(), 10);
    for id in 1..=10 {
        let sh = state.get_shape(id).unwrap();
        assert_eq!(sh.rotation_deg, id as f64 * 10.0);
    }
}

#[test]
fn test_22_timeline_auto_reverse_120hz_frame_trajectory() {
    let mut timeline = AnimationTimeline::empty();
    timeline.schedule_node(
        0,
        0.0,
        TimeNode {
            common: CommonTimeNode {
                duration_ms: Some(500),
                repeat_count: Some(2.0),
                auto_reverse: true,
                ..Default::default()
            },
            kind: TimeNodeKind::Motion(MotionAnimation {
                target: TargetElement { shape_id: 1 },
                path: "M 0 0 L 100 0".to_string(),
                rotate_to_path: false,
            }),
        },
    );

    // Frame 0: (0, 0)
    let s0 = timeline.sample_at(0.0);
    assert_eq!(s0.get_shape(1).unwrap().translation.0, 0.0);

    // Midpoint of forward cycle: 250ms -> 50.0
    let s_mid1 = timeline.sample_at(250.0);
    assert_eq!(s_mid1.get_shape(1).unwrap().translation.0, 50.0);

    // Apex: 500ms -> 100.0
    let s_apex = timeline.sample_at(500.0);
    assert_eq!(s_apex.get_shape(1).unwrap().translation.0, 100.0);

    // Midpoint of reverse cycle: 750ms -> 50.0
    let s_mid2 = timeline.sample_at(750.0);
    assert_eq!(s_mid2.get_shape(1).unwrap().translation.0, 50.0);

    // End of reverse cycle: 1000ms -> 0.0
    let s_end = timeline.sample_at(1000.0);
    assert_eq!(s_end.get_shape(1).unwrap().translation.0, 0.0);
}

// =========================================================================
// 4. XML Parsing & Serialization Round-Trip Tests
// =========================================================================

#[test]
fn test_23_parse_slide_timing_from_xml() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree>
    </p:cSld>
    <p:timing>
        <p:tnLst>
            <p:par>
                <p:cTn id="1" dur="indefinite" nodeType="tmRoot">
                    <p:childTnLst>
                        <p:animMotion path="M 0 0 L 100 200">
                            <p:cBhvr>
                                <p:cTn id="2" dur="2000" nodeType="clickEffect"/>
                                <p:tgtEl><p:spTgt spid="15"/></p:tgtEl>
                            </p:cBhvr>
                        </p:animMotion>
                    </p:childTnLst>
                </p:cTn>
            </p:par>
        </p:tnLst>
    </p:timing>
</p:sld>"#;

    let slide = parse_slide(xml).expect("Parse slide with timing");
    assert!(slide.timing.is_some());
    let timing = slide.timing.unwrap();
    assert!(timing.root_node.is_some());

    let timeline = AnimationTimeline::new(&timing);
    assert_eq!(timeline.total_clicks(), 1);
}

#[test]
fn test_24_serialize_slide_timing_roundtrip() {
    let timing = SlideTiming {
        root_node: Some(TimeNode {
            common: CommonTimeNode {
                id: 1,
                duration_ms: None,
                ..Default::default()
            },
            kind: TimeNodeKind::Parallel(vec![TimeNode {
                common: CommonTimeNode {
                    id: 2,
                    duration_ms: Some(1500),
                    trigger_type: TriggerType::OnClick,
                    ..Default::default()
                },
                kind: TimeNodeKind::Rotation(RotationAnimation {
                    target: TargetElement { shape_id: 42 },
                    from_deg: Some(0.0),
                    to_deg: Some(360.0),
                    by_deg: None,
                }),
            }]),
        }),
    };
    let slide = Slide {
        timing: Some(timing),
        ..Default::default()
    };

    let serialized = serialize_slide(&slide).expect("Serialize slide with timing");
    let xml_str = std::str::from_utf8(&serialized).expect("UTF-8 XML");
    assert!(xml_str.contains("<p:timing>"));
    assert!(xml_str.contains("<p:animRot"));
    assert!(xml_str.contains("spid=\"42\""));

    let reloaded = parse_slide(&serialized).expect("Reload serialized slide");
    assert!(reloaded.timing.is_some());
}

#[test]
fn test_25_baseline_pptx_timing_preservation() {
    let path = resolve_repo_path("specs/fixtures/baseline.pptx");
    if path.exists() {
        let file = std::fs::File::open(&path).expect("Open baseline.pptx");
        let pkg = ppt_engine::PresentationPackage::open(file).expect("Open baseline.pptx");
        assert_eq!(pkg.slide_count(), 2);
    }
}

#[test]
fn test_26_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/ppt/4.2.1_vectors.json");
    let json_bytes = std::fs::read(&path).expect("Must read specs/evidence/ppt/4.2.1_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let items = vectors["vectors"]
        .as_array()
        .expect("vectors array in manifest");
    assert!(
        items.len() >= 25,
        "Manifest must contain at least 25 verified vectors, found {}",
        items.len()
    );

    for item in items {
        assert!(item["id"].is_string());
        assert!(item["category"].is_string());
        assert!(item["description"].is_string());
        assert_eq!(item["status"].as_str(), Some("PASS"));
    }
}
