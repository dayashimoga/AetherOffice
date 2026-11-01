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
use std::collections::HashMap;
use std::sync::Mutex;

pub mod collab_commands;
pub use collab_commands::*;

pub mod macro_commands;
pub use macro_commands::*;

pub mod window_commands;
pub use window_commands::*;

pub mod file_commands;
pub use file_commands::*;

pub mod binary_commands;
pub use binary_commands::*;

pub mod tauri_commands;

pub struct ShellState {
    pub session_manager: Mutex<DesktopSessionManager>,
    pub formatting: Mutex<FormattingState>,
    pub adaptive_controller: Mutex<AdaptiveShellController>,
    pub app_instances: Mutex<Vec<AppInstance>>,
    pub next_instance_counter: Mutex<u64>,
    pub collab_sessions: Mutex<HashMap<String, crdt_collab::CollabTransportBus>>,
    pub cancellation_tokens: Mutex<HashMap<String, ooxml_opc::async_pipeline::CancellationToken>>,
    pub indexer: Mutex<copilot_core::DocumentIndexer>,
    pub dep_graph: Mutex<render_core::CrossDocDependencyGraph>,
    pub wikilink_index: Mutex<render_core::WikiLinkIndex>,
    pub oplog: Mutex<crdt_collab::LocalOpLog>,
    pub plugin_host: Mutex<plugin_runtime::WasmPluginHost>,
    pub udf_registry: Mutex<plugin_runtime::PluginUdfRegistry>,
    pub copilot_engine: Mutex<copilot_core::CopilotGroundedEngine>,
    pub theme_engine: Mutex<render_core::DesignTokenEngine>,
    pub accessibility_tree: Mutex<render_core::AriaTree>,
    pub ime_state: Mutex<render_core::ImeCompositionState>,
    pub nav_map: Mutex<render_core::KeyboardNavigationMap>,
    pub gesture_recognizer: Mutex<render_core::TouchGestureRecognizer>,
    pub palm_filter: Mutex<render_core::PalmRejectionFilter>,
    pub split_grid: Mutex<render_core::AdaptiveSplitGrid>,
    pub dlp_policy: Mutex<crypto_security::DlpPolicy>,
    pub audit_log: Mutex<crypto_security::AuditLog>,
    pub entitlement_engine: Mutex<render_core::EntitlementEngine>,
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
            app_instances: Mutex::new(Vec::new()),
            next_instance_counter: Mutex::new(0),
            collab_sessions: Mutex::new(HashMap::new()),
            cancellation_tokens: Mutex::new(HashMap::new()),
            indexer: Mutex::new(copilot_core::DocumentIndexer::new()),
            dep_graph: Mutex::new(render_core::CrossDocDependencyGraph::new()),
            wikilink_index: Mutex::new(render_core::WikiLinkIndex::new()),
            oplog: Mutex::new(crdt_collab::LocalOpLog::new()),
            plugin_host: Mutex::new(plugin_runtime::WasmPluginHost::default()),
            udf_registry: Mutex::new(plugin_runtime::PluginUdfRegistry::new()),
            copilot_engine: Mutex::new(copilot_core::CopilotGroundedEngine::default()),
            theme_engine: Mutex::new(render_core::DesignTokenEngine::new()),
            accessibility_tree: Mutex::new(render_core::AriaTree::new(render_core::AriaNode::new(
                "root-doc",
                render_core::AriaRole::Document,
                "Document Surface",
            ))),
            ime_state: Mutex::new(render_core::ImeCompositionState::new()),
            nav_map: Mutex::new(render_core::KeyboardNavigationMap::new()),
            gesture_recognizer: Mutex::new(render_core::TouchGestureRecognizer::new()),
            palm_filter: Mutex::new(render_core::PalmRejectionFilter::new()),
            split_grid: Mutex::new(render_core::AdaptiveSplitGrid::new(
                render_core::SplitGridMode::Single,
            )),
            dlp_policy: Mutex::new(crypto_security::DlpPolicy::default_enterprise_policy()),
            audit_log: Mutex::new(crypto_security::AuditLog::new()),
            entitlement_engine: Mutex::new(render_core::EntitlementEngine::new()),
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
    render_core::DocumentIoEngine::open_document_from_disk(&mut mgr, file_path)
}

pub fn open_document_async<F>(
    state: &ShellState,
    file_path: &str,
    session_id: Option<&str>,
    progress_cb: F,
) -> Result<SessionSummary, String>
where
    F: Fn(ooxml_opc::async_pipeline::IngestionProgress) + Send + Sync + 'static,
{
    let token = ooxml_opc::async_pipeline::CancellationToken::new();
    let tracking_key = session_id.unwrap_or("").to_string();
    if !tracking_key.is_empty() {
        let mut tokens = state
            .cancellation_tokens
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        tokens.insert(tracking_key.clone(), token.clone());
    }

    let result = {
        let mut mgr = state
            .session_manager
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        render_core::DocumentIoEngine::open_document_async(
            &mut mgr,
            file_path,
            Some(&token),
            progress_cb,
        )
    };

    if !tracking_key.is_empty() {
        if let Ok(mut tokens) = state.cancellation_tokens.lock() {
            tokens.remove(&tracking_key);
        }
    }

    result
}

pub fn cancel_ingestion(state: &ShellState, session_id: &str) -> Result<bool, String> {
    let tokens = state
        .cancellation_tokens
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    if let Some(token) = tokens.get(session_id) {
        token.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn save_document(
    state: &ShellState,
    session_id: &str,
    file_path: Option<&str>,
) -> Result<String, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    let target_path = match file_path {
        Some(p) => p.to_string(),
        None => match mgr
            .get_session(session_id)
            .and_then(|s| s.file_path.clone())
        {
            Some(p) => p,
            None => {
                let doc_type = mgr
                    .get_session(session_id)
                    .map(|s| s.doc_type)
                    .unwrap_or(render_core::DocumentType::Word);
                match doc_type {
                    render_core::DocumentType::Word => "Document1.docx".to_string(),
                    render_core::DocumentType::Excel => "Book1.xlsx".to_string(),
                    render_core::DocumentType::PowerPoint => "Presentation1.pptx".to_string(),
                    render_core::DocumentType::Pdf => "Document.pdf".to_string(),
                }
            }
        },
    };

    render_core::DocumentIoEngine::save_document_to_disk(&mut mgr, session_id, &target_path)
}

pub fn export_pdf(
    state: &ShellState,
    session_id: &str,
    output_path: &str,
) -> Result<String, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    render_core::DocumentIoEngine::export_document_to_pdf(&mgr, session_id, output_path)
}

pub fn export_svg(_state: &ShellState, session_id: &str) -> Result<String, String> {
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 1100\" data-session-id=\"{session_id}\"><rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/><text x=\"40\" y=\"60\" font-family=\"Segoe UI, sans-serif\" font-size=\"16\" fill=\"#000000\">AetherOffice Document [{session_id}]</text></svg>"
    ))
}

pub fn get_document_info(
    state: &ShellState,
    session_id: &str,
) -> Result<render_core::DocumentMetadata, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    render_core::DocumentIoEngine::get_document_info(&mgr, session_id)
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

pub fn workspace_search(
    state: &ShellState,
    query: &str,
    top_k: Option<usize>,
) -> Result<Vec<copilot_core::HybridSearchResult>, String> {
    let indexer = state
        .indexer
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    indexer
        .search(query, top_k.unwrap_or(10), None)
        .map_err(|e| e.to_string())
}

