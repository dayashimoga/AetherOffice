//! Subphase 5.1.1 Integration Test Suite:
//! Unified 120fps WebGPU / Cosmic-Text Render Core
//!
//! Validates:
//! - GPU draw primitives (`QuadInstance`, `GlyphRun`, `BezierPathBatch`)
//! - Viewport culling and spatial grid indexing (`SpatialGrid`)
//! - Subpixel text shaping and BiDi / RTL / CJK handling (`ShapedTextEngine`)
//! - Word, Excel, and PowerPoint display list compilers
//! - Headless SVG export and pixel buffer rasterization
//! - 120Hz frame pacing SLA (< 2.0 ms display list compilation for 4K viewport)
//! - Vector manifest validation against `specs/evidence/render/5.1.1_vectors.json`

use ppt_engine::{
    DrawingPath, MorphedShapeState, MorphedSlideState, MorphedTransform, PathCommand, Point2D,
    Slide,
};
use render_core::{
    BezierPathBatch, ColorRgba, Compositor, DisplayCommand, DisplayList, ExcelDisplayCell,
    ExcelDisplayCompiler, ExcelSelectionRange, PptDisplayCompiler, QuadInstance, Rect2D,
    ShapedTextEngine, SlideCanvasConfig, SoftwareRasterizer, SpatialGrid, TextScript,
    TransformMatrix, ViewportConfig, WordDisplayCompiler, WordDisplayLine, WordDisplayPage,
    WordDisplayTableCell,
};
use std::path::{Path, PathBuf};

fn resolve_repo_path(rel_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root");
    repo_root.join(rel_path)
}

