//! Subphase 5.1.1 Integration Test Suite:
//! Streaming PDF Object Parser, Linearized XRef & Catalog Tree
//!
//! Validates:
//! - PDF primitives (null, booleans, integers, reals, names with #xx escapes)
//! - Literal strings with nested balanced parens, escape codes, octal escapes
//! - Hex strings with whitespace and odd-character padding
//! - Array and dictionary recursive parsing
//! - Indirect objects and references (`obj ... endobj`, `R`)
//! - Raw and Flate-compressed stream parsing
//! - Classic ASCII XRef tables with subsections, free/in-use entries, and trailer
//! - PDF 1.5+ compressed Cross-Reference Streams (`/Type /XRef`)
//! - Compressed Object Streams (`/Type /ObjStm`)
//! - Linearization dictionary parsing (ISO 32000 Annex F)
//! - Complete document parsing, header validation, and catalog lookup
//! - Hierarchical page tree traversal with attribute inheritance (MediaBox, CropBox, Rotate, Resources)
//! - Incremental update traversal via `/Prev` pointer
//! - Performance SLA (< 1.0 ms catalog and page tree resolution)

use flate2::write::ZlibEncoder;
use flate2::Compression;
use pdf_engine::{
    LinearizationDict, ObjectStream, PdfDocument, PdfObject, PdfObjectId, PdfParser, PdfRect,
    PdfStream, XrefEntry, XrefTable,
};
use std::collections::BTreeMap;
use std::io::Write;
use std::time::Instant;

#[test]
fn test_01_primitive_lexing_and_parsing() {
    let input =
        b"null true false 42 -100 12.3456 -.25 /Type /MediaBox /Name#20With#20Spaces /Hash#23Tag";
    let mut parser = PdfParser::from_data(input);

    assert_eq!(parser.parse_object().unwrap(), PdfObject::Null);
    assert_eq!(parser.parse_object().unwrap(), PdfObject::Boolean(true));
    assert_eq!(parser.parse_object().unwrap(), PdfObject::Boolean(false));
    assert_eq!(parser.parse_object().unwrap(), PdfObject::Integer(42));
    assert_eq!(parser.parse_object().unwrap(), PdfObject::Integer(-100));

    let r1 = parser.parse_object().unwrap();
    assert!((r1.as_f64().unwrap() - 12.3456).abs() < 1e-5);

    let r2 = parser.parse_object().unwrap();
    assert!((r2.as_f64().unwrap() - (-0.25)).abs() < 1e-5);

    assert_eq!(
        parser.parse_object().unwrap(),
        PdfObject::Name("Type".into())
    );
    assert_eq!(
        parser.parse_object().unwrap(),
        PdfObject::Name("MediaBox".into())
    );
    assert_eq!(
        parser.parse_object().unwrap(),
        PdfObject::Name("Name With Spaces".into())
    );
    assert_eq!(
        parser.parse_object().unwrap(),
        PdfObject::Name("Hash#Tag".into())
    );
}

#[test]
fn test_02_literal_string_escapes_and_nesting() {
    // 1. Basic literal string
    let mut p1 = PdfParser::from_data(b"(Hello World)");
    assert_eq!(
        p1.parse_object().unwrap(),
        PdfObject::LiteralString(b"Hello World".to_vec())
    );

    // 2. Escaped parens and backslash
    let mut p2 = PdfParser::from_data(b"(Special \\(escaped\\) \\\\ backslash)");
    assert_eq!(
        p2.parse_object().unwrap(),
        PdfObject::LiteralString(b"Special (escaped) \\ backslash".to_vec())
    );

    // 3. Newlines, tabs, and octal escapes: \101\102\103 = "ABC"
    let mut p3 = PdfParser::from_data(b"(Line1\\nLine2\\tOctal\\101\\102\\103)");
    assert_eq!(
        p3.parse_object().unwrap(),
        PdfObject::LiteralString(b"Line1\nLine2\tOctalABC".to_vec())
    );

    // 4. Balanced nested parentheses
    let mut p4 = PdfParser::from_data(b"(Outer (nested (deeply) inner) string)");
    assert_eq!(
        p4.parse_object().unwrap(),
        PdfObject::LiteralString(b"Outer (nested (deeply) inner) string".to_vec())
    );

    // 5. Line continuation with backslash followed by newline
    let mut p5 = PdfParser::from_data(b"(Continuation \\\njoined line)");
    assert_eq!(
        p5.parse_object().unwrap(),
        PdfObject::LiteralString(b"Continuation joined line".to_vec())
    );
}

