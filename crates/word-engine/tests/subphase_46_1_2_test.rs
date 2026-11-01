//! Sub-Phase 46.1.2 Integration Tests: Anchored Threaded Comments & Resolved History
//!
//! Validates:
//! - Anchored comment creation (`add_comment`) linking to document text.
//! - Threaded replies (`add_reply`) with parent-child hierarchy.
//! - Thread resolution toggle (`set_resolved`) on root and replies.
//! - Comment deletion: `delete_thread` removes anchors and comments; `delete_reply` removes individual reply.
//! - Comments XML round-trip serialization and parsing with `w:parent` and `w:done`.
//! - Summary statistics (`get_summary`).
//! - Performance SLA under high comment volume.

use std::time::Instant;
use word_engine::comments::*;
use word_engine::model::*;
use word_engine::parser::parse_comments;
use word_engine::serializer::serialize_comments;
use word_engine::sub_documents::*;

#[test]
fn test_anchored_comment_creation_and_linking() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "The quarterly revenue grew by 15%.".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let mut comments_doc = CommentsDocument::default();
    let comment_id = CommentEngine::add_comment(
        &mut doc,
        &mut comments_doc,
        0,
        "Alice Smith",
        Some("AS"),
        "Please double check the 15% growth figure against audited financials.",
    );

    assert_eq!(comment_id, "1");
    assert_eq!(comments_doc.comments.len(), 1);

    // Verify document paragraph contains anchors
    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        let has_start = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentRangeStart(ref id) if id == "1"));
        let has_end = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentRangeEnd(ref id) if id == "1"));
        let has_ref = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentReference(ref id) if id == "1"));
        assert!(has_start, "Paragraph must contain CommentRangeStart");
        assert!(has_end, "Paragraph must contain CommentRangeEnd");
        assert!(has_ref, "Paragraph must contain CommentReference");
    } else {
        panic!("Element 0 must be a paragraph");
    }

    // Verify list_threads extracts correct anchor text
    let threads = CommentEngine::list_threads(&doc, &comments_doc);
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id(), "1");
    assert_eq!(
        threads[0].anchor_text.as_deref(),
        Some("The quarterly revenue grew by 15%.")
    );
    assert_eq!(threads[0].paragraph_index, Some(0));
    assert_eq!(threads[0].root.author, "Alice Smith");
    assert_eq!(threads[0].root.initials.as_deref(), Some("AS"));
    assert!(!threads[0].is_resolved);
    assert_eq!(threads[0].replies.len(), 0);
    assert_eq!(threads[0].total_comments(), 1);
}

#[test]
fn test_threaded_replies_hierarchy() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content
        .push(RunContent::Text("Target clause text".to_string(), false));
    p.children.push(ParagraphChild::Run(r));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let mut comments_doc = CommentsDocument::default();
    let root_id = CommentEngine::add_comment(
        &mut doc,
        &mut comments_doc,
        0,
        "Alice Smith",
        Some("AS"),
        "Initial review feedback.",
    );

    // Add first reply
    let reply1_id = CommentEngine::add_reply(
        &mut comments_doc,
        &root_id,
        "Bob Jones",
        Some("BJ"),
        "I verified with legal, this clause is standard.",
    )
    .expect("Reply 1 should succeed");

    assert_eq!(reply1_id, "2");

    // Add second reply
    let reply2_id = CommentEngine::add_reply(
        &mut comments_doc,
        &root_id,
        "Charlie Brown",
        Some("CB"),
        "Concurring with Bob, no change needed.",
    )
    .expect("Reply 2 should succeed");

    assert_eq!(reply2_id, "3");

    // Attempt reply to non-existent parent
    let invalid_reply = CommentEngine::add_reply(
        &mut comments_doc,
        "9999",
        "Ghost User",
        None,
        "This should fail.",
    );
    assert!(
        invalid_reply.is_none(),
        "Reply to non-existent parent must return None"
    );

    // Verify list_threads structure
    let threads = CommentEngine::list_threads(&doc, &comments_doc);
    assert_eq!(threads.len(), 1);
    let thread = &threads[0];
    assert_eq!(thread.root.id, root_id);
    assert_eq!(thread.replies.len(), 2);
    assert_eq!(thread.total_comments(), 3);
    assert_eq!(thread.replies[0].id, reply1_id);
    assert_eq!(
        thread.replies[0].parent_id.as_deref(),
        Some(root_id.as_str())
    );
    assert_eq!(thread.replies[1].id, reply2_id);
    assert_eq!(
        thread.replies[1].parent_id.as_deref(),
        Some(root_id.as_str())
    );
}