#[test]
fn test_01_color_rgba_hex_parsing_and_premultiplied() {
    let red = ColorRgba::from_hex("#FF0000").expect("Parse #FF0000");
    assert_eq!(red, ColorRgba::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(red.to_hex(), "#FF0000");

    let green_half = ColorRgba::from_hex("#00FF0080").expect("Parse #00FF0080");
    assert!((green_half.r - 0.0).abs() < 1e-4);
    assert!((green_half.g - 1.0).abs() < 1e-4);
    assert!((green_half.b - 0.0).abs() < 1e-4);
    assert!((green_half.a - 128.0 / 255.0).abs() < 1e-3);

    let premul = green_half.to_premultiplied();
    assert!((premul.g - (128.0 / 255.0)).abs() < 1e-3);
    assert!((premul.a - (128.0 / 255.0)).abs() < 1e-3);
}

#[test]
fn test_02_rect2d_intersections() {
    let r_a = Rect2D::new(0.0, 0.0, 100.0, 100.0);
    let r_overlap = Rect2D::new(50.0, 50.0, 100.0, 100.0);
    let r_disjoint = Rect2D::new(200.0, 200.0, 50.0, 50.0);

    assert!(r_a.intersects(&r_overlap));
    assert!(r_overlap.intersects(&r_a));
    assert!(!r_a.intersects(&r_disjoint));
    assert!(!r_disjoint.intersects(&r_a));
}

#[test]
fn test_03_rect2d_union_and_containment() {
    let r1 = Rect2D::new(10.0, 20.0, 100.0, 50.0);
    let r2 = Rect2D::new(50.0, 40.0, 120.0, 80.0);

    let u = r1.union(&r2);
    assert_eq!(u.x, 10.0);
    assert_eq!(u.y, 20.0);
    assert_eq!(u.width, 160.0);
    assert_eq!(u.height, 100.0);

    assert!(u.contains_point(30.0, 40.0));
    assert!(u.contains_point(10.0, 20.0));
    assert!(u.contains_point(170.0, 120.0));
    assert!(!u.contains_point(200.0, 200.0));
}

#[test]
fn test_04_transform_matrix_affine() {
    let t = TransformMatrix::translate(100.0, 200.0);
    let s = TransformMatrix::scale(2.0, 3.0);
    // Combine: first scale, then translate
    let combined = s.multiply(&t);

    let (px, py) = combined.apply_point(10.0, 10.0);
    assert_eq!(px, 120.0);
    assert_eq!(py, 230.0);

    let r = Rect2D::new(0.0, 0.0, 50.0, 50.0);
    let mapped = combined.apply_rect(&r);
    assert_eq!(mapped.x, 100.0);
    assert_eq!(mapped.y, 200.0);
    assert_eq!(mapped.width, 100.0);
    assert_eq!(mapped.height, 150.0);
}

#[test]
fn test_05_quad_instance_attributes() {
    let q_solid = QuadInstance::solid(Rect2D::new(10.0, 10.0, 100.0, 50.0), ColorRgba::BLACK);
    assert_eq!(q_solid.rect.width, 100.0);
    assert_eq!(q_solid.border_width, 0.0);

    let q_bordered = QuadInstance::bordered(
        Rect2D::new(0.0, 0.0, 200.0, 100.0),
        ColorRgba::WHITE,
        ColorRgba::BLACK,
        2.5,
    );
    assert_eq!(q_bordered.border_width, 2.5);
    assert_eq!(q_bordered.border_color, Some(ColorRgba::BLACK));

    let q_rounded =
        QuadInstance::rounded(Rect2D::new(0.0, 0.0, 100.0, 100.0), ColorRgba::BLACK, 8.0);
    assert_eq!(q_rounded.corner_radius, 8.0);
}

#[test]
fn test_06_glyph_instance_subpixel_metrics() {
    let run = ShapedTextEngine::shape_text("Hello", "Calibri", 16.0, ColorRgba::BLACK, (0.0, 0.0));
    assert_eq!(run.glyphs.len(), 5);
    assert_eq!(run.glyphs[0].x, 0.0);
    // Subpixel advances must be monotonically increasing
    for i in 1..run.glyphs.len() {
        assert!(run.glyphs[i].x > run.glyphs[i - 1].x);
    }
}

#[test]
fn test_07_glyph_run_bidi_detection() {
    let ltr = "Quarterly Sales Report";
    assert_eq!(ShapedTextEngine::detect_bidi_level(ltr), 0);
    assert_eq!(ShapedTextEngine::detect_script(ltr), TextScript::Latin);

    let arabic = "تقرير المبيعات";
    assert_eq!(ShapedTextEngine::detect_bidi_level(arabic), 1);
    assert_eq!(ShapedTextEngine::detect_script(arabic), TextScript::Arabic);

    let hebrew = "דוח מכירות";
    assert_eq!(ShapedTextEngine::detect_bidi_level(hebrew), 1);
    assert_eq!(ShapedTextEngine::detect_script(hebrew), TextScript::Hebrew);
}

#[test]
fn test_08_glyph_run_cjk_detection() {
    let cjk = "売上高推移表";
    assert_eq!(ShapedTextEngine::detect_script(cjk), TextScript::Cjk);

    let run = ShapedTextEngine::shape_text(cjk, "MS Gothic", 20.0, ColorRgba::BLACK, (0.0, 0.0));
    assert_eq!(run.glyphs.len(), 6);
    assert_eq!(run.script, TextScript::Cjk);
    // In CJK monospace, each character width is equal to font_size_px
    for g in &run.glyphs {
        assert_eq!(g.width, 20.0);
    }
}

#[test]
fn test_09_bezier_path_batch_commands() {
    let path = DrawingPath {
        commands: vec![
            PathCommand::MoveTo(Point2D { x: 0.0, y: 0.0 }),
            PathCommand::LineTo(Point2D { x: 300.0, y: 0.0 }),
            PathCommand::LineTo(Point2D { x: 150.0, y: 200.0 }),
            PathCommand::Close,
        ],
        fill: ppt_engine::PathFillMode::Norm,
        stroke: true,
        width: None,
        height: None,
    };

    let batch = BezierPathBatch {
        path,
        fill_color: Some(ColorRgba::new(0.0, 0.5, 1.0, 1.0)),
        stroke_color: Some(ColorRgba::BLACK),
        stroke_width: 2.0,
        bounds: Rect2D::new(0.0, 0.0, 300.0, 200.0),
    };

    assert_eq!(batch.bounds.width, 300.0);
    assert_eq!(batch.bounds.height, 200.0);
    assert_eq!(batch.path.commands.len(), 4);
}

#[test]
fn test_10_spatial_grid_indexing() {
    let mut grid = SpatialGrid::new(256.0);
    let item_rect = Rect2D::new(100.0, 200.0, 300.0, 150.0);
    grid.insert(item_rect, "TestBox");

    let ((c_min, r_min), (c_max, r_max)) = grid.tile_range(&item_rect);
    assert_eq!(c_min, 0);
    assert_eq!(c_max, 1); // 400 / 256 = 1
    assert_eq!(r_min, 0);
    assert_eq!(r_max, 1); // 350 / 256 = 1
}

#[test]
fn test_11_viewport_culling_query() {
    let mut grid = SpatialGrid::new(256.0);
    grid.insert(Rect2D::new(100.0, 100.0, 200.0, 200.0), "OnscreenItem");
    grid.insert(Rect2D::new(1200.0, 900.0, 200.0, 200.0), "OffscreenItem");

    let viewport = Rect2D::new(0.0, 0.0, 800.0, 600.0);
    let queried = grid.query(&viewport);

    assert_eq!(queried.len(), 1);
    assert_eq!(*queried[0], "OnscreenItem");
}

#[test]
fn test_12_display_list_recording() {
    let mut dl = DisplayList::new();
    assert!(dl.is_empty());

    dl.push(DisplayCommand::PushClip(Rect2D::new(
        0.0, 0.0, 500.0, 500.0,
    )));
    dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
        Rect2D::new(10.0, 10.0, 100.0, 100.0),
        ColorRgba::BLACK,
    )));
    dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
        Rect2D::new(50.0, 50.0, 200.0, 200.0),
        ColorRgba::WHITE,
    )));
    dl.push(DisplayCommand::PopClip);

    assert_eq!(dl.len(), 4);
    assert_eq!(dl.quad_count, 2);
    assert_eq!(dl.glyph_count, 0);
    assert_eq!(dl.path_count, 0);
}

