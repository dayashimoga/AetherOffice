---
description: Automatically reads specs/stage-tracker.json, loads the next PENDING subphase prompt, executes it under GEC-v1.0, and advances the tracker.
---
# Workflow: Run Next Subphase (/run-next-subphase)

When triggered by the user:

1. Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Status` in the terminal to identify the current active or next PENDING subphase ID.
2. Read the corresponding subphase specification file from `.agents/stages/stage-<N>/subphase-<ID>.md`.
3. If the subphase specifies `REA Binary Analysis Required: YES` and `.agents/skills/rea-binary-analyst.md` is not present, run `powershell -ExecutionPolicy Bypass -File .\rea_bootstrap.ps1`.
4. Execute all steps in `.agents/workflows/execute-subphase.md` for that subphase ID.
5. Once `tools/verify-contract.ps1` passes, mark the subphase complete by running:
   ```powershell
   powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete <ID>
   ```
6. Report the updated project completion percentage and ask if the user wants to trigger `/run-next-subphase` for the next item.
