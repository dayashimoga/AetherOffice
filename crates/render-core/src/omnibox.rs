//! Omnibox Command, File & Symbol Launcher Engine
//!
//! Complies strictly with GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Pure safe Rust with zero panics or unhandled error states.
//! - Sub-millisecond fuzzy ranking across commands, open files, and document symbols.
//! - Context-aware prefix routing: `>` (commands), `@` (files), `:` (symbols), `?` (search).

use crate::shell::{DesktopSessionManager, DocumentType};
use serde::{Deserialize, Serialize};

/// High-level category of an Omnibox result item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OmniboxCategory {
    Command,
    File,
    Symbol,
    Search,
}

/// An entry displayed and selectable within the Omnibox launcher palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmniboxItem {
    pub id: String,
    pub category: OmniboxCategory,
    pub title: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub icon: String,
    pub score: i32,
    pub payload: String,
}

/// Core fuzzy scoring and catalog aggregation engine for Omnibox (Ctrl+K).
pub struct OmniboxEngine;

impl OmniboxEngine {
    /// Computes a fuzzy matching score between `query` and `target`.
    /// Returns `None` if `query` is not a subsequence of `target`.
    pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
        let q = query.trim().to_lowercase();
        let t = target.trim().to_lowercase();

        if q.is_empty() {
            return Some(100);
        }

        if t == q {
            return Some(2000);
        }

        if t.starts_with(&q) {
            let bonus = 1000 - ((t.len() - q.len()) as i32 * 2);
            return Some(bonus.max(600));
        }

        // Subsequence check with boundary and consecutive matching bonuses
        let mut q_chars = q.chars().peekable();
        let mut score: i32 = 200;
        let mut prev_idx: Option<usize> = None;
        let mut consecutive_matches: i32 = 0;

        for (idx, ch) in t.char_indices() {
            if let Some(&qc) = q_chars.peek() {
                if ch == qc {
                    q_chars.next();

                    // Word boundary bonus (preceded by whitespace, underscore, hyphen, or dot)
                    if idx == 0 || t[..idx].ends_with(|c: char| !c.is_alphanumeric()) {
                        score += 60;
                    }

                    // Consecutive match bonus
                    if let Some(prev) = prev_idx {
                        if idx == prev + 1 {
                            consecutive_matches += 1;
                            score += 25 * consecutive_matches;
                        } else {
                            consecutive_matches = 0;
                            // Penalty for gaps
                            score -= (idx - prev) as i32 * 2;
                        }
                    } else {
                        consecutive_matches = 1;
                    }

                    prev_idx = Some(idx);
                }
            }
        }

