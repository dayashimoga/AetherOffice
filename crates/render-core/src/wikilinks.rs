#![forbid(unsafe_code)]
//! Bidirectional [[WikiLinks]] & Backlinks Engine (GEC-v7.0 Article I & III)
//!
//! Provides:
//! - Deterministic parsing of `[[Target]]`, `[[Target|Alias]]`, and `[[Target#Heading|Alias]]`.
//! - Bidirectional link graph indexing across Notes, Word documents, and PDFs.
//! - Linked references with contextual snippet extraction.
//! - Unlinked mentions discovery (identifying plain text matching document titles).
//! - Fuzzy autocomplete and suggestion scoring for inline wikilink insertion (`[[`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single parsed `[[WikiLink]]` occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiLink {
    pub target_doc: String,
    pub target_anchor: Option<String>,
    pub display_text: Option<String>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub raw_text: String,
}

/// Autocomplete suggestion for inline wikilink creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WikiLinkSuggestion {
    pub doc_id: String,
    pub title: String,
    pub anchor: Option<String>,
    pub preview: Option<String>,
    pub score: f64,
}

/// An incoming backlink reference to a document with surrounding context snippet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacklinkReference {
    pub source_doc_id: String,
    pub source_doc_title: String,
    pub target_anchor: Option<String>,
    pub display_text: Option<String>,
    pub snippet: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// An unlinked mention where a document title appears in text without `[[...]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlinkedMention {
    pub source_doc_id: String,
    pub source_doc_title: String,
    pub matched_text: String,
    pub snippet: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Summary statistics for the workspace wikilink index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiLinkIndexStats {
    pub total_documents: usize,
    pub total_wikilinks: usize,
    pub total_unique_targets: usize,
}

#[derive(Debug, Clone)]
struct IndexedDocInfo {
    title: String,
    content: String,
    links: Vec<WikiLink>,
}

/// Parser for `[[WikiLinks]]` with anchor and alias extraction.
pub struct WikiLinkParser;

impl WikiLinkParser {
    /// Parses all `[[...]]` occurrences from a text string.
    pub fn parse(text: &str) -> Vec<WikiLink> {
        let mut links = Vec::new();
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut idx = 0;

        while idx + 3 < len {
            if bytes[idx] == b'[' && bytes[idx + 1] == b'[' {
                let start = idx;
                let inner_start = idx + 2;
                idx = inner_start;

                let mut close_idx = None;
                while idx + 1 < len {
                    if bytes[idx] == b']' && bytes[idx + 1] == b']' {
                        close_idx = Some(idx);
                        break;
                    }
                    if bytes[idx] == b'\n' || bytes[idx] == b'\r' {
                        // WikiLinks do not span multiple lines
                        break;
                    }
                    idx += 1;
                }

                if let Some(end) = close_idx {
                    if let Ok(inner) = std::str::from_utf8(&bytes[inner_start..end]) {
                        let trimmed = inner.trim();
                        if !trimmed.is_empty() {
                            let (target_part, display_text) = match trimmed.find('|') {
                                Some(pipe_pos) => {
                                    let t = trimmed[..pipe_pos].trim();
                                    let a = trimmed[pipe_pos + 1..].trim();
                                    (
                                        t,
                                        if a.is_empty() {
                                            None
                                        } else {
                                            Some(a.to_string())
                                        },
                                    )
                                }
                                None => (trimmed, None),
                            };

                            let (target_doc, target_anchor) = match target_part.find('#') {
                                Some(hash_pos) => {
                                    let d = target_part[..hash_pos].trim();
                                    let anc = target_part[hash_pos + 1..].trim();
                                    (
                                        d.to_string(),
                                        if anc.is_empty() {
                                            None
                                        } else {
                                            Some(anc.to_string())
                                        },
                                    )
                                }
                                None => (target_part.to_string(), None),
                            };

                            let raw_text = match std::str::from_utf8(&bytes[start..end + 2]) {
                                Ok(r) => r.to_string(),
                                Err(_) => String::new(),
                            };

                            links.push(WikiLink {
                                target_doc,
                                target_anchor,
                                display_text,
                                start_offset: start,
                                end_offset: end + 2,
                                raw_text,
                            });
                        }
                    }
                    idx = end + 2;
                    continue;
                }
            }
            idx += 1;
        }

        links
    }

