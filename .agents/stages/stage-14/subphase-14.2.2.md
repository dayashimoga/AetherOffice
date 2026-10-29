# Subphase 14.2.2: Live Remote Awareness, Cursors & Selection Highlights
- **Stage:** 14 | **Phase:** 14.2
- **Target Subsystem:** `unified-shell`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Multi-user live carets with color badges and usernames in WriterCanvas, multi-cell remote selection borders in CalcGridCanvas, peer presence avatar bar in top header.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 14.2.2
Invoke @adaptive-ui-builder under GEC-v2.0:
1. In `apps/unified-shell/src/state/collabState.ts`:
   - Track active peer sessions, client IDs, colors, names, and presence states.
   - Listen to incoming remote cursor and selection updates via IPC.
2. In `WriterCanvas.ts`:
   - Render remote carets with floating name tooltips and distinct peer colors on paginated text.
   - Render remote text selection highlights with semi-transparent tint.
3. In `CalcGridCanvas.ts`:
   - Render multi-peer active cell outlines and range selection boxes with peer color badges.
4. In top header bar (`UnifiedShellApp`):
   - Add peer presence avatars list with active user count and Share / Join Session modal.
5. Add automated Vitest integration tests in `apps/unified-shell/src/__tests__/collab_presence.test.ts`.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
