use crate::chunker::{DocumentChunk, DocumentLocation};
use crate::error::CopilotError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A grounded citation referencing a specific source passage or cell range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub marker: String, // e.g. "[1]", "[2]"
    pub chunk_id: String,
    pub doc_id: String,
    pub location: DocumentLocation,
    pub snippet: String,
    pub confidence_score: f32,
    pub verified: bool,
}

/// Result of citation and hallucination verification against retrieved context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingVerificationResult {
    pub is_grounded: bool,
    pub verified_citations: Vec<Citation>,
    pub ungrounded_markers: Vec<String>,
    pub detected_hallucinations: Vec<String>,
    pub overall_grounding_score: f32,
}

/// Engine to ground generated AI claims against source chunks and detect hallucinations.
#[derive(Debug, Clone, Default)]
pub struct CitationGroundingEngine;

impl CitationGroundingEngine {
    pub fn new() -> Self {
        Self
    }

    /// Computes token overlap (Jaccard-like) between claim snippet and candidate source text.
    pub fn compute_overlap_score(claim: &str, source: &str) -> f32 {
        let clean_tokens = |text: &str| -> HashSet<String> {
            text.split_whitespace()
                .map(|w| {
                    w.to_lowercase()
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                })
                .filter(|w| w.len() > 2)
                .collect()
        };

        let claim_set = clean_tokens(claim);
        let source_set = clean_tokens(source);

        if claim_set.is_empty() {
            return 1.0;
        }

        let intersection = claim_set.intersection(&source_set).count();
        (intersection as f32) / (claim_set.len() as f32)
    }

    /// Extract numbers and currency figures from text for strict factual consistency checking.
    pub fn extract_numerical_entities(text: &str) -> HashSet<String> {
        let mut numbers = HashSet::new();
        for word in text.split_whitespace() {
            // Ignore citation markers like [1], [2], [10]
            if word.starts_with('[') && word.contains(']') {
                continue;
            }

            let filtered: String = word
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '%' || *c == '$')
                .collect();
            let clean = filtered.trim_end_matches(['.', ',']);
            if clean.chars().any(|c| c.is_ascii_digit()) {
                numbers.insert(clean.to_string());
                // Also insert normalized version without commas
                let no_commas: String = clean.chars().filter(|c| *c != ',').collect();
                numbers.insert(no_commas);
            }
        }
        numbers
    }

    /// Verifies that all citations and factual numbers in `generated_text` are strictly grounded in `chunks`.
    pub fn verify_grounding(
        &self,
        generated_text: &str,
        context_chunks: &[DocumentChunk],
    ) -> Result<GroundingVerificationResult, CopilotError> {
        if context_chunks.is_empty() {
            return Ok(GroundingVerificationResult {
                is_grounded: false,
                verified_citations: Vec::new(),
                ungrounded_markers: Vec::new(),
                detected_hallucinations: vec![
                    "No context chunks available for grounding".to_string()
                ],
                overall_grounding_score: 0.0,
            });
        }

        let mut verified_citations = Vec::new();
        let mut ungrounded_markers = Vec::new();
        let mut detected_hallucinations = Vec::new();

        // Parse citation markers like [1], [2] in text
        let mut found_indices = Vec::new();
        for (idx, _) in context_chunks.iter().enumerate() {
            let marker = format!("[{}]", idx + 1);
            if generated_text.contains(&marker) {
                found_indices.push((marker, idx));
            }
        }

        let mut score_sum = 0.0f32;
        let mut count = 0;

        for (marker, chunk_idx) in found_indices {
            if let Some(chunk) = context_chunks.get(chunk_idx) {
                // Find sentence or clause containing the marker
                let claim_context = Self::extract_sentence_with_marker(generated_text, &marker);
                let overlap = Self::compute_overlap_score(&claim_context, &chunk.raw_text);

                if overlap >= 0.25 {
                    verified_citations.push(Citation {
                        marker: marker.clone(),
                        chunk_id: chunk.id.clone(),
                        doc_id: chunk.doc_id.clone(),
                        location: chunk.location.clone(),
                        snippet: claim_context,
                        confidence_score: overlap,
                        verified: true,
                    });
                    score_sum += overlap;
                    count += 1;
                } else {
                    ungrounded_markers.push(marker);
                }
            } else {
                ungrounded_markers.push(marker);
            }
        }

        // Strict Hallucination Guard: Check numerical entities in generated text against context
        let all_context_text = context_chunks
            .iter()
            .map(|c| c.composite_text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let context_numbers = Self::extract_numerical_entities(&all_context_text);
        let generated_numbers = Self::extract_numerical_entities(generated_text);

        for gen_num in generated_numbers {
            if !context_numbers.contains(&gen_num) {
                detected_hallucinations.push(format!(
                    "Fabricated numerical claim detected: '{}' does not appear in source context",
                    gen_num
                ));
            }
        }

        let overall_grounding_score = if count > 0 {
            (score_sum / count as f32).clamp(0.0, 1.0)
        } else if detected_hallucinations.is_empty() && ungrounded_markers.is_empty() {
            1.0
        } else {
            0.0
        };

        let is_grounded = ungrounded_markers.is_empty()
            && detected_hallucinations.is_empty()
            && overall_grounding_score >= 0.3;

        Ok(GroundingVerificationResult {
            is_grounded,
            verified_citations,
            ungrounded_markers,
            detected_hallucinations,
            overall_grounding_score,
        })
    }

    fn extract_sentence_with_marker(text: &str, marker: &str) -> String {
        for sentence in text.split(&['.', '\n', '!', '?'][..]) {
            if sentence.contains(marker) {
                return sentence.replace(marker, "").trim().to_string();
            }
        }
        text.replace(marker, "").trim().to_string()
    }
}
