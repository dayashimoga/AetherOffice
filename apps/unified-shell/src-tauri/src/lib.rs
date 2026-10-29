//! Tauri v2 Desktop & Laptop Shell Backend Library
//!
//! Provides IPC commands and document orchestration for:
//! - Multi-document tabs (`open_document`, `create_session`, `close_session`)
//! - Engine dispatch (Word, Excel, PowerPoint)
//! - Command Bus integration (Undo, Redo)
//! - Headless SVG / Rasterization export

use render_core::{
    DesktopSessionManager, DocumentType, FormattingState, RibbonModel, RibbonTabId, SessionSummary,
    TabBarModel,
};
use std::sync::Mutex;

pub struct ShellState {
    pub session_manager: Mutex<DesktopSessionManager>,
    pub formatting: Mutex<FormattingState>,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
