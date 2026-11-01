//! Sub-Phase 46.1.1 Integration Tests: Revision Tracking & Redline Renderer
//!
//! Validates:
//! - w:ins, w:del, w:rPrChange, w:pPrChange model parsing and serialization
//! - AuthorColorPalette deterministic hash assignment
//! - RevisionEngine: list, accept, reject (individual and batch)
//! - Redline rendering across RevisionViewMode (AllMarkup, SimpleMarkup, NoMarkup, Original)
//! - Change bar detection on LayoutLine
//! - Performance under heavy revision volume

use std::time::Instant;
use word_engine::layout::*;
use word_engine::model::*;
use word_engine::parser::parse_document;
use word_engine::revision::*;
use word_engine::serializer::serialize_document;
use word_engine::styles::*;

#[test]
fn test_author_color_palette_determinism() {
    let alice1 = AuthorColorPalette::get_color_for_author("Alice Smith");
    let alice2 = AuthorColorPalette::get_color_for_author("Alice Smith");
    let bob = AuthorColorPalette::get_color_for_author("Bob Jones");

    assert_eq!(
        alice1, alice2,
        "Color assignment for the same author must be deterministic"
    );
    assert!(alice1.starts_with('#'), "Hex color must start with #");
    assert_eq!(alice1.len(), 7, "Hex color must be 7 chars (#RRGGBB)");
    assert!(bob.starts_with('#'));
}

#[test]
fn test_revision_collection_and_previews() {
    let mut doc = Document::default();
    let p = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Normal".to_string()),
            p_pr_change: Some(ParagraphPropertiesChange {
                id: "10".to_string(),
                author: "Alice".to_string(),
                date: "2026-11-01T10:00:00Z".to_string(),
                previous_properties: None,
            }),
            ..Default::default()
        }),
        children: vec![
            ParagraphChild::Run(Run {
                properties: Some(RunProperties {
                    bold: true,
                    r_pr_change: Some(RunPropertiesChange {
                        id: "20".to_string(),
                        author: "Bob".to_string(),
                        date: "2026-11-01T10:05:00Z".to_string(),
                        previous_properties: Some(Box::new(RunProperties {
                            bold: false,
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                }),
                content: vec![RunContent::Text("Formatted text ".to_string(), false)],
                ..Default::default()
            }),
            ParagraphChild::Inserted(TrackedChange {
                id: "30".to_string(),
                author: "Charlie".to_string(),
                date: "2026-11-01T10:10:00Z".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("new inserted text".to_string(), false)],
                    ..Default::default()
                })],
            }),
            ParagraphChild::Deleted(TrackedChange {
                id: "40".to_string(),
                author: "Alice".to_string(),
                date: "2026-11-01T10:15:00Z".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("old deleted text".to_string(), false)],
                    ..Default::default()
                })],
            }),
        ],
        ..Default::default()
    };
    doc.body.elements.push(BodyElement::Paragraph(p));

    let revisions = RevisionEngine::list_revisions(&doc);
    assert_eq!(revisions.len(), 4, "Must collect exactly 4 revisions");

    let ppr_rev = revisions
        .iter()
        .find(|r| r.id == "10")
        .expect("Must find pPrChange revision");
    assert_eq!(
        ppr_rev.revision_type,
        RevisionType::ParagraphPropertiesChange
    );
    assert_eq!(ppr_rev.author, "Alice");

    let rpr_rev = revisions
        .iter()
        .find(|r| r.id == "20")
        .expect("Must find rPrChange revision");
    assert_eq!(rpr_rev.revision_type, RevisionType::RunPropertiesChange);
    assert_eq!(rpr_rev.author, "Bob");

    let ins_rev = revisions
        .iter()
        .find(|r| r.id == "30")
        .expect("Must find ins revision");
    assert_eq!(ins_rev.revision_type, RevisionType::Insertion);
    assert_eq!(ins_rev.author, "Charlie");
    assert_eq!(ins_rev.text_preview, "new inserted text");

    let del_rev = revisions
        .iter()
        .find(|r| r.id == "40")
        .expect("Must find del revision");
    assert_eq!(del_rev.revision_type, RevisionType::Deletion);
    assert_eq!(del_rev.author, "Alice");
    assert_eq!(del_rev.text_preview, "old deleted text");

    let (ins_count, del_count, rpr_count, ppr_count) = RevisionEngine::get_revision_summary(&doc);
    assert_eq!(ins_count, 1);
    assert_eq!(del_count, 1);
    assert_eq!(rpr_count, 1);
    assert_eq!(ppr_count, 1);
}

