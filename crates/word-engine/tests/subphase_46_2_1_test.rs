//! Sub-Phase 46.2.1 Integration Tests: Dynamic Table of Contents, Footnotes/Endnotes & Bibliography Engine
//!
//! Validates:
//! - Heading detection (outline_level and styles) and TOC extraction.
//! - TOC paragraph generation with bookmarks, hyperlinks, dot leaders, and page numbers.
//! - TOC update and in-place replacement in document body.
//! - Footnote/Endnote numbering format conversions (Arabic, Roman, Letters, Symbols).
//! - BibTeX parser across multiple source types.
//! - Inline citation formatting (APA, IEEE, MLA, Chicago).
//! - Complete Bibliography generation and paragraph styling.
//! - Performance SLA under high volume.

use std::collections::HashMap;
use std::time::Instant;
use word_engine::citations::*;
use word_engine::model::*;
use word_engine::styles::*;
use word_engine::sub_documents::*;
use word_engine::toc::*;

#[test]
fn test_toc_heading_detection_and_extraction() {
    let mut doc = Document::default();
    let stylesheet = StyleSheet::default();
    let cascade = stylesheet.cascade_resolver();

    // 1. Heading 1 via style
    let mut p1 = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Heading 1".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r1 = Run::default();
    r1.content.push(RunContent::Text(
        "Introduction & Executive Summary".to_string(),
        false,
    ));
    p1.children.push(ParagraphChild::Run(r1));
    doc.body.elements.push(BodyElement::Paragraph(p1));

    // 2. Normal text
    let mut p_body = Paragraph::default();
    let mut r_body = Run::default();
    r_body.content.push(RunContent::Text(
        "This is standard body text that must not appear in TOC.".to_string(),
        false,
    ));
    p_body.children.push(ParagraphChild::Run(r_body));
    doc.body.elements.push(BodyElement::Paragraph(p_body));

    // 3. Heading 2 via style
    let mut p2 = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Heading 2".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r2 = Run::default();
    r2.content.push(RunContent::Text(
        "Methodology & Experimental Setup".to_string(),
        false,
    ));
    p2.children.push(ParagraphChild::Run(r2));
    doc.body.elements.push(BodyElement::Paragraph(p2));

    // 4. Heading 3 via outline_level = Some(2) (0-based in OpenXML => level 3)
    let mut p3 = Paragraph {
        properties: Some(ParagraphProperties {
            outline_level: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r3 = Run::default();
    r3.content.push(RunContent::Text(
        "Data Collection Procedures".to_string(),
        false,
    ));
    p3.children.push(ParagraphChild::Run(r3));
    doc.body.elements.push(BodyElement::Paragraph(p3));

    let mut page_map = HashMap::new();
    page_map.insert(0, 1);
    page_map.insert(2, 4);
    page_map.insert(3, 7);

    let headings = TocEngine::extract_headings(&doc, &cascade, Some(&page_map));
    assert_eq!(headings.len(), 3, "Must extract exactly 3 headings");

    assert_eq!(headings[0].level, 1);
    assert_eq!(headings[0].title, "Introduction & Executive Summary");
    assert_eq!(headings[0].page_number, 1);
    assert_eq!(headings[0].bookmark_name, "_Toc_0");

    assert_eq!(headings[1].level, 2);
    assert_eq!(headings[1].title, "Methodology & Experimental Setup");
    assert_eq!(headings[1].page_number, 4);
    assert_eq!(headings[1].bookmark_name, "_Toc_2");

    assert_eq!(headings[2].level, 3);
    assert_eq!(headings[2].title, "Data Collection Procedures");
    assert_eq!(headings[2].page_number, 7);
    assert_eq!(headings[2].bookmark_name, "_Toc_3");
}

#[test]
fn test_toc_generation_with_options() {
    let entries = vec![
        TocEntry {
            level: 1,
            title: "Chapter 1: Foundations".to_string(),
            page_number: 1,
            bookmark_name: "_Toc_Ch1".to_string(),
            paragraph_index: 0,
        },
        TocEntry {
            level: 2,
            title: "1.1 Architectural Overview".to_string(),
            page_number: 3,
            bookmark_name: "_Toc_Ch1_1".to_string(),
            paragraph_index: 2,
        },
        TocEntry {
            level: 3,
            title: "1.1.1 Memory Subsystem".to_string(),
            page_number: 5,
            bookmark_name: "_Toc_Ch1_1_1".to_string(),
            paragraph_index: 4,
        },
    ];

    // Filter to levels 1..=2 (excluding level 3)
    let options = TocOptions {
        start_level: 1,
        end_level: 2,
        use_hyperlinks: true,
        show_page_numbers: true,
        right_align_page_numbers: true,
        tab_leader: TabLeader::Dot,
    };

    let paragraphs = TocEngine::generate_toc_paragraphs(&entries, &options);
    // 1 TOC Heading + 2 entries = 3 paragraphs
    assert_eq!(paragraphs.len(), 3);

    // Verify header
    assert_eq!(paragraphs[0].get_text(), "Table of Contents");

    // Verify first entry
    let p_entry1 = &paragraphs[1];
    assert_eq!(
        p_entry1.properties.as_ref().unwrap().style.as_deref(),
        Some("TOC 1")
    );
    assert_eq!(p_entry1.children.len(), 1);

    if let ParagraphChild::Hyperlink(ref h) = p_entry1.children[0] {
        assert_eq!(h.anchor.as_deref(), Some("_Toc_Ch1"));
        assert_eq!(h.children.len(), 3); // Title, Tab, Page Number
        assert_eq!(h.children[0].get_text(), "Chapter 1: Foundations");
        assert_eq!(h.children[2].get_text(), "1");
    } else {
        panic!("Entry 1 child must be a Hyperlink");
    }

    // Verify second entry
    let p_entry2 = &paragraphs[2];
    assert_eq!(
        p_entry2.properties.as_ref().unwrap().style.as_deref(),
        Some("TOC 2")
    );
    if let ParagraphChild::Hyperlink(ref h) = p_entry2.children[0] {
        assert_eq!(h.anchor.as_deref(), Some("_Toc_Ch1_1"));
        assert_eq!(h.children[0].get_text(), "1.1 Architectural Overview");
        assert_eq!(h.children[2].get_text(), "3");
    } else {
        panic!("Entry 2 child must be a Hyperlink");
    }
}

#[test]
fn test_toc_update_and_bookmark_insertion() {
    let mut doc = Document::default();
    let stylesheet = StyleSheet::default();
    let cascade = stylesheet.cascade_resolver();

    let mut p_h1 = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Heading 1".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r1 = Run::default();
    r1.content
        .push(RunContent::Text("Initial Chapter".to_string(), false));
    p_h1.children.push(ParagraphChild::Run(r1));
    doc.body.elements.push(BodyElement::Paragraph(p_h1));

    let options = TocOptions::default();

    // 1. Initial TOC insertion
    let (entries_count, toc_p_count) =
        TocEngine::update_toc_fields(&mut doc, &cascade, None, &options);
    assert_eq!(entries_count, 1);
    assert_eq!(toc_p_count, 2); // Header + 1 entry
    assert_eq!(doc.body.elements.len(), 3); // TOC Header + 1 TOC entry + Original Heading 1

    // Verify target heading now contains BookmarkStart and BookmarkEnd
    if let BodyElement::Paragraph(ref heading_p) = doc.body.elements[2] {
        let has_bm_start = heading_p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::BookmarkStart(ref b) if b.name == "_Toc_0"));
        let has_bm_end = heading_p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::BookmarkEnd(_)));
        assert!(has_bm_start, "Heading must be decorated with BookmarkStart");
        assert!(has_bm_end, "Heading must be decorated with BookmarkEnd");
    } else {
        panic!("Element 2 must be paragraph");
    }

    // 2. Add second heading and re-update TOC: must replace existing TOC in place without duplication
    let mut p_h2 = Paragraph {
        properties: Some(ParagraphProperties {
            style: Some("Heading 1".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut r2 = Run::default();
    r2.content
        .push(RunContent::Text("Second Chapter".to_string(), false));
    p_h2.children.push(ParagraphChild::Run(r2));
    doc.body.elements.push(BodyElement::Paragraph(p_h2));

    let (entries_count_2, toc_p_count_2) =
        TocEngine::update_toc_fields(&mut doc, &cascade, None, &options);
    assert_eq!(entries_count_2, 2);
    assert_eq!(toc_p_count_2, 3); // Header + 2 entries
    assert_eq!(doc.body.elements.len(), 5); // 3 TOC paragraphs + 2 original headings
}

#[test]
fn test_footnote_and_endnote_creation_and_numbering() {
    let fn1 = Footnote::new(1, "First footnote commentary.");
    assert_eq!(fn1.id, 1);
    assert_eq!(fn1.get_text(), " First footnote commentary.");

    let en1 = Endnote::new(10, "Endnote citation details.");
    assert_eq!(en1.id, 10);
    assert_eq!(en1.get_text(), " Endnote citation details.");

    // Arabic format
    let arabic = FootnoteNumberingFormat::Arabic;
    assert_eq!(arabic.format_index(1), "1");
    assert_eq!(arabic.format_index(42), "42");

    // Roman lower & upper
    let roman_lower = FootnoteNumberingFormat::RomanLower;
    assert_eq!(roman_lower.format_index(1), "i");
    assert_eq!(roman_lower.format_index(4), "iv");
    assert_eq!(roman_lower.format_index(9), "ix");
    assert_eq!(roman_lower.format_index(14), "xiv");

    let roman_upper = FootnoteNumberingFormat::RomanUpper;
    assert_eq!(roman_upper.format_index(1), "I");
    assert_eq!(roman_upper.format_index(4), "IV");
    assert_eq!(roman_upper.format_index(14), "XIV");

    // Letter lower & upper
    let letter_lower = FootnoteNumberingFormat::LetterLower;
    assert_eq!(letter_lower.format_index(1), "a");
    assert_eq!(letter_lower.format_index(26), "z");

    let letter_upper = FootnoteNumberingFormat::LetterUpper;
    assert_eq!(letter_upper.format_index(1), "A");
    assert_eq!(letter_upper.format_index(2), "B");

    // Symbol format
    let symbol = FootnoteNumberingFormat::Symbol;
    assert_eq!(symbol.format_index(1), "*");
    assert_eq!(symbol.format_index(2), "†");
    assert_eq!(symbol.format_index(3), "‡");
    assert_eq!(symbol.format_index(4), "§");
    assert_eq!(symbol.format_index(7), "**");
}

#[test]
fn test_bibtex_parser() {
    let bibtex_sample = r#"
@article{Knuth1984,
  author = {Donald E. Knuth},
  title = {Literate Programming},
  journal = {The Computer Journal},
  year = {1984},
  volume = {27},
  number = {2},
  pages = {97--111},
  doi = {10.1093/comjnl/27.2.97}
}

@book{Lamport1994,
  author = {Leslie Lamport},
  title = {LaTeX: A Document Preparation System},
  publisher = {Addison-Wesley},
  year = {1994}
}

@inproceedings{Shannon1948,
  author = {Claude E. Shannon and Warren Weaver},
  title = {A Mathematical Theory of Communication},
  year = {1948}
}
"#;

    let sources = CitationEngine::parse_bibtex(bibtex_sample);
    assert_eq!(sources.len(), 3, "Must parse exactly 3 BibTeX sources");

    // Knuth entry
    let knuth = &sources[0];
    assert_eq!(knuth.id, "Knuth1984");
    assert_eq!(knuth.source_type, SourceType::Article);
    assert_eq!(knuth.authors, vec!["Donald E. Knuth"]);
    assert_eq!(knuth.title, "Literate Programming");
    assert_eq!(knuth.year, Some(1984));
    assert_eq!(knuth.journal.as_deref(), Some("The Computer Journal"));
    assert_eq!(knuth.volume.as_deref(), Some("27"));
    assert_eq!(knuth.issue.as_deref(), Some("2"));
    assert_eq!(knuth.pages.as_deref(), Some("97--111"));
    assert_eq!(knuth.doi.as_deref(), Some("10.1093/comjnl/27.2.97"));

    // Lamport entry
    let lamport = &sources[1];
    assert_eq!(lamport.id, "Lamport1994");
    assert_eq!(lamport.source_type, SourceType::Book);
    assert_eq!(lamport.publisher.as_deref(), Some("Addison-Wesley"));
    assert_eq!(lamport.year, Some(1994));

    // Shannon entry
    let shannon = &sources[2];
    assert_eq!(shannon.id, "Shannon1948");
    assert_eq!(shannon.source_type, SourceType::InProceedings);
    assert_eq!(shannon.authors.len(), 2);
    assert_eq!(shannon.authors[0], "Claude E. Shannon");
    assert_eq!(shannon.authors[1], "Warren Weaver");
}

#[test]
fn test_citation_formatting_styles() {
    let single_author = CitationSource {
        id: "Knuth1984".to_string(),
        source_type: SourceType::Article,
        authors: vec!["Donald E. Knuth".to_string()],
        title: "Literate Programming".to_string(),
        year: Some(1984),
        ..Default::default()
    };

    let two_authors = CitationSource {
        id: "Shannon1948".to_string(),
        source_type: SourceType::InProceedings,
        authors: vec!["Claude E. Shannon".to_string(), "Warren Weaver".to_string()],
        title: "A Mathematical Theory of Communication".to_string(),
        year: Some(1948),
        ..Default::default()
    };

    let multi_authors = CitationSource {
        id: "Smith2026".to_string(),
        source_type: SourceType::Article,
        authors: vec![
            "Alice Smith".to_string(),
            "Bob Jones".to_string(),
            "Charlie Brown".to_string(),
        ],
        title: "Distributed Consensus in Collaborative Editors".to_string(),
        year: Some(2026),
        ..Default::default()
    };

    // 1. IEEE format: [index]
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Ieee, None, Some(1)),
        "[1]"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Ieee, Some("42"), Some(1)),
        "[1, p. 42]"
    );

    // 2. APA format: (Author, Year)
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Apa, None, None),
        "(Knuth, 1984)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&two_authors], CitationStyle::Apa, None, None),
        "(Shannon & Weaver, 1948)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&multi_authors], CitationStyle::Apa, None, None),
        "(Smith et al., 2026)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Apa, Some("105"), None),
        "(Knuth, 1984, p. 105)"
    );

    // 3. MLA format: (Author Page)
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Mla, None, None),
        "(Knuth)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Mla, Some("99"), None),
        "(Knuth 99)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&two_authors], CitationStyle::Mla, Some("12"), None),
        "(Shannon & Weaver 12)"
    );

    // 4. Chicago format: (Author Year)
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Chicago, None, None),
        "(Knuth 1984)"
    );
    assert_eq!(
        CitationEngine::format_inline(&[&single_author], CitationStyle::Chicago, Some("101"), None),
        "(Knuth 1984, 101)"
    );
}

