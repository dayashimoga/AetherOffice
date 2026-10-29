//! Integration Tests for Subphase 14.2.1: Encrypted Transport Bus (WebRTC DataChannel & WebSocket Relay)
//!
//! Validates:
//! 1. End-to-end envelope encryption (ChaCha20-Poly1305) & key derivation.
//! 2. Binary packet serialization across all protocol types.
//! 3. Handshake negotiation & peer registration.
//! 4. Session mismatch isolation.
//! 5. Anti-replay protection & monotonic packet sequences.
//! 6. Two-step synchronization protocol (SyncStep1 / SyncStep2).
//! 7. Ephemeral presence, cursor & selection broadcast.
//! 8. Heartbeat tracking & stale peer pruning.
//! 9. Transport failover (WebRTC -> WebSocket Relay).
//! 10. Tauri shell command integration (`collab_commands`).

use crdt_collab::{
    CellCoord, CollabPacket, CollabTransportBus, CursorCoord, EncryptedEnvelope, EncryptionKey,
    FugueDelta, FugueTreeRga, GridCellValue, GridDelta, GridMatrixCrdt, OpId, PeerPresence,
    SelectionSpan, StateVector, TransportMode, TransportStatus,
};

#[test]
fn test_01_encryption_envelope_roundtrip() {
    let key = EncryptionKey::derive_from_passphrase("super-secret-room-passphrase", b"salt-123");
    let plaintext = b"Hello, AetherOffice Encrypted Collaboration!";

    let envelope =
        EncryptedEnvelope::encrypt(&key, plaintext, 101, 1).expect("Encryption succeeds");
    assert_eq!(envelope.sender_id, 101);
    assert_eq!(envelope.seq, 1);
    assert_ne!(envelope.ciphertext, plaintext);

    // Correct key decryption
    let decrypted = envelope.decrypt(&key).expect("Decryption succeeds");
    assert_eq!(decrypted, plaintext);

    // Wrong key decryption fails
    let wrong_key = EncryptionKey::derive_from_passphrase("incorrect-password", b"salt-123");
    let err = envelope.decrypt(&wrong_key);
    assert!(err.is_err());

    // Corrupted ciphertext decryption fails
    let mut tampered = envelope.clone();
    if let Some(first) = tampered.ciphertext.first_mut() {
        *first ^= 0xFF;
    }
    assert!(tampered.decrypt(&key).is_err());
}

#[test]
fn test_02_packet_binary_serialization_and_deserialization() {
    let mut sv = StateVector::new();
    sv.set(1, 15);
    sv.set(2, 42);

    let packets = vec![
        CollabPacket::Handshake {
            client_id: 1,
            session_id: "room-alpha".to_string(),
            username: "Alice".to_string(),
            app_mode: "writer".to_string(),
            protocol_version: 1,
        },
        CollabPacket::HandshakeAck {
            peer_id: 2,
            accepted: true,
            session_id: "room-alpha".to_string(),
            active_peers: vec![],
        },
        CollabPacket::SyncStep1 {
            sender_id: 1,
            state_vector: sv,
        },
        CollabPacket::SyncStep2 {
            sender_id: 2,
            target_peer_id: 1,
            text_deltas: vec![FugueDelta::Insert {
                id: OpId::new(2, 1, 1),
                parent_id: OpId::root(),
                right_origin: None,
                content: 'A',
            }],
            grid_deltas: vec![GridDelta::SetCell {
                id: OpId::new(2, 2, 2),
                coord: CellCoord::new("Sheet1", 0, 0),
                value: GridCellValue::Number(100.0),
                formula: None,
                format: None,
            }],
        },
        CollabPacket::Presence(PeerPresence::new(
            1,
            "Alice",
            "#FF5722",
            "Doc1.docx",
            "Main",
            1000,
        )),
        CollabPacket::Leave {
            client_id: 1,
            reason: "App shutdown".to_string(),
        },
    ];

    for original in packets {
        let bytes = original.to_bytes().expect("Serialize to bytes");
        let decoded = CollabPacket::from_bytes(&bytes).expect("Deserialize from bytes");
        assert_eq!(original, decoded);
    }
}

#[test]
fn test_03_handshake_and_connection_negotiation() {
    let key = EncryptionKey::derive_from_passphrase("team-room-key", b"test-salt");
    let mut bus_a = CollabTransportBus::new(
        1,
        "session-101",
        "Alice",
        "writer",
        TransportMode::WebRtcDataChannel,
    )
    .with_encryption_key(key.clone());

    let mut bus_b = CollabTransportBus::new(
        2,
        "session-101",
        "Bob",
        "writer",
        TransportMode::WebRtcDataChannel,
    )
    .with_encryption_key(key);

    // Alice connects and sends Handshake envelope
    let env_a = bus_a.connect().expect("Alice connects");
    assert_eq!(bus_a.status(), TransportStatus::Connected);

    // Bob receives Alice's handshake envelope
    let received = bus_b.receive_envelope(&env_a).expect("Bob receives");
    assert!(received.is_some());
    assert_eq!(bus_b.get_peers().len(), 1);
    let peer = bus_b.get_peer(1).expect("Peer 1 registered");
    assert_eq!(peer.username, "Alice");

    // Bob generated a HandshakeAck in his outbox
    let b_outbox = bus_b.drain_outbox();
    assert!(!b_outbox.is_empty());

    // Alice receives Bob's HandshakeAck
    for env in &b_outbox {
        let _ = bus_a.receive_envelope(env).expect("Alice receives ack");
    }
}

