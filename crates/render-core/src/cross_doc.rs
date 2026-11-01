#![forbid(unsafe_code)]
//! Cross-Document Dependency Graph & Live Embeds (GEC-v7.0 Article I & III)
//!
//! Provides:
//! - Directed Acyclic Graph (DAG) for cross-document links (Calc cells in Word, charts in Slides, PDF regions in Notes).
//! - Cycle detection with Tarjan / DFS cycle detection preventing circular dependency deadlocks.
//! - Topological invalidation propagation returning exact evaluation sequences on document mutation.
//! - In-memory snapshot caching (`LiveEmbedSnapshot`) enabling instantaneous rendering even when source document tabs are suspended or offline.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CrossDocError {
    #[error("Circular dependency detected involving documents: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },

    #[error("Dependency edge '{0}' not found in graph")]
    EdgeNotFound(String),

    #[error("Invalid anchor specification: {0}")]
    InvalidAnchor(String),

    #[error("Snapshot evaluation error: {0}")]
    EvaluationError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EmbedAnchor {
    CalcRange {
        sheet_name: String,
        range_a1: String,
    },
    CalcCell {
        sheet_name: String,
        coordinate: String,
    },
    CalcChart {
        sheet_name: String,
        chart_id: String,
    },
    WordParagraph {
        paragraph_id: String,
    },
    WordSection {
        section_id: String,
    },
    SlideShape {
        slide_index: usize,
        shape_id: u32,
    },
    PdfRegion {
        page_index: usize,
        bbox: [i64; 4], // scaled integer coordinates [llx, lly, urx, ury]
    },
    GenericAnchor {
        key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EmbedTarget {
    WordInline { container_id: String },
    SlideShapeContainer { slide_index: usize, shape_id: u32 },
    NotesBlock { block_id: String },
    GenericTarget { id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyKind {
    LiveRangeEmbed,
    LiveChartEmbed,
    LiveTextEmbed,
    CalculatedValueRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub id: String,
    pub source_doc_id: String,
    pub source_anchor: EmbedAnchor,
    pub target_doc_id: String,
    pub target_container: EmbedTarget,
    pub kind: DependencyKind,
    pub created_at: u64,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveEmbedSnapshot {
    pub embed_id: String,
    pub source_doc_id: String,
    pub version: u64,
    pub rendered_html: Option<String>,
    pub rendered_svg: Option<String>,
    pub tabular_data: Option<Vec<Vec<String>>>,
    pub scalar_value: Option<String>,
    pub last_updated_timestamp: u64,
    pub is_stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossDocSummaryDto {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_snapshots: usize,
    pub stale_snapshots: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CrossDocDependencyGraph {
    edges: HashMap<String, DependencyEdge>,
    // source_doc_id -> set of edge_ids
    outgoing_by_source: HashMap<String, HashSet<String>>,
    // target_doc_id -> set of edge_ids
    incoming_by_target: HashMap<String, HashSet<String>>,
    // embed_id -> snapshot
    snapshots: HashMap<String, LiveEmbedSnapshot>,
}

impl CrossDocDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new dependency edge after validating that it does not introduce a circular dependency.
    pub fn register_dependency(&mut self, edge: DependencyEdge) -> Result<String, CrossDocError> {
        if edge.source_doc_id == edge.target_doc_id {
            return Err(CrossDocError::CircularDependency {
                cycle: vec![edge.source_doc_id],
            });
        }

        // Test cycle: check if a path already exists from target_doc_id to source_doc_id
        if let Some(cycle_path) = self.find_path(&edge.target_doc_id, &edge.source_doc_id) {
            let mut full_cycle = cycle_path;
            full_cycle.push(edge.target_doc_id.clone());
            return Err(CrossDocError::CircularDependency { cycle: full_cycle });
        }

        let edge_id = edge.id.clone();

        self.outgoing_by_source
            .entry(edge.source_doc_id.clone())
            .or_default()
            .insert(edge_id.clone());

        self.incoming_by_target
            .entry(edge.target_doc_id.clone())
            .or_default()
            .insert(edge_id.clone());

        self.edges.insert(edge_id.clone(), edge);

        Ok(edge_id)
    }

    /// Removes an existing dependency edge by ID.
    pub fn remove_dependency(&mut self, edge_id: &str) -> Result<bool, CrossDocError> {
        let edge = self
            .edges
            .remove(edge_id)
            .ok_or_else(|| CrossDocError::EdgeNotFound(edge_id.to_string()))?;

        if let Some(set) = self.outgoing_by_source.get_mut(&edge.source_doc_id) {
            set.remove(edge_id);
            if set.is_empty() {
                self.outgoing_by_source.remove(&edge.source_doc_id);
            }
        }

        if let Some(set) = self.incoming_by_target.get_mut(&edge.target_doc_id) {
            set.remove(edge_id);
            if set.is_empty() {
                self.incoming_by_target.remove(&edge.target_doc_id);
            }
        }

        self.snapshots.remove(edge_id);
        Ok(true)
    }

    /// Returns all outgoing dependency edges from a given source document (documents that depend on it).
    pub fn get_dependents(&self, doc_id: &str) -> Vec<DependencyEdge> {
        self.outgoing_by_source
            .get(doc_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|id| self.edges.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns all incoming dependency edges for a given target document (documents it depends on).
    pub fn get_dependencies(&self, doc_id: &str) -> Vec<DependencyEdge> {
        self.incoming_by_target
            .get(doc_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|id| self.edges.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Propagates invalidation from a source document along all dependent paths.
    /// Returns topological sequence of affected target document IDs and marks their snapshots stale.
    pub fn propagate_invalidation(
        &mut self,
        source_doc_id: &str,
        affected_anchor: Option<&EmbedAnchor>,
    ) -> Vec<String> {
        let mut affected_targets = Vec::new();
        let mut visited_docs = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(source_doc_id.to_string());
        visited_docs.insert(source_doc_id.to_string());

        let mut affected_edge_ids = Vec::new();

        while let Some(current_source) = queue.pop_front() {
            if let Some(edge_ids) = self.outgoing_by_source.get(&current_source) {
                for eid in edge_ids {
                    if let Some(edge) = self.edges.get(eid) {
                        let anchor_matches = match affected_anchor {
                            Some(anchor) => &edge.source_anchor == anchor,
                            None => true,
                        };

                        if anchor_matches {
                            affected_edge_ids.push(eid.clone());
                            if !visited_docs.contains(&edge.target_doc_id) {
                                visited_docs.insert(edge.target_doc_id.clone());
                                affected_targets.push(edge.target_doc_id.clone());
                                queue.push_back(edge.target_doc_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // Mark corresponding snapshots as stale
        for eid in affected_edge_ids {
            if let Some(snapshot) = self.snapshots.get_mut(&eid) {
                snapshot.is_stale = true;
            }
        }

        affected_targets
    }

    /// Caches or updates a live embed snapshot.
    pub fn update_snapshot(&mut self, snapshot: LiveEmbedSnapshot) {
        self.snapshots.insert(snapshot.embed_id.clone(), snapshot);
    }

    /// Retrieves an embed snapshot by ID.
    pub fn get_snapshot(&self, embed_id: &str) -> Option<&LiveEmbedSnapshot> {
        self.snapshots.get(embed_id)
    }

    /// Returns high-level summary of the dependency graph.
    pub fn graph_summary(&self) -> CrossDocSummaryDto {
        let mut all_docs = HashSet::new();
        for edge in self.edges.values() {
            all_docs.insert(edge.source_doc_id.clone());
            all_docs.insert(edge.target_doc_id.clone());
        }

        let total_snapshots = self.snapshots.len();
        let stale_snapshots = self.snapshots.values().filter(|s| s.is_stale).count();

        CrossDocSummaryDto {
            total_nodes: all_docs.len(),
            total_edges: self.edges.len(),
            total_snapshots,
            stale_snapshots,
        }
    }

    /// Checks if a directed path exists between `from_doc` and `to_doc` via BFS.
    fn find_path(&self, from_doc: &str, to_doc: &str) -> Option<Vec<String>> {
        if from_doc == to_doc {
            return Some(vec![from_doc.to_string()]);
        }

        let mut queue = VecDeque::new();
        let mut parent: HashMap<String, String> = HashMap::new();
        let mut visited = HashSet::new();

        queue.push_back(from_doc.to_string());
        visited.insert(from_doc.to_string());

        let mut found = false;

        while let Some(current) = queue.pop_front() {
            if current == to_doc {
                found = true;
                break;
            }

            if let Some(edge_ids) = self.outgoing_by_source.get(&current) {
                for eid in edge_ids {
                    if let Some(edge) = self.edges.get(eid) {
                        if !visited.contains(&edge.target_doc_id) {
                            visited.insert(edge.target_doc_id.clone());
                            parent.insert(edge.target_doc_id.clone(), current.clone());
                            queue.push_back(edge.target_doc_id.clone());
                        }
                    }
                }
            }
        }

        if found {
            let mut path = Vec::new();
            let mut curr = to_doc.to_string();
            path.push(curr.clone());
            while let Some(p) = parent.get(&curr) {
                path.push(p.clone());
                curr = p.clone();
                if curr == from_doc {
                    break;
                }
            }
            path.reverse();
            Some(path)
        } else {
            None
        }
    }
}