        if q_chars.peek().is_none() {
            // Target contains full subsequence of query
            let length_penalty = (t.len().saturating_sub(q.len())) as i32;
            Some((score - length_penalty).max(10))
        } else {
            None
        }
    }

    /// Built-in command palette definitions.
    pub fn default_commands() -> Vec<OmniboxItem> {
        vec![
            OmniboxItem {
                id: "cmd-new-tab".to_string(),
                category: OmniboxCategory::Command,
                title: "New Document".to_string(),
                description: "Create a new document in a fresh workspace tab".to_string(),
                shortcut: Some("Ctrl+N".to_string()),
                icon: "file-plus".to_string(),
                score: 500,
                payload: "NewTab".to_string(),
            },
            OmniboxItem {
                id: "cmd-open-file".to_string(),
                category: OmniboxCategory::Command,
                title: "Open File...".to_string(),
                description: "Open an existing document from disk".to_string(),
                shortcut: Some("Ctrl+O".to_string()),
                icon: "folder-open".to_string(),
                score: 490,
                payload: "OpenFile".to_string(),
            },
            OmniboxItem {
                id: "cmd-save-doc".to_string(),
                category: OmniboxCategory::Command,
                title: "Save Document".to_string(),
                description: "Save active document changes to disk".to_string(),
                shortcut: Some("Ctrl+S".to_string()),
                icon: "save".to_string(),
                score: 480,
                payload: "SaveDocument".to_string(),
            },
            OmniboxItem {
                id: "cmd-save-as".to_string(),
                category: OmniboxCategory::Command,
                title: "Save As...".to_string(),
                description: "Save active document under a new name or location".to_string(),
                shortcut: Some("Ctrl+Shift+S".to_string()),
                icon: "save".to_string(),
                score: 470,
                payload: "SaveAs".to_string(),
            },
            OmniboxItem {
                id: "cmd-federated-search".to_string(),
                category: OmniboxCategory::Command,
                title: "Workspace Search & Replace".to_string(),
                description: "Search and replace text across all open workspace tabs".to_string(),
                shortcut: Some("Ctrl+Shift+F".to_string()),
                icon: "search".to_string(),
                score: 460,
                payload: "FederatedSearch".to_string(),
            },
            OmniboxItem {
                id: "cmd-close-tab".to_string(),
                category: OmniboxCategory::Command,
                title: "Close Active Tab".to_string(),
                description: "Close current document tab".to_string(),
                shortcut: Some("Ctrl+W".to_string()),
                icon: "x".to_string(),
                score: 450,
                payload: "CloseActiveTab".to_string(),
            },
            OmniboxItem {
                id: "cmd-theme-toggle".to_string(),
                category: OmniboxCategory::Command,
                title: "Toggle Theme (Dark / Light)".to_string(),
                description: "Switch between dark mode and light mode".to_string(),
                shortcut: None,
                icon: "sun-moon".to_string(),
                score: 440,
                payload: "ToggleTheme".to_string(),
            },
            OmniboxItem {
                id: "cmd-split-horizontal".to_string(),
                category: OmniboxCategory::Command,
                title: "Split View: Side by Side".to_string(),
                description: "Divide workspace into 2 horizontal split panes".to_string(),
                shortcut: None,
                icon: "columns".to_string(),
                score: 430,
                payload: "SplitHorizontal".to_string(),
            },
            OmniboxItem {
                id: "cmd-split-vertical".to_string(),
                category: OmniboxCategory::Command,
                title: "Split View: Stacked".to_string(),
                description: "Divide workspace into 2 vertical split panes".to_string(),
                shortcut: None,
                icon: "rows".to_string(),
                score: 420,
                payload: "SplitVertical".to_string(),
            },
            OmniboxItem {
                id: "cmd-split-single".to_string(),
                category: OmniboxCategory::Command,
                title: "Reset Single View".to_string(),
                description: "Collapse split panes back into a unified view".to_string(),
                shortcut: None,
                icon: "square".to_string(),
                score: 410,
                payload: "SplitSingle".to_string(),
            },
            OmniboxItem {
                id: "cmd-word-count".to_string(),
                category: OmniboxCategory::Command,
                title: "Word Count Statistics".to_string(),
                description: "View statistics on words, characters, and pages".to_string(),
                shortcut: None,
                icon: "file-text".to_string(),
                score: 400,
                payload: "WordCount".to_string(),
            },
            OmniboxItem {
                id: "cmd-copilot-toggle".to_string(),
                category: OmniboxCategory::Command,
                title: "Toggle AetherCopilot AI".to_string(),
                description: "Show or hide AI assistant sidebar".to_string(),
                shortcut: None,
                icon: "sparkles".to_string(),
                score: 390,
                payload: "ToggleCopilot".to_string(),
            },
            OmniboxItem {
                id: "cmd-undo".to_string(),
                category: OmniboxCategory::Command,
                title: "Undo".to_string(),
                description: "Revert last document modification".to_string(),
                shortcut: Some("Ctrl+Z".to_string()),
                icon: "rotate-ccw".to_string(),
                score: 380,
                payload: "Undo".to_string(),
            },
            OmniboxItem {
                id: "cmd-redo".to_string(),
                category: OmniboxCategory::Command,
                title: "Redo".to_string(),
                description: "Reapply previously reverted modification".to_string(),
                shortcut: Some("Ctrl+Y".to_string()),
                icon: "rotate-cw".to_string(),
                score: 370,
                payload: "Redo".to_string(),
            },
        ]
    }

    /// Collects open document tabs from the active workspace.
    pub fn collect_open_tabs(mgr: &DesktopSessionManager) -> Vec<OmniboxItem> {
        let mut items = Vec::new();

        for tab in &mgr.tab_bar.tabs {
            let doc_label = match tab.doc_type {
                DocumentType::Word => "Word Document",
                DocumentType::Excel => "Excel Spreadsheet",
                DocumentType::PowerPoint => "PowerPoint Deck",
                DocumentType::Pdf => "PDF Document",
            };
            let icon_str = match tab.doc_type {
                DocumentType::Word => "file-text",
                DocumentType::Excel => "table",
                DocumentType::PowerPoint => "presentation",
                DocumentType::Pdf => "file-pdf",
            };

            let status_note = if tab.is_dirty {
                "Unsaved changes"
            } else {
                "Saved"
            };

            items.push(OmniboxItem {
                id: format!("tab:{}", tab.id),
                category: OmniboxCategory::File,
                title: tab.title.clone(),
                description: format!("{doc_label} • {status_note}"),
                shortcut: None,
                icon: icon_str.to_string(),
                score: 600,
                payload: tab.id.clone(),
            });
        }

        items
    }

    /// Aggregates document structural symbols (headings, sheets, slides, ranges) from open sessions.
    pub fn collect_document_symbols(mgr: &DesktopSessionManager) -> Vec<OmniboxItem> {
        let mut items = Vec::new();

        for tab in &mgr.tab_bar.tabs {
            let session_id = &tab.id;
            let doc_title = &tab.title;

            if let Some(state) = mgr.office_states.get(session_id) {
                match tab.doc_type {
                    DocumentType::Word => {
                        for (idx, line) in state.word_content.lines().enumerate() {
                            let trimmed = line.trim();
                            if trimmed.starts_with('#')
                                || trimmed.starts_with("Heading")
                                || (trimmed.len() > 3 && trimmed.len() < 60 && idx < 5)
                            {
                                items.push(OmniboxItem {
                                    id: format!("sym:{session_id}:w:{idx}"),
                                    category: OmniboxCategory::Symbol,
                                    title: trimmed.trim_start_matches('#').trim().to_string(),
                                    description: format!(
                                        "Heading in \"{doc_title}\" (Line {})",
                                        idx + 1
                                    ),
                                    shortcut: None,
                                    icon: "heading".to_string(),
                                    score: 350,
                                    payload: format!("{session_id}:{idx}"),
                                });
                            }
                        }
                    }
                    DocumentType::Excel => {
                        for sheet in &state.excel_sheets {
                            items.push(OmniboxItem {
                                id: format!("sym:{session_id}:sheet:{sheet}"),
                                category: OmniboxCategory::Symbol,
                                title: format!("Sheet: {sheet}"),
                                description: format!("Worksheet in \"{doc_title}\""),
                                shortcut: None,
                                icon: "sheet".to_string(),
                                score: 380,
                                payload: format!("{session_id}:sheet:{sheet}"),
                            });
                        }
                    }
                    DocumentType::PowerPoint => {
                        for slide_num in &state.ppt_slides {
                            items.push(OmniboxItem {
                                id: format!("sym:{session_id}:slide:{slide_num}"),
                                category: OmniboxCategory::Symbol,
                                title: format!("Slide {slide_num}"),
                                description: format!("Slide in presentation \"{doc_title}\""),
                                shortcut: None,
                                icon: "presentation".to_string(),
                                score: 350,
                                payload: format!("{session_id}:slide:{slide_num}"),
                            });
                        }
                    }
                    DocumentType::Pdf => {
                        for page_num in 1..=10 {
                            items.push(OmniboxItem {
                                id: format!("sym:{session_id}:page:{page_num}"),
                                category: OmniboxCategory::Symbol,
                                title: format!("Page {page_num}"),
                                description: format!("Page in PDF document \"{doc_title}\""),
                                shortcut: None,
                                icon: "file-pdf".to_string(),
                                score: 300,
                                payload: format!("{session_id}:page:{page_num}"),
                            });
                        }
                    }
                }
            }
        }

        items
    }

    /// Queries the Omnibox catalog using context-aware prefix routing and fuzzy ranking.
    pub fn query(
        mgr: &DesktopSessionManager,
        raw_query: &str,
        max_results: usize,
    ) -> Vec<OmniboxItem> {
        let trimmed = raw_query.trim();

        // 1. Prefix routing analysis
        let (filter_cat, actual_query) = if let Some(q) = trimmed.strip_prefix('>') {
            (Some(OmniboxCategory::Command), q.trim())
        } else if let Some(q) = trimmed
            .strip_prefix('@')
            .or_else(|| trimmed.strip_prefix('#'))
        {
            (Some(OmniboxCategory::File), q.trim())
        } else if let Some(q) = trimmed
            .strip_prefix(':')
            .or_else(|| trimmed.strip_prefix('$'))
        {
            (Some(OmniboxCategory::Symbol), q.trim())
        } else if let Some(q) = trimmed.strip_prefix('?') {
            (Some(OmniboxCategory::Search), q.trim())
        } else {
            (None, trimmed)
        };

        // 2. Gather candidate entries
        let mut candidates = Vec::new();

        if filter_cat.is_none() || filter_cat == Some(OmniboxCategory::Command) {
            candidates.extend(Self::default_commands());
        }

        if filter_cat.is_none() || filter_cat == Some(OmniboxCategory::File) {
            candidates.extend(Self::collect_open_tabs(mgr));
        }

        if filter_cat.is_none() || filter_cat == Some(OmniboxCategory::Symbol) {
            candidates.extend(Self::collect_document_symbols(mgr));
        }

        // 3. Score and rank candidates
        let mut scored_items: Vec<OmniboxItem> = Vec::new();

        for mut item in candidates {
            if let Some(cat) = filter_cat {
                if item.category != cat {
                    continue;
                }
            }

            if actual_query.is_empty() {
                // Return default sorted catalog
                scored_items.push(item);
            } else {
                let title_score = Self::fuzzy_score(actual_query, &item.title);
                let desc_score = Self::fuzzy_score(actual_query, &item.description);

                let best_match = match (title_score, desc_score) {
                    (Some(t), Some(d)) => Some(t.max(d - 50)),
                    (Some(t), None) => Some(t),
                    (None, Some(d)) => Some(d - 50),
                    (None, None) => None,
                };

                if let Some(calculated_score) = best_match {
                    item.score = calculated_score;
                    scored_items.push(item);
                }
            }
        }

        // 4. If search prefix or general query with no command match, append Search item
        if filter_cat == Some(OmniboxCategory::Search)
            || (!actual_query.is_empty() && filter_cat.is_none())
        {
            let search_term = if actual_query.is_empty() {
                "workspace"
            } else {
                actual_query
            };
            scored_items.push(OmniboxItem {
                id: format!("action-search:{search_term}"),
                category: OmniboxCategory::Search,
                title: format!("Search workspace for \"{search_term}\""),
                description: "Perform federated keyword and full-text search across all open tabs"
                    .to_string(),
                shortcut: Some("Enter".to_string()),
                icon: "search".to_string(),
                score: if filter_cat == Some(OmniboxCategory::Search) {
                    1500
                } else {
                    150
                },
                payload: search_term.to_string(),
            });
        }

        // 5. Sort descending by score
        scored_items.sort_by_key(|a| std::cmp::Reverse(a.score));

        // 6. Limit to max_results
        scored_items.truncate(max_results);

        scored_items
    }
}
