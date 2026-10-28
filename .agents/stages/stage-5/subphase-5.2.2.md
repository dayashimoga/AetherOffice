# Subphase 5.2.2: Adaptive Tablet & Mobile UI (Touch Drawers, Stylus Ink, Reflow Mode)
- **Stage:** 5 | **Phase:** 5.2
- **Target Crate:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Seamless Tablet (768px-1023px) and Mobile (< 768px) adaptation for Android and iOS.
- **Validation Command:** `cargo test --workspace --all-targets`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 5.2.2
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Tablet & Mobile Adaptation).
1. In `apps/unified-shell/src/`, implement automatic responsive layout switching based on viewport breakpoints, safe-area insets (`env(safe-area-inset-*)`), and input modality (`pointer: coarse`):
   - **Tablet Mode (`768px - 1023px`):** Collapsible touch-friendly Ribbon, direct stylus/Apple Pencil/S-Pen freehand ink annotation layer (`DrawingML ink`), and split-screen multitasking support.
   - **Mobile Mode (`< 768px`):** Replace top Ribbon with thumb-zone swipeable bottom command sheets, virtual-keyboard viewport height compensation, one-tap **Mobile Reflow Reader Mode** for Word documents, **Row Card Inspector** for rapid mobile Excel data entry, and touch laser-pointer mode for PowerPoint presentations.
2. Verify responsive state transitions and commit with `[GEC-Verified]`.
```