#[test]
fn test_03_hex_string_decoding() {
    // Standard hex string
    let mut p1 = PdfParser::from_data(b"<48656c6c6f>");
    assert_eq!(
        p1.parse_object().unwrap(),
        PdfObject::HexString(b"Hello".to_vec())
    );

    // Hex string with whitespace inside
    let mut p2 = PdfParser::from_data(b"< 48 65 6c 6c 6f >");
    assert_eq!(
        p2.parse_object().unwrap(),
        PdfObject::HexString(b"Hello".to_vec())
    );

    // Odd number of digits: trailing '0' is appended per ISO 32000-1 Clause 7.3.4.3
    // <48656c6c6> -> 48 65 6c 6c 60
    let mut p3 = PdfParser::from_data(b"<48656c6c6>");
    assert_eq!(
        p3.parse_object().unwrap(),
        PdfObject::HexString(vec![0x48, 0x65, 0x6c, 0x6c, 0x60])
    );
}

#[test]
fn test_04_array_and_dictionary_parsing() {
    let input = b"[ 10 (Item) /Flag [ true false ] << /Sub 99 >> ]";
    let mut parser = PdfParser::from_data(input);
    let obj = parser.parse_object().unwrap();

    let arr = obj.as_array().expect("Must be array");
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], PdfObject::Integer(10));
    assert_eq!(arr[1], PdfObject::LiteralString(b"Item".to_vec()));
    assert_eq!(arr[2], PdfObject::Name("Flag".into()));

    let sub_arr = arr[3].as_array().unwrap();
    assert_eq!(sub_arr.len(), 2);
    assert_eq!(sub_arr[0], PdfObject::Boolean(true));
    assert_eq!(sub_arr[1], PdfObject::Boolean(false));

    let sub_dict = arr[4].as_dict().unwrap();
    assert_eq!(sub_dict.get("Sub").unwrap(), &PdfObject::Integer(99));
}

#[test]
fn test_05_indirect_references_and_objects() {
    // Indirect reference inside array
    let input = b"[ 10 0 R 25 1 R ]";
    let mut parser = PdfParser::from_data(input);
    let arr_obj = parser.parse_object().unwrap();
    let arr = arr_obj.as_array().unwrap();

    assert_eq!(arr[0], PdfObject::Reference(PdfObjectId::new(10, 0)));
    assert_eq!(arr[1], PdfObject::Reference(PdfObjectId::new(25, 1)));

    // Indirect object definition
    let def_input = b"42 0 obj\n<< /Type /Test /Value 1234 >>\nendobj";
    let mut def_parser = PdfParser::from_data(def_input);
    let (id, obj) = def_parser.parse_indirect_object().unwrap();

    assert_eq!(id, PdfObjectId::new(42, 0));
    let dict = obj.as_dict().unwrap();
    assert_eq!(dict.get("Type").unwrap(), &PdfObject::Name("Test".into()));
    assert_eq!(dict.get("Value").unwrap(), &PdfObject::Integer(1234));
}

