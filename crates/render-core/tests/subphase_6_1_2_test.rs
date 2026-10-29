//! Subphase 6.1.2 Integration Test Suite:
//! Multi-Window Independent Process Launcher & Tab Detachment
//!
//! Validates:
//! - WindowInstance lifecycle and WindowState modes (Normal, Minimized, Maximized, Fullscreen)
//! - MultiWindowManager window creation, tab counting, active window switching, and dirty close guards
//! - TabDetachmentEngine tear-off threshold evaluation and new window spawning
//! - TabDockingEngine snap target detection, insertion index calculation, and empty window pruning
//! - ProcessLaunchConfig CLI arguments generation and lossless parsing
//! - ProcessIpcEnvelope cross-process JSON-RPC serialization & deserialization
//! - Transactional Command Bus integration for detachment and docking
//! - DesktopSessionManager multi-window and tab detachment orchestration
//! - Sub-millisecond hit-testing and snapping latency SLA (< 1.0 ms)

use render_core::commands::{GlobalCommandBus, GlobalOfficeState, OfficeTransaction};
use render_core::primitive::Rect2D;
use render_core::shell::{DesktopSessionManager, DocumentTab, DocumentType};
use render_core::window::{
    DetachTabTransaction, DockTabTransaction, MultiWindowManager, ProcessIpcEnvelope,
    ProcessIpcPayload, ProcessLaunchConfig, TabDetachmentEngine, TabDockingEngine, TabDragSession,
    WindowInstance, WindowState,
};
use std::time::Instant;

// =========================================================================
// Tests
// =========================================================================

#[test]
fn test_01_window_instance_and_state_initialization() {
    let bounds = Rect2D::new(100.0, 100.0, 1280.0, 800.0);
    let mut win = WindowInstance::new("win-1", "Main Window", bounds, true);

    assert_eq!(win.id, "win-1");
    assert_eq!(win.title, "Main Window");
    assert_eq!(win.state, WindowState::Normal);
    assert!(win.is_primary);
    assert_eq!(win.tab_count(), 0);
    assert!(!win.has_tabs());
    assert_eq!(win.active_tab(), None);

    // Tab bar occupies top 36 DIPs
    let tab_bar = win.tab_bar_rect();
    assert_eq!(tab_bar.x, 100.0);
    assert_eq!(tab_bar.y, 100.0);
    assert_eq!(tab_bar.width, 1280.0);
    assert_eq!(tab_bar.height, 36.0);

    // State changes
    win.state = WindowState::Maximized;
    assert_eq!(win.state, WindowState::Maximized);
    win.state = WindowState::Fullscreen;
    assert_eq!(win.state, WindowState::Fullscreen);
}

#[test]
fn test_02_multi_window_manager_create_and_query() {
    let mut mgr = MultiWindowManager::new();
    assert_eq!(mgr.window_count(), 0);

    // Create primary window
    let win1_id = mgr.create_window(
        "Primary Window",
        Rect2D::new(0.0, 0.0, 1920.0, 1080.0),
        true,
    );
    assert_eq!(mgr.window_count(), 1);
    assert_eq!(mgr.active_window_id.as_deref(), Some(win1_id.as_str()));

    // Create secondary window
    let win2_id = mgr.create_window(
        "Secondary Window",
        Rect2D::new(200.0, 200.0, 1024.0, 768.0),
        false,
    );
    assert_eq!(mgr.window_count(), 2);

    // Add tabs to windows
    let win1 = mgr.get_window_mut(&win1_id).unwrap();
    win1.tabs
        .add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Report.docx"));
    win1.tabs
        .add_tab(DocumentTab::new("tab-2", DocumentType::Excel, "Data.xlsx"));

    let win2 = mgr.get_window_mut(&win2_id).unwrap();
    win2.tabs
        .add_tab(DocumentTab::new("tab-3", DocumentType::Pdf, "Manual.pdf"));

    assert_eq!(mgr.total_tab_count(), 3);
    assert_eq!(mgr.get_window(&win1_id).unwrap().tab_count(), 2);
    assert_eq!(mgr.get_window(&win2_id).unwrap().tab_count(), 1);

    // Query window by tab ID
    assert_eq!(mgr.find_window_by_tab_id("tab-1").unwrap().id, win1_id);
    assert_eq!(mgr.find_window_by_tab_id("tab-3").unwrap().id, win2_id);
    assert!(mgr.find_window_by_tab_id("tab-999").is_none());
}

