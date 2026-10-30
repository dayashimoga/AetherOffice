description: Automated execution workflow for Stage 27: Universal File Open/Save/Print & Full E2E Verification
Workflow: Execute Stage 27 (/stage-27)
Objective: Stage 27: Universal File Open/Save/Print & Full E2E Verification

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 27.1.1: Universal File Open/Save/Export (.docx, .xlsx, .pptx, .pdf, .txt) & Shortcuts
Spec File: .agents/stages/stage-27/subphase-27.1.1.md
/execute-subphase 27.1.1
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v3.0.
1. Wire the File Backstage Menu and global keyboard shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+P, Ctrl+Z, Ctrl+Y, Ctrl+F) so they work across all 5 modules (Word, Excel, PowerPoint, PDF, Notepad):
   - **Open File (Ctrl+O):** Uses native <input type="file"> + Tauri file dialog to open real .docx, .xlsx, .csv, .pptx, .pdf, .txt, and .md files from the user's computer and load their contents into the appropriate editor tab.
   - **Save / Download File (Ctrl+S):** Serializes the current active tab via the Rust backend (or browser Blob download fallback) into a valid .docx, .xlsx, .csv, .pptx, .pdf, or .txt file on the user's machine.
   - **Print (Ctrl+P):** Clean print stylesheet that prints only the active document canvas (window.print()).
2. Verify round-trip Save & Open across all 5 formats with automated tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 27.1.1
Subphase 27.1.2: Final GEC-v3.0 E2E Verification & Single-App Release Packaging
Spec File: .agents/stages/stage-27/subphase-27.1.2.md
/execute-subphase 27.1.2
Execute final GEC-v3.0 Release Audit:
1. Write a comprehensive Vitest DOM test suite (pps/unified-shell/src/__tests__/gec_v3_overhaul.test.tsx) asserting:
   - Zero prepopulated fake strings appear in any newly created Word, Excel, PowerPoint, PDF, or Notepad tab.
   - Switching via the Left App Rail opens Word, Excel, PowerPoint, PDF, and Notepad accurately (never defaulting to .docx).
   - Word allows typing, formatting, and creating Page 2 (+ New Page).
   - Excel allows typing into cells, evaluating =SUM(A1:A2), and adding Sheet2.
   - PowerPoint allows adding slides, typing in text boxes, and inserting shapes.
   - PDF Studio allows adding pages, adding/editing text boxes, drawing annotations, and filling forms.
   - Notepad allows typing plain text/Markdown and toggling Dark/Light theme.
2. Run powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1 and verify a clean single-binary release build (cargo build --release --jobs 2). Commit and push with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 27.1.2

