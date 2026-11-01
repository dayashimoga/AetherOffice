//! Sub-Phase 49.1.2 Integration Tests:
//! Page Split, Merge, Extract, Bates Numbering & Optical Character Recognition (OCR)
//!
//! Validates:
//! - Page extraction with object tree rebasing and orphan elimination
//! - Document splitting by arbitrary boundary indices and individual page explosion
//! - Multi-document merging with collision-free object ID rebasing
//! - Preservation of disparate page dimensions (MediaBox) across merged documents
//! - Sequential Bates numbering overlays (prefix, padding, position, margins)
//! - ISO 32000-1 §9.3.5 Invisible OCR text layer (`3 Tr`) injection with bounding box placement
//! - Full round-trip searchable text extraction from OCR-injected documents

use pdf_engine::page_ops::{
    apply_bates_numbering, extract_pages, inject_ocr_layer, merge_documents, split_document,
    BatesNumberingConfig, BatesPosition, OcrBoundingBox, OcrPageData, OcrWord,
};
use pdf_engine::PdfDocument;

fn create_multipage_pdf(pages: &[&str], media_boxes: Option<&[(f64, f64)]>) -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let num_pages = pages.len();
    // Catalog (1), Pages (2), plus for each page: Page obj, Content obj, Font obj (for text extraction)
    let total_objs = 2 + num_pages * 3;
    let mut offsets = vec![0usize; total_objs + 1];

    // Catalog (1 0 obj)
    offsets[1] = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Kids array for Pages dict
    let mut kids_str = String::from("[ ");
    for i in 0..num_pages {
        let page_id = 3 + i * 3;
        kids_str.push_str(&format!("{page_id} 0 R "));
    }
    kids_str.push(']');

    // Pages (2 0 obj)
    offsets[2] = doc_bytes.len();
    let pages_obj = format!(
        "2 0 obj\n<< /Type /Pages /Kids {} /Count {} /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
        kids_str, num_pages
    );
    doc_bytes.extend_from_slice(pages_obj.as_bytes());

    // Pages, Content streams & Fonts
    for (i, content_text) in pages.iter().enumerate() {
        let page_id = 3 + i * 3;
        let content_id = 4 + i * 3;
        let font_id = 5 + i * 3;

        let (w, h) = if let Some(boxes) = media_boxes {
            if i < boxes.len() {
                boxes[i]
            } else {
                (612.0, 792.0)
            }
        } else {
            (612.0, 792.0)
        };

        // Font obj
        offsets[font_id] = doc_bytes.len();
        let font_obj = format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
            font_id
        );
        doc_bytes.extend_from_slice(font_obj.as_bytes());

        // Page obj
        offsets[page_id] = doc_bytes.len();
        let page_dict = format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> /MediaBox [ 0 0 {} {} ] >>\nendobj\n",
            page_id, content_id, font_id, w, h
        );
        doc_bytes.extend_from_slice(page_dict.as_bytes());

        // Content stream
        let stream_body = format!("BT /F1 12 Tf 50 700 Td ({}) Tj ET\n", content_text);
        offsets[content_id] = doc_bytes.len();
        let stream_obj = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content_id,
            stream_body.len(),
            stream_body
        );
        doc_bytes.extend_from_slice(stream_obj.as_bytes());
    }

    // XRef & Trailer
    let xref_offset = doc_bytes.len();
    let num_entries = total_objs + 1;
    let mut xref_str = format!("xref\n0 {num_entries}\n0000000000 65535 f \r\n");
    for off in &offsets[1..=total_objs] {
        xref_str.push_str(&format!("{:010} 00000 n \r\n", off));
    }
    xref_str.push_str(&format!(
        "trailer\n<< /Size {num_entries} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF"
    ));
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    doc_bytes
}

#[test]
fn test_01_extract_single_page() {
    let pdf = create_multipage_pdf(
        &["Page 1 Content", "Page 2 Content", "Page 3 Content"],
        None,
    );
    let extracted = extract_pages(&pdf, &[1]).expect("Page extraction should succeed");

    let doc = PdfDocument::parse(extracted).expect("Extracted PDF must be valid");
    assert_eq!(doc.page_count(), 1);
    let page = doc.page(0).expect("Page exists");
    let page_text = page
        .extract_plain_text(&doc)
        .expect("Text extraction must succeed");
    assert!(page_text.contains("Page 2 Content"));
    assert!(!page_text.contains("Page 1 Content"));
}