    /// Extracts a surrounding context snippet around a byte offset range.
    pub fn extract_snippet(
        text: &str,
        start_offset: usize,
        end_offset: usize,
        radius: usize,
    ) -> String {
        let char_indices: Vec<(usize, char)> = text.char_indices().collect();
        if char_indices.is_empty() {
            return String::new();
        }

        let start_char_idx = char_indices
            .iter()
            .position(|&(byte_pos, _)| byte_pos >= start_offset)
            .unwrap_or(0);
        let end_char_idx = char_indices
            .iter()
            .position(|&(byte_pos, _)| byte_pos >= end_offset)
            .unwrap_or(char_indices.len());

        let snippet_start = start_char_idx.saturating_sub(radius);
        let snippet_end = (end_char_idx + radius).min(char_indices.len());

        let mut snippet = String::new();
        if snippet_start > 0 {
            snippet.push_str("... ");
        }

        for &(_, ch) in &char_indices[snippet_start..snippet_end] {
            snippet.push(ch);
        }

        if snippet_end < char_indices.len() {
            snippet.push_str(" ...");
        }

        snippet
    }
}

/// In-memory bidirectional wikilink and backlink index.
#[derive(Debug, Clone, Default)]
pub struct WikiLinkIndex {
    // doc_id -> IndexedDocInfo
    docs: HashMap<String, IndexedDocInfo>,
    // normalized title -> doc_id
    title_to_doc_id: HashMap<String, String>,
    // normalized target -> list of BacklinkReference
    backlinks_by_target: HashMap<String, Vec<BacklinkReference>>,
}

