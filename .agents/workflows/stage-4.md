---
description: Automated execution workflow for Stage 4: PowerPoint Engine (PresentationML Inheritance, 187 Shapes, SMIL & Morph)
---
# Workflow: Execute Stage 4 (/stage-4)
## Objective: Stage 4: PowerPoint Engine (PresentationML Inheritance, 187 Shapes, SMIL & Morph)

When invoked via `/stage-4`, execute the following subphases sequentially. Do NOT advance to the next subphase until `tools/verify-contract.ps1` exits with code 0 and `specs/stage-tracker.json` is updated:

### Subphase 4.1.1: SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance Resolver
- **Prompt Spec File:** `.agents/stages/stage-4/subphase-4.1.1.md`

**Instructions:**
```text
/execute-subphase 4.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine`, implement streaming parsers and serializers for `ppt/presentation.xml`, `ppt/slideMasters/*.xml`, `ppt/slideLayouts/*.xml`, `ppt/slides/*.xml`, `ppt/notesSlides/*.xml`, and `ppt/theme/*.xml`.
2. Build the 3-tier inheritance solver:
   - Resolve placeholder shapes (`p:ph` matched by `idx` or `type`) inheriting geometry (`a:xfrm`), fill/stroke styles, and `p:txStyles` (`titleStyle`, `bodyStyle`, `otherStyle`) from `SlideMaster` -> `SlideLayout` -> `Slide`.
   - Resolve Theme color schemes (`a:clrScheme` + tint/shade/lumMod/lumOff transforms) and font schemes (`+mj-lt`, `+mn-lt`).
3. Validate with >= 25 inheritance test vectors in `crates/ppt-engine/tests/subphase_4_1_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 4.1.1` to verify contract compliance and advance the state tracker.

### Subphase 4.1.2: DrawingML 187 Preset Geometries & Guide Formula Compiler
- **Prompt Spec File:** `.agents/stages/stage-4/subphase-4.1.2.md`

**Instructions:**
```text
/execute-subphase 4.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/drawingml.rs`, implement the ECMA-376 Part 1 DrawingML geometry formula evaluator (`a:avLst`, `a:gdLst`, `a:pathLst`):
   - Support all DrawingML guide operations (`*/`, `+-`, `+/`, `?:`, `abs`, `at2`, `cat2`, `cos`, `sin`, `sqrt`, `tan`, `val`, `min`, `max`, `pin`, `mod`).
   - Compile all 187 standard `a:prstGeom` shapes (`rect`, `roundRect`, `ellipse`, `bentArrow`, `chevron`, `star5`, `flowChart*`, `math*`, etc.) and custom paths (`a:custGeom`) into normalized Bezier path commands (`MoveTo`, `LineTo`, `CubicTo`, `QuadTo`, `ArcTo`, `Close`).
2. Write >= 30 geometry evaluation tests in `subphase_4_1_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 4.1.2` to verify contract compliance and advance the state tracker.

### Subphase 4.2.1: SMIL Animation Timing Tree (p:timing) State Machine
- **Prompt Spec File:** `.agents/stages/stage-4/subphase-4.2.1.md`

**Instructions:**
```text
/execute-subphase 4.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/animation.rs`, implement the PresentationML SMIL animation timing engine (`p:timing` / `p:tnLst`):
   - Parallel (`p:par`) and sequential (`p:seq`) time containers, click triggers (`onClick`), `withPrev`, `afterPrev`, repeat counts, auto-reverse, and acceleration/deceleration bezier easing curves.
   - Property interpolators for `p:anim`, `p:animEffect`, `p:animMotion` (SVG/DrawingML motion paths), `p:animRot`, `p:animScale`, and `p:animClr`.
2. Verify deterministic 120Hz frame sampling (`sample_at(time_ms)`) across >= 25 timeline vectors in `subphase_4_2_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 4.2.1` to verify contract compliance and advance the state tracker.

### Subphase 4.2.2: REA Morph Transition Vertex Solver & SmartArt Layout Engine
- **Prompt Spec File:** `.agents/stages/stage-4/subphase-4.2.2.md`

**Instructions:**
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

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 4.2.2` to verify contract compliance and advance the state tracker.

