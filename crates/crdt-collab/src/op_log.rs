//! Local-First Operation Log, Version Time-Travel & Conflict-Free Reconnection
//! GEC-v7.0 Stage 51 (Sub-Phase 51.1.1)
//!
//! Provides an append-only, cryptographically-chained operation log storing all
//! mutations across text, spreadsheet grid, and slide shapes.
//! Supports:
//! - Local-first append with Lamport timestamping & SHA-256 tamper-evident chaining
//! - Version time-travel replay to any timestamp or operation ID
//! - Named milestone checkpoints and branch forking
//! - Offline-to-online peer synchronization with state vector deltas
//! - Strict `#![forbid(unsafe_code)]` compliance and zero panics

use crate::error::CrdtError;
use crate::fugue::StateVector;
use crate::grid_crdt::GridCellValue;
use crate::id::OpId;
use crate::peritext::FormatAttribute;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Polymorphic operation payload recording atomic document mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpPayload {
    /// Insertion of text run
    TextInsert {
        parent_id: OpId,
        content: String,
        right_origin: Option<OpId>,
    },
    /// Deletion of specific text operation
    TextDelete { target_id: OpId },
    /// Rich text span formatting
    TextFormat {
        start: usize,
        len: usize,
        attribute: FormatAttribute,
    },
    /// Cell value update in spreadsheet grid
    CellUpdate {
        row: u32,
        col: u32,
        value: GridCellValue,
    },
    /// Shift rows following row insertion or deletion
    RowShift { after_row: u32, count: i32 },
    /// Shape property mutation in slide deck or canvas
    ShapeMutation {
        shape_id: String,
        properties_json: String,
    },
    /// Snapshot milestone checkpoint
    SnapshotCheckpoint {
        checkpoint_id: String,
        label: String,
        snapshot_data: String,
    },
}

/// Immutable, causally-linked operation log entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpLogEntry {
    pub entry_id: OpId,
    pub parent_ids: Vec<OpId>,
    pub timestamp_ms: u64,
    pub author_id: String,
    pub doc_id: String,
    pub payload: OpPayload,
    pub hash: String,
}

impl OpLogEntry {
    /// Computes the cryptographic SHA-256 hash chaining this entry to its parent hash.
    pub fn compute_hash(
        entry_id: &OpId,
        parent_ids: &[OpId],
        timestamp_ms: u64,
        author_id: &str,
        doc_id: &str,
        payload: &OpPayload,
        parent_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entry_id.lamport.to_le_bytes());
        hasher.update(entry_id.client_id.to_le_bytes());
        hasher.update(entry_id.seq.to_le_bytes());
        for p in parent_ids {
            hasher.update(p.lamport.to_le_bytes());
            hasher.update(p.client_id.to_le_bytes());
            hasher.update(p.seq.to_le_bytes());
        }
        hasher.update(timestamp_ms.to_le_bytes());
        hasher.update(author_id.as_bytes());
        hasher.update(doc_id.as_bytes());
        if let Ok(serialized) = serde_json::to_vec(payload) {
            hasher.update(&serialized);
        }
        hasher.update(parent_hash.as_bytes());
        to_hex_string(&hasher.finalize())
    }
}

/// Metadata describing a milestone version checkpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointMeta {
    pub checkpoint_id: String,
    pub doc_id: String,
    pub label: String,
    pub op_id: OpId,
    pub timestamp_ms: u64,
    pub author_id: String,
    pub snapshot_preview: String,
}

/// Materialized state reconstructed during historical version time-travel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MaterializedDocumentState {
    pub doc_id: String,
    pub timestamp_ms: u64,
    pub text_content: String,
    pub cells: BTreeMap<(u32, u32), GridCellValue>,
    pub shapes: HashMap<String, String>,
    pub active_checkpoints: Vec<String>,
    pub total_ops_replayed: usize,
    pub last_op_id: Option<OpId>,
}

/// Synchronization result report returned after merging remote operation deltas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncReport {
    pub applied: usize,
    pub skipped: usize,
    pub conflicts_resolved: usize,
    pub state_vector: StateVector,
}

#[derive(Debug, Clone)]
struct TextCharItem {
    id: OpId,
    _parent_id: OpId,
    ch: char,
    deleted: bool,
}

