use crate::error::CopilotError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Office document category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentKind {
    Word,
    Calc,
    Slides,
    Pdf,
    PlainText,
}

/// Structural location within a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentLocation {
    Word {
        section: usize,
        paragraph_index: usize,
        heading_level: Option<u8>,
        heading_title: Option<String>,
    },
    Calc {
        sheet_name: String,
        row_start: u32,
        row_end: u32,
        col_start: u16,
        col_end: u16,
        a1_reference: String,
    },
    Slides {
        slide_number: usize,
        slide_title: Option<String>,
        shape_id: Option<String>,
        is_notes: bool,
    },
    Pdf {
        page_number: usize,
        line_start: usize,
        line_end: usize,
    },
    Generic {
        offset: usize,
        length: usize,
    },
}

/// A semantic chunk extracted from a document with full structural context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub doc_id: String,
    pub kind: DocumentKind,
    pub location: DocumentLocation,
    pub structural_context: String,
    pub raw_text: String,
    pub composite_text: String,
    pub token_count_estimate: usize,
    pub metadata: HashMap<String, String>,
}

/// Input representation for a Word processing paragraph.
#[derive(Debug, Clone)]
pub struct WordParagraphInput {
    pub section: usize,
    pub paragraph_index: usize,
    pub heading_level: Option<u8>,
    pub text: String,
}

/// Input representation for a Slide shape.
#[derive(Debug, Clone)]
pub struct SlideShapeInput {
    pub shape_id: String,
    pub shape_name: String,
    pub text: String,
}

/// Input representation for a Presentation slide.
#[derive(Debug, Clone)]
pub struct SlideInput {
    pub slide_number: usize,
    pub slide_title: Option<String>,
    pub shapes: Vec<SlideShapeInput>,
    pub notes: Option<String>,
}

/// Configuration options for the semantic document chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub max_tokens_per_chunk: usize,
    pub overlap_tokens: usize,
    pub min_chunk_tokens: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_chunk: 200,
            overlap_tokens: 30,
            min_chunk_tokens: 15,
        }
    }
}

/// Hierarchical document chunker preserving structural context across Office document types.
#[derive(Debug, Clone, Default)]
pub struct SemanticChunker {
    config: ChunkerConfig,
}