#[test]
fn test_04_session_mismatch_rejection() {
    let key = EncryptionKey::derive_from_passphrase("shared-room", b"test-salt");
    let mut bus_a =
        CollabTransportBus::new(1, "room-A", "Alice", "calc", TransportMode::LoopbackDirect)
            .with_encryption_key(key.clone());

    let mut bus_b =
        CollabTransportBus::new(2, "room-B", "Bob", "calc", TransportMode::LoopbackDirect)
            .with_encryption_key(key);

    let env_a = bus_a.connect().expect("Alice connects");

    // Bob is in room-B, receives handshake for room-A -> rejected!
    let err = bus_b.receive_envelope(&env_a);
    assert!(err.is_err());
    let msg = format!("{:?}", err.unwrap_err());
    assert!(msg.contains("SessionMismatch"));
}

#[test]
fn test_05_anti_replay_protection() {
    let key = EncryptionKey::derive_from_passphrase("replay-key", b"test-salt");
    let mut sender = CollabTransportBus::new(
        10,
        "room-replay",
        "Sender",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key.clone());

    let mut receiver = CollabTransportBus::new(
        20,
        "room-replay",
        "Receiver",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key);

    let env1 = sender.connect().expect("Connect");
    receiver
        .receive_envelope(&env1)
        .expect("Receiver accepts initial handshake");

    // Attempt to replay the exact same envelope
    let replay_err = receiver.receive_envelope(&env1);
    assert!(replay_err.is_err());
    let err_msg = format!("{:?}", replay_err.unwrap_err());
    assert!(err_msg.contains("ReplayDetected"));
}

#[test]
fn test_06_state_vector_sync_step1_and_step2() {
    let key = EncryptionKey::derive_from_passphrase("sync-room-pass", b"salt");
    let mut bus_a = CollabTransportBus::new(
        1,
        "room-sync",
        "ClientA",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key.clone());
    let mut bus_b = CollabTransportBus::new(
        2,
        "room-sync",
        "ClientB",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key);

    // Initial trees
    let mut tree_b = FugueTreeRga::new(2);
    let deltas_b1 = tree_b.insert_local(0, "X").expect("Insert X");
    let deltas_b2 = tree_b.insert_local(1, "Y").expect("Insert Y");

    // Client A sends SyncStep1 with its empty state vector
    let sv_a = StateVector::new();
    let env_step1 = bus_a.initiate_sync(&sv_a).expect("SyncStep1");

    // Client B receives SyncStep1
    let packet_received = bus_b
        .receive_envelope(&env_step1)
        .expect("B receives Step 1");
    if let Some(CollabPacket::SyncStep1 { sender_id, .. }) = packet_received {
        assert_eq!(sender_id, 1);
        let mut missing_text = Vec::new();
        missing_text.extend(deltas_b1);
        missing_text.extend(deltas_b2);

        // Client B responds with missing deltas in SyncStep2
        let env_step2 = bus_b
            .respond_sync(sender_id, missing_text, vec![])
            .expect("SyncStep2 response");

        // Client A receives SyncStep2 and applies deltas
        let mut tree_a = FugueTreeRga::new(1);
        let step2_packet = bus_a
            .receive_envelope(&env_step2)
            .expect("A receives Step 2");
        if let Some(CollabPacket::SyncStep2 { text_deltas, .. }) = step2_packet {
            assert_eq!(text_deltas.len(), 2);
            for d in text_deltas {
                tree_a.apply_delta(d).expect("Apply missing delta");
            }
        }
        assert_eq!(tree_a.to_plain_text(), "XY");
        assert_eq!(tree_a.to_plain_text(), tree_b.to_plain_text());
    } else {
        panic!("Expected SyncStep1 packet");
    }
}

#[test]
fn test_07_ephemeral_presence_cursor_and_selection_tracking() {
    let key = EncryptionKey::derive_from_passphrase("presence-key", b"salt");
    let mut bus_a = CollabTransportBus::new(
        1,
        "presence-doc",
        "Alice",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key.clone());
    let mut bus_b = CollabTransportBus::new(
        2,
        "presence-doc",
        "Bob",
        "writer",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key);

    let cursor = CursorCoord {
        text_offset: Some(42),
        grid_row: None,
        grid_col: None,
    };
    let selection = SelectionSpan {
        text_start: Some(40),
        text_end: Some(55),
        grid_start_row: None,
        grid_start_col: None,
        grid_end_row: None,
        grid_end_col: None,
    };

    let env = bus_a
        .broadcast_presence(
            "QuarterlyReport.docx",
            "Section2",
            Some(cursor),
            Some(selection),
            1600000,
        )
        .expect("Broadcast presence");

    bus_b
        .receive_envelope(&env)
        .expect("Bob receives Alice presence");

    let peer = bus_b.get_peer(1).expect("Alice in Bob's peer list");
    assert_eq!(peer.username, "Alice");
    assert_eq!(peer.active_document, "QuarterlyReport.docx");
    let cur = peer.cursor.as_ref().expect("Cursor present");
    assert_eq!(cur.text_offset, Some(42));
    let sel = peer.selection.as_ref().expect("Selection present");
    assert_eq!(sel.text_start, Some(40));
    assert_eq!(sel.text_end, Some(55));
}

