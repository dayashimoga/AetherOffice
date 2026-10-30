---
description: Automated execution workflow for Stage 29: Word (.docx) Complete Fix - Typing, Multi-Page, Shapes and All 7 Ribbons
---

# Workflow: Execute Stage 29 (/stage-29)

## Objective: Stage 29: Word (.docx) Complete Fix - Typing, Multi-Page, Shapes and All 7 Ribbons
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

### Subphase 29.1.1: Fix Word Typing, Cursor Focus, Multi-Page Creation & Draggable/Resizable Shapes
- **Spec File:** `.agents/stages/stage-29/subphase-29.1.1.md`
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. **Fix Word Typing & Cursor Loss Bug:**
   - Fix the React/DOM controlled-input bug that prevented typing inside Word pages! Each page's .word-page-body must use an uncontrolled contentEditable="true" ref (synchronized via onInput without overwriting innerHTML while the element is focused) AND support programmatic insertion at the saved Selection range so users can type continuously at 120fps, press Enter, Backspace, Delete, Tab, and select text freely.
2. **Fix Giant Blue Shapes Bug (Seen in image_25f41c.png):**
   - In image_25f41c.png, inserting shapes created massive full-width blue rectangles/ovals stacked across the entire page!
   - Fix Insert -> Shape and Insert -> Picture so shapes insert at a sensible default size (140px Ã— 90px), with an interactive selection border, **drag-to-move**, **8-corner resize handles**, **color picker**, **text inside shape**, and **Delete (Del key or trash icon)**.
3. **Working Multi-Page Creation (+ New Page, Page Break, Auto-Overflow):**
   - Add a floating **+ Add Page** button between/below pages AND wire Insert -> Blank Page and Insert -> Page Break (Ctrl+Enter) to immediately create Page 2, Page 3, etc., and scroll/focus the new page.
   - Allow deleting any extra page via a Delete Page button on the page header/ribbon.
4. Write automated DOM tests simulating continuous typing, creating 3 pages, inserting/resizing a shape, and deleting it. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 29.1.1`

### Subphase 29.1.2: Fix Word Home, Insert & Layout Ribbons (100% Working Controls)
- **Spec File:** `.agents/stages/stage-29/subphase-29.1.2.md`
/execute-subphase 29.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's Home, Insert, and Layout ribbons (preserving editor selection before executing commands using onMouseDown={(e) => e.preventDefault()} on toolbar buttons so clicking a button NEVER steals focus from the selected text!):
   - **Home Tab:** Clipboard (Cut, Copy, Paste, Format Painter), Font Family (Calibri, Arial, Times New Roman, Georgia, Verdana, Consolas, Inter â€” wraps selection or sets active typing style), Font Size (8pt to 72pt), Grow/Shrink Font (A+, A-), Change Case (Aa), Clear Formatting, Bold (B), Italic (I), Underline (U), Strikethrough (S), Subscript (Xâ‚‚), Superscript (XÂ²), Text Color palette, Highlight Color palette, Bullet List, Numbered List, Multilevel List, Decrease/Increase Indent, Sort Paragraphs, Show/Hide Â¶ Formatting Marks, Align Left/Center/Right/Justify, Line & Paragraph Spacing (1.0, 1.15, 1.5, 2.0), Shading fill, Borders, and Styles (Normal, No Spacing, Heading 1, Heading 2, Heading 3, Title, Subtitle, Quote).
   - **Insert Tab:** Cover Page (inserts a styled title cover page), Blank Page, Page Break, Table (interactive grid size selector + add/remove row/column/border toolbar), Picture (file upload), Shapes (Rectangle, Rounded Rect, Circle/Oval, Triangle, Arrow, Star, Callout), Chart (inserts editable inline bar/line/pie chart), Hyperlink, Bookmark, Header, Footer, Page Number, Text Box, WordArt, Drop Cap, Date & Time, Symbol (Î©, â‚¬, Â£, Â©, â„¢, Â±, â‰ , â‰¤, â‰¥), and Horizontal Line.
   - **Layout Tab:** Margins (Normal 1", Narrow 0.5", Moderate, Wide â€” visibly updates page padding!), Orientation (Portrait vs Landscape â€” visibly swaps page width/height 816px Ã— 1056px vs 1056px Ã— 816px!), Size (Letter, A4, Legal), Columns (One, Two, Three â€” applies CSS multi-column layout to the page body!), Breaks, Line Numbers toggle, Hyphenation, Page Color picker, Page Borders selector, and Watermark (CONFIDENTIAL, DRAFT, custom).
2. Write automated tests clicking and asserting every single control in Home, Insert, and Layout. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 29.1.2`

### Subphase 29.2.1: Fix Word References, Review & View Ribbons (100% Working Controls)
- **Spec File:** `.agents/stages/stage-29/subphase-29.2.1.md`
/execute-subphase 29.2.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's References, Review, and View ribbons:
   - **References Tab:** Table of Contents (automatically scans all Heading 1/2/3 elements in the document and inserts a clickable Table of Contents block at the top of the document + Update Table), Insert Footnote (inserts a superscript [1] reference at the cursor and appends a numbered footnote entry in the page footer area), Insert Endnote, Insert Citation & Insert Bibliography (APA/MLA/IEEE citation manager modal), and Insert Caption (for figures/tables).
   - **Review Tab:** Spelling & Grammar check drawer, Thesaurus lookup, Word Count modal (showing Pages, Words, Characters no spaces, Characters with spaces, Paragraphs, Lines), Read Aloud (uses browser window.speechSynthesis TTS to read the document aloud with Play/Pause/Stop!), Translate Selection, New Comment (attaches a comment card in a right-hand Comments gutter), Delete/Next/Prev Comment, Track Changes toggle (records subsequent typing in <ins> and deletions in <del>), Accept All Changes, Reject All Changes, and Protect Document (Read-Only Lock).
   - **View Tab:** Print Layout (standard paged sheets), Web Layout (full-width fluid canvas), Read Mode (distraction-free two-column book reader), Focus Mode (hides ribbon), Ruler checkbox, Gridlines checkbox (renders subtle alignment grid overlay on the page), Navigation Pane checkbox, Dark Page Canvas toggle, Zoom (50%, 75%, 100%, 150%, 200%), One Page, Multi Pages (side-by-side pages), and Page Width.
2. Verify every button in References, Review, and View with automated Vitest DOM assertions. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 29.2.1`

