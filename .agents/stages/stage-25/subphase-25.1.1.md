# Subphase 25.1.1: PDF Studio Workspace: Create Blank PDF, Open Local PDF & Page Management
- **Stage:** 25 | **Phase:** 25.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** Full PDF viewer and page organizer supporting Open .pdf, Create Blank PDF, Add/Delete/Rotate/Reorder pages, and Zoom.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 25.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Build the interactive **PDF Studio** module (pps/unified-shell/src/apps/pdf/) accessible from the Left App Rail and New Tab menu:
   - Start with a clean, editable blank PDF page (or click Open PDF to load and parse any .pdf file from disk via pdf-engine).
   - **Page Management Toolbar:** + Add Blank Page, Delete Page, Rotate 90 deg CW, Rotate 90 deg CCW, Move Page Up/Down, and left Page Thumbnails sidebar.
   - **Direct Text & Image Authoring on PDF:** Click + Add Text to place movable, editable text blocks anywhere on any PDF page (with Font, Size, Color, Bold/Italic controls); click + Add Image to stamp images onto a PDF page; click any existing text block in Edit Text mode to modify its text in-place!
2. Verify PDF page addition, rotation, and text block editing with automated DOM tests. Commit with [GEC-Verified].
```