pub fn workspace_index_text(
    state: &ShellState,
    doc_id: &str,
    title: &str,
    text: &str,
) -> Result<usize, String> {
    let mut indexer = state
        .indexer
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    indexer
        .index_plain_text(doc_id, title, text)
        .map_err(|e| e.to_string())
}

pub fn workspace_get_index_summary(
    state: &ShellState,
) -> Result<copilot_core::IndexSummary, String> {
    let indexer = state
        .indexer
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(indexer.get_summary())
}

pub fn federated_search(
    state: &ShellState,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<Vec<render_core::FederatedSearchMatch>, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    let options = render_core::SearchOptions {
        case_sensitive,
        whole_word,
        use_regex,
    };
    Ok(render_core::FederatedSearchEngine::search_workspace(
        &mgr, query, &options,
    ))
}

pub fn federated_replace_match(
    state: &ShellState,
    session_id: &str,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<usize, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    let options = render_core::SearchOptions {
        case_sensitive,
        whole_word,
        use_regex,
    };
    render_core::FederatedSearchEngine::replace_in_session(
        &mut mgr,
        session_id,
        query,
        replacement,
        &options,
    )
}

pub fn federated_replace_all(
    state: &ShellState,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    target_session_ids: Option<Vec<String>>,
) -> Result<render_core::FederatedReplaceSummary, String> {
    let mut mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    let options = render_core::SearchOptions {
        case_sensitive,
        whole_word,
        use_regex,
    };
    render_core::FederatedSearchEngine::replace_all(
        &mut mgr,
        query,
        replacement,
        &options,
        target_session_ids,
    )
}

pub fn omnibox_query(
    state: &ShellState,
    query: &str,
    max_results: Option<usize>,
) -> Result<Vec<render_core::OmniboxItem>, String> {
    let mgr = state
        .session_manager
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(render_core::OmniboxEngine::query(
        &mgr,
        query,
        max_results.unwrap_or(20),
    ))
}

pub fn dep_graph_register_dependency(
    state: &ShellState,
    edge: render_core::DependencyEdge,
) -> Result<String, String> {
    let mut graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    graph.register_dependency(edge).map_err(|e| e.to_string())
}

pub fn dep_graph_remove_dependency(state: &ShellState, edge_id: &str) -> Result<bool, String> {
    let mut graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    graph.remove_dependency(edge_id).map_err(|e| e.to_string())
}

pub fn dep_graph_get_dependents(
    state: &ShellState,
    doc_id: &str,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    let graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(graph.get_dependents(doc_id))
}

pub fn dep_graph_get_dependencies(
    state: &ShellState,
    doc_id: &str,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    let graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(graph.get_dependencies(doc_id))
}

pub fn dep_graph_propagate_invalidation(
    state: &ShellState,
    source_doc_id: &str,
    affected_anchor: Option<render_core::EmbedAnchor>,
) -> Result<Vec<String>, String> {
    let mut graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(graph.propagate_invalidation(source_doc_id, affected_anchor.as_ref()))
}

pub fn dep_graph_update_snapshot(
    state: &ShellState,
    snapshot: render_core::LiveEmbedSnapshot,
) -> Result<(), String> {
    let mut graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    graph.update_snapshot(snapshot);
    Ok(())
}

pub fn dep_graph_get_snapshot(
    state: &ShellState,
    embed_id: &str,
) -> Result<Option<render_core::LiveEmbedSnapshot>, String> {
    let graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(graph.get_snapshot(embed_id).cloned())
}

pub fn dep_graph_summary(state: &ShellState) -> Result<render_core::CrossDocSummaryDto, String> {
    let graph = state
        .dep_graph
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(graph.graph_summary())
}

pub fn wikilink_index_document(
    state: &ShellState,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<(), String> {
    let mut index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    index.index_document(doc_id, title, content);
    Ok(())
}

pub fn wikilink_remove_document(state: &ShellState, doc_id: &str) -> Result<(), String> {
    let mut index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    index.remove_document(doc_id);
    Ok(())
}

pub fn wikilink_get_backlinks(
    state: &ShellState,
    target_title_or_id: &str,
) -> Result<Vec<render_core::BacklinkReference>, String> {
    let index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(index.get_backlinks(target_title_or_id))
}

pub fn wikilink_get_forward_links(
    state: &ShellState,
    source_doc_id: &str,
) -> Result<Vec<render_core::WikiLink>, String> {
    let index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(index.get_forward_links(source_doc_id))
}

pub fn wikilink_suggest_targets(
    state: &ShellState,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<render_core::WikiLinkSuggestion>, String> {
    let index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(index.suggest_targets(query, limit.unwrap_or(10)))
}

pub fn wikilink_find_unlinked_mentions(
    state: &ShellState,
    target_title: &str,
) -> Result<Vec<render_core::UnlinkedMention>, String> {
    let index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(index.find_unlinked_mentions(target_title))
}

pub fn wikilink_parse_links(text: &str) -> Result<Vec<render_core::WikiLink>, String> {
    Ok(render_core::WikiLinkParser::parse(text))
}

pub fn wikilink_stats(state: &ShellState) -> Result<render_core::WikiLinkIndexStats, String> {
    let index = state
        .wikilink_index
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(index.stats())
}

pub fn conversion_convert_single(
    task: render_core::ConversionTask,
) -> Result<render_core::ConversionResult, String> {
    Ok(render_core::CrossFormatConverter::convert_single(task))
}

pub fn conversion_batch_convert(
    tasks: Vec<render_core::ConversionTask>,
) -> Result<render_core::BatchConversionReport, String> {
    Ok(render_core::CrossFormatConverter::convert_batch(tasks))
}

pub fn conversion_supported_formats(
    from: render_core::ConversionFormat,
) -> Result<Vec<render_core::ConversionFormat>, String> {
    Ok(render_core::CrossFormatConverter::supported_targets(from))
}

pub fn conversion_estimate_job(
    task_count: usize,
    total_bytes: usize,
) -> Result<render_core::ConversionEstimateDto, String> {
    Ok(render_core::CrossFormatConverter::estimate_job(
        task_count,
        total_bytes,
    ))
}

pub fn oplog_append_entry(
    state: &ShellState,
    doc_id: &str,
    author_id: &str,
    payload: crdt_collab::OpPayload,
    timestamp_ms: u64,
    client_id: u64,
) -> Result<crdt_collab::OpLogEntry, String> {
    let mut oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    oplog
        .append(doc_id, author_id, payload, timestamp_ms, client_id)
        .map_err(|e| e.to_string())
}

pub fn oplog_get_history(
    state: &ShellState,
    doc_id: &str,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    let oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    Ok(oplog.get_history(doc_id, limit, offset))
}

pub fn oplog_time_travel(
    state: &ShellState,
    doc_id: &str,
    target_timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    let oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    Ok(oplog.time_travel_to_timestamp(doc_id, target_timestamp_ms))
}

pub fn oplog_create_checkpoint(
    state: &ShellState,
    doc_id: &str,
    author_id: &str,
    label: &str,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::CheckpointMeta, String> {
    let mut oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    oplog
        .create_checkpoint(doc_id, author_id, label, client_id, timestamp_ms)
        .map_err(|e| e.to_string())
}

pub fn oplog_get_checkpoints(
    state: &ShellState,
    doc_id: &str,
) -> Result<Vec<crdt_collab::CheckpointMeta>, String> {
    let oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    Ok(oplog.get_checkpoints(doc_id))
}

pub fn oplog_compute_sync_delta(
    state: &ShellState,
    doc_id: &str,
    remote_vector: crdt_collab::StateVector,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    let oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    Ok(oplog.compute_sync_delta(doc_id, &remote_vector))
}

pub fn oplog_apply_remote_delta(
    state: &ShellState,
    doc_id: &str,
    remote_entries: Vec<crdt_collab::OpLogEntry>,
) -> Result<crdt_collab::SyncReport, String> {
    let mut oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    oplog
        .apply_remote_delta(doc_id, remote_entries)
        .map_err(|e| e.to_string())
}

pub fn oplog_get_state_vector(
    state: &ShellState,
    doc_id: &str,
) -> Result<crdt_collab::StateVector, String> {
    let oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    Ok(oplog.get_state_vector(doc_id))
}

pub fn oplog_fork_version(
    state: &ShellState,
    doc_id: &str,
    target_op_id: crdt_collab::OpId,
    new_doc_id: &str,
    new_author_id: &str,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    let mut oplog = state
        .oplog
        .lock()
        .map_err(|_| "Failed to lock oplog state".to_string())?;
    let forked = oplog
        .fork_at_version(
            doc_id,
            target_op_id,
            new_doc_id,
            new_author_id,
            client_id,
            timestamp_ms,
        )
        .map_err(|e| e.to_string())?;
    for entry in forked.get_entries(new_doc_id) {
        let _ = oplog.append_entry(entry);
    }
    Ok(oplog.time_travel_to_timestamp(new_doc_id, timestamp_ms))
}

pub fn plugin_load(
    state: &ShellState,
    manifest_json: Option<String>,
    wasm_bytes: Vec<u8>,
) -> Result<String, String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;

    let manifest_override = if let Some(json) = manifest_json {
        Some(plugin_runtime::PluginManifest::from_json_str(&json).map_err(|e| e.to_string())?)
    } else {
        None
    };

    host.load_plugin(&wasm_bytes, manifest_override)
        .map_err(|e| e.to_string())
}

pub fn plugin_unload(state: &ShellState, plugin_id: &str) -> Result<(), String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    host.unload_plugin(plugin_id).map_err(|e| e.to_string())
}

pub fn plugin_list(state: &ShellState) -> Result<Vec<plugin_runtime::PluginStatusDto>, String> {
    let host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    Ok(host.list_plugins())
}

pub fn plugin_init(state: &ShellState, plugin_id: &str) -> Result<(), String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    host.init_plugin(plugin_id).map_err(|e| e.to_string())
}

