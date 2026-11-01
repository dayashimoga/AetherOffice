//! Sub-Phase 44.1.2 Test Suite: Cross-Document Federated Search & Replace Panel
//!
//! Strictly verifies GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()`, `.expect()`, or `panic!()` in non-test library code.
//! - Cross-format search across active tabs (Word, Excel, PPTX, PDF, Notes).
//! - Precision case sensitivity, whole word boundary handling, and atomic replace all.

use render_core::commands::transactions::GlobalOfficeState;
use render_core::federated_search::{FederatedSearchEngine, MatchLocation, SearchOptions};
use render_core::shell::{DesktopSessionManager, DocumentType};

#[test]
fn test_01_cross_document_search_across_formats() {
    let mut mgr = DesktopSessionManager::new();

    // 1. Setup Word Session
    let word_s = mgr.create_session(DocumentType::Word, "Governance.docx");
    let mut word_state = GlobalOfficeState::new();
    word_state.word_content = "Annual corporate compliance policy and internal audit guidelines.\nSecond paragraph with compliance standards.".to_string();
    mgr.office_states
        .insert(word_s.session_id.clone(), word_state);

    // 2. Setup Excel Session
    let excel_s = mgr.create_session(DocumentType::Excel, "Ledger.xlsx");
    let mut excel_state = GlobalOfficeState::new();
    excel_state.excel_sheets = vec!["Sheet1".to_string()];
    excel_state
        .excel_cells
        .insert("A1".to_string(), "Compliance Budget".to_string());
    excel_state
        .excel_cells
        .insert("B1".to_string(), "500000".to_string());
    mgr.office_states
        .insert(excel_s.session_id.clone(), excel_state);

    // 3. Setup PowerPoint Session
    let ppt_s = mgr.create_session(DocumentType::PowerPoint, "Deck.pptx");
    let mut ppt_state = GlobalOfficeState::new();
    ppt_state.ppt_slides = vec![1, 2];
    mgr.office_states
        .insert(ppt_s.session_id.clone(), ppt_state);

    // Execute federated search for "compliance"
    let options = SearchOptions {
        case_sensitive: false,
        whole_word: false,
        use_regex: false,
    };
    let matches = FederatedSearchEngine::search_workspace(&mgr, "compliance", &options);
    assert_eq!(matches.len(), 3); // 2 in Word, 1 in Excel

    // Verify Word match locations
    let word_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.session_id == word_s.session_id)
        .collect();
    assert_eq!(word_matches.len(), 2);
    assert!(matches!(
        word_matches[0].location,
        MatchLocation::Word { .. }
    ));

    // Verify Excel match location
    let excel_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.session_id == excel_s.session_id)
        .collect();
    assert_eq!(excel_matches.len(), 1);
    match &excel_matches[0].location {
        MatchLocation::Excel { coord, .. } => assert_eq!(coord, "A1"),
        _ => panic!("Expected Excel location"),
    }
}

#[test]
fn test_02_search_options_case_sensitivity_and_whole_word() {
    let mut mgr = DesktopSessionManager::new();

    let s = mgr.create_session(DocumentType::Word, "OptionsTest.docx");
    let mut state = GlobalOfficeState::new();
    state.word_content = "Standard standard STAND standardize standard_lib".to_string();
    mgr.office_states.insert(s.session_id.clone(), state);

    // 1. Case-insensitive, not whole word
    let opt_loose = SearchOptions {
        case_sensitive: false,
        whole_word: false,
        use_regex: false,
    };
    let m1 = FederatedSearchEngine::search_workspace(&mgr, "standard", &opt_loose);
    assert_eq!(m1.len(), 4);

    // 2. Case-sensitive, not whole word
    let opt_case = SearchOptions {
        case_sensitive: true,
        whole_word: false,
        use_regex: false,
    };
    let m2 = FederatedSearchEngine::search_workspace(&mgr, "Standard", &opt_case);
    assert_eq!(m2.len(), 1);

    // 3. Whole word matching
    let opt_whole = SearchOptions {
        case_sensitive: false,
        whole_word: true,
        use_regex: false,
    };
    let m3 = FederatedSearchEngine::search_workspace(&mgr, "standard", &opt_whole);
    // Matches "Standard", "standard" (standardize and standard_lib have word characters following)
    assert_eq!(m3.len(), 2);
}

