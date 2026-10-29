# Subphase 14.2.1: Encrypted Transport Bus (WebRTC DataChannel & WebSocket Relay)
- **Stage:** 14 | **Phase:** 14.2
- **Target Subsystem:** `crdt-collab` & `unified-shell-tauri`
- **Primary Agent Skill:** `@network-engineer`
- **Deliverable:** End-to-end encrypted packet protocol, local peer discovery, room connection handshake, delta compaction, state vector exchange.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 14.2.1
Invoke @network-engineer under GEC-v2.0:
1. In `crates/crdt-collab`, create `CollabTransportBus`:
   - Binary packet encoding using bincode/protobuf with ChaCha20-Poly1305 / AES-GCM envelope encryption.
   - Dual-mode transport: direct WebRTC DataChannel (P2P LAN) with fallback WebSocket relay client.
   - State vector exchange protocol (`SyncStep1: StateVector`, `SyncStep2: MissingDeltas`, `Update: DeltaBatch`).
   - Ephemeral presence messages (cursor position, selection ranges, username, active tab).
2. Wire Tauri IPC commands in `apps/unified-shell/src-tauri/src/collab_commands.rs` for `collab_join_session`, `collab_leave_session`, `collab_broadcast_delta`, `collab_get_peers`.
3. Add tests verifying connection negotiation, message serialization, and delta reassembly.
4. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
