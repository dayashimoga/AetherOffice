//! Sub-Phase 49.1.1 Integration Tests:
//! ISO 32000-2 Vector Path, Image XObject & Font CMap Canvas Renderer
//!
//! Validates:
//! - 2D Affine Matrix composition, multiplication, and point transformation
//! - Vector path parsing and SVG serialization (MoveTo, LineTo, CurveTo, Rectangle, ClosePath)
//! - Line styling (line width, cap, join, dash patterns)
//! - Color space translation (DeviceRGB, DeviceGray, DeviceCMYK to sRGB hex)
//! - Text element extraction and positioning with font CMap resolution
//! - Graphics state stack (q / Q push & pop)
//! - Image XObject detection, extraction, and geometry placement
//! - Standalone SVG markup generation with valid viewBox and elements
//! - High-DPI canvas scaling factor preservation

use pdf_engine::renderer::{AffineMatrix, PdfPageRenderer};
use pdf_engine::PdfDocument;

fn create_test_pdf(content: &[u8], extra_objects: &[(&str, &[u8])], resources: &str) -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let mut offsets = Vec::new();
    offsets.push(doc_bytes.len());
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    offsets.push(doc_bytes.len());
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    // Page object (ID 3)
    let page_obj_idx = offsets.len();
    offsets.push(0); // placeholder

    // Content stream (ID 4)
    offsets.push(doc_bytes.len());
    doc_bytes.extend_from_slice(
        format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len()).as_bytes(),
    );
    doc_bytes.extend_from_slice(content);
    doc_bytes.extend_from_slice(b"\nendstream\nendobj\n");

    let mut next_id = 5usize;
    let mut res_str = resources.to_string();
    if res_str.is_empty() {
        res_str = "/Font << /F1 5 0 R >>".to_string();
    }

    for (_name, obj_data) in extra_objects {
        offsets.push(doc_bytes.len());
        doc_bytes.extend_from_slice(format!("{next_id} 0 obj\n").as_bytes());
        doc_bytes.extend_from_slice(obj_data);
        doc_bytes.extend_from_slice(b"\nendobj\n");
        next_id += 1;
    }

    // Default font if no extra objects supplied
    if extra_objects.is_empty() {
        offsets.push(doc_bytes.len());
        doc_bytes.extend_from_slice(
            b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
        );
        next_id += 1;
    }

    // Write page object at page_obj_idx
    let page_offset = doc_bytes.len();
    offsets[page_obj_idx] = page_offset;
    doc_bytes.extend_from_slice(
        format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << {} >> >>\nendobj\n",
            res_str
        )
        .as_bytes(),
    );

    let xref_offset = doc_bytes.len();
    let num_objects = next_id;
    let mut xref_str = format!("xref\n0 {num_objects}\n0000000000 65535 f \r\n");
    for off in &offsets {
        xref_str.push_str(&format!("{:010} 00000 n \r\n", off));
    }
    xref_str.push_str(&format!(
        "trailer\n<< /Size {num_objects} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF"
    ));
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    doc_bytes
}

#[test]
fn test_01_affine_matrix_multiplication_and_point_transform() {
    let identity = AffineMatrix::default();
    let (x, y) = identity.transform_point(10.0, 20.0);
    assert_eq!((x, y), (10.0, 20.0));

    // Translation by (50, 100)
    let t = AffineMatrix::new(1.0, 0.0, 0.0, 1.0, 50.0, 100.0);
    let (tx, ty) = t.transform_point(10.0, 20.0);
    assert_eq!((tx, ty), (60.0, 120.0));

    // Scale by 2.0x
    let s = AffineMatrix::new(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    let (sx, sy) = s.transform_point(10.0, 20.0);
    assert_eq!((sx, sy), (20.0, 40.0));

    // Combined: S * T
    let combined = s.multiply(&t);
    let (cx, cy) = combined.transform_point(10.0, 20.0);
    // (10 * 1 + 50) * 2 = 120
    // (20 * 1 + 100) * 2 = 240
    assert_eq!((cx, cy), (120.0, 240.0));
}

#[test]
fn test_02_vector_path_rectangle_and_fill_rendering() {
    // PDF drawing a rectangle with blue fill: 0.27 0.45 0.77 rg 100 200 300 150 re f
    let content = b"0.27 0.45 0.77 rg 100 200 300 150 re f";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, Some(1.0)).expect("Canvas rendering");

    assert_eq!(canvas.page_index, 0);
    assert_eq!(canvas.width_pt, 612.0);
    assert_eq!(canvas.height_pt, 792.0);
    assert_eq!(canvas.vector_paths.len(), 1);

    let path = &canvas.vector_paths[0];
    assert!(path.d.contains("M "), "Must contain MoveTo");
    assert!(path.d.contains("Z"), "Must close path");
    assert!(path.fill_color.is_some());
    let fill = path.fill_color.as_ref().unwrap();
    assert_eq!(fill, "#4573C4"); // 0.27*255=69 (45), 0.45*255=115 (73), 0.77*255=196 (C4)
    assert_eq!(path.fill_rule, "nonzero");
}

#[test]
fn test_03_vector_path_stroking_line_properties() {
    // Stroked red line from (50, 100) to (250, 100) with width 3, round cap (1 J), round join (1 j), dash [5 3]
    let content = b"1 0 0 RG 3 w 1 J 1 j [5 3] 0 d 50 100 m 250 100 l S";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, None).expect("Canvas rendering");

    assert_eq!(canvas.vector_paths.len(), 1);
    let path = &canvas.vector_paths[0];
    assert_eq!(path.stroke_color.as_deref(), Some("#FF0000"));
    assert_eq!(path.stroke_width, 3.0);
    assert_eq!(path.line_cap, "round");
    assert_eq!(path.line_join, "round");
    assert_eq!(path.dash_array.as_deref(), Some("5.00,3.00"));
}

