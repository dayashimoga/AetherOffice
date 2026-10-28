//! ECMA-376 Formula Token Definitions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::reference::Reference;
use crate::grid::CellError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Source code span (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Token with source position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(item: T, span: Span) -> Self {
        Self { item, span }
    }
}

/// Tokens produced by the formula lexical analyzer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Error(CellError),

    // References
    Reference(Reference),

    // Identifiers & Function Names (e.g. SUM, AVERAGE, XLOOKUP, _xlfn.LET)
    Ident(String),

    // Arithmetic Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Percent,

    // Concatenation Operator
    Ampersand,

    // Comparison Operators
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // Reference & Set Operators
    Colon,
    Comma,
    Intersection, // Space between reference operands

    // Dynamic Array Operators
    Hash, // Postfix spill operator (#)
    At,   // Prefix implicit intersection operator (@)

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Semicolon,

    // End of formula stream
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{}\"", s.replace('"', "\"\"")),
            Self::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Self::Error(e) => write!(f, "{e}"),
            Self::Reference(r) => write!(f, "{r}"),
            Self::Ident(id) => write!(f, "{id}"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Power => write!(f, "^"),
            Self::Percent => write!(f, "%"),
            Self::Ampersand => write!(f, "&"),
            Self::Equal => write!(f, "="),
            Self::NotEqual => write!(f, "<>"),
            Self::LessThan => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Intersection => write!(f, " "),
            Self::Hash => write!(f, "#"),
            Self::At => write!(f, "@"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::Semicolon => write!(f, ";"),
            Self::Eof => write!(f, "<EOF>"),
        }
    }
}
