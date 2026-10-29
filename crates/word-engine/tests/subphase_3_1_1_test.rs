//! Subphase 3.1.1 Test Suite: WordprocessingML Piece-Tree Rope DOM & Sub-Document Parser
//! Strictly conforms to GEC-v1.0 (Zero Panics in production code, Performance SLAs, Zero-Loss ECMA-376 Fidelity).

#![allow(clippy::field_reassign_with_default)]

use ooxml_opc::unparsed::UnparsedPart;
use std::time::Instant;
use word_engine::model::*;
use word_engine::sub_documents::*;
use word_engine::{
    parse_comments, parse_document, parse_endnotes, parse_footer, parse_footnotes, parse_header,
    serialize_comments, serialize_document, serialize_endnotes, serialize_footer,
    serialize_footnotes, serialize_header, PieceTree, WordPackage,
};

fn resolve_repo_path(rel: &str) -> std::path::PathBuf {
    let direct = std::path::Path::new(rel);
    if direct.exists() {
        return direct.to_path_buf();
    }
    let from_manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel);
    if from_manifest.exists() {
        return from_manifest;
    }
    std::path::PathBuf::from(rel)
}

#[test]
fn test_01_piecetree_initialization_and_len() {
    let text = "Hello world!\nThis is line 2.\nAnd line 3.";
    let tree = PieceTree::new(text);

    assert_eq!(tree.len(), text.len());
    assert_eq!(tree.line_count(), 3);
    assert!(!tree.is_empty());
    assert_eq!(tree.get_text(), text);

    let empty = PieceTree::new("");
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.line_count(), 1);
    assert!(empty.is_empty());
    assert_eq!(empty.get_text(), "");
}

#[test]
fn test_02_piecetree_insert_beginning_middle_end() {
    let mut tree = PieceTree::new("world");

    // Insert at beginning
    tree.insert(0, "Hello ").unwrap();
    assert_eq!(tree.get_text(), "Hello world");
    assert_eq!(tree.len(), 11);

    // Insert in the middle
    tree.insert(5, " brave new").unwrap();
    assert_eq!(tree.get_text(), "Hello brave new world");

    // Insert at end
    tree.insert(tree.len(), "!").unwrap();
    assert_eq!(tree.get_text(), "Hello brave new world!");
}

#[test]
fn test_03_piecetree_delete_slices() {
    let mut tree = PieceTree::new("The quick brown fox jumps over the lazy dog");

    // Delete at start
    tree.delete(0, 4).unwrap();
    assert_eq!(tree.get_text(), "quick brown fox jumps over the lazy dog");

    // Delete in middle
    let brown_idx = tree.get_text().find("brown ").unwrap();
    tree.delete(brown_idx, 6).unwrap();
    assert_eq!(tree.get_text(), "quick fox jumps over the lazy dog");

    // Delete at end
    let dog_idx = tree.get_text().find(" dog").unwrap();
    tree.delete(dog_idx, 4).unwrap();
    assert_eq!(tree.get_text(), "quick fox jumps over the lazy");
}

#[test]
fn test_04_piecetree_slice_extraction() {
    let mut tree = PieceTree::new("First part");
    tree.insert(5, " (inserted) ").unwrap();
    // "First (inserted)  part"
    let full = tree.get_text();
    assert_eq!(full, "First (inserted)  part");

    // Slices across original and add buffers
    let sub = tree.slice(2, 10).unwrap();
    assert_eq!(sub, "rst (ins");

    let end_sub = tree.slice(15, 22).unwrap();
    assert_eq!(end_sub, ")  part");
}

#[test]
fn test_05_piecetree_line_offsets_and_conversion() {
    let mut tree = PieceTree::new("Alpha\nBeta\nGamma\nDelta");
    assert_eq!(tree.line_count(), 4);

    assert_eq!(tree.line_to_offset(0).unwrap(), 0);
    assert_eq!(tree.line_to_offset(1).unwrap(), 6);
    assert_eq!(tree.line_to_offset(2).unwrap(), 11);
    assert_eq!(tree.line_to_offset(3).unwrap(), 17);
    assert!(tree.line_to_offset(4).is_err());

    assert_eq!(tree.offset_to_line_col(0).unwrap(), (0, 0));
    assert_eq!(tree.offset_to_line_col(5).unwrap(), (0, 5));
    assert_eq!(tree.offset_to_line_col(6).unwrap(), (1, 0));
    assert_eq!(tree.offset_to_line_col(8).unwrap(), (1, 2));

    // Multi-line insertion preserves accurate line counts
    tree.insert(10, "\nNewline1\nNewline2").unwrap();
    assert_eq!(tree.line_count(), 6);
}

