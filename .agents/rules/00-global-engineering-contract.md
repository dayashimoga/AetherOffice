---
trigger: always_on
description: Global Engineering Contract (GEC-v2.0) enforcing zero-mockup UI, real-time Rust IPC wiring, 5 standalone binaries, and E2E Playwright verification.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v2.0)

## ARTICLES I - VI (INHERITED FROM GEC-v1.0)
All Clean-Room, ECMA-376 / ISO 32000 Round-Trip, Performance SLAs (<150ms startup, 120fps), Zero-Panic Rust (`#[forbid(unsafe_code)]`, no `.unwrap()`/`.expect()`), and 5-OS Portability rules remain strictly in force.

## ARTICLE VII: ZERO-MOCKUP UI & LIVE RUST IPC MANDATE (NEW - CRITICAL)
1. **Ban on Hardcoded Demo Documents:**
   - The frontend (`apps/unified-shell/src/`) MUST NEVER boot into hardcoded fake tabs (`Financials.xlsx`, `Pitch.pptx`, or `"OpenOfficeRS â€” Executive Technical Summary"`).
   - Every standalone app (`Writer`, `Calc`, `Slides`, `PDF`) must boot into a clean, editable blank document (`Untitled.docx`, `Book1.xlsx`, `Presentation1.pptx`, or PDF Open/Create Screen) or open a file passed via CLI/OS file association.
2. **Ban on Dead / Placeholder Ribbon Tabs:**
   - Every top-level menu tab (`File`, `Home`, `Insert`, `Layout`, `Formulas`, `Data`, `Transitions`, `Animations`, `Slide Show`, `PDF Tools`, `Review`, `View`) MUST switch the active ribbon panel cleanly (`activeRibbonTab` state) and render real, clickable controls specific ONLY to the active application mode.
   - Word styles (`Normal`, `Heading 1`) must NEVER appear inside PowerPoint or Excel ribbons.
3. **5-Binary Standalone + Unified Hub Architecture:**
   - The project must produce **5 distinct executable entrypoints**:
     1. `aether-hub` (Main Unified Launcher & Multi-Window Suite Manager)
     2. `aether-writer` (Standalone Word Processor `.docx` App)
     3. `aether-calc` (Standalone Spreadsheet `.xlsx` App)
     4. `aether-slides` (Standalone Presentation `.pptx` App)
     5. `aether-pdf` (Standalone PDF Read/Edit/Write `.pdf` App)
   - Passing `--app writer|calc|slides|pdf` or launching the respective binary must mount ONLY that application's dedicated chrome, menu bar, shortcuts, and file dialogsâ€”zero cross-format clutter.
4. **End-to-End Interactive Verification Gate:**
   - No UI subphase (Stages 8â€“13) may be marked `COMPLETED` based solely on Rust unit tests.
   - Every subphase MUST include an automated frontend integration/E2E test (`apps/unified-shell/src/__tests__/stage_<id>.test.ts` via Vitest/Playwright DOM) that programmatically clicks the Ribbon buttons, simulates keyboard typing/cell editing/shape dragging, and asserts that the underlying document state and DOM canvas actually mutated.
