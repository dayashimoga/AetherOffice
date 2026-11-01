# Sub-Phase 46.1.2 Specification: Anchored Threaded Comments & Resolved History

## Goal
Implement full ECMA-376 / ISO 29500 anchored threaded comments and resolution history across `word-engine` and the presentation shell, supporting nested replies, resolution tracking (`done="1"`), anchor range linking (`w:commentRangeStart`, `w:commentRangeEnd`, `w:commentReference`), and side-gutter card alignment.

## Technical Requirements

### 1. Domain Model & OOXML Integrity (`word-engine`)
- Parse and serialize `word/comments.xml` and `word/commentsExtended.xml` (`w15:commentEx`).
- Link paragraph anchors (`w:commentRangeStart`, `w:commentRangeEnd`, `w:commentReference`) to comment instances.
- Threaded hierarchy: parent comment ID (`paraIdParent`), replies list, resolution status (`is_resolved` / `done`), author initials, timestamps.
- Zero data loss: preserve unmodeled XML attributes and formatting inside comments.

### 2. Comment Engine (`crates/word-engine/src/comments.rs`)
- `CommentThread`: contains root comment and nested chronological replies.
- `CommentEngine`:
  - `list_threads(doc: &Document, comments_doc: &CommentsDocument) -> Vec<CommentThread>`
  - `add_comment(doc: &mut Document, comments_doc: &mut CommentsDocument, range_start: usize, range_end: usize, author: &str, text: &str) -> String`
  - `add_reply(comments_doc: &mut CommentsDocument, parent_id: &str, author: &str, text: &str) -> String`
  - `set_resolved(comments_doc: &mut CommentsDocument, thread_id: &str, resolved: bool) -> bool`
  - `delete_thread(doc: &mut Document, comments_doc: &mut CommentsDocument, thread_id: &str) -> bool`

### 3. Layout & Presentation Shell
- Anchor position tracking: baseline $Y$-coordinate calculation for margin callouts.
- Responsive side-gutter comment stream with collapse/expand toggles.
- Filter: All Comments, Unresolved Only, Mentioning Me.

### 4. Safety & Standards
- `#![forbid(unsafe_code)]` enforced.
- Zero unhandled panics, unwraps, or expects in library code.
- Automated gatekeeper `tools/verify-contract.ps1` must pass 100%.

