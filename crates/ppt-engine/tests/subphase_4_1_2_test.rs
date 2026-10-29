#![forbid(unsafe_code)]
//! Subphase 4.1.2 Test Suite: DrawingML 187 Preset Geometries & Guide Formula Compiler
//!
//! Validates:
//! - All 17 DrawingML guide arithmetic and trigonometric operations.
//! - Context constants (`w`, `h`, `l`, `t`, `r`, `b`, `hc`, `vc`, `ss`, `ls`, angles).
//! - All 187 standard ECMA-376 preset shapes catalog and path generation.
//! - Parametric adjustments (`a:avLst`).
//! - Custom geometry compilation (`a:custGeom`).
//! - Elliptical arc to cubic Bezier conversion.
//! - SVG path serialization (`d="..."`).
//! - Path bounding box and hit testing.
//! - `ResolvedShape::evaluate_paths` integration.

use ppt_engine::drawingml::*;
use ppt_engine::model::*;
use ppt_engine::ResolvedShape;
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
// 1. Guide Formula Arithmetic Tests
// =========================================================================

#[test]
fn test_01_guide_val_operator() {
    let ctx = GuideContext::new(1000, 2000);
    assert_eq!(ctx.eval_formula("val 42").unwrap(), 42);
    assert_eq!(ctx.eval_formula("val -150").unwrap(), -150);
    assert_eq!(ctx.eval_formula("val w").unwrap(), 1000);
    assert_eq!(ctx.eval_formula("val h").unwrap(), 2000);
}

#[test]
fn test_02_guide_add_sub_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // +- x y z = x + y - z
    assert_eq!(ctx.eval_formula("+- 100 50 30").unwrap(), 120);
    assert_eq!(ctx.eval_formula("+- w h 500").unwrap(), 2500);
}

#[test]
fn test_03_guide_mul_div_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // */ x y z = (x * y) / z
    assert_eq!(ctx.eval_formula("*/ 100 20 5").unwrap(), 400);
    assert_eq!(ctx.eval_formula("*/ w 1 2").unwrap(), 500);
    // Division by zero returns 0 safely
    assert_eq!(ctx.eval_formula("*/ 100 20 0").unwrap(), 0);
}

#[test]
fn test_04_guide_add_div_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // +/ x y z = (x + y) / z
    assert_eq!(ctx.eval_formula("+/ 100 50 3").unwrap(), 50);
    assert_eq!(ctx.eval_formula("+/ w h 3").unwrap(), 1000);
    assert_eq!(ctx.eval_formula("+/ 10 20 0").unwrap(), 0);
}

#[test]
fn test_05_guide_ternary_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // ?: x y z = if x > 0 then y else z
    assert_eq!(ctx.eval_formula("?: 1 100 200").unwrap(), 100);
    assert_eq!(ctx.eval_formula("?: 0 100 200").unwrap(), 200);
    assert_eq!(ctx.eval_formula("?: -5 100 200").unwrap(), 200);
}

#[test]
fn test_06_guide_abs_operator() {
    let ctx = GuideContext::new(1000, 2000);
    assert_eq!(ctx.eval_formula("abs 50").unwrap(), 50);
    assert_eq!(ctx.eval_formula("abs -120").unwrap(), 120);
}

#[test]
fn test_07_guide_at2_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // at2 x y = atan2(y, x) in 60000ths of a degree
    // atan2(0, 1) = 0 deg
    assert_eq!(ctx.eval_formula("at2 1 0").unwrap(), 0);
    // atan2(1, 0) = 90 deg = 5,400,000
    assert_eq!(ctx.eval_formula("at2 0 1").unwrap(), 5_400_000);
    // atan2(1, 1) = 45 deg = 2,700,000
    assert_eq!(ctx.eval_formula("at2 1 1").unwrap(), 2_700_000);
}

#[test]
fn test_08_guide_cat2_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // cat2 x y z = x * cos(atan2(z, y))
    // y=1, z=0 -> angle=0, cos=1 -> x
    assert_eq!(ctx.eval_formula("cat2 500 1 0").unwrap(), 500);
    // y=0, z=1 -> angle=90 deg, cos=0 -> 0
    assert_eq!(ctx.eval_formula("cat2 500 0 1").unwrap(), 0);
}

#[test]
fn test_09_guide_sat2_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // sat2 x y z = x * sin(atan2(z, y))
    // y=1, z=0 -> angle=0, sin=0 -> 0
    assert_eq!(ctx.eval_formula("sat2 500 1 0").unwrap(), 0);
    // y=0, z=1 -> angle=90 deg, sin=1 -> x
    assert_eq!(ctx.eval_formula("sat2 500 0 1").unwrap(), 500);
}

