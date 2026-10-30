# ==============================================================================
# bootstrap_stages_v4.ps1 - Generates Stages 28 to 34 (14 Subphases for GEC-v4.0)
# Preserves existing Stages 1-27 without overwriting!
# Fixes: Separate App Windows + Session Hub, Full-App Dark/Light Theme,
# Ghost Sidebar Bug, Word Typing/Pages/Ribbon, Excel Ribbon, PPT Ribbon,
# Complete PDF Editor (Open/Edit/Encrypt/Resize/Pages), TXT I/O, and Auto-Testing.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v4.0 2/2] Generating Stages 28-34 Prompts & Workflows ===" -ForegroundColor Cyan

foreach ($s in 28..34) {
    New-Item -ItemType Directory -Force -Path ".agents/stages/stage-$s" | Out-Null
}

$v4Subphases = @(
    # ==========================================================================
    # STAGE 28: MULTI-WINDOW PER FORMAT, FULL-APP THEME & GHOST SIDEBAR FIX
    # ==========================================================================
    @{
        id = "28.1.1"; stage = 28; phase = "28.1"; title = "Separate App Windows Per Document Type + Shared Session Manager"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Dedicated windows per document format (Word Window, Excel Window, PPT Window, PDF Window, Notes Window) sharing 1 low-RAM process + Global Active Documents switcher."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 28.1.1
Invoke @adaptive-ui-builder under GEC-v4.0 Article I.
1. **Separate Window Per Document Type (Without RAM Hang):**
   - Implement Tauri multi-window routing (`spawn_app_window(app_type: "word" | "excel" | "ppt" | "pdf" | "notes")` in `src-tauri/src/lib.rs` using `tauri::WebviewWindowBuilder::new` with `?app=<app_type>` URL query parameter, plus an in-app multi-window workspace manager fallback when running in browser/test mode).
   - Each window filters its tab bar strictly to its own document format (e.g., the **Aether Word** window shows only `.docx` tabs; **Aether Excel** shows only `.xlsx` tabs; **Aether PowerPoint** shows only `.pptx` tabs; **Aether PDF** shows only `.pdf` tabs; **Aether Notes** shows only `.txt/.md` tabs).
   - Clicking an app icon (`W`, `X`, `P`, `PDF`, `N`) on the Left App Rail or Home Hub either focuses/spawns that app's dedicated window OR switches cleanly to that isolated app workspace, while a **Global Open Documents Drawer (`All Open Files`)** lists every active document across all 5 apps with live auto-save to `localStorage`/`IndexedDB` so zero work is ever lost!
   - Also add an `"Open in New Window ↗"` button on every tab so any document can be popped out into a separate OS window on demand.
2. Write automated Vitest tests verifying isolated per-app tab filtering, session persistence, and new window spawning. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "28.1.2"; stage = 28; phase = "28.1"; title = "Fix Ghost Word Sidebar Bug, Hide Alt-Badges & Fix Global Dark/Light Theme"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Eliminates Word Navigation sidebar bleeding into PDF/Notes, fixes hardcoded '22 words' status bar, hides [F]/[H] badges unless Alt is held, and applies Dark/Light theme to 100% of the app."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 28.1.2
Invoke @adaptive-ui-builder under GEC-v4.0 Article II.
1. **Fix Ghost Navigation Sidebar Bug (Seen in image_2586db.png & image_258017.png):**
   - Currently, Word's white `Navigation` sidebar is rendered globally outside the Word view, causing it to appear next to PDF's `Pages` sidebar and Notes' `SCRATCHPADS` sidebar!
   - Move Word's `Navigation` sidebar strictly inside the Word editor component so it NEVER renders when viewing Excel, PowerPoint, PDF, or Notes.
2. **Fix Permanent `[F] [H] [N] [P]` Badges on Ribbon Tabs:**
   - Hide the black Alt-keytip badges (`F`, `H`, `N`, `P`, `S`, `R`, `W`) on Ribbon tabs by default; only show them when `altKeyTipsActive === true` (toggled by pressing and releasing `Alt`).
3. **Full-App Dark / Light Theme Cascade:**
   - Fix the theme toggle so switching between `☀️ Light` and `🌙 Dark` updates the entire application root, Titlebar, Tab Bar, Ribbon, Left Sidebar, Workspace background, Modals, and Status Bar (while keeping an optional `Dark Page / Paper Canvas` toggle in View for Word/PDF).
4. **Fix Hardcoded `22 words` Status Bar:**
   - Replace the static `22 words | English (US)` string at the bottom of the screen with dynamic per-app status indicators (Word: real live word/char/page count; Excel: active sheet count + selected cell `SUM/AVG/COUNT`; PPT: slide index + shape count; PDF: page index + page size in mm/in + encryption status; Notes: line/col/word/char count).
5. Write automated DOM tests verifying zero ghost sidebars in PDF/Notes, full-app theme class propagation, and live dynamic status bar counts. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "28.2.1"; stage = 28; phase = "28.2"; title = "Universal File Menu Backstage: Real Open, Save, Save As, Export & Print for All 5 Formats"
        crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false
        deliverable = "100% working File -> New, Open From Computer, Save, Save As, Export to PDF/HTML/CSV/TXT, and Print across Word, Excel, PPT, PDF, and Notes."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 28.2.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v4.0 Article III.
1. Build a unified, rock-solid File I/O service (`apps/unified-shell/src/services/fileIoService.ts`) connected to both the `File` Backstage menu AND top quick-action toolbar buttons (`📂 Open`, `💾 Save`, `🖨️ Print`):
   - **Open Existing File (`Ctrl+O`):** Triggers a hidden `<input type="file">` (plus Tauri file dialog when in desktop runtime) supporting `.docx`, `.doc`, `.html`, `.xlsx`, `.csv`, `.tsv`, `.pptx`, `.pdf`, `.txt`, `.md`, and `.json`. Parses the file immediately and populates the active editor canvas!
   - **Save / Save As (`Ctrl+S` / `Ctrl+Shift+S`):** Serializes the current document state (Word HTML/OOXML package, Excel workbook/CSV, PPTX slide deck JSON/OOXML, PDF binary with edits, or TXT/MD string), writes it via Tauri FS or triggers an immediate browser file download with the exact filename, and clears the unsaved `*` indicator on the tab.
   - **Print (`Ctrl+P`):** Opens a dedicated clean print view / iframe containing ONLY the formatted document pages/sheet/slides/PDF/note and invokes `window.print()`.
2. Write automated tests verifying File Open, Save, and Print across all 5 document types. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 29: WORD (.DOCX) COMPLETE FIX — TYPING, PAGES, SHAPES & ALL 7 RIBBONS
    # ==========================================================================
    @{
        id = "29.1.1"; stage = 29; phase = "29.1"; title = "Fix Word Typing, Cursor Focus, Multi-Page Creation & Draggable/Resizable Shapes"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Smooth uninterrupted typing in Word pages, + New Page / Delete Page, and properly sized draggable floating shapes/images/tables."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. **Fix Word Typing & Cursor Loss Bug:**
   - Fix the React/DOM controlled-input bug that prevented typing inside Word pages! Each page's `.word-page-body` must use an uncontrolled `contentEditable="true"` ref (synchronized via `onInput` without overwriting `innerHTML` while the element is focused) AND support programmatic insertion at the saved `Selection` range so users can type continuously at 120fps, press `Enter`, `Backspace`, `Delete`, `Tab`, and select text freely.
2. **Fix Giant Blue Shapes Bug (Seen in image_25f41c.png):**
   - In `image_25f41c.png`, inserting shapes created massive full-width blue rectangles/ovals stacked across the entire page!
   - Fix `Insert -> Shape` and `Insert -> Picture` so shapes insert at a sensible default size (`140px × 90px`), with an interactive selection border, **drag-to-move**, **8-corner resize handles**, **color picker**, **text inside shape**, and **Delete (`Del` key or trash icon)**.
3. **Working Multi-Page Creation (`+ New Page`, `Page Break`, Auto-Overflow):**
   - Add a floating **`+ Add Page`** button between/below pages AND wire `Insert -> Blank Page` and `Insert -> Page Break` (`Ctrl+Enter`) to immediately create Page 2, Page 3, etc., and scroll/focus the new page.
   - Allow deleting any extra page via a `Delete Page` button on the page header/ribbon.
4. Write automated DOM tests simulating continuous typing, creating 3 pages, inserting/resizing a shape, and deleting it. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "29.1.2"; stage = 29; phase = "29.1"; title = "Fix Word Home, Insert & Layout Ribbons (100% Working Controls)"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Every single button in Word's Home, Insert, and Layout ribbons wired and verified via automated tests."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 29.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's `Home`, `Insert`, and `Layout` ribbons (preserving editor selection before executing commands using `onMouseDown={(e) => e.preventDefault()}` on toolbar buttons so clicking a button NEVER steals focus from the selected text!):
   - **Home Tab:** Clipboard (`Cut`, `Copy`, `Paste`, `Format Painter`), Font Family (`Calibri`, `Arial`, `Times New Roman`, `Georgia`, `Verdana`, `Consolas`, `Inter` — wraps selection or sets active typing style), Font Size (`8pt` to `72pt`), Grow/Shrink Font (`A+`, `A-`), Change Case (`Aa`), Clear Formatting, Bold (`B`), Italic (`I`), Underline (`U`), Strikethrough (`S`), Subscript (`X₂`), Superscript (`X²`), Text Color palette, Highlight Color palette, Bullet List, Numbered List, Multilevel List, Decrease/Increase Indent, Sort Paragraphs, Show/Hide `¶` Formatting Marks, Align Left/Center/Right/Justify, Line & Paragraph Spacing (`1.0`, `1.15`, `1.5`, `2.0`), Shading fill, Borders, and Styles (`Normal`, `No Spacing`, `Heading 1`, `Heading 2`, `Heading 3`, `Title`, `Subtitle`, `Quote`).
   - **Insert Tab:** `Cover Page` (inserts a styled title cover page), `Blank Page`, `Page Break`, `Table` (interactive grid size selector + add/remove row/column/border toolbar), `Picture` (file upload), `Shapes` (`Rectangle`, `Rounded Rect`, `Circle/Oval`, `Triangle`, `Arrow`, `Star`, `Callout`), `Chart` (inserts editable inline bar/line/pie chart), `Hyperlink`, `Bookmark`, `Header`, `Footer`, `Page Number`, `Text Box`, `WordArt`, `Drop Cap`, `Date & Time`, `Symbol (Ω, €, £, ©, ™, ±, ≠, ≤, ≥)`, and `Horizontal Line`.
   - **Layout Tab:** `Margins` (`Normal 1"`, `Narrow 0.5"`, `Moderate`, `Wide` — visibly updates page padding!), `Orientation` (`Portrait` vs `Landscape` — visibly swaps page width/height `816px × 1056px` vs `1056px × 816px`!), `Size` (`Letter`, `A4`, `Legal`), `Columns` (`One`, `Two`, `Three` — applies CSS multi-column layout to the page body!), `Breaks`, `Line Numbers` toggle, `Hyphenation`, `Page Color` picker, `Page Borders` selector, and `Watermark` (`CONFIDENTIAL`, `DRAFT`, custom).
2. Write automated tests clicking and asserting every single control in `Home`, `Insert`, and `Layout`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "29.2.1"; stage = 29; phase = "29.2"; title = "Fix Word References, Review & View Ribbons (100% Working Controls)"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Every single button in Word's References, Review, and View ribbons wired and verified via automated tests."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 29.2.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's `References`, `Review`, and `View` ribbons:
   - **References Tab:** `Table of Contents` (automatically scans all `Heading 1/2/3` elements in the document and inserts a clickable Table of Contents block at the top of the document + `Update Table`), `Insert Footnote` (inserts a superscript `[1]` reference at the cursor and appends a numbered footnote entry in the page footer area), `Insert Endnote`, `Insert Citation` & `Insert Bibliography` (APA/MLA/IEEE citation manager modal), and `Insert Caption` (for figures/tables).
   - **Review Tab:** `Spelling & Grammar` check drawer, `Thesaurus` lookup, `Word Count` modal (showing Pages, Words, Characters no spaces, Characters with spaces, Paragraphs, Lines), `Read Aloud` (uses browser `window.speechSynthesis` TTS to read the document aloud with Play/Pause/Stop!), `Translate Selection`, `New Comment` (attaches a comment card in a right-hand Comments gutter), `Delete/Next/Prev Comment`, `Track Changes` toggle (records subsequent typing in `<ins>` and deletions in `<del>`), `Accept All Changes`, `Reject All Changes`, and `Protect Document (Read-Only Lock)`.
   - **View Tab:** `Print Layout` (standard paged sheets), `Web Layout` (full-width fluid canvas), `Read Mode` (distraction-free two-column book reader), `Focus Mode` (hides ribbon), `Ruler` checkbox, `Gridlines` checkbox (renders subtle alignment grid overlay on the page), `Navigation Pane` checkbox, `Dark Page Canvas` toggle, `Zoom` (`50%`, `75%`, `100%`, `150%`, `200%`), `One Page`, `Multi Pages` (side-by-side pages), and `Page Width`.
2. Verify every button in `References`, `Review`, and `View` with automated Vitest DOM assertions. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 30: EXCEL (.XLSX) COMPLETE FIX — ALL 7 RIBBONS, FORMULAS & TOOLS
    # ==========================================================================
    @{
        id = "30.1.1"; stage = 30; phase = "30.1"; title = "Excel Home, Insert & Page Layout Ribbons (100% Working Controls)"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Complete working Excel Home, Insert (Tables, Pivot Summary, Charts, Sparklines), and Page Layout ribbons."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 30.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Audit and wire **100% of the buttons** in Excel's `Home`, `Insert`, and `Page Layout` ribbons:
   - **Home Tab:** Cut/Copy/Paste, Font Family & Size, Bold/Italic/Underline, Borders (`All`, `Outside`, `Thick Bottom`, `No Border`), Fill Color, Font Color, Align Top/Middle/Bottom & Left/Center/Right, `Wrap Text`, `Merge & Center`, Number Formats (`General`, `Number`, `Currency $`, `Accounting`, `Short Date`, `Percentage %`, `Fraction`, `Comma`, `Increase/Decrease Decimal`), `Conditional Formatting` (`Highlight >`, `Highlight <`, `Top 10%`, `Data Bars`, `Color Scales`), `Format as Table` (applies alternating banded row styling to selected range), `Insert/Delete/Format Rows & Columns` (including AutoFit Row Height & Column Width), `AutoSum`, `Fill Down/Right/Series`, `Clear (All / Formats / Contents)`, and `Find & Select`.
   - **Insert Tab:** `PivotTable Summary` (generates an automatic grouped summary table from selected data onto a new or current sheet), `Table`, `Picture`, `Shapes`, `Column Chart`, `Line Chart`, `Pie/Doughnut Chart`, `Scatter Plot` (draggable, resizable, live-updating SVG charts on the sheet canvas), `Sparklines` (renders mini inline SVG line/bar sparklines inside cells!), and `Hyperlink / Comment`.
   - **Page Layout Tab:** Fix the non-working buttons: `Themes` (applies color/font palette to grid & charts), `Colors`, `Fonts`, `Margins` (`Normal`, `Narrow`, `Wide`), `Orientation` (`Portrait`, `Landscape` — updates dashed print-boundary lines on grid), `Print Area` (`Set Print Area`, `Clear Print Area`), `Scale to Fit`, `View Grid` (toggles cell gridlines on/off!), `Print Grid`, `View Headings` (toggles `A, B, C` and `1, 2, 3` headers on/off!), and `Print Headings`.
2. Write automated tests verifying every control in `Home`, `Insert`, and `Page Layout`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "30.1.2"; stage = 30; phase = "30.1"; title = "Excel Formulas, Data, Review & View Ribbons (100% Working Controls)"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Complete working Excel Formulas library, Data tools (Sort, Filter, Validation, Goal Seek), Review comments/protection, and View freeze panes."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 30.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in Excel's `Formulas`, `Data`, `Review`, and `View` ribbons:
   - **Formulas Tab:** `Insert Function (fx)` searchable wizard, `AutoSum (Sum, Average, Count, Max, Min)`, Category Dropdowns that insert & evaluate functions (`Financial: PMT,FV,PV,NPV,IRR`, `Logical: IF,AND,OR,NOT,IFERROR,IFS`, `Text: CONCAT,LEFT,RIGHT,MID,LEN,UPPER,LOWER,TRIM,SUBSTITUTE`, `Date & Time: TODAY,NOW,YEAR,MONTH,DAY,DATEDIF`, `Lookup & Ref: VLOOKUP,XLOOKUP,INDEX,MATCH`, `Math & Stat: ROUND,SUMIF,SUMIFS,COUNTIF,AVERAGEIF,STDEV,MEDIAN,RAND,RANDBETWEEN`), `Name Manager` (define named ranges like `TaxRate = 0.18` usable in formulas!), `Show Formulas` toggle, `Trace Precedents / Dependents` (highlights referenced cells), and `Calculate Now (F9)`.
   - **Data Tab:** `From CSV / JSON Import`, `Sort A → Z`, `Sort Z → A`, `Custom Multi-Column Sort`, `Filter` (interactive dropdown filter on column headers), `Clear Filter`, `Text to Columns`, `Flash Fill`, `Remove Duplicates`, `Data Validation` (restrict cells to Dropdown List, Whole Number range, or Date!), `Subtotals`, and `What-If Goal Seek` solver!
   - **Review & View Tabs:** `New Comment / Note` (adds red corner indicator and hover tooltip on cell), `Protect Sheet` (locks cells from editing with optional password), `Workbook Statistics`, `Normal / Page Break Preview`, `Formula Bar` checkbox, `Gridlines` checkbox, `Headings` checkbox, `Freeze Panes` (`Freeze Top Row`, `Freeze First Column`, `Unfreeze`), and Zoom controls.
2. Verify all `Formulas`, `Data`, `Review`, and `View` actions with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 31: POWERPOINT (.PPTX) COMPLETE FIX — ALL 8 RIBBONS & SLIDE SHOW
    # ==========================================================================
    @{
        id = "31.1.1"; stage = 31; phase = "31.1"; title = "PowerPoint Home, Insert, Design & Transitions Ribbons (100% Working)"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Complete working PowerPoint Home, Insert (SmartArt, Charts, Media, Shapes, Icons), Design Themes, and Transitions."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 31.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in PowerPoint's `Home`, `Insert`, `Design`, and `Transitions` ribbons:
   - **Home Tab:** `+ New Slide` (with 6 Layout templates: `Title Slide`, `Title & Content`, `Section Header`, `Two Content`, `Title Only`, `Blank`), `Duplicate Slide`, `Delete Slide`, `Reset Slide`, Font/Size/Bold/Italic/Underline/Shadow/Color, Bullet/Numbered Lists inside text boxes, Align Left/Center/Right & Top/Middle/Bottom, Quick Shapes gallery, `Arrange` (`Bring to Front`, `Send to Back`, `Align Center`, `Align Middle`, `Distribute Horizontally/Vertically`, `Group/Ungroup`), `Shape Fill`, `Shape Outline`, `Shape Effects (Drop Shadow, Glow)`, and `Find & Replace`.
   - **Insert Tab:** `New Slide`, `Table` (inserts editable slide table), `Picture` (upload image onto slide), `Shapes` (16 vector shapes including arrows, stars, banners, callouts, flowcharts), `Icons / Stickers`, `SmartArt Diagram` (inserts editable Process, Cycle, Hierarchy, or Pyramid diagram!), `Chart` (inserts editable Bar, Line, or Pie chart onto the slide), `Text Box`, `Header & Footer`, `WordArt`, `Slide Number`, and `Audio / Video` embed.
   - **Design Tab:** 10 visual Presentation Themes (instantly updates slide background, typography, and accent colors across the deck), `Variants` color palettes, `Slide Size` (`Standard 4:3` vs `Widescreen 16:9` — visibly changes stage aspect ratio!), and `Format Background` (Solid Color, Linear Gradient, Radial Gradient, Pattern, `Apply to All`).
   - **Transitions Tab:** `None`, `Fade`, `Push`, `Wipe`, `Split`, `Reveal`, `Random Bars`, `Shape Circle`, `Zoom`, `Morph` (interpolates matching shapes across slides!), `Effect Options` (direction), `Transition Sound`, `Duration (s)`, `Apply to All`, `Advance Slide After X sec`, and `Preview Transition` (plays live CSS/WAAPI transition animation on the canvas!).
2. Verify every button in `Home`, `Insert`, `Design`, and `Transitions` via automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "31.1.2"; stage = 31; phase = "31.1"; title = "PowerPoint Animations, Slide Show, Review & View Ribbons (100% Working)"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Working Animations timeline, Slide Show (F5, Presenter View, Rehearse Timings, Laser/Pen), Review comments, and Slide Sorter/Notes Master views."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in PowerPoint's `Animations`, `Slide Show`, `Review`, and `View` ribbons:
   - **Animations Tab:** Entrance (`Appear`, `Fade In`, `Fly In`, `Float In`, `Split`, `Wipe`, `Shape`, `Wheel`, `Bounce`, `Zoom`), Emphasis (`Pulse`, `Teeter`, `Spin`, `Grow/Shrink`), Exit (`Disappear`, `Fade Out`, `Fly Out`, `Zoom Out`), `Preview` (plays the animation on the selected shape immediately!), `Animation Pane` (right sidebar listing queued animations with reorder & delete), `Trigger` (`On Click`, `With Previous`, `After Previous`), `Duration`, and `Delay`.
   - **Slide Show Tab:** Make every button work: `Beginning (F5)` (starts fullscreen slideshow from Slide 1, skipping `[Hidden]` slides!), `Current (Shift+F5)` (starts from current slide), `Presenter View` (opens dual-pane Presenter console showing Current Slide, Next Slide preview, Speaker Notes, live elapsed Timer, and On-Slide Laser Pointer / Ink Pen!), `Hide Slide` (toggles `[Hidden]` state on current slide), `Rehearse Timings` (launches live slide timing recorder), and `Set Up Show` (Loop continuously option).
   - **Review & View Tabs:** `Spell Check`, `New Comment` (slide comment pins + sidebar), `Compare`, `Normal View`, `Outline View` (editable text outline of all slides), `Slide Sorter View` (full-screen grid of all slide thumbnails for fast drag-reordering!), `Notes Page View`, `Ruler`, `Gridlines` (alignment grid on slide), `Guides` (center crosshair guides), `Notes` toggle (bottom speaker notes drawer), `Grayscale / Black & White` preview mode, and Zoom controls.
2. Verify all `Animations`, `Slide Show`, `Review`, and `View` features with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 32: PDF STUDIO (.PDF) COMPLETE OVERHAUL — OPEN, EDIT, ENCRYPT, RESIZE & PAGES
    # ==========================================================================
    @{
        id = "32.1.1"; stage = 32; phase = "32.1"; title = "PDF Open/Save, Multi-Page Organizer, Page Resize & Password Encrypt/Decrypt"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "100% working PDF Open/Create/Save, Add/Delete/Duplicate/Rotate/Extract/Split/Merge pages, Resize PDF Page Dimensions, and Password Encrypt/Decrypt."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 32.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Fix the completely non-responsive PDF workspace so every single button works:
   - **Open & Render Existing PDFs + Create Blank PDFs:** Clicking `Open` (in Home or File) opens any `.pdf` file from disk, parses its pages/text/dimensions via `pdf-engine` (and browser PDF canvas/object renderer), and renders every page interactively in both the main viewport and the left `Pages` thumbnail strip!
   - **Page Tools Tab (Full Page Manipulation):**
     - `+ Insert Blank Page` (inserts a new page before/after current page and updates page count).
     - `Delete Page` (removes selected page, with confirmation if >1 page).
     - `Duplicate Page` (clones selected page and all its elements).
     - `Rotate CW 90°` & `Rotate CCW 90°` (rotates the page dimensions and content).
     - `Move Page Up / Down` & drag-and-drop thumbnail reordering.
     - **`Resize PDF Page Size` (Requested Feature):** Modal & toolbar selector to change page dimensions (`A4 210×297mm`, `Letter 8.5×11in`, `Legal 8.5×14in`, `A3`, `A5`, or `Custom Width × Height mm/px`) with option to `Scale Content to Fit New Page Size` and apply to Current Page or All Pages!
     - `Split PDF` (extract page range into a new PDF) & `Merge PDF` (append another `.pdf` file's pages into the current document).
   - **Sign & Protect Tab (Password Encryption & Decryption — Requested Feature):**
     - **`Encrypt with Password`:** Opens a security modal allowing the user to set an **Open Password (User Password)** and **Permissions Password (Owner Password — restrict printing/editing/copying)**. When saved or reopened, the PDF requires the password to unlock!
     - **`Decrypt / Remove Password`:** Allows unlocking a password-protected PDF by entering its password and removing encryption restrictions.
     - **`Compress / Optimize PDF Size`:** Reduces image resolution / strips unused objects and displays the before/after file size in KB.
2. Write automated tests verifying PDF Open, `+ Insert Page`, `Delete Page`, `Rotate`, `Resize PDF Page Size`, and `Encrypt / Decrypt Password`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "32.1.2"; stage = 32; phase = "32.1"; title = "PDF Direct Text/Image Editing, Annotations, Redaction, Forms & Signatures"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "100% working Home, Edit PDF, Forms, Comment & Annotate, Sign, and View tabs in PDF Studio."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 32.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** across PDF Studio's `Home`, `Edit PDF`, `Forms`, `Comment & Annotate`, `Sign & Protect`, and `View` tabs:
   - **Home Tab:** Make `Open`, `Hand` (drag-pan canvas), `Select` (select text/objects), `Snapshot` (captures selected region or current page as PNG to clipboard/download), `100%`, `Fit Page`, `Fit Width`, `Prev Page`, `Next Page`, `+ In` (Zoom In), `- Out` (Zoom Out), `Search` (opens live PDF text search bar highlighting matches across pages), `Print`, and `Save` (generates and downloads valid `.pdf` file) 100% functional!
   - **Edit PDF Tab:** `Edit Text & Objects` mode (click any text block on the PDF page to edit its text, font, size, or color in-place, drag to reposition, or press Delete), `+ Add Text` (click anywhere on the page to create a new formatted text block), `+ Add Image` (upload and resize image on the PDF page), `Add Links`, `Header & Footer` (stamps page numbers/dates across all pages), `Watermark` (diagonal or centered text/image watermark with opacity slider), and `Background Color`.
   - **Comment & Annotate Tab:** `Highlight`, `Underline`, `Strikethrough`, `Sticky Note` (places interactive comment marker + syncs with left `Notes` tab), `Callout Text Box`, `Freehand Draw / Pencil` (smooth ink drawing on the PDF canvas with color/thickness picker), `Eraser`, `Shapes (Rect, Circle, Arrow, Line)`, `Stamps (APPROVED, CONFIDENTIAL, DRAFT, REVIEWED, FINAL, VOID)`, and `Redact Content` (draws blackout redaction box that permanently strips underlying text/objects).
   - **Forms & Sign Tabs:** `+ Text Field`, `+ Checkbox`, `+ Radio Button`, `+ Dropdown`, `+ Date Field`, `+ Clear Form`, `Fill Form Mode`, and `Place Digital Signature` (draw signature on canvas pad or type cursive signature + date stamp, and place resizable signature onto the PDF).
2. Verify every single PDF tool and ribbon button with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 33: NOTEPAD / NOTES (.TXT / .MD) COMPLETE FIX — I/O & ALL RIBBONS
    # ==========================================================================
    @{
        id = "33.1.1"; stage = 33; phase = "33.1"; title = "Notepad / Notes Open/Save (.txt/.md) & Complete Home, Edit, View, Markdown Ribbons"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Notepad/Notes app with File Open/Save/Download, Find & Replace, Text Transformation Tools, and Split Markdown Preview."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 33.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix all missing and broken features in the **Notepad / Notes (`N`)** app:
   - **Open & Save Existing `.txt` / `.md` / `.json` / `.log` Files:** Wire `File -> Open` and `Home -> Open` to load any text/markdown file from disk into a scratchpad tab; wire `Save` (`Ctrl+S`) and `Save As` to download/write the `.txt` or `.md` file to disk; add `Rename Scratchpad` (double-click filename in left `SCRATCHPADS` list).
   - **Fix Live Status Bar Counts:** Wire both editor counters and global status bar to update on every keystroke and cursor movement!
   - **Home & Edit Ribbons:** `New Scratchpad`, `Open File`, `Save (.txt)`, `Save (.md)`, `Print`, `Undo`, `Redo`, `Cut`, `Copy`, `Paste`, `Select All`, `Find & Replace` bar (with Find Next/Prev, Replace, Replace All, Case Sensitive, and Regex toggles), `Go To Line (Ctrl+G)`, and **Text Power Tools** (`UPPERCASE`, `lowercase`, `Title Case`, `Sort Lines A-Z`, `Sort Lines Z-A`, `Remove Duplicate Lines`, `Trim Trailing Whitespace`, `Insert Time/Date F5`, `Word/Line Statistics`).
   - **View & Markdown Ribbons:** Font Family selector (`JetBrains Mono / Consolas`, `Inter`, `Serif`), Font Size (`11px`–`28px`), `Line # On/Off`, `Word Wrap On/Off`, `Editor / Split / Preview` mode switcher, and working Markdown toolbar buttons (`H1`, `H2`, `H3`, `> Quote`, `B Bold`, `i Italic`, `` ` Code ``, `Code Block`, `☑ Task Checkbox`, `Table`, `Link`, `Horizontal Rule`) that insert syntax at the cursor position in the `<textarea>`.
2. Verify all Notepad/Notes buttons and file Open/Save with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 34: EXHAUSTIVE AUTOMATED FEATURE-MATRIX VERIFICATION SUITE
    # ==========================================================================
    @{
        id = "34.1.1"; stage = 34; phase = "34.1"; title = "Automated Feature-Matrix Harness Testing Every Button in Word, Excel, PPT, PDF & TXT"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Automated test suite (exhaustive_feature_matrix.test.tsx) executing 150+ button/feature assertions across all 5 apps with 0 failures."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 34.1.1
Invoke @adaptive-ui-builder under GEC-v4.0 Article III.2.
1. Create `apps/unified-shell/src/__tests__/exhaustive_feature_matrix.test.tsx` that programmatically renders and tests every single feature implemented across all 5 applications:
   - **Suite 1 — Global Shell & Theme:** Asserts full-app Dark/Light theme cascade on root, sidebar, ribbon, workspace, and status bar; asserts zero Word `Navigation` ghost sidebar when in Excel, PPT, PDF, or Notes; asserts separate per-app window/mode filtering and `All Open Files` session manager.
   - **Suite 2 — Word (`.docx`):** Programmatically types paragraphs into page 1, verifies live word count updates, clicks `+ New Page` / `Blank Page` (asserts page count = 2), changes Font Family & Font Size, toggles Bold/Italic/Underline/Align/Bullets, applies `Heading 1` and verifies left Navigation outline updates, inserts Table & resizable Shape, changes Page Orientation to `Landscape` and Margins to `Narrow`, generates a `Table of Contents`, inserts a `Footnote`, toggles `Track Changes`, adds a `Comment`, and verifies `Save` / `Open`.
   - **Suite 3 — Excel (`.xlsx`):** Types numbers & formulas (`=SUM`, `=AVERAGE`, `=IF`), adds `Sheet2`, applies Currency `$` and Borders, inserts a Chart, sorts `A-Z` and `Z-A`, filters rows, runs `Goal Seek`, toggles `View Grid` and `View Headings`, freezes panes, and verifies `Save` / `Open`.
   - **Suite 4 — PowerPoint (`.pptx`):** Clicks `+ New Slide`, edits slide text inline, inserts and resizes a Shape, inserts a SmartArt/Chart, changes Design Theme and Slide Size (`16:9` / `4:3`), applies `Morph` transition, adds an Entrance animation, toggles `Hide Slide`, launches `Beginning (F5)` fullscreen player and steps through slides, and verifies `Save` / `Open`.
   - **Suite 5 — PDF Studio (`.pdf`):** Opens/creates a PDF, clicks `+ Insert Blank Page`, rotates page 90°, **resizes page dimensions (`A4` -> `Letter`)**, **encrypts PDF with a password and unlocks/decrypts it**, edits text in-place, adds a Sticky Note & Highlight, draws Freehand Ink, fills an AcroForm field, places a Digital Signature, and saves `.pdf`.
   - **Suite 6 — Notepad / Notes (`.txt`/`.md`):** Types text, verifies live `Ln/Col/Words/Chars` counters, clicks Markdown `H1`/`Bold`/`Task`/`Table` buttons, switches to `Split` preview, runs `Sort Lines A-Z` and `Find & Replace`, creates a second scratchpad, and saves/opens `.txt`.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1`, fix any failing assertion until 100% of tests pass, build the release binary, and commit + push with `[GEC-Verified]`.
"@
    }
)

# 1. Write Subphase Prompt Files (.agents/stages/stage-28..34/subphase-X.Y.Z.md)
foreach ($sp in $v4Subphases) {
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
    28 = 'Stage 28: Multi-Window Per Format, Full-App Theme, Ghost Sidebar Fix and Universal File IO'
    29 = 'Stage 29: Word (.docx) Complete Fix - Typing, Multi-Page, Shapes and All 7 Ribbons'
    30 = 'Stage 30: Excel (.xlsx) Complete Fix - All 7 Ribbons, Formulas, Charts, Sort/Filter and Page Layout'
    31 = 'Stage 31: PowerPoint (.pptx) Complete Fix - All 8 Ribbons, SmartArt, Animations and F5 Show'
    32 = 'Stage 32: PDF Studio (.pdf) Complete Overhaul - Open/Save, Edit Text, Page Resize and Password Encrypt/Decrypt'
    33 = 'Stage 33: Notepad / Notes (.txt/.md) Complete Fix - Open/Save, Text Tools and Split Markdown'
    34 = 'Stage 34: Exhaustive Automated Feature-Matrix Verification (150+ Automated UI Assertions)'
}

foreach ($s in 28..34) {
    $items = $v4Subphases | Where-Object { $_.stage -eq $s }
    $wfText = @"
---
description: Automated execution workflow for $($stageTitles[$s])
---

# Workflow: Execute Stage $s (/stage-$s)

## Objective: $($stageTitles[$s])
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

"@
    foreach ($sp in $items) {
        $wfText += @"

### Subphase $($sp.id): $($sp.title)
- **Spec File:** ``.agents/stages/stage-$s/subphase-$($sp.id).md``
$($sp.prompt)
- **Completion Gate:** ``powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete $($sp.id)``

"@
    }
    $wfText | Set-Content -Path ".agents/workflows/stage-$s.md" -Encoding UTF8
    Write-Host "  Created workflow: .agents/workflows/stage-$s.md" -ForegroundColor Gray
}

# 3. Append Stages 28-34 Non-Destructively to specs/stage-tracker.json
$trackerPath = "specs/stage-tracker.json"
if (Test-Path $trackerPath) {
    $tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json
    $existingIds = @($tracker.subphases | ForEach-Object { $_.id })
    $added = 0
    foreach ($sp in $v4Subphases) {
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
    $tracker.contract = "GEC-v4.0"
    $tracker.total_stages = 34
    $tracker.total_subphases = $tracker.subphases.Count
    $tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8
    Write-Host "  Appended $added subphases to specs/stage-tracker.json (Now 34 Stages, $($tracker.total_subphases) total subphases)." -ForegroundColor Green
}

Write-Host "=== [GEC-v4.0 Ready] Run /stage-28 through /stage-34 in Antigravity IDE! ===" -ForegroundColor Green