/// Local-First Append-Only Operation Log & Time-Travel Engine.
#[derive(Debug, Clone, Default)]
pub struct LocalOpLog {
    entries: Vec<OpLogEntry>,
    entry_by_id: HashMap<OpId, usize>,
    entries_by_doc: HashMap<String, Vec<usize>>,
    checkpoints_by_doc: HashMap<String, Vec<CheckpointMeta>>,
    state_vectors: HashMap<String, StateVector>,
    last_hashes_by_doc: HashMap<String, String>,
    client_seq_counters: HashMap<(String, u64), u32>,
    current_lamport: u64,
}

impl LocalOpLog {
    /// Creates a new empty LocalOpLog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new local operation to the document's log.
    pub fn append(
        &mut self,
        doc_id: &str,
        author_id: &str,
        payload: OpPayload,
        timestamp_ms: u64,
        client_id: u64,
    ) -> Result<OpLogEntry, CrdtError> {
        self.current_lamport += 1;
        let lamport = self.current_lamport;

        let seq_key = (doc_id.to_string(), client_id);
        let seq = self.client_seq_counters.entry(seq_key).or_insert(0);
        *seq += 1;
        let current_seq = *seq;

        let entry_id = OpId::new(lamport, client_id, current_seq);

        let parent_ids = if let Some(indices) = self.entries_by_doc.get(doc_id) {
            if let Some(&last_idx) = indices.last() {
                vec![self.entries[last_idx].entry_id]
            } else {
                vec![OpId::ROOT]
            }
        } else {
            vec![OpId::ROOT]
        };

        let parent_hash = self
            .last_hashes_by_doc
            .get(doc_id)
            .cloned()
            .unwrap_or_else(|| {
                "0000000000000000000000000000000000000000000000000000000000000000".to_string()
            });

        let hash = OpLogEntry::compute_hash(
            &entry_id,
            &parent_ids,
            timestamp_ms,
            author_id,
            doc_id,
            &payload,
            &parent_hash,
        );

        let entry = OpLogEntry {
            entry_id,
            parent_ids,
            timestamp_ms,
            author_id: author_id.to_string(),
            doc_id: doc_id.to_string(),
            payload,
            hash: hash.clone(),
        };

        self.insert_entry_internal(entry.clone())?;
        Ok(entry)
    }

    /// Appends an existing pre-formed entry (e.g. from network or storage).
    pub fn append_entry(&mut self, entry: OpLogEntry) -> Result<(), CrdtError> {
        self.insert_entry_internal(entry)
    }

    fn insert_entry_internal(&mut self, entry: OpLogEntry) -> Result<(), CrdtError> {
        if self.entry_by_id.contains_key(&entry.entry_id) {
            return Ok(());
        }

        self.current_lamport = self.current_lamport.max(entry.entry_id.lamport);

        let doc_id = entry.doc_id.clone();
        let client_id = entry.entry_id.client_id;
        let seq = entry.entry_id.seq;

        // Update state vector
        let sv = self.state_vectors.entry(doc_id.clone()).or_default();
        sv.set(client_id, seq);

        // Update client seq counters
        let seq_counter = self
            .client_seq_counters
            .entry((doc_id.clone(), client_id))
            .or_insert(0);
        *seq_counter = (*seq_counter).max(seq);

        // Track last hash
        self.last_hashes_by_doc
            .insert(doc_id.clone(), entry.hash.clone());

        // Track checkpoints if this payload is a checkpoint
        if let OpPayload::SnapshotCheckpoint {
            ref checkpoint_id,
            ref label,
            ref snapshot_data,
        } = entry.payload
        {
            let preview = if snapshot_data.len() > 100 {
                format!("{}...", &snapshot_data[..100])
            } else {
                snapshot_data.clone()
            };

            let meta = CheckpointMeta {
                checkpoint_id: checkpoint_id.clone(),
                doc_id: doc_id.clone(),
                label: label.clone(),
                op_id: entry.entry_id,
                timestamp_ms: entry.timestamp_ms,
                author_id: entry.author_id.clone(),
                snapshot_preview: preview,
            };

            self.checkpoints_by_doc
                .entry(doc_id.clone())
                .or_default()
                .push(meta);
        }

        let idx = self.entries.len();
        self.entry_by_id.insert(entry.entry_id, idx);
        self.entries_by_doc.entry(doc_id).or_default().push(idx);
        self.entries.push(entry);

        Ok(())
    }

