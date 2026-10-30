# Subphase 24.1.2: Working PowerPoint Ribbon: Shapes, Images, Themes, Transitions & Fullscreen Show
- **Stage:** 24 | **Phase:** 24.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Complete working PowerPoint ribbon across Home, Insert, Design, Transitions, Animations, and Fullscreen Slide Show.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 24.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the PowerPoint Ribbon:
   - **Home & Insert Tabs:** + New Slide, Insert Text Box, Insert Image (from file), Insert Table, Insert Shape (Rectangle, Rounded Rect, Circle, Triangle, Arrow, Star, Callout), Font/Size/Bold/Italic/Color/Alignment for selected shape text, Shape Fill Color, Shape Border Color/Width, Bring to Front, Send to Back, Duplicate, Delete.
   - **Design Tab:** 8 curated Slide Themes (Dark Slate, Ocean Blue, Emerald, Sunset, Minimal White, Executive Navy, Warm Sand, Cyber Neon) and custom Slide Background Color/Gradient picker.
   - **Transitions & Animations Tabs:** Slide Transition selector (None, Fade, Slide Left, Zoom, Flip), Object Animation selector (None, Fade In, Fly Up, Pop, Spin), and Preview Animation button.
   - **Slide Show Tab (F5):** Fullscreen Presentation Player overlay with animated slide transitions, Left/Right/Spacebar navigation, slide counter (1 / N), Laser Pointer cursor toggle, and Esc to exit.
   - **Speaker Notes Bar:** Collapsible bottom panel to type speaker notes for each slide.
2. Verify via automated DOM tests and commit with [GEC-Verified].
```