#[test]
fn test_06_piecetree_1000_page_keystroke_latency_sla() {
    // Generate ~1,000 pages of text: 3,000 characters per page = 3,000,000 characters
    let page_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\n".repeat(10);
    assert!(page_text.len() >= 3000);

    let doc_text = page_text.repeat(1000);
    assert!(
        doc_text.len() >= 3_000_000,
        "Document must be at least 3M chars"
    );

    let mut tree = PieceTree::new(&doc_text);
    assert_eq!(tree.len(), doc_text.len());

    let num_edits = 100;
    let mut max_edit_duration = std::time::Duration::ZERO;
    let start_all = Instant::now();

    for i in 0..num_edits {
        let offset = (i * 31337) % (tree.len() - 100);

        // Test insertion latency (simulating typing a character)
        let t0 = Instant::now();
        tree.insert(offset, "X").unwrap();
        let dur = t0.elapsed();
        if dur > max_edit_duration {
            max_edit_duration = dur;
        }

        // Test deletion latency (simulating backspace)
        let t1 = Instant::now();
        tree.delete(offset, 1).unwrap();
        let dur_del = t1.elapsed();
        if dur_del > max_edit_duration {
            max_edit_duration = dur_del;
        }
    }

    let total_time = start_all.elapsed();
    let avg_per_edit = total_time / (num_edits * 2) as u32;

    println!(
        "1000-page benchmark: max keystroke latency = {:?}, avg = {:?}",
        max_edit_duration, avg_per_edit
    );

    // GEC-v1.0 SLA: Word typing < 1.0 ms per keystroke, subphase requirement: < 0.5 ms
    assert!(
        max_edit_duration.as_micros() < 500,
        "Max keystroke latency exceeded SLA: {:?}",
        max_edit_duration
    );
}

#[test]
fn test_07_paragraph_and_run_properties_dom() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();

    let mut run = Run::default();
    run.content
        .push(RunContent::Text("Formatted text run".to_string(), false));
    run.properties = Some(RunProperties {
        bold: true,
        italic: true,
        underline: Some("single".to_string()),
        strike: true,
        double_strike: false,
        color: Some("FF0000".to_string()),
        size_half_points: Some(24),
        font_ascii: Some("Calibri".to_string()),
        font_h_ansi: Some("Calibri".to_string()),
        font_east_asia: None,
        font_cs: None,
        highlight: Some("yellow".to_string()),
        raw_elements: Vec::new(),
    });

    p.children.push(ParagraphChild::Run(run));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let xml_str = std::str::from_utf8(&xml).unwrap();
    assert!(xml_str.contains("<w:b/>"));
    assert!(xml_str.contains("<w:i/>"));
    assert!(xml_str.contains("<w:u w:val=\"single\"/>"));
    assert!(xml_str.contains("<w:strike/>"));
    assert!(xml_str.contains("<w:color w:val=\"FF0000\"/>"));
    assert!(xml_str.contains("<w:sz w:val=\"24\"/>"));
    assert!(xml_str.contains("<w:rFonts w:ascii=\"Calibri\""));

    let parsed = parse_document(&xml).expect("Parsing failed");
    assert_eq!(parsed.get_full_text(), "Formatted text run");
    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        if let ParagraphChild::Run(ref r) = p.children[0] {
            let props = r.properties.as_ref().unwrap();
            assert!(props.bold);
            assert!(props.italic);
            assert_eq!(props.underline.as_deref(), Some("single"));
            assert!(props.strike);
            assert_eq!(props.color.as_deref(), Some("FF0000"));
            assert_eq!(props.size_half_points, Some(24));
            assert_eq!(props.font_ascii.as_deref(), Some("Calibri"));
        } else {
            panic!("Expected Run child");
        }
    } else {
        panic!("Expected Paragraph element");
    }
}

