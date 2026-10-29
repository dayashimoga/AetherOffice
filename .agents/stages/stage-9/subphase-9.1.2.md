# Subphase 9.1.2: Multi-Page Creation, Page Breaks, Headers/Footers & Zoom/Ruler
- **Stage:** 9 | **Phase:** 9.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Automatic page overflow creation, manual Page Break (Ctrl+Enter), '+ New Page' button, editable Header/Footer zones, and working Ruler.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 9.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement multi-page layout and creation in Writer:
   - **Automatic Pagination:** When paragraphs exceed the vertical body height of Page 1 (11in minus top/bottom margins), automatically flow overflow paragraphs onto Page 2, Page 3, etc., rendered as distinct visual paper sheets separated by a page gap.
   - **Manual Page Creation:** Wire Insert -> Blank Page, Insert -> Page Break, and Ctrl+Enter to immediately insert a new page and move the cursor to the top of the new page.
   - **Editable Headers & Footers:** Double-clicking the top or bottom margin of any page activates the Header/Footer editor with dynamic Page X of Y numbering.
   - **Interactive Horizontal Ruler:** Dragging the left/right indent markers on the top ruler dynamically updates page margins and paragraph indentation.
2. Write DOM tests verifying that Insert -> Blank Page and Ctrl+Enter increase the rendered .word-page count from 1 to 2+. Commit with [GEC-Verified].
`