#[test]
fn test_03_switch_active_window() {
    let mut mgr = MultiWindowManager::new();
    let win1 = mgr.create_window("Win 1", Rect2D::new(0.0, 0.0, 800.0, 600.0), true);
    let win2 = mgr.create_window("Win 2", Rect2D::new(100.0, 100.0, 800.0, 600.0), false);

    assert_eq!(mgr.active_window_id.as_deref(), Some(win1.as_str()));

    // Switch to win2
    assert!(mgr.switch_window(&win2));
    assert_eq!(mgr.active_window_id.as_deref(), Some(win2.as_str()));

    // Switch to non-existent window
    assert!(!mgr.switch_window("win-nonexistent"));
    assert_eq!(mgr.active_window_id.as_deref(), Some(win2.as_str()));
}

#[test]
fn test_04_close_window_with_dirty_guard() {
    let mut mgr = MultiWindowManager::new();
    let win1 = mgr.create_window("Win 1", Rect2D::new(0.0, 0.0, 800.0, 600.0), true);
    let win2 = mgr.create_window("Win 2", Rect2D::new(100.0, 100.0, 800.0, 600.0), false);

    // Add dirty tab to win2
    let mut dirty_tab = DocumentTab::new("tab-dirty", DocumentType::Word, "Unsaved.docx");
    dirty_tab.is_dirty = true;
    mgr.get_window_mut(&win2).unwrap().tabs.add_tab(dirty_tab);

    // Non-forced close must fail due to dirty tab
    let close_res = mgr.close_window(&win2, false);
    assert!(close_res.is_err());
    assert_eq!(mgr.window_count(), 2);

    // Forced close succeeds
    let next_active = mgr.close_window(&win2, true).expect("Forced close");
    assert_eq!(mgr.window_count(), 1);
    assert_eq!(next_active.as_deref(), Some(win1.as_str()));
}

#[test]
fn test_05_tab_detachment_can_detach_predicate() {
    let bounds = Rect2D::new(0.0, 0.0, 1024.0, 768.0);
    let mut primary_win = WindowInstance::new("win-p", "Primary", bounds, true);

    // 1 tab in primary window cannot detach
    primary_win
        .tabs
        .add_tab(DocumentTab::new("t1", DocumentType::Word, "Doc 1"));
    assert!(!TabDetachmentEngine::can_detach(&primary_win, "t1"));

    // 2 tabs in primary window allows detachment
    primary_win
        .tabs
        .add_tab(DocumentTab::new("t2", DocumentType::Excel, "Sheet 1"));
    assert!(TabDetachmentEngine::can_detach(&primary_win, "t1"));
    assert!(TabDetachmentEngine::can_detach(&primary_win, "t2"));

    // Single tab in secondary standalone window allows detachment
    let mut secondary_win = WindowInstance::new("win-s", "Secondary", bounds, false);
    secondary_win
        .tabs
        .add_tab(DocumentTab::new("t3", DocumentType::Pdf, "Manual.pdf"));
    assert!(TabDetachmentEngine::can_detach(&secondary_win, "t3"));
}

#[test]
fn test_06_tab_drag_session_detachment_threshold() {
    let start_pos = (200.0, 20.0);
    let mut drag = TabDragSession::new("win-1", "tab-1", start_pos);

    assert_eq!(drag.source_window_id, "win-1");
    assert_eq!(drag.tab_id, "tab-1");
    assert!(!drag.is_detached);

    // Move pointer 10px right and 10px down -> distance = sqrt(200) ~= 14.14px < 30px
    let is_detached = drag.update_pointer((210.0, 30.0));
    assert!(!is_detached);
    assert!(!drag.is_detached);

    // Move pointer 40px down -> distance from start = sqrt(0^2 + 40^2) = 40px >= 30px
    let is_detached = drag.update_pointer((200.0, 60.0));
    assert!(is_detached);
    assert!(drag.is_detached);
}

