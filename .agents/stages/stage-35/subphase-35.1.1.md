# Subphase 35.1.1: Native Tauri + Browser File Save & Open Bridge for All 5 Formats

- **Stage:** 35 | **Phase:** 35.1
- **Target Subsystem:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **Deliverable:** Working Save, Save As, and Open across Desktop (Tauri Rust FS) and Web for .docx, .xlsx, .pptx, .pdf, and .txt.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
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
````
