# Subphase 41.1.1: Responsive Desktop / Tablet / Mobile Adaptation & Exhaustive Automated Test Suite

- **Stage:** 41 | **Phase:** 41.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Seamless adaptation across Desktop, Tablet, and Mobile viewports + 300-assertion automated test suite verifying every menu item and file round-trip.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 41.1.1
Invoke @adaptive-ui-builder under GEC-v6.0.
1. **Cross-Platform Responsive Adaptation (Desktop, Laptop, Tablet, Mobile):**
   - Ensure the UI adapts smoothly across all screen sizes and OS targets (Windows, Linux, macOS, Android, iOS):
     - **Desktop / Laptop (`>= 1024px`):** Left vertical App Rail + full wrapping Ribbon + collapsible left/right sidebars + Split Screen support.
     - **Tablet (`768px - 1023px`):** Collapsible sidebars (drawer toggle button) + touch-sized `34px` ribbon buttons + pinch-to-zoom canvas support.
     - **Mobile (`< 768px`):** Bottom App Navigation Bar (`Home`, `W`, `X`, `P`, `PDF`, `N`) instead of left rail, compact single-row horizontally scrollable touch toolbar, full-width document canvas, and slide-over sidebar drawers.
2. **Exhaustive Automated Verification Suite (`apps/unified-shell/src/__tests__/gec_v6_full_verification.test.tsx`):**
   - Automatically test and assert:
     1. **Word Body vs Header Isolation:** Typing in Word edits ONLY `.word-page-body` and NEVER duplicates text into the header; double-clicking Header edits only the header and `Exit Header/Footer` returns to body.
     2. **File Open & Save Round-Trip for All 5 Formats:** Programmatically saves and re-opens `.docx`, `.xlsx`, `.pptx`, `.pdf`, and `.txt` files and asserts 100% of the saved text, cells, slides, and PDF objects load back into the editors (zero empty files!).
     3. **App-Scoped Multi-Document Workspaces:** Opening 3 `.docx` files, 3 `.xlsx` files, 2 `.pptx` files, 2 `.pdf` files, and 2 `.txt` files filters tabs cleanly per app when clicking `W`, `X`, `P`, `PDF`, and `N`.
     4. **Full Dark/Light Theme:** Asserts PDF Studio sidebar, Word, Excel, PPT, and Notes all apply dark theme backgrounds and high-contrast text.
     5. **100% Menu Button Execution:** Crawls every tab and clicks every button across all 5 apps, asserting real state/DOM updates.
3. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1`, build the release binary, and commit + push to origin with `[GEC-Verified]`.
````
