//! Revision Tracking (w:ins, w:del, w:rPrChange, w:pPrChange) & Redline Engine
//!
//! Complies strictly with GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero unwrap or expect calls in non-test code.
//! - ECMA-376 Part 1 §17.13 tracked revisions.
//! - Complete redline modes: SimpleMarkup, AllMarkup, NoMarkup, Original.
//! - Individual and batch revision acceptance/rejection with zero data loss.

use crate::model::{BodyElement, Document, Paragraph, ParagraphChild, RunContent, Table};
use serde::{Deserialize, Serialize};

/// Type of tracked revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RevisionType {
    Insertion,
    Deletion,
    RunPropertiesChange,
    ParagraphPropertiesChange,
}

/// Active display mode for tracked changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RevisionViewMode {
    /// Shows vertical change bars in the margin, deletions hidden, insertions normal.
    SimpleMarkup,
    /// Shows all visual markup: insertions underlined, deletions struck through in author color.
    #[default]
    AllMarkup,
    /// Shows the document with all revisions accepted.
    NoMarkup,
    /// Shows the document with all revisions rejected.
    Original,
}

/// Metadata describing an individual tracked revision in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionItem {
    pub id: String,
    pub author: String,
    pub date: String,
    pub revision_type: RevisionType,
    pub text_preview: String,
    pub paragraph_index: usize,
}

/// Deterministic semantic color palette assigning consistent colors per author.
pub struct AuthorColorPalette;

impl AuthorColorPalette {
    pub const PALETTE: &'static [&'static str] = &[
        "#0284c7", // Sky blue
        "#dc2626", // Red
        "#16a34a", // Green
        "#9333ea", // Purple
        "#ea580c", // Orange
        "#0d9488", // Teal
        "#d97706", // Amber
        "#db2777", // Pink
    ];

    /// Returns a deterministic hex color string for an author name.
    pub fn get_color_for_author(author: &str) -> &'static str {
        if author.is_empty() {
            return Self::PALETTE[0];
        }
        let hash: usize = author
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        Self::PALETTE[hash % Self::PALETTE.len()]
    }
}

/// Core engine managing revision enumeration, acceptance, and rejection.
pub struct RevisionEngine;

