# Sub-Phase 52.2.1: Synchronized ARIA Shadow DOM, Full Keyboard Navigation & CJK/RTL IME Bridge

## Objective
Establish complete WCAG 2.1 AA accessibility, keyboard accessibility, screen reader interoperability, and international text composition (CJK IME & RTL BiDi) across all canvas viewports and shell containers:
1. **Authoritative Rust Core (`crates/render-core/src/accessibility.rs`, `lib.rs`)**:
   - `AriaNode`: Semantic node model with `id`, `role` (`Document`, `Paragraph`, `Heading`, `Grid`, `Row`, `Cell`, `Shape`, `Button`), `label`, `value`, `level`, `selected`, `bounds_rect`, `children`.
   - `AriaTree`: Virtualized accessibility tree builder and updater with incremental reconciliation.
   - `ImeCompositionState`: Composition buffer, selection span, candidate list coordinates, and pre-edit text tracker.
   - `BidiDirection`: `Ltr`, `Rtl`, `Auto` with Unicode BiDi algorithm analysis.
   - `KeyboardNavigationMap`: Spatial and sequential tab navigation router (`F6` pane cycling, arrow keys, Tab/Shift-Tab).
2. **Native Tauri Desktop IPC (`apps/unified-shell/src-tauri`)**:
   - Commands:
     - `accessibility_get_aria_tree`: Queries resolved virtualized accessibility tree for active document viewport.
     - `accessibility_update_cursor`: Syncs screen reader virtual cursor position with document caret.
     - `ime_process_composition`: Processes IME composition updates and computes candidate popup bounds.
     - `bidi_analyze_text`: Computes BiDi runs and text direction markers for paragraph text.
3. **TypeScript Presentation Shell Coordinator (`apps/unified-shell/src/state/AccessibilityAndImeCoordinator.ts`)**:
   - Renders synchronized virtualized ARIA overlay on canvas viewports (VoiceOver / NVDA / Narrator / JAWS).
   - Manages off-screen IME hidden input buffer positioned at document caret coordinates.
   - Implements `F6` pane cycling (Title Bar <-> Ribbon <-> Side Pane <-> Canvas <-> Status Bar).
   - Supports RTL text alignment and CJK multi-stage composition.
4. **Verification & Invariants**:
   - 10 integration tests in `crates/render-core/tests/subphase_52_2_1_test.rs`.
   - 10 unit/integration tests in `apps/unified-shell/src/__tests__/accessibility_ime_coordinator.test.ts`.
   - 100% pass on `tools/verify-contract.ps1`.

