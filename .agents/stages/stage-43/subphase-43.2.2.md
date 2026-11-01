# Subphase 43.2.2: WASM Core Compilation for Browser/Test Fallback

- **Stage:** 43 | **Phase:** 43.2
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@wasm-architect`
- **Deliverables:**
  1. Implement `wasm_bridge` module in `crates/render-core/src/wasm_bridge.rs`:
     - Safe, zero-panic Rust API bridge for WebAssembly & headless browser execution.
     - Document creation: `wasm_create_session(doc_type_str: &str, title: &str) -> Result<String, String>`.
     - Byte ingestion: `wasm_open_document(bytes: &[u8], file_name: &str) -> Result<String, String>`.
     - Action dispatch: `wasm_dispatch_action(session_id: &str, action_json: &str) -> Result<String, String>`.
     - Document export & serialization: `wasm_save_document(session_id: &str) -> Result<Vec<u8>, String>`.
     - Display SVG export: `wasm_export_svg(session_id: &str) -> Result<String, String>`.
     - Lossless OPC roundtrip: `wasm_roundtrip_opc(format: &str, original_bytes: &[u8], updates_json: &str) -> Result<Vec<u8>, String>`.
     - Expose typed structs with serde serialization for seamless JavaScript/TypeScript interop.
  2. Register `wasm_bridge` in `crates/render-core/src/lib.rs`.
  3. Implement WASM Core bridge fallback loader in `apps/unified-shell/src/wasm_bridge.ts` and integrate with `apps/unified-shell/src/ipc.ts`:
     - Dynamic fallback engine providing high-fidelity document state simulation and SVG generation when running outside native Tauri.
     - Integration into `IPC` client ensuring 100% feature parity between Desktop Tauri and Web preview modes.
  4. Comprehensive Test Coverage:
     - Rust unit & integration test suite in `crates/render-core/tests/subphase_43_2_2_test.rs`:
       - Document session creation and metadata extraction.
       - Binary package ingestion and serialization roundtrip.
       - UI action dispatch and undo/redo state tracking.
       - Zero-loss OPC part preservation via WASM bridge.
     - Frontend Vitest suite in `apps/unified-shell/src/__tests__/wasm_core_bridge.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

