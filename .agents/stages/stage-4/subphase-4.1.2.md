# Subphase 4.1.2: DrawingML 187 Preset Geometries & Guide Formula Compiler
- **Stage:** 4 | **Phase:** 4.1
- **Target Crate:** `ppt-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** DrawingML vector path evaluator supporting all 187 ECMA-376 preset geometries and interactive adjustment handles.
- **Validation Command:** `cargo test --package ppt-engine --test subphase_4_1_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 4.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/drawingml.rs`, implement the ECMA-376 Part 1 DrawingML geometry formula evaluator (`a:avLst`, `a:gdLst`, `a:pathLst`):
   - Support all DrawingML guide operations (`*/`, `+-`, `+/`, `?:`, `abs`, `at2`, `cat2`, `cos`, `sin`, `sqrt`, `tan`, `val`, `min`, `max`, `pin`, `mod`).
   - Compile all 187 standard `a:prstGeom` shapes (`rect`, `roundRect`, `ellipse`, `bentArrow`, `chevron`, `star5`, `flowChart*`, `math*`, etc.) and custom paths (`a:custGeom`) into normalized Bezier path commands (`MoveTo`, `LineTo`, `CubicTo`, `QuadTo`, `ArcTo`, `Close`).
2. Write >= 30 geometry evaluation tests in `subphase_4_1_2_test.rs` and commit with `[GEC-Verified]`.
```
