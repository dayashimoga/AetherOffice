# Subphase 4.1.1: SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance Resolver
- **Stage:** 4 | **Phase:** 4.1
- **Target Crate:** `ppt-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Complete PresentationML parser and 3-tier property/placeholder/theme cascade engine.
- **Validation Command:** `cargo test --package ppt-engine --test subphase_4_1_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 4.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine`, implement streaming parsers and serializers for `ppt/presentation.xml`, `ppt/slideMasters/*.xml`, `ppt/slideLayouts/*.xml`, `ppt/slides/*.xml`, `ppt/notesSlides/*.xml`, and `ppt/theme/*.xml`.
2. Build the 3-tier inheritance solver:
   - Resolve placeholder shapes (`p:ph` matched by `idx` or `type`) inheriting geometry (`a:xfrm`), fill/stroke styles, and `p:txStyles` (`titleStyle`, `bodyStyle`, `otherStyle`) from `SlideMaster` -> `SlideLayout` -> `Slide`.
   - Resolve Theme color schemes (`a:clrScheme` + tint/shade/lumMod/lumOff transforms) and font schemes (`+mj-lt`, `+mn-lt`).
3. Validate with >= 25 inheritance test vectors in `crates/ppt-engine/tests/subphase_4_1_1_test.rs` and commit with `[GEC-Verified]`.
```
