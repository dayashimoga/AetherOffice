use crate::error::CrdtError;
use crate::fugue::{FugueDelta, StateVector};
use crate::grid_crdt::GridDelta;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};

/// Symmetric 256-bit encryption key for end-to-end encrypted packet transport.
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptionKey(pub [u8; 32]);

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptionKey(***)")
    }
}

impl EncryptionKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Derives a 32-byte key from a passphrase and optional salt using iterated SHA-256 rounds.
    pub fn derive_from_passphrase(passphrase: &str, salt: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        hasher.update(salt);
        let mut digest = hasher.finalize();

        // 1000 iterative key-stretching rounds
        for round in 1u32..1000 {
            let mut h = Sha256::new();
            h.update(digest);
            h.update(round.to_le_bytes());
            digest = h.finalize();
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&digest);
        Self(key)
    }
}

/// Binary wire envelope with 96-bit nonce, sender ID, sequence counter, and AEAD ciphertext + tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub version: u8,
    pub sender_id: u64,
    pub seq: u64,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

impl EncryptedEnvelope {
    pub const CURRENT_VERSION: u8 = 1;

    /// Encrypts plaintext bytes using ChaCha20-Poly1305 with a monotonic (sender_id + seq) nonce.
    pub fn encrypt(
        key: &EncryptionKey,
        plaintext: &[u8],
        sender_id: u64,
        seq: u64,
    ) -> Result<Self, CrdtError> {
        let mut nonce = [0u8; 12];
        nonce[0..8].copy_from_slice(&sender_id.to_le_bytes());
        nonce[8..12].copy_from_slice(&(seq as u32).to_le_bytes());

        let cipher_key = Key::from_slice(&key.0);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        let cipher_nonce = Nonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(cipher_nonce, plaintext)
            .map_err(|e| CrdtError::EncryptionError(e.to_string()))?;

        Ok(Self {
            version: Self::CURRENT_VERSION,
            sender_id,
            seq,
            nonce,
            ciphertext,
        })
    }

    /// Decrypts ciphertext bytes using ChaCha20-Poly1305.
    pub fn decrypt(&self, key: &EncryptionKey) -> Result<Vec<u8>, CrdtError> {
        let cipher_key = Key::from_slice(&key.0);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        let cipher_nonce = Nonce::from_slice(&self.nonce);

        cipher
            .decrypt(cipher_nonce, self.ciphertext.as_ref())
            .map_err(|e| CrdtError::DecryptionError(e.to_string()))
    }
}

/// Ephemeral cursor coordinates across Word (character offset) or Calc (row/col).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CursorCoord {
    pub text_offset: Option<usize>,
    pub grid_row: Option<u32>,
    pub grid_col: Option<u32>,
}

/// Ephemeral selection ranges across Word (start..end offset) or Calc (start cell .. end cell).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SelectionSpan {
    pub text_start: Option<usize>,
    pub text_end: Option<usize>,
    pub grid_start_row: Option<u32>,
    pub grid_start_col: Option<u32>,
    pub grid_end_row: Option<u32>,
    pub grid_end_col: Option<u32>,
}

/// Real-time multi-user presence status broadcasted periodically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerPresence {
    pub client_id: u64,
    pub username: String,
    pub color_hex: String,
    pub active_document: String,
    pub active_tab: String,
    pub cursor: Option<CursorCoord>,
    pub selection: Option<SelectionSpan>,
    pub last_seen_ms: u64,
}

impl PeerPresence {
    pub fn new(
        client_id: u64,
        username: impl Into<String>,
        color_hex: impl Into<String>,
        active_document: impl Into<String>,
        active_tab: impl Into<String>,
        now_ms: u64,
    ) -> Self {
        Self {
            client_id,
            username: username.into(),
            color_hex: color_hex.into(),
            active_document: active_document.into(),
            active_tab: active_tab.into(),
            cursor: None,
            selection: None,
            last_seen_ms: now_ms,
        }
    }
}

