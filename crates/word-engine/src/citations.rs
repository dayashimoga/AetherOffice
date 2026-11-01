//! Bibliography & Citation Engine
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - ECMA-376 Part 1 §17.4.4 (CITATION field) & §17.16 (Bibliography).
//! - BibTeX parser supporting books, articles, conference papers, and reports.
//! - APA, IEEE, MLA, and Chicago citation styles.

use crate::model::{
    BodyElement, Document, Paragraph, ParagraphChild, ParagraphProperties, Run, RunContent,
    RunProperties,
};
use serde::{Deserialize, Serialize};

/// Type of bibliographic source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SourceType {
    Book,
    Article,
    InProceedings,
    TechReport,
    Web,
    #[default]
    Misc,
}

/// Bibliographic citation source entry.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CitationSource {
    pub id: String,
    pub source_type: SourceType,
    pub authors: Vec<String>,
    pub title: String,
    pub year: Option<u32>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
}

/// Supported citation and bibliography styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CitationStyle {
    #[default]
    Apa,
    Ieee,
    Mla,
    Chicago,
}

/// Citation and Bibliography formatting engine.
pub struct CitationEngine;

impl CitationEngine {
    /// Parses BibTeX formatted string into a vector of CitationSource objects.
    pub fn parse_bibtex(bibtex: &str) -> Vec<CitationSource> {
        let mut sources = Vec::new();
        let mut chars = bibtex.chars().peekable();

        while let Some(&ch) = chars.peek() {
            if ch == '@' {
                chars.next();
                let mut type_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '{' || c == '(' || c.is_whitespace() {
                        break;
                    }
                    type_str.push(c);
                    chars.next();
                }

                // Consume opening delimiter
                while let Some(&c) = chars.peek() {
                    if c == '{' || c == '(' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }

                // Parse citation key (ID)
                let mut id = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ',' || c == '}' || c == ')' {
                        break;
                    }
                    id.push(c);
                    chars.next();
                }
                id = id.trim().to_string();

                if let Some(&',') = chars.peek() {
                    chars.next();
                }

                let source_type = match type_str.to_lowercase().as_str() {
                    "article" => SourceType::Article,
                    "book" | "booklet" => SourceType::Book,
                    "inproceedings" | "conference" | "proceedings" => SourceType::InProceedings,
                    "techreport" | "report" => SourceType::TechReport,
                    "web" | "online" | "misc" => SourceType::Web,
                    _ => SourceType::Misc,
                };

                let mut source = CitationSource {
                    id,
                    source_type,
                    ..Default::default()
                };

                // Parse key-value field pairs
                while let Some(&c) = chars.peek() {
                    if c == '}' || c == ')' {
                        chars.next();
                        break;
                    }
                    if c == ',' || c.is_whitespace() {
                        chars.next();
                        continue;
                    }

                    // Read field name
                    let mut field_name = String::new();
                    while let Some(&f) = chars.peek() {
                        if f == '=' || f.is_whitespace() {
                            break;
                        }
                        field_name.push(f);
                        chars.next();
                    }

                    // Skip to '='
                    while let Some(&f) = chars.peek() {
                        if f == '=' {
                            chars.next();
                            break;
                        }
                        chars.next();
                    }

                    // Skip whitespace
                    while let Some(&f) = chars.peek() {
                        if !f.is_whitespace() {
                            break;
                        }
                        chars.next();
                    }

                    // Read field value (braced or quoted or bare)
                    let mut field_value = String::new();
                    if let Some(&'{') = chars.peek() {
                        chars.next();
                        let mut brace_depth = 1;
                        while let Some(&v) = chars.peek() {
                            if v == '{' {
                                brace_depth += 1;
                            } else if v == '}' {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    chars.next();
                                    break;
                                }
                            }
                            field_value.push(v);
                            chars.next();
                        }
                    } else if let Some(&'"') = chars.peek() {
                        chars.next();
                        while let Some(&v) = chars.peek() {
                            if v == '"' {
                                chars.next();
                                break;
                            }
                            field_value.push(v);
                            chars.next();
                        }
                    } else {
                        while let Some(&v) = chars.peek() {
                            if v == ',' || v == '}' || v == ')' || v == '\n' {
                                break;
                            }
                            field_value.push(v);
                            chars.next();
                        }
                    }

                    field_value = field_value.trim().to_string();
                    let field_lower = field_name.trim().to_lowercase();

                    match field_lower.as_str() {
                        "author" | "authors" => {
                            source.authors = field_value
                                .split(" and ")
                                .map(|a| a.trim().to_string())
                                .filter(|a| !a.is_empty())
                                .collect();
                        }
                        "title" => source.title = field_value,
                        "year" => {
                            if let Ok(y) = field_value.parse::<u32>() {
                                source.year = Some(y);
                            }
                        }
                        "journal" => source.journal = Some(field_value),
                        "volume" => source.volume = Some(field_value),
                        "number" | "issue" => source.issue = Some(field_value),
                        "pages" => source.pages = Some(field_value),
                        "publisher" => source.publisher = Some(field_value),
                        "doi" => source.doi = Some(field_value),
                        "url" => source.url = Some(field_value),
                        _ => {}
                    }
                }

