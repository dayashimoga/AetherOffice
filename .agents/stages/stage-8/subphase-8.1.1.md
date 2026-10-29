# Subphase 8.1.1: 5-Executable Build Wiring & App-Mode Boot Router
- **Stage:** 8 | **Phase:** 8.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 5 compiled binaries (aether-hub, aether-writer, aether-calc, aether-slides, aether-pdf) routing directly into isolated app shells.
- **Validation Command:** `cargo build --manifest-path apps/unified-shell/src-tauri/Cargo.toml --bins`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 8.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. In pps/unified-shell/src-tauri/Cargo.toml, register the 5 [[bin]] targets (ether-hub, ether-writer, ether-calc, ether-slides, ether-pdf) pointing to src/bin/*.rs.
2. Expose a Tauri IPC command get_boot_config() in pps/unified-shell/src-tauri/src/lib.rs that returns { app_mode: "hub" | "writer" | "calc" | "slides" | "pdf", initial_file: Option<String> } read from AETHER_APP_MODE and CLI std::env::args().
3. Remove the hardcoded boot tabs (Document 1 *, Financials.xlsx *, Pitch.pptx *) from pps/unified-shell/src/. When launched as ether-writer, mount ONLY the Writer App shell with a clean Untitled 1.docx; when ether-calc, mount ONLY Calc with Book1.xlsx; when ether-slides, mount ONLY Slides with Presentation1.pptx; when ether-pdf, mount ONLY PDF Studio.
4. Write a Vitest test asserting each boot mode mounts strictly its own isolated application shell and zero cross-format tabs. Commit with [GEC-Verified].
`
