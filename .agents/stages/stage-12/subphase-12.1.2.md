# Subphase 12.1.2: PDF In-Place Text/Image Editor, Annotations, Redaction & AcroForms Filler
- **Stage:** 12 | **Phase:** 12.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** Direct in-place editing of PDF text blocks, adding new text/images, Highlight/Sticky Note/Freehand Ink annotations, Form Builder/Filler, and Signature pad.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 12.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Wire all editing, form, and signing tools in PdfRibbon.tsx directly to pdf-engine:
   - **Edit PDF Tab:** Edit Existing Text mode (clicking any PDF text block turns it into an editable bounding box that updates the underlying PDF content stream in pdf-engine), + Add Text Box, + Add Image, and Add Watermark.
   - **Comment & Annotate Tab:** Highlight Text, Underline, Strikethrough, Sticky Note (with pop-up comment popover), Freehand Ink Pen (draw directly on the PDF canvas with color/thickness picker), Eraser, and Redact Area (permanently blacks out and removes underlying text).
   - **Forms & Sign Tab:** Interactive filling of /AcroForm fields (Text input, Checkbox, Radio, Dropdown), **Add Form Field** tool (place new interactive text/checkbox fields onto any PDF page), and **Sign Document Modal** (draw signature with mouse/stylus or type signature and stamp onto the page, followed by Incremental Save via pdf-engine).
2. Write automated DOM tests verifying PDF text editing, form filling, signature placement, and binary PDF export. Commit with [GEC-Verified].
`