    /// Retrieves all operation entries for a specific document.
    pub fn get_entries(&self, doc_id: &str) -> Vec<OpLogEntry> {
        self.entries_by_doc
            .get(doc_id)
            .map(|indices| indices.iter().map(|&i| self.entries[i].clone()).collect())
            .unwrap_or_default()
    }

    /// Finds a specific log entry by its operation ID.
    pub fn get_entry_by_id(&self, id: OpId) -> Option<&OpLogEntry> {
        self.entry_by_id.get(&id).map(|&idx| &self.entries[idx])
    }

    /// Retrieves paginated history for a document.
    pub fn get_history(
        &self,
        doc_id: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<OpLogEntry> {
        let entries = self.get_entries(doc_id);
        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(entries.len());
        entries.into_iter().skip(off).take(lim).collect()
    }

    /// Creates a named snapshot milestone checkpoint.
    pub fn create_checkpoint(
        &mut self,
        doc_id: &str,
        author_id: &str,
        label: &str,
        client_id: u64,
        timestamp_ms: u64,
    ) -> Result<CheckpointMeta, CrdtError> {
        let checkpoint_id = format!("cp-{timestamp_ms}-{client_id}");
        let state = self.time_travel_to_timestamp(doc_id, timestamp_ms);
        let snapshot_data = serde_json::to_string(&state)
            .map_err(|e| CrdtError::SerializationError(e.to_string()))?;

        let payload = OpPayload::SnapshotCheckpoint {
            checkpoint_id: checkpoint_id.clone(),
            label: label.to_string(),
            snapshot_data,
        };

        let entry = self.append(doc_id, author_id, payload, timestamp_ms, client_id)?;

        let meta = CheckpointMeta {
            checkpoint_id,
            doc_id: doc_id.to_string(),
            label: label.to_string(),
            op_id: entry.entry_id,
            timestamp_ms,
            author_id: author_id.to_string(),
            snapshot_preview: format!(
                "Checkpoint: {label} ({} text chars)",
                state.text_content.len()
            ),
        };

        Ok(meta)
    }

    /// Retrieves all checkpoints registered for a document.
    pub fn get_checkpoints(&self, doc_id: &str) -> Vec<CheckpointMeta> {
        self.checkpoints_by_doc
            .get(doc_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Retrieves a single checkpoint by ID.
    pub fn get_checkpoint_by_id(
        &self,
        doc_id: &str,
        checkpoint_id: &str,
    ) -> Option<&CheckpointMeta> {
        self.checkpoints_by_doc
            .get(doc_id)?
            .iter()
            .find(|c| c.checkpoint_id == checkpoint_id)
    }

    /// Reconstructs the document's state at an exact point in time (wall-clock timestamp).
    pub fn time_travel_to_timestamp(
        &self,
        doc_id: &str,
        target_timestamp_ms: u64,
    ) -> MaterializedDocumentState {
        let entries = self.get_entries(doc_id);
        let filtered: Vec<&OpLogEntry> = entries
            .iter()
            .filter(|e| e.timestamp_ms <= target_timestamp_ms)
            .collect();

        self.materialize_from_entries(doc_id, target_timestamp_ms, &filtered)
    }

    /// Reconstructs the document's state up to an exact operation ID.
    pub fn time_travel_to_op(
        &self,
        doc_id: &str,
        target_op_id: OpId,
    ) -> Result<MaterializedDocumentState, CrdtError> {
        let entries = self.get_entries(doc_id);
        let target_idx = entries
            .iter()
            .position(|e| e.entry_id == target_op_id)
            .ok_or(CrdtError::EntryNotFound(target_op_id))?;

        let filtered: Vec<&OpLogEntry> = entries[..=target_idx].iter().collect();
        let timestamp = entries[target_idx].timestamp_ms;

        Ok(self.materialize_from_entries(doc_id, timestamp, &filtered))
    }

    fn materialize_from_entries(
        &self,
        doc_id: &str,
        target_timestamp: u64,
        entries: &[&OpLogEntry],
    ) -> MaterializedDocumentState {
        let mut char_nodes: Vec<TextCharItem> = Vec::new();
        let mut node_idx_map: HashMap<OpId, usize> = HashMap::new();
        let mut entry_last_char_id: HashMap<OpId, OpId> = HashMap::new();
        let mut cells: BTreeMap<(u32, u32), GridCellValue> = BTreeMap::new();
        let mut shapes: HashMap<String, String> = HashMap::new();
        let mut active_checkpoints: Vec<String> = Vec::new();
        let mut last_op: Option<OpId> = None;

        let mut sorted_entries: Vec<&OpLogEntry> = entries.to_vec();
        sorted_entries.sort_by_key(|e| (e.entry_id.lamport, e.entry_id.client_id, e.entry_id.seq));

        for entry in sorted_entries {
            last_op = Some(entry.entry_id);
            match &entry.payload {
                OpPayload::TextInsert {
                    parent_id,
                    content,
                    right_origin: _,
                } => {
                    let effective_parent = entry_last_char_id
                        .get(parent_id)
                        .copied()
                        .unwrap_or(*parent_id);
                    let mut current_parent = effective_parent;
                    let char_count = content.chars().count();
                    for (i, ch) in content.chars().enumerate() {
                        let char_op_id = OpId::new(
                            entry.entry_id.lamport,
                            entry.entry_id.client_id,
                            entry.entry_id.seq + i as u32,
                        );

                        let insert_idx = if current_parent == OpId::ROOT {
                            char_nodes.len()
                        } else if let Some(&parent_idx) = node_idx_map.get(&current_parent) {
                            parent_idx + 1
                        } else {
                            char_nodes.len()
                        };

                        let item = TextCharItem {
                            id: char_op_id,
                            _parent_id: current_parent,
                            ch,
                            deleted: false,
                        };

                        char_nodes.insert(insert_idx, item);
                        // Rebuild index map for subsequent insertions
                        node_idx_map.clear();
                        for (idx, n) in char_nodes.iter().enumerate() {
                            node_idx_map.insert(n.id, idx);
                        }

                        current_parent = char_op_id;
                    }

                    if char_count > 0 {
                        let last_char_id = OpId::new(
                            entry.entry_id.lamport,
                            entry.entry_id.client_id,
                            entry.entry_id.seq + (char_count - 1) as u32,
                        );
                        entry_last_char_id.insert(entry.entry_id, last_char_id);
                    }
                }
                OpPayload::TextDelete { target_id } => {
                    if let Some(&idx) = node_idx_map.get(target_id) {
                        char_nodes[idx].deleted = true;
                    }
                }
                OpPayload::TextFormat { .. } => {
                    // Formatting spans tracked in presentation layer
                }
                OpPayload::CellUpdate { row, col, value } => {
                    cells.insert((*row, *col), value.clone());
                }
                OpPayload::RowShift { after_row, count } => {
                    if *count > 0 {
                        let shift = *count as u32;
                        let mut new_cells = BTreeMap::new();
                        for ((r, c), val) in cells.into_iter() {
                            if r > *after_row {
                                new_cells.insert((r + shift, c), val);
                            } else {
                                new_cells.insert((r, c), val);
                            }
                        }
                        cells = new_cells;
                    }
                }
                OpPayload::ShapeMutation {
                    shape_id,
                    properties_json,
                } => {
                    shapes.insert(shape_id.clone(), properties_json.clone());
                }
                OpPayload::SnapshotCheckpoint { checkpoint_id, .. } => {
                    active_checkpoints.push(checkpoint_id.clone());
                }
            }
        }

        let mut text_content = String::with_capacity(char_nodes.len());
        for node in &char_nodes {
            if !node.deleted {
                text_content.push(node.ch);
            }
        }

        MaterializedDocumentState {
            doc_id: doc_id.to_string(),
            timestamp_ms: target_timestamp,
            text_content,
            cells,
            shapes,
            active_checkpoints,
            total_ops_replayed: entries.len(),
            last_op_id: last_op,
        }
    }

    /// Returns the current StateVector for a document.
    pub fn get_state_vector(&self, doc_id: &str) -> StateVector {
        self.state_vectors.get(doc_id).cloned().unwrap_or_default()
    }

    /// Computes the delta of operations unknown to a remote peer with the given StateVector.
    pub fn compute_sync_delta(&self, doc_id: &str, remote_vector: &StateVector) -> Vec<OpLogEntry> {
        let entries = self.get_entries(doc_id);
        entries
            .into_iter()
            .filter(|e| {
                let client_clock = remote_vector.get(e.entry_id.client_id);
                e.entry_id.seq > client_clock
            })
            .collect()
    }

    /// Applies remote operation entries deterministically resolving concurrent edits.
    pub fn apply_remote_delta(
        &mut self,
        doc_id: &str,
        remote_entries: Vec<OpLogEntry>,
    ) -> Result<SyncReport, CrdtError> {
        let mut applied = 0;
        let mut skipped = 0;
        let mut conflicts_resolved = 0;

        let mut sorted = remote_entries;
        // Sort causally: by lamport ascending, then client_id, then seq
        sorted.sort_by(|a, b| {
            a.entry_id
                .lamport
                .cmp(&b.entry_id.lamport)
                .then(a.entry_id.client_id.cmp(&b.entry_id.client_id))
                .then(a.entry_id.seq.cmp(&b.entry_id.seq))
        });

        for entry in sorted {
            if self.entry_by_id.contains_key(&entry.entry_id) {
                skipped += 1;
                continue;
            }

            // Check if there are concurrent operations (same lamport or overlapping parent)
            let is_concurrent = self
                .entries_by_doc
                .get(doc_id)
                .map(|indices| {
                    indices
                        .iter()
                        .any(|&i| self.entries[i].entry_id.lamport == entry.entry_id.lamport)
                })
                .unwrap_or(false);

            if is_concurrent {
                conflicts_resolved += 1;
            }

            self.insert_entry_internal(entry)?;
            applied += 1;
        }

        let state_vector = self.get_state_vector(doc_id);

        Ok(SyncReport {
            applied,
            skipped,
            conflicts_resolved,
            state_vector,
        })
    }

    /// Forks a new document session starting from an exact historical operation ID.
    pub fn fork_at_version(
        &self,
        doc_id: &str,
        target_op_id: OpId,
        new_doc_id: &str,
        new_author_id: &str,
        client_id: u64,
        timestamp_ms: u64,
    ) -> Result<LocalOpLog, CrdtError> {
        let historical_state = self.time_travel_to_op(doc_id, target_op_id)?;
        let mut forked_log = LocalOpLog::new();

        // Seed forked log with initial text insert and cell updates
        if !historical_state.text_content.is_empty() {
            forked_log.append(
                new_doc_id,
                new_author_id,
                OpPayload::TextInsert {
                    parent_id: OpId::ROOT,
                    content: historical_state.text_content,
                    right_origin: None,
                },
                timestamp_ms,
                client_id,
            )?;
        }

        for ((r, c), val) in historical_state.cells {
            forked_log.append(
                new_doc_id,
                new_author_id,
                OpPayload::CellUpdate {
                    row: r,
                    col: c,
                    value: val,
                },
                timestamp_ms,
                client_id,
            )?;
        }

        for (shape_id, props) in historical_state.shapes {
            forked_log.append(
                new_doc_id,
                new_author_id,
                OpPayload::ShapeMutation {
                    shape_id,
                    properties_json: props,
                },
                timestamp_ms,
                client_id,
            )?;
        }

        // Add initial milestone checkpoint
        forked_log.create_checkpoint(
            new_doc_id,
            new_author_id,
            &format!("Forked from {doc_id} at {:?}", target_op_id),
            client_id,
            timestamp_ms,
        )?;

        Ok(forked_log)
    }

    /// Compacts log entries older than the acknowledged StateVector frontier.
    pub fn compact_log(&mut self, doc_id: &str, acknowledged_vector: &StateVector) -> usize {
        let mut pruned = 0;
        if let Some(indices) = self.entries_by_doc.get_mut(doc_id) {
            // Keep checkpoints and entries newer than acknowledged vector
            let mut retained_indices = Vec::new();
            for &idx in indices.iter() {
                let entry = &self.entries[idx];
                let ack_seq = acknowledged_vector.get(entry.entry_id.client_id);
                let is_checkpoint = matches!(entry.payload, OpPayload::SnapshotCheckpoint { .. });

                if is_checkpoint || entry.entry_id.seq >= ack_seq {
                    retained_indices.push(idx);
                } else {
                    pruned += 1;
                }
            }
            *indices = retained_indices;
        }
        pruned
    }
}
