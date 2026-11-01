//! Sub-Phase 50.2.1 Integration Tests: Cross-Format Conversion Pipeline & Batch Exporter
//!
//! Complies strictly with GEC-v7.0 Article I (Architectural Integrity & Zero Global Document Singletons),
//! Article III (Real Binary File Open & Save, Zero Data Loss), and Article IV (Code Safety & Zero Panic):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in production library code.
//! - Comprehensive cross-format conversions: Markdown <-> Docx, Markdown <-> HTML, CSV <-> XLSX,
//!   CSV <-> JSON, Text -> PDF.
//! - Batch conversion reports and duration estimation.

use excel_engine::package::ExcelWorkbookPackage;
use pdf_engine::catalog::PdfDocument;
use render_core::conversion::{
    BatchConversionReport, ConversionError, ConversionFormat, ConversionOptions, ConversionTask,
    CrossFormatConverter,
};
use std::io::Cursor;
use word_engine::package::WordPackage;

#[test]
fn test_markdown_to_html_conversion() {
    let md = "# Title Heading\n\nThis is **bold** text and `inline_code`.\n\n* Item 1\n* Item 2";
    let task = ConversionTask {
        id: "task-md-html-1".to_string(),
        source_format: ConversionFormat::Markdown,
        target_format: ConversionFormat::Html,
        input_bytes: md.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    assert_eq!(result.target_format, ConversionFormat::Html);

    let html = String::from_utf8(result.output_bytes).expect("Valid UTF-8");
    assert!(html.contains("<h1>Title Heading</h1>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<code>inline_code</code>"));
    assert!(html.contains("<li>Item 1</li>"));
    assert!(html.contains("<li>Item 2</li>"));
}

#[test]
fn test_html_to_markdown_conversion() {
    let html = "<!DOCTYPE html><html><body><h1>Project Roadmap</h1><p>Here is <strong>critical</strong> information.</p><ul><li>Alpha</li><li>Beta</li></ul></body></html>";
    let task = ConversionTask {
        id: "task-html-md-1".to_string(),
        source_format: ConversionFormat::Html,
        target_format: ConversionFormat::Markdown,
        input_bytes: html.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    assert_eq!(result.target_format, ConversionFormat::Markdown);

    let md = String::from_utf8(result.output_bytes).expect("Valid UTF-8");
    assert!(md.contains("# Project Roadmap"));
    assert!(md.contains("**critical**"));
    assert!(md.contains("* Alpha"));
    assert!(md.contains("* Beta"));
}

#[test]
fn test_markdown_to_docx_conversion() {
    let md = "# Architectural Specification\n\nUniversal cross-format conversion pipeline in pure Rust.\n\n## Core Principles\n\nZero panics and strict safe code.";
    let task = ConversionTask {
        id: "task-md-docx-1".to_string(),
        source_format: ConversionFormat::Markdown,
        target_format: ConversionFormat::Docx,
        input_bytes: md.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    assert_eq!(result.target_format, ConversionFormat::Docx);
    assert!(!result.output_bytes.is_empty());

    // Verify output by opening it with WordPackage
    let reader = Cursor::new(&result.output_bytes);
    let pkg = WordPackage::open(reader).expect("Output must be valid DOCX package");
    let doc = &pkg.document;
    assert!(!doc.body.elements.is_empty());

    // Verify text exists in body
    let mut extracted_texts = Vec::new();
    for el in &doc.body.elements {
        if let word_engine::model::BodyElement::Paragraph(p) = el {
            for child in &p.children {
                if let word_engine::model::ParagraphChild::Run(r) = child {
                    for content in &r.content {
                        if let word_engine::model::RunContent::Text(t, _) = content {
                            extracted_texts.push(t.as_str());
                        }
                    }
                }
            }
        }
    }
    let full_text = extracted_texts.join(" ");

    assert!(full_text.contains("Architectural Specification"));
    assert!(full_text.contains("Universal cross-format"));
}

#[test]
fn test_docx_to_markdown_conversion() {
    // Generate docx first from markdown, then convert back
    let original_md = "# Module Design\n\nDetailed breakdown of components.";
    let gen_task = ConversionTask {
        id: "gen-docx".to_string(),
        source_format: ConversionFormat::Markdown,
        target_format: ConversionFormat::Docx,
        input_bytes: original_md.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };
    let gen_res = CrossFormatConverter::convert_single(gen_task);
    assert!(gen_res.success);

    // Now convert docx to markdown
    let task = ConversionTask {
        id: "task-docx-md-1".to_string(),
        source_format: ConversionFormat::Docx,
        target_format: ConversionFormat::Markdown,
        input_bytes: gen_res.output_bytes,
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    let md = String::from_utf8(result.output_bytes).expect("Valid UTF-8");
    assert!(md.contains("Module Design"));
    assert!(md.contains("Detailed breakdown of components"));
}

#[test]
fn test_csv_to_json_conversion() {
    let csv = "Product,Price,Quantity\nWidget,19.99,10\nGadget,49.50,5\nSensor,12.00,42";
    let task = ConversionTask {
        id: "task-csv-json-1".to_string(),
        source_format: ConversionFormat::Csv,
        target_format: ConversionFormat::Json,
        input_bytes: csv.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);

    let json_str = String::from_utf8(result.output_bytes).expect("Valid UTF-8");
    let json_val: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output must be valid JSON");

    assert!(json_val.is_array());
    let arr = json_val.as_array().expect("Array");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["Product"], "Widget");
    assert_eq!(arr[0]["Price"], "19.99");
    assert_eq!(arr[0]["Quantity"], "10");
    assert_eq!(arr[1]["Product"], "Gadget");
    assert_eq!(arr[2]["Product"], "Sensor");
}

#[test]
fn test_json_to_csv_conversion() {
    let json_data = r#"[
        {"id": "001", "name": "Alice", "role": "Architect"},
        {"id": "002", "name": "Bob", "role": "Engineer"}
    ]"#;

    let task = ConversionTask {
        id: "task-json-csv-1".to_string(),
        source_format: ConversionFormat::Json,
        target_format: ConversionFormat::Csv,
        input_bytes: json_data.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);

    let csv_str = String::from_utf8(result.output_bytes).expect("Valid UTF-8");
    let lines: Vec<&str> = csv_str.trim().lines().collect();
    assert!(lines.len() >= 3);
    assert!(csv_str.contains("Alice"));
    assert!(csv_str.contains("Bob"));
    assert!(csv_str.contains("Architect"));
    assert!(csv_str.contains("Engineer"));
}

#[test]
fn test_csv_to_xlsx_conversion() {
    let csv = "Region,Q1,Q2,Q3,Q4\nNorth,100,120,140,160\nSouth,80,90,110,130";
    let task = ConversionTask {
        id: "task-csv-xlsx-1".to_string(),
        source_format: ConversionFormat::Csv,
        target_format: ConversionFormat::Xlsx,
        input_bytes: csv.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    assert!(!result.output_bytes.is_empty());

    // Verify output with ExcelWorkbookPackage
    let reader = Cursor::new(&result.output_bytes);
    let pkg = ExcelWorkbookPackage::open(reader).expect("Must open as valid XLSX package");
    assert!(!pkg.worksheets.is_empty());
    let ws = &pkg.worksheets[0].1;
    assert!(ws.grid.populated_cell_count() >= 10);
    assert!(ws.grid.is_occupied(0, 0));
    assert!(ws.grid.is_occupied(1, 0));
}

#[test]
fn test_xlsx_to_csv_roundtrip() {
    let original_csv = "Item,Count,Status\nAlpha,10,Active\nBeta,25,Pending";
    let to_xlsx_task = ConversionTask {
        id: "csv-to-xlsx".to_string(),
        source_format: ConversionFormat::Csv,
        target_format: ConversionFormat::Xlsx,
        input_bytes: original_csv.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };
    let xlsx_res = CrossFormatConverter::convert_single(to_xlsx_task);
    assert!(xlsx_res.success);

    let to_csv_task = ConversionTask {
        id: "xlsx-to-csv".to_string(),
        source_format: ConversionFormat::Xlsx,
        target_format: ConversionFormat::Csv,
        input_bytes: xlsx_res.output_bytes,
        options: ConversionOptions::default(),
    };
    let csv_res = CrossFormatConverter::convert_single(to_csv_task);
    assert!(csv_res.success);

    let csv_str = String::from_utf8(csv_res.output_bytes).expect("Valid UTF-8");
    assert!(csv_str.contains("Item"));
    assert!(csv_str.contains("Alpha"));
    assert!(csv_str.contains("Beta"));
}

#[test]
fn test_text_to_pdf_conversion() {
    let text =
        "AetherOffice Official Document\nConfidential and Proprietary.\nLine 3 of PDF output.";
    let task = ConversionTask {
        id: "task-txt-pdf-1".to_string(),
        source_format: ConversionFormat::PlainText,
        target_format: ConversionFormat::Pdf,
        input_bytes: text.as_bytes().to_vec(),
        options: ConversionOptions::default(),
    };

    let result = CrossFormatConverter::convert_single(task);
    assert!(result.success);
    assert!(!result.output_bytes.is_empty());

    // Verify PDF header bytes
    assert!(result.output_bytes.starts_with(b"%PDF-"));

    // Verify PDF parsing with PdfDocument
    let doc = PdfDocument::parse(result.output_bytes).expect("Output must be valid PDF");
    assert_eq!(doc.page_count(), 1);
}

#[test]
fn test_batch_conversion_and_unsupported_error_handling() {
    let task1 = ConversionTask {
        id: "batch-1".to_string(),
        source_format: ConversionFormat::Markdown,
        target_format: ConversionFormat::Html,
        input_bytes: b"# Batch One".to_vec(),
        options: ConversionOptions::default(),
    };

    let task2 = ConversionTask {
        id: "batch-2".to_string(),
        source_format: ConversionFormat::Csv,
        target_format: ConversionFormat::Json,
        input_bytes: b"ColA,ColB\nVal1,Val2".to_vec(),
        options: ConversionOptions::default(),
    };

    // Unsupported conversion: DOCX -> XLSX
    let task3 = ConversionTask {
        id: "batch-3".to_string(),
        source_format: ConversionFormat::Docx,
        target_format: ConversionFormat::Xlsx,
        input_bytes: b"dummy docx bytes".to_vec(),
        options: ConversionOptions::default(),
    };

    let report: BatchConversionReport =
        CrossFormatConverter::batch_convert(vec![task1, task2, task3]);

    assert_eq!(report.total_tasks, 3);
    assert_eq!(report.succeeded, 2);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results.len(), 3);

    // Verify task 1 succeeded
    assert!(report.results[0].success);
    // Verify task 2 succeeded
    assert!(report.results[1].success);
    // Verify task 3 failed gracefully
    assert!(!report.results[2].success);
    assert!(report.results[2]
        .error_message
        .as_ref()
        .unwrap()
        .contains("Unsupported conversion"));

    // Test estimate_job calculation
    let estimate = CrossFormatConverter::estimate_job(10, 10240);
    assert_eq!(estimate.task_count, 10);
    assert_eq!(estimate.total_bytes, 10240);
    assert!(estimate.estimated_duration_ms >= 100);

    // Test supported formats query
    let md_targets = CrossFormatConverter::supported_targets(ConversionFormat::Markdown);
    assert!(md_targets.contains(&ConversionFormat::Html));
    assert!(md_targets.contains(&ConversionFormat::Docx));
    assert!(md_targets.contains(&ConversionFormat::Pdf));

    // Unsupported direct conversion error variant
    let err_res = CrossFormatConverter::convert(
        ConversionFormat::Docx,
        ConversionFormat::Xlsx,
        b"data",
        &ConversionOptions::default(),
    );
    assert!(matches!(
        err_res,
        Err(ConversionError::UnsupportedConversion { .. })
    ));
}
