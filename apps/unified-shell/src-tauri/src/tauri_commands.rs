//! Tauri Command Registration Wrappers (GEC-v7.0 Stage 43)
//!
//! Exposes strongly-typed Tauri command entrypoints for:
//! - Multi-document sessions (`create_session`, `open_document`, `save_document`, `close_session`, `get_document_info`)
//! - Engine undo/redo & UI dispatch (`execute_undo`, `execute_redo`, `dispatch_ui_action`)
//! - Export engines (`export_pdf`, `export_svg`)
//! - Multi-window process lifecycle (`launch_standalone_app`, `list_app_instances`, `close_app_instance`, `focus_app_instance`, `get_boot_config`)
//! - Collaborative editing (`collab_join_session`, `collab_leave_session`, `collab_broadcast_delta`, `collab_get_peers`)

use crate::{AppInstance, BootConfig, ShellState};
use render_core::{
    DocumentMetadata, DocumentType, SessionSummary, UiActionEvent, UiDispatchResult,
};

// -----------------------------------------------------------------------------
// Production Tauri Commands (#[cfg(not(test))])
// -----------------------------------------------------------------------------

#[cfg(not(test))]
#[tauri::command]
pub fn create_session(
    state: tauri::State<'_, ShellState>,
    doc_type: DocumentType,
    title: String,
) -> Result<SessionSummary, String> {
    crate::create_session(&state, doc_type, &title)
}

#[cfg(not(test))]
#[tauri::command]
pub fn open_document(
    state: tauri::State<'_, ShellState>,
    file_path: String,
) -> Result<SessionSummary, String> {
    crate::open_document(&state, &file_path)
}

#[cfg(not(test))]
#[tauri::command]
pub async fn open_document_async(
    app: tauri::AppHandle,
    state: tauri::State<'_, ShellState>,
    file_path: String,
    session_id: Option<String>,
) -> Result<SessionSummary, String> {
    use tauri::Emitter;
    let app_handle = app.clone();
    crate::open_document_async(&state, &file_path, session_id.as_deref(), move |progress| {
        let _ = app_handle.emit("ingestion-progress", &progress);
    })
}

