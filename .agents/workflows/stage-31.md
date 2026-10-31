description: Automated execution workflow for Stage 31: PowerPoint (.pptx) Fix - Real .pptx Open/Save, Expanded Ribbons, Animations and Presenter View
Workflow: Execute Stage 31 (/stage-31)
Objective: Stage 31: PowerPoint (.pptx) Fix - Real .pptx Open/Save, Expanded Ribbons, Animations and Presenter View

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 31.1.1: PowerPoint Real .pptx Open/Save & Expanded Home, Insert, Design, Transitions Ribbons
- Spec File: .agents/stages/stage-31/subphase-31.1.1.md
/execute-subphase 31.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.3.
1. **Fix Opening & Saving Real .pptx Files:**
   - Wire File -> Open and the top ðŸ“‚ Open button in PowerPoint so selecting an existing .pptx file unzips ppt/slides/slide*.xml (using ooxml-opc / JSZip), parses all slide text boxes, shapes, colors, and positions, and loads them into the editable slide deck!
   - Wire File -> Save / ðŸ’¾ Save (Ctrl+S) to package all current slides, shapes, text, themes, and notes into a valid downloadable/saveable .pptx archive!
2. **Fix Ribbon Overflow & Expand Missing PowerPoint Controls (Home, Insert, Design, Transitions):**
   - **Home Tab:** + New Slide (with 6 layouts), Duplicate Slide, Delete Slide, Move Up/Down, Reset Layout, Clipboard (Cut, Copy, Paste), Font (Family, Size, A+, A-, Bold, Italic, Underline, Strikethrough, Text Shadow, Text Color, Highlight, Clear), Paragraph (Bullets, Numbering, Left/Center/Right/Justify, Top/Middle/Bottom vertical align, Line Spacing, Columns), Drawing & Arrange (Text Box, 12 Quick Shapes, Shape Fill, Shape Outline, Border Weight, Bring to Front, Send to Back, Align Left/Center/Right/Top/Middle/Bottom, Rotate 90Â°, Duplicate Shape, Delete Shape), Find & Replace.
   - **Insert Tab:** New Slide, Table (inserts editable table on slide), Pictures (upload local image onto slide), Shapes (full 20-shape gallery), Icons & Badges, SmartArt (inserts editable Process, Cycle, Hierarchy, Pyramid diagrams), Chart (inserts editable Bar, Line, Pie chart on slide), Text Box, Header & Footer, WordArt, Date & Time, Slide Number, Equation / Symbol, Audio / Video Clip.
   - **Design Tab:** 12 curated High-Contrast Presentation Themes (with live preview thumbnails), Color Variants, Slide Size (16:9 Widescreen vs 4:3 Standard), and Format Background (Solid Color, Gradient, Pattern, Image, Apply to All Slides).
   - **Transitions Tab:** 12 Slide Transitions (None, Fade, Push, Wipe, Split, Reveal, Cut, Cover, Flash, Zoom, Flip 3D, Morph), Effect Direction, Duration, Sound, Advance On Click / After X sec, Apply to All, and Preview (plays transition immediately on canvas).
3. Write automated tests verifying .pptx Open/Save and every control in Home, Insert, Design, and Transitions. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 31.1.1

### Subphase 31.1.2: PowerPoint Animations, Slide Show, Presenter Console, Review & View Ribbons
- Spec File: .agents/stages/stage-31/subphase-31.1.2.md
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Wire **100% of the controls** in PowerPoint's Animations, Slide Show, Review, and View ribbons:
   - **Animations Tab:** Entrance (Appear, Fade In, Fly In, Float In, Split, Wipe, Shape, Wheel, Random Bars, Grow & Turn, Zoom, Swivel, Bounce), Emphasis (Pulse, Color Pulse, Teeter, Spin, Grow/Shrink), Exit (Disappear, Fade Out, Fly Out, Float Out, Shrink & Turn), Effect Options, Add Animation, Animation Pane (right sidebar showing ordered animation steps per slide with Play, Move Up/Down, Delete), Trigger (On Click / With Previous / After Previous), Duration, Delay, and Preview Animation.
   - **Slide Show Tab:** From Beginning (F5), From Current Slide (Shift+F5), Presenter View (launches split presenter console with current slide, next slide, speaker notes, live timer, laser pointer, and freehand slide pen!), Custom Slide Show, Set Up Slide Show (Loop continuously, Kiosk mode), Hide Slide, Rehearse Timings (records per-slide rehearsal clock), Record Slide Show.
   - **Review & View Tabs:** Spelling, Thesaurus, New Comment (attaches comment pins to slide coordinates + right Comments drawer), Delete/Prev/Next Comment, Compare, Normal View, Outline View (edit all slide titles/bullets in a continuous document tree), Slide Sorter View (grid of all slides with drag-to-reorder), Notes Page View, Reading View, Ruler, Gridlines, Guides (center alignment crosshairs), Speaker Notes bottom panel, Color / Grayscale / Black & White mode, and Zoom slider.
2. Verify all buttons across Animations, Slide Show, Review, and View via automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 31.1.2

