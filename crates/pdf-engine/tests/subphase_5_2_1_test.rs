//! Subphase 5.2.1 Integration Test Suite:
//! Incremental Update Writer & Digital Signature Preservation
//!
//! Validates:
//! - Serialization of PDF primitives, composite objects, and streams
//! - Non-destructive incremental update writing (`PdfIncrementalWriter`)
//! - Adding new indirect objects and overriding existing objects
//! - Appending incremental `xref` tables and chained `/Prev` trailer pointers
//! - Multi-revision document history inspection (`RevisionHistory`)
//! - Rollback extraction of historic revision byte snapshots
//! - Digital signature detection (`/Type /Sig`, `/FT /Sig`)
//! - Cryptographic `/ByteRange` validation and SHA-256 hash computation
//! - Signature preservation across subsequent incremental updates
//! - Detection of byte tampering within signed ranges
//! - Sub-millisecond incremental update SLA benchmark (< 1.0 ms)

use pdf_engine::signature::SignatureValidator;
use pdf_engine::writer::{serialize_object, PdfIncrementalWriter};
use pdf_engine::{PdfDocument, PdfObject, PdfObjectId, PdfRect, PdfStream, RevisionHistory};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Instant;

fn build_minimal_pdf() -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes
        .extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>\nendobj\n");

    let content = b"BT /F1 12 Tf 72 712 Td (Baseline Document) Tj ET";
    let off_cont = doc_bytes.len();
    doc_bytes.extend_from_slice(
        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content.len(),
            std::str::from_utf8(content).unwrap()
        )
        .as_bytes(),
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 5\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_cont, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    doc_bytes
}

#[test]
fn test_01_serialize_pdf_primitives() {
    let mut out = Vec::new();

    serialize_object(&PdfObject::Null, &mut out);
    assert_eq!(out, b"null");
    out.clear();

    serialize_object(&PdfObject::Boolean(true), &mut out);
    assert_eq!(out, b"true");
    out.clear();

    serialize_object(&PdfObject::Integer(42), &mut out);
    assert_eq!(out, b"42");
    out.clear();

    serialize_object(&PdfObject::Real(12.5), &mut out);
    assert_eq!(out, b"12.5");
    out.clear();

    serialize_object(&PdfObject::Name("Type".into()), &mut out);
    assert_eq!(out, b"/Type");
    out.clear();

    // Name with spaces and delimiters
    serialize_object(&PdfObject::Name("Name With Spaces#Tag".into()), &mut out);
    assert_eq!(out, b"/Name#20With#20Spaces#23Tag");
    out.clear();
}

#[test]
fn test_02_serialize_strings_and_hex() {
    let mut out = Vec::new();

    // Literal string with escapes
    serialize_object(
        &PdfObject::LiteralString(b"Line1\nLine2 (nested) \\ backslash".to_vec()),
        &mut out,
    );
    assert_eq!(out, b"(Line1\\nLine2 \\(nested\\) \\\\ backslash)");
    out.clear();

    // Hex string
    serialize_object(
        &PdfObject::HexString(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]),
        &mut out,
    );
    assert_eq!(out, b"<48656C6C6F>");
    out.clear();
}

#[test]
fn test_03_serialize_composite_objects() {
    let mut out = Vec::new();

    let arr = PdfObject::Array(vec![
        PdfObject::Integer(1),
        PdfObject::Name("Item".into()),
        PdfObject::Reference(PdfObjectId::new(10, 0)),
    ]);
    serialize_object(&arr, &mut out);
    assert_eq!(out, b"[1 /Item 10 0 R]");
    out.clear();

    let mut dict = BTreeMap::new();
    dict.insert("Type".into(), PdfObject::Name("Catalog".into()));
    dict.insert("Pages".into(), PdfObject::Reference(PdfObjectId::new(2, 0)));
    serialize_object(&PdfObject::Dictionary(dict), &mut out);
    assert!(out.starts_with(b"<<\n"));
    assert!(out.ends_with(b">>"));
    out.clear();
}