#[test]
fn test_bibliography_generation_and_document_insertion() {
    let sources = vec![CitationSource {
        id: "Knuth1984".to_string(),
        source_type: SourceType::Article,
        authors: vec!["Donald E. Knuth".to_string()],
        title: "Literate Programming".to_string(),
        year: Some(1984),
        journal: Some("The Computer Journal".to_string()),
        volume: Some("27".to_string()),
        issue: Some("2".to_string()),
        pages: Some("97-111".to_string()),
        doi: Some("10.1093/comjnl/27.2.97".to_string()),
        ..Default::default()
    }];

    // APA Bibliography
    let apa_paragraphs =
        CitationEngine::generate_bibliography_paragraphs(&sources, CitationStyle::Apa);
    assert_eq!(apa_paragraphs.len(), 2);
    assert_eq!(apa_paragraphs[0].get_text(), "Bibliography");
    let apa_entry_text = apa_paragraphs[1].get_text();
    assert!(apa_entry_text.contains("Donald E. Knuth"));
    assert!(apa_entry_text.contains("(1984)"));
    assert!(apa_entry_text.contains("Literate Programming"));

    // IEEE References
    let ieee_paragraphs =
        CitationEngine::generate_bibliography_paragraphs(&sources, CitationStyle::Ieee);
    assert_eq!(ieee_paragraphs.len(), 2);
    assert_eq!(ieee_paragraphs[0].get_text(), "References");
    let ieee_entry_text = ieee_paragraphs[1].get_text();
    assert!(ieee_entry_text.starts_with("[1]"));

    // Document insertion test
    let mut doc = Document::default();
    let p = Paragraph::default();
    doc.body.elements.push(BodyElement::Paragraph(p));

    let inserted = CitationEngine::insert_citation(
        &mut doc,
        0,
        "Knuth1984",
        CitationStyle::Apa,
        &sources,
        None,
    );
    assert!(inserted, "Citation insertion should succeed");

    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        assert_eq!(p.get_text(), " (Knuth, 1984)");
    } else {
        panic!("Element 0 must be paragraph");
    }
}