#[test]
fn test_07_detach_tab_to_new_window() {
    let mut mgr = MultiWindowManager::new();
    let win1_id = mgr.create_window("Window 1", Rect2D::new(0.0, 0.0, 1280.0, 800.0), true);

    // Add 3 tabs to win1
    let win1 = mgr.get_window_mut(&win1_id).unwrap();
    win1.tabs
        .add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Doc1.docx"));
    win1.tabs.add_tab(DocumentTab::new(
        "tab-2",
        DocumentType::Excel,
        "Sheet1.xlsx",
    ));
    win1.tabs.add_tab(DocumentTab::new(
        "tab-3",
        DocumentType::PowerPoint,
        "Slides1.pptx",
    ));

    assert_eq!(mgr.get_window(&win1_id).unwrap().tab_count(), 3);

    // Detach tab-2 into a new window positioned at (400, 300, 1024, 768)
    let new_bounds = Rect2D::new(400.0, 300.0, 1024.0, 768.0);
    let win2_id =
        TabDetachmentEngine::detach_tab_to_new_window(&mut mgr, &win1_id, "tab-2", new_bounds)
            .expect("Detach tab-2");

    assert_eq!(mgr.window_count(), 2);
    let win1 = mgr.get_window(&win1_id).unwrap();
    assert_eq!(win1.tab_count(), 2);
    assert_eq!(win1.tabs.tabs[0].id, "tab-1");
    assert_eq!(win1.tabs.tabs[1].id, "tab-3");

    let win2 = mgr.get_window(&win2_id).unwrap();
    assert_eq!(win2.tab_count(), 1);
    assert_eq!(win2.tabs.tabs[0].id, "tab-2");
    assert_eq!(win2.bounds, new_bounds);
    assert_eq!(win2.title, "Sheet1.xlsx");
}

#[test]
fn test_08_dock_target_detection_tab_bar_hit_testing() {
    let bounds = Rect2D::new(100.0, 100.0, 1280.0, 800.0);
    let mut win = WindowInstance::new("win-target", "Target Window", bounds, true);
    win.tabs
        .add_tab(DocumentTab::new("t1", DocumentType::Word, "Doc 1"));
    win.tabs
        .add_tab(DocumentTab::new("t2", DocumentType::Word, "Doc 2"));

    let windows = [&win];

    // Pointer far away from tab bar -> None
    let target = TabDockingEngine::detect_dock_target(
        (500.0, 500.0),
        &windows,
        "win-dragged",
        TabDockingEngine::DEFAULT_SNAP_THRESHOLD,
    );
    assert!(target.is_none());

    // Pointer hovering inside target window's tab bar (100..1380 x, 100..136 y)
    // Pointer at x=150, y=110 -> first tab position -> insert_index = 0
    let target = TabDockingEngine::detect_dock_target(
        (150.0, 110.0),
        &windows,
        "win-dragged",
        TabDockingEngine::DEFAULT_SNAP_THRESHOLD,
    );
    assert!(target.is_some());
    let dock = target.unwrap();
    assert_eq!(dock.target_window_id, "win-target");
    assert_eq!(dock.insert_index, 0);

    // Pointer at x=450, y=110 -> relative x = 350 -> floor(350/160) = 2 -> insert_index = 2
    let target2 = TabDockingEngine::detect_dock_target(
        (450.0, 110.0),
        &windows,
        "win-dragged",
        TabDockingEngine::DEFAULT_SNAP_THRESHOLD,
    );
    assert!(target2.is_some());
    assert_eq!(target2.unwrap().insert_index, 2);
}

#[test]
fn test_09_dock_tab_into_target_window_with_auto_prune() {
    let mut mgr = MultiWindowManager::new();
    let win1_id = mgr.create_window("Primary", Rect2D::new(0.0, 0.0, 1280.0, 800.0), true);
    let win2_id = mgr.create_window("Secondary", Rect2D::new(200.0, 200.0, 1024.0, 768.0), false);

    // Primary has tab-1, secondary has tab-2
    mgr.get_window_mut(&win1_id)
        .unwrap()
        .tabs
        .add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Doc1"));
    mgr.get_window_mut(&win2_id)
        .unwrap()
        .tabs
        .add_tab(DocumentTab::new("tab-2", DocumentType::Excel, "Doc2"));

    assert_eq!(mgr.window_count(), 2);

    // Dock tab-2 from win2 into win1 at index 1
    TabDockingEngine::dock_tab(&mut mgr, &win2_id, "tab-2", &win1_id, 1).expect("Dock tab-2");

    // win1 now has 2 tabs
    let win1 = mgr.get_window(&win1_id).unwrap();
    assert_eq!(win1.tab_count(), 2);
    assert_eq!(win1.tabs.tabs[0].id, "tab-1");
    assert_eq!(win1.tabs.tabs[1].id, "tab-2");
    assert_eq!(win1.tabs.active_tab_id.as_deref(), Some("tab-2"));

    // Secondary window win2 was left with 0 tabs and is not primary -> auto-pruned!
    assert_eq!(mgr.window_count(), 1);
    assert!(mgr.get_window(&win2_id).is_none());
}

