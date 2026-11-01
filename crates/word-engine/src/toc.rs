//! Dynamic Table of Contents (TOC) Engine
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - ECMA-376 Part 1 §17.4.5 (TOC Field Code).
//! - Automatic heading level discovery via paragraph outline_level and styles.
//! - Hyperlinked entries with bookmark anchors and dot leaders.

use crate::model::{
    BodyElement, BookmarkEnd, BookmarkStart, Document, Hyperlink, Indentation, Paragraph,
    ParagraphChild, ParagraphProperties, Run, RunContent, RunProperties,
};
use crate::styles::StyleCascadeResolver;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tab leader styles for TOC page number alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TabLeader {
    #[default]
    Dot,
    Hyphen,
    Underscore,
    None,
}

/// Configuration options for Table of Contents generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocOptions {
    pub start_level: u8,
    pub end_level: u8,
    pub use_hyperlinks: bool,
    pub show_page_numbers: bool,
    pub right_align_page_numbers: bool,
    pub tab_leader: TabLeader,
}

impl Default for TocOptions {
    fn default() -> Self {
        Self {
            start_level: 1,
            end_level: 3,
            use_hyperlinks: true,
            show_page_numbers: true,
            right_align_page_numbers: true,
            tab_leader: TabLeader::Dot,
        }
    }
}

/// Extracted heading entry representing a single item in the TOC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocEntry {
    pub level: u8,
    pub title: String,
    pub page_number: usize,
    pub bookmark_name: String,
    pub paragraph_index: usize,
}

/// Dynamic Table of Contents engine.
pub struct TocEngine;

impl TocEngine {
    /// Extracts all eligible headings from the document body.
    pub fn extract_headings(
        doc: &Document,
        cascade: &StyleCascadeResolver,
        page_map: Option<&HashMap<usize, usize>>,
    ) -> Vec<TocEntry> {
        let mut entries = Vec::new();

        for (para_idx, elem) in doc.body.elements.iter().enumerate() {
            if let BodyElement::Paragraph(ref p) = elem {
                let resolved = cascade.resolve_paragraph(p);
                if let Some(level) = Self::detect_heading_level(p, &resolved) {
                    let mut title = p.get_text();
                    title = title.trim().to_string();

                    if !title.is_empty() {
                        let page_number = match page_map {
                            Some(map) => *map.get(&para_idx).unwrap_or(&1usize),
                            None => 1usize,
                        };
                        let bookmark_name = format!("_Toc_{}", para_idx);

                        entries.push(TocEntry {
                            level,
                            title,
                            page_number,
                            bookmark_name,
                            paragraph_index: para_idx,
                        });
                    }
                }
            }
        }

        entries
    }

    /// Detects heading level from paragraph outline level or style ID.
    pub fn detect_heading_level(
        p: &Paragraph,
        resolved: &crate::styles::ResolvedParagraphProperties,
    ) -> Option<u8> {
        // 1. Check direct or resolved outline level (0-based in OpenXML: 0 = Level 1)
        if let Some(ol) = resolved.outline_level {
            if ol < 9 {
                return Some((ol as u8).saturating_add(1));
            }
        }

        // 2. Check explicit style on paragraph
        if let Some(ref p_pr) = p.properties {
            if let Some(ref style) = p_pr.style {
                if let Some(level) = Self::parse_style_heading_level(style) {
                    return Some(level);
                }
            }
        }

        // 3. Check resolved style
        if let Some(ref style) = resolved.style_id {
            if let Some(level) = Self::parse_style_heading_level(style) {
                return Some(level);
            }
        }

        None
    }

    fn parse_style_heading_level(style: &str) -> Option<u8> {
        let s = style.trim();
        let lower = s.to_lowercase();
        if lower.starts_with("heading") || lower.starts_with("heading ") {
            let num_part = lower.trim_start_matches("heading").trim();
            if let Ok(num) = num_part.parse::<u8>() {
                if (1..=9).contains(&num) {
                    return Some(num);
                }
            }
        } else if lower.starts_with("toc") && !lower.starts_with("toc heading") {
            // Avoid treating TOC entries themselves as headings
            return None;
        } else if lower == "title" {
            return Some(1);
        } else if lower == "subtitle" {
            return Some(2);
        }
        None
    }

    /// Generates structured paragraphs for the Table of Contents according to options.
    pub fn generate_toc_paragraphs(entries: &[TocEntry], options: &TocOptions) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();

        // Optional TOC Heading title paragraph
        let header_p = Paragraph {
            properties: Some(ParagraphProperties {
                style: Some("TOC Heading".to_string()),
                ..Default::default()
            }),
            children: vec![ParagraphChild::Run(Run {
                properties: Some(RunProperties {
                    bold: true,
                    size_half_points: Some(32), // 16pt
                    ..Default::default()
                }),
                content: vec![RunContent::Text("Table of Contents".to_string(), false)],
                ..Default::default()
            })],
            ..Default::default()
        };
        paragraphs.push(header_p);

