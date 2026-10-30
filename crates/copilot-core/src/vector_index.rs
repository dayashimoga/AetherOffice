use crate::chunker::DocumentChunk;
use crate::embedding::{
    cosine_similarity, cosine_similarity_quantized, QuantizedVector, EMBEDDING_DIM,
};
use crate::error::CopilotError;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// An indexed entry storing the chunk and both full-precision and quantized embeddings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexEntry {
    pub chunk: DocumentChunk,
    pub embedding: Vec<f32>,
    pub quantized: QuantizedVector,
}

/// A search match returned with similarity score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: DocumentChunk,
    pub similarity: f32,
}

#[derive(Clone)]
struct ScoredCandidate {
    score: f32,
    index: usize,
}

impl PartialEq for ScoredCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.index == other.index
    }
}

impl Eq for ScoredCandidate {}

impl PartialOrd for ScoredCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
    }
}

/// High-performance in-memory vector index with top-k ranking and incremental updates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorIndex {
    entries: Vec<IndexEntry>,
    chunk_id_to_idx: HashMap<String, usize>,
    doc_id_to_chunks: HashMap<String, HashSet<String>>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            chunk_id_to_idx: HashMap::new(),
            doc_id_to_chunks: HashMap::new(),
        }
    }

    /// Total number of indexed chunks.
    pub fn total_chunks(&self) -> usize {
        self.entries.len()
    }

    /// Total number of distinct documents currently indexed.
    pub fn total_documents(&self) -> usize {
        self.doc_id_to_chunks.len()
    }

    /// List of all document IDs indexed.
    pub fn document_ids(&self) -> Vec<String> {
        self.doc_id_to_chunks.keys().cloned().collect()
    }

    /// Add a single chunk with its embedding into the index.
    pub fn add_chunk(
        &mut self,
        chunk: DocumentChunk,
        embedding: Vec<f32>,
    ) -> Result<(), CopilotError> {
        if embedding.len() != EMBEDDING_DIM {
            return Err(CopilotError::DimensionMismatch {
                expected: EMBEDDING_DIM,
                found: embedding.len(),
            });
        }

        // If chunk ID already exists, update it in place
        if let Some(&existing_idx) = self.chunk_id_to_idx.get(&chunk.id) {
            let quantized = QuantizedVector::from_f32(&embedding);
            self.entries[existing_idx] = IndexEntry {
                chunk,
                embedding,
                quantized,
            };
            return Ok(());
        }

        let chunk_id = chunk.id.clone();
        let doc_id = chunk.doc_id.clone();
        let quantized = QuantizedVector::from_f32(&embedding);
        let new_idx = self.entries.len();

        self.entries.push(IndexEntry {
            chunk,
            embedding,
            quantized,
        });

        self.chunk_id_to_idx.insert(chunk_id.clone(), new_idx);
        self.doc_id_to_chunks
            .entry(doc_id)
            .or_default()
            .insert(chunk_id);

        Ok(())
    }

    /// Batch add chunks with embeddings.
    pub fn add_chunks(
        &mut self,
        chunks: Vec<(DocumentChunk, Vec<f32>)>,
    ) -> Result<(), CopilotError> {
        for (chunk, embedding) in chunks {
            self.add_chunk(chunk, embedding)?;
        }
        Ok(())
    }

    /// Remove a single chunk by ID.
    pub fn remove_chunk(&mut self, chunk_id: &str) -> bool {
        if let Some(idx) = self.chunk_id_to_idx.remove(chunk_id) {
            let entry = self.entries.swap_remove(idx);

            // Update mapped index for the swapped element if any
            if idx < self.entries.len() {
                let swapped_id = &self.entries[idx].chunk.id;
                self.chunk_id_to_idx.insert(swapped_id.clone(), idx);
            }

            // Remove from doc mapping
            if let Some(chunk_set) = self.doc_id_to_chunks.get_mut(&entry.chunk.doc_id) {
                chunk_set.remove(chunk_id);
                if chunk_set.is_empty() {
                    self.doc_id_to_chunks.remove(&entry.chunk.doc_id);
                }
            }
            true
        } else {
            false
        }
    }

    /// Incremental document purge: removes all chunks belonging to a document.
    /// Used when a document is modified and re-indexed.
    pub fn remove_document(&mut self, doc_id: &str) -> usize {
        let chunk_ids = match self.doc_id_to_chunks.remove(doc_id) {
            Some(ids) => ids,
            None => return 0,
        };

        let count = chunk_ids.len();
        for cid in chunk_ids {
            if let Some(idx) = self.chunk_id_to_idx.remove(&cid) {
                if idx < self.entries.len() {
                    self.entries.swap_remove(idx);
                    if idx < self.entries.len() {
                        let swapped_id = &self.entries[idx].chunk.id;
                        self.chunk_id_to_idx.insert(swapped_id.clone(), idx);
                    }
                }
            }
        }

        count
    }

    /// Incremental document update: purges existing chunks for the document and re-inserts fresh chunks.
    pub fn update_document(
        &mut self,
        doc_id: &str,
        new_chunks: Vec<(DocumentChunk, Vec<f32>)>,
    ) -> Result<usize, CopilotError> {
        self.remove_document(doc_id);
        let count = new_chunks.len();
        self.add_chunks(new_chunks)?;
        Ok(count)
    }

    /// Search the index using floating-point cosine similarity with bounded min-heap for top-k.
    pub fn search(
        &self,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>, CopilotError> {
        if self.entries.is_empty() {
            return Ok(Vec::new());
        }
        if query_vector.len() != EMBEDDING_DIM {
            return Err(CopilotError::DimensionMismatch {
                expected: EMBEDDING_DIM,
                found: query_vector.len(),
            });
        }
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let mut heap: BinaryHeap<std::cmp::Reverse<ScoredCandidate>> =
            BinaryHeap::with_capacity(top_k + 1);

        for (idx, entry) in self.entries.iter().enumerate() {
            let sim = cosine_similarity(query_vector, &entry.embedding)?;
            if sim >= threshold {
                let candidate = ScoredCandidate {
                    score: sim,
                    index: idx,
                };
                if heap.len() < top_k {
                    heap.push(std::cmp::Reverse(candidate));
                } else if let Some(min_top) = heap.peek() {
                    if sim > min_top.0.score {
                        heap.pop();
                        heap.push(std::cmp::Reverse(candidate));
                    }
                }
            }
        }

        let mut results = Vec::with_capacity(heap.len());
        while let Some(std::cmp::Reverse(candidate)) = heap.pop() {
            if let Some(entry) = self.entries.get(candidate.index) {
                results.push(SearchResult {
                    chunk: entry.chunk.clone(),
                    similarity: candidate.score,
                });
            }
        }

        // Min-heap pops smallest first, so reverse to order descending by similarity
        results.reverse();
        Ok(results)
    }

    /// Search using quantized int8 vectors for maximum throughput.
    pub fn search_quantized(
        &self,
        query_quantized: &QuantizedVector,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>, CopilotError> {
        if self.entries.is_empty() {
            return Ok(Vec::new());
        }
        if query_quantized.data.len() != EMBEDDING_DIM {
            return Err(CopilotError::DimensionMismatch {
                expected: EMBEDDING_DIM,
                found: query_quantized.data.len(),
            });
        }
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let mut heap: BinaryHeap<std::cmp::Reverse<ScoredCandidate>> =
            BinaryHeap::with_capacity(top_k + 1);

        for (idx, entry) in self.entries.iter().enumerate() {
            let sim = cosine_similarity_quantized(query_quantized, &entry.quantized)?;
            if sim >= threshold {
                let candidate = ScoredCandidate {
                    score: sim,
                    index: idx,
                };
                if heap.len() < top_k {
                    heap.push(std::cmp::Reverse(candidate));
                } else if let Some(min_top) = heap.peek() {
                    if sim > min_top.0.score {
                        heap.pop();
                        heap.push(std::cmp::Reverse(candidate));
                    }
                }
            }
        }

        let mut results = Vec::with_capacity(heap.len());
        while let Some(std::cmp::Reverse(candidate)) = heap.pop() {
            if let Some(entry) = self.entries.get(candidate.index) {
                results.push(SearchResult {
                    chunk: entry.chunk.clone(),
                    similarity: candidate.score,
                });
            }
        }

        results.reverse();
        Ok(results)
    }

    /// Search with arbitrary predicate filter (e.g. document ID or kind).
    pub fn search_filtered<F>(
        &self,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
        filter: F,
    ) -> Result<Vec<SearchResult>, CopilotError>
    where
        F: Fn(&DocumentChunk) -> bool,
    {
        if self.entries.is_empty() {
            return Ok(Vec::new());
        }
        if query_vector.len() != EMBEDDING_DIM {
            return Err(CopilotError::DimensionMismatch {
                expected: EMBEDDING_DIM,
                found: query_vector.len(),
            });
        }
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let mut heap: BinaryHeap<std::cmp::Reverse<ScoredCandidate>> =
            BinaryHeap::with_capacity(top_k + 1);

        for (idx, entry) in self.entries.iter().enumerate() {
            if !filter(&entry.chunk) {
                continue;
            }
            let sim = cosine_similarity(query_vector, &entry.embedding)?;
            if sim >= threshold {
                let candidate = ScoredCandidate {
                    score: sim,
                    index: idx,
                };
                if heap.len() < top_k {
                    heap.push(std::cmp::Reverse(candidate));
                } else if let Some(min_top) = heap.peek() {
                    if sim > min_top.0.score {
                        heap.pop();
                        heap.push(std::cmp::Reverse(candidate));
                    }
                }
            }
        }

        let mut results = Vec::with_capacity(heap.len());
        while let Some(std::cmp::Reverse(candidate)) = heap.pop() {
            if let Some(entry) = self.entries.get(candidate.index) {
                results.push(SearchResult {
                    chunk: entry.chunk.clone(),
                    similarity: candidate.score,
                });
            }
        }

        results.reverse();
        Ok(results)
    }

    /// Clear all index data.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.chunk_id_to_idx.clear();
        self.doc_id_to_chunks.clear();
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, CopilotError> {
        serde_json::to_string(self).map_err(|e| CopilotError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, CopilotError> {
        serde_json::from_str(json_str)
            .map_err(|e| CopilotError::DeserializationError(e.to_string()))
    }
}
