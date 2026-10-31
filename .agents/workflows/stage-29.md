description: Automated execution workflow for Stage 29: Word (.docx) Fix - 100% Zoom Text Visibility, Scroll Flicker Fix, Headers/Footers and All Ribbons
Workflow: Execute Stage 29 (/stage-29)
Objective: Stage 29: Word (.docx) Fix - 100% Zoom Text Visibility, Scroll Flicker Fix, Headers/Footers and All Ribbons

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 29.1.1: Fix Word 100% Zoom Text Clipping, Fix Scroll Flicker/Jump, Visible Headers/Footers & + New Page
- Spec File: .agents/stages/stage-29/subphase-29.1.1.md
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.1.
1. **Fix the 100% Zoom Invisible/Clipped Text Bug (Seen in image_b6a07e.png vs image_b64ac6.png):**
   - Look at why text was only visible at 20% zoom in image_b64ac6.png: the scroll parent had vertical centering (justify-content: center) or 	ransform: scale() without 	ransform-origin: top center, clipping the top half of the page off-screen at 100% zoom!
   - Fix .word-scroll-viewport: set display: block; overflow-y: auto; overflow-x: auto; padding: 32px 24px; position: relative; and center each .word-page-sheet horizontally with margin: 0 auto 32px auto; transform-origin: top center; (or scale width/padding/font-size proportionally by zoom / 100 so layout flow is 100% native and never clips!).
   - Ensure text inside .word-page-body has ont-size: 11pt (14.67px); color: #0f172a; line-height: 1.5; min-height: 860px; so typed characters are immediately large, dark, and crystal clear at 100% zoom!
2. **Fix the Scroll Flicker & Jump-to-Top Bug:**
   - Remove any key={...} on the scroll container or page list that changes on keystroke/scroll, and remove any useEffect calling scrollTo(0, 0) or element.focus({ preventScroll: false }) on re-render! When focusing a page programmatically, always pass { preventScroll: true } so scrolling down a multi-page document NEVER flickers or snaps back to the top.
3. **Visible, Always-Accessible Header, Footer & + Add New Page:**
   - Render a clearly delineated **Header Zone** (order-bottom: 1px dashed #cbd5e1, placeholder \"Click to edit header\") at the top of every page and a **Footer Zone** (order-top: 1px dashed #cbd5e1, placeholder \"Click to edit footer\", plus Page X of Y) at the bottom of every page.
   - Make **+ Page / Blank Page / + Blank Page / New Page** immediately append a new page object to doc.pages, scroll smoothly to the new page, and place the cursor inside the new page's body.
4. Write automated DOM tests verifying that at zoom = 100%, typed text is rendered inside the visible viewport (getBoundingClientRect().top >= 0), scrolling down preserves scrollTop, Header/Footer inputs are editable, and clicking + Page increases page count from 1 to 2. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 29.1.1

### Subphase 29.1.2: Word Complete Ribbon Overhaul: Font Type/Size, Home, Insert, Layout, References, Review, View
- Spec File: .agents/stages/stage-29/subphase-29.1.2.md
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

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 29.1.2

