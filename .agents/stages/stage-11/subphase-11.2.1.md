# Subphase 11.2.1: Slides Transitions, Animations & Fullscreen Slide Show (F5) Engine
- **Stage:** 11 | **Phase:** 11.2
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Working Transitions tab (Fade, Push, Wipe, Morph), Animations tab (Entrance/Emphasis/Exit + Animation Pane), and Fullscreen Presenter Show.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 11.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the dead "SlideShow commands for PowerPoint" placeholder seen in image_e63126.png with complete Transitions, Animations, and Slide Show ribbons:
   - **Transitions Tab:** Transition gallery (None, Fade, Push, Wipe, Split, Morph), Effect Options, Duration spinner, Apply to All Slides button, and Preview button that plays the transition right on the stage.
   - **Animations Tab:** Entrance effects (Appear, Fade In, Fly In, Zoom, Bounce), Emphasis (Pulse, Spin), Exit (Fade Out, Fly Out), Trigger (On Click, With Previous, After Previous), and collapsible **Animation Timeline Pane** on the right.
   - **Slide Show Tab:** From Beginning (F5), From Current Slide (Shift+F5), Presenter View (with timer & speaker notes), and fullscreen interactive presentation player where clicking or pressing Right Arrow / Space steps through shape animations and slide transitions!
2. Verify fullscreen player and animation triggers via automated DOM tests. Commit with [GEC-Verified].
`
