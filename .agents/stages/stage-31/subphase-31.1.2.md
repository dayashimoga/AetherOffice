# Subphase 31.1.2: PowerPoint Animations, Slide Show, Presenter Console, Review & View Ribbons
- **Stage:** 31 | **Phase:** 31.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Animations timeline, Fullscreen Slide Show, Presenter View, Laser/Pen annotation on slides, Slide Sorter, and Notes.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Wire **100% of the controls** in PowerPoint's Animations, Slide Show, Review, and View ribbons:
   - **Animations Tab:** Entrance (Appear, Fade In, Fly In, Float In, Split, Wipe, Shape, Wheel, Random Bars, Grow & Turn, Zoom, Swivel, Bounce), Emphasis (Pulse, Color Pulse, Teeter, Spin, Grow/Shrink), Exit (Disappear, Fade Out, Fly Out, Float Out, Shrink & Turn), Effect Options, Add Animation, Animation Pane (right sidebar showing ordered animation steps per slide with Play, Move Up/Down, Delete), Trigger (On Click / With Previous / After Previous), Duration, Delay, and Preview Animation.
   - **Slide Show Tab:** From Beginning (F5), From Current Slide (Shift+F5), Presenter View (launches split presenter console with current slide, next slide, speaker notes, live timer, laser pointer, and freehand slide pen!), Custom Slide Show, Set Up Slide Show (Loop continuously, Kiosk mode), Hide Slide, Rehearse Timings (records per-slide rehearsal clock), Record Slide Show.
   - **Review & View Tabs:** Spelling, Thesaurus, New Comment (attaches comment pins to slide coordinates + right Comments drawer), Delete/Prev/Next Comment, Compare, Normal View, Outline View (edit all slide titles/bullets in a continuous document tree), Slide Sorter View (grid of all slides with drag-to-reorder), Notes Page View, Reading View, Ruler, Gridlines, Guides (center alignment crosshairs), Speaker Notes bottom panel, Color / Grayscale / Black & White mode, and Zoom slider.
2. Verify all buttons across Animations, Slide Show, Review, and View via automated DOM tests. Commit with [GEC-Verified].
```