#[test]
fn test_08_paragraph_alignment_and_spacing() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("Heading1".to_string()),
        justification: Some("center".to_string()),
        spacing: Some(Spacing {
            before: Some(240),
            after: Some(120),
            line: Some(360),
            line_rule: Some("auto".to_string()),
        }),
        indentation: Some(Indentation {
            left: Some(720),
            right: Some(360),
            first_line: Some(360),
            hanging: None,
        }),
        r_pr: None,
        raw_elements: Vec::new(),
    });

    let mut run = Run::default();
    run.content
        .push(RunContent::Text("Aligned Heading".to_string(), false));
    p.children.push(ParagraphChild::Run(run));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let parsed = parse_document(&xml).expect("Parsing failed");

    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        let p_pr = p.properties.as_ref().unwrap();
        assert_eq!(p_pr.style.as_deref(), Some("Heading1"));
        assert_eq!(p_pr.justification.as_deref(), Some("center"));
        let sp = p_pr.spacing.as_ref().unwrap();
        assert_eq!(sp.before, Some(240));
        assert_eq!(sp.after, Some(120));
        assert_eq!(sp.line, Some(360));
        let ind = p_pr.indentation.as_ref().unwrap();
        assert_eq!(ind.left, Some(720));
        assert_eq!(ind.right, Some(360));
        assert_eq!(ind.first_line, Some(360));
    } else {
        panic!("Expected Paragraph");
    }
}

#[test]
fn test_09_bookmarks_roundtrip() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();

    p.children
        .push(ParagraphChild::BookmarkStart(BookmarkStart {
            id: "0".to_string(),
            name: "Chapter1".to_string(),
            col_first: None,
            col_last: None,
        }));

    let mut r = Run::default();
    r.content
        .push(RunContent::Text("Bookmarked Section".to_string(), false));
    p.children.push(ParagraphChild::Run(r));

    p.children.push(ParagraphChild::BookmarkEnd(BookmarkEnd {
        id: "0".to_string(),
    }));

    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let xml_str = std::str::from_utf8(&xml).unwrap();
    assert!(xml_str.contains("<w:bookmarkStart w:id=\"0\" w:name=\"Chapter1\"/>"));
    assert!(xml_str.contains("<w:bookmarkEnd w:id=\"0\"/>"));

    let parsed = parse_document(&xml).expect("Parsing failed");
    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        assert_eq!(p.children.len(), 3);
        match &p.children[0] {
            ParagraphChild::BookmarkStart(b) => {
                assert_eq!(b.id, "0");
                assert_eq!(b.name, "Chapter1");
            }
            _ => panic!("Expected BookmarkStart"),
        }
        match &p.children[2] {
            ParagraphChild::BookmarkEnd(b) => {
                assert_eq!(b.id, "0");
            }
            _ => panic!("Expected BookmarkEnd"),
        }
    }
}

#[test]
fn test_10_hyperlinks_roundtrip() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();

    let mut run = Run::default();
    run.content
        .push(RunContent::Text("Click here".to_string(), false));

    let hl = Hyperlink {
        id: Some("rId4".to_string()),
        anchor: Some("section2".to_string()),
        tooltip: Some("Go to Section 2".to_string()),
        history: true,
        children: vec![ParagraphChild::Run(run)],
    };

    p.children.push(ParagraphChild::Hyperlink(hl));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let parsed = parse_document(&xml).expect("Parsing failed");
    assert_eq!(parsed.get_full_text(), "Click here");

    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        if let ParagraphChild::Hyperlink(ref hl) = p.children[0] {
            assert_eq!(hl.id.as_deref(), Some("rId4"));
            assert_eq!(hl.anchor.as_deref(), Some("section2"));
            assert_eq!(hl.tooltip.as_deref(), Some("Go to Section 2"));
            assert_eq!(hl.children.len(), 1);
        } else {
            panic!("Expected Hyperlink child");
        }
    }
}

