//! Mail Merge Engine
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - ECMA-376 Part 1 §17.16.5.38 (MERGEFIELD) & standard chevrons `«FieldName»`.
//! - CSV data source parsing and multi-record batch document synthesis.

use crate::model::{BodyElement, Document, Paragraph, ParagraphChild, Run, RunContent};
use std::collections::{HashMap, HashSet};

/// Mail Merge and data substitution engine.
pub struct MailMergeEngine;

impl MailMergeEngine {
    /// Parses a CSV data source into column headers and a list of key-value records.
    pub fn parse_csv(csv_data: &str) -> (Vec<String>, Vec<HashMap<String, String>>) {
        let lines: Vec<&str> = csv_data
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let headers = Self::parse_csv_row(lines[0]);
        let mut records = Vec::new();

        for line in &lines[1..] {
            let row = Self::parse_csv_row(line);
            let mut record = HashMap::new();
            for (idx, header) in headers.iter().enumerate() {
                let val = row.get(idx).map(|s| s.as_str()).unwrap_or("");
                record.insert(header.clone(), val.to_string());
            }
            records.push(record);
        }

        (headers, records)
    }

    fn parse_csv_row(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut cur = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '"' {
                if in_quotes && chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else if c == ',' && !in_quotes {
                fields.push(cur.trim().to_string());
                cur.clear();
            } else {
                cur.push(c);
            }
        }
        fields.push(cur.trim().to_string());
        fields
    }

    /// Identifies all merge field names present in the document.
    pub fn find_merge_fields(doc: &Document) -> Vec<String> {
        let mut fields_set = HashSet::new();

        for elem in &doc.body.elements {
            if let BodyElement::Paragraph(ref p) = elem {
                for child in &p.children {
                    if let ParagraphChild::Run(ref r) = child {
                        for rc in &r.content {
                            if let RunContent::Text(ref s, _) = rc {
                                Self::extract_field_names(s, &mut fields_set);
                            }
                        }
                    }
                }
            }
        }

        let mut sorted: Vec<String> = fields_set.into_iter().collect();
        sorted.sort();
        sorted
    }

    fn extract_field_names(text: &str, set: &mut HashSet<String>) {
        // Match chevrons «FieldName»
        let mut remainder = text;
        while let Some(start) = remainder.find('«') {
            let after_start = &remainder[start.saturating_add('«'.len_utf8())..];
            if let Some(end) = after_start.find('»') {
                let name = after_start[..end].trim();
                if !name.is_empty() {
                    set.insert(name.to_string());
                }
                remainder = &after_start[end.saturating_add('»'.len_utf8())..];
            } else {
                break;
            }
        }

        // Match MERGEFIELD Name
        let upper = text.to_uppercase();
        let mut idx = 0;
        while let Some(found) = upper[idx..].find("MERGEFIELD") {
            let start = idx.saturating_add(found).saturating_add(10);
            let rest = &text[start..];
            let name = rest
                .split_whitespace()
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            if !name.is_empty() {
                set.insert(name.to_string());
            }
            idx = start;
        }
    }

    /// Merges a single data record into a clone of the template document.
    pub fn merge_record(template: &Document, record: &HashMap<String, String>) -> Document {
        let mut doc = template.clone();

        for elem in &mut doc.body.elements {
            if let BodyElement::Paragraph(ref mut p) = elem {
                for child in &mut p.children {
                    if let ParagraphChild::Run(ref mut r) = child {
                        for rc in &mut r.content {
                            if let RunContent::Text(ref mut s, _) = rc {
                                *s = Self::replace_placeholders(s, record);
                            }
                        }
                    }
                }
            }
        }

        doc
    }

    /// Merges multiple records into a single multi-section combined document.
    pub fn merge_batch(template: &Document, records: &[HashMap<String, String>]) -> Document {
        let mut combined = Document {
            namespaces: template.namespaces.clone(),
            ..Default::default()
        };

        for (idx, record) in records.iter().enumerate() {
            let single_doc = Self::merge_record(template, record);

            // Add elements
            for elem in single_doc.body.elements {
                combined.body.elements.push(elem);
            }

            // Append section / page break between records (except after the final record)
            if idx.saturating_add(1) < records.len() {
                let mut break_p = Paragraph::default();
                let mut break_r = Run::default();
                break_r
                    .content
                    .push(RunContent::Break(Some("page".to_string())));
                break_p.children.push(ParagraphChild::Run(break_r));
                combined.body.elements.push(BodyElement::Paragraph(break_p));
            }
        }

        combined
    }

    fn replace_placeholders(text: &str, record: &HashMap<String, String>) -> String {
        let mut out = text.to_string();

        for (key, val) in record {
            // Replace chevrons
            let chevron_target = format!("«{}»", key);
            out = out.replace(&chevron_target, val);

            // Replace MERGEFIELD
            let mergefield_target1 = format!("MERGEFIELD \"{}\"", key);
            out = out.replace(&mergefield_target1, val);

            let mergefield_target2 = format!("MERGEFIELD {}", key);
            out = out.replace(&mergefield_target2, val);
        }

        out
    }
}
