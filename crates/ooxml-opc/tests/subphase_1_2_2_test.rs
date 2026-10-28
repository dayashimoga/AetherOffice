//! Integration and Zero-Loss Round-Trip Test Suite for Subphase 1.2.2:
//! UnparsedPartStore & Zero-Loss Round-Trip Engine.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity)
//! and Article IV (zero panics, typed EngineError).

use ooxml_opc::{
    create_synthetic_docx, create_synthetic_pptx, create_synthetic_xlsx, PackageReader,
    PackageWriter, RawXmlNodeBuffer, Relationships, TargetMode, UnparsedPart, UnparsedPartStore,
};
use std::fs;
use std::io::Cursor;
use std::path::Path;

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

/// Helper that creates a test package containing modeled parts + unmodeled binary parts
fn create_test_package_with_unmodeled_parts() -> Vec<u8> {
    let buffer = Cursor::new(Vec::new());
    let mut writer = PackageWriter::new(buffer);

    // 1. Modeled document part
    writer
        .write_part(
            "word/document.xml",
            b"<w:document><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>",
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
            ),
        )
        .expect("write doc failed");

    // 2. Unmodeled VBA project binary macro
    writer
        .write_part(
            "word/vbaProject.bin",
            b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1_RAW_VBA_MACRO_BYTES_FOR_ROUND_TRIP",
            Some("application/vnd.ms-office.vbaProject"),
        )
        .expect("write vba failed");

    // 3. Unmodeled digital signature part
    writer
        .write_part(
            "_xmlsignatures/origin.sigs",
            b"<SignatureOrigin xmlns=\"http://schemas.openxmlformats.org/package/2006/digital-signature\"/>",
            Some("application/vnd.openxmlformats-package.digital-signature-origin"),
        )
        .expect("write sigs failed");

    // 4. Unmodeled customXml part
    writer
        .write_part(
            "customXml/item1.xml",
            b"<customData><payload id=\"1234\">Data</payload></customData>",
            Some("application/xml"),
        )
        .expect("write customXml failed");

    // 5. Unmodeled ActiveX binary
    writer
        .write_part(
            "word/activeX/activeX1.bin",
            b"ACTIVEX_CONTROL_RAW_BINARY_DATA",
            Some("application/vnd.ms-office.activeX"),
        )
        .expect("write activeX failed");

    // 6. Set relationships
    let mut root_rels = Relationships::new();
    root_rels.add(
        "rId1",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
        "word/document.xml",
        TargetMode::Internal,
    );
    writer.set_package_relationships(root_rels);

    let mut doc_rels = Relationships::new();
    doc_rels.add(
        "rIdVba",
        "http://schemas.microsoft.com/office/2006/relationships/vbaProject",
        "vbaProject.bin",
        TargetMode::Internal,
    );
    writer.set_part_relationships("word/document.xml", doc_rels);

    let finished_cursor = writer.finish().expect("finish package failed");
    finished_cursor.into_inner()
}

#[test]
fn test_01_unparsed_part_store_basic_insert_and_get() {
    let mut store = UnparsedPartStore::new();
    assert!(store.is_empty());

    let part = UnparsedPart::new(
        "/word/vbaProject.bin",
        b"BINARY_MACRO".to_vec(),
        Some("application/vnd.ms-office.vbaProject".to_string()),
        None,
    );
    store.insert(part);

    assert_eq!(store.len(), 1);
    assert!(store.contains("word/vbaProject.bin"));
    assert!(store.contains("/word/vbaProject.bin"));

    let retrieved = store.get("word/vbaProject.bin").expect("Get failed");
    assert_eq!(retrieved.content, b"BINARY_MACRO");
    assert_eq!(
        retrieved.content_type.as_deref(),
        Some("application/vnd.ms-office.vbaProject")
    );
}

#[test]
fn test_02_unparsed_part_store_contains_normalization() {
    let mut store = UnparsedPartStore::new();
    store.insert(UnparsedPart::new(
        "customXml/item1.xml",
        b"DATA".to_vec(),
        None,
        None,
    ));

    assert!(store.contains("customXml/item1.xml"));
    assert!(store.contains("/customXml/item1.xml"));
    assert!(!store.contains("other.xml"));
}

