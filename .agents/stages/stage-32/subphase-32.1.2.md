# Subphase 32.1.2: PDF Direct Text/Image Editing, Annotations, Redaction, Forms & Signatures
- **Stage:** 32 | **Phase:** 32.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** 100% working Home, Edit PDF, Forms, Comment & Annotate, Sign, and View tabs in PDF Studio.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 32.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** across PDF Studio's Home, Edit PDF, Forms, Comment & Annotate, Sign & Protect, and View tabs:
   - **Home Tab:** Make Open, Hand (drag-pan canvas), Select (select text/objects), Snapshot (captures selected region or current page as PNG to clipboard/download), 100%, Fit Page, Fit Width, Prev Page, Next Page, + In (Zoom In), - Out (Zoom Out), Search (opens live PDF text search bar highlighting matches across pages), Print, and Save (generates and downloads valid .pdf file) 100% functional!
   - **Edit PDF Tab:** Edit Text & Objects mode (click any text block on the PDF page to edit its text, font, size, or color in-place, drag to reposition, or press Delete), + Add Text (click anywhere on the page to create a new formatted text block), + Add Image (upload and resize image on the PDF page), Add Links, Header & Footer (stamps page numbers/dates across all pages), Watermark (diagonal or centered text/image watermark with opacity slider), and Background Color.
   - **Comment & Annotate Tab:** Highlight, Underline, Strikethrough, Sticky Note (places interactive comment marker + syncs with left Notes tab), Callout Text Box, Freehand Draw / Pencil (smooth ink drawing on the PDF canvas with color/thickness picker), Eraser, Shapes (Rect, Circle, Arrow, Line), Stamps (APPROVED, CONFIDENTIAL, DRAFT, REVIEWED, FINAL, VOID), and Redact Content (draws blackout redaction box that permanently strips underlying text/objects).
   - **Forms & Sign Tabs:** + Text Field, + Checkbox, + Radio Button, + Dropdown, + Date Field, + Clear Form, Fill Form Mode, and Place Digital Signature (draw signature on canvas pad or type cursive signature + date stamp, and place resizable signature onto the PDF).
2. Verify every single PDF tool and ribbon button with automated DOM tests. Commit with [GEC-Verified].
```
