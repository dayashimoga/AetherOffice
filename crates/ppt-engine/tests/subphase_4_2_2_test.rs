//! Subphase 4.2.2 Integration Test Suite:
//! REA Morph Transition Vertex Solver & SmartArt Layout Engine
//!
//! Validates:
//! - Explicit `!!` shape naming and heuristic similarity scoring
//! - Equal-subdivision cubic Bezier path morphing
//! - Text morphing (`byObject`, `byWord`, `byChar`)
//! - 120Hz frame evaluation SLA (< 8.333 ms)
//! - SmartArt constraint layout algorithms (`lin`, `pyra`, `cycle`, `hierChild`, `snake`)
//! - Synced `p:spTree` shape cache generation
//! - Round-trip XML serialization & deserialization of `p:transition` / `p159:morph`
//! - Vector manifest validation against `specs/evidence/ppt/4.2.2_vectors.json`

use ppt_engine::{
    parse_slide, serialize_slide, CubicBezierSegment, DiagramConnection, DiagramDataModel,
    DiagramPoint, DiagramPointType, Geometry, LinearDirection, MorphEvaluator, MorphMatcher,
    MorphOption, MorphTransition, MorphedTransform, PathMorpher, Rect, Shape, ShapeElement,
    ShapeProperties, Slide, SlideTransition, SmartArtLayoutEngine, SmartArtNodeLayout, TextBody,
    TextMorpher, TextParagraph, TextRunElement, Transform2D,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn resolve_repo_path(rel_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root");
    repo_root.join(rel_path)
}

#[allow(clippy::too_many_arguments)]
fn make_test_shape(
    id: u32,
    name: &str,
    x: i64,
    y: i64,
    cx: u32,
    cy: u32,
    preset_geom: Option<&str>,
    text: Option<&str>,
) -> Shape {
    Shape {
        id,
        name: name.to_string(),
        properties: ShapeProperties {
            transform: Some(Transform2D {
                x,
                y,
                cx,
                cy,
                rot: 0,
                flip_h: false,
                flip_v: false,
            }),
            geometry: preset_geom.map(|g| Geometry::Preset(g.to_string())),
            ..Default::default()
        },
        text_body: text.map(|t| TextBody {
            body_properties: Default::default(),
            list_style: None,
            paragraphs: vec![TextParagraph {
                properties: None,
                runs: vec![TextRunElement::Text(t.to_string(), None)],
                end_paragraph_run_properties: None,
            }],
        }),
        ..Default::default()
    }
}

fn make_slide_with_shapes(shapes: Vec<Shape>) -> Slide {
    let mut slide = Slide::default();
    slide.common_slide_data.shape_tree.shapes = shapes
        .into_iter()
        .map(|s| ShapeElement::Shape(Box::new(s)))
        .collect();
    slide
}

#[test]
fn test_01_explicit_bang_bang_matching() {
    let s_a = make_test_shape(
        1,
        "!!HeaderBox",
        100,
        100,
        300,
        100,
        Some("rect"),
        Some("Title"),
    );
    let s_b = make_test_shape(
        2,
        "!!HeaderBox",
        800,
        500,
        400,
        200,
        Some("ellipse"),
        Some("Other"),
    );

    let slide_a = make_slide_with_shapes(vec![s_a]);
    let slide_b = make_slide_with_shapes(vec![s_b]);

    let pairs = MorphMatcher::match_slides(&slide_a, &slide_b, 1000.0, 1000.0);
    assert_eq!(pairs.len(), 1);
    assert!(pairs[0].is_explicit_match);
    assert_eq!(pairs[0].source_id, Some(1));
    assert_eq!(pairs[0].target_id, Some(2));
    assert_eq!(pairs[0].score, 1.0);
}

#[test]
fn test_02_geometry_similarity_scoring() {
    let s_rect1 = make_test_shape(1, "Box1", 0, 0, 100, 100, Some("rect"), None);
    let s_rect2 = make_test_shape(2, "Box2", 0, 0, 100, 100, Some("rect"), None);
    let s_round = make_test_shape(3, "Box3", 0, 0, 100, 100, Some("roundRect"), None);
    let s_star = make_test_shape(4, "Star", 0, 0, 100, 100, Some("star5"), None);

    assert_eq!(MorphMatcher::score_geometry(&s_rect1, &s_rect2), 1.0);
    assert_eq!(MorphMatcher::score_geometry(&s_rect1, &s_round), 0.5);
    assert_eq!(MorphMatcher::score_geometry(&s_round, &s_rect1), 0.5);
    assert_eq!(MorphMatcher::score_geometry(&s_rect1, &s_star), 0.0);
}

#[test]
fn test_03_position_similarity_scoring() {
    let s1 = make_test_shape(1, "A", 400, 400, 200, 200, None, None);
    let s2 = make_test_shape(2, "B", 400, 400, 200, 200, None, None);
    let s_far = make_test_shape(3, "C", 0, 0, 200, 200, None, None);

    let score_coincident = MorphMatcher::score_position(&s1, &s2, 1000.0, 1000.0);
    assert_eq!(score_coincident, 1.0);

    let score_offset = MorphMatcher::score_position(&s1, &s_far, 1000.0, 1000.0);
    assert!(score_offset < 1.0);
    assert!(score_offset > 0.4);
}

#[test]
fn test_04_size_similarity_scoring() {
    let s_100k = make_test_shape(1, "A", 0, 0, 500, 200, None, None);
    let s_100k_b = make_test_shape(2, "B", 0, 0, 200, 500, None, None);
    let s_50k = make_test_shape(3, "C", 0, 0, 250, 200, None, None);
    let s_10k = make_test_shape(4, "D", 0, 0, 100, 100, None, None);

    assert_eq!(MorphMatcher::score_size(&s_100k, &s_100k_b), 1.0);
    assert_eq!(MorphMatcher::score_size(&s_100k, &s_50k), 0.5);
    assert_eq!(MorphMatcher::score_size(&s_100k, &s_10k), 0.1);
}

#[test]
fn test_05_text_bigram_similarity_scoring() {
    let s_exact1 = make_test_shape(1, "A", 0, 0, 100, 100, None, Some("Quarterly Sales"));
    let s_exact2 = make_test_shape(2, "B", 0, 0, 100, 100, None, Some("Quarterly Sales"));
    let s_partial = make_test_shape(3, "C", 0, 0, 100, 100, None, Some("Annual Sales"));
    let s_empty1 = make_test_shape(4, "D", 0, 0, 100, 100, None, None);
    let s_empty2 = make_test_shape(5, "E", 0, 0, 100, 100, None, None);

    assert_eq!(MorphMatcher::score_text(&s_exact1, &s_exact2), 1.0);
    assert_eq!(MorphMatcher::score_text(&s_empty1, &s_empty2), 1.0);
    assert_eq!(MorphMatcher::score_text(&s_exact1, &s_empty1), 0.0);

    let partial_score = MorphMatcher::score_text(&s_exact1, &s_partial);
    assert!(partial_score > 0.2 && partial_score < 1.0);
}

#[test]
fn test_06_overall_weighted_score_and_threshold() {
    let s_a = make_test_shape(1, "S1", 100, 100, 100, 100, Some("rect"), Some("Overview"));
    let s_b = make_test_shape(2, "S2", 120, 110, 95, 100, Some("rect"), Some("Overview"));

    let score = MorphMatcher::score_shapes(&s_a, &s_b, 1000.0, 1000.0);
    assert!(score > 0.85);

    let slide_a = make_slide_with_shapes(vec![s_a]);
    let slide_b = make_slide_with_shapes(vec![s_b]);
    let pairs = MorphMatcher::match_slides(&slide_a, &slide_b, 1000.0, 1000.0);

    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].source_id, Some(1));
    assert_eq!(pairs[0].target_id, Some(2));
}