#[test]
fn test_high_volume_toc_and_citations_performance() {
    let mut doc = Document::default();
    let stylesheet = StyleSheet::default();
    let cascade = stylesheet.cascade_resolver();

    for i in 0..500 {
        let mut p = Paragraph::default();
        let level = (i % 3) + 1;
        p.properties = Some(ParagraphProperties {
            style: Some(format!("Heading {}", level)),
            ..Default::default()
        });
        let mut r = Run::default();
        r.content.push(RunContent::Text(
            format!("Section {} Major Title", i),
            false,
        ));
        p.children.push(ParagraphChild::Run(r));
        doc.body.elements.push(BodyElement::Paragraph(p));
    }

    let options = TocOptions::default();

    let start = Instant::now();
    let (entries_count, toc_p_count) =
        TocEngine::update_toc_fields(&mut doc, &cascade, None, &options);
    let toc_duration = start.elapsed();

    assert_eq!(entries_count, 500);
    assert_eq!(toc_p_count, 501); // Header + 500 entries
    assert!(
        toc_duration.as_millis() < 50,
        "TOC update for 500 headings must be < 50ms, took {:?}",
        toc_duration
    );

    // Citations parsing performance
    let mut bibtex_bulk = String::new();
    for i in 0..500 {
        bibtex_bulk.push_str(&format!(
            "@article{{Ref_{},\n  author = {{Author Name {}}},\n  title = {{Paper Title {}}},\n  year = {{202{}}},\n  journal = {{Journal of Systems}}\n}}\n",
            i, i, i, i % 10
        ));
    }

    let bib_start = Instant::now();
    let parsed_sources = CitationEngine::parse_bibtex(&bibtex_bulk);
    let bib_duration = bib_start.elapsed();

    assert_eq!(parsed_sources.len(), 500);
    assert!(
        bib_duration.as_millis() < 50,
        "BibTeX parsing for 500 sources must be < 50ms, took {:?}",
        bib_duration
    );

    let gen_start = Instant::now();
    let bib_paragraphs =
        CitationEngine::generate_bibliography_paragraphs(&parsed_sources, CitationStyle::Apa);
    let gen_duration = gen_start.elapsed();

    assert_eq!(bib_paragraphs.len(), 501);
    assert!(
        gen_duration.as_millis() < 50,
        "Generating bibliography paragraphs for 500 sources must be < 50ms, took {:?}",
        gen_duration
    );
}
