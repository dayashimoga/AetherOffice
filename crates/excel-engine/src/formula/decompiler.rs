//! ECMA-376 Formula Decompiler.
//! Reconstructs human-readable canonical formula strings from AST and compiled bytecode.
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::ast::{AstNode, BinaryOperator, UnaryOperator};
use crate::formula::bytecode::{BytecodeProgram, OpCode};
use ooxml_opc::error::{EngineError, Result};

/// Decompiles an AST node to a canonical formula string.
pub fn decompile_ast(ast: &AstNode) -> String {
    format!("={}", ast.to_formula_string())
}

/// Decompiles a bytecode program back to a canonical formula string using stack simulation.
pub fn decompile_bytecode(program: &BytecodeProgram) -> Result<String> {
    if program.instructions.is_empty() {
        return Ok("=".to_string());
    }

    #[derive(Clone)]
    struct StackItem {
        text: String,
        precedence: u8,
    }

    let mut stack: Vec<StackItem> = Vec::new();

    for op in &program.instructions {
        match op {
            OpCode::PushNumber(n) => {
                stack.push(StackItem {
                    text: n.to_string(),
                    precedence: 255,
                });
            }
            OpCode::PushString(s) => {
                stack.push(StackItem {
                    text: format!("\"{}\"", s.replace('"', "\"\"")),
                    precedence: 255,
                });
            }
            OpCode::PushBool(b) => {
                stack.push(StackItem {
                    text: if *b { "TRUE".into() } else { "FALSE".into() },
                    precedence: 255,
                });
            }
            OpCode::PushError(e) => {
                stack.push(StackItem {
                    text: e.to_string(),
                    precedence: 255,
                });
            }
            OpCode::PushEmpty => {
                stack.push(StackItem {
                    text: String::new(),
                    precedence: 255,
                });
            }
            OpCode::PushReference(r) => {
                stack.push(StackItem {
                    text: r.to_formula_string(),
                    precedence: 255,
                });
            }
            OpCode::UnaryOp(u) => {
                let item = stack.pop().ok_or_else(|| {
                    EngineError::FormulaError("Stack underflow during unary decompilation".into())
                })?;
                let text = match u {
                    UnaryOperator::Percent | UnaryOperator::Spill => {
                        format!("{}{}", item.text, u)
                    }
                    UnaryOperator::Plus
                    | UnaryOperator::Neg
                    | UnaryOperator::ImplicitIntersection => {
                        format!("{}{}", u, item.text)
                    }
                };
                stack.push(StackItem {
                    text,
                    precedence: 60,
                });
            }
            OpCode::BinaryOp(b) => {
                let right = stack.pop().ok_or_else(|| {
                    EngineError::FormulaError(
                        "Stack underflow during binary right decompilation".into(),
                    )
                })?;
                let left = stack.pop().ok_or_else(|| {
                    EngineError::FormulaError(
                        "Stack underflow during binary left decompilation".into(),
                    )
                })?;

                let parent_prec = b.precedence();
                let left_str = if left.precedence < parent_prec {
                    format!("({})", left.text)
                } else {
                    left.text
                };

                let right_str = if right.precedence < parent_prec
                    || (right.precedence == parent_prec
                        && matches!(
                            b,
                            BinaryOperator::Sub | BinaryOperator::Div | BinaryOperator::Pow
                        )) {
                    format!("({})", right.text)
                } else {
                    right.text
                };

                let text = match b {
                    BinaryOperator::Intersection => format!("{left_str} {right_str}"),
                    BinaryOperator::Range | BinaryOperator::Union => {
                        format!("{}{}{}", left_str, b.as_str(), right_str)
                    }
                    _ => format!("{} {} {}", left_str, b.as_str(), right_str),
                };

                stack.push(StackItem {
                    text,
                    precedence: parent_prec,
                });
            }
            OpCode::CallFunction { name, arg_count } => {
                if stack.len() < *arg_count {
                    return Err(EngineError::FormulaError(format!(
                        "Stack underflow for function '{name}', expected {arg_count} args, found {}",
                        stack.len()
                    )));
                }
                let mut args = Vec::with_capacity(*arg_count);
                for _ in 0..*arg_count {
                    args.push(stack.pop().unwrap_or(StackItem {
                        text: String::new(),
                        precedence: 255,
                    }));
                }
                args.reverse();
                let arg_strs: Vec<String> = args.into_iter().map(|a| a.text).collect();
                stack.push(StackItem {
                    text: format!("{}({})", name, arg_strs.join(",")),
                    precedence: 255,
                });
            }
            OpCode::MakeArray { rows, cols } => {
                let total = rows * cols;
                if stack.len() < total {
                    return Err(EngineError::FormulaError(
                        "Stack underflow during array literal decompilation".into(),
                    ));
                }
                let mut items = Vec::with_capacity(total);
                for _ in 0..total {
                    items.push(stack.pop().unwrap_or(StackItem {
                        text: String::new(),
                        precedence: 255,
                    }));
                }
                items.reverse();

                let mut row_strs = Vec::with_capacity(*rows);
                for r in 0..*rows {
                    let mut col_strs = Vec::with_capacity(*cols);
                    for c in 0..*cols {
                        let idx = r * cols + c;
                        col_strs.push(items[idx].text.clone());
                    }
                    row_strs.push(col_strs.join(","));
                }

                stack.push(StackItem {
                    text: format!("{{{}}}", row_strs.join(";")),
                    precedence: 255,
                });
            }
        }
    }

    if stack.len() != 1 {
        return Err(EngineError::FormulaError(format!(
            "Decompilation ended with {} items on stack instead of 1",
            stack.len()
        )));
    }

    Ok(format!("={}", stack[0].text))
}
