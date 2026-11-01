#![forbid(unsafe_code)]
//! Sub-Phase 48.1.2 Integration Tests: SmartArt Builder, Grouping/Z-Order & Smart Guides
//! GEC-v7.0 Strict Clean-Room Test Suite

use ppt_engine::layers::SlideLayerEngine;
use ppt_engine::model::*;
use ppt_engine::package::PresentationPackage;
use ppt_engine::smartart_builder::{SmartArtBuildRequest, SmartArtBuilder, SmartArtItemDto};
use render_core::smart_guides::{GuideKind, GuideOrientation, RectBounds, SmartGuidesEngine};

fn create_test_shape(id: u32, name: &str, x: i64, y: i64, cx: u32, cy: u32) -> ShapeElement {
    ShapeElement::Shape(Box::new(Shape {
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
    }))
}

#[test]
fn test_1_group_shapes_composite_bounding_box() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(
            10, "Box1", 100_000, 200_000, 300_000, 400_000,
        ));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(
            11, "Box2", 500_000, 600_000, 200_000, 300_000,
        ));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(
            12, "Box3", 1_000_000, 1_000_000, 500_000, 500_000,
        ));

    let group_id =
        SlideLayerEngine::group_shapes(slide, &[10, 11]).expect("Should group Box1 and Box2");

    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 2);
    let group_elem = slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .find(|s| s.id() == group_id)
        .expect("Group shape must exist in shape tree");

    if let ShapeElement::Group(ref grp) = group_elem {
        assert_eq!(grp.children.len(), 2);
        let xf = grp.transform.as_ref().expect("Group transform");
        // min_x: 100_000, min_y: 200_000, max_x: 700_000, max_y: 900_000
        assert_eq!(xf.x, 100_000);
        assert_eq!(xf.y, 200_000);
        assert_eq!(xf.cx, 600_000);
        assert_eq!(xf.cy, 700_000);

        // Child 1 local coords: 100_000 - 100_000 = 0, 200_000 - 200_000 = 0
        if let ShapeElement::Shape(ref c1) = grp.children[0] {
            let c_xf = c1.properties.transform.as_ref().expect("Child transform");
            assert_eq!(c_xf.x, 0);
            assert_eq!(c_xf.y, 0);
        } else {
            panic!("Expected shape child");
        }
    } else {
        panic!("Expected group shape element");
    }
}

#[test]
fn test_2_ungroup_shapes_restores_parent_coordinates() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(
            10, "Box1", 100_000, 200_000, 300_000, 400_000,
        ));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(
            11, "Box2", 500_000, 600_000, 200_000, 300_000,
        ));

    let group_id = SlideLayerEngine::group_shapes(slide, &[10, 11]).expect("Group");
    let child_ids = SlideLayerEngine::ungroup_shapes(slide, group_id).expect("Ungroup");

    assert_eq!(child_ids, vec![10, 11]);
    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 2);

    let sh1 = slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .find(|s| s.id() == 10)
        .expect("sh1");
    if let ShapeElement::Shape(ref sh) = sh1 {
        let xf = sh.properties.transform.as_ref().expect("xf");
        assert_eq!(xf.x, 100_000);
        assert_eq!(xf.y, 200_000);
    } else {
        panic!("Expected Shape");
    }
}

#[test]
fn test_3_nested_grouping_support() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(1, "A", 100, 100, 50, 50));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(2, "B", 200, 200, 50, 50));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(3, "C", 400, 400, 50, 50));

    let g1 = SlideLayerEngine::group_shapes(slide, &[1, 2]).expect("Group 1");
    let g2 = SlideLayerEngine::group_shapes(slide, &[g1, 3]).expect("Group 2");

    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 1);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[0].id(), g2);

    // Ungroup outer group
    let children_g2 = SlideLayerEngine::ungroup_shapes(slide, g2).expect("Ungroup g2");
    assert_eq!(children_g2, vec![g1, 3]);
    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 2);

    // Ungroup inner group
    let children_g1 = SlideLayerEngine::ungroup_shapes(slide, g1).expect("Ungroup g1");
    assert_eq!(children_g1, vec![1, 2]);
    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 3);
}

