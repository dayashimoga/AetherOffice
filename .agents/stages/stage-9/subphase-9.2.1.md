# Subphase 9.2.1: Writer Home & Layout Ribbons: Rich Run Formatting, Lists & Styles
- **Stage:** 9 | **Phase:** 9.2
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Working Font family/size dropdowns, Color pickers, Bold/Italic/Underline/Strike, Bullets/Numbering, Line Spacing, and Style gallery.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 9.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every control in WriterRibbon.tsx (Home and Layout tabs) to mutate the selected text range or active cursor formatting in word-engine:
   - **Font Group:** Font Family dropdown (Calibri, Arial, Times New Roman, Inter, Consolas, Georgia), Font Size dropdown (8 to 72), Grow/Shrink Font (A+, A-), Text Color picker, Highlight Color picker, and Clear Formatting button.
   - **Paragraph Group:** Toggle Bullet List, Toggle Numbered List (1., ., i.), Decrease/Increase Indent, Left/Center/Right/Justify alignment, and Line Spacing (1.0, 1.15, 1.5, 2.0).
   - **Styles Gallery:** Clicking Normal, Heading 1, Heading 2, Heading 3, Title, or Subtitle applies the style to the current paragraph and automatically updates the left **Navigation Outline Pane** with clickable heading links!
   - **Layout Tab:** Margins preset selector (Normal, Narrow, Wide), Orientation (Portrait / Landscape), Paper Size (A4 / Letter), and Columns (1, 2, 3).
2. Verify all formatting actions with automated DOM tests and commit with [GEC-Verified].
`
