# Subphase 42.2.3: Smart Workspace Persistence (.aether-workspace)

- **Stage:** 42 | **Phase:** 42.2
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@workspace-persistence-architect`
- **Deliverables:**
  1. Implement `WorkspacePersistenceService.ts` in `apps/unified-shell/src/services/` or `controllers/` defining:
     - Robust `.aether-workspace` format schema (version, title, timestamp, tabs array with file paths, docTypes, cursor/scroll coordinates, split-pane layout mode, split ratios, slot tab assignments, focused slot, project explorer state).
     - Clean-room serializer (`serializeWorkspace`) and validator/deserializer (`deserializeWorkspace`) with graceful fallbacks.
     - Native file save/load via Tauri dialog/fs with browser download/FileReader fallback.
  2. Wire workspace save/restore into `UnifiedShellApp` (`main.ts`):
     - `saveWorkspace(filePath?)`: saves current workspace state to `.aether-workspace`.
     - `loadWorkspace(workspaceData | filePath)`: rehydrates all tabs, reopens files, recreates split layout and slot assignments, restores project explorer.
     - Global shortcut (`Ctrl+Shift+S` / `Cmd+Shift+S`) or command palette action for workspace persistence.
  3. Comprehensive automated test suite in `apps/unified-shell/src/__tests__/workspace_persistence.test.ts` validating:
     - Schema serialization and deserialization.
     - Validation of corrupt/malformed workspace JSON without crash.
     - Restoration of mixed-format tabs (.docx, .xlsx, .pptx, .pdf, .md).
     - Restoration of 2-way and 4-way split layouts and slot tab assignments.
     - Round-trip fidelity (serialize -> deserialize preserves all properties).
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

