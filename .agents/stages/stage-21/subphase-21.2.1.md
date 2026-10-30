# Subphase 21.2.1: Visual Overhaul: Non-Overflowing Flexbox Ribbon & Dark/Light Theme Toggle
- **Stage:** 21 | **Phase:** 21.2
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Pixel-perfect aligned Ribbon without horizontal/vertical scrollbars, fixed active tab highlight, and instant Dark/Light mode switch.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 21.2.1
Invoke @adaptive-ui-builder under GEC-v3.0 Article III.
1. **Fix Ribbon Overflow & Alignment Bug:**
   - Remove the broken overflow: scroll / fixed small height on the Ribbon toolbar that caused ugly scrollbars to cover the buttons.
   - Redesign the Ribbon using a clean, modern single-row or compact two-row CSS Flexbox layout (min-height: 88px, overflow: visible, crisp 1px solid var(--border) group dividers, properly aligned 28px buttons and dropdowns).
   - Fix the tab highlight bug: File must only be highlighted when the File Backstage menu is open; when Home, Insert, Layout, etc. are clicked, ONLY the clicked tab shows the active underline/pill.
2. **Implement Global Dark / Light Theme Toggle (Light / Dark):**
   - Add a prominent Theme Toggle button in the top titlebar (Light Mode vs Dark Mode).
   - Define full CSS custom properties for Light (#f8fafc app bg, #ffffff ribbon/surface, #0f172a text) and Dark (#0f172a app bg, #1e293b ribbon/surface, #f8fafc text, #334155 borders) so the entire UI flips smoothly with zero unreadable text.
3. Verify with DOM tests that toggling the theme updates document.documentElement.dataset.theme and that every Ribbon tab switches cleanly. Commit with [GEC-Verified].
```
