description: Automated execution workflow for Stage 13: End-to-End Automated UI Verification & 5-Binary Release Packaging
Workflow: Execute Stage 13 (/stage-13)
Objective: Stage 13: End-to-End Automated UI Verification & 5-Binary Release Packaging

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 13.1.1: Comprehensive Automated E2E UI Test Suite Across All 5 Apps
Spec File: .agents/stages/stage-13/subphase-13.1.1.md
/execute-subphase 13.1.1
Invoke @adaptive-ui-builder under GEC-v2.0 Article VII.4.
1. Create a comprehensive automated UI verification suite in pps/unified-shell/src/__tests__/full_suite_e2e.test.tsx that programmatically verifies:
   - **Hub:** Launching Writer, Calc, Slides, and PDF independently.
   - **Writer:** Typing paragraphs, inserting a new page (Blank Page), applying Bold/Heading 1/Table, and saving .docx.
   - **Calc:** Editing cell A1 (100) and A2 (200), entering =SUM(A1:A2) in A3, asserting 300 renders, adding Sheet2, sorting, and inserting a Chart.
   - **Slides:** Adding a slide, inserting a Star shape, dragging/resizing the shape, editing text inline, applying a Transition, and starting Fullscreen Slide Show.
   - **PDF:** Creating/opening a PDF, editing text in-place, adding a Page, filling a Form field, placing a Signature, and saving .pdf.
2. Run 	ools/verify-contract.ps1 and ensure 100% pass rate with zero skipped tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 13.1.1
Subphase 13.1.2: Final 5-Binary Release Build & GitHub Actions Multi-Platform Verification
Spec File: .agents/stages/stage-13/subphase-13.1.2.md
/execute-subphase 13.1.2
Execute final GEC-v2.0 Release Audit:
1. Update .github/workflows/build-all-platforms.yml so desktop builds package and upload all 5 executables (ether-hub, ether-writer, ether-calc, ether-slides, ether-pdf) alongside the unified installer bundle.
2. Run powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1 to verify all Rust crates, Clippy, Formatting, Anti-Placeholder scans, and Frontend E2E UI tests pass 100%.
3. Commit, tag 2.0.0, and push to origin.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 13.1.2