#[test]
fn test_13_word_compiler_single_page() {
    let page = WordDisplayPage {
        page_number: 1,
        bounds: Rect2D::new(100.0, 0.0, 816.0, 1056.0),
        lines: vec![
            WordDisplayLine {
                text: "Document Header".into(),
                x: 72.0,
                y: 72.0,
                font_size_px: 24.0,
                color: ColorRgba::BLACK,
            },
            WordDisplayLine {
                text: "First paragraph body text.".into(),
                x: 72.0,
                y: 120.0,
                font_size_px: 16.0,
                color: ColorRgba::BLACK,
            },
        ],
        table_cells: Vec::new(),
    };

    let viewport = Rect2D::new(0.0, 0.0, 1200.0, 1200.0);
    let dl = WordDisplayCompiler::compile_document(&[page], &viewport, 20.0);

    assert!(dl.quad_count >= 2); // shadow + page background
    assert_eq!(dl.glyph_count, 41); // 15 + 26
}

#[test]
fn test_14_word_compiler_virtualized_multi_page_culling() {
    let mut pages = Vec::new();
    for i in 1..=50 {
        pages.push(WordDisplayPage {
            page_number: i,
            bounds: Rect2D::new(100.0, 0.0, 816.0, 1056.0),
            lines: vec![WordDisplayLine {
                text: format!("Page {} content", i),
                x: 72.0,
                y: 72.0,
                font_size_px: 16.0,
                color: ColorRgba::BLACK,
            }],
            table_cells: Vec::new(),
        });
    }

    // Viewport covers only the first page and a half
    let viewport = Rect2D::new(0.0, 0.0, 1200.0, 1200.0);
    let dl = WordDisplayCompiler::compile_document(&pages, &viewport, 20.0);

    // Only pages 1 and 2 should be compiled into quads, the other 48 pages culled
    assert!(dl.quad_count <= 4);
}

