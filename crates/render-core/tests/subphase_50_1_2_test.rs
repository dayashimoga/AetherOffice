//! Sub-Phase 50.1.2 Integration Tests: Bidirectional [[WikiLinks]] & Backlinks
//!
//! Complies strictly with GEC-v7.0 Article I (Architectural Integrity & Zero Global Document Singletons)
//! and Article III (Real Binary File Open & Save, Zero Data Loss):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in production library code.
//! - Bidirectional linking and backlink indexing across Notes, Word, and PDFs.
//! - Contextual snippet extraction and unlinked mentions discovery.

use render_core::wikilinks::{WikiLinkIndex, WikiLinkParser};

#[test]
fn test_parse_simple_wikilink() {
    let text = "Here is a reference to [[Project Roadmap]] in the text.";
    let links = WikiLinkParser::parse(text);

    assert_eq!(links.len(), 1);
    let link = &links[0];
    assert_eq!(link.target_doc, "Project Roadmap");
    assert!(link.target_anchor.is_none());
    assert!(link.display_text.is_none());
    assert_eq!(link.raw_text, "[[Project Roadmap]]");
    assert_eq!(link.start_offset, 23);
    assert_eq!(link.end_offset, 42);
}

#[test]
fn test_parse_aliased_wikilink() {
    let text = "Consult the [[Financial Model 2026|Annual Budget]] for projection.";
    let links = WikiLinkParser::parse(text);

    assert_eq!(links.len(), 1);
    let link = &links[0];
    assert_eq!(link.target_doc, "Financial Model 2026");
    assert!(link.target_anchor.is_none());
    assert_eq!(link.display_text.as_deref(), Some("Annual Budget"));
    assert_eq!(link.raw_text, "[[Financial Model 2026|Annual Budget]]");
}

#[test]
fn test_parse_anchored_wikilink() {
    let text = "Details in [[System Architecture#Storage Engine]].";
    let links = WikiLinkParser::parse(text);

    assert_eq!(links.len(), 1);
    let link = &links[0];
    assert_eq!(link.target_doc, "System Architecture");
    assert_eq!(link.target_anchor.as_deref(), Some("Storage Engine"));
    assert!(link.display_text.is_none());
}

#[test]
fn test_parse_anchored_and_aliased_wikilink() {
    let text = "Review [[Security Audit#PAdES Implementation|Signature Verification Spec]].";
    let links = WikiLinkParser::parse(text);

    assert_eq!(links.len(), 1);
    let link = &links[0];
    assert_eq!(link.target_doc, "Security Audit");
    assert_eq!(link.target_anchor.as_deref(), Some("PAdES Implementation"));
    assert_eq!(
        link.display_text.as_deref(),
        Some("Signature Verification Spec")
    );
}

#[test]
fn test_multiline_and_malformed_rejected() {
    let text = "Ignore [Single Bracket], [[\nNew Line Target\n]], and [[ ]] empty.";
    let links = WikiLinkParser::parse(text);
    assert!(links.is_empty());
}

#[test]
fn test_index_document_and_query_backlinks() {
    let mut index = WikiLinkIndex::new();

    let doc_a_content = "This proposal depends on [[Project Roadmap]] and [[Design System]].";
    index.index_document("doc-proposal", "Executive Proposal", doc_a_content);

    let doc_b_content = "The delivery schedule is aligned with [[Project Roadmap#Phase 1]].";
    index.index_document("doc-schedule", "Delivery Schedule", doc_b_content);

    let backlinks = index.get_backlinks("Project Roadmap");
    assert_eq!(backlinks.len(), 2);

    assert_eq!(backlinks[0].source_doc_id, "doc-proposal");
    assert_eq!(backlinks[0].source_doc_title, "Executive Proposal");
    assert!(backlinks[0].snippet.contains("[[Project Roadmap]]"));

    assert_eq!(backlinks[1].source_doc_id, "doc-schedule");
    assert_eq!(backlinks[1].target_anchor.as_deref(), Some("Phase 1"));

    let design_backlinks = index.get_backlinks("Design System");
    assert_eq!(design_backlinks.len(), 1);
    assert_eq!(design_backlinks[0].source_doc_id, "doc-proposal");
}

#[test]
fn test_forward_links_retrieval() {
    let mut index = WikiLinkIndex::new();

    let content = "Check [[First Target]] and [[Second Target|Second]].";
    index.index_document("doc-source", "Source Document", content);

    let forward = index.get_forward_links("doc-source");
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].target_doc, "First Target");
    assert_eq!(forward[1].target_doc, "Second Target");
    assert_eq!(forward[1].display_text.as_deref(), Some("Second"));
}

#[test]
fn test_remove_document_cleans_backlinks() {
    let mut index = WikiLinkIndex::new();

    index.index_document("doc-1", "Doc One", "Points to [[Shared Knowledge]].");
    index.index_document("doc-2", "Doc Two", "Also points to [[Shared Knowledge]].");

    assert_eq!(index.get_backlinks("Shared Knowledge").len(), 2);

    index.remove_document("doc-1");
    let after_remove = index.get_backlinks("Shared Knowledge");
    assert_eq!(after_remove.len(), 1);
    assert_eq!(after_remove[0].source_doc_id, "doc-2");
}

#[test]
fn test_suggest_targets_scoring() {
    let mut index = WikiLinkIndex::new();

    index.index_document(
        "doc-1",
        "AetherOffice Roadmap",
        "Product vision and milestones.",
    );
    index.index_document(
        "doc-2",
        "Sprint Planning",
        "Current sprint tasks and timeline.",
    );
    index.index_document("doc-3", "Roadmap Q4", "Detailed quarterly targets.");

    let suggestions = index.suggest_targets("Roadmap", 5);
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].title, "Roadmap Q4"); // Starts with query
    assert_eq!(suggestions[1].title, "AetherOffice Roadmap"); // Contains query
    assert!(suggestions[0].score > suggestions[1].score);
    assert!(suggestions[0].preview.is_some());
}

#[test]
fn test_unlinked_mentions_discovery() {
    let mut index = WikiLinkIndex::new();

    // Document defining the topic
    index.index_document(
        "doc-security",
        "Security Audit",
        "Master security guidelines.",
    );

    // Another document mentioning "Security Audit" as plain text (not in [[...]])
    let report_content =
        "We conducted a comprehensive Security Audit yesterday to ensure compliance.";
    index.index_document("doc-report", "Q4 Report", report_content);

    // Another document with an existing [[WikiLink]] should not trigger unlinked mention
    let linked_content = "Please read the [[Security Audit]] document.";
    index.index_document("doc-linked", "Onboarding", linked_content);

    let mentions = index.find_unlinked_mentions("Security Audit");
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].source_doc_id, "doc-report");
    assert_eq!(mentions[0].matched_text, "Security Audit");
    assert!(mentions[0].snippet.contains("Security Audit"));
}
