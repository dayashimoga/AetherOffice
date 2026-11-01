use ppt_engine::animation::TriggerType;
use ppt_engine::animation_bridge::{MorphTransitionBridge, SmilAnimationBridge};
use ppt_engine::model::{
    BodyProperties, ColorChoice, Fill, Geometry, Shape, ShapeElement, ShapeProperties, TextBody,
    TextParagraph, TextRunElement, Transform2D,
};
use ppt_engine::package::PresentationPackage;
use render_core::timeline_scrubber::{AffineMatrix2D, FpsTarget, PlaybackState, TimelineScrubber};

fn create_test_shape(id: u32, name: &str, x: i64, y: i64, cx: u32, cy: u32) -> Shape {
    Shape {
        id,
        name: name.to_string(),
        is_text_box: false,
        placeholder: None,
        properties: ShapeProperties {
            transform: Some(Transform2D::new(x, y, cx, cy)),
            geometry: Some(Geometry::Preset("rect".to_string())),
            fill: Some(Fill::Solid(ColorChoice::Srgb("0078D7".to_string()))),
            line: None,
        },
        text_body: None,
        raw_elements: Vec::new(),
    }
}

fn create_test_text_shape(
    id: u32,
    name: &str,
    x: i64,
    y: i64,
    cx: u32,
    cy: u32,
    text: &str,
) -> Shape {
    let mut s = create_test_shape(id, name, x, y, cx, cy);
    s.text_body = Some(TextBody {
        body_properties: BodyProperties::default(),
        list_style: None,
        paragraphs: vec![TextParagraph {
            properties: None,
            runs: vec![TextRunElement::Text(text.to_string(), None)],
            end_paragraph_run_properties: None,
        }],
    });
    s
}

#[test]
fn test_1_morph_bridge_matching_explicit_bang_bang() {
    let mut pkg = PresentationPackage::new();
    // Slide 1 has shape "!!HeroCard"
    let s1 = create_test_shape(10, "!!HeroCard", 1_000_000, 1_000_000, 2_000_000, 1_000_000);
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    // Duplicate slide 0 to create slide 1 with morphed shape "!!HeroCard" at new position
    let s2 = create_test_shape(20, "!!HeroCard", 4_000_000, 2_000_000, 4_000_000, 2_000_000);
    let mut slide2 = pkg.slides[0].1.clone();
    slide2.common_slide_data.shape_tree.shapes.clear();
    slide2
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));
    pkg.slides
        .push(("ppt/slides/slide2.xml".to_string(), slide2));

    let res = MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, 0.5, "byObject")
        .expect("Morph evaluation should succeed");

    assert_eq!(res.t, 0.5);
    assert!(!res.shapes.is_empty());
    let morphed_hero = res
        .shapes
        .iter()
        .find(|s| s.name == "!!HeroCard")
        .expect("HeroCard should exist");
    // At t=0.5, X should be halfway between 1,000,000 and 4,000,000 = 2,500,000
    assert_eq!(morphed_hero.transform.x, 2_500_000.0);
    assert_eq!(morphed_hero.transform.y, 1_500_000.0);
    assert_eq!(morphed_hero.transform.width, 3_000_000.0);
}

#[test]
fn test_2_morph_bridge_continuous_interpolation_t() {
    let mut pkg = PresentationPackage::new();
    let s1 = create_test_shape(1, "Box", 0, 0, 1_000_000, 1_000_000);
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    let s2 = create_test_shape(2, "Box", 10_000_000, 5_000_000, 2_000_000, 2_000_000);
    let mut slide2 = pkg.slides[0].1.clone();
    slide2.common_slide_data.shape_tree.shapes.clear();
    slide2
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));
    pkg.slides
        .push(("ppt/slides/slide2.xml".to_string(), slide2));

    // Sample at t = 0.0, 0.25, 0.75, 1.0
    for progress in [0.0, 0.25, 0.75, 1.0] {
        let res =
            MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, progress, "byObject")
                .unwrap();
        assert_eq!(res.t, progress);
        let shape = &res.shapes[0];
        let expected_x = progress * 10_000_000.0;
        let expected_y = progress * 5_000_000.0;
        assert!((shape.transform.x - expected_x).abs() < 1e-4);
        assert!((shape.transform.y - expected_y).abs() < 1e-4);
    }
}