#[test]
fn test_07_unmatched_source_shape_fades_out() {
    let s_a = make_test_shape(10, "OldBox", 0, 0, 100, 100, Some("rect"), None);
    let slide_a = make_slide_with_shapes(vec![s_a]);
    let slide_b = make_slide_with_shapes(vec![]);

    let state_0 = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        0.0,
        1000.0,
        1000.0,
    );
    assert_eq!(state_0.shapes.len(), 1);
    assert_eq!(state_0.shapes[0].opacity, 1.0);

    let state_half = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        0.5,
        1000.0,
        1000.0,
    );
    assert!((state_half.shapes[0].opacity - 0.5).abs() < 1e-5);

    let state_1 = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        1.0,
        1000.0,
        1000.0,
    );
    assert_eq!(state_1.shapes[0].opacity, 0.0);
}

#[test]
fn test_08_unmatched_target_shape_fades_in() {
    let s_b = make_test_shape(20, "NewBox", 0, 0, 100, 100, Some("rect"), None);
    let slide_a = make_slide_with_shapes(vec![]);
    let slide_b = make_slide_with_shapes(vec![s_b]);

    let state_0 = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        0.0,
        1000.0,
        1000.0,
    );
    assert_eq!(state_0.shapes.len(), 1);
    assert_eq!(state_0.shapes[0].opacity, 0.0);

    let state_half = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        0.5,
        1000.0,
        1000.0,
    );
    assert!((state_half.shapes[0].opacity - 0.5).abs() < 1e-5);

    let state_1 = MorphEvaluator::evaluate_transition(
        &slide_a,
        &slide_b,
        MorphOption::ByObject,
        1.0,
        1000.0,
        1000.0,
    );
    assert_eq!(state_1.shapes[0].opacity, 1.0);
}

