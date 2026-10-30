---
description: Automated execution workflow for Stage 31: PowerPoint (.pptx) Complete Fix - All 8 Ribbons, SmartArt, Animations and F5 Show
---

# Workflow: Execute Stage 31 (/stage-31)

## Objective: Stage 31: PowerPoint (.pptx) Complete Fix - All 8 Ribbons, SmartArt, Animations and F5 Show
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

### Subphase 31.1.1: PowerPoint Home, Insert, Design & Transitions Ribbons (100% Working)
- **Spec File:** `.agents/stages/stage-31/subphase-31.1.1.md`
/execute-subphase 31.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in PowerPoint's Home, Insert, Design, and Transitions ribbons:
   - **Home Tab:** + New Slide (with 6 Layout templates: Title Slide, Title & Content, Section Header, Two Content, Title Only, Blank), Duplicate Slide, Delete Slide, Reset Slide, Font/Size/Bold/Italic/Underline/Shadow/Color, Bullet/Numbered Lists inside text boxes, Align Left/Center/Right & Top/Middle/Bottom, Quick Shapes gallery, Arrange (Bring to Front, Send to Back, Align Center, Align Middle, Distribute Horizontally/Vertically, Group/Ungroup), Shape Fill, Shape Outline, Shape Effects (Drop Shadow, Glow), and Find & Replace.
   - **Insert Tab:** New Slide, Table (inserts editable slide table), Picture (upload image onto slide), Shapes (16 vector shapes including arrows, stars, banners, callouts, flowcharts), Icons / Stickers, SmartArt Diagram (inserts editable Process, Cycle, Hierarchy, or Pyramid diagram!), Chart (inserts editable Bar, Line, or Pie chart onto the slide), Text Box, Header & Footer, WordArt, Slide Number, and Audio / Video embed.
   - **Design Tab:** 10 visual Presentation Themes (instantly updates slide background, typography, and accent colors across the deck), Variants color palettes, Slide Size (Standard 4:3 vs Widescreen 16:9 â€” visibly changes stage aspect ratio!), and Format Background (Solid Color, Linear Gradient, Radial Gradient, Pattern, Apply to All).
   - **Transitions Tab:** None, Fade, Push, Wipe, Split, Reveal, Random Bars, Shape Circle, Zoom, Morph (interpolates matching shapes across slides!), Effect Options (direction), Transition Sound, Duration (s), Apply to All, Advance Slide After X sec, and Preview Transition (plays live CSS/WAAPI transition animation on the canvas!).
2. Verify every button in Home, Insert, Design, and Transitions via automated DOM tests. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 31.1.1`

### Subphase 31.1.2: PowerPoint Animations, Slide Show, Review & View Ribbons (100% Working)
- **Spec File:** `.agents/stages/stage-31/subphase-31.1.2.md`
/execute-subphase 31.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in PowerPoint's Animations, Slide Show, Review, and View ribbons:
   - **Animations Tab:** Entrance (Appear, Fade In, Fly In, Float In, Split, Wipe, Shape, Wheel, Bounce, Zoom), Emphasis (Pulse, Teeter, Spin, Grow/Shrink), Exit (Disappear, Fade Out, Fly Out, Zoom Out), Preview (plays the animation on the selected shape immediately!), Animation Pane (right sidebar listing queued animations with reorder & delete), Trigger (On Click, With Previous, After Previous), Duration, and Delay.
   - **Slide Show Tab:** Make every button work: Beginning (F5) (starts fullscreen slideshow from Slide 1, skipping [Hidden] slides!), Current (Shift+F5) (starts from current slide), Presenter View (opens dual-pane Presenter console showing Current Slide, Next Slide preview, Speaker Notes, live elapsed Timer, and On-Slide Laser Pointer / Ink Pen!), Hide Slide (toggles [Hidden] state on current slide), Rehearse Timings (launches live slide timing recorder), and Set Up Show (Loop continuously option).
   - **Review & View Tabs:** Spell Check, New Comment (slide comment pins + sidebar), Compare, Normal View, Outline View (editable text outline of all slides), Slide Sorter View (full-screen grid of all slide thumbnails for fast drag-reordering!), Notes Page View, Ruler, Gridlines (alignment grid on slide), Guides (center crosshair guides), Notes toggle (bottom speaker notes drawer), Grayscale / Black & White preview mode, and Zoom controls.
2. Verify all Animations, Slide Show, Review, and View features with automated DOM tests. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 31.1.2`

