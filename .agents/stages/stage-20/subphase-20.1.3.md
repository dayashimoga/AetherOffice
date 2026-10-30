# Subphase 20.1.3: Interactive AetherCopilot Assistant UI Sidebar & Inline Ghost Text
- **Stage:** 20 | **Phase:** 20.1
- **Target Subsystem:** `unified-shell`
- **Primary Agent Skill:** `@frontend-architect`
- **Deliverable:** Copilot Chat Sidebar (`CopilotSidebar.ts`), in-canvas inline ghost-text completion with Tab acceptance, diff review modal, and contextual prompt chips.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 20.1.3
Invoke @frontend-architect under GEC-v2.0:
1. Implement `CopilotSidebar.ts` in `unified-shell` with conversation message thread, suggested prompt chips, and citation links.
2. Implement inline ghost-text completion overlay in `WriterCanvas.ts` displaying light gray completion suggestions with `[Tab]` to accept and `[Esc]` to dismiss.
3. Implement `DiffPreviewModal.ts` displaying before-and-after visual diffs before applying AI text or formula changes.
4. Wire Ribbon "Aether Copilot" button in the Home and Review ribbon tabs to toggle the Copilot sidebar.
5. Add Vitest integration tests for chat interaction, ghost text typing and acceptance, and diff application.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