#[cfg(not(test))]
#[tauri::command]
pub fn cancel_ingestion(
    state: tauri::State<'_, ShellState>,
    session_id: String,
) -> Result<bool, String> {
    crate::cancel_ingestion(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn save_document(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    file_path: Option<String>,
) -> Result<String, String> {
    crate::save_document(&state, &session_id, file_path.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn export_pdf(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    output_path: String,
) -> Result<String, String> {
    crate::export_pdf(&state, &session_id, &output_path)
}

#[cfg(not(test))]
#[tauri::command]
pub fn get_document_info(
    state: tauri::State<'_, ShellState>,
    session_id: String,
) -> Result<DocumentMetadata, String> {
    crate::get_document_info(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn close_session(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    force: bool,
) -> Result<Option<String>, String> {
    crate::close_session(&state, &session_id, force)
}

#[cfg(not(test))]
#[tauri::command]
pub fn execute_undo(state: tauri::State<'_, ShellState>, session_id: String) -> Result<(), String> {
    crate::execute_undo(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn execute_redo(state: tauri::State<'_, ShellState>, session_id: String) -> Result<(), String> {
    crate::execute_redo(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn export_svg(
    state: tauri::State<'_, ShellState>,
    session_id: String,
) -> Result<String, String> {
    crate::export_svg(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dispatch_ui_action(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    action: UiActionEvent,
) -> Result<UiDispatchResult, String> {
    crate::dispatch_ui_action(&state, &session_id, action)
}

#[cfg(not(test))]
#[tauri::command]
pub fn get_boot_config() -> BootConfig {
    crate::get_boot_config()
}

#[cfg(not(test))]
#[tauri::command]
pub fn launch_standalone_app(
    state: tauri::State<'_, ShellState>,
    app_mode: String,
    file_path: Option<String>,
) -> Result<AppInstance, String> {
    crate::launch_standalone_app(&state, &app_mode, file_path.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn list_app_instances(state: tauri::State<'_, ShellState>) -> Result<Vec<AppInstance>, String> {
    crate::list_app_instances(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn close_app_instance(
    state: tauri::State<'_, ShellState>,
    instance_id: String,
) -> Result<bool, String> {
    crate::close_app_instance(&state, &instance_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn focus_app_instance(
    state: tauri::State<'_, ShellState>,
    instance_id: String,
) -> Result<bool, String> {
    crate::focus_app_instance(&state, &instance_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn collab_join_session(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    username: String,
    app_mode: String,
    room_key: Option<String>,
    transport_mode: Option<String>,
) -> Result<crate::collab_commands::CollabSessionInfo, String> {
    crate::collab_commands::collab_join_session(
        &state,
        &session_id,
        &username,
        &app_mode,
        room_key.as_deref(),
        transport_mode.as_deref(),
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn collab_leave_session(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    reason: Option<String>,
) -> Result<bool, String> {
    crate::collab_commands::collab_leave_session(&state, &session_id, reason.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn collab_broadcast_delta(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    delta_type: String,
    delta_json: String,
) -> Result<bool, String> {
    crate::collab_commands::collab_broadcast_delta(&state, &session_id, &delta_type, &delta_json)
}

#[cfg(not(test))]
#[tauri::command]
pub fn collab_get_peers(
    state: tauri::State<'_, ShellState>,
    session_id: String,
) -> Result<Vec<crdt_collab::PeerPresence>, String> {
    crate::collab_commands::collab_get_peers(&state, &session_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn scan_macro_security(code: String) -> Result<macro_engine::SecurityAuditReport, String> {
    crate::macro_commands::scan_macro_security(&code)
}

#[cfg(not(test))]
#[tauri::command]
pub fn execute_macro(
    code: String,
    routine_name: String,
    granted_capabilities: Vec<String>,
    gas_limit: Option<u64>,
) -> Result<crate::macro_commands::MacroExecutionResult, String> {
    crate::macro_commands::execute_macro(&code, &routine_name, &granted_capabilities, gas_limit)
}

#[cfg(not(test))]
#[tauri::command]
pub fn workspace_search(
    state: tauri::State<'_, ShellState>,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<copilot_core::HybridSearchResult>, String> {
    crate::workspace_search(&state, &query, top_k)
}

#[cfg(not(test))]
#[tauri::command]
pub fn workspace_index_text(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    title: String,
    text: String,
) -> Result<usize, String> {
    crate::workspace_index_text(&state, &doc_id, &title, &text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn workspace_get_index_summary(
    state: tauri::State<'_, ShellState>,
) -> Result<copilot_core::IndexSummary, String> {
    crate::workspace_get_index_summary(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn federated_search(
    state: tauri::State<'_, ShellState>,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<Vec<render_core::FederatedSearchMatch>, String> {
    crate::federated_search(&state, &query, case_sensitive, whole_word, use_regex)
}

#[cfg(not(test))]
#[tauri::command]
pub fn federated_replace_match(
    state: tauri::State<'_, ShellState>,
    session_id: String,
    query: String,
    replacement: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<usize, String> {
    crate::federated_replace_match(
        &state,
        &session_id,
        &query,
        &replacement,
        case_sensitive,
        whole_word,
        use_regex,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn federated_replace_all(
    state: tauri::State<'_, ShellState>,
    query: String,
    replacement: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    target_session_ids: Option<Vec<String>>,
) -> Result<render_core::FederatedReplaceSummary, String> {
    crate::federated_replace_all(
        &state,
        &query,
        &replacement,
        case_sensitive,
        whole_word,
        use_regex,
        target_session_ids,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn omnibox_query(
    state: tauri::State<'_, ShellState>,
    query: String,
    max_results: Option<usize>,
) -> Result<Vec<render_core::OmniboxItem>, String> {
    crate::omnibox_query(&state, &query, max_results)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_register_dependency(
    state: tauri::State<'_, ShellState>,
    edge: render_core::DependencyEdge,
) -> Result<String, String> {
    crate::dep_graph_register_dependency(&state, edge)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_remove_dependency(
    state: tauri::State<'_, ShellState>,
    edge_id: String,
) -> Result<bool, String> {
    crate::dep_graph_remove_dependency(&state, &edge_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_get_dependents(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    crate::dep_graph_get_dependents(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_get_dependencies(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    crate::dep_graph_get_dependencies(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_propagate_invalidation(
    state: tauri::State<'_, ShellState>,
    source_doc_id: String,
    affected_anchor: Option<render_core::EmbedAnchor>,
) -> Result<Vec<String>, String> {
    crate::dep_graph_propagate_invalidation(&state, &source_doc_id, affected_anchor)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_update_snapshot(
    state: tauri::State<'_, ShellState>,
    snapshot: render_core::LiveEmbedSnapshot,
) -> Result<(), String> {
    crate::dep_graph_update_snapshot(&state, snapshot)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_get_snapshot(
    state: tauri::State<'_, ShellState>,
    embed_id: String,
) -> Result<Option<render_core::LiveEmbedSnapshot>, String> {
    crate::dep_graph_get_snapshot(&state, &embed_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dep_graph_summary(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::CrossDocSummaryDto, String> {
    crate::dep_graph_summary(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_index_document(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    crate::wikilink_index_document(&state, &doc_id, &title, &content)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_remove_document(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<(), String> {
    crate::wikilink_remove_document(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_get_backlinks(
    state: tauri::State<'_, ShellState>,
    target_title_or_id: String,
) -> Result<Vec<render_core::BacklinkReference>, String> {
    crate::wikilink_get_backlinks(&state, &target_title_or_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_get_forward_links(
    state: tauri::State<'_, ShellState>,
    source_doc_id: String,
) -> Result<Vec<render_core::WikiLink>, String> {
    crate::wikilink_get_forward_links(&state, &source_doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_suggest_targets(
    state: tauri::State<'_, ShellState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<render_core::WikiLinkSuggestion>, String> {
    crate::wikilink_suggest_targets(&state, &query, limit)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_find_unlinked_mentions(
    state: tauri::State<'_, ShellState>,
    target_title: String,
) -> Result<Vec<render_core::UnlinkedMention>, String> {
    crate::wikilink_find_unlinked_mentions(&state, &target_title)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_parse_links(text: String) -> Result<Vec<render_core::WikiLink>, String> {
    crate::wikilink_parse_links(&text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn wikilink_stats(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::WikiLinkIndexStats, String> {
    crate::wikilink_stats(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn conversion_convert_single(
    task: render_core::ConversionTask,
) -> Result<render_core::ConversionResult, String> {
    crate::conversion_convert_single(task)
}

#[cfg(not(test))]
#[tauri::command]
pub fn conversion_batch_convert(
    tasks: Vec<render_core::ConversionTask>,
) -> Result<render_core::BatchConversionReport, String> {
    crate::conversion_batch_convert(tasks)
}

#[cfg(not(test))]
#[tauri::command]
pub fn conversion_supported_formats(
    from: render_core::ConversionFormat,
) -> Result<Vec<render_core::ConversionFormat>, String> {
    crate::conversion_supported_formats(from)
}

#[cfg(not(test))]
#[tauri::command]
pub fn conversion_estimate_job(
    task_count: usize,
    total_bytes: usize,
) -> Result<render_core::ConversionEstimateDto, String> {
    crate::conversion_estimate_job(task_count, total_bytes)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_append_entry(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    author_id: String,
    payload: crdt_collab::OpPayload,
    timestamp_ms: u64,
    client_id: u64,
) -> Result<crdt_collab::OpLogEntry, String> {
    crate::oplog_append_entry(
        &state,
        &doc_id,
        &author_id,
        payload,
        timestamp_ms,
        client_id,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_get_history(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    crate::oplog_get_history(&state, &doc_id, limit, offset)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_time_travel(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    target_timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    crate::oplog_time_travel(&state, &doc_id, target_timestamp_ms)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_create_checkpoint(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    author_id: String,
    label: String,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::CheckpointMeta, String> {
    crate::oplog_create_checkpoint(&state, &doc_id, &author_id, &label, client_id, timestamp_ms)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_get_checkpoints(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<Vec<crdt_collab::CheckpointMeta>, String> {
    crate::oplog_get_checkpoints(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_compute_sync_delta(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    remote_vector: crdt_collab::StateVector,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    crate::oplog_compute_sync_delta(&state, &doc_id, remote_vector)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_apply_remote_delta(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    remote_entries: Vec<crdt_collab::OpLogEntry>,
) -> Result<crdt_collab::SyncReport, String> {
    crate::oplog_apply_remote_delta(&state, &doc_id, remote_entries)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_get_state_vector(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<crdt_collab::StateVector, String> {
    crate::oplog_get_state_vector(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn oplog_fork_version(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    target_op_id: crdt_collab::OpId,
    new_doc_id: String,
    new_author_id: String,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    crate::oplog_fork_version(
        &state,
        &doc_id,
        target_op_id,
        &new_doc_id,
        &new_author_id,
        client_id,
        timestamp_ms,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_load(
    state: tauri::State<'_, ShellState>,
    manifest_json: Option<String>,
    wasm_bytes: Vec<u8>,
) -> Result<String, String> {
    crate::plugin_load(&state, manifest_json, wasm_bytes)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_unload(state: tauri::State<'_, ShellState>, plugin_id: String) -> Result<(), String> {
    crate::plugin_unload(&state, &plugin_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_list(
    state: tauri::State<'_, ShellState>,
) -> Result<Vec<plugin_runtime::PluginStatusDto>, String> {
    crate::plugin_list(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_init(state: tauri::State<'_, ShellState>, plugin_id: String) -> Result<(), String> {
    crate::plugin_init(&state, &plugin_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_suspend(
    state: tauri::State<'_, ShellState>,
    plugin_id: String,
) -> Result<(), String> {
    crate::plugin_suspend(&state, &plugin_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_resume(state: tauri::State<'_, ShellState>, plugin_id: String) -> Result<(), String> {
    crate::plugin_resume(&state, &plugin_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_refill_fuel(
    state: tauri::State<'_, ShellState>,
    plugin_id: String,
) -> Result<(), String> {
    crate::plugin_refill_fuel(&state, &plugin_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_execute_action(
    state: tauri::State<'_, ShellState>,
    plugin_id: String,
    action: plugin_runtime::PluginAction,
) -> Result<plugin_runtime::ActionResult, String> {
    crate::plugin_execute_action(&state, &plugin_id, action)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_call_udf(
    state: tauri::State<'_, ShellState>,
    name: String,
    args: Vec<String>,
) -> Result<plugin_runtime::PluginUdfCallResultDto, String> {
    crate::plugin_call_udf(&state, &name, &args)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_list_udfs(
    state: tauri::State<'_, ShellState>,
) -> Result<Vec<plugin_runtime::PluginUdfMetaDto>, String> {
    crate::plugin_list_udfs(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn plugin_register_mock_udf(
    state: tauri::State<'_, ShellState>,
    name: String,
    description: String,
    min_args: usize,
    max_args: usize,
    formula_kind: String,
) -> Result<(), String> {
    crate::plugin_register_mock_udf(
        &state,
        &name,
        &description,
        min_args,
        max_args,
        &formula_kind,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn copilot_chat(
    state: tauri::State<'_, ShellState>,
    request: copilot_core::CopilotChatRequest,
) -> Result<copilot_core::CopilotChatResponse, String> {
    crate::copilot_chat(&state, request)
}

#[cfg(not(test))]
#[tauri::command]
pub fn copilot_configure_provider(
    state: tauri::State<'_, ShellState>,
    config: copilot_core::ProviderConfig,
) -> Result<copilot_core::ProviderStatusDto, String> {
    crate::copilot_configure_provider(&state, config)
}

#[cfg(not(test))]
#[tauri::command]
pub fn copilot_get_provider_config(
    state: tauri::State<'_, ShellState>,
) -> Result<copilot_core::ProviderStatusDto, String> {
    crate::copilot_get_provider_config(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn copilot_index_document(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
    title: String,
    text: String,
) -> Result<usize, String> {
    crate::copilot_index_document(&state, &doc_id, &title, &text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn copilot_remove_document(
    state: tauri::State<'_, ShellState>,
    doc_id: String,
) -> Result<usize, String> {
    crate::copilot_remove_document(&state, &doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn theme_get_tokens(
    state: tauri::State<'_, ShellState>,
    theme_mode: Option<String>,
    ribbon_mode: Option<String>,
) -> Result<render_core::ThemeTokens, String> {
    crate::theme_get_tokens(&state, theme_mode, ribbon_mode)
}

#[cfg(not(test))]
#[tauri::command]
pub fn theme_set_mode(
    state: tauri::State<'_, ShellState>,
    theme_mode: String,
) -> Result<render_core::ThemeTokens, String> {
    crate::theme_set_mode(&state, &theme_mode)
}

#[cfg(not(test))]
#[tauri::command]
pub fn ribbon_set_layout_mode(
    state: tauri::State<'_, ShellState>,
    layout_mode: String,
) -> Result<render_core::RibbonLayoutConfig, String> {
    crate::ribbon_set_layout_mode(&state, &layout_mode)
}

#[cfg(not(test))]
#[tauri::command]
pub fn ribbon_get_layout_config(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::RibbonLayoutConfig, String> {
    crate::ribbon_get_layout_config(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn accessibility_get_aria_tree(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::AriaTree, String> {
    crate::accessibility_get_aria_tree(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn accessibility_set_aria_tree(
    state: tauri::State<'_, ShellState>,
    tree: render_core::AriaTree,
) -> Result<usize, String> {
    crate::accessibility_set_aria_tree(&state, tree)
}

#[cfg(not(test))]
#[tauri::command]
pub fn ime_start_composition(
    state: tauri::State<'_, ShellState>,
    caret_x: f32,
    caret_y: f32,
    caret_h: f32,
) -> Result<render_core::AriaBoundingBox, String> {
    crate::ime_start_composition(&state, caret_x, caret_y, caret_h)
}

#[cfg(not(test))]
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn ime_update_composition(
    state: tauri::State<'_, ShellState>,
    text: String,
    cursor: usize,
    sel_start: usize,
    sel_end: usize,
    caret_x: f32,
    caret_y: f32,
    caret_h: f32,
) -> Result<render_core::AriaBoundingBox, String> {
    crate::ime_update_composition(
        &state, text, cursor, sel_start, sel_end, caret_x, caret_y, caret_h,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn ime_finish_composition(state: tauri::State<'_, ShellState>) -> Result<String, String> {
    crate::ime_finish_composition(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn ime_cancel_composition(state: tauri::State<'_, ShellState>) -> Result<(), String> {
    crate::ime_cancel_composition(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn bidi_analyze_text(text: String) -> Result<crate::BidiAnalysisResult, String> {
    crate::bidi_analyze_text(text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn keyboard_nav_cycle_pane(
    state: tauri::State<'_, ShellState>,
    reverse: Option<bool>,
) -> Result<render_core::NavFocusPane, String> {
    crate::keyboard_nav_cycle_pane(&state, reverse)
}

#[cfg(not(test))]
#[tauri::command]
pub fn gesture_process_touch_start(
    state: tauri::State<'_, ShellState>,
    touch: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_start(&state, touch)
}

#[cfg(not(test))]
#[tauri::command]
pub fn gesture_process_touch_move(
    state: tauri::State<'_, ShellState>,
    touch: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_move(&state, touch)
}

#[cfg(not(test))]
#[tauri::command]
pub fn gesture_process_touch_end(
    state: tauri::State<'_, ShellState>,
    touch_id: u64,
    timestamp_ms: u64,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_end(&state, touch_id, timestamp_ms)
}

#[cfg(not(test))]
#[tauri::command]
pub fn gesture_process_touch_cancel(
    state: tauri::State<'_, ShellState>,
    touch_id: u64,
) -> Result<(), String> {
    crate::gesture_process_touch_cancel(&state, touch_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn palm_rejection_filter_contact(
    state: tauri::State<'_, ShellState>,
    touch: render_core::TouchPoint,
    active_stylus: Option<render_core::TouchPoint>,
    screen_w: f64,
    screen_h: f64,
) -> Result<bool, String> {
    crate::palm_rejection_filter_contact(&state, touch, active_stylus, screen_w, screen_h)
}

#[cfg(not(test))]
#[tauri::command]
pub fn adaptive_split_set_layout(
    state: tauri::State<'_, ShellState>,
    mode_str: String,
    ratio_x: Option<f64>,
    ratio_y: Option<f64>,
) -> Result<render_core::AdaptiveSplitGrid, String> {
    crate::adaptive_split_set_layout(&state, mode_str, ratio_x, ratio_y)
}

#[cfg(not(test))]
#[tauri::command]
pub fn adaptive_split_get_rects(
    state: tauri::State<'_, ShellState>,
    total_width: f64,
    total_height: f64,
    inset_top: Option<f64>,
    inset_bottom: Option<f64>,
) -> Result<Vec<render_core::primitive::Rect2D>, String> {
    crate::adaptive_split_get_rects(&state, total_width, total_height, inset_top, inset_bottom)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dlp_evaluate_text(
    state: tauri::State<'_, ShellState>,
    text: String,
) -> Result<crypto_security::DlpEvaluationResult, String> {
    crate::dlp_evaluate_text(&state, &text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dlp_evaluate_and_redact(
    state: tauri::State<'_, ShellState>,
    text: String,
) -> Result<(crypto_security::DlpEvaluationResult, String), String> {
    crate::dlp_evaluate_and_redact(&state, &text)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dlp_set_policy(
    state: tauri::State<'_, ShellState>,
    policy: crypto_security::DlpPolicy,
) -> Result<(), String> {
    crate::dlp_set_policy(&state, policy)
}

#[cfg(not(test))]
#[tauri::command]
pub fn dlp_get_policy(
    state: tauri::State<'_, ShellState>,
) -> Result<crypto_security::DlpPolicy, String> {
    crate::dlp_get_policy(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn sanitization_generate_certificate(
    doc_id: String,
    original_bytes: Vec<u8>,
    sanitized_bytes: Vec<u8>,
    policy_id: String,
    timestamp_ms: u64,
    redactions: Vec<(String, usize)>,
    secret_key: Vec<u8>,
) -> Result<crypto_security::SanitizationCertificate, String> {
    crate::sanitization_generate_certificate(
        &doc_id,
        &original_bytes,
        &sanitized_bytes,
        &policy_id,
        timestamp_ms,
        redactions,
        &secret_key,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn sanitization_verify_certificate(
    cert: crypto_security::SanitizationCertificate,
    sanitized_bytes: Option<Vec<u8>>,
    secret_key: Vec<u8>,
) -> Result<bool, String> {
    crate::sanitization_verify_certificate(&cert, sanitized_bytes.as_deref(), &secret_key)
}

#[cfg(not(test))]
#[tauri::command]
pub fn audit_log_append(
    state: tauri::State<'_, ShellState>,
    timestamp_ms: u64,
    event_type: crypto_security::AuditEventType,
    user_id: String,
    doc_id: String,
    details: String,
) -> Result<crypto_security::AuditLogEntry, String> {
    crate::audit_log_append(
        &state,
        timestamp_ms,
        event_type,
        &user_id,
        &doc_id,
        &details,
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn audit_log_get_entries(
    state: tauri::State<'_, ShellState>,
    doc_id: Option<String>,
) -> Result<Vec<crypto_security::AuditLogEntry>, String> {
    crate::audit_log_get_entries(&state, doc_id)
}

#[cfg(not(test))]
#[tauri::command]
pub fn audit_log_verify_integrity(state: tauri::State<'_, ShellState>) -> Result<usize, String> {
    crate::audit_log_verify_integrity(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_get_tier(state: tauri::State<'_, ShellState>) -> Result<String, String> {
    crate::entitlement_get_tier(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_check_feature(
    state: tauri::State<'_, ShellState>,
    feature: String,
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    crate::entitlement_check_feature(&state, &feature, now_ms)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_apply_license(
    state: tauri::State<'_, ShellState>,
    token: render_core::LicenseToken,
    verification_key: Vec<u8>,
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    crate::entitlement_apply_license(&state, token, &verification_key, now_ms)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_get_license_info(
    state: tauri::State<'_, ShellState>,
) -> Result<Option<render_core::LicenseToken>, String> {
    crate::entitlement_get_license_info(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_set_tier(
    state: tauri::State<'_, ShellState>,
    tier: String,
) -> Result<String, String> {
    crate::entitlement_set_tier(&state, &tier)
}

#[cfg(not(test))]
#[tauri::command]
pub fn entitlement_remove_license(state: tauri::State<'_, ShellState>) -> Result<bool, String> {
    crate::entitlement_remove_license(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn sla_run_benchmarks(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::SlaBenchmarkReport, String> {
    crate::sla_run_benchmarks(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn sla_verify_budgets(
    report: render_core::SlaBenchmarkReport,
    thresholds: Option<render_core::SlaBudgetThresholds>,
) -> Result<bool, String> {
    crate::sla_verify_budgets(&report, thresholds)
}

#[cfg(not(test))]
#[tauri::command]
pub fn fuzz_test_document_buffer(
    payload: Vec<u8>,
    mutation_type: String,
) -> Result<Vec<u8>, String> {
    crate::fuzz_test_document_buffer(&payload, &mutation_type)
}

#[cfg(not(test))]
#[tauri::command]
pub fn differential_audit_evaluate(
    base_display: render_core::DisplayList,
    candidate_display: render_core::DisplayList,
) -> Result<render_core::DifferentialAuditResult, String> {
    crate::differential_audit_evaluate(&base_display, &candidate_display)
}

#[cfg(not(test))]
#[tauri::command]
pub fn release_matrix_get_manifest(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::InstallerMatrixRegistry, String> {
    crate::release_matrix_get_manifest(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn sbom_generate_cyclonedx(state: tauri::State<'_, ShellState>) -> Result<String, String> {
    crate::sbom_generate_cyclonedx(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn sbom_generate_spdx(
    state: tauri::State<'_, ShellState>,
    format_type: Option<String>,
) -> Result<String, String> {
    crate::sbom_generate_spdx(&state, format_type.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn sbom_audit_licenses(
    state: tauri::State<'_, ShellState>,
) -> Result<render_core::LicenseAuditReport, String> {
    crate::sbom_audit_licenses(&state)
}

#[cfg(not(test))]
#[tauri::command]
pub fn installer_verify_checksum(
    state: tauri::State<'_, ShellState>,
    artifact_id: String,
    payload: Option<Vec<u8>>,
    checksum_hex: Option<String>,
) -> Result<bool, String> {
    crate::installer_verify_checksum(
        &state,
        &artifact_id,
        payload.as_deref(),
        checksum_hex.as_deref(),
    )
}

// -----------------------------------------------------------------------------
// Test Configuration Wrappers (#[cfg(test)])
// -----------------------------------------------------------------------------

#[cfg(test)]
pub fn create_session(
    state: &ShellState,
    doc_type: DocumentType,
    title: String,
) -> Result<SessionSummary, String> {
    crate::create_session(state, doc_type, &title)
}

#[cfg(test)]
pub fn open_document(state: &ShellState, file_path: String) -> Result<SessionSummary, String> {
    crate::open_document(state, &file_path)
}

#[cfg(test)]
pub fn open_document_async(
    state: &ShellState,
    file_path: String,
    session_id: Option<String>,
) -> Result<SessionSummary, String> {
    crate::open_document_async(state, &file_path, session_id.as_deref(), |_| {})
}

#[cfg(test)]
pub fn cancel_ingestion(state: &ShellState, session_id: String) -> Result<bool, String> {
    crate::cancel_ingestion(state, &session_id)
}

#[cfg(test)]
pub fn save_document(
    state: &ShellState,
    session_id: String,
    file_path: Option<String>,
) -> Result<String, String> {
    crate::save_document(state, &session_id, file_path.as_deref())
}

#[cfg(test)]
pub fn export_pdf(
    state: &ShellState,
    session_id: String,
    output_path: String,
) -> Result<String, String> {
    crate::export_pdf(state, &session_id, &output_path)
}

#[cfg(test)]
pub fn get_document_info(
    state: &ShellState,
    session_id: String,
) -> Result<DocumentMetadata, String> {
    crate::get_document_info(state, &session_id)
}

#[cfg(test)]
pub fn close_session(
    state: &ShellState,
    session_id: String,
    force: bool,
) -> Result<Option<String>, String> {
    crate::close_session(state, &session_id, force)
}

#[cfg(test)]
pub fn execute_undo(state: &ShellState, session_id: String) -> Result<(), String> {
    crate::execute_undo(state, &session_id)
}

#[cfg(test)]
pub fn execute_redo(state: &ShellState, session_id: String) -> Result<(), String> {
    crate::execute_redo(state, &session_id)
}

#[cfg(test)]
pub fn export_svg(state: &ShellState, session_id: String) -> Result<String, String> {
    crate::export_svg(state, &session_id)
}

#[cfg(test)]
pub fn dispatch_ui_action(
    state: &ShellState,
    session_id: String,
    action: UiActionEvent,
) -> Result<UiDispatchResult, String> {
    crate::dispatch_ui_action(state, &session_id, action)
}

#[cfg(test)]
pub fn get_boot_config() -> BootConfig {
    crate::get_boot_config()
}

#[cfg(test)]
pub fn launch_standalone_app(
    state: &ShellState,
    app_mode: String,
    file_path: Option<String>,
) -> Result<AppInstance, String> {
    crate::launch_standalone_app(state, &app_mode, file_path.as_deref())
}

#[cfg(test)]
pub fn list_app_instances(state: &ShellState) -> Result<Vec<AppInstance>, String> {
    crate::list_app_instances(state)
}

#[cfg(test)]
pub fn close_app_instance(state: &ShellState, instance_id: String) -> Result<bool, String> {
    crate::close_app_instance(state, &instance_id)
}

#[cfg(test)]
pub fn focus_app_instance(state: &ShellState, instance_id: String) -> Result<bool, String> {
    crate::focus_app_instance(state, &instance_id)
}

#[cfg(test)]
pub fn collab_join_session(
    state: &ShellState,
    session_id: String,
    username: String,
    app_mode: String,
    room_key: Option<String>,
    transport_mode: Option<String>,
) -> Result<crate::collab_commands::CollabSessionInfo, String> {
    crate::collab_commands::collab_join_session(
        state,
        &session_id,
        &username,
        &app_mode,
        room_key.as_deref(),
        transport_mode.as_deref(),
    )
}

#[cfg(test)]
pub fn collab_leave_session(
    state: &ShellState,
    session_id: String,
    reason: Option<String>,
) -> Result<bool, String> {
    crate::collab_commands::collab_leave_session(state, &session_id, reason.as_deref())
}

#[cfg(test)]
pub fn collab_broadcast_delta(
    state: &ShellState,
    session_id: String,
    delta_type: String,
    delta_json: String,
) -> Result<bool, String> {
    crate::collab_commands::collab_broadcast_delta(state, &session_id, &delta_type, &delta_json)
}

#[cfg(test)]
pub fn collab_get_peers(
    state: &ShellState,
    session_id: String,
) -> Result<Vec<crdt_collab::PeerPresence>, String> {
    crate::collab_commands::collab_get_peers(state, &session_id)
}

#[cfg(test)]
pub fn scan_macro_security(code: String) -> Result<macro_engine::SecurityAuditReport, String> {
    crate::macro_commands::scan_macro_security(&code)
}

#[cfg(test)]
pub fn execute_macro(
    code: String,
    routine_name: String,
    granted_capabilities: Vec<String>,
    gas_limit: Option<u64>,
) -> Result<crate::macro_commands::MacroExecutionResult, String> {
    crate::macro_commands::execute_macro(&code, &routine_name, &granted_capabilities, gas_limit)
}

#[cfg(test)]
pub fn workspace_search(
    state: &ShellState,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<copilot_core::HybridSearchResult>, String> {
    crate::workspace_search(state, &query, top_k)
}

#[cfg(test)]
pub fn workspace_index_text(
    state: &ShellState,
    doc_id: String,
    title: String,
    text: String,
) -> Result<usize, String> {
    crate::workspace_index_text(state, &doc_id, &title, &text)
}

#[cfg(test)]
pub fn workspace_get_index_summary(
    state: &ShellState,
) -> Result<copilot_core::IndexSummary, String> {
    crate::workspace_get_index_summary(state)
}

#[cfg(test)]
pub fn federated_search(
    state: &ShellState,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<Vec<render_core::FederatedSearchMatch>, String> {
    crate::federated_search(state, &query, case_sensitive, whole_word, use_regex)
}

#[cfg(test)]
pub fn federated_replace_match(
    state: &ShellState,
    session_id: String,
    query: String,
    replacement: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<usize, String> {
    crate::federated_replace_match(
        state,
        &session_id,
        &query,
        &replacement,
        case_sensitive,
        whole_word,
        use_regex,
    )
}

#[cfg(test)]
pub fn federated_replace_all(
    state: &ShellState,
    query: String,
    replacement: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    target_session_ids: Option<Vec<String>>,
) -> Result<render_core::FederatedReplaceSummary, String> {
    crate::federated_replace_all(
        state,
        &query,
        &replacement,
        case_sensitive,
        whole_word,
        use_regex,
        target_session_ids,
    )
}

#[cfg(test)]
pub fn omnibox_query(
    state: &ShellState,
    query: String,
    max_results: Option<usize>,
) -> Result<Vec<render_core::OmniboxItem>, String> {
    crate::omnibox_query(state, &query, max_results)
}

#[cfg(test)]
pub fn dep_graph_register_dependency(
    state: &ShellState,
    edge: render_core::DependencyEdge,
) -> Result<String, String> {
    crate::dep_graph_register_dependency(state, edge)
}

#[cfg(test)]
pub fn dep_graph_remove_dependency(state: &ShellState, edge_id: String) -> Result<bool, String> {
    crate::dep_graph_remove_dependency(state, &edge_id)
}

#[cfg(test)]
pub fn dep_graph_get_dependents(
    state: &ShellState,
    doc_id: String,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    crate::dep_graph_get_dependents(state, &doc_id)
}

#[cfg(test)]
pub fn dep_graph_get_dependencies(
    state: &ShellState,
    doc_id: String,
) -> Result<Vec<render_core::DependencyEdge>, String> {
    crate::dep_graph_get_dependencies(state, &doc_id)
}

#[cfg(test)]
pub fn dep_graph_propagate_invalidation(
    state: &ShellState,
    source_doc_id: String,
    affected_anchor: Option<render_core::EmbedAnchor>,
) -> Result<Vec<String>, String> {
    crate::dep_graph_propagate_invalidation(state, &source_doc_id, affected_anchor)
}

#[cfg(test)]
pub fn dep_graph_update_snapshot(
    state: &ShellState,
    snapshot: render_core::LiveEmbedSnapshot,
) -> Result<(), String> {
    crate::dep_graph_update_snapshot(state, snapshot)
}

#[cfg(test)]
pub fn dep_graph_get_snapshot(
    state: &ShellState,
    embed_id: String,
) -> Result<Option<render_core::LiveEmbedSnapshot>, String> {
    crate::dep_graph_get_snapshot(state, &embed_id)
}

#[cfg(test)]
pub fn dep_graph_summary(state: &ShellState) -> Result<render_core::CrossDocSummaryDto, String> {
    crate::dep_graph_summary(state)
}

#[cfg(test)]
pub fn wikilink_index_document(
    state: &ShellState,
    doc_id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    crate::wikilink_index_document(state, &doc_id, &title, &content)
}

#[cfg(test)]
pub fn wikilink_remove_document(state: &ShellState, doc_id: String) -> Result<(), String> {
    crate::wikilink_remove_document(state, &doc_id)
}

#[cfg(test)]
pub fn wikilink_get_backlinks(
    state: &ShellState,
    target_title_or_id: String,
) -> Result<Vec<render_core::BacklinkReference>, String> {
    crate::wikilink_get_backlinks(state, &target_title_or_id)
}

#[cfg(test)]
pub fn wikilink_get_forward_links(
    state: &ShellState,
    source_doc_id: String,
) -> Result<Vec<render_core::WikiLink>, String> {
    crate::wikilink_get_forward_links(state, &source_doc_id)
}

#[cfg(test)]
pub fn wikilink_suggest_targets(
    state: &ShellState,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<render_core::WikiLinkSuggestion>, String> {
    crate::wikilink_suggest_targets(state, &query, limit)
}

#[cfg(test)]
pub fn wikilink_find_unlinked_mentions(
    state: &ShellState,
    target_title: String,
) -> Result<Vec<render_core::UnlinkedMention>, String> {
    crate::wikilink_find_unlinked_mentions(state, &target_title)
}

#[cfg(test)]
pub fn wikilink_parse_links(text: String) -> Result<Vec<render_core::WikiLink>, String> {
    crate::wikilink_parse_links(&text)
}

#[cfg(test)]
pub fn wikilink_stats(state: &ShellState) -> Result<render_core::WikiLinkIndexStats, String> {
    crate::wikilink_stats(state)
}

#[cfg(test)]
pub fn conversion_convert_single(
    task: render_core::ConversionTask,
) -> Result<render_core::ConversionResult, String> {
    crate::conversion_convert_single(task)
}

#[cfg(test)]
pub fn conversion_batch_convert(
    tasks: Vec<render_core::ConversionTask>,
) -> Result<render_core::BatchConversionReport, String> {
    crate::conversion_batch_convert(tasks)
}

#[cfg(test)]
pub fn conversion_supported_formats(
    from: render_core::ConversionFormat,
) -> Result<Vec<render_core::ConversionFormat>, String> {
    crate::conversion_supported_formats(from)
}

#[cfg(test)]
pub fn conversion_estimate_job(
    task_count: usize,
    total_bytes: usize,
) -> Result<render_core::ConversionEstimateDto, String> {
    crate::conversion_estimate_job(task_count, total_bytes)
}

#[cfg(test)]
pub fn oplog_append_entry(
    state: &ShellState,
    doc_id: String,
    author_id: String,
    payload: crdt_collab::OpPayload,
    timestamp_ms: u64,
    client_id: u64,
) -> Result<crdt_collab::OpLogEntry, String> {
    crate::oplog_append_entry(state, &doc_id, &author_id, payload, timestamp_ms, client_id)
}

#[cfg(test)]
pub fn oplog_get_history(
    state: &ShellState,
    doc_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    crate::oplog_get_history(state, &doc_id, limit, offset)
}

#[cfg(test)]
pub fn oplog_time_travel(
    state: &ShellState,
    doc_id: String,
    target_timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    crate::oplog_time_travel(state, &doc_id, target_timestamp_ms)
}

#[cfg(test)]
pub fn oplog_create_checkpoint(
    state: &ShellState,
    doc_id: String,
    author_id: String,
    label: String,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::CheckpointMeta, String> {
    crate::oplog_create_checkpoint(state, &doc_id, &author_id, &label, client_id, timestamp_ms)
}

#[cfg(test)]
pub fn oplog_get_checkpoints(
    state: &ShellState,
    doc_id: String,
) -> Result<Vec<crdt_collab::CheckpointMeta>, String> {
    crate::oplog_get_checkpoints(state, &doc_id)
}

#[cfg(test)]
pub fn oplog_compute_sync_delta(
    state: &ShellState,
    doc_id: String,
    remote_vector: crdt_collab::StateVector,
) -> Result<Vec<crdt_collab::OpLogEntry>, String> {
    crate::oplog_compute_sync_delta(state, &doc_id, remote_vector)
}

#[cfg(test)]
pub fn oplog_apply_remote_delta(
    state: &ShellState,
    doc_id: String,
    remote_entries: Vec<crdt_collab::OpLogEntry>,
) -> Result<crdt_collab::SyncReport, String> {
    crate::oplog_apply_remote_delta(state, &doc_id, remote_entries)
}

#[cfg(test)]
pub fn oplog_get_state_vector(
    state: &ShellState,
    doc_id: String,
) -> Result<crdt_collab::StateVector, String> {
    crate::oplog_get_state_vector(state, &doc_id)
}

#[cfg(test)]
pub fn oplog_fork_version(
    state: &ShellState,
    doc_id: String,
    target_op_id: crdt_collab::OpId,
    new_doc_id: String,
    new_author_id: String,
    client_id: u64,
    timestamp_ms: u64,
) -> Result<crdt_collab::MaterializedDocumentState, String> {
    crate::oplog_fork_version(
        state,
        &doc_id,
        target_op_id,
        &new_doc_id,
        &new_author_id,
        client_id,
        timestamp_ms,
    )
}

#[cfg(test)]
pub fn plugin_load(
    state: &ShellState,
    manifest_json: Option<String>,
    wasm_bytes: Vec<u8>,
) -> Result<String, String> {
    crate::plugin_load(state, manifest_json, wasm_bytes)
}

#[cfg(test)]
pub fn plugin_unload(state: &ShellState, plugin_id: String) -> Result<(), String> {
    crate::plugin_unload(state, &plugin_id)
}

#[cfg(test)]
pub fn plugin_list(state: &ShellState) -> Result<Vec<plugin_runtime::PluginStatusDto>, String> {
    crate::plugin_list(state)
}

#[cfg(test)]
pub fn plugin_init(state: &ShellState, plugin_id: String) -> Result<(), String> {
    crate::plugin_init(state, &plugin_id)
}

#[cfg(test)]
pub fn plugin_suspend(state: &ShellState, plugin_id: String) -> Result<(), String> {
    crate::plugin_suspend(state, &plugin_id)
}

#[cfg(test)]
pub fn plugin_resume(state: &ShellState, plugin_id: String) -> Result<(), String> {
    crate::plugin_resume(state, &plugin_id)
}

#[cfg(test)]
pub fn plugin_refill_fuel(state: &ShellState, plugin_id: String) -> Result<(), String> {
    crate::plugin_refill_fuel(state, &plugin_id)
}

#[cfg(test)]
pub fn plugin_execute_action(
    state: &ShellState,
    plugin_id: String,
    action: plugin_runtime::PluginAction,
) -> Result<plugin_runtime::ActionResult, String> {
    crate::plugin_execute_action(state, &plugin_id, action)
}

#[cfg(test)]
pub fn plugin_call_udf(
    state: &ShellState,
    name: String,
    args: Vec<String>,
) -> Result<plugin_runtime::PluginUdfCallResultDto, String> {
    crate::plugin_call_udf(state, &name, &args)
}

#[cfg(test)]
pub fn plugin_list_udfs(
    state: &ShellState,
) -> Result<Vec<plugin_runtime::PluginUdfMetaDto>, String> {
    crate::plugin_list_udfs(state)
}

#[cfg(test)]
pub fn plugin_register_mock_udf(
    state: &ShellState,
    name: String,
    description: String,
    min_args: usize,
    max_args: usize,
    formula_kind: String,
) -> Result<(), String> {
    crate::plugin_register_mock_udf(
        state,
        &name,
        &description,
        min_args,
        max_args,
        &formula_kind,
    )
}

#[cfg(test)]
pub fn copilot_chat(
    state: &ShellState,
    request: copilot_core::CopilotChatRequest,
) -> Result<copilot_core::CopilotChatResponse, String> {
    crate::copilot_chat(state, request)
}

#[cfg(test)]
pub fn copilot_configure_provider(
    state: &ShellState,
    config: copilot_core::ProviderConfig,
) -> Result<copilot_core::ProviderStatusDto, String> {
    crate::copilot_configure_provider(state, config)
}

#[cfg(test)]
pub fn copilot_get_provider_config(
    state: &ShellState,
) -> Result<copilot_core::ProviderStatusDto, String> {
    crate::copilot_get_provider_config(state)
}

#[cfg(test)]
pub fn copilot_index_document(
    state: &ShellState,
    doc_id: String,
    title: String,
    text: String,
) -> Result<usize, String> {
    crate::copilot_index_document(state, &doc_id, &title, &text)
}

#[cfg(test)]
pub fn copilot_remove_document(state: &ShellState, doc_id: String) -> Result<usize, String> {
    crate::copilot_remove_document(state, &doc_id)
}

#[cfg(test)]
pub fn theme_get_tokens(
    state: &ShellState,
    theme_mode: Option<String>,
    ribbon_mode: Option<String>,
) -> Result<render_core::ThemeTokens, String> {
    crate::theme_get_tokens(state, theme_mode, ribbon_mode)
}

#[cfg(test)]
pub fn theme_set_mode(
    state: &ShellState,
    theme_mode: String,
) -> Result<render_core::ThemeTokens, String> {
    crate::theme_set_mode(state, &theme_mode)
}

#[cfg(test)]
pub fn ribbon_set_layout_mode(
    state: &ShellState,
    layout_mode: String,
) -> Result<render_core::RibbonLayoutConfig, String> {
    crate::ribbon_set_layout_mode(state, &layout_mode)
}

#[cfg(test)]
pub fn ribbon_get_layout_config(
    state: &ShellState,
) -> Result<render_core::RibbonLayoutConfig, String> {
    crate::ribbon_get_layout_config(state)
}

#[cfg(test)]
pub fn accessibility_get_aria_tree(state: &ShellState) -> Result<render_core::AriaTree, String> {
    crate::accessibility_get_aria_tree(state)
}

#[cfg(test)]
pub fn accessibility_set_aria_tree(
    state: &ShellState,
    tree: render_core::AriaTree,
) -> Result<usize, String> {
    crate::accessibility_set_aria_tree(state, tree)
}

#[cfg(test)]
pub fn ime_start_composition(
    state: &ShellState,
    caret_x: f32,
    caret_y: f32,
    caret_h: f32,
) -> Result<render_core::AriaBoundingBox, String> {
    crate::ime_start_composition(state, caret_x, caret_y, caret_h)
}

#[cfg(test)]
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
    crate::ime_update_composition(
        state, text, cursor, sel_start, sel_end, caret_x, caret_y, caret_h,
    )
}

#[cfg(test)]
pub fn ime_finish_composition(state: &ShellState) -> Result<String, String> {
    crate::ime_finish_composition(state)
}

#[cfg(test)]
pub fn ime_cancel_composition(state: &ShellState) -> Result<(), String> {
    crate::ime_cancel_composition(state)
}

#[cfg(test)]
pub fn bidi_analyze_text(text: String) -> Result<crate::BidiAnalysisResult, String> {
    crate::bidi_analyze_text(text)
}

#[cfg(test)]
pub fn keyboard_nav_cycle_pane(
    state: &ShellState,
    reverse: Option<bool>,
) -> Result<render_core::NavFocusPane, String> {
    crate::keyboard_nav_cycle_pane(state, reverse)
}

#[cfg(test)]
pub fn gesture_process_touch_start(
    state: &ShellState,
    touch: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_start(state, touch)
}

#[cfg(test)]
pub fn gesture_process_touch_move(
    state: &ShellState,
    touch: render_core::TouchPoint,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_move(state, touch)
}

#[cfg(test)]
pub fn gesture_process_touch_end(
    state: &ShellState,
    touch_id: u64,
    timestamp_ms: u64,
) -> Result<render_core::TouchGesture, String> {
    crate::gesture_process_touch_end(state, touch_id, timestamp_ms)
}

#[cfg(test)]
pub fn gesture_process_touch_cancel(state: &ShellState, touch_id: u64) -> Result<(), String> {
    crate::gesture_process_touch_cancel(state, touch_id)
}

#[cfg(test)]
pub fn palm_rejection_filter_contact(
    state: &ShellState,
    touch: render_core::TouchPoint,
    active_stylus: Option<render_core::TouchPoint>,
    screen_w: f64,
    screen_h: f64,
) -> Result<bool, String> {
    crate::palm_rejection_filter_contact(state, touch, active_stylus, screen_w, screen_h)
}

#[cfg(test)]
pub fn adaptive_split_set_layout(
    state: &ShellState,
    mode_str: String,
    ratio_x: Option<f64>,
    ratio_y: Option<f64>,
) -> Result<render_core::AdaptiveSplitGrid, String> {
    crate::adaptive_split_set_layout(state, mode_str, ratio_x, ratio_y)
}

#[cfg(test)]
pub fn adaptive_split_get_rects(
    state: &ShellState,
    total_width: f64,
    total_height: f64,
    inset_top: Option<f64>,
    inset_bottom: Option<f64>,
) -> Result<Vec<render_core::primitive::Rect2D>, String> {
    crate::adaptive_split_get_rects(state, total_width, total_height, inset_top, inset_bottom)
}

#[cfg(test)]
pub fn dlp_evaluate_text(
    state: &ShellState,
    text: String,
) -> Result<crypto_security::DlpEvaluationResult, String> {
    crate::dlp_evaluate_text(state, &text)
}

#[cfg(test)]
pub fn dlp_evaluate_and_redact(
    state: &ShellState,
    text: String,
) -> Result<(crypto_security::DlpEvaluationResult, String), String> {
    crate::dlp_evaluate_and_redact(state, &text)
}

#[cfg(test)]
pub fn dlp_set_policy(
    state: &ShellState,
    policy: crypto_security::DlpPolicy,
) -> Result<(), String> {
    crate::dlp_set_policy(state, policy)
}

#[cfg(test)]
pub fn dlp_get_policy(state: &ShellState) -> Result<crypto_security::DlpPolicy, String> {
    crate::dlp_get_policy(state)
}

#[cfg(test)]
pub fn sanitization_generate_certificate(
    doc_id: String,
    original_bytes: Vec<u8>,
    sanitized_bytes: Vec<u8>,
    policy_id: String,
    timestamp_ms: u64,
    redactions: Vec<(String, usize)>,
    secret_key: Vec<u8>,
) -> Result<crypto_security::SanitizationCertificate, String> {
    crate::sanitization_generate_certificate(
        &doc_id,
        &original_bytes,
        &sanitized_bytes,
        &policy_id,
        timestamp_ms,
        redactions,
        &secret_key,
    )
}

#[cfg(test)]
pub fn sanitization_verify_certificate(
    cert: crypto_security::SanitizationCertificate,
    sanitized_bytes: Option<Vec<u8>>,
    secret_key: Vec<u8>,
) -> Result<bool, String> {
    crate::sanitization_verify_certificate(&cert, sanitized_bytes.as_deref(), &secret_key)
}

#[cfg(test)]
pub fn audit_log_append(
    state: &ShellState,
    timestamp_ms: u64,
    event_type: crypto_security::AuditEventType,
    user_id: String,
    doc_id: String,
    details: String,
) -> Result<crypto_security::AuditLogEntry, String> {
    crate::audit_log_append(state, timestamp_ms, event_type, &user_id, &doc_id, &details)
}

#[cfg(test)]
pub fn audit_log_get_entries(
    state: &ShellState,
    doc_id: Option<String>,
) -> Result<Vec<crypto_security::AuditLogEntry>, String> {
    crate::audit_log_get_entries(state, doc_id)
}

#[cfg(test)]
pub fn audit_log_verify_integrity(state: &ShellState) -> Result<usize, String> {
    crate::audit_log_verify_integrity(state)
}

#[cfg(test)]
pub fn entitlement_get_tier(state: &ShellState) -> Result<String, String> {
    crate::entitlement_get_tier(state)
}

#[cfg(test)]
pub fn entitlement_check_feature(
    state: &ShellState,
    feature: String,
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    crate::entitlement_check_feature(state, &feature, now_ms)
}

#[cfg(test)]
pub fn entitlement_apply_license(
    state: &ShellState,
    token: render_core::LicenseToken,
    verification_key: Vec<u8>,
    now_ms: Option<u64>,
) -> Result<render_core::EntitlementStatus, String> {
    crate::entitlement_apply_license(state, token, &verification_key, now_ms)
}

#[cfg(test)]
pub fn entitlement_get_license_info(
    state: &ShellState,
) -> Result<Option<render_core::LicenseToken>, String> {
    crate::entitlement_get_license_info(state)
}

#[cfg(test)]
pub fn entitlement_set_tier(state: &ShellState, tier: String) -> Result<String, String> {
    crate::entitlement_set_tier(state, &tier)
}

#[cfg(test)]
pub fn entitlement_remove_license(state: &ShellState) -> Result<bool, String> {
    crate::entitlement_remove_license(state)
}

#[cfg(test)]
pub fn sla_run_benchmarks(state: &ShellState) -> Result<render_core::SlaBenchmarkReport, String> {
    crate::sla_run_benchmarks(state)
}

#[cfg(test)]
pub fn sla_verify_budgets(
    report: render_core::SlaBenchmarkReport,
    thresholds: Option<render_core::SlaBudgetThresholds>,
) -> Result<bool, String> {
    crate::sla_verify_budgets(&report, thresholds)
}

#[cfg(test)]
pub fn fuzz_test_document_buffer(
    payload: Vec<u8>,
    mutation_type: String,
) -> Result<Vec<u8>, String> {
    crate::fuzz_test_document_buffer(&payload, &mutation_type)
}

#[cfg(test)]
pub fn differential_audit_evaluate(
    base_display: render_core::DisplayList,
    candidate_display: render_core::DisplayList,
) -> Result<render_core::DifferentialAuditResult, String> {
    crate::differential_audit_evaluate(&base_display, &candidate_display)
}

#[cfg(test)]
pub fn release_matrix_get_manifest(
    state: &ShellState,
) -> Result<render_core::InstallerMatrixRegistry, String> {
    crate::release_matrix_get_manifest(state)
}

#[cfg(test)]
pub fn sbom_generate_cyclonedx(state: &ShellState) -> Result<String, String> {
    crate::sbom_generate_cyclonedx(state)
}

#[cfg(test)]
pub fn sbom_generate_spdx(
    state: &ShellState,
    format_type: Option<String>,
) -> Result<String, String> {
    crate::sbom_generate_spdx(state, format_type.as_deref())
}

#[cfg(test)]
pub fn sbom_audit_licenses(state: &ShellState) -> Result<render_core::LicenseAuditReport, String> {
    crate::sbom_audit_licenses(state)
}

#[cfg(test)]
pub fn installer_verify_checksum(
    state: &ShellState,
    artifact_id: String,
    payload: Option<Vec<u8>>,
    checksum_hex: Option<String>,
) -> Result<bool, String> {
    crate::installer_verify_checksum(
        state,
        &artifact_id,
        payload.as_deref(),
        checksum_hex.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tauri_command_wrapper_execution() {
        let state = ShellState::new();
        let summary =
            create_session(&state, DocumentType::Word, "Wrapped.docx".to_string()).unwrap();
        assert_eq!(summary.title, "Wrapped.docx");

        let doc_info = get_document_info(&state, summary.session_id.clone()).unwrap();
        assert_eq!(doc_info.session_id, summary.session_id);

        let saved = save_document(
            &state,
            summary.session_id.clone(),
            Some("C:/docs/Saved.docx".to_string()),
        )
        .unwrap();
        assert!(saved.contains("Saved.docx"));

        // Populate content to test federated search
        {
            let mut mgr = state.session_manager.lock().unwrap();
            if let Some(st) = mgr.office_states.get_mut(&summary.session_id) {
                st.word_content = "Document Heading Section\nParagraph content.".to_string();
            }
        }

        // Test federated search & replace command wrappers
        let fed_hits =
            federated_search(&state, "Heading".to_string(), false, false, false).unwrap();
        assert!(!fed_hits.is_empty());

        let replace_summary = federated_replace_all(
            &state,
            "Heading".to_string(),
            "Title".to_string(),
            false,
            false,
            false,
            None,
        )
        .unwrap();
        assert!(replace_summary.total_replacements > 0);

        // Test omnibox query command wrapper
        let omnibox_items = omnibox_query(&state, "save".to_string(), Some(5)).unwrap();
        assert!(!omnibox_items.is_empty());

        let closed = close_session(&state, summary.session_id, true).unwrap();
        assert!(closed.is_none());

        // Test cancel ingestion command wrapper
        let cancel_res = cancel_ingestion(&state, "non-existent-tracking-id".to_string()).unwrap();
        assert!(!cancel_res);

        // Test open_document_async error handling for non-existent path
        let async_err = open_document_async(
            &state,
            "non_existent_file.docx".to_string(),
            Some("track-1".to_string()),
        );
        assert!(async_err.is_err());

        // Test workspace indexing and search command wrappers
        let indexed_count = workspace_index_text(
            &state,
            "doc-search-test".to_string(),
            "Searchable Note".to_string(),
            "Important architectural contract and high performance indexing.".to_string(),
        )
        .unwrap();
        assert!(indexed_count > 0);

        let search_hits =
            workspace_search(&state, "architectural contract".to_string(), Some(5)).unwrap();
        assert!(!search_hits.is_empty());

        let summary_res = workspace_get_index_summary(&state).unwrap();
        assert_eq!(summary_res.total_documents, 1);

        // Test cross-document dependency graph command wrappers
        let edge = render_core::DependencyEdge {
            id: "edge-test-wrapper".to_string(),
            source_doc_id: "calc-doc-1".to_string(),
            source_anchor: render_core::EmbedAnchor::CalcRange {
                sheet_name: "Sheet1".to_string(),
                range_a1: "A1:B10".to_string(),
            },
            target_doc_id: "word-doc-1".to_string(),
            target_container: render_core::EmbedTarget::WordInline {
                container_id: "embed-c1".to_string(),
            },
            kind: render_core::DependencyKind::LiveRangeEmbed,
            created_at: 1000,
            version: 1,
        };
        let registered_id = dep_graph_register_dependency(&state, edge.clone()).unwrap();
        assert_eq!(registered_id, "edge-test-wrapper");

        let deps = dep_graph_get_dependents(&state, "calc-doc-1".to_string()).unwrap();
        assert_eq!(deps.len(), 1);

        let incoming = dep_graph_get_dependencies(&state, "word-doc-1".to_string()).unwrap();
        assert_eq!(incoming.len(), 1);

        let snapshot = render_core::LiveEmbedSnapshot {
            embed_id: "edge-test-wrapper".to_string(),
            source_doc_id: "calc-doc-1".to_string(),
            version: 1,
            rendered_html: Some("<table><tr><td>100</td></tr></table>".to_string()),
            rendered_svg: None,
            tabular_data: Some(vec![vec!["100".to_string()]]),
            scalar_value: None,
            last_updated_timestamp: 1000,
            is_stale: false,
        };
        dep_graph_update_snapshot(&state, snapshot).unwrap();

        let fetched_snap = dep_graph_get_snapshot(&state, "edge-test-wrapper".to_string())
            .unwrap()
            .unwrap();
        assert!(!fetched_snap.is_stale);

        let invalidated = dep_graph_propagate_invalidation(
            &state,
            "calc-doc-1".to_string(),
            Some(edge.source_anchor.clone()),
        )
        .unwrap();
        assert_eq!(invalidated, vec!["word-doc-1".to_string()]);

        let stale_snap = dep_graph_get_snapshot(&state, "edge-test-wrapper".to_string())
            .unwrap()
            .unwrap();
        assert!(stale_snap.is_stale);

        let summary = dep_graph_summary(&state).unwrap();
        assert_eq!(summary.total_edges, 1);
        assert_eq!(summary.stale_snapshots, 1);

        let removed = dep_graph_remove_dependency(&state, "edge-test-wrapper".to_string()).unwrap();
        assert!(removed);

        // Test wikilink command wrappers
        let parsed =
            wikilink_parse_links("Check [[Project Roadmap#Milestones|Roadmap]] now.".to_string())
                .unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].target_doc, "Project Roadmap");
        assert_eq!(parsed[0].target_anchor.as_deref(), Some("Milestones"));
        assert_eq!(parsed[0].display_text.as_deref(), Some("Roadmap"));

        wikilink_index_document(
            &state,
            "doc-notes-1".to_string(),
            "Meeting Notes".to_string(),
            "Refer to [[Project Roadmap]] for deadlines.".to_string(),
        )
        .unwrap();

        wikilink_index_document(
            &state,
            "doc-roadmap".to_string(),
            "Project Roadmap".to_string(),
            "Key milestones and deliverables.".to_string(),
        )
        .unwrap();

        let backlinks = wikilink_get_backlinks(&state, "Project Roadmap".to_string()).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_doc_id, "doc-notes-1");

        let fwd = wikilink_get_forward_links(&state, "doc-notes-1".to_string()).unwrap();
        assert_eq!(fwd.len(), 1);

        let suggestions = wikilink_suggest_targets(&state, "Roadmap".to_string(), Some(5)).unwrap();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].title, "Project Roadmap");

        let stats = wikilink_stats(&state).unwrap();
        assert_eq!(stats.total_documents, 2);
        assert_eq!(stats.total_wikilinks, 1);

        let unlinked =
            wikilink_find_unlinked_mentions(&state, "Meeting Notes".to_string()).unwrap();
        assert!(unlinked.is_empty());

        wikilink_remove_document(&state, "doc-notes-1".to_string()).unwrap();
        let stats_after = wikilink_stats(&state).unwrap();
        assert_eq!(stats_after.total_documents, 1);

        // Test conversion command wrappers
        let supported =
            conversion_supported_formats(render_core::ConversionFormat::Markdown).unwrap();
        assert!(supported.contains(&render_core::ConversionFormat::Html));
        assert!(supported.contains(&render_core::ConversionFormat::Docx));

        let estimate = conversion_estimate_job(4, 2048).unwrap();
        assert_eq!(estimate.task_count, 4);
        assert_eq!(estimate.total_bytes, 2048);
        assert!(estimate.estimated_duration_ms > 0);

        let conv_task = render_core::ConversionTask {
            id: "conv-wrapper-test-1".to_string(),
            source_format: render_core::ConversionFormat::Markdown,
            target_format: render_core::ConversionFormat::Html,
            input_bytes: b"# Hello World\nTesting conversion pipeline.".to_vec(),
            options: render_core::ConversionOptions::default(),
        };
        let single_res = conversion_convert_single(conv_task.clone()).unwrap();
        assert!(single_res.success);
        let out_str = String::from_utf8_lossy(&single_res.output_bytes);
        assert!(out_str.contains("Hello World"));

        let batch_report = conversion_batch_convert(vec![conv_task]).unwrap();
        assert_eq!(batch_report.total_tasks, 1);
        assert_eq!(batch_report.succeeded, 1);
        assert_eq!(batch_report.failed, 0);

        // Test local oplog command wrappers
        let op_entry = oplog_append_entry(
            &state,
            "doc-oplog-test".to_string(),
            "author-alice".to_string(),
            crdt_collab::OpPayload::TextInsert {
                parent_id: crdt_collab::OpId::ROOT,
                content: "Hello Local-First!".to_string(),
                right_origin: None,
            },
            1000,
            1,
        )
        .unwrap();
        assert_eq!(op_entry.author_id, "author-alice");
        assert_eq!(op_entry.doc_id, "doc-oplog-test");

        let history = oplog_get_history(&state, "doc-oplog-test".to_string(), None, None).unwrap();
        assert_eq!(history.len(), 1);

        let cp = oplog_create_checkpoint(
            &state,
            "doc-oplog-test".to_string(),
            "author-alice".to_string(),
            "Initial Draft".to_string(),
            1,
            1050,
        )
        .unwrap();
        assert_eq!(cp.label, "Initial Draft");

        let cps = oplog_get_checkpoints(&state, "doc-oplog-test".to_string()).unwrap();
        assert_eq!(cps.len(), 1);

        let traveled = oplog_time_travel(&state, "doc-oplog-test".to_string(), 1050).unwrap();
        assert!(traveled.text_content.contains("Hello Local-First!"));

        let sv = oplog_get_state_vector(&state, "doc-oplog-test".to_string()).unwrap();
        assert!(sv.get(1) > 0);

        let delta = oplog_compute_sync_delta(
            &state,
            "doc-oplog-test".to_string(),
            crdt_collab::StateVector::new(),
        )
        .unwrap();
        assert!(!delta.is_empty());

        let forked = oplog_fork_version(
            &state,
            "doc-oplog-test".to_string(),
            op_entry.entry_id,
            "doc-forked-1".to_string(),
            "author-bob".to_string(),
            2,
            1100,
        )
        .unwrap();
        assert_eq!(forked.doc_id, "doc-forked-1");
        assert!(forked.text_content.contains("Hello Local-First!"));

        // Test plugin runtime and calculation UDF command wrappers
        let wasm = plugin_runtime::WasmBinaryBuilder::new()
            .component(false)
            .build();
        let manifest_json = r#"{
            "id": "com.test.wrapper-plugin",
            "name": "Wrapper Test Plugin",
            "version": "1.0.0",
            "capabilities": ["document-read", "document-mutate"]
        }"#
        .to_string();

        let pid = plugin_load(&state, Some(manifest_json), wasm).unwrap();
        assert_eq!(pid, "com.test.wrapper-plugin");

        let plugins = plugin_list(&state).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "com.test.wrapper-plugin");
        assert_eq!(plugins[0].state, "loaded");

        plugin_init(&state, pid.clone()).unwrap();
        assert_eq!(plugin_list(&state).unwrap()[0].state, "active");

        plugin_suspend(&state, pid.clone()).unwrap();
        assert_eq!(plugin_list(&state).unwrap()[0].state, "suspended");

        plugin_resume(&state, pid.clone()).unwrap();
        assert_eq!(plugin_list(&state).unwrap()[0].state, "active");

        plugin_refill_fuel(&state, pid.clone()).unwrap();

        let action = plugin_runtime::PluginAction::InsertText {
            doc_id: "doc-1".to_string(),
            text: "From Plugin".to_string(),
        };
        let act_res = plugin_execute_action(&state, pid.clone(), action).unwrap();
        assert_eq!(act_res, plugin_runtime::ActionResult::MutationSuccess);

        // Register custom mock UDF
        plugin_register_mock_udf(
            &state,
            "DOUBLE_VAL".to_string(),
            "Doubles input number".to_string(),
            1,
            1,
            "double".to_string(),
        )
        .unwrap();

        let udfs = plugin_list_udfs(&state).unwrap();
        assert_eq!(udfs.len(), 1);
        assert_eq!(udfs[0].name, "DOUBLE_VAL");

        let call_res =
            plugin_call_udf(&state, "DOUBLE_VAL".to_string(), vec!["21".to_string()]).unwrap();
        assert!(call_res.success);
        assert_eq!(call_res.result_value, Some("42".to_string()));

        plugin_unload(&state, pid).unwrap();
        assert_eq!(plugin_list(&state).unwrap().len(), 0);

        // Test grounded copilot commands
        let status = copilot_get_provider_config(&state).unwrap();
        assert_eq!(status.provider_kind, "OfflineTemplate");

        let custom_cfg = copilot_core::ProviderConfig {
            provider_kind: copilot_core::CopilotProviderKind::LocalOllama,
            base_url: "http://localhost:11434".to_string(),
            model_name: "llama3.2".to_string(),
            ..Default::default()
        };
        let updated_status = copilot_configure_provider(&state, custom_cfg).unwrap();
        assert_eq!(updated_status.provider_kind, "LocalOllama");

        let indexed_chunks = copilot_index_document(
            &state,
            "doc-tauri-spec".to_string(),
            "Spec Doc".to_string(),
            "AetherOffice architecture ensures local performance and privacy.".to_string(),
        )
        .unwrap();
        assert!(indexed_chunks >= 1);

        let chat_req = copilot_core::CopilotChatRequest {
            query: "What ensures local performance?".to_string(),
            session_id: "doc-tauri-spec".to_string(),
            active_selection: None,
            target_doc_ids: None,
            conversation_history: vec![],
        };
        let chat_resp = copilot_chat(&state, chat_req).unwrap();
        assert!(!chat_resp.response_text.is_empty());
        assert!(!chat_resp.citations.is_empty());

        let removed = copilot_remove_document(&state, "doc-tauri-spec".to_string()).unwrap();
        assert!(removed >= 1);

        // Test theme and ribbon layout commands
        let tokens = theme_get_tokens(&state, None, None).unwrap();
        assert_eq!(tokens.mode, render_core::ThemeMode::Light);
        assert_eq!(tokens.ribbon_mode, render_core::RibbonLayoutMode::Classic);

        let dark_tokens = theme_set_mode(&state, "dark".to_string()).unwrap();
        assert_eq!(dark_tokens.mode, render_core::ThemeMode::Dark);
        assert_eq!(dark_tokens.bg_surface, "#1e293b");

        let r_compact = ribbon_set_layout_mode(&state, "compact".to_string()).unwrap();
        assert_eq!(r_compact.mode, render_core::RibbonLayoutMode::Compact);
        assert!(r_compact.is_single_row);
        assert_eq!(r_compact.height_px, 44.0);

        let r_cfg = ribbon_get_layout_config(&state).unwrap();
        assert_eq!(r_cfg.mode, render_core::RibbonLayoutMode::Compact);

        // Test accessibility and IME commands
        let tree = accessibility_get_aria_tree(&state).unwrap();
        assert_eq!(tree.root.role, render_core::AriaRole::Document);

        let custom_tree = render_core::AriaTree::new(render_core::AriaNode::new(
            "page-1",
            render_core::AriaRole::Document,
            "Page 1",
        ));
        let node_count = accessibility_set_aria_tree(&state, custom_tree).unwrap();
        assert_eq!(node_count, 1);

        let cbox = ime_start_composition(&state, 100.0, 200.0, 20.0).unwrap();
        assert_eq!(cbox.x, 100.0);
        assert_eq!(cbox.y, 220.0);

        let updated_cbox =
            ime_update_composition(&state, "nihon".to_string(), 5, 0, 5, 120.0, 200.0, 20.0)
                .unwrap();
        assert_eq!(updated_cbox.x, 120.0);

        let committed = ime_finish_composition(&state).unwrap();
        assert_eq!(committed, "nihon");

        let bidi = bidi_analyze_text("مرحبا World".to_string()).unwrap();
        assert_eq!(bidi.base_direction, render_core::BidiDirection::Rtl);
        assert!(!bidi.runs.is_empty());

        let next_pane = keyboard_nav_cycle_pane(&state, None).unwrap();
        assert_eq!(next_pane, render_core::NavFocusPane::StatusBar);

        // Test touch gesture and adaptive split commands
        let t1 = render_core::TouchPoint {
            id: 1,
            x: 100.0,
            y: 100.0,
            radius_x: 5.0,
            radius_y: 5.0,
            pressure: 1.0,
            is_stylus: false,
            timestamp_ms: 1000,
        };
        let g1 = gesture_process_touch_start(&state, t1).unwrap();
        assert_eq!(g1, render_core::TouchGesture::None);

        let t1_move = render_core::TouchPoint {
            id: 1,
            x: 120.0,
            y: 100.0,
            radius_x: 5.0,
            radius_y: 5.0,
            pressure: 1.0,
            is_stylus: false,
            timestamp_ms: 1020,
        };
        let g2 = gesture_process_touch_move(&state, t1_move).unwrap();
        match g2 {
            render_core::TouchGesture::PanScroll { delta_x, delta_y } => {
                assert!((delta_x - 20.0).abs() < 1e-4);
                assert!((delta_y - 0.0).abs() < 1e-4);
            }
            _ => panic!("Expected PanScroll"),
        }

        let g3 = gesture_process_touch_end(&state, 1, 1030).unwrap();
        assert_eq!(g3, render_core::TouchGesture::None);

        gesture_process_touch_cancel(&state, 1).unwrap();

        // Test palm rejection filter wrapper
        let palm_touch = render_core::TouchPoint {
            id: 99,
            x: 200.0,
            y: 200.0,
            radius_x: 25.0,
            radius_y: 20.0,
            pressure: 1.0,
            is_stylus: false,
            timestamp_ms: 1000,
        };
        let is_palm =
            palm_rejection_filter_contact(&state, palm_touch, None, 1920.0, 1080.0).unwrap();
        assert!(is_palm);

        // Test adaptive split layout wrappers
        let grid =
            adaptive_split_set_layout(&state, "two-way-horizontal".to_string(), Some(0.5), None)
                .unwrap();
        assert_eq!(grid.mode, render_core::SplitGridMode::TwoWayHorizontal);

        let fetched_rects = adaptive_split_get_rects(&state, 1200.0, 800.0, None, None).unwrap();
        assert_eq!(fetched_rects.len(), 2);
        assert_eq!(fetched_rects[0].width, 600.0);
        assert_eq!(fetched_rects[1].width, 600.0);

        // Test DLP commands
        let policy = dlp_get_policy(&state).unwrap();
        assert_eq!(policy.id, "ent-dlp-std");

        let dlp_res =
            dlp_evaluate_text(&state, "Employee SSN 123-45-6789 detected.".to_string()).unwrap();
        assert!(dlp_res.is_violation);
        assert_eq!(dlp_res.action, crypto_security::DlpAction::RedactAndAllow);

        let (redact_eval, redacted_txt) =
            dlp_evaluate_and_redact(&state, "Employee SSN 123-45-6789 detected.".to_string())
                .unwrap();
        assert!(redact_eval.is_violation);
        assert!(!redacted_txt.contains("123-45-6789"));

        let custom_policy = crypto_security::DlpPolicy::new("custom-p", "Custom", 2);
        dlp_set_policy(&state, custom_policy).unwrap();
        assert_eq!(dlp_get_policy(&state).unwrap().id, "custom-p");

        // Test sanitization certificate commands
        let key = b"cert-secret-key-123".to_vec();
        let cert = sanitization_generate_certificate(
            "doc-1".to_string(),
            b"raw text with ssn".to_vec(),
            b"raw text with [REDACTED]".to_vec(),
            "ent-dlp-std".to_string(),
            1000,
            vec![("SSN".to_string(), 1)],
            key.clone(),
        )
        .unwrap();
        assert_eq!(cert.total_redactions(), 1);

        let verified =
            sanitization_verify_certificate(cert, Some(b"raw text with [REDACTED]".to_vec()), key)
                .unwrap();
        assert!(verified);

        // Test audit log commands
        let entry = audit_log_append(
            &state,
            1000,
            crypto_security::AuditEventType::DocumentOpened,
            "alice".to_string(),
            "doc-1".to_string(),
            "open doc".to_string(),
        )
        .unwrap();
        assert_eq!(entry.sequence_number, 1);

        let entries = audit_log_get_entries(&state, Some("doc-1".to_string())).unwrap();
        assert_eq!(entries.len(), 1);

        let verified_count = audit_log_verify_integrity(&state).unwrap();
        assert_eq!(verified_count, 1);

        // Test entitlement commands
        let tier = entitlement_get_tier(&state).unwrap();
        assert_eq!(tier, "community");

        let basic_feat =
            entitlement_check_feature(&state, "basic-editing".to_string(), Some(1000)).unwrap();
        assert!(basic_feat.is_granted());

        let ent_feat =
            entitlement_check_feature(&state, "enterprise-dlp".to_string(), Some(1000)).unwrap();
        assert!(ent_feat.is_denied());

        let lic_key = b"lic-key-secret-999".to_vec();
        let lic_token = render_core::LicenseGenerator::create_signed_token(
            "LIC-WRAPPER-1",
            "Enterprise User",
            Some("Enterprise Corp"),
            render_core::EditionTier::Enterprise,
            1000,
            None,
            10,
            vec![],
            &lic_key,
        );
        let apply_res = entitlement_apply_license(&state, lic_token, lic_key, Some(1000)).unwrap();
        assert_eq!(apply_res, render_core::EntitlementStatus::Granted);

        let active_info = entitlement_get_license_info(&state).unwrap();
        assert!(active_info.is_some());
        assert_eq!(active_info.unwrap().license_id, "LIC-WRAPPER-1");

        let tier_after = entitlement_get_tier(&state).unwrap();
        assert_eq!(tier_after, "enterprise");

        let ent_feat_after =
            entitlement_check_feature(&state, "enterprise-dlp".to_string(), Some(1000)).unwrap();
        assert!(ent_feat_after.is_granted());

        let removed = entitlement_remove_license(&state).unwrap();
        assert!(removed);
        let tier_reverted = entitlement_get_tier(&state).unwrap();
        assert_eq!(tier_reverted, "community");

        let set_res = entitlement_set_tier(&state, "professional".to_string()).unwrap();
        assert_eq!(set_res, "professional");
        assert_eq!(entitlement_get_tier(&state).unwrap(), "professional");

        // Test SLA & Benchmark commands
        let report = sla_run_benchmarks(&state).unwrap();
        assert_eq!(report.metrics.len(), 5);
        assert!(report.all_passed);

        let verified = sla_verify_budgets(report.clone(), None).unwrap();
        assert!(verified);

        let fuzzed =
            fuzz_test_document_buffer(b"<root>hello</root>".to_vec(), "truncated-zip".to_string())
                .unwrap();
        assert!(fuzzed.len() < b"<root>hello</root>".len());

        let dl1 = render_core::DisplayList {
            bounds: render_core::primitive::Rect2D::new(0.0, 0.0, 800.0, 600.0),
            ..Default::default()
        };
        let dl2 = dl1.clone();
        let diff = differential_audit_evaluate(dl1, dl2).unwrap();
        assert!(diff.is_identical);

        // Test Release Matrix & SBOM commands
        let manifest = release_matrix_get_manifest(&state).unwrap();
        assert_eq!(manifest.artifacts.len(), 10);
        assert!(manifest.verify_coverage().unwrap());

        let cyclonedx = sbom_generate_cyclonedx(&state).unwrap();
        assert!(cyclonedx.contains("\"bomFormat\": \"CycloneDX\""));

        let spdx_json = sbom_generate_spdx(&state, Some("json".to_string())).unwrap();
        assert!(spdx_json.contains("\"spdxVersion\": \"SPDX-2.3\""));

        let spdx_tag = sbom_generate_spdx(&state, Some("tag-value".to_string())).unwrap();
        assert!(spdx_tag.starts_with("SPDXVersion: SPDX-2.3"));

        let license_audit = sbom_audit_licenses(&state).unwrap();
        assert!(license_audit.is_clean_room_compliant);
        assert!(!license_audit.has_gpl_contamination);

        let verified_payload =
            installer_verify_checksum(&state, "win-x64-msi".to_string(), Some(b"".to_vec()), None)
                .unwrap();
        assert!(verified_payload);

        let verified_hex = installer_verify_checksum(
            &state,
            "win-x64-msi".to_string(),
            None,
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
        )
        .unwrap();
        assert!(verified_hex);
    }
}