#[test]
fn test_thread_resolution_toggle() {
    let mut doc = Document::default();
    let p = Paragraph::default();
    doc.body.elements.push(BodyElement::Paragraph(p));

    let mut comments_doc = CommentsDocument::default();
    let root_id = CommentEngine::add_comment(
        &mut doc,
        &mut comments_doc,
        0,
        "Alice",
        None,
        "Needs revision.",
    );
    CommentEngine::add_reply(&mut comments_doc, &root_id, "Bob", None, "Done.").expect("Reply ok");

    assert!(!CommentEngine::list_threads(&doc, &comments_doc)[0].is_resolved);

    // Resolve thread
    let res = CommentEngine::set_resolved(&mut comments_doc, &root_id, true);
    assert!(res, "set_resolved must return true");

    let threads = CommentEngine::list_threads(&doc, &comments_doc);
    assert!(threads[0].is_resolved, "Thread must now be resolved");
    assert!(threads[0].root.is_resolved);
    assert!(threads[0].replies[0].is_resolved);

    // Reopen thread
    let res_reopen = CommentEngine::set_resolved(&mut comments_doc, &root_id, false);
    assert!(res_reopen);

    let threads_reopened = CommentEngine::list_threads(&doc, &comments_doc);
    assert!(
        !threads_reopened[0].is_resolved,
        "Thread must now be unresolved"
    );
    assert!(!threads_reopened[0].root.is_resolved);
    assert!(!threads_reopened[0].replies[0].is_resolved);
}

#[test]
fn test_comment_deletion_thread_vs_reply() {
    let mut doc = Document::default();
    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text(
        "Important text to annotate".to_string(),
        false,
    ));
    p.children.push(ParagraphChild::Run(r));
    doc.body.elements.push(BodyElement::Paragraph(p));

    let mut comments_doc = CommentsDocument::default();
    let root_id = CommentEngine::add_comment(
        &mut doc,
        &mut comments_doc,
        0,
        "Alice",
        None,
        "Root comment",
    );
    let reply1_id =
        CommentEngine::add_reply(&mut comments_doc, &root_id, "Bob", None, "Reply 1").expect("ok");
    let _reply2_id =
        CommentEngine::add_reply(&mut comments_doc, &root_id, "Charlie", None, "Reply 2")
            .expect("ok");

    assert_eq!(comments_doc.comments.len(), 3);

    // Delete single reply
    let del_reply = CommentEngine::delete_reply(&mut comments_doc, &reply1_id);
    assert!(del_reply, "delete_reply must return true");
    assert_eq!(comments_doc.comments.len(), 2);

    let threads = CommentEngine::list_threads(&doc, &comments_doc);
    assert_eq!(threads[0].replies.len(), 1);
    assert_eq!(threads[0].replies[0].author, "Charlie");

    // Paragraph anchors must still exist
    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        let has_anchors = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentRangeStart(ref id) if id == &root_id));
        assert!(has_anchors, "Anchors must remain when deleting a reply");
    }

    // Now delete entire thread
    let del_thread = CommentEngine::delete_thread(&mut doc, &mut comments_doc, &root_id);
    assert!(del_thread, "delete_thread must return true");
    assert_eq!(comments_doc.comments.len(), 0);

    // Paragraph anchors must be removed
    if let BodyElement::Paragraph(ref p) = doc.body.elements[0] {
        let has_start = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentRangeStart(_)));
        let has_end = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentRangeEnd(_)));
        let has_ref = p
            .children
            .iter()
            .any(|c| matches!(c, ParagraphChild::CommentReference(_)));
        assert!(!has_start, "CommentRangeStart must be removed");
        assert!(!has_end, "CommentRangeEnd must be removed");
        assert!(!has_ref, "CommentReference must be removed");
    }
}

