description: Automated execution workflow for Stage 35: Real Binary File Open/Save Round-Trip (.docx, .xlsx, .pptx, .pdf, .txt) & App-Scoped Multi-Doc Tabs
Workflow: Execute Stage 35 (/stage-35)
Objective: Stage 35: Real Binary File Open/Save Round-Trip (.docx, .xlsx, .pptx, .pdf, .txt) & App-Scoped Multi-Doc Tabs

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v6.0) compliance:

Subphase 35.1.1: Native Tauri + Browser File Save & Open Bridge for All 5 Formats
Spec File: .agents/stages/stage-35/subphase-35.1.1.md
/execute-subphase 35.1.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v6.0 Article II.
1. **Fix Why 'Save' Does Nothing in Windows Tauri App:**
   - In Windows WebView2 (Tauri), synthetic <a download> links often fail silently or get blocked!
   - In `apps/unified-shell/src-tauri/src/lib.rs`, implement two Tauri IPC commands:
     - `#[tauri::command] fn save_file_bytes(file_name: String, bytes: Vec<u8>, custom_path: Option<String>) -> Result<String, String>`: Writes the bytes directly to `custom_path` or the user's `Downloads` / `Documents` directory (and supports native save path selection), returning the full saved file path (e.g., `C:\Users\...\Downloads\Book1.xlsx`).
     - `#[tauri::command] fn read_file_bytes(file_path: String) -> Result<Vec<u8>, String>`: Reads raw bytes from disk.
   - In the frontend, whenever the user clicks **`Save` (`Ctrl+S`)**, **`Save As`**, or **`Export`**, invoke `save_file_bytes` (with automatic fallback to `Blob` download in browser/test environments), clear the dirty `*` marker on the tab, and display a visible **Toast Notification banner** at the bottom-right: `"✅ Saved successfully: <filename>"`!
2. **Fix Why Opening Existing `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.txt` Files Opens Empty Tabs:**
   - Build a real binary parser pipeline in `apps/unified-shell/src/services/fileCodec.ts` (backed by Rust IPC / JSZip + XML DOMParser):
     - **`.docx` Open:** Unzips the `.docx` `ArrayBuffer`, parses `word/document.xml` (`<w:p>`, `<w:r>`, `<w:t>`, `<w:tbl>`, `<w:drawing>`), `word/header*.xml`, and `word/footer*.xml`, and populates `pages[0].bodyHtml`, `headerHtml`, and `footerHtml` with the exact content of the file! Also supports opening `.html`/`.txt`/JSON-wrapped `.docx` files saved by the app.
     - **`.xlsx` / `.csv` Open:** Unzips `xl/sharedStrings.xml` and `xl/worksheets/sheet*.xml` (and parses CSV/JSON-wrapped `.xlsx`), extracts all sheet names and cell `(row, col)` values/formulas, and populates the Excel grid immediately!
     - **`.pptx` Open:** Unzips `ppt/slides/slide*.xml` (and JSON-wrapped `.pptx`), extracts every slide's title, text boxes, shapes, and colors, and populates the PowerPoint slide deck!
     - **`.pdf` Open:** Parses the `.pdf` binary buffer (extracting page count, dimensions, embedded text strings between `BT..ET` / `(...) Tj`, and any embedded Aether PDF state metadata) OR renders the PDF pages via an embedded PDF object/canvas layer so existing PDF content is visible and editable!
     - **`.txt` / `.md` Open:** Reads full UTF-8 text and populates the Notepad buffer!
3. Write automated round-trip tests in Vitest: create a document with real content in Word, Excel, PPT, PDF, and TXT -> serialize via `saveFile` -> load the resulting `ArrayBuffer` via `openFile` -> assert 100% of the content is restored (NEVER empty!). Commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 35.1.1

Subphase 35.1.2: App-Scoped Multi-Instance Workspace Manager & Modern Visual Polish
Spec File: .agents/stages/stage-35/subphase-35.1.2.md
/execute-subphase 35.1.2
Invoke @adaptive-ui-builder under GEC-v6.0 Article III.
1. **Fix Multi-Document Organization (Stop Mixing All 5 File Types by Default):**
   - Change the default tab filter mode from `All Open Tabs` to **`Current App Only`** (`showAllFormats = false` by default, with a toggle pill `App Tabs | All Tabs (N)`):
     - When **Word (`W`)** is selected on the Left App Rail, the top tab bar shows **ONLY `.docx` tabs** (`Untitled 1.docx`, `test.docx`, etc.) + a **`+ New .docx`** button and **`📂 Open .docx`** button right next to the tabs!
     - When **Excel (`X`)** is selected, the top tab bar shows **ONLY `.xlsx` tabs** (`Book1.xlsx`, `test.xlsx`) + **`+ New .xlsx`** and **`📂 Open .xlsx`**.
     - When **PowerPoint (`P`)** is selected, shows **ONLY `.pptx` tabs** + **`+ New .pptx`** and **`📂 Open .pptx`**.
     - When **PDF (`PDF`)** is selected, shows **ONLY `.pdf` tabs** + **`+ New .pdf`** and **`📂 Open .pdf`**.
     - When **Notes (`N`)** is selected, shows **ONLY `.txt/.md` tabs** + **`+ New .txt`** and **`📂 Open .txt`**.
   - This makes opening 5 PDFs, 4 Word docs, and 6 Excel sheets completely intuitive, fast, and clutter-free!
2. **Fix Partial Dark / Light Theme Across All Views (Seen in image_9756a4.png):**
   - In `image_9756a4.png`, PDF Studio's left `Pages / Outline / Notes` sidebar and canvas backdrop stayed bright white even when `🌙 Dark` mode was active!
   - Audit every component in `apps/unified-shell/src/` and replace all hardcoded `#ffffff`, `#f8fafc`, `#f1f5f9`, `#e2e8f0` sidebar/panel/backdrop colors with theme-aware CSS variables (`var(--bg-surface)`, `var(--bg-sidebar)`, `var(--bg-canvas-surround)`, `var(--text-primary)`, `var(--border-color)`).
3. **Remove Empty Ribbon Group Stubs (Seen in image_97c287.png):**
   - In `image_97c287.png`, Word's Home ribbon has empty `Styles` and `Copilot` group headers on row 2 with no buttons visible inside them! Ensure every ribbon group renders its buttons visibly inside the group container with zero vertical clipping.
4. Verify with automated DOM tests and commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 35.1.2
