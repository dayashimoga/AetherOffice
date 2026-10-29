# Subphase 9.2.2: Writer Insert, Review & View Ribbons: Tables, Images, Find/Replace & Word Count
- **Stage:** 9 | **Phase:** 9.2
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Interactive Table grid insertion/editing, Image upload, Hyperlinks, Find & Replace dialog, Track Changes toggle, and Status Bar word count.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 9.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the Insert, Review, and View ribbons in Writer:
   - **Insert Tab:** Insert Table (interactive RxC picker + ability to type inside cells, add/delete rows & columns), Insert Picture (from local file/data URL with resize handles), Insert Shape, Insert Hyperlink, Insert Comment, and Insert Page Number.
   - **Review Tab:** Live Word/Character/Page Count modal, Track Changes toggle (highlights insertions in underlined blue and deletions in strikethrough red), Accept/Reject Change buttons, and New/Delete Comment sidebar.
   - **View Tab & Status Bar:** Print Layout vs Web Layout vs Mobile Reflow mode toggle, Ruler visibility checkbox, Navigation Pane checkbox, Find & Replace bar (Ctrl+F / Ctrl+H), and bottom Status Bar showing live Page X of Y, N words, and Zoom slider (50% to 200%).
2. Verify via automated DOM tests and commit with [GEC-Verified].
`
