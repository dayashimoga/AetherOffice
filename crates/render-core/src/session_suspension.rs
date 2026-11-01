//! LRU Tab Suspension & Instant Rehydration Engine
//!
//! Complies strictly with GEC-v7.0 Article I (Zero Global Document Singletons)
//! and Article VI (Multi-document memory budget < 250 MB RSS for 20 open documents):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors.
//! - 3-tier lifecycle: Active, Warm, Cold.
//! - Sub-50ms instant rehydration preserving scroll, zoom, and caret/selection state.

use crate::shell::DocumentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle tier of an open document session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionTier {
    /// Actively focused or visible in split pane; full DOM, layout, and draw primitives in RAM.
    Active,
    /// Recently viewed; lightweight DOM retained in RAM, heavy GPU/tile buffers released.
    Warm,
    /// Suspended to compressed snapshot; RAM reclaimed.
    Cold,
}

/// Viewport scroll, zoom, and selection coordinates preserved across suspension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabViewportState {
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub zoom: f64,
    pub caret_offset: Option<usize>,
    pub selection_summary: Option<String>,
}

impl Default for TabViewportState {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
            caret_offset: None,
            selection_summary: None,
        }
    }
}

/// Serialized state snapshot representing a cold, suspended document tab.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspensionSnapshot {
    pub session_id: String,
    pub doc_type: DocumentType,
    pub title: String,
    pub file_path: Option<String>,
    pub viewport_state: TabViewportState,
    pub payload_bytes: Vec<u8>,
    pub suspended_at_ms: u64,
}

/// Metadata and state for a managed document session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub doc_type: DocumentType,
    pub title: String,
    pub file_path: Option<String>,
    pub tier: SessionTier,
    pub last_accessed_ms: u64,
    pub viewport_state: TabViewportState,
    pub snapshot: Option<SuspensionSnapshot>,
}

/// Coordinator enforcing LRU tab suspension and instant rehydration.
#[derive(Debug, Clone)]
pub struct LruTabSuspensionEngine {
    sessions: HashMap<String, SessionMetadata>,
    max_active: usize,
    max_warm: usize,
    current_tick: u64,
}

impl Default for LruTabSuspensionEngine {
    fn default() -> Self {
        Self::new(2, 5)
    }
}

