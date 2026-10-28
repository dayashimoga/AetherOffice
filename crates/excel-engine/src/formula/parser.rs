//! ECMA-376 Zero-Panic Pratt Formula Parser.
//! Implements Top-Down Operator Precedence (Pratt Parsing) with exact Excel precedence
//! and associativity rules for all operators, 3D ranges, structured tables, and spill refs.
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::ast::{AstNode, BinaryOperator, Literal, UnaryOperator};
use crate::formula::lexer::Lexer;
use crate::formula::reference::Reference;
use crate::formula::token::{Spanned, Token};
use ooxml_opc::error::{EngineError, Result};

const PREFIX_UNARY_BP: u8 = 60;
const POSTFIX_UNARY_BP: u8 = 70;

/// Pratt Parser for ECMA-376 spreadsheet formulas.
pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    cursor: usize,
    allow_comma_union: bool,
}

impl Parser {
    /// Creates a new parser from a formula string.
    pub fn new(formula: &str) -> Result<Self> {
        let mut lexer = Lexer::new(formula);
        let tokens = lexer.tokenize()?;
        Ok(Self {
            tokens,
            cursor: 0,
            allow_comma_union: false,
        })
    }

    /// Parses the formula into a complete AST.
    pub fn parse(&mut self) -> Result<AstNode> {
        if self.peek_token() == &Token::Eof {
            return Ok(AstNode::Literal(Literal::Empty));
        }

        let expr = self.parse_expr(0)?;
        if self.peek_token() != &Token::Eof {
            return Err(EngineError::FormulaError(format!(
                "Unexpected trailing token '{}' at formula position {}",
                self.peek_token(),
                self.current_span_start()
            )));
        }

        Ok(expr)
    }

