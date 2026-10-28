---
description: Executes any Stage.Phase.Subphase prompt under strict Global Engineering Contract (GEC-v1.0) enforcement.
---
# Workflow: Execute Subphase (`/execute-subphase`)

Execute this deterministic 8-step loop for the requested subphase:
1. **Tool Check & Podman Fallback:** If any required tool/package/CLI is not installed locally on the host, execute it using `tools/podman-runner.ps1` via Podman.
2. **Spec & Evidence:**
   - If binary behavior analysis is needed and `.agents/skills/rea-binary-analyst.md` is installed, invoke `@rea-binary-analyst` to write mathematical specs to `specs/evidence/<app>/<subphase_id>.md`.
   - Otherwise, document ECMA-376 schema rules in `specs/evidence/<app>/<subphase_id>.md` and generate JSON test vectors (`*_vectors.json`).
3. **Red Test Phase:** Write integration tests in `crates/<crate>/tests/` loading `specs/evidence/<app>/<subphase_id>_vectors.json`.
4. **Green Code Phase:** Implement clean-room Rust/UI code obeying GEC-v1.0 (zero `.unwrap()`/`.expect()`/`todo!()`, full `UnparsedPartStore` preservation).
5. **Contract Audit:** Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and fix any violations until it exits `0`.
6. **Cross-Target Check:** Run `cargo check --workspace --all-targets`.
7. **Atomic Commit & Tracker Advance:** Commit with `git commit -m "feat(subphase-<id>): <summary> [GEC-Verified]"`, then run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete <id>`.
8. **Human-in-the-Loop Approval Gate:** Summarize the completed subphase, present the updated tracker status, and explicitly ask the user for confirmation before proceeding to the next subphase or stage.
