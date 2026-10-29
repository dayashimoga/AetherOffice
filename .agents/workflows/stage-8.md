description: Automated execution workflow for Stage 8: De-Clubbing Architecture, 5 Standalone Binaries, File I/O & Contextual Ribbons
Workflow: Execute Stage 8 (/stage-8)
Objective: Stage 8: De-Clubbing Architecture, 5 Standalone Binaries, File I/O & Contextual Ribbons

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 8.1.1: 5-Executable Build Wiring & App-Mode Boot Router
Spec File: .agents/stages/stage-8/subphase-8.1.1.md
/execute-subphase 8.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. In pps/unified-shell/src-tauri/Cargo.toml, register the 5 [[bin]] targets (ether-hub, ether-writer, ether-calc, ether-slides, ether-pdf) pointing to src/bin/*.rs.
2. Expose a Tauri IPC command get_boot_config() in pps/unified-shell/src-tauri/src/lib.rs that returns { app_mode: "hub" | "writer" | "calc" | "slides" | "pdf", initial_file: Option<String> } read from AETHER_APP_MODE and CLI std::env::args().
3. Remove the hardcoded boot tabs (Document 1 *, Financials.xlsx *, Pitch.pptx *) from pps/unified-shell/src/. When launched as ether-writer, mount ONLY the Writer App shell with a clean Untitled 1.docx; when ether-calc, mount ONLY Calc with Book1.xlsx; when ether-slides, mount ONLY Slides with Presentation1.pptx; when ether-pdf, mount ONLY PDF Studio.
4. Write a Vitest test asserting each boot mode mounts strictly its own isolated application shell and zero cross-format tabs. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 8.1.1
Subphase 8.1.2: AetherHub Main Suite Launcher & OS Process Spawner
Spec File: .agents/stages/stage-8/subphase-8.1.2.md
/execute-subphase 8.1.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. Build the AetherHub main application view (pps/unified-shell/src/apps/hub/HubLauncher.tsx) displayed when pp_mode == "hub":
   - 4 Prominent App Launch Cards: **Writer (Word .docx)**, **Calc (Excel .xlsx)**, **Slides (PowerPoint .pptx)**, and **PDF Studio (.pdf)**.
   - Template Gallery (Blank Document, Financial Budget Sheet, Pitch Deck, Blank/Form PDF) and Recent Files list persisted in local storage / OS config dir.
2. Implement the Tauri command launch_standalone_app(app_mode: String, file_path: Option<String>) in src-tauri/src/lib.rs that spawns a separate native OS window (	auri::WebviewWindowBuilder) or child process for the chosen app, AND allow switching/managing open windows from the Hub.
3. Verify with automated tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 8.1.2
Subphase 8.2.1: Native File Open, Save, Save-As & Backstage File Menu
Spec File: .agents/stages/stage-8/subphase-8.2.1.md
/execute-subphase 8.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v2.0.
1. Fix the broken File tab in the ribbon: clicking File MUST open a full Backstage overlay menu containing:
   - **New** (Blank Document / Sheet / Presentation / PDF, plus reset current state).
   - **Open...** (Triggers native OS file picker or path input, reads .docx, .xlsx, .pptx, or .pdf bytes from disk through word-engine, excel-engine, ppt-engine, or pdf-engine, and populates the live canvas).
   - **Save / Save As...** (Serializes the current in-memory Rust engine state back to a valid .docx, .xlsx, .pptx, or .pdf file on disk and clears the dirty * indicator).
   - **Export to PDF** (Renders the current document/sheet/slide deck into a valid ISO 32000 PDF via pdf-engine).
2. Write integration tests verifying round-trip disk Open -> Edit -> Save for all 4 formats. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 8.2.1
Subphase 8.2.2: Strict Per-App Contextual Ribbon Router & Mobile Bar Cleanup
Spec File: .agents/stages/stage-8/subphase-8.2.2.md
/execute-subphase 8.2.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.2.
1. Fix the Ribbon Tab state bug seen in screenshots where File stays underlined while Home is shown:
   - Maintain a strict ctiveRibbonTab state per window (Home, Insert, Layout, etc.) where clicking any tab updates the blue active indicator AND immediately switches the toolbar panel below it.
2. Separate the Ribbon definitions into 4 isolated components (WriterRibbon.tsx, CalcRibbon.tsx, SlidesRibbon.tsx, PdfRibbon.tsx):
   - **Writer ONLY:** Home, Insert, Layout, References, Review, View.
   - **Calc ONLY:** Home, Insert, Page Layout, Formulas, Data, Review, View.
   - **Slides ONLY:** Home, Insert, Design, Transitions, Animations, Slide Show, Review, View (Remove Word's Normal/Heading 1 styles from Slides!).
   - **PDF ONLY:** Home, Edit PDF, Forms, Comment & Annotate, Page Tools, Sign & Protect, View.
3. Hide the mobile bottom drawer handle and bottom touch bar whenever window.innerWidth >= 768px.
4. Verify with DOM tests asserting every tab click renders unique, non-placeholder buttons. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 8.2.2