#[test]
fn test_10_guide_cos_sin_tan_operators() {
    let ctx = GuideContext::new(1000, 2000);
    // angle in 60,000ths of a degree
    // 0 deg
    assert_eq!(ctx.eval_formula("cos 1000 0").unwrap(), 1000);
    assert_eq!(ctx.eval_formula("sin 1000 0").unwrap(), 0);
    assert_eq!(ctx.eval_formula("tan 1000 0").unwrap(), 0);

    // 90 deg = 5,400,000
    assert_eq!(ctx.eval_formula("cos 1000 5400000").unwrap(), 0);
    assert_eq!(ctx.eval_formula("sin 1000 5400000").unwrap(), 1000);

    // 45 deg = 2,700,000: tan(45 deg) = 1.0 -> 1000
    assert_eq!(ctx.eval_formula("tan 1000 2700000").unwrap(), 1000);
}

#[test]
fn test_11_guide_sqrt_operator() {
    let ctx = GuideContext::new(1000, 2000);
    assert_eq!(ctx.eval_formula("sqrt 100").unwrap(), 10);
    assert_eq!(ctx.eval_formula("sqrt 0").unwrap(), 0);
    assert_eq!(ctx.eval_formula("sqrt -50").unwrap(), 0);
}

#[test]
fn test_12_guide_min_max_operators() {
    let ctx = GuideContext::new(1000, 2000);
    assert_eq!(ctx.eval_formula("min 50 100").unwrap(), 50);
    assert_eq!(ctx.eval_formula("max 50 100").unwrap(), 100);
    assert_eq!(ctx.eval_formula("min w h").unwrap(), 1000);
    assert_eq!(ctx.eval_formula("max w h").unwrap(), 2000);
}

#[test]
fn test_13_guide_pin_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // pin x y z = clamp y between x and z
    assert_eq!(ctx.eval_formula("pin 10 5 100").unwrap(), 10);
    assert_eq!(ctx.eval_formula("pin 10 50 100").unwrap(), 50);
    assert_eq!(ctx.eval_formula("pin 10 150 100").unwrap(), 100);
}

#[test]
fn test_14_guide_mod_operator() {
    let ctx = GuideContext::new(1000, 2000);
    // mod x y z = sqrt(x^2 + y^2 + z^2)
    // 3, 4, 0 -> 5
    assert_eq!(ctx.eval_formula("mod 3 4 0").unwrap(), 5);
    // 1, 2, 2 -> sqrt(1+4+4) = 3
    assert_eq!(ctx.eval_formula("mod 1 2 2").unwrap(), 3);
}

#[test]
fn test_15_guide_builtin_constants_and_dimensions() {
    let ctx = GuideContext::new(6000, 4000);
    assert_eq!(ctx.get("w"), Some(6000));
    assert_eq!(ctx.get("h"), Some(4000));
    assert_eq!(ctx.get("l"), Some(0));
    assert_eq!(ctx.get("t"), Some(0));
    assert_eq!(ctx.get("r"), Some(6000));
    assert_eq!(ctx.get("b"), Some(4000));
    assert_eq!(ctx.get("hc"), Some(3000));
    assert_eq!(ctx.get("vc"), Some(2000));
    assert_eq!(ctx.get("ss"), Some(4000));
    assert_eq!(ctx.get("ls"), Some(6000));
    assert_eq!(ctx.get("ssd2"), Some(2000));

    // DrawingML angle constants
    assert_eq!(ctx.get("cd2"), Some(10_800_000));
    assert_eq!(ctx.get("cd4"), Some(5_400_000));
    assert_eq!(ctx.get("cd8"), Some(2_700_000));
    assert_eq!(ctx.get("3cd4"), Some(16_200_000));
    assert_eq!(ctx.get("3cd8"), Some(8_100_000));
    assert_eq!(ctx.get("5cd8"), Some(13_500_000));
    assert_eq!(ctx.get("7cd8"), Some(18_900_000));
}

#[test]
fn test_16_guide_sequential_evaluation_chain() {
    let mut ctx = GuideContext::new(1000, 1000);
    let guides = vec![
        GeomGuide::new("g1", "*/ w 1 4"),
        GeomGuide::new("g2", "+- g1 50 0"),
        GeomGuide::new("g3", "*/ g2 2 1"),
    ];
    ctx.evaluate_guides(&guides).unwrap();
    assert_eq!(ctx.get("g1"), Some(250));
    assert_eq!(ctx.get("g2"), Some(300));
    assert_eq!(ctx.get("g3"), Some(600));
}

