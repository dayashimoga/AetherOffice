# Subphase 32.1.1: Fix Local PDF File Open/Render/Save & Replace Hardcoded 'John Doe' with Real Signature Pad
- **Stage:** 32 | **Phase:** 32.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** Open any local .pdf file and render its pages/text, save edited .pdf, and interactive Signature/Initials Pad (Draw, Type, or Upload Image) with zero 'John Doe' defaults.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
