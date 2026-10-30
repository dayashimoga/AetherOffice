use crate::error::MacroError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Keywords
    Sub,
    Function,
    End,
    Dim,
    ReDim,
    As,
    If,
    Then,
    Else,
    ElseIf,
    For,
    To,
    Step,
    Next,
    Each,
    In,
    Do,
    While,
    Until,
    Loop,
    Wend,
    With,
    Exit,
    Select,
    Case,
    Call,
    Set,
    Let,
    New,
    Const,
    True,
    False,
    Nothing,
    Empty,
    Null,
    On,
    Error,
    GoTo,
    Resume,
    Public,
    Private,
    ByVal,
    ByRef,
    Optional,
    Not,
    And,
    Or,
    Xor,
    Mod,

    // Identifiers & Literals
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),

    // Operators & Symbols
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Backslash,    // \ (integer division)
    Caret,        // ^
    Ampersand,    // & (string concatenation)
    Equal,        // =
    NotEqual,     // <>
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    OpenParen,    // (
    CloseParen,   // )
    Comma,        // ,
    Dot,          // .
    Colon,        // : (statement separator)
    Semicolon,    // ;
    Newline,      // Line break statement separator
    Eof,          // End of file
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct VbaLexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> VbaLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, MacroError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            let start_line = self.line;
            let start_col = self.col;

            match ch {
                // Whitespace
                ' ' | '\t' | '\r' => {
                    self.advance();
                }

                // Line continuations: _ followed by newline
                '_' => {
                    let mut lookahead = self.pos + 1;
                    let mut is_continuation = false;
                    while lookahead < self.chars.len() {
                        let c = self.chars[lookahead];
                        if c == ' ' || c == '\t' || c == '\r' {
                            lookahead += 1;
                        } else if c == '\n' {
                            is_continuation = true;
                            break;
                        } else {
                            break;
                        }
                    }

                    if is_continuation {
                        self.pos = lookahead + 1;
                        self.line += 1;
                        self.col = 1;
                    } else {
                        // Identifier starting with _
                        let ident = self.read_ident();
                        tokens.push(Token {
                            kind: self.resolve_ident(&ident),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }

                // Comments
                '\'' => {
                    // Consume till end of line
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }

                // Newlines
                '\n' => {
                    self.advance();
                    // Avoid multiple consecutive newlines in token stream
                    if tokens.last().map(|t| &t.kind) != Some(&TokenKind::Newline) {
                        tokens.push(Token {
                            kind: TokenKind::Newline,
                            line: start_line,
                            col: start_col,
                        });
                    }
                }

                // Punctuation and single-char operators
                ':' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Colon,
                        line: start_line,
                        col: start_col,
                    });
                }
                ';' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Semicolon,
                        line: start_line,
                        col: start_col,
                    });
                }
                '(' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::OpenParen,
                        line: start_line,
                        col: start_col,
                    });
                }
                ')' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::CloseParen,
                        line: start_line,
                        col: start_col,
                    });
                }
                ',' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Comma,
                        line: start_line,
                        col: start_col,
                    });
                }
                '.' => {
                    self.advance();
                    // Check if it's a decimal number like .5
                    if let Some(next) = self.peek() {
                        if next.is_ascii_digit() {
                            let mut num_str = "0.".to_string();
                            while let Some(d) = self.peek() {
                                if d.is_ascii_digit() {
                                    num_str.push(d);
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            let val: f64 = num_str
                                .parse()
                                .map_err(|e| MacroError::LexError(format!("Invalid float: {e}")))?;
                            tokens.push(Token {
                                kind: TokenKind::Float(val),
                                line: start_line,
                                col: start_col,
                            });
                            continue;
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::Dot,
                        line: start_line,
                        col: start_col,
                    });
                }
                '+' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Plus,
                        line: start_line,
                        col: start_col,
                    });
                }
                '-' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Minus,
                        line: start_line,
                        col: start_col,
                    });
                }
                '*' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Star,
                        line: start_line,
                        col: start_col,
                    });
                }
                '/' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Slash,
                        line: start_line,
                        col: start_col,
                    });
                }
                '\\' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Backslash,
                        line: start_line,
                        col: start_col,
                    });
                }
                '^' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Caret,
                        line: start_line,
                        col: start_col,
                    });
                }
                '&' => {
                    self.advance();
                    // Check for hex or octal: &HFF or &O77
                    if let Some(h) = self.peek() {
                        if h == 'h' || h == 'H' {
                            self.advance();
                            let mut hex_str = String::new();
                            while let Some(c) = self.peek() {
                                if c.is_ascii_hexdigit() {
                                    hex_str.push(c);
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            let val = i64::from_str_radix(&hex_str, 16).map_err(|e| {
                                MacroError::LexError(format!("Invalid hex literal: {e}"))
                            })?;
                            tokens.push(Token {
                                kind: TokenKind::Int(val),
                                line: start_line,
                                col: start_col,
                            });
                            continue;
                        } else if h == 'o' || h == 'O' {
                            self.advance();
                            let mut oct_str = String::new();
                            while let Some(c) = self.peek() {
                                if ('0'..='7').contains(&c) {
                                    oct_str.push(c);
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            let val = i64::from_str_radix(&oct_str, 8).map_err(|e| {
                                MacroError::LexError(format!("Invalid octal literal: {e}"))
                            })?;
                            tokens.push(Token {
                                kind: TokenKind::Int(val),
                                line: start_line,
                                col: start_col,
                            });
                            continue;
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::Ampersand,
                        line: start_line,
                        col: start_col,
                    });
                }

                // Comparison operators
                '=' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Equal,
                        line: start_line,
                        col: start_col,
                    });
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::NotEqual,
                            line: start_line,
                            col: start_col,
                        });
                    } else if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::LessEqual,
                            line: start_line,
                            col: start_col,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Less,
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::GreaterEqual,
                            line: start_line,
                            col: start_col,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Greater,
                            line: start_line,
                            col: start_col,
                        });
                    }
                }

                // String Literals
                '"' => {
                    self.advance();
                    let mut s = String::new();
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            self.advance();
                            if self.peek() == Some('"') {
                                // Escaped quote ""
                                s.push('"');
                                self.advance();
                            } else {
                                break;
                            }
                        } else if c == '\n' {
                            return Err(MacroError::LexError(
                                "Unterminated string literal at newline".to_string(),
                            ));
                        } else {
                            s.push(c);
                            self.advance();
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::Str(s),
                        line: start_line,
                        col: start_col,
                    });
                }

                // Numbers
                '0'..='9' => {
                    let mut num_str = String::new();
                    let mut is_float = false;

                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            num_str.push(c);
                            self.advance();
                        } else if c == '.'
                            && !is_float
                            && self.peek_next().is_some_and(|n| n.is_ascii_digit())
                        {
                            is_float = true;
                            num_str.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    if is_float {
                        let val: f64 = num_str.parse().map_err(|e| {
                            MacroError::LexError(format!("Invalid float literal: {e}"))
                        })?;
                        tokens.push(Token {
                            kind: TokenKind::Float(val),
                            line: start_line,
                            col: start_col,
                        });
                    } else {
                        let val: i64 = num_str.parse().map_err(|e| {
                            MacroError::LexError(format!("Invalid integer literal: {e}"))
                        })?;
                        tokens.push(Token {
                            kind: TokenKind::Int(val),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }

                // Identifiers or Keywords
                'a'..='z' | 'A'..='Z' => {
                    let ident = self.read_ident();
                    // Check for REM comment keyword
                    if ident.eq_ignore_ascii_case("rem") {
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }

                    tokens.push(Token {
                        kind: self.resolve_ident(&ident),
                        line: start_line,
                        col: start_col,
                    });
                }

                _ => {
                    return Err(MacroError::LexError(format!(
                        "Unexpected character '{ch}' at line {start_line}, col {start_col}"
                    )));
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
        });

        Ok(tokens)
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn resolve_ident(&self, ident: &str) -> TokenKind {
        let lower = ident.to_ascii_lowercase();
        match lower.as_str() {
            "sub" => TokenKind::Sub,
            "function" => TokenKind::Function,
            "end" => TokenKind::End,
            "dim" => TokenKind::Dim,
            "redim" => TokenKind::ReDim,
            "as" => TokenKind::As,
            "if" => TokenKind::If,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::ElseIf,
            "for" => TokenKind::For,
            "to" => TokenKind::To,
            "step" => TokenKind::Step,
            "next" => TokenKind::Next,
            "each" => TokenKind::Each,
            "in" => TokenKind::In,
            "do" => TokenKind::Do,
            "while" => TokenKind::While,
            "until" => TokenKind::Until,
            "loop" => TokenKind::Loop,
            "wend" => TokenKind::Wend,
            "with" => TokenKind::With,
            "exit" => TokenKind::Exit,
            "select" => TokenKind::Select,
            "case" => TokenKind::Case,
            "call" => TokenKind::Call,
            "set" => TokenKind::Set,
            "let" => TokenKind::Let,
            "new" => TokenKind::New,
            "const" => TokenKind::Const,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nothing" => TokenKind::Nothing,
            "empty" => TokenKind::Empty,
            "null" => TokenKind::Null,
            "on" => TokenKind::On,
            "error" => TokenKind::Error,
            "goto" => TokenKind::GoTo,
            "resume" => TokenKind::Resume,
            "public" => TokenKind::Public,
            "private" => TokenKind::Private,
            "byval" => TokenKind::ByVal,
            "byref" => TokenKind::ByRef,
            "optional" => TokenKind::Optional,
            "not" => TokenKind::Not,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "xor" => TokenKind::Xor,
            "mod" => TokenKind::Mod,
            _ => TokenKind::Ident(ident.to_string()),
        }
    }
}