#[test]
fn test_4_z_order_bring_to_front_and_send_to_back() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(1, "Bottom", 0, 0, 100, 100));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(2, "Middle", 10, 10, 100, 100));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(3, "Top", 20, 20, 100, 100));

    // Bring Bottom (1) to front
    SlideLayerEngine::bring_to_front(slide, 1).expect("Bring to front");
    assert_eq!(slide.common_slide_data.shape_tree.shapes[0].id(), 2);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[1].id(), 3);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[2].id(), 1);

    // Send it back to bottom
    SlideLayerEngine::send_to_back(slide, 1).expect("Send to back");
    assert_eq!(slide.common_slide_data.shape_tree.shapes[0].id(), 1);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[1].id(), 2);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[2].id(), 3);
}

#[test]
fn test_5_z_order_bring_forward_and_send_backward() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(1, "S1", 0, 0, 100, 100));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(2, "S2", 10, 10, 100, 100));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(3, "S3", 20, 20, 100, 100));

    // Bring S1 forward once (swaps with S2)
    SlideLayerEngine::bring_forward(slide, 1).expect("Bring forward");
    assert_eq!(slide.common_slide_data.shape_tree.shapes[0].id(), 2);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[1].id(), 1);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[2].id(), 3);

    // Send S1 backward once (swaps back with S2)
    SlideLayerEngine::send_backward(slide, 1).expect("Send backward");
    assert_eq!(slide.common_slide_data.shape_tree.shapes[0].id(), 1);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[1].id(), 2);
    assert_eq!(slide.common_slide_data.shape_tree.shapes[2].id(), 3);
}

#[test]
fn test_6_reorder_entire_shape_tree() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(1, "A", 0, 0, 10, 10));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(2, "B", 0, 0, 10, 10));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(3, "C", 0, 0, 10, 10));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(4, "D", 0, 0, 10, 10));

    // Reorder as [3, 1, 4, 2]
    SlideLayerEngine::reorder_shapes(slide, &[3, 1, 4, 2]).expect("Reorder");
    let ids: Vec<u32> = slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .map(|s| s.id())
        .collect();
    assert_eq!(ids, vec![3, 1, 4, 2]);
}

#[test]
fn test_7_extract_layer_tree_hierarchical() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(1, "TextCard", 100, 100, 200, 200));
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(2, "BoxCard", 400, 400, 200, 200));

    let group_id = SlideLayerEngine::group_shapes(slide, &[1, 2]).expect("Group");
    slide
        .common_slide_data
        .shape_tree
        .shapes
        .push(create_test_shape(3, "Independent", 800, 800, 100, 100));

    let tree = SlideLayerEngine::extract_layer_tree(slide, 0);
    assert_eq!(tree.slide_index, 0);
    assert_eq!(tree.items.len(), 2);

    let grp_item = &tree.items[0];
    assert_eq!(grp_item.id, group_id);
    assert_eq!(grp_item.kind, "group");
    assert_eq!(grp_item.children.len(), 2);
    assert_eq!(grp_item.children[0].id, 1);
    assert_eq!(grp_item.children[1].id, 2);

    let indep_item = &tree.items[1];
    assert_eq!(indep_item.id, 3);
    assert_eq!(indep_item.kind, "shape");
}

