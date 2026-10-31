description: Automated execution workflow for Stage 39: PDF Studio (.pdf) Fix - Local PDF Open/Save & Interactive On-Page Text/Image/Ink/Form Editor
Workflow: Execute Stage 39 (/stage-39)
Objective: Stage 39: PDF Studio (.pdf) Fix - Local PDF Open/Save & Interactive On-Page Text/Image/Ink/Form Editor

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v6.0) compliance:

Subphase 39.1.1: PDF Studio Real File Open/Save & Interactive Layer Canvas (Text, Images, Shapes, Ink, Forms)
Spec File: .agents/stages/stage-39/subphase-39.1.1.md
/execute-subphase 39.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v6.0 Articles II & III.
1. **Fix Why PDF Studio Appears Blank & Uneditable (`image_9756a4.png`):**
   - Change the default active tool in PDF Studio from `Hand` (pan-only) to **`Select / Edit`** so clicking on the PDF page immediately selects or creates editable elements!
   - **Opening Existing `.pdf` Files:** When a user selects a local `.pdf` file via `Open`:
     - Extract all pages, page dimensions, and text blocks from the PDF buffer (handling both standard PDF text operators `Tj`/`TJ` AND FlateDecode compressed streams via `DecompressionStream("deflate")` / `lopdf` IPC, plus rendering an embedded PDF preview canvas/object fallback so complex vector graphics and scanned PDFs also display faithfully!).
     - Populate each PDF page with interactive, selectable, editable text/image/annotation layers!
2. **Make Every PDF Editing Tool Immediately Interactive on the Page Canvas:**
   - **`Edit Text` / `+ Add Text`:** Clicking `+ Add Text` (or clicking on the page in `Add Text` mode) creates an interactive, draggable, resizable text box directly on the PDF page where the user can type, change Font Family, Font Size, Color, Bold, Italic, or Delete! Clicking any extracted text block lets the user edit its text in-place!
   - **`+ Add Image` / `Watermark` / `Header & Footer` / `Stamps`:** Immediately places the image, diagonal watermark, header/footer text, or stamp (`APPROVED`, `CONFIDENTIAL`, `DRAFT`, `SIGNED`) onto the PDF page where it can be dragged, resized, or deleted.
   - **`Highlight` / `Freehand Ink Draw` / `Sticky Note` / `Shapes` / `Redaction`:** Drawing with the mouse/touch on the PDF page creates live highlight boxes, smooth freehand ink paths (erasable with `Eraser`), sticky notes, shapes, or blackout redaction boxes!
   - **`Forms` (`+ Text Field`, `+ Checkbox`, `+ Dropdown`, `+ Date`):** Places real interactive HTML/PDF form inputs onto the page that can be typed into, checked, dragged, and saved!
   - **`Page Tools` (`+ Insert Page`, `Duplicate`, `Delete`, `Rotate 90°`, `Resize Page Size A4/Letter/Legal/Custom`, `Split`, `Merge`)** and **`Sign & Protect` (Custom Drawn/Typed Signature Pad, Password Encrypt/Decrypt, Compress)**.
3. Write automated DOM tests verifying opening a `.pdf` file, adding & editing text on Page 1, drawing ink, filling a form field, stamping a custom signature, resizing page dimensions, encrypting with a password, and saving the `.pdf`. Commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 39.1.1
