# Subphase 4.2.1: SMIL Animation Timing Tree (p:timing) State Machine
- **Stage:** 4 | **Phase:** 4.2
- **Target Crate:** `ppt-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** 120Hz frame-accurate slide animation timeline evaluator (entrance, emphasis, exit, motion paths).
- **Validation Command:** `cargo test --package ppt-engine --test subphase_4_2_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 4.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/animation.rs`, implement the PresentationML SMIL animation timing engine (`p:timing` / `p:tnLst`):
   - Parallel (`p:par`) and sequential (`p:seq`) time containers, click triggers (`onClick`), `withPrev`, `afterPrev`, repeat counts, auto-reverse, and acceleration/deceleration bezier easing curves.
   - Property interpolators for `p:anim`, `p:animEffect`, `p:animMotion` (SVG/DrawingML motion paths), `p:animRot`, `p:animScale`, and `p:animClr`.
2. Verify deterministic 120Hz frame sampling (`sample_at(time_ms)`) across >= 25 timeline vectors in `subphase_4_2_1_test.rs` and commit with `[GEC-Verified]`.
```