#[test]
fn test_09_lineto_to_cubic_conversion() {
    let seg = CubicBezierSegment::from_line((0.0, 0.0), (300.0, 600.0));
    assert_eq!(seg.p0, (0.0, 0.0));
    assert_eq!(seg.c1, (100.0, 200.0));
    assert_eq!(seg.c2, (200.0, 400.0));
    assert_eq!(seg.p3, (300.0, 600.0));

    // Midpoint check
    let mid = seg.sample(0.5);
    assert!((mid.0 - 150.0).abs() < 1e-5);
    assert!((mid.1 - 300.0).abs() < 1e-5);
}

#[test]
fn test_10_quadbez_to_cubic_conversion() {
    let seg = CubicBezierSegment::from_quad((0.0, 0.0), (150.0, 300.0), (300.0, 0.0));
    assert_eq!(seg.p0, (0.0, 0.0));
    assert_eq!(seg.c1, (100.0, 200.0));
    assert_eq!(seg.c2, (200.0, 200.0));
    assert_eq!(seg.p3, (300.0, 0.0));

    // Apex at u=0.5
    let apex = seg.sample(0.5);
    assert!((apex.0 - 150.0).abs() < 1e-5);
    assert!((apex.1 - 150.0).abs() < 1e-5);
}

#[test]
fn test_11_path_subdivision_normalization() {
    let seg1 = CubicBezierSegment::from_line((0.0, 0.0), (100.0, 0.0));
    let seg2 = CubicBezierSegment::from_line((100.0, 0.0), (100.0, 100.0));
    let seg3 = CubicBezierSegment::from_line((100.0, 100.0), (0.0, 0.0));

    let triangle = vec![seg1, seg2, seg3];
    let normalized = PathMorpher::normalize_segments(&triangle, 8);

    assert_eq!(normalized.len(), 8);
    // Closed path preservation: start is p0 of first, end is p3 of last
    assert!((normalized[0].p0.0 - 0.0).abs() < 1e-4);
    assert!((normalized[0].p0.1 - 0.0).abs() < 1e-4);
    assert!((normalized[7].p3.0 - 0.0).abs() < 1e-4);
    assert!((normalized[7].p3.1 - 0.0).abs() < 1e-4);
}

#[test]
fn test_12_bezier_control_points_time_interpolation() {
    let seg_a = CubicBezierSegment::new((0.0, 0.0), (10.0, 20.0), (20.0, 20.0), (30.0, 0.0));
    let seg_b = CubicBezierSegment::new(
        (100.0, 100.0),
        (110.0, 120.0),
        (120.0, 120.0),
        (130.0, 100.0),
    );

    let interp = PathMorpher::interpolate_segments(&[seg_a], &[seg_b], 0.5);
    assert_eq!(interp.len(), 1);
    assert_eq!(interp[0].p0, (50.0, 50.0));
    assert_eq!(interp[0].c1, (60.0, 70.0));
    assert_eq!(interp[0].c2, (70.0, 70.0));
    assert_eq!(interp[0].p3, (80.0, 50.0));
}