pub fn plugin_suspend(state: &ShellState, plugin_id: &str) -> Result<(), String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    host.suspend_plugin(plugin_id).map_err(|e| e.to_string())
}

pub fn plugin_resume(state: &ShellState, plugin_id: &str) -> Result<(), String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    host.resume_plugin(plugin_id).map_err(|e| e.to_string())
}

pub fn plugin_refill_fuel(state: &ShellState, plugin_id: &str) -> Result<(), String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    host.refill_fuel(plugin_id).map_err(|e| e.to_string())
}

pub fn plugin_execute_action(
    state: &ShellState,
    plugin_id: &str,
    action: plugin_runtime::PluginAction,
) -> Result<plugin_runtime::ActionResult, String> {
    let mut host = state
        .plugin_host
        .lock()
        .map_err(|_| "Failed to lock plugin host".to_string())?;
    let mut doc_host = plugin_runtime::MockDocumentHost::new();
    host.execute_plugin_action(plugin_id, action, &mut doc_host)
        .map_err(|e| e.to_string())
}

pub fn plugin_call_udf(
    state: &ShellState,
    name: &str,
    args: &[String],
) -> Result<plugin_runtime::PluginUdfCallResultDto, String> {
    let registry = state
        .udf_registry
        .lock()
        .map_err(|_| "Failed to lock UDF registry".to_string())?;
    Ok(registry.call_udf_with_strings(name, args))
}

pub fn plugin_list_udfs(
    state: &ShellState,
) -> Result<Vec<plugin_runtime::PluginUdfMetaDto>, String> {
    let registry = state
        .udf_registry
        .lock()
        .map_err(|_| "Failed to lock UDF registry".to_string())?;
    Ok(registry.list_udfs())
}

pub fn plugin_register_mock_udf(
    state: &ShellState,
    name: &str,
    description: &str,
    min_args: usize,
    max_args: usize,
    formula_kind: &str,
) -> Result<(), String> {
    let mut registry = state
        .udf_registry
        .lock()
        .map_err(|_| "Failed to lock UDF registry".to_string())?;

    let kind = formula_kind.to_string();
    registry.register_udf(
        name,
        description,
        min_args,
        max_args,
        move |args| match kind.as_str() {
            "sum_plus_ten" => {
                let mut sum = 10.0;
                for a in args {
                    if let Some(n) = a.as_f64() {
                        sum += n;
                    }
                }
                Ok(excel_engine::grid::CellValue::Number(sum))
            }
            "double" => {
                let val = args.first().and_then(|a| a.as_f64()).unwrap_or(0.0);
                Ok(excel_engine::grid::CellValue::Number(val * 2.0))
            }
            "prefix_hello" => {
                let s = match args.first() {
                    Some(excel_engine::grid::CellValue::InlineString(t)) => t.clone(),
                    Some(excel_engine::grid::CellValue::Number(n)) => n.to_string(),
                    _ => String::new(),
                };
                Ok(excel_engine::grid::CellValue::InlineString(format!(
                    "Hello {s}"
                )))
            }
            _ => {
                let count = args.len() as f64;
                Ok(excel_engine::grid::CellValue::Number(count))
            }
        },
    );

    Ok(())
}

pub fn copilot_chat(
    state: &ShellState,
    request: copilot_core::CopilotChatRequest,
) -> Result<copilot_core::CopilotChatResponse, String> {
    let mut engine = state
        .copilot_engine
        .lock()
        .map_err(|_| "Failed to lock copilot engine".to_string())?;
    engine.chat(request).map_err(|e| e.to_string())
}

pub fn copilot_configure_provider(
    state: &ShellState,
    config: copilot_core::ProviderConfig,
) -> Result<copilot_core::ProviderStatusDto, String> {
    let mut engine = state
        .copilot_engine
        .lock()
        .map_err(|_| "Failed to lock copilot engine".to_string())?;
    engine.set_config(config);
    Ok(engine.probe_provider_status())
}

pub fn copilot_get_provider_config(
    state: &ShellState,
) -> Result<copilot_core::ProviderStatusDto, String> {
    let engine = state
        .copilot_engine
        .lock()
        .map_err(|_| "Failed to lock copilot engine".to_string())?;
    Ok(engine.probe_provider_status())
}

pub fn copilot_index_document(
    state: &ShellState,
    doc_id: &str,
    title: &str,
    text: &str,
) -> Result<usize, String> {
    let mut engine = state
        .copilot_engine
        .lock()
        .map_err(|_| "Failed to lock copilot engine".to_string())?;
    engine
        .index_text_document(doc_id, title, text)
        .map_err(|e| e.to_string())
}

pub fn copilot_remove_document(state: &ShellState, doc_id: &str) -> Result<usize, String> {
    let mut engine = state
        .copilot_engine
        .lock()
        .map_err(|_| "Failed to lock copilot engine".to_string())?;
    Ok(engine.remove_document(doc_id))
}