/// Collaborative protocol message payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollabPacket {
    Handshake {
        client_id: u64,
        session_id: String,
        username: String,
        app_mode: String,
        protocol_version: u32,
    },
    HandshakeAck {
        peer_id: u64,
        accepted: bool,
        session_id: String,
        active_peers: Vec<PeerPresence>,
    },
    SyncStep1 {
        sender_id: u64,
        state_vector: StateVector,
    },
    SyncStep2 {
        sender_id: u64,
        target_peer_id: u64,
        text_deltas: Vec<FugueDelta>,
        grid_deltas: Vec<GridDelta>,
    },
    UpdateText {
        delta: FugueDelta,
    },
    UpdateGrid {
        delta: GridDelta,
    },
    DeltaBatch {
        sender_id: u64,
        text_deltas: Vec<FugueDelta>,
        grid_deltas: Vec<GridDelta>,
    },
    Presence(PeerPresence),
    Leave {
        client_id: u64,
        reason: String,
    },
    Heartbeat {
        client_id: u64,
        timestamp_ms: u64,
    },
}

impl CollabPacket {
    pub fn to_bytes(&self) -> Result<Vec<u8>, CrdtError> {
        bincode::serialize(self).map_err(|e| CrdtError::SerializationError(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CrdtError> {
        bincode::deserialize(bytes).map_err(|e| CrdtError::SerializationError(e.to_string()))
    }
}

/// Dual-mode network transport selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// Peer-to-peer WebRTC DataChannel (LAN direct or ICE-negotiated).
    WebRtcDataChannel,
    /// Fallback server-mediated WebSocket relay.
    WebSocketRelay,
    /// In-memory direct channel for multi-instance local tests.
    LoopbackDirect,
}

impl TransportMode {
    pub fn is_direct_p2p(&self) -> bool {
        matches!(self, Self::WebRtcDataChannel)
    }

    pub fn is_relay(&self) -> bool {
        matches!(self, Self::WebSocketRelay)
    }
}

/// Connection lifecycle status of the collaborative transport bus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Closed,
}

/// Transport orchestrator managing encryption, handshake negotiation, presence, and delta routing.
pub struct CollabTransportBus {
    pub client_id: u64,
    pub session_id: String,
    pub username: String,
    pub app_mode: String,
    status: TransportStatus,
    mode: TransportMode,
    encryption_key: Option<EncryptionKey>,
    seq_counter: u64,
    peers: HashMap<u64, PeerPresence>,
    seen_seqs: HashMap<u64, u64>,
    outbox: VecDeque<EncryptedEnvelope>,
    inbox: VecDeque<CollabPacket>,
}

impl CollabTransportBus {
    pub fn new(
        client_id: u64,
        session_id: impl Into<String>,
        username: impl Into<String>,
        app_mode: impl Into<String>,
        mode: TransportMode,
    ) -> Self {
        Self {
            client_id,
            session_id: session_id.into(),
            username: username.into(),
            app_mode: app_mode.into(),
            status: TransportStatus::Disconnected,
            mode,
            encryption_key: None,
            seq_counter: 0,
            peers: HashMap::new(),
            seen_seqs: HashMap::new(),
            outbox: VecDeque::new(),
            inbox: VecDeque::new(),
        }
    }

    pub fn with_encryption_key(mut self, key: EncryptionKey) -> Self {
        self.encryption_key = Some(key);
        self
    }

    pub fn set_encryption_key(&mut self, key: EncryptionKey) {
        self.encryption_key = Some(key);
    }

    pub fn status(&self) -> TransportStatus {
        self.status.clone()
    }

    pub fn mode(&self) -> TransportMode {
        self.mode
    }

    /// Transitions to WebSocket relay if direct WebRTC fails.
    pub fn fallback_to_relay(&mut self) {
        self.mode = TransportMode::WebSocketRelay;
        self.status = TransportStatus::Reconnecting;
    }

    /// Next strictly monotonic sequence number for outgoing envelopes.
    fn next_seq(&mut self) -> u64 {
        self.seq_counter += 1;
        self.seq_counter
    }