#[test]
fn test_13_text_morph_by_object() {
    let s_a = make_test_shape(1, "A", 100, 100, 200, 50, None, Some("Report 2025"));
    let s_b = make_test_shape(2, "B", 300, 200, 400, 100, None, Some("Report 2026"));

    let xfrm_t = MorphedTransform::new(200.0, 150.0, 300.0, 75.0, 0.0);
    let elements = TextMorpher::morph_text(&s_a, &s_b, MorphOption::ByObject, 0.5, &xfrm_t);

    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].text, "Report 2026");
    assert_eq!(elements[0].x, 210.0);
    assert_eq!(elements[0].y, 160.0);
}

#[test]
fn test_14_text_morph_by_word() {
    let s_a = make_test_shape(1, "A", 0, 0, 500, 100, None, Some("Revenue Growth 2025"));
    let s_b = make_test_shape(2, "B", 0, 0, 500, 100, None, Some("Revenue Target 2026"));

    let xfrm = MorphedTransform::default();
    let elements = TextMorpher::morph_text(&s_a, &s_b, MorphOption::ByWord, 0.5, &xfrm);

    // Common word "Revenue" should have opacity 1.0
    let rev = elements.iter().find(|e| e.text == "Revenue");
    assert!(rev.is_some());
    assert_eq!(rev.unwrap().opacity, 1.0);

    // Exiting words "Growth", "2025" fade out
    let growth = elements.iter().find(|e| e.text == "Growth");
    assert!(growth.is_some());
    assert!((growth.unwrap().opacity - 0.5).abs() < 1e-5);

    // Entering words "Target", "2026" fade in
    let target = elements.iter().find(|e| e.text == "Target");
    assert!(target.is_some());
    assert!((target.unwrap().opacity - 0.5).abs() < 1e-5);
}

#[test]
fn test_15_text_morph_by_char() {
    let s_a = make_test_shape(1, "A", 0, 0, 300, 100, None, Some("OPEN"));
    let s_b = make_test_shape(2, "B", 0, 0, 300, 100, None, Some("OPERATE"));

    let xfrm = MorphedTransform::default();
    let elements = TextMorpher::morph_text(&s_a, &s_b, MorphOption::ByChar, 0.5, &xfrm);

    // Characters 'O', 'P', 'E' are matched
    let o = elements.iter().find(|e| e.text == "O");
    assert!(o.is_some());
    assert_eq!(o.unwrap().opacity, 1.0);

    // 'N' fades out
    let n = elements.iter().find(|e| e.text == "N");
    assert!(n.is_some());
    assert!((n.unwrap().opacity - 0.5).abs() < 1e-5);

    // 'R', 'A', 'T', 'E' fade in
    let r = elements.iter().find(|e| e.text == "R");
    assert!(r.is_some());
    assert!((r.unwrap().opacity - 0.5).abs() < 1e-5);
}

#[test]
fn test_16_smartart_linear_horizontal() {
    let pts = vec![
        DiagramPoint {
            model_id: "1".into(),
            point_type: DiagramPointType::Node,
            title: "Phase 1".into(),
        },
        DiagramPoint {
            model_id: "2".into(),
            point_type: DiagramPointType::Node,
            title: "Phase 2".into(),
        },
        DiagramPoint {
            model_id: "3".into(),
            point_type: DiagramPointType::Node,
            title: "Phase 3".into(),
        },
        DiagramPoint {
            model_id: "4".into(),
            point_type: DiagramPointType::Node,
            title: "Phase 4".into(),
        },
    ];

    let bounds = Rect::new(0.0, 0.0, 1000.0, 200.0);
    let layouts =
        SmartArtLayoutEngine::layout_linear(&pts, LinearDirection::Horizontal, bounds, 40.0);

    assert_eq!(layouts.len(), 4);
    // (1000 - 3*40) / 4 = 880 / 4 = 220
    assert_eq!(layouts[0].bounds.w, 220.0);
    assert_eq!(layouts[0].bounds.x, 0.0);
    assert_eq!(layouts[1].bounds.x, 260.0);
    assert_eq!(layouts[2].bounds.x, 520.0);
    assert_eq!(layouts[3].bounds.x, 780.0);
}

