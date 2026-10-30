# Subphase 29.1.1: Fix Word Typing, Cursor Focus, Multi-Page Creation & Draggable/Resizable Shapes
- **Stage:** 29 | **Phase:** 29.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Smooth uninterrupted typing in Word pages, + New Page / Delete Page, and properly sized draggable floating shapes/images/tables.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. **Fix Word Typing & Cursor Loss Bug:**
   - Fix the React/DOM controlled-input bug that prevented typing inside Word pages! Each page's .word-page-body must use an uncontrolled contentEditable="true" ref (synchronized via onInput without overwriting innerHTML while the element is focused) AND support programmatic insertion at the saved Selection range so users can type continuously at 120fps, press Enter, Backspace, Delete, Tab, and select text freely.
2. **Fix Giant Blue Shapes Bug (Seen in image_25f41c.png):**
   - In image_25f41c.png, inserting shapes created massive full-width blue rectangles/ovals stacked across the entire page!
   - Fix Insert -> Shape and Insert -> Picture so shapes insert at a sensible default size (140px Ã— 90px), with an interactive selection border, **drag-to-move**, **8-corner resize handles**, **color picker**, **text inside shape**, and **Delete (Del key or trash icon)**.
3. **Working Multi-Page Creation (+ New Page, Page Break, Auto-Overflow):**
   - Add a floating **+ Add Page** button between/below pages AND wire Insert -> Blank Page and Insert -> Page Break (Ctrl+Enter) to immediately create Page 2, Page 3, etc., and scroll/focus the new page.
   - Allow deleting any extra page via a Delete Page button on the page header/ribbon.
4. Write automated DOM tests simulating continuous typing, creating 3 pages, inserting/resizing a shape, and deleting it. Commit with [GEC-Verified].
```
