#![forbid(unsafe_code)]

pub mod ast;
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod ole_cfb;
pub mod ovba_decompress;
pub mod parser;
pub mod sandbox;
pub mod security;
pub mod udf_bridge;
pub mod vba_project;
pub mod virtual_dom;

pub use ast::{
    BinaryOp, ExitKind, RoutineDef, RoutineKind, UnaryOp, VbaExpr, VbaLiteral, VbaParam,
    VbaProgram, VbaStatement,
};
pub use error::MacroError;
pub use evaluator::VbaEvaluator;
pub use lexer::{Token, TokenKind, VbaLexer};
pub use ole_cfb::{DirectoryEntry, EntryType, OleCompoundReader, OLE_MAGIC};
pub use ovba_decompress::{
    compress_vba_stream, copy_token_offset_bit_count, decompress_vba_stream,
};
pub use parser::VbaParser;
pub use sandbox::MacroSandbox;
pub use security::{
    AuditLogEntry, Capability, PermissionStatus, RiskLevel, SecurityAuditLog, SecurityAuditReport,
    SecurityFinding, SecurityScanner, Severity,
};
pub use udf_bridge::{cell_value_to_vba_value, create_vba_udf_handler, vba_value_to_cell_value};
pub use vba_project::{VbaModule, VbaModuleType, VbaProject};
pub use virtual_dom::{
    RangeAddress, VbaValue, VirtualApplication, VirtualCell, VirtualOfficeDom, VirtualWorkbook,
    VirtualWorksheet,
};