impl RevisionEngine {
    /// Enumerates all tracked revisions in the document.
    pub fn list_revisions(doc: &Document) -> Vec<RevisionItem> {
        let mut items = Vec::new();
        let mut para_idx = 0;

        for elem in &doc.body.elements {
            match elem {
                BodyElement::Paragraph(p) => {
                    Self::collect_paragraph_revisions(p, para_idx, &mut items);
                    para_idx += 1;
                }
                BodyElement::Table(tbl) => {
                    for row in &tbl.rows {
                        for cell in &row.cells {
                            for cell_elem in &cell.elements {
                                if let BodyElement::Paragraph(p) = cell_elem {
                                    Self::collect_paragraph_revisions(p, para_idx, &mut items);
                                    para_idx += 1;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        items
    }

    fn collect_paragraph_revisions(p: &Paragraph, para_idx: usize, items: &mut Vec<RevisionItem>) {
        // Paragraph properties change
        if let Some(ref p_pr) = p.properties {
            if let Some(ref p_change) = p_pr.p_pr_change {
                items.push(RevisionItem {
                    id: p_change.id.clone(),
                    author: p_change.author.clone(),
                    date: p_change.date.clone(),
                    revision_type: RevisionType::ParagraphPropertiesChange,
                    text_preview: "Paragraph format revision".to_string(),
                    paragraph_index: para_idx,
                });
            }
        }

        // Children revisions
        for child in &p.children {
            match child {
                ParagraphChild::Inserted(ref ins) => {
                    let mut preview = String::new();
                    for c in &ins.children {
                        c.write_text(&mut preview);
                    }
                    items.push(RevisionItem {
                        id: ins.id.clone(),
                        author: ins.author.clone(),
                        date: ins.date.clone(),
                        revision_type: RevisionType::Insertion,
                        text_preview: preview,
                        paragraph_index: para_idx,
                    });
                }
                ParagraphChild::Deleted(ref del) => {
                    let mut preview = String::new();
                    for c in &del.children {
                        if let ParagraphChild::Run(ref r) = c {
                            for rc in &r.content {
                                match rc {
                                    RunContent::DeletedText(ref s, _) => preview.push_str(s),
                                    RunContent::Text(ref s, _) => preview.push_str(s),
                                    _ => {}
                                }
                            }
                        }
                    }
                    items.push(RevisionItem {
                        id: del.id.clone(),
                        author: del.author.clone(),
                        date: del.date.clone(),
                        revision_type: RevisionType::Deletion,
                        text_preview: preview,
                        paragraph_index: para_idx,
                    });
                }
                ParagraphChild::Run(ref r) => {
                    if let Some(ref r_pr) = r.properties {
                        if let Some(ref r_change) = r_pr.r_pr_change {
                            items.push(RevisionItem {
                                id: r_change.id.clone(),
                                author: r_change.author.clone(),
                                date: r_change.date.clone(),
                                revision_type: RevisionType::RunPropertiesChange,
                                text_preview: r.get_text(),
                                paragraph_index: para_idx,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Accepts an individual revision by ID across all body elements.
    pub fn accept_revision(doc: &mut Document, revision_id: &str) -> bool {
        let mut modified = false;
        for elem in &mut doc.body.elements {
            match elem {
                BodyElement::Paragraph(p) => {
                    if Self::accept_in_paragraph(p, revision_id) {
                        modified = true;
                    }
                }
                BodyElement::Table(tbl) => {
                    if Self::accept_in_table(tbl, revision_id) {
                        modified = true;
                    }
                }
                _ => {}
            }
        }
        modified
    }

    fn accept_in_paragraph(p: &mut Paragraph, revision_id: &str) -> bool {
        let mut modified = false;

        // 1. Paragraph property change: accept new props by dropping p_pr_change
        if let Some(ref mut p_pr) = p.properties {
            if let Some(ref change) = p_pr.p_pr_change {
                if change.id == revision_id {
                    p_pr.p_pr_change = None;
                    modified = true;
                }
            }
        }

        // 2. Run property changes
        for child in &mut p.children {
            if let ParagraphChild::Run(ref mut r) = child {
                if let Some(ref mut r_pr) = r.properties {
                    if let Some(ref change) = r_pr.r_pr_change {
                        if change.id == revision_id {
                            r_pr.r_pr_change = None;
                            modified = true;
                        }
                    }
                }
            }
        }

        // 3. Insertions (unwrap into normal children) and Deletions (remove)
        let mut new_children = Vec::with_capacity(p.children.len());
        for child in std::mem::take(&mut p.children) {
            match child {
                ParagraphChild::Inserted(ins) if ins.id == revision_id => {
                    new_children.extend(ins.children);
                    modified = true;
                }
                ParagraphChild::Deleted(del) if del.id == revision_id => {
                    // Accepted deletion: permanently remove deleted children
                    modified = true;
                }
                other => new_children.push(other),
            }
        }
        p.children = new_children;

        modified
    }

    fn accept_in_table(tbl: &mut Table, revision_id: &str) -> bool {
        let mut modified = false;
        for row in &mut tbl.rows {
            for cell in &mut row.cells {
                for cell_elem in &mut cell.elements {
                    if let BodyElement::Paragraph(p) = cell_elem {
                        if Self::accept_in_paragraph(p, revision_id) {
                            modified = true;
                        }
                    }
                }
            }
        }
        modified
    }

    /// Rejects an individual revision by ID across all body elements.
    pub fn reject_revision(doc: &mut Document, revision_id: &str) -> bool {
        let mut modified = false;
        for elem in &mut doc.body.elements {
            match elem {
                BodyElement::Paragraph(p) => {
                    if Self::reject_in_paragraph(p, revision_id) {
                        modified = true;
                    }
                }
                BodyElement::Table(tbl) => {
                    if Self::reject_in_table(tbl, revision_id) {
                        modified = true;
                    }
                }
                _ => {}
            }
        }
        modified
    }

    fn reject_in_paragraph(p: &mut Paragraph, revision_id: &str) -> bool {
        let mut modified = false;

        // 1. Paragraph property change: restore previous properties
        if let Some(ref mut p_pr) = p.properties {
            if let Some(ref mut change) = p_pr.p_pr_change {
                if change.id == revision_id {
                    if let Some(prev) = change.previous_properties.take() {
                        *p_pr = *prev;
                    } else {
                        p_pr.p_pr_change = None;
                    }
                    modified = true;
                }
            }
        }

        // 2. Run property changes: restore previous properties
        for child in &mut p.children {
            if let ParagraphChild::Run(ref mut r) = child {
                if let Some(ref mut r_pr) = r.properties {
                    if let Some(ref mut change) = r_pr.r_pr_change {
                        if change.id == revision_id {
                            if let Some(prev) = change.previous_properties.take() {
                                *r_pr = *prev;
                            } else {
                                r_pr.r_pr_change = None;
                            }
                            modified = true;
                        }
                    }
                }
            }
        }

        // 3. Insertions (remove) and Deletions (unwrap back into active runs)
        let mut new_children = Vec::with_capacity(p.children.len());
        for child in std::mem::take(&mut p.children) {
            match child {
                ParagraphChild::Inserted(ins) if ins.id == revision_id => {
                    // Rejected insertion: discard inserted children
                    modified = true;
                }
                ParagraphChild::Deleted(mut del) if del.id == revision_id => {
                    // Rejected deletion: restore deleted text back as normal runs
                    for c in &mut del.children {
                        if let ParagraphChild::Run(ref mut r) = c {
                            for rc in &mut r.content {
                                if let RunContent::DeletedText(ref s, preserve) = rc {
                                    *rc = RunContent::Text(s.clone(), *preserve);
                                }
                            }
                        }
                    }
                    new_children.extend(del.children);
                    modified = true;
                }
                other => new_children.push(other),
            }
        }
        p.children = new_children;

        modified
    }

    fn reject_in_table(tbl: &mut Table, revision_id: &str) -> bool {
        let mut modified = false;
        for row in &mut tbl.rows {
            for cell in &mut row.cells {
                for cell_elem in &mut cell.elements {
                    if let BodyElement::Paragraph(p) = cell_elem {
                        if Self::reject_in_paragraph(p, revision_id) {
                            modified = true;
                        }
                    }
                }
            }
        }
        modified
    }

    /// Accepts all tracked revisions across the entire document in a single O(N) pass.
    pub fn accept_all_revisions(doc: &mut Document) -> usize {
        let revisions = Self::list_revisions(doc);
        let count = revisions.len();
        if count == 0 {
            return 0;
        }

        for elem in &mut doc.body.elements {
            match elem {
                BodyElement::Paragraph(p) => {
                    Self::accept_all_in_paragraph(p);
                }
                BodyElement::Table(tbl) => {
                    for row in &mut tbl.rows {
                        for cell in &mut row.cells {
                            for cell_elem in &mut cell.elements {
                                if let BodyElement::Paragraph(p) = cell_elem {
                                    Self::accept_all_in_paragraph(p);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        count
    }

    fn accept_all_in_paragraph(p: &mut Paragraph) {
        if let Some(ref mut p_pr) = p.properties {
            p_pr.p_pr_change = None;
        }

        for child in &mut p.children {
            if let ParagraphChild::Run(ref mut r) = child {
                if let Some(ref mut r_pr) = r.properties {
                    r_pr.r_pr_change = None;
                }
            }
        }

        let mut new_children = Vec::with_capacity(p.children.len());
        for child in std::mem::take(&mut p.children) {
            match child {
                ParagraphChild::Inserted(ins) => {
                    new_children.extend(ins.children);
                }
                ParagraphChild::Deleted(_) => {
                    // Accepted deletion: permanently remove
                }
                other => new_children.push(other),
            }
        }
        p.children = new_children;
    }

    /// Rejects all tracked revisions across the entire document in a single O(N) pass.
    pub fn reject_all_revisions(doc: &mut Document) -> usize {
        let revisions = Self::list_revisions(doc);
        let count = revisions.len();
        if count == 0 {
            return 0;
        }

        for elem in &mut doc.body.elements {
            match elem {
                BodyElement::Paragraph(p) => {
                    Self::reject_all_in_paragraph(p);
                }
                BodyElement::Table(tbl) => {
                    for row in &mut tbl.rows {
                        for cell in &mut row.cells {
                            for cell_elem in &mut cell.elements {
                                if let BodyElement::Paragraph(p) = cell_elem {
                                    Self::reject_all_in_paragraph(p);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        count
    }

    fn reject_all_in_paragraph(p: &mut Paragraph) {
        if let Some(ref mut p_pr) = p.properties {
            if let Some(ref mut change) = p_pr.p_pr_change {
                if let Some(prev) = change.previous_properties.take() {
                    *p_pr = *prev;
                } else {
                    p_pr.p_pr_change = None;
                }
            }
        }

        for child in &mut p.children {
            if let ParagraphChild::Run(ref mut r) = child {
                if let Some(ref mut r_pr) = r.properties {
                    if let Some(ref mut change) = r_pr.r_pr_change {
                        if let Some(prev) = change.previous_properties.take() {
                            *r_pr = *prev;
                        } else {
                            r_pr.r_pr_change = None;
                        }
                    }
                }
            }
        }

        let mut new_children = Vec::with_capacity(p.children.len());
        for child in std::mem::take(&mut p.children) {
            match child {
                ParagraphChild::Inserted(_) => {
                    // Rejected insertion: discard
                }
                ParagraphChild::Deleted(mut del) => {
                    // Rejected deletion: restore deleted text as normal runs
                    for c in &mut del.children {
                        if let ParagraphChild::Run(ref mut r) = c {
                            for rc in &mut r.content {
                                if let RunContent::DeletedText(ref s, preserve) = rc {
                                    *rc = RunContent::Text(s.clone(), *preserve);
                                }
                            }
                        }
                    }
                    new_children.extend(del.children);
                }
                other => new_children.push(other),
            }
        }
        p.children = new_children;
    }

    /// Returns counts for `(insertions, deletions, run_formatting, para_formatting)`.
    pub fn get_revision_summary(doc: &Document) -> (usize, usize, usize, usize) {
        let mut ins = 0;
        let mut del = 0;
        let mut r_pr = 0;
        let mut p_pr = 0;

        for r in Self::list_revisions(doc) {
            match r.revision_type {
                RevisionType::Insertion => ins += 1,
                RevisionType::Deletion => del += 1,
                RevisionType::RunPropertiesChange => r_pr += 1,
                RevisionType::ParagraphPropertiesChange => p_pr += 1,
            }
        }

        (ins, del, r_pr, p_pr)
    }
}