    /// Wraps and encrypts a packet into a wire envelope.
    pub fn pack_envelope(&mut self, packet: &CollabPacket) -> Result<EncryptedEnvelope, CrdtError> {
        let plaintext = packet.to_bytes()?;
        let seq = self.next_seq();
        let key = self.encryption_key.as_ref().ok_or_else(|| {
            CrdtError::EncryptionError("No encryption key configured for transport bus".to_string())
        })?;
        EncryptedEnvelope::encrypt(key, &plaintext, self.client_id, seq)
    }

    /// Decrypts and unpacks a wire envelope, enforcing anti-replay sequence validation.
    pub fn unpack_envelope(
        &mut self,
        envelope: &EncryptedEnvelope,
    ) -> Result<CollabPacket, CrdtError> {
        // Enforce anti-replay check
        if let Some(&last_seq) = self.seen_seqs.get(&envelope.sender_id) {
            if envelope.seq <= last_seq {
                return Err(CrdtError::ReplayDetected {
                    sender_id: envelope.sender_id,
                    seq: envelope.seq,
                });
            }
        }
        self.seen_seqs.insert(envelope.sender_id, envelope.seq);

        let key = self.encryption_key.as_ref().ok_or_else(|| {
            CrdtError::DecryptionError("No encryption key configured for transport bus".to_string())
        })?;
        let plaintext = envelope.decrypt(key)?;
        CollabPacket::from_bytes(&plaintext)
    }

    /// Initiates connection handshake envelope.
    pub fn connect(&mut self) -> Result<EncryptedEnvelope, CrdtError> {
        self.status = TransportStatus::Connecting;
        let handshake = CollabPacket::Handshake {
            client_id: self.client_id,
            session_id: self.session_id.clone(),
            username: self.username.clone(),
            app_mode: self.app_mode.clone(),
            protocol_version: 1,
        };
        let env = self.pack_envelope(&handshake)?;
        self.outbox.push_back(env.clone());
        self.status = TransportStatus::Connected;
        Ok(env)
    }

    /// Leaves session and queues Leave packet.
    pub fn disconnect(&mut self, reason: &str) -> Result<EncryptedEnvelope, CrdtError> {
        let leave = CollabPacket::Leave {
            client_id: self.client_id,
            reason: reason.to_string(),
        };
        let env = self.pack_envelope(&leave)?;
        self.outbox.push_back(env.clone());
        self.status = TransportStatus::Closed;
        Ok(env)
    }

    /// Queues and serializes a packet for transmission.
    pub fn send_packet(&mut self, packet: CollabPacket) -> Result<EncryptedEnvelope, CrdtError> {
        let env = self.pack_envelope(&packet)?;
        self.outbox.push_back(env.clone());
        Ok(env)
    }

    /// Processes an incoming raw envelope, validates session and handshake, and updates peers.
    pub fn receive_envelope(
        &mut self,
        envelope: &EncryptedEnvelope,
    ) -> Result<Option<CollabPacket>, CrdtError> {
        let packet = self.unpack_envelope(envelope)?;
        match &packet {
            CollabPacket::Handshake {
                client_id,
                session_id,
                username,
                ..
            } => {
                if *session_id != self.session_id {
                    return Err(CrdtError::SessionMismatch {
                        expected: self.session_id.clone(),
                        actual: session_id.clone(),
                    });
                }
                // Register peer
                let peer = PeerPresence::new(
                    *client_id,
                    username.clone(),
                    "#2196F3",
                    "Document1",
                    "Tab1",
                    0,
                );
                self.peers.insert(*client_id, peer);

                // Auto-reply with HandshakeAck
                let ack = CollabPacket::HandshakeAck {
                    peer_id: self.client_id,
                    accepted: true,
                    session_id: self.session_id.clone(),
                    active_peers: self.get_peers(),
                };
                let _ = self.send_packet(ack);
            }
            CollabPacket::HandshakeAck {
                peer_id,
                accepted,
                active_peers,
                ..
            } => {
                if !*accepted {
                    return Err(CrdtError::HandshakeRejected(format!(
                        "Peer {peer_id} rejected connection"
                    )));
                }
                for p in active_peers {
                    if p.client_id != self.client_id {
                        self.peers.insert(p.client_id, p.clone());
                    }
                }
            }
            CollabPacket::Presence(presence) => {
                if presence.client_id != self.client_id {
                    self.peers.insert(presence.client_id, presence.clone());
                }
            }
            CollabPacket::Leave { client_id, .. } => {
                self.peers.remove(client_id);
            }
            _ => {}
        }

        self.inbox.push_back(packet.clone());
        Ok(Some(packet))
    }