    fn peek_token(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor].item
        } else {
            &Token::Eof
        }
    }

    fn current_span_start(&self) -> usize {
        if self.cursor < self.tokens.len() {
            self.tokens[self.cursor].span.start
        } else {
            0
        }
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].item.clone();
            self.cursor += 1;
            tok
        } else {
            Token::Eof
        }
    }

    /// Core Pratt parsing routine.
    pub fn parse_expr(&mut self, min_bp: u8) -> Result<AstNode> {
        let mut left = self.parse_prefix()?;

        loop {
            let next_tok = self.peek_token().clone();

            // 1. Check for postfix unary operators (% or #)
            if let Some(op) = postfix_operator(&next_tok) {
                if POSTFIX_UNARY_BP < min_bp {
                    break;
                }
                self.advance();
                left = AstNode::UnaryOp {
                    op,
                    expr: Box::new(left),
                };
                continue;
            }

            // 2. Check for binary operators
            if let Some((op, lbp, rbp)) = self.binary_operator(&next_tok) {
                if lbp < min_bp {
                    break;
                }
                self.advance();
                let right = self.parse_expr(rbp)?;
                left = AstNode::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }

            break;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<AstNode> {
        let tok = self.advance();
        match tok {
            Token::Number(n) => Ok(AstNode::Literal(Literal::Number(n))),
            Token::String(s) => Ok(AstNode::Literal(Literal::String(s))),
            Token::Boolean(b) => Ok(AstNode::Literal(Literal::Boolean(b))),
            Token::Error(e) => Ok(AstNode::Literal(Literal::Error(e))),
            Token::Reference(r) => Ok(AstNode::Reference(r)),
            Token::Ident(name) => {
                if self.peek_token() == &Token::LParen {
                    self.parse_function_call(name)
                } else if self.peek_token() == &Token::Hash {
                    self.advance();
                    Ok(AstNode::Reference(Reference::Spill(Box::new(
                        Reference::Named(name),
                    ))))
                } else {
                    Ok(AstNode::Reference(Reference::Named(name)))
                }
            }
            Token::Plus => {
                let expr = self.parse_expr(PREFIX_UNARY_BP)?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Plus,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                let expr = self.parse_expr(PREFIX_UNARY_BP)?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::At => {
                let expr = self.parse_expr(PREFIX_UNARY_BP)?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::ImplicitIntersection,
                    expr: Box::new(expr),
                })
            }
            Token::LParen => {
                let prev_allow_comma = self.allow_comma_union;
                self.allow_comma_union = true;
                let expr = self.parse_expr(0)?;
                self.allow_comma_union = prev_allow_comma;
                if self.advance() != Token::RParen {
                    return Err(EngineError::FormulaError(
                        "Expected closing parenthesis ')'".to_string(),
                    ));
                }
                Ok(AstNode::Parenthesized(Box::new(expr)))
            }
            Token::LBrace => self.parse_array_literal(),
            Token::Comma => {
                // Empty argument at start of expression
                Ok(AstNode::Literal(Literal::Empty))
            }
            other => Err(EngineError::FormulaError(format!(
                "Unexpected token in formula prefix: '{other}'"
            ))),
        }
    }

    fn parse_function_call(&mut self, name: String) -> Result<AstNode> {
        self.advance(); // consume '('
        let mut args = Vec::new();

        if self.peek_token() == &Token::RParen {
            self.advance(); // consume ')'
            return Ok(AstNode::FunctionCall { name, args });
        }

        loop {
            if self.peek_token() == &Token::RParen {
                break;
            }

            if self.peek_token() == &Token::Comma {
                // Empty argument e.g. IF(A1, , 2)
                args.push(AstNode::Literal(Literal::Empty));
                self.advance(); // consume ','
                continue;
            }

            let prev_allow_comma = self.allow_comma_union;
            self.allow_comma_union = false; // Comma separates arguments, not union
            let arg = self.parse_expr(0)?;
            self.allow_comma_union = prev_allow_comma;
            args.push(arg);

            if self.peek_token() == &Token::Comma {
                self.advance(); // consume ','
                if self.peek_token() == &Token::RParen {
                    // Trailing comma before ')' e.g. IF(A1, 1, )
                    args.push(AstNode::Literal(Literal::Empty));
                    break;
                }
            } else if self.peek_token() == &Token::RParen {
                break;
            } else {
                return Err(EngineError::FormulaError(format!(
                    "Expected ',' or ')' in function '{name}', found '{}'",
                    self.peek_token()
                )));
            }
        }

        if self.advance() != Token::RParen {
            return Err(EngineError::FormulaError(format!(
                "Unclosed function call '{name}', expected ')'"
            )));
        }

        Ok(AstNode::FunctionCall { name, args })
    }

    fn parse_array_literal(&mut self) -> Result<AstNode> {
        let mut rows: Vec<Vec<AstNode>> = Vec::new();
        let mut current_row: Vec<AstNode> = Vec::new();

        loop {
            if self.peek_token() == &Token::RBrace {
                break;
            }

            let item = self.parse_expr(0)?;
            current_row.push(item);

            if self.peek_token() == &Token::Comma {
                self.advance(); // next column in row
            } else if self.peek_token() == &Token::Semicolon {
                self.advance(); // next row
                rows.push(std::mem::take(&mut current_row));
            } else if self.peek_token() == &Token::RBrace {
                break;
            } else {
                return Err(EngineError::FormulaError(format!(
                    "Expected ',' or ';' in array literal, found '{}'",
                    self.peek_token()
                )));
            }
        }

        if !current_row.is_empty() {
            rows.push(current_row);
        }

        if self.advance() != Token::RBrace {
            return Err(EngineError::FormulaError(
                "Unclosed array literal, expected '}'".to_string(),
            ));
        }

        Ok(AstNode::ArrayLiteral { rows })
    }

    fn binary_operator(&self, tok: &Token) -> Option<(BinaryOperator, u8, u8)> {
        match tok {
            Token::Equal => Some((BinaryOperator::Equal, 10, 11)),
            Token::NotEqual => Some((BinaryOperator::NotEqual, 10, 11)),
            Token::LessThan => Some((BinaryOperator::LessThan, 10, 11)),
            Token::LessEqual => Some((BinaryOperator::LessEqual, 10, 11)),
            Token::GreaterThan => Some((BinaryOperator::GreaterThan, 10, 11)),
            Token::GreaterEqual => Some((BinaryOperator::GreaterEqual, 10, 11)),
            Token::Ampersand => Some((BinaryOperator::Concat, 20, 21)),
            Token::Plus => Some((BinaryOperator::Add, 30, 31)),
            Token::Minus => Some((BinaryOperator::Sub, 30, 31)),
            Token::Multiply => Some((BinaryOperator::Mul, 40, 41)),
            Token::Divide => Some((BinaryOperator::Div, 40, 41)),
            // Left-associative in Excel (2^3^2 = 64)
            Token::Power => Some((BinaryOperator::Pow, 50, 51)),
            Token::Comma if self.allow_comma_union => Some((BinaryOperator::Union, 80, 81)),
            Token::Intersection => Some((BinaryOperator::Intersection, 90, 91)),
            Token::Colon => Some((BinaryOperator::Range, 100, 101)),
            _ => None,
        }
    }
}

fn postfix_operator(tok: &Token) -> Option<UnaryOperator> {
    match tok {
        Token::Percent => Some(UnaryOperator::Percent),
        Token::Hash => Some(UnaryOperator::Spill),
        _ => None,
    }
}