#[test]
fn test_3_morph_bridge_text_morphing_options() {
    let mut pkg = PresentationPackage::new();
    let s1 = create_test_text_shape(1, "TextShape", 100, 100, 500, 200, "Alpha Beta");
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    let s2 = create_test_text_shape(2, "TextShape", 100, 100, 500, 200, "Alpha Gamma");
    let mut slide2 = pkg.slides[0].1.clone();
    slide2.common_slide_data.shape_tree.shapes.clear();
    slide2
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));
    pkg.slides
        .push(("ppt/slides/slide2.xml".to_string(), slide2));

    for opt in ["byObject", "byWord", "byChar"] {
        let res = MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, 0.5, opt).unwrap();
        assert!(!res.shapes.is_empty());
    }
}

#[test]
fn test_4_morph_bridge_unmatched_shape_crossfade() {
    let mut pkg = PresentationPackage::new();
    let s1 = create_test_text_shape(
        1,
        "OldShapeOnly",
        1_000_000,
        1_000_000,
        1_000_000,
        1_000_000,
        "UniqueTextA",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    let mut s2 = create_test_text_shape(
        2,
        "BrandNewShapeOnly",
        8_000_000,
        8_000_000,
        4_000_000,
        3_000_000,
        "UniqueTextB",
    );
    s2.properties.geometry = Some(Geometry::Preset("ellipse".to_string()));
    let mut slide2 = pkg.slides[0].1.clone();
    slide2.common_slide_data.shape_tree.shapes.clear();
    slide2
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));
    pkg.slides
        .push(("ppt/slides/slide2.xml".to_string(), slide2));

    let res_start =
        MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, 0.0, "byObject").unwrap();
    let old_start = res_start
        .shapes
        .iter()
        .find(|s| s.name == "OldShapeOnly")
        .unwrap();
    assert_eq!(old_start.opacity, 1.0);

    let res_mid =
        MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, 0.5, "byObject").unwrap();
    let old_mid = res_mid
        .shapes
        .iter()
        .find(|s| s.name == "OldShapeOnly")
        .unwrap();
    assert_eq!(old_mid.opacity, 0.5);
    let new_mid = res_mid
        .shapes
        .iter()
        .find(|s| s.name == "BrandNewShapeOnly")
        .unwrap();
    assert_eq!(new_mid.opacity, 0.5);

    let res_end =
        MorphTransitionBridge::evaluate_morph_transition(&pkg, 0, 1, 1.0, "byObject").unwrap();
    let old_end = res_end
        .shapes
        .iter()
        .find(|s| s.name == "OldShapeOnly")
        .unwrap();
    assert_eq!(old_end.opacity, 0.0);
    let new_end = res_end
        .shapes
        .iter()
        .find(|s| s.name == "BrandNewShapeOnly")
        .unwrap();
    assert_eq!(new_end.opacity, 1.0);
}

#[test]
fn test_5_smil_bridge_extract_animation_tracks() {
    let mut pkg = PresentationPackage::new();
    let slide = &mut pkg.slides[0].1;
    let s1 = create_test_shape(101, "HeaderTitle", 100, 100, 400, 100);
    let s2 = create_test_shape(102, "ChartPanel", 100, 250, 400, 300);
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));

    // Inject 2 animations
    SmilAnimationBridge::add_animation_effect(slide, 101, "Fade", TriggerType::OnClick, 1000);
    SmilAnimationBridge::add_animation_effect(slide, 102, "Motion", TriggerType::AfterPrev, 1500);

    let tracks_res = SmilAnimationBridge::extract_slide_animation_tracks(&pkg, 0)
        .expect("Tracks extraction should succeed");

    assert_eq!(tracks_res.slide_index, 0);
    assert_eq!(tracks_res.tracks.len(), 2);
    assert_eq!(tracks_res.tracks[0].shape_id, 101);
    assert_eq!(tracks_res.tracks[0].shape_name, "HeaderTitle");
    assert_eq!(tracks_res.tracks[1].shape_id, 102);
    assert_eq!(tracks_res.tracks[1].shape_name, "ChartPanel");
    assert_eq!(tracks_res.tracks[1].duration_ms, 1500.0);
}

#[test]
fn test_6_smil_bridge_deterministic_120hz_scrubbing() {
    let mut pkg = PresentationPackage::new();
    let slide = &mut pkg.slides[0].1;
    let s1 = create_test_shape(101, "Box", 0, 0, 100, 100);
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    SmilAnimationBridge::add_animation_effect(slide, 101, "Scale", TriggerType::OnClick, 1000);

    let scene_0 = SmilAnimationBridge::scrub_animation(&pkg, 0, 0.0, Some(1)).unwrap();
    assert_eq!(scene_0.time_ms, 0.0);

    let scene_mid = SmilAnimationBridge::scrub_animation(&pkg, 0, 500.0, Some(1)).unwrap();
    assert_eq!(scene_mid.time_ms, 500.0);
    let shape_mid = scene_mid.shapes.iter().find(|s| s.shape_id == 101).unwrap();
    // Scale interpolates from 1.0 to 1.5 -> at 500ms it should be 1.25
    assert!((shape_mid.scale_x - 1.25).abs() < 1e-3);

    let scene_end = SmilAnimationBridge::scrub_animation(&pkg, 0, 1000.0, Some(1)).unwrap();
    let shape_end = scene_end.shapes.iter().find(|s| s.shape_id == 101).unwrap();
    assert!((shape_end.scale_x - 1.5).abs() < 1e-3);
}