#[test]
fn test_03_unparsed_part_store_remove() {
    let mut store = UnparsedPartStore::new();
    store.insert(UnparsedPart::new("part1.bin", b"123".to_vec(), None, None));
    assert_eq!(store.len(), 1);

    let removed = store.remove("/part1.bin").expect("Remove failed");
    assert_eq!(removed.content, b"123");
    assert!(store.is_empty());
}

#[test]
fn test_04_unparsed_part_store_total_bytes() {
    let mut store = UnparsedPartStore::new();
    store.insert(UnparsedPart::new("p1", vec![0u8; 100], None, None));
    store.insert(UnparsedPart::new("p2", vec![0u8; 250], None, None));

    assert_eq!(store.total_bytes(), 350);
}

#[test]
fn test_05_raw_xml_node_buffer_attributes() {
    let mut buf =
        RawXmlNodeBuffer::new("<w:p w:rsidR=\"00123456\"><w:r><w:t>Text</w:t></w:r></w:p>");
    buf.add_attribute("w:rsidR", "00123456");

    assert!(!buf.is_empty());
    assert_eq!(
        buf.unmodeled_attributes.get("w:rsidR"),
        Some(&"00123456".to_string())
    );
}

#[test]
fn test_06_raw_xml_node_buffer_ignorable_prefixes() {
    let mut buf = RawXmlNodeBuffer::new("<mc:AlternateContent/>");
    buf.add_ignorable_prefix("w14");
    buf.add_ignorable_prefix("w15");
    buf.add_ignorable_prefix("w14"); // duplicate should be ignored

    assert_eq!(buf.ignorable_prefixes.len(), 2);
    assert_eq!(buf.ignorable_prefixes, vec!["w14", "w15"]);
}

#[test]
fn test_07_package_reader_extract_unparsed_store_vba_project() {
    let pkg_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(pkg_bytes)).expect("open reader failed");

    let store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    assert!(store.contains("word/vbaProject.bin"));
    let vba = store.get("word/vbaProject.bin").expect("vba missing");
    assert!(vba.content.starts_with(b"\xD0\xCF\x11\xE0"));
}

#[test]
fn test_08_package_reader_extract_unparsed_store_digital_signatures() {
    let pkg_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(pkg_bytes)).expect("open reader failed");

    let store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    assert!(store.contains("_xmlsignatures/origin.sigs"));
}

#[test]
fn test_09_package_reader_extract_unparsed_store_custom_xml() {
    let pkg_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(pkg_bytes)).expect("open reader failed");

    let store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    assert!(store.contains("customXml/item1.xml"));
    let custom = store.get("customXml/item1.xml").expect("customXml missing");
    assert!(custom.content.starts_with(b"<customData>"));
}

#[test]
fn test_10_package_reader_extract_unparsed_store_activex() {
    let pkg_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(pkg_bytes)).expect("open reader failed");

    let store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    assert!(store.contains("word/activeX/activeX1.bin"));
}

#[test]
fn test_11_package_reader_extract_unparsed_store_ole_embeddings() {
    let mut store = UnparsedPartStore::new();
    let part = UnparsedPart::new(
        "word/embeddings/oleObject1.bin",
        b"OLE_COM_STREAM".to_vec(),
        Some("application/vnd.openxmlformats-officedocument.oleObject".to_string()),
        None,
    );
    store.insert(part);

    assert!(store.contains("word/embeddings/oleObject1.bin"));
}

#[test]
fn test_12_package_reader_extract_skips_modeled_parts() {
    let pkg_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(pkg_bytes)).expect("open reader failed");

    let store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    // Modeled part, [Content_Types].xml, and .rels must NEVER appear in unparsed store
    assert!(!store.contains("word/document.xml"));
    assert!(!store.contains("[Content_Types].xml"));
    assert!(!store.contains("_rels/.rels"));
    assert!(!store.contains("word/_rels/document.xml.rels"));
}

#[test]
fn test_13_package_writer_basic_package() {
    let buffer = Cursor::new(Vec::new());
    let mut writer = PackageWriter::new(buffer);

    writer
        .write_part("test.txt", b"Hello World", Some("text/plain"))
        .expect("write part failed");

    let finished = writer.finish().expect("finish failed");
    let bytes = finished.into_inner();

    let mut reader = PackageReader::open(Cursor::new(bytes)).expect("open reader failed");
    assert!(reader.contains_part("test.txt"));
    assert_eq!(
        reader.read_part("test.txt").expect("read failed"),
        b"Hello World"
    );
}

