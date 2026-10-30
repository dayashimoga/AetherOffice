description: Automated execution workflow for Stage 22: Word (Writer) - Blank Editable Pages, Multi-Page & Working Ribbon
Workflow: Execute Stage 22 (/stage-22)
Objective: Stage 22: Word (Writer) - Blank Editable Pages, Multi-Page & Working Ribbon

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 22.1.1: Blank Editable Word Pages, Rich Typing, Selection & New Page Creation
Spec File: .agents/stages/stage-22/subphase-22.1.1.md
/execute-subphase 22.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Word (Writer) page editor component (pps/unified-shell/src/apps/writer/) so it is **immediately editable**:
   - Every new .docx tab starts with **1 clean, completely blank A4/Letter page** (zero prepopulated text!) focused automatically so the user can start typing immediately.
   - Use a synchronized rich-text contentEditable page array backed by the Rust word-engine state so typing, Backspace, Delete, Enter, Cut/Copy/Paste, Undo (Ctrl+Z), and Redo (Ctrl+Y) work with zero lag.
2. **Working Multi-Page Creation & Management:**
   - Add a **+ New Page** button on the Home and Insert ribbons (and at the bottom of the last page), plus Ctrl+Enter shortcut, that immediately appends a new editable page sheet (Page 2 of 2, Page 3 of 3) and focuses it.
   - Add a **Delete Empty Page** action and automatic page overflow splitting when text on a page exceeds the page height.
   - Make the Header and Footer areas at the top/bottom of each page editable on click.
3. Write automated DOM tests proving a new Word tab has empty text, accepts typed input, and creates Page 2 when + New Page is clicked. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 22.1.1
Subphase 22.1.2: Working Word Ribbon: Font, Alignment, Lists, Styles, Tables, Images & Export
Spec File: .agents/stages/stage-22/subphase-22.1.2.md
/execute-subphase 22.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Word Ribbon so clicking it actually modifies the selected text or inserts elements into the active page:
   - **Home Tab:** Font family dropdown (Calibri, Arial, Times New Roman, Inter, Georgia, Courier New), Font Size (8â€“72), A+ / A-, Bold (B), Italic (I), Underline (U), Strikethrough (S), Subscript (X2), Superscript (X2), Text Color picker, Highlight Color picker, Clear Formatting, Bullet List, Numbered List, Indent/Outdent, Align Left/Center/Right/Justify, Line Spacing (1.0, 1.15, 1.5, 2.0), and Styles (Normal, Heading 1, Heading 2, Heading 3, Title, Quote).
   - **Live Navigation Outline:** Whenever a user formats a line as Heading 1/2/3, dynamically list that real heading in the left Navigation sidebar (never show fake hardcoded headings!).
   - **Insert Tab:** + Blank Page, Insert Table (inserts an editable HTML/OOXML table with resizable cells and add/remove row/col controls), Insert Image (opens file picker and inserts resizable image), Insert Horizontal Line, Insert Link, Insert Date/Time.
   - **Layout Tab:** Page Orientation (Portrait / Landscape), Page Margins (Normal, Narrow, Wide), Paper Size (A4, Letter), Page Background Color.
   - **Review & View Tabs:** Live Word/Character/Paragraph/Page count dialog, Find & Replace bar (Ctrl+F), Zoom (50%â€“200%), Toggle Ruler, Toggle Navigation Sidebar.
2. Verify all Word ribbon actions with automated tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 22.1.2

