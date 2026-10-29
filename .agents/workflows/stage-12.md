description: Automated execution workflow for Stage 12: Standalone PDF Studio - Multi-Page PDF Read, In-Place Edit, Forms & Signing
Workflow: Execute Stage 12 (/stage-12)
Objective: Stage 12: Standalone PDF Studio - Multi-Page PDF Read, In-Place Edit, Forms & Signing

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 12.1.1: Standalone PDF Studio Workspace, Multi-Page Renderer & Page Organizer
Spec File: .agents/stages/stage-12/subphase-12.1.1.md
/execute-subphase 12.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Build the dedicated **PDF Studio** workspace (pps/unified-shell/src/apps/pdf/PdfWorkspace.tsx) accessible via ether-pdf and ether-hub:
   - Support creating a new multi-page PDF from scratch OR opening any existing .pdf file via pdf-engine.
   - Left Sidebar with 3 tabs: **Page Thumbnails** (click to jump, drag to reorder), **Bookmarks / Outline Tree**, and **Annotations List**.
   - **Page Tools Ribbon:** + Insert Blank Page, Delete Page, Rotate Clockwise 90°, Rotate Counter-Clockwise 90°, Move Page Up/Down, Extract Page, and Merge PDF.
   - Continuous scrolling, Single-Page fit, Two-Page spread, and Zoom controls (25% to 400%).
2. Write automated DOM tests verifying PDF page creation, rotation, and deletion. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 12.1.1
Subphase 12.1.2: PDF In-Place Text/Image Editor, Annotations, Redaction & AcroForms Filler
Spec File: .agents/stages/stage-12/subphase-12.1.2.md
/execute-subphase 12.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Wire all editing, form, and signing tools in PdfRibbon.tsx directly to pdf-engine:
   - **Edit PDF Tab:** Edit Existing Text mode (clicking any PDF text block turns it into an editable bounding box that updates the underlying PDF content stream in pdf-engine), + Add Text Box, + Add Image, and Add Watermark.
   - **Comment & Annotate Tab:** Highlight Text, Underline, Strikethrough, Sticky Note (with pop-up comment popover), Freehand Ink Pen (draw directly on the PDF canvas with color/thickness picker), Eraser, and Redact Area (permanently blacks out and removes underlying text).
   - **Forms & Sign Tab:** Interactive filling of /AcroForm fields (Text input, Checkbox, Radio, Dropdown), **Add Form Field** tool (place new interactive text/checkbox fields onto any PDF page), and **Sign Document Modal** (draw signature with mouse/stylus or type signature and stamp onto the page, followed by Incremental Save via pdf-engine).
2. Write automated DOM tests verifying PDF text editing, form filling, signature placement, and binary PDF export. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 12.1.2
