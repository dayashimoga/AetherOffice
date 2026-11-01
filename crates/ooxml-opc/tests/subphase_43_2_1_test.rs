//! Sub-Phase 43.2.1 Tests: Non-Blocking Async File Ingestion Pipeline
//!
//! Conforms strictly to GEC-v7.0 Article I, III, IV, and VI:
//! - Verification of cooperative cancellation via CancellationToken
//! - Validation of stage sequencing and percentage calculation
//! - Zero panics and complete unparsed store retention

use ooxml_opc::async_pipeline::{
    AsyncIngestionPipeline, CancellationToken, IngestionProgress, IngestionStage,
};
use ooxml_opc::fixtures::create_synthetic_docx;
use std::sync::{Arc, Mutex};

#[test]
fn test_01_async_ingestion_pipeline_progress_sequence() {
    let docx_bytes = create_synthetic_docx().unwrap();
    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&progress_events);

    let result =
        AsyncIngestionPipeline::ingest_bytes(&docx_bytes, None, move |p: IngestionProgress| {
            events_clone.lock().unwrap().push(p);
        })
        .expect("Ingestion succeeds");

    assert!(!result.parts.is_empty());
    assert!(result.total_bytes_decompressed > 0);

    let events = progress_events.lock().unwrap();
    assert!(events.len() >= 5);

    // Verify stage sequence
    assert_eq!(events[0].stage, IngestionStage::ReadingCentralDirectory);
    assert_eq!(events[1].stage, IngestionStage::ResolvingContentTypes);
    assert_eq!(events[2].stage, IngestionStage::ParsingRelationships);

    let has_extracting = events
        .iter()
        .any(|e| matches!(e.stage, IngestionStage::ExtractingParts));
    assert!(has_extracting);

    let has_building_dom = events
        .iter()
        .any(|e| matches!(e.stage, IngestionStage::BuildingDom));
    assert!(has_building_dom);

    let last_event = events.last().unwrap();
    assert_eq!(last_event.stage, IngestionStage::Complete);
    assert_eq!(last_event.percentage, 100.0);
}

#[test]
fn test_02_async_ingestion_cancellation_early() {
    let docx_bytes = create_synthetic_docx().unwrap();
    let token = CancellationToken::new();
    token.cancel(); // Pre-cancel

    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&progress_events);

    let result = AsyncIngestionPipeline::ingest_bytes(&docx_bytes, Some(&token), move |p| {
        events_clone.lock().unwrap().push(p);
    });

    assert!(result.is_err());
    let err_msg = format!("{}", result.err().unwrap());
    assert!(err_msg.contains("cancelled"));

    let events = progress_events.lock().unwrap();
    assert!(!events.is_empty());
    assert_eq!(events[0].stage, IngestionStage::Cancelled);
}

#[test]
fn test_03_async_ingestion_cancellation_mid_stream() {
    let docx_bytes = create_synthetic_docx().unwrap();
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&progress_events);

    let result = AsyncIngestionPipeline::ingest_bytes(&docx_bytes, Some(&token), move |p| {
        events_clone.lock().unwrap().push(p.clone());
        // Trigger cancellation upon entering ExtractingParts stage
        if matches!(p.stage, IngestionStage::ExtractingParts) {
            token_clone.cancel();
        }
    });

    assert!(result.is_err());
    let err_msg = format!("{}", result.err().unwrap());
    assert!(err_msg.contains("cancelled"));

    let events = progress_events.lock().unwrap();
    let has_cancelled_event = events
        .iter()
        .any(|e| matches!(e.stage, IngestionStage::Cancelled));
    assert!(has_cancelled_event);
}

#[test]
fn test_04_async_ingestion_unparsed_store_retention() {
    let docx_bytes = create_synthetic_docx().unwrap();

    let result = AsyncIngestionPipeline::ingest_bytes(&docx_bytes, None, |_| {})
        .expect("Ingestion succeeds");

    assert!(!result.parts.is_empty());
    assert!(!result.unparsed_store.is_empty());
}
