//! ECMA-376 Formula Abstract Syntax Tree (AST).
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::reference::Reference;
use crate::grid::CellError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Primitive formula literal value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Error(CellError),
    /// Represents an empty/omitted function argument, e.g. `IF(A1, 1, )`.
    Empty,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{}\"", s.replace('"', "\"\"")),
            Self::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Self::Error(e) => write!(f, "{e}"),
            Self::Empty => Ok(()),
        }
    }
}

/// Unary operators in Excel formulas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Unary positive `+expr`
    Plus,
    /// Unary negation `-expr`
    Neg,
    /// Postfix percent `expr%` (divides by 100)
    Percent,
    /// Dynamic array spill operator `expr#`
    Spill,
    /// Implicit intersection operator `@expr`
    ImplicitIntersection,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Neg => write!(f, "-"),
            Self::Percent => write!(f, "%"),
            Self::Spill => write!(f, "#"),
            Self::ImplicitIntersection => write!(f, "@"),
        }
    }
}

/// Binary operators in ECMA-376 formulas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    // String concatenation
    Concat,
    // Comparisons
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    // Reference operators
    Range,
    Union,
    Intersection,
}

impl BinaryOperator {
    /// Returns symbol for display and decompilation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Pow => "^",
            Self::Concat => "&",
            Self::Equal => "=",
            Self::NotEqual => "<>",
            Self::LessThan => "<",
            Self::LessEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterEqual => ">=",
            Self::Range => ":",
            Self::Union => ",",
            Self::Intersection => " ",
        }
    }

    /// Binding power for precedence comparison.
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Equal
            | Self::NotEqual
            | Self::LessThan
            | Self::LessEqual
            | Self::GreaterThan
            | Self::GreaterEqual => 10,
            Self::Concat => 20,
            Self::Add | Self::Sub => 30,
            Self::Mul | Self::Div => 40,
            Self::Pow => 50,
            Self::Union => 80,
            Self::Intersection => 90,
            Self::Range => 100,
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Abstract Syntax Tree node for ECMA-376 formulas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    /// Literal constant (number, string, bool, error, empty)
    Literal(Literal),
    /// Cell, range, 3D, table, or spill reference
    Reference(Reference),
    /// Unary operation (+, -, %, #, @)
    UnaryOp {
        op: UnaryOperator,
        expr: Box<AstNode>,
    },
    /// Binary operation (+, -, *, /, ^, &, =, <>, <, <=, >, >=, :, ,, ' ')
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    /// Function invocation (e.g. `SUM(A1, B1)`)
    FunctionCall { name: String, args: Vec<AstNode> },
    /// In-line matrix/array literal (e.g. `{1, 2; 3, 4}`)
    ArrayLiteral { rows: Vec<Vec<AstNode>> },
    /// Explicit parenthesized expression `(expr)`
    Parenthesized(Box<AstNode>),
}

impl AstNode {
    /// Formats this AST node as a canonical formula string without leading `=`.
    pub fn to_formula_string(&self) -> String {
        match self {
            Self::Literal(lit) => lit.to_string(),
            Self::Reference(r) => r.to_formula_string(),
            Self::UnaryOp { op, expr } => match op {
                UnaryOperator::Percent | UnaryOperator::Spill => {
                    format!("{}{}", expr.to_formula_string(), op)
                }
                UnaryOperator::Plus | UnaryOperator::Neg | UnaryOperator::ImplicitIntersection => {
                    format!("{}{}", op, expr.to_formula_string())
                }
            },
            Self::BinaryOp { op, left, right } => {
                let left_str = format_operand_with_precedence(left, *op, true);
                let right_str = format_operand_with_precedence(right, *op, false);
                match op {
                    BinaryOperator::Intersection => format!("{left_str} {right_str}"),
                    BinaryOperator::Range | BinaryOperator::Union => {
                        format!("{}{}{}", left_str, op.as_str(), right_str)
                    }
                    _ => format!("{} {} {}", left_str, op.as_str(), right_str),
                }
            }
            Self::FunctionCall { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_formula_string()).collect();
                format!("{}({})", name, args_str.join(","))
            }
            Self::ArrayLiteral { rows } => {
                let row_strs: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        let col_strs: Vec<String> =
                            row.iter().map(|c| c.to_formula_string()).collect();
                        col_strs.join(",")
                    })
                    .collect();
                format!("{{{}}}", row_strs.join(";"))
            }
            Self::Parenthesized(inner) => {
                format!("({})", inner.to_formula_string())
            }
        }
    }
}

fn format_operand_with_precedence(
    operand: &AstNode,
    parent_op: BinaryOperator,
    is_left: bool,
) -> String {
    let s = operand.to_formula_string();
    if let AstNode::BinaryOp { op: child_op, .. } = operand {
        let parent_prec = parent_op.precedence();
        let child_prec = child_op.precedence();
        if child_prec < parent_prec {
            return format!("({s})");
        }
        if child_prec == parent_prec && !is_left {
            // For left-associative operators like -, /, ^, parentheses on the right are required
            // e.g. a - (b - c), a / (b / c), a ^ (b ^ c)
            match parent_op {
                BinaryOperator::Sub | BinaryOperator::Div | BinaryOperator::Pow => {
                    return format!("({s})");
                }
                _ => {}
            }
        }
    }
    s
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_formula_string())
    }
}
