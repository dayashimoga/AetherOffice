# Sub-Phase 50.1.1: Cross-Document Dependency Graph & Live Embeds

## Focus
`render-core`, `unified-shell`

## Goals
1. Implement pure safe-Rust Cross-Document Dependency Graph & Live Embeds in `render-core::cross_doc`:
   - Directed Acyclic Graph (DAG) of cross-document dependencies:
     - Document nodes, source anchors (spreadsheet ranges, charts, paragraphs, slide shapes, PDF regions), target embed containers.
     - Cycle detection with Tarjan / DFS cycle detection preventing circular dependency deadlocks.
     - Topological invalidation propagation returning exact evaluation sequences on document mutation.
     - In-memory cached snapshot representation (`LiveEmbedSnapshot`) preserving tabular matrix, SVG charts, or formatted text runs for immediate rendering even when source document tabs are suspended or offline.
2. Register high-speed Tauri IPC bridge:
   - `dep_graph_register_dependency`
   - `dep_graph_remove_dependency`
   - `dep_graph_get_dependents`
   - `dep_graph_get_dependencies`
   - `dep_graph_propagate_invalidation`
   - `dep_graph_resolve_embed`
3. Build presentation shell coordinator:
   - `CrossDocDependencyCoordinator.ts`:
     - Tracks active cross-document embeds across dockable split panes.
     - Reactive propagation of cell/range/paragraph changes to embedded components.
     - Embed status badges ("Live Linked", "Stale", "Source Suspended").
4. Comprehensive test coverage:
   - 10 Rust unit/integration tests in `crates/render-core/tests/subphase_50_1_1_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/cross_doc_dependency.test.ts`.
5. Pass `tools/verify-contract.ps1` with exit code 0.

