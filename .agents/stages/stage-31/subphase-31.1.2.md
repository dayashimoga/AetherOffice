# Subphase 31.1.2: PowerPoint Animations, Slide Show, Review & View Ribbons (100% Working)
- **Stage:** 31 | **Phase:** 31.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Working Animations timeline, Slide Show (F5, Presenter View, Rehearse Timings, Laser/Pen), Review comments, and Slide Sorter/Notes Master views.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in PowerPoint's Animations, Slide Show, Review, and View ribbons:
   - **Animations Tab:** Entrance (Appear, Fade In, Fly In, Float In, Split, Wipe, Shape, Wheel, Bounce, Zoom), Emphasis (Pulse, Teeter, Spin, Grow/Shrink), Exit (Disappear, Fade Out, Fly Out, Zoom Out), Preview (plays the animation on the selected shape immediately!), Animation Pane (right sidebar listing queued animations with reorder & delete), Trigger (On Click, With Previous, After Previous), Duration, and Delay.
   - **Slide Show Tab:** Make every button work: Beginning (F5) (starts fullscreen slideshow from Slide 1, skipping [Hidden] slides!), Current (Shift+F5) (starts from current slide), Presenter View (opens dual-pane Presenter console showing Current Slide, Next Slide preview, Speaker Notes, live elapsed Timer, and On-Slide Laser Pointer / Ink Pen!), Hide Slide (toggles [Hidden] state on current slide), Rehearse Timings (launches live slide timing recorder), and Set Up Show (Loop continuously option).
   - **Review & View Tabs:** Spell Check, New Comment (slide comment pins + sidebar), Compare, Normal View, Outline View (editable text outline of all slides), Slide Sorter View (full-screen grid of all slide thumbnails for fast drag-reordering!), Notes Page View, Ruler, Gridlines (alignment grid on slide), Guides (center crosshair guides), Notes toggle (bottom speaker notes drawer), Grayscale / Black & White preview mode, and Zoom controls.
2. Verify all Animations, Slide Show, Review, and View features with automated DOM tests. Commit with [GEC-Verified].
```
