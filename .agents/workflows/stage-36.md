description: Automated execution workflow for Stage 36: Word (.docx) Fix - Stop Duplicate Header/Body Typing, Independent Header/Footer & Complete Ribbon
Workflow: Execute Stage 36 (/stage-36)
Objective: Stage 36: Word (.docx) Fix - Stop Duplicate Header/Body Typing, Independent Header/Footer & Complete Ribbon

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v6.0) compliance:

Subphase 36.1.1: Fix Word Header/Body Duplicate Typing Bug & Independent Header/Body/Footer Zones
Spec File: .agents/stages/stage-36/subphase-36.1.1.md
/execute-subphase 36.1.1
Invoke @adaptive-ui-builder under GEC-v6.0 Article I.
1. **Fix the Critical Bug Seen in `image_97c287.png` & `image_97b3a4.png` (Typing Appearing in Header & Body Simultaneously):**
   - Inspect why `asdfasdfadsf...` appeared twice (once in the top header box and once in the body below the shadow):
     - Remove any duplicate rendering of `page.content` or shared state between the header box and the body box!
     - Give each `WordPage` three strictly independent properties:
       - `headerText: string` (default `""`)
       - `bodyHtml: string` (default `""`)
       - `footerText: string` (default `""`)
     - Maintain `editingZone: "body" | "header" | "footer"` (default `"body"`).
2. **Proper Microsoft Word Header / Body / Footer UX:**
   - **Main Page Body (`editingZone === "body"`):** Clicking anywhere on the main white page sheet focuses the **Body** editor (`min-height: 820px; padding: 24px 64px; outline: none;`). Everything typed goes **strictly into `bodyHtml`**—NEVER into the header!
   - **Header Zone (Top of Page) & Footer Zone (Bottom of Page):**
     - When `editingZone === "body"`, the Header and Footer areas are rendered softly (`opacity: 0.65`) at the top and bottom margins of the same unified paper sheet (NOT as a detached separate card with a shadow gap splitting the page in two!).
     - **Double-clicking** the Header or Footer (or clicking `Insert -> Header` / `Insert -> Footer` on the ribbon) sets `editingZone = "header"` (or `"footer"`), focuses that input, and displays a clear **`[✓ Exit Header/Footer (Esc)]`** pill button that switches `editingZone` back to `"body"` and focuses the document body!
3. Write an automated regression test in Vitest that:
   - Types `"Hello Body Text"` into the Word document body and asserts `headerText` is `""` (empty) and `"Hello Body Text"` appears ONLY once on the page (inside `.word-page-body`).
   - Double-clicks the Header zone, types `"Confidential Header"`, clicks `Exit Header/Footer`, and asserts `headerText === "Confidential Header"` while `bodyHtml` remains `"Hello Body Text"`.
   Commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 36.1.1

Subphase 36.1.2: Word Complete Ribbon Fix: All 7 Tabs Working & Visible (Zero Clipped Groups)
Spec File: .agents/stages/stage-36/subphase-36.1.2.md
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

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 36.1.2
