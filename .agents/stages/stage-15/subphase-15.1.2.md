# Subphase 15.1.2: Clean-Room VBA AST Compiler & Virtual Office Object Model
- **Stage:** 15 | **Phase:** 15.1
- **Target Subsystem:** `macro-engine`
- **Primary Agent Skill:** `@compiler-engineer`
- **Deliverable:** Pratt AST parser for VBA statements, variables, loops, conditionals, subroutines, and the core Office DOM (Application, Workbook, Worksheet, Range, Selection).
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 15.1.2
Invoke @compiler-engineer under GEC-v2.0:
1. In `crates/macro-engine`, implement `VbaAstCompiler`:
   - Lexer for VBA tokens, keywords (`Sub`, `Function`, `Dim`, `If`, `Else`, `For`, `Next`, `Do`, `Loop`, `Set`).
   - Pratt AST parser producing structured expression and statement nodes.
2. Implement `VirtualOfficeModel`:
   - Bridge objects: `Application`, `Workbook`, `Worksheet`, `Range`, `Selection`, `Cells`.
   - Core properties and methods: `.Value`, `.Formula`, `.Interior.Color`, `.Offset`, `.Clear`, `.Copy`, `.Select`.
3. Add tests verifying parsing of classic macro automation scripts and mock evaluation of Range mutations.
4. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