#[test]
fn test_11_tracked_changes_insertions_and_deletions() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();

    let mut ins_run = Run::default();
    ins_run
        .content
        .push(RunContent::Text("Added text".to_string(), false));
    let ins = TrackedChange {
        id: "1".to_string(),
        author: "Alice".to_string(),
        date: "2026-10-28T12:00:00Z".to_string(),
        children: vec![ParagraphChild::Run(ins_run)],
    };

    let mut del_run = Run::default();
    del_run
        .content
        .push(RunContent::DeletedText("Removed text".to_string(), false));
    let del = TrackedChange {
        id: "2".to_string(),
        author: "Bob".to_string(),
        date: "2026-10-28T12:05:00Z".to_string(),
        children: vec![ParagraphChild::Run(del_run)],
    };

    p.children.push(ParagraphChild::Inserted(ins));
    p.children.push(ParagraphChild::Deleted(del));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let xml_str = std::str::from_utf8(&xml).unwrap();
    assert!(xml_str.contains("<w:ins w:id=\"1\" w:author=\"Alice\""));
    assert!(xml_str.contains("<w:del w:id=\"2\" w:author=\"Bob\""));

    let parsed = parse_document(&xml).expect("Parsing failed");
    assert_eq!(parsed.get_full_text(), "Added text");

    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        assert_eq!(p.children.len(), 2);
        match &p.children[0] {
            ParagraphChild::Inserted(i) => {
                assert_eq!(i.id, "1");
                assert_eq!(i.author, "Alice");
            }
            _ => panic!("Expected Inserted child"),
        }
        match &p.children[1] {
            ParagraphChild::Deleted(d) => {
                assert_eq!(d.id, "2");
                assert_eq!(d.author, "Bob");
            }
            _ => panic!("Expected Deleted child"),
        }
    }
}

#[test]
fn test_12_table_grid_and_cell_properties() {
    let mut doc = Document::default();
    let mut table = Table::default();

    table.grid = Some(TableGrid {
        columns: vec![4000, 5000],
    });

    let mut row = TableRow::default();
    for val in ["Cell A1", "Cell B1"] {
        let mut cell = TableCell::default();
        cell.width = Some(TableWidth {
            width: 4000,
            width_type: "dxa".to_string(),
        });
        let mut p = Paragraph::default();
        let mut r = Run::default();
        r.content.push(RunContent::Text(val.to_string(), false));
        p.children.push(ParagraphChild::Run(r));
        cell.elements.push(BodyElement::Paragraph(p));
        row.cells.push(cell);
    }
    table.rows.push(row);
    doc.body.elements.push(BodyElement::Table(table));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let parsed = parse_document(&xml).expect("Parsing failed");

    let full_text = parsed.get_full_text();
    assert!(full_text.contains("Cell A1"));
    assert!(full_text.contains("Cell B1"));

    if let BodyElement::Table(ref tbl) = parsed.body.elements[0] {
        assert_eq!(tbl.grid.as_ref().unwrap().columns.len(), 2);
        assert_eq!(tbl.rows.len(), 1);
        let cell = &tbl.rows[0].cells[0];
        let w = cell.width.as_ref().unwrap();
        assert_eq!(w.width, 4000);
    } else {
        panic!("Expected Table");
    }
}

#[test]
fn test_13_breaks_and_hyphenation() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();

    r.content
        .push(RunContent::Text("Page 1".to_string(), false));
    r.content.push(RunContent::Break(Some("page".to_string())));
    r.content
        .push(RunContent::Text("Page 2".to_string(), false));
    r.content.push(RunContent::Tab);
    r.content.push(RunContent::SoftHyphen);
    r.content.push(RunContent::NoBreakHyphen);

    p.children.push(ParagraphChild::Run(r));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_document(&doc).expect("Serialization failed");
    let xml_str = std::str::from_utf8(&xml).unwrap();
    assert!(xml_str.contains("<w:br w:type=\"page\"/>"));
    assert!(xml_str.contains("<w:tab/>"));
    assert!(xml_str.contains("<w:softHyphen/>"));
    assert!(xml_str.contains("<w:noBreakHyphen/>"));

    let parsed = parse_document(&xml).expect("Parsing failed");
    if let BodyElement::Paragraph(ref p) = parsed.body.elements[0] {
        if let ParagraphChild::Run(ref r) = p.children[0] {
            assert_eq!(r.content.len(), 6);
            assert!(matches!(&r.content[1], RunContent::Break(Some(t)) if t == "page"));
            assert!(matches!(&r.content[3], RunContent::Tab));
            assert!(matches!(&r.content[4], RunContent::SoftHyphen));
            assert!(matches!(&r.content[5], RunContent::NoBreakHyphen));
        }
    }
}

