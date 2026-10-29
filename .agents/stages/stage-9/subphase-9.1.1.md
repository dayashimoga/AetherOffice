# Subphase 9.1.1: Interactive Paginated Word Canvas, Real Caret & Keyboard Typing
- **Stage:** 9 | **Phase:** 9.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Live editable A4/Letter pages where typing, Backspace, Enter, Arrow keys, and mouse click/drag text selection work seamlessly.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 9.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the static non-editable text block in Writer (pps/unified-shell/src/apps/writer/) with a true interactive multi-page document editor synchronized with word-engine:
   - Support full keyboard input (keydown, InputEvent, IME composition, Backspace, Delete, Enter for new paragraph, Shift+Enter for line break, Arrow/Home/End/PageUp/PageDown cursor navigation).
   - Support mouse click to place the blinking caret at the exact character offset, double-click to select word, triple-click to select paragraph, and click-drag to highlight text ranges across runs and paragraphs.
2. Ensure every edit dispatches UiActionEvent to word-engine and immediately re-renders the updated paragraphs on screen.
3. Write automated DOM tests simulating typing a 3-paragraph document, selecting text, and deleting characters. Commit with [GEC-Verified].
`