#[test]
fn test_comments_xml_roundtrip() {
    let mut comments_doc = CommentsDocument::default();
    comments_doc.namespaces.push((
        "xmlns:w".to_string(),
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main".to_string(),
    ));

    let mut root = Comment::new("101", "Alice Smith", "Please update the table header.");
    root.initials = Some("AS".to_string());
    root.date = Some("2026-11-01T12:00:00Z".to_string());
    root.is_resolved = false;
    comments_doc.comments.push(root);

    let mut reply = Comment::new("102", "Bob Jones", "Updated in revision 3.");
    reply.initials = Some("BJ".to_string());
    reply.date = Some("2026-11-01T12:30:00Z".to_string());
    reply.parent_id = Some("101".to_string());
    reply.is_resolved = true;
    comments_doc.comments.push(reply);

    let xml_bytes = serialize_comments(&comments_doc).expect("Serialization must succeed");
    let xml_str = String::from_utf8_lossy(&xml_bytes);

    assert!(xml_str.contains(r#"w:id="101""#));
    assert!(xml_str.contains(r#"w:author="Alice Smith""#));
    assert!(xml_str.contains(r#"w:initials="AS""#));
    assert!(xml_str.contains(r#"w:id="102""#));
    assert!(xml_str.contains(r#"w:parent="101""#));
    assert!(xml_str.contains(r#"w:done="1""#));

    // Parse back
    let parsed_doc = parse_comments(&xml_bytes).expect("Parsing must succeed");
    assert_eq!(parsed_doc.comments.len(), 2);

    let parsed_root = &parsed_doc.comments[0];
    assert_eq!(parsed_root.id, "101");
    assert_eq!(parsed_root.author, "Alice Smith");
    assert_eq!(parsed_root.initials.as_deref(), Some("AS"));
    assert_eq!(parsed_root.parent_id, None);
    assert!(!parsed_root.is_resolved);
    assert_eq!(parsed_root.get_text(), "Please update the table header.");

    let parsed_reply = &parsed_doc.comments[1];
    assert_eq!(parsed_reply.id, "102");
    assert_eq!(parsed_reply.author, "Bob Jones");
    assert_eq!(parsed_reply.initials.as_deref(), Some("BJ"));
    assert_eq!(parsed_reply.parent_id.as_deref(), Some("101"));
    assert!(parsed_reply.is_resolved);
    assert_eq!(parsed_reply.get_text(), "Updated in revision 3.");
}

#[test]
fn test_comments_summary_statistics() {
    let mut comments_doc = CommentsDocument::default();

    // 2 unresolved root comments, 1 resolved root comment
    let mut c1 = Comment::new("1", "User A", "Comment 1");
    c1.is_resolved = false;
    comments_doc.comments.push(c1);

    let mut c2 = Comment::new("2", "User B", "Comment 2");
    c2.is_resolved = true;
    comments_doc.comments.push(c2);

    let mut c3 = Comment::new("3", "User C", "Comment 3");
    c3.is_resolved = false;
    comments_doc.comments.push(c3);

    // Replies should not increase total root threads count
    let mut reply = Comment::new("4", "User A", "Reply to 1");
    reply.parent_id = Some("1".to_string());
    comments_doc.comments.push(reply);

    let (total, unresolved, resolved) = CommentEngine::get_summary(&comments_doc);
    assert_eq!(total, 3, "Total root threads must be 3");
    assert_eq!(unresolved, 2, "Unresolved threads must be 2");
    assert_eq!(resolved, 1, "Resolved threads must be 1");
}

#[test]
fn test_high_volume_comments_performance() {
    let mut doc = Document::default();
    for i in 0..100 {
        let mut p = Paragraph::default();
        let mut r = Run::default();
        r.content.push(RunContent::Text(
            format!("Paragraph text content for index {}.", i),
            false,
        ));
        p.children.push(ParagraphChild::Run(r));
        doc.body.elements.push(BodyElement::Paragraph(p));
    }

    let mut comments_doc = CommentsDocument::default();

    let start = Instant::now();
    for i in 0..1000 {
        let para_idx = i % 100;
        let root_id = CommentEngine::add_comment(
            &mut doc,
            &mut comments_doc,
            para_idx,
            "Reviewer Agent",
            Some("RA"),
            "Automated review note regarding document styling and compliance.",
        );
        CommentEngine::add_reply(
            &mut comments_doc,
            &root_id,
            "Resolution Agent",
            Some("RSA"),
            "Compliance verified, ready for final signoff.",
        );
    }
    let insertion_duration = start.elapsed();
    assert!(
        insertion_duration.as_millis() < 500,
        "1,000 threads insertion should be < 500ms, took {:?}",
        insertion_duration
    );

    // Test list_threads performance
    let list_start = Instant::now();
    let threads = CommentEngine::list_threads(&doc, &comments_doc);
    let list_duration = list_start.elapsed();

    assert_eq!(threads.len(), 1000);
    assert!(
        list_duration.as_millis() < 50,
        "Listing 1,000 threads must complete in < 50ms, took {:?}",
        list_duration
    );

    let summary_start = Instant::now();
    let (total, unresolved, resolved) = CommentEngine::get_summary(&comments_doc);
    let summary_duration = summary_start.elapsed();

    assert_eq!(total, 1000);
    assert_eq!(unresolved, 1000);
    assert_eq!(resolved, 0);
    assert!(
        summary_duration.as_millis() < 10,
        "Summary of 1,000 comments must be < 10ms, took {:?}",
        summary_duration
    );
}
