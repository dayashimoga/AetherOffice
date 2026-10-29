//! Tauri v2 Desktop & Laptop Shell Backend Library
//!
//! Provides IPC commands and document orchestration for:
//! - Multi-document tabs (`open_document`, `create_session`, `close_session`)
//! - Engine dispatch (Word, Excel, PowerPoint)
//! - Command Bus integration (Undo, Redo)
//! - Headless SVG / Rasterization export

use render_core::{
    AdaptiveShellController, DesktopSessionManager, DocumentType, FormFactor, FormattingState,
    PointerModality, ReleaseMatrix, RibbonModel, RibbonTabId, SessionSummary, TabBarModel,
    UiActionEvent, UiCommandDispatcher, UiDispatchResult,
};
use std::sync::Mutex;

pub struct ShellState {
    pub session_manager: Mutex<DesktopSessionManager>,
    pub formatting: Mutex<FormattingState>,
    pub adaptive_controller: Mutex<AdaptiveShellController>,
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            session_manager: Mutex::new(DesktopSessionManager::new()),
            formatting: Mutex::new(FormattingState::default()),
            adaptive_controller: Mutex::new(AdaptiveShellController::new(
                1280.0,
                800.0,
                PointerModality::Fine,
            )),
        }
    }
}

// -------------------------------------------------------------------------
// IPC Command Handlers
// -------------------------------------------------------------------------

pub fn create_session(
    state: &ShellState,
    doc_type: DocumentType,
    title: &str,
) -> Result<SessionSummary, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(mgr.create_session(doc_type, title))
}

pub fn open_document(state: &ShellState, file_path: &str) -> Result<SessionSummary, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(mgr.open_session(file_path))
}

pub fn close_session(
    state: &ShellState,
    session_id: &str,
    force: bool,
) -> Result<Option<String>, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.close_session(session_id, force)
        .map_err(|e| e.to_string())
}

pub fn execute_undo(state: &ShellState, session_id: &str) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    if let Some(bus) = mgr.command_buses.get_mut(session_id) {
        let mut doc_state = render_core::DocumentState::default();
        bus.undo(&mut doc_state)?;
        Ok(())
    } else {
        Err("Session not found".into())
    }
}

pub fn execute_redo(state: &ShellState, session_id: &str) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    if let Some(bus) = mgr.command_buses.get_mut(session_id) {
        let mut doc_state = render_core::DocumentState::default();
        bus.redo(&mut doc_state)?;
        Ok(())
    } else {
        Err("Session not found".into())
    }
}

pub fn execute_global_undo(state: &ShellState, session_id: &str) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.undo_session(session_id)
}

pub fn execute_global_redo(state: &ShellState, session_id: &str) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.redo_session(session_id)
}

pub fn mark_document_saved(state: &ShellState, session_id: &str) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.mark_session_clean(session_id)
}

pub fn is_document_dirty(state: &ShellState, session_id: &str) -> Result<bool, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.is_session_dirty(session_id)
}

pub fn get_ribbon_for_doc_type(doc_type: DocumentType) -> Vec<RibbonTabId> {
    RibbonModel::for_document_type(doc_type).visible_tabs()
}

pub fn get_tab_bar_state(state: &ShellState) -> Result<TabBarModel, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(mgr.tab_bar.clone())
}

pub fn create_window(
    state: &ShellState,
    title: &str,
    bounds: render_core::primitive::Rect2D,
) -> Result<String, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(mgr.window_manager.create_window(title, bounds, false))
}

pub fn close_window(
    state: &ShellState,
    window_id: &str,
    force: bool,
) -> Result<Option<String>, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.window_manager
        .close_window(window_id, force)
        .map_err(|e| e.to_string())
}

pub fn detach_tab_to_window(
    state: &ShellState,
    source_window_id: &str,
    tab_id: &str,
    new_bounds: render_core::primitive::Rect2D,
) -> Result<String, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.detach_tab(source_window_id, tab_id, new_bounds)
        .map_err(|e| e.to_string())
}

pub fn dock_tab_to_window(
    state: &ShellState,
    source_window_id: &str,
    tab_id: &str,
    target_window_id: &str,
    insert_index: usize,
) -> Result<(), String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    mgr.dock_tab(source_window_id, tab_id, target_window_id, insert_index)
        .map_err(|e| e.to_string())
}

pub fn get_window_manager_state(
    state: &ShellState,
) -> Result<render_core::window::MultiWindowManager, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(mgr.window_manager.clone())
}

pub fn build_child_process_config(
    window_id: &str,
    session_id: &str,
    doc_type: DocumentType,
    bounds: render_core::primitive::Rect2D,
) -> render_core::window::ProcessLaunchConfig {
    render_core::window::ProcessLaunchConfig::new(
        "unified-shell",
        window_id,
        session_id,
        doc_type,
        bounds,
    )
}

