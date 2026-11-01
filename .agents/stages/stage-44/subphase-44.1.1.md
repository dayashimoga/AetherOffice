# Subphase 44.1.1: Incremental Multi-Format Document Indexer

- **Stage:** 44 | **Phase:** 44.1
- **Target Subsystem:** `copilot-core` / `render-core` / `unified-shell`
- **Primary Agent Skill:** `@search-engine-architect`
- **Deliverables:**
  1. Implement `document_indexer` in `crates/copilot-core/src/document_indexer.rs`:
     - Multi-format ingestion adapter for DOCX, XLSX, PPTX, PDF, and Markdown/TXT documents.
     - Incremental re-indexing pipeline:
       - Computes content hashes per chunk/block to bypass re-embedding unchanged sections.
       - Purges outdated chunks automatically when document updates are applied.
     - Hybrid lexical (BM25-style term frequency) + semantic (dense cosine similarity) ranking.
     - Structured metadata filters by document format, title, modification timestamp, and tag.
  2. Expose `document_indexer` module in `crates/copilot-core/src/lib.rs`.
  3. Tauri IPC Integration & Frontend Bridge:
     - Register `workspace_search` in `apps/unified-shell/src-tauri` and `apps/unified-shell/src/ipc.ts`.
     - Support instant search across all currently open and indexed documents.
  4. Comprehensive Test Coverage:
     - Rust integration test suite in `crates/copilot-core/tests/subphase_44_1_1_test.rs`:
       - Multi-format document indexing (DOCX, XLSX, PPTX, PDF, Markdown).
       - Incremental update efficiency (verifying unchanged chunks are preserved).
       - Hybrid search accuracy and ranking score verification.
       - Document deletion and index compaction.
     - Frontend Vitest suite in `apps/unified-shell/src/__tests__/document_indexer.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

