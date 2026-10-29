//! Subphase 5.2.2 Integration Test Suite:
//! Interactive AcroForms Engine, Annotations & Ink Path Layers
//!
//! Validates:
//! - Freehand stylus ink stroke bounding box computation
//! - Vector ink path `/AP /N` appearance stream compiler (w, RG, m, l, S)
//! - Ink annotation dictionary serialization (`/Subtype /Ink`, `/InkList`, `/BS`)
//! - Generic annotation parsing (Sticky Note, Link, Highlight, Shape)
//! - Annotation discovery across document pages
//! - AcroForm text fields (`/FT /Tx`) parsing and text mutation
//! - AcroForm checkboxes (`/FT /Btn`) toggling and appearance regeneration
//! - AcroForm choice dropdowns (`/FT /Ch`) with `/Opt` arrays
//! - Hierarchical form field inheritance and dot-separated full naming
//! - Persistence of form modifications and newly drawn ink via `PdfIncrementalWriter`
//! - Sub-millisecond performance SLA benchmark (< 1.0 ms)

use pdf_engine::acroform::{AcroFormEngine, FormFieldType, FormFieldValue};
use pdf_engine::annotation::{AnnotationEngine, AnnotationSubtype, PdfAnnotation};
use pdf_engine::ink::{InkAnnotation, InkStroke};
use pdf_engine::writer::PdfIncrementalWriter;
use pdf_engine::{PdfDocument, PdfObject, PdfObjectId, PdfRect};
use std::collections::BTreeMap;
use std::time::Instant;

fn build_baseline_acroform_pdf() -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [ 5 0 R 6 0 R 7 0 R ] >> >>\nendobj\n",
    );

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Annots [ 5 0 R 6 0 R 7 0 R ] >>\nendobj\n",
    );

    // Text field (Obj 5 0): Full_Name = "John Doe"
    let off_f1 = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"5 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Tx /T (Full_Name) /V (John Doe) /Rect [ 100 700 250 730 ] /DA (/Helv 12 Tf 0 g) >>\nendobj\n",
    );

    // Checkbox field (Obj 6 0): Agreed = Yes
    let off_f2 = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"6 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Btn /T (Agreed) /V /Yes /Rect [ 100 650 120 670 ] >>\nendobj\n",
    );

    // Choice dropdown field (Obj 7 0): Country = "USA" with options ["Canada", "USA", "UK"]
    let off_f3 = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"7 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Ch /Ff 131072 /T (Country) /V (USA) /Opt [ (Canada) (USA) (UK) ] /Rect [ 100 600 250 630 ] >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 8\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 8 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_f1, off_f2, off_f3, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    doc_bytes
}

#[test]
fn test_01_ink_stroke_bounding_box_computation() {
    let stroke1 = InkStroke::new(vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)]);
    let stroke2 = InkStroke::new(vec![(5.0, 15.0), (60.0, 80.0)]);

    let bbox = InkAnnotation::calculate_bounding_rect(&[stroke1, stroke2], 2.0);

    // Padding = 2.0 -> min_x=5-2=3, min_y=15-2=13, max_x=60+2=62, max_y=80+2=82
    assert_eq!(bbox.llx, 3.0);
    assert_eq!(bbox.lly, 13.0);
    assert_eq!(bbox.urx, 62.0);
    assert_eq!(bbox.ury, 82.0);
    assert_eq!(bbox.width(), 59.0);
    assert_eq!(bbox.height(), 69.0);
}

#[test]
fn test_02_ink_annotation_appearance_stream_generation() {
    let stroke = InkStroke::new(vec![(100.0, 200.0), (120.0, 220.0), (150.0, 210.0)]);
    let ink = InkAnnotation::new(vec![stroke], [0.0, 0.4, 0.8], 3.0);

    let stream = ink.compile_appearance_stream();
    let decoded = stream.decode().unwrap();
    let script = String::from_utf8(decoded).unwrap();

    // Verify drawing commands: width, stroke color, start moveto, lineto, and stroke operator
    assert!(script.contains("3.00 w"));
    assert!(script.contains("0.000 0.400 0.800 RG"));
    assert!(script.contains("m\n"));
    assert!(script.contains("l\n"));
    assert!(script.contains("S\n"));

    let dict = &stream.dict;
    assert_eq!(
        dict.get("Type").unwrap(),
        &PdfObject::Name("XObject".into())
    );
    assert_eq!(
        dict.get("Subtype").unwrap(),
        &PdfObject::Name("Form".into())
    );
}

