# Subphase 32.1.2: PDF Studio Complete Ribbon Overhaul: Home, Edit PDF, Forms, Comment, Page Tools, Protect, View
- **Stage:** 32 | **Phase:** 32.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** 100% working PDF editing, text/image modification, annotations, redaction, form builder/filler, page resizing, and password security.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 32.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v5.0.
1. Make **100% of PDF Studio's Ribbon tabs and tools** work seamlessly on the active PDF page:
   - **Home Tab:** Open PDF, Create Blank PDF, Save PDF, Print, Hand Tool (drag to scroll), Select Tool (click/drag to move/resize/edit any element on the page), Snapshot, Fit Page, Fit Width, Actual Size (100%), Zoom In (+), Zoom Out (-), First / Prev / Next / Last Page, and Search Text (highlights matching text across PDF pages).
   - **Edit PDF Tab:** Edit Text Mode (makes all text blocks on the PDF page directly editable in-place with Font Family, Font Size, Color, Bold, Italic, Align, and Delete controls!), + Add Text Box (places a new editable text box on the page), + Add Image (uploads and places a resizable image on the page), Crop Page, Header & Footer (adds custom top/bottom text & page numbers across pages), Watermark (adds customizable diagonal/centered watermark text with opacity & rotation), Background Color.
   - **Forms Tab:** + Text Field, + Checkbox, + Radio Button, + Dropdown List, + Date Picker, + Signature Field, Highlight Existing Fields, Clear All Form Fields, Export Form Data (JSON/CSV).
   - **Comment & Annotate Tab:** Highlight, Underline, Strikethrough, Sticky Note (adds comment pin + syncs with left Notes list), Text Callout, Freehand Ink Pen (draw directly over the PDF page with color & thickness controls), Eraser, Shapes (Rectangle, Circle, Arrow, Line), Stamps (APPROVED, CONFIDENTIAL, DRAFT, FINAL, SIGNED, URGENT).
   - **Page Tools Tab:** + Insert Blank Page, Duplicate Page, Delete Page, Rotate 90Â° CW, Rotate 90Â° CCW, Move Page Up/Down, **Resize Page Size (A4, Letter, Legal, A3, Custom Width Ã— Height with content scaling)**, Extract Page, Split PDF, Merge Another PDF.
   - **Sign & Protect Tab:** Add Signature (Draw/Type/Upload modal), Add Initials, Encrypt with Password (locks PDF with password + shows lock screen until unlocked), Decrypt / Remove Password, Restrict Editing, Mark for Redaction & Apply Redactions (permanently blacks out and deletes underlying content), Compress PDF.
2. Verify every single PDF button and modal with automated DOM tests. Commit with [GEC-Verified].
```
