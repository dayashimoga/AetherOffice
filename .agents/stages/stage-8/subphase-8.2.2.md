# Subphase 8.2.2: Strict Per-App Contextual Ribbon Router & Mobile Bar Cleanup
- **Stage:** 8 | **Phase:** 8.2
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Dedicated Ribbon schemas per app, fixed active-tab underline indicator, and removal of mobile bottom bar on desktop viewports.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 8.2.2
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.2.
1. Fix the Ribbon Tab state bug seen in screenshots where File stays underlined while Home is shown:
   - Maintain a strict ctiveRibbonTab state per window (Home, Insert, Layout, etc.) where clicking any tab updates the blue active indicator AND immediately switches the toolbar panel below it.
2. Separate the Ribbon definitions into 4 isolated components (WriterRibbon.tsx, CalcRibbon.tsx, SlidesRibbon.tsx, PdfRibbon.tsx):
   - **Writer ONLY:** Home, Insert, Layout, References, Review, View.
   - **Calc ONLY:** Home, Insert, Page Layout, Formulas, Data, Review, View.
   - **Slides ONLY:** Home, Insert, Design, Transitions, Animations, Slide Show, Review, View (Remove Word's Normal/Heading 1 styles from Slides!).
   - **PDF ONLY:** Home, Edit PDF, Forms, Comment & Annotate, Page Tools, Sign & Protect, View.
3. Hide the mobile bottom drawer handle and bottom touch bar whenever window.innerWidth >= 768px.
4. Verify with DOM tests asserting every tab click renders unique, non-placeholder buttons. Commit with [GEC-Verified].
`
