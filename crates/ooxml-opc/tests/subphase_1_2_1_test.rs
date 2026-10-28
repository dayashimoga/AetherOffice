//! Integration and Edge-Case Test Suite for Subphase 1.2.1:
//! Streaming ZIP & [Content_Types].xml / .rels Parser.
//! Strictly conforms to GEC-v1.0 Article IV.

use ooxml_opc::{
    create_synthetic_docx, create_synthetic_xlsx, ContentTypes, EngineError, PackageReader,
    Relationships, TargetMode,
};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

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
fn test_01_parse_content_types_defaults_and_overrides() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

    let ct = ContentTypes::parse(Cursor::new(xml)).expect("Failed to parse ContentTypes");
    assert_eq!(ct.defaults.len(), 2);
    assert_eq!(ct.overrides.len(), 1);
    assert_eq!(
        ct.get_content_type("/word/document.xml"),
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml")
    );
    assert_eq!(
        ct.get_content_type("/word/settings.xml"),
        Some("application/xml")
    );
}

#[test]
fn test_02_content_types_override_priority_over_default() {
    let mut ct = ContentTypes::new();
    ct.add_default("xml", "application/xml");
    ct.add_override("/custom.xml", "application/custom+xml");

    assert_eq!(
        ct.get_content_type("/custom.xml"),
        Some("application/custom+xml")
    );
    assert_eq!(ct.get_content_type("/other.xml"), Some("application/xml"));
}

#[test]
fn test_03_content_types_case_insensitive_extension() {
    let mut ct = ContentTypes::new();
    ct.add_default("png", "image/png");

    assert_eq!(ct.get_content_type("/media/image1.PNG"), Some("image/png"));
    assert_eq!(ct.get_content_type("/media/image2.png"), Some("image/png"));
    assert_eq!(ct.get_content_type("/media/image3.Png"), Some("image/png"));
}

#[test]
fn test_04_content_types_normalized_part_name() {
    let mut ct = ContentTypes::new();
    ct.add_override("word/document.xml", "application/test+xml");

    // Must match with or without leading slash
    assert_eq!(
        ct.get_content_type("word/document.xml"),
        Some("application/test+xml")
    );
    assert_eq!(
        ct.get_content_type("/word/document.xml"),
        Some("application/test+xml")
    );
}

#[test]
fn test_05_content_types_serialization_round_trip() {
    let mut original = ContentTypes::new();
    original.add_default("xml", "application/xml");
    original.add_default(
        "rels",
        "application/vnd.openxmlformats-package.relationships+xml",
    );
    original.add_override("/doc.xml", "application/doc+xml");

    let bytes = original.serialize().expect("Serialization failed");
    let reparsed = ContentTypes::parse(Cursor::new(bytes)).expect("Reparsing failed");

    assert_eq!(original, reparsed);
}

#[test]
fn test_06_content_types_unknown_extension_returns_none() {
    let ct = ContentTypes::new();
    assert_eq!(ct.get_content_type("/unknown.foo"), None);
}

#[test]
fn test_07_parse_relationships_basic() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://example.com/rel" Target="target.xml"/>
  <Relationship Id="rId2" Type="http://example.com/ext" Target="https://example.com" TargetMode="External"/>
</Relationships>"#;

    let rels = Relationships::parse(Cursor::new(xml)).expect("Failed to parse Relationships");
    assert_eq!(rels.len(), 2);

    let r1 = rels.get("rId1").expect("rId1 missing");
    assert_eq!(r1.target, "target.xml");
    assert_eq!(r1.target_mode, TargetMode::Internal);

    let r2 = rels.get("rId2").expect("rId2 missing");
    assert_eq!(r2.target, "https://example.com");
    assert_eq!(r2.target_mode, TargetMode::External);
}

#[test]
fn test_08_parse_relationships_with_target_mode() {
    let mut rels = Relationships::new();
    rels.add("rId1", "type1", "target1", TargetMode::Internal);
    rels.add("rId2", "type2", "https://example.org", TargetMode::External);

    assert_eq!(
        rels.get("rId1").map(|r| r.target_mode),
        Some(TargetMode::Internal)
    );
    assert_eq!(
        rels.get("rId2").map(|r| r.target_mode),
        Some(TargetMode::External)
    );
}

#[test]
fn test_09_relationships_serialization_round_trip() {
    let mut original = Relationships::new();
    original.add("rId1", "http://type1", "target1.xml", TargetMode::Internal);
    original.add(
        "rId2",
        "http://type2",
        "http://external.com",
        TargetMode::External,
    );

    let bytes = original.serialize().expect("Serialize rels failed");
    let reparsed = Relationships::parse(Cursor::new(bytes)).expect("Reparse rels failed");

    assert_eq!(original, reparsed);
}