#[test]
fn test_04_serialize_stream_object() {
    let mut dict = BTreeMap::new();
    dict.insert("Type".into(), PdfObject::Name("XObject".into()));
    let content = b"RAW STREAM DATA BYTES".to_vec();

    let stream = PdfStream::new(dict, content);
    let mut out = Vec::new();
    serialize_object(&PdfObject::Stream(stream), &mut out);

    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("/Length 21"));
    assert!(text.contains("stream\r\nRAW STREAM DATA BYTES\r\nendstream"));
}

#[test]
fn test_05_incremental_writer_empty_returns_original() {
    let base = build_minimal_pdf();
    let writer = PdfIncrementalWriter::from_bytes(&base).unwrap();
    assert_eq!(writer.pending_count(), 0);

    let finished = writer.finish().unwrap();
    assert_eq!(finished, base);
}

#[test]
fn test_06_incremental_writer_add_new_objects() {
    let base = build_minimal_pdf();
    let mut writer = PdfIncrementalWriter::from_bytes(&base).unwrap();

    // Allocate and add a new metadata object (Obj 5 0)
    let mut meta = BTreeMap::new();
    meta.insert(
        "Title".into(),
        PdfObject::LiteralString(b"Incremental PDF".to_vec()),
    );
    meta.insert("Revision".into(), PdfObject::Integer(1));
    let meta_id = writer.add_object(PdfObject::Dictionary(meta));

    assert_eq!(meta_id, PdfObjectId::new(5, 0));

    let updated_bytes = writer.finish().unwrap();

    // 1. Ensure original bytes are preserved byte-for-byte at the start
    assert!(updated_bytes.starts_with(&base));

    // 2. Parse updated document
    let doc = PdfDocument::parse(updated_bytes).unwrap();
    assert_eq!(doc.page_count(), 1);

    // 3. Resolve newly added object
    let resolved_meta = doc.resolve_object(meta_id).unwrap();
    let d = resolved_meta.as_dict().unwrap();
    assert_eq!(d.get("Revision").unwrap(), &PdfObject::Integer(1));
    assert_eq!(
        d.get("Title").unwrap(),
        &PdfObject::LiteralString(b"Incremental PDF".to_vec())
    );
}

#[test]
fn test_07_incremental_writer_override_existing_object() {
    let base = build_minimal_pdf();
    let mut writer = PdfIncrementalWriter::from_bytes(&base).unwrap();

    // Override Page 3 with custom CropBox
    let mut updated_page = BTreeMap::new();
    updated_page.insert("Type".into(), PdfObject::Name("Page".into()));
    updated_page.insert(
        "Parent".into(),
        PdfObject::Reference(PdfObjectId::new(2, 0)),
    );
    updated_page.insert(
        "Contents".into(),
        PdfObject::Reference(PdfObjectId::new(4, 0)),
    );
    updated_page.insert(
        "CropBox".into(),
        PdfObject::Array(vec![
            PdfObject::Integer(10),
            PdfObject::Integer(10),
            PdfObject::Integer(500),
            PdfObject::Integer(700),
        ]),
    );

    writer.set_object(PdfObjectId::new(3, 0), PdfObject::Dictionary(updated_page));
    let updated_bytes = writer.finish().unwrap();

    let doc = PdfDocument::parse(updated_bytes).unwrap();
    let page = doc.page(0).unwrap();
    assert_eq!(page.crop_box, PdfRect::new(10.0, 10.0, 500.0, 700.0));
}