#[test]
fn test_7_smil_bridge_click_advancement() {
    let mut pkg = PresentationPackage::new();
    let slide = &mut pkg.slides[0].1;
    let s1 = create_test_shape(10, "Title", 0, 0, 100, 50);
    let s2 = create_test_shape(20, "Subtitle", 0, 60, 100, 50);
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));

    // Click 1: Title
    SmilAnimationBridge::add_animation_effect(slide, 10, "Fade", TriggerType::OnClick, 1000);
    // Click 2: Subtitle
    SmilAnimationBridge::add_animation_effect(slide, 20, "Fade", TriggerType::OnClick, 1000);

    let res_next = SmilAnimationBridge::advance_click(&pkg, 0, 0, true).unwrap();
    assert_eq!(res_next.click_index, 1);

    let res_prev = SmilAnimationBridge::advance_click(&pkg, 0, 1, false).unwrap();
    assert_eq!(res_prev.click_index, 0);
}

#[test]
fn test_8_render_core_affine_matrix_composition() {
    let mat = AffineMatrix2D::from_transform(100.0, 50.0, 0.0, 2.0, 2.0);
    let (px, py) = mat.transform_point(10.0, 20.0);
    // x' = 2.0 * 10.0 + 100.0 = 120.0
    // y' = 2.0 * 20.0 + 50.0 = 90.0
    assert_eq!(px, 120.0);
    assert_eq!(py, 90.0);

    // 90 degree rotation
    let mat_rot = AffineMatrix2D::from_transform(0.0, 0.0, 90.0, 1.0, 1.0);
    let (rx, ry) = mat_rot.transform_point(1.0, 0.0);
    assert!(rx.abs() < 1e-6);
    assert!((ry - 1.0).abs() < 1e-6);
}

#[test]
fn test_9_render_core_gpu_matrix4_uniform() {
    let mat = AffineMatrix2D::from_transform(30.0, 40.0, 0.0, 1.5, 1.5);
    let gpu_mat = mat.to_gpu_matrix4();
    assert_eq!(gpu_mat.len(), 16);
    // Column 0: [a, b, 0, 0]
    assert_eq!(gpu_mat[0], 1.5);
    assert_eq!(gpu_mat[1], 0.0);
    // Column 1: [c, d, 0, 0]
    assert_eq!(gpu_mat[4], 0.0);
    assert_eq!(gpu_mat[5], 1.5);
    // Column 3: [tx, ty, 0, 1]
    assert_eq!(gpu_mat[12], 30.0);
    assert_eq!(gpu_mat[13], 40.0);
    assert_eq!(gpu_mat[15], 1.0);
}

#[test]
fn test_10_render_core_timeline_scrubber_pacing() {
    let mut scrubber = TimelineScrubber::new(1000.0, FpsTarget::Fps60);
    assert_eq!(scrubber.playback_state, PlaybackState::Stopped);
    assert_eq!(scrubber.current_time_ms, 0.0);

    scrubber.play();
    assert_eq!(scrubber.playback_state, PlaybackState::Playing);

    // Tick one frame at 60fps (~16.667ms)
    scrubber.tick(None);
    assert!((scrubber.current_time_ms - 16.666_666).abs() < 1e-2);
    assert!(scrubber.progress() > 0.0 && scrubber.progress() < 0.1);

    scrubber.seek_progress(0.5);
    assert_eq!(scrubber.current_time_ms, 500.0);
    assert_eq!(scrubber.progress(), 0.5);

    scrubber.seek_ms(990.0);
    scrubber.tick(Some(20.0));
    // Without loop, reaches duration and pauses
    assert_eq!(scrubber.current_time_ms, 1000.0);
    assert_eq!(scrubber.playback_state, PlaybackState::Paused);

    // With loop enabled
    scrubber.set_loop(true);
    scrubber.play();
    scrubber.tick(Some(20.0));
    assert_eq!(scrubber.current_time_ms, 20.0);
}
