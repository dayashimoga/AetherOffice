//! Sub-Phase 49.2.1 Integration Tests:
//! Side-by-Side PDF Visual & Text Diff Engine
//!
//! Validates:
//! - Exact comparison of identical documents (zero diffs)
//! - Text insertion detection with word-level bounding box placement
//! - Text deletion detection with word-level bounding box placement
//! - In-place text modification / substitution detection
//! - Case insensitive diff option evaluation
//! - Page count additions (inserted pages in revised doc)
//! - Page count deletions (deleted pages in revised doc)
//! - MediaBox page geometry dimension changes
//! - Standalone SVG diff overlay generation with coordinate inversion
//! - Multi-page document differential comparison

use pdf_engine::diff::{DiffKind, PdfDiffEngine, PdfDiffOptions};

fn create_test_pdf(pages: &[&str], media_boxes: Option<&[(f64, f64)]>) -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let num_pages = pages.len();
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
fn test_01_identical_documents_zero_diffs() {
    let doc_a = create_test_pdf(&["Contract clause one terms and conditions."], None);
    let doc_b = create_test_pdf(&["Contract clause one terms and conditions."], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert_eq!(res.summary.total_diffs, 0);
    assert_eq!(res.summary.insertions, 0);
    assert_eq!(res.summary.deletions, 0);
    assert_eq!(res.summary.modifications, 0);
    assert_eq!(res.summary.pages_with_diffs, 0);
    assert!(!res.pages[0].has_changes);
}

#[test]
fn test_02_text_insertion_detected() {
    let doc_a = create_test_pdf(&["Standard Warranty Coverage"], None);
    let doc_b = create_test_pdf(&["Standard Extended Warranty Coverage"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert!(res.summary.insertions >= 1);
    assert!(res.pages[0].has_changes);

    let inserted_item = res.pages[0]
        .items
        .iter()
        .find(|item| item.kind == DiffKind::Inserted)
        .expect("Inserted item must exist");

    assert!(inserted_item
        .text_b
        .as_deref()
        .unwrap_or("")
        .contains("Extended"));
    assert!(inserted_item.bbox_b.is_some());
}

#[test]
fn test_03_text_deletion_detected() {
    let doc_a = create_test_pdf(&["Notice to Terminate Immediately"], None);
    let doc_b = create_test_pdf(&["Notice to Terminate"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert!(res.summary.deletions >= 1);
    assert!(res.pages[0].has_changes);

    let deleted_item = res.pages[0]
        .items
        .iter()
        .find(|item| item.kind == DiffKind::Deleted)
        .expect("Deleted item must exist");

    assert!(deleted_item
        .text_a
        .as_deref()
        .unwrap_or("")
        .contains("Immediately"));
    assert!(deleted_item.bbox_a.is_some());
}

#[test]
fn test_04_text_modification_detected() {
    let doc_a = create_test_pdf(&["Total fee: USD 1000 due quarterly"], None);
    let doc_b = create_test_pdf(&["Total fee: USD 2500 due quarterly"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert!(res.summary.modifications >= 1);
    let modified = res.pages[0]
        .items
        .iter()
        .find(|item| item.kind == DiffKind::Modified)
        .expect("Modified item must exist");

    assert_eq!(modified.text_a.as_deref(), Some("1000"));
    assert_eq!(modified.text_b.as_deref(), Some("2500"));
    assert!(modified.bbox_a.is_some());
    assert!(modified.bbox_b.is_some());
}

#[test]
fn test_05_case_insensitive_option() {
    let doc_a = create_test_pdf(&["CONFIDENTIAL AGREEMENT"], None);
    let doc_b = create_test_pdf(&["confidential agreement"], None);

    // Case sensitive: modifications found
    let res_sensitive = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Sensitive compare");
    assert!(res_sensitive.summary.total_diffs > 0);

    // Case insensitive: 0 diffs
    let opt = PdfDiffOptions {
        ignore_case: true,
        ignore_whitespace: true,
        detect_moves: false,
    };
    let res_insensitive =
        PdfDiffEngine::compare(&doc_a, &doc_b, Some(&opt)).expect("Insensitive compare");
    assert_eq!(res_insensitive.summary.total_diffs, 0);
}

#[test]
fn test_06_page_count_added_page_detected() {
    let doc_a = create_test_pdf(&["Page 1 Only"], None);
    let doc_b = create_test_pdf(&["Page 1 Only", "Appendix Added"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert_eq!(res.summary.page_count_a, 1);
    assert_eq!(res.summary.page_count_b, 2);
    assert_eq!(res.pages.len(), 2);
    assert!(!res.pages[0].has_changes);
    assert!(res.pages[1].has_changes);

    let added_page = res.pages[1]
        .items
        .iter()
        .find(|item| item.kind == DiffKind::Inserted)
        .expect("Inserted page diff item");
    assert!(added_page.description.contains("Page 2 inserted"));
}

#[test]
fn test_07_page_count_deleted_page_detected() {
    let doc_a = create_test_pdf(&["Section One", "Section Two Obsolete"], None);
    let doc_b = create_test_pdf(&["Section One"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert_eq!(res.summary.page_count_a, 2);
    assert_eq!(res.summary.page_count_b, 1);
    assert!(res.pages[1].has_changes);

    let deleted_page = res.pages[1]
        .items
        .iter()
        .find(|item| item.kind == DiffKind::Deleted)
        .expect("Deleted page diff item");
    assert!(deleted_page.description.contains("Page 2 deleted"));
}

#[test]
fn test_08_mediabox_dimension_change_detected() {
    let doc_a = create_test_pdf(&["Standard Letter Page"], Some(&[(612.0, 792.0)]));
    let doc_b = create_test_pdf(&["Standard Letter Page"], Some(&[(595.28, 841.89)]));

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert!(res.pages[0].media_box_changed);
    assert!(res.pages[0].has_changes);
    assert_eq!(res.summary.modifications, 1);
}

#[test]
fn test_09_diff_overlay_svg_generation() {
    let doc_a = create_test_pdf(&["Preliminary Draft"], None);
    let doc_b = create_test_pdf(&["Final Approved Version"], None);

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");
    let items = &res.pages[0].items;

    // Overlay for Document A (shows deletions/modifications in red/amber)
    let svg_a =
        PdfDiffEngine::generate_diff_overlay_svg(&doc_a, 0, items, false).expect("Generate SVG A");
    assert!(svg_a.starts_with("<svg"));
    assert!(svg_a.ends_with("</svg>"));

    // Overlay for Document B (shows insertions/modifications in green/amber)
    let svg_b =
        PdfDiffEngine::generate_diff_overlay_svg(&doc_b, 0, items, true).expect("Generate SVG B");
    assert!(svg_b.starts_with("<svg"));
    assert!(svg_b.ends_with("</svg>"));
}

#[test]
fn test_10_multi_page_document_comprehensive_diff() {
    let doc_a = create_test_pdf(
        &[
            "Executive Summary Original",
            "Financial Results Unchanged",
            "Conclusion Version 1",
        ],
        None,
    );
    let doc_b = create_test_pdf(
        &[
            "Executive Summary Revised",
            "Financial Results Unchanged",
            "Conclusion Version 2",
        ],
        None,
    );

    let res = PdfDiffEngine::compare(&doc_a, &doc_b, None).expect("Diff comparison");

    assert_eq!(res.pages.len(), 3);
    assert!(res.pages[0].has_changes, "Page 0 must have changes");
    assert!(!res.pages[1].has_changes, "Page 1 must be identical");
    assert!(res.pages[2].has_changes, "Page 2 must have changes");
    assert_eq!(res.summary.pages_with_diffs, 2);
}
