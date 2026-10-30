description: Automated execution workflow for Stage 24: PowerPoint (Slides) - Blank Slides, Shape Editor & Slide Show
Workflow: Execute Stage 24 (/stage-24)
Objective: Stage 24: PowerPoint (Slides) - Blank Slides, Shape Editor & Slide Show

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 24.1.1: Blank Editable Slides, + New Slide, Drag/Resize/Rotate & Inline Text Editing
Spec File: .agents/stages/stage-24/subphase-24.1.1.md
/execute-subphase 24.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the PowerPoint (Slides) workspace (pps/unified-shell/src/apps/slides/) so it starts with **1 clean blank Title Slide** (with editable "Click to add title" and "Click to add subtitle" placeholdersâ€”zero hardcoded stars or fake demo text!):
   - **Inline Text Editing:** Clicking any text box or shape lets the user immediately type, delete, and format text directly on the 16:9 slide canvas.
   - **Interactive Drag, Resize & Rotate:** Selecting any object shows an interactive bounding frame with **8 resize handles** and a **rotation handle**. Dragging moves the element smoothly; dragging handles resizes or rotates it; pressing Delete removes the selected element.
   - **Slide Thumbnail Rail:** Working **+ New Slide** button (with layout choices: Title Slide, Title & Content, Two Column, Blank), Duplicate Slide, Delete Slide, and Move Up/Down buttons in the left sidebar.
2. Write automated DOM tests verifying + New Slide, typing text into a slide box, and inserting/resizing a shape. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 24.1.1
Subphase 24.1.2: Working PowerPoint Ribbon: Shapes, Images, Themes, Transitions & Fullscreen Show
Spec File: .agents/stages/stage-24/subphase-24.1.2.md
/execute-subphase 24.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the PowerPoint Ribbon:
   - **Home & Insert Tabs:** + New Slide, Insert Text Box, Insert Image (from file), Insert Table, Insert Shape (Rectangle, Rounded Rect, Circle, Triangle, Arrow, Star, Callout), Font/Size/Bold/Italic/Color/Alignment for selected shape text, Shape Fill Color, Shape Border Color/Width, Bring to Front, Send to Back, Duplicate, Delete.
   - **Design Tab:** 8 curated Slide Themes (Dark Slate, Ocean Blue, Emerald, Sunset, Minimal White, Executive Navy, Warm Sand, Cyber Neon) and custom Slide Background Color/Gradient picker.
   - **Transitions & Animations Tabs:** Slide Transition selector (None, Fade, Slide Left, Zoom, Flip), Object Animation selector (None, Fade In, Fly Up, Pop, Spin), and Preview Animation button.
   - **Slide Show Tab (F5):** Fullscreen Presentation Player overlay with animated slide transitions, Left/Right/Spacebar navigation, slide counter (1 / N), Laser Pointer cursor toggle, and Esc to exit.
   - **Speaker Notes Bar:** Collapsible bottom panel to type speaker notes for each slide.
2. Verify via automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 24.1.2

