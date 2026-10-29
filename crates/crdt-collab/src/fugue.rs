use crate::error::CrdtError;
use crate::id::OpId;
use crate::peritext::{FormatAttribute, PeritextSpan, RichTextRun};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FugueNode {
    pub id: OpId,
    pub parent_id: OpId,
    pub right_origin: Option<OpId>,
    pub content: char,
    pub deleted: bool,
    pub deleted_by: Option<OpId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FugueDelta {
    Insert {
        id: OpId,
        parent_id: OpId,
        right_origin: Option<OpId>,
        content: char,
    },
    Delete {
        id: OpId,
        target_id: OpId,
    },
    Format {
        span: PeritextSpan,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateVector {
    pub clocks: HashMap<u64, u32>,
}

impl StateVector {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn get(&self, client_id: u64) -> u32 {
        *self.clocks.get(&client_id).unwrap_or(&0)
    }

    pub fn set(&mut self, client_id: u64, seq: u32) {
        let current = self.clocks.entry(client_id).or_insert(0);
        if seq > *current {
            *current = seq;
        }
    }
}

#[derive(Debug, Clone)]
pub struct FugueTreeRga {
    pub client_id: u64,
    pub lamport: u64,
    pub seq: u32,
    nodes: Vec<FugueNode>,
    id_to_idx: HashMap<OpId, usize>,
    formatting_spans: Vec<PeritextSpan>,
    state_vector: StateVector,
}

impl FugueTreeRga {
    pub fn new(client_id: u64) -> Self {
        let mut rga = Self {
            client_id,
            lamport: 0,
            seq: 0,
            nodes: Vec::new(),
            id_to_idx: HashMap::new(),
            formatting_spans: Vec::new(),
            state_vector: StateVector::new(),
        };

        // Root sentinel node
        let root = FugueNode {
            id: OpId::ROOT,
            parent_id: OpId::ROOT,
            right_origin: None,
            content: '\0',
            deleted: true,
            deleted_by: None,
        };
        rga.nodes.push(root);
        rga.id_to_idx.insert(OpId::ROOT, 0);
        rga
    }

    pub fn visible_len(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| !n.deleted && !n.id.is_root())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.visible_len() == 0
    }

    pub fn get_state_vector(&self) -> &StateVector {
        &self.state_vector
    }

    pub fn to_plain_text(&self) -> String {
        let mut s = String::with_capacity(self.visible_len());
        for node in &self.nodes {
            if !node.deleted && !node.id.is_root() {
                s.push(node.content);
            }
        }
        s
    }

    /// Maps 0-based visible character index (0..visible_len) to the node's index in `self.nodes`.
    pub fn visible_char_to_node_idx(&self, char_idx: usize) -> Result<usize, CrdtError> {
        let mut count = 0;
        for (idx, node) in self.nodes.iter().enumerate() {
            if !node.deleted && !node.id.is_root() {
                if count == char_idx {
                    return Ok(idx);
                }
                count += 1;
            }
        }
        Err(CrdtError::InvalidOffset {
            offset: char_idx,
            len: count,
        })
    }

    /// Cursor offset: 0 means before the first character (parent is ROOT).
    /// offset K means after the K-th visible character.
    fn cursor_offset_to_parent_idx(&self, offset: usize) -> Result<usize, CrdtError> {
        if offset == 0 {
            return Ok(0); // ROOT
        }
        self.visible_char_to_node_idx(offset - 1)
    }

    fn next_visible_node_id(&self, from_idx: usize) -> Option<OpId> {
        for idx in (from_idx + 1)..self.nodes.len() {
            let n = match self.nodes.get(idx) {
                Some(node) => node,
                None => break,
            };
            if !n.deleted && !n.id.is_root() {
                return Some(n.id);
            }
        }
        None
    }

    pub fn insert_local(
        &mut self,
        offset: usize,
        text: &str,
    ) -> Result<Vec<FugueDelta>, CrdtError> {
        let visible_total = self.visible_len();
        if offset > visible_total {
            return Err(CrdtError::InvalidOffset {
                offset,
                len: visible_total,
            });
        }

        let mut deltas = Vec::with_capacity(text.chars().count());
        let mut prev_idx = self.cursor_offset_to_parent_idx(offset)?;
        let right_origin = self.next_visible_node_id(prev_idx);

        for ch in text.chars() {
            self.lamport += 1;
            self.seq += 1;

            let parent_id = match self.nodes.get(prev_idx) {
                Some(node) => node.id,
                None => OpId::ROOT,
            };

            let op_id = OpId::new(self.lamport, self.client_id, self.seq);
            let delta = FugueDelta::Insert {
                id: op_id,
                parent_id,
                right_origin,
                content: ch,
            };

            self.apply_delta(delta.clone())?;
            deltas.push(delta);

            if let Some(&idx) = self.id_to_idx.get(&op_id) {
                prev_idx = idx;
            }
        }

        self.state_vector.set(self.client_id, self.seq);
        Ok(deltas)
    }

    pub fn delete_local(
        &mut self,
        offset: usize,
        length: usize,
    ) -> Result<Vec<FugueDelta>, CrdtError> {
        let visible_total = self.visible_len();
        if offset + length > visible_total {
            return Err(CrdtError::InvalidOffset {
                offset: offset + length,
                len: visible_total,
            });
        }

        let mut deltas = Vec::with_capacity(length);
        for char_pos in offset..(offset + length) {
            let node_idx = self.visible_char_to_node_idx(char_pos)?;
            if let Some(node) = self.nodes.get(node_idx) {
                self.lamport += 1;
                self.seq += 1;
                let del_id = OpId::new(self.lamport, self.client_id, self.seq);
                deltas.push(FugueDelta::Delete {
                    id: del_id,
                    target_id: node.id,
                });
            }
        }

        for delta in &deltas {
            self.apply_delta(delta.clone())?;
        }

        self.state_vector.set(self.client_id, self.seq);
        Ok(deltas)
    }

    pub fn format_local(
        &mut self,
        start_offset: usize,
        end_offset: usize,
        attr: FormatAttribute,
    ) -> Result<FugueDelta, CrdtError> {
        if start_offset >= end_offset {
            return Err(CrdtError::InvalidRange {
                start: start_offset,
                end: end_offset,
            });
        }
        let visible_total = self.visible_len();
        if end_offset > visible_total {
            return Err(CrdtError::InvalidOffset {
                offset: end_offset,
                len: visible_total,
            });
        }

        let start_idx = self.visible_char_to_node_idx(start_offset)?;
        let end_idx = self.visible_char_to_node_idx(end_offset - 1)?;

        let start_id = match self.nodes.get(start_idx) {
            Some(n) => n.id,
            None => OpId::ROOT,
        };
        let end_id = match self.nodes.get(end_idx) {
            Some(n) => n.id,
            None => OpId::ROOT,
        };

        self.lamport += 1;
        self.seq += 1;
        let span_id = OpId::new(self.lamport, self.client_id, self.seq);

        let span = PeritextSpan {
            id: span_id,
            start_id,
            end_id,
            attr,
            lamport: self.lamport,
        };

        let delta = FugueDelta::Format { span };
        self.apply_delta(delta.clone())?;
        self.state_vector.set(self.client_id, self.seq);
        Ok(delta)
    }

    pub fn apply_delta(&mut self, delta: FugueDelta) -> Result<(), CrdtError> {
        match delta {
            FugueDelta::Insert {
                id,
                parent_id,
                right_origin,
                content,
            } => {
                if self.id_to_idx.contains_key(&id) {
                    return Ok(()); // Idempotent
                }

                if id.lamport > self.lamport {
                    self.lamport = id.lamport;
                }
                self.state_vector.set(id.client_id, id.seq);

                let parent_idx = match self.id_to_idx.get(&parent_id) {
                    Some(&idx) => idx,
                    None => 0, // Fallback to root
                };

                let insert_idx = self.find_insertion_index(parent_idx, right_origin, id);

                let node = FugueNode {
                    id,
                    parent_id,
                    right_origin,
                    content,
                    deleted: false,
                    deleted_by: None,
                };

                self.nodes.insert(insert_idx, node);
                self.rebuild_index();
                Ok(())
            }
            FugueDelta::Delete { id, target_id } => {
                if id.lamport > self.lamport {
                    self.lamport = id.lamport;
                }
                self.state_vector.set(id.client_id, id.seq);

                if let Some(&idx) = self.id_to_idx.get(&target_id) {
                    if let Some(node) = self.nodes.get_mut(idx) {
                        node.deleted = true;
                        node.deleted_by = Some(id);
                    }
                }
                Ok(())
            }
            FugueDelta::Format { span } => {
                if span.lamport > self.lamport {
                    self.lamport = span.lamport;
                }
                self.state_vector.set(span.id.client_id, span.id.seq);

                self.formatting_spans.push(span);
                self.formatting_spans.sort_by_key(|s| s.lamport);
                Ok(())
            }
        }
    }

    fn is_descendant(&self, mut node_id: OpId, target: OpId) -> bool {
        while !node_id.is_root() {
            if node_id == target {
                return true;
            }
            let idx = match self.id_to_idx.get(&node_id) {
                Some(&i) => i,
                None => break,
            };
            let node = match self.nodes.get(idx) {
                Some(n) => n,
                None => break,
            };
            if node.parent_id.lamport < target.lamport {
                break;
            }
            node_id = node.parent_id;
        }
        false
    }

    fn find_insertion_index(
        &self,
        parent_idx: usize,
        right_origin: Option<OpId>,
        new_id: OpId,
    ) -> usize {
        let parent_id = match self.nodes.get(parent_idx) {
            Some(n) => n.id,
            None => OpId::ROOT,
        };

        let mut dest_idx = parent_idx + 1;

        while dest_idx < self.nodes.len() {
            let next_node = match self.nodes.get(dest_idx) {
                Some(n) => n,
                None => break,
            };

            // Crucial tree constraint: all descendants of parent_id are contiguous.
            // A new child of parent_id cannot be placed past the end of parent_id's subtree.
            if !parent_id.is_root() && !self.is_descendant(next_node.id, parent_id) {
                break;
            }

            // Stop at right origin boundary
            if let Some(ro) = right_origin {
                if next_node.id == ro {
                    break;
                }
            }

            // If concurrent node has the same parent:
            if next_node.parent_id == parent_id {
                // Higher OpId takes precedence (placed before)
                if new_id > next_node.id {
                    break;
                } else {
                    // Skip next_node and all its descendants
                    let curr_id = next_node.id;
                    dest_idx += 1;
                    while dest_idx < self.nodes.len() {
                        let candidate_id = match self.nodes.get(dest_idx) {
                            Some(n) => n.id,
                            None => break,
                        };
                        if self.is_descendant(candidate_id, curr_id) {
                            dest_idx += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }

            dest_idx += 1;
        }

        dest_idx
    }

    fn rebuild_index(&mut self) {
        self.id_to_idx.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            self.id_to_idx.insert(node.id, idx);
        }
    }

    pub fn to_rich_runs(&self) -> Vec<RichTextRun> {
        let mut runs: Vec<RichTextRun> = Vec::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            if node.deleted || node.id.is_root() {
                continue;
            }

            let mut bold = false;
            let mut italic = false;
            let mut underline = false;
            let mut font_size = None;
            let mut color = None;
            let mut style = None;

            for span in &self.formatting_spans {
                let start_idx = self.id_to_idx.get(&span.start_id).copied().unwrap_or(0);
                let end_idx = self
                    .id_to_idx
                    .get(&span.end_id)
                    .copied()
                    .unwrap_or(self.nodes.len());

                if idx >= start_idx && idx <= end_idx {
                    match &span.attr {
                        FormatAttribute::Bold(b) => bold = *b,
                        FormatAttribute::Italic(i) => italic = *i,
                        FormatAttribute::Underline(u) => underline = *u,
                        FormatAttribute::FontSize(sz) => font_size = Some(*sz),
                        FormatAttribute::Color(c) => color = Some(c.clone()),
                        FormatAttribute::Style(s) => style = Some(s.clone()),
                    }
                }
            }

            let can_merge = match runs.last_mut() {
                Some(last) => {
                    last.bold == bold
                        && last.italic == italic
                        && last.underline == underline
                        && last.font_size == font_size
                        && last.color == color
                        && last.style == style
                }
                None => false,
            };

            if can_merge {
                if let Some(last) = runs.last_mut() {
                    last.text.push(node.content);
                }
            } else {
                let mut text = String::new();
                text.push(node.content);
                runs.push(RichTextRun {
                    text,
                    bold,
                    italic,
                    underline,
                    font_size,
                    color,
                    style,
                });
            }
        }

        runs
    }

    pub fn to_piece_tree(&self) -> Result<word_engine::PieceTree, CrdtError> {
        let text = self.to_plain_text();
        Ok(word_engine::PieceTree::new(&text))
    }

    pub fn from_piece_tree(
        client_id: u64,
        tree: &word_engine::PieceTree,
    ) -> Result<Self, CrdtError> {
        let mut rga = Self::new(client_id);
        let text = tree.get_text();
        if !text.is_empty() {
            let _ = rga.insert_local(0, &text)?;
        }
        Ok(rga)
    }
}
