# Subphase 28.1.1: Modern Fluent/Linear Visual Overhaul, Full-App Dark Theme & High-Contrast Fix
- **Stage:** 28 | **Phase:** 28.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Visually modern UI design, zero clipped ribbon items, full-app dark/light theme including canvas backdrop, and fixed thumbnail contrast.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
