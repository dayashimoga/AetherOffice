//! Sub-Phase 45.2.1 Integration Tests: LRU Tab Suspension & Instant Rehydration Engine
//!
//! Complies strictly with GEC-v7.0 Article I (Zero Global Document Singletons)
//! and Article VI (Multi-document memory budget < 250 MB RSS for 20 open documents):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in non-test code.
//! - 3-tier lifecycle: Active, Warm, Cold.
//! - Instant rehydration preserving scroll, zoom, and caret/selection state.

use render_core::session_suspension::{LruTabSuspensionEngine, SessionTier, TabViewportState};
use render_core::shell::DocumentType;
use std::time::Instant;

#[test]
fn test_lru_active_and_warm_quotas() {
    // Max 2 active tabs, max 3 warm tabs
    let mut engine = LruTabSuspensionEngine::new(2, 3);
    assert_eq!(engine.max_active(), 2);
    assert_eq!(engine.max_warm(), 3);
    assert_eq!(engine.session_count(), 0);

    // Register 6 tabs across formats
    engine.register_session("tab-1", DocumentType::Word, "Doc 1.docx", None);
    engine.register_session("tab-2", DocumentType::Excel, "Sheet 1.xlsx", None);
    engine.register_session("tab-3", DocumentType::PowerPoint, "Slides 1.pptx", None);
    engine.register_session("tab-4", DocumentType::Pdf, "Manual.pdf", None);
    engine.register_session("tab-5", DocumentType::Word, "Doc 2.docx", None);
    engine.register_session("tab-6", DocumentType::Excel, "Sheet 2.xlsx", None);

    assert_eq!(engine.session_count(), 6);

    // Activate tab-1 and tab-2 -> both active, 3 warm, 1 demoted to cold because max_warm=3
    engine.activate_session("tab-1");
    engine.activate_session("tab-2");
    let (active, warm, cold) = engine.memory_breakdown();
    assert_eq!(active, 2);
    assert_eq!(warm, 3);
    assert_eq!(cold, 1);

    // Activate tab-3 -> tab-3 & tab-2 active, tab-1 demoted to warm
    engine.activate_session("tab-3");
    assert_eq!(engine.get_tier("tab-3"), Some(SessionTier::Active));
    assert_eq!(engine.get_tier("tab-2"), Some(SessionTier::Active));
    assert_eq!(engine.get_tier("tab-1"), Some(SessionTier::Warm));

    // Activate tab-4, tab-5, tab-6 sequentially: warm tabs should spill to Cold (max_warm = 3)
    engine.activate_session("tab-4");
    engine.activate_session("tab-5");
    engine.activate_session("tab-6");

    let (active_end, warm_end, cold_end) = engine.memory_breakdown();
    assert_eq!(active_end, 2);
    assert_eq!(warm_end, 3);
    assert_eq!(cold_end, 1);

    // Oldest tab (tab-1) should have been demoted to Cold
    assert_eq!(engine.get_tier("tab-1"), Some(SessionTier::Cold));
}

#[test]
fn test_viewport_preservation_and_instant_rehydration_sla() {
    let mut engine = LruTabSuspensionEngine::new(1, 1);

    engine.register_session(
        "tab-writer",
        DocumentType::Word,
        "Contract.docx",
        Some("/docs/Contract.docx"),
    );
    engine.activate_session("tab-writer");

    // User scrolls and places caret
    let viewport = TabViewportState {
        scroll_x: 0.0,
        scroll_y: 1450.5,
        zoom: 1.25,
        caret_offset: Some(342),
        selection_summary: Some("Section 4.1 Termination".to_string()),
    };
    assert!(engine.save_viewport_state("tab-writer", viewport.clone()));

    // Suspend tab-writer to Cold with state payload bytes
    let payload = b"{\"text\":\"Contract Body Text\",\"version\":2}".to_vec();
    // Demote to warm first by activating another tab
    engine.register_session("tab-other", DocumentType::Excel, "Calc.xlsx", None);
    engine.activate_session("tab-other");

    assert!(engine.suspend_session_to_cold("tab-writer", payload.clone()));
    assert_eq!(engine.get_tier("tab-writer"), Some(SessionTier::Cold));

    // Instant rehydration SLA measurement
    let start = Instant::now();
    let snapshot_opt = engine.rehydrate_session("tab-writer");
    let elapsed = start.elapsed();

    println!(
        "Sub-Phase 45.2.1 SLA: Rehydrated cold session in {:?}",
        elapsed
    );

    // Rehydration SLA: must execute in < 50ms (typically < 100 microseconds in Rust)
    assert!(
        elapsed.as_millis() < 50,
        "Rehydration took {:?}, exceeding 50ms SLA",
        elapsed
    );

    assert!(snapshot_opt.is_some());
    let snapshot = match snapshot_opt {
        Some(s) => s,
        None => panic!("Expected snapshot to be present"),
    };
    assert_eq!(snapshot.session_id, "tab-writer");
    assert_eq!(snapshot.doc_type, DocumentType::Word);
    assert_eq!(snapshot.viewport_state, viewport);
    assert_eq!(snapshot.payload_bytes, payload);
    assert_eq!(engine.get_tier("tab-writer"), Some(SessionTier::Active));
}

#[test]
fn test_close_and_prune_sessions() {
    let mut engine = LruTabSuspensionEngine::new(2, 2);
    engine.register_session("tab-a", DocumentType::Word, "A", None);
    engine.register_session("tab-b", DocumentType::Excel, "B", None);

    assert_eq!(engine.session_count(), 2);
    assert!(engine.close_session("tab-a"));
    assert_eq!(engine.session_count(), 1);
    assert_eq!(engine.get_tier("tab-a"), None);
    assert!(!engine.close_session("tab-a"));
}