#[test]
fn test_03_ink_annotation_to_dict_serialization() {
    let stroke = InkStroke::new(vec![(50.0, 50.0), (100.0, 100.0)]);
    let ink = InkAnnotation::new(vec![stroke], [1.0, 0.0, 0.0], 2.0);

    let dict = ink.to_dict_with_ap(PdfObjectId::new(10, 0));
    assert_eq!(dict.get("Type").unwrap(), &PdfObject::Name("Annot".into()));
    assert_eq!(dict.get("Subtype").unwrap(), &PdfObject::Name("Ink".into()));

    let ink_list = dict.get("InkList").unwrap().as_array().unwrap();
    assert_eq!(ink_list.len(), 1);
    let pts = ink_list[0].as_array().unwrap();
    assert_eq!(pts.len(), 4); // x1, y1, x2, y2

    assert!(dict.contains_key("AP"));
    assert!(dict.contains_key("BS"));
}

#[test]
fn test_04_annotation_parsing_sticky_note() {
    let mut dict = BTreeMap::new();
    dict.insert("Type".into(), PdfObject::Name("Annot".into()));
    dict.insert("Subtype".into(), PdfObject::Name("Text".into()));
    dict.insert(
        "Rect".into(),
        PdfRect::new(50.0, 50.0, 80.0, 80.0).to_array(),
    );
    dict.insert(
        "Contents".into(),
        PdfObject::LiteralString(b"Check this paragraph".to_vec()),
    );
    dict.insert("T".into(), PdfObject::LiteralString(b"Reviewer".to_vec()));
    dict.insert(
        "C".into(),
        PdfObject::Array(vec![
            PdfObject::Real(1.0),
            PdfObject::Real(1.0),
            PdfObject::Real(0.0),
        ]),
    );

    let annot = PdfAnnotation::from_dict(PdfObjectId::new(10, 0), 0, &dict).unwrap();
    assert_eq!(annot.subtype, AnnotationSubtype::Text);
    assert_eq!(annot.contents, Some("Check this paragraph".into()));
    assert_eq!(annot.author, Some("Reviewer".into()));
    assert_eq!(annot.color, Some([1.0, 1.0, 0.0]));
}

#[test]
fn test_05_annotation_parsing_highlight_and_quadpoints() {
    let mut dict = BTreeMap::new();
    dict.insert("Type".into(), PdfObject::Name("Annot".into()));
    dict.insert("Subtype".into(), PdfObject::Name("Highlight".into()));
    dict.insert(
        "Rect".into(),
        PdfRect::new(72.0, 700.0, 200.0, 715.0).to_array(),
    );

    let qp = [72.0, 715.0, 200.0, 715.0, 72.0, 700.0, 200.0, 700.0];
    let qp_arr = qp.iter().copied().map(PdfObject::Real).collect();
    dict.insert("QuadPoints".into(), PdfObject::Array(qp_arr));

    let annot = PdfAnnotation::from_dict(PdfObjectId::new(12, 0), 1, &dict).unwrap();
    if let AnnotationSubtype::Highlight { quad_points } = annot.subtype {
        assert_eq!(quad_points.len(), 8);
        assert_eq!(quad_points[0], 72.0);
    } else {
        panic!("Expected Highlight subtype");
    }
}

#[test]
fn test_06_annotation_parsing_links_and_shapes() {
    // 1. Link annotation
    let mut link_dict = BTreeMap::new();
    link_dict.insert("Subtype".into(), PdfObject::Name("Link".into()));
    let mut a_dict = BTreeMap::new();
    a_dict.insert(
        "URI".into(),
        PdfObject::LiteralString(b"https://openoffice.org".to_vec()),
    );
    link_dict.insert("A".into(), PdfObject::Dictionary(a_dict));

    let link_annot = PdfAnnotation::from_dict(PdfObjectId::new(14, 0), 0, &link_dict).unwrap();
    assert_eq!(
        link_annot.subtype,
        AnnotationSubtype::Link {
            uri: Some("https://openoffice.org".into())
        }
    );

    // 2. Square annotation
    let mut sq_dict = BTreeMap::new();
    sq_dict.insert("Subtype".into(), PdfObject::Name("Square".into()));
    let sq_annot = PdfAnnotation::from_dict(PdfObjectId::new(15, 0), 0, &sq_dict).unwrap();
    assert_eq!(sq_annot.subtype, AnnotationSubtype::Square);
}