#[test]
fn test_14_section_properties() {
    let mut doc = Document::default();
    let sect = SectionProperties {
        header_refs: vec![HeaderFooterRef {
            ref_type: "default".to_string(),
            rel_id: "rId5".to_string(),
        }],
        footer_refs: vec![HeaderFooterRef {
            ref_type: "default".to_string(),
            rel_id: "rId6".to_string(),
        }],
        page_size: Some(PageSize {
            width: 12240,
            height: 15840,
            orientation: Some("portrait".to_string()),
        }),
        page_margin: Some(PageMargin {
            top: 1440,
            right: 1440,
            bottom: 1440,
            left: 1440,
            header: 720,
            footer: 720,
            gutter: 0,
        }),
        columns: None,
        doc_grid: None,
        raw_elements: Vec::new(),
    };
    doc.body.sect_pr = Some(sect);

    let xml = serialize_document(&doc).expect("Serialization failed");
    let parsed = parse_document(&xml).expect("Parsing failed");

    let sect_pr = parsed.body.sect_pr.as_ref().unwrap();
    let ps = sect_pr.page_size.as_ref().unwrap();
    assert_eq!(ps.width, 12240);
    assert_eq!(ps.height, 15840);
    assert_eq!(ps.orientation.as_deref(), Some("portrait"));

    let pm = sect_pr.page_margin.as_ref().unwrap();
    assert_eq!(pm.top, 1440);
    assert_eq!(pm.bottom, 1440);

    assert_eq!(sect_pr.header_refs.len(), 1);
    assert_eq!(sect_pr.header_refs[0].rel_id, "rId5");
    assert_eq!(sect_pr.footer_refs.len(), 1);
    assert_eq!(sect_pr.footer_refs[0].rel_id, "rId6");
}

#[test]
fn test_15_raw_xml_preservation() {
    let mut doc = Document::default();
    let mut raw = RawXmlNode::new("w:customBlock");
    raw.attributes
        .push(("customAttr".to_string(), "customValue".to_string()));
    let mut child = RawXmlNode::new("w:childElem");
    child.content = Some("custom inner text".to_string());
    raw.children.push(child);

    doc.unparsed_nodes.push(raw);

    let xml = serialize_document(&doc).expect("Serialization failed");
    let xml_str = std::str::from_utf8(&xml).unwrap();
    assert!(xml_str.contains("<w:customBlock customAttr=\"customValue\">"));
    assert!(xml_str.contains("<w:childElem>custom inner text</w:childElem>"));

    let parsed = parse_document(&xml).expect("Parsing failed");
    assert_eq!(parsed.unparsed_nodes.len(), 1);
    assert_eq!(parsed.unparsed_nodes[0].tag, "w:customBlock");
    assert_eq!(
        parsed.unparsed_nodes[0].attributes[0],
        ("customAttr".to_string(), "customValue".to_string())
    );
    assert_eq!(
        parsed.unparsed_nodes[0].children[0].content.as_deref(),
        Some("custom inner text")
    );
}

#[test]
fn test_16_header_document_roundtrip() {
    let mut hdr = HeaderDocument::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Confidential Document Header".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    hdr.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_header(&hdr).expect("Serialization failed");
    let parsed = parse_header(&xml).expect("Parsing failed");

    assert_eq!(parsed.get_text(), "Confidential Document Header");
}

#[test]
fn test_17_footer_document_roundtrip() {
    let mut ftr = FooterDocument::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content
        .push(RunContent::Text("Page 1 of 10".to_string(), false));
    p.children.push(ParagraphChild::Run(r));
    ftr.body.elements.push(BodyElement::Paragraph(p));

    let xml = serialize_footer(&ftr).expect("Serialization failed");
    let parsed = parse_footer(&xml).expect("Parsing failed");

    assert_eq!(parsed.get_text(), "Page 1 of 10");
}

