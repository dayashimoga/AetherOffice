# Subphase 31.1.1: PowerPoint Real .pptx Open/Save & Expanded Home, Insert, Design, Transitions Ribbons
- **Stage:** 31 | **Phase:** 31.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Real .pptx file Open/Save, non-overflowing rich ribbon, SmartArt, Charts, Tables, Images, Align/Distribute/Group, and Design Themes.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
