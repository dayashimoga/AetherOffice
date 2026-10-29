use crdt_collab::{CrdtError, FormatAttribute, FugueTreeRga};

#[test]
fn test_01_single_client_sequential_insert_and_read() {
    let mut doc = FugueTreeRga::new(1);
    assert_eq!(doc.visible_len(), 0);
    assert!(doc.is_empty());

    let deltas = doc.insert_local(0, "Hello World").expect("insert");
    assert_eq!(deltas.len(), 11);
    assert_eq!(doc.visible_len(), 11);
    assert_eq!(doc.to_plain_text(), "Hello World");

    // Insert in the middle
    let _ = doc.insert_local(5, ", Beautiful").expect("insert mid");
    assert_eq!(doc.to_plain_text(), "Hello, Beautiful World");
}

#[test]
fn test_02_concurrent_edits_two_clients_converge() {
    let mut client_a = FugueTreeRga::new(1);
    let mut client_b = FugueTreeRga::new(2);

    // Initial synchronized baseline
    let init_deltas = client_a.insert_local(0, "ABC").expect("init");
    for d in init_deltas {
        client_b.apply_delta(d).expect("apply init to b");
    }
    assert_eq!(client_a.to_plain_text(), "ABC");
    assert_eq!(client_b.to_plain_text(), "ABC");

    // Client A inserts "X" at offset 1 ("AXBC")
    let deltas_a = client_a.insert_local(1, "X").expect("A insert");

    // Client B inserts "Y" at offset 1 ("AYBC")
    let deltas_b = client_b.insert_local(1, "Y").expect("B insert");

    // Cross-sync: deliver B's deltas to A, and A's deltas to B
    for d in deltas_b {
        client_a.apply_delta(d).expect("A applies B");
    }
    for d in deltas_a {
        client_b.apply_delta(d).expect("B applies A");
    }

    // Both MUST converge to the exact same text
    let text_a = client_a.to_plain_text();
    let text_b = client_b.to_plain_text();
    assert_eq!(text_a, text_b);
    assert!(text_a == "AXYBC" || text_a == "AYXBC");
}

#[test]
fn test_03_concurrent_three_clients_fuzz_convergence() {
    let mut c1 = FugueTreeRga::new(1);
    let mut c2 = FugueTreeRga::new(2);
    let mut c3 = FugueTreeRga::new(3);

    // Synchronized preamble
    let init = c1.insert_local(0, "Root").expect("init");
    for d in &init {
        c2.apply_delta(d.clone()).expect("c2 init");
        c3.apply_delta(d.clone()).expect("c3 init");
    }

    // Concurrent typing
    let d1 = c1.insert_local(4, " [A]").expect("c1");
    let d2 = c2.insert_local(4, " [B]").expect("c2");
    let d3 = c3.insert_local(4, " [C]").expect("c3");

    // Exchange all deltas in arbitrary orders
    for d in &d2 {
        c1.apply_delta(d.clone()).expect("c1 <- c2");
    }
    for d in &d3 {
        c1.apply_delta(d.clone()).expect("c1 <- c3");
    }

    for d in &d1 {
        c2.apply_delta(d.clone()).expect("c2 <- c1");
    }
    for d in &d3 {
        c2.apply_delta(d.clone()).expect("c2 <- c3");
    }

    for d in &d3 {
        c3.apply_delta(d.clone()).expect("c3 self idempotent");
    }
    for d in &d1 {
        c3.apply_delta(d.clone()).expect("c3 <- c1");
    }
    for d in &d2 {
        c3.apply_delta(d.clone()).expect("c3 <- c2");
    }

    let r1 = c1.to_plain_text();
    let r2 = c2.to_plain_text();
    let r3 = c3.to_plain_text();

    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
    assert!(r1.starts_with("Root"));
}

#[test]
fn test_04_deletion_and_tombstone_compaction() {
    let mut doc = FugueTreeRga::new(1);
    doc.insert_local(0, "The Quick Brown Fox").expect("insert");
    assert_eq!(doc.visible_len(), 19);

    // Delete "Quick " (offset 4, len 6)
    let del_deltas = doc.delete_local(4, 6).expect("delete");
    assert_eq!(del_deltas.len(), 6);
    assert_eq!(doc.to_plain_text(), "The Brown Fox");
    assert_eq!(doc.visible_len(), 13);
}

