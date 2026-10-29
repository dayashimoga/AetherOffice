description: Automated execution workflow for Stage 9: Standalone Writer (Word) - Real Typing, Multi-Page Creation & Full Ribbon
Workflow: Execute Stage 9 (/stage-9)
Objective: Stage 9: Standalone Writer (Word) - Real Typing, Multi-Page Creation & Full Ribbon

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 9.1.1: Interactive Paginated Word Canvas, Real Caret & Keyboard Typing
Spec File: .agents/stages/stage-9/subphase-9.1.1.md
/execute-subphase 9.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Replace the static non-editable text block in Writer (pps/unified-shell/src/apps/writer/) with a true interactive multi-page document editor synchronized with word-engine:
   - Support full keyboard input (keydown, InputEvent, IME composition, Backspace, Delete, Enter for new paragraph, Shift+Enter for line break, Arrow/Home/End/PageUp/PageDown cursor navigation).
   - Support mouse click to place the blinking caret at the exact character offset, double-click to select word, triple-click to select paragraph, and click-drag to highlight text ranges across runs and paragraphs.
2. Ensure every edit dispatches UiActionEvent to word-engine and immediately re-renders the updated paragraphs on screen.
3. Write automated DOM tests simulating typing a 3-paragraph document, selecting text, and deleting characters. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 9.1.1
Subphase 9.1.2: Multi-Page Creation, Page Breaks, Headers/Footers & Zoom/Ruler
Spec File: .agents/stages/stage-9/subphase-9.1.2.md
/execute-subphase 9.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement multi-page layout and creation in Writer:
   - **Automatic Pagination:** When paragraphs exceed the vertical body height of Page 1 (11in minus top/bottom margins), automatically flow overflow paragraphs onto Page 2, Page 3, etc., rendered as distinct visual paper sheets separated by a page gap.
   - **Manual Page Creation:** Wire Insert -> Blank Page, Insert -> Page Break, and Ctrl+Enter to immediately insert a new page and move the cursor to the top of the new page.
   - **Editable Headers & Footers:** Double-clicking the top or bottom margin of any page activates the Header/Footer editor with dynamic Page X of Y numbering.
   - **Interactive Horizontal Ruler:** Dragging the left/right indent markers on the top ruler dynamically updates page margins and paragraph indentation.
2. Write DOM tests verifying that Insert -> Blank Page and Ctrl+Enter increase the rendered .word-page count from 1 to 2+. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 9.1.2
Subphase 9.2.1: Writer Home & Layout Ribbons: Rich Run Formatting, Lists & Styles
Spec File: .agents/stages/stage-9/subphase-9.2.1.md
/execute-subphase 9.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every control in WriterRibbon.tsx (Home and Layout tabs) to mutate the selected text range or active cursor formatting in word-engine:
   - **Font Group:** Font Family dropdown (Calibri, Arial, Times New Roman, Inter, Consolas, Georgia), Font Size dropdown (8 to 72), Grow/Shrink Font (A+, A-), Text Color picker, Highlight Color picker, and Clear Formatting button.
   - **Paragraph Group:** Toggle Bullet List, Toggle Numbered List (1., ., i.), Decrease/Increase Indent, Left/Center/Right/Justify alignment, and Line Spacing (1.0, 1.15, 1.5, 2.0).
   - **Styles Gallery:** Clicking Normal, Heading 1, Heading 2, Heading 3, Title, or Subtitle applies the style to the current paragraph and automatically updates the left **Navigation Outline Pane** with clickable heading links!
   - **Layout Tab:** Margins preset selector (Normal, Narrow, Wide), Orientation (Portrait / Landscape), Paper Size (A4 / Letter), and Columns (1, 2, 3).
2. Verify all formatting actions with automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 9.2.1
Subphase 9.2.2: Writer Insert, Review & View Ribbons: Tables, Images, Find/Replace & Word Count
Spec File: .agents/stages/stage-9/subphase-9.2.2.md
/execute-subphase 9.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the Insert, Review, and View ribbons in Writer:
   - **Insert Tab:** Insert Table (interactive RxC picker + ability to type inside cells, add/delete rows & columns), Insert Picture (from local file/data URL with resize handles), Insert Shape, Insert Hyperlink, Insert Comment, and Insert Page Number.
   - **Review Tab:** Live Word/Character/Page Count modal, Track Changes toggle (highlights insertions in underlined blue and deletions in strikethrough red), Accept/Reject Change buttons, and New/Delete Comment sidebar.
   - **View Tab & Status Bar:** Print Layout vs Web Layout vs Mobile Reflow mode toggle, Ruler visibility checkbox, Navigation Pane checkbox, Find & Replace bar (Ctrl+F / Ctrl+H), and bottom Status Bar showing live Page X of Y, N words, and Zoom slider (50% to 200%).
2. Verify via automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 9.2.2
