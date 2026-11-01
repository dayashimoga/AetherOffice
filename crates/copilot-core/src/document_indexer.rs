//! Incremental Multi-Format Document Indexer & Hybrid Search Engine
//!
//! Complies strictly with GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled error states.
//! - Incremental content-hashed re-indexing: skips re-embedding unchanged sections.
//! - Multi-format ingestion for Word, Excel, PowerPoint, PDF, and PlainText/Markdown.
//! - Hybrid lexical (term frequency) + dense semantic (cosine similarity) ranking.

use crate::chunker::{
    ChunkerConfig, DocumentChunk, DocumentKind, SemanticChunker, SlideInput, WordParagraphInput,
};
use crate::embedding::{LocalEmbeddingGenerator, EMBEDDING_DIM};
use crate::error::CopilotError;
use crate::vector_index::VectorIndex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Search filter criteria for narrowing document search queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Optional document format restriction.
    pub kind: Option<DocumentKind>,
    /// Optional whitelist of document IDs.
    pub doc_ids: Option<Vec<String>>,
    /// Minimum composite similarity score (0.0 to 1.0).
    pub min_score: Option<f32>,
}

/// A matched search result combining semantic and lexical relevance scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// Matched document chunk.
    pub chunk: DocumentChunk,
    /// Dense vector cosine similarity score [0.0..1.0].
    pub semantic_score: f32,
    /// Lexical token / keyword match score [0.0..1.0].
    pub lexical_score: f32,
    /// Combined hybrid score [0.0..1.0].
    pub score: f32,
}

/// Statistics and health summary of the workspace index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSummary {
    /// Total number of unique documents indexed.
    pub total_documents: usize,
    /// Total number of semantic chunks stored.
    pub total_chunks: usize,
    /// Distribution of chunks by document category.
    pub chunks_by_kind: HashMap<String, usize>,
    /// Estimated token count across all active chunks.
    pub total_estimated_tokens: usize,
}

/// High-performance incremental document indexer.
pub struct DocumentIndexer {
    chunker: SemanticChunker,
    embedder: LocalEmbeddingGenerator,
    index: VectorIndex,
    /// Maps chunk_id -> content hash for change detection.
    chunk_hashes: HashMap<String, u64>,
    /// Maps doc_id -> DocumentKind.
    doc_kinds: HashMap<String, DocumentKind>,
}

impl Default for DocumentIndexer {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentIndexer {
    /// Creates a new document indexer with default chunking and embedding configurations.
    pub fn new() -> Self {
        Self {
            chunker: SemanticChunker::new(ChunkerConfig::default()),
            embedder: LocalEmbeddingGenerator::new(),
            index: VectorIndex::new(),
            chunk_hashes: HashMap::new(),
            doc_kinds: HashMap::new(),
        }
    }

    /// Computes a 64-bit content hash of the chunk text and location.
    fn compute_chunk_hash(chunk: &DocumentChunk) -> u64 {
        let mut hasher = DefaultHasher::new();
        chunk.composite_text.hash(&mut hasher);
        chunk.structural_context.hash(&mut hasher);
        hasher.finish()
    }

    /// Incrementally updates the index with a fresh list of document chunks.
    ///
    /// Preserves existing chunks that have unchanged content hashes, purging obsolete
    /// chunks and only computing embeddings for new or modified content.
    pub fn index_chunks(&mut self, chunks: Vec<DocumentChunk>) -> Result<usize, CopilotError> {
        if chunks.is_empty() {
            return Ok(0);
        }

        let doc_id = match chunks.first() {
            Some(c) => c.doc_id.clone(),
            None => return Ok(0),
        };

        if let Some(first_chunk) = chunks.first() {
            self.doc_kinds.insert(doc_id.clone(), first_chunk.kind);
        }

        let mut incoming_ids = HashSet::with_capacity(chunks.len());
        let mut chunks_to_embed = Vec::new();
        let mut retained_count = 0;

        for chunk in chunks {
            let chunk_id = chunk.id.clone();
            incoming_ids.insert(chunk_id.clone());
            let current_hash = Self::compute_chunk_hash(&chunk);

            // Check if chunk is already indexed with identical content hash
            if let Some(&stored_hash) = self.chunk_hashes.get(&chunk_id) {
                if stored_hash == current_hash {
                    retained_count += 1;
                    continue;
                }
            }

            // Generate dense embedding for new or modified chunk
            let embedding = self.embedder.embed_text(&chunk.composite_text);
            self.chunk_hashes.insert(chunk_id, current_hash);
            chunks_to_embed.push((chunk, embedding));
        }

        // Purge chunks that belonged to this document but are absent in the incoming list
        let mut ids_to_remove = Vec::new();
        for cid in self.chunk_hashes.keys() {
            if cid.starts_with(&format!("{doc_id}:")) && !incoming_ids.contains(cid) {
                ids_to_remove.push(cid.clone());
            }
        }

        for cid in ids_to_remove {
            self.chunk_hashes.remove(&cid);
            self.index.remove_chunk(&cid);
        }

        let newly_indexed = chunks_to_embed.len();
        self.index.add_chunks(chunks_to_embed)?;

        Ok(retained_count + newly_indexed)
    }

    /// Indexes a Word document from structured paragraph inputs.
    pub fn index_word_document(
        &mut self,
        doc_id: &str,
        doc_title: &str,
        paragraphs: &[WordParagraphInput],
    ) -> Result<usize, CopilotError> {
        let chunks = self
            .chunker
            .chunk_word_document(doc_id, doc_title, paragraphs)?;
        self.index_chunks(chunks)
    }

