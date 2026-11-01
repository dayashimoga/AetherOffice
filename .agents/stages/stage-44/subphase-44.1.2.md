# Subphase 44.1.2: Cross-Document Federated Search & Replace Panel

- **Stage:** 44 | **Phase:** 44.1
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@ui-systems-architect`
- **Deliverables:**
  1. Implement `federated_search` in `crates/render-core/src/federated_search.rs`:
     - Cross-document search across all active tabs in `DesktopSessionManager` (Word, Excel, PowerPoint, PDF, Notes).
     - Search options: `case_sensitive: bool`, `whole_word: bool`, `use_regex: bool`.
     - Returns `FederatedSearchMatch`:
       - `session_id`, `doc_type`, `title`, `snippet` (with surrounding context), `location` (paragraph/cell/slide/page), `match_range`.
     - Cross-document atomic replace:
       - `replace_match(mgr, session_id, match_info, replacement) -> Result<ReplaceResult, String>`
       - `replace_all(mgr, query, replacement, options, target_session_ids) -> Result<FederatedReplaceSummary, String>`
       - Dispatches through transactional command buses with undo/redo preservation.
  2. Register `federated_search` module in `crates/render-core/src/lib.rs`.
  3. Tauri IPC Integration (`apps/unified-shell/src-tauri`):
     - Register `federated_search`, `federated_replace_match`, `federated_replace_all` commands in `lib.rs` and `tauri_commands.rs`.
  4. Frontend Search & Replace Panel (`apps/unified-shell/src/components/FederatedSearchPanel.ts` & `apps/unified-shell/src/ipc.ts`):
     - Interactive dockable search panel with match tree grouped by document tab.
     - Match clicking jumps to and focuses the matching tab and element.
     - "Replace" and "Replace All" controls with immediate feedback.
     - Toggleable via `Ctrl+Shift+F` and UI rail/toolbar.
  5. Comprehensive Test Coverage:
     - Rust integration test suite in `crates/render-core/tests/subphase_44_1_2_test.rs`:
       - Multi-document cross-format search (Word + Excel + Slides + Notes).
       - Case sensitivity, whole word, and regex search evaluation.
       - Atomic single match replacement and cross-document "Replace All" with undo/redo rollback.
     - Frontend Vitest suite in `apps/unified-shell/src/__tests__/federated_search_panel.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