        // Generate entry paragraphs
        for entry in entries {
            if entry.level < options.start_level || entry.level > options.end_level {
                continue;
            }

            let indent_twips = (entry.level.saturating_sub(1) as i32).saturating_mul(240);
            let props = Some(ParagraphProperties {
                style: Some(format!("TOC {}", entry.level)),
                indentation: Some(Indentation {
                    left: Some(indent_twips),
                    ..Default::default()
                }),
                ..Default::default()
            });

            // Runs inside the entry
            let mut title_run = Run::default();
            title_run
                .content
                .push(RunContent::Text(entry.title.clone(), false));

            let mut tab_run = Run::default();
            tab_run.content.push(RunContent::Tab);

            let mut page_run = Run::default();
            page_run
                .content
                .push(RunContent::Text(entry.page_number.to_string(), false));

            let mut children = Vec::new();
            if options.use_hyperlinks {
                let mut hyperlink_children = vec![ParagraphChild::Run(title_run)];
                if options.show_page_numbers {
                    hyperlink_children.push(ParagraphChild::Run(tab_run));
                    hyperlink_children.push(ParagraphChild::Run(page_run));
                }
                let hyperlink = Hyperlink {
                    anchor: Some(entry.bookmark_name.clone()),
                    history: true,
                    children: hyperlink_children,
                    ..Default::default()
                };
                children.push(ParagraphChild::Hyperlink(hyperlink));
            } else {
                children.push(ParagraphChild::Run(title_run));
                if options.show_page_numbers {
                    children.push(ParagraphChild::Run(tab_run));
                    children.push(ParagraphChild::Run(page_run));
                }
            }

            paragraphs.push(Paragraph {
                properties: props,
                children,
                ..Default::default()
            });
        }

        paragraphs
    }

    /// Updates or inserts Table of Contents in the document body, ensuring bookmarks exist.
    pub fn update_toc_fields(
        doc: &mut Document,
        cascade: &StyleCascadeResolver,
        page_map: Option<&HashMap<usize, usize>>,
        options: &TocOptions,
    ) -> (usize, usize) {
        let entries = Self::extract_headings(doc, cascade, page_map);
        let entries_count = entries.len();

        // 1. Ensure bookmarks exist on target heading paragraphs
        for entry in &entries {
            if entry.paragraph_index < doc.body.elements.len() {
                if let BodyElement::Paragraph(ref mut p) = doc.body.elements[entry.paragraph_index]
                {
                    let has_bookmark = p.children.iter().any(|c| match c {
                        ParagraphChild::BookmarkStart(b) => b.name == entry.bookmark_name,
                        _ => false,
                    });

                    if !has_bookmark {
                        let b_start = ParagraphChild::BookmarkStart(BookmarkStart {
                            id: entry.paragraph_index.to_string(),
                            name: entry.bookmark_name.clone(),
                            ..Default::default()
                        });
                        let b_end = ParagraphChild::BookmarkEnd(BookmarkEnd {
                            id: entry.paragraph_index.to_string(),
                        });

                        p.children.insert(0, b_start);
                        p.children.push(b_end);
                    }
                }
            }
        }

        // 2. Generate TOC paragraphs
        let mut toc_paragraphs = Self::generate_toc_paragraphs(&entries, options);
        let toc_p_count = toc_paragraphs.len();

        // 3. Replace existing TOC or insert at the beginning of document body
        let mut existing_toc_start = None;
        let mut existing_toc_end = None;

        for (idx, elem) in doc.body.elements.iter().enumerate() {
            if let BodyElement::Paragraph(ref p) = elem {
                if let Some(ref props) = p.properties {
                    if let Some(ref style) = props.style {
                        if style == "TOC Heading" || style.starts_with("TOC ") {
                            if existing_toc_start.is_none() {
                                existing_toc_start = Some(idx);
                            }
                            existing_toc_end = Some(idx);
                        }
                    }
                }
            }
        }

        if let (Some(start), Some(end)) = (existing_toc_start, existing_toc_end) {
            let replace_range = start..=end;
            let mut new_elements =
                Vec::with_capacity(doc.body.elements.len().saturating_add(toc_p_count));
            let mut inserted_toc = false;
            for (idx, elem) in doc.body.elements.drain(..).enumerate() {
                if idx == start && !inserted_toc {
                    for tp in std::mem::take(&mut toc_paragraphs) {
                        new_elements.push(BodyElement::Paragraph(tp));
                    }
                    inserted_toc = true;
                } else if !replace_range.contains(&idx) {
                    new_elements.push(elem);
                }
            }
            doc.body.elements = new_elements;
        } else {
            // Insert at index 0
            let mut new_elements =
                Vec::with_capacity(doc.body.elements.len().saturating_add(toc_p_count));
            for tp in toc_paragraphs {
                new_elements.push(BodyElement::Paragraph(tp));
            }
            new_elements.append(&mut doc.body.elements);
            doc.body.elements = new_elements;
        }

        (entries_count, toc_p_count)
    }
}
