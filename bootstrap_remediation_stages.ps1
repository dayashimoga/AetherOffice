# ==============================================================================
# bootstrap_remediation_stages.ps1 - Appends Stages 8 to 13 (24 Subphases)
# Fixes all UI/Engine gaps: Standalone Apps, Real Editing, New Pages/Sheets/Slides,
# Context-Specific Ribbons, PDF Studio UI, and Native File Open/Save.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [Remediation 3/3] Generating Stages 8-13 Prompts & Workflows ===" -ForegroundColor Cyan

foreach ($s in 8..13) {
    New-Item -ItemType Directory -Force -Path ".agents/stages/stage-$s" | Out-Null
}

$remediationSubphases = @(
    # ==========================================================================
    # STAGE 8: ARCHITECTURE DE-CLUBBING, 5 BINARIES & MAIN SUITE LAUNCHER
    # ==========================================================================
    @{
        id = "8.1.1"; stage = 8; phase = "8.1"; title = "5-Executable Build Wiring & App-Mode Boot Router"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "5 compiled binaries (aether-hub, aether-writer, aether-calc, aether-slides, aether-pdf) routing directly into isolated app shells."
        validation = "cargo build --manifest-path apps/unified-shell/src-tauri/Cargo.toml --bins"
        prompt = @"
/execute-subphase 8.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. In `apps/unified-shell/src-tauri/Cargo.toml`, register the 5 `[[bin]]` targets (`aether-hub`, `aether-writer`, `aether-calc`, `aether-slides`, `aether-pdf`) pointing to `src/bin/*.rs`.
2. Expose a Tauri IPC command `get_boot_config()` in `apps/unified-shell/src-tauri/src/lib.rs` that returns `{ app_mode: "hub" | "writer" | "calc" | "slides" | "pdf", initial_file: Option<String> }` read from `AETHER_APP_MODE` and CLI `std::env::args()`.
3. Remove the hardcoded boot tabs (`Document 1 *`, `Financials.xlsx *`, `Pitch.pptx *`) from `apps/unified-shell/src/`. When launched as `aether-writer`, mount ONLY the Writer App shell with a clean `Untitled 1.docx`; when `aether-calc`, mount ONLY Calc with `Book1.xlsx`; when `aether-slides`, mount ONLY Slides with `Presentation1.pptx`; when `aether-pdf`, mount ONLY PDF Studio.
4. Write a Vitest test asserting each boot mode mounts strictly its own isolated application shell and zero cross-format tabs. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "8.1.2"; stage = 8; phase = "8.1"; title = "AetherHub Main Suite Launcher & OS Process Spawner"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Main Suite Hub window with App Launch Cards (Word, Excel, PowerPoint, PDF), Recent Files, Templates, and OS child-process spawning."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 8.1.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.
1. Build the `AetherHub` main application view (`apps/unified-shell/src/apps/hub/HubLauncher.tsx`) displayed when `app_mode == "hub"`:
   - 4 Prominent App Launch Cards: **Writer (Word .docx)**, **Calc (Excel .xlsx)**, **Slides (PowerPoint .pptx)**, and **PDF Studio (.pdf)**.
   - Template Gallery (Blank Document, Financial Budget Sheet, Pitch Deck, Blank/Form PDF) and Recent Files list persisted in local storage / OS config dir.
2. Implement the Tauri command `launch_standalone_app(app_mode: String, file_path: Option<String>)` in `src-tauri/src/lib.rs` that spawns a separate native OS window (`tauri::WebviewWindowBuilder`) or child process for the chosen app, AND allow switching/managing open windows from the Hub.
3. Verify with automated tests and commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "8.2.1"; stage = 8; phase = "8.2"; title = "Native File Open, Save, Save-As & Backstage File Menu"
        crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false
        deliverable = "Working File Backstage menu (New, Open, Save, Save As, Export PDF, Print) wired to real disk I/O for .docx, .xlsx, .pptx, and .pdf."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 8.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v2.0.
1. Fix the broken `File` tab in the ribbon: clicking `File` MUST open a full Backstage overlay menu containing:
   - **New** (Blank Document / Sheet / Presentation / PDF, plus reset current state).
   - **Open...** (Triggers native OS file picker or path input, reads `.docx`, `.xlsx`, `.pptx`, or `.pdf` bytes from disk through `word-engine`, `excel-engine`, `ppt-engine`, or `pdf-engine`, and populates the live canvas).
   - **Save / Save As...** (Serializes the current in-memory Rust engine state back to a valid `.docx`, `.xlsx`, `.pptx`, or `.pdf` file on disk and clears the dirty `*` indicator).
   - **Export to PDF** (Renders the current document/sheet/slide deck into a valid ISO 32000 PDF via `pdf-engine`).
2. Write integration tests verifying round-trip disk Open -> Edit -> Save for all 4 formats. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "8.2.2"; stage = 8; phase = "8.2"; title = "Strict Per-App Contextual Ribbon Router & Mobile Bar Cleanup"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Dedicated Ribbon schemas per app, fixed active-tab underline indicator, and removal of mobile bottom bar on desktop viewports."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 8.2.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.2.
1. Fix the Ribbon Tab state bug seen in screenshots where `File` stays underlined while `Home` is shown:
   - Maintain a strict `activeRibbonTab` state per window (`Home`, `Insert`, `Layout`, etc.) where clicking any tab updates the blue active indicator AND immediately switches the toolbar panel below it.
2. Separate the Ribbon definitions into 4 isolated components (`WriterRibbon.tsx`, `CalcRibbon.tsx`, `SlidesRibbon.tsx`, `PdfRibbon.tsx`):
   - **Writer ONLY:** Home, Insert, Layout, References, Review, View.
   - **Calc ONLY:** Home, Insert, Page Layout, Formulas, Data, Review, View.
   - **Slides ONLY:** Home, Insert, Design, Transitions, Animations, Slide Show, Review, View (Remove Word's `Normal/Heading 1` styles from Slides!).
   - **PDF ONLY:** Home, Edit PDF, Forms, Comment & Annotate, Page Tools, Sign & Protect, View.
3. Hide the mobile bottom drawer handle and bottom touch bar whenever `window.innerWidth >= 768px`.
4. Verify with DOM tests asserting every tab click renders unique, non-placeholder buttons. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 9: STANDALONE WORD (WRITER) - FULL EDITING, PAGINATION & RIBBON
    # ==========================================================================
    @{
        id = "9.1.1"; stage = 9; phase = "9.1"; title = "Interactive Paginated Word Canvas, Real Caret & Keyboard Typing"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Live editable A4/Letter pages where typing, Backspace, Enter, Arrow keys, and mouse click/drag text selection work seamlessly."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 9.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the static non-editable text block in Writer (`apps/unified-shell/src/apps/writer/`) with a true interactive multi-page document editor synchronized with `word-engine`:
   - Support full keyboard input (`keydown`, `InputEvent`, IME composition, `Backspace`, `Delete`, `Enter` for new paragraph, `Shift+Enter` for line break, Arrow/Home/End/PageUp/PageDown cursor navigation).
   - Support mouse click to place the blinking caret at the exact character offset, double-click to select word, triple-click to select paragraph, and click-drag to highlight text ranges across runs and paragraphs.
2. Ensure every edit dispatches `UiActionEvent` to `word-engine` and immediately re-renders the updated paragraphs on screen.
3. Write automated DOM tests simulating typing a 3-paragraph document, selecting text, and deleting characters. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "9.1.2"; stage = 9; phase = "9.1"; title = "Multi-Page Creation, Page Breaks, Headers/Footers & Zoom/Ruler"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Automatic page overflow creation, manual Page Break (Ctrl+Enter), '+ New Page' button, editable Header/Footer zones, and working Ruler."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 9.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement multi-page layout and creation in Writer:
   - **Automatic Pagination:** When paragraphs exceed the vertical body height of Page 1 (`11in` minus top/bottom margins), automatically flow overflow paragraphs onto Page 2, Page 3, etc., rendered as distinct visual paper sheets separated by a page gap.
   - **Manual Page Creation:** Wire `Insert -> Blank Page`, `Insert -> Page Break`, and `Ctrl+Enter` to immediately insert a new page and move the cursor to the top of the new page.
   - **Editable Headers & Footers:** Double-clicking the top or bottom margin of any page activates the Header/Footer editor with dynamic `Page X of Y` numbering.
   - **Interactive Horizontal Ruler:** Dragging the left/right indent markers on the top ruler dynamically updates page margins and paragraph indentation.
2. Write DOM tests verifying that `Insert -> Blank Page` and `Ctrl+Enter` increase the rendered `.word-page` count from 1 to 2+. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "9.2.1"; stage = 9; phase = "9.2"; title = "Writer Home & Layout Ribbons: Rich Run Formatting, Lists & Styles"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Working Font family/size dropdowns, Color pickers, Bold/Italic/Underline/Strike, Bullets/Numbering, Line Spacing, and Style gallery."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 9.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every control in `WriterRibbon.tsx` (`Home` and `Layout` tabs) to mutate the selected text range or active cursor formatting in `word-engine`:
   - **Font Group:** Font Family dropdown (`Calibri`, `Arial`, `Times New Roman`, `Inter`, `Consolas`, `Georgia`), Font Size dropdown (`8` to `72`), Grow/Shrink Font (`A+`, `A-`), Text Color picker, Highlight Color picker, and Clear Formatting button.
   - **Paragraph Group:** Toggle Bullet List, Toggle Numbered List (`1.`, `a.`, `i.`), Decrease/Increase Indent, Left/Center/Right/Justify alignment, and Line Spacing (`1.0`, `1.15`, `1.5`, `2.0`).
   - **Styles Gallery:** Clicking `Normal`, `Heading 1`, `Heading 2`, `Heading 3`, `Title`, or `Subtitle` applies the style to the current paragraph and automatically updates the left **Navigation Outline Pane** with clickable heading links!
   - **Layout Tab:** Margins preset selector (`Normal`, `Narrow`, `Wide`), Orientation (`Portrait` / `Landscape`), Paper Size (`A4` / `Letter`), and Columns (`1`, `2`, `3`).
2. Verify all formatting actions with automated DOM tests and commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "9.2.2"; stage = 9; phase = "9.2"; title = "Writer Insert, Review & View Ribbons: Tables, Images, Find/Replace & Word Count"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Interactive Table grid insertion/editing, Image upload, Hyperlinks, Find & Replace dialog, Track Changes toggle, and Status Bar word count."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 9.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the `Insert`, `Review`, and `View` ribbons in Writer:
   - **Insert Tab:** Insert Table (interactive `RxC` picker + ability to type inside cells, add/delete rows & columns), Insert Picture (from local file/data URL with resize handles), Insert Shape, Insert Hyperlink, Insert Comment, and Insert Page Number.
   - **Review Tab:** Live Word/Character/Page Count modal, Track Changes toggle (highlights insertions in underlined blue and deletions in strikethrough red), Accept/Reject Change buttons, and New/Delete Comment sidebar.
   - **View Tab & Status Bar:** Print Layout vs Web Layout vs Mobile Reflow mode toggle, Ruler visibility checkbox, Navigation Pane checkbox, Find & Replace bar (`Ctrl+F` / `Ctrl+H`), and bottom Status Bar showing live `Page X of Y`, `N words`, and Zoom slider (`50%` to `200%`).
2. Verify via automated DOM tests and commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 10: STANDALONE EXCEL (CALC) - LIVE GRID EDITING, SHEETS & FORMULAS
    # ==========================================================================
    @{
        id = "10.1.1"; stage = 10; phase = "10.1"; title = "Editable Spreadsheet Grid, In-Cell Input, Multi-Sheet Tabs & Column Resizing"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Full cell editing (double-click, F2, direct typing), Formula Bar sync, Sheet1/Sheet2/+ tab bar, and column/row drag-resizing."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 10.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the uneditable Excel grid seen in `image_cb4d38.png`:
   - **Direct Cell Editing:** Typing any printable character on a selected cell, pressing `F2`, or double-clicking a cell MUST open an active in-cell input editor synchronized in real time with the top Formula Bar (`fx`).
   - **Commit & Navigation:** Pressing `Enter` commits the value/formula to `excel-engine`, triggers DAG recalculation, displays the computed result in the cell, and moves selection down (`Shift+Enter` up, `Tab` right, `Shift+Tab` left, `Escape` cancels edit).
   - **Multi-Sheet Management:** Populate the left `Workbook Sheets` panel AND a bottom sheet tab bar with `Sheet1`, a `+ New Sheet` button, double-click to rename sheet, and delete sheet button.
   - **Row/Column Manipulation:** Drag column/row header dividers to resize width/height; right-click row/column headers to Insert Row/Column, Delete Row/Column, or Clear Contents.
2. Write automated DOM tests verifying cell typing, `=SUM(A1:A3)` evaluation, and creating `Sheet2`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "10.1.2"; stage = 10; phase = "10.1"; title = "Range Selection, Fill Handle Auto-Series & Clipboard Cut/Copy/Paste"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Click-drag rectangular range selection (A1:D10), Shift+Arrow selection, bottom-right Fill Handle auto-fill, and TSV/CSV clipboard paste."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 10.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement full range mechanics in Calc:
   - **Range Selection:** Mouse click-and-drag or `Shift + Arrow` selects a rectangular cell range (`A1:C5`), updating the Name Box (`A1`) and displaying live `Average`, `Count`, and `Sum` in the bottom Status Bar.
   - **Fill Handle Drag:** Dragging the small green square in the bottom-right corner of the selection box auto-fills linear numeric series (`1, 2 -> 3, 4, 5`), months/days, and shifts relative formula references (`=A1+B1` -> `=A2+B2`).
   - **Multi-Cell Clipboard:** `Ctrl+C`, `Ctrl+X`, and `Ctrl+V` copy and paste multi-cell blocks cleanly (compatible with system TSV/Excel clipboard).
2. Verify with automated DOM tests and commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "10.2.1"; stage = 10; phase = "10.2"; title = "Calc Home, Formulas & Data Ribbons: Number Formats, Function Wizard & Sort/Filter"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Working Number Formatting ($, %, Comma, Decimals), Cell Borders/Fill, AutoSum, Function Library inserter, Sort A-Z/Z-A, Filter, and Flash Fill."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 10.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every button across the `Home`, `Formulas`, and `Data` ribbons in `CalcRibbon.tsx`:
   - **Home Tab:** Font/Size/Bold/Italic, Cell Background Color, Text Color, Borders (`All Borders`, `Thick Box Border`, `Bottom Border`), Alignment, Wrap Text, Merge & Center, and **Number Format** (`General`, `Currency $`, `Accounting`, `Percentage %`, `Comma`, `Date`, `Increase/Decrease Decimal`).
   - **Formulas Tab:** Replace the dead placeholder text with working buttons: **Insert Function (`fx` modal with search across all 450+ functions)**, **AutoSum (`Σ` Sum, Average, Count, Max, Min)**, Financial dropdown, Logical dropdown, Text dropdown, Lookup & Reference dropdown, Name Manager, and Show Formulas toggle.
   - **Data Tab:** Make `Sort A-Z`, `Sort Z-A`, `Filter` (adds dropdown arrows to header cells to filter rows by value), `Flash Fill`, `Remove Duplicates`, and `Text to Columns` 100% functional on the selected grid range!
2. Write automated tests verifying `Sort A-Z`, `AutoSum`, and `Currency` formatting. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "10.2.2"; stage = 10; phase = "10.2"; title = "Calc Insert & View Ribbons: Live Charts (Bar, Line, Pie), Conditional Formatting & Freeze Panes"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Interactive floating SVG/Canvas Charts bound to cell ranges, Conditional Formatting rules, and Freeze Panes/Gridlines toggles."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 10.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the `Insert`, `Page Layout`, and `View` ribbons in Calc:
   - **Insert Charts:** Selecting a data range (e.g., `A1:B4`) and clicking `Insert -> Column Chart`, `Line Chart`, or `Pie Chart` renders an interactive, draggable, resizable chart overlay on the sheet that automatically updates whenever cell values change!
   - **Conditional Formatting:** Highlight Cells Rules (`Greater Than`, `Less Than`, `Duplicate Values`) and Data Bars / Color Scales applied dynamically to selected ranges.
   - **View Tab:** Toggle Gridlines, Toggle Formula Bar, Toggle Row/Column Headings, and **Freeze Panes** (`Freeze Top Row`, `Freeze First Column`, `Freeze Panes` at selection).
2. Verify chart generation and conditional formatting via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 11: STANDALONE POWERPOINT (SLIDES) - VECTOR STAGE, MANIPULATION & SHOW
    # ==========================================================================
    @{
        id = "11.1.1"; stage = 11; phase = "11.1"; title = "Interactive Slide Canvas: Drag, 8-Point Resize, Rotate & In-Place Text Editing"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Full interactive manipulation of slide text boxes, shapes, and images with selection handles, snapping guides, and live text editing."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 11.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the static, non-interactive slide canvas seen in `image_cb4c9a.png`:
   - **Selection & Transform Gizmos:** Clicking any text box or shape (like the title or star) renders a selection bounding box with **8 resize handles** (corners and midpoints) and a **top rotation handle**.
   - **Live Drag, Resize & Rotate:** Dragging a shape moves its `(x, y)` coordinates with smart center/edge alignment guides; dragging any handle resizes `(w, h)` or rotates `angle_deg` in real time inside `ppt-engine`.
   - **In-Place Text Editing:** Double-clicking any shape or text box activates an inline rich-text editor inside the shape so users can type, edit, and format slide text directly on the canvas!
   - **Slide Management:** Make `+ Slide`, Duplicate Slide, Delete Slide, Move Slide Up/Down, and Slide Layout picker (`Title Slide`, `Title & Content`, `Two Content`, `Blank`) work seamlessly.
2. Verify shape dragging, resizing, and inline text editing with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "11.1.2"; stage = 11; phase = "11.1"; title = "Slides Home, Insert & Design Ribbons: Shapes, Images, Tables, Themes & Backgrounds"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Working Shape/Text Box/Image/Table insertion, Z-Order (Bring to Front/Send to Back), Shape Fill/Stroke, and Design Theme gallery."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 11.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire all controls in `SlidesRibbon.tsx` (`Home`, `Insert`, and `Design` tabs):
   - **Home & Insert Tabs:** New Slide (with layout dropdown), Insert Text Box, Insert Image, Insert Table, Insert Shape (`Rectangle`, `Rounded Rect`, `Ellipse`, `Triangle`, `Right Arrow`, `5-Point Star`, `Callout`, `Diamond`, `Heart`), Arrange (`Bring to Front`, `Send to Back`, `Align Left/Center/Middle`, `Duplicate`, `Delete`), Shape Fill color picker, Shape Outline color/weight picker, and Speaker Notes pane at the bottom of the stage.
   - **Design Tab:** 8 Built-in Presentation Themes (updating Master background, title font, and accent palette across all slides), Slide Size (`16:9 Widescreen` vs `4:3 Standard`), and Format Background (Solid Color / Gradient).
2. Verify with automated DOM tests and commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "11.2.1"; stage = 11; phase = "11.2"; title = "Slides Transitions, Animations & Fullscreen Slide Show (F5) Engine"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Working Transitions tab (Fade, Push, Wipe, Morph), Animations tab (Entrance/Emphasis/Exit + Animation Pane), and Fullscreen Presenter Show."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 11.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the dead `"SlideShow commands for PowerPoint"` placeholder seen in `image_e63126.png` with complete `Transitions`, `Animations`, and `Slide Show` ribbons:
   - **Transitions Tab:** Transition gallery (`None`, `Fade`, `Push`, `Wipe`, `Split`, `Morph`), Effect Options, Duration spinner, `Apply to All Slides` button, and `Preview` button that plays the transition right on the stage.
   - **Animations Tab:** Entrance effects (`Appear`, `Fade In`, `Fly In`, `Zoom`, `Bounce`), Emphasis (`Pulse`, `Spin`), Exit (`Fade Out`, `Fly Out`), Trigger (`On Click`, `With Previous`, `After Previous`), and collapsible **Animation Timeline Pane** on the right.
   - **Slide Show Tab:** `From Beginning (F5)`, `From Current Slide (Shift+F5)`, `Presenter View` (with timer & speaker notes), and fullscreen interactive presentation player where clicking or pressing `Right Arrow` / `Space` steps through shape animations and slide transitions!
2. Verify fullscreen player and animation triggers via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 12: STANDALONE PDF STUDIO - READ, IN-PLACE EDIT, FORMS, SIGN & WRITE
    # ==========================================================================
    @{
        id = "12.1.1"; stage = 12; phase = "12.1"; title = "Standalone PDF Studio Workspace, Multi-Page Renderer & Page Organizer"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "Dedicated PDF Studio UI with multi-page canvas, Thumbnails/Bookmarks sidebar, Page Add/Delete/Rotate/Reorder/Merge, and Create Blank PDF."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 12.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Build the dedicated **PDF Studio** workspace (`apps/unified-shell/src/apps/pdf/PdfWorkspace.tsx`) accessible via `aether-pdf` and `aether-hub`:
   - Support creating a new multi-page PDF from scratch OR opening any existing `.pdf` file via `pdf-engine`.
   - Left Sidebar with 3 tabs: **Page Thumbnails** (click to jump, drag to reorder), **Bookmarks / Outline Tree**, and **Annotations List**.
   - **Page Tools Ribbon:** `+ Insert Blank Page`, `Delete Page`, `Rotate Clockwise 90°`, `Rotate Counter-Clockwise 90°`, `Move Page Up/Down`, `Extract Page`, and `Merge PDF`.
   - Continuous scrolling, Single-Page fit, Two-Page spread, and Zoom controls (`25%` to `400%`).
2. Write automated DOM tests verifying PDF page creation, rotation, and deletion. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "12.1.2"; stage = 12; phase = "12.1"; title = "PDF In-Place Text/Image Editor, Annotations, Redaction & AcroForms Filler"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "Direct in-place editing of PDF text blocks, adding new text/images, Highlight/Sticky Note/Freehand Ink annotations, Form Builder/Filler, and Signature pad."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 12.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Wire all editing, form, and signing tools in `PdfRibbon.tsx` directly to `pdf-engine`:
   - **Edit PDF Tab:** `Edit Existing Text` mode (clicking any PDF text block turns it into an editable bounding box that updates the underlying PDF content stream in `pdf-engine`), `+ Add Text Box`, `+ Add Image`, and `Add Watermark`.
   - **Comment & Annotate Tab:** `Highlight Text`, `Underline`, `Strikethrough`, `Sticky Note` (with pop-up comment popover), `Freehand Ink Pen` (draw directly on the PDF canvas with color/thickness picker), `Eraser`, and `Redact Area` (permanently blacks out and removes underlying text).
   - **Forms & Sign Tab:** Interactive filling of `/AcroForm` fields (Text input, Checkbox, Radio, Dropdown), **Add Form Field** tool (place new interactive text/checkbox fields onto any PDF page), and **Sign Document Modal** (draw signature with mouse/stylus or type signature and stamp onto the page, followed by Incremental Save via `pdf-engine`).
2. Write automated DOM tests verifying PDF text editing, form filling, signature placement, and binary PDF export. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 13: FULL SUITE E2E AUDIT, MULTI-WINDOW SYNC & 5-BINARY PACKAGING
    # ==========================================================================
    @{
        id = "13.1.1"; stage = 13; phase = "13.1"; title = "Comprehensive Automated E2E UI Test Suite Across All 5 Apps"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Automated E2E verification suite testing every single Ribbon tab, button, keyboard shortcut, and canvas edit across Hub, Writer, Calc, Slides, and PDF."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 13.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.4.
1. Create a comprehensive automated UI verification suite in `apps/unified-shell/src/__tests__/full_suite_e2e.test.tsx` that programmatically verifies:
   - **Hub:** Launching Writer, Calc, Slides, and PDF independently.
   - **Writer:** Typing paragraphs, inserting a new page (`Blank Page`), applying Bold/Heading 1/Table, and saving `.docx`.
   - **Calc:** Editing cell `A1` (`100`) and `A2` (`200`), entering `=SUM(A1:A2)` in `A3`, asserting `300` renders, adding `Sheet2`, sorting, and inserting a Chart.
   - **Slides:** Adding a slide, inserting a Star shape, dragging/resizing the shape, editing text inline, applying a Transition, and starting Fullscreen Slide Show.
   - **PDF:** Creating/opening a PDF, editing text in-place, adding a Page, filling a Form field, placing a Signature, and saving `.pdf`.
2. Run `tools/verify-contract.ps1` and ensure 100% pass rate with zero skipped tests. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "13.1.2"; stage = 13; phase = "13.1"; title = "Final 5-Binary Release Build & GitHub Actions Multi-Platform Verification"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Verified release builds of aether-hub, aether-writer, aether-calc, aether-slides, and aether-pdf pushed and packaged across CI/CD."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 13.1.2
Execute final GEC-v2.0 Release Audit:
1. Update `.github/workflows/build-all-platforms.yml` so desktop builds package and upload all 5 executables (`aether-hub`, `aether-writer`, `aether-calc`, `aether-slides`, `aether-pdf`) alongside the unified installer bundle.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1` to verify all Rust crates, Clippy, Formatting, Anti-Placeholder scans, and Frontend E2E UI tests pass 100%.
3. Commit, tag `v2.0.0`, and push to origin.
"@
    }
)

# 1. Write Subphase Markdown Prompt Files (.agents/stages/stage-8..13/subphase-X.Y.Z.md)
foreach ($sp in $remediationSubphases) {
    $stageFolder = ".agents/stages/stage-$($sp.stage)"
    $filePath = "$stageFolder/subphase-$($sp.id).md"
    $docText = @"
# Subphase $($sp.id): $($sp.title)
- **Stage:** $($sp.stage) | **Phase:** $($sp.phase)
- **Target Subsystem:** ``$($sp.crate)``
- **Primary Agent Skill:** ``$($sp.skill)``
- **Deliverable:** $($sp.deliverable)
- **Validation Command:** ``$($sp.validation)``

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
$($sp.prompt)
```
"@
    Set-Content -Path $filePath -Value $docText -Encoding UTF8
Write-Host "  Created prompt spec: $filePath" -ForegroundColor Gray
}

# 2. Write Slash Workflows (.agents/workflows/stage-8.md .. stage-13.md)
$stageTitles = @{
8  = "Stage 8: De-Clubbing Architecture, 5 Standalone Binaries, File I/O & Contextual Ribbons"
9  = "Stage 9: Standalone Writer (Word) - Real Typing, Multi-Page Creation & Full Ribbon"
10 = "Stage 10: Standalone Calc (Excel) - Live Cell Editing, Multi-Sheets, Formulas & Charts"
11 = "Stage 11: Standalone Slides (PowerPoint) - Vector Manipulation, Animations & Fullscreen Show"
12 = "Stage 12: Standalone PDF Studio - Multi-Page PDF Read, In-Place Edit, Forms & Signing"
13 = "Stage 13: End-to-End Automated UI Verification & 5-Binary Release Packaging"
}

foreach ($s in 8..13) {
$stageItems = $remediationSubphases | Where-Object { $_.stage -eq $s }
$wfText = @"
description: Automated execution workflow for $($stageTitles[$s])
Workflow: Execute Stage $s (/stage-$s)
Objective: $($stageTitles[$s])

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

"@
foreach ($sp in $stageItems) {
$wfText += @"

Subphase $($sp.id): $($sp.title)
Spec File: .agents/stages/stage-$s/subphase-$($sp.id).md
$($sp.prompt)

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete $($sp.id)
"@
}
Set-Content -Path ".agents/workflows/stage-$s.md" -Value $wfText -Encoding UTF8
Write-Host "  Created workflow: .agents/workflows/stage-$s.md" -ForegroundColor Gray
}

# 3. Append Stages 8-13 Non-Destructively to specs/stage-tracker.json
$trackerPath = "specs/stage-tracker.json"
if (Test-Path $trackerPath) {
$tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json
$existingIds = @($tracker.subphases | ForEach-Object { $_.id })

$added = 0
foreach ($sp in $remediationSubphases) {
    if ($existingIds -notcontains $sp.id) {
        $newEntry = [ordered]@{
            id           = $sp.id
            stage        = $sp.stage
            phase        = $sp.phase
            title        = $sp.title
            crate        = $sp.crate
            rea_required = $sp.rea
            status       = "PENDING"
            prompt_file  = ".agents/stages/stage-$($sp.stage)/subphase-$($sp.id).md"
            completed_at = $null
        }
        $tracker.subphases += [pscustomobject]$newEntry
        $added++
    }
}
$tracker.contract = "GEC-v2.0"
$tracker.total_stages = 13
$tracker.total_subphases = $tracker.subphases.Count
$tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8
Write-Host "  Appended $added remediation subphases to specs/stage-tracker.json (Total: $($tracker.total_subphases) subphases across 13 stages)." -ForegroundColor Green
}

Write-Host "=== [Remediation Complete] Stages 8 to 13 Ready for Execution! ===" -ForegroundColor Green