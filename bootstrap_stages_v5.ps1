# ==============================================================================
# bootstrap_stages_v5.ps1 - Generates Stages 28 to 34 (21 Subphases)
# Fixes: Modern UI Overhaul, Non-Overflowing Ribbon, High-Contrast Dark Theme,
# Multi-Document Per Format + Split Screen, Word 100% Zoom/Scroll/Header/Footer/IO,
# Excel Multi-Row/Col Selection & Ribbon, PPTX Open/Save & Full Ribbon,
# PDF Local Open/Save/Edit/Custom Signature Pad, Notepad Pro Features, & Auto-Tests.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v5.0 2/2] Generating Stages 28-34 Prompts & Workflows ===" -ForegroundColor Cyan

foreach ($s in 28..34) {
    New-Item -ItemType Directory -Force -Path ".agents/stages/stage-$s" | Out-Null
}

$v5Subphases = @(
    # ==========================================================================
    # STAGE 28: MODERN VISUAL OVERHAUL, NON-OVERFLOWING RIBBON, CONTRAST & MULTI-DOC
    # ==========================================================================
    @{
        id = "28.1.1"; stage = 28; phase = "28.1"; title = "Modern Fluent/Linear Visual Overhaul, Full-App Dark Theme & High-Contrast Fix"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Visually modern UI design, zero clipped ribbon items, full-app dark/light theme including canvas backdrop, and fixed thumbnail contrast."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 28.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article II.
1. **Modern Visual Design Overhaul:**
   - Upgrade the entire application shell (`apps/unified-shell/src/`) with a sleek, modern design system (crisp `Inter` / `Segoe UI Variable` typography, refined `6px` border-radius buttons with subtle hover/active states, active state pill indicators, clean SVG iconography, and modern elevation shadows).
2. **Fix Ribbon Overflow & Clipped Right-Side Controls (Seen in image_b6a07e.png & image_b64b07.png):**
   - Fix the Ribbon container so controls on the right (`Styles`, `Subtotal`, `Drawing & Arrange`) are NEVER clipped off-screen!
   - Give `.ribbon-content` `display: flex; flex-wrap: wrap; gap: 6px; padding: 6px 12px; height: auto; min-height: 76px; max-height: 136px; overflow-y: visible;` AND compact group spacing so every single button and dropdown wraps cleanly into view on any screen width.
3. **Fix Partial Dark Mode & Invisible Slide Thumbnail Text (Seen in image_b5d5ee.png):**
   - Apply theme CSS variables (`--bg-canvas-backdrop`, `--bg-panel`, `--bg-ribbon`, `--text-primary`, `--text-secondary`, `--border-color`) across 100% of the viewport so switching to **Dark Mode** turns the gray workspace surround into a rich dark slate (`#0b1120`), styles all sidebars/dialogs/status bars in dark mode, and ensures **high contrast everywhere**.
   - Specifically fix the bug in `image_b5d5ee.png` where slide thumbnail cards had white backgrounds with white/light-gray text: ensure thumbnail preview cards explicitly set `color: #0f172a` when their background is light, or style thumbnail cards with dark surfaces (`#1e293b`) and bright text (`#f8fafc`).
4. Write automated DOM tests verifying zero horizontal overflow clipping on the ribbon and high-contrast theme propagation. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "28.1.2"; stage = 28; phase = "28.1"; title = "Multi-Document Per Format Manager, Side-by-Side Split View & Real Binary File Open/Save"
        crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false
        deliverable = "Open unlimited simultaneous .docx, .xlsx, .pptx, .pdf, and .txt files, filter by app or view all, edit side-by-side in Split View, and open/save real binary files."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 28.1.2
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v5.0 Article I.
1. **Seamless Multi-Document Management (Multiple `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.txt` at the same time):**
   - Upgrade the Left App Rail (`Home`, `W`, `X`, `P`, `PDF`, `N`):
     - Clicking an app icon (`W`, `X`, `P`, `PDF`, `N`) shows a badge with the count of open files of that type (e.g., `W (3)`, `X (2)`). If none are open, it creates one; if already viewing that app, clicking `+` on the tab bar or clicking the app icon's `+ New` / `📂 Open` quick menu opens **additional simultaneous documents** of that format (`Untitled 1.docx`, `Untitled 2.docx`, `Project.docx`, `Budget.xlsx`, `Forecast.xlsx`, etc.)!
     - Add a tab bar filter toggle: **`Current App Tabs` vs `All Open Tabs (N)`** so users can either focus strictly on their open Excel files OR see all open documents across all formats in one unified tab strip.
     - Add a **`◫ Split View` (Side-by-Side Workspace)** button in the top bar: when enabled, splits the main workspace into **Left Pane** and **Right Pane** with independent tab selectors so a user can edit two Word docs, two Excel sheets, or a Word doc + PDF side-by-side simultaneously!
2. **Fix Real Local File Open & Save for `.docx`, `.xlsx`, `.pptx`, `.pdf`, and `.txt`:**
   - Use JSZip / `ooxml-opc` + `FileReader.readAsArrayBuffer()` so when a user clicks `Open` (`Ctrl+O`) and selects a real `.docx`, `.xlsx`, `.pptx`, `.pdf`, or `.txt` file from their computer:
     - **`.docx`:** Unzips `word/document.xml`, parses paragraphs (`<w:p>`), runs (`<w:r>`, `<w:t>`, bold/italic/color/size), and tables (`<w:tbl>`) into editable HTML pages!
     - **`.xlsx` / `.csv`:** Unzips `xl/sharedStrings.xml` and `xl/worksheets/sheet*.xml` (or parses CSV) and populates all sheets, cell values, and formulas into the grid!
     - **`.pptx`:** Unzips `ppt/slides/slide*.xml`, extracts text shapes (`<p:sp>`, `<a:t>`) and geometry into editable slides!
     - **`.pdf`:** Reads the binary PDF buffer, parses page count, dimensions, and text streams (via `pdf-engine` / PDF parser), and renders every page onto the PDF canvas!
   - When clicking `Save` (`Ctrl+S`), package and download/write a valid binary `.docx`, `.xlsx`, `.pptx`, `.pdf`, or `.txt` file!
3. Write automated tests verifying opening 3 simultaneous `.docx` and 2 `.xlsx` files, enabling Split View, and round-trip binary parsing/saving for all 5 formats. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 29: WORD (.DOCX) — FIX 100% ZOOM VISIBILITY, SCROLL FLICKER, PAGES & RIBBON
    # ==========================================================================
    @{
        id = "29.1.1"; stage = 29; phase = "29.1"; title = "Fix Word 100% Zoom Text Clipping, Fix Scroll Flicker/Jump, Visible Headers/Footers & + New Page"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Crisp visible text at 100% zoom, rock-solid smooth scrolling without jumping to top, visible editable Headers/Footers, and instant + New Page."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.1.
1. **Fix the 100% Zoom Invisible/Clipped Text Bug (Seen in image_b6a07e.png vs image_b64ac6.png):**
   - Look at why text was only visible at 20% zoom in `image_b64ac6.png`: the scroll parent had vertical centering (`justify-content: center`) or `transform: scale()` without `transform-origin: top center`, clipping the top half of the page off-screen at 100% zoom!
   - Fix `.word-scroll-viewport`: set `display: block; overflow-y: auto; overflow-x: auto; padding: 32px 24px; position: relative;` and center each `.word-page-sheet` horizontally with `margin: 0 auto 32px auto; transform-origin: top center;` (or scale width/padding/font-size proportionally by `zoom / 100` so layout flow is 100% native and never clips!).
   - Ensure text inside `.word-page-body` has `font-size: 11pt (14.67px); color: #0f172a; line-height: 1.5; min-height: 860px;` so typed characters are immediately large, dark, and crystal clear at `100%` zoom!
2. **Fix the Scroll Flicker & Jump-to-Top Bug:**
   - Remove any `key={...}` on the scroll container or page list that changes on keystroke/scroll, and remove any `useEffect` calling `scrollTo(0, 0)` or `element.focus({ preventScroll: false })` on re-render! When focusing a page programmatically, always pass `{ preventScroll: true }` so scrolling down a multi-page document NEVER flickers or snaps back to the top.
3. **Visible, Always-Accessible Header, Footer & `+ Add New Page`:**
   - Render a clearly delineated **Header Zone** (`border-bottom: 1px dashed #cbd5e1`, placeholder `\"Click to edit header\"`) at the top of every page and a **Footer Zone** (`border-top: 1px dashed #cbd5e1`, placeholder `\"Click to edit footer\"`, plus `Page X of Y`) at the bottom of every page.
   - Make **`+ Page` / `Blank Page` / `+ Blank Page / New Page`** immediately append a new page object to `doc.pages`, scroll smoothly to the new page, and place the cursor inside the new page's body.
4. Write automated DOM tests verifying that at `zoom = 100%`, typed text is rendered inside the visible viewport (`getBoundingClientRect().top >= 0`), scrolling down preserves `scrollTop`, Header/Footer inputs are editable, and clicking `+ Page` increases page count from 1 to 2. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "29.1.2"; stage = 29; phase = "29.1"; title = "Word Complete Ribbon Overhaul: Font Type/Size, Home, Insert, Layout, References, Review, View"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Word Font family/size, paragraph styling, tables, images, shapes, layout, references, review, and view tools."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 29.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. **Fix Word Font Family, Font Size & Home Ribbon Not Working:**
   - Why Font Type & Font Size failed: clicking a `<select>` dropdown steals DOM focus from the `contentEditable` page, losing the user's text selection!
   - **Solution:** Track the last valid `Selection` `Range` in a ref (`savedSelectionRangeRef`) on `onSelect`/`onKeyUp`/`onMouseUp` inside the page editor. Whenever the user changes **Font Family**, **Font Size**, **Text Color**, **Highlight Color**, or clicks **Bold, Italic, Underline, Strikethrough, Subscript, Superscript, Align, Bullets, Numbering, Line Spacing, or Styles (`Heading 1..3`, `Title`)**, first restore `savedSelectionRangeRef` (or if collapsed, apply directly to the active paragraph / current typing span style) so it works 100% of the time whether text is highlighted or the user is about to type!
2. **Fix All Remaining Word Ribbon Tabs (`Insert`, `Layout`, `References`, `Review`, `View`):**
   - **Insert Tab:** `Cover Page`, `Blank Page`, `Page Break`, `Table` (inserts editable table with `+ Row`, `+ Col`, `Delete Table` mini-bar), `Picture` (uploads local image file onto the page with resize slider), `Shapes` (inserts draggable/resizable SVG shape with text edit & delete button), `Hyperlink` (modal to insert clickable `<a href>`), `Header & Footer` (focuses header/footer), `Page Number`, `Text Box`, `Date & Time`, `Symbol`, `Horizontal Line`, `Equation (∑, √, π, ∫)`.
   - **Layout Tab:** `Margins` (`Normal`, `Narrow`, `Wide`, `Custom` — updates page padding), `Orientation` (`Portrait` / `Landscape` — swaps page width/height), `Size` (`A4`, `Letter`, `Legal`), `Columns` (`1`, `2`, `3` — sets `columnCount` on page body), `Page Color` (changes paper background color), `Page Borders` (adds solid/double/dashed frame around page), `Watermark` (renders diagonal watermark text behind page content).
   - **References Tab:** `Insert Table of Contents` (generates live linked TOC from all headings), `Insert Footnote` (adds superscript number + footnote at bottom of page), `Insert Endnote`, `Insert Citation & Bibliography`, `Insert Caption`.
   - **Review Tab:** `Spell Check & Grammar`, `Word Count` dialog, `Read Aloud (TTS)`, `New Comment` (adds comment card in right sidebar linked to selected text), `Delete Comment`, `Track Changes` (on/off), `Accept All`, `Reject All`, `Protect Document`.
   - **View Tab:** `Print Layout`, `Web Layout`, `Read Mode`, `Ruler`, `Gridlines`, `Nav Pane`, `Zoom (50%..200%)`, `One Page`, `Multi Pages` (displays pages side-by-side in a 2-column grid!), `Page Width`.
3. Write automated tests clicking and asserting every single Word menu item. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 30: EXCEL (.XLSX) — MULTI-ROW/COL SELECTION, BATCH EDITING & ALL RIBBONS
    # ==========================================================================
    @{
        id = "30.1.1"; stage = 30; phase = "30.1"; title = "Excel Multi-Row & Multi-Column Header Selection + Batch Row/Col Formatting & Editing"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Click/drag row headers (1..N) and column headers (A..Z) to select whole rows/columns and apply batch formatting, formulas, resize, insert, or delete."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 30.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.2.
1. **Implement Full Row, Multi-Row, Column, Multi-Column & Rectangular Range Selection in Excel:**
   - **Column Selection:** Clicking column header `B` selects the entire Column `B` (highlighting all cells in Column `B` and updating the Name Box to `B:B`). Clicking `B` and dragging across to `E` (or `Shift+Click` on `E`) selects Columns `B:E`. `Ctrl+Click` adds non-contiguous columns.
   - **Row Selection:** Clicking row header `3` selects the entire Row `3` (`3:3`). Dragging `3` to `10` (or `Shift+Click` on `10`) selects Rows `3:10`.
   - **Select All Sheet Corner (`◢`):** Clicking the top-left corner cell above `1` and left of `A` selects the entire worksheet!
   - **Cell Range Drag Selection:** Clicking cell `B2` and dragging to `D8` (or `Shift+Arrow`) selects the 2D range `B2:D8`.
2. **Row-Wise & Column-Wise Batch Operations:**
   - When multiple rows, columns, or cell ranges are selected, clicking ANY formatting or editing command (Bold, Italic, Underline, Font Family, Font Size, Text Color, Fill Color, Borders, Align Left/Center/Right, Number Format `$ / % / , / .00`, Wrap Text, Clear Contents, Delete, Insert Rows/Columns, Resize Column Width / Row Height) MUST apply simultaneously to **every cell in the selected rows/columns/range**!
3. Write automated DOM tests verifying: clicking Column `B` and dragging to `D`, applying Bold & Currency `$` format, and asserting cells `B1..D20` all receive Bold & Currency formatting; doing the same for Rows `2..5`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "30.1.2"; stage = 30; phase = "30.1"; title = "Excel Complete Ribbon Overhaul: Home, Insert, Page Layout, Formulas, Data, Review, View"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Excel ribbon options across all 7 tabs with zero overflow and automated verification."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 30.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Ensure **100% of Excel's Ribbon controls** across all 7 tabs are accessible (zero right-edge clipping) and fully functional:
   - **Home Tab:** Clipboard (`Cut`, `Copy`, `Paste`, `Paste Values`), Font (`Family`, `Size`, `A+`, `A-`, `Bold`, `Italic`, `Underline`, `Borders`, `Fill Color`, `Text Color`), Alignment (`Top/Mid/Bottom`, `Left/Center/Right`, `Wrap Text`, `Merge & Center`), Number (`Format Dropdown`, `$`, `%`, `,`, `.00`, `.0`), Styles (`Conditional Formatting` rules + `Format as Table` banded themes), Cells (`Insert Row`, `Insert Col`, `Delete Row`, `Delete Col`, `AutoFit Row Height`, `AutoFit Col Width`), Editing (`AutoSum Σ`, `Fill Down`, `Fill Right`, `Fill Series`, `Clear All/Formats/Contents`, `Sort & Filter`, `Find & Replace`).
   - **Insert Tab:** `PivotTable` (opens interactive Pivot Table builder grouping selected range by Row/Column/Sum/Average!), `Table`, `Picture`, `Shapes`, `Column Chart`, `Line Chart`, `Pie Chart`, `Area/Scatter Chart` (interactive floating charts on grid), `Sparklines (Line/Column)`, `Link`, `Comment`.
   - **Page Layout Tab:** `Themes`, `Colors`, `Fonts`, `Margins`, `Orientation`, `Size`, `Set Print Area`, `Clear Print Area`, `Gridlines (View / Print)`, `Headings (View / Print)`.
   - **Formulas Tab:** `Insert Function (fx)` modal, `AutoSum (Sum, Average, Count, Max, Min)`, `Financial`, `Logical`, `Text`, `Date & Time`, `Lookup & Reference`, `Math & Trig`, `Name Manager`, `Show Formulas`, `Trace Precedents/Dependents`, `Evaluate Formula`.
   - **Data Tab:** `From CSV/JSON`, `Sort A-Z`, `Sort Z-A`, `Custom Multi-Column Sort`, `Filter` (interactive column header filter checkboxes), `Clear Filter`, `Text to Columns`, `Flash Fill`, `Remove Duplicates`, `Data Validation`, `Group / Ungroup Rows & Cols`, `Subtotal`, `Goal Seek`.
   - **Review & View Tabs:** `New Comment`, `Delete Comment`, `Show All Comments`, `Protect Sheet`, `Workbook Stats`, `Normal / Page Break View`, `Formula Bar`, `Gridlines`, `Headings`, `Freeze Top Row / First Column / Selected Cell`, `Split Sheet`, `Zoom`.
2. Verify all Excel ribbon buttons via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 31: POWERPOINT (.PPTX) — OPEN/SAVE, EXPANDED RIBBON & ALL FEATURES
    # ==========================================================================
    @{
        id = "31.1.1"; stage = 31; phase = "31.1"; title = "PowerPoint Real .pptx Open/Save & Expanded Home, Insert, Design, Transitions Ribbons"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Real .pptx file Open/Save, non-overflowing rich ribbon, SmartArt, Charts, Tables, Images, Align/Distribute/Group, and Design Themes."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 31.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.3.
1. **Fix Opening & Saving Real `.pptx` Files:**
   - Wire `File -> Open` and the top `📂 Open` button in PowerPoint so selecting an existing `.pptx` file unzips `ppt/slides/slide*.xml` (using `ooxml-opc` / JSZip), parses all slide text boxes, shapes, colors, and positions, and loads them into the editable slide deck!
   - Wire `File -> Save` / `💾 Save` (`Ctrl+S`) to package all current slides, shapes, text, themes, and notes into a valid downloadable/saveable `.pptx` archive!
2. **Fix Ribbon Overflow & Expand Missing PowerPoint Controls (`Home`, `Insert`, `Design`, `Transitions`):**
   - **Home Tab:** `+ New Slide` (with 6 layouts), `Duplicate Slide`, `Delete Slide`, `Move Up/Down`, `Reset Layout`, Clipboard (`Cut`, `Copy`, `Paste`), Font (`Family`, `Size`, `A+`, `A-`, `Bold`, `Italic`, `Underline`, `Strikethrough`, `Text Shadow`, `Text Color`, `Highlight`, `Clear`), Paragraph (`Bullets`, `Numbering`, `Left/Center/Right/Justify`, `Top/Middle/Bottom` vertical align, `Line Spacing`, `Columns`), Drawing & Arrange (`Text Box`, `12 Quick Shapes`, `Shape Fill`, `Shape Outline`, `Border Weight`, `Bring to Front`, `Send to Back`, `Align Left/Center/Right/Top/Middle/Bottom`, `Rotate 90°`, `Duplicate Shape`, `Delete Shape`), `Find & Replace`.
   - **Insert Tab:** `New Slide`, `Table` (inserts editable table on slide), `Pictures` (upload local image onto slide), `Shapes` (full 20-shape gallery), `Icons & Badges`, `SmartArt` (inserts editable Process, Cycle, Hierarchy, Pyramid diagrams), `Chart` (inserts editable Bar, Line, Pie chart on slide), `Text Box`, `Header & Footer`, `WordArt`, `Date & Time`, `Slide Number`, `Equation / Symbol`, `Audio / Video Clip`.
   - **Design Tab:** 12 curated High-Contrast Presentation Themes (with live preview thumbnails), `Color Variants`, `Slide Size (16:9 Widescreen vs 4:3 Standard)`, and `Format Background` (Solid Color, Gradient, Pattern, Image, `Apply to All Slides`).
   - **Transitions Tab:** 12 Slide Transitions (`None`, `Fade`, `Push`, `Wipe`, `Split`, `Reveal`, `Cut`, `Cover`, `Flash`, `Zoom`, `Flip 3D`, `Morph`), `Effect Direction`, `Duration`, `Sound`, `Advance On Click / After X sec`, `Apply to All`, and `Preview` (plays transition immediately on canvas).
3. Write automated tests verifying `.pptx` Open/Save and every control in `Home`, `Insert`, `Design`, and `Transitions`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "31.1.2"; stage = 31; phase = "31.1"; title = "PowerPoint Animations, Slide Show, Presenter Console, Review & View Ribbons"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Animations timeline, Fullscreen Slide Show, Presenter View, Laser/Pen annotation on slides, Slide Sorter, and Notes."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Wire **100% of the controls** in PowerPoint's `Animations`, `Slide Show`, `Review`, and `View` ribbons:
   - **Animations Tab:** Entrance (`Appear`, `Fade In`, `Fly In`, `Float In`, `Split`, `Wipe`, `Shape`, `Wheel`, `Random Bars`, `Grow & Turn`, `Zoom`, `Swivel`, `Bounce`), Emphasis (`Pulse`, `Color Pulse`, `Teeter`, `Spin`, `Grow/Shrink`), Exit (`Disappear`, `Fade Out`, `Fly Out`, `Float Out`, `Shrink & Turn`), `Effect Options`, `Add Animation`, `Animation Pane` (right sidebar showing ordered animation steps per slide with Play, Move Up/Down, Delete), `Trigger (On Click / With Previous / After Previous)`, `Duration`, `Delay`, and `Preview Animation`.
   - **Slide Show Tab:** `From Beginning (F5)`, `From Current Slide (Shift+F5)`, `Presenter View` (launches split presenter console with current slide, next slide, speaker notes, live timer, laser pointer, and freehand slide pen!), `Custom Slide Show`, `Set Up Slide Show` (Loop continuously, Kiosk mode), `Hide Slide`, `Rehearse Timings` (records per-slide rehearsal clock), `Record Slide Show`.
   - **Review & View Tabs:** `Spelling`, `Thesaurus`, `New Comment` (attaches comment pins to slide coordinates + right Comments drawer), `Delete/Prev/Next Comment`, `Compare`, `Normal View`, `Outline View` (edit all slide titles/bullets in a continuous document tree), `Slide Sorter View` (grid of all slides with drag-to-reorder), `Notes Page View`, `Reading View`, `Ruler`, `Gridlines`, `Guides` (center alignment crosshairs), `Speaker Notes` bottom panel, `Color / Grayscale / Black & White` mode, and Zoom slider.
2. Verify all buttons across `Animations`, `Slide Show`, `Review`, and `View` via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 32: PDF STUDIO (.PDF) — LOCAL PDF OPEN/SAVE, REAL SIGNATURE PAD & FULL EDITING
    # ==========================================================================
    @{
        id = "32.1.1"; stage = 32; phase = "32.1"; title = "Fix Local PDF File Open/Render/Save & Replace Hardcoded 'John Doe' with Real Signature Pad"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "Open any local .pdf file and render its pages/text, save edited .pdf, and interactive Signature/Initials Pad (Draw, Type, or Upload Image) with zero 'John Doe' defaults."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 32.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v5.0 Article III.4.
1. **Remove Hardcoded `\"John Doe\"` Signature Bug (Seen in image_b643a0.png) & Build Real Signature Modal:**
   - Delete `\"John Doe\"` completely from the codebase!
   - When the user clicks `Add Signature`, `Initials`, or `Sign` in the `Sign & Protect` tab:
     - Open an interactive **Signature & Initials Studio Modal** with 3 tabs:
       1. **Draw Signature:** An HTML5 `<canvas>` pad where the user draws their real signature with mouse, trackpad, or touch/stylus (with ink color picker: Black, Classic Blue, Fountain Red, and `Clear` button). Captures the exact drawn stroke as a transparent PNG (`canvas.toDataURL(\"image/png\")`) and places it as a draggable, resizable signature stamp on the active PDF page!
       2. **Type Signature:** An `<input type=\"text\" placeholder=\"Enter your full name or initials...\" />` where the user types their own name (e.g., `\"Dayanand T.\"`), chooses from 4 signature script styles, and stamps their exact typed name + optional timestamp onto the PDF page!
       3. **Upload Signature Image:** Upload a scan/photo of a real signature (`.png`/`.jpg`) and stamp it onto the page.
2. **Fix Opening & Saving Real Local `.pdf` Files:**
   - When the user clicks `Open` (in `File`, `Home`, or top quick bar) and selects any `.pdf` file from their computer:
     - Parse the binary `ArrayBuffer` using `pdf-engine` / client PDF parser to extract the actual pages, page dimensions, embedded text blocks (`BT..ET` / `Tj` / `TJ`), and render high-resolution page views + thumbnails!
   - When clicking `Save` or `Export PDF`, serialize all pages, edited text blocks, added shapes/images, form values, redactions, and drawn/typed signature images into a valid `.pdf` binary and download/save it to disk!
3. Write automated tests verifying local `.pdf` buffer loading, drawing a signature on the canvas pad, typing a custom name (`\"Custom Signer\"`), asserting `\"John Doe\"` never appears, and saving the `.pdf`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "32.1.2"; stage = 32; phase = "32.1"; title = "PDF Studio Complete Ribbon Overhaul: Home, Edit PDF, Forms, Comment, Page Tools, Protect, View"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "100% working PDF editing, text/image modification, annotations, redaction, form builder/filler, page resizing, and password security."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 32.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v5.0.
1. Make **100% of PDF Studio's Ribbon tabs and tools** work seamlessly on the active PDF page:
   - **Home Tab:** `Open PDF`, `Create Blank PDF`, `Save PDF`, `Print`, `Hand Tool` (drag to scroll), `Select Tool` (click/drag to move/resize/edit any element on the page), `Snapshot`, `Fit Page`, `Fit Width`, `Actual Size (100%)`, `Zoom In (+)`, `Zoom Out (-)`, `First / Prev / Next / Last Page`, and `Search Text` (highlights matching text across PDF pages).
   - **Edit PDF Tab:** `Edit Text Mode` (makes all text blocks on the PDF page directly editable in-place with Font Family, Font Size, Color, Bold, Italic, Align, and Delete controls!), `+ Add Text Box` (places a new editable text box on the page), `+ Add Image` (uploads and places a resizable image on the page), `Crop Page`, `Header & Footer` (adds custom top/bottom text & page numbers across pages), `Watermark` (adds customizable diagonal/centered watermark text with opacity & rotation), `Background Color`.
   - **Forms Tab:** `+ Text Field`, `+ Checkbox`, `+ Radio Button`, `+ Dropdown List`, `+ Date Picker`, `+ Signature Field`, `Highlight Existing Fields`, `Clear All Form Fields`, `Export Form Data (JSON/CSV)`.
   - **Comment & Annotate Tab:** `Highlight`, `Underline`, `Strikethrough`, `Sticky Note` (adds comment pin + syncs with left `Notes` list), `Text Callout`, `Freehand Ink Pen` (draw directly over the PDF page with color & thickness controls), `Eraser`, `Shapes (Rectangle, Circle, Arrow, Line)`, `Stamps (APPROVED, CONFIDENTIAL, DRAFT, FINAL, SIGNED, URGENT)`.
   - **Page Tools Tab:** `+ Insert Blank Page`, `Duplicate Page`, `Delete Page`, `Rotate 90° CW`, `Rotate 90° CCW`, `Move Page Up/Down`, **`Resize Page Size` (`A4`, `Letter`, `Legal`, `A3`, `Custom Width × Height` with content scaling)**, `Extract Page`, `Split PDF`, `Merge Another PDF`.
   - **Sign & Protect Tab:** `Add Signature` (Draw/Type/Upload modal), `Add Initials`, `Encrypt with Password` (locks PDF with password + shows lock screen until unlocked), `Decrypt / Remove Password`, `Restrict Editing`, `Mark for Redaction` & `Apply Redactions` (permanently blacks out and deletes underlying content), `Compress PDF`.
2. Verify every single PDF button and modal with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 33: NOTEPAD / TXT (.TXT / .MD) — PRO FEATURES & COMPLETE RIBBON
    # ==========================================================================
    @{
        id = "33.1.1"; stage = 33; phase = "33.1"; title = "Notepad / Notes Complete Ribbon Overhaul: File, Home, Edit, View, Markdown & Power Tools"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Notepad/Markdown editor with expanded ribbons, encoding/EOL controls, JSON/XML formatter, regex find/replace, and live markdown preview."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 33.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.5.
1. Expand and fix **100% of the menu options** in the **Notepad / Notes (`N`)** workspace (`image_b642e4.png`):
   - **Fix Zoom Default Bug:** In `image_b642e4.png`, the bottom-right zoom slider says `20%` by default! Ensure all 5 apps (Word, Excel, PPT, PDF, Notes) initialize at **`100%` zoom** by default!
   - **Home Tab:** `New Note (+ Scratchpad)`, `Open (.txt, .md, .json, .xml, .csv, .log, .ini, .yaml)`, `Save (.txt)`, `Save (.md)`, `Rename Note`, `Delete Note`, `Print`, Font Family (`Consolas`, `JetBrains Mono`, `Courier New`, `Inter`, `System UI`), Font Size (`10px`–`32px`), `Zoom In / Out / 100%`, `Time/Date (F5)`.
   - **Edit Tab:** `Undo`, `Redo`, `Cut`, `Copy`, `Paste`, `Delete`, `Select All`, `Find` (`Ctrl+F` interactive search bar with match counter & Next/Prev), `Replace` (`Ctrl+H` with `Replace` & `Replace All`, Case-Sensitive, and Regex mode), `Go To Line (Ctrl+G)`, `Duplicate Line (Ctrl+D)`, `Delete Line (Ctrl+Shift+K)`, `Move Line Up/Down`.
   - **Text Tools / Transform Tab (New Pro Features):** `UPPERCASE`, `lowercase`, `Title Case`, `Sentence case`, `Invert Case`, `Sort Lines A→Z`, `Sort Lines Z→A`, `Remove Duplicate Lines`, `Remove Empty Lines`, `Trim Leading/Trailing Whitespace`, `Format / Prettify JSON`, `Minify JSON`, `Base64 Encode / Decode`, `URL Encode / Decode`, `Word & Frequency Statistics`.
   - **View & Markdown Tabs:** `Editor Only`, `Split View (Editor + Live Preview)`, `Preview Only`, `Line Numbers On/Off`, `Word Wrap On/Off`, `Show Whitespace`, `Line Endings (CRLF / LF)`, `Encoding (UTF-8 / UTF-16 / ASCII)`, and working Markdown formatting buttons (`H1`, `H2`, `H3`, `Bold`, `Italic`, `Strikethrough`, `Inline Code`, `Code Block`, `Blockquote`, `Bullet List`, `Numbered List`, `Task Checkbox - [ ]`, `Insert Table`, `Insert Link`, `Insert Image`, `Horizontal Rule`).
2. Verify all Notepad/Notes buttons, text transforms, JSON formatter, Find & Replace, and file I/O via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 34: 100% AUTOMATED MENU & FEATURE VERIFICATION HARNESS
    # ==========================================================================
    @{
        id = "34.1.1"; stage = 34; phase = "34.1"; title = "Comprehensive Automated Menu Crawler & E2E Assertion Suite (250+ Automated Checks)"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Automated test suite crawling every tab and button across Word, Excel, PowerPoint, PDF, and Notepad and verifying state/DOM mutations."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 34.1.1
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Build an exhaustive automated verification suite in `apps/unified-shell/src/__tests__/gec_v5_complete_audit.test.tsx` that automatically tests every single menu item and user workflow so the user never has to manually hunt for broken buttons:
   - **Automated Ribbon Crawler:** Programmatically iterates through all 5 apps (`word`, `excel`, `ppt`, `pdf`, `notes`), clicks every single Ribbon Tab (`Home`, `Insert`, `Layout`, `Page Layout`, `Formulas`, `Data`, `Design`, `Transitions`, `Animations`, `Slide Show`, `Edit PDF`, `Forms`, `Comment & Annotate`, `Page Tools`, `Sign & Protect`, `References`, `Review`, `View`, `Edit`, `Text Tools`, `Markdown`), inspects every `<button>` and `<select>` rendered inside that tab, verifies none are clipped off-screen or disabled without cause, clicks each control, and asserts zero runtime exceptions and expected state transitions!
   - **Targeted Bug Regression Assertions:**
     1. Asserts Word initializes at `100%` zoom, typed text is visible at `100%` zoom, scrolling does not reset `scrollTop`, Header/Footer are editable, Font Family/Size mutate text, `+ Page` creates Page 2, and `.docx` Open/Save work.
     2. Asserts Excel supports clicking/dragging Row headers (`1..5`) and Column headers (`A..C`) to select multiple whole rows/columns and applies batch formatting (Bold, Fill Color, Currency `$`, Clear) across all cells in those rows/columns.
     3. Asserts PowerPoint opens/saves `.pptx`, supports all ribbon tools, and has readable high-contrast slide thumbnail text in both Light and Dark mode.
     4. Asserts PDF Studio opens/saves `.pdf`, supports Draw Signature and Type Custom Name signature stamps (and asserts `\"John Doe\"` does not exist anywhere!), encrypts/decrypts passwords, resizes page dimensions, and edits PDF text/annotations.
     5. Asserts opening multiple simultaneous `.docx`, `.xlsx`, `.pptx`, `.pdf`, and `.txt` tabs and activating `Split View` works smoothly.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1`, ensure 100% of Rust and Frontend tests pass, build the release binary, and commit + push to origin with `[GEC-Verified]`.
"@
    }
)

# 1. Write Subphase Prompt Files (.agents/stages/stage-28..34/subphase-X.Y.Z.md)
foreach ($sp in $v5Subphases) {
    $stageFolder = ".agents/stages/stage-$($sp.stage)"
    $filePath = "$stageFolder/subphase-$($sp.id).md"
@"
# Subphase $($sp.id): $($sp.title)
- **Stage:** $($sp.stage) | **Phase:** $($sp.phase)
- **Target Subsystem:** ``$($sp.crate)``
- **Primary Agent Skill:** ``$($sp.skill)``
- **Deliverable:** $($sp.deliverable)
- **Validation Command:** ``$($sp.validation)``

---

## Antigravity IDE Execution Prompt

``````text
$($sp.prompt)
``````
"@ | Set-Content -Path $filePath -Encoding UTF8
    Write-Host "  Created prompt spec: $filePath" -ForegroundColor Gray
}

# 2. Write Slash Workflows (.agents/workflows/stage-28.md .. stage-34.md)
$stageTitles = @{
    28 = 'Stage 28: Modern Visual Overhaul, Non-Overflowing Ribbon, High-Contrast Dark Mode and Multi-Document Split View'
    29 = 'Stage 29: Word (.docx) Fix - 100% Zoom Text Visibility, Scroll Flicker Fix, Headers/Footers and All Ribbons'
    30 = 'Stage 30: Excel (.xlsx) Fix - Multi-Row/Column Selection, Batch Row/Col Formatting and All Ribbons'
    31 = 'Stage 31: PowerPoint (.pptx) Fix - Real .pptx Open/Save, Expanded Ribbons, Animations and Presenter View'
    32 = 'Stage 32: PDF Studio (.pdf) Fix - Local PDF Open/Save, Custom Drawn/Typed Signature Pad and Full Editing'
    33 = 'Stage 33: Notepad / Notes (.txt/.md) Fix - 100% Default Zoom, Text Power Tools and Complete Ribbons'
    34 = 'Stage 34: Automated Ribbon Crawler and 250+ Assertion Production Verification Gate'
}

foreach ($s in 28..34) {
    $items = $v5Subphases | Where-Object { $_.stage -eq $s }
    $wfText = @"
description: Automated execution workflow for $($stageTitles[$s])
Workflow: Execute Stage $s (/stage-$s)
Objective: $($stageTitles[$s])

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

"@
    foreach ($sp in $items) {
        $wfText += @"

### Subphase $($sp.id): $($sp.title)
- Spec File: .agents/stages/stage-$s/subphase-$($sp.id).md
$($sp.prompt)

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete $($sp.id)

"@
    }
    $wfText | Set-Content -Path ".agents/workflows/stage-$s.md" -Encoding UTF8
    Write-Host "  Created workflow: .agents/workflows/stage-$s.md" -ForegroundColor Gray
}

# 3. Append Stages 28-34 Non-Destructively to specs/stage-tracker.json preserving Stages 1-27
$trackerPath = "specs/stage-tracker.json"
if (Test-Path $trackerPath) {
    $tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json
    $preservedSubphases = @($tracker.subphases | Where-Object { [int]$_.stage -lt 28 })

    $v5Entries = @()
    foreach ($sp in $v5Subphases) {
        $v5Entries += [pscustomobject][ordered]@{
            id           = $sp.id
            stage        = [int]$sp.stage
            phase        = $sp.phase
            title        = $sp.title
            crate        = $sp.crate
            rea_required = $sp.rea
            status       = "PENDING"
            prompt_file  = ".agents/stages/stage-$($sp.stage)/subphase-$($sp.id).md"
            completed_at = $null
        }
    }
    $tracker.subphases = $preservedSubphases + $v5Entries
    $tracker.contract = "GEC-v5.0"
    $tracker.total_stages = 34
    $tracker.total_subphases = $tracker.subphases.Count
    $tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8
    Write-Host "  Preserved $($preservedSubphases.Count) subphases for Stages 1-27. Appended $($v5Entries.Count) subphases for Stages 28-34 to specs/stage-tracker.json (Total: $($tracker.total_subphases) subphases)." -ForegroundColor Green
}

Write-Host "=== [GEC-v5.0 Ready] Run /stage-28 through /stage-34 in Antigravity IDE! ===" -ForegroundColor Green