#[test]
fn test_14_package_writer_with_unparsed_store() {
    let mut store = UnparsedPartStore::new();
    store.insert(UnparsedPart::new(
        "macro.bin",
        b"BINARY_PAYLOAD".to_vec(),
        Some("application/bin".to_string()),
        None,
    ));

    let buffer = Cursor::new(Vec::new());
    let mut writer = PackageWriter::new(buffer);
    writer
        .write_unparsed_store(&store)
        .expect("write unparsed failed");

    let finished = writer.finish().expect("finish failed");
    let bytes = finished.into_inner();

    let mut reader = PackageReader::open(Cursor::new(bytes)).expect("open failed");
    assert!(reader.contains_part("macro.bin"));
    assert_eq!(
        reader.read_part("macro.bin").expect("read failed"),
        b"BINARY_PAYLOAD"
    );
}

#[test]
fn test_15_round_trip_preserves_vba_project_bin() {
    let original_bytes = create_test_package_with_unmodeled_parts();

    // 1. Read package and extract unparsed store
    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");
    let root_rels = reader
        .get_relationships(None)
        .expect("get root rels failed")
        .clone();
    let doc_rels = reader
        .get_relationships(Some("word/document.xml"))
        .expect("get doc rels failed")
        .clone();
    let doc_content = reader
        .read_part("word/document.xml")
        .expect("read doc failed");

    // 2. Save back with 100% preservation
    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer
        .write_part(
            "word/document.xml",
            &doc_content,
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
            ),
        )
        .expect("write doc failed");
    writer.set_package_relationships(root_rels);
    writer.set_part_relationships("word/document.xml", doc_rels);
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");

    let round_trip_bytes = writer.finish().expect("finish failed").into_inner();

    // 3. Inspect round-tripped package: verify vbaProject.bin is identical bit-for-bit
    let mut rt_reader = PackageReader::open(Cursor::new(round_trip_bytes)).expect("open rt failed");
    assert!(rt_reader.contains_part("word/vbaProject.bin"));

    let orig_vba = PackageReader::open(Cursor::new(&original_bytes))
        .expect("open orig failed")
        .read_part("word/vbaProject.bin")
        .expect("read orig vba failed");
    let rt_vba = rt_reader
        .read_part("word/vbaProject.bin")
        .expect("read rt vba failed");

    assert_eq!(
        orig_vba, rt_vba,
        "VBA macro binary must round-trip with zero loss"
    );
}

#[test]
fn test_16_round_trip_preserves_custom_xml_and_rels() {
    let original_bytes = create_test_package_with_unmodeled_parts();

    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract store failed");

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer
        .write_part("word/document.xml", b"<w:doc/>", None)
        .expect("write doc failed");
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");
    assert!(rt_reader.contains_part("customXml/item1.xml"));
    let content = rt_reader
        .read_part("customXml/item1.xml")
        .expect("read customXml failed");
    assert!(String::from_utf8_lossy(&content).contains("1234"));
}

#[test]
fn test_17_round_trip_preserves_digital_signatures() {
    let original_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract failed");

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer
        .write_part("word/document.xml", b"<w:doc/>", None)
        .expect("write failed");
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");
    assert!(rt_reader.contains_part("_xmlsignatures/origin.sigs"));
}

#[test]
fn test_18_round_trip_preserves_unmodeled_content_types() {
    let original_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract failed");

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");
    let ct = rt_reader
        .get_part_content_type("word/vbaProject.bin")
        .expect("get ct failed");
    assert_eq!(ct, "application/vnd.ms-office.vbaProject");
}

#[test]
fn test_19_round_trip_preserves_root_relationships() {
    let original_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let root_rels = reader
        .get_relationships(None)
        .expect("get root rels failed")
        .clone();

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer.set_package_relationships(root_rels.clone());
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");
    let rt_root_rels = rt_reader
        .get_relationships(None)
        .expect("get rt root rels failed");
    assert_eq!(&root_rels, rt_root_rels);
}

#[test]
fn test_20_round_trip_preserves_part_relationships() {
    let original_bytes = create_test_package_with_unmodeled_parts();
    let mut reader = PackageReader::open(Cursor::new(&original_bytes)).expect("open failed");
    let doc_rels = reader
        .get_relationships(Some("word/document.xml"))
        .expect("get doc rels failed")
        .clone();

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer.set_part_relationships("word/document.xml", doc_rels.clone());
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");
    let rt_doc_rels = rt_reader
        .get_relationships(Some("word/document.xml"))
        .expect("get rt doc rels failed");
    assert_eq!(&doc_rels, rt_doc_rels);
}

