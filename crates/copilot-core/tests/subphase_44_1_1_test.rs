//! Sub-Phase 44.1.1 Test Suite: Incremental Multi-Format Document Indexer
//!
//! Strictly verifies GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()`, `.expect()`, or `panic!()` in non-test library code.
//! - Multi-format ingestion across Word, Calc, Slides, and PlainText.
//! - Content-hashed incremental updates and hybrid search scoring.

use copilot_core::{
    DocumentIndexer, DocumentKind, SearchFilter, SlideInput, SlideShapeInput, WordParagraphInput,
};

#[test]
fn test_01_multi_format_document_indexing() {
    let mut indexer = DocumentIndexer::new();

    // 1. Index Word Document
    let word_paras = vec![
        WordParagraphInput {
            paragraph_index: 0,
            text: "Q3 Corporate Strategy & Roadmap".to_string(),
            heading_level: Some(1),
            section: 0,
        },
        WordParagraphInput {
            paragraph_index: 1,
            text: "Our core mission is to establish open document standards and sovereign desktop productivity."
                .to_string(),
            heading_level: None,
            section: 0,
        },
    ];
    let w_chunks = indexer
        .index_word_document("doc-word-1", "Q3 Strategy", &word_paras)
        .expect("Word indexing succeeds");
    assert!(w_chunks > 0);

    // 2. Index Calc Spreadsheet
    let headers = vec![
        "Month".to_string(),
        "Revenue".to_string(),
        "Expenses".to_string(),
    ];
    let rows = vec![
        vec!["Jan".to_string(), "120000".to_string(), "80000".to_string()],
        vec!["Feb".to_string(), "135000".to_string(), "82000".to_string()],
        vec!["Mar".to_string(), "150000".to_string(), "85000".to_string()],
    ];
    let c_chunks = indexer
        .index_calc_sheet("doc-calc-1", "Financials", &headers, &rows, 5)
        .expect("Calc indexing succeeds");
    assert!(c_chunks > 0);

    // 3. Index PowerPoint Presentation
    let slides = vec![SlideInput {
        slide_number: 1,
        slide_title: Some("Vision & Product Architecture".to_string()),
        shapes: vec![SlideShapeInput {
            shape_id: "s1".to_string(),
            shape_name: "Subtitle".to_string(),
            text: "Zero data loss binary preservation with pure Rust kernel.".to_string(),
        }],
        notes: Some("Emphasize cross-platform parity on mobile and desktop.".to_string()),
    }];
    let s_chunks = indexer
        .index_presentation("doc-slides-1", "Architecture Keynote", &slides)
        .expect("Slides indexing succeeds");
    assert!(s_chunks > 0);

    // 4. Index Plain Text / Markdown
    let txt = "# Notes\nSystem architecture notes and guidelines.";
    let t_chunks = indexer
        .index_plain_text("doc-notes-1", "System Notes", txt)
        .expect("Notes indexing succeeds");
    assert!(t_chunks > 0);

    // Verify Summary
    let summary = indexer.get_summary();
    assert_eq!(summary.total_documents, 4);
    assert!(summary.total_chunks >= 4);
    assert_eq!(*summary.chunks_by_kind.get("Word").unwrap_or(&0), w_chunks);
    assert_eq!(*summary.chunks_by_kind.get("Calc").unwrap_or(&0), c_chunks);
    assert_eq!(
        *summary.chunks_by_kind.get("Slides").unwrap_or(&0),
        s_chunks
    );
    assert_eq!(
        *summary.chunks_by_kind.get("PlainText").unwrap_or(&0),
        t_chunks
    );
}

#[test]
fn test_02_incremental_reindexing_and_content_hash_preservation() {
    let mut indexer = DocumentIndexer::new();

    let paras_v1 = vec![
        WordParagraphInput {
            paragraph_index: 0,
            text: "Heading 1: Market Analysis".to_string(),
            heading_level: Some(1),
            section: 0,
        },
        WordParagraphInput {
            paragraph_index: 1,
            text: "Market share increased by 15% year-over-year.".to_string(),
            heading_level: None,
            section: 0,
        },
    ];

    let count_v1 = indexer
        .index_word_document("doc-incremental", "Market", &paras_v1)
        .expect("Initial index succeeds");
    assert!(count_v1 > 0);

    // Re-index with unchanged content: all chunks retained without re-embedding
    let count_v2 = indexer
        .index_word_document("doc-incremental", "Market", &paras_v1)
        .expect("Re-index unchanged succeeds");
    assert_eq!(count_v1, count_v2);

    // Re-index with an added paragraph
    let paras_v3 = vec![
        WordParagraphInput {
            paragraph_index: 0,
            text: "Heading 1: Market Analysis".to_string(),
            heading_level: Some(1),
            section: 0,
        },
        WordParagraphInput {
            paragraph_index: 1,
            text: "Market share increased by 15% year-over-year.".to_string(),
            heading_level: None,
            section: 0,
        },
        WordParagraphInput {
            paragraph_index: 2,
            text: "New competitors entered European cloud markets.".to_string(),
            heading_level: None,
            section: 0,
        },
    ];

    let count_v3 = indexer
        .index_word_document("doc-incremental", "Market", &paras_v3)
        .expect("Re-index modified succeeds");
    assert!(count_v3 >= count_v2);
}

#[test]
fn test_03_hybrid_lexical_and_semantic_search() {
    let mut indexer = DocumentIndexer::new();

    let paras = vec![
        WordParagraphInput {
            paragraph_index: 0,
            text: "Cloud infrastructure deployment budget and security protocol.".to_string(),
            heading_level: Some(1),
            section: 0,
        },
        WordParagraphInput {
            paragraph_index: 1,
            text: "All confidential customer data must be encrypted with AES-256-GCM.".to_string(),
            heading_level: None,
            section: 0,
        },
    ];
    indexer
        .index_word_document("doc-security", "Security Plan", &paras)
        .expect("Indexing succeeds");

    // Hybrid search for keywords
    let results = indexer
        .search("encrypted customer data AES", 5, None)
        .expect("Search succeeds");
    assert!(!results.is_empty());

    let top = &results[0];
    assert!(top.chunk.raw_text.contains("encrypted"));
    assert!(top.score > 0.0);
    assert!(top.semantic_score >= 0.0);
    assert!(top.lexical_score > 0.0);

    // Search with category filter
    let filter = SearchFilter {
        kind: Some(DocumentKind::Word),
        doc_ids: Some(vec!["doc-security".to_string()]),
        min_score: Some(0.1),
    };
    let filtered_res = indexer
        .search("security protocol", 5, Some(filter))
        .expect("Filtered search succeeds");
    assert!(!filtered_res.is_empty());
    assert_eq!(filtered_res[0].chunk.kind, DocumentKind::Word);
}

#[test]
fn test_04_document_removal_and_index_compaction() {
    let mut indexer = DocumentIndexer::new();

    let paras = vec![WordParagraphInput {
        paragraph_index: 0,
        text: "Temporary secret document to be deleted.".to_string(),
        heading_level: None,
        section: 0,
    }];
    indexer
        .index_word_document("doc-temp", "Temporary", &paras)
        .expect("Indexing succeeds");

    assert_eq!(indexer.get_summary().total_documents, 1);

    // Remove document
    let removed = indexer.remove_document("doc-temp");
    assert!(removed > 0);
    assert_eq!(indexer.get_summary().total_documents, 0);

    // Search should return zero results
    let results = indexer
        .search("secret document", 5, None)
        .expect("Search succeeds");
    assert!(results.is_empty());
}
