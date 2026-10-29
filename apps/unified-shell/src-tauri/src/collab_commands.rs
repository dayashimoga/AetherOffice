//! Collaborative Session Tauri IPC Commands (GEC-v2.0 Subphase 14.2.1)
//!
//! Provides IPC commands for multi-user session management:
//! - `collab_join_session`
//! - `collab_leave_session`
//! - `collab_broadcast_delta`
//! - `collab_get_peers`

use crate::ShellState;
use crdt_collab::{
    CollabTransportBus, EncryptionKey, FugueDelta, GridDelta, PeerPresence, TransportMode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSessionInfo {
    pub session_id: String,
    pub client_id: u64,
    pub username: String,
    pub app_mode: String,
    pub status: String,
    pub transport_mode: String,
    pub peers: Vec<PeerPresence>,
}

pub fn collab_join_session(
    state: &ShellState,
    session_id: &str,
    username: &str,
    app_mode: &str,
    room_key: Option<&str>,
    transport_mode: Option<&str>,
) -> Result<CollabSessionInfo, String> {
    if session_id.trim().is_empty() {
        return Err("Session ID cannot be empty".to_string());
    }
    if username.trim().is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    let mut sessions = state
        .collab_sessions
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    let mode = match transport_mode
        .unwrap_or("webrtc")
        .to_ascii_lowercase()
        .as_str()
    {
        "websocket" | "ws" | "relay" => TransportMode::WebSocketRelay,
        "direct" | "loopback" => TransportMode::LoopbackDirect,
        _ => TransportMode::WebRtcDataChannel,
    };

    let client_id = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&username, &mut hasher);
        std::hash::Hash::hash(&session_id, &mut hasher);
        let id = std::hash::Hasher::finish(&hasher);
        if id == 0 {
            1
        } else {
            id
        }
    };

    let mut bus = CollabTransportBus::new(client_id, session_id, username, app_mode, mode);

    let key = if let Some(k) = room_key {
        EncryptionKey::derive_from_passphrase(k, b"aether-collab-salt-v1")
    } else {
        EncryptionKey::derive_from_passphrase("aether-default-passphrase", b"aether-collab-salt-v1")
    };
    bus.set_encryption_key(key);

    bus.connect()
        .map_err(|e| format!("Failed to initiate handshake: {e}"))?;

    let info = CollabSessionInfo {
        session_id: session_id.to_string(),
        client_id,
        username: username.to_string(),
        app_mode: app_mode.to_string(),
        status: format!("{:?}", bus.status()),
        transport_mode: format!("{:?}", bus.mode()),
        peers: bus.get_peers(),
    };

    sessions.insert(session_id.to_string(), bus);
    Ok(info)
}

pub fn collab_leave_session(
    state: &ShellState,
    session_id: &str,
    reason: Option<&str>,
) -> Result<bool, String> {
    let mut sessions = state
        .collab_sessions
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    if let Some(mut bus) = sessions.remove(session_id) {
        let _ = bus.disconnect(reason.unwrap_or("User departed"));
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn collab_broadcast_delta(
    state: &ShellState,
    session_id: &str,
    delta_type: &str,
    delta_json: &str,
) -> Result<bool, String> {
    let mut sessions = state
        .collab_sessions
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    let bus = sessions
        .get_mut(session_id)
        .ok_or_else(|| format!("Collab session '{session_id}' not found"))?;

    match delta_type.to_ascii_lowercase().as_str() {
        "text" => {
            let delta: FugueDelta = serde_json::from_str(delta_json)
                .map_err(|e| format!("Invalid FugueDelta JSON: {e}"))?;
            bus.broadcast_text_delta(delta)
                .map_err(|e| format!("Broadcast error: {e}"))?;
            Ok(true)
        }
        "grid" => {
            let delta: GridDelta = serde_json::from_str(delta_json)
                .map_err(|e| format!("Invalid GridDelta JSON: {e}"))?;
            bus.broadcast_grid_delta(delta)
                .map_err(|e| format!("Broadcast error: {e}"))?;
            Ok(true)
        }
        other => Err(format!(
            "Unknown delta type: '{other}' (expected 'text' or 'grid')"
        )),
    }
}

pub fn collab_get_peers(state: &ShellState, session_id: &str) -> Result<Vec<PeerPresence>, String> {
    let sessions = state
        .collab_sessions
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;

    let bus = sessions
        .get(session_id)
        .ok_or_else(|| format!("Collab session '{session_id}' not found"))?;

    Ok(bus.get_peers())
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_collab_join_session(
    state: tauri::State<ShellState>,
    session_id: String,
    username: String,
    app_mode: String,
    room_key: Option<String>,
    transport_mode: Option<String>,
) -> Result<CollabSessionInfo, String> {
    collab_join_session(
        &state,
        &session_id,
        &username,
        &app_mode,
        room_key.as_deref(),
        transport_mode.as_deref(),
    )
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_collab_leave_session(
    state: tauri::State<ShellState>,
    session_id: String,
    reason: Option<String>,
) -> Result<bool, String> {
    collab_leave_session(&state, &session_id, reason.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_collab_broadcast_delta(
    state: tauri::State<ShellState>,
    session_id: String,
    delta_type: String,
    delta_json: String,
) -> Result<bool, String> {
    collab_broadcast_delta(&state, &session_id, &delta_type, &delta_json)
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_collab_get_peers(
    state: tauri::State<ShellState>,
    session_id: String,
) -> Result<Vec<PeerPresence>, String> {
    collab_get_peers(&state, &session_id)
}
