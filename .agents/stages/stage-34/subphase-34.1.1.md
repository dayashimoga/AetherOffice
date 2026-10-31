# Subphase 34.1.1: Comprehensive Automated Menu Crawler & E2E Assertion Suite (250+ Automated Checks)
- **Stage:** 34 | **Phase:** 34.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Automated test suite crawling every tab and button across Word, Excel, PowerPoint, PDF, and Notepad and verifying state/DOM mutations.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 34.1.1
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Build an exhaustive automated verification suite in pps/unified-shell/src/__tests__/gec_v5_complete_audit.test.tsx that automatically tests every single menu item and user workflow so the user never has to manually hunt for broken buttons:
   - **Automated Ribbon Crawler:** Programmatically iterates through all 5 apps (word, excel, ppt, pdf, 
otes), clicks every single Ribbon Tab (Home, Insert, Layout, Page Layout, Formulas, Data, Design, Transitions, Animations, Slide Show, Edit PDF, Forms, Comment & Annotate, Page Tools, Sign & Protect, References, Review, View, Edit, Text Tools, Markdown), inspects every <button> and <select> rendered inside that tab, verifies none are clipped off-screen or disabled without cause, clicks each control, and asserts zero runtime exceptions and expected state transitions!
   - **Targeted Bug Regression Assertions:**
     1. Asserts Word initializes at 100% zoom, typed text is visible at 100% zoom, scrolling does not reset scrollTop, Header/Footer are editable, Font Family/Size mutate text, + Page creates Page 2, and .docx Open/Save work.
     2. Asserts Excel supports clicking/dragging Row headers (1..5) and Column headers (A..C) to select multiple whole rows/columns and applies batch formatting (Bold, Fill Color, Currency $, Clear) across all cells in those rows/columns.
     3. Asserts PowerPoint opens/saves .pptx, supports all ribbon tools, and has readable high-contrast slide thumbnail text in both Light and Dark mode.
     4. Asserts PDF Studio opens/saves .pdf, supports Draw Signature and Type Custom Name signature stamps (and asserts \"John Doe\" does not exist anywhere!), encrypts/decrypts passwords, resizes page dimensions, and edits PDF text/annotations.
     5. Asserts opening multiple simultaneous .docx, .xlsx, .pptx, .pdf, and .txt tabs and activating Split View works smoothly.
2. Run powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1, ensure 100% of Rust and Frontend tests pass, build the release binary, and commit + push to origin with [GEC-Verified].
```