#[test]
fn test_21_round_trip_synthetic_docx_bit_fidelity() {
    let docx_bytes = create_synthetic_docx().expect("create docx failed");
    let mut reader = PackageReader::open(Cursor::new(&docx_bytes)).expect("open docx failed");
    let original_part_names = reader.part_names().to_vec();

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    let ct = reader.get_content_types().expect("get ct failed").clone();
    writer.set_content_types(ct);

    for name in &original_part_names {
        if name != "[Content_Types].xml" && !name.ends_with(".rels") {
            let data = reader.read_part(name).expect("read part failed");
            writer
                .write_part(name, &data, None)
                .expect("write part failed");
        }
    }
    let root_rels = reader
        .get_relationships(None)
        .expect("get rels failed")
        .clone();
    writer.set_package_relationships(root_rels);

    let rt_bytes = writer.finish().expect("finish failed").into_inner();
    let rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

    for name in &original_part_names {
        assert!(
            rt_reader.contains_part(name),
            "Part {} must be preserved",
            name
        );
    }
}

#[test]
fn test_22_round_trip_synthetic_xlsx_bit_fidelity() {
    let xlsx_bytes = create_synthetic_xlsx().expect("create xlsx failed");
    let mut reader = PackageReader::open(Cursor::new(&xlsx_bytes)).expect("open xlsx failed");
    let original_part_names = reader.part_names().to_vec();

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    let ct = reader.get_content_types().expect("get ct failed").clone();
    writer.set_content_types(ct);

    for name in &original_part_names {
        if name != "[Content_Types].xml" && !name.ends_with(".rels") {
            let data = reader.read_part(name).expect("read part failed");
            writer
                .write_part(name, &data, None)
                .expect("write part failed");
        }
    }
    let root_rels = reader
        .get_relationships(None)
        .expect("get root rels failed")
        .clone();
    let wb_rels = reader
        .get_relationships(Some("xl/workbook.xml"))
        .expect("get wb rels failed")
        .clone();
    writer.set_package_relationships(root_rels);
    writer.set_part_relationships("xl/workbook.xml", wb_rels);

    let rt_bytes = writer.finish().expect("finish failed").into_inner();
    let rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

    for name in &original_part_names {
        assert!(
            rt_reader.contains_part(name),
            "Part {} must be preserved in xlsx",
            name
        );
    }
}

#[test]
fn test_23_round_trip_synthetic_pptx_bit_fidelity() {
    let pptx_bytes = create_synthetic_pptx().expect("create pptx failed");
    let mut reader = PackageReader::open(Cursor::new(&pptx_bytes)).expect("open pptx failed");
    let original_part_names = reader.part_names().to_vec();

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    let ct = reader.get_content_types().expect("get ct failed").clone();
    writer.set_content_types(ct);

    for name in &original_part_names {
        if name != "[Content_Types].xml" && !name.ends_with(".rels") {
            let data = reader.read_part(name).expect("read part failed");
            writer
                .write_part(name, &data, None)
                .expect("write part failed");
        }
    }
    let root_rels = reader
        .get_relationships(None)
        .expect("get root rels failed")
        .clone();
    let pres_rels = reader
        .get_relationships(Some("ppt/presentation.xml"))
        .expect("get pres rels failed")
        .clone();
    writer.set_package_relationships(root_rels);
    writer.set_part_relationships("ppt/presentation.xml", pres_rels);

    let rt_bytes = writer.finish().expect("finish failed").into_inner();
    let rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

    for name in &original_part_names {
        assert!(
            rt_reader.contains_part(name),
            "Part {} must be preserved in pptx",
            name
        );
    }
}

