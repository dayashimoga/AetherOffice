# Subphase 11.1.2: Slides Home, Insert & Design Ribbons: Shapes, Images, Tables, Themes & Backgrounds
- **Stage:** 11 | **Phase:** 11.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Working Shape/Text Box/Image/Table insertion, Z-Order (Bring to Front/Send to Back), Shape Fill/Stroke, and Design Theme gallery.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 11.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire all controls in SlidesRibbon.tsx (Home, Insert, and Design tabs):
   - **Home & Insert Tabs:** New Slide (with layout dropdown), Insert Text Box, Insert Image, Insert Table, Insert Shape (Rectangle, Rounded Rect, Ellipse, Triangle, Right Arrow, 5-Point Star, Callout, Diamond, Heart), Arrange (Bring to Front, Send to Back, Align Left/Center/Middle, Duplicate, Delete), Shape Fill color picker, Shape Outline color/weight picker, and Speaker Notes pane at the bottom of the stage.
   - **Design Tab:** 8 Built-in Presentation Themes (updating Master background, title font, and accent palette across all slides), Slide Size (16:9 Widescreen vs 4:3 Standard), and Format Background (Solid Color / Gradient).
2. Verify with automated DOM tests and commit with [GEC-Verified].
`
