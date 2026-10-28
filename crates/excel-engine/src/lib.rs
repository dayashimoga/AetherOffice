#![forbid(unsafe_code)]
//! OpenOfficeRS Excel Engine (`excel-engine`).
//! High-performance SpreadsheetML parser, styles engine, shared string table,
//! and parallel multi-threaded package reader/writer.
//! Strictly conforms to GEC-v1.0 (Zero Contamination, Zero-Loss Round-Trip Fidelity,
//! SLAs, and Zero Panics).

pub mod package;
pub mod shared_strings;
pub mod styles;
pub mod workbook;

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
