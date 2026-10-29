# Subphase 8.1.2: AetherHub Main Suite Launcher & OS Process Spawner
- **Stage:** 8 | **Phase:** 8.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Main Suite Hub window with App Launch Cards (Word, Excel, PowerPoint, PDF), Recent Files, Templates, and OS child-process spawning.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 8.1.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. Build the AetherHub main application view (pps/unified-shell/src/apps/hub/HubLauncher.tsx) displayed when pp_mode == "hub":
   - 4 Prominent App Launch Cards: **Writer (Word .docx)**, **Calc (Excel .xlsx)**, **Slides (PowerPoint .pptx)**, and **PDF Studio (.pdf)**.
   - Template Gallery (Blank Document, Financial Budget Sheet, Pitch Deck, Blank/Form PDF) and Recent Files list persisted in local storage / OS config dir.
2. Implement the Tauri command launch_standalone_app(app_mode: String, file_path: Option<String>) in src-tauri/src/lib.rs that spawns a separate native OS window (	auri::WebviewWindowBuilder) or child process for the chosen app, AND allow switching/managing open windows from the Hub.
3. Verify with automated tests and commit with [GEC-Verified].
`
