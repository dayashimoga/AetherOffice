# Stage 46: Advanced Word Processing & Publishing Workflows
## Sub-Phase 46.1.1: Revision Tracking (w:ins, w:del, w:rPrChange) & Redline Renderer

### 1. Context & Objectives
Under GEC-v7.0 Articles I, III, IV, and VI:
- WordprocessingML revision tracking (ECMA-376 Part 1 §17.13) must be fully supported across the document DOM, layout engine, and serializer with zero data loss.
- Must support 4 distinct revision types:
  - `<w:ins>`: Tracked insertions with author, date, and run children.
  - `<w:del>`: Tracked deletions with author, date, and `<w:delText>` runs.
  - `<w:rPrChange>`: Tracked run formatting changes preserving the previous `<w:rPr>`.
  - `<w:pPrChange>`: Tracked paragraph formatting changes preserving the previous `<w:pPr>`.
- Redline layout engine must support 4 display modes:
  - `SimpleMarkup`: Displays subtle vertical margin change bars, deletions hidden, insertions displayed with normal layout.
  - `AllMarkup`: Full visual redline: insertions underlined in author color, deletions shown inline struck-through in author color, property revisions indicated.
  - `NoMarkup`: Displays final document text as if all revisions were accepted.
  - `Original`: Displays original document text as if all revisions were rejected.
- Revision management operations:
  - Individual `accept_revision` and `reject_revision`.
  - Bulk `accept_all_revisions` and `reject_all_revisions`.
  - Deterministic author color palette assignment.

### 2. Implementation Tasks
1. **DOM Models (`crates/word-engine/src/model.rs`)**:
   - Add `RunPropertiesChange` (`<w:rPrChange>`).
   - Add `ParagraphPropertiesChange` (`<w:pPrChange>`).
   - Wire into `RunProperties` and `ParagraphProperties`.
2. **Revision Engine (`crates/word-engine/src/revision.rs`)**:
   - `RevisionType`, `RevisionItem`, `RevisionViewMode`, `AuthorColorPalette`.
   - `RevisionEngine`: inspection, individual & batch accept/reject with PieceTree synchronization.
3. **Redline Layout Renderer (`crates/word-engine/src/layout.rs`)**:
   - Tokenize `Inserted`, `Deleted`, and `Hyperlink` paragraph children.
   - Decorate `LayoutRun` with `underline`, `strike`, `revision_type`, and `revision_author`.
   - Add margin change bar calculation (`has_change_bar: bool`).
4. **Parser & Serializer (`parser.rs`, `serializer.rs`)**:
   - Parse and serialize `<w:rPrChange>` and `<w:pPrChange>`.
5. **Frontend State & Test Suites**:
   - Rust integration test suite `crates/word-engine/tests/subphase_46_1_1_test.rs`.
   - Vitest suite `apps/unified-shell/src/__tests__/word_revision_tracking.test.ts`.
   - Contract verification with `tools/verify-contract.ps1`.