    /// Indexes an Excel spreadsheet from grid row data.
    pub fn index_calc_sheet(
        &mut self,
        doc_id: &str,
        sheet_name: &str,
        headers: &[String],
        rows: &[Vec<String>],
        rows_per_chunk: usize,
    ) -> Result<usize, CopilotError> {
        let chunks =
            self.chunker
                .chunk_calc_sheet(doc_id, sheet_name, headers, rows, rows_per_chunk)?;
        self.index_chunks(chunks)
    }

    /// Indexes a PowerPoint presentation from slide shape inputs.
    pub fn index_presentation(
        &mut self,
        doc_id: &str,
        presentation_title: &str,
        slides: &[SlideInput],
    ) -> Result<usize, CopilotError> {
        let chunks = self
            .chunker
            .chunk_slides(doc_id, presentation_title, slides)?;
        self.index_chunks(chunks)
    }

    /// Indexes a plain text or Markdown document.
    pub fn index_plain_text(
        &mut self,
        doc_id: &str,
        doc_title: &str,
        text: &str,
    ) -> Result<usize, CopilotError> {
        let chunks = self.chunker.chunk_plain_text(doc_id, doc_title, text)?;
        self.index_chunks(chunks)
    }

    /// Completely removes a document and all its chunks from the index.
    pub fn remove_document(&mut self, doc_id: &str) -> usize {
        self.doc_kinds.remove(doc_id);
        let prefix = format!("{doc_id}:");
        self.chunk_hashes.retain(|k, _| !k.starts_with(&prefix));
        self.index.remove_document(doc_id)
    }

    /// Calculates a lexical score [0.0..1.0] based on query token matches.
    fn calculate_lexical_score(query: &str, chunk: &DocumentChunk) -> f32 {
        let query_lower = query.to_ascii_lowercase();
        let query_tokens: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .collect();

        if query_tokens.is_empty() {
            return 0.0;
        }

        let raw_lower = chunk.raw_text.to_ascii_lowercase();
        let context_lower = chunk.structural_context.to_ascii_lowercase();

        let mut matches = 0;
        for token in &query_tokens {
            if raw_lower.contains(token) || context_lower.contains(token) {
                matches += 1;
            }
        }

        let token_coverage = (matches as f32) / (query_tokens.len() as f32);

        // Boost exact phrase matches
        let exact_boost = if raw_lower.contains(&query_lower) {
            0.2
        } else {
            0.0
        };

        (token_coverage * 0.8 + exact_boost).clamp(0.0, 1.0)
    }

    /// Performs hybrid search combining semantic similarity and keyword presence.
    pub fn search(
        &self,
        query: &str,
        top_k: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<HybridSearchResult>, CopilotError> {
        if top_k == 0 || query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let query_vector = self.embedder.embed_text(query);
        if query_vector.len() != EMBEDDING_DIM {
            return Err(CopilotError::DimensionMismatch {
                expected: EMBEDDING_DIM,
                found: query_vector.len(),
            });
        }

        // Retrieve top candidates using vector index
        let semantic_results = self.index.search(&query_vector, top_k * 2, -1.0)?;

        let mut hybrid_matches = Vec::new();
        let min_threshold = filter.as_ref().and_then(|f| f.min_score).unwrap_or(0.0);

        for res in semantic_results {
            // Apply filter criteria
            if let Some(ref f) = filter {
                if let Some(expected_kind) = f.kind {
                    if res.chunk.kind != expected_kind {
                        continue;
                    }
                }
                if let Some(ref allowed_ids) = f.doc_ids {
                    if !allowed_ids.contains(&res.chunk.doc_id) {
                        continue;
                    }
                }
            }

            let semantic_score = ((res.similarity + 1.0) / 2.0).clamp(0.0, 1.0);
            let lexical_score = Self::calculate_lexical_score(query, &res.chunk);

            // Weighted combination: 65% semantic, 35% lexical
            let score = (0.65 * semantic_score + 0.35 * lexical_score).clamp(0.0, 1.0);

            if score >= min_threshold {
                hybrid_matches.push(HybridSearchResult {
                    chunk: res.chunk,
                    semantic_score,
                    lexical_score,
                    score,
                });
            }
        }

        // Sort descending by hybrid score
        hybrid_matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if hybrid_matches.len() > top_k {
            hybrid_matches.truncate(top_k);
        }

        Ok(hybrid_matches)
    }

    /// Returns a statistical summary of the indexed corpus.
    pub fn get_summary(&self) -> IndexSummary {
        let mut chunks_by_kind = HashMap::new();
        let mut total_tokens = 0;

        for entry in self.index.entries() {
            let kind_name = match entry.chunk.kind {
                DocumentKind::Word => "Word",
                DocumentKind::Calc => "Calc",
                DocumentKind::Slides => "Slides",
                DocumentKind::Pdf => "Pdf",
                DocumentKind::PlainText => "PlainText",
            };
            *chunks_by_kind.entry(kind_name.to_string()).or_insert(0) += 1;
            total_tokens += entry.chunk.token_count_estimate;
        }

        IndexSummary {
            total_documents: self.index.total_documents(),
            total_chunks: self.index.total_chunks(),
            chunks_by_kind,
            total_estimated_tokens: total_tokens,
        }
    }
}
