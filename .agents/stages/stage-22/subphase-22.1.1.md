# Subphase 22.1.1: Blank Editable Word Pages, Rich Typing, Selection & New Page Creation
- **Stage:** 22 | **Phase:** 22.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% blank starting page where users can type, edit, delete, format rich text, and create/delete multiple pages.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 22.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Word (Writer) page editor component (pps/unified-shell/src/apps/writer/) so it is **immediately editable**:
   - Every new .docx tab starts with **1 clean, completely blank A4/Letter page** (zero prepopulated text!) focused automatically so the user can start typing immediately.
   - Use a synchronized rich-text contentEditable page array backed by the Rust word-engine state so typing, Backspace, Delete, Enter, Cut/Copy/Paste, Undo (Ctrl+Z), and Redo (Ctrl+Y) work with zero lag.
2. **Working Multi-Page Creation & Management:**
   - Add a **+ New Page** button on the Home and Insert ribbons (and at the bottom of the last page), plus Ctrl+Enter shortcut, that immediately appends a new editable page sheet (Page 2 of 2, Page 3 of 3) and focuses it.
   - Add a **Delete Empty Page** action and automatic page overflow splitting when text on a page exceeds the page height.
   - Make the Header and Footer areas at the top/bottom of each page editable on click.
3. Write automated DOM tests proving a new Word tab has empty text, accepts typed input, and creates Page 2 when + New Page is clicked. Commit with [GEC-Verified].
```
