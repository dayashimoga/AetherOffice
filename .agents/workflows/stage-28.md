---
description: Automated execution workflow for Stage 28: Multi-Window Per Format, Full-App Theme, Ghost Sidebar Fix and Universal File IO
---

# Workflow: Execute Stage 28 (/stage-28)

## Objective: Stage 28: Multi-Window Per Format, Full-App Theme, Ghost Sidebar Fix and Universal File IO
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

### Subphase 28.1.1: Separate App Windows Per Document Type + Shared Session Manager
- **Spec File:** `.agents/stages/stage-28/subphase-28.1.1.md`
/execute-subphase 28.1.1
Invoke @adaptive-ui-builder under GEC-v4.0 Article I.
1. **Separate Window Per Document Type (Without RAM Hang):**
   - Implement Tauri multi-window routing (spawn_app_window(app_type: "word" | "excel" | "ppt" | "pdf" | "notes") in src-tauri/src/lib.rs using 	auri::WebviewWindowBuilder::new with ?app=<app_type> URL query parameter, plus an in-app multi-window workspace manager fallback when running in browser/test mode).
   - Each window filters its tab bar strictly to its own document format (e.g., the **Aether Word** window shows only .docx tabs; **Aether Excel** shows only .xlsx tabs; **Aether PowerPoint** shows only .pptx tabs; **Aether PDF** shows only .pdf tabs; **Aether Notes** shows only .txt/.md tabs).
   - Clicking an app icon (W, X, P, PDF, N) on the Left App Rail or Home Hub either focuses/spawns that app's dedicated window OR switches cleanly to that isolated app workspace, while a **Global Open Documents Drawer (All Open Files)** lists every active document across all 5 apps with live auto-save to localStorage/IndexedDB so zero work is ever lost!
   - Also add an "Open in New Window â†—" button on every tab so any document can be popped out into a separate OS window on demand.
2. Write automated Vitest tests verifying isolated per-app tab filtering, session persistence, and new window spawning. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 28.1.1`

### Subphase 28.1.2: Fix Ghost Word Sidebar Bug, Hide Alt-Badges & Fix Global Dark/Light Theme
- **Spec File:** `.agents/stages/stage-28/subphase-28.1.2.md`
/execute-subphase 28.1.2
Invoke @adaptive-ui-builder under GEC-v4.0 Article II.
1. **Fix Ghost Navigation Sidebar Bug (Seen in image_2586db.png & image_258017.png):**
   - Currently, Word's white Navigation sidebar is rendered globally outside the Word view, causing it to appear next to PDF's Pages sidebar and Notes' SCRATCHPADS sidebar!
   - Move Word's Navigation sidebar strictly inside the Word editor component so it NEVER renders when viewing Excel, PowerPoint, PDF, or Notes.
2. **Fix Permanent [F] [H] [N] [P] Badges on Ribbon Tabs:**
   - Hide the black Alt-keytip badges (F, H, N, P, S, R, W) on Ribbon tabs by default; only show them when ltKeyTipsActive === true (toggled by pressing and releasing Alt).
3. **Full-App Dark / Light Theme Cascade:**
   - Fix the theme toggle so switching between â˜€ï¸ Light and ðŸŒ™ Dark updates the entire application root, Titlebar, Tab Bar, Ribbon, Left Sidebar, Workspace background, Modals, and Status Bar (while keeping an optional Dark Page / Paper Canvas toggle in View for Word/PDF).
4. **Fix Hardcoded 22 words Status Bar:**
   - Replace the static 22 words | English (US) string at the bottom of the screen with dynamic per-app status indicators (Word: real live word/char/page count; Excel: active sheet count + selected cell SUM/AVG/COUNT; PPT: slide index + shape count; PDF: page index + page size in mm/in + encryption status; Notes: line/col/word/char count).
5. Write automated DOM tests verifying zero ghost sidebars in PDF/Notes, full-app theme class propagation, and live dynamic status bar counts. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 28.1.2`

### Subphase 28.2.1: Universal File Menu Backstage: Real Open, Save, Save As, Export & Print for All 5 Formats
- **Spec File:** `.agents/stages/stage-28/subphase-28.2.1.md`
/execute-subphase 28.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v4.0 Article III.
1. Build a unified, rock-solid File I/O service (pps/unified-shell/src/services/fileIoService.ts) connected to both the File Backstage menu AND top quick-action toolbar buttons (ðŸ“‚ Open, ðŸ’¾ Save, ðŸ–¨ï¸ Print):
   - **Open Existing File (Ctrl+O):** Triggers a hidden <input type="file"> (plus Tauri file dialog when in desktop runtime) supporting .docx, .doc, .html, .xlsx, .csv, .tsv, .pptx, .pdf, .txt, .md, and .json. Parses the file immediately and populates the active editor canvas!
   - **Save / Save As (Ctrl+S / Ctrl+Shift+S):** Serializes the current document state (Word HTML/OOXML package, Excel workbook/CSV, PPTX slide deck JSON/OOXML, PDF binary with edits, or TXT/MD string), writes it via Tauri FS or triggers an immediate browser file download with the exact filename, and clears the unsaved * indicator on the tab.
   - **Print (Ctrl+P):** Opens a dedicated clean print view / iframe containing ONLY the formatted document pages/sheet/slides/PDF/note and invokes window.print().
2. Write automated tests verifying File Open, Save, and Print across all 5 document types. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 28.2.1`