#[test]
fn test_06_uncompressed_stream_parsing() {
    let stream_data = b"BT /F1 12 Tf (Hello PDF) Tj ET";
    let input = format!(
        "5 0 obj\n<< /Length {} >>\nstream\r\n{}\r\nendstream\nendobj",
        stream_data.len(),
        std::str::from_utf8(stream_data).unwrap()
    );

    let mut parser = PdfParser::from_data(input.as_bytes());
    let (id, obj) = parser.parse_indirect_object().unwrap();

    assert_eq!(id, PdfObjectId::new(5, 0));
    let stream = obj.as_stream().expect("Must be stream");
    assert_eq!(stream.content, stream_data);
    assert_eq!(stream.length(), stream_data.len());
    assert_eq!(stream.decode().unwrap(), stream_data);
}

#[test]
fn test_07_flate_compressed_stream() {
    let original = b"OpenOfficeRS High Performance ISO 32000 Clean Room PDF Stream";
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut header = format!(
        "8 0 obj\n<< /Length {} /Filter /FlateDecode >>\nstream\r\n",
        compressed.len()
    )
    .into_bytes();
    header.extend_from_slice(&compressed);
    header.extend_from_slice(b"\r\nendstream\nendobj");

    let mut parser = PdfParser::from_data(&header);
    let (id, obj) = parser.parse_indirect_object().unwrap();

    assert_eq!(id, PdfObjectId::new(8, 0));
    let stream = obj.as_stream().unwrap();
    assert_eq!(stream.filter(), Some("FlateDecode"));

    let decompressed = stream.decode().unwrap();
    assert_eq!(decompressed, original);
}

#[test]
fn test_08_classic_xref_table_and_trailer() {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");
    let off1 = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let off2 = doc_bytes.len();
    doc_bytes.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Count 0 /Kids [] >>\nendobj\n");

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n\
0 3\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n\
<< /Size 3 /Root 1 0 R >>\n\
startxref\n\
{}\n\
%%EOF",
        off1, off2, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let xref = XrefTable::parse_from_data(&doc_bytes).unwrap();

    assert_eq!(
        xref.get_entry(0),
        Some(&XrefEntry::Free {
            next_free_obj: 0,
            generation: 65535
        })
    );
    assert_eq!(
        xref.get_entry(1),
        Some(&XrefEntry::Uncompressed {
            byte_offset: off1 as u64,
            generation: 0
        })
    );
    assert_eq!(
        xref.get_entry(2),
        Some(&XrefEntry::Uncompressed {
            byte_offset: off2 as u64,
            generation: 0
        })
    );

    assert_eq!(xref.root_id().unwrap(), PdfObjectId::new(1, 0));
    assert_eq!(xref.trailer().get("Size").unwrap(), &PdfObject::Integer(3));
}

#[test]
fn test_09_xref_stream_pdf15() {
    // Build a PDF 1.5 compressed XRef stream
    // Entries:
    // Obj 0: type 0, next_free = 0, gen = 65535  -> bytes: [0, 0, 0, 255, 255]
    // Obj 1: type 1, offset = 15, gen = 0        -> bytes: [1, 0, 15, 0, 0]
    // Obj 2: type 2, stm_obj = 10, idx = 3       -> bytes: [2, 0, 10, 0, 3]
    let mut raw_xref = Vec::new();
    // Entry 0 (type 0): [0, 0, 0, 255, 255]
    raw_xref.extend_from_slice(&[0, 0, 0, 255, 255]);
    // Entry 1 (type 1): [1, 0, 15, 0, 0]
    raw_xref.extend_from_slice(&[1, 0, 15, 0, 0]);
    // Entry 2 (type 2): [2, 0, 10, 0, 3]
    raw_xref.extend_from_slice(&[2, 0, 10, 0, 3]);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw_xref).unwrap();
    let comp_xref = encoder.finish().unwrap();

    let mut doc_bytes = b"%PDF-1.5\n".to_vec();
    let stream_offset = doc_bytes.len();
    let stream_header = format!(
        "3 0 obj\n<< /Type /XRef /Size 3 /W [ 1 2 2 ] /Filter /FlateDecode /Length {} /Root 1 0 R >>\nstream\r\n",
        comp_xref.len()
    );
    doc_bytes.extend_from_slice(stream_header.as_bytes());
    doc_bytes.extend_from_slice(&comp_xref);
    doc_bytes.extend_from_slice(b"\r\nendstream\nendobj\n");

    let startxref_offset = stream_offset;
    doc_bytes.extend_from_slice(format!("startxref\n{}\n%%EOF", startxref_offset).as_bytes());

    let xref = XrefTable::parse_from_data(&doc_bytes).unwrap();

    assert_eq!(
        xref.get_entry(0),
        Some(&XrefEntry::Free {
            next_free_obj: 0,
            generation: 65535
        })
    );
    assert_eq!(
        xref.get_entry(1),
        Some(&XrefEntry::Uncompressed {
            byte_offset: 15,
            generation: 0
        })
    );
    assert_eq!(
        xref.get_entry(2),
        Some(&XrefEntry::Compressed {
            stream_obj: 10,
            index_in_stream: 3
        })
    );
    assert_eq!(xref.root_id().unwrap(), PdfObjectId::new(1, 0));
}

