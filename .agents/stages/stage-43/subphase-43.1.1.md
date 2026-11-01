# Subphase 43.1.1: Complete Tauri Command Registration & Binary Buffer Bridge

- **Stage:** 43 | **Phase:** 43.1
- **Target Subsystem:** `unified-shell-tauri` / `ipc.ts` / `render-core`
- **Primary Agent Skill:** `@rust-ipc-architect`
- **Deliverables:**
  1. Update `apps/unified-shell/src-tauri/Cargo.toml` with all 12 workspace crates (`pdf-engine`, `plugin-runtime`, `crypto-security`, `data-connectors`, `copilot-core`, `ooxml-opc`, `word-engine`, `excel-engine`, `ppt-engine`, `render-core`, `crdt-collab`, `macro-engine`).
  2. Implement and register all core Tauri commands in `apps/unified-shell/src-tauri/src/lib.rs` via `tauri::generate_handler!`:
     - Session & document commands (`create_session`, `open_document`, `close_session`, `get_document_info`, `save_document`).
     - Undo / Redo & action dispatch (`execute_undo`, `execute_redo`, `dispatch_ui_action`).
     - Window & instance management (`spawn_app_window`, `launch_standalone_app`, `list_app_instances`, `close_app_instance`, `focus_app_instance`).
     - Export commands (`export_pdf`, `export_svg`).
     - Zero-copy Binary Buffer Bridge commands:
       - `read_file_binary` (raw byte vector / binary response).
       - `save_file_binary` (binary byte payload).
       - `render_pdf_page_binary` (PDF page rendering to binary buffer).
       - `crypto_hash_binary` (cryptographic hashing via `crypto-security`).
       - `collab_sync_binary` (binary delta exchange via `crdt-collab`).
       - `macro_execute_code` (macro sandboxing via `macro-engine`).
       - `copilot_query` (prompt synthesis via `copilot-core`).
  3. Wire frontend binary IPC client in `apps/unified-shell/src/ipc.ts` with browser mock fallbacks.
  4. Comprehensive unit test suites in `apps/unified-shell/src/__tests__/binary_ipc_bridge.test.ts` and Rust unit tests in `src-tauri`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