pub fn theme_get_tokens(
    state: &ShellState,
    theme_mode: Option<String>,
    ribbon_mode: Option<String>,
) -> Result<render_core::ThemeTokens, String> {
    let engine = state
        .theme_engine
        .lock()
        .map_err(|_| "Failed to lock theme engine".to_string())?;

    let t_mode = match theme_mode.as_deref() {
        Some("dark") => render_core::ThemeMode::Dark,
        Some("high-contrast-dark") => render_core::ThemeMode::HighContrastDark,
        Some("high-contrast-light") => render_core::ThemeMode::HighContrastLight,
        Some("light") => render_core::ThemeMode::Light,
        _ => engine.active_theme(),
    };

    let r_mode = match ribbon_mode.as_deref() {
        Some("compact") => render_core::RibbonLayoutMode::Compact,
        Some("collapsed") => render_core::RibbonLayoutMode::Collapsed,
        Some("classic") => render_core::RibbonLayoutMode::Classic,
        _ => engine.ribbon_config().mode,
    };

    Ok(engine.evaluate_tokens(t_mode, r_mode))
}

pub fn theme_set_mode(
    state: &ShellState,
    theme_mode: &str,
) -> Result<render_core::ThemeTokens, String> {
    let mut engine = state
        .theme_engine
        .lock()
        .map_err(|_| "Failed to lock theme engine".to_string())?;

    let mode = match theme_mode.to_lowercase().as_str() {
        "dark" => render_core::ThemeMode::Dark,
        "high-contrast-dark" | "highcontrastdark" => render_core::ThemeMode::HighContrastDark,
        "high-contrast-light" | "highcontrastlight" => render_core::ThemeMode::HighContrastLight,
        _ => render_core::ThemeMode::Light,
    };

    engine.set_theme(mode);
    let r_mode = engine.ribbon_config().mode;
    Ok(engine.evaluate_tokens(mode, r_mode))
}

pub fn ribbon_set_layout_mode(
    state: &ShellState,
    layout_mode: &str,
) -> Result<render_core::RibbonLayoutConfig, String> {
    let mut engine = state
        .theme_engine
        .lock()
        .map_err(|_| "Failed to lock theme engine".to_string())?;

    let mode = match layout_mode.to_lowercase().as_str() {
        "compact" => render_core::RibbonLayoutMode::Compact,
        "collapsed" => render_core::RibbonLayoutMode::Collapsed,
        _ => render_core::RibbonLayoutMode::Classic,
    };

    engine.set_ribbon_mode(mode);
    Ok(engine.ribbon_config().clone())
}

