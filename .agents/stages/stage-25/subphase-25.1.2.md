# Subphase 25.1.2: PDF Studio Annotations, Freehand Ink, Form Builder/Filler, Signature & Export
- **Stage:** 25 | **Phase:** 25.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** Highlight, Freehand Draw, Sticky Notes, Redaction, Interactive Form Fields, Signature Pad, and valid .pdf file download/save.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 25.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Complete all remaining PDF Studio Ribbon tabs:
   - **Annotate Tab:** Highlight Box, Freehand Ink Pen (smooth HTML5 Canvas drawing overlay per page with color & stroke width picker + Eraser), Sticky Note (draggable comment pin with editable note text), and Redact Box (blackout rectangle).
   - **Forms Tab:** + Add Text Field and + Add Checkbox (places interactive form inputs onto the PDF page that users can type into and toggle).
   - **Sign & Export Tab:** Sign Document modal where users can draw their signature with mouse/touch or type their name in a cursive signature font and place the transparent signature stamp anywhere on the PDF page; Save / Export PDF button that compiles all pages, text, ink, form values, and signatures into a real downloadable/saveable .pdf binary via pdf-engine.
2. Verify via automated DOM tests and commit with [GEC-Verified].
```
