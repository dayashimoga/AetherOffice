# Subphase 14.1.1: Core Fugue Tree-RGA Piece Tree CRDT for Rich Text
- **Stage:** 14 | **Phase:** 14.1
- **Target Subsystem:** `crdt-collab` & `word-engine`
- **Primary Agent Skill:** `@crdt-architect`
- **Deliverable:** Pure deterministic CRDT piece-tree data structure, Lamport timestamp ordering, causal insertion/deletion tree, Peritext rich-formatting intervals, concurrent edit fuzz tests.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 14.1.1
Invoke @crdt-architect under GEC-v2.0:
1. Create new workspace crate `crates/crdt-collab` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Implement `FugueTreeRga`:
   - Deterministic character/block ID generation using `(client_id: u64, lamport_clock: u64, session_seq: u32)`.
   - Causal tree structure preserving maximal user syntactic intention without interleaving concurrent runs.
   - Non-destructive tombstones for deletion with garbage collection / compaction after causal stability.
   - Rich text formatting intervals using Peritext spans (bold, italic, font size, color, headings).
3. Connect `FugueTreeRga` with `word-engine` Piece-Tree Rope DOM to allow bidirectional conversion.
4. Add comprehensive unit tests in `crates/crdt-collab/tests/` verifying concurrent edit convergence, non-interleaving splits, and state-vector synchronization.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
