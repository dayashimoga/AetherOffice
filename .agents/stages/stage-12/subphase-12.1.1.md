# Subphase 12.1.1: Standalone PDF Studio Workspace, Multi-Page Renderer & Page Organizer
- **Stage:** 12 | **Phase:** 12.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** Dedicated PDF Studio UI with multi-page canvas, Thumbnails/Bookmarks sidebar, Page Add/Delete/Rotate/Reorder/Merge, and Create Blank PDF.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 12.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v2.0.
1. Build the dedicated **PDF Studio** workspace (pps/unified-shell/src/apps/pdf/PdfWorkspace.tsx) accessible via ether-pdf and ether-hub:
   - Support creating a new multi-page PDF from scratch OR opening any existing .pdf file via pdf-engine.
   - Left Sidebar with 3 tabs: **Page Thumbnails** (click to jump, drag to reorder), **Bookmarks / Outline Tree**, and **Annotations List**.
   - **Page Tools Ribbon:** + Insert Blank Page, Delete Page, Rotate Clockwise 90°, Rotate Counter-Clockwise 90°, Move Page Up/Down, Extract Page, and Merge PDF.
   - Continuous scrolling, Single-Page fit, Two-Page spread, and Zoom controls (25% to 400%).
2. Write automated DOM tests verifying PDF page creation, rotation, and deletion. Commit with [GEC-Verified].
`