#[test]
fn test_17_smartart_linear_vertical() {
    let pts = vec![
        DiagramPoint {
            model_id: "1".into(),
            point_type: DiagramPointType::Node,
            title: "Step 1".into(),
        },
        DiagramPoint {
            model_id: "2".into(),
            point_type: DiagramPointType::Node,
            title: "Step 2".into(),
        },
        DiagramPoint {
            model_id: "3".into(),
            point_type: DiagramPointType::Node,
            title: "Step 3".into(),
        },
    ];

    let bounds = Rect::new(0.0, 0.0, 300.0, 800.0);
    let layouts =
        SmartArtLayoutEngine::layout_linear(&pts, LinearDirection::Vertical, bounds, 40.0);

    assert_eq!(layouts.len(), 3);
    // (800 - 2*40) / 3 = 720 / 3 = 240
    assert_eq!(layouts[0].bounds.h, 240.0);
    assert_eq!(layouts[0].bounds.y, 0.0);
    assert_eq!(layouts[1].bounds.y, 280.0);
    assert_eq!(layouts[2].bounds.y, 560.0);
}

#[test]
fn test_18_smartart_pyramid_layout() {
    let pts = vec![
        DiagramPoint {
            model_id: "1".into(),
            point_type: DiagramPointType::Node,
            title: "Apex".into(),
        },
        DiagramPoint {
            model_id: "2".into(),
            point_type: DiagramPointType::Node,
            title: "Middle".into(),
        },
        DiagramPoint {
            model_id: "3".into(),
            point_type: DiagramPointType::Node,
            title: "Base".into(),
        },
    ];

    let bounds = Rect::new(0.0, 0.0, 600.0, 600.0);
    let layouts = SmartArtLayoutEngine::layout_pyramid(&pts, false, bounds);

    assert_eq!(layouts.len(), 3);
    // 3 tiers, each 200 high
    assert_eq!(layouts[0].bounds.h, 200.0);
    assert_eq!(layouts[1].bounds.h, 200.0);
    assert_eq!(layouts[2].bounds.h, 200.0);

    // Widths expand: 600*(1/3)=200, 600*(2/3)=400, 600*(3/3)=600
    assert_eq!(layouts[0].bounds.w, 200.0);
    assert_eq!(layouts[1].bounds.w, 400.0);
    assert_eq!(layouts[2].bounds.w, 600.0);

    // Centered: (600-200)/2=200, (600-400)/2=100, 0
    assert_eq!(layouts[0].bounds.x, 200.0);
    assert_eq!(layouts[1].bounds.x, 100.0);
    assert_eq!(layouts[2].bounds.x, 0.0);
}

#[test]
fn test_19_smartart_cycle_layout() {
    let pts = vec![
        DiagramPoint {
            model_id: "1".into(),
            point_type: DiagramPointType::Node,
            title: "North".into(),
        },
        DiagramPoint {
            model_id: "2".into(),
            point_type: DiagramPointType::Node,
            title: "East".into(),
        },
        DiagramPoint {
            model_id: "3".into(),
            point_type: DiagramPointType::Node,
            title: "South".into(),
        },
        DiagramPoint {
            model_id: "4".into(),
            point_type: DiagramPointType::Node,
            title: "West".into(),
        },
    ];

    let bounds = Rect::new(0.0, 0.0, 800.0, 800.0);
    let layouts = SmartArtLayoutEngine::layout_cycle(&pts, bounds, 100.0, 100.0);

    assert_eq!(layouts.len(), 4);
    // Node 0 (North): top center (cx = 400 - 50 = 350, y = 400 - 350 - 50 = 0)
    assert!((layouts[0].bounds.x - 350.0).abs() < 1e-4);
    assert!((layouts[0].bounds.y - 0.0).abs() < 1e-4);

    // Node 1 (East): right center (x = 400 + 350 - 50 = 700, y = 400 - 50 = 350)
    assert!((layouts[1].bounds.x - 700.0).abs() < 1e-4);
    assert!((layouts[1].bounds.y - 350.0).abs() < 1e-4);
}

