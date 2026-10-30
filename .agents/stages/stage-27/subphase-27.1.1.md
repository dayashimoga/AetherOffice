# Subphase 27.1.1: Universal File Open/Save/Export (.docx, .xlsx, .pptx, .pdf, .txt) & Shortcuts
- **Stage:** 27 | **Phase:** 27.1
- **Target Subsystem:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **Deliverable:** 100% working File -> New, Open, Save, Save As, Export PDF/CSV/HTML/TXT, Print, and global keyboard shortcuts.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 27.1.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v3.0.
1. Wire the File Backstage Menu and global keyboard shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+P, Ctrl+Z, Ctrl+Y, Ctrl+F) so they work across all 5 modules (Word, Excel, PowerPoint, PDF, Notepad):
   - **Open File (Ctrl+O):** Uses native <input type="file"> + Tauri file dialog to open real .docx, .xlsx, .csv, .pptx, .pdf, .txt, and .md files from the user's computer and load their contents into the appropriate editor tab.
   - **Save / Download File (Ctrl+S):** Serializes the current active tab via the Rust backend (or browser Blob download fallback) into a valid .docx, .xlsx, .csv, .pptx, .pdf, or .txt file on the user's machine.
   - **Print (Ctrl+P):** Clean print stylesheet that prints only the active document canvas (window.print()).
2. Verify round-trip Save & Open across all 5 formats with automated tests and commit with [GEC-Verified].
```
