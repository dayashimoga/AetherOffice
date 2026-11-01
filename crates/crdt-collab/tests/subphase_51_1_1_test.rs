//! Sub-Phase 51.1.1 Integration Tests: Local-First Operation Log, Time-Travel & Sync
//!
//! Complies strictly with GEC-v7.0 Article I (Architectural Integrity & Zero Global Document Singletons),
//! Article III (Real Binary File Open & Save, Zero Data Loss), and Article IV (Code Safety & Zero Panic):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in production library code.
//! - Append-only operation logging with SHA-256 tamper-evident parent hashing.
//! - Historical version time-travel to timestamps and exact OpIds.
//! - Milestone checkpoints and document branch forking.
//! - State vector delta exchange and deterministic concurrent merge.

use crdt_collab::fugue::StateVector;
use crdt_collab::grid_crdt::GridCellValue;
use crdt_collab::id::OpId;
use crdt_collab::op_log::{LocalOpLog, OpPayload};

#[test]
fn test_append_operation_and_sha256_hash_chain() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-chain-test";

    let op1 = oplog
        .append(
            doc_id,
            "author-1",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "First entry".to_string(),
                right_origin: None,
            },
            1000,
            1,
        )
        .expect("Append op1");

    assert_eq!(op1.entry_id.seq, 1);
    assert_eq!(op1.entry_id.client_id, 1);
    assert_eq!(op1.parent_ids, vec![OpId::ROOT]);
    assert!(!op1.hash.is_empty());

    let op2 = oplog
        .append(
            doc_id,
            "author-1",
            OpPayload::TextInsert {
                parent_id: op1.entry_id,
                content: " Second entry".to_string(),
                right_origin: None,
            },
            1050,
            1,
        )
        .expect("Append op2");

    assert_eq!(op2.entry_id.seq, 2);
    assert_eq!(op2.parent_ids, vec![op1.entry_id]);
    assert_ne!(op1.hash, op2.hash);

    let entries = oplog.get_entries(doc_id);
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_text_insert_and_delete_materialization() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-text-test";

    let ins1 = oplog
        .append(
            doc_id,
            "author-alice",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Hello World".to_string(),
                right_origin: None,
            },
            1000,
            1,
        )
        .expect("Insert text");

    let state1 = oplog.time_travel_to_timestamp(doc_id, 1000);
    assert_eq!(state1.text_content, "Hello World");

    // Delete character 'H' (target_id is first char op)
    let _del = oplog
        .append(
            doc_id,
            "author-alice",
            OpPayload::TextDelete {
                target_id: ins1.entry_id,
            },
            1100,
            1,
        )
        .expect("Delete char");

    let state2 = oplog.time_travel_to_timestamp(doc_id, 1100);
    assert_eq!(state2.text_content, "ello World");

    // Historical inspection: timestamp 1000 still yields "Hello World"
    let historical = oplog.time_travel_to_timestamp(doc_id, 1000);
    assert_eq!(historical.text_content, "Hello World");
}

#[test]
fn test_cell_update_and_row_shift_materialization() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-grid-test";

    oplog
        .append(
            doc_id,
            "calc-user",
            OpPayload::CellUpdate {
                row: 0,
                col: 0,
                value: GridCellValue::Text("Header".to_string()),
            },
            1000,
            1,
        )
        .expect("Cell 0,0");

    oplog
        .append(
            doc_id,
            "calc-user",
            OpPayload::CellUpdate {
                row: 1,
                col: 0,
                value: GridCellValue::Number(42.0),
            },
            1010,
            1,
        )
        .expect("Cell 1,0");

    let state1 = oplog.time_travel_to_timestamp(doc_id, 1010);
    assert_eq!(state1.cells.len(), 2);
    assert_eq!(
        state1.cells.get(&(1, 0)),
        Some(&GridCellValue::Number(42.0))
    );

    // Shift rows: insert 2 rows after row 0
    oplog
        .append(
            doc_id,
            "calc-user",
            OpPayload::RowShift {
                after_row: 0,
                count: 2,
            },
            1020,
            1,
        )
        .expect("RowShift");

    let state2 = oplog.time_travel_to_timestamp(doc_id, 1020);
    // Row 1 shifted to Row 3 (1 + 2 = 3)
    assert_eq!(
        state2.cells.get(&(3, 0)),
        Some(&GridCellValue::Number(42.0))
    );
    assert_eq!(
        state2.cells.get(&(0, 0)),
        Some(&GridCellValue::Text("Header".to_string()))
    );
}