impl SemanticChunker {
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Fast token count heuristic based on whitespace and punctuation.
    pub fn estimate_tokens(text: &str) -> usize {
        if text.trim().is_empty() {
            return 0;
        }
        let words = text.split_whitespace().count();
        let punctuation_count = text
            .chars()
            .filter(|c| matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']'))
            .count();
        // Standard rule of thumb: ~1.2 words per token + punctuation
        words + (punctuation_count / 2) + 1
    }

    /// Cleanly split text into sentence units respecting common abbreviations and numerical decimals.
    pub fn split_sentences(text: &str) -> Vec<String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut sentences = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = trimmed.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];
            current.push(c);

            if matches!(c, '.' | '?' | '!') {
                // Check if this is an abbreviation (e.g., e.g., i.e., vs., Dr., Mr.) or decimal number
                let is_decimal = if i > 0 && i + 1 < len {
                    chars[i - 1].is_ascii_digit() && chars[i + 1].is_ascii_digit()
                } else {
                    false
                };

                let is_abbr = if c == '.' && current.len() >= 3 {
                    let tail = current.trim_end_matches('.');
                    tail.ends_with("Dr")
                        || tail.ends_with("Mr")
                        || tail.ends_with("Ms")
                        || tail.ends_with("Prof")
                        || tail.ends_with("vs")
                        || tail.ends_with("eg")
                        || tail.ends_with("ie")
                        || tail.ends_with("Inc")
                        || tail.ends_with("Corp")
                        || tail.ends_with("Ltd")
                } else {
                    false
                };

                if !is_decimal && !is_abbr {
                    // Check if following character is whitespace or end of string
                    let is_sentence_end = if i + 1 < len {
                        chars[i + 1].is_whitespace()
                    } else {
                        true
                    };

                    if is_sentence_end {
                        let candidate = current.trim().to_string();
                        if !candidate.is_empty() {
                            sentences.push(candidate);
                        }
                        current.clear();
                    }
                }
            }
            i += 1;
        }

        let remainder = current.trim().to_string();
        if !remainder.is_empty() {
            sentences.push(remainder);
        }

        sentences
    }

    /// Chunks a Word document, preserving section and heading hierarchies.
    pub fn chunk_word_document(
        &self,
        doc_id: &str,
        doc_title: &str,
        paragraphs: &[WordParagraphInput],
    ) -> Result<Vec<DocumentChunk>, CopilotError> {
        if paragraphs.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut active_headings: Vec<(u8, String)> = Vec::new();
        let mut current_sentences: Vec<String> = Vec::new();
        let mut current_tokens = 0;
        let mut current_start_para = 0;
        let mut current_section = 0;
        let mut chunk_counter = 0;

        for p in paragraphs {
            let p_text = p.text.trim();
            if p_text.is_empty() {
                continue;
            }

            // Update heading hierarchy
            if let Some(level) = p.heading_level {
                active_headings.retain(|(lvl, _)| *lvl < level);
                active_headings.push((level, p_text.to_string()));
            }

            let sentences = Self::split_sentences(p_text);
            if sentences.is_empty() {
                continue;
            }

            if current_sentences.is_empty() {
                current_start_para = p.paragraph_index;
                current_section = p.section;
            }

            for s in sentences {
                let s_tokens = Self::estimate_tokens(&s);

                if current_tokens + s_tokens > self.config.max_tokens_per_chunk
                    && !current_sentences.is_empty()
                {
                    // Emit chunk
                    chunk_counter += 1;
                    let heading_path = active_headings
                        .iter()
                        .map(|(lvl, h)| format!("H{}: {}", lvl, h))
                        .collect::<Vec<_>>()
                        .join(" > ");
                    let structural_ctx = format!(
                        "[Document: {} > Section {} > {} > Para {}]",
                        doc_title,
                        current_section,
                        if heading_path.is_empty() {
                            "Body".to_string()
                        } else {
                            heading_path.clone()
                        },
                        current_start_para
                    );

                    let raw_text = current_sentences.join(" ");
                    let composite_text = format!("{}\n{}", structural_ctx, raw_text);
                    let mut meta = HashMap::new();
                    meta.insert("doc_title".to_string(), doc_title.to_string());
                    meta.insert(
                        "heading".to_string(),
                        active_headings
                            .last()
                            .map(|(_, h)| h.clone())
                            .unwrap_or_default(),
                    );

                    chunks.push(DocumentChunk {
                        id: format!("{}_chunk_{:04}", doc_id, chunk_counter),
                        doc_id: doc_id.to_string(),
                        kind: DocumentKind::Word,
                        location: DocumentLocation::Word {
                            section: current_section,
                            paragraph_index: current_start_para,
                            heading_level: active_headings.last().map(|(lvl, _)| *lvl),
                            heading_title: active_headings.last().map(|(_, h)| h.clone()),
                        },
                        structural_context: structural_ctx,
                        raw_text,
                        composite_text,
                        token_count_estimate: current_tokens,
                        metadata: meta,
                    });

                    // Prepare overlap sentences
                    let mut overlap_sentences = Vec::new();
                    let mut overlap_tokens = 0;
                    for prev_s in current_sentences.iter().rev() {
                        let t = Self::estimate_tokens(prev_s);
                        if overlap_tokens + t <= self.config.overlap_tokens {
                            overlap_tokens += t;
                            overlap_sentences.push(prev_s.clone());
                        } else {
                            break;
                        }
                    }
                    overlap_sentences.reverse();

                    current_sentences = overlap_sentences;
                    current_tokens = overlap_tokens;
                    current_start_para = p.paragraph_index;
                    current_section = p.section;
                }

                current_sentences.push(s);
                current_tokens += s_tokens;
            }
        }

        // Emit final chunk
        if !current_sentences.is_empty() {
            chunk_counter += 1;
            let heading_path = active_headings
                .iter()
                .map(|(lvl, h)| format!("H{}: {}", lvl, h))
                .collect::<Vec<_>>()
                .join(" > ");
            let structural_ctx = format!(
                "[Document: {} > Section {} > {} > Para {}]",
                doc_title,
                current_section,
                if heading_path.is_empty() {
                    "Body".to_string()
                } else {
                    heading_path
                },
                current_start_para
            );

            let raw_text = current_sentences.join(" ");
            let composite_text = format!("{}\n{}", structural_ctx, raw_text);
            let mut meta = HashMap::new();
            meta.insert("doc_title".to_string(), doc_title.to_string());
            meta.insert(
                "heading".to_string(),
                active_headings
                    .last()
                    .map(|(_, h)| h.clone())
                    .unwrap_or_default(),
            );

            chunks.push(DocumentChunk {
                id: format!("{}_chunk_{:04}", doc_id, chunk_counter),
                doc_id: doc_id.to_string(),
                kind: DocumentKind::Word,
                location: DocumentLocation::Word {
                    section: current_section,
                    paragraph_index: current_start_para,
                    heading_level: active_headings.last().map(|(lvl, _)| *lvl),
                    heading_title: active_headings.last().map(|(_, h)| h.clone()),
                },
                structural_context: structural_ctx,
                raw_text,
                composite_text,
                token_count_estimate: current_tokens,
                metadata: meta,
            });
        }

        Ok(chunks)
    }

    /// Chunks a Calc spreadsheet sheet into contextual row groups with column header binding.
    pub fn chunk_calc_sheet(
        &self,
        doc_id: &str,
        sheet_name: &str,
        headers: &[String],
        rows: &[Vec<String>],
        rows_per_chunk: usize,
    ) -> Result<Vec<DocumentChunk>, CopilotError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let chunk_size = if rows_per_chunk == 0 {
            5
        } else {
            rows_per_chunk
        };
        let mut chunks = Vec::new();
        let total_rows = rows.len();
        let col_count = headers.len();

        let mut row_idx = 0;
        let mut chunk_counter = 0;

        while row_idx < total_rows {
            let end_idx = (row_idx + chunk_size).min(total_rows);
            chunk_counter += 1;

            let row_start_1based = (row_idx + 2) as u32; // Assuming row 1 is header
            let row_end_1based = (end_idx + 1) as u32;

            let col_letter_end = if col_count <= 26 {
                let c = (b'A' + (col_count.saturating_sub(1) as u8)) as char;
                c.to_string()
            } else {
                "Z".to_string()
            };
            let a1_ref = format!("A{}:{}{}", row_start_1based, col_letter_end, row_end_1based);

            let structural_ctx = format!(
                "[Spreadsheet: {} > Sheet: {} > Range: {}]",
                doc_id, sheet_name, a1_ref
            );

            // Format rows with column bindings
            let mut row_descriptions = Vec::new();
            for (r_offset, row) in rows[row_idx..end_idx].iter().enumerate() {
                let actual_row_num = row_idx + r_offset + 2;
                let mut cell_pairs = Vec::new();
                for (c_idx, cell_val) in row.iter().enumerate() {
                    let header_name = headers.get(c_idx).map(|s| s.as_str()).unwrap_or("Column");
                    if !cell_val.trim().is_empty() {
                        cell_pairs.push(format!("{}: {}", header_name, cell_val.trim()));
                    }
                }
                if !cell_pairs.is_empty() {
                    row_descriptions.push(format!(
                        "Row {}: {}",
                        actual_row_num,
                        cell_pairs.join(", ")
                    ));
                }
            }

            let raw_text = row_descriptions.join("\n");
            let composite_text = format!("{}\n{}", structural_ctx, raw_text);
            let tokens = Self::estimate_tokens(&composite_text);

            let mut meta = HashMap::new();
            meta.insert("sheet_name".to_string(), sheet_name.to_string());
            meta.insert("a1_reference".to_string(), a1_ref.clone());

            chunks.push(DocumentChunk {
                id: format!("{}_{}_chunk_{:04}", doc_id, sheet_name, chunk_counter),
                doc_id: doc_id.to_string(),
                kind: DocumentKind::Calc,
                location: DocumentLocation::Calc {
                    sheet_name: sheet_name.to_string(),
                    row_start: row_start_1based,
                    row_end: row_end_1based,
                    col_start: 1,
                    col_end: col_count as u16,
                    a1_reference: a1_ref,
                },
                structural_context: structural_ctx,
                raw_text,
                composite_text,
                token_count_estimate: tokens,
                metadata: meta,
            });

            row_idx = end_idx;
        }

        Ok(chunks)
    }

    /// Chunks a Presentation, creating chunks per slide with title, shapes, and speaker notes.
    pub fn chunk_slides(
        &self,
        doc_id: &str,
        presentation_title: &str,
        slides: &[SlideInput],
    ) -> Result<Vec<DocumentChunk>, CopilotError> {
        if slides.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut chunk_counter = 0;

        for slide in slides {
            let slide_title = slide.slide_title.as_deref().unwrap_or("Untitled Slide");

            // Body content
            let mut shape_lines = Vec::new();
            for shape in &slide.shapes {
                let text = shape.text.trim();
                if !text.is_empty() {
                    shape_lines.push(format!("{}: {}", shape.shape_name, text));
                }
            }

            if !shape_lines.is_empty() {
                chunk_counter += 1;
                let structural_ctx = format!(
                    "[Presentation: {} > Slide {}: {}]",
                    presentation_title, slide.slide_number, slide_title
                );
                let raw_text = shape_lines.join("\n");
                let composite_text = format!("{}\n{}", structural_ctx, raw_text);
                let tokens = Self::estimate_tokens(&composite_text);

                let mut meta = HashMap::new();
                meta.insert("slide_title".to_string(), slide_title.to_string());
                meta.insert("slide_number".to_string(), slide.slide_number.to_string());

                chunks.push(DocumentChunk {
                    id: format!(
                        "{}_slide_{:03}_chunk_{:04}",
                        doc_id, slide.slide_number, chunk_counter
                    ),
                    doc_id: doc_id.to_string(),
                    kind: DocumentKind::Slides,
                    location: DocumentLocation::Slides {
                        slide_number: slide.slide_number,
                        slide_title: Some(slide_title.to_string()),
                        shape_id: None,
                        is_notes: false,
                    },
                    structural_context: structural_ctx,
                    raw_text,
                    composite_text,
                    token_count_estimate: tokens,
                    metadata: meta,
                });
            }

            // Speaker notes if present
            if let Some(notes) = &slide.notes {
                let notes_trimmed = notes.trim();
                if !notes_trimmed.is_empty() {
                    chunk_counter += 1;
                    let structural_ctx = format!(
                        "[Presentation: {} > Slide {}: {} > Speaker Notes]",
                        presentation_title, slide.slide_number, slide_title
                    );
                    let raw_text = notes_trimmed.to_string();
                    let composite_text = format!("{}\n{}", structural_ctx, raw_text);
                    let tokens = Self::estimate_tokens(&composite_text);

                    let mut meta = HashMap::new();
                    meta.insert("slide_title".to_string(), slide_title.to_string());
                    meta.insert("slide_number".to_string(), slide.slide_number.to_string());
                    meta.insert("is_speaker_notes".to_string(), "true".to_string());

                    chunks.push(DocumentChunk {
                        id: format!(
                            "{}_slide_{:03}_notes_{:04}",
                            doc_id, slide.slide_number, chunk_counter
                        ),
                        doc_id: doc_id.to_string(),
                        kind: DocumentKind::Slides,
                        location: DocumentLocation::Slides {
                            slide_number: slide.slide_number,
                            slide_title: Some(slide_title.to_string()),
                            shape_id: None,
                            is_notes: true,
                        },
                        structural_context: structural_ctx,
                        raw_text,
                        composite_text,
                        token_count_estimate: tokens,
                        metadata: meta,
                    });
                }
            }
        }

        Ok(chunks)
    }

    /// Generic plain text chunker.
    pub fn chunk_plain_text(
        &self,
        doc_id: &str,
        doc_title: &str,
        text: &str,
    ) -> Result<Vec<DocumentChunk>, CopilotError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let sentences = Self::split_sentences(trimmed);
        if sentences.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut current_sentences: Vec<String> = Vec::new();
        let mut current_tokens = 0;
        let mut chunk_counter = 0;
        let mut char_offset = 0;

        for s in sentences {
            let s_tokens = Self::estimate_tokens(&s);
            if current_tokens + s_tokens > self.config.max_tokens_per_chunk
                && !current_sentences.is_empty()
            {
                chunk_counter += 1;
                let raw_text = current_sentences.join(" ");
                let length = raw_text.len();
                let structural_ctx = format!("[Document: {} > Offset: {}]", doc_title, char_offset);
                let composite_text = format!("{}\n{}", structural_ctx, raw_text);

                let mut meta = HashMap::new();
                meta.insert("doc_title".to_string(), doc_title.to_string());

                chunks.push(DocumentChunk {
                    id: format!("{}_chunk_{:04}", doc_id, chunk_counter),
                    doc_id: doc_id.to_string(),
                    kind: DocumentKind::PlainText,
                    location: DocumentLocation::Generic {
                        offset: char_offset,
                        length,
                    },
                    structural_context: structural_ctx,
                    raw_text,
                    composite_text,
                    token_count_estimate: current_tokens,
                    metadata: meta,
                });

                char_offset += length;
                // Overlap
                let mut overlap_sentences: Vec<String> = Vec::new();
                let mut overlap_tokens = 0;
                for prev_s in current_sentences.iter().rev() {
                    let t = Self::estimate_tokens(prev_s);
                    if overlap_tokens + t <= self.config.overlap_tokens {
                        overlap_tokens += t;
                        overlap_sentences.push(prev_s.clone());
                    } else {
                        break;
                    }
                }
                overlap_sentences.reverse();
                current_sentences = overlap_sentences;
                current_tokens = overlap_tokens;
            }

            current_sentences.push(s);
            current_tokens += s_tokens;
        }

        if !current_sentences.is_empty() {
            chunk_counter += 1;
            let raw_text = current_sentences.join(" ");
            let length = raw_text.len();
            let structural_ctx = format!("[Document: {} > Offset: {}]", doc_title, char_offset);
            let composite_text = format!("{}\n{}", structural_ctx, raw_text);

            let mut meta = HashMap::new();
            meta.insert("doc_title".to_string(), doc_title.to_string());

            chunks.push(DocumentChunk {
                id: format!("{}_chunk_{:04}", doc_id, chunk_counter),
                doc_id: doc_id.to_string(),
                kind: DocumentKind::PlainText,
                location: DocumentLocation::Generic {
                    offset: char_offset,
                    length,
                },
                structural_context: structural_ctx,
                raw_text,
                composite_text,
                token_count_estimate: current_tokens,
                metadata: meta,
            });
        }

        Ok(chunks)
    }
}
