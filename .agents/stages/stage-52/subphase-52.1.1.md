# Sub-Phase 52.1.1: Semantic Design Token Overhaul & Classic/Compact Ribbon Switcher

## 1. Context & Objectives
As defined in GEC-v7.0 Article V and Stage 52 of the master roadmap, Sub-Phase 52.1.1 implements a comprehensive semantic design token architecture and adaptive ribbon layout engine (`render-core`, `unified-shell`).

Key objectives:
1. **Rust Design Token Engine (`render-core::theme`)**:
   - Semantic color and metric design tokens supporting `Light`, `Dark`, `HighContrastDark`, and `HighContrastLight`.
   - WCAG 2.1 AA compliant contrast ratio guarantees (4.5:1 standard, 7.0:1 high-contrast).
   - Dynamic CSS variable generation exporting 100% theme coverage for canvas surrounds, sidebars, rulers, grids, tabs, and modals.
   - Ribbon layout modes: `Classic` (multi-row with large buttons and grouped labels), `Compact` (single-row horizontal streamline), and `Collapsed` (tab-strip only).
2. **Tauri IPC Command Endpoints**:
   - `theme_get_tokens`: Retrieves semantic CSS design tokens for an active theme mode.
   - `theme_set_mode`: Sets active shell theme.
   - `ribbon_set_layout_mode`: Switches between Classic, Compact, and Collapsed ribbon modes.
   - `ribbon_get_layout_config`: Returns current ribbon layout and height metrics.
3. **Presentation Shell Integration**:
   - `ThemeAndRibbonCoordinator.ts` in `apps/unified-shell/src/state/`:
     - Reactive state coordinator managing theme switching, high contrast modes, and ribbon layout switching.
     - Automatically updates DOM attributes (`data-theme`, `data-ribbon-mode`, `data-high-contrast`).
     - Integrated toggle button on Ribbon header for seamless switching between Classic and Compact layouts.
4. **Verification & Invariants**:
   - 10 unit & integration tests in `crates/render-core/tests/subphase_52_1_1_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/theme_ribbon_coordinator.test.ts`.
   - Master gatekeeper `tools/verify-contract.ps1` exit code 0.