#[test]
fn test_15_word_compiler_table_cells() {
    let mut cells = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            cells.push(WordDisplayTableCell {
                bounds: Rect2D::new(
                    72.0 + (c as f64) * 100.0,
                    200.0 + (r as f64) * 30.0,
                    100.0,
                    30.0,
                ),
                fill_color: Some(ColorRgba::WHITE),
                border_color: Some(ColorRgba::BLACK),
                border_width: 1.0,
                text: Some(format!("R{}C{}", r, c)),
            });
        }
    }

    let page = WordDisplayPage {
        page_number: 1,
        bounds: Rect2D::new(100.0, 0.0, 816.0, 1056.0),
        lines: Vec::new(),
        table_cells: cells,
    };

    let viewport = Rect2D::new(0.0, 0.0, 1200.0, 1200.0);
    let dl = WordDisplayCompiler::compile_document(&[page], &viewport, 20.0);

    assert_eq!(dl.quad_count, 11); // shadow + page + 9 table cells
}

#[test]
fn test_16_excel_compiler_gridlines() {
    let viewport = Rect2D::new(0.0, 0.0, 800.0, 400.0);
    let dl = ExcelDisplayCompiler::compile_grid(20.0, 80.0, &[], None, &viewport, 1000, 100);

    // Background quad + horizontal lines + vertical lines
    assert!(dl.quad_count >= 30);
}

#[test]
fn test_17_excel_compiler_binary_search_window() {
    let viewport = Rect2D::new(160.0, 100.0, 320.0, 200.0);
    let (r_min, r_max, c_min, c_max) =
        ExcelDisplayCompiler::find_visible_window(20.0, 80.0, &viewport, 1000, 100);

    assert_eq!(c_min, 2);
    assert_eq!(c_max, 6);
    assert_eq!(r_min, 5);
    assert_eq!(r_max, 15);
}

#[test]
fn test_18_excel_compiler_cell_fill_and_text() {
    let cell = ExcelDisplayCell {
        row: 5,
        col: 2,
        formatted_text: "$42,500.00".into(),
        fill_color: Some(ColorRgba::new(0.9, 0.95, 1.0, 1.0)),
        text_color: ColorRgba::BLACK,
        font_size_px: 13.0,
    };

    let viewport = Rect2D::new(100.0, 80.0, 400.0, 300.0);
    let dl = ExcelDisplayCompiler::compile_grid(20.0, 80.0, &[cell], None, &viewport, 100, 50);

    assert_eq!(dl.glyph_count, 10);
    assert!(dl.quad_count > 10);
}

#[test]
fn test_19_excel_compiler_selection_highlight() {
    let sel = ExcelSelectionRange {
        min_row: 2,
        min_col: 1,
        max_row: 4,
        max_col: 4,
    };

    let viewport = Rect2D::new(0.0, 0.0, 800.0, 600.0);
    let dl = ExcelDisplayCompiler::compile_grid(20.0, 80.0, &[], Some(&sel), &viewport, 100, 50);

    let sel_quad = dl.commands.iter().find(|cmd| {
        if let DisplayCommand::DrawQuad(q) = cmd {
            q.border_width == 2.0 && q.rect.width == 320.0 && q.rect.height == 60.0
        } else {
            false
        }
    });
    assert!(sel_quad.is_some(), "Selection quad must be emitted");
}

#[test]
fn test_20_ppt_compiler_slide_canvas_aspect_ratio() {
    let cfg = SlideCanvasConfig {
        canvas_width: 1920.0,
        canvas_height: 1080.0,
        background_color: ColorRgba::WHITE,
    };
    let viewport = Rect2D::new(0.0, 0.0, 2560.0, 1440.0);
    let (dest_rect, scale) = PptDisplayCompiler::compute_slide_viewport_transform(&cfg, &viewport);

    assert!((dest_rect.width - 2560.0).abs() < 1e-4);
    assert!((dest_rect.height - 1440.0).abs() < 1e-4);
    assert!((scale - (2560.0 / 1920.0)).abs() < 1e-4);
}

#[test]
fn test_21_ppt_compiler_shape_tree_z_order() {
    let slide = Slide::default();
    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let cfg = SlideCanvasConfig::default();

    let dl = PptDisplayCompiler::compile_slide(&slide, &[], None, &cfg, &viewport);
    assert!(dl.quad_count >= 3); // dark letterbox + drop shadow + slide canvas
}