#[test]
fn test_10_dock_tab_rejects_same_window() {
    let mut mgr = MultiWindowManager::new();
    let win_id = mgr.create_window("Window", Rect2D::new(0.0, 0.0, 1280.0, 800.0), true);
    mgr.get_window_mut(&win_id)
        .unwrap()
        .tabs
        .add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Doc1"));

    let res = TabDockingEngine::dock_tab(&mut mgr, &win_id, "tab-1", &win_id, 0);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "Cannot dock tab into same window");
}

#[test]
fn test_11_process_launch_config_cli_args_roundtrip() {
    let bounds = Rect2D::new(150.0, 250.0, 1024.0, 768.0);
    let mut config = ProcessLaunchConfig::new(
        "openoffice-bin",
        "win-child-7",
        "sess-excel-9",
        DocumentType::Excel,
        bounds,
    );
    config.file_path = Some("C:/Documents/Budget.xlsx".to_string());

    let cli_args = config.build_cli_args();
    assert!(cli_args.contains(&"--child-window".to_string()));
    assert!(cli_args.contains(&"--window-id".to_string()));
    assert!(cli_args.contains(&"win-child-7".to_string()));
    assert!(cli_args.contains(&"--session-id".to_string()));
    assert!(cli_args.contains(&"sess-excel-9".to_string()));
    assert!(cli_args.contains(&"Excel".to_string()));
    assert!(cli_args.contains(&"C:/Documents/Budget.xlsx".to_string()));

    // Parse back
    let parsed = ProcessLaunchConfig::parse_cli_args(&cli_args).expect("Parse CLI args");
    assert_eq!(parsed.window_id, "win-child-7");
    assert_eq!(parsed.session_id, "sess-excel-9");
    assert_eq!(parsed.doc_type, DocumentType::Excel);
    assert_eq!(
        parsed.file_path.as_deref(),
        Some("C:/Documents/Budget.xlsx")
    );
    assert_eq!(parsed.screen_bounds.x, 150.0);
    assert_eq!(parsed.screen_bounds.y, 250.0);
    assert_eq!(parsed.screen_bounds.width, 1024.0);
    assert_eq!(parsed.screen_bounds.height, 768.0);
}

#[test]
fn test_12_process_ipc_envelope_json_serialization() {
    // 1. DetachTabRequest
    let payload1 = ProcessIpcPayload::DetachTabRequest {
        source_window_id: "win-1".into(),
        tab_id: "tab-42".into(),
        drop_pos: (600.0, 400.0),
    };
    let env1 = ProcessIpcEnvelope::new(1234, payload1.clone());
    let json1 = env1.to_json().expect("Serialize to JSON");
    let deserialized1 = ProcessIpcEnvelope::from_json(&json1).expect("Deserialize JSON");
    assert_eq!(deserialized1.payload, payload1);

    // 2. DockTabRequest
    let payload2 = ProcessIpcPayload::DockTabRequest {
        source_window_id: "win-2".into(),
        target_window_id: "win-1".into(),
        tab_id: "tab-42".into(),
        insert_index: 3,
    };
    let env2 = ProcessIpcEnvelope::new(5678, payload2.clone());
    let json2 = env2.to_json().expect("Serialize JSON");
    let deserialized2 = ProcessIpcEnvelope::from_json(&json2).expect("Deserialize JSON");
    assert_eq!(deserialized2.payload, payload2);

    // 3. SessionSync
    let payload3 = ProcessIpcPayload::SessionSync {
        session_id: "sess-1".into(),
        doc_type: DocumentType::Pdf,
        title: "Contract.pdf".into(),
        is_dirty: true,
        file_path: Some("/docs/Contract.pdf".into()),
    };
    let env3 = ProcessIpcEnvelope::new(9999, payload3.clone());
    let json3 = env3.to_json().expect("Serialize SessionSync");
    let deserialized3 = ProcessIpcEnvelope::from_json(&json3).expect("Deserialize SessionSync");
    assert_eq!(deserialized3.payload, payload3);

    // 4. Heartbeat
    let payload4 = ProcessIpcPayload::Heartbeat {
        timestamp_ms: 1729000000,
    };
    let env4 = ProcessIpcEnvelope::new(1000, payload4.clone());
    let json4 = env4.to_json().expect("Serialize Heartbeat");
    let deserialized4 = ProcessIpcEnvelope::from_json(&json4).expect("Deserialize Heartbeat");
    assert_eq!(deserialized4.payload, payload4);
}

