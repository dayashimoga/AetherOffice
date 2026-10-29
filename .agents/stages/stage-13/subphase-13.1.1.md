# Subphase 13.1.1: Comprehensive Automated E2E UI Test Suite Across All 5 Apps
- **Stage:** 13 | **Phase:** 13.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Automated E2E verification suite testing every single Ribbon tab, button, keyboard shortcut, and canvas edit across Hub, Writer, Calc, Slides, and PDF.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 13.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.4.
1. Create a comprehensive automated UI verification suite in pps/unified-shell/src/__tests__/full_suite_e2e.test.tsx that programmatically verifies:
   - **Hub:** Launching Writer, Calc, Slides, and PDF independently.
   - **Writer:** Typing paragraphs, inserting a new page (Blank Page), applying Bold/Heading 1/Table, and saving .docx.
   - **Calc:** Editing cell A1 (100) and A2 (200), entering =SUM(A1:A2) in A3, asserting 300 renders, adding Sheet2, sorting, and inserting a Chart.
   - **Slides:** Adding a slide, inserting a Star shape, dragging/resizing the shape, editing text inline, applying a Transition, and starting Fullscreen Slide Show.
   - **PDF:** Creating/opening a PDF, editing text in-place, adding a Page, filling a Form field, placing a Signature, and saving .pdf.
2. Run 	ools/verify-contract.ps1 and ensure 100% pass rate with zero skipped tests. Commit with [GEC-Verified].
`
