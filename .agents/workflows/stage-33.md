description: Automated execution workflow for Stage 33: Notepad / Notes (.txt/.md) Fix - 100% Default Zoom, Text Power Tools and Complete Ribbons
Workflow: Execute Stage 33 (/stage-33)
Objective: Stage 33: Notepad / Notes (.txt/.md) Fix - 100% Default Zoom, Text Power Tools and Complete Ribbons

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 33.1.1: Notepad / Notes Complete Ribbon Overhaul: File, Home, Edit, View, Markdown & Power Tools
- Spec File: .agents/stages/stage-33/subphase-33.1.1.md
/execute-subphase 33.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.5.
1. Expand and fix **100% of the menu options** in the **Notepad / Notes (N)** workspace (image_b642e4.png):
   - **Fix Zoom Default Bug:** In image_b642e4.png, the bottom-right zoom slider says 20% by default! Ensure all 5 apps (Word, Excel, PPT, PDF, Notes) initialize at **100% zoom** by default!
   - **Home Tab:** New Note (+ Scratchpad), Open (.txt, .md, .json, .xml, .csv, .log, .ini, .yaml), Save (.txt), Save (.md), Rename Note, Delete Note, Print, Font Family (Consolas, JetBrains Mono, Courier New, Inter, System UI), Font Size (10pxâ€“32px), Zoom In / Out / 100%, Time/Date (F5).
   - **Edit Tab:** Undo, Redo, Cut, Copy, Paste, Delete, Select All, Find (Ctrl+F interactive search bar with match counter & Next/Prev), Replace (Ctrl+H with Replace & Replace All, Case-Sensitive, and Regex mode), Go To Line (Ctrl+G), Duplicate Line (Ctrl+D), Delete Line (Ctrl+Shift+K), Move Line Up/Down.
   - **Text Tools / Transform Tab (New Pro Features):** UPPERCASE, lowercase, Title Case, Sentence case, Invert Case, Sort Lines Aâ†’Z, Sort Lines Zâ†’A, Remove Duplicate Lines, Remove Empty Lines, Trim Leading/Trailing Whitespace, Format / Prettify JSON, Minify JSON, Base64 Encode / Decode, URL Encode / Decode, Word & Frequency Statistics.
   - **View & Markdown Tabs:** Editor Only, Split View (Editor + Live Preview), Preview Only, Line Numbers On/Off, Word Wrap On/Off, Show Whitespace, Line Endings (CRLF / LF), Encoding (UTF-8 / UTF-16 / ASCII), and working Markdown formatting buttons (H1, H2, H3, Bold, Italic, Strikethrough, Inline Code, Code Block, Blockquote, Bullet List, Numbered List, Task Checkbox - [ ], Insert Table, Insert Link, Insert Image, Horizontal Rule).
2. Verify all Notepad/Notes buttons, text transforms, JSON formatter, Find & Replace, and file I/O via automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 33.1.1