#[test]
fn test_08_incremental_writer_multi_updates_chaining() {
    let rev0 = build_minimal_pdf();

    // Rev 1: Add note object 5 0
    let mut w1 = PdfIncrementalWriter::from_bytes(&rev0).unwrap();
    let mut note = BTreeMap::new();
    note.insert("Note".into(), PdfObject::LiteralString(b"Note 1".to_vec()));
    let id5 = w1.add_object(PdfObject::Dictionary(note));
    let rev1 = w1.finish().unwrap();

    // Rev 2: Add note object 6 0
    let mut w2 = PdfIncrementalWriter::from_bytes(&rev1).unwrap();
    let mut note2 = BTreeMap::new();
    note2.insert("Note".into(), PdfObject::LiteralString(b"Note 2".to_vec()));
    let id6 = w2.add_object(PdfObject::Dictionary(note2));
    let rev2 = w2.finish().unwrap();

    // Verify rev 2 contains both additions
    let doc2 = PdfDocument::parse(rev2).unwrap();
    let obj5 = doc2.resolve_object(id5).unwrap();
    assert_eq!(
        obj5.as_dict().unwrap().get("Note").unwrap(),
        &PdfObject::LiteralString(b"Note 1".to_vec())
    );

    let obj6 = doc2.resolve_object(id6).unwrap();
    assert_eq!(
        obj6.as_dict().unwrap().get("Note").unwrap(),
        &PdfObject::LiteralString(b"Note 2".to_vec())
    );
}

#[test]
fn test_09_multi_revision_history_inspection() {
    let rev0 = build_minimal_pdf();

    let mut w1 = PdfIncrementalWriter::from_bytes(&rev0).unwrap();
    w1.add_object(PdfObject::Integer(100));
    let rev1 = w1.finish().unwrap();

    let mut w2 = PdfIncrementalWriter::from_bytes(&rev1).unwrap();
    w2.add_object(PdfObject::Integer(200));
    let rev2 = w2.finish().unwrap();

    let history = RevisionHistory::inspect(&rev2).unwrap();
    assert_eq!(history.count(), 3);

    let r0 = history.revision(0).unwrap();
    assert_eq!(r0.revision_index, 0);

    let r1 = history.revision(1).unwrap();
    assert_eq!(r1.revision_index, 1);
    assert_eq!(r1.modified_objects, vec![5]);

    let r2 = history.revision(2).unwrap();
    assert_eq!(r2.revision_index, 2);
    assert_eq!(r2.modified_objects, vec![6]);
}

#[test]
fn test_10_extract_historic_revision_bytes() {
    let rev0 = build_minimal_pdf();

    let mut w1 = PdfIncrementalWriter::from_bytes(&rev0).unwrap();
    w1.add_object(PdfObject::LiteralString(b"Update 1".to_vec()));
    let rev1 = w1.finish().unwrap();

    let history = RevisionHistory::inspect(&rev1).unwrap();
    assert_eq!(history.count(), 2);

    let extracted_rev0_bytes = history.extract_revision_bytes(0, &rev1).unwrap();
    assert_eq!(extracted_rev0_bytes, rev0.as_slice());

    // Ensure extracted snapshot parses cleanly on its own
    let parsed_rev0 = PdfDocument::parse(extracted_rev0_bytes.to_vec()).unwrap();
    assert_eq!(parsed_rev0.page_count(), 1);
}

#[test]
fn test_11_digital_signature_byterange_extraction() {
    // Construct a PDF document with an AcroForm signature field
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

    let sig_bytes = vec![0xAB, 0xCD, 0xEF, 0x12];
    let off_sig = doc_bytes.len();
    doc_bytes.extend_from_slice(
        format!(
            "6 0 obj\n<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /adbe.pkcs7.detached /ByteRange [ 0 100 200 150 ] /Contents <{:02X}{:02X}{:02X}{:02X}> /Reason (Approval) /M (D:20261029140000Z) >>\nendobj\n",
            sig_bytes[0], sig_bytes[1], sig_bytes[2], sig_bytes[3]
        )
        .as_bytes(),
    );

    let off_field = doc_bytes.len();
    doc_bytes.extend_from_slice(b"5 0 obj\n<< /FT /Sig /T (Signature1) /V 6 0 R >>\nendobj\n");

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
        off_cat, off_pages, off_page, off_field, off_sig, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes.clone()).unwrap();
    let signatures = SignatureValidator::extract_signatures(&doc_bytes, &doc).unwrap();

    assert_eq!(signatures.len(), 1);
    let sig = &signatures[0];
    assert_eq!(sig.field_name, "Signature1");
    assert_eq!(sig.filter, "Adobe.PPKLite");
    assert_eq!(sig.sub_filter, Some("adbe.pkcs7.detached".into()));
    assert_eq!(sig.byte_range, [0, 100, 200, 150]);
    assert_eq!(sig.reason, Some("Approval".into()));
    assert_eq!(sig.contents_bytes, sig_bytes);
    assert!(sig.is_byte_range_valid);
}