#[test]
fn test_18_footnotes_document_roundtrip() {
    let mut fn_doc = FootnotesDocument::default();

    // Separator footnote
    let mut fn_sep = Footnote {
        id: -1,
        footnote_type: Some("separator".to_string()),
        body: Body::default(),
    };
    let mut sep_p = Paragraph::default();
    let mut sep_r = Run::default();
    sep_r.content.push(RunContent::Separator);
    sep_p.children.push(ParagraphChild::Run(sep_r));
    fn_sep.body.elements.push(BodyElement::Paragraph(sep_p));
    fn_doc.footnotes.push(fn_sep);

    // Regular footnote
    let mut fn_user = Footnote {
        id: 1,
        footnote_type: None,
        body: Body::default(),
    };
    let mut user_p = Paragraph::default();
    let mut user_r = Run::default();
    user_r.content.push(RunContent::Text(
        "ISO/IEC 29500:2012 Standard.".to_string(),
        false,
    ));
    user_p.children.push(ParagraphChild::Run(user_r));
    fn_user.body.elements.push(BodyElement::Paragraph(user_p));
    fn_doc.footnotes.push(fn_user);

    let xml = serialize_footnotes(&fn_doc).expect("Serialization failed");
    let parsed = parse_footnotes(&xml).expect("Parsing failed");

    assert_eq!(parsed.footnotes.len(), 2);
    assert_eq!(parsed.footnotes[0].id, -1);
    assert_eq!(
        parsed.footnotes[0].footnote_type.as_deref(),
        Some("separator")
    );
    assert_eq!(parsed.footnotes[1].id, 1);
    assert_eq!(
        parsed.footnotes[1].get_text(),
        "ISO/IEC 29500:2012 Standard."
    );
}

#[test]
fn test_19_endnotes_document_roundtrip() {
    let mut en_doc = EndnotesDocument::default();

    let mut en = Endnote {
        id: 1,
        endnote_type: None,
        body: Body::default(),
    };
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Bibliography reference.".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    en.body.elements.push(BodyElement::Paragraph(p));
    en_doc.endnotes.push(en);

    let xml = serialize_endnotes(&en_doc).expect("Serialization failed");
    let parsed = parse_endnotes(&xml).expect("Parsing failed");

    assert_eq!(parsed.endnotes.len(), 1);
    assert_eq!(parsed.endnotes[0].id, 1);
    assert_eq!(parsed.endnotes[0].get_text(), "Bibliography reference.");
}

#[test]
fn test_20_comments_document_roundtrip() {
    let mut cm_doc = CommentsDocument::default();
    let mut cm = Comment {
        id: "1".to_string(),
        author: "Chief Architect".to_string(),
        initials: Some("CA".to_string()),
        date: Some("2026-10-28T14:30:00Z".to_string()),
        body: Body::default(),
    };
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Review required for Section 3.".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    cm.body.elements.push(BodyElement::Paragraph(p));
    cm_doc.comments.push(cm);

    let xml = serialize_comments(&cm_doc).expect("Serialization failed");
    let parsed = parse_comments(&xml).expect("Parsing failed");

    assert_eq!(parsed.comments.len(), 1);
    assert_eq!(parsed.comments[0].id, "1");
    assert_eq!(parsed.comments[0].author, "Chief Architect");
    assert_eq!(parsed.comments[0].initials.as_deref(), Some("CA"));
    assert_eq!(
        parsed.comments[0].date.as_deref(),
        Some("2026-10-28T14:30:00Z")
    );
    assert_eq!(
        parsed.comments[0].get_text(),
        "Review required for Section 3."
    );
}

#[test]
fn test_21_baseline_docx_open_and_text_extraction() {
    let pkg = WordPackage::from_file(resolve_repo_path("specs/evidence/word/baseline.docx"))
        .expect("Failed to open baseline.docx");

    let full_text = pkg.get_full_text();
    println!("Baseline text:\n{}", full_text);

    assert!(full_text.contains("OpenOfficeRS Architecture Specification"));
    assert!(full_text.contains("This is a clean-room Rust engine conforming strictly to ECMA-376."));
    assert!(full_text.contains("Formatted text run with bold and italic properties."));
    assert!(full_text.contains("Feature"));
    assert!(full_text.contains("Typing Latency"));
    assert!(full_text.contains("< 1.0 ms"));

    // Verify unmodeled parts are captured in unparsed_store
    assert!(
        pkg.unparsed_store.contains("word/theme/theme1.xml")
            || pkg.unparsed_store.contains("/word/theme/theme1.xml")
    );
    assert!(
        pkg.unparsed_store.contains("word/styles.xml")
            || pkg.unparsed_store.contains("/word/styles.xml")
    );
    assert!(
        pkg.unparsed_store.contains("word/settings.xml")
            || pkg.unparsed_store.contains("/word/settings.xml")
    );
    assert!(
        pkg.unparsed_store.contains("word/fontTable.xml")
            || pkg.unparsed_store.contains("/word/fontTable.xml")
    );
}

