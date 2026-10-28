# Subphase 4.2.2: REA Morph Transition Vertex Solver & SmartArt Layout Engine
- **Stage:** 4 | **Phase:** 4.2
- **Target Crate:** `ppt-engine`
- **Primary Agent Skill:** `@rea-binary-analyst`
- **REA Binary Analysis Required:** YES (Run .\rea_bootstrap.ps1 before starting)
- **Deliverable:** Clean-room Morph transition shape/character interpolator and SmartArt (dgm:layoutDef) constraint solver.
- **Validation Command:** `cargo test --package ppt-engine --test subphase_4_2_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 4.2.2
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting `ppcore.dll`. After extracting specs, run `.\removeRea_bootstrap.ps1` to permanently lock the clean room!
1. Invoke @rea-binary-analyst on staged `ppcore.dll` to document:
   - Morph transition (`p159:morph`) correspondence matching rules: explicit `!!` shape naming, automatic shape/text/word/character correspondence scoring, and equal-subdivision cubic Bezier path correspondence when morphing between shapes with different vertex counts.
   - SmartArt (`ppt/diagrams/layout*.xml`) constraint solver (`dgm:alg` types: `lin`, `pyra`, `cycle`, `hierChild`, `snake`, `tx`, `sp`) and fallback synced `p:spTree` cache sync.
2. Save the clean-room mathematical spec to `specs/evidence/ppt/4.2.2.md` and >= 25 test vectors to `specs/evidence/ppt/4.2.2_vectors.json`.
3. Run `powershell -ExecutionPolicy Bypass -File .\removeRea_bootstrap.ps1` to purge REA and staged binaries from the workspace.
4. Implement the clean-room Morph and SmartArt solver in `crates/ppt-engine/src/morph.rs`, pass `subphase_4_2_2_test.rs`, and commit with `[GEC-Verified]`.
```