#[test]
fn test_05_concurrent_insert_and_delete() {
    let mut c1 = FugueTreeRga::new(1);
    let mut c2 = FugueTreeRga::new(2);

    let init = c1.insert_local(0, "AetherOffice").expect("init");
    for d in init {
        c2.apply_delta(d).expect("c2 init");
    }

    // C1 deletes "Office" (offset 6, len 6)
    let d1 = c1.delete_local(6, 6).expect("c1 delete");

    // C2 inserts " Suite" at offset 12
    let d2 = c2.insert_local(12, " Suite").expect("c2 insert");

    // Sync
    for d in d2 {
        c1.apply_delta(d).expect("c1 <- c2");
    }
    for d in d1 {
        c2.apply_delta(d).expect("c2 <- c1");
    }

    assert_eq!(c1.to_plain_text(), c2.to_plain_text());
    assert_eq!(c1.to_plain_text(), "Aether Suite");
}

#[test]
fn test_06_peritext_rich_text_formatting() {
    let mut doc = FugueTreeRga::new(1);
    doc.insert_local(0, "Hello World").expect("insert");

    // Format "World" with Bold (chars 6..11)
    let _delta = doc
        .format_local(6, 11, FormatAttribute::Bold(true))
        .expect("format bold");

    let runs = doc.to_rich_runs();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "Hello ");
    assert!(!runs[0].bold);
    assert_eq!(runs[1].text, "World");
    assert!(runs[1].bold);
}

#[test]
fn test_07_overlapping_formatting_spans() {
    let mut doc = FugueTreeRga::new(1);
    doc.insert_local(0, "Antigravity").expect("insert");

    // Bold index 0..4 ("Anti")
    doc.format_local(0, 4, FormatAttribute::Bold(true))
        .expect("bold");

    // Italic index 2..9 ("tigravi")
    doc.format_local(2, 9, FormatAttribute::Italic(true))
        .expect("italic");

    let runs = doc.to_rich_runs();
    // Expected partition: "An" (Bold), "ti" (Bold + Italic), "gravi" (Italic), "ty" (Plain)
    assert_eq!(runs.len(), 4);
    assert_eq!(runs[0].text, "An");
    assert!(runs[0].bold);
    assert!(!runs[0].italic);

    assert_eq!(runs[1].text, "ti");
    assert!(runs[1].bold);
    assert!(runs[1].italic);

    assert_eq!(runs[2].text, "gravi");
    assert!(!runs[2].bold);
    assert!(runs[2].italic);

    assert_eq!(runs[3].text, "ty");
    assert!(!runs[3].bold);
    assert!(!runs[3].italic);
}

#[test]
fn test_08_bidirectional_piecetree_conversion() {
    let mut doc = FugueTreeRga::new(1);
    doc.insert_local(0, "PieceTree Collaboration Test")
        .expect("insert");

    // Export to word_engine::PieceTree
    let pt = doc.to_piece_tree().expect("to_piece_tree");
    assert_eq!(pt.get_text(), "PieceTree Collaboration Test");
    assert_eq!(pt.len(), 28);

    // Import from PieceTree into another client
    let doc2 = FugueTreeRga::from_piece_tree(2, &pt).expect("from_piece_tree");
    assert_eq!(doc2.to_plain_text(), "PieceTree Collaboration Test");
    assert_eq!(doc2.visible_len(), 28);
}

#[test]
fn test_09_state_vector_and_delta_idempotency() {
    let mut doc = FugueTreeRga::new(1);
    let deltas = doc.insert_local(0, "ABC").expect("insert");

    let sv = doc.get_state_vector();
    assert_eq!(sv.get(1), 3);

    // Re-applying the exact same deltas must be safe and idempotent
    for d in deltas {
        let res = doc.apply_delta(d);
        assert!(res.is_ok());
    }
    assert_eq!(doc.to_plain_text(), "ABC");
}

#[test]
fn test_10_invalid_offset_and_range_error_handling() {
    let mut doc = FugueTreeRga::new(1);
    doc.insert_local(0, "Test").expect("insert");

    // Out of bounds insert
    let err_insert = doc.insert_local(100, "X");
    assert!(matches!(err_insert, Err(CrdtError::InvalidOffset { .. })));

    // Out of bounds delete
    let err_delete = doc.delete_local(2, 10);
    assert!(matches!(err_delete, Err(CrdtError::InvalidOffset { .. })));

    // Inverted formatting range
    let err_fmt = doc.format_local(3, 1, FormatAttribute::Bold(true));
    assert!(matches!(err_fmt, Err(CrdtError::InvalidRange { .. })));
}
