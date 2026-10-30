# Subphase 34.1.1: Automated Feature-Matrix Harness Testing Every Button in Word, Excel, PPT, PDF & TXT
- **Stage:** 34 | **Phase:** 34.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Automated test suite (exhaustive_feature_matrix.test.tsx) executing 150+ button/feature assertions across all 5 apps with 0 failures.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 34.1.1
Invoke @adaptive-ui-builder under GEC-v4.0 Article III.2.
1. Create pps/unified-shell/src/__tests__/exhaustive_feature_matrix.test.tsx that programmatically renders and tests every single feature implemented across all 5 applications:
   - **Suite 1 â€” Global Shell & Theme:** Asserts full-app Dark/Light theme cascade on root, sidebar, ribbon, workspace, and status bar; asserts zero Word Navigation ghost sidebar when in Excel, PPT, PDF, or Notes; asserts separate per-app window/mode filtering and All Open Files session manager.
   - **Suite 2 â€” Word (.docx):** Programmatically types paragraphs into page 1, verifies live word count updates, clicks + New Page / Blank Page (asserts page count = 2), changes Font Family & Font Size, toggles Bold/Italic/Underline/Align/Bullets, applies Heading 1 and verifies left Navigation outline updates, inserts Table & resizable Shape, changes Page Orientation to Landscape and Margins to Narrow, generates a Table of Contents, inserts a Footnote, toggles Track Changes, adds a Comment, and verifies Save / Open.
   - **Suite 3 â€” Excel (.xlsx):** Types numbers & formulas (=SUM, =AVERAGE, =IF), adds Sheet2, applies Currency $ and Borders, inserts a Chart, sorts A-Z and Z-A, filters rows, runs Goal Seek, toggles View Grid and View Headings, freezes panes, and verifies Save / Open.
   - **Suite 4 â€” PowerPoint (.pptx):** Clicks + New Slide, edits slide text inline, inserts and resizes a Shape, inserts a SmartArt/Chart, changes Design Theme and Slide Size (16:9 / 4:3), applies Morph transition, adds an Entrance animation, toggles Hide Slide, launches Beginning (F5) fullscreen player and steps through slides, and verifies Save / Open.
   - **Suite 5 â€” PDF Studio (.pdf):** Opens/creates a PDF, clicks + Insert Blank Page, rotates page 90Â°, **resizes page dimensions (A4 -> Letter)**, **encrypts PDF with a password and unlocks/decrypts it**, edits text in-place, adds a Sticky Note & Highlight, draws Freehand Ink, fills an AcroForm field, places a Digital Signature, and saves .pdf.
   - **Suite 6 â€” Notepad / Notes (.txt/.md):** Types text, verifies live Ln/Col/Words/Chars counters, clicks Markdown H1/Bold/Task/Table buttons, switches to Split preview, runs Sort Lines A-Z and Find & Replace, creates a second scratchpad, and saves/opens .txt.
2. Run powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1, fix any failing assertion until 100% of tests pass, build the release binary, and commit + push with [GEC-Verified].
```
