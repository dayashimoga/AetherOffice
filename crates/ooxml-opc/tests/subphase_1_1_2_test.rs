//! Integration & Vector Test Suite for Subphase 1.1.2:
//! COM Ground-Truth Differential Harness & Synthetic Fixtures.
//! Conforms strictly to GEC-v1.0 Article IV.

use ooxml_opc::{
    create_synthetic_docx, create_synthetic_pptx, create_synthetic_xlsx, inspect_opc_package,
    read_part_content, EngineError,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn test_01_synthetic_docx_generation() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    assert!(!docx.is_empty(), "Docx buffer must not be empty");
    assert!(docx.len() > 100, "Docx must have realistic size");
    assert_eq!(
        &docx[0..2],
        b"PK",
        "Docx must have standard ZIP magic bytes PK"
    );
}

#[test]
fn test_02_synthetic_docx_contains_content_types() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let manifest = inspect_opc_package(&docx).expect("Manifest inspect failed");
    assert!(manifest.has_content_types);
    assert!(manifest.parts.contains(&"[Content_Types].xml".to_string()));
}

#[test]
fn test_03_synthetic_docx_contains_root_rels() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let manifest = inspect_opc_package(&docx).expect("Manifest inspect failed");
    assert!(manifest.has_root_rels);
    assert!(manifest.parts.contains(&"_rels/.rels".to_string()));
}

#[test]
fn test_04_synthetic_docx_contains_document_xml() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let manifest = inspect_opc_package(&docx).expect("Manifest inspect failed");
    assert!(manifest.parts.contains(&"word/document.xml".to_string()));
}

#[test]
fn test_05_synthetic_docx_xml_validity() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let content = read_part_content(&docx, "word/document.xml").expect("Read part failed");
    assert!(content.contains("<w:document"));
    assert!(content.contains("OpenOfficeRS Synthetic Word Document"));
}

#[test]
fn test_06_synthetic_xlsx_generation() {
    let xlsx = create_synthetic_xlsx().expect("Failed to create synthetic xlsx");
    assert!(!xlsx.is_empty(), "Xlsx buffer must not be empty");
    assert_eq!(
        &xlsx[0..2],
        b"PK",
        "Xlsx must have standard ZIP magic bytes PK"
    );
}

#[test]
fn test_07_synthetic_xlsx_contains_content_types() {
    let xlsx = create_synthetic_xlsx().expect("Failed to create synthetic xlsx");
    let manifest = inspect_opc_package(&xlsx).expect("Manifest inspect failed");
    assert!(manifest.has_content_types);
    assert!(manifest.parts.contains(&"[Content_Types].xml".to_string()));
}

#[test]
fn test_08_synthetic_xlsx_contains_root_rels() {
    let xlsx = create_synthetic_xlsx().expect("Failed to create synthetic xlsx");
    let manifest = inspect_opc_package(&xlsx).expect("Manifest inspect failed");
    assert!(manifest.has_root_rels);
    assert!(manifest.parts.contains(&"_rels/.rels".to_string()));
}

#[test]
fn test_09_synthetic_xlsx_contains_workbook_xml() {
    let xlsx = create_synthetic_xlsx().expect("Failed to create synthetic xlsx");
    let manifest = inspect_opc_package(&xlsx).expect("Manifest inspect failed");
    assert!(manifest.parts.contains(&"xl/workbook.xml".to_string()));
}

#[test]
fn test_10_synthetic_xlsx_contains_worksheet_xml() {
    let xlsx = create_synthetic_xlsx().expect("Failed to create synthetic xlsx");
    let manifest = inspect_opc_package(&xlsx).expect("Manifest inspect failed");
    assert!(manifest
        .parts
        .contains(&"xl/worksheets/sheet1.xml".to_string()));
    let content = read_part_content(&xlsx, "xl/worksheets/sheet1.xml").expect("Read sheet failed");
    assert!(content.contains("<worksheet"));
    assert!(content.contains("Synthetic Value A1"));
}

#[test]
fn test_11_synthetic_pptx_generation() {
    let pptx = create_synthetic_pptx().expect("Failed to create synthetic pptx");
    assert!(!pptx.is_empty(), "Pptx buffer must not be empty");
    assert_eq!(
        &pptx[0..2],
        b"PK",
        "Pptx must have standard ZIP magic bytes PK"
    );
}

#[test]
fn test_12_synthetic_pptx_contains_content_types() {
    let pptx = create_synthetic_pptx().expect("Failed to create synthetic pptx");
    let manifest = inspect_opc_package(&pptx).expect("Manifest inspect failed");
    assert!(manifest.has_content_types);
    assert!(manifest.parts.contains(&"[Content_Types].xml".to_string()));
}

#[test]
fn test_13_synthetic_pptx_contains_root_rels() {
    let pptx = create_synthetic_pptx().expect("Failed to create synthetic pptx");
    let manifest = inspect_opc_package(&pptx).expect("Manifest inspect failed");
    assert!(manifest.has_root_rels);
    assert!(manifest.parts.contains(&"_rels/.rels".to_string()));
}

#[test]
fn test_14_synthetic_pptx_contains_presentation_xml() {
    let pptx = create_synthetic_pptx().expect("Failed to create synthetic pptx");
    let manifest = inspect_opc_package(&pptx).expect("Manifest inspect failed");
    assert!(manifest.parts.contains(&"ppt/presentation.xml".to_string()));
}