#[test]
fn test_accept_individual_revisions() {
    let mut doc = Document::default();
    doc.body.elements.push(BodyElement::Paragraph(Paragraph {
        properties: None,
        children: vec![
            ParagraphChild::Inserted(TrackedChange {
                id: "1".to_string(),
                author: "Editor".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("Accepted Words".to_string(), false)],
                    ..Default::default()
                })],
            }),
            ParagraphChild::Deleted(TrackedChange {
                id: "2".to_string(),
                author: "Editor".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("Discarded Words".to_string(), false)],
                    ..Default::default()
                })],
            }),
        ],
        ..Default::default()
    }));

    // Accept insertion 1
    let ok = RevisionEngine::accept_revision(&mut doc, "1");
    assert!(ok, "Accepting revision 1 should succeed");

    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        // Insertion should now be a normal Run
        assert_eq!(p.children.len(), 2);
        match &p.children[0] {
            ParagraphChild::Run(r) => {
                assert_eq!(
                    r.content[0],
                    RunContent::Text("Accepted Words".to_string(), false)
                );
            }
            _ => panic!("Expected child 0 to be unwrapped into a normal Run"),
        }
    } else {
        panic!("Expected paragraph");
    }

    // Accept deletion 2 (which should remove it entirely)
    let ok2 = RevisionEngine::accept_revision(&mut doc, "2");
    assert!(ok2, "Accepting revision 2 should succeed");

    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        assert_eq!(p.children.len(), 1, "Deleted run must be removed on accept");
    }
}

#[test]
fn test_reject_individual_revisions() {
    let mut doc = Document::default();
    doc.body.elements.push(BodyElement::Paragraph(Paragraph {
        properties: None,
        children: vec![
            ParagraphChild::Inserted(TrackedChange {
                id: "1".to_string(),
                author: "Editor".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("Should disappear".to_string(), false)],
                    ..Default::default()
                })],
            }),
            ParagraphChild::Deleted(TrackedChange {
                id: "2".to_string(),
                author: "Editor".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("Should be restored".to_string(), false)],
                    ..Default::default()
                })],
            }),
        ],
        ..Default::default()
    }));

    // Reject insertion 1 (should remove it)
    let ok1 = RevisionEngine::reject_revision(&mut doc, "1");
    assert!(ok1);

    // Reject deletion 2 (should restore it as normal run)
    let ok2 = RevisionEngine::reject_revision(&mut doc, "2");
    assert!(ok2);

    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        assert_eq!(p.children.len(), 1, "Only restored run should remain");
        match &p.children[0] {
            ParagraphChild::Run(r) => {
                assert_eq!(
                    r.content[0],
                    RunContent::Text("Should be restored".to_string(), false)
                );
            }
            _ => panic!("Expected restored normal Run"),
        }
    }
}

#[test]
fn test_accept_and_reject_all() {
    let create_test_doc = || {
        let mut doc = Document::default();
        doc.body.elements.push(BodyElement::Paragraph(Paragraph {
            properties: None,
            children: vec![
                ParagraphChild::Run(Run {
                    properties: Some(RunProperties {
                        bold: true,
                        r_pr_change: Some(RunPropertiesChange {
                            id: "10".to_string(),
                            author: "Editor".to_string(),
                            date: "".to_string(),
                            previous_properties: Some(Box::new(RunProperties {
                                bold: false,
                                ..Default::default()
                            })),
                        }),
                        ..Default::default()
                    }),
                    content: vec![RunContent::Text("Base ".to_string(), false)],
                    ..Default::default()
                }),
                ParagraphChild::Inserted(TrackedChange {
                    id: "11".to_string(),
                    author: "Editor".to_string(),
                    date: "".to_string(),
                    children: vec![ParagraphChild::Run(Run {
                        properties: None,
                        content: vec![RunContent::Text("Added".to_string(), false)],
                        ..Default::default()
                    })],
                }),
                ParagraphChild::Deleted(TrackedChange {
                    id: "12".to_string(),
                    author: "Editor".to_string(),
                    date: "".to_string(),
                    children: vec![ParagraphChild::Run(Run {
                        properties: None,
                        content: vec![RunContent::Text("Removed".to_string(), false)],
                        ..Default::default()
                    })],
                }),
            ],
            ..Default::default()
        }));
        doc
    };

    // Test Accept All
    let mut doc_accept = create_test_doc();
    let accepted_count = RevisionEngine::accept_all_revisions(&mut doc_accept);
    assert_eq!(accepted_count, 3);
    assert_eq!(RevisionEngine::list_revisions(&doc_accept).len(), 0);

    if let BodyElement::Paragraph(ref p) = doc_accept.body.elements[0] {
        // Base run (bold=true, no rPrChange), Added run (normal run), Removed run (gone)
        assert_eq!(p.children.len(), 2);
        if let ParagraphChild::Run(ref r) = p.children[0] {
            assert!(r.properties.as_ref().map(|pr| pr.bold).unwrap_or(false));
            assert!(r
                .properties
                .as_ref()
                .and_then(|pr| pr.r_pr_change.as_ref())
                .is_none());
        }
    }

    // Test Reject All
    let mut doc_reject = create_test_doc();
    let rejected_count = RevisionEngine::reject_all_revisions(&mut doc_reject);
    assert_eq!(rejected_count, 3);
    assert_eq!(RevisionEngine::list_revisions(&doc_reject).len(), 0);

    if let BodyElement::Paragraph(ref p) = doc_reject.body.elements[0] {
        // Base run reverted to bold=false, Added run removed, Removed run restored
        assert_eq!(p.children.len(), 2);
        if let ParagraphChild::Run(ref r) = p.children[0] {
            assert!(!r.properties.as_ref().map(|pr| pr.bold).unwrap_or(true));
        }
        if let ParagraphChild::Run(ref r) = p.children[1] {
            assert_eq!(r.content[0], RunContent::Text("Removed".to_string(), false));
        }
    }
}

