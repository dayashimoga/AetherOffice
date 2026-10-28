---
trigger: always_on
description: Global Engineering Contract (GEC-v1.0) governing clean-room reverse engineering, ECMA-376 fidelity, performance SLAs, cross-platform adaptive UI, and zero-defect verification.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v1.0)
**Project:** OpenOfficeRS (Word, Excel, PowerPoint Unified Rust Engine & Shell)
**Status:** MANDATORY / NON-NEGOTIABLE FOR ALL ANTIGRAVITY AGENTS

## ARTICLE I: LEGAL CLEAN-ROOM & BINARY ISOLATION WALL
1. **Open Format First:** All `.docx`, `.xlsx`, and `.pptx` structural parsing and serialization MUST follow the public ECMA-376 / ISO/IEC 29500 standard.
2. **Strict Two-Phase Separation (When REA is Active):**
   - **Analysis Phase (`@rea-binary-analyst`):** May inspect staged binaries ONLY to document mathematical equations, state transitions, and input/output test vectors inside `specs/evidence/<app>/`.
   - **Implementation Phase (`@ecma376-architect`, `@adaptive-ui-builder`):** Writes clean, idiomatic Rust/TypeScript in `crates/` and `apps/` reading ONLY from ECMA-376 specs and `specs/evidence/`.
3. **Zero Contamination:** Never paste decompiled C/C++, x86_64 assembly, or Microsoft internal symbols into `crates/` or `apps/`.

## ARTICLE II: ZERO-LOSS ECMA-376 ROUND-TRIP FIDELITY
1. Opening and saving any `.docx`, `.xlsx`, or `.pptx` file MUST NEVER strip or corrupt unmodeled XML elements, attributes (`mc:Ignorable`), macros (`vbaProject.bin`), or OPC `.rels` relationships.
2. Every parser must retain unrecognized nodes/parts in an `UnparsedPartStore` and write them back deterministically on save.

## ARTICLE III: HARD PERFORMANCE & RESOURCE SLAs
1. **Startup:** `< 150 ms` on Desktop; `< 300 ms` on Mobile.
2. **Memory:** Idle `< 35 MB` RSS; 1M populated Excel cells `< 180 MB` RSS via Compressed Sparse Page Grid (CSPG).
3. **Latency:** Viewport scrolling/zoom locked at `120 fps` (`< 8.33 ms` frame budget); Word typing `< 1.0 ms` per keystroke via Piece-Tree Rope.
4. **Threading:** Never block the UI or GPU thread with file I/O, XML parsing, or formula evaluation.

## ARTICLE IV: RUST & TYPESCRIPT CODE QUALITY INVARIANTS
1. `#[forbid(unsafe_code)]` enforced in `ooxml-opc`, `word-engine`, `excel-engine`, and `ppt-engine`.
2. `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, and `unimplemented!()` are **BANNED** in non-test library code (`crates/*/src/**`). Return typed `Result<T, EngineError>` using `thiserror`.
3. Zero warnings allowed under `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.

## ARTICLE V: UNIVERSAL CROSS-PLATFORM & ADAPTIVE UI PARITY
1. All engine crates must compile cleanly for Windows, Linux, macOS, Android, and iOS.
2. `apps/unified-shell` must automatically adapt across:
   - **Desktop/Laptop (`>= 1024px`, fine pointer):** Multi-tab workspace, full Ribbon, formula bar, dockable panels, keyboard shortcuts.
   - **Tablet (`768px - 1023px`, touch/stylus):** Compact touch ribbon, stylus ink overlay, split-view support.
   - **Mobile (`< 768px`, touch):** Bottom command sheets, virtual-keyboard inset avoidance, Word Reflow Reader mode, Excel card editor.

## ARTICLE VI: DEFINITION OF DONE (DoD) GATEKEEPING
No subphase is complete until `tools/verify-contract.ps1` exits with code `0` and changes are committed with `[GEC-Verified]`.

## ARTICLE VII: HUMAN-IN-THE-LOOP APPROVAL GATES & CONTAINER FALLBACK
1. **Never Silently Auto-Advance:** After completing any subphase or stage and verifying with `tools/verify-contract.ps1`, the agent MUST pause and present a concise summary of deliverables and contract audit results to the user.
2. **Explicit User Approval Required:** The agent MUST explicitly ask the user for confirmation/permission before proceeding to the next subphase or advancing to the next stage. Never start the next subphase without affirmative user consent.
3. **Podman Container Fallback:** If any required tool, package, compiler, or CLI (such as `gh`, Android NDK, or headless Linux utilities) is not installed locally on the host machine, the agent MUST run it via `tools/podman-runner.ps1` using a Podman container.
