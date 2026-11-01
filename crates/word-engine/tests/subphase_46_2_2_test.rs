//! Sub-Phase 46.2.2 Integration Tests: Mail Merge, OMML/LaTeX Equations & .odt/.rtf Import/Export
//!
//! Validates:
//! - Mail merge CSV parsing, merge field extraction, and single/batch merge generation.
//! - LaTeX to OMML and OMML to LaTeX bidirectional math parsing and XML serialization.
//! - OASIS OpenDocument Text (.odt) lossless encode and decode.
//! - Rich Text Format (.rtf) lossless encode and decode.
//! - High-throughput performance under heavy workloads.

use std::collections::HashMap;
use std::time::Instant;
use word_engine::mail_merge::*;
use word_engine::math::*;
use word_engine::model::*;
use word_engine::odt_codec::*;
use word_engine::rtf_codec::*;

#[test]
fn test_mail_merge_csv_parsing_and_field_discovery() {
    let csv_sample = r#"FirstName,LastName,Company,Balance
Alice,"Smith, Jr.",Acme Corp,$1500
Bob,Jones,"Global Tech, LLC",$250
Charlie,Brown,Peanuts Inc,$0
"#;

    let (headers, records) = MailMergeEngine::parse_csv(csv_sample);
    assert_eq!(headers, vec!["FirstName", "LastName", "Company", "Balance"]);
    assert_eq!(records.len(), 3);

    // Verify Alice row with quoted comma
    assert_eq!(records[0].get("FirstName").unwrap(), "Alice");
    assert_eq!(records[0].get("LastName").unwrap(), "Smith, Jr.");
    assert_eq!(records[0].get("Company").unwrap(), "Acme Corp");
    assert_eq!(records[0].get("Balance").unwrap(), "$1500");

    // Verify Bob row with quoted company
    assert_eq!(records[1].get("Company").unwrap(), "Global Tech, LLC");

    // Template with both chevrons and MERGEFIELD syntax
    let mut template = Document::default();
    let mut p1 = Paragraph::default();
    let mut r1 = Run::default();
    r1.content.push(RunContent::Text(
        "Dear «FirstName» «LastName», your account at MERGEFIELD Company has balance «Balance»."
            .to_string(),
        false,
    ));
    p1.children.push(ParagraphChild::Run(r1));
    template.body.elements.push(BodyElement::Paragraph(p1));

    let fields = MailMergeEngine::find_merge_fields(&template);
    assert_eq!(fields, vec!["Balance", "Company", "FirstName", "LastName"]);
}

#[test]
fn test_mail_merge_single_and_batch_execution() {
    let mut template = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Invoice for «Customer»: Total due is «Amount» by «DueDate».".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    template.body.elements.push(BodyElement::Paragraph(p));

    let mut record = HashMap::new();
    record.insert("Customer".to_string(), "Quantum Corp".to_string());
    record.insert("Amount".to_string(), "$9,500.00".to_string());
    record.insert("DueDate".to_string(), "Nov 15, 2026".to_string());

    // 1. Single merge
    let merged_doc = MailMergeEngine::merge_record(&template, &record);
    assert_eq!(
        merged_doc.body.get_full_text(),
        "Invoice for Quantum Corp: Total due is $9,500.00 by Nov 15, 2026."
    );

    // 2. Batch merge
    let records = vec![record.clone(), {
        let mut r2 = HashMap::new();
        r2.insert("Customer".to_string(), "Apex Dynamics".to_string());
        r2.insert("Amount".to_string(), "$1,200.00".to_string());
        r2.insert("DueDate".to_string(), "Nov 20, 2026".to_string());
        r2
    }];

    let batch_doc = MailMergeEngine::merge_batch(&template, &records);
    // 2 record paragraphs + 1 page break paragraph = 3 paragraphs
    assert_eq!(batch_doc.body.elements.len(), 3);

    // Verify first record
    assert_eq!(
        batch_doc.body.elements[0].get_text(),
        "Invoice for Quantum Corp: Total due is $9,500.00 by Nov 15, 2026."
    );

    // Verify page break
    if let BodyElement::Paragraph(ref bp) = batch_doc.body.elements[1] {
        assert!(bp.children.iter().any(|c| match c {
            ParagraphChild::Run(ref r) => r
                .content
                .iter()
                .any(|rc| matches!(rc, RunContent::Break(Some(ref t)) if t == "page")),
            _ => false,
        }));
    } else {
        panic!("Element 1 must be page break paragraph");
    }

    // Verify second record
    assert_eq!(
        batch_doc.body.elements[2].get_text(),
        "Invoice for Apex Dynamics: Total due is $1,200.00 by Nov 20, 2026."
    );
}

