# Subphase 29.1.1: Fix Word 100% Zoom Text Clipping, Fix Scroll Flicker/Jump, Visible Headers/Footers & + New Page
- **Stage:** 29 | **Phase:** 29.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Crisp visible text at 100% zoom, rock-solid smooth scrolling without jumping to top, visible editable Headers/Footers, and instant + New Page.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 29.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.1.
1. **Fix the 100% Zoom Invisible/Clipped Text Bug (Seen in image_b6a07e.png vs image_b64ac6.png):**
   - Look at why text was only visible at 20% zoom in image_b64ac6.png: the scroll parent had vertical centering (justify-content: center) or 	ransform: scale() without 	ransform-origin: top center, clipping the top half of the page off-screen at 100% zoom!
   - Fix .word-scroll-viewport: set display: block; overflow-y: auto; overflow-x: auto; padding: 32px 24px; position: relative; and center each .word-page-sheet horizontally with margin: 0 auto 32px auto; transform-origin: top center; (or scale width/padding/font-size proportionally by zoom / 100 so layout flow is 100% native and never clips!).
   - Ensure text inside .word-page-body has ont-size: 11pt (14.67px); color: #0f172a; line-height: 1.5; min-height: 860px; so typed characters are immediately large, dark, and crystal clear at 100% zoom!
2. **Fix the Scroll Flicker & Jump-to-Top Bug:**
   - Remove any key={...} on the scroll container or page list that changes on keystroke/scroll, and remove any useEffect calling scrollTo(0, 0) or element.focus({ preventScroll: false }) on re-render! When focusing a page programmatically, always pass { preventScroll: true } so scrolling down a multi-page document NEVER flickers or snaps back to the top.
3. **Visible, Always-Accessible Header, Footer & + Add New Page:**
   - Render a clearly delineated **Header Zone** (order-bottom: 1px dashed #cbd5e1, placeholder \"Click to edit header\") at the top of every page and a **Footer Zone** (order-top: 1px dashed #cbd5e1, placeholder \"Click to edit footer\", plus Page X of Y) at the bottom of every page.
   - Make **+ Page / Blank Page / + Blank Page / New Page** immediately append a new page object to doc.pages, scroll smoothly to the new page, and place the cursor inside the new page's body.
4. Write automated DOM tests verifying that at zoom = 100%, typed text is rendered inside the visible viewport (getBoundingClientRect().top >= 0), scrolling down preserves scrollTop, Header/Footer inputs are editable, and clicking + Page increases page count from 1 to 2. Commit with [GEC-Verified].
```
