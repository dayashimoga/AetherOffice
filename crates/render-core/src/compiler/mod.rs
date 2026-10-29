//! Unified Display List Compilers for Word, Excel, and PowerPoint

pub mod excel;
pub mod ppt;
pub mod word;

pub use excel::{ExcelDisplayCell, ExcelDisplayCompiler, ExcelSelectionRange};
pub use ppt::{PptDisplayCompiler, SlideCanvasConfig};
pub use word::{WordDisplayCompiler, WordDisplayLine, WordDisplayPage, WordDisplayTableCell};