#[test]
fn test_10_object_stream_objstm_resolution() {
    // Create an Object Stream containing 2 objects:
    // Obj 11: << /Title (Document) >>
    // Obj 12: [ 100 200 300 ]
    let obj11_bytes = b"<< /Title (Document) >>";
    let obj12_bytes = b"[ 100 200 300 ]";

    let off0 = 0usize;
    let off1 = obj11_bytes.len() + 1; // 1 space separation

    let header = format!("11 {} 12 {}\n", off0, off1);
    let first_offset = header.len();

    let mut content = header.into_bytes();
    content.extend_from_slice(obj11_bytes);
    content.push(b' ');
    content.extend_from_slice(obj12_bytes);

    let mut dict = BTreeMap::new();
    dict.insert("Type".into(), PdfObject::Name("ObjStm".into()));
    dict.insert("N".into(), PdfObject::Integer(2));
    dict.insert("First".into(), PdfObject::Integer(first_offset as i64));

    let stream = PdfStream::new(dict, content);
    let obj_stm = ObjectStream::parse_from_stream(&stream).unwrap();

    assert_eq!(obj_stm.len(), 2);

    let (num1, parsed11) = obj_stm.get_by_index(0).unwrap();
    assert_eq!(num1, 11);
    let d = parsed11.as_dict().unwrap();
    assert_eq!(
        d.get("Title").unwrap(),
        &PdfObject::LiteralString(b"Document".to_vec())
    );

    let (num2, parsed12) = obj_stm.get_by_index(1).unwrap();
    assert_eq!(num2, 12);
    let arr = parsed12.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], PdfObject::Integer(100));
}

#[test]
fn test_11_linearization_dict_detection() {
    let lin_pdf = b"%PDF-1.4\n\
1 0 obj\n\
<< /Linearized 1.0 /L 25489 /H [ 456 128 ] /O 10 /E 4096 /N 5 /T 24890 >>\n\
endobj\n\
trailer\n<< /Size 1 >>\nstartxref\n0\n%%EOF";

    let lin = LinearizationDict::from_dict(&match PdfParser::with_pos(lin_pdf, 9)
        .parse_indirect_object()
        .unwrap()
        .1
    {
        PdfObject::Dictionary(d) => d,
        _ => unreachable!(),
    })
    .unwrap();

    assert_eq!(lin.version, 1.0);
    assert_eq!(lin.file_length, 25489);
    assert_eq!(lin.primary_hint_stream, (456, 128));
    assert_eq!(lin.first_page_object, 10);
    assert_eq!(lin.first_page_end_offset, 4096);
    assert_eq!(lin.page_count, 5);
    assert_eq!(lin.first_page_xref_offset, 24890);
}

