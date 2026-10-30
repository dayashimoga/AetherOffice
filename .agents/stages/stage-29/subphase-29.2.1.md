# Subphase 29.2.1: Fix Word References, Review & View Ribbons (100% Working Controls)
- **Stage:** 29 | **Phase:** 29.2
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Every single button in Word's References, Review, and View ribbons wired and verified via automated tests.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 29.2.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Fix and wire **100% of the buttons** in Word's References, Review, and View ribbons:
   - **References Tab:** Table of Contents (automatically scans all Heading 1/2/3 elements in the document and inserts a clickable Table of Contents block at the top of the document + Update Table), Insert Footnote (inserts a superscript [1] reference at the cursor and appends a numbered footnote entry in the page footer area), Insert Endnote, Insert Citation & Insert Bibliography (APA/MLA/IEEE citation manager modal), and Insert Caption (for figures/tables).
   - **Review Tab:** Spelling & Grammar check drawer, Thesaurus lookup, Word Count modal (showing Pages, Words, Characters no spaces, Characters with spaces, Paragraphs, Lines), Read Aloud (uses browser window.speechSynthesis TTS to read the document aloud with Play/Pause/Stop!), Translate Selection, New Comment (attaches a comment card in a right-hand Comments gutter), Delete/Next/Prev Comment, Track Changes toggle (records subsequent typing in <ins> and deletions in <del>), Accept All Changes, Reject All Changes, and Protect Document (Read-Only Lock).
   - **View Tab:** Print Layout (standard paged sheets), Web Layout (full-width fluid canvas), Read Mode (distraction-free two-column book reader), Focus Mode (hides ribbon), Ruler checkbox, Gridlines checkbox (renders subtle alignment grid overlay on the page), Navigation Pane checkbox, Dark Page Canvas toggle, Zoom (50%, 75%, 100%, 150%, 200%), One Page, Multi Pages (side-by-side pages), and Page Width.
2. Verify every button in References, Review, and View with automated Vitest DOM assertions. Commit with [GEC-Verified].
```
