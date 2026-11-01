//! Integration Test Suite for Sub-Phase 51.2.2: Multi-Document Grounded AI Copilot
//!
//! GEC-v7.0 Stage 51 (Sub-Phase 51.2.2)
//!
//! Verifies:
//! - Provider configuration and switching (Ollama, WebGPU, Cloud, Offline)
//! - Provider health probing and latency estimation
//! - Multi-document indexing (Word, Excel, Slides, Notes)
//! - Cross-document hybrid retrieval with grounded citations
//! - Active selection priority and budget-capped assembly
//! - Structured inline diff generation for document rewriting
//! - Targeted document ID filtering
//! - Document removal and index pruning
//! - Error handling for empty queries
//! - Confidence scoring and token usage tracking

use copilot_core::chunker::{DocumentKind, WordParagraphInput};
use copilot_core::context_assembler::ActiveSelectionContext;
use copilot_core::provider::{
    CopilotChatRequest, CopilotGroundedEngine, CopilotProviderKind, ProviderConfig,
};

#[test]
fn test_provider_config_default_and_customization() {
    let mut engine = CopilotGroundedEngine::default();
    assert_eq!(
        engine.config().provider_kind,
        CopilotProviderKind::OfflineTemplate
    );
    assert_eq!(engine.config().model_name, "llama3.2");

    let custom_config = ProviderConfig {
        provider_kind: CopilotProviderKind::LocalOllama,
        base_url: "http://127.0.0.1:11434".to_string(),
        model_name: "mistral:latest".to_string(),
        api_key: None,
        timeout_ms: 15000,
        temperature: 0.1,
        max_tokens: 4096,
    };

    engine.set_config(custom_config);
    assert_eq!(
        engine.config().provider_kind,
        CopilotProviderKind::LocalOllama
    );
    assert_eq!(engine.config().model_name, "mistral:latest");
    assert_eq!(engine.config().max_tokens, 4096);
}

#[test]
fn test_provider_probe_status_reporting() {
    let engine = CopilotGroundedEngine::new(ProviderConfig {
        provider_kind: CopilotProviderKind::LocalOllama,
        base_url: "http://localhost:11434".to_string(),
        model_name: "llama3.2".to_string(),
        ..Default::default()
    });

    let status = engine.probe_provider_status();
    assert_eq!(status.provider_kind, "LocalOllama");
    assert_eq!(status.model_name, "llama3.2");
    assert_eq!(status.endpoint, "http://localhost:11434");
    assert!(status.is_available);
    assert!(status.latency_ms > 0);
}

#[test]
fn test_multi_document_indexing_word_and_spreadsheet() {
    let mut engine = CopilotGroundedEngine::default();

    // 1. Index Word document
    let paragraphs = vec![
        WordParagraphInput {
            section: 0,
            paragraph_index: 0,
            heading_level: Some(1),
            text: "AetherOffice enterprise deployment architecture supports zero-trust boundaries and local execution.".to_string(),
        },
        WordParagraphInput {
            section: 0,
            paragraph_index: 1,
            heading_level: None,
            text: "All collaborative operations are recorded in an append-only operation log with SHA-256 hash chains.".to_string(),
        },
    ];
    let word_chunks = engine
        .index_word_paragraphs("doc-word-specs", "Architectural Whitepaper", &paragraphs)
        .expect("Index Word");
    assert!(word_chunks >= 1);

    // 2. Index Spreadsheet
    let headers = vec!["Quarter".to_string(), "Revenue".to_string()];
    let rows = vec![
        vec!["Q1".to_string(), "420000".to_string()],
        vec!["Q2".to_string(), "580000".to_string()],
    ];

    let sheet_chunks = engine
        .index_calc_sheet("doc-calc-finances", "Sheet1", &headers, &rows)
        .expect("Index Sheet");
    assert!(sheet_chunks >= 1);

    assert_eq!(engine.document_count(), 2);
}

#[test]
fn test_multi_document_cross_retrieval_and_citations() {
    let mut engine = CopilotGroundedEngine::default();

    // Index technical specs
    let _ = engine
        .index_text_document(
            "doc-specs",
            "Core Invariants",
            "The Rust core enforces zero panics and forbid unsafe code across all crates.",
        )
        .expect("Index specs");

    // Index financial sheet
    let headers = vec!["Budget".to_string(), "Status".to_string()];
    let rows = vec![vec!["Infrastructure".to_string(), "Approved".to_string()]];

    let _ = engine
        .index_calc_sheet("doc-finance", "Allocations", &headers, &rows)
        .expect("Index finance");

    let req = CopilotChatRequest {
        query: "What are the Rust safety invariants and budget status?".to_string(),
        session_id: "doc-specs".to_string(),
        active_selection: None,
        target_doc_ids: None, // Cross-document across all active documents
        conversation_history: vec![],
    };

    let resp = engine.chat(req).expect("Chat response");
    assert!(!resp.response_text.is_empty());
    assert!(!resp.citations.is_empty());
    assert!(resp.confidence_score > 0.7);

    // Verifies citations reference document IDs correctly
    let doc_ids: Vec<String> = resp.citations.iter().map(|c| c.doc_id.clone()).collect();
    assert!(doc_ids
        .iter()
        .any(|d| d == "doc-specs" || d == "doc-finance"));
}