#[test]
fn test_latex_to_omml_fractions_radicals_scripts() {
    // 1. Fraction: \frac{a + b}{c - d}
    let frac = MathConverter::latex_to_omml("\\frac{a + b}{c - d}");
    match frac {
        MathElement::Fraction { num, den } => {
            assert!(matches!(*num, MathElement::Group(_)));
            assert!(matches!(*den, MathElement::Group(_)));
        }
        _ => panic!("Expected MathElement::Fraction"),
    }

    // 2. Square root: \sqrt{x^2 + y^2}
    let sqrt = MathConverter::latex_to_omml("\\sqrt{x^2 + y^2}");
    match sqrt {
        MathElement::Radical { base, degree } => {
            assert!(degree.is_none());
            assert!(matches!(*base, MathElement::Group(_)));
        }
        _ => panic!("Expected MathElement::Radical"),
    }

    // 3. Nth root: \sqrt[3]{8}
    let nth_root = MathConverter::latex_to_omml("\\sqrt[3]{8}");
    match nth_root {
        MathElement::Radical { base, degree } => {
            assert_eq!(*degree.unwrap(), MathElement::Text("3".to_string()));
            assert_eq!(*base, MathElement::Text("8".to_string()));
        }
        _ => panic!("Expected MathElement::Radical with degree"),
    }

    // 4. Superscript and subscript: x_i^2
    let script = MathConverter::latex_to_omml("x_i^2");
    match script {
        MathElement::Superscript { base, exp } => {
            assert_eq!(*exp, MathElement::Text("2".to_string()));
            match *base {
                MathElement::Subscript {
                    base: sub_base,
                    sub,
                } => {
                    assert_eq!(*sub_base, MathElement::Text("x".to_string()));
                    assert_eq!(*sub, MathElement::Text("i".to_string()));
                }
                _ => panic!("Expected Subscript base"),
            }
        }
        _ => panic!("Expected Superscript"),
    }

    // 5. Delimiters: \left( x + 1 \right)
    let delim = MathConverter::latex_to_omml("\\left( x + 1 \\right)");
    match delim {
        MathElement::Delimiter {
            open,
            close,
            content,
        } => {
            assert_eq!(open, "(");
            assert_eq!(close, ")");
            assert!(matches!(*content, MathElement::Group(_)));
        }
        _ => panic!("Expected Delimiter"),
    }
}

#[test]
fn test_omml_bidirectional_roundtrip_and_xml() {
    let original_latex = "\\frac{x + 1}{y - 1}";
    let omml = MathConverter::latex_to_omml(original_latex);
    let reconstructed_latex = MathConverter::omml_to_latex(&omml);
    assert_eq!(reconstructed_latex, "\\frac{x + 1}{y - 1}");

    // Test XML serialization
    let xml = MathConverter::omml_to_xml(&omml);
    assert!(xml.starts_with("<m:oMath"));
    assert!(xml.contains("<m:f>"));
    assert!(xml.contains("<m:num>"));
    assert!(xml.contains("<m:den>"));
    assert!(xml.ends_with("</m:oMath>"));

    // Greek symbols translation
    let greek_latex = "\\alpha + \\beta = \\gamma";
    let omml_greek = MathConverter::latex_to_omml(greek_latex);
    let back_to_latex = MathConverter::omml_to_latex(&omml_greek);
    assert_eq!(back_to_latex, "\\alpha + \\beta = \\gamma");
}

#[test]
fn test_odt_codec_encode_and_decode() {
    let mut doc = Document::default();

    // 1. Heading paragraph
    let mut p_h = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Heading 1".to_string()),
            outline_level: Some(0),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r_h = Run::default();
    r_h.content.push(RunContent::Text(
        "ODF Standard Architecture".to_string(),
        false,
    ));
    p_h.children.push(ParagraphChild::Run(r_h));
    doc.body.elements.push(BodyElement::Paragraph(p_h));

    // 2. Normal text
    let mut p_text = Paragraph::default();
    let mut r_text = Run::default();
    r_text.content.push(RunContent::Text(
        "OpenDocument Text format interchange test.".to_string(),
        false,
    ));
    p_text.children.push(ParagraphChild::Run(r_text));
    doc.body.elements.push(BodyElement::Paragraph(p_text));

    // 3. Table
    let mut tbl = Table::default();
    let mut row = TableRow::default();
    let mut cell1 = TableCell::default();
    let mut p_cell = Paragraph::default();
    let mut r_cell = Run::default();
    r_cell
        .content
        .push(RunContent::Text("Cell 1 Data".to_string(), false));
    p_cell.children.push(ParagraphChild::Run(r_cell));
    cell1.elements.push(BodyElement::Paragraph(p_cell));
    row.cells.push(cell1);
    tbl.rows.push(row);
    doc.body.elements.push(BodyElement::Table(tbl));

    // Encode to .odt binary ZIP bytes
    let odt_bytes = OdtCodec::encode(&doc).expect("ODT encode must succeed");
    assert!(!odt_bytes.is_empty());
    // ZIP magic signature PK\x03\x04
    assert_eq!(&odt_bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);

    // Decode back from .odt bytes
    let decoded_doc = OdtCodec::decode(&odt_bytes).expect("ODT decode must succeed");
    assert_eq!(decoded_doc.body.elements.len(), 3);

    // Verify heading
    if let BodyElement::Paragraph(ref p) = decoded_doc.body.elements[0] {
        assert_eq!(p.get_text(), "ODF Standard Architecture");
        assert_eq!(p.properties.as_ref().unwrap().outline_level, Some(0));
    } else {
        panic!("Element 0 must be paragraph");
    }

    // Verify text
    assert_eq!(
        decoded_doc.body.elements[1].get_text(),
        "OpenDocument Text format interchange test."
    );

    // Verify table
    if let BodyElement::Table(ref t) = decoded_doc.body.elements[2] {
        assert_eq!(t.rows.len(), 1);
        assert_eq!(t.rows[0].cells.len(), 1);
        assert_eq!(t.rows[0].cells[0].elements[0].get_text(), "Cell 1 Data");
    } else {
        panic!("Element 2 must be table");
    }
}

