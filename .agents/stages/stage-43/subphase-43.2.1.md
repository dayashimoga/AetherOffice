# Subphase 43.2.1: Non-Blocking Async File Ingestion Pipeline

- **Stage:** 43 | **Phase:** 43.2
- **Target Subsystem:** `ooxml-opc` / `render-core` / `unified-shell-tauri` / `ipc.ts`
- **Primary Agent Skill:** `@async-systems-architect`
- **Deliverables:**
  1. Implement `async_pipeline` module in `crates/ooxml-opc`:
     - `IngestionStage` enum (`ReadingCentralDirectory`, `ResolvingContentTypes`, `ParsingRelationships`, `ExtractingParts`, `BuildingDom`, `Complete`, `Cancelled`, `Failed`).
     - `IngestionProgress` struct with percentage calculation, byte counters, and current part tracking.
     - `CancellationToken` for cooperative thread-safe aborts.
     - `AsyncIngestionPipeline` capable of streaming OPC package decompression with chunked progress reporting.
  2. Implement async ingestion in `crates/render-core::document_io`:
     - `open_document_async` accepting `CancellationToken` and progress callback.
     - Non-blocking streaming ingestion for DOCX, XLSX, PPTX, and PDF.
  3. Expose `open_document_async` and `cancel_ingestion` in `apps/unified-shell/src-tauri`:
     - Register `open_document_async` and `cancel_ingestion` commands.
     - Support progress event dispatch (`ingestion-progress`) to Tauri webviews.
  4. Implement `IPC.openDocumentAsync` and `IPC.cancelIngestion` in `apps/unified-shell/src/ipc.ts` with browser mock fallbacks.
  5. Comprehensive tests:
     - Unit tests in `crates/ooxml-opc/tests/subphase_43_2_1_test.rs` validating cancellation, progress stages, and chunked streaming.
     - Vitest suite in `apps/unified-shell/src/__tests__/async_file_ingestion.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

