//! ECMA-376 Formula Bytecode Engine.
//! Compact Reverse Polish Notation (RPN) instruction set compiled from formula AST.
//! Enables high-performance non-recursive evaluation and lossless persistence.
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::ast::{AstNode, BinaryOperator, Literal, UnaryOperator};
use crate::formula::reference::Reference;
use crate::grid::CellError;
use ooxml_opc::error::{EngineError, Result};
use serde::{Deserialize, Serialize};

/// Atomic bytecode instruction for the OpenOfficeRS spreadsheet formula virtual machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    /// Push numeric literal
    PushNumber(f64),
    /// Push string literal
    PushString(String),
    /// Push boolean literal
    PushBool(bool),
    /// Push cell error
    PushError(CellError),
    /// Push empty/omitted argument
    PushEmpty,
    /// Push reference (cell, range, 3D, table, spill, implicit intersection)
    PushReference(Reference),
    /// Perform unary operation on top of stack
    UnaryOp(UnaryOperator),
    /// Perform binary operation on top 2 elements of stack
    BinaryOp(BinaryOperator),
    /// Call named function with specified argument count
    CallFunction { name: String, arg_count: usize },
    /// Construct inline array with specified rows and columns
    MakeArray { rows: usize, cols: usize },
}

/// Linear bytecode program executed via stack machine.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub instructions: Vec<OpCode>,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    /// Compiles a formula AST into a linear Reverse Polish Notation (RPN) bytecode program.
    pub fn compile(ast: &AstNode) -> Self {
        let mut program = Self::new();
        program.emit_ast(ast);
        program
    }

    fn emit_ast(&mut self, node: &AstNode) {
        match node {
            AstNode::Literal(lit) => match lit {
                Literal::Number(n) => self.instructions.push(OpCode::PushNumber(*n)),
                Literal::String(s) => self.instructions.push(OpCode::PushString(s.clone())),
                Literal::Boolean(b) => self.instructions.push(OpCode::PushBool(*b)),
                Literal::Error(e) => self.instructions.push(OpCode::PushError(*e)),
                Literal::Empty => self.instructions.push(OpCode::PushEmpty),
            },
            AstNode::Reference(r) => {
                self.instructions.push(OpCode::PushReference(r.clone()));
            }
            AstNode::UnaryOp { op, expr } => {
                self.emit_ast(expr);
                self.instructions.push(OpCode::UnaryOp(*op));
            }
            AstNode::BinaryOp { op, left, right } => {
                self.emit_ast(left);
                self.emit_ast(right);
                self.instructions.push(OpCode::BinaryOp(*op));
            }
            AstNode::FunctionCall { name, args } => {
                for arg in args {
                    self.emit_ast(arg);
                }
                self.instructions.push(OpCode::CallFunction {
                    name: name.clone(),
                    arg_count: args.len(),
                });
            }
            AstNode::ArrayLiteral { rows } => {
                let row_count = rows.len();
                let col_count = rows.first().map(|r| r.len()).unwrap_or(0);
                for row in rows {
                    for cell in row {
                        self.emit_ast(cell);
                    }
                }
                self.instructions.push(OpCode::MakeArray {
                    rows: row_count,
                    cols: col_count,
                });
            }
            AstNode::Parenthesized(inner) => {
                self.emit_ast(inner);
            }
        }
    }

    /// Serializes bytecode to a compact JSON string or binary byte representation.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(EngineError::Json)
    }

    /// Deserializes bytecode from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(EngineError::Json)
    }
}
