# Sub-Phase 51.1.1: Local-First Operation Log, Version Time-Travel & Conflict-Free Reconnection

## Focus
`crdt-collab`, `unified-shell`

## Goals
1. Implement pure safe-Rust Local-First Operation Log & Time-Travel Engine in `crdt-collab::op_log`:
   - Append-only `LocalOpLog` storing immutable, causally-linked operations (`OpLogEntry`):
     - `entry_id: OpId`, `parent_ids: Vec<OpId>`, `timestamp_ms: u64`, `author_id: String`, `doc_id: String`.
     - Polymorphic operation payloads:
       - `TextInsert { parent_id: OpId, content: String, right_origin: Option<OpId> }`
       - `TextDelete { target_id: OpId }`
       - `TextFormat { start: usize, len: usize, attribute: FormatAttribute }`
       - `CellUpdate { row: u32, col: u32, value: GridCellValue }`
       - `RowShift { after_row: u32, count: i32 }`
       - `ShapeMutation { shape_id: String, properties_json: String }`
       - `SnapshotCheckpoint { checkpoint_id: String, label: String, snapshot_data: String }`
     - SHA-256 cryptographic chain linking each entry with parent hashes to detect tampering or corruption.
   - Version Time-Travel & Historical Replay:
     - `time_travel_to_timestamp(doc_id, target_timestamp_ms)` & `time_travel_to_op(doc_id, target_op_id)`
     - Materializes historical text, cell states, and checkpoints without mutating the forward live log.
     - Checkpoint registry: named milestone checkpoints (e.g. "Draft 1", "Before Budget Revision").
     - Historical branch/fork: `fork_at_version(doc_id, target_op_id, new_doc_id)` creating a clean new document line from any historical state.
   - Conflict-Free Reconnection & Peer Sync Protocol:
     - State Vector tracking (`StateVector`) across all active and disconnected clients.
     - `compute_sync_delta(doc_id, remote_vector) -> Vec<OpLogEntry>` returning exactly the missing operations required by the remote peer.
     - `apply_remote_entries(doc_id, entries) -> SyncReport` merging concurrent edits deterministically using causal ancestry and Fugue Tree-RGA / LWW registers.
     - Compaction / garbage collection pruning acknowledged tombstoned operations older than the stable vector frontier.
2. Register high-speed Tauri IPC bridge in `apps/unified-shell/src-tauri`:
   - `oplog_append_entry`
   - `oplog_get_history`
   - `oplog_time_travel`
   - `oplog_create_checkpoint`
   - `oplog_get_checkpoints`
   - `oplog_compute_sync_delta`
   - `oplog_apply_remote_delta`
   - `oplog_get_state_vector`
   - `oplog_fork_version`
3. Build presentation shell coordinator:
   - `LocalOpLogCoordinator.ts`:
     - Tracks operation log status, offline/online synchronization state, and pending un-synced operations.
     - Time-travel scrubber state (timeline position, active version preview, step-by-step diffs).
     - Checkpoint creation, milestone labels, and one-click historical rollback / fork.
     - Conflict-free peer sync event emission and reactive subscriber notifications.
4. Comprehensive test coverage:
   - 10 Rust unit/integration tests in `crates/crdt-collab/tests/subphase_51_1_1_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/local_oplog_coordinator.test.ts`.
5. Pass `tools/verify-contract.ps1` with exit code 0.