#[test]
fn test_shape_mutation_materialization() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-shapes-test";

    oplog
        .append(
            doc_id,
            "slide-author",
            OpPayload::ShapeMutation {
                shape_id: "rect-1".to_string(),
                properties_json: r##"{"x": 100, "y": 200, "fill": "#ff0000"}"##.to_string(),
            },
            1000,
            1,
        )
        .expect("Shape 1");

    oplog
        .append(
            doc_id,
            "slide-author",
            OpPayload::ShapeMutation {
                shape_id: "rect-1".to_string(),
                properties_json: r##"{"x": 150, "y": 250, "fill": "#00ff00"}"##.to_string(),
            },
            1050,
            1,
        )
        .expect("Shape 1 update");

    let state_early = oplog.time_travel_to_timestamp(doc_id, 1000);
    assert!(state_early.shapes.get("rect-1").unwrap().contains("100"));

    let state_late = oplog.time_travel_to_timestamp(doc_id, 1050);
    assert!(state_late.shapes.get("rect-1").unwrap().contains("150"));
}

#[test]
fn test_snapshot_milestone_checkpoints() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-cp-test";

    oplog
        .append(
            doc_id,
            "editor-1",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "First Draft Content".to_string(),
                right_origin: None,
            },
            1000,
            1,
        )
        .expect("Insert");

    let cp = oplog
        .create_checkpoint(doc_id, "editor-1", "Version 1.0 Final", 1, 1050)
        .expect("Create checkpoint");

    assert_eq!(cp.label, "Version 1.0 Final");
    assert_eq!(cp.doc_id, doc_id);

    let checkpoints = oplog.get_checkpoints(doc_id);
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].checkpoint_id, cp.checkpoint_id);

    let fetched = oplog
        .get_checkpoint_by_id(doc_id, &cp.checkpoint_id)
        .expect("Fetched CP");
    assert_eq!(fetched.label, "Version 1.0 Final");
}

#[test]
fn test_version_time_travel_exact_timestamps() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-timetravel-test";

    oplog
        .append(
            doc_id,
            "u1",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Alpha".to_string(),
                right_origin: None,
            },
            100,
            1,
        )
        .expect("t=100");

    oplog
        .append(
            doc_id,
            "u1",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: " Beta".to_string(),
                right_origin: None,
            },
            200,
            1,
        )
        .expect("t=200");

    oplog
        .append(
            doc_id,
            "u1",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: " Gamma".to_string(),
                right_origin: None,
            },
            300,
            1,
        )
        .expect("t=300");

    let state_150 = oplog.time_travel_to_timestamp(doc_id, 150);
    assert_eq!(state_150.text_content, "Alpha");
    assert_eq!(state_150.total_ops_replayed, 1);

    let state_250 = oplog.time_travel_to_timestamp(doc_id, 250);
    assert_eq!(state_250.text_content, "Alpha Beta");
    assert_eq!(state_250.total_ops_replayed, 2);

    let state_350 = oplog.time_travel_to_timestamp(doc_id, 350);
    assert_eq!(state_350.text_content, "Alpha Beta Gamma");
    assert_eq!(state_350.total_ops_replayed, 3);
}

#[test]
fn test_version_time_travel_by_op_id() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-op-travel-test";

    let op1 = oplog
        .append(
            doc_id,
            "user",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Step One".to_string(),
                right_origin: None,
            },
            100,
            1,
        )
        .expect("op1");

    let op2 = oplog
        .append(
            doc_id,
            "user",
            OpPayload::TextInsert {
                parent_id: op1.entry_id,
                content: " Step Two".to_string(),
                right_origin: None,
            },
            200,
            1,
        )
        .expect("op2");

    let state_at_op1 = oplog
        .time_travel_to_op(doc_id, op1.entry_id)
        .expect("Replay to op1");
    assert_eq!(state_at_op1.text_content, "Step One");
    assert_eq!(state_at_op1.last_op_id, Some(op1.entry_id));

    let state_at_op2 = oplog
        .time_travel_to_op(doc_id, op2.entry_id)
        .expect("Replay to op2");
    assert_eq!(state_at_op2.text_content, "Step One Step Two");
    assert_eq!(state_at_op2.last_op_id, Some(op2.entry_id));
}