#[test]
fn test_22_ppt_compiler_animated_morphed_state() {
    let slide = Slide::default();
    let morphed = MorphedSlideState {
        t: 0.5,
        shapes: vec![MorphedShapeState {
            shape_id: 1,
            name: "MorphedBox".into(),
            transform: MorphedTransform::new(200.0, 150.0, 400.0, 200.0, 0.0),
            opacity: 1.0,
            geometry_path: None,
            text_elements: vec![],
        }],
    };

    let viewport = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let cfg = SlideCanvasConfig::default();

    let dl = PptDisplayCompiler::compile_slide(&slide, &[], Some(&morphed), &cfg, &viewport);
    assert!(dl.quad_count >= 4);
}

#[test]
fn test_23_software_rasterizer_svg_exporter() {
    let mut dl = DisplayList::new();
    dl.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
        Rect2D::new(50.0, 50.0, 200.0, 100.0),
        ColorRgba::WHITE,
        ColorRgba::BLACK,
        1.5,
    )));
    dl.push(DisplayCommand::DrawText(ShapedTextEngine::shape_text(
        "RenderCore",
        "Calibri",
        18.0,
        ColorRgba::BLACK,
        (60.0, 70.0),
    )));

    let svg = SoftwareRasterizer::export_svg(&dl, 800.0, 600.0);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect"));
    assert!(svg.contains("<text"));
    assert!(svg.contains("RenderCore"));
    assert!(svg.ends_with("</svg>\n"));
}

#[test]
fn test_24_software_rasterizer_pixel_blending() {
    let mut dl = DisplayList::new();
    // White background
    dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
        Rect2D::new(0.0, 0.0, 100.0, 100.0),
        ColorRgba::WHITE,
    )));
    // Semi-transparent red overlay 50% alpha
    dl.push(DisplayCommand::DrawQuad(QuadInstance::solid(
        Rect2D::new(0.0, 0.0, 100.0, 100.0),
        ColorRgba::new(1.0, 0.0, 0.0, 0.5),
    )));

    let pixels = SoftwareRasterizer::render_to_pixels(&dl, 100, 100);
    assert_eq!(pixels.len(), 100 * 100 * 4);

    // Blended pixel: (255*0.5 + 255*1.0*0.5)/1.0 = 255 red, (0*0.5 + 255*0.5) = 127.5 green/blue
    let r = pixels[0];
    let g = pixels[1];
    let b = pixels[2];
    let a = pixels[3];

    assert_eq!(r, 255);
    assert!((g as i32 - 127).abs() <= 1);
    assert!((b as i32 - 127).abs() <= 1);
    assert_eq!(a, 255);
}

#[test]
fn test_25_120hz_compilation_sla_benchmark() {
    let mut compositor = Compositor::new(ViewportConfig {
        rect: Rect2D::new(0.0, 0.0, 3840.0, 2160.0), // 4K Viewport
        zoom: 1.0,
        device_pixel_ratio: 2.0,
    });

    let (dl, stats) = compositor.profile_compile(|vp| {
        ExcelDisplayCompiler::compile_grid(20.0, 80.0, &[], None, vp, 10000, 500)
    });

    assert!(!dl.is_empty());
    assert!(
        stats.compile_time_ms < 2.0,
        "4K compilation latency {} ms must be within SLA (< 2.0 ms)",
        stats.compile_time_ms
    );
    assert!(stats.within_120fps_budget);
}

#[test]
fn test_26_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/render/5.1.1_vectors.json");
    assert!(path.exists(), "Vectors manifest must exist");

    let content = std::fs::read_to_string(path).expect("Read vectors manifest");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(v["subphase"], "5.1.1");
    assert_eq!(v["vector_count"], 26);
    let vectors = v["vectors"].as_array().expect("Vectors array");
    assert_eq!(vectors.len(), 26);

    for (i, vec_item) in vectors.iter().enumerate() {
        let expected_id = format!("VEC-5.1.1-{:02}", i + 1);
        assert_eq!(vec_item["id"], expected_id.as_str());
        assert!(!vec_item["description"].as_str().unwrap().is_empty());
    }
}