// =========================================================================
// 2. 187 Preset Geometries Catalog Tests
// =========================================================================

#[test]
fn test_17_all_187_preset_geometries_exist_and_generate() {
    let names = list_preset_names();
    assert_eq!(
        names.len(),
        187,
        "Catalog must contain exactly 187 ECMA-376 preset shapes"
    );

    for name in names {
        let def = get_preset_geometry(name);
        assert!(def.is_some(), "Preset shape '{}' must be defined", name);

        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert!(
            !paths.is_empty(),
            "Preset shape '{}' must generate at least 1 path",
            name
        );
        assert!(
            !paths[0].commands.is_empty(),
            "Preset shape '{}' path must have commands",
            name
        );
    }
}

#[test]
fn test_18_preset_rect_geometry() {
    let paths = GeometryCompiler::compile_preset("rect", 0, 0, 1000, 500, &[]).unwrap();
    assert_eq!(paths.len(), 1);
    let p = &paths[0];
    assert_eq!(p.commands.len(), 5);
    assert_eq!(p.commands[0], PathCommand::MoveTo(Point2D::new(0.0, 0.0)));
    assert_eq!(
        p.commands[1],
        PathCommand::LineTo(Point2D::new(1000.0, 0.0))
    );
    assert_eq!(
        p.commands[2],
        PathCommand::LineTo(Point2D::new(1000.0, 500.0))
    );
    assert_eq!(p.commands[3], PathCommand::LineTo(Point2D::new(0.0, 500.0)));
    assert_eq!(p.commands[4], PathCommand::Close);
}

#[test]
fn test_19_preset_round_rect_geometry_with_adjust() {
    // Default adjust: 16667 (16.67% of min(w, h)) -> min(1000, 1000) * 16667 / 100000 = 166.67
    let paths = GeometryCompiler::compile_preset("roundRect", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(paths.len(), 1);
    let p = &paths[0];
    // Contains 4 lines, 4 arcs, 1 close, 1 move_to -> 10 commands
    assert_eq!(p.commands.len(), 10);

    // Custom adjust: adj = 25000 (radius 250)
    let custom_adj = vec![GeomGuide::new("adj", "val 25000")];
    let paths_adj =
        GeometryCompiler::compile_preset("roundRect", 0, 0, 1000, 1000, &custom_adj).unwrap();
    assert_eq!(paths_adj.len(), 1);
    let p_adj = &paths_adj[0];
    assert_eq!(
        p_adj.commands[0],
        PathCommand::MoveTo(Point2D::new(250.0, 0.0))
    );
}

#[test]
fn test_20_preset_ellipse_geometry() {
    let paths = GeometryCompiler::compile_preset("ellipse", 100, 200, 600, 400, &[]).unwrap();
    assert_eq!(paths.len(), 1);
    let p = &paths[0];
    assert_eq!(p.commands.len(), 4); // MoveTo, ArcTo, ArcTo, Close
    assert_eq!(
        p.commands[0],
        PathCommand::MoveTo(Point2D::new(400.0, 200.0))
    ); // center x + 100 = 300 + 100 = 400
}

#[test]
fn test_21_preset_triangle_and_rt_triangle() {
    let p_tri = GeometryCompiler::compile_preset("triangle", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_tri.len(), 1);
    assert_eq!(
        p_tri[0].commands[0],
        PathCommand::MoveTo(Point2D::new(500.0, 0.0))
    );

    let p_rt = GeometryCompiler::compile_preset("rtTriangle", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_rt.len(), 1);
    assert_eq!(
        p_rt[0].commands[0],
        PathCommand::MoveTo(Point2D::new(0.0, 0.0))
    );
}

#[test]
fn test_22_preset_diamond_parallelogram_trapezoid() {
    let p_diamond = GeometryCompiler::compile_preset("diamond", 0, 0, 800, 600, &[]).unwrap();
    assert_eq!(
        p_diamond[0].commands[0],
        PathCommand::MoveTo(Point2D::new(400.0, 0.0))
    );

    let p_para = GeometryCompiler::compile_preset("parallelogram", 0, 0, 1000, 500, &[]).unwrap();
    assert_eq!(p_para.len(), 1);

    let p_trap = GeometryCompiler::compile_preset("trapezoid", 0, 0, 1000, 500, &[]).unwrap();
    assert_eq!(p_trap.len(), 1);
}

#[test]
fn test_23_preset_regular_polygons_pentagon_to_dodecagon() {
    let polygons = [
        ("pentagon", 5),
        ("hexagon", 6),
        ("heptagon", 7),
        ("octagon", 8),
        ("decagon", 10),
        ("dodecagon", 12),
    ];
    for &(name, sides) in &polygons {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert_eq!(paths.len(), 1);
        // MoveTo + (sides - 1) LineTo + Close = sides + 1 commands
        assert_eq!(paths[0].commands.len(), sides + 1);
    }
}

#[test]
fn test_24_preset_stars_4_to_32() {
    let stars = [
        ("star4", 4),
        ("star5", 5),
        ("star6", 6),
        ("star7", 7),
        ("star8", 8),
        ("star10", 10),
        ("star12", 12),
        ("star16", 16),
        ("star24", 24),
        ("star32", 32),
    ];
    for &(name, points) in &stars {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert_eq!(paths.len(), 1);
        // 2 * points vertices + Close = 2 * points + 1 commands
        assert_eq!(paths[0].commands.len(), 2 * points + 1);
    }
}

#[test]
fn test_25_preset_arrows_4_directions() {
    for name in &["rightArrow", "leftArrow", "upArrow", "downArrow"] {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].commands.len(), 8); // 7 vertices + Close
    }
}