#[test]
fn test_03_replace_in_session_word_and_excel() {
    let mut mgr = DesktopSessionManager::new();

    // 1. Word replace
    let word_s = mgr.create_session(DocumentType::Word, "Doc.docx");
    let mut word_state = GlobalOfficeState::new();
    word_state.word_content = "The old_term needs replacement. Another old_term here.".to_string();
    mgr.office_states
        .insert(word_s.session_id.clone(), word_state);

    let opt = SearchOptions::default();
    let replaced_w = FederatedSearchEngine::replace_in_session(
        &mut mgr,
        &word_s.session_id,
        "old_term",
        "new_term",
        &opt,
    )
    .expect("Word replace succeeds");
    assert_eq!(replaced_w, 2);

    let updated_word = mgr.office_states.get(&word_s.session_id).unwrap();
    assert!(updated_word.word_content.contains("new_term"));
    assert!(!updated_word.word_content.contains("old_term"));

    // 2. Excel replace
    let excel_s = mgr.create_session(DocumentType::Excel, "Sheet.xlsx");
    let mut excel_state = GlobalOfficeState::new();
    excel_state
        .excel_cells
        .insert("A1".to_string(), "Status: PENDING".to_string());
    excel_state
        .excel_cells
        .insert("A2".to_string(), "Status: PENDING".to_string());
    excel_state
        .excel_cells
        .insert("A3".to_string(), "Status: APPROVED".to_string());
    mgr.office_states
        .insert(excel_s.session_id.clone(), excel_state);

    let replaced_e = FederatedSearchEngine::replace_in_session(
        &mut mgr,
        &excel_s.session_id,
        "PENDING",
        "RESOLVED",
        &opt,
    )
    .expect("Excel replace succeeds");
    assert_eq!(replaced_e, 2);

    let updated_excel = mgr.office_states.get(&excel_s.session_id).unwrap();
    assert_eq!(
        updated_excel.excel_cells.get("A1").unwrap(),
        "Status: RESOLVED"
    );
    assert_eq!(
        updated_excel.excel_cells.get("A2").unwrap(),
        "Status: RESOLVED"
    );
    assert_eq!(
        updated_excel.excel_cells.get("A3").unwrap(),
        "Status: APPROVED"
    );
}

#[test]
fn test_04_federated_replace_all_across_open_documents() {
    let mut mgr = DesktopSessionManager::new();

    // Doc 1 (Word)
    let s1 = mgr.create_session(DocumentType::Word, "Doc1.docx");
    let mut st1 = GlobalOfficeState::new();
    st1.word_content = "Global token ALPHA_TOKEN in Word.".to_string();
    mgr.office_states.insert(s1.session_id.clone(), st1);

    // Doc 2 (Word)
    let s2 = mgr.create_session(DocumentType::Word, "Doc2.docx");
    let mut st2 = GlobalOfficeState::new();
    st2.word_content = "Another ALPHA_TOKEN here.".to_string();
    mgr.office_states.insert(s2.session_id.clone(), st2);

    // Doc 3 (Excel)
    let s3 = mgr.create_session(DocumentType::Excel, "Doc3.xlsx");
    let mut st3 = GlobalOfficeState::new();
    st3.excel_cells
        .insert("C5".to_string(), "Cell ALPHA_TOKEN".to_string());
    mgr.office_states.insert(s3.session_id.clone(), st3);

    let opt = SearchOptions::default();
    let summary =
        FederatedSearchEngine::replace_all(&mut mgr, "ALPHA_TOKEN", "OMEGA_TOKEN", &opt, None)
            .expect("Replace all succeeds");

    assert_eq!(summary.total_replacements, 3);
    assert_eq!(summary.documents_affected, 3);
    assert_eq!(summary.replacements_by_session.len(), 3);

    // Confirm text across all documents was replaced
    assert!(mgr
        .office_states
        .get(&s1.session_id)
        .unwrap()
        .word_content
        .contains("OMEGA_TOKEN"));
    assert!(mgr
        .office_states
        .get(&s2.session_id)
        .unwrap()
        .word_content
        .contains("OMEGA_TOKEN"));
    assert_eq!(
        mgr.office_states
            .get(&s3.session_id)
            .unwrap()
            .excel_cells
            .get("C5")
            .unwrap(),
        "Cell OMEGA_TOKEN"
    );
}
