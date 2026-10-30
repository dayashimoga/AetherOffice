use crate::chunker::{DocumentChunk, DocumentKind, DocumentLocation, SemanticChunker};
use crate::error::CopilotError;
use crate::vector_index::SearchResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Active user selection in the editor canvas (cursor, highlighted text, cell range).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveSelectionContext {
    pub kind: DocumentKind,
    pub selected_text: Option<String>,
    pub selected_range: Option<String>, // e.g. "B2:D10" or "Paragraph 4" or "Slide 3"
    pub location: Option<DocumentLocation>,
}

/// Assembled RAG context ready to be fed into LLM / on-device model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssembledContext {
    pub system_instructions: String,
    pub context_block: String,
    pub selected_chunks: Vec<DocumentChunk>,
    pub citation_mapping: Vec<(String, String)>, // ("[1]", chunk_id)
    pub total_tokens_estimated: usize,
    pub is_truncated: bool,
}

/// Configuration options for the context assembler.
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    pub max_token_budget: usize,
    pub max_chunks: usize,
    pub prioritize_active_selection: bool,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            max_token_budget: 1500,
            max_chunks: 5,
            prioritize_active_selection: true,
        }
    }
}

/// Assembles grounded context from vector search results and active canvas selections.
#[derive(Debug, Clone, Default)]
pub struct ContextAssembler {
    config: AssemblerConfig,
}

impl ContextAssembler {
    pub fn new(config: AssemblerConfig) -> Self {
        Self { config }
    }

    /// Assemble ranked chunks and active selection into formatted, budgeted context.
    pub fn assemble(
        &self,
        query: &str,
        search_results: &[SearchResult],
        active_selection: Option<&ActiveSelectionContext>,
        doc_metadata: &HashMap<String, String>,
    ) -> Result<AssembledContext, CopilotError> {
        let mut selected_chunks = Vec::new();
        let mut citation_mapping = Vec::new();
        let mut context_sections = Vec::new();
        let mut accumulated_tokens = 0;
        let mut is_truncated = false;

        // Base system instructions for strict grounding
        let system_instructions = "You are AetherCopilot, a privacy-first, zero-telemetry enterprise AI assistant.\
Follow these strict grounding rules:\
1. Base your answer strictly and exclusively on the provided context passages numbered [1], [2], etc.\
2. For every factual claim, append the citation marker, e.g. [1] or [2], directly after the sentence.\
3. Never fabricate or extrapolate numerical values, names, or financial figures not explicitly present in the context.\
4. If the context does not contain sufficient information to answer the question, state clearly that the document lacks the necessary data."
            .to_string();

        let system_tokens = SemanticChunker::estimate_tokens(&system_instructions);

        // Include active selection with top priority if present
        if self.config.prioritize_active_selection {
            if let Some(sel) = active_selection {
                if let Some(sel_text) = &sel.selected_text {
                    if !sel_text.trim().is_empty() {
                        let header = format!(
                            "[Active Selection: {:?}{}]",
                            sel.kind,
                            sel.selected_range
                                .as_ref()
                                .map(|r| format!(" ({})", r))
                                .unwrap_or_default()
                        );
                        let block = format!("{}\n{}", header, sel_text.trim());
                        let tokens = SemanticChunker::estimate_tokens(&block);

                        if accumulated_tokens + tokens <= self.config.max_token_budget {
                            context_sections.push(block);
                            accumulated_tokens += tokens;
                        }
                    }
                }
            }
        }

        // Add search result chunks in order of similarity
        let mut citation_idx = 1;
        for result in search_results {
            if selected_chunks.len() >= self.config.max_chunks {
                is_truncated = true;
                break;
            }

            let marker = format!("[{}]", citation_idx);
            let section_header = format!(
                "{} Source: {} | Structure: {}",
                marker, result.chunk.doc_id, result.chunk.structural_context
            );
            let formatted_entry = format!("{}\n{}", section_header, result.chunk.raw_text.trim());
            let entry_tokens = SemanticChunker::estimate_tokens(&formatted_entry);

            if accumulated_tokens + entry_tokens <= self.config.max_token_budget {
                selected_chunks.push(result.chunk.clone());
                citation_mapping.push((marker, result.chunk.id.clone()));
                context_sections.push(formatted_entry);
                accumulated_tokens += entry_tokens;
                citation_idx += 1;
            } else {
                is_truncated = true;
                break;
            }
        }

        // Add document metadata block if present
        if !doc_metadata.is_empty() {
            let mut meta_lines = Vec::new();
            for (k, v) in doc_metadata {
                meta_lines.push(format!("{}: {}", k, v));
            }
            let meta_block = format!("[Document Metadata]\n{}", meta_lines.join("\n"));
            let meta_tokens = SemanticChunker::estimate_tokens(&meta_block);
            if accumulated_tokens + meta_tokens <= self.config.max_token_budget {
                context_sections.push(meta_block);
                accumulated_tokens += meta_tokens;
            }
        }

        let context_block = if context_sections.is_empty() {
            format!("[User Query]\n{}", query)
        } else {
            format!(
                "[Retrieved Context Passages]\n{}\n\n[User Query]\n{}",
                context_sections.join("\n\n"),
                query
            )
        };

        Ok(AssembledContext {
            system_instructions,
            context_block,
            selected_chunks,
            citation_mapping,
            total_tokens_estimated: system_tokens + accumulated_tokens,
            is_truncated,
        })
    }
}
