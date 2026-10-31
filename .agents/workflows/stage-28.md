description: Automated execution workflow for Stage 28: Modern Visual Overhaul, Non-Overflowing Ribbon, High-Contrast Dark Mode and Multi-Document Split View
Workflow: Execute Stage 28 (/stage-28)
Objective: Stage 28: Modern Visual Overhaul, Non-Overflowing Ribbon, High-Contrast Dark Mode and Multi-Document Split View

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 28.1.1: Modern Fluent/Linear Visual Overhaul, Full-App Dark Theme & High-Contrast Fix
- Spec File: .agents/stages/stage-28/subphase-28.1.1.md
/execute-subphase 28.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article II.
1. **Modern Visual Design Overhaul:**
   - Upgrade the entire application shell (pps/unified-shell/src/) with a sleek, modern design system (crisp Inter / Segoe UI Variable typography, refined 6px border-radius buttons with subtle hover/active states, active state pill indicators, clean SVG iconography, and modern elevation shadows).
2. **Fix Ribbon Overflow & Clipped Right-Side Controls (Seen in image_b6a07e.png & image_b64b07.png):**
   - Fix the Ribbon container so controls on the right (Styles, Subtotal, Drawing & Arrange) are NEVER clipped off-screen!
   - Give .ribbon-content display: flex; flex-wrap: wrap; gap: 6px; padding: 6px 12px; height: auto; min-height: 76px; max-height: 136px; overflow-y: visible; AND compact group spacing so every single button and dropdown wraps cleanly into view on any screen width.
3. **Fix Partial Dark Mode & Invisible Slide Thumbnail Text (Seen in image_b5d5ee.png):**
   - Apply theme CSS variables (--bg-canvas-backdrop, --bg-panel, --bg-ribbon, --text-primary, --text-secondary, --border-color) across 100% of the viewport so switching to **Dark Mode** turns the gray workspace surround into a rich dark slate (#0b1120), styles all sidebars/dialogs/status bars in dark mode, and ensures **high contrast everywhere**.
   - Specifically fix the bug in image_b5d5ee.png where slide thumbnail cards had white backgrounds with white/light-gray text: ensure thumbnail preview cards explicitly set color: #0f172a when their background is light, or style thumbnail cards with dark surfaces (#1e293b) and bright text (#f8fafc).
4. Write automated DOM tests verifying zero horizontal overflow clipping on the ribbon and high-contrast theme propagation. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 28.1.1

### Subphase 28.1.2: Multi-Document Per Format Manager, Side-by-Side Split View & Real Binary File Open/Save
- Spec File: .agents/stages/stage-28/subphase-28.1.2.md
/execute-subphase 28.1.2
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v5.0 Article I.
1. **Seamless Multi-Document Management (Multiple .docx, .xlsx, .pptx, .pdf, .txt at the same time):**
   - Upgrade the Left App Rail (Home, W, X, P, PDF, N):
     - Clicking an app icon (W, X, P, PDF, N) shows a badge with the count of open files of that type (e.g., W (3), X (2)). If none are open, it creates one; if already viewing that app, clicking + on the tab bar or clicking the app icon's + New / ðŸ“‚ Open quick menu opens **additional simultaneous documents** of that format (Untitled 1.docx, Untitled 2.docx, Project.docx, Budget.xlsx, Forecast.xlsx, etc.)!
     - Add a tab bar filter toggle: **Current App Tabs vs All Open Tabs (N)** so users can either focus strictly on their open Excel files OR see all open documents across all formats in one unified tab strip.
     - Add a **â—« Split View (Side-by-Side Workspace)** button in the top bar: when enabled, splits the main workspace into **Left Pane** and **Right Pane** with independent tab selectors so a user can edit two Word docs, two Excel sheets, or a Word doc + PDF side-by-side simultaneously!
2. **Fix Real Local File Open & Save for .docx, .xlsx, .pptx, .pdf, and .txt:**
   - Use JSZip / ooxml-opc + FileReader.readAsArrayBuffer() so when a user clicks Open (Ctrl+O) and selects a real .docx, .xlsx, .pptx, .pdf, or .txt file from their computer:
     - **.docx:** Unzips word/document.xml, parses paragraphs (<w:p>), runs (<w:r>, <w:t>, bold/italic/color/size), and tables (<w:tbl>) into editable HTML pages!
     - **.xlsx / .csv:** Unzips xl/sharedStrings.xml and xl/worksheets/sheet*.xml (or parses CSV) and populates all sheets, cell values, and formulas into the grid!
     - **.pptx:** Unzips ppt/slides/slide*.xml, extracts text shapes (<p:sp>, <a:t>) and geometry into editable slides!
     - **.pdf:** Reads the binary PDF buffer, parses page count, dimensions, and text streams (via pdf-engine / PDF parser), and renders every page onto the PDF canvas!
   - When clicking Save (Ctrl+S), package and download/write a valid binary .docx, .xlsx, .pptx, .pdf, or .txt file!
3. Write automated tests verifying opening 3 simultaneous .docx and 2 .xlsx files, enabling Split View, and round-trip binary parsing/saving for all 5 formats. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 28.1.2

