---
trigger: always_on
description: Global Engineering Contract (GEC-v7.0) enforcing Unified Multi-Document Split-Pane Workspace, Rust Single-Source-of-Truth IPC, Word Header/Body/Footer Isolation, Real Binary File Round-Tripping (.docx, .xlsx, .pptx, .pdf, .txt, .md, .csv, .odt, .ods, .odp), Zero-Panic Rust Invariants, WCAG 2.1 AA Accessibility, 120fps Virtualized Rendering, and 5-Platform Release Parity.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v7.0)

## ARTICLE I: ARCHITECTURAL INTEGRITY & SINGLE SOURCE OF TRUTH IN RUST
1. **Rust Core Authority:**
   - All document parsing, DOM/grid storage, formula evaluation, typographic layout, search indexing, and file serialization reside authoritatively in Rust crates (`crates/*`) and are exposed to the UI via typed Tauri IPC commands.
   - The frontend TypeScript layer acts as a high-performance reactive presentation shell (dockable split panes, virtualized canvas surface, accessibility shadow DOM), eliminating the "Split-Brain" anti-pattern.
2. **Instance-Scoped State (Zero Global Document Singletons):**
   - Every open document is assigned a unique `DocumentSessionId` (UUIDv7) managed by `DocumentSessionRegistry`.
   - Global mutable singletons holding single-document state are strictly prohibited. Multiple documents of the same or different formats can be opened, rendered, and edited concurrently side-by-side.
3. **Format-Agnostic Workspace & Dockable Split Panes:**
   - The UI shell must support opening any combination of `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.md`, and `.csv` tabs in a shared, unified workspace.
   - Users can split the workspace into 2-way or 4-way dockable editor panes with focus-driven contextual ribbon switching.

## ARTICLE II: WORD (.DOCX) HEADER / BODY / FOOTER STRICT ISOLATION
1. **Three Distinct Data Zones Per Page:**
   - Every Word page object MUST maintain three strictly separate fields: `headerHtml: string`, `bodyHtml: string`, and `footerHtml: string`.
   - **NEVER** bind the Header container and the Body container to the same state variable, and **NEVER** render a duplicate preview of the body text at the top of the page!
2. **Header/Footer Double-Click Lock & Explicit Exit:**
   - By default, `activeEditZone` MUST be `"body"`. Typing in the document edits ONLY `bodyHtml`.
   - The Header and Footer zones are read-only until the user **double-clicks** the Header/Footer area or clicks `Insert -> Header` / `Insert -> Footer`.
   - When `activeEditZone` is `"header"` or `"footer"`, display a prominent floating **`[✓ Close Header & Footer (Esc)]`** button that immediately returns focus to the main page `"body"`.

## ARTICLE III: REAL BINARY FILE OPEN & SAVE FOR ALL FORMATS (ZERO DATA LOSS)
1. **Opening Existing Files (`Ctrl+O` / `Open`):**
   - **`.docx`:** Must unzip and parse via Rust `ooxml-opc` / `word-engine` IPC, extracting all text/formatting/tables from `word/document.xml`, `word/header*.xml`, and `word/footer*.xml`, preserving unmodeled parts in `UnparsedPartStore`.
   - **`.xlsx` / `.csv`:** Must extract cell coordinates, shared strings, numbers, and formulas into the CSPG grid.
   - **`.pptx`:** Must extract slide shapes, geometry, text, master cascades, and animations.
   - **`.pdf`:** Must parse PDF object streams, linearized XRef, font CMaps, and AcroForms onto the interactive PDF Studio canvas.
   - **`.txt` / `.md`:** Must read UTF-8 text and populate the editor immediately.
2. **Saving Files (`Ctrl+S` / `Save` / `Save As`):**
   - Must execute in BOTH native Tauri Desktop (writing directly to disk via `save_document_bytes` with path notification toast) AND browser download fallback.
   - Zero-loss round-trip guarantee: Unmodeled XML nodes, attributes, relationships, macros (`vbaProject.bin`), printer configurations, and digital signatures are never stripped or corrupted.

## ARTICLE IV: CODE SAFETY, ZERO-PANIC INVARIANTS & CLEAN-ROOM PROVENANCE
1. **Rust Safety Invariants:**
   - `#![forbid(unsafe_code)]` enforced across all domain crates.
   - Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in non-test library code.
   - Zero compiler warnings under `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.
2. **Clean-Room Isolation:**
   - Pure ECMA-376, ISO 32000-2, OASIS ODF, and CommonMark open standard implementation. Zero proprietary Microsoft internal symbols or decompiled assembly.

## ARTICLE V: VISUAL ERGONOMICS, ACCESSIBILITY (a11y) & THEMES
1. **100% Dark / Light / High-Contrast Theme Coverage:**
   - Every single panel—including PDF Studio's sidebar, canvas surrounds, Excel sheet tabs, Word ruler, modals, and dropdowns—MUST inherit semantic CSS design tokens. Zero unstyled white panels in Dark Mode.
2. **Zero Dead UI Controls:**
   - Every button, dropdown item, context menu action, and keyboard shortcut rendered in the UI must execute a real, state-mutating command with immediate feedback. Placeholder stubs are contract violations.
3. **WCAG 2.1 AA Compliance:**
   - High-contrast text ratios, visible focus indicators, full keyboard operability, and a synchronized virtualized ARIA Shadow DOM overlay for screen readers on all canvas viewports.

## ARTICLE VI: PERFORMANCE BUDGETS & SLAs
- **Desktop Startup Latency:** $< 100\text{ ms}$ cold start.
- **Idle Memory Footprint:** $< 35\text{ MB}$ RSS.
- **Active Document Memory:** $< 250\text{ MB}$ RSS for 20 open documents using Active/Warm/Cold LRU session suspension.
- **Keystroke / Cell Edit Latency:** $< 1.0\text{ ms}$ p99 on 1,000-page documents.
- **120fps Rendering Frame Budget:** $\le 8.33\text{ ms}$ per frame under continuous scrolling.

## ARTICLE VII: NON-DESTRUCTIVE REGRESSION & GATEKEEPING
1. **Sub-Phase Execution Order:**
   - Execution proceeds strictly one sub-phase at a time.
   - All existing unit, integration, and E2E test suites must continue to pass 100% with zero regressions.
2. **Automated Gatekeeper:**
   - `tools/verify-contract.ps1` must exit `0` before any sub-phase can be marked `COMPLETED` in `specs/stage-tracker.json`.
