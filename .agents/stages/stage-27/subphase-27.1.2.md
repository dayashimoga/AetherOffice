# Subphase 27.1.2: Final GEC-v3.0 E2E Verification & Single-App Release Packaging
- **Stage:** 27 | **Phase:** 27.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% passing E2E interactive tests across Word, Excel, PPT, PDF, and Notepad + clean single-binary release build.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