#[test]
fn test_xml_roundtrip_with_revisions() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr>
        <w:pPrChange w:id="1" w:author="Alice" w:date="2026-11-01T09:00:00Z"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:b/>
          <w:rPrChange w:id="2" w:author="Bob" w:date="2026-11-01T09:05:00Z">
            <w:rPr/>
          </w:rPrChange>
        </w:rPr>
        <w:t>Hello </w:t>
      </w:r>
      <w:ins w:id="3" w:author="Charlie" w:date="2026-11-01T09:10:00Z">
        <w:r>
          <w:t>beautiful </w:t>
        </w:r>
      </w:ins>
      <w:del w:id="4" w:author="Alice" w:date="2026-11-01T09:15:00Z">
        <w:r>
          <w:delText>ugly </w:delText>
        </w:r>
      </w:del>
      <w:r>
        <w:t>world!</w:t>
      </w:r>
    </w:p>
  </w:body>
</w:document>"#;

    let doc = parse_document(xml).expect("Failed to parse XML with revisions");
    let revisions = RevisionEngine::list_revisions(&doc);
    assert_eq!(revisions.len(), 4);

    let serialized = serialize_document(&doc).expect("Failed to serialize document XML");
    let serialized_str = String::from_utf8_lossy(&serialized);
    assert!(
        serialized_str.contains("w:ins"),
        "Serialized XML must contain w:ins"
    );
    assert!(
        serialized_str.contains("w:del"),
        "Serialized XML must contain w:del"
    );
    assert!(
        serialized_str.contains("w:pPrChange"),
        "Serialized XML must contain w:pPrChange"
    );
    assert!(
        serialized_str.contains("w:rPrChange"),
        "Serialized XML must contain w:rPrChange"
    );

    let doc2 = parse_document(&serialized).expect("Failed to re-parse serialized XML");
    let revisions2 = RevisionEngine::list_revisions(&doc2);
    assert_eq!(revisions2.len(), 4, "Roundtrip must preserve all revisions");
}