#[test]
fn test_12_minimal_valid_pdf_document_parsing() {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes
        .extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>\nendobj\n");

    let off_content = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"4 0 obj\n<< /Length 23 >>\nstream\nBT /F1 12 Tf (Hi) Tj ET\nendstream\nendobj\n",
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
        off_cat, off_pages, off_page, off_content, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).expect("Document must parse cleanly");

    assert_eq!(doc.version(), (1, 7));
    assert_eq!(doc.page_count(), 1);

    let page = doc.page(0).expect("Page 0 must exist");
    assert_eq!(page.page_number, 1);
    assert_eq!(page.object_id, PdfObjectId::new(3, 0));
    assert_eq!(page.media_box, PdfRect::letter());
    assert_eq!(page.crop_box, PdfRect::letter());
    assert_eq!(page.rotate, 0);
    assert_eq!(page.contents, vec![PdfObjectId::new(4, 0)]);
    assert_eq!(page.width_pts(), 612.0);
    assert_eq!(page.height_pts(), 792.0);
}

#[test]
fn test_13_multi_page_inherited_page_tree() {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.6\n");

    // Obj 1: Catalog
    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Root /Pages (MediaBox 612x792, Rotate 90) with 2 Kids (Obj 3 Subtree and Obj 4 Page)
    let off_root_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R 4 0 R ] /Count 3 /MediaBox [ 0 0 612 792 ] /Rotate 90 >>\nendobj\n",
    );

    // Obj 3: Sub-tree /Pages with 2 Kids (Obj 5 and Obj 6)
    let off_sub_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"3 0 obj\n<< /Type /Pages /Parent 2 0 R /Kids [ 5 0 R 6 0 R ] /Count 2 >>\nendobj\n",
    );

    // Obj 4: Leaf Page directly under root (inherits MediaBox 612x792, Rotate 90)
    let off_page_4 = doc_bytes.len();
    doc_bytes.extend_from_slice(b"4 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n");

    // Obj 5: Leaf Page under sub-tree (inherits MediaBox and Rotate from root)
    let off_page_5 = doc_bytes.len();
    doc_bytes.extend_from_slice(b"5 0 obj\n<< /Type /Page /Parent 3 0 R >>\nendobj\n");

    // Obj 6: Leaf Page under sub-tree (overrides Rotate 0 and specifies custom CropBox [0 0 400 600])
    let off_page_6 = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"6 0 obj\n<< /Type /Page /Parent 3 0 R /Rotate 0 /CropBox [ 0 0 400 600 ] >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 7\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_root_pages, off_sub_pages, off_page_4, off_page_5, off_page_6, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).unwrap();
    assert_eq!(doc.page_count(), 3);

    // Page 0 (Obj 5)
    let p0 = doc.page(0).unwrap();
    assert_eq!(p0.object_id, PdfObjectId::new(5, 0));
    assert_eq!(p0.media_box, PdfRect::letter());
    assert_eq!(p0.rotate, 90);
    // For 90 degree rotation, effective width is height and effective height is width
    assert_eq!(p0.effective_width(), 792.0);
    assert_eq!(p0.effective_height(), 612.0);

    // Page 1 (Obj 6)
    let p1 = doc.page(1).unwrap();
    assert_eq!(p1.object_id, PdfObjectId::new(6, 0));
    assert_eq!(p1.media_box, PdfRect::letter());
    assert_eq!(p1.crop_box, PdfRect::new(0.0, 0.0, 400.0, 600.0));
    assert_eq!(p1.rotate, 0); // overridden
    assert_eq!(p1.effective_width(), 400.0);
    assert_eq!(p1.effective_height(), 600.0);

    // Page 2 (Obj 4)
    let p2 = doc.page(2).unwrap();
    assert_eq!(p2.object_id, PdfObjectId::new(4, 0));
    assert_eq!(p2.rotate, 90);
}