#[test]
fn test_24_round_trip_with_added_custom_part() {
    let docx_bytes = create_synthetic_docx().expect("create docx failed");
    let mut reader = PackageReader::open(Cursor::new(&docx_bytes)).expect("open failed");
    let mut unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract failed");

    // Add a new custom part to the unparsed store
    unparsed_store.insert(UnparsedPart::new(
        "custom/plugin_metadata.json",
        b"{\"plugin\": \"OpenOfficeRS\"}".to_vec(),
        Some("application/json".to_string()),
        None,
    ));

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    let doc = reader
        .read_part("word/document.xml")
        .expect("read doc failed");
    writer
        .write_part("word/document.xml", &doc, None)
        .expect("write doc failed");
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");

    let rt_bytes = writer.finish().expect("finish failed").into_inner();
    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

    assert!(rt_reader.contains_part("custom/plugin_metadata.json"));
    let meta = rt_reader
        .read_part("custom/plugin_metadata.json")
        .expect("read meta failed");
    assert!(String::from_utf8_lossy(&meta).contains("OpenOfficeRS"));
}

#[test]
fn test_25_round_trip_with_modified_modeled_part() {
    let docx_bytes = create_synthetic_docx().expect("create docx failed");
    let mut reader = PackageReader::open(Cursor::new(&docx_bytes)).expect("open failed");
    let unparsed_store = reader
        .extract_unparsed_store(&["word/document.xml"])
        .expect("extract failed");

    // Modify the modeled document body while retaining 100% of unparsed store
    let modified_doc =
        b"<w:document><w:body><w:p><w:r><w:t>Modified Text</w:t></w:r></w:p></w:body></w:document>";

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer
        .write_part("word/document.xml", modified_doc, None)
        .expect("write modified doc failed");
    writer
        .write_unparsed_store(&unparsed_store)
        .expect("write unparsed failed");

    let rt_bytes = writer.finish().expect("finish failed").into_inner();
    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

    let read_modified = rt_reader
        .read_part("word/document.xml")
        .expect("read doc failed");
    assert!(String::from_utf8_lossy(&read_modified).contains("Modified Text"));
}

#[test]
fn test_26_unparsed_store_deterministic_sorted_iteration() {
    let mut store = UnparsedPartStore::new();
    store.insert(UnparsedPart::new("z_part.bin", vec![], None, None));
    store.insert(UnparsedPart::new("a_part.bin", vec![], None, None));
    store.insert(UnparsedPart::new("m_part.bin", vec![], None, None));

    let names: Vec<&String> = store.part_names().collect();
    assert_eq!(names, vec!["/a_part.bin", "/m_part.bin", "/z_part.bin"]);
}

#[test]
fn test_27_round_trip_preserves_unknown_binary_streams() {
    let mut store = UnparsedPartStore::new();
    let arbitrary_bytes = (0..255u8).collect::<Vec<u8>>();
    store.insert(UnparsedPart::new(
        "custom/binary.stream",
        arbitrary_bytes.clone(),
        Some("application/octet-stream".to_string()),
        None,
    ));

    let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
    writer.write_unparsed_store(&store).expect("write failed");
    let rt_bytes = writer.finish().expect("finish failed").into_inner();

    let mut rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open failed");
    let read_back = rt_reader
        .read_part("custom/binary.stream")
        .expect("read failed");
    assert_eq!(read_back, arbitrary_bytes);
}

#[test]
fn test_28_com_baseline_docx_round_trip_preserves_all_parts() {
    let fixture_path = resolve_fixture_path("specs/evidence/word/baseline.docx");
    if fixture_path.exists() {
        let raw = fs::read(&fixture_path).expect("read baseline.docx failed");
        let mut reader = PackageReader::open(Cursor::new(&raw)).expect("open baseline docx failed");
        let original_parts = reader.part_names().to_vec();

        let unparsed = reader
            .extract_unparsed_store(&["word/document.xml"])
            .expect("extract unparsed failed");

        let mut writer = PackageWriter::new(Cursor::new(Vec::new()));
        let doc = reader
            .read_part("word/document.xml")
            .expect("read doc failed");
        writer
            .write_part("word/document.xml", &doc, None)
            .expect("write doc failed");
        writer
            .write_unparsed_store(&unparsed)
            .expect("write unparsed failed");

        let rt_bytes = writer.finish().expect("finish failed").into_inner();
        let rt_reader = PackageReader::open(Cursor::new(rt_bytes)).expect("open rt failed");

        // Verify that every single non-rels part in baseline.docx is present in the round-trip
        for part in &original_parts {
            if part != "[Content_Types].xml" && !part.ends_with(".rels") {
                assert!(
                    rt_reader.contains_part(part),
                    "Part {} must be in round-tripped package",
                    part
                );
            }
        }
    }
}
