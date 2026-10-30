# Subphase 24.1.1: Blank Editable Slides, + New Slide, Drag/Resize/Rotate & Inline Text Editing
- **Stage:** 24 | **Phase:** 24.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Clean blank presentation where users can add/delete slides, type directly into slide boxes, and drag/resize/rotate shapes.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 24.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the PowerPoint (Slides) workspace (pps/unified-shell/src/apps/slides/) so it starts with **1 clean blank Title Slide** (with editable "Click to add title" and "Click to add subtitle" placeholdersâ€”zero hardcoded stars or fake demo text!):
   - **Inline Text Editing:** Clicking any text box or shape lets the user immediately type, delete, and format text directly on the 16:9 slide canvas.
   - **Interactive Drag, Resize & Rotate:** Selecting any object shows an interactive bounding frame with **8 resize handles** and a **rotation handle**. Dragging moves the element smoothly; dragging handles resizes or rotates it; pressing Delete removes the selected element.
   - **Slide Thumbnail Rail:** Working **+ New Slide** button (with layout choices: Title Slide, Title & Content, Two Column, Blank), Duplicate Slide, Delete Slide, and Move Up/Down buttons in the left sidebar.
2. Write automated DOM tests verifying + New Slide, typing text into a slide box, and inserting/resizing a shape. Commit with [GEC-Verified].
```