#[test]
fn test_20_smartart_hierarchy_layout() {
    let model = DiagramDataModel {
        points: vec![
            DiagramPoint {
                model_id: "ceo".into(),
                point_type: DiagramPointType::Node,
                title: "CEO".into(),
            },
            DiagramPoint {
                model_id: "cto".into(),
                point_type: DiagramPointType::Node,
                title: "CTO".into(),
            },
            DiagramPoint {
                model_id: "cfo".into(),
                point_type: DiagramPointType::Node,
                title: "CFO".into(),
            },
            DiagramPoint {
                model_id: "coo".into(),
                point_type: DiagramPointType::Node,
                title: "COO".into(),
            },
        ],
        connections: vec![
            DiagramConnection {
                model_id: "c1".into(),
                src_id: "ceo".into(),
                dest_id: "cto".into(),
                cxn_type: "parOf".into(),
            },
            DiagramConnection {
                model_id: "c2".into(),
                src_id: "ceo".into(),
                dest_id: "cfo".into(),
                cxn_type: "parOf".into(),
            },
            DiagramConnection {
                model_id: "c3".into(),
                src_id: "ceo".into(),
                dest_id: "coo".into(),
                cxn_type: "parOf".into(),
            },
        ],
    };

    let bounds = Rect::new(0.0, 0.0, 900.0, 600.0);
    let layouts = SmartArtLayoutEngine::layout_hierarchy(&model, bounds, 150.0, 80.0);

    assert_eq!(layouts.len(), 4);
    let ceo = layouts.iter().find(|l| l.model_id == "ceo").unwrap();
    assert_eq!(ceo.depth_level, 0);
    assert_eq!(ceo.bounds.x, (900.0 - 150.0) / 2.0);

    let cto = layouts.iter().find(|l| l.model_id == "cto").unwrap();
    assert_eq!(cto.depth_level, 1);
    assert_eq!(cto.bounds.y, 300.0);
}

#[test]
fn test_21_smartart_snake_layout() {
    let mut pts = Vec::new();
    for i in 1..=6 {
        pts.push(DiagramPoint {
            model_id: i.to_string(),
            point_type: DiagramPointType::Node,
            title: format!("Item {}", i),
        });
    }

    let bounds = Rect::new(0.0, 0.0, 600.0, 400.0);
    let layouts = SmartArtLayoutEngine::layout_snake(&pts, 3, bounds, 20.0);

    assert_eq!(layouts.len(), 6);
    // Row 0 left->right: items 0, 1, 2
    assert_eq!(layouts[0].depth_level, 0);
    assert_eq!(layouts[1].depth_level, 0);
    assert_eq!(layouts[2].depth_level, 0);
    assert!(layouts[0].bounds.x < layouts[1].bounds.x);
    assert!(layouts[1].bounds.x < layouts[2].bounds.x);

    // Row 1 right->left: items 3, 4, 5
    assert_eq!(layouts[3].depth_level, 1);
    assert_eq!(layouts[4].depth_level, 1);
    assert_eq!(layouts[5].depth_level, 1);
    assert!(layouts[3].bounds.x > layouts[4].bounds.x);
    assert!(layouts[4].bounds.x > layouts[5].bounds.x);
}

#[test]
fn test_22_smartart_data_model_connections() {
    let model = DiagramDataModel {
        points: vec![
            DiagramPoint {
                model_id: "p1".into(),
                point_type: DiagramPointType::Node,
                title: "A".into(),
            },
            DiagramPoint {
                model_id: "p2".into(),
                point_type: DiagramPointType::Node,
                title: "B".into(),
            },
            DiagramPoint {
                model_id: "p3".into(),
                point_type: DiagramPointType::Assistant,
                title: "Asst".into(),
            },
            DiagramPoint {
                model_id: "p4".into(),
                point_type: DiagramPointType::Node,
                title: "C".into(),
            },
        ],
        connections: vec![
            DiagramConnection {
                model_id: "cx1".into(),
                src_id: "p1".into(),
                dest_id: "p2".into(),
                cxn_type: "parOf".into(),
            },
            DiagramConnection {
                model_id: "cx2".into(),
                src_id: "p1".into(),
                dest_id: "p4".into(),
                cxn_type: "parOf".into(),
            },
            DiagramConnection {
                model_id: "cx3".into(),
                src_id: "p1".into(),
                dest_id: "p3".into(),
                cxn_type: "presOf".into(),
            },
        ],
    };

    assert_eq!(model.points.len(), 4);
    assert_eq!(model.connections.len(), 3);
    assert_eq!(
        model
            .points
            .iter()
            .filter(|p| p.point_type == DiagramPointType::Node)
            .count(),
        3
    );
}