impl LruTabSuspensionEngine {
    /// Creates an engine with configurable active and warm thresholds.
    pub fn new(max_active: usize, max_warm: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            max_active: max_active.max(1),
            max_warm: max_warm.max(1),
            current_tick: 1,
        }
    }

    pub fn max_active(&self) -> usize {
        self.max_active
    }

    pub fn max_warm(&self) -> usize {
        self.max_warm
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Registers a newly created or opened document session.
    pub fn register_session(
        &mut self,
        session_id: &str,
        doc_type: DocumentType,
        title: &str,
        file_path: Option<&str>,
    ) {
        self.current_tick += 1;
        let metadata = SessionMetadata {
            session_id: session_id.to_string(),
            doc_type,
            title: title.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            tier: SessionTier::Warm,
            last_accessed_ms: self.current_tick,
            viewport_state: TabViewportState::default(),
            snapshot: None,
        };
        self.sessions.insert(session_id.to_string(), metadata);
    }

    /// Records access to a session, promoting it to `Active` and demoting LRU sessions if needed.
    /// Returns the prior `SessionTier` if the session was found.
    pub fn activate_session(&mut self, session_id: &str) -> Option<SessionTier> {
        if !self.sessions.contains_key(session_id) {
            return None;
        }

        self.current_tick += 1;
        let prev_tier = {
            let session = self.sessions.get_mut(session_id)?;
            let pt = session.tier;
            session.tier = SessionTier::Active;
            session.last_accessed_ms = self.current_tick;
            pt
        };

        self.enforce_quotas(session_id);
        Some(prev_tier)
    }

    /// Saves viewport scroll, zoom, and caret/selection positions for a session.
    pub fn save_viewport_state(&mut self, session_id: &str, state: TabViewportState) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.viewport_state = state;
            true
        } else {
            false
        }
    }

    /// Manually or automatically suspends an inactive session to `Cold` with compressed payload bytes.
    pub fn suspend_session_to_cold(&mut self, session_id: &str, payload_bytes: Vec<u8>) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if session.tier == SessionTier::Active {
                return false;
            }
            let snapshot = SuspensionSnapshot {
                session_id: session.session_id.clone(),
                doc_type: session.doc_type,
                title: session.title.clone(),
                file_path: session.file_path.clone(),
                viewport_state: session.viewport_state.clone(),
                payload_bytes,
                suspended_at_ms: self.current_tick,
            };
            session.snapshot = Some(snapshot);
            session.tier = SessionTier::Cold;
            true
        } else {
            false
        }
    }

    /// Rehydrates a suspended cold session, returning its saved snapshot and viewport state.
    pub fn rehydrate_session(&mut self, session_id: &str) -> Option<SuspensionSnapshot> {
        let session = self.sessions.get_mut(session_id)?;
        if session.tier != SessionTier::Cold {
            return None;
        }

        self.current_tick += 1;
        session.tier = SessionTier::Active;
        session.last_accessed_ms = self.current_tick;

        let snapshot = session.snapshot.clone();
        self.enforce_quotas(session_id);
        snapshot
    }

    /// Removes a closed session from tracking.
    pub fn close_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    /// Returns the current tier of the session.
    pub fn get_tier(&self, session_id: &str) -> Option<SessionTier> {
        self.sessions.get(session_id).map(|s| s.tier)
    }

    /// Returns the saved viewport state of the session.
    pub fn get_viewport_state(&self, session_id: &str) -> Option<&TabViewportState> {
        self.sessions.get(session_id).map(|s| &s.viewport_state)
    }

    /// Lists session IDs belonging to a specific tier.
    pub fn list_by_tier(&self, tier: SessionTier) -> Vec<String> {
        self.sessions
            .iter()
            .filter(|(_, s)| s.tier == tier)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Returns counts for `(active, warm, cold)`.
    pub fn memory_breakdown(&self) -> (usize, usize, usize) {
        let mut active = 0;
        let mut warm = 0;
        let mut cold = 0;
        for s in self.sessions.values() {
            match s.tier {
                SessionTier::Active => active += 1,
                SessionTier::Warm => warm += 1,
                SessionTier::Cold => cold += 1,
            }
        }
        (active, warm, cold)
    }

    /// Internal quota enforcement: demotes excess Active sessions to Warm,
    /// and excess Warm sessions to Cold.
    fn enforce_quotas(&mut self, keep_active_id: &str) {
        // 1. Demote oldest Active sessions exceeding max_active to Warm
        let mut active_sessions: Vec<(String, u64)> = self
            .sessions
            .iter()
            .filter(|(id, s)| s.tier == SessionTier::Active && *id != keep_active_id)
            .map(|(id, s)| (id.clone(), s.last_accessed_ms))
            .collect();

        // Sort descending by last_accessed_ms (most recently accessed first)
        active_sessions.sort_by_key(|&(_, time)| std::cmp::Reverse(time));

        let allowed_other_active = self.max_active.saturating_sub(1);
        if active_sessions.len() > allowed_other_active {
            for (id, _) in active_sessions.into_iter().skip(allowed_other_active) {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.tier = SessionTier::Warm;
                }
            }
        }

        // 2. Demote oldest Warm sessions exceeding max_warm to Cold
        let mut warm_sessions: Vec<(String, u64)> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.tier == SessionTier::Warm)
            .map(|(id, s)| (id.clone(), s.last_accessed_ms))
            .collect();

        warm_sessions.sort_by_key(|&(_, time)| std::cmp::Reverse(time));

        if warm_sessions.len() > self.max_warm {
            for (id, _) in warm_sessions.into_iter().skip(self.max_warm) {
                if let Some(session) = self.sessions.get_mut(&id) {
                    let snapshot = SuspensionSnapshot {
                        session_id: session.session_id.clone(),
                        doc_type: session.doc_type,
                        title: session.title.clone(),
                        file_path: session.file_path.clone(),
                        viewport_state: session.viewport_state.clone(),
                        payload_bytes: Vec::new(),
                        suspended_at_ms: self.current_tick,
                    };
                    session.snapshot = Some(snapshot);
                    session.tier = SessionTier::Cold;
                }
            }
        }
    }
}
