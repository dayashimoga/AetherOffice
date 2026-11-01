# Sub-Phase 44.2.1 Specification: Omnibox Command, File & Symbol Launcher (Ctrl+K)

## 1. Context & Objectives
In compliance with GEC-v7.0 Article I, III, IV, and VI:
- Deliver a unified spotlight-style Omnibox command launcher triggered by `Ctrl+K` / `Cmd+K`.
- Support fast fuzzy search over:
  - Application commands (ribbon actions, backstage options, split panes, theme switching, global undo/redo).
  - Open document tabs and recent files (filtered via `@` prefix).
  - Document structural symbols (headings, sheets, slides, bookmarks, ranges, filtered via `:` prefix).
  - Direct search query fallback to workspace federated search.
- Clean-room Rust implementation in `crates/render-core/src/omnibox.rs` with `#![forbid(unsafe_code)]` and zero unhandled errors.
- Sub-millisecond ($< 1.0\text{ ms}$) fuzzy matching and ranking SLA.
- Responsive, theme-aware frontend modal overlay with dark/light mode token compliance and full keyboard navigation.

## 2. Technical Scope

### 2.1 Rust Core (`crates/render-core/src/omnibox.rs`)
- `OmniboxCategory`: `Command`, `File`, `Symbol`, `Search`.
- `OmniboxItem`: `id`, `category`, `title`, `description`, `shortcut`, `icon`, `score`.
- `OmniboxEngine`:
  - `fuzzy_score(query: &str, target: &str) -> Option<i32>`
  - `query_omnibox(mgr: &DesktopSessionManager, query: &str, max_results: usize) -> Vec<OmniboxItem>`
  - Prefix routing:
    - `>` or default: Commands
    - `@`: Files & open tabs
    - `:`: Structural document symbols
    - `?`: Search mode
- Module registration in `crates/render-core/src/lib.rs`.

### 2.2 Tauri IPC Commands (`apps/unified-shell/src-tauri`)
- `omnibox_query(state, query, max_results)`
- Register in `tauri::generate_handler!` and `lib.rs`.
- `#[cfg(test)]` wrappers and unit test execution assertions in `tauri_commands.rs`.

### 2.3 Frontend Presentation Shell (`apps/unified-shell`)
- IPC typing in `src/ipc.ts`: `OmniboxCategory`, `OmniboxItem`, `IPC.omniboxQuery`.
- Component `src/components/OmniboxLauncher.ts`:
  - Centered modal dialog with backdrop blur.
  - Search input with prefix badges.
  - Keyboard navigation: ArrowUp, ArrowDown, Enter, Esc.
  - Execution callbacks for commands, file switches, and symbol navigation.
- Wire `Ctrl+K` in `KeyboardShortcuts.ts` and `main.ts`.

### 2.4 Verification Gates
- 100% Rust unit tests in `crates/render-core/tests/subphase_44_2_1_test.rs`.
- 100% Vitest unit tests in `apps/unified-shell/src/__tests__/omnibox_launcher.test.ts`.
- `tools/verify-contract.ps1` exit code 0.

