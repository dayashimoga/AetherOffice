# Subphase 22.1.2: Working Word Ribbon: Font, Alignment, Lists, Styles, Tables, Images & Export
- **Stage:** 22 | **Phase:** 22.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Word ribbon buttons across Home, Insert, Layout, Review, and View tabs.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 22.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Word Ribbon so clicking it actually modifies the selected text or inserts elements into the active page:
   - **Home Tab:** Font family dropdown (Calibri, Arial, Times New Roman, Inter, Georgia, Courier New), Font Size (8â€“72), A+ / A-, Bold (B), Italic (I), Underline (U), Strikethrough (S), Subscript (X2), Superscript (X2), Text Color picker, Highlight Color picker, Clear Formatting, Bullet List, Numbered List, Indent/Outdent, Align Left/Center/Right/Justify, Line Spacing (1.0, 1.15, 1.5, 2.0), and Styles (Normal, Heading 1, Heading 2, Heading 3, Title, Quote).
   - **Live Navigation Outline:** Whenever a user formats a line as Heading 1/2/3, dynamically list that real heading in the left Navigation sidebar (never show fake hardcoded headings!).
   - **Insert Tab:** + Blank Page, Insert Table (inserts an editable HTML/OOXML table with resizable cells and add/remove row/col controls), Insert Image (opens file picker and inserts resizable image), Insert Horizontal Line, Insert Link, Insert Date/Time.
   - **Layout Tab:** Page Orientation (Portrait / Landscape), Page Margins (Normal, Narrow, Wide), Paper Size (A4, Letter), Page Background Color.
   - **Review & View Tabs:** Live Word/Character/Paragraph/Page count dialog, Find & Replace bar (Ctrl+F), Zoom (50%â€“200%), Toggle Ruler, Toggle Navigation Sidebar.
2. Verify all Word ribbon actions with automated tests and commit with [GEC-Verified].
```