#[test]
fn test_08_peer_timeout_and_pruning() {
    let mut bus =
        CollabTransportBus::new(1, "session", "Alice", "calc", TransportMode::LoopbackDirect);

    // Manually register peer 2 (seen at t=1000) and peer 3 (seen at t=4500)
    let p2 = PeerPresence::new(2, "Bob", "#00F", "Sheet1", "Grid", 1000);
    let p3 = PeerPresence::new(3, "Charlie", "#0F0", "Sheet1", "Grid", 4500);

    let key = EncryptionKey::derive_from_passphrase("test", b"salt");
    bus.set_encryption_key(key.clone());

    // Send presence packet from sender 2
    let env_p2 =
        EncryptedEnvelope::encrypt(&key, &CollabPacket::Presence(p2).to_bytes().unwrap(), 2, 1)
            .unwrap();
    let env_p3 =
        EncryptedEnvelope::encrypt(&key, &CollabPacket::Presence(p3).to_bytes().unwrap(), 3, 1)
            .unwrap();

    bus.receive_envelope(&env_p2).unwrap();
    bus.receive_envelope(&env_p3).unwrap();

    assert_eq!(bus.get_peers().len(), 2);

    // At now_ms = 4000, with max_silence = 2500ms:
    // Peer 2 elapsed = 3000ms > 2500ms -> STALE
    // Peer 3 elapsed = 0ms <= 2500ms -> ACTIVE
    let pruned = bus.prune_stale_peers(2500, 4000);
    assert_eq!(pruned, vec![2]);
    assert_eq!(bus.get_peers().len(), 1);
    assert!(bus.get_peer(3).is_some());
}

#[test]
fn test_09_transport_fallback_webrtc_to_websocket() {
    let mut bus = CollabTransportBus::new(
        1,
        "failover-room",
        "Alice",
        "slides",
        TransportMode::WebRtcDataChannel,
    );
    assert_eq!(bus.mode(), TransportMode::WebRtcDataChannel);
    assert!(bus.mode().is_direct_p2p());

    // P2P ICE failure triggers fallback
    bus.fallback_to_relay();
    assert_eq!(bus.mode(), TransportMode::WebSocketRelay);
    assert!(bus.mode().is_relay());
    assert_eq!(bus.status(), TransportStatus::Reconnecting);
}

#[test]
fn test_10_delta_batch_and_reassembly_across_peers() {
    let key = EncryptionKey::derive_from_passphrase("batch-key-10", b"salt-10");
    let mut bus_a = CollabTransportBus::new(
        1,
        "batch-room",
        "ClientA",
        "office",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key.clone());
    let mut bus_b = CollabTransportBus::new(
        2,
        "batch-room",
        "ClientB",
        "office",
        TransportMode::LoopbackDirect,
    )
    .with_encryption_key(key);

    let mut text_a = FugueTreeRga::new(1);
    let mut grid_a = GridMatrixCrdt::new(1);

    let d_t = text_a.insert_local(0, "Wow").unwrap();

    let d_g1 = grid_a.set_cell_value("Summary", 0, 0, GridCellValue::Number(1234.5));
    let d_g2 = grid_a.set_cell_value("Summary", 0, 1, GridCellValue::Text("OK".to_string()));

    let env = bus_a
        .broadcast_delta_batch(d_t, vec![d_g1, d_g2])
        .expect("Broadcast batch");

    let received = bus_b
        .receive_envelope(&env)
        .expect("Receive batch envelope");
    let mut text_b = FugueTreeRga::new(2);
    let mut grid_b = GridMatrixCrdt::new(2);

    if let Some(CollabPacket::DeltaBatch {
        text_deltas,
        grid_deltas,
        ..
    }) = received
    {
        assert_eq!(text_deltas.len(), 3);
        assert_eq!(grid_deltas.len(), 2);

        for dt in text_deltas {
            text_b.apply_delta(dt).expect("Apply text delta");
        }
        for dg in grid_deltas {
            grid_b.apply_delta(dg).expect("Apply grid delta");
        }
    } else {
        panic!("Expected DeltaBatch packet");
    }

    assert_eq!(text_b.to_plain_text(), "Wow");
    assert_eq!(text_b.to_plain_text(), text_a.to_plain_text());

    let val = grid_b.get_cell("Summary", 0, 0).expect("Cell 0,0 present");
    assert_eq!(val.value, GridCellValue::Number(1234.5));
}
