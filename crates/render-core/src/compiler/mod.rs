//! Unified Display List Compilers for Word, Excel, and PowerPoint

pub mod excel;
pub mod ppt;
pub mod word;

pub use excel::{
    cell_coord_to_indices, col_to_name, indices_to_cell_coord, name_to_col, AutoFillEngine,
    ExcelDisplayCell, ExcelDisplayCompiler, ExcelSelectionRange, FreezePanesConfig,
    GridDimensionMap, GridViewportConfig, HeaderRenderConfig, MultiRangeSelectionModel,
    COL_HEADER_HEIGHT, DEFAULT_COL_WIDTH, DEFAULT_ROW_HEIGHT, MAX_EXCEL_COLS, MAX_EXCEL_ROWS,
    ROW_HEADER_WIDTH,
};
pub use ppt::{PptDisplayCompiler, SlideCanvasConfig};
pub use word::{
    CaretNavDirection, PageLayoutMode, PageMargins, PageSize, ParagraphAlignment,
    SubpixelTextLayout, UnderlineStyle, WordCaretModel, WordCaretPosition, WordDisplayCompiler,
    WordDisplayLine, WordDisplayPage, WordDisplayTableCell, WordHitResult, WordHitTester,
    WordNavigationEngine, WordSelectionRange, WordViewportConfig,
};