#[test]
fn test_10_relationships_resolve_root_target() {
    let mut rels = Relationships::new();
    rels.add("rId1", "type", "word/document.xml", TargetMode::Internal);

    let resolved = rels.resolve_target("rId1", None).expect("Resolve failed");
    assert_eq!(resolved, "word/document.xml");
}

#[test]
fn test_11_relationships_resolve_relative_sibling_target() {
    let mut rels = Relationships::new();
    rels.add("rId2", "type", "styles.xml", TargetMode::Internal);

    let resolved = rels
        .resolve_target("rId2", Some("word/document.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "word/styles.xml");
}

#[test]
fn test_12_relationships_resolve_relative_nested_target() {
    let mut rels = Relationships::new();
    rels.add("rId3", "type", "media/image1.png", TargetMode::Internal);

    let resolved = rels
        .resolve_target("rId3", Some("word/document.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "word/media/image1.png");
}

#[test]
fn test_13_relationships_resolve_parent_traversal() {
    let mut rels = Relationships::new();
    rels.add(
        "rId4",
        "type",
        "../customXml/item1.xml",
        TargetMode::Internal,
    );

    let resolved = rels
        .resolve_target("rId4", Some("word/document.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "customXml/item1.xml");
}

#[test]
fn test_14_relationships_resolve_multi_parent_traversal() {
    let mut rels = Relationships::new();
    rels.add(
        "rId5",
        "type",
        "../../shared/theme.xml",
        TargetMode::Internal,
    );

    let resolved = rels
        .resolve_target("rId5", Some("ppt/slides/slide1.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "shared/theme.xml");
}

#[test]
fn test_15_relationships_resolve_absolute_internal_target() {
    let mut rels = Relationships::new();
    rels.add("rId6", "type", "/docProps/core.xml", TargetMode::Internal);

    let resolved = rels
        .resolve_target("rId6", Some("word/document.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "docProps/core.xml");
}

#[test]
fn test_16_relationships_resolve_external_target_preserves_url() {
    let mut rels = Relationships::new();
    rels.add(
        "rId7",
        "type",
        "https://rust-lang.org/learn",
        TargetMode::External,
    );

    let resolved = rels
        .resolve_target("rId7", Some("word/document.xml"))
        .expect("Resolve failed");
    assert_eq!(resolved, "https://rust-lang.org/learn");
}

#[test]
fn test_17_relationships_missing_rel_id_returns_error() {
    let rels = Relationships::new();
    let err = rels.resolve_target("rId999", None).unwrap_err();
    match err {
        EngineError::RelationshipNotFound(msg) => {
            assert!(msg.contains("rId999"));
        }
        other => panic!("Expected RelationshipNotFound, got: {:?}", other),
    }
}

#[test]
fn test_18_rels_part_name_for_root_and_parts() {
    assert_eq!(Relationships::rels_part_name_for(None), "_rels/.rels");
    assert_eq!(Relationships::rels_part_name_for(Some("/")), "_rels/.rels");
    assert_eq!(
        Relationships::rels_part_name_for(Some("word/document.xml")),
        "word/_rels/document.xml.rels"
    );
    assert_eq!(
        Relationships::rels_part_name_for(Some("xl/worksheets/sheet1.xml")),
        "xl/worksheets/_rels/sheet1.xml.rels"
    );
}

#[test]
fn test_19_package_reader_open_synthetic_docx() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    assert_eq!(reader.len(), 3);
    assert!(reader.contains_part("[Content_Types].xml"));
    assert!(reader.contains_part("_rels/.rels"));
    assert!(reader.contains_part("word/document.xml"));
}

#[test]
fn test_20_package_reader_contains_part_with_and_without_slash() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    assert!(reader.contains_part("word/document.xml"));
    assert!(reader.contains_part("/word/document.xml"));
    assert!(!reader.contains_part("word/missing.xml"));
}

#[test]
fn test_21_package_reader_lazy_content_types() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let ct = reader
        .get_content_types()
        .expect("Get content types failed");
    assert!(ct.overrides.contains_key("/word/document.xml"));
}

#[test]
fn test_22_package_reader_get_part_content_type() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let ct_str = reader
        .get_part_content_type("word/document.xml")
        .expect("Get content type failed");
    assert_eq!(
        ct_str,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
    );
}

#[test]
fn test_23_package_reader_lazy_relationships() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let root_rels = reader
        .get_relationships(None)
        .expect("Get root rels failed");
    assert_eq!(root_rels.len(), 1);
    assert!(root_rels.get("rId1").is_some());
}

#[test]
fn test_24_package_reader_resolve_relationship_target() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let target = reader
        .resolve_relationship_target(None, "rId1")
        .expect("Resolve target failed");
    assert_eq!(target, "word/document.xml");
}

#[test]
fn test_25_package_reader_read_part_data() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let data = reader
        .read_part("word/document.xml")
        .expect("Read part failed");
    assert!(!data.is_empty());
    let xml_str = String::from_utf8(data).expect("Invalid utf8");
    assert!(xml_str.contains("OpenOfficeRS Synthetic Word Document"));
}

#[test]
fn test_26_package_reader_missing_part_returns_typed_error() {
    let docx_bytes = create_synthetic_docx().expect("Create docx failed");
    let mut reader = PackageReader::open(Cursor::new(docx_bytes)).expect("Open reader failed");

    let err = reader.read_part("word/missing.xml").unwrap_err();
    match err {
        EngineError::PartNotFound(name) => {
            assert!(name.contains("word/missing.xml"));
        }
        other => panic!("Expected EngineError::PartNotFound, got: {:?}", other),
    }
}

#[test]
fn test_27_opc_central_directory_indexing_latency_under_15ms() {
    // Generate a package and benchmark open + root relationship resolution
    let xlsx_bytes = create_synthetic_xlsx().expect("Create xlsx failed");

    let start = Instant::now();
    let mut reader = PackageReader::open(Cursor::new(&xlsx_bytes)).expect("Open failed");
    let rel_target = reader
        .resolve_relationship_target(None, "rId1")
        .expect("Resolve failed");
    let elapsed = start.elapsed();

    assert_eq!(rel_target, "xl/workbook.xml");
    assert!(
        elapsed.as_millis() < 15,
        "Expected open + relationship resolution in < 15ms, took {} ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_28_verify_all_26_edge_case_vectors_from_json() {
    let path = resolve_fixture_path("specs/evidence/word/1.2.1_vectors.json");
    assert!(path.exists(), "Vector spec must exist at {:?}", path);

    let content = fs::read_to_string(&path).expect("Failed to read 1.2.1_vectors.json");
    let json: Value = serde_json::from_str(&content).expect("Invalid JSON in vector spec");

    let vectors = json["vectors"].as_array().expect("vectors must be array");
    assert!(
        vectors.len() >= 25,
        "Expected >= 25 test vectors, found {}",
        vectors.len()
    );

    for v in vectors {
        let v_type = v["type"].as_str().expect("type required");
        match v_type {
            "content_type" => {
                let mut ct = ContentTypes::new();
                if let Some(defaults) = v["defaults"].as_object() {
                    for (ext, mime) in defaults {
                        ct.add_default(ext, mime.as_str().expect("mime must be string"));
                    }
                }
                if let Some(overrides) = v["overrides"].as_object() {
                    for (part, mime) in overrides {
                        ct.add_override(part, mime.as_str().expect("mime must be string"));
                    }
                }

                let input_part = v["input_part"].as_str().expect("input_part required");
                let actual = ct.get_content_type(input_part);
                let expected = v["expected_content_type"].as_str();
                assert_eq!(
                    actual, expected,
                    "Vector {} failed for part {}",
                    v["id"], input_part
                );
            }
            "relationship" => {
                let source_part = v["source_part"].as_str();
                let rel_data = &v["rel"];
                let id = rel_data["id"].as_str().expect("id required");
                let r_type = rel_data["type"].as_str().expect("type required");
                let target = rel_data["target"].as_str().expect("target required");
                let mode_str = rel_data["target_mode"].as_str().expect("mode required");
                let mode = TargetMode::from_str_opt(mode_str);

                let mut rels = Relationships::new();
                rels.add(id, r_type, target, mode);

                let resolved = rels
                    .resolve_target(id, source_part)
                    .expect("Resolve failed");
                let expected = v["expected_target"]
                    .as_str()
                    .expect("expected target required");
                assert_eq!(
                    resolved, expected,
                    "Vector {} failed: expected {} but got {}",
                    v["id"], expected, resolved
                );
            }
            "rels_naming" => {
                let source = v["source_part"].as_str();
                let actual = Relationships::rels_part_name_for(source);
                let expected = v["expected_rels_part"].as_str().expect("expected required");
                assert_eq!(actual, expected, "Vector {} failed", v["id"]);
            }
            "target_mode" => {
                let raw = v["raw_attr"].as_str().unwrap_or("");
                let mode = TargetMode::from_str_opt(raw);
                let expected_str = v["expected_mode"].as_str().expect("expected mode required");
                assert_eq!(mode.as_str(), expected_str, "Vector {} failed", v["id"]);
            }
            "package_lookup" => {
                let mut ct = ContentTypes::new();
                let stored = v["stored_part"].as_str().expect("stored_part required");
                ct.add_override(stored, "test/type");

                let query = v["query_part"].as_str().expect("query_part required");
                let found = ct.get_content_type(query).is_some();
                let expected = v["expected_found"]
                    .as_bool()
                    .expect("expected_found required");
                assert_eq!(found, expected, "Vector {} failed", v["id"]);
            }
            other => panic!("Unknown vector type: {}", other),
        }
    }
}
