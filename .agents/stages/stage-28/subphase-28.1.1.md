# Subphase 28.1.1: Separate App Windows Per Document Type + Shared Session Manager
- **Stage:** 28 | **Phase:** 28.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Dedicated windows per document format (Word Window, Excel Window, PPT Window, PDF Window, Notes Window) sharing 1 low-RAM process + Global Active Documents switcher.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 28.1.1
Invoke @adaptive-ui-builder under GEC-v4.0 Article I.
1. **Separate Window Per Document Type (Without RAM Hang):**
   - Implement Tauri multi-window routing (spawn_app_window(app_type: "word" | "excel" | "ppt" | "pdf" | "notes") in src-tauri/src/lib.rs using 	auri::WebviewWindowBuilder::new with ?app=<app_type> URL query parameter, plus an in-app multi-window workspace manager fallback when running in browser/test mode).
   - Each window filters its tab bar strictly to its own document format (e.g., the **Aether Word** window shows only .docx tabs; **Aether Excel** shows only .xlsx tabs; **Aether PowerPoint** shows only .pptx tabs; **Aether PDF** shows only .pdf tabs; **Aether Notes** shows only .txt/.md tabs).
   - Clicking an app icon (W, X, P, PDF, N) on the Left App Rail or Home Hub either focuses/spawns that app's dedicated window OR switches cleanly to that isolated app workspace, while a **Global Open Documents Drawer (All Open Files)** lists every active document across all 5 apps with live auto-save to localStorage/IndexedDB so zero work is ever lost!
   - Also add an "Open in New Window â†—" button on every tab so any document can be popped out into a separate OS window on demand.
2. Write automated Vitest tests verifying isolated per-app tab filtering, session persistence, and new window spawning. Commit with [GEC-Verified].
```
