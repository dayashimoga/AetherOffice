//! ECMA-376 Formula Subsystem.
//! Includes Pratt parser, AST, bytecode compiler, RPN VM definitions,
//! and lossless canonical decompiler for Excel formulas.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).

pub mod ast;
pub mod bytecode;
pub mod decompiler;
pub mod lexer;
pub mod parser;
pub mod reference;
pub mod token;

pub use ast::{AstNode, BinaryOperator, Literal, UnaryOperator};
pub use bytecode::{BytecodeProgram, OpCode};
pub use decompiler::{decompile_ast, decompile_bytecode};
pub use lexer::Lexer;
pub use parser::Parser;
pub use reference::{
    CellAddress, RangeAddress, Reference, SheetRef, StructuredTableRef, TableSection,
};
pub use token::{Span, Spanned, Token};

use ooxml_opc::error::Result;

/// High-level convenience function to parse a formula string into an AST.
pub fn parse_formula(formula: &str) -> Result<AstNode> {
    let mut parser = Parser::new(formula)?;
    parser.parse()
}

/// High-level convenience function to parse and compile a formula string into bytecode.
pub fn compile_formula(formula: &str) -> Result<BytecodeProgram> {
    let ast = parse_formula(formula)?;
    Ok(BytecodeProgram::compile(&ast))
}

/// High-level convenience function to decompile bytecode back to a canonical formula string.
pub fn decompile_formula(program: &BytecodeProgram) -> Result<String> {
    decompile_bytecode(program)
}
