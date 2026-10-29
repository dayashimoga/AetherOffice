# Subphase 15.1.1: OLE Compound Binary Reader & vbaProject.bin Decompressor
- **Stage:** 15 | **Phase:** 15.1
- **Target Subsystem:** `macro-engine` & `ooxml-opc`
- **Primary Agent Skill:** `@security-engineer`
- **Deliverable:** Extract and decompress compressed VBA streams from word/vbaProject.bin and xl/vbaProject.bin, tokenizing VBA source lines.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 15.1.1
Invoke @security-engineer under GEC-v2.0:
1. Create new crate `crates/macro-engine` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Implement `OleCompoundReader`:
   - Parse OLE Compound Document (CFBF / MS-CFB) sector allocation tables (FAT/MiniFAT) and directory entries.
   - Locate and decompress MS-OVBA streams (`CompressedSourceCode` algorithm).
   - Extract raw module streams (`ThisWorkbook`, `Sheet1`, user code modules, userforms).
3. Connect with `ooxml-opc` to inspect `vbaProject.bin` parts without corrupting lossless roundtrip saves.
4. Add unit tests with real OLE stream fixtures verifying decompressed source text parity.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