#[test]
fn test_02_extract_multiple_noncontiguous_pages() {
    let pdf = create_multipage_pdf(&["Alpha", "Bravo", "Charlie", "Delta"], None);
    let extracted = extract_pages(&pdf, &[0, 2]).expect("Extraction of [0, 2] should succeed");

    let doc = PdfDocument::parse(extracted).expect("Parsed extracted doc");
    assert_eq!(doc.page_count(), 2);
    let p0 = doc.page(0).unwrap().extract_plain_text(&doc).unwrap();
    let p1 = doc.page(1).unwrap().extract_plain_text(&doc).unwrap();
    assert!(p0.contains("Alpha"));
    assert!(p1.contains("Charlie"));
    assert!(!p0.contains("Bravo"));
    assert!(!p1.contains("Delta"));
}

#[test]
fn test_03_split_document_by_boundary() {
    let pdf = create_multipage_pdf(
        &["Doc Page 1", "Doc Page 2", "Doc Page 3", "Doc Page 4"],
        None,
    );
    // Boundary [2] splits into: pages [0, 1] and pages [2, 3]
    let parts = split_document(&pdf, &[2]).expect("Split by boundary should succeed");
    assert_eq!(parts.len(), 2);

    let doc1 = PdfDocument::parse(parts[0].clone()).expect("Doc 1 must parse");
    let doc2 = PdfDocument::parse(parts[1].clone()).expect("Doc 2 must parse");

    assert_eq!(doc1.page_count(), 2);
    assert_eq!(doc2.page_count(), 2);

    assert!(doc1
        .page(0)
        .unwrap()
        .extract_plain_text(&doc1)
        .unwrap()
        .contains("Doc Page 1"));
    assert!(doc1
        .page(1)
        .unwrap()
        .extract_plain_text(&doc1)
        .unwrap()
        .contains("Doc Page 2"));
    assert!(doc2
        .page(0)
        .unwrap()
        .extract_plain_text(&doc2)
        .unwrap()
        .contains("Doc Page 3"));
    assert!(doc2
        .page(1)
        .unwrap()
        .extract_plain_text(&doc2)
        .unwrap()
        .contains("Doc Page 4"));
}

#[test]
fn test_04_split_document_into_single_pages() {
    let pdf = create_multipage_pdf(&["Slice A", "Slice B", "Slice C"], None);
    // Empty boundary splits into single pages
    let singles = split_document(&pdf, &[]).expect("Single-page explosion");
    assert_eq!(singles.len(), 3);

    for (i, expected) in ["Slice A", "Slice B", "Slice C"].iter().enumerate() {
        let single_doc = PdfDocument::parse(singles[i].clone()).expect("Single doc parse");
        assert_eq!(single_doc.page_count(), 1);
        assert!(single_doc
            .page(0)
            .unwrap()
            .extract_plain_text(&single_doc)
            .unwrap()
            .contains(expected));
    }
}

#[test]
fn test_05_merge_two_documents() {
    let doc1_bytes = create_multipage_pdf(&["Doc 1 Page 1", "Doc 1 Page 2"], None);
    let doc2_bytes = create_multipage_pdf(&["Doc 2 Page 1", "Doc 2 Page 2"], None);

    let merged_bytes = merge_documents(&[&doc1_bytes, &doc2_bytes]).expect("Merge must succeed");
    let merged_doc = PdfDocument::parse(merged_bytes).expect("Merged PDF must parse");

    assert_eq!(merged_doc.page_count(), 4);
    assert!(merged_doc
        .page(0)
        .unwrap()
        .extract_plain_text(&merged_doc)
        .unwrap()
        .contains("Doc 1 Page 1"));
    assert!(merged_doc
        .page(1)
        .unwrap()
        .extract_plain_text(&merged_doc)
        .unwrap()
        .contains("Doc 1 Page 2"));
    assert!(merged_doc
        .page(2)
        .unwrap()
        .extract_plain_text(&merged_doc)
        .unwrap()
        .contains("Doc 2 Page 1"));
    assert!(merged_doc
        .page(3)
        .unwrap()
        .extract_plain_text(&merged_doc)
        .unwrap()
        .contains("Doc 2 Page 2"));
}

#[test]
fn test_06_merge_preserves_page_sizes() {
    // Doc 1 has Letter (612 x 792)
    let doc1_bytes = create_multipage_pdf(&["Letter Page"], Some(&[(612.0, 792.0)]));
    // Doc 2 has A4 (595.28 x 841.89)
    let doc2_bytes = create_multipage_pdf(&["A4 Page"], Some(&[(595.28, 841.89)]));

    let merged_bytes = merge_documents(&[&doc1_bytes, &doc2_bytes]).expect("Merge");
    let merged = PdfDocument::parse(merged_bytes).expect("Parsed merged");

    assert_eq!(merged.page_count(), 2);
    let p0 = merged.page(0).unwrap();
    let p1 = merged.page(1).unwrap();

    assert!((p0.width_pts() - 612.0).abs() < 0.1);
    assert!((p0.height_pts() - 792.0).abs() < 0.1);
    assert!((p1.width_pts() - 595.28).abs() < 0.1);
    assert!((p1.height_pts() - 841.89).abs() < 0.1);
}