#[test]
fn test_07_annotation_engine_extract_from_page() {
    let pdf_bytes = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(pdf_bytes).unwrap();

    let annotations = AnnotationEngine::extract_annotations(&doc).unwrap();
    // 3 form widget annotations defined in baseline PDF
    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations[0].subtype, AnnotationSubtype::Widget);
}

#[test]
fn test_08_acroform_text_field_parsing() {
    let pdf_bytes = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(pdf_bytes).unwrap();

    let engine = AcroFormEngine::parse(&doc).unwrap();
    assert_eq!(engine.fields().len(), 3);

    let name_field = engine
        .get_field("Full_Name")
        .expect("Field Full_Name must exist");
    assert_eq!(name_field.full_name, "Full_Name");
    assert_eq!(name_field.value, Some("John Doe".into()));
    assert_eq!(
        name_field.field_type,
        FormFieldType::Text {
            multiline: false,
            password: false
        }
    );
    assert_eq!(name_field.rect.width(), 150.0);
}

#[test]
fn test_09_acroform_checkbox_parsing_and_toggling() {
    let pdf_bytes = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(pdf_bytes).unwrap();

    let mut engine = AcroFormEngine::parse(&doc).unwrap();
    let agree_field = engine.get_field("Agreed").expect("Field Agreed must exist");
    assert_eq!(agree_field.value, Some("Yes".into()));

    // Toggle checkbox to false (Off)
    engine
        .set_field_value("Agreed", FormFieldValue::Boolean(false))
        .unwrap();

    let updated = engine.get_field("Agreed").unwrap();
    assert_eq!(updated.value, Some("Off".into()));
    assert!(updated.appearance_stream.is_some());
}

#[test]
fn test_10_acroform_choice_dropdown_options() {
    let pdf_bytes = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(pdf_bytes).unwrap();

    let mut engine = AcroFormEngine::parse(&doc).unwrap();
    let country_field = engine
        .get_field("Country")
        .expect("Country field must exist");

    if let FormFieldType::Choice { ref options, .. } = country_field.field_type {
        assert_eq!(options, &vec!["Canada", "USA", "UK"]);
    } else {
        panic!("Expected Choice field type");
    }

    assert_eq!(country_field.value, Some("USA".into()));

    // Mutate choice selection
    engine
        .set_field_value("Country", FormFieldValue::Choice("Canada".into()))
        .unwrap();

    assert_eq!(
        engine.get_field("Country").unwrap().value,
        Some("Canada".into())
    );
}

#[test]
fn test_11_acroform_hierarchical_field_names() {
    // Construct PDF with parent field "Customer" and child "Address"
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [ 5 0 R ] >> >>\nendobj\n",
    );

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n");

    // Parent field (Obj 5 0): /T (Customer) /Kids [ 6 0 R ] /FT /Tx
    let off_parent = doc_bytes.len();
    doc_bytes.extend_from_slice(b"5 0 obj\n<< /T (Customer) /FT /Tx /Kids [ 6 0 R ] >>\nendobj\n");

    // Child leaf field (Obj 6 0): /T (Address) /V (123 Main St)
    let off_child = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"6 0 obj\n<< /Type /Annot /Subtype /Widget /T (Address) /V (123 Main St) /Rect [ 100 500 300 530 ] >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 7\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_parent, off_child, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).unwrap();
    let engine = AcroFormEngine::parse(&doc).unwrap();

    let field = engine
        .get_field("Customer.Address")
        .expect("Full name Customer.Address must resolve");

    assert_eq!(field.full_name, "Customer.Address");
    assert_eq!(field.value, Some("123 Main St".into()));
    // Inherited /FT /Tx
    assert!(matches!(field.field_type, FormFieldType::Text { .. }));
}