impl WikiLinkIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes or re-indexes a document with plain text content.
    pub fn index_document(&mut self, doc_id: &str, title: &str, content: &str) {
        // Remove prior index entry if exists
        self.remove_document(doc_id);

        let normalized_title = title.trim().to_lowercase();
        self.title_to_doc_id
            .insert(normalized_title, doc_id.to_string());

        let parsed_links = WikiLinkParser::parse(content);

        // Populate backlinks for each parsed link
        for link in &parsed_links {
            let normalized_target = link.target_doc.trim().to_lowercase();
            let snippet =
                WikiLinkParser::extract_snippet(content, link.start_offset, link.end_offset, 40);

            let backlink_ref = BacklinkReference {
                source_doc_id: doc_id.to_string(),
                source_doc_title: title.to_string(),
                target_anchor: link.target_anchor.clone(),
                display_text: link.display_text.clone(),
                snippet,
                start_offset: link.start_offset,
                end_offset: link.end_offset,
            };

            self.backlinks_by_target
                .entry(normalized_target)
                .or_default()
                .push(backlink_ref);
        }

        self.docs.insert(
            doc_id.to_string(),
            IndexedDocInfo {
                title: title.to_string(),
                content: content.to_string(),
                links: parsed_links,
            },
        );
    }

    /// Removes a document and all its outgoing backlinks from the index.
    pub fn remove_document(&mut self, doc_id: &str) {
        if let Some(info) = self.docs.remove(doc_id) {
            let norm_title = info.title.trim().to_lowercase();
            if let Some(mapped_id) = self.title_to_doc_id.get(&norm_title) {
                if mapped_id == doc_id {
                    self.title_to_doc_id.remove(&norm_title);
                }
            }

            // Remove all backlinks originating from this document
            for backlinks in self.backlinks_by_target.values_mut() {
                backlinks.retain(|b| b.source_doc_id != doc_id);
            }

            // Prune empty backlink vectors
            self.backlinks_by_target.retain(|_, v| !v.is_empty());
        }
    }

    /// Returns all backlinks pointing to the specified document title or ID.
    pub fn get_backlinks(&self, target_title_or_id: &str) -> Vec<BacklinkReference> {
        let norm = target_title_or_id.trim().to_lowercase();

        // Check direct target lookup
        if let Some(list) = self.backlinks_by_target.get(&norm) {
            return list.clone();
        }

        // If target_title_or_id is a doc_id, resolve title
        if let Some(info) = self.docs.get(target_title_or_id) {
            let norm_title = info.title.trim().to_lowercase();
            if let Some(list) = self.backlinks_by_target.get(&norm_title) {
                return list.clone();
            }
        }

        Vec::new()
    }

    /// Returns all forward wikilinks parsed from the specified document.
    pub fn get_forward_links(&self, source_doc_id: &str) -> Vec<WikiLink> {
        self.docs
            .get(source_doc_id)
            .map(|info| info.links.clone())
            .unwrap_or_default()
    }

    /// Provides fuzzy target suggestions for autocomplete triggers.
    pub fn suggest_targets(&self, query: &str, limit: usize) -> Vec<WikiLinkSuggestion> {
        let q_norm = query.trim().to_lowercase();
        let mut suggestions = Vec::new();

        for (doc_id, info) in &self.docs {
            let title_norm = info.title.to_lowercase();

            let score = if q_norm.is_empty() {
                1.0
            } else if title_norm == q_norm {
                100.0
            } else if title_norm.starts_with(&q_norm) {
                80.0
            } else if title_norm.contains(&q_norm) {
                50.0
            } else {
                // Word boundary match
                let match_word = title_norm
                    .split_whitespace()
                    .any(|w| w.starts_with(&q_norm));
                if match_word {
                    60.0
                } else {
                    0.0
                }
            };

            if score > 0.0 {
                let preview = if info.content.len() > 120 {
                    let mut p = info.content.chars().take(120).collect::<String>();
                    p.push_str("...");
                    Some(p)
                } else if !info.content.is_empty() {
                    Some(info.content.clone())
                } else {
                    None
                };

                suggestions.push(WikiLinkSuggestion {
                    doc_id: doc_id.clone(),
                    title: info.title.clone(),
                    anchor: None,
                    preview,
                    score,
                });
            }
        }

        // Sort descending by score, then alphabetically by title
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.title.cmp(&b.title))
        });

        if suggestions.len() > limit {
            suggestions.truncate(limit);
        }

        suggestions
    }

    /// Discovers unlinked mentions of a document title in all other indexed documents.
    pub fn find_unlinked_mentions(&self, target_title: &str) -> Vec<UnlinkedMention> {
        let title_clean = target_title.trim();
        if title_clean.is_empty() {
            return Vec::new();
        }

        let target_doc_id = self
            .title_to_doc_id
            .get(&title_clean.to_lowercase())
            .cloned();
        let title_lower = title_clean.to_lowercase();

        let mut mentions = Vec::new();

        for (doc_id, info) in &self.docs {
            // Do not search inside the target document itself
            if Some(doc_id) == target_doc_id.as_ref() {
                continue;
            }

            // Exclude spans that are already part of a [[WikiLink]]
            let linked_spans: Vec<(usize, usize)> = info
                .links
                .iter()
                .map(|l| (l.start_offset, l.end_offset))
                .collect();

            let content = &info.content;
            let content_lower = content.to_lowercase();

            let mut search_start = 0;
            while let Some(found_idx) = content_lower[search_start..].find(&title_lower) {
                let abs_idx = search_start + found_idx;
                let end_idx = abs_idx + title_clean.len();

                // Check if inside any existing [[WikiLink]] span
                let is_inside_link = linked_spans
                    .iter()
                    .any(|&(s, e)| abs_idx >= s && abs_idx < e);

                if !is_inside_link {
                    // Check word boundary: character before and after shouldn't be alphanumeric
                    let char_before = if abs_idx > 0 {
                        content[..abs_idx].chars().last()
                    } else {
                        None
                    };

                    let char_after = content[end_idx..].chars().next();

                    let is_word_start = char_before.map(|c| !c.is_alphanumeric()).unwrap_or(true);
                    let is_word_end = char_after.map(|c| !c.is_alphanumeric()).unwrap_or(true);

                    if is_word_start && is_word_end {
                        let snippet =
                            WikiLinkParser::extract_snippet(content, abs_idx, end_idx, 30);
                        let matched_text = content[abs_idx..end_idx].to_string();

                        mentions.push(UnlinkedMention {
                            source_doc_id: doc_id.clone(),
                            source_doc_title: info.title.clone(),
                            matched_text,
                            snippet,
                            start_offset: abs_idx,
                            end_offset: end_idx,
                        });
                    }
                }

                search_start = abs_idx + title_clean.len();
                if search_start >= content.len() {
                    break;
                }
            }
        }

        mentions
    }

    /// Returns high-level statistics of the link index.
    pub fn stats(&self) -> WikiLinkIndexStats {
        let total_wikilinks = self.docs.values().map(|d| d.links.len()).sum();
        let total_unique_targets = self.backlinks_by_target.len();

        WikiLinkIndexStats {
            total_documents: self.docs.len(),
            total_wikilinks,
            total_unique_targets,
        }
    }
}