#[test]
fn test_07_apply_bates_numbering_bottom_right() {
    let pdf = create_multipage_pdf(&["Page One", "Page Two"], None);
    let config = BatesNumberingConfig {
        prefix: "CASE-".to_string(),
        suffix: "-SEC".to_string(),
        start_number: 1,
        num_digits: 6,
        position: BatesPosition::BottomRight,
        font_size: 10.0,
        color_rgb: [0.0, 0.0, 0.0],
        margin_pt: 36.0,
    };

    let stamped = apply_bates_numbering(&pdf, &config).expect("Bates stamping");
    let doc = PdfDocument::parse(stamped).expect("Parsed stamped doc");

    assert_eq!(doc.page_count(), 2);
    let t0 = doc.page(0).unwrap().extract_plain_text(&doc).unwrap();
    let t1 = doc.page(1).unwrap().extract_plain_text(&doc).unwrap();

    assert!(t0.contains("CASE-000001-SEC"));
    assert!(t1.contains("CASE-000002-SEC"));
}

#[test]
fn test_08_apply_bates_numbering_top_center() {
    let pdf = create_multipage_pdf(&["Header Doc"], None);
    let config = BatesNumberingConfig {
        prefix: "CONFIDENTIAL_".to_string(),
        suffix: String::new(),
        start_number: 42,
        num_digits: 4,
        position: BatesPosition::TopCenter,
        font_size: 12.0,
        color_rgb: [0.0, 0.0, 0.0],
        margin_pt: 25.0,
    };

    let stamped = apply_bates_numbering(&pdf, &config).expect("Bates stamping top center");
    let doc = PdfDocument::parse(stamped).expect("Parse doc");

    let text = doc.page(0).unwrap().extract_plain_text(&doc).unwrap();
    assert!(text.contains("CONFIDENTIAL_0042"));
}

#[test]
fn test_09_inject_ocr_invisible_text_layer() {
    let pdf = create_multipage_pdf(&["Scanned Raster Base"], None);
    let ocr_data = OcrPageData {
        page_index: 0,
        words: vec![
            OcrWord {
                text: "INVOICE".to_string(),
                bbox: OcrBoundingBox {
                    x: 100.0,
                    y: 700.0,
                    width: 80.0,
                    height: 20.0,
                },
                confidence: 0.99,
            },
            OcrWord {
                text: "TOTAL".to_string(),
                bbox: OcrBoundingBox {
                    x: 100.0,
                    y: 650.0,
                    width: 50.0,
                    height: 18.0,
                },
                confidence: 0.98,
            },
        ],
    };

    let ocr_injected = inject_ocr_layer(&pdf, 0, &ocr_data).expect("OCR layer injection");

    // Verify invisible text rendering mode "3 Tr" per ISO 32000-1 §9.3.5
    let ocr_str = String::from_utf8_lossy(&ocr_injected);
    assert!(
        ocr_str.contains("3 Tr"),
        "Must contain invisible text rendering mode (3 Tr)"
    );
    assert!(ocr_str.contains("(INVOICE) Tj"));
    assert!(ocr_str.contains("(TOTAL) Tj"));
}

#[test]
fn test_10_ocr_injected_document_text_extraction() {
    let pdf = create_multipage_pdf(&["Underlying Image"], None);
    let ocr_data = OcrPageData {
        page_index: 0,
        words: vec![
            OcrWord {
                text: "PATENT".to_string(),
                bbox: OcrBoundingBox {
                    x: 50.0,
                    y: 750.0,
                    width: 60.0,
                    height: 15.0,
                },
                confidence: 0.95,
            },
            OcrWord {
                text: "SPECIFICATION".to_string(),
                bbox: OcrBoundingBox {
                    x: 120.0,
                    y: 750.0,
                    width: 110.0,
                    height: 15.0,
                },
                confidence: 0.97,
            },
        ],
    };

    let ocr_injected = inject_ocr_layer(&pdf, 0, &ocr_data).expect("Inject OCR layer");
    let doc = PdfDocument::parse(ocr_injected).expect("Parsed OCR injected PDF");

    let page_text = doc
        .page(0)
        .expect("Page exists")
        .extract_plain_text(&doc)
        .expect("Extract text from OCR page");
    assert!(page_text.contains("PATENT"));
    assert!(page_text.contains("SPECIFICATION"));
}
