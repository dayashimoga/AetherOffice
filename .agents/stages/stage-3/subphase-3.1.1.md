# Subphase 3.1.1: WordprocessingML Piece-Tree Rope DOM & Sub-Document Parser
- **Stage:** 3 | **Phase:** 3.1
- **Target Crate:** `word-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Immutable B-Tree Piece-Tree DOM for word/document.xml, headers/footers, footnotes, comments, and tracked changes.
- **Validation Command:** `cargo test --package word-engine --test subphase_3_1_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 3.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine`, implement parsers and lossless serializers for `word/document.xml`, `word/header*.xml`, `word/footer*.xml`, `word/footnotes.xml`, `word/endnotes.xml`, and `word/comments.xml`.
2. Model paragraphs (`w:p`), runs (`w:r`), bookmarks (`w:bookmarkStart`/`End`), hyperlinks, and revision tracking (`w:ins`, `w:del`) over an immutable B-Tree Piece-Tree / Rope so keystroke insertion/deletion in a 1,000-page document completes in `< 0.5 ms`.
3. Write >= 25 round-trip and edit-latency test vectors in `crates/word-engine/tests/subphase_3_1_1_test.rs` and commit with `[GEC-Verified]`.
```
