description: Automated execution workflow for Stage 32: PDF Studio (.pdf) Fix - Local PDF Open/Save, Custom Drawn/Typed Signature Pad and Full Editing
Workflow: Execute Stage 32 (/stage-32)
Objective: Stage 32: PDF Studio (.pdf) Fix - Local PDF Open/Save, Custom Drawn/Typed Signature Pad and Full Editing

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 32.1.1: Fix Local PDF File Open/Render/Save & Replace Hardcoded 'John Doe' with Real Signature Pad
- Spec File: .agents/stages/stage-32/subphase-32.1.1.md
/execute-subphase 32.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v5.0 Article III.4.
1. **Remove Hardcoded \"John Doe\" Signature Bug (Seen in image_b643a0.png) & Build Real Signature Modal:**
   - Delete \"John Doe\" completely from the codebase!
   - When the user clicks Add Signature, Initials, or Sign in the Sign & Protect tab:
     - Open an interactive **Signature & Initials Studio Modal** with 3 tabs:
       1. **Draw Signature:** An HTML5 <canvas> pad where the user draws their real signature with mouse, trackpad, or touch/stylus (with ink color picker: Black, Classic Blue, Fountain Red, and Clear button). Captures the exact drawn stroke as a transparent PNG (canvas.toDataURL(\"image/png\")) and places it as a draggable, resizable signature stamp on the active PDF page!
       2. **Type Signature:** An <input type=\"text\" placeholder=\"Enter your full name or initials...\" /> where the user types their own name (e.g., \"Dayanand T.\"), chooses from 4 signature script styles, and stamps their exact typed name + optional timestamp onto the PDF page!
       3. **Upload Signature Image:** Upload a scan/photo of a real signature (.png/.jpg) and stamp it onto the page.
2. **Fix Opening & Saving Real Local .pdf Files:**
   - When the user clicks Open (in File, Home, or top quick bar) and selects any .pdf file from their computer:
     - Parse the binary ArrayBuffer using pdf-engine / client PDF parser to extract the actual pages, page dimensions, embedded text blocks (BT..ET / Tj / TJ), and render high-resolution page views + thumbnails!
   - When clicking Save or Export PDF, serialize all pages, edited text blocks, added shapes/images, form values, redactions, and drawn/typed signature images into a valid .pdf binary and download/save it to disk!
3. Write automated tests verifying local .pdf buffer loading, drawing a signature on the canvas pad, typing a custom name (\"Custom Signer\"), asserting \"John Doe\" never appears, and saving the .pdf. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 32.1.1

### Subphase 32.1.2: PDF Studio Complete Ribbon Overhaul: Home, Edit PDF, Forms, Comment, Page Tools, Protect, View
- Spec File: .agents/stages/stage-32/subphase-32.1.2.md
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

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 32.1.2

