---
description: Automated workflow for Calc (Excel) Standalone Engine
---
# Dedicated Workflow: Calc (Excel) Standalone Engine
Execute the following subphases sequentially under GEC-v1.0:

- **Subphase 2.1.1**: SpreadsheetML Streaming Parser & Interned SharedStrings (.agents/stages/stage-2/subphase-2.1.1.md)
- **Subphase 2.1.2**: Compressed Sparse Page Grid (CSPG) Memory Model (1M Rows) (.agents/stages/stage-2/subphase-2.1.2.md)
- **Subphase 2.2.1**: Pratt Formula Lexer, AST Compiler & Spill Range Resolvers (.agents/stages/stage-2/subphase-2.2.1.md)
- **Subphase 2.2.2**: Excel Calc Chain DAG, Circular Ref Trapper & IEEE-754 Parity (.agents/stages/stage-2/subphase-2.2.2.md)
- **Subphase 2.2.3**: Parallel SIMD Evaluation of 450+ Built-in Excel Functions (.agents/stages/stage-2/subphase-2.2.3.md)
- **Subphase 2.3.1**: Virtualized 120fps Grid Viewport (Rows 1..1048576, Cols A..XFD) (.agents/stages/stage-2/subphase-2.3.1.md)
- **Subphase 2.3.2**: Interactive Formula Bar, Multi-Range Cell Selection & Auto-Fill (.agents/stages/stage-2/subphase-2.3.2.md)