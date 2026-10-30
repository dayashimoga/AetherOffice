description: Automated execution workflow for Stage 26: Notepad / Notes App (Plain Text, Markdown & Quick Notes)
Workflow: Execute Stage 26 (/stage-26)
Objective: Stage 26: Notepad / Notes App (Plain Text, Markdown & Quick Notes)

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 26.1.1: Ultra-Fast Notepad & Markdown Notes Editor (.txt / .md) with Live Preview
Spec File: .agents/stages/stage-26/subphase-26.1.1.md
/execute-subphase 26.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build the 5th core module in the suite: **Notepad & Notes (pps/unified-shell/src/apps/notes/)**:
   - **Editor Modes:** Supports both **Classic Notepad Mode** (ultra-fast plain text editor with synchronized line-number gutter, word-wrap toggle, monospace/sans font selector, tab-size selector, and F5 Insert Time/Date) AND **Rich Markdown Notes Mode** (side-by-side live Markdown preview rendering headings, tables, code blocks, and interactive - [ ] task checkboxes).
   - **Notes Notebook Sidebar:** Left sidebar allowing users to create and organize multiple quick notes/scratchpads persisted automatically, plus open and save .txt, .md, .json, and .log files.
   - **Notepad Ribbon:** New Note, Open .txt/.md, Save, Find & Replace (with Case Sensitive & Regex toggles), Word Wrap, Line Numbers, Split Preview, Uppercase/Lowercase/Sort Lines text utilities, and live Line/Column/Char/Word/Encoding (UTF-8) status bar.
2. Write automated DOM tests verifying note creation, typing, line counting, and Markdown preview. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 26.1.1