#[test]
fn test_13_reversible_detach_and_dock_transactions() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Detach transaction
    let detach_tx = Box::new(DetachTabTransaction::new(
        "win-1",
        "tab-10",
        Rect2D::new(100.0, 100.0, 800.0, 600.0),
    ));
    assert_eq!(detach_tx.description(), "Shell: Detach Tab to Window");
    assert_eq!(detach_tx.target_app(), DocumentType::Word);

    bus.execute(&mut state, detach_tx)
        .expect("Execute detach tx");
    assert_eq!(bus.undo_stack_len(), 1);

    // Dock transaction
    let dock_tx = Box::new(DockTabTransaction::new("win-2", "win-1", "tab-10", 0));
    assert_eq!(dock_tx.description(), "Shell: Dock Tab into Window");
    bus.execute(&mut state, dock_tx).expect("Execute dock tx");
    assert_eq!(bus.undo_stack_len(), 2);

    // Undo dock
    bus.undo(&mut state).expect("Undo dock");
    assert_eq!(bus.undo_stack_len(), 1);

    // Undo detach
    bus.undo(&mut state).expect("Undo detach");
    assert_eq!(bus.undo_stack_len(), 0);
}

#[test]
fn test_14_desktop_session_manager_multi_window_integration() {
    let mut mgr = DesktopSessionManager::new();

    // Initially 1 primary window
    assert_eq!(mgr.window_manager.window_count(), 1);
    let primary_id = mgr.window_manager.active_window_id.clone().unwrap();

    // Create 3 document sessions
    let doc1 = mgr.create_session(DocumentType::Word, "Report.docx");
    let doc2 = mgr.create_session(DocumentType::Excel, "Budget.xlsx");
    let doc3 = mgr.create_session(DocumentType::Pdf, "UserManual.pdf");

    assert_eq!(
        mgr.window_manager
            .get_window(&primary_id)
            .unwrap()
            .tab_count(),
        3
    );

    // Detach PDF session into a secondary window
    let detached_bounds = Rect2D::new(500.0, 300.0, 1024.0, 768.0);
    let new_win_id = mgr
        .detach_tab(&primary_id, &doc3.session_id, detached_bounds)
        .expect("Detach PDF tab");

    assert_eq!(mgr.window_manager.window_count(), 2);
    assert_eq!(
        mgr.window_manager
            .get_window(&primary_id)
            .unwrap()
            .tab_count(),
        2
    );
    assert_eq!(
        mgr.window_manager
            .get_window(&new_win_id)
            .unwrap()
            .tab_count(),
        1
    );

    // Dock PDF session back into primary window at index 0
    mgr.dock_tab(&new_win_id, &doc3.session_id, &primary_id, 0)
        .expect("Dock PDF tab back");

    assert_eq!(mgr.window_manager.window_count(), 1); // Secondary window pruned
    let primary = mgr.window_manager.get_window(&primary_id).unwrap();
    assert_eq!(primary.tab_count(), 3);
    assert_eq!(primary.tabs.tabs[0].id, doc3.session_id);
    assert_eq!(primary.tabs.tabs[1].id, doc1.session_id);
    assert_eq!(primary.tabs.tabs[2].id, doc2.session_id);
}

#[test]
fn test_15_rapid_detachment_and_docking_sla_benchmark() {
    let bounds = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);
    let mut win = WindowInstance::new("win-bench", "Benchmark Window", bounds, true);
    for i in 0..10 {
        win.tabs.add_tab(DocumentTab::new(
            format!("t{i}"),
            DocumentType::Word,
            format!("Doc {i}"),
        ));
    }
    let windows = [&win];

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let x = 100.0 + ((i % 500) as f64);
        let y = 10.0 + ((i % 50) as f64);

        // Benchmark dock target hit-testing
        let _ = TabDockingEngine::detect_dock_target(
            (x, y),
            &windows,
            "win-dragging",
            TabDockingEngine::DEFAULT_SNAP_THRESHOLD,
        );
    }

    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Benchmark: 10,000 dock hit-test operations took {:?} (average: {:.2} µs / op)",
        elapsed, avg_micros
    );

    // SLA: Hit-testing and snapping must take strictly < 1.0 ms (1,000 µs)
    assert!(
        avg_micros < 1000.0,
        "Dock hit-testing latency {:.2} µs exceeded 1.0 ms SLA",
        avg_micros
    );
}
