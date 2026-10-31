description: Automated execution workflow for Stage 38: PowerPoint (.pptx) Fix - Working File Save/Open (.pptx), Unclipped Shapes & Complete 8-Tab Ribbon
Workflow: Execute Stage 38 (/stage-38)
Objective: Stage 38: PowerPoint (.pptx) Fix - Working File Save/Open (.pptx), Unclipped Shapes & Complete 8-Tab Ribbon

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v6.0) compliance:

Subphase 38.1.1: PowerPoint Save/Open (.pptx) Round-Trip & Complete 8-Tab Ribbon Functionality
Spec File: .agents/stages/stage-38/subphase-38.1.1.md
/execute-subphase 38.1.1
Invoke @adaptive-ui-builder under GEC-v6.0 Article II.
1. **Fix PowerPoint Open & Save Existing `.pptx` Files:**
   - When clicking `Save` (`Ctrl+S`) or `File -> Save` in PowerPoint, serialize all slides, text boxes, shapes, colors, transitions, animations, and speaker notes into a valid `.pptx` package, save via `save_file_bytes` / download, and show the `"✅ Saved <filename>.pptx"` toast!
   - When clicking `Open` (`Ctrl+O`) and selecting an existing `.pptx` file, parse `ppt/slides/slide*.xml` (plus embedded deck metadata) so all slides, titles, bullet points, and shapes load directly onto the slide canvas and left thumbnail rail!
2. **Complete & Verify All 8 PowerPoint Ribbon Tabs (`Home`, `Insert`, `Design`, `Transitions`, `Animations`, `Slide Show`, `Review`, `View`):**
   - Fix the layout in `image_975970.jpg` where `Drawing` on row 2 of the Home ribbon had its shape buttons clipped: ensure all shape/arrange buttons (`Rect`, `Round`, `Oval`, `Triangle`, `Arrow`, `Star`, `Fill`, `Outline`, `Bring Front`, `Send Back`) are 100% visible!
   - Ensure `Insert` (`Table`, `Picture`, `Shapes`, `SmartArt`, `Chart`, `Text Box`, `WordArt`, `Slide Number`, `Audio/Video`), `Design` (12 Themes, `16:9 / 4:3`, Background color/gradient), `Transitions` (with live visual preview on the slide), `Animations` (with right-hand Animation Timeline Pane), `Slide Show` (`F5` Fullscreen player + Presenter View), `Review` (Comments), and `View` (`Normal`, `Outline View`, `Slide Sorter` grid, `Notes Page`, `Ruler`, `Gridlines`, `Guides`) all work 100%.
3. Write automated tests verifying `.pptx` Save -> Open round-tripping and all 8 PowerPoint ribbon tabs. Commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 38.1.1