#[test]
fn test_active_selection_context_prioritization() {
    let mut engine = CopilotGroundedEngine::default();

    let _ = engine
        .index_text_document(
            "doc-1",
            "Overview",
            "General background information regarding enterprise office productivity suites.",
        )
        .expect("Index doc");

    let selection = ActiveSelectionContext {
        kind: DocumentKind::Word,
        selected_text: Some(
            "CRITICAL CLAUSE: All customer data must be encrypted with AES-256-GCM at rest."
                .to_string(),
        ),
        selected_range: Some("Paragraph 2".to_string()),
        location: None,
    };

    let req = CopilotChatRequest {
        query: "Summarize the critical security requirements.".to_string(),
        session_id: "doc-1".to_string(),
        active_selection: Some(selection),
        target_doc_ids: None,
        conversation_history: vec![],
    };

    let resp = engine.chat(req).expect("Chat response");
    assert!(resp.tokens_used > 0);
    assert!(!resp.response_text.is_empty());
}

#[test]
fn test_grounded_diff_generation_on_rewrite_query() {
    let mut engine = CopilotGroundedEngine::default();

    let _ = engine
        .index_text_document(
            "doc-proposal",
            "Project Proposal",
            "Our team intends to deliver the project in Q3 if resources permit.",
        )
        .expect("Index doc");

    let selection = ActiveSelectionContext {
        kind: DocumentKind::Word,
        selected_text: Some(
            "Our team intends to deliver the project in Q3 if resources permit.".to_string(),
        ),
        selected_range: Some("Para 1".to_string()),
        location: None,
    };

    let req = CopilotChatRequest {
        query: "Rewrite this sentence into a formal executive commitment.".to_string(),
        session_id: "doc-proposal".to_string(),
        active_selection: Some(selection),
        target_doc_ids: None,
        conversation_history: vec![],
    };

    let resp = engine.chat(req).expect("Chat response");
    assert!(resp.suggested_diff.is_some());

    let diff = resp.suggested_diff.unwrap();
    assert_eq!(diff.doc_id, "doc-proposal");
    assert!(diff.original.contains("Our team intends"));
    assert!(diff.modified.contains("Executive Summary"));
    assert!(resp.response_text.contains("Apply Diff"));
}

#[test]
fn test_targeted_doc_id_filter_restriction() {
    let mut engine = CopilotGroundedEngine::default();

    let _ = engine
        .index_text_document(
            "doc-alpha",
            "Project Alpha",
            "Project Alpha targets autonomous robotics navigation.",
        )
        .expect("Index alpha");

    let _ = engine
        .index_text_document(
            "doc-beta",
            "Project Beta",
            "Project Beta develops quantum computing simulation algorithms.",
        )
        .expect("Index beta");

    // Query targeting only doc-beta
    let req = CopilotChatRequest {
        query: "What does this project develop?".to_string(),
        session_id: "doc-beta".to_string(),
        active_selection: None,
        target_doc_ids: Some(vec!["doc-beta".to_string()]),
        conversation_history: vec![],
    };

    let resp = engine.chat(req).expect("Chat response");
    for citation in &resp.citations {
        assert_eq!(citation.doc_id, "doc-beta");
    }
}

#[test]
fn test_document_removal_and_corpus_update() {
    let mut engine = CopilotGroundedEngine::default();

    let _ = engine
        .index_text_document(
            "temp-doc",
            "Temporary Document",
            "Ephemeral notes to be purged upon closing session.",
        )
        .expect("Index temp");

    assert_eq!(engine.document_count(), 1);

    let removed_count = engine.remove_document("temp-doc");
    assert!(removed_count > 0);
    assert_eq!(engine.document_count(), 0);
}

#[test]
fn test_empty_query_error_handling() {
    let mut engine = CopilotGroundedEngine::default();

    let req = CopilotChatRequest {
        query: "   ".to_string(),
        session_id: "doc-1".to_string(),
        active_selection: None,
        target_doc_ids: None,
        conversation_history: vec![],
    };

    let result = engine.chat(req);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        copilot_core::CopilotError::EmptyDocument
    );
}

#[test]
fn test_confidence_scoring_and_token_accounting() {
    let mut engine = CopilotGroundedEngine::default();

    let _ = engine
        .index_text_document(
            "doc-knowledge",
            "Knowledge Base",
            "AetherOffice is built entirely with Rust and WebAssembly, maintaining 12 domain crates.",
        )
        .expect("Index knowledge");

    let req = CopilotChatRequest {
        query: "How many domain crates are maintained?".to_string(),
        session_id: "doc-knowledge".to_string(),
        active_selection: None,
        target_doc_ids: None,
        conversation_history: vec![],
    };

    let resp = engine.chat(req).expect("Chat response");
    assert!(resp.confidence_score >= 0.85);
    assert!(resp.tokens_used > 0);
    assert_eq!(resp.model_used, "llama3.2");
}
