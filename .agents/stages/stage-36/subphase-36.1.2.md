# Subphase 36.1.2: Word Complete Ribbon Fix: All 7 Tabs Working & Visible (Zero Clipped Groups)

- **Stage:** 36 | **Phase:** 36.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Word ribbon buttons across File, Home, Insert, Layout, References, Review, and View.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 36.1.2
Invoke @adaptive-ui-builder under GEC-v6.0.
1. Fix the Word Ribbon layout seen in `image_97c287.png` where `Styles` and `Copilot` had their buttons hidden/clipped below the ribbon border:
   - Ensure `.ribbon-panel` has `height: auto; max-height: none; overflow: visible;` so every group (`Pages`, `Clipboard`, `Font`, `Paragraph`, `Styles`, `Editing`) displays all of its buttons clearly!
2. Wire and verify **every single button** across all 7 Word tabs on the active `.word-page-body`:
   - **Home:** `+ Page`, `Delete Page`, `Cut`, `Copy`, `Paste`, `Format Painter`, `Font Family` (applies font to selection or active body style), `Font Size` (`8`–`72`), `A+`, `A-`, `Change Case (UPPER/lower/Title)`, `Clear Formatting`, `Bold`, `Italic`, `Underline`, `Strikethrough`, `Subscript`, `Superscript`, `Text Color`, `Highlight Color`, `Bullets`, `Numbering`, `Multilevel List`, `Outdent`, `Indent`, `Sort A-Z`, `Show/Hide ¶`, `Align Left/Center/Right/Justify`, `Line Spacing (1.0, 1.15, 1.5, 2.0)`, `Shading`, `Borders`, `Styles (Normal, No Spacing, Heading 1, Heading 2, Heading 3, Title, Subtitle, Quote)` (updates left Navigation headings panel live!), and `Find & Replace`.
   - **Insert:** `Cover Page`, `Blank Page`, `Page Break`, `Table` (inserts interactive editable table), `Picture` (opens file picker and inserts resizable image into body), `Shapes` (inserts draggable/resizable shape), `Chart` (inserts inline chart), `Hyperlink`, `Bookmark`, `Header` (activates Header zone), `Footer` (activates Footer zone), `Page Number`, `Text Box`, `Date & Time`, `Symbol`, `Equation`, `Horizontal Line`.
   - **Layout:** `Margins (Normal, Narrow, Wide)`, `Orientation (Portrait / Landscape)`, `Size (A4 / Letter / Legal)`, `Columns (1 / 2 / 3)`, `Line Numbers`, `Page Color`, `Page Borders`, `Watermark`.
   - **References:** `Table of Contents` (inserts generated TOC from document headings), `Insert Footnote` (adds footnote at bottom of page), `Insert Endnote`, `Insert Citation`, `Insert Bibliography`, `Insert Caption`.
   - **Review:** `Spell Check`, `Word Count` modal, `Read Aloud`, `New Comment`, `Delete Comment`, `Track Changes` toggle, `Accept All`, `Reject All`, `Protect Document`.
   - **View:** `Print Layout`, `Web Layout`, `Read Mode`, `Ruler`, `Gridlines`, `Nav Pane`, `Zoom (50%..200%)`, `One Page`, `Multi Pages`, `Page Width`.
3. Write automated tests verifying every Word ribbon tab and button. Commit with `[GEC-Verified]`.
````