#[test]
fn test_23_smartart_shape_tree_sync() {
    let layouts = vec![
        SmartArtNodeLayout {
            model_id: "1".into(),
            title: "Analysis".into(),
            bounds: Rect::new(50.0, 100.0, 200.0, 120.0),
            depth_level: 0,
            preset_geom: "roundRect".into(),
        },
        SmartArtNodeLayout {
            model_id: "2".into(),
            title: "Implementation".into(),
            bounds: Rect::new(300.0, 100.0, 200.0, 120.0),
            depth_level: 0,
            preset_geom: "roundRect".into(),
        },
        SmartArtNodeLayout {
            model_id: "3".into(),
            title: "Release".into(),
            bounds: Rect::new(550.0, 100.0, 200.0, 120.0),
            depth_level: 0,
            preset_geom: "roundRect".into(),
        },
    ];

    let shapes = SmartArtLayoutEngine::sync_to_shape_tree(&layouts, 101);
    assert_eq!(shapes.len(), 3);
    assert_eq!(shapes[0].id, 101);
    assert_eq!(shapes[1].id, 102);
    assert_eq!(shapes[2].id, 103);

    let xfrm0 = shapes[0].properties.transform.unwrap();
    assert_eq!(xfrm0.x, 50);
    assert_eq!(xfrm0.y, 100);
    assert_eq!(xfrm0.cx, 200);
    assert_eq!(xfrm0.cy, 120);

    let text0 = MorphMatcher::extract_plain_text(&shapes[0]);
    assert_eq!(text0, "Analysis");
}

#[test]
fn test_24_120hz_morph_evaluation_budget() {
    let s_a = make_test_shape(
        1,
        "!!Box",
        100,
        100,
        300,
        200,
        Some("rect"),
        Some("Old Content"),
    );
    let s_b = make_test_shape(
        2,
        "!!Box",
        500,
        300,
        600,
        400,
        Some("roundRect"),
        Some("New Content"),
    );

    let slide_a = make_slide_with_shapes(vec![s_a]);
    let slide_b = make_slide_with_shapes(vec![s_b]);

    let frame_budget_ms = 1000.0 / 120.0; // 8.333333 ms
    let frame_count = 120;

    let start = Instant::now();
    for i in 0..frame_count {
        let t = i as f64 / (frame_count - 1) as f64;
        let state = MorphEvaluator::evaluate_transition(
            &slide_a,
            &slide_b,
            MorphOption::ByWord,
            t,
            1280.0,
            720.0,
        );
        assert_eq!(state.shapes.len(), 1);
    }
    let elapsed = start.elapsed();
    let per_frame_ms = elapsed.as_secs_f64() * 1000.0 / (frame_count as f64);

    assert!(
        per_frame_ms < frame_budget_ms,
        "Per-frame evaluation time {} ms must be within 120Hz SLA (< 8.333 ms)",
        per_frame_ms
    );
}

#[test]
fn test_25_xml_roundtrip_morph_transition() {
    let slide = Slide {
        transition: Some(SlideTransition {
            duration_ms: Some(1500),
            advance_on_click: true,
            advance_after_ms: None,
            speed: Some("fast".into()),
            morph: Some(MorphTransition {
                option: MorphOption::ByChar,
            }),
            raw_elements: Vec::new(),
        }),
        ..Default::default()
    };

    let xml = serialize_slide(&slide).expect("Serialize slide with morph transition");
    let xml_str = std::str::from_utf8(&xml).expect("Valid UTF-8 XML");

    assert!(xml_str.contains("<p:transition"));
    assert!(xml_str.contains("<p159:morph"));
    assert!(xml_str.contains("option=\"byChar\""));

    let parsed = parse_slide(&xml).expect("Parse slide with morph transition");
    let trans = parsed.transition.expect("Slide transition parsed");
    let morph = trans.morph.expect("Morph transition parsed");
    assert_eq!(morph.option, MorphOption::ByChar);
}

#[test]
fn test_26_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/ppt/4.2.2_vectors.json");
    assert!(path.exists(), "Vectors manifest must exist");

    let content = std::fs::read_to_string(path).expect("Read vectors manifest");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(v["subphase"], "4.2.2");
    assert_eq!(v["vector_count"], 26);
    let vectors = v["vectors"].as_array().expect("Vectors array");
    assert_eq!(vectors.len(), 26);

    for (i, vec_item) in vectors.iter().enumerate() {
        let expected_id = format!("VEC-4.2.2-{:02}", i + 1);
        assert_eq!(vec_item["id"], expected_id.as_str());
        assert!(!vec_item["description"].as_str().unwrap().is_empty());
    }
}