#[test]
fn test_12_acroform_appearance_stream_generation() {
    let rect = PdfRect::new(100.0, 700.0, 250.0, 730.0);
    let ap = AcroFormEngine::generate_text_appearance(&rect, "Jane Smith", "/Helv 12 Tf 0 g");

    let script = String::from_utf8(ap.decode().unwrap()).unwrap();
    assert!(script.contains("Jane Smith"));
    assert!(script.contains("/Helv 12 Tf 0 g"));
}

#[test]
fn test_13_acroform_export_to_incremental_writer() {
    let base_pdf = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(base_pdf.clone()).unwrap();

    let mut engine = AcroFormEngine::parse(&doc).unwrap();
    engine
        .set_field_value("Full_Name", FormFieldValue::Text("Alice Cooper".into()))
        .unwrap();

    let mut writer = PdfIncrementalWriter::from_bytes(&base_pdf).unwrap();
    engine.export_incremental_update(&mut writer).unwrap();
    let updated_bytes = writer.finish().unwrap();

    // Verify updated PDF has the new form value
    let updated_doc = PdfDocument::parse(updated_bytes).unwrap();
    let updated_engine = AcroFormEngine::parse(&updated_doc).unwrap();

    let name_field = updated_engine.get_field("Full_Name").unwrap();
    assert_eq!(name_field.value, Some("Alice Cooper".into()));
}

#[test]
fn test_14_ink_annotation_added_via_incremental_writer() {
    let base_pdf = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(base_pdf.clone()).unwrap();

    // Create freehand stylus signature
    let stroke = InkStroke::new(vec![(150.0, 400.0), (180.0, 420.0), (220.0, 390.0)]);
    let ink = InkAnnotation::new(vec![stroke], [0.1, 0.2, 0.9], 2.5);

    let mut writer = PdfIncrementalWriter::from_bytes(&base_pdf).unwrap();
    let ink_id = writer.add_object(PdfObject::Dictionary(ink.to_dict()));

    // Update Page 3 /Annots to include newly drawn ink
    let mut page_dict = doc
        .resolve_object(PdfObjectId::new(3, 0))
        .unwrap()
        .as_dict()
        .unwrap()
        .clone();
    let mut annots = page_dict
        .get("Annots")
        .unwrap()
        .as_array()
        .unwrap()
        .to_vec();
    annots.push(PdfObject::Reference(ink_id));
    page_dict.insert("Annots".into(), PdfObject::Array(annots));

    writer.set_object(PdfObjectId::new(3, 0), PdfObject::Dictionary(page_dict));
    let updated_bytes = writer.finish().unwrap();

    let updated_doc = PdfDocument::parse(updated_bytes).unwrap();
    let annotations = AnnotationEngine::extract_annotations(&updated_doc).unwrap();

    assert_eq!(annotations.len(), 4); // 3 original widgets + 1 newly added ink
    let ink_annot = annotations.iter().find(|a| a.id == ink_id).unwrap();
    assert!(matches!(ink_annot.subtype, AnnotationSubtype::Ink { .. }));
}

#[test]
fn test_15_performance_acroform_and_ink_sla() {
    let base_pdf = build_baseline_acroform_pdf();
    let doc = PdfDocument::parse(base_pdf.clone()).unwrap();

    let start = Instant::now();
    let mut engine = AcroFormEngine::parse(&doc).unwrap();
    engine
        .set_field_value("Full_Name", FormFieldValue::Text("Performance SLA".into()))
        .unwrap();

    let stroke = InkStroke::new(vec![(10.0, 10.0), (50.0, 50.0), (100.0, 20.0)]);
    let ink = InkAnnotation::new(vec![stroke], [0.0, 0.0, 0.0], 2.0);
    let _ap = ink.compile_appearance_stream();
    let elapsed = start.elapsed();

    // SLA: Form parsing, field value mutation, and ink appearance generation < 1.0 ms
    assert!(
        elapsed.as_micros() < 1000,
        "AcroForm and Ink SLA took {:?}, must be < 1.0 ms",
        elapsed
    );
}