#[test]
fn test_conflict_free_peer_sync_and_deltas() {
    let mut peer_a = LocalOpLog::new();
    let mut peer_b = LocalOpLog::new();
    let doc_id = "doc-collab-sync";

    // Both start with shared baseline
    let base_op = peer_a
        .append(
            doc_id,
            "alice",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Shared Baseline".to_string(),
                right_origin: None,
            },
            100,
            1,
        )
        .expect("base");

    peer_b
        .append_entry(base_op.clone())
        .expect("peer_b ingest base");

    // Peer A makes offline edits
    let _a_edit = peer_a
        .append(
            doc_id,
            "alice",
            OpPayload::TextInsert {
                parent_id: base_op.entry_id,
                content: " (Alice Edit)".to_string(),
                right_origin: None,
            },
            200,
            1,
        )
        .expect("a edit");

    // Peer B makes concurrent offline edits
    let _b_edit = peer_b
        .append(
            doc_id,
            "bob",
            OpPayload::TextInsert {
                parent_id: base_op.entry_id,
                content: " (Bob Edit)".to_string(),
                right_origin: None,
            },
            200,
            2,
        )
        .expect("b edit");

    // Reconnection & Sync:
    // 1. Peer A queries delta for Peer B
    let sv_b = peer_b.get_state_vector(doc_id);
    let delta_for_b = peer_a.compute_sync_delta(doc_id, &sv_b);
    assert_eq!(delta_for_b.len(), 1);

    // 2. Peer B queries delta for Peer A
    let sv_a = peer_a.get_state_vector(doc_id);
    let delta_for_a = peer_b.compute_sync_delta(doc_id, &sv_a);
    assert_eq!(delta_for_a.len(), 1);

    // 3. Apply deltas in both directions
    let report_b = peer_b
        .apply_remote_delta(doc_id, delta_for_b)
        .expect("Apply to B");
    assert_eq!(report_b.applied, 1);

    let report_a = peer_a
        .apply_remote_delta(doc_id, delta_for_a)
        .expect("Apply to A");
    assert_eq!(report_a.applied, 1);

    // Verify state vectors match
    assert_eq!(
        peer_a.get_state_vector(doc_id),
        peer_b.get_state_vector(doc_id)
    );

    // Verify both peers converged to identical text content
    let final_a = peer_a.time_travel_to_timestamp(doc_id, 300);
    let final_b = peer_b.time_travel_to_timestamp(doc_id, 300);
    assert_eq!(final_a.text_content, final_b.text_content);
    assert!(final_a.text_content.contains("Shared Baseline"));
    assert!(final_a.text_content.contains("Alice Edit"));
    assert!(final_a.text_content.contains("Bob Edit"));
}

#[test]
fn test_fork_document_from_history() {
    let mut oplog = LocalOpLog::new();
    let original_doc = "doc-original-roadmap";

    let op1 = oplog
        .append(
            original_doc,
            "architect",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Initial Architecture Plan".to_string(),
                right_origin: None,
            },
            100,
            1,
        )
        .expect("op1");

    let _op2 = oplog
        .append(
            original_doc,
            "architect",
            OpPayload::TextInsert {
                parent_id: op1.entry_id,
                content: " with Deprecated Subsystems".to_string(),
                right_origin: None,
            },
            200,
            1,
        )
        .expect("op2");

    // Fork at version 1 (before deprecated subsystems were added)
    let forked_log = oplog
        .fork_at_version(
            original_doc,
            op1.entry_id,
            "doc-forked-v2",
            "engineer",
            2,
            250,
        )
        .expect("Fork");

    let forked_state = forked_log.time_travel_to_timestamp("doc-forked-v2", 300);
    assert_eq!(forked_state.text_content, "Initial Architecture Plan");
    assert!(!forked_state.text_content.contains("Deprecated"));

    let forked_cps = forked_log.get_checkpoints("doc-forked-v2");
    assert_eq!(forked_cps.len(), 1);
    assert!(forked_cps[0]
        .label
        .contains("Forked from doc-original-roadmap"));
}

#[test]
fn test_log_compaction_with_checkpoints_preserved() {
    let mut oplog = LocalOpLog::new();
    let doc_id = "doc-compact-test";

    let _op1 = oplog
        .append(
            doc_id,
            "user",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: "Part 1".to_string(),
                right_origin: None,
            },
            100,
            1,
        )
        .expect("op1");

    let _cp = oplog
        .create_checkpoint(doc_id, "user", "Checkpoint Milestone", 1, 150)
        .expect("cp");

    let _op2 = oplog
        .append(
            doc_id,
            "user",
            OpPayload::TextInsert {
                parent_id: OpId::ROOT,
                content: " Part 2".to_string(),
                right_origin: None,
            },
            200,
            1,
        )
        .expect("op2");

    // Acknowledge up to seq 2
    let mut ack_vector = StateVector::new();
    ack_vector.set(1, 2);

    let pruned = oplog.compact_log(doc_id, &ack_vector);
    assert!(pruned >= 1);

    // Checkpoints remain intact
    let cps = oplog.get_checkpoints(doc_id);
    assert_eq!(cps.len(), 1);
    assert_eq!(cps[0].label, "Checkpoint Milestone");
}
