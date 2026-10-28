# Subphase 5.2.1: Tauri v2 Desktop & Laptop Shell (Ribbon, Multi-Tab, Formula Bar)
- **Stage:** 5 | **Phase:** 5.2
- **Target Crate:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Complete Tauri v2 Desktop UI (>= 1024px) in apps/unified-shell with multi-document tabs, contextual Ribbon, and keyboard shortcuts.
- **Validation Command:** `cargo test --workspace --all-targets`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 5.2.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Desktop/Laptop Form Factor).
1. Scaffold the Tauri v2 application inside `apps/unified-shell/` (`package.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) linking `ooxml-opc`, `word-engine`, `excel-engine`, `ppt-engine`, and `render-core`.
2. Implement the Desktop/Laptop UI (`>= 1024px`, fine pointer):
   - Multi-document workspace tab bar allowing `.docx`, `.xlsx`, and `.pptx` files to be edited side-by-side in a single lightweight window.
   - Contextual Ribbon toolbar (Home, Insert, Layout, Formulas, Slide Show, Review, View), Excel Formula Bar with live syntax highlighting, Word Ruler & Navigation Pane, and PowerPoint Slide Sorter filmstrip.
   - Full keyboard shortcut engine (`Ctrl/Cmd+C/V/Z/Y/S/F`, Excel `F2`/`F4`/Arrow navigation, Alt keytips).
3. Verify Tauri IPC commands and workspace tests via `tools/verify-contract.ps1` and commit with `[GEC-Verified]`.
```
