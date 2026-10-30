# Subphase 19.1.2: 3D Shader Transitions & Keyframe Motion Paths
- **Stage:** 19 | **Phase:** 19.1
- **Target Subsystem:** `ppt-engine` & `render-core`
- **Primary Agent Skill:** `@graphics-engineer`
- **Deliverable:** Hardware-accelerated 3D slide transitions (Vortex, Honeycomb, Origami, Cube, Flip, Ripple) using CSS 3D transforms & WebGL, and custom keyframe bezier motion path animations.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 19.1.2
Invoke @graphics-engineer under GEC-v2.0:
1. Extend `SlideTransitionEngine` with 3D transition effects: Cube, Flip, Vortex, Honeycomb, and Origami.
2. Implement keyframe bezier motion path compiler translating DrawingML `<animMotion>` into smooth CSS/Canvas animation trajectories.
3. Support custom velocity easing curves (Ease-In-Out, Cubic Bezier, Bounce, Spring).
4. Integrate transitions and motion paths into both the interactive Slide Designer preview and the Fullscreen (F5) Slide Show.
5. Add unit and UI tests verifying 60fps render loop timing, transform matrices, and fallback behavior on software rendering.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

