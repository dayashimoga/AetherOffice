# Subphase 21.1.2: Unified Left App Switcher Rail, Home Hub & Multi-Document Tab Router
- **Stage:** 21 | **Phase:** 21.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Single-window Office Suite with Left App Rail (Home, Word, Excel, PowerPoint, PDF, Notes) and working multi-format tab bar.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 21.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build a unified Microsoft 365 / WPS-style **Left App Switcher Rail** (48px wide vertical dock on the left edge of the window) with icons and tooltips for:
   - **Home Hub** (Dashboard with 5 large 'Create New' cards: Blank Word .docx, Blank Excel .xlsx, Blank PowerPoint .pptx, Open/Blank PDF .pdf, and Blank Note .txt/.md).
   - **Word (W blue icon)** â€” Instantly creates/switches to a blank editable .docx tab.
   - **Excel (X green icon)** â€” Instantly creates/switches to a blank editable .xlsx tab (Book1.xlsx), NEVER opening .docx by mistake!
   - **PowerPoint (P orange icon)** â€” Instantly creates/switches to a blank editable .pptx tab (Presentation1.pptx).
   - **PDF Studio (PDF red icon)** â€” Instantly creates/switches to a PDF Studio tab (Document.pdf).
   - **Notepad / Notes (N purple icon)** â€” Instantly creates/switches to a clean Notepad/Markdown tab (Untitled.txt).
2. Fix the top + New Tab button: clicking + opens a quick dropdown menu to launch a blank Word, Excel, PowerPoint, PDF, or Note tab, and clicking x on any tab closes that tab cleanly.
3. Verify via automated DOM tests that launching Excel, PPT, PDF, or Notes renders its exact respective engine, never .docx. Commit with [GEC-Verified].
```
