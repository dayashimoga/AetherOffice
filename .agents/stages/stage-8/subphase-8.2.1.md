# Subphase 8.2.1: Native File Open, Save, Save-As & Backstage File Menu
- **Stage:** 8 | **Phase:** 8.2
- **Target Subsystem:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **Deliverable:** Working File Backstage menu (New, Open, Save, Save As, Export PDF, Print) wired to real disk I/O for .docx, .xlsx, .pptx, and .pdf.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 8.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v2.0.
1. Fix the broken File tab in the ribbon: clicking File MUST open a full Backstage overlay menu containing:
   - **New** (Blank Document / Sheet / Presentation / PDF, plus reset current state).
   - **Open...** (Triggers native OS file picker or path input, reads .docx, .xlsx, .pptx, or .pdf bytes from disk through word-engine, excel-engine, ppt-engine, or pdf-engine, and populates the live canvas).
   - **Save / Save As...** (Serializes the current in-memory Rust engine state back to a valid .docx, .xlsx, .pptx, or .pdf file on disk and clears the dirty * indicator).
   - **Export to PDF** (Renders the current document/sheet/slide deck into a valid ISO 32000 PDF via pdf-engine).
2. Write integration tests verifying round-trip disk Open -> Edit -> Save for all 4 formats. Commit with [GEC-Verified].
`
