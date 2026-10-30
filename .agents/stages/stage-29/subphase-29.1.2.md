# Subphase 29.1.2: Fix Word Home, Insert & Layout Ribbons (100% Working Controls)
- **Stage:** 29 | **Phase:** 29.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Every single button in Word's Home, Insert, and Layout ribbons wired and verified via automated tests.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 29.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's Home, Insert, and Layout ribbons (preserving editor selection before executing commands using onMouseDown={(e) => e.preventDefault()} on toolbar buttons so clicking a button NEVER steals focus from the selected text!):
   - **Home Tab:** Clipboard (Cut, Copy, Paste, Format Painter), Font Family (Calibri, Arial, Times New Roman, Georgia, Verdana, Consolas, Inter â€” wraps selection or sets active typing style), Font Size (8pt to 72pt), Grow/Shrink Font (A+, A-), Change Case (Aa), Clear Formatting, Bold (B), Italic (I), Underline (U), Strikethrough (S), Subscript (Xâ‚‚), Superscript (XÂ²), Text Color palette, Highlight Color palette, Bullet List, Numbered List, Multilevel List, Decrease/Increase Indent, Sort Paragraphs, Show/Hide Â¶ Formatting Marks, Align Left/Center/Right/Justify, Line & Paragraph Spacing (1.0, 1.15, 1.5, 2.0), Shading fill, Borders, and Styles (Normal, No Spacing, Heading 1, Heading 2, Heading 3, Title, Subtitle, Quote).
   - **Insert Tab:** Cover Page (inserts a styled title cover page), Blank Page, Page Break, Table (interactive grid size selector + add/remove row/column/border toolbar), Picture (file upload), Shapes (Rectangle, Rounded Rect, Circle/Oval, Triangle, Arrow, Star, Callout), Chart (inserts editable inline bar/line/pie chart), Hyperlink, Bookmark, Header, Footer, Page Number, Text Box, WordArt, Drop Cap, Date & Time, Symbol (Î©, â‚¬, Â£, Â©, â„¢, Â±, â‰ , â‰¤, â‰¥), and Horizontal Line.
   - **Layout Tab:** Margins (Normal 1", Narrow 0.5", Moderate, Wide â€” visibly updates page padding!), Orientation (Portrait vs Landscape â€” visibly swaps page width/height 816px Ã— 1056px vs 1056px Ã— 816px!), Size (Letter, A4, Legal), Columns (One, Two, Three â€” applies CSS multi-column layout to the page body!), Breaks, Line Numbers toggle, Hyphenation, Page Color picker, Page Borders selector, and Watermark (CONFIDENTIAL, DRAFT, custom).
2. Write automated tests clicking and asserting every single control in Home, Insert, and Layout. Commit with [GEC-Verified].
```
