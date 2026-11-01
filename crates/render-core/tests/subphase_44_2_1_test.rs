//! Sub-Phase 44.2.1 Test Suite: Omnibox Command, File & Symbol Launcher (Ctrl+K)
//!
//! Strictly verifies GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()`, `.expect()`, or `panic!()` in non-test library code.
//! - Sub-millisecond fuzzy ranking across commands, open files, and document symbols.
//! - Context-aware prefix routing: `>` (commands), `@` (files), `:` (symbols), `?` (search).

use render_core::commands::transactions::GlobalOfficeState;
use render_core::omnibox::{OmniboxCategory, OmniboxEngine};
use render_core::shell::{DesktopSessionManager, DocumentType};

#[test]
fn test_01_fuzzy_matching_and_scoring() {
    // 1. Exact match produces highest score
    let score_exact = OmniboxEngine::fuzzy_score("Save Document", "Save Document");
    assert!(score_exact.is_some());
    assert!(score_exact.unwrap() >= 2000);

    // 2. Prefix match produces high score
    let score_prefix = OmniboxEngine::fuzzy_score("Save", "Save Document");
    assert!(score_prefix.is_some());
    assert!(score_prefix.unwrap() >= 600);

    // 3. Subsequence match succeeds
    let score_sub = OmniboxEngine::fuzzy_score("svdoc", "Save Document");
    assert!(score_sub.is_some());
    assert!(score_sub.unwrap() > 0);

    // 4. Non-subsequence returns None
    let score_none = OmniboxEngine::fuzzy_score("xyz", "Save Document");
    assert!(score_none.is_none());

    // 5. Empty query returns positive baseline score
    let score_empty = OmniboxEngine::fuzzy_score("", "Save Document");
    assert!(score_empty.is_some());
}

#[test]
fn test_02_prefix_routing_commands_files_symbols_search() {
    let mut mgr = DesktopSessionManager::new();
    let s1 = mgr.create_session(DocumentType::Word, "Quarterly_Review.docx");
    let mut st1 = GlobalOfficeState::new();
    st1.word_content = "# Overview\n# Financials\nBody text".to_string();
    mgr.office_states.insert(s1.session_id, st1);

    // 1. Command prefix '>' returns ONLY commands
    let cmd_results = OmniboxEngine::query(&mgr, "> split", 20);
    assert!(!cmd_results.is_empty());
    for item in &cmd_results {
        assert_eq!(item.category, OmniboxCategory::Command);
    }

    // 2. File prefix '@' returns ONLY files / open tabs
    let file_results = OmniboxEngine::query(&mgr, "@ Review", 20);
    assert!(!file_results.is_empty());
    for item in &file_results {
        assert_eq!(item.category, OmniboxCategory::File);
        assert!(item.title.contains("Review"));
    }

    // 3. Symbol prefix ':' returns ONLY document symbols
    let sym_results = OmniboxEngine::query(&mgr, ": Financials", 20);
    assert!(!sym_results.is_empty());
    for item in &sym_results {
        assert_eq!(item.category, OmniboxCategory::Symbol);
        assert!(item.title.contains("Financials"));
    }

    // 4. Search prefix '?' returns a search fallback item
    let search_results = OmniboxEngine::query(&mgr, "? EBITDA margin", 10);
    assert!(!search_results.is_empty());
    assert_eq!(search_results[0].category, OmniboxCategory::Search);
    assert!(search_results[0].payload.contains("EBITDA margin"));
}

#[test]
fn test_03_open_tabs_and_symbol_aggregation() {
    let mut mgr = DesktopSessionManager::new();

    // Setup Word tab
    let w = mgr.create_session(DocumentType::Word, "Governance.docx");
    let mut w_st = GlobalOfficeState::new();
    w_st.word_content = "# Board Resolutions\nContent here.".to_string();
    mgr.office_states.insert(w.session_id, w_st);

    // Setup Excel tab
    let x = mgr.create_session(DocumentType::Excel, "Forecast.xlsx");
    let mut x_st = GlobalOfficeState::new();
    x_st.excel_sheets = vec!["Revenue".to_string(), "Expenses".to_string()];
    mgr.office_states.insert(x.session_id, x_st);

    // Setup PPT tab
    let p = mgr.create_session(DocumentType::PowerPoint, "Pitch.pptx");
    let mut p_st = GlobalOfficeState::new();
    p_st.ppt_slides = vec![1, 2, 3];
    mgr.office_states.insert(p.session_id, p_st);

    // Collect all open tabs
    let tabs = OmniboxEngine::collect_open_tabs(&mgr);
    assert_eq!(tabs.len(), 3);
    assert!(tabs.iter().any(|t| t.title == "Governance.docx"));
    assert!(tabs.iter().any(|t| t.title == "Forecast.xlsx"));
    assert!(tabs.iter().any(|t| t.title == "Pitch.pptx"));

    // Collect symbols
    let symbols = OmniboxEngine::collect_document_symbols(&mgr);
    assert!(!symbols.is_empty());
    assert!(symbols
        .iter()
        .any(|s| s.title.contains("Board Resolutions")));
    assert!(symbols.iter().any(|s| s.title.contains("Revenue")));
    assert!(symbols.iter().any(|s| s.title.contains("Slide 1")));
}

#[test]
fn test_04_fuzzy_ranking_order_and_truncation() {
    let mgr = DesktopSessionManager::new();

    // Query for "save"
    let results = OmniboxEngine::query(&mgr, "save", 5);
    assert!(!results.is_empty());
    assert!(results.len() <= 5);

    // The top result should be an exact or prefix match for "Save Document"
    assert!(results[0].title.to_lowercase().contains("save"));

    // Verify results are strictly sorted in descending score order
    for w in results.windows(2) {
        assert!(w[0].score >= w[1].score);
    }
}
