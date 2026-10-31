# Subphase 40.1.1: Notepad / TXT File Open/Save Round-Trip & Complete 5-Tab Ribbon Functionality

- **Stage:** 40 | **Phase:** 40.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Open/Save for .txt/.md/.json/.log, working Home, Edit, Text Tools, View, and Markdown ribbons.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 40.1.1
Invoke @adaptive-ui-builder under GEC-v6.0.
1. **Fix Notepad File Open & Save (`image_974303.jpg`):**
   - Wire `Open` in Notepad to read any local `.txt`, `.md`, `.json`, `.csv`, `.xml`, `.log`, or `.ini` file and load its full text into the active editor and `SCRATCHPADS` list!
   - Wire `Save`, `Save .txt`, and `Save .md` to write the file via `save_file_bytes` / download and show the `"✅ Saved <filename>"` toast!
2. **Wire & Verify Every Button Across All 5 Notepad Ribbon Tabs (`Home`, `Edit`, `Text Tools`, `View`, `Markdown`):**
   - **Home:** `New`, `Open`, `Save .txt`, `Save .md`, `Rename`, `Delete`, `Print`, `Font Family`, `Font Size`, `Wrap`, `Line Numbers`, `Zoom In/Out/100%`, `Time/Date (F5)`, plus quick utilities (`AA Upper`, `aa Lower`, `Aa Title`, `Sort A-Z`, `Sort Z-A`, `De-dup Lines`, `Trim Whitespace`, `Stats` modal).
   - **Edit:** `Undo`, `Redo`, `Cut`, `Copy`, `Paste`, `Select All`, `Find` (opens interactive search bar that highlights matches and jumps to Next/Prev), `Replace` (opens interactive Replace / Replace All bar with Case & Regex toggles), `Go To Line` (jumps cursor to line N), `Duplicate Line`, `Delete Line`.
   - **Text Tools:** `Format JSON`, `Minify JSON`, `Base64 Encode`, `Base64 Decode`, `URL Encode`, `URL Decode`, `Remove Empty Lines`, `Number Lines (1. 2. 3.)`, `Reverse Lines`, `Word Frequency Analysis`.
   - **View & Markdown:** `Editor`, `Split` (side-by-side editor and live rendered HTML Markdown preview), `Preview`, `Encoding (UTF-8 / ASCII)`, `Line Endings (LF / CRLF)`, and all Markdown syntax insertion buttons (`H1`, `H2`, `H3`, `Bold`, `Italic`, `Strikethrough`, `Code`, `Code Block`, `Quote`, `Task - [ ]`, `Bullet List`, `Numbered List`, `Table`, `Link`, `Image`, `Horizontal Rule`).
3. Verify every Notepad button and `.txt`/`.md` Open/Save round-trip with automated DOM tests. Commit with `[GEC-Verified]`.
````
