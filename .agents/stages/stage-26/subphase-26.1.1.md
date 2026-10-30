# Subphase 26.1.1: Ultra-Fast Notepad & Markdown Notes Editor (.txt / .md) with Live Preview
- **Stage:** 26 | **Phase:** 26.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Built-in Notepad & Notes app with line numbers, Markdown split-preview, Regex Find/Replace, Checklist support, and .txt/.md I/O.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 26.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build the 5th core module in the suite: **Notepad & Notes (pps/unified-shell/src/apps/notes/)**:
   - **Editor Modes:** Supports both **Classic Notepad Mode** (ultra-fast plain text editor with synchronized line-number gutter, word-wrap toggle, monospace/sans font selector, tab-size selector, and F5 Insert Time/Date) AND **Rich Markdown Notes Mode** (side-by-side live Markdown preview rendering headings, tables, code blocks, and interactive - [ ] task checkboxes).
   - **Notes Notebook Sidebar:** Left sidebar allowing users to create and organize multiple quick notes/scratchpads persisted automatically, plus open and save .txt, .md, .json, and .log files.
   - **Notepad Ribbon:** New Note, Open .txt/.md, Save, Find & Replace (with Case Sensitive & Regex toggles), Word Wrap, Line Numbers, Split Preview, Uppercase/Lowercase/Sort Lines text utilities, and live Line/Column/Char/Word/Encoding (UTF-8) status bar.
2. Write automated DOM tests verifying note creation, typing, line counting, and Markdown preview. Commit with [GEC-Verified].
```