                if !source.id.is_empty() {
                    sources.push(source);
                }
            } else {
                chars.next();
            }
        }

        sources
    }

    /// Formats an in-text inline citation string according to active style.
    pub fn format_inline(
        sources: &[&CitationSource],
        style: CitationStyle,
        page: Option<&str>,
        order_index: Option<usize>,
    ) -> String {
        if sources.is_empty() {
            return String::new();
        }

        match style {
            CitationStyle::Ieee => {
                let idx = order_index.unwrap_or(1);
                if let Some(p) = page {
                    format!("[{}, p. {}]", idx, p)
                } else {
                    format!("[{}]", idx)
                }
            }
            CitationStyle::Apa => {
                let mut parts = Vec::new();
                for s in sources {
                    let author_part = Self::format_authors_short(&s.authors);
                    let year_part = s
                        .year
                        .map(|y| y.to_string())
                        .unwrap_or_else(|| "n.d.".to_string());
                    if let Some(p) = page {
                        parts.push(format!("{}, {}, p. {}", author_part, year_part, p));
                    } else {
                        parts.push(format!("{}, {}", author_part, year_part));
                    }
                }
                format!("({})", parts.join("; "))
            }
            CitationStyle::Mla => {
                let mut parts = Vec::new();
                for s in sources {
                    let author_part = Self::format_authors_short(&s.authors);
                    if let Some(p) = page {
                        parts.push(format!("{} {}", author_part, p));
                    } else {
                        parts.push(author_part);
                    }
                }
                format!("({})", parts.join("; "))
            }
            CitationStyle::Chicago => {
                let mut parts = Vec::new();
                for s in sources {
                    let author_part = Self::format_authors_short(&s.authors);
                    let year_part = s
                        .year
                        .map(|y| y.to_string())
                        .unwrap_or_else(|| "n.d.".to_string());
                    if let Some(p) = page {
                        parts.push(format!("{} {}, {}", author_part, year_part, p));
                    } else {
                        parts.push(format!("{} {}", author_part, year_part));
                    }
                }
                format!("({})", parts.join("; "))
            }
        }
    }

    fn format_authors_short(authors: &[String]) -> String {
        if authors.is_empty() {
            return "Anonymous".to_string();
        }
        let first = Self::get_last_name(&authors[0]);
        match authors.len() {
            1 => first,
            2 => format!("{} & {}", first, Self::get_last_name(&authors[1])),
            _ => format!("{} et al.", first),
        }
    }

    fn get_last_name(name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.contains(',') {
            // Already "Last, First"
            return trimmed
                .split(',')
                .next()
                .unwrap_or(trimmed)
                .trim()
                .to_string();
        }
        // "First Last"
        trimmed
            .split_whitespace()
            .last()
            .unwrap_or(trimmed)
            .to_string()
    }

    /// Formats a complete bibliographic reference entry for a single source.
    pub fn format_bibliography_entry(
        source: &CitationSource,
        style: CitationStyle,
        index: usize,
    ) -> String {
        match style {
            CitationStyle::Ieee => {
                let mut authors = String::new();
                for (i, a) in source.authors.iter().enumerate() {
                    if i > 0 {
                        authors.push_str(", ");
                    }
                    authors.push_str(a);
                }
                let mut entry = format!("[{}] {}, \"{}\"", index, authors, source.title);
                if let Some(ref j) = source.journal {
                    entry.push_str(&format!(", {}", j));
                }
                if let Some(ref v) = source.volume {
                    entry.push_str(&format!(", vol. {}", v));
                }
                if let Some(ref iss) = source.issue {
                    entry.push_str(&format!(", no. {}", iss));
                }
                if let Some(ref p) = source.pages {
                    entry.push_str(&format!(", pp. {}", p));
                }
                if let Some(y) = source.year {
                    entry.push_str(&format!(", {}.", y));
                } else {
                    entry.push('.');
                }
                entry
            }
            CitationStyle::Apa => {
                let authors = if source.authors.is_empty() {
                    "Anonymous".to_string()
                } else {
                    source.authors.join(", ")
                };
                let year = source
                    .year
                    .map(|y| format!("({}).", y))
                    .unwrap_or_else(|| "(n.d.).".to_string());
                let mut entry = format!("{} {} {}.", authors, year, source.title);
                if let Some(ref j) = source.journal {
                    entry.push_str(&format!(" {}", j));
                    if let Some(ref v) = source.volume {
                        entry.push_str(&format!(", {}", v));
                        if let Some(ref iss) = source.issue {
                            entry.push_str(&format!("({})", iss));
                        }
                    }
                    if let Some(ref p) = source.pages {
                        entry.push_str(&format!(", {}.", p));
                    } else {
                        entry.push('.');
                    }
                }
                if let Some(ref doi) = source.doi {
                    entry.push_str(&format!(" https://doi.org/{}", doi));
                }
                entry
            }
            CitationStyle::Mla => {
                let authors = if source.authors.is_empty() {
                    "Anonymous.".to_string()
                } else {
                    format!("{}.", source.authors.join(", "))
                };
                let mut entry = format!("{} \"{}.\"", authors, source.title);
                if let Some(ref j) = source.journal {
                    entry.push_str(&format!(" {},", j));
                }
                if let Some(ref v) = source.volume {
                    entry.push_str(&format!(" vol. {},", v));
                }
                if let Some(ref iss) = source.issue {
                    entry.push_str(&format!(" no. {},", iss));
                }
                if let Some(y) = source.year {
                    entry.push_str(&format!(" {},", y));
                }
                if let Some(ref p) = source.pages {
                    entry.push_str(&format!(" pp. {}.", p));
                } else {
                    entry.push('.');
                }
                entry
            }
            CitationStyle::Chicago => {
                let authors = if source.authors.is_empty() {
                    "Anonymous".to_string()
                } else {
                    source.authors.join(", ")
                };
                let year = source
                    .year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "n.d.".to_string());
                let mut entry = format!("{}. {}. \"{}.\"", authors, year, source.title);
                if let Some(ref j) = source.journal {
                    entry.push_str(&format!(" {}", j));
                }
                if let Some(ref v) = source.volume {
                    entry.push_str(&format!(" {}", v));
                    if let Some(ref iss) = source.issue {
                        entry.push_str(&format!(" ({})", iss));
                    }
                }
                if let Some(ref p) = source.pages {
                    entry.push_str(&format!(": {}.", p));
                } else {
                    entry.push('.');
                }
                entry
            }
        }
    }

    /// Generates structured paragraphs for the complete Bibliography / References section.
    pub fn generate_bibliography_paragraphs(
        sources: &[CitationSource],
        style: CitationStyle,
    ) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();

        // Heading paragraph
        let heading_title = match style {
            CitationStyle::Ieee => "References",
            CitationStyle::Mla => "Works Cited",
            CitationStyle::Apa | CitationStyle::Chicago => "Bibliography",
        };
        let h_p = Paragraph {
            properties: Some(ParagraphProperties {
                style: Some("Heading 1".to_string()),
                ..Default::default()
            }),
            children: vec![ParagraphChild::Run(Run {
                properties: Some(RunProperties {
                    bold: true,
                    size_half_points: Some(32),
                    ..Default::default()
                }),
                content: vec![RunContent::Text(heading_title.to_string(), false)],
                ..Default::default()
            })],
            ..Default::default()
        };
        paragraphs.push(h_p);

        // Individual entry paragraphs
        for (idx, source) in sources.iter().enumerate() {
            let entry_text = Self::format_bibliography_entry(source, style, idx.saturating_add(1));
            let p = Paragraph {
                properties: Some(ParagraphProperties {
                    style: Some("Bibliography".to_string()),
                    ..Default::default()
                }),
                children: vec![ParagraphChild::Run(Run {
                    content: vec![RunContent::Text(entry_text, false)],
                    ..Default::default()
                })],
                ..Default::default()
            };
            paragraphs.push(p);
        }

        paragraphs
    }

    /// Inserts an in-text citation into the target paragraph.
    pub fn insert_citation(
        doc: &mut Document,
        para_idx: usize,
        source_id: &str,
        style: CitationStyle,
        catalog: &[CitationSource],
        order_index: Option<usize>,
    ) -> bool {
        let source = catalog.iter().find(|s| s.id == source_id);
        if let Some(src) = source {
            if para_idx < doc.body.elements.len() {
                if let BodyElement::Paragraph(ref mut p) = doc.body.elements[para_idx] {
                    let citation_text = Self::format_inline(&[src], style, None, order_index);
                    let mut run = Run::default();
                    run.content
                        .push(RunContent::Text(format!(" {}", citation_text), false));
                    p.children.push(ParagraphChild::Run(run));
                    return true;
                }
            }
        }
        false
    }
}
