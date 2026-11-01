//! Threaded Comments & Resolved History Engine
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - ECMA-376 Part 1 §17.13.4 and ISO/IEC 29500-1 §17.13.4.
//! - Anchored comment ranges: w:commentRangeStart, w:commentRangeEnd, w:commentReference.
//! - Threaded replies via parent comment relationships and resolution history.

use crate::model::{BodyElement, Document, ParagraphChild};
use crate::sub_documents::{Comment, CommentsDocument};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A threaded comment tree node containing a root comment, nested replies,
/// and associated text anchor metadata in the document body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentThread {
    pub root: Comment,
    pub replies: Vec<Comment>,
    pub anchor_text: Option<String>,
    pub paragraph_index: Option<usize>,
    pub is_resolved: bool,
}

impl CommentThread {
    pub fn id(&self) -> &str {
        &self.root.id
    }

    pub fn total_comments(&self) -> usize {
        1usize.saturating_add(self.replies.len())
    }
}

/// Core engine for manipulating anchored threaded comments and resolution history.
pub struct CommentEngine;

impl CommentEngine {
    /// Extracts all threaded comments from the document and comments sub-document,
    /// linking root comments with their anchored text in the document body.
    pub fn list_threads(doc: &Document, comments_doc: &CommentsDocument) -> Vec<CommentThread> {
        let mut anchors: HashMap<String, (String, usize)> = HashMap::new();
        let mut active_ranges: HashMap<String, (String, usize)> = HashMap::new();

        for (para_idx, elem) in doc.body.elements.iter().enumerate() {
            if let BodyElement::Paragraph(ref p) = elem {
                for child in &p.children {
                    match child {
                        ParagraphChild::CommentRangeStart(ref id) => {
                            active_ranges.insert(id.clone(), (String::new(), para_idx));
                        }
                        ParagraphChild::Run(ref r) => {
                            let mut run_text = String::new();
                            r.write_text(&mut run_text);
                            for (text, _) in active_ranges.values_mut() {
                                text.push_str(&run_text);
                            }
                        }
                        ParagraphChild::CommentRangeEnd(ref id) => {
                            if let Some((text, idx)) = active_ranges.remove(id) {
                                anchors.insert(id.clone(), (text, idx));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut roots = Vec::new();
        let mut replies_map: HashMap<String, Vec<Comment>> = HashMap::new();

        for c in &comments_doc.comments {
            if let Some(ref parent_id) = c.parent_id {
                replies_map
                    .entry(parent_id.clone())
                    .or_default()
                    .push(c.clone());
            } else {
                roots.push(c.clone());
            }
        }

        let mut threads = Vec::new();
        for root in roots {
            let (anchor_text, paragraph_index) = match anchors.get(&root.id) {
                Some((text, idx)) => (Some(text.clone()), Some(*idx)),
                None => (None, None),
            };
            let replies = replies_map.remove(&root.id).unwrap_or_default();
            let is_resolved =
                root.is_resolved || (!replies.is_empty() && replies.iter().all(|r| r.is_resolved));

            threads.push(CommentThread {
                root,
                replies,
                anchor_text,
                paragraph_index,
                is_resolved,
            });
        }

        threads
    }

    /// Adds a new anchored root comment to the document body and comments sub-document.
    pub fn add_comment(
        doc: &mut Document,
        comments_doc: &mut CommentsDocument,
        para_idx: usize,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> String {
        let comment_id = Self::next_id(comments_doc);

        if para_idx < doc.body.elements.len() {
            if let BodyElement::Paragraph(ref mut p) = doc.body.elements[para_idx] {
                let mut new_children = Vec::with_capacity(p.children.len().saturating_add(3));
                new_children.push(ParagraphChild::CommentRangeStart(comment_id.clone()));
                new_children.append(&mut p.children);
                new_children.push(ParagraphChild::CommentRangeEnd(comment_id.clone()));
                new_children.push(ParagraphChild::CommentReference(comment_id.clone()));
                p.children = new_children;
            }
        }

        let mut comment = Comment::new(&comment_id, author, text);
        comment.initials = initials.map(|s| s.to_string());
        comment.date = Some("2026-11-01T12:00:00Z".to_string());
        comment.parent_id = None;
        comment.is_resolved = false;
        comments_doc.comments.push(comment);

        comment_id
    }

    /// Adds a threaded reply to an existing comment.
    pub fn add_reply(
        comments_doc: &mut CommentsDocument,
        parent_id: &str,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> Option<String> {
        let parent_exists = comments_doc.comments.iter().any(|c| c.id == parent_id);
        if !parent_exists {
            return None;
        }

        let reply_id = Self::next_id(comments_doc);
        let mut reply = Comment::new(&reply_id, author, text);
        reply.initials = initials.map(|s| s.to_string());
        reply.date = Some("2026-11-01T12:00:00Z".to_string());
        reply.parent_id = Some(parent_id.to_string());
        reply.is_resolved = false;
        comments_doc.comments.push(reply);

        Some(reply_id)
    }

    /// Resolves or reopens a comment thread.
    pub fn set_resolved(
        comments_doc: &mut CommentsDocument,
        thread_id: &str,
        resolved: bool,
    ) -> bool {
        let mut found = false;
        for c in &mut comments_doc.comments {
            if c.id == thread_id || c.parent_id.as_deref() == Some(thread_id) {
                c.is_resolved = resolved;
                found = true;
            }
        }
        found
    }

    /// Deletes an entire comment thread (root + all replies) and removes its anchors from the document.
    pub fn delete_thread(
        doc: &mut Document,
        comments_doc: &mut CommentsDocument,
        thread_id: &str,
    ) -> bool {
        for elem in &mut doc.body.elements {
            if let BodyElement::Paragraph(ref mut p) = elem {
                p.children.retain(|child| match child {
                    ParagraphChild::CommentRangeStart(ref id) => id != thread_id,
                    ParagraphChild::CommentRangeEnd(ref id) => id != thread_id,
                    ParagraphChild::CommentReference(ref id) => id != thread_id,
                    _ => true,
                });
            }
        }

        let initial_len = comments_doc.comments.len();
        comments_doc
            .comments
            .retain(|c| c.id != thread_id && c.parent_id.as_deref() != Some(thread_id));
        comments_doc.comments.len() < initial_len
    }

    /// Deletes an individual reply without deleting the root comment or anchor.
    pub fn delete_reply(comments_doc: &mut CommentsDocument, reply_id: &str) -> bool {
        let initial_len = comments_doc.comments.len();
        comments_doc
            .comments
            .retain(|c| c.id != reply_id || c.parent_id.is_none());
        comments_doc.comments.len() < initial_len
    }

    /// Returns summary statistics: (total_threads, unresolved_threads, resolved_threads).
    pub fn get_summary(comments_doc: &CommentsDocument) -> (usize, usize, usize) {
        let mut total = 0usize;
        let mut resolved = 0usize;
        let mut unresolved = 0usize;

        for c in &comments_doc.comments {
            if c.parent_id.is_none() {
                total = total.saturating_add(1);
                if c.is_resolved {
                    resolved = resolved.saturating_add(1);
                } else {
                    unresolved = unresolved.saturating_add(1);
                }
            }
        }

        (total, unresolved, resolved)
    }

    fn next_id(comments_doc: &CommentsDocument) -> String {
        let mut max_id = 0u64;
        for c in &comments_doc.comments {
            if let Ok(num) = c.id.parse::<u64>() {
                max_id = max_id.max(num);
            }
        }
        max_id.saturating_add(1).to_string()
    }
}
