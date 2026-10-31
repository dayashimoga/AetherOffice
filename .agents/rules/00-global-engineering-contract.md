---
trigger: always_on
description: Global Engineering Contract (GEC-v5.0) enforcing Production-Grade UI/UX, Real Binary File I/O, Multi-Instance Per-Format Tabs + Split View, Full Row/Col Excel Selection, Real Drawn/Typed PDF Signatures, and 100% Automated Ribbon Testing.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v5.0 â€” PRODUCTION READINESS)

## ARTICLE I: MULTI-DOCUMENT WORKSPACE, APP GROUPING & SPLIT-VIEW ARCHITECTURE
1. **Unlimited Simultaneous Documents Per Format:**
   - Users MUST be able to open and edit multiple `.docx` files, multiple `.xlsx` files, multiple `.pptx` files, multiple `.pdf` files, and multiple `.txt/.md` files simultaneously.
   - Clicking an app icon on the Left App Rail (`W`, `X`, `P`, `PDF`, `N`) filters the workspace to show all open tabs of that format (or `All` mode to see all tabs), while every app rail icon also has a `+` action and `Open File` action to spawn additional documents of that type at any time.
   - Include a **Side-by-Side Split View (`â—« Split Screen`)** toggle in the top bar so users can view and edit any two open documents side-by-side in the same window, PLUS `â†— Pop Out Window` to detach any tab into a separate OS window.

## ARTICLE II: VISUAL OVERHAUL, ZERO OVERFLOW & WCAG HIGH-CONTRAST THEME
1. **Modern Glass/Fluent UI Overhaul & Zero Ribbon Overflow:**
   - The Ribbon toolbar MUST NEVER clip or cut off buttons on the right edge (`Titl`, `Subtota`). Use a responsive wrapping layout (`flex-wrap: wrap`) or compact adaptive groups with horizontal scroll buttons so 100% of tools are visible and clickable at any window width.
2. **Complete Dark & Light Theme with High-Contrast Text:**
   - Toggling Dark/Light theme MUST style the entire viewport including the canvas surround (`#090d16` in Dark, `#f1f5f9` in Light), sidebars, status bar, modals, and thumbnails.
   - **Contrast Invariant:** White cards (like slide thumbnails or white pages) MUST ALWAYS use dark text (`#0f172a`) inside the white card, while dark sidebars (`#1e293b`) MUST ALWAYS use bright text (`#f8fafc`). Never render light text on a white background!

## ARTICLE III: FORMAT-SPECIFIC PRODUCTION MANDATES
1. **Word (`.docx`):**
   - **Fix 100% Zoom Clipping & Scroll Flicker:** Never use `justify-content: center` on the vertical axis of the `.word-canvas-scroll` container! Use `display: flex; flex-direction: column; align-items: center; justify-content: flex-start; overflow-y: auto` and `transform-origin: top center` on pages so Page 1, Header, Footer, and line 1 of text are 100% visible at `100%` zoom, and scrolling never flickers or jumps back to the top.
   - Real `.docx` file opening (parsing XML `w:p`/`w:r`/`w:tbl` inside the ZIP package) and saving.
2. **Excel (`.xlsx`):**
   - **Full Row & Column Selection:** Clicking column header `B` selects all of Column B; dragging `B` to `E` selects Columns B..E; `Ctrl+Click` selects non-adjacent columns/rows. Clicking row header `5` or dragging `5` to `12` selects entire rows. Any formatting (Bold, Fill, Font, Size, Align, Number Format, Delete, Clear) applied while rows/columns are selected MUST apply to all cells in those rows/columns!
3. **PowerPoint (`.pptx`):**
   - Real `.pptx` file opening (reading slide XMLs from ZIP) and saving, plus complete Slide Show, Transitions, Animations, Shape Alignment, SmartArt, Tables, and Media tools.
4. **PDF Studio (`.pdf`):**
   - **Ban on Hardcoded `"John Doe"` Signature:** `"John Doe"` is STRICTLY BANNED from `apps/`. Clicking `Sign` or `Add Signature` MUST open an interactive Signature Pad Modal where the user draws their real signature with mouse/touch OR types their own custom name/initials, and stamps that exact image/text onto the PDF page.
   - Real local `.pdf` file opening (rendering pages and extracting text boxes), real saving, password encryption/decryption, and page size resizing.
5. **Notepad (`.txt` / `.md`):**
   - Complete menu bar (File, Edit, Search, View, Encoding, Text Tools, Markdown, Macros) with 100% working commands and automated tests.
