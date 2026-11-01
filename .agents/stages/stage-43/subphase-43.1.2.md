# Subphase 43.1.2: Zero-Loss OPC Round-Trip Bridge

- **Stage:** 43 | **Phase:** 43.1
- **Target Subsystem:** `ooxml-opc` / `render-core` / `unified-shell-tauri` / `ipc.ts`
- **Primary Agent Skill:** `@ooxml-architect`
- **Deliverables:**
  1. Enhance `GlobalOfficeState` and `DocumentIoEngine` in `crates/render-core` to retain parsed package representations (`WordPackage`, `ExcelWorkbookPackage`, `PresentationPackage`) and `UnparsedPartStore` across session lifetimes.
  2. Implement zero-loss save operations preserving unmodeled parts (`vbaProject.bin`, drawings, themes, settings, fonts, custom XML, digital signatures) upon document save (`save_document_to_disk`).
  3. Implement in-memory OPC byte roundtrip engines:
     - `roundtrip_docx_bytes(original_bytes: &[u8], updated_content: &str) -> Result<Vec<u8>, String>`
     - `roundtrip_xlsx_bytes(original_bytes: &[u8], updated_cells: &HashMap<String, String>) -> Result<Vec<u8>, String>`
     - `roundtrip_pptx_bytes(original_bytes: &[u8]) -> Result<Vec<u8>, String>`
  4. Expose `roundtrip_opc_binary` Tauri command in `apps/unified-shell/src-tauri` and wire `IPC.roundtripOpcBinary` in `apps/unified-shell/src/ipc.ts`.
  5. Full verification with Rust unit tests in `crates/render-core/tests/subphase_43_1_2_test.rs` and frontend Vitest suite `apps/unified-shell/src/__tests__/opc_roundtrip_bridge.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