#[test]
fn test_15_synthetic_pptx_contains_slide1_xml() {
    let pptx = create_synthetic_pptx().expect("Failed to create synthetic pptx");
    let manifest = inspect_opc_package(&pptx).expect("Manifest inspect failed");
    assert!(manifest
        .parts
        .contains(&"ppt/slides/slide1.xml".to_string()));
    let content = read_part_content(&pptx, "ppt/slides/slide1.xml").expect("Read slide failed");
    assert!(content.contains("<p:sld"));
}

#[test]
fn test_16_inspect_opc_package_valid_manifest() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let manifest = inspect_opc_package(&docx).expect("Manifest inspect failed");
    assert!(manifest.total_bytes > 0);
    assert_eq!(manifest.parts.len(), 3);
}

#[test]
fn test_17_inspect_empty_buffer_returns_malformed_error() {
    let empty = b"";
    let err = inspect_opc_package(empty).unwrap_err();
    match err {
        EngineError::MalformedPackage(msg) => {
            assert!(msg.contains("empty"));
        }
        other => panic!("Expected MalformedPackage, got: {:?}", other),
    }
}

#[test]
fn test_18_inspect_corrupted_zip_returns_zip_error() {
    let garbage = b"THIS_IS_NOT_A_VALID_ZIP_PACKAGE_DATA";
    let err = inspect_opc_package(garbage).unwrap_err();
    match err {
        EngineError::Zip(_) => {}
        other => panic!("Expected EngineError::Zip, got: {:?}", other),
    }
}

#[test]
fn test_19_read_part_content_existing_part() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let types = read_part_content(&docx, "[Content_Types].xml").expect("Read content types failed");
    assert!(types.contains("<Types"));
    assert!(types.contains("Extension=\"rels\""));
}

#[test]
fn test_20_read_part_content_missing_part_returns_not_found() {
    let docx = create_synthetic_docx().expect("Failed to create synthetic docx");
    let err = read_part_content(&docx, "non_existent_part.xml").unwrap_err();
    match err {
        EngineError::PartNotFound(name) => {
            assert_eq!(name, "non_existent_part.xml");
        }
        other => panic!("Expected EngineError::PartNotFound, got: {:?}", other),
    }
}

fn resolve_fixture_path(rel_path: &str) -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        Path::new(rel_path).to_path_buf(),
        manifest_dir.join("../../").join(rel_path),
        manifest_dir.join("../").join(rel_path),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

#[test]
fn test_21_excel_baseline_json_vector_parsing() {
    let path = resolve_fixture_path("specs/evidence/excel/baseline_vectors.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read excel baseline vectors");
        let json: Value = serde_json::from_str(&content).expect("Invalid JSON in excel vectors");
        assert_eq!(json["app"], "excel");
        let vectors = json["vectors"].as_array().expect("vectors must be array");
        assert!(vectors.len() >= 15, "Expected at least 15 formula vectors");
    }
}

#[test]
fn test_22_excel_baseline_ieee_snap_vectors() {
    let path = resolve_fixture_path("specs/evidence/excel/baseline_vectors.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read excel baseline vectors");
        let json: Value = serde_json::from_str(&content).expect("Invalid JSON");
        let vectors = json["vectors"].as_array().expect("vectors must be array");

        let snap_case = vectors
            .iter()
            .find(|v| v["id"] == "ieee_snap_01")
            .expect("ieee_snap_01 must be present");
        assert_eq!(snap_case["formula"], "=A1-B1-0.108");
    }
}

#[test]
fn test_23_excel_baseline_financial_pmt_vector() {
    let path = resolve_fixture_path("specs/evidence/excel/baseline_vectors.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read excel baseline vectors");
        let json: Value = serde_json::from_str(&content).expect("Invalid JSON");
        let vectors = json["vectors"].as_array().expect("vectors must be array");

        let pmt_case = vectors
            .iter()
            .find(|v| v["id"] == "fin_pmt_01")
            .expect("fin_pmt_01 must be present");
        assert_eq!(pmt_case["formula"], "=PMT(0.08/12, 10, 10000)");
    }
}

#[test]
fn test_24_word_baseline_json_vector_parsing() {
    let path = resolve_fixture_path("specs/evidence/word/baseline_vectors.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read word baseline vectors");
        let json: Value = serde_json::from_str(&content).expect("Invalid JSON");
        assert_eq!(json["app"], "word");
        assert!(json["document"]["sections"].is_array());
    }
}

#[test]
fn test_25_ppt_baseline_json_vector_parsing() {
    let path = resolve_fixture_path("specs/evidence/ppt/baseline_vectors.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read ppt baseline vectors");
        let json: Value = serde_json::from_str(&content).expect("Invalid JSON");
        assert_eq!(json["app"], "ppt");
        assert!(json["presentation"]["slides"].is_array());
    }
}

#[test]
fn test_26_com_or_synthetic_fixture_files_exist() {
    let excel_fixture = resolve_fixture_path("specs/evidence/excel/baseline.xlsx");
    let word_fixture = resolve_fixture_path("specs/evidence/word/baseline.docx");
    let ppt_fixture = resolve_fixture_path("specs/evidence/ppt/baseline.pptx");

    assert!(
        excel_fixture.exists(),
        "Excel fixture must exist at {:?}",
        excel_fixture
    );
    assert!(
        word_fixture.exists(),
        "Word fixture must exist at {:?}",
        word_fixture
    );
    assert!(
        ppt_fixture.exists(),
        "PPT fixture must exist at {:?}",
        ppt_fixture
    );
}
