# Sub-Phase 48.2.1 Specification: Hardware-Accelerated Morph Transition & SMIL Timeline Scrubbing

## Objective
Establish high-fidelity, hardware-accelerated Morph transitions and continuous SMIL animation timeline scrubbing for AetherSlides (`ppt-engine`, `render-core`, `unified-shell`).

## Architecture & Deliverables
1. **`ppt-engine` (Morph & SMIL DTOs & Evaluator Bridge)**:
   - Export serializable DTOs for Morph transitions (`MorphedSlideStateDto`, `MorphedShapeStateDto`, `MorphedTransformDto`).
   - Implement `MorphTransitionBridge`:
     - `evaluate_morph(source_slide, target_slide, progress, morph_option) -> MorphedSlideStateDto`
     - Heuristic & explicit matching (`!!name`), coordinate interpolation, opacity crossfade, text alignment.
   - Implement `SmilAnimationBridge`:
     - `extract_slide_animation_tracks(slide) -> SlideAnimationTracksDto`
     - `scrub_animation(slide, time_ms) -> AnimatedSceneStateDto`
     - `step_click(slide, current_click, forward: bool) -> AnimationClickResultDto`

2. **`render-core` (Hardware-Accelerated Timeline Scrubber & Matrix Interpolator)**:
   - `TimelineScrubber`:
     - Continuous time playback state machine (`Playing`, `Paused`, `Stopped`).
     - Frame delta calculation supporting 60fps and 120fps ticks.
     - Matrix compositor: maps shape transforms (translation, rotation, scale, opacity) into hardware renderable 2D affine / 4x4 matrix transforms.

3. **Desktop Tauri IPC Bridge (`apps/unified-shell/src-tauri`)**:
   - `ppt_evaluate_morph_transition`
   - `ppt_get_slide_animation_tracks`
   - `ppt_scrub_slide_animation`
   - `ppt_advance_animation_click`

4. **Unified Presentation Shell (`apps/unified-shell`)**:
   - `SlideAnimationCoordinator.ts`:
     - Controls playback, scrub bar slider, play/pause toggles, click advances, and morph previews.
   - Full Vitest suite: `apps/unified-shell/src/__tests__/slide_animation_morph_scrubber.test.ts`.

5. **Test & Verification**:
   - Rust integration test suite: `crates/ppt-engine/tests/subphase_48_2_1_test.rs`.
   - Vitest suite: 10/10 tests.
   - Automated gatekeeper: `tools/verify-contract.ps1` exit code 0.

