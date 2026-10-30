# Subphase 28.1.2: Fix Ghost Word Sidebar Bug, Hide Alt-Badges & Fix Global Dark/Light Theme
- **Stage:** 28 | **Phase:** 28.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Eliminates Word Navigation sidebar bleeding into PDF/Notes, fixes hardcoded '22 words' status bar, hides [F]/[H] badges unless Alt is held, and applies Dark/Light theme to 100% of the app.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 28.1.2
Invoke @adaptive-ui-builder under GEC-v4.0 Article II.
1. **Fix Ghost Navigation Sidebar Bug (Seen in image_2586db.png & image_258017.png):**
   - Currently, Word's white Navigation sidebar is rendered globally outside the Word view, causing it to appear next to PDF's Pages sidebar and Notes' SCRATCHPADS sidebar!
   - Move Word's Navigation sidebar strictly inside the Word editor component so it NEVER renders when viewing Excel, PowerPoint, PDF, or Notes.
2. **Fix Permanent [F] [H] [N] [P] Badges on Ribbon Tabs:**
   - Hide the black Alt-keytip badges (F, H, N, P, S, R, W) on Ribbon tabs by default; only show them when ltKeyTipsActive === true (toggled by pressing and releasing Alt).
3. **Full-App Dark / Light Theme Cascade:**
   - Fix the theme toggle so switching between â˜€ï¸ Light and ðŸŒ™ Dark updates the entire application root, Titlebar, Tab Bar, Ribbon, Left Sidebar, Workspace background, Modals, and Status Bar (while keeping an optional Dark Page / Paper Canvas toggle in View for Word/PDF).
4. **Fix Hardcoded 22 words Status Bar:**
   - Replace the static 22 words | English (US) string at the bottom of the screen with dynamic per-app status indicators (Word: real live word/char/page count; Excel: active sheet count + selected cell SUM/AVG/COUNT; PPT: slide index + shape count; PDF: page index + page size in mm/in + encryption status; Notes: line/col/word/char count).
5. Write automated DOM tests verifying zero ghost sidebars in PDF/Notes, full-app theme class propagation, and live dynamic status bar counts. Commit with [GEC-Verified].
```