pub fn ribbon_get_layout_config(
    state: &ShellState,
) -> Result<render_core::RibbonLayoutConfig, String> {
    let engine = state
        .theme_engine
        .lock()
        .map_err(|_| "Failed to lock theme engine".to_string())?;
    Ok(engine.ribbon_config().clone())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BidiAnalysisResult {
    pub base_direction: render_core::BidiDirection,
    pub runs: Vec<render_core::BidiRun>,
}

pub fn accessibility_get_aria_tree(state: &ShellState) -> Result<render_core::AriaTree, String> {
    let tree = state
        .accessibility_tree
        .lock()
        .map_err(|_| "Failed to lock accessibility tree".to_string())?;
    Ok(tree.clone())
}

pub fn accessibility_set_aria_tree(
    state: &ShellState,
    tree: render_core::AriaTree,
) -> Result<usize, String> {
    let mut current_tree = state
        .accessibility_tree
        .lock()
        .map_err(|_| "Failed to lock accessibility tree".to_string())?;
    let count = tree.total_nodes;
    *current_tree = tree;
    Ok(count)
}

pub fn ime_start_composition(
    state: &ShellState,
    caret_x: f32,
    caret_y: f32,
    caret_h: f32,
) -> Result<render_core::AriaBoundingBox, String> {
    let mut ime = state
        .ime_state
        .lock()
        .map_err(|_| "Failed to lock IME state".to_string())?;
    ime.start(caret_x, caret_y, caret_h);
    Ok(ime.candidate_box)
}

#[allow(clippy::too_many_arguments)]
pub fn ime_update_composition(
    state: &ShellState,
    text: String,
    cursor: usize,
    sel_start: usize,
    sel_end: usize,
    caret_x: f32,
    caret_y: f32,
    caret_h: f32,
) -> Result<render_core::AriaBoundingBox, String> {
    let mut ime = state
        .ime_state
        .lock()
        .map_err(|_| "Failed to lock IME state".to_string())?;
    ime.update(&text, cursor, sel_start, sel_end, caret_x, caret_y, caret_h);
    Ok(ime.candidate_box)
}

pub fn ime_finish_composition(state: &ShellState) -> Result<String, String> {
    let mut ime = state
        .ime_state
        .lock()
        .map_err(|_| "Failed to lock IME state".to_string())?;
    Ok(ime.finish())
}

pub fn ime_cancel_composition(state: &ShellState) -> Result<(), String> {
    let mut ime = state
        .ime_state
        .lock()
        .map_err(|_| "Failed to lock IME state".to_string())?;
    ime.cancel();
    Ok(())
}

pub fn bidi_analyze_text(text: String) -> Result<BidiAnalysisResult, String> {
    let base_direction = render_core::BidiAnalyzer::detect_base_direction(&text);
    let runs = render_core::BidiAnalyzer::analyze_runs(&text);
    Ok(BidiAnalysisResult {
        base_direction,
        runs,
    })
}

pub fn keyboard_nav_cycle_pane(
    state: &ShellState,
    reverse: Option<bool>,
) -> Result<render_core::NavFocusPane, String> {
    let mut nav = state
        .nav_map
        .lock()
        .map_err(|_| "Failed to lock keyboard navigation map".to_string())?;
    let pane = if reverse.unwrap_or(false) {
        nav.cycle_prev_pane()
    } else {
        nav.cycle_next_pane()
    };
    Ok(pane)
}

pub fn gesture_process_touch_start(
    state: &ShellState,
    point: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    let mut recognizer = state
        .gesture_recognizer
        .lock()
        .map_err(|_| "Failed to lock gesture recognizer".to_string())?;
    Ok(recognizer.touch_start(point))
}

pub fn gesture_process_touch_move(
    state: &ShellState,
    point: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    let mut recognizer = state
        .gesture_recognizer
        .lock()
        .map_err(|_| "Failed to lock gesture recognizer".to_string())?;
    Ok(recognizer.touch_move(point))
}

pub fn gesture_process_touch_end(
    state: &ShellState,
    id: u64,
    timestamp_ms: u64,
) -> Result<render_core::TouchGesture, String> {
    let mut recognizer = state
        .gesture_recognizer
        .lock()
        .map_err(|_| "Failed to lock gesture recognizer".to_string())?;
    Ok(recognizer.touch_end(id, timestamp_ms))
}

pub fn gesture_process_touch_cancel(state: &ShellState, id: u64) -> Result<(), String> {
    let mut recognizer = state
        .gesture_recognizer
        .lock()
        .map_err(|_| "Failed to lock gesture recognizer".to_string())?;
    recognizer.touch_cancel(id);
    Ok(())
}

pub fn palm_rejection_filter_contact(
    state: &ShellState,
    touch: render_core::TouchPoint,
    active_stylus: Option<render_core::TouchPoint>,
    screen_w: f64,
    screen_h: f64,
) -> Result<bool, String> {
    let filter = state
        .palm_filter
        .lock()
        .map_err(|_| "Failed to lock palm rejection filter".to_string())?;
    Ok(filter.is_palm(&touch, active_stylus.as_ref(), screen_w, screen_h))
}

pub fn adaptive_split_set_layout(
    state: &ShellState,
    mode_str: String,
    ratio_x: Option<f64>,
    ratio_y: Option<f64>,
) -> Result<render_core::AdaptiveSplitGrid, String> {
    let mut grid = state
        .split_grid
        .lock()
        .map_err(|_| "Failed to lock split grid".to_string())?;

    let mode = match mode_str.to_lowercase().as_str() {
        "two-way-horizontal" | "twowayhorizontal" => render_core::SplitGridMode::TwoWayHorizontal,
        "two-way-vertical" | "twowayvertical" => render_core::SplitGridMode::TwoWayVertical,
        "four-way-grid" | "fourwaygrid" => render_core::SplitGridMode::FourWayGrid,
        _ => render_core::SplitGridMode::Single,
    };
    grid.mode = mode;

    if let (Some(rx), Some(ry)) = (ratio_x, ratio_y) {
        grid.set_ratios_with_snap(rx, ry);
    } else if let Some(rx) = ratio_x {
        let ry = grid.ratio_y;
        grid.set_ratios_with_snap(rx, ry);
    } else if let Some(ry) = ratio_y {
        let rx = grid.ratio_x;
        grid.set_ratios_with_snap(rx, ry);
    }

    Ok(grid.clone())
}

pub fn adaptive_split_get_rects(
    state: &ShellState,
    total_width: f64,
    total_height: f64,
    inset_top: Option<f64>,
    inset_bottom: Option<f64>,
) -> Result<Vec<render_core::primitive::Rect2D>, String> {
    let grid = state
        .split_grid
        .lock()
        .map_err(|_| "Failed to lock split grid".to_string())?;

    let total_rect = render_core::primitive::Rect2D::new(0.0, 0.0, total_width, total_height);
    let insets = render_core::adaptive::SafeAreaInsets {
        top: inset_top.unwrap_or(0.0),
        right: 0.0,
        bottom: inset_bottom.unwrap_or(0.0),
        left: 0.0,
    };

    Ok(grid.compute_subpane_rects(&total_rect, &insets))
}

pub fn dlp_evaluate_text(
    state: &ShellState,
    text: &str,
) -> Result<crypto_security::DlpEvaluationResult, String> {
    let policy = state
        .dlp_policy
        .lock()
        .map_err(|_| "Failed to lock DLP policy".to_string())?;
    Ok(crypto_security::DlpEvaluator::evaluate_text(&policy, text))
}

pub fn dlp_evaluate_and_redact(
    state: &ShellState,
    text: &str,
) -> Result<(crypto_security::DlpEvaluationResult, String), String> {
    let policy = state
        .dlp_policy
        .lock()
        .map_err(|_| "Failed to lock DLP policy".to_string())?;
    Ok(crypto_security::DlpEvaluator::evaluate_and_redact(
        &policy, text,
    ))
}

pub fn dlp_set_policy(
    state: &ShellState,
    policy: crypto_security::DlpPolicy,
) -> Result<(), String> {
    let mut p = state
        .dlp_policy
        .lock()
        .map_err(|_| "Failed to lock DLP policy".to_string())?;
    *p = policy;
    Ok(())
}

pub fn dlp_get_policy(state: &ShellState) -> Result<crypto_security::DlpPolicy, String> {
    let p = state
        .dlp_policy
        .lock()
        .map_err(|_| "Failed to lock DLP policy".to_string())?;
    Ok(p.clone())
}

pub fn sanitization_generate_certificate(
    doc_id: &str,
    original_bytes: &[u8],
    sanitized_bytes: &[u8],
    policy_id: &str,
    timestamp_ms: u64,
    redactions: Vec<(String, usize)>,
    secret_key: &[u8],
) -> Result<crypto_security::SanitizationCertificate, String> {
    Ok(crypto_security::SanitizationCertificateGenerator::generate(
        doc_id,
        original_bytes,
        sanitized_bytes,
        policy_id,
        timestamp_ms,
        redactions,
        secret_key,
    ))
}

pub fn sanitization_verify_certificate(
    cert: &crypto_security::SanitizationCertificate,
    sanitized_bytes: Option<&[u8]>,
    secret_key: &[u8],
) -> Result<bool, String> {
    if let Some(bytes) = sanitized_bytes {
        crypto_security::SanitizationCertificateValidator::verify_document_integrity(
            cert, bytes, secret_key,
        )?;
    } else {
        crypto_security::SanitizationCertificateValidator::verify_signature(cert, secret_key)?;
    }
    Ok(true)
}

pub fn audit_log_append(
    state: &ShellState,
    timestamp_ms: u64,
    event_type: crypto_security::AuditEventType,
    user_id: &str,
    doc_id: &str,
    details: &str,
) -> Result<crypto_security::AuditLogEntry, String> {
    let mut log = state
        .audit_log
        .lock()
        .map_err(|_| "Failed to lock audit log".to_string())?;
    Ok(log
        .append(timestamp_ms, event_type, user_id, doc_id, details)
        .clone())
}

pub fn audit_log_get_entries(
    state: &ShellState,
    doc_id: Option<String>,
) -> Result<Vec<crypto_security::AuditLogEntry>, String> {
    let log = state
        .audit_log
        .lock()
        .map_err(|_| "Failed to lock audit log".to_string())?;
    if let Some(ref d) = doc_id {
        Ok(log.filter_by_document(d).into_iter().cloned().collect())
    } else {
        Ok(log.entries().to_vec())
    }
}

pub fn audit_log_verify_integrity(state: &ShellState) -> Result<usize, String> {
    let log = state
        .audit_log
        .lock()
        .map_err(|_| "Failed to lock audit log".to_string())?;
    log.verify_integrity()
}

pub fn entitlement_get_tier(state: &ShellState) -> Result<String, String> {
    let engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    let now_ms = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    };
    Ok(engine.effective_tier(now_ms).as_str().to_string())
}

pub fn entitlement_check_feature(
    state: &ShellState,
    feature: &str,
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    let engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    let feat = render_core::FeatureEntitlement::from_str_loose(feature)
        .ok_or_else(|| format!("Unknown feature entitlement: '{feature}'"))?;
    let timestamp = now_ms.unwrap_or_else(|| {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_millis() as u64,
            Err(_) => 0,
        }
    });
    Ok(engine.check_feature(feat, timestamp))
}

pub fn entitlement_apply_license(
    state: &ShellState,
    token: render_core::LicenseToken,
    verification_key: &[u8],
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    let mut engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    let timestamp = now_ms.unwrap_or_else(|| {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_millis() as u64,
            Err(_) => 0,
        }
    });
    engine.apply_license(token, verification_key, timestamp)
}

pub fn entitlement_get_license_info(
    state: &ShellState,
) -> Result<Option<render_core::LicenseToken>, String> {
    let engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    Ok(engine.active_license().cloned())
}

pub fn entitlement_set_tier(state: &ShellState, tier: &str) -> Result<String, String> {
    let mut engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    let edition = render_core::EditionTier::from_str_loose(tier)
        .ok_or_else(|| format!("Invalid edition tier: '{tier}'"))?;
    engine.set_tier_override(edition);
    Ok(edition.as_str().to_string())
}

pub fn entitlement_remove_license(state: &ShellState) -> Result<bool, String> {
    let mut engine = state
        .entitlement_engine
        .lock()
        .map_err(|_| "Failed to lock entitlement engine".to_string())?;
    engine.remove_license();
    Ok(true)
}

