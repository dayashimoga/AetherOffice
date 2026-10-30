description: Automated execution workflow for Stage 25: PDF Studio - Create, Open, In-Place Edit, Annotate, Forms & Sign
Workflow: Execute Stage 25 (/stage-25)
Objective: Stage 25: PDF Studio - Create, Open, In-Place Edit, Annotate, Forms & Sign

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 25.1.1: PDF Studio Workspace: Create Blank PDF, Open Local PDF & Page Management
Spec File: .agents/stages/stage-25/subphase-25.1.1.md
/execute-subphase 25.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Build the interactive **PDF Studio** module (pps/unified-shell/src/apps/pdf/) accessible from the Left App Rail and New Tab menu:
   - Start with a clean, editable blank PDF page (or click Open PDF to load and parse any .pdf file from disk via pdf-engine).
   - **Page Management Toolbar:** + Add Blank Page, Delete Page, Rotate 90 deg CW, Rotate 90 deg CCW, Move Page Up/Down, and left Page Thumbnails sidebar.
   - **Direct Text & Image Authoring on PDF:** Click + Add Text to place movable, editable text blocks anywhere on any PDF page (with Font, Size, Color, Bold/Italic controls); click + Add Image to stamp images onto a PDF page; click any existing text block in Edit Text mode to modify its text in-place!
2. Verify PDF page addition, rotation, and text block editing with automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 25.1.1
Subphase 25.1.2: PDF Studio Annotations, Freehand Ink, Form Builder/Filler, Signature & Export
Spec File: .agents/stages/stage-25/subphase-25.1.2.md
/execute-subphase 25.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v3.0.
1. Complete all remaining PDF Studio Ribbon tabs:
   - **Annotate Tab:** Highlight Box, Freehand Ink Pen (smooth HTML5 Canvas drawing overlay per page with color & stroke width picker + Eraser), Sticky Note (draggable comment pin with editable note text), and Redact Box (blackout rectangle).
   - **Forms Tab:** + Add Text Field and + Add Checkbox (places interactive form inputs onto the PDF page that users can type into and toggle).
   - **Sign & Export Tab:** Sign Document modal where users can draw their signature with mouse/touch or type their name in a cursive signature font and place the transparent signature stamp anywhere on the PDF page; Save / Export PDF button that compiles all pages, text, ink, form values, and signatures into a real downloadable/saveable .pdf binary via pdf-engine.
2. Verify via automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 25.1.2

