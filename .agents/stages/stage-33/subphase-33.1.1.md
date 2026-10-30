# Subphase 33.1.1: Notepad / Notes Open/Save (.txt/.md) & Complete Home, Edit, View, Markdown Ribbons
- **Stage:** 33 | **Phase:** 33.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Notepad/Notes app with File Open/Save/Download, Find & Replace, Text Transformation Tools, and Split Markdown Preview.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 33.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix all missing and broken features in the **Notepad / Notes (N)** app:
   - **Open & Save Existing .txt / .md / .json / .log Files:** Wire File -> Open and Home -> Open to load any text/markdown file from disk into a scratchpad tab; wire Save (Ctrl+S) and Save As to download/write the .txt or .md file to disk; add Rename Scratchpad (double-click filename in left SCRATCHPADS list).
   - **Fix Live Status Bar Counts:** Wire both editor counters and global status bar to update on every keystroke and cursor movement!
   - **Home & Edit Ribbons:** New Scratchpad, Open File, Save (.txt), Save (.md), Print, Undo, Redo, Cut, Copy, Paste, Select All, Find & Replace bar (with Find Next/Prev, Replace, Replace All, Case Sensitive, and Regex toggles), Go To Line (Ctrl+G), and **Text Power Tools** (UPPERCASE, lowercase, Title Case, Sort Lines A-Z, Sort Lines Z-A, Remove Duplicate Lines, Trim Trailing Whitespace, Insert Time/Date F5, Word/Line Statistics).
   - **View & Markdown Ribbons:** Font Family selector (JetBrains Mono / Consolas, Inter, Serif), Font Size (11pxâ€“28px), Line # On/Off, Word Wrap On/Off, Editor / Split / Preview mode switcher, and working Markdown toolbar buttons (H1, H2, H3, > Quote, B Bold, i Italic, `  Code `, Code Block, â˜‘ Task Checkbox, Table, Link, Horizontal Rule) that insert syntax at the cursor position in the <textarea>.
2. Verify all Notepad/Notes buttons and file Open/Save with automated DOM tests. Commit with [GEC-Verified].
```
