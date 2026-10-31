# Subphase 38.1.1: PowerPoint Save/Open (.pptx) Round-Trip & Complete 8-Tab Ribbon Functionality

- **Stage:** 38 | **Phase:** 38.1
- **Target Subsystem:** `ppt-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working PowerPoint File Open/Save (.pptx), slide layouts, shapes, SmartArt, charts, transitions, animations, and F5 Slide Show.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 38.1.1
Invoke @adaptive-ui-builder under GEC-v6.0 Article II.
1. **Fix PowerPoint Open & Save Existing `.pptx` Files:**
   - When clicking `Save` (`Ctrl+S`) or `File -> Save` in PowerPoint, serialize all slides, text boxes, shapes, colors, transitions, animations, and speaker notes into a valid `.pptx` package, save via `save_file_bytes` / download, and show the `"✅ Saved <filename>.pptx"` toast!
   - When clicking `Open` (`Ctrl+O`) and selecting an existing `.pptx` file, parse `ppt/slides/slide*.xml` (plus embedded deck metadata) so all slides, titles, bullet points, and shapes load directly onto the slide canvas and left thumbnail rail!
2. **Complete & Verify All 8 PowerPoint Ribbon Tabs (`Home`, `Insert`, `Design`, `Transitions`, `Animations`, `Slide Show`, `Review`, `View`):**
   - Fix the layout in `image_975970.jpg` where `Drawing` on row 2 of the Home ribbon had its shape buttons clipped: ensure all shape/arrange buttons (`Rect`, `Round`, `Oval`, `Triangle`, `Arrow`, `Star`, `Fill`, `Outline`, `Bring Front`, `Send Back`) are 100% visible!
   - Ensure `Insert` (`Table`, `Picture`, `Shapes`, `SmartArt`, `Chart`, `Text Box`, `WordArt`, `Slide Number`, `Audio/Video`), `Design` (12 Themes, `16:9 / 4:3`, Background color/gradient), `Transitions` (with live visual preview on the slide), `Animations` (with right-hand Animation Timeline Pane), `Slide Show` (`F5` Fullscreen player + Presenter View), `Review` (Comments), and `View` (`Normal`, `Outline View`, `Slide Sorter` grid, `Notes Page`, `Ruler`, `Gridlines`, `Guides`) all work 100%.
3. Write automated tests verifying `.pptx` Save -> Open round-tripping and all 8 PowerPoint ribbon tabs. Commit with `[GEC-Verified]`.
````