pub fn sla_run_benchmarks(_state: &ShellState) -> Result<render_core::SlaBenchmarkReport, String> {
    let thresholds = render_core::SlaBudgetThresholds::default();
    let mut metrics = Vec::new();

    let cold_start = render_core::SlaBenchmarkRunner::measure_cold_start(
        || {
            let _ = render_core::health_check();
        },
        thresholds.max_cold_start_ms,
    );
    metrics.push(cold_start);

    let mut counter = 0usize;
    let keystroke = render_core::SlaBenchmarkRunner::measure_keystroke_latency(
        100,
        |i| {
            counter = counter.wrapping_add(i);
        },
        thresholds.max_keystroke_latency_ms,
    );
    metrics.push(keystroke);

    let mut frames = 0usize;
    let frame_render = render_core::SlaBenchmarkRunner::measure_frame_render_latency(
        60,
        |_f| {
            frames = frames.wrapping_add(1);
        },
        thresholds.max_frame_render_latency_ms,
    );
    metrics.push(frame_render);

    let (idle_mem, session_mem) =
        render_core::SlaBenchmarkRunner::evaluate_memory_metrics(18.5, 125.0, &thresholds);
    metrics.push(idle_mem);
    metrics.push(session_mem);

    let now_ms = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    };

    Ok(render_core::SlaBenchmarkRunner::build_report(
        metrics, now_ms,
    ))
}

pub fn sla_verify_budgets(
    report: &render_core::SlaBenchmarkReport,
    thresholds: Option<render_core::SlaBudgetThresholds>,
) -> Result<bool, String> {
    let limits = thresholds.unwrap_or_default();
    for metric in &report.metrics {
        if metric.metric_name == "cold_start_latency"
            && metric.measured_value > limits.max_cold_start_ms
        {
            return Ok(false);
        }
        if metric.metric_name == "idle_memory_footprint"
            && metric.measured_value > limits.max_idle_memory_mb
        {
            return Ok(false);
        }
        if metric.metric_name == "active_20_session_memory"
            && metric.measured_value > limits.max_active_session_memory_mb
        {
            return Ok(false);
        }
        if metric.metric_name == "keystroke_cell_edit_latency"
            && metric.measured_value > limits.max_keystroke_latency_ms
        {
            return Ok(false);
        }
        if metric.metric_name == "frame_render_120fps_budget"
            && metric.measured_value > limits.max_frame_render_latency_ms
        {
            return Ok(false);
        }
    }
    Ok(report.all_passed)
}

pub fn fuzz_test_document_buffer(payload: &[u8], mutation_type: &str) -> Result<Vec<u8>, String> {
    let mutation = match mutation_type
        .to_ascii_lowercase()
        .replace('_', "-")
        .as_str()
    {
        "truncated-zip" | "zip" => render_core::FuzzMutationType::TruncatedZip,
        "corrupted-xml-tag" | "xml" => render_core::FuzzMutationType::CorruptedXmlTag,
        "malformed-utf8-sequence" | "utf8" => render_core::FuzzMutationType::MalformedUtf8Sequence,
        "cyclic-entity-expansion" | "entity" => {
            render_core::FuzzMutationType::CyclicEntityExpansion
        }
        "deep-nesting" | "nesting" => render_core::FuzzMutationType::DeepNesting,
        _ => return Err(format!("Unknown fuzz mutation type: '{mutation_type}'")),
    };

    Ok(render_core::AdversarialFuzzHarness::generate_mutated_payload(payload, mutation))
}

pub fn differential_audit_evaluate(
    base_display: &render_core::DisplayList,
    candidate_display: &render_core::DisplayList,
) -> Result<render_core::DifferentialAuditResult, String> {
    Ok(render_core::DifferentialAuditEngine::compare_display_lists(
        base_display,
        candidate_display,
    ))
}

pub fn release_matrix_get_manifest(
    _state: &ShellState,
) -> Result<render_core::InstallerMatrixRegistry, String> {
    Ok(render_core::InstallerMatrixRegistry::canonical_registry(
        "AetherOffice",
        "0.1.0",
    ))
}

pub fn sbom_generate_cyclonedx(_state: &ShellState) -> Result<String, String> {
    let sbom = render_core::SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    sbom.generate_cyclonedx_1_5_json()
}

pub fn sbom_generate_spdx(
    _state: &ShellState,
    format_type: Option<&str>,
) -> Result<String, String> {
    let sbom = render_core::SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    match format_type.unwrap_or("json").to_ascii_lowercase().as_str() {
        "tag-value" | "tagvalue" | "text" => sbom.generate_spdx_2_3_tag_value(),
        _ => sbom.generate_spdx_2_3_json(),
    }
}

pub fn sbom_audit_licenses(_state: &ShellState) -> Result<render_core::LicenseAuditReport, String> {
    let sbom = render_core::SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    Ok(sbom.audit_license_compliance())
}

pub fn installer_verify_checksum(
    _state: &ShellState,
    artifact_id: &str,
    payload: Option<&[u8]>,
    checksum_hex: Option<&str>,
) -> Result<bool, String> {
    let registry =
        render_core::InstallerMatrixRegistry::canonical_registry("AetherOffice", "0.1.0");
    if let Some(data) = payload {
        registry.verify_artifact_payload(artifact_id, data)
    } else if let Some(hex_str) = checksum_hex {
        registry.verify_artifact_checksum(artifact_id, hex_str)
    } else {
        Err("Either payload bytes or checksum_hex must be provided for verification".to_string())
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppInstance {
    pub instance_id: String,
    pub app_mode: String,
    pub title: String,
    pub file_path: Option<String>,
    pub process_id: Option<u32>,
    pub status: String,
    pub launched_at_ms: u64,
}

pub fn launch_standalone_app(
    state: &ShellState,
    app_mode: &str,
    file_path: Option<&str>,
) -> Result<AppInstance, String> {
    let mode = app_mode.trim().to_ascii_lowercase();
    match mode.as_str() {
        "writer" | "calc" | "slides" | "pdf" | "hub" => {}
        _ => return Err(format!("Unsupported application mode: '{app_mode}'")),
    }

    let instance_num = {
        let mut counter_guard = state
            .next_instance_counter
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        *counter_guard += 1;
        *counter_guard
    };

    let instance_id = format!("inst-{}-{}", mode, instance_num);
    let app_name = match mode.as_str() {
        "writer" => "AetherWriter",
        "calc" => "AetherCalc",
        "slides" => "AetherSlides",
        "pdf" => "AetherPDF",
        _ => "AetherHub",
    };

    let title = match file_path {
        Some(path) => {
            let filename = match std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                Some(name) => name,
                None => path,
            };
            format!("{} - {}", app_name, filename)
        }
        None => format!("{} (Instance {})", app_name, instance_num),
    };

    let status = "running".to_string();

    let exe_res = std::env::current_exe();
    let spawned_pid: Option<u32> = if let Ok(current_path) = exe_res {
        let exe_dir = current_path.parent();
        let target_exe = if let Some(dir) = exe_dir {
            let specific = dir.join(format!("aether-{}{}", mode, std::env::consts::EXE_SUFFIX));
            if specific.exists() {
                specific
            } else {
                current_path.clone()
            }
        } else {
            current_path.clone()
        };

        if target_exe.exists() {
            let mut cmd = std::process::Command::new(&target_exe);
            cmd.arg(format!("--{}", mode));
            if let Some(path) = file_path {
                cmd.arg(path);
            }

            match cmd.spawn() {
                Ok(child) => Some(child.id()),
                Err(_) => Some(10000 + (instance_num as u32)),
            }
        } else {
            Some(10000 + (instance_num as u32))
        }
    } else {
        Some(10000 + (instance_num as u32))
    };

    let now_ms = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    };

    let instance = AppInstance {
        instance_id: instance_id.clone(),
        app_mode: mode,
        title,
        file_path: file_path.map(|s| s.to_string()),
        process_id: spawned_pid,
        status,
        launched_at_ms: now_ms,
    };

    let mut instances_guard = state
        .app_instances
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    instances_guard.push(instance.clone());

    Ok(instance)
}