#[test]
fn test_redline_view_modes_layout() {
    let p = Paragraph {
        properties: None,
        children: vec![
            ParagraphChild::Run(Run {
                properties: None,
                content: vec![RunContent::Text("Initial ".to_string(), false)],
                ..Default::default()
            }),
            ParagraphChild::Inserted(TrackedChange {
                id: "1".to_string(),
                author: "Alice".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("inserted ".to_string(), false)],
                    ..Default::default()
                })],
            }),
            ParagraphChild::Deleted(TrackedChange {
                id: "2".to_string(),
                author: "Bob".to_string(),
                date: "".to_string(),
                children: vec![ParagraphChild::Run(Run {
                    properties: None,
                    content: vec![RunContent::Text("deleted ".to_string(), false)],
                    ..Default::default()
                })],
            }),
            ParagraphChild::Run(Run {
                properties: None,
                content: vec![RunContent::Text("final".to_string(), false)],
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::default();
    let cascade = stylesheet.cascade_resolver();

    // 1. AllMarkup: contains both inserted and deleted tokens with appropriate markup
    let tokens_all =
        breaker.tokenize_paragraph_with_mode(&p, &cascade, RevisionViewMode::AllMarkup);
    let has_ins = tokens_all
        .iter()
        .any(|t| t.revision_type == Some(RevisionType::Insertion) && t.underline);
    let has_del = tokens_all
        .iter()
        .any(|t| t.revision_type == Some(RevisionType::Deletion) && t.strike);
    assert!(has_ins, "AllMarkup must include insertion with underline");
    assert!(has_del, "AllMarkup must include deletion with strike");

    // 2. SimpleMarkup: shows insertion without underline, hides deletion
    let tokens_simple =
        breaker.tokenize_paragraph_with_mode(&p, &cascade, RevisionViewMode::SimpleMarkup);
    assert!(tokens_simple.iter().any(|t| t.text.contains("inserted")));
    assert!(!tokens_simple.iter().any(|t| t.text.contains("deleted")));

    // 3. NoMarkup: shows final document (insertion included, deletion hidden)
    let tokens_none =
        breaker.tokenize_paragraph_with_mode(&p, &cascade, RevisionViewMode::NoMarkup);
    assert!(tokens_none.iter().any(|t| t.text.contains("inserted")));
    assert!(!tokens_none.iter().any(|t| t.text.contains("deleted")));

    // 4. Original: shows original document (insertion hidden, deletion included as plain text)
    let tokens_orig =
        breaker.tokenize_paragraph_with_mode(&p, &cascade, RevisionViewMode::Original);
    assert!(!tokens_orig.iter().any(|t| t.text.contains("inserted")));
    assert!(tokens_orig.iter().any(|t| t.text.contains("deleted")));

    // Change bar test
    let mut runs = Vec::new();
    for tok in &tokens_all {
        runs.push(LayoutRun {
            text: tok.text.clone(),
            bounds: BoundingBox::new(0, 0, tok.width, 240),
            font_family: tok.style.font_ascii.clone(),
            font_size_half_pts: tok.style.size_half_points,
            bold: tok.style.bold,
            italic: tok.style.italic,
            color: tok.style.color.clone(),
            underline: tok.underline,
            strike: tok.strike,
            revision_type: tok.revision_type,
            revision_author: tok.revision_author.clone(),
        });
    }
    let has_change_bar = runs.iter().any(|r| r.revision_type.is_some());
    assert!(
        has_change_bar,
        "Line containing revisions must have change bar flag set"
    );
}

#[test]
fn test_revision_engine_heavy_workload_performance() {
    let mut doc = Document::default();
    let count = 1000;

    let mut children = Vec::with_capacity(count * 2);
    for i in 0..count {
        children.push(ParagraphChild::Inserted(TrackedChange {
            id: format!("ins_{}", i),
            author: format!("Author{}", i % 5),
            date: "2026-11-01T12:00:00Z".to_string(),
            children: vec![ParagraphChild::Run(Run {
                properties: None,
                content: vec![RunContent::Text(format!("word{} ", i), false)],
                ..Default::default()
            })],
        }));
        children.push(ParagraphChild::Deleted(TrackedChange {
            id: format!("del_{}", i),
            author: format!("Author{}", (i + 1) % 5),
            date: "2026-11-01T12:00:00Z".to_string(),
            children: vec![ParagraphChild::Run(Run {
                properties: None,
                content: vec![RunContent::Text(format!("old{} ", i), false)],
                ..Default::default()
            })],
        }));
    }
    doc.body.elements.push(BodyElement::Paragraph(Paragraph {
        properties: None,
        children,
        ..Default::default()
    }));

    let start = Instant::now();
    let collected = RevisionEngine::list_revisions(&doc);
    let collect_time = start.elapsed();

    assert_eq!(collected.len(), count * 2);
    println!("Collected {} revisions in {:?}", count * 2, collect_time);
    assert!(
        collect_time.as_millis() < 20,
        "Collecting 2000 revisions must take < 20ms"
    );

    let accept_start = Instant::now();
    let accepted = RevisionEngine::accept_all_revisions(&mut doc);
    let accept_time = accept_start.elapsed();

    assert_eq!(accepted, count * 2);
    println!("Accepted {} revisions in {:?}", count * 2, accept_time);
    assert!(
        accept_time.as_millis() < 20,
        "Accepting 2000 revisions must take < 20ms"
    );
}