pub fn resize_viewport(state: &ShellState, width: f64, height: f64) -> Result<FormFactor, String> {
    let mut ctrl = state
        .adaptive_controller
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    ctrl.resize(width, height);
    Ok(ctrl.form_factor())
}

pub fn get_form_factor(state: &ShellState) -> Result<FormFactor, String> {
    let ctrl = state
        .adaptive_controller
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(ctrl.form_factor())
}

pub fn dispatch_ui_action(
    state: &ShellState,
    session_id: &str,
    action: UiActionEvent,
) -> Result<UiDispatchResult, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    UiCommandDispatcher::dispatch(&mut mgr, session_id, action)
}

pub fn get_release_matrix() -> ReleaseMatrix {
    ReleaseMatrix::default()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BootConfig {
    pub app_mode: String,
    pub initial_file: Option<String>,
}

pub fn get_boot_config() -> BootConfig {
    let env_mode = match std::env::var("AETHER_APP_MODE") {
        Ok(m) if !m.trim().is_empty() => m.trim().to_ascii_lowercase(),
        _ => "hub".to_string(),
    };

    let mut final_mode = env_mode;
    let mut initial_file: Option<String> = None;

    let args: Vec<String> = std::env::args().skip(1).collect();
    for arg in args {
        if arg == "--writer" || arg == "-w" {
            final_mode = "writer".to_string();
        } else if arg == "--calc" || arg == "-c" {
            final_mode = "calc".to_string();
        } else if arg == "--slides" || arg == "-s" {
            final_mode = "slides".to_string();
        } else if arg == "--pdf" || arg == "-p" {
            final_mode = "pdf".to_string();
        } else if arg == "--hub" {
            final_mode = "hub".to_string();
        } else if !arg.starts_with('-') && initial_file.is_none() {
            initial_file = Some(arg);
        }
    }

    if let Some(ref path) = initial_file {
        if final_mode == "hub" {
            let lower = path.to_ascii_lowercase();
            if lower.ends_with(".docx") || lower.ends_with(".doc") {
                final_mode = "writer".to_string();
            } else if lower.ends_with(".xlsx") || lower.ends_with(".xls") {
                final_mode = "calc".to_string();
            } else if lower.ends_with(".pptx") || lower.ends_with(".ppt") {
                final_mode = "slides".to_string();
            } else if lower.ends_with(".pdf") {
                final_mode = "pdf".to_string();
            }
        }
    }

    BootConfig {
        app_mode: final_mode,
        initial_file,
    }
}

#[cfg(not(test))]
pub fn run() {
    if let Err(e) = tauri::Builder::default().run(tauri::generate_context!()) {
        eprintln!("Error while running AetherOffice: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
pub fn run() {}

#[cfg(test)]
mod tests {
    use super::*;
    use render_core::primitive::Rect2D;

    #[test]
    fn test_shell_state_initialization() {
        let state = ShellState::new();
        let summary = create_session(&state, DocumentType::Word, "Test.docx")
            .expect("Create session successfully");
        assert_eq!(summary.title, "Test.docx");
        assert_eq!(summary.doc_type, DocumentType::Word);
        assert!(!summary.is_dirty);
    }

    #[test]
    fn test_adaptive_viewport_breakpoint_transitions() {
        let state = ShellState::new();
        assert_eq!(get_form_factor(&state).unwrap(), FormFactor::Desktop);

        // Resize to Tablet (768..1023px)
        let ff_tablet = resize_viewport(&state, 820.0, 1180.0).unwrap();
        assert_eq!(ff_tablet, FormFactor::Tablet);

        // Resize to Mobile (< 768px)
        let ff_mobile = resize_viewport(&state, 390.0, 844.0).unwrap();
        assert_eq!(ff_mobile, FormFactor::Mobile);

        // Verify mobile mode toggles
        let ctrl = state.adaptive_controller.lock().unwrap();
        assert!(ctrl.should_show_bottom_sheet());
        assert!(!ctrl.should_show_full_ribbon());
        assert!(ctrl.word_reflow_mode);
        assert!(ctrl.excel_card_mode);
    }

    #[test]
    fn test_shell_state_pdf_session_and_ribbon() {
        let state = ShellState::new();
        let summary = create_session(&state, DocumentType::Pdf, "Manual.pdf")
            .expect("Create PDF session successfully");
        assert_eq!(summary.title, "Manual.pdf");
        assert_eq!(summary.doc_type, DocumentType::Pdf);
        assert!(!summary.is_dirty);

        let pdf_tabs = get_ribbon_for_doc_type(DocumentType::Pdf);
        assert!(pdf_tabs.contains(&RibbonTabId::File));
        assert!(pdf_tabs.contains(&RibbonTabId::Home));
        assert!(pdf_tabs.contains(&RibbonTabId::Insert));
        assert!(pdf_tabs.contains(&RibbonTabId::Review));
        assert!(pdf_tabs.contains(&RibbonTabId::View));
        assert!(!pdf_tabs.contains(&RibbonTabId::Formulas));
    }

    #[test]
    fn test_multi_window_tab_detach_and_dock() {
        let state = ShellState::new();

        // Create two document sessions in primary window
        let _doc1 = create_session(&state, DocumentType::Word, "Doc1.docx").unwrap();
        let doc2 = create_session(&state, DocumentType::Excel, "Sheet1.xlsx").unwrap();

        let win_mgr = get_window_manager_state(&state).unwrap();
        assert_eq!(win_mgr.window_count(), 1);
        let primary_win_id = win_mgr.active_window_id.clone().unwrap();
        assert_eq!(win_mgr.get_window(&primary_win_id).unwrap().tab_count(), 2);

        // Detach Doc2 into a new standalone window
        let new_win_id = detach_tab_to_window(
            &state,
            &primary_win_id,
            &doc2.session_id,
            Rect2D::new(500.0, 200.0, 1024.0, 768.0),
        )
        .expect("Detach Doc2 to new window");

        let win_mgr_after_detach = get_window_manager_state(&state).unwrap();
        assert_eq!(win_mgr_after_detach.window_count(), 2);
        assert_eq!(
            win_mgr_after_detach
                .get_window(&primary_win_id)
                .unwrap()
                .tab_count(),
            1
        );
        assert_eq!(
            win_mgr_after_detach
                .get_window(&new_win_id)
                .unwrap()
                .tab_count(),
            1
        );

        // Dock Doc2 back into primary window at index 1
        dock_tab_to_window(&state, &new_win_id, &doc2.session_id, &primary_win_id, 1)
            .expect("Dock Doc2 back");

        let win_mgr_after_dock = get_window_manager_state(&state).unwrap();
        assert_eq!(win_mgr_after_dock.window_count(), 1); // Empty secondary window automatically pruned
        assert_eq!(
            win_mgr_after_dock
                .get_window(&primary_win_id)
                .unwrap()
                .tab_count(),
            2
        );
    }

    #[test]
    fn test_ui_action_dispatch_routes_to_command_bus() {
        let state = ShellState::new();
        let session = create_session(&state, DocumentType::Word, "Document.docx").unwrap();

        // 1. Dispatch text insertion action
        let res1 = dispatch_ui_action(
            &state,
            &session.session_id,
            UiActionEvent::InsertText {
                text: "Hello OpenOfficeRS!".to_string(),
            },
        )
        .expect("Dispatch insert text");
        assert_eq!(res1.undo_stack_depth, 1);
        assert!(res1.is_dirty);
        assert!(is_document_dirty(&state, &session.session_id).unwrap());

        // 2. Dispatch formatting action
        let res2 = dispatch_ui_action(&state, &session.session_id, UiActionEvent::ToggleBold)
            .expect("Dispatch toggle bold");
        assert_eq!(res2.undo_stack_depth, 2);

        // 3. Dispatch Undo action
        let res3 = dispatch_ui_action(&state, &session.session_id, UiActionEvent::Undo)
            .expect("Dispatch undo");
        assert_eq!(res3.undo_stack_depth, 1);
        assert_eq!(res3.redo_stack_depth, 1);

        // 4. Dispatch Save / Mark Clean action
        let res4 = dispatch_ui_action(&state, &session.session_id, UiActionEvent::MarkSaved)
            .expect("Dispatch save");
        assert!(!res4.is_dirty);
        assert!(!is_document_dirty(&state, &session.session_id).unwrap());
    }

    #[test]
    fn test_release_matrix_ci_cd_targets() {
        let matrix = get_release_matrix();
        assert_eq!(matrix.app_name, "OpenOfficeRS");
        assert!(matrix.targets.len() >= 8);
        assert!(matrix.validate_full_coverage().is_ok());
        assert!(matrix.validate_triplets().is_ok());
    }

    #[test]
    fn test_boot_config_resolution() {
        std::env::set_var("AETHER_APP_MODE", "writer");
        let cfg = get_boot_config();
        assert_eq!(cfg.app_mode, "writer");

        std::env::set_var("AETHER_APP_MODE", "calc");
        let cfg2 = get_boot_config();
        assert_eq!(cfg2.app_mode, "calc");

        std::env::set_var("AETHER_APP_MODE", "slides");
        let cfg3 = get_boot_config();
        assert_eq!(cfg3.app_mode, "slides");

        std::env::set_var("AETHER_APP_MODE", "pdf");
        let cfg4 = get_boot_config();
        assert_eq!(cfg4.app_mode, "pdf");

        std::env::remove_var("AETHER_APP_MODE");
    }
}
