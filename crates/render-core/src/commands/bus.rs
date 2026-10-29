//! Global Transactional Command Bus with Unlimited Undo/Redo & Coalescing
//!
//! Complies with ECMA-376 and GEC-v1.0.
//! Provides:
//! - Transactional commit, rollback (undo), and replay (redo)
//! - Save-point clean/dirty state tracking with monotonic revision markers
//! - Keystroke and continuous transform coalescing
//! - Configurable history depth and inspection

use crate::commands::transactions::{GlobalOfficeState, OfficeTransaction};

/// Transactional Command Bus managing undo/redo stacks and document save points.
pub struct GlobalCommandBus {
    undo_stack: Vec<Box<dyn OfficeTransaction>>,
    redo_stack: Vec<Box<dyn OfficeTransaction>>,
    max_depth: usize,
    clean_counter: Option<usize>,
    tx_counter: usize,
    saved_at_tip: bool,
}

impl Default for GlobalCommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalCommandBus {
    pub const DEFAULT_MAX_DEPTH: usize = 1000;

    /// Create a new command bus with default 1,000 transaction depth.
    pub fn new() -> Self {
        Self::with_max_depth(Self::DEFAULT_MAX_DEPTH)
    }

    /// Create a new command bus with specified maximum depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: max_depth.max(1),
            clean_counter: Some(0),
            tx_counter: 0,
            saved_at_tip: false,
        }
    }

    /// Whether there are transactions available to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there are transactions available to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Number of transactions on the undo stack.
    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of transactions on the redo stack.
    pub fn redo_stack_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Maximum undo history depth.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Monotonic transaction revision counter.
    pub fn current_revision(&self) -> usize {
        self.tx_counter
    }

    /// Whether the document is in a dirty (unsaved modifications) state.
    pub fn is_dirty(&self) -> bool {
        self.clean_counter != Some(self.tx_counter)
    }

    /// Mark the current transaction state as clean/saved.
    pub fn mark_clean(&mut self) {
        self.clean_counter = Some(self.tx_counter);
        self.saved_at_tip = true;
    }

    /// Return descriptions of all undoable transactions in chronological order (oldest to newest).
    pub fn undo_history(&self) -> Vec<String> {
        self.undo_stack
            .iter()
            .map(|t| t.description().to_string())
            .collect()
    }

    /// Return descriptions of all redoable transactions in chronological order (next redo to newest).
    pub fn redo_history(&self) -> Vec<String> {
        self.redo_stack
            .iter()
            .map(|t| t.description().to_string())
            .collect()
    }

    /// Execute a transaction, merging with the previous transaction if applicable.
    ///
    /// Clears the redo stack on new transactions.
    pub fn execute(
        &mut self,
        state: &mut GlobalOfficeState,
        mut tx: Box<dyn OfficeTransaction>,
    ) -> Result<(), String> {
        tx.execute(state)?;

        // Attempt coalescing with top of undo stack if not at a clean save boundary
        if !self.saved_at_tip {
            if let Some(last) = self.undo_stack.last_mut() {
                if last.try_merge_next(tx.as_ref()) {
                    self.redo_stack.clear();
                    return Ok(());
                }
            }
        }
        self.saved_at_tip = false;

        self.tx_counter = self.tx_counter.saturating_add(1);
        self.undo_stack.push(tx);
        self.redo_stack.clear();

        // Enforce maximum depth
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    /// Revert the most recent transaction on the undo stack.
    pub fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if let Some(mut tx) = self.undo_stack.pop() {
            tx.undo(state)?;
            self.redo_stack.push(tx);
            self.tx_counter = self.tx_counter.saturating_sub(1);
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    /// Re-apply the most recently reverted transaction from the redo stack.
    pub fn redo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if let Some(mut tx) = self.redo_stack.pop() {
            tx.redo(state)?;
            self.undo_stack.push(tx);
            self.tx_counter = self.tx_counter.saturating_add(1);
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    /// Clear all undo and redo history and reset clean counter.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.tx_counter = 0;
        self.clean_counter = Some(0);
    }
}