#[test]
fn test_rtf_codec_encode_and_decode() {
    let mut doc = Document::default();

    let mut p = Paragraph {
        properties: Some(ParagraphProperties {
            justification: Some("center".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let r_bold = Run {
        properties: Some(RunProperties {
            bold: true,
            ..Default::default()
        }),
        content: vec![RunContent::Text("Bold Title".to_string(), false)],
        ..Default::default()
    };
    p.children.push(ParagraphChild::Run(r_bold));

    let r_italic = Run {
        properties: Some(RunProperties {
            italic: true,
            ..Default::default()
        }),
        content: vec![RunContent::Text(" Italic Subtitle".to_string(), false)],
        ..Default::default()
    };
    p.children.push(ParagraphChild::Run(r_italic));
    doc.body.elements.push(BodyElement::Paragraph(p));

    // Encode to RTF
    let rtf_bytes = RtfCodec::encode(&doc).expect("RTF encode must succeed");
    let rtf_str = String::from_utf8_lossy(&rtf_bytes);
    assert!(rtf_str.starts_with("{\\rtf1"));
    assert!(rtf_str.contains("\\qc"));
    assert!(rtf_str.contains("\\b"));
    assert!(rtf_str.contains("Bold Title"));
    assert!(rtf_str.contains("\\i"));
    assert!(rtf_str.contains("Italic Subtitle"));
    assert!(rtf_str.ends_with('}'));

    // Decode back from RTF
    let decoded_doc = RtfCodec::decode(&rtf_bytes).expect("RTF decode must succeed");
    assert_eq!(decoded_doc.body.elements.len(), 1);
    let full_text = decoded_doc.body.get_full_text();
    assert!(full_text.contains("Bold Title"));
    assert!(full_text.contains("Italic Subtitle"));
}

#[test]
fn test_subphase_46_2_2_heavy_workload_performance() {
    let mut template = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Employee «ID»: «Name» in department «Dept» has salary «Salary».".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    template.body.elements.push(BodyElement::Paragraph(p));

    let mut records = Vec::with_capacity(1000);
    for i in 0..1000 {
        let mut rec = HashMap::new();
        rec.insert("ID".to_string(), format!("EMP_{:04}", i));
        rec.insert("Name".to_string(), format!("Employee Name {}", i));
        rec.insert("Dept".to_string(), "Engineering".to_string());
        rec.insert("Salary".to_string(), format!("${}", 80000 + i * 50));
        records.push(rec);
    }

    // 1. Benchmark 1,000 record mail merge batch
    let start_merge = Instant::now();
    let merged_batch = MailMergeEngine::merge_batch(&template, &records);
    let merge_duration = start_merge.elapsed();

    // 1000 records + 999 page break paragraphs = 1999 paragraphs
    assert_eq!(merged_batch.body.elements.len(), 1999);
    assert!(
        merge_duration.as_millis() < 50,
        "1,000 records batch merge took {:?}, expected < 50ms",
        merge_duration
    );

    // 2. Benchmark 500 LaTeX conversions
    let latex_expr = "\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}";
    let start_math = Instant::now();
    for _ in 0..500 {
        let omml = MathConverter::latex_to_omml(latex_expr);
        let _ = MathConverter::omml_to_latex(&omml);
        let _ = MathConverter::omml_to_xml(&omml);
    }
    let math_duration = start_math.elapsed();
    assert!(
        math_duration.as_millis() < 50,
        "500 LaTeX conversions took {:?}, expected < 50ms",
        math_duration
    );
}
