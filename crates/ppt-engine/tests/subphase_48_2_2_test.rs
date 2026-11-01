use ppt_engine::deck_export::{DeckPdfExporter, SlideSvgExporter};
use ppt_engine::model::{
    BodyProperties, ColorChoice, Fill, Geometry, LineProperties, Shape, ShapeElement,
    ShapeProperties, Slide, TextBody, TextParagraph, TextRunElement, Transform2D,
};
use ppt_engine::odp::OdpCodec;
use ppt_engine::package::PresentationPackage;

fn create_test_shape(id: u32, name: &str, x: i64, y: i64, cx: u32, cy: u32) -> Shape {
    Shape {
        id,
        name: name.to_string(),
        is_text_box: false,
        placeholder: None,
        properties: ShapeProperties {
            transform: Some(Transform2D::new(x, y, cx, cy)),
            geometry: Some(Geometry::Preset("rect".to_string())),
            fill: Some(Fill::Solid(ColorChoice::Srgb("4472C4".to_string()))),
            line: Some(LineProperties {
                width: Some(12700),
                fill: Some(Fill::Solid(ColorChoice::Srgb("000000".to_string()))),
                dash_style: None,
            }),
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
    s.is_text_box = true;
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
fn test_1_odp_encode_produces_valid_odf_archive() {
    let mut pkg = PresentationPackage::new();
    let shape = create_test_text_shape(
        101,
        "HeadingBox",
        500_000,
        500_000,
        4_000_000,
        1_000_000,
        "Quarterly Earnings",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(shape)));

    let odp_bytes = OdpCodec::encode(&pkg).expect("ODP encode should succeed");
    assert!(!odp_bytes.is_empty(), "ODP byte array must not be empty");

    // Must be a valid zip archive with the standard ODF layout
    let cursor = std::io::Cursor::new(&odp_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("ODP must be a valid zip container");

    // 1. mimetype MUST be the first file entry and uncompressed
    let (first_name, first_comp) = {
        let first_file = archive
            .by_index(0)
            .expect("Archive must contain at least one file");
        (first_file.name().to_string(), first_file.compression())
    };
    assert_eq!(first_name, "mimetype");
    assert_eq!(first_comp, zip::CompressionMethod::Stored);

    // 2. META-INF/manifest.xml must exist
    assert!(archive.by_name("META-INF/manifest.xml").is_ok());

    // 3. content.xml and styles.xml must exist
    assert!(archive.by_name("content.xml").is_ok());
    assert!(archive.by_name("styles.xml").is_ok());
}

#[test]
fn test_2_odp_decode_manifest_and_pages() {
    let mut pkg = PresentationPackage::new();
    let s1 = create_test_text_shape(
        10,
        "Title",
        100_000,
        100_000,
        2_000_000,
        500_000,
        "Slide 1 Content",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    let odp_bytes = OdpCodec::encode(&pkg).expect("Encode ODP");
    let decoded = OdpCodec::decode(&odp_bytes).expect("Decode ODP");

    assert_eq!(
        decoded.slides.len(),
        1,
        "Decoded package should have 1 slide"
    );
    let slide = &decoded.slides[0].1;
    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 1);

    if let ShapeElement::Shape(s) = &slide.common_slide_data.shape_tree.shapes[0] {
        assert_eq!(s.name, "Title");
        let body = s.text_body.as_ref().expect("Should have text body");
        assert_eq!(body.paragraphs.len(), 1);
        if let TextRunElement::Text(txt, _) = &body.paragraphs[0].runs[0] {
            assert_eq!(txt, "Slide 1 Content");
        } else {
            panic!("Expected text run");
        }
    } else {
        panic!("Expected Shape");
    }
}

#[test]
fn test_3_odp_dimension_parsing_units() {
    let odf_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
  xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body>
    <office:presentation>
      <draw:page draw:name="Slide1">
        <draw:custom-shape draw:name="ShapePt" svg:x="10pt" svg:y="20pt" svg:width="100pt" svg:height="50pt">
          <text:p>Pt Units</text:p>
        </draw:custom-shape>
        <draw:custom-shape draw:name="ShapeCm" svg:x="1cm" svg:y="2cm" svg:width="10cm" svg:height="5cm">
          <text:p>Cm Units</text:p>
        </draw:custom-shape>
        <draw:custom-shape draw:name="ShapeIn" svg:x="1in" svg:y="2in" svg:width="4in" svg:height="3in">
          <text:p>In Units</text:p>
        </draw:custom-shape>
      </draw:page>
    </office:presentation>
  </office:body>
</office:document-content>"#;

    // Pack into ODP zip
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file(
            "mimetype",
            opts.compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
        std::io::Write::write_all(&mut zip, b"application/vnd.oasis.opendocument.presentation")
            .unwrap();
        zip.start_file("content.xml", opts).unwrap();
        std::io::Write::write_all(&mut zip, odf_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let decoded = OdpCodec::decode(&buffer.into_inner()).expect("Decode dimension test");
    assert_eq!(decoded.slides.len(), 1);
    let shapes = &decoded.slides[0].1.common_slide_data.shape_tree.shapes;
    assert_eq!(shapes.len(), 3);

    // 10pt = 10 * 12700 = 127000 EMUs
    if let ShapeElement::Shape(s) = &shapes[0] {
        assert_eq!(s.name, "ShapePt");
        let t = s.properties.transform.as_ref().unwrap();
        assert_eq!(t.x, 127_000);
        assert_eq!(t.y, 254_000);
        assert_eq!(t.cx, 1_270_000);
        assert_eq!(t.cy, 635_000);
    } else {
        panic!("Expected ShapePt");
    }

    // 1cm = 360000 EMUs
    if let ShapeElement::Shape(s) = &shapes[1] {
        assert_eq!(s.name, "ShapeCm");
        let t = s.properties.transform.as_ref().unwrap();
        assert_eq!(t.x, 360_000);
        assert_eq!(t.y, 720_000);
        assert_eq!(t.cx, 3_600_000);
        assert_eq!(t.cy, 1_800_000);
    } else {
        panic!("Expected ShapeCm");
    }

    // 1in = 914400 EMUs
    if let ShapeElement::Shape(s) = &shapes[2] {
        assert_eq!(s.name, "ShapeIn");
        let t = s.properties.transform.as_ref().unwrap();
        assert_eq!(t.x, 914_400);
        assert_eq!(t.y, 1_828_800);
        assert_eq!(t.cx, 3_657_600);
        assert_eq!(t.cy, 2_743_200);
    } else {
        panic!("Expected ShapeIn");
    }
}

#[test]
fn test_4_odp_multiple_pages_and_text_boxes() {
    let mut pkg = PresentationPackage::new();
    // Slide 1
    let s1 = create_test_text_shape(1, "S1_Shape", 0, 0, 1_000_000, 1_000_000, "First Slide");
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    // Slide 2
    let mut slide2 = Slide::default();
    let s2 = create_test_text_shape(2, "S2_Shape", 0, 0, 1_000_000, 1_000_000, "Second Slide");
    slide2
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s2)));
    pkg.slides
        .push(("ppt/slides/slide2.xml".to_string(), slide2));

    let odp_bytes = OdpCodec::encode(&pkg).expect("Encode multi-page ODP");
    let decoded = OdpCodec::decode(&odp_bytes).expect("Decode multi-page ODP");

    assert_eq!(
        decoded.slides.len(),
        2,
        "Decoded package should have 2 slides"
    );
}

#[test]
fn test_5_svg_export_structure_and_dimensions() {
    let mut pkg = PresentationPackage::new();
    pkg.presentation.slide_size.cx = 12_192_000; // 960 pt
    pkg.presentation.slide_size.cy = 6_858_000; // 540 pt

    let shape = create_test_shape(10, "Card", 1_270_000, 1_270_000, 2_540_000, 1_270_000);
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(shape)));

    let svg = SlideSvgExporter::export_slide_svg(&pkg, 0).expect("SVG export");
    assert!(svg.starts_with("<svg "), "Must start with <svg tag");
    assert!(
        svg.contains(r#"viewBox="0 0 960.00 540.00""#),
        "Must contain correct viewBox"
    );
    assert!(
        svg.contains(r#"xmlns="http://www.w3.org/2000/svg""#),
        "Must declare SVG namespace"
    );
    assert!(svg.ends_with("</svg>"), "Must close with </svg>");
}

#[test]
fn test_6_svg_vector_rendering_rect_and_text() {
    let mut pkg = PresentationPackage::new();
    let text_shape = create_test_text_shape(
        25,
        "WelcomeBanner",
        1_270_000, // 100 pt
        2_540_000, // 200 pt
        3_810_000, // 300 pt
        1_270_000, // 100 pt
        "Antigravity Presentation",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(text_shape)));

    let svg = SlideSvgExporter::export_slide_svg(&pkg, 0).expect("SVG export");
    // Verify rect tag with x="100.00" y="200.00" width="300.00" height="100.00"
    assert!(svg.contains(r#"x="100.00""#));
    assert!(svg.contains(r#"y="200.00""#));
    assert!(svg.contains(r#"width="300.00""#));
    assert!(svg.contains(r#"height="100.00""#));
    assert!(
        svg.contains("Antigravity Presentation"),
        "SVG must include text run"
    );
}

#[test]
fn test_7_deck_pdf_export_iso32000_structure() {
    let mut pkg = PresentationPackage::new();
    let s1 = create_test_text_shape(
        1,
        "SlideTitle",
        500_000,
        500_000,
        3_000_000,
        800_000,
        "Executive Summary",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    let pdf = DeckPdfExporter::export_deck_pdf(&pkg).expect("PDF deck export");
    assert!(
        pdf.starts_with(b"%PDF-1.7\n"),
        "Must start with %PDF-1.7 header"
    );
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_str.contains("/Type /Catalog"),
        "Must contain Catalog dictionary"
    );
    assert!(pdf_str.contains("/Type /Pages"), "Must contain Pages root");
    assert!(
        pdf_str.contains("/Type /Page"),
        "Must contain Page dictionary"
    );
    assert!(
        pdf_str.contains("/Type /Font"),
        "Must define Font dictionary"
    );
    assert!(
        pdf_str.contains("xref\n"),
        "Must contain cross-reference table"
    );
    assert!(
        pdf_str.contains("trailer\n"),
        "Must contain trailer dictionary"
    );
    assert!(pdf_str.contains("%%EOF"), "Must terminate with %%EOF");
}

#[test]
fn test_8_deck_pdf_vector_stream_instructions() {
    let mut pkg = PresentationPackage::new();
    let s = create_test_text_shape(
        5,
        "KeyMetrics",
        1_270_000, // 100 pt
        1_270_000, // 100 pt
        2_540_000, // 200 pt
        1_270_000, // 100 pt
        "Revenue: +42%",
    );
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s)));

    let pdf = DeckPdfExporter::export_deck_pdf(&pkg).expect("PDF export");
    let pdf_str = String::from_utf8_lossy(&pdf);

    // Verify vector path drawing instructions:
    // 're' for rectangle, 'B' or 'f' or 's' for fill/stroke
    assert!(
        pdf_str.contains("re\n"),
        "Must emit PDF rectangle operator 're'"
    );
    // Verify text block instructions:
    // 'BT' (Begin Text), '/F1', 'Tf', 'Td', 'Tj', 'ET' (End Text)
    assert!(
        pdf_str.contains("BT\n"),
        "Must emit PDF Begin Text operator 'BT'"
    );
    assert!(
        pdf_str.contains("ET\n"),
        "Must emit PDF End Text operator 'ET'"
    );
    assert!(
        pdf_str.contains("(Revenue: +42%) Tj\n"),
        "Must emit text string instruction"
    );
}

#[test]
fn test_9_roundtrip_odp_to_pptx_to_odp() {
    let mut initial_pkg = PresentationPackage::new();
    let shape = create_test_text_shape(
        42,
        "RoundTripBox",
        1_000_000,
        2_000_000,
        3_000_000,
        1_500_000,
        "Seamless Interchange",
    );
    initial_pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(shape)));

    // 1. Encode to ODP
    let odp1 = OdpCodec::encode(&initial_pkg).expect("Encode initial ODP");

    // 2. Decode to PresentationPackage (simulating PPTX import)
    let decoded_pkg = OdpCodec::decode(&odp1).expect("Decode ODP to pkg");
    assert_eq!(decoded_pkg.slides.len(), 1);

    // 3. Re-encode to ODP
    let odp2 = OdpCodec::encode(&decoded_pkg).expect("Re-encode to ODP");

    // 4. Re-decode to ensure data integrity
    let final_pkg = OdpCodec::decode(&odp2).expect("Decode re-encoded ODP");
    assert_eq!(final_pkg.slides.len(), 1);
    let shapes = &final_pkg.slides[0].1.common_slide_data.shape_tree.shapes;
    assert_eq!(shapes.len(), 1);
    if let ShapeElement::Shape(s) = &shapes[0] {
        assert_eq!(s.name, "RoundTripBox");
        let body = s.text_body.as_ref().unwrap();
        if let TextRunElement::Text(txt, _) = &body.paragraphs[0].runs[0] {
            assert_eq!(txt, "Seamless Interchange");
        } else {
            panic!("Expected text run");
        }
    } else {
        panic!("Expected Shape");
    }
}

#[test]
fn test_10_multi_slide_deck_pdf_page_count_and_sizing() {
    let mut pkg = PresentationPackage::new();
    // 3 Slides with 16:9 widescreen dimensions (13.33 x 7.5 inches = 12192000 x 6858000 EMUs)
    pkg.presentation.slide_size.cx = 12_192_000;
    pkg.presentation.slide_size.cy = 6_858_000;

    let s1 = create_test_text_shape(1, "S1", 0, 0, 1_000_000, 500_000, "Deck Intro");
    pkg.slides[0]
        .1
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(s1)));

    for i in 2..=3 {
        let mut slide = Slide::default();
        let s = create_test_text_shape(
            i as u32,
            &format!("S{i}"),
            0,
            0,
            1_000_000,
            500_000,
            &format!("Content for Slide {i}"),
        );
        slide
            .common_slide_data
            .shape_tree
            .shapes
            .push(ShapeElement::Shape(Box::new(s)));
        pkg.slides.push((format!("ppt/slides/slide{i}.xml"), slide));
    }

    let pdf = DeckPdfExporter::export_deck_pdf(&pkg).expect("Multi-slide PDF");
    let pdf_str = String::from_utf8_lossy(&pdf);

    // Verify /Count 3 in Pages catalog
    assert!(pdf_str.contains("/Count 3"));
    // Verify MediaBox [0 0 960.00 540.00]
    assert!(pdf_str.contains("/MediaBox [0 0 960.00 540.00]"));
    // Verify all 3 slide titles are rendered in their respective content streams
    assert!(pdf_str.contains("(Deck Intro) Tj"));
    assert!(pdf_str.contains("(Content for Slide 2) Tj"));
    assert!(pdf_str.contains("(Content for Slide 3) Tj"));
}
