# Subphase 31.1.1: PowerPoint Home, Insert, Design & Transitions Ribbons (100% Working)
- **Stage:** 31 | **Phase:** 31.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Complete working PowerPoint Home, Insert (SmartArt, Charts, Media, Shapes, Icons), Design Themes, and Transitions.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 31.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in PowerPoint's Home, Insert, Design, and Transitions ribbons:
   - **Home Tab:** + New Slide (with 6 Layout templates: Title Slide, Title & Content, Section Header, Two Content, Title Only, Blank), Duplicate Slide, Delete Slide, Reset Slide, Font/Size/Bold/Italic/Underline/Shadow/Color, Bullet/Numbered Lists inside text boxes, Align Left/Center/Right & Top/Middle/Bottom, Quick Shapes gallery, Arrange (Bring to Front, Send to Back, Align Center, Align Middle, Distribute Horizontally/Vertically, Group/Ungroup), Shape Fill, Shape Outline, Shape Effects (Drop Shadow, Glow), and Find & Replace.
   - **Insert Tab:** New Slide, Table (inserts editable slide table), Picture (upload image onto slide), Shapes (16 vector shapes including arrows, stars, banners, callouts, flowcharts), Icons / Stickers, SmartArt Diagram (inserts editable Process, Cycle, Hierarchy, or Pyramid diagram!), Chart (inserts editable Bar, Line, or Pie chart onto the slide), Text Box, Header & Footer, WordArt, Slide Number, and Audio / Video embed.
   - **Design Tab:** 10 visual Presentation Themes (instantly updates slide background, typography, and accent colors across the deck), Variants color palettes, Slide Size (Standard 4:3 vs Widescreen 16:9 â€” visibly changes stage aspect ratio!), and Format Background (Solid Color, Linear Gradient, Radial Gradient, Pattern, Apply to All).
   - **Transitions Tab:** None, Fade, Push, Wipe, Split, Reveal, Random Bars, Shape Circle, Zoom, Morph (interpolates matching shapes across slides!), Effect Options (direction), Transition Sound, Duration (s), Apply to All, Advance Slide After X sec, and Preview Transition (plays live CSS/WAAPI transition animation on the canvas!).
2. Verify every button in Home, Insert, Design, and Transitions via automated DOM tests. Commit with [GEC-Verified].
```