#[test]
fn test_12_digital_signature_byterange_hash_validation() {
    let mut data = vec![0u8; 500];
    // Fill specific bytes in range 1: [0..100]
    for (i, item) in data.iter_mut().enumerate().take(100) {
        *item = (i % 256) as u8;
    }
    // Fill specific bytes in range 2: [200..350]
    for (i, item) in data.iter_mut().enumerate().take(350).skip(200) {
        *item = ((i * 3) % 256) as u8;
    }

    // Compute expected hash manually
    let mut expected_hasher = Sha256::new();
    expected_hasher.update(&data[0..100]);
    expected_hasher.update(&data[200..350]);
    let expected_hash = expected_hasher.finalize();

    let mut dict = BTreeMap::new();
    dict.insert("Filter".into(), PdfObject::Name("Adobe.PPKLite".into()));
    dict.insert(
        "ByteRange".into(),
        PdfObject::Array(vec![
            PdfObject::Integer(0),
            PdfObject::Integer(100),
            PdfObject::Integer(200),
            PdfObject::Integer(150),
        ]),
    );

    let parsed_sig = SignatureValidator::parse_signature_dict(&data, "SigTest", &dict).unwrap();
    assert_eq!(parsed_sig.computed_sha256, expected_hash.as_slice());
}

#[test]
fn test_13_digital_signature_preservation_across_incremental_update() {
    let signed_doc = build_minimal_pdf();
    let signed_len = signed_doc.len();

    // Suppose signed doc covers [0, 50, 100, signed_len - 100]
    let signed_range = [0, 50, 100, signed_len - 100];
    assert!(SignatureValidator::verify_range_intact(
        &signed_doc,
        &signed_range
    ));

    // Append an incremental update to the document
    let mut writer = PdfIncrementalWriter::from_bytes(&signed_doc).unwrap();
    writer.add_object(PdfObject::Integer(9999));
    let updated_doc = writer.finish().unwrap();

    // Verify that the original signed range is completely intact in the updated file!
    assert!(SignatureValidator::verify_range_intact(
        &updated_doc,
        &signed_range
    ));
    assert_eq!(&updated_doc[..signed_len], signed_doc.as_slice());
}

#[test]
fn test_14_tampered_byte_detection() {
    let mut data = vec![0u8; 400];

    let mut dict = BTreeMap::new();
    dict.insert(
        "ByteRange".into(),
        PdfObject::Array(vec![
            PdfObject::Integer(0),
            PdfObject::Integer(100),
            PdfObject::Integer(150),
            PdfObject::Integer(100),
        ]),
    );

    let sig_original = SignatureValidator::parse_signature_dict(&data, "Sig", &dict).unwrap();

    // Tamper with a single byte inside range 1: data[50]
    data[50] = 0xFF;
    let sig_tampered = SignatureValidator::parse_signature_dict(&data, "Sig", &dict).unwrap();

    assert_ne!(
        sig_original.computed_sha256, sig_tampered.computed_sha256,
        "Tampered byte must produce a different cryptographic hash"
    );
}

#[test]
fn test_15_performance_incremental_write_sla() {
    let base = build_minimal_pdf();

    let start = Instant::now();
    let mut writer = PdfIncrementalWriter::from_bytes(&base).unwrap();
    writer.add_object(PdfObject::Integer(123456));
    let result = writer.finish().unwrap();
    let elapsed = start.elapsed();

    assert!(result.len() > base.len());
    // SLA: Incremental write must complete in < 1.0 ms
    assert!(
        elapsed.as_micros() < 1000,
        "Incremental write took {:?}, must be < 1.0 ms",
        elapsed
    );
}
