# Subphase 28.2.1: Universal File Menu Backstage: Real Open, Save, Save As, Export & Print for All 5 Formats
- **Stage:** 28 | **Phase:** 28.2
- **Target Subsystem:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **Deliverable:** 100% working File -> New, Open From Computer, Save, Save As, Export to PDF/HTML/CSV/TXT, and Print across Word, Excel, PPT, PDF, and Notes.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 28.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v4.0 Article III.
1. Build a unified, rock-solid File I/O service (pps/unified-shell/src/services/fileIoService.ts) connected to both the File Backstage menu AND top quick-action toolbar buttons (ðŸ“‚ Open, ðŸ’¾ Save, ðŸ–¨ï¸ Print):
   - **Open Existing File (Ctrl+O):** Triggers a hidden <input type="file"> (plus Tauri file dialog when in desktop runtime) supporting .docx, .doc, .html, .xlsx, .csv, .tsv, .pptx, .pdf, .txt, .md, and .json. Parses the file immediately and populates the active editor canvas!
   - **Save / Save As (Ctrl+S / Ctrl+Shift+S):** Serializes the current document state (Word HTML/OOXML package, Excel workbook/CSV, PPTX slide deck JSON/OOXML, PDF binary with edits, or TXT/MD string), writes it via Tauri FS or triggers an immediate browser file download with the exact filename, and clears the unsaved * indicator on the tab.
   - **Print (Ctrl+P):** Opens a dedicated clean print view / iframe containing ONLY the formatted document pages/sheet/slides/PDF/note and invokes window.print().
2. Write automated tests verifying File Open, Save, and Print across all 5 document types. Commit with [GEC-Verified].
```
