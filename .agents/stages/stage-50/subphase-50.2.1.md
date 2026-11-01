# Stage 50: Integrated Office Suite Commercial Ecosystem
## Sub-Phase 50.2.1: Cross-Format Conversion Pipeline & Batch Exporter

### 1. Context & Objective
In GEC-v7.0 Article I (Architectural Integrity & Single Source of Truth in Rust) and Article III (Real Binary File Open & Save, Zero Data Loss), users require seamless high-throughput batch and single-file conversions across all productivity formats:
- Word (`.docx`) <-> Markdown (`.md`), HTML, Plain Text (`.txt`), and PDF (`.pdf`).
- Excel (`.xlsx`) <-> CSV (`.csv`), JSON (`.json`), HTML Table, and PDF (`.pdf`).
- PowerPoint (`.pptx`) <-> PDF presentation deck (`.pdf`) and SVG slide bundles.
- Industrial batch exporter: Queue multiple documents across formats, execute batch transformation with progress streaming, cancellation support, and comprehensive per-file conversion diagnostics (`BatchConversionReport`).

### 2. Implementation Scope
1. **Rust Domain Crate (`crates/render-core/src/conversion.rs`):**
   - Pure safe Rust `#![forbid(unsafe_code)]` with zero `.unwrap()`, `.expect()`, or panics in production library code.
   - `ConversionFormat`: `Docx`, `Xlsx`, `Pptx`, `Pdf`, `Markdown`, `Csv`, `Html`, `PlainText`, `Json`.
   - `ConversionOptions`: delimiter, include_metadata, preserve_tables, page_orientation, pretty_print.
   - `ConversionTask`: task ID, input format, output format, input data (bytes or path), options.
   - `ConversionResult`: task ID, success, input format, output format, output bytes, output path, duration_ms, error.
   - `BatchConversionReport`: total_tasks, succeeded, failed, results, duration_total_ms.
   - `CrossFormatConverter`: Core engine performing deterministic conversions across text, tabular, and document formats.
   - Register `pub mod conversion;` and `pub use conversion::*;` in `crates/render-core/src/lib.rs`.

2. **IPC Layer (`apps/unified-shell/src-tauri` & `src/ipc.ts`):**
   - Tauri commands:
     - `conversion_convert_single`
     - `conversion_batch_convert`
     - `conversion_supported_formats`
     - `conversion_estimate_job`
   - IPC DTO types and browser preview / mock handlers in `ipc.ts`.

3. **Presentation Shell Coordinator (`apps/unified-shell/src/state/ConversionCoordinator.ts`):**
   - Batch conversion queue, progress tracking, per-file status, format compatibility check, and export options.
   - Export in `apps/unified-shell/src/state/index.ts`.

4. **Test Suites & Verification:**
   - Rust integration test suite: `crates/render-core/tests/subphase_50_2_1_test.rs` (10 tests).
   - Vitest suite: `apps/unified-shell/src/__tests__/conversion_coordinator.test.ts` (10 tests).
   - Gatekeeper verification: `cargo fmt`, `cargo clippy`, `tools/verify-contract.ps1` exit code 0.