#[test]
fn test_8_build_smartart_process_and_hierarchy() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    // 1. Process diagram
    let proc_req = SmartArtBuildRequest {
        layout_type: "process".to_string(),
        items: vec![
            SmartArtItemDto {
                title: "Phase 1".to_string(),
                description: Some("Plan".to_string()),
                level: 0,
                color_hex: None,
            },
            SmartArtItemDto {
                title: "Phase 2".to_string(),
                description: Some("Execute".to_string()),
                level: 0,
                color_hex: None,
            },
            SmartArtItemDto {
                title: "Phase 3".to_string(),
                description: Some("Review".to_string()),
                level: 0,
                color_hex: None,
            },
        ],
        x: 100_000,
        y: 100_000,
        cx: 6_000_000,
        cy: 2_000_000,
        palette: None,
    };
    let proc_group_id = SmartArtBuilder::build_and_insert(slide, proc_req).expect("Build process");
    assert!(slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .any(|s| s.id() == proc_group_id));

    // 2. Hierarchy diagram
    let hier_req = SmartArtBuildRequest {
        layout_type: "hierarchy".to_string(),
        items: vec![
            SmartArtItemDto {
                title: "CEO".to_string(),
                description: None,
                level: 0,
                color_hex: None,
            },
            SmartArtItemDto {
                title: "VP Product".to_string(),
                description: None,
                level: 1,
                color_hex: None,
            },
            SmartArtItemDto {
                title: "VP Engineering".to_string(),
                description: None,
                level: 1,
                color_hex: None,
            },
        ],
        x: 100_000,
        y: 2_500_000,
        cx: 6_000_000,
        cy: 3_000_000,
        palette: None,
    };
    let hier_group_id =
        SmartArtBuilder::build_and_insert(slide, hier_req).expect("Build hierarchy");
    assert!(slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .any(|s| s.id() == hier_group_id));
}

#[test]
fn test_9_build_smartart_cycle_matrix_pyramid() {
    let mut pkg = PresentationPackage::create_empty();
    let slide = &mut pkg.slides[0].1;
    slide.common_slide_data.shape_tree.shapes.clear();

    let items = vec![
        SmartArtItemDto {
            title: "A".to_string(),
            description: None,
            level: 0,
            color_hex: None,
        },
        SmartArtItemDto {
            title: "B".to_string(),
            description: None,
            level: 0,
            color_hex: None,
        },
        SmartArtItemDto {
            title: "C".to_string(),
            description: None,
            level: 0,
            color_hex: None,
        },
        SmartArtItemDto {
            title: "D".to_string(),
            description: None,
            level: 0,
            color_hex: None,
        },
    ];

    // Cycle
    let c_id = SmartArtBuilder::build_and_insert(
        slide,
        SmartArtBuildRequest {
            layout_type: "cycle".to_string(),
            items: items.clone(),
            x: 0,
            y: 0,
            cx: 3_000_000,
            cy: 3_000_000,
            palette: None,
        },
    )
    .expect("Cycle");
    assert!(slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .any(|s| s.id() == c_id));

    // Matrix
    let m_id = SmartArtBuilder::build_and_insert(
        slide,
        SmartArtBuildRequest {
            layout_type: "matrix".to_string(),
            items: items.clone(),
            x: 3_500_000,
            y: 0,
            cx: 3_000_000,
            cy: 3_000_000,
            palette: None,
        },
    )
    .expect("Matrix");
    assert!(slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .any(|s| s.id() == m_id));

    // Pyramid
    let p_id = SmartArtBuilder::build_and_insert(
        slide,
        SmartArtBuildRequest {
            layout_type: "pyramid".to_string(),
            items,
            x: 0,
            y: 3_500_000,
            cx: 3_000_000,
            cy: 3_000_000,
            palette: None,
        },
    )
    .expect("Pyramid");
    assert!(slide
        .common_slide_data
        .shape_tree
        .shapes
        .iter()
        .any(|s| s.id() == p_id));
}

#[test]
fn test_10_render_core_smart_guides_magnetic_snapping() {
    let engine = SmartGuidesEngine::new(5.0);
    let moving = RectBounds::new(102.0, 98.0, 100.0, 100.0);
    let neighbor = RectBounds::new(100.0, 300.0, 100.0, 100.0);
    let canvas = RectBounds::new(0.0, 0.0, 1000.0, 1000.0);

    let snap = engine.compute_snap(&moving, &[neighbor], Some(&canvas));

    // moving.left is 102.0, neighbor.left is 100.0 -> delta is -2.0 (<= 5.0 threshold)
    assert_eq!(snap.snapped_x, 100.0);
    assert_eq!(snap.dx, -2.0);
    assert!(snap
        .guides
        .iter()
        .any(|g| g.orientation == GuideOrientation::Vertical && g.kind == GuideKind::Edge));
}
