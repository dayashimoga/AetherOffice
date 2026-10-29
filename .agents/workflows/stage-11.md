description: Automated execution workflow for Stage 11: Standalone Slides (PowerPoint) - Vector Manipulation, Animations & Fullscreen Show
Workflow: Execute Stage 11 (/stage-11)
Objective: Stage 11: Standalone Slides (PowerPoint) - Vector Manipulation, Animations & Fullscreen Show

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 11.1.1: Interactive Slide Canvas: Drag, 8-Point Resize, Rotate & In-Place Text Editing
Spec File: .agents/stages/stage-11/subphase-11.1.1.md
/execute-subphase 11.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the static, non-interactive slide canvas seen in image_cb4c9a.png:
   - **Selection & Transform Gizmos:** Clicking any text box or shape (like the title or star) renders a selection bounding box with **8 resize handles** (corners and midpoints) and a **top rotation handle**.
   - **Live Drag, Resize & Rotate:** Dragging a shape moves its (x, y) coordinates with smart center/edge alignment guides; dragging any handle resizes (w, h) or rotates ngle_deg in real time inside ppt-engine.
   - **In-Place Text Editing:** Double-clicking any shape or text box activates an inline rich-text editor inside the shape so users can type, edit, and format slide text directly on the canvas!
   - **Slide Management:** Make + Slide, Duplicate Slide, Delete Slide, Move Slide Up/Down, and Slide Layout picker (Title Slide, Title & Content, Two Content, Blank) work seamlessly.
2. Verify shape dragging, resizing, and inline text editing with automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 11.1.1
Subphase 11.1.2: Slides Home, Insert & Design Ribbons: Shapes, Images, Tables, Themes & Backgrounds
Spec File: .agents/stages/stage-11/subphase-11.1.2.md
/execute-subphase 11.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire all controls in SlidesRibbon.tsx (Home, Insert, and Design tabs):
   - **Home & Insert Tabs:** New Slide (with layout dropdown), Insert Text Box, Insert Image, Insert Table, Insert Shape (Rectangle, Rounded Rect, Ellipse, Triangle, Right Arrow, 5-Point Star, Callout, Diamond, Heart), Arrange (Bring to Front, Send to Back, Align Left/Center/Middle, Duplicate, Delete), Shape Fill color picker, Shape Outline color/weight picker, and Speaker Notes pane at the bottom of the stage.
   - **Design Tab:** 8 Built-in Presentation Themes (updating Master background, title font, and accent palette across all slides), Slide Size (16:9 Widescreen vs 4:3 Standard), and Format Background (Solid Color / Gradient).
2. Verify with automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 11.1.2
Subphase 11.2.1: Slides Transitions, Animations & Fullscreen Slide Show (F5) Engine
Spec File: .agents/stages/stage-11/subphase-11.2.1.md
/execute-subphase 11.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the dead "SlideShow commands for PowerPoint" placeholder seen in image_e63126.png with complete Transitions, Animations, and Slide Show ribbons:
   - **Transitions Tab:** Transition gallery (None, Fade, Push, Wipe, Split, Morph), Effect Options, Duration spinner, Apply to All Slides button, and Preview button that plays the transition right on the stage.
   - **Animations Tab:** Entrance effects (Appear, Fade In, Fly In, Zoom, Bounce), Emphasis (Pulse, Spin), Exit (Fade Out, Fly Out), Trigger (On Click, With Previous, After Previous), and collapsible **Animation Timeline Pane** on the right.
   - **Slide Show Tab:** From Beginning (F5), From Current Slide (Shift+F5), Presenter View (with timer & speaker notes), and fullscreen interactive presentation player where clicking or pressing Right Arrow / Space steps through shape animations and slide transitions!
2. Verify fullscreen player and animation triggers via automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 11.2.1
