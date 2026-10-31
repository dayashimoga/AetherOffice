# Subphase 29.1.2: Word Complete Ribbon Overhaul: Font Type/Size, Home, Insert, Layout, References, Review, View
- **Stage:** 29 | **Phase:** 29.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Word Font family/size, paragraph styling, tables, images, shapes, layout, references, review, and view tools.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 29.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. **Fix Word Font Family, Font Size & Home Ribbon Not Working:**
   - Why Font Type & Font Size failed: clicking a <select> dropdown steals DOM focus from the contentEditable page, losing the user's text selection!
   - **Solution:** Track the last valid Selection Range in a ref (savedSelectionRangeRef) on onSelect/onKeyUp/onMouseUp inside the page editor. Whenever the user changes **Font Family**, **Font Size**, **Text Color**, **Highlight Color**, or clicks **Bold, Italic, Underline, Strikethrough, Subscript, Superscript, Align, Bullets, Numbering, Line Spacing, or Styles (Heading 1..3, Title)**, first restore savedSelectionRangeRef (or if collapsed, apply directly to the active paragraph / current typing span style) so it works 100% of the time whether text is highlighted or the user is about to type!
2. **Fix All Remaining Word Ribbon Tabs (Insert, Layout, References, Review, View):**
   - **Insert Tab:** Cover Page, Blank Page, Page Break, Table (inserts editable table with + Row, + Col, Delete Table mini-bar), Picture (uploads local image file onto the page with resize slider), Shapes (inserts draggable/resizable SVG shape with text edit & delete button), Hyperlink (modal to insert clickable <a href>), Header & Footer (focuses header/footer), Page Number, Text Box, Date & Time, Symbol, Horizontal Line, Equation (âˆ‘, âˆš, Ï€, âˆ«).
   - **Layout Tab:** Margins (Normal, Narrow, Wide, Custom â€” updates page padding), Orientation (Portrait / Landscape â€” swaps page width/height), Size (A4, Letter, Legal), Columns (1, 2, 3 â€” sets columnCount on page body), Page Color (changes paper background color), Page Borders (adds solid/double/dashed frame around page), Watermark (renders diagonal watermark text behind page content).
   - **References Tab:** Insert Table of Contents (generates live linked TOC from all headings), Insert Footnote (adds superscript number + footnote at bottom of page), Insert Endnote, Insert Citation & Bibliography, Insert Caption.
   - **Review Tab:** Spell Check & Grammar, Word Count dialog, Read Aloud (TTS), New Comment (adds comment card in right sidebar linked to selected text), Delete Comment, Track Changes (on/off), Accept All, Reject All, Protect Document.
   - **View Tab:** Print Layout, Web Layout, Read Mode, Ruler, Gridlines, Nav Pane, Zoom (50%..200%), One Page, Multi Pages (displays pages side-by-side in a 2-column grid!), Page Width.
3. Write automated tests clicking and asserting every single Word menu item. Commit with [GEC-Verified].
```
