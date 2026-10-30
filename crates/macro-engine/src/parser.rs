use crate::ast::*;
use crate::error::MacroError;
use crate::lexer::{Token, TokenKind, VbaLexer};

pub struct VbaParser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 0,
    Or = 1,
    And = 2,
    Comp = 3,
    Concat = 4,
    AddSub = 5,
    Mod = 6,
    MulDiv = 7,
    Unary = 8,
    Power = 9,
    Call = 10,
    Member = 11,
}

impl VbaParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn from_source(source: &str) -> Result<Self, MacroError> {
        let mut lexer = VbaLexer::new(source);
        let tokens = lexer.tokenize()?;
        Ok(Self::new(tokens))
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            Token {
                kind: TokenKind::Eof,
                line: 0,
                col: 0,
            }
        }
    }

    fn match_token(&mut self, expected: &TokenKind) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_token(&mut self, expected: &TokenKind) -> Result<Token, MacroError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            let cur = self.peek_token();
            let line = cur.map(|t| t.line).unwrap_or(0);
            let col = cur.map(|t| t.col).unwrap_or(0);
            Err(MacroError::ParseError(format!(
                "Expected token {expected:?}, found {:?} at line {line}, col {col}",
                self.peek()
            )))
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &TokenKind::Newline || self.peek() == &TokenKind::Colon {
            self.advance();
        }
    }

    pub fn parse_program(&mut self) -> Result<VbaProgram, MacroError> {
        let mut routines = Vec::new();
        let mut top_level_statements = Vec::new();

        self.skip_newlines();

        while self.peek() != &TokenKind::Eof {
            // Check for Sub or Function
            let is_sub = self.peek() == &TokenKind::Sub
                || (self.peek() == &TokenKind::Public && self.peek_ahead(1) == &TokenKind::Sub)
                || (self.peek() == &TokenKind::Private && self.peek_ahead(1) == &TokenKind::Sub);

            let is_func = self.peek() == &TokenKind::Function
                || (self.peek() == &TokenKind::Public
                    && self.peek_ahead(1) == &TokenKind::Function)
                || (self.peek() == &TokenKind::Private
                    && self.peek_ahead(1) == &TokenKind::Function);

            if is_sub || is_func {
                let routine = self.parse_routine()?;
                routines.push(routine);
            } else {
                let stmt = self.parse_statement()?;
                top_level_statements.push(stmt);
            }

            self.skip_newlines();
        }

        Ok(VbaProgram {
            routines,
            top_level_statements,
        })
    }

    fn peek_ahead(&self, n: usize) -> &TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn parse_routine(&mut self) -> Result<RoutineDef, MacroError> {
        // Optional Public / Private
        if self.peek() == &TokenKind::Public || self.peek() == &TokenKind::Private {
            self.advance();
        }

        let kind = if self.match_token(&TokenKind::Sub) {
            RoutineKind::Sub
        } else if self.match_token(&TokenKind::Function) {
            RoutineKind::Function
        } else {
            return Err(MacroError::ParseError(format!(
                "Expected Sub or Function, found {:?}",
                self.peek()
            )));
        };

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => {
                return Err(MacroError::ParseError(format!(
                    "Expected routine name identifier, found {other:?}"
                )))
            }
        };

        // Parameters: ( [param1], [param2] )
        let mut params = Vec::new();
        if self.match_token(&TokenKind::OpenParen) {
            while self.peek() != &TokenKind::CloseParen && self.peek() != &TokenKind::Eof {
                let mut is_optional = false;
                let mut is_by_val = false;

                if self.match_token(&TokenKind::Optional) {
                    is_optional = true;
                }
                if self.match_token(&TokenKind::ByVal) {
                    is_by_val = true;
                } else if self.match_token(&TokenKind::ByRef) {
                    is_by_val = false;
                }

                let param_name = match self.advance().kind {
                    TokenKind::Ident(p) => p,
                    other => {
                        return Err(MacroError::ParseError(format!(
                            "Expected parameter name, found {other:?}"
                        )))
                    }
                };

                let mut param_type = None;
                if self.match_token(&TokenKind::As) {
                    if let TokenKind::Ident(t) = self.advance().kind {
                        param_type = Some(t);
                    }
                }

                params.push(VbaParam {
                    name: param_name,
                    param_type,
                    is_by_val,
                    is_optional,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect_token(&TokenKind::CloseParen)?;
        }

        // Return type for Function
        let mut return_type = None;
        if kind == RoutineKind::Function && self.match_token(&TokenKind::As) {
            if let TokenKind::Ident(t) = self.advance().kind {
                return_type = Some(t);
            }
        }

        self.skip_newlines();

        // Routine body
        let mut body = Vec::new();
        while self.peek() != &TokenKind::Eof {
            // Check for End Sub or End Function
            if self.peek() == &TokenKind::End {
                let next = self.peek_ahead(1);
                if (kind == RoutineKind::Sub && next == &TokenKind::Sub)
                    || (kind == RoutineKind::Function && next == &TokenKind::Function)
                {
                    self.advance(); // End
                    self.advance(); // Sub or Function
                    break;
                }
            }

            let stmt = self.parse_statement()?;
            body.push(stmt);
            self.skip_newlines();
        }

        Ok(RoutineDef {
            kind,
            name,
            params,
            return_type,
            body,
        })
    }

    pub fn parse_statement(&mut self) -> Result<VbaStatement, MacroError> {
        self.skip_newlines();

        match self.peek() {
            TokenKind::Dim => {
                self.advance();
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => {
                        return Err(MacroError::ParseError(format!(
                            "Expected variable name after Dim, found {other:?}"
                        )))
                    }
                };

                let mut var_type = None;
                if self.match_token(&TokenKind::As) {
                    if let TokenKind::Ident(t) = self.advance().kind {
                        var_type = Some(t);
                    }
                }

                Ok(VbaStatement::Dim { name, var_type })
            }

            TokenKind::Set => {
                self.advance();
                let target = self.parse_expr(Precedence::Lowest)?;
                self.expect_token(&TokenKind::Equal)?;
                let value = self.parse_expr(Precedence::Lowest)?;
                Ok(VbaStatement::Set { target, value })
            }

            TokenKind::Let => {
                self.advance();
                let target = self.parse_expr(Precedence::Lowest)?;
                self.expect_token(&TokenKind::Equal)?;
                let value = self.parse_expr(Precedence::Lowest)?;
                Ok(VbaStatement::Assign { target, value })
            }

            TokenKind::Call => {
                self.advance();
                let target = self.parse_expr(Precedence::Lowest)?;
                // Call might be `Call Foo(x, y)` which is parsed into `IndexOrCall`
                if let VbaExpr::IndexOrCall { callee, args } = target {
                    Ok(VbaStatement::Call {
                        target: *callee,
                        args,
                    })
                } else {
                    Ok(VbaStatement::Call {
                        target,
                        args: Vec::new(),
                    })
                }
            }

            TokenKind::If => {
                self.advance();
                let condition = self.parse_expr(Precedence::Lowest)?;
                self.expect_token(&TokenKind::Then)?;

                // Single-line If vs Multi-line block If
                // If followed immediately by statement on same line, it is single line If
                if self.peek() != &TokenKind::Newline {
                    let single_stmt = self.parse_statement()?;
                    let mut else_branch = None;
                    if self.match_token(&TokenKind::Else) {
                        let else_stmt = self.parse_statement()?;
                        else_branch = Some(vec![else_stmt]);
                    }
                    return Ok(VbaStatement::If {
                        condition,
                        then_branch: vec![single_stmt],
                        else_ifs: Vec::new(),
                        else_branch,
                    });
                }

                self.skip_newlines();

                let mut then_branch = Vec::new();
                let mut else_ifs = Vec::new();
                let mut else_branch = None;

                while self.peek() != &TokenKind::Eof {
                    if self.peek() == &TokenKind::End && self.peek_ahead(1) == &TokenKind::If {
                        self.advance(); // End
                        self.advance(); // If
                        break;
                    }

                    if self.peek() == &TokenKind::ElseIf {
                        self.advance();
                        let cond = self.parse_expr(Precedence::Lowest)?;
                        self.expect_token(&TokenKind::Then)?;
                        self.skip_newlines();

                        let mut elseif_stmts = Vec::new();
                        while self.peek() != &TokenKind::Eof
                            && self.peek() != &TokenKind::ElseIf
                            && self.peek() != &TokenKind::Else
                            && !(self.peek() == &TokenKind::End
                                && self.peek_ahead(1) == &TokenKind::If)
                        {
                            elseif_stmts.push(self.parse_statement()?);
                            self.skip_newlines();
                        }
                        else_ifs.push((cond, elseif_stmts));
                        continue;
                    }

                    if self.peek() == &TokenKind::Else {
                        self.advance();
                        self.skip_newlines();

                        let mut else_stmts = Vec::new();
                        while self.peek() != &TokenKind::Eof
                            && !(self.peek() == &TokenKind::End
                                && self.peek_ahead(1) == &TokenKind::If)
                        {
                            else_stmts.push(self.parse_statement()?);
                            self.skip_newlines();
                        }
                        else_branch = Some(else_stmts);
                        self.skip_newlines();
                        if self.peek() == &TokenKind::End && self.peek_ahead(1) == &TokenKind::If {
                            self.advance();
                            self.advance();
                        }
                        break;
                    }

                    then_branch.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                Ok(VbaStatement::If {
                    condition,
                    then_branch,
                    else_ifs,
                    else_branch,
                })
            }

            TokenKind::For => {
                self.advance();
                let counter = match self.advance().kind {
                    TokenKind::Ident(c) => c,
                    other => {
                        return Err(MacroError::ParseError(format!(
                            "Expected counter variable in For loop, found {other:?}"
                        )))
                    }
                };

                self.expect_token(&TokenKind::Equal)?;
                let start = self.parse_expr(Precedence::Lowest)?;
                self.expect_token(&TokenKind::To)?;
                let end = self.parse_expr(Precedence::Lowest)?;

                let mut step = None;
                if self.match_token(&TokenKind::Step) {
                    step = Some(self.parse_expr(Precedence::Lowest)?);
                }

                self.skip_newlines();

                let mut body = Vec::new();
                while self.peek() != &TokenKind::Eof && self.peek() != &TokenKind::Next {
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                self.expect_token(&TokenKind::Next)?;
                // Optional variable name after Next: Next i
                if let TokenKind::Ident(_) = self.peek() {
                    self.advance();
                }

                Ok(VbaStatement::For {
                    counter,
                    start,
                    end,
                    step,
                    body,
                })
            }

            TokenKind::Do => {
                self.advance();
                let mut condition = None;
                let mut is_while = true;
                let mut is_post_test = false;

                if self.match_token(&TokenKind::While) {
                    condition = Some(self.parse_expr(Precedence::Lowest)?);
                    is_while = true;
                } else if self.match_token(&TokenKind::Until) {
                    condition = Some(self.parse_expr(Precedence::Lowest)?);
                    is_while = false;
                }

                self.skip_newlines();

                let mut body = Vec::new();
                while self.peek() != &TokenKind::Eof && self.peek() != &TokenKind::Loop {
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                self.expect_token(&TokenKind::Loop)?;

                // Check post-test condition: Loop While x < 10 or Loop Until x > 10
                if condition.is_none() {
                    if self.match_token(&TokenKind::While) {
                        condition = Some(self.parse_expr(Precedence::Lowest)?);
                        is_while = true;
                        is_post_test = true;
                    } else if self.match_token(&TokenKind::Until) {
                        condition = Some(self.parse_expr(Precedence::Lowest)?);
                        is_while = false;
                        is_post_test = true;
                    }
                }

                Ok(VbaStatement::DoLoop {
                    condition,
                    is_while,
                    is_post_test,
                    body,
                })
            }

            TokenKind::While => {
                self.advance();
                let condition = self.parse_expr(Precedence::Lowest)?;
                self.skip_newlines();

                let mut body = Vec::new();
                while self.peek() != &TokenKind::Eof && self.peek() != &TokenKind::Wend {
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }
                self.expect_token(&TokenKind::Wend)?;

                Ok(VbaStatement::WhileWend { condition, body })
            }

            TokenKind::With => {
                self.advance();
                let object = self.parse_expr(Precedence::Lowest)?;
                self.skip_newlines();

                let mut body = Vec::new();
                while self.peek() != &TokenKind::Eof {
                    if self.peek() == &TokenKind::End && self.peek_ahead(1) == &TokenKind::With {
                        self.advance();
                        self.advance();
                        break;
                    }
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                Ok(VbaStatement::With { object, body })
            }

            TokenKind::Exit => {
                self.advance();
                let exit_kind = match self.advance().kind {
                    TokenKind::Sub => ExitKind::Sub,
                    TokenKind::Function => ExitKind::Function,
                    TokenKind::For => ExitKind::For,
                    TokenKind::Do => ExitKind::Do,
                    other => {
                        return Err(MacroError::ParseError(format!(
                            "Invalid Exit target: {other:?}"
                        )))
                    }
                };
                Ok(VbaStatement::Exit(exit_kind))
            }

            _ => {
                // Parse expression or assignment
                let expr = self.parse_expr(Precedence::Lowest)?;

                // In VBA, top-level '=' in a statement is always an assignment
                if let VbaExpr::BinaryOp {
                    op: BinaryOp::Eq,
                    left,
                    right,
                } = expr
                {
                    return Ok(VbaStatement::Assign {
                        target: *left,
                        value: *right,
                    });
                }

                // If followed by Equal, it's an assignment: target = value
                if self.match_token(&TokenKind::Equal) {
                    let value = self.parse_expr(Precedence::Lowest)?;
                    Ok(VbaStatement::Assign {
                        target: expr,
                        value,
                    })
                } else if self.is_sub_arg_ahead() {
                    // Subroutine invocation without parens: e.g. `MsgBox "Hello", 64`
                    let mut args = Vec::new();
                    args.push(self.parse_expr(Precedence::Lowest)?);
                    while self.match_token(&TokenKind::Comma) {
                        args.push(self.parse_expr(Precedence::Lowest)?);
                    }
                    Ok(VbaStatement::Call { target: expr, args })
                } else {
                    Ok(VbaStatement::Expr(expr))
                }
            }
        }
    }

    fn is_sub_arg_ahead(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Str(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::Ident(_)
                | TokenKind::True
                | TokenKind::False
        )
    }

    pub fn parse_expression(&mut self) -> Result<VbaExpr, MacroError> {
        self.parse_expr(Precedence::Lowest)
    }

    fn parse_expr(&mut self, prec: Precedence) -> Result<VbaExpr, MacroError> {
        let mut left = self.parse_prefix()?;

        while prec < self.current_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<VbaExpr, MacroError> {
        match self.peek() {
            TokenKind::Int(val) => {
                let v = *val;
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Int(v)))
            }
            TokenKind::Float(val) => {
                let v = *val;
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Float(v)))
            }
            TokenKind::Str(val) => {
                let v = val.clone();
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Str(v)))
            }
            TokenKind::True => {
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Bool(false)))
            }
            TokenKind::Empty => {
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Empty))
            }
            TokenKind::Null => {
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Null))
            }
            TokenKind::Nothing => {
                self.advance();
                Ok(VbaExpr::Literal(VbaLiteral::Nothing))
            }

            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();
                Ok(VbaExpr::Variable(id))
            }

            TokenKind::Dot => {
                // Member without leading object -> inside With block: .Value, .Interior.Color
                self.advance();
                match self.advance().kind {
                    TokenKind::Ident(m) => Ok(VbaExpr::ImplicitMember(m)),
                    other => Err(MacroError::ParseError(format!(
                        "Expected member name after dot, found {other:?}"
                    ))),
                }
            }

            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr(Precedence::Unary)?;
                Ok(VbaExpr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }

            TokenKind::Not => {
                self.advance();
                let expr = self.parse_expr(Precedence::Unary)?;
                Ok(VbaExpr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }

            TokenKind::OpenParen => {
                self.advance();
                let expr = self.parse_expr(Precedence::Lowest)?;
                self.expect_token(&TokenKind::CloseParen)?;
                Ok(expr)
            }

            other => Err(MacroError::ParseError(format!(
                "Unexpected prefix token {other:?} at line {}, col {}",
                self.peek_token().map(|t| t.line).unwrap_or(0),
                self.peek_token().map(|t| t.col).unwrap_or(0),
            ))),
        }
    }

    fn parse_infix(&mut self, left: VbaExpr) -> Result<VbaExpr, MacroError> {
        match self.peek() {
            TokenKind::Plus => {
                self.advance();
                let right = self.parse_expr(Precedence::AddSub)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Minus => {
                self.advance();
                let right = self.parse_expr(Precedence::AddSub)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Sub,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Star => {
                self.advance();
                let right = self.parse_expr(Precedence::MulDiv)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Mul,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Slash => {
                self.advance();
                let right = self.parse_expr(Precedence::MulDiv)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Div,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Backslash => {
                self.advance();
                let right = self.parse_expr(Precedence::MulDiv)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::IntDiv,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Caret => {
                self.advance();
                // Exponent is right-associative: pass Precedence::Power - 1
                let right = self.parse_expr(Precedence::Unary)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Pow,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Ampersand => {
                self.advance();
                let right = self.parse_expr(Precedence::Concat)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Concat,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Mod => {
                self.advance();
                let right = self.parse_expr(Precedence::Mod)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Mod,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Equal => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::NotEqual => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::NotEq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Less => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Lt,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::LessEqual => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::LtEq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Greater => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Gt,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::GreaterEqual => {
                self.advance();
                let right = self.parse_expr(Precedence::Comp)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::GtEq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::And => {
                self.advance();
                let right = self.parse_expr(Precedence::And)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Or => {
                self.advance();
                let right = self.parse_expr(Precedence::Or)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::Xor => {
                self.advance();
                let right = self.parse_expr(Precedence::Or)?;
                Ok(VbaExpr::BinaryOp {
                    op: BinaryOp::Xor,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }

            TokenKind::Dot => {
                self.advance();
                let member = match self.advance().kind {
                    TokenKind::Ident(m) => m,
                    other => {
                        return Err(MacroError::ParseError(format!(
                            "Expected identifier after '.', found {other:?}"
                        )))
                    }
                };
                Ok(VbaExpr::MemberAccess {
                    object: Box::new(left),
                    member,
                })
            }

            TokenKind::OpenParen => {
                self.advance();
                let mut args = Vec::new();
                if self.peek() != &TokenKind::CloseParen {
                    args.push(self.parse_expr(Precedence::Lowest)?);
                    while self.match_token(&TokenKind::Comma) {
                        args.push(self.parse_expr(Precedence::Lowest)?);
                    }
                }
                self.expect_token(&TokenKind::CloseParen)?;
                Ok(VbaExpr::IndexOrCall {
                    callee: Box::new(left),
                    args,
                })
            }

            other => Err(MacroError::ParseError(format!(
                "Unexpected infix token {other:?}"
            ))),
        }
    }

    fn current_precedence(&self) -> Precedence {
        match self.peek() {
            TokenKind::Or | TokenKind::Xor => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comp,
            TokenKind::Ampersand => Precedence::Concat,
            TokenKind::Plus | TokenKind::Minus => Precedence::AddSub,
            TokenKind::Mod => Precedence::Mod,
            TokenKind::Star | TokenKind::Slash | TokenKind::Backslash => Precedence::MulDiv,
            TokenKind::Caret => Precedence::Power,
            TokenKind::OpenParen => Precedence::Call,
            TokenKind::Dot => Precedence::Member,
            _ => Precedence::Lowest,
        }
    }
}