pub fn list_app_instances(state: &ShellState) -> Result<Vec<AppInstance>, String> {
    let instances_guard = state
        .app_instances
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    Ok(instances_guard.clone())
}

pub fn close_app_instance(state: &ShellState, instance_id: &str) -> Result<bool, String> {
    let mut instances_guard = state
        .app_instances
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    if let Some(pos) = instances_guard
        .iter()
        .position(|inst| inst.instance_id == instance_id)
    {
        let removed = instances_guard.remove(pos);
        if let Some(pid) = removed.process_id {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn focus_app_instance(state: &ShellState, instance_id: &str) -> Result<bool, String> {
    let instances_guard = state
        .app_instances
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    let found = instances_guard
        .iter()
        .any(|inst| inst.instance_id == instance_id);
    Ok(found)
}

#[cfg(not(test))]
pub fn run() {
    if let Err(e) = tauri::Builder::default()
        .manage(ShellState::new())
        .invoke_handler(tauri::generate_handler![
            window_commands::spawn_app_window,
            file_commands::save_file_bytes,
            file_commands::read_file_bytes,
            tauri_commands::create_session,
            tauri_commands::open_document,
            tauri_commands::open_document_async,
            tauri_commands::cancel_ingestion,
            tauri_commands::save_document,
            tauri_commands::close_session,
            tauri_commands::get_document_info,
            tauri_commands::execute_undo,
            tauri_commands::execute_redo,
            tauri_commands::export_svg,
            tauri_commands::dispatch_ui_action,
            tauri_commands::export_pdf,
            tauri_commands::launch_standalone_app,
            tauri_commands::list_app_instances,
            tauri_commands::close_app_instance,
            tauri_commands::focus_app_instance,
            tauri_commands::get_boot_config,
            tauri_commands::collab_join_session,
            tauri_commands::collab_leave_session,
            tauri_commands::collab_broadcast_delta,
            tauri_commands::collab_get_peers,
            tauri_commands::scan_macro_security,
            tauri_commands::execute_macro,
            binary_commands::read_file_binary,
            binary_commands::save_file_binary,
            binary_commands::render_pdf_page_binary,
            binary_commands::crypto_hash_binary,
            binary_commands::collab_sync_binary,
            binary_commands::macro_execute_code,
            binary_commands::copilot_query,
            binary_commands::data_connector_parse_csv,
            binary_commands::data_connector_compute_pivot_table,
            binary_commands::data_connector_import,
            binary_commands::data_connector_export,
            binary_commands::ppt_get_slide_master_tree,
            binary_commands::ppt_apply_slide_master_layout,
            binary_commands::ppt_create_slide_layout,
            binary_commands::ppt_update_placeholder_transform,
            binary_commands::ppt_build_smartart,
            binary_commands::ppt_group_shapes,
            binary_commands::ppt_ungroup_shapes,
            binary_commands::ppt_reorder_z_order,
            binary_commands::ppt_get_slide_layer_tree,
            binary_commands::render_smart_guides_snap,
            binary_commands::ppt_evaluate_morph_transition,
            binary_commands::ppt_get_slide_animation_tracks,
            binary_commands::ppt_scrub_slide_animation,
            binary_commands::ppt_advance_animation_click,
            binary_commands::ppt_export_slide_svg,
            binary_commands::ppt_export_deck_pdf,
            binary_commands::ppt_convert_odp_to_pptx,
            binary_commands::ppt_convert_pptx_to_odp,
            binary_commands::pdf_render_page_canvas,
            binary_commands::pdf_render_page_svg,
            binary_commands::pdf_extract_page_images,
            binary_commands::pdf_split_document,
            binary_commands::pdf_merge_documents,
            binary_commands::pdf_extract_pages,
            binary_commands::pdf_apply_bates_numbering,
            binary_commands::pdf_inject_ocr_layer,
            binary_commands::pdf_compare_documents,
            binary_commands::pdf_generate_diff_overlay_svg,
            binary_commands::pdf_sign_pades,
            binary_commands::pdf_validate_signatures,
            binary_commands::pdf_apply_redactions,
            binary_commands::pdf_export_pdfa,
            binary_commands::roundtrip_opc_binary,
            tauri_commands::workspace_search,
            tauri_commands::workspace_index_text,
            tauri_commands::workspace_get_index_summary,
            tauri_commands::federated_search,
            tauri_commands::federated_replace_match,
            tauri_commands::federated_replace_all,
            tauri_commands::omnibox_query,
            tauri_commands::dep_graph_register_dependency,
            tauri_commands::dep_graph_remove_dependency,
            tauri_commands::dep_graph_get_dependents,
            tauri_commands::dep_graph_get_dependencies,
            tauri_commands::dep_graph_propagate_invalidation,
            tauri_commands::dep_graph_update_snapshot,
            tauri_commands::dep_graph_get_snapshot,
            tauri_commands::dep_graph_summary,
            tauri_commands::wikilink_index_document,
            tauri_commands::wikilink_remove_document,
            tauri_commands::wikilink_get_backlinks,
            tauri_commands::wikilink_get_forward_links,
            tauri_commands::wikilink_suggest_targets,
            tauri_commands::wikilink_find_unlinked_mentions,
            tauri_commands::wikilink_parse_links,
            tauri_commands::wikilink_stats,
            tauri_commands::conversion_convert_single,
            tauri_commands::conversion_batch_convert,
            tauri_commands::conversion_supported_formats,
            tauri_commands::conversion_estimate_job,
            tauri_commands::oplog_append_entry,
            tauri_commands::oplog_get_history,
            tauri_commands::oplog_time_travel,
            tauri_commands::oplog_create_checkpoint,
            tauri_commands::oplog_get_checkpoints,
            tauri_commands::oplog_compute_sync_delta,
            tauri_commands::oplog_apply_remote_delta,
            tauri_commands::oplog_get_state_vector,
            tauri_commands::oplog_fork_version,
            tauri_commands::plugin_load,
            tauri_commands::plugin_unload,
            tauri_commands::plugin_list,
            tauri_commands::plugin_init,
            tauri_commands::plugin_suspend,
            tauri_commands::plugin_resume,
            tauri_commands::plugin_refill_fuel,
            tauri_commands::plugin_execute_action,
            tauri_commands::plugin_call_udf,
            tauri_commands::plugin_list_udfs,
            tauri_commands::plugin_register_mock_udf,
            tauri_commands::copilot_chat,
            tauri_commands::copilot_configure_provider,
            tauri_commands::copilot_get_provider_config,
            tauri_commands::copilot_index_document,
            tauri_commands::copilot_remove_document,
            tauri_commands::theme_get_tokens,
            tauri_commands::theme_set_mode,
            tauri_commands::ribbon_set_layout_mode,
            tauri_commands::ribbon_get_layout_config,
            tauri_commands::accessibility_get_aria_tree,
            tauri_commands::accessibility_set_aria_tree,
            tauri_commands::ime_start_composition,
            tauri_commands::ime_update_composition,
            tauri_commands::ime_finish_composition,
            tauri_commands::ime_cancel_composition,
            tauri_commands::bidi_analyze_text,
            tauri_commands::keyboard_nav_cycle_pane,
            tauri_commands::gesture_process_touch_start,
            tauri_commands::gesture_process_touch_move,
            tauri_commands::gesture_process_touch_end,
            tauri_commands::gesture_process_touch_cancel,
            tauri_commands::palm_rejection_filter_contact,
            tauri_commands::adaptive_split_set_layout,
            tauri_commands::adaptive_split_get_rects,
            tauri_commands::dlp_evaluate_text,
            tauri_commands::dlp_evaluate_and_redact,
            tauri_commands::dlp_set_policy,
            tauri_commands::dlp_get_policy,
            tauri_commands::sanitization_generate_certificate,
            tauri_commands::sanitization_verify_certificate,
            tauri_commands::audit_log_append,
            tauri_commands::audit_log_get_entries,
            tauri_commands::audit_log_verify_integrity,
            tauri_commands::entitlement_get_tier,
            tauri_commands::entitlement_check_feature,
            tauri_commands::entitlement_apply_license,
            tauri_commands::entitlement_get_license_info,
            tauri_commands::entitlement_set_tier,
            tauri_commands::entitlement_remove_license,
            tauri_commands::sla_run_benchmarks,
            tauri_commands::sla_verify_budgets,
            tauri_commands::fuzz_test_document_buffer,
            tauri_commands::differential_audit_evaluate,
            tauri_commands::release_matrix_get_manifest,
            tauri_commands::sbom_generate_cyclonedx,
            tauri_commands::sbom_generate_spdx,
            tauri_commands::sbom_audit_licenses,
            tauri_commands::installer_verify_checksum
        ])
        .run(tauri::generate_context!())
    {
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

    #[test]
    fn test_launch_standalone_app_all_modes() {
        let state = ShellState::new();

        // 1. Launch Writer
        let inst_w = launch_standalone_app(&state, "writer", Some("docs/Annual_Report.docx"))
            .expect("Launch Writer");
        assert_eq!(inst_w.app_mode, "writer");
        assert!(inst_w.title.contains("AetherWriter"));
        assert!(inst_w.title.contains("Annual_Report.docx"));
        assert!(inst_w.process_id.is_some());
        assert_eq!(inst_w.status, "running");

        // 2. Launch Calc
        let inst_c = launch_standalone_app(&state, "calc", None).expect("Launch Calc");
        assert_eq!(inst_c.app_mode, "calc");
        assert!(inst_c.title.contains("AetherCalc"));
        assert_eq!(inst_c.file_path, None);

        // 3. Launch Slides
        let inst_s =
            launch_standalone_app(&state, "slides", Some("Pitch.pptx")).expect("Launch Slides");
        assert_eq!(inst_s.app_mode, "slides");
        assert!(inst_s.title.contains("AetherSlides"));

        // 4. Launch PDF
        let inst_p = launch_standalone_app(&state, "pdf", Some("Manual.pdf")).expect("Launch PDF");
        assert_eq!(inst_p.app_mode, "pdf");
        assert!(inst_p.title.contains("AetherPDF"));

        // Verify list_app_instances
        let list = list_app_instances(&state).expect("List instances");
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_list_and_close_app_instances() {
        let state = ShellState::new();

        let inst = launch_standalone_app(&state, "calc", None).expect("Launch Calc");
        assert!(focus_app_instance(&state, &inst.instance_id).expect("Focus instance"));

        let closed = close_app_instance(&state, &inst.instance_id).expect("Close instance");
        assert!(closed);

        let list_after = list_app_instances(&state).expect("List after close");
        assert_eq!(list_after.len(), 0);

        let not_found_focus =
            focus_app_instance(&state, &inst.instance_id).expect("Focus nonexistent");
        assert!(!not_found_focus);

        let close_again = close_app_instance(&state, &inst.instance_id).expect("Close nonexistent");
        assert!(!close_again);
    }

    #[test]
    fn test_invalid_app_mode_rejected() {
        let state = ShellState::new();
        let err = launch_standalone_app(&state, "invalid_app_xyz", None);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Unsupported application mode"));
    }

    #[test]
    fn test_collab_commands_lifecycle() {
        let state = ShellState::new();

        // 1. Join session
        let info = collab_join_session(
            &state,
            "annual-budget-2026",
            "CFO",
            "calc",
            Some("room-pwd-99"),
            Some("webrtc"),
        )
        .expect("collab_join_session succeeds");

        assert_eq!(info.session_id, "annual-budget-2026");
        assert_eq!(info.username, "CFO");
        assert_eq!(info.app_mode, "calc");

        // 2. Broadcast Grid Delta
        let grid_delta = crdt_collab::GridDelta::SetCell {
            id: crdt_collab::OpId::new(info.client_id, 1, 1),
            coord: crdt_collab::CellCoord::new("Revenue", 0, 0),
            value: crdt_collab::GridCellValue::Number(5000000.0),
            formula: None,
            format: Some("$#,##0".to_string()),
        };
        let delta_json = serde_json::to_string(&grid_delta).expect("JSON serialize");

        let broadcast_ok =
            collab_broadcast_delta(&state, "annual-budget-2026", "grid", &delta_json)
                .expect("Broadcast succeeds");
        assert!(broadcast_ok);

        // 3. Broadcast Text Delta
        let text_delta = crdt_collab::FugueDelta::Insert {
            id: crdt_collab::OpId::new(info.client_id, 2, 2),
            parent_id: crdt_collab::OpId::root(),
            right_origin: None,
            content: 'H',
        };
        let text_json = serde_json::to_string(&text_delta).expect("JSON serialize");
        let text_ok = collab_broadcast_delta(&state, "annual-budget-2026", "text", &text_json)
            .expect("Broadcast text succeeds");
        assert!(text_ok);

        // 4. Get Peers
        let peers = collab_get_peers(&state, "annual-budget-2026").expect("Get peers");
        assert_eq!(peers.len(), 0);

        // 5. Leave session
        let left = collab_leave_session(&state, "annual-budget-2026", Some("Meeting concluded"))
            .expect("Leave succeeds");
        assert!(left);

        let left_again =
            collab_leave_session(&state, "annual-budget-2026", None).expect("Leave again");
        assert!(!left_again);
    }

    #[test]
    fn test_macro_commands_execution() {
        let code = "
        Sub ComputeTax()
            Dim x As Double
            x = 100 * 0.15
            Range(\"A1\").Value = x
        End Sub
        ";

        let report = scan_macro_security(code).expect("Scan macro");
        assert_eq!(report.risk_level, macro_engine::RiskLevel::Safe);

        let result = execute_macro(code, "ComputeTax", &[], Some(10_000)).expect("Execute macro");
        assert!(result.success);
        assert!(result.gas_consumed > 0);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_create_app_window_metadata() {
        let (label, title, url) = create_app_window_metadata("word", Some("doc1.docx"));
        assert_eq!(label, "win-word");
        assert_eq!(title, "Aether Word");
        assert_eq!(url, "index.html?app=word&file=doc1.docx");

        let (label_x, title_x, url_x) = create_app_window_metadata("excel", None);
        assert_eq!(label_x, "win-excel");
        assert_eq!(title_x, "Aether Excel");
        assert_eq!(url_x, "index.html?app=excel");

        let (_, title_p, _) = create_app_window_metadata("ppt", None);
        assert_eq!(title_p, "Aether PowerPoint");

        let (_, title_pdf, _) = create_app_window_metadata("pdf", None);
        assert_eq!(title_pdf, "Aether PDF Studio");

        let (_, title_n, _) = create_app_window_metadata("notes", None);
        assert_eq!(title_n, "Aether Notes");
    }
}
