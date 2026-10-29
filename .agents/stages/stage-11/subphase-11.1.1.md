# Subphase 11.1.1: Interactive Slide Canvas: Drag, 8-Point Resize, Rotate & In-Place Text Editing
- **Stage:** 11 | **Phase:** 11.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Full interactive manipulation of slide text boxes, shapes, and images with selection handles, snapping guides, and live text editing.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 11.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the static, non-interactive slide canvas seen in image_cb4c9a.png:
   - **Selection & Transform Gizmos:** Clicking any text box or shape (like the title or star) renders a selection bounding box with **8 resize handles** (corners and midpoints) and a **top rotation handle**.
   - **Live Drag, Resize & Rotate:** Dragging a shape moves its (x, y) coordinates with smart center/edge alignment guides; dragging any handle resizes (w, h) or rotates ngle_deg in real time inside ppt-engine.
   - **In-Place Text Editing:** Double-clicking any shape or text box activates an inline rich-text editor inside the shape so users can type, edit, and format slide text directly on the canvas!
   - **Slide Management:** Make + Slide, Duplicate Slide, Delete Slide, Move Slide Up/Down, and Slide Layout picker (Title Slide, Title & Content, Two Content, Blank) work seamlessly.
2. Verify shape dragging, resizing, and inline text editing with automated DOM tests. Commit with [GEC-Verified].
`
