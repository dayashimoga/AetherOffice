//! Tamper-Evident Hash-Chained Audit Log.
//!
//! Maintains an append-only, Merkle/blockchain-style cryptographic hash chain of
//! security and document compliance events (open, save, DLP violations, sanitization, export, signing).

use crate::digest::{compute_digest, HashAlgorithm};
use serde::{Deserialize, Serialize};

pub const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Audit event classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "kebab-case")]
pub enum AuditEventType {
    DocumentOpened,
    DocumentSaved,
    DlpViolationDetected,
    DocumentSanitized,
    ExportExecuted,
    SignatureApplied,
    AccessRevoked,
    Custom(String),
}

impl AuditEventType {
    pub fn label(&self) -> String {
        match self {
            Self::DocumentOpened => "DOCUMENT_OPENED".to_string(),
            Self::DocumentSaved => "DOCUMENT_SAVED".to_string(),
            Self::DlpViolationDetected => "DLP_VIOLATION_DETECTED".to_string(),
            Self::DocumentSanitized => "DOCUMENT_SANITIZED".to_string(),
            Self::ExportExecuted => "EXPORT_EXECUTED".to_string(),
            Self::SignatureApplied => "SIGNATURE_APPLIED".to_string(),
            Self::AccessRevoked => "ACCESS_REVOKED".to_string(),
            Self::Custom(c) => format!("CUSTOM_{c}"),
        }
    }
}

/// An immutable, cryptographically chained audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub sequence_number: u64,
    pub timestamp_ms: u64,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub doc_id: String,
    pub details: String,
    pub prev_hash: String,
    pub entry_hash: String,
}

impl AuditLogEntry {
    /// Formats canonical payload string used for computing `entry_hash`.
    pub fn canonical_payload(&self) -> String {
        format!(
            "AETHER-AUDIT:v1|seq={}|ts={}|event={}|user={}|doc={}|details={}|prev={}",
            self.sequence_number,
            self.timestamp_ms,
            self.event_type.label(),
            self.user_id,
            self.doc_id,
            self.details,
            self.prev_hash
        )
    }

    /// Computes expected SHA-256 hex hash from entry fields and `prev_hash`.
    pub fn compute_expected_hash(&self) -> String {
        let payload = self.canonical_payload();
        let digest = compute_digest(HashAlgorithm::Sha256, payload.as_bytes());
        bytes_to_hex(&digest)
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Tamper-evident append-only audit log manager.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLog {
    entries: Vec<AuditLogEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends a new event entry, linking it cryptographically to the preceding entry hash.
    pub fn append(
        &mut self,
        timestamp_ms: u64,
        event_type: AuditEventType,
        user_id: impl Into<String>,
        doc_id: impl Into<String>,
        details: impl Into<String>,
    ) -> &AuditLogEntry {
        let sequence_number = (self.entries.len() as u64) + 1;
        let prev_hash = match self.entries.last() {
            Some(last) => last.entry_hash.clone(),
            None => GENESIS_PREV_HASH.to_string(),
        };

        let mut entry = AuditLogEntry {
            sequence_number,
            timestamp_ms,
            event_type,
            user_id: user_id.into(),
            doc_id: doc_id.into(),
            details: details.into(),
            prev_hash,
            entry_hash: String::new(),
        };

        entry.entry_hash = entry.compute_expected_hash();
        self.entries.push(entry);
        let last_idx = self.entries.len() - 1;
        &self.entries[last_idx]
    }

    /// Total number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retrieves immutable slice of all recorded log entries.
    pub fn entries(&self) -> &[AuditLogEntry] {
        &self.entries
    }

    /// Verifies complete cryptographic hash-chain integrity from genesis to the latest entry.
    /// Detects any modified, deleted, inserted, or reordered entries.
    pub fn verify_integrity(&self) -> Result<usize, String> {
        let mut expected_prev_hash = GENESIS_PREV_HASH.to_string();

        for (idx, entry) in self.entries.iter().enumerate() {
            let expected_seq = (idx as u64) + 1;
            if entry.sequence_number != expected_seq {
                return Err(format!(
                    "Audit log sequence break at index {idx}: expected {expected_seq}, found {}",
                    entry.sequence_number
                ));
            }

            if !entry.prev_hash.eq_ignore_ascii_case(&expected_prev_hash) {
                return Err(format!(
                    "Audit log hash-chain break at sequence {}: prev_hash {} does not match previous entry hash {}",
                    entry.sequence_number, entry.prev_hash, expected_prev_hash
                ));
            }

            let recomputed_hash = entry.compute_expected_hash();
            if !entry.entry_hash.eq_ignore_ascii_case(&recomputed_hash) {
                return Err(format!(
                    "Audit log tamper detected at sequence {}: recorded hash {} does not match computed hash {}",
                    entry.sequence_number, entry.entry_hash, recomputed_hash
                ));
            }

            expected_prev_hash = entry.entry_hash.clone();
        }

        Ok(self.entries.len())
    }

    /// Filters audit entries for a specific document.
    pub fn filter_by_document(&self, doc_id: &str) -> Vec<&AuditLogEntry> {
        self.entries.iter().filter(|e| e.doc_id == doc_id).collect()
    }

    /// Filters audit entries for a specific event type.
    pub fn filter_by_event(&self, event_type: &AuditEventType) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| &e.event_type == event_type)
            .collect()
    }
}