#[test]
fn test_26_preset_chevron_and_homeplate() {
    let p_chev = GeometryCompiler::compile_preset("chevron", 0, 0, 1000, 500, &[]).unwrap();
    assert_eq!(p_chev.len(), 1);
    assert_eq!(p_chev[0].commands.len(), 7); // 6 vertices + Close

    let p_home = GeometryCompiler::compile_preset("homePlate", 0, 0, 1000, 500, &[]).unwrap();
    assert_eq!(p_home.len(), 1);
}

#[test]
fn test_27_preset_curves_heart_lightning_donut_pie() {
    let p_heart = GeometryCompiler::compile_preset("heart", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_heart.len(), 1);
    assert!(!p_heart[0].commands.is_empty());

    let p_light = GeometryCompiler::compile_preset("lightningBolt", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_light.len(), 1);

    let p_donut = GeometryCompiler::compile_preset("donut", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_donut.len(), 2); // outer path + inner hole path

    let p_pie = GeometryCompiler::compile_preset("pie", 0, 0, 1000, 1000, &[]).unwrap();
    assert_eq!(p_pie.len(), 1);
}

#[test]
fn test_28_preset_flowchart_symbols() {
    let flowcharts = [
        "flowChartProcess",
        "flowChartDecision",
        "flowChartInputOutput",
        "flowChartTerminator",
        "flowChartPreparation",
        "flowChartConnector",
        "flowChartPunchedCard",
    ];
    for name in &flowcharts {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert!(!paths.is_empty());
    }
}

#[test]
fn test_29_preset_action_buttons() {
    let buttons = [
        "actionButtonBlank",
        "actionButtonHome",
        "actionButtonHelp",
        "actionButtonInformation",
        "actionButtonForwardNext",
        "actionButtonBackPrevious",
    ];
    for name in &buttons {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert!(!paths.is_empty());
    }
}

#[test]
fn test_30_preset_math_symbols() {
    let math_symbols = [
        "mathPlus",
        "mathMinus",
        "mathMultiply",
        "mathDivide",
        "mathEqual",
        "mathNotEqual",
    ];
    for name in &math_symbols {
        let paths = GeometryCompiler::compile_preset(name, 0, 0, 1000, 1000, &[]).unwrap();
        assert!(!paths.is_empty());
    }
}

// =========================================================================
// 3. Custom Geometry & Path Operations
// =========================================================================

#[test]
fn test_31_custom_geometry_evaluation() {
    let custom = CustomGeometry {
        adjust_values: vec![GeomGuide::new("adj", "val 200")],
        guides: vec![
            GeomGuide::new("x1", "+- l adj 0"), // 200
            GeomGuide::new("x2", "+- r 0 adj"), // 800
        ],
        paths: vec![CustomPath {
            width: Some(1000),
            height: Some(1000),
            fill: PathFillMode::Norm,
            stroke: true,
            instructions: vec![
                PathInstruction::MoveTo {
                    x: "x1".into(),
                    y: "t".into(),
                },
                PathInstruction::LineTo {
                    x: "x2".into(),
                    y: "t".into(),
                },
                PathInstruction::LineTo {
                    x: "r".into(),
                    y: "b".into(),
                },
                PathInstruction::LineTo {
                    x: "l".into(),
                    y: "b".into(),
                },
                PathInstruction::Close,
            ],
        }],
    };

    let paths = GeometryCompiler::compile_custom(&custom, 0, 0, 1000, 1000).unwrap();
    assert_eq!(paths.len(), 1);
    let p = &paths[0];
    assert_eq!(p.commands.len(), 5);
    assert_eq!(p.commands[0], PathCommand::MoveTo(Point2D::new(200.0, 0.0)));
    assert_eq!(p.commands[1], PathCommand::LineTo(Point2D::new(800.0, 0.0)));
}

