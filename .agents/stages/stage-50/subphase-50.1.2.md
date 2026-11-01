# Stage 50: Integrated Office Suite Commercial Ecosystem
## Sub-Phase 50.1.2: Bidirectional [[WikiLinks]] & Backlinks Across Notes, Word & PDFs

### 1. Context & Objective
In GEC-v7.0 Article I (Architectural Integrity & Single Source of Truth in Rust) and Article III (Zero Data Loss), AetherOffice functions as a unified, hyper-connected knowledge workspace. 
Sub-Phase 50.1.2 delivers an industrial-grade **Bidirectional [[WikiLinks]] & Backlinks Engine**:
- Fast, clean-room parsing of `[[Document Title]]`, `[[Document Title|Display Text]]`, and `[[Document Title#Anchor|Display Text]]` wikilinks across Notes (Markdown), Word documents (`.docx`), and PDF text/annotations.
- High-performance in-memory backlink index (`WikiLinkIndex`) in `render-core`:
  - Bidirectional link graph mapping source documents to target documents and target anchors.
  - Linked references with snippet extraction (surrounding context).
  - Unlinked mentions discovery (identifying candidate text in documents matching known document titles).
  - Autocomplete / suggestion provider for omnibox and inline typing triggers (`[[`).
- Tauri IPC bridge with strongly-typed commands.
- Presentation shell coordinator (`WikiLinkCoordinator.ts`) with reactive state management, linked reference panels, and mention conversions.

### 2. Implementation Scope
1. **Rust Domain Crate (`crates/render-core/src/wikilinks.rs`):**
   - Pure safe Rust `#![forbid(unsafe_code)]` with zero `.unwrap()`, `.expect()`, or panics in production library code.
   - `WikiLink`: target document title/id, optional heading anchor, optional display alias, character span.
   - `WikiLinkParser`: Extracts wikilinks with regex or deterministic scanner.
   - `BacklinkReference`: source document id, source title, link span, snippet (context before & after).
   - `UnlinkedMention`: source document id, candidate title, snippet, character offset.
   - `WikiLinkIndex`:
     - `index_document(doc_id: &str, title: &str, content: &str)`
     - `remove_document(doc_id: &str)`
     - `get_backlinks(target_title_or_id: &str) -> Vec<BacklinkReference>`
     - `get_forward_links(source_doc_id: &str) -> Vec<WikiLink>`
     - `suggest_targets(query: &str, limit: usize) -> Vec<WikiLinkSuggestion>`
     - `find_unlinked_mentions(target_title: &str) -> Vec<UnlinkedMention>`
     - `stats() -> WikiLinkIndexStats`
   - Register `pub mod wikilinks;` in `crates/render-core/src/lib.rs`.

2. **IPC Layer (`apps/unified-shell/src-tauri` & `src/ipc.ts`):**
   - Add state-managed `pub wikilink_index: Mutex<render_core::WikiLinkIndex>` to `ShellState`.
   - Expose Tauri commands:
     - `wikilink_index_document`
     - `wikilink_remove_document`
     - `wikilink_get_backlinks`
     - `wikilink_get_forward_links`
     - `wikilink_suggest_targets`
     - `wikilink_find_unlinked_mentions`
     - `wikilink_parse_links`
     - `wikilink_stats`
   - Add TypeScript types and mock handlers in `apps/unified-shell/src/ipc.ts`.

3. **Presentation Shell Coordinator (`apps/unified-shell/src/state/WikiLinkCoordinator.ts`):**
   - Manages active document backlink state, linked references, unlinked mentions, target suggestions, and insertion formatting.
   - Export in `apps/unified-shell/src/state/index.ts`.

4. **Test Suites & Verification:**
   - Rust integration test suite: `crates/render-core/tests/subphase_50_1_2_test.rs` (10 tests).
   - Vitest suite: `apps/unified-shell/src/__tests__/wikilink_coordinator.test.ts` (10 tests).
   - Automated gatekeeper: `cargo fmt`, `cargo clippy`, `tools/verify-contract.ps1` exit code 0.

