# Subphase 21.1.1: Single-Process Consolidation, Lazy Memory State & Native Window Controls
- **Stage:** 21 | **Phase:** 21.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Single AetherOffice.exe binary (<45MB RAM), working Minimize/Maximize/Close buttons, and removal of all background CPU/RAM leaks.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
