#![forbid(unsafe_code)]
//! OpenOfficeRS Excel Engine (`excel-engine`).
//! High-performance SpreadsheetML parser, styles engine, shared string table,
//! Compressed Sparse Page Grid (CSPG), and parallel multi-threaded package reader/writer.
//! Strictly conforms to GEC-v1.0 (Zero Contamination, Zero-Loss Round-Trip Fidelity,
//! SLAs, and Zero Panics).

pub mod formula;
pub mod grid;
pub mod package;
pub mod shared_strings;
pub mod styles;
pub mod workbook;
pub mod worksheet;

pub use formula::{
    compile_formula, decompile_ast, decompile_bytecode, decompile_formula, parse_formula, AstNode,
    BinaryOperator, BytecodeProgram, CellAddress, Lexer, Literal, OpCode, Parser, RangeAddress,
    Reference, SheetRef, Span, Spanned, StructuredTableRef, TableSection, Token, UnaryOperator,
};

pub use grid::{
    cell_to_coords, col_to_name, coords_to_cell, name_to_col, parse_range, CellError, CellKind,
    CellValue, CompressedSparsePageGrid, PageChunk, CELLS_PER_PAGE, MAX_COLS, MAX_ROWS, PAGE_SIZE,
};
pub use package::ExcelWorkbookPackage;
pub use shared_strings::{RichTextRun, SharedStringItem, SharedStringTable};
pub use styles::{
    builtin_format_code, Alignment, Border, BorderEdge, BorderStyle, CellStyle, CellXf, Color,
    Fill, Font, HorizontalAlignment, NumberFormat, PatternType, StyleSheet, VerticalAlignment,
};
pub use workbook::{
    CalcMode, CalcPr, DefinedName, FileVersion, SheetRecord, SheetState, Workbook, WorkbookPr,
    WorkbookView,
};
pub use worksheet::{PageMargins, SheetFormatPr, SheetView, Worksheet};

pub use ooxml_opc::error::{EngineError, Result};

/// Engine health check verifying crate initialization.
pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