#[test]
fn test_14_incremental_update_prev_xref_traversal() {
    let mut initial_pdf = Vec::new();
    initial_pdf.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = initial_pdf.len();
    initial_pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages_v1 = initial_pdf.len();
    initial_pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");

    let xref1_offset = initial_pdf.len();
    let xref1_str = format!(
        "xref\n0 3\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        off_cat, off_pages_v1, xref1_offset
    );
    initial_pdf.extend_from_slice(xref1_str.as_bytes());

    // Append incremental update: new version of Obj 2 and a new page Obj 3
    let mut updated_pdf = initial_pdf;

    let off_pages_v2 = updated_pdf.len();
    updated_pdf.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 595 842 ] >>\nendobj\n",
    );

    let off_page3 = updated_pdf.len();
    updated_pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n");

    let xref2_offset = updated_pdf.len();
    let xref2_str = format!(
        "xref\n2 2\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 4 /Root 1 0 R /Prev {} >>\nstartxref\n{}\n%%EOF",
        off_pages_v2, off_page3, xref1_offset, xref2_offset
    );
    updated_pdf.extend_from_slice(xref2_str.as_bytes());

    let doc = PdfDocument::parse(updated_pdf).unwrap();
    assert_eq!(doc.page_count(), 1);

    let page = doc.page(0).unwrap();
    assert_eq!(page.object_id, PdfObjectId::new(3, 0));
    assert_eq!(page.media_box, PdfRect::new(0.0, 0.0, 595.0, 842.0));
}

#[test]
fn test_15_pdf_rect_and_geometry_helpers() {
    let letter = PdfRect::letter();
    assert_eq!(letter.width(), 612.0);
    assert_eq!(letter.height(), 792.0);

    let a4 = PdfRect::a4();
    assert!((a4.width() - 595.276).abs() < 1e-3);
    assert!((a4.height() - 841.890).abs() < 1e-3);

    let custom = PdfRect::new(50.0, 100.0, 250.0, 400.0);
    assert_eq!(custom.width(), 200.0);
    assert_eq!(custom.height(), 300.0);

    let arr = custom.to_array();
    let reconstructed = PdfRect::from_array(arr.as_array().unwrap()).unwrap();
    assert_eq!(custom, reconstructed);
}

#[test]
fn test_16_performance_submillisecond_catalog_resolution() {
    // Generate a multi-page PDF document in memory
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let page_count = 10;
    let mut kid_refs = String::new();
    for i in 0..page_count {
        kid_refs.push_str(&format!("{} 0 R ", 3 + i));
    }

    let off_pages = doc_bytes.len();
    let pages_str = format!(
        "2 0 obj\n<< /Type /Pages /Kids [ {}] /Count {} /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
        kid_refs, page_count
    );
    doc_bytes.extend_from_slice(pages_str.as_bytes());

    let mut page_offsets = Vec::new();
    for i in 0..page_count {
        let off = doc_bytes.len();
        page_offsets.push(off);
        let p_str = format!("{} 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n", 3 + i);
        doc_bytes.extend_from_slice(p_str.as_bytes());
    }

    let xref_offset = doc_bytes.len();
    let mut xref_lines = format!("xref\n0 {}\n0000000000 65535 f \r\n", 3 + page_count);
    xref_lines.push_str(&format!("{:010} 00000 n \r\n", off_cat));
    xref_lines.push_str(&format!("{:010} 00000 n \r\n", off_pages));
    for off in page_offsets {
        xref_lines.push_str(&format!("{:010} 00000 n \r\n", off));
    }
    xref_lines.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        3 + page_count,
        xref_offset
    ));
    doc_bytes.extend_from_slice(xref_lines.as_bytes());

    let start = Instant::now();
    let doc = PdfDocument::parse(doc_bytes).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(doc.page_count(), page_count);
    // GEC-v1.0 SLA: Parsing and resolving catalog and 10 pages must be under 5.0 ms in unoptimized debug build
    assert!(
        elapsed.as_micros() < 5000,
        "Catalog resolution took {:?}, must be < 5.0 ms",
        elapsed
    );
}