#[test]
fn test_22_baseline_docx_piecetree_rope() {
    let pkg = WordPackage::from_file(resolve_repo_path("specs/evidence/word/baseline.docx"))
        .expect("Failed to open baseline.docx");

    let mut tree = pkg.create_piece_tree();
    assert!(!tree.is_empty());
    assert!(tree
        .get_text()
        .contains("OpenOfficeRS Architecture Specification"));

    // Perform interactive editing via Piece-Tree
    tree.insert(0, "[CONFIDENTIAL] ").unwrap();
    assert!(tree.get_text().starts_with("[CONFIDENTIAL] OpenOfficeRS"));

    let idx = tree.get_text().find("< 1.0 ms").unwrap();
    tree.delete(idx, 8).unwrap();
    tree.insert(idx, "< 0.5 ms (Exceeded SLA)").unwrap();

    assert!(tree.get_text().contains("< 0.5 ms (Exceeded SLA)"));
}

#[test]
fn test_23_baseline_docx_lossless_roundtrip() {
    let pkg = WordPackage::from_file(resolve_repo_path("specs/evidence/word/baseline.docx"))
        .expect("Failed to open baseline.docx");

    let original_text = pkg.get_full_text();

    // Round-trip to in-memory bytes
    let bytes = pkg.to_bytes().expect("Failed to serialize WordPackage");
    assert!(!bytes.is_empty());

    // Re-open from bytes
    let reloaded =
        WordPackage::from_bytes(&bytes).expect("Failed to reload WordPackage from bytes");
    let reloaded_text = reloaded.get_full_text();

    assert_eq!(reloaded_text, original_text);

    // Verify unmodeled parts survived the round-trip
    assert!(
        reloaded.unparsed_store.contains("word/theme/theme1.xml")
            || reloaded.unparsed_store.contains("/word/theme/theme1.xml")
    );
    assert!(
        reloaded.unparsed_store.contains("word/styles.xml")
            || reloaded.unparsed_store.contains("/word/styles.xml")
    );
    assert!(
        reloaded.unparsed_store.contains("word/settings.xml")
            || reloaded.unparsed_store.contains("/word/settings.xml")
    );
    assert!(
        reloaded.unparsed_store.contains("word/fontTable.xml")
            || reloaded.unparsed_store.contains("/word/fontTable.xml")
    );
}

#[test]
fn test_24_wordpackage_create_new_and_save() {
    let mut pkg = WordPackage::new();

    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Generated Document by OpenOfficeRS".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    pkg.document.body.elements.push(BodyElement::Paragraph(p));

    let bytes = pkg.to_bytes().expect("Failed to save new WordPackage");
    assert!(!bytes.is_empty());

    let reloaded = WordPackage::from_bytes(&bytes).expect("Failed to open newly saved package");
    assert_eq!(
        reloaded.get_full_text(),
        "Generated Document by OpenOfficeRS"
    );
}

#[test]
fn test_25_unmodeled_parts_preservation() {
    let mut pkg = WordPackage::new();

    // Add a custom unmodeled part (e.g. customXml or VBA)
    let custom_xml = b"<customRoot><item key=\"k1\" value=\"v1\"/></customRoot>";
    pkg.unparsed_store.insert(UnparsedPart::new(
        "customXml/item1.xml",
        custom_xml.to_vec(),
        Some("application/xml".to_string()),
        None,
    ));

    let bytes = pkg
        .to_bytes()
        .expect("Failed to save package with custom part");
    let reloaded = WordPackage::from_bytes(&bytes).expect("Failed to reload package");

    let retrieved = reloaded
        .unparsed_store
        .get("customXml/item1.xml")
        .expect("Custom unmodeled part must be preserved");

    assert_eq!(retrieved.content, custom_xml);
    assert_eq!(retrieved.content_type.as_deref(), Some("application/xml"));
}

#[test]
fn test_26_vectors_manifest_validation() {
    let json_bytes = std::fs::read(resolve_repo_path("specs/evidence/word/3.1.1_vectors.json"))
        .expect("Must read specs/evidence/word/3.1.1_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let count = vectors["vectors"].as_array().expect("vectors array").len();
    assert!(
        count >= 25,
        "Subphase 3.1.1 requires at least 25 test vectors, found: {}",
        count
    );

    // Verify subphase id and title
    assert_eq!(vectors["subphase_id"].as_str(), Some("3.1.1"));
    assert_eq!(
        vectors["title"].as_str(),
        Some("WordprocessingML Piece-Tree Rope DOM & Sub-Document Parser")
    );
}
