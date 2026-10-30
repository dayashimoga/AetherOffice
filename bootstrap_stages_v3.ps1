# ==============================================================================
# bootstrap_stages_v3.ps1 - Generates Stages 21 to 27 (Complete Overhaul)
# Without overwriting existing Stages 1 to 20!
# Covers: Single-Process Memory Fix, Window Controls, Dark/Light Theme, Ribbon Fix,
#         Blank Editable Word, Excel, PowerPoint, PDF Studio, and Notepad/Notes.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [Overhaul 2/2] Generating Stages 21-27 Prompts, Workflows & Updating Tracker ===" -ForegroundColor Cyan

foreach ($s in 21..27) {
    New-Item -ItemType Directory -Force -Path ".agents/stages/stage-$s" | Out-Null
}

$overhaulSubphases = @(
    # ==========================================================================
    # STAGE 21: MEMORY HANG FIX, SINGLE-APP SHELL, WINDOW CONTROLS & THEME ENGINE
    # ==========================================================================
    @{
        id = "21.1.1"; stage = 21; phase = "21.1"; title = "Single-Process Consolidation, Lazy Memory State & Native Window Controls"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Single AetherOffice.exe binary (<45MB RAM), working Minimize/Maximize/Close buttons, and removal of all background CPU/RAM leaks."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 21.1.1
Invoke @adaptive-ui-builder under GEC-v3.0 Article I & III.
1. **Fix Memory Hang & Consolidate Binary:**
   - Ensure `apps/unified-shell/src-tauri/Cargo.toml` builds ONLY ONE binary (`AetherOffice`) so opening multiple documents never spawns multiple WebView2 Chromium instances.
   - Audit `apps/unified-shell/src-tauri/src/` and `crates/` to ensure no giant vectors or 1M-cell arrays are allocated on boot. Use lazy `HashMap` storage initialized empty.
2. **Fix Window Minimize, Maximize & Close Buttons:**
   - In `apps/unified-shell/src-tauri/tauri.conf.json`, enable native window controls (`"decorations": true`) OR wire the custom titlebar buttons (`_`, `[]`, `X`) directly to Tauri v2's window API (`getCurrentWindow().minimize()`, `getCurrentWindow().toggleMaximize()`, `getCurrentWindow().close()`) with fallback `window.close()`.
3. **Purge All Hardcoded Fake Demo Text:**
   - Remove `"OpenOfficeRS — Executive Technical Summary"`, `"Performance SLAs & Benchmark Compliance"`, `"Universal Adaptive Form-Factor Parity"`, and all prepopulated numbers/shapes from the codebase so Gate 2 of `tools/verify-contract.ps1` passes.
4. Commit with `feat(subphase-21.1.1): single-process memory fix and working window controls [GEC-Verified]`.
"@
    },
    @{
        id = "21.1.2"; stage = 21; phase = "21.1"; title = "Unified Left App Switcher Rail, Home Hub & Multi-Document Tab Router"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Single-window Office Suite with Left App Rail (Home, Word, Excel, PowerPoint, PDF, Notes) and working multi-format tab bar."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 21.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build a unified Microsoft 365 / WPS-style **Left App Switcher Rail** (`48px` wide vertical dock on the left edge of the window) with icons and tooltips for:
   - **Home Hub** (Dashboard with 5 large 'Create New' cards: Blank Word `.docx`, Blank Excel `.xlsx`, Blank PowerPoint `.pptx`, Open/Blank PDF `.pdf`, and Blank Note `.txt/.md`).
   - **Word (`W` blue icon)** — Instantly creates/switches to a blank editable `.docx` tab.
   - **Excel (`X` green icon)** — Instantly creates/switches to a blank editable `.xlsx` tab (`Book1.xlsx`), NEVER opening `.docx` by mistake!
   - **PowerPoint (`P` orange icon)** — Instantly creates/switches to a blank editable `.pptx` tab (`Presentation1.pptx`).
   - **PDF Studio (`PDF` red icon)** — Instantly creates/switches to a PDF Studio tab (`Document.pdf`).
   - **Notepad / Notes (`N` purple icon)** — Instantly creates/switches to a clean Notepad/Markdown tab (`Untitled.txt`).
2. Fix the top `+` New Tab button: clicking `+` opens a quick dropdown menu to launch a blank Word, Excel, PowerPoint, PDF, or Note tab, and clicking `x` on any tab closes that tab cleanly.
3. Verify via automated DOM tests that launching Excel, PPT, PDF, or Notes renders its exact respective engine, never `.docx`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "21.2.1"; stage = 21; phase = "21.2"; title = "Visual Overhaul: Non-Overflowing Flexbox Ribbon & Dark/Light Theme Toggle"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Pixel-perfect aligned Ribbon without horizontal/vertical scrollbars, fixed active tab highlight, and instant Dark/Light mode switch."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 21.2.1
Invoke @adaptive-ui-builder under GEC-v3.0 Article III.
1. **Fix Ribbon Overflow & Alignment Bug:**
   - Remove the broken `overflow: scroll` / fixed small height on the Ribbon toolbar that caused ugly scrollbars to cover the buttons.
   - Redesign the Ribbon using a clean, modern single-row or compact two-row CSS Flexbox layout (`min-height: 88px`, `overflow: visible`, crisp `1px solid var(--border)` group dividers, properly aligned `28px` buttons and dropdowns).
   - Fix the tab highlight bug: `File` must only be highlighted when the File Backstage menu is open; when `Home`, `Insert`, `Layout`, etc. are clicked, ONLY the clicked tab shows the active underline/pill.
2. **Implement Global Dark / Light Theme Toggle (`Light / Dark`):**
   - Add a prominent Theme Toggle button in the top titlebar (`Light Mode` vs `Dark Mode`).
   - Define full CSS custom properties for Light (`#f8fafc` app bg, `#ffffff` ribbon/surface, `#0f172a` text) and Dark (`#0f172a` app bg, `#1e293b` ribbon/surface, `#f8fafc` text, `#334155` borders) so the entire UI flips smoothly with zero unreadable text.
3. Verify with DOM tests that toggling the theme updates `document.documentElement.dataset.theme` and that every Ribbon tab switches cleanly. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 22: WORD (WRITER) — BLANK EDITABLE PAGES, MULTI-PAGE & WORKING RIBBON
    # ==========================================================================
    @{
        id = "22.1.1"; stage = 22; phase = "22.1"; title = "Blank Editable Word Pages, Rich Typing, Selection & New Page Creation"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% blank starting page where users can type, edit, delete, format rich text, and create/delete multiple pages."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 22.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Word (`Writer`) page editor component (`apps/unified-shell/src/apps/writer/`) so it is **immediately editable**:
   - Every new `.docx` tab starts with **1 clean, completely blank A4/Letter page** (zero prepopulated text!) focused automatically so the user can start typing immediately.
   - Use a synchronized rich-text `contentEditable` page array backed by the Rust `word-engine` state so typing, Backspace, Delete, Enter, Cut/Copy/Paste, Undo (`Ctrl+Z`), and Redo (`Ctrl+Y`) work with zero lag.
2. **Working Multi-Page Creation & Management:**
   - Add a **`+ New Page`** button on the `Home` and `Insert` ribbons (and at the bottom of the last page), plus `Ctrl+Enter` shortcut, that immediately appends a new editable page sheet (`Page 2 of 2`, `Page 3 of 3`) and focuses it.
   - Add a **`Delete Empty Page`** action and automatic page overflow splitting when text on a page exceeds the page height.
   - Make the Header and Footer areas at the top/bottom of each page editable on click.
3. Write automated DOM tests proving a new Word tab has empty text, accepts typed input, and creates Page 2 when `+ New Page` is clicked. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "22.1.2"; stage = 22; phase = "22.1"; title = "Working Word Ribbon: Font, Alignment, Lists, Styles, Tables, Images & Export"
        crate = "word-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Word ribbon buttons across Home, Insert, Layout, Review, and View tabs."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 22.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Word Ribbon so clicking it actually modifies the selected text or inserts elements into the active page:
   - **Home Tab:** Font family dropdown (`Calibri`, `Arial`, `Times New Roman`, `Inter`, `Georgia`, `Courier New`), Font Size (`8`–`72`), `A+` / `A-`, Bold (`B`), Italic (`I`), Underline (`U`), Strikethrough (`S`), Subscript (`X2`), Superscript (`X2`), Text Color picker, Highlight Color picker, Clear Formatting, Bullet List, Numbered List, Indent/Outdent, Align Left/Center/Right/Justify, Line Spacing (`1.0`, `1.15`, `1.5`, `2.0`), and Styles (`Normal`, `Heading 1`, `Heading 2`, `Heading 3`, `Title`, `Quote`).
   - **Live Navigation Outline:** Whenever a user formats a line as `Heading 1/2/3`, dynamically list that real heading in the left `Navigation` sidebar (never show fake hardcoded headings!).
   - **Insert Tab:** `+ Blank Page`, `Insert Table` (inserts an editable HTML/OOXML table with resizable cells and add/remove row/col controls), `Insert Image` (opens file picker and inserts resizable image), `Insert Horizontal Line`, `Insert Link`, `Insert Date/Time`.
   - **Layout Tab:** Page Orientation (`Portrait` / `Landscape`), Page Margins (`Normal`, `Narrow`, `Wide`), Paper Size (`A4`, `Letter`), Page Background Color.
   - **Review & View Tabs:** Live Word/Character/Paragraph/Page count dialog, Find & Replace bar (`Ctrl+F`), Zoom (`50%`–`200%`), Toggle Ruler, Toggle Navigation Sidebar.
2. Verify all Word ribbon actions with automated tests and commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 23: EXCEL (CALC) — BLANK VIRTUALIZED GRID, FORMULAS, SHEETS & CHARTS
    # ==========================================================================
    @{
        id = "23.1.1"; stage = 23; phase = "23.1"; title = "Blank Virtualized Excel Grid, Instant Typing, Formula Engine & Multi-Sheet Bar"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% blank spreadsheet grid with direct cell typing, working Formula Bar (=SUM, =AVERAGE, etc.), and + New Sheet tabs."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 23.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Excel (`Calc`) workspace (`apps/unified-shell/src/apps/calc/`) so it opens to a **100% blank, editable grid** (`Book1.xlsx` -> `Sheet1` with zero fake numbers):
   - **Lightweight Virtualized Grid:** Render a smooth scrollable grid where clicking any cell selects it, and **typing any key immediately starts editing that cell** (or double-clicking / pressing `F2`).
   - **Navigation Keys:** `Enter` commits and moves down, `Tab` commits and moves right, `Arrow` keys move selection when not editing, `Delete`/`Backspace` clears selected cell(s), and `Escape` cancels editing.
   - **Live Formula Evaluator:** Typing `=SUM(A1:B5)`, `=AVERAGE(...)`, `=MIN(...)`, `=MAX(...)`, `=COUNT(...)`, `=IF(A1>10, "Yes", "No")`, `=VLOOKUP(...)`, or math expressions (`=A1*B1+100`) evaluates immediately and updates dependent cells automatically when source cells change.
   - **Working Multi-Sheet Management:** Add a bottom Sheet Tab Bar (`Sheet1`, `Sheet2`, ...) with a working **`+` button to create new sheets**, double-click to rename a sheet, right-click/button to delete a sheet, and sync with the left `Workbook Sheets` list.
2. Write automated DOM tests verifying blank boot, typing numbers into `A1` and `A2`, evaluating `=SUM(A1:A2)` in `A3`, and clicking `+` to create `Sheet2`. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "23.1.2"; stage = 23; phase = "23.1"; title = "Working Excel Ribbon: Formatting, Sort/Filter, Row/Col Ops & Live Charts"
        crate = "excel-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% working Excel ribbon across Home, Insert (Charts), Page Layout, Formulas, Data (Sort/Filter), and View tabs."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 23.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Excel Ribbon to operate on the selected cell or cell range:
   - **Home Tab:** Font family/size, Bold/Italic/Underline, Cell Fill Color, Font Color, Cell Borders (`All`, `Outside`, `None`), Align Left/Center/Right, Number Format (`General`, `Currency $`, `Percentage %`, `Comma ,`, `+ .00`, `- .00`), `Insert Row Above`, `Insert Column Left`, `Delete Row`, `Delete Column`, `Clear All`.
   - **Insert Tab:** `Bar Chart`, `Line Chart`, `Pie Chart` (creates a floating, draggable, live-updating SVG chart from the currently selected cells), `Insert Comment`.
   - **Formulas Tab:** `AutoSum (Σ)` (automatically inserts `=SUM(...)` for the cells above/left), `AVERAGE`, `COUNT`, `MAX`, `MIN`, `Insert Function (fx)` dialog, and `Show Formulas` toggle (displays raw formulas vs computed values across the sheet).
   - **Data Tab:** Working `Sort A -> Z`, `Sort Z -> A`, `Filter` toggle, `Remove Duplicates`, and `Text to Columns` (splits comma-separated cell text across adjacent columns).
   - **Status Bar:** Live display of `SUM`, `AVERAGE`, `COUNT`, `MIN`, `MAX` for any multi-cell selection, plus Zoom slider.
2. Verify via automated DOM tests and commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 24: POWERPOINT (SLIDES) — BLANK SLIDES, SHAPE EDITOR & SLIDE SHOW
    # ==========================================================================
    @{
        id = "24.1.1"; stage = 24; phase = "24.1"; title = "Blank Editable Slides, + New Slide, Drag/Resize/Rotate & Inline Text Editing"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Clean blank presentation where users can add/delete slides, type directly into slide boxes, and drag/resize/rotate shapes."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 24.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the PowerPoint (`Slides`) workspace (`apps/unified-shell/src/apps/slides/`) so it starts with **1 clean blank Title Slide** (with editable `"Click to add title"` and `"Click to add subtitle"` placeholders—zero hardcoded stars or fake demo text!):
   - **Inline Text Editing:** Clicking any text box or shape lets the user immediately type, delete, and format text directly on the 16:9 slide canvas.
   - **Interactive Drag, Resize & Rotate:** Selecting any object shows an interactive bounding frame with **8 resize handles** and a **rotation handle**. Dragging moves the element smoothly; dragging handles resizes or rotates it; pressing `Delete` removes the selected element.
   - **Slide Thumbnail Rail:** Working **`+ New Slide`** button (with layout choices: `Title Slide`, `Title & Content`, `Two Column`, `Blank`), `Duplicate Slide`, `Delete Slide`, and Move Up/Down buttons in the left sidebar.
2. Write automated DOM tests verifying `+ New Slide`, typing text into a slide box, and inserting/resizing a shape. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "24.1.2"; stage = 24; phase = "24.1"; title = "Working PowerPoint Ribbon: Shapes, Images, Themes, Transitions & Fullscreen Show"
        crate = "ppt-engine"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Complete working PowerPoint ribbon across Home, Insert, Design, Transitions, Animations, and Fullscreen Slide Show."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 24.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the PowerPoint Ribbon:
   - **Home & Insert Tabs:** `+ New Slide`, `Insert Text Box`, `Insert Image` (from file), `Insert Table`, `Insert Shape` (`Rectangle`, `Rounded Rect`, `Circle`, `Triangle`, `Arrow`, `Star`, `Callout`), Font/Size/Bold/Italic/Color/Alignment for selected shape text, Shape Fill Color, Shape Border Color/Width, `Bring to Front`, `Send to Back`, `Duplicate`, `Delete`.
   - **Design Tab:** 8 curated Slide Themes (Dark Slate, Ocean Blue, Emerald, Sunset, Minimal White, Executive Navy, Warm Sand, Cyber Neon) and custom Slide Background Color/Gradient picker.
   - **Transitions & Animations Tabs:** Slide Transition selector (`None`, `Fade`, `Slide Left`, `Zoom`, `Flip`), Object Animation selector (`None`, `Fade In`, `Fly Up`, `Pop`, `Spin`), and `Preview Animation` button.
   - **Slide Show Tab (`F5`):** Fullscreen Presentation Player overlay with animated slide transitions, Left/Right/Spacebar navigation, slide counter (`1 / N`), Laser Pointer cursor toggle, and `Esc` to exit.
   - **Speaker Notes Bar:** Collapsible bottom panel to type speaker notes for each slide.
2. Verify via automated DOM tests and commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 25: PDF STUDIO — CREATE, OPEN, IN-PLACE EDIT, ANNOTATE, FORMS & SIGN
    # ==========================================================================
    @{
        id = "25.1.1"; stage = 25; phase = "25.1"; title = "PDF Studio Workspace: Create Blank PDF, Open Local PDF & Page Management"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "Full PDF viewer and page organizer supporting Open .pdf, Create Blank PDF, Add/Delete/Rotate/Reorder pages, and Zoom."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 25.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Build the interactive **PDF Studio** module (`apps/unified-shell/src/apps/pdf/`) accessible from the Left App Rail and New Tab menu:
   - Start with a clean, editable blank PDF page (or click `Open PDF` to load and parse any `.pdf` file from disk via `pdf-engine`).
   - **Page Management Toolbar:** `+ Add Blank Page`, `Delete Page`, `Rotate 90 deg CW`, `Rotate 90 deg CCW`, `Move Page Up/Down`, and left Page Thumbnails sidebar.
   - **Direct Text & Image Authoring on PDF:** Click `+ Add Text` to place movable, editable text blocks anywhere on any PDF page (with Font, Size, Color, Bold/Italic controls); click `+ Add Image` to stamp images onto a PDF page; click any existing text block in `Edit Text` mode to modify its text in-place!
2. Verify PDF page addition, rotation, and text block editing with automated DOM tests. Commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "25.1.2"; stage = 25; phase = "25.1"; title = "PDF Studio Annotations, Freehand Ink, Form Builder/Filler, Signature & Export"
        crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false
        deliverable = "Highlight, Freehand Draw, Sticky Notes, Redaction, Interactive Form Fields, Signature Pad, and valid .pdf file download/save."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 25.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Complete all remaining PDF Studio Ribbon tabs:
   - **Annotate Tab:** `Highlight Box`, `Freehand Ink Pen` (smooth HTML5 Canvas drawing overlay per page with color & stroke width picker + `Eraser`), `Sticky Note` (draggable comment pin with editable note text), and `Redact Box` (blackout rectangle).
   - **Forms Tab:** `+ Add Text Field` and `+ Add Checkbox` (places interactive form inputs onto the PDF page that users can type into and toggle).
   - **Sign & Export Tab:** `Sign Document` modal where users can draw their signature with mouse/touch or type their name in a cursive signature font and place the transparent signature stamp anywhere on the PDF page; `Save / Export PDF` button that compiles all pages, text, ink, form values, and signatures into a real downloadable/saveable `.pdf` binary via `pdf-engine`.
2. Verify via automated DOM tests and commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 26: NOTEPAD / NOTES APP (PLAIN TEXT, MARKDOWN & QUICK NOTES)
    # ==========================================================================
    @{
        id = "26.1.1"; stage = 26; phase = "26.1"; title = "Ultra-Fast Notepad & Markdown Notes Editor (.txt / .md) with Live Preview"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "Built-in Notepad & Notes app with line numbers, Markdown split-preview, Regex Find/Replace, Checklist support, and .txt/.md I/O."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 26.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build the 5th core module in the suite: **Notepad & Notes (`apps/unified-shell/src/apps/notes/`)**:
   - **Editor Modes:** Supports both **Classic Notepad Mode** (ultra-fast plain text editor with synchronized line-number gutter, word-wrap toggle, monospace/sans font selector, tab-size selector, and `F5` Insert Time/Date) AND **Rich Markdown Notes Mode** (side-by-side live Markdown preview rendering headings, tables, code blocks, and interactive `- [ ]` task checkboxes).
   - **Notes Notebook Sidebar:** Left sidebar allowing users to create and organize multiple quick notes/scratchpads persisted automatically, plus open and save `.txt`, `.md`, `.json`, and `.log` files.
   - **Notepad Ribbon:** `New Note`, `Open .txt/.md`, `Save`, `Find & Replace` (with Case Sensitive & Regex toggles), `Word Wrap`, `Line Numbers`, `Split Preview`, `Uppercase/Lowercase/Sort Lines` text utilities, and live Line/Column/Char/Word/Encoding (`UTF-8`) status bar.
2. Write automated DOM tests verifying note creation, typing, line counting, and Markdown preview. Commit with `[GEC-Verified]`.
"@
    },

    # ==========================================================================
    # STAGE 27: UNIVERSAL FILE OPEN/SAVE/PRINT & FULL E2E VERIFICATION
    # ==========================================================================
    @{
        id = "27.1.1"; stage = 27; phase = "27.1"; title = "Universal File Open/Save/Export (.docx, .xlsx, .pptx, .pdf, .txt) & Shortcuts"
        crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false
        deliverable = "100% working File -> New, Open, Save, Save As, Export PDF/CSV/HTML/TXT, Print, and global keyboard shortcuts."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 27.1.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v3.0.
1. Wire the `File` Backstage Menu and global keyboard shortcuts (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+P`, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+F`) so they work across all 5 modules (Word, Excel, PowerPoint, PDF, Notepad):
   - **Open File (`Ctrl+O`):** Uses native `<input type="file">` + Tauri file dialog to open real `.docx`, `.xlsx`, `.csv`, `.pptx`, `.pdf`, `.txt`, and `.md` files from the user's computer and load their contents into the appropriate editor tab.
   - **Save / Download File (`Ctrl+S`):** Serializes the current active tab via the Rust backend (or browser Blob download fallback) into a valid `.docx`, `.xlsx`, `.csv`, `.pptx`, `.pdf`, or `.txt` file on the user's machine.
   - **Print (`Ctrl+P`):** Clean print stylesheet that prints only the active document canvas (`window.print()`).
2. Verify round-trip Save & Open across all 5 formats with automated tests and commit with `[GEC-Verified]`.
"@
    },
    @{
        id = "27.1.2"; stage = 27; phase = "27.1"; title = "Final GEC-v3.0 E2E Verification & Single-App Release Packaging"
        crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false
        deliverable = "100% passing E2E interactive tests across Word, Excel, PPT, PDF, and Notepad + clean single-binary release build."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @"
/execute-subphase 27.1.2
Execute final GEC-v3.0 Release Audit:
1. Write a comprehensive Vitest DOM test suite (`apps/unified-shell/src/__tests__/gec_v3_overhaul.test.tsx`) asserting:
   - Zero prepopulated fake strings appear in any newly created Word, Excel, PowerPoint, PDF, or Notepad tab.
   - Switching via the Left App Rail opens Word, Excel, PowerPoint, PDF, and Notepad accurately (never defaulting to `.docx`).
   - Word allows typing, formatting, and creating Page 2 (`+ New Page`).
   - Excel allows typing into cells, evaluating `=SUM(A1:A2)`, and adding `Sheet2`.
   - PowerPoint allows adding slides, typing in text boxes, and inserting shapes.
   - PDF Studio allows adding pages, adding/editing text boxes, drawing annotations, and filling forms.
   - Notepad allows typing plain text/Markdown and toggling Dark/Light theme.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1` and verify a clean single-binary release build (`cargo build --release --jobs 2`). Commit and push with `[GEC-Verified]`.
"@
    }
)

# 1. Write Subphase Prompt Files (.agents/stages/stage-21..27/subphase-X.Y.Z.md)
foreach ($sp in $overhaulSubphases) {
    $stageFolder = ".agents/stages/stage-$($sp.stage)"
    $filePath = "$stageFolder/subphase-$($sp.id).md"
    $specContent = @"
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
"@
    Set-Content -Path $filePath -Value $specContent -Encoding UTF8
    Write-Host "  Created prompt spec: $filePath" -ForegroundColor Gray
}

# 2. Write Stage Workflow Files (.agents/workflows/stage-21.md .. stage-27.md)
$stageTitles = @{
    "21" = "Single-Process Memory Fix, Window Controls, App Rail & Dark/Light Theme"
    "22" = "Word (Writer) - Blank Editable Pages, Multi-Page & Working Ribbon"
    "23" = "Excel (Calc) - Blank Virtualized Grid, Formulas, Sheets & Charts"
    "24" = "PowerPoint (Slides) - Blank Slides, Shape Editor & Slide Show"
    "25" = "PDF Studio - Create, Open, In-Place Edit, Annotate, Forms & Sign"
    "26" = "Notepad / Notes App (Plain Text, Markdown & Quick Notes)"
    "27" = "Universal File Open/Save/Print & Full E2E Verification"
}

foreach ($s in 21..27) {
    $sSubphases = $overhaulSubphases | Where-Object { $_.stage -eq $s }
    $title = $stageTitles["$s"]
    $wfContent = @"
description: Automated execution workflow for Stage $($s): $($title)
Workflow: Execute Stage $($s) (/stage-$($s))
Objective: Stage $($s): $($title)

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:

"@
    foreach ($sp in $sSubphases) {
        $wfContent += @"
Subphase $($sp.id): $($sp.title)
Spec File: .agents/stages/stage-$($s)/subphase-$($sp.id).md
$($sp.prompt)

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete $($sp.id)

"@
    }
    $wfPath = ".agents/workflows/stage-$s.md"
    Set-Content -Path $wfPath -Value $wfContent -Encoding UTF8
    Write-Host "  Created workflow: $wfPath" -ForegroundColor Gray
}

# 3. Update specs/stage-tracker.json non-destructively
$trackerPath = "specs/stage-tracker.json"
$tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json

$tracker.contract = "GEC-v3.0"
$tracker.total_stages = 27

# Add only subphases that aren't already present in $tracker.subphases
$existingIds = @($tracker.subphases | ForEach-Object { $_.id })
$addedCount = 0

foreach ($sp in $overhaulSubphases) {
    if ($existingIds -notcontains $sp.id) {
        $newObj = [PSCustomObject]@{
            id = $sp.id
            stage = [int]$sp.stage
            phase = [string]$sp.phase
            title = [string]$sp.title
            crate = [string]$sp.crate
            rea_required = [bool]$sp.rea
            status = "PENDING"
            prompt_file = ".agents/stages/stage-$($sp.stage)/subphase-$($sp.id).md"
            completed_at = $null
        }
        $tracker.subphases += $newObj
        $addedCount++
    }
}

$tracker.total_subphases = $tracker.subphases.Count

$tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8
Write-Host "  Updated specs/stage-tracker.json (Added $addedCount new subphases, total: $($tracker.total_subphases))." -ForegroundColor Green

Write-Host "=== Overhaul Stages 21-27 successfully installed! ===" -ForegroundColor Green