    /// Broadcasts text delta to all peers.
    pub fn broadcast_text_delta(
        &mut self,
        delta: FugueDelta,
    ) -> Result<EncryptedEnvelope, CrdtError> {
        let packet = CollabPacket::UpdateText { delta };
        self.send_packet(packet)
    }

    /// Broadcasts grid delta to all peers.
    pub fn broadcast_grid_delta(
        &mut self,
        delta: GridDelta,
    ) -> Result<EncryptedEnvelope, CrdtError> {
        let packet = CollabPacket::UpdateGrid { delta };
        self.send_packet(packet)
    }

    /// Broadcasts batch of deltas.
    pub fn broadcast_delta_batch(
        &mut self,
        text_deltas: Vec<FugueDelta>,
        grid_deltas: Vec<GridDelta>,
    ) -> Result<EncryptedEnvelope, CrdtError> {
        let packet = CollabPacket::DeltaBatch {
            sender_id: self.client_id,
            text_deltas,
            grid_deltas,
        };
        self.send_packet(packet)
    }

    /// Broadcasts ephemeral presence update (cursor + selection).
    pub fn broadcast_presence(
        &mut self,
        active_document: &str,
        active_tab: &str,
        cursor: Option<CursorCoord>,
        selection: Option<SelectionSpan>,
        now_ms: u64,
    ) -> Result<EncryptedEnvelope, CrdtError> {
        let presence = PeerPresence {
            client_id: self.client_id,
            username: self.username.clone(),
            color_hex: "#4CAF50".to_string(),
            active_document: active_document.to_string(),
            active_tab: active_tab.to_string(),
            cursor,
            selection,
            last_seen_ms: now_ms,
        };
        let packet = CollabPacket::Presence(presence);
        self.send_packet(packet)
    }

    /// Sync Step 1: Initiates synchronization by sending local StateVector.
    pub fn initiate_sync(&mut self, sv: &StateVector) -> Result<EncryptedEnvelope, CrdtError> {
        let packet = CollabPacket::SyncStep1 {
            sender_id: self.client_id,
            state_vector: sv.clone(),
        };
        self.send_packet(packet)
    }

    /// Sync Step 2: Responds to a peer's SyncStep1 with missing deltas.
    pub fn respond_sync(
        &mut self,
        target_peer_id: u64,
        text_deltas: Vec<FugueDelta>,
        grid_deltas: Vec<GridDelta>,
    ) -> Result<EncryptedEnvelope, CrdtError> {
        let packet = CollabPacket::SyncStep2 {
            sender_id: self.client_id,
            target_peer_id,
            text_deltas,
            grid_deltas,
        };
        self.send_packet(packet)
    }

    /// Retrieves all currently connected peers.
    pub fn get_peers(&self) -> Vec<PeerPresence> {
        self.peers.values().cloned().collect()
    }

    pub fn get_peer(&self, peer_id: u64) -> Option<&PeerPresence> {
        self.peers.get(&peer_id)
    }

    /// Removes peers who haven't sent heartbeats/updates within `max_silence_ms`.
    pub fn prune_stale_peers(&mut self, max_silence_ms: u64, now_ms: u64) -> Vec<u64> {
        let mut stale = Vec::new();
        for (&id, peer) in &self.peers {
            if now_ms.saturating_sub(peer.last_seen_ms) > max_silence_ms {
                stale.push(id);
            }
        }
        for id in &stale {
            self.peers.remove(id);
        }
        stale
    }

    /// Empties and returns queued outbound envelopes.
    pub fn drain_outbox(&mut self) -> Vec<EncryptedEnvelope> {
        self.outbox.drain(..).collect()
    }

    /// Empties and returns queued inbound packets.
    pub fn drain_inbox(&mut self) -> Vec<CollabPacket> {
        self.inbox.drain(..).collect()
    }
}
