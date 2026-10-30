description: Automated execution workflow for Stage 21: Single-Process Memory Fix, Window Controls, App Rail & Dark/Light Theme
Workflow: Execute Stage 21 (/stage-21)
Objective: Stage 21: Single-Process Memory Fix, Window Controls, App Rail & Dark/Light Theme

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 21.1.1: Single-Process Consolidation, Lazy Memory State & Native Window Controls
Spec File: .agents/stages/stage-21/subphase-21.1.1.md
/execute-subphase 21.1.1
Invoke @adaptive-ui-builder under GEC-v3.0 Article I & III.
1. **Fix Memory Hang & Consolidate Binary:**
   - Ensure pps/unified-shell/src-tauri/Cargo.toml builds ONLY ONE binary (AetherOffice) so opening multiple documents never spawns multiple WebView2 Chromium instances.
   - Audit pps/unified-shell/src-tauri/src/ and crates/ to ensure no giant vectors or 1M-cell arrays are allocated on boot. Use lazy HashMap storage initialized empty.
2. **Fix Window Minimize, Maximize & Close Buttons:**
   - In pps/unified-shell/src-tauri/tauri.conf.json, enable native window controls ("decorations": true) OR wire the custom titlebar buttons (_, [], X) directly to Tauri v2's window API (getCurrentWindow().minimize(), getCurrentWindow().toggleMaximize(), getCurrentWindow().close()) with fallback window.close().
3. **Purge All Hardcoded Fake Demo Text:**
   - Remove "OpenOfficeRS â€” Executive Technical Summary", "Performance SLAs & Benchmark Compliance", "Universal Adaptive Form-Factor Parity", and all prepopulated numbers/shapes from the codebase so Gate 2 of 	ools/verify-contract.ps1 passes.
4. Commit with eat(subphase-21.1.1): single-process memory fix and working window controls [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 21.1.1
Subphase 21.1.2: Unified Left App Switcher Rail, Home Hub & Multi-Document Tab Router
Spec File: .agents/stages/stage-21/subphase-21.1.2.md
/execute-subphase 21.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Build a unified Microsoft 365 / WPS-style **Left App Switcher Rail** (48px wide vertical dock on the left edge of the window) with icons and tooltips for:
   - **Home Hub** (Dashboard with 5 large 'Create New' cards: Blank Word .docx, Blank Excel .xlsx, Blank PowerPoint .pptx, Open/Blank PDF .pdf, and Blank Note .txt/.md).
   - **Word (W blue icon)** â€” Instantly creates/switches to a blank editable .docx tab.
   - **Excel (X green icon)** â€” Instantly creates/switches to a blank editable .xlsx tab (Book1.xlsx), NEVER opening .docx by mistake!
   - **PowerPoint (P orange icon)** â€” Instantly creates/switches to a blank editable .pptx tab (Presentation1.pptx).
   - **PDF Studio (PDF red icon)** â€” Instantly creates/switches to a PDF Studio tab (Document.pdf).
   - **Notepad / Notes (N purple icon)** â€” Instantly creates/switches to a clean Notepad/Markdown tab (Untitled.txt).
2. Fix the top + New Tab button: clicking + opens a quick dropdown menu to launch a blank Word, Excel, PowerPoint, PDF, or Note tab, and clicking x on any tab closes that tab cleanly.
3. Verify via automated DOM tests that launching Excel, PPT, PDF, or Notes renders its exact respective engine, never .docx. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 21.1.2
Subphase 21.2.1: Visual Overhaul: Non-Overflowing Flexbox Ribbon & Dark/Light Theme Toggle
Spec File: .agents/stages/stage-21/subphase-21.2.1.md
/execute-subphase 21.2.1
Invoke @adaptive-ui-builder under GEC-v3.0 Article III.
1. **Fix Ribbon Overflow & Alignment Bug:**
   - Remove the broken overflow: scroll / fixed small height on the Ribbon toolbar that caused ugly scrollbars to cover the buttons.
   - Redesign the Ribbon using a clean, modern single-row or compact two-row CSS Flexbox layout (min-height: 88px, overflow: visible, crisp 1px solid var(--border) group dividers, properly aligned 28px buttons and dropdowns).
   - Fix the tab highlight bug: File must only be highlighted when the File Backstage menu is open; when Home, Insert, Layout, etc. are clicked, ONLY the clicked tab shows the active underline/pill.
2. **Implement Global Dark / Light Theme Toggle (Light / Dark):**
   - Add a prominent Theme Toggle button in the top titlebar (Light Mode vs Dark Mode).
   - Define full CSS custom properties for Light (#f8fafc app bg, #ffffff ribbon/surface, #0f172a text) and Dark (#0f172a app bg, #1e293b ribbon/surface, #f8fafc text, #334155 borders) so the entire UI flips smoothly with zero unreadable text.
3. Verify with DOM tests that toggling the theme updates document.documentElement.dataset.theme and that every Ribbon tab switches cleanly. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 21.2.1