#[test]
fn test_04_vector_path_cubic_bezier_curves() {
    // Cubic Bézier curve
    let content = b"0 0 m 50 100 150 100 200 0 c S";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, None).expect("Canvas rendering");

    assert_eq!(canvas.vector_paths.len(), 1);
    let path = &canvas.vector_paths[0];
    assert!(
        path.d.contains("C "),
        "SVG path must contain cubic Bézier 'C' command"
    );
}

#[test]
fn test_05_color_spaces_rgb_gray_cmyk_conversion() {
    // DeviceGray, DeviceRGB, and DeviceCMYK fills
    let content = b"0.5 g 10 10 50 50 re f 1 0.5 0 rg 70 10 50 50 re f 0 1 1 0 k 130 10 50 50 re f";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, None).expect("Canvas rendering");

    assert_eq!(canvas.vector_paths.len(), 3);
    // 0.5 gray = #808080
    assert_eq!(
        canvas.vector_paths[0].fill_color.as_deref(),
        Some("#808080")
    );
    // 1.0, 0.5, 0.0 RGB = #FF8000
    assert_eq!(
        canvas.vector_paths[1].fill_color.as_deref(),
        Some("#FF8000")
    );
    // C=0, M=1, Y=1, K=0 CMYK -> R=(1-0)(1-0)=1, G=(1-1)=0, B=(1-1)=0 -> #FF0000
    assert_eq!(
        canvas.vector_paths[2].fill_color.as_deref(),
        Some("#FF0000")
    );
}

#[test]
fn test_06_text_element_extraction_with_font_cmap() {
    let content = b"BT /F1 24 Tf 100 700 Td (AetherPDF Engine) Tj ET";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, None).expect("Canvas rendering");

    assert_eq!(canvas.text_elements.len(), 1);
    let text = &canvas.text_elements[0];
    assert_eq!(text.text, "AetherPDF Engine");
    assert_eq!(text.font_size, 24.0);
    assert_eq!(text.font_name, "F1");
}

#[test]
fn test_07_graphics_state_stack_push_pop() {
    // q (save) -> 2 0 0 2 0 0 cm -> draw rect -> Q (restore) -> draw rect at original scale
    let content = b"q 2 0 0 2 0 0 cm 10 10 20 20 re f Q 10 10 20 20 re f";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas = PdfPageRenderer::render_page_canvas(&doc, 0, None).expect("Canvas rendering");

    assert_eq!(canvas.vector_paths.len(), 2);
    // The first rect was scaled by 2.0x, so its width is 40 in page space
    // The second rect was rendered after Q (restore), so its width is 20
    assert_ne!(canvas.vector_paths[0].d, canvas.vector_paths[1].d);
}

#[test]
fn test_08_image_xobject_extraction_and_geometry() {
    // Create an Image XObject (ID 6)
    let img_stream = b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>\nstream\n\xFF\x00\x00\x00\xFF\x00\x00\x00\xFF\xFF\xFF\xFF\nendstream";
    let extra_objs = [
        (
            "font",
            &b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"[..],
        ),
        ("img", img_stream),
    ];
    let res_str = "/Font << /F1 5 0 R >> /XObject << /Im1 6 0 R >>";
    // Place image at (100, 200) with size 150x100: 150 0 0 100 100 200 cm /Im1 Do
    let content = b"q 150 0 0 100 100 200 cm /Im1 Do Q";
    let pdf_bytes = create_test_pdf(content, &extra_objs, res_str);

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let images = PdfPageRenderer::extract_page_images(&doc, 0).expect("Extract images");

    assert_eq!(images.len(), 1);
    let img = &images[0];
    assert_eq!(img.name, "Im1");
    assert_eq!(img.width, 150.0);
    assert_eq!(img.height, 100.0);
    assert!(!img.data_base64.is_empty(), "Must produce base64 payload");
}

#[test]
fn test_09_standalone_svg_markup_generation() {
    let content = b"0 0.5 1 rg 50 50 200 100 re f BT /F1 16 Tf 60 700 Td (Vector Document) Tj ET";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let svg = PdfPageRenderer::render_page_svg(&doc, 0).expect("Render SVG");

    assert!(svg.starts_with("<svg "), "Must begin with <svg");
    assert!(
        svg.contains(r#"viewBox="0 0 612.00 792.00""#),
        "Must contain valid viewBox"
    );
    assert!(svg.contains(r#"<path d=""#), "Must contain path tag");
    assert!(svg.contains(r#"<text "#), "Must contain text tag");
    assert!(svg.contains("Vector Document"), "Must contain text payload");
    assert!(svg.ends_with("</svg>"), "Must terminate with </svg>");
}

#[test]
fn test_10_high_dpi_canvas_scale_factor_preservation() {
    let content = b"10 10 100 100 re f";
    let pdf_bytes = create_test_pdf(content, &[], "");

    let doc = PdfDocument::parse(pdf_bytes).expect("Valid PDF");
    let canvas_1x = PdfPageRenderer::render_page_canvas(&doc, 0, Some(1.0)).expect("1x");
    let canvas_2x = PdfPageRenderer::render_page_canvas(&doc, 0, Some(2.0)).expect("2x");

    assert_eq!(canvas_1x.scale, 1.0);
    assert_eq!(canvas_2x.scale, 2.0);
    assert_eq!(canvas_1x.width_pt, canvas_2x.width_pt);
    assert_eq!(canvas_1x.height_pt, canvas_2x.height_pt);
}