#[test]
fn test_32_arc_to_cubic_bezier_flattening() {
    let mut path = DrawingPath::new();
    path.move_to(500.0, 0.0);
    // 90 degree arc of radius 500
    path.arc_to(500.0, 500.0, 270.0, 90.0);
    path.close();

    let bezier_path = path.to_cubic_beziers();
    assert_eq!(bezier_path.commands.len(), 3); // MoveTo, CubicTo, Close
    if let PathCommand::CubicTo { to, .. } = bezier_path.commands[1] {
        // End point should be (1000, 500)
        assert!((to.x - 1000.0).abs() < 1e-3);
        assert!((to.y - 500.0).abs() < 1e-3);
    } else {
        panic!("Expected CubicTo command");
    }
}

#[test]
fn test_33_svg_path_data_export() {
    let mut path = DrawingPath::new();
    path.move_to(10.0, 20.0);
    path.line_to(100.0, 20.0);
    path.line_to(100.0, 80.0);
    path.close();

    let svg_d = path.to_svg_path_data();
    assert_eq!(svg_d, "M 10.00 20.00 L 100.00 20.00 L 100.00 80.00 Z");
}

#[test]
fn test_34_bounding_box_computation() {
    let mut path = DrawingPath::new();
    path.move_to(50.0, 100.0);
    path.line_to(450.0, 100.0);
    path.line_to(450.0, 300.0);
    path.line_to(50.0, 300.0);
    path.close();

    let (min_x, min_y, max_x, max_y) = path.bounds();
    assert_eq!(min_x, 50.0);
    assert_eq!(min_y, 100.0);
    assert_eq!(max_x, 450.0);
    assert_eq!(max_y, 300.0);
}

#[test]
fn test_35_point_in_path_hit_testing() {
    let mut path = DrawingPath::new();
    path.move_to(0.0, 0.0);
    path.line_to(100.0, 0.0);
    path.line_to(100.0, 100.0);
    path.line_to(0.0, 100.0);
    path.close();

    // Inside
    assert!(path.contains_point(Point2D::new(50.0, 50.0)));
    assert!(path.contains_point(Point2D::new(10.0, 90.0)));

    // Outside
    assert!(!path.contains_point(Point2D::new(-10.0, 50.0)));
    assert!(!path.contains_point(Point2D::new(150.0, 50.0)));
    assert!(!path.contains_point(Point2D::new(50.0, -10.0)));
    assert!(!path.contains_point(Point2D::new(50.0, 110.0)));
}

#[test]
fn test_36_resolved_shape_evaluate_paths_integration() {
    let shape = ResolvedShape {
        id: 1,
        name: "Test Star".to_string(),
        is_text_box: false,
        placeholder_type: None,
        transform: Transform2D::new(100, 200, 500, 500),
        geometry: Geometry::Preset("star5".to_string()),
        fill: Some(Fill::Solid(ColorChoice::Srgb("FF0000".to_string()))),
        line: None,
        paragraphs: Vec::new(),
    };

    let paths = shape.evaluate_paths().expect("Must evaluate star5 paths");
    assert_eq!(paths.len(), 1);
    let (min_x, min_y, max_x, max_y) = paths[0].bounds();
    assert!(min_x >= 100.0);
    assert!(min_y >= 200.0);
    assert!(max_x <= 600.0);
    assert!(max_y <= 700.0);
}

// =========================================================================
// 4. Evidence Manifest Validation
// =========================================================================

#[test]
fn test_37_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/ppt/4.1.2_vectors.json");
    let json_bytes = std::fs::read(&path).expect("Must read specs/evidence/ppt/4.1.2_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let items = vectors["vectors"]
        .as_array()
        .expect("vectors array in manifest");
    assert!(
        items.len() >= 30,
        "Manifest must contain at least 30 verified vectors, found {}",
        items.len()
    );

    for item in items {
        assert!(item["id"].is_string());
        assert!(item["category"].is_string());
        assert!(item["description"].is_string());
        assert_eq!(item["status"].as_str(), Some("PASS"));
    }
}
