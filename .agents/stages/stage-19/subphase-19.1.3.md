# Subphase 19.1.3: Presenter Studio Dual-Screen Mode & Audience Annotation Laser
- **Stage:** 19 | **Phase:** 19.1
- **Target Subsystem:** `unified-shell`
- **Primary Agent Skill:** `@frontend-architect`
- **Deliverable:** Multi-window Presenter Console with current slide, next slide preview, teleprompter speaker notes, presentation timer, and audience laser pointer / pen ink overlay.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 19.1.3
Invoke @frontend-architect under GEC-v2.0:
1. Implement `PresenterConsoleWindow.ts` designed for dual-monitor or projection setups.
2. Render active slide viewport, upcoming next-slide card, elapsed/remaining clock timer, and rich formatted speaker notes.
3. Implement live presentation laser pointer (virtual glowing cursor dot) and ink pen annotation overlay across the presentation display.
4. Implement synchronized slide advance / slide sorter quick navigation between Presenter View and Audience Display.
5. Add keyboard shortcuts (B for black screen, W for white screen, P for pen, L for laser, Esc to exit).
6. Add Vitest integration tests covering multi-window state synchronization, timer controls, and annotation event dispatch.
7. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
