//! ECMA-376 Formula Lexical Analyzer.
//! Tokenizes formulas with full support for A1, R1C1, 3D sheet ranges,
//! structured table references, dynamic array `#` and `@`, and intersection spaces.
//! Strictly conforms to GEC-v1.0 (Zero Panics, #[forbid(unsafe_code)]).

use crate::formula::reference::{
    CellAddress, RangeAddress, Reference, SheetRef, StructuredTableRef, TableSection,
};
use crate::formula::token::{Span, Spanned, Token};
use crate::grid::CellError;
use ooxml_opc::error::{EngineError, Result};

/// Lexer state for formula tokenization.
pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let trimmed = if let Some(stripped) = input.strip_prefix('=') {
            stripped
        } else {
            input
        };
        let chars: Vec<(usize, char)> = trimmed.char_indices().collect();
        Self {
            input: trimmed,
            chars,
            pos: 0,
        }
    }

    /// Tokenizes the entire formula string into a list of spanned tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Spanned<Token>>> {
        let mut tokens: Vec<Spanned<Token>> = Vec::new();

        while self.pos < self.chars.len() {
            let had_whitespace = self.skip_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }

            // Check for intersection operator:
            // If there was whitespace AND previous token could end an operand AND next token could start an operand
            if had_whitespace && !tokens.is_empty() {
                let prev_can_end =
                    can_end_operand(tokens.last().map(|t| &t.item).unwrap_or(&Token::Eof));
                if prev_can_end && self.can_start_operand() {
                    let start_idx = tokens.last().map(|t| t.span.end).unwrap_or(0);
                    let end_idx = self.current_byte_pos();
                    tokens.push(Spanned::new(
                        Token::Intersection,
                        Span::new(start_idx, end_idx),
                    ));
                }
            }

            let start_byte = self.current_byte_pos();
            let token = self.next_token()?;
            let end_byte = self.current_byte_pos();
            tokens.push(Spanned::new(token, Span::new(start_byte, end_byte)));
        }

        let eof_pos = self.current_byte_pos();
        tokens.push(Spanned::new(Token::Eof, Span::new(eof_pos, eof_pos)));
        Ok(tokens)
    }

    fn current_byte_pos(&self) -> usize {
        if self.pos < self.chars.len() {
            self.chars[self.pos].0
        } else {
            self.input.len()
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos].1)
        } else {
            None
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let idx = self.pos + offset;
        if idx < self.chars.len() {
            Some(self.chars[idx].1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos].1;
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut had = false;
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                had = true;
                self.advance();
            } else {
                break;
            }
        }
        had
    }

    fn can_start_operand(&self) -> bool {
        if let Some(c) = self.peek_char() {
            c.is_ascii_alphanumeric()
                || c == '$'
                || c == '"'
                || c == '('
                || c == '{'
                || c == '['
                || c == '@'
                || c == '\''
                || c == '.'
        } else {
            false
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        let c = self.peek_char().ok_or_else(|| {
            EngineError::FormulaError("Unexpected end of formula input".to_string())
        })?;

        // 1. Quoted string: "Hello ""World"""
        if c == '"' {
            return self.lex_string();
        }

        // 2. Quoted sheet reference: 'Sheet 1'! or 'Jan:Dec'!
        if c == '\'' {
            return self.lex_quoted_sheet_or_reference();
        }

        // 3. Error literal: #NULL!, #DIV/0!, #REF!, etc.
        if c == '#' && self.is_error_start() {
            return self.lex_error();
        }

        // 4. Structured Table Reference start: [@Amount] or [[#Headers],[Total]]
        if c == '[' {
            return self.lex_structured_table_ref(None);
        }

        // 5. Dynamic array prefix: @ (implicit intersection)
        if c == '@' {
            self.advance();
            return Ok(Token::At);
        }

        // 6. Dynamic array postfix: # (spill)
        if c == '#' {
            self.advance();
            return Ok(Token::Hash);
        }

        // 7. Numbers or leading decimal point: 123, 3.14, .5
        if c.is_ascii_digit()
            || (c == '.'
                && self
                    .peek_ahead(1)
                    .map(|d| d.is_ascii_digit())
                    .unwrap_or(false))
        {
            // Check if this might be a whole row range like "1:10"
            if let Some(range) = self.try_lex_whole_row_range()? {
                return Ok(Token::Reference(Reference::Range { sheet: None, range }));
            }
            return self.lex_number();
        }

        // 8. Delimiters & Single-char operators
        match c {
            '(' => {
                self.advance();
                return Ok(Token::LParen);
            }
            ')' => {
                self.advance();
                return Ok(Token::RParen);
            }
            '{' => {
                self.advance();
                return Ok(Token::LBrace);
            }
            '}' => {
                self.advance();
                return Ok(Token::RBrace);
            }
            ';' => {
                self.advance();
                return Ok(Token::Semicolon);
            }
            ',' => {
                self.advance();
                return Ok(Token::Comma);
            }
            ':' => {
                self.advance();
                return Ok(Token::Colon);
            }
            '+' => {
                self.advance();
                return Ok(Token::Plus);
            }
            '-' => {
                self.advance();
                return Ok(Token::Minus);
            }
            '*' => {
                self.advance();
                return Ok(Token::Multiply);
            }
            '/' => {
                self.advance();
                return Ok(Token::Divide);
            }
            '^' => {
                self.advance();
                return Ok(Token::Power);
            }
            '%' => {
                self.advance();
                return Ok(Token::Percent);
            }
            '&' => {
                self.advance();
                return Ok(Token::Ampersand);
            }
            '=' => {
                self.advance();
                return Ok(Token::Equal);
            }
            '<' => {
                self.advance();
                if let Some('>') = self.peek_char() {
                    self.advance();
                    return Ok(Token::NotEqual);
                }
                if let Some('=') = self.peek_char() {
                    self.advance();
                    return Ok(Token::LessEqual);
                }
                return Ok(Token::LessThan);
            }
            '>' => {
                self.advance();
                if let Some('=') = self.peek_char() {
                    self.advance();
                    return Ok(Token::GreaterEqual);
                }
                return Ok(Token::GreaterThan);
            }
            _ => {}
        }

        // 9. Try parsing as R1C1 reference
        if let Some(r1c1_tok) = self.try_lex_r1c1()? {
            return Ok(r1c1_tok);
        }

        // 10. Identifier, cell/range reference, or 3D sheet reference
        self.lex_identifier_or_reference()
    }

    fn try_lex_r1c1(&mut self) -> Result<Option<Token>> {
        let saved_pos = self.pos;
        let c = match self.peek_char() {
            Some(ch) if ch == 'R' || ch == 'r' => ch,
            _ => return Ok(None),
        };

        let mut s = String::new();
        s.push(c);
        self.advance();

        // Optional row: [signed_int] or digits
        if self.peek_char() == Some('[') {
            s.push(self.advance().unwrap_or('['));
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek_char() == Some(']') {
                s.push(self.advance().unwrap_or(']'));
            } else {
                self.pos = saved_pos;
                return Ok(None);
            }
        } else {
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Must have 'C' or 'c'
        match self.peek_char() {
            Some(ch) if ch == 'C' || ch == 'c' => {
                s.push(ch);
                self.advance();
            }
            _ => {
                self.pos = saved_pos;
                return Ok(None);
            }
        }

        // Optional col: [signed_int] or digits
        if self.peek_char() == Some('[') {
            s.push(self.advance().unwrap_or('['));
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek_char() == Some(']') {
                s.push(self.advance().unwrap_or(']'));
            } else {
                self.pos = saved_pos;
                return Ok(None);
            }
        } else {
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check that next char is not alphanumeric or underscore
        if let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.pos = saved_pos;
                return Ok(None);
            }
        }

        if let Ok(cell) = CellAddress::parse_r1c1(&s, None) {
            // Check for range ':'
            if self.peek_char() == Some(':') {
                let range_pos = self.pos;
                self.advance(); // consume ':'
                if let Some(Token::Reference(Reference::Cell { cell: cell2, .. })) =
                    self.try_lex_r1c1()?
                {
                    return Ok(Some(Token::Reference(Reference::Range {
                        sheet: None,
                        range: RangeAddress::new(cell, cell2),
                    })));
                }
                self.pos = range_pos;
            }
            return Ok(Some(Token::Reference(Reference::Cell {
                sheet: None,
                cell,
            })));
        }

        self.pos = saved_pos;
        Ok(None)
    }

    fn lex_string(&mut self) -> Result<Token> {
        self.advance(); // consume opening '"'
        let mut result = String::new();

        while let Some(c) = self.advance() {
            if c == '"' {
                if let Some('"') = self.peek_char() {
                    // Escaped double quote
                    self.advance();
                    result.push('"');
                } else {
                    // End of string
                    return Ok(Token::String(result));
                }
            } else {
                result.push(c);
            }
        }

        Err(EngineError::FormulaError(
            "Unterminated string literal in formula".to_string(),
        ))
    }

    fn is_error_start(&self) -> bool {
        let rest = &self.input[self.current_byte_pos()..];
        for err_str in &[
            "#NULL!",
            "#DIV/0!",
            "#VALUE!",
            "#REF!",
            "#NAME?",
            "#NUM!",
            "#N/A",
            "#GETTING_DATA",
            "#SPILL!",
            "#CALC!",
        ] {
            if rest.starts_with(err_str) {
                return true;
            }
        }
        false
    }

    fn lex_error(&mut self) -> Result<Token> {
        let rest = &self.input[self.current_byte_pos()..];
        for &err_str in &[
            "#GETTING_DATA",
            "#DIV/0!",
            "#VALUE!",
            "#SPILL!",
            "#NAME?",
            "#NULL!",
            "#CALC!",
            "#REF!",
            "#NUM!",
            "#N/A",
        ] {
            if rest.starts_with(err_str) {
                for _ in 0..err_str.chars().count() {
                    self.advance();
                }
                let err = CellError::from_str_opt(err_str).ok_or_else(|| {
                    EngineError::FormulaError(format!("Unrecognized error literal: {err_str}"))
                })?;
                return Ok(Token::Error(err));
            }
        }

        Err(EngineError::FormulaError(format!(
            "Invalid error token near: '{}'",
            &rest[..rest.len().min(10)]
        )))
    }

    fn lex_number(&mut self) -> Result<Token> {
        let start_byte = self.current_byte_pos();
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                self.advance();
            } else if (c == 'e' || c == 'E') && !has_exp {
                has_exp = true;
                self.advance();
                if let Some(next) = self.peek_char() {
                    if next == '+' || next == '-' {
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        let end_byte = self.current_byte_pos();
        let num_str = &self.input[start_byte..end_byte];
        let val: f64 = num_str.parse().map_err(|e| {
            EngineError::FormulaError(format!("Invalid number literal '{num_str}': {e}"))
        })?;

        Ok(Token::Number(val))
    }

    fn try_lex_whole_row_range(&mut self) -> Result<Option<RangeAddress>> {
        let saved_pos = self.pos;
        let mut r1_str = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                r1_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if !r1_str.is_empty() && self.peek_char() == Some(':') {
            self.advance(); // consume ':'
            let mut r2_str = String::new();
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    r2_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if !r2_str.is_empty() {
                let r1: u32 = r1_str.parse().map_err(|e: std::num::ParseIntError| {
                    EngineError::FormulaError(e.to_string())
                })?;
                let r2: u32 = r2_str.parse().map_err(|e: std::num::ParseIntError| {
                    EngineError::FormulaError(e.to_string())
                })?;
                let range = RangeAddress::whole_rows(r1 - 1, r2 - 1, false, false)?;
                return Ok(Some(range));
            }
        }

        self.pos = saved_pos;
        Ok(None)
    }

    fn lex_quoted_sheet_or_reference(&mut self) -> Result<Token> {
        self.advance(); // consume opening '\''
        let mut name = String::new();

        while let Some(c) = self.advance() {
            if c == '\'' {
                if let Some('\'') = self.peek_char() {
                    self.advance();
                    name.push('\'');
                } else {
                    break;
                }
            } else {
                name.push(c);
            }
        }

        if self.peek_char() != Some('!') {
            return Err(EngineError::FormulaError(format!(
                "Expected '!' after quoted sheet name '{name}'"
            )));
        }
        self.advance(); // consume '!'

        // Check for 3D span inside quote, e.g. 'Sheet1:Sheet3'!
        let sheet_ref = if let Some(colon_pos) = name.find(':') {
            let s1 = name[..colon_pos].trim().to_string();
            let s2 = name[colon_pos + 1..].trim().to_string();
            SheetRef::Span(s1, s2)
        } else {
            SheetRef::Single(name)
        };

        // Now parse the target cell or range after '!'
        self.lex_target_reference_after_sheet(Some(sheet_ref))
    }

    fn lex_structured_table_ref(&mut self, table_name: Option<String>) -> Result<Token> {
        self.advance(); // consume '['
        let mut depth = 1;
        let mut content = String::new();

        while let Some(c) = self.advance() {
            if c == '[' {
                depth += 1;
                content.push(c);
            } else if c == ']' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                content.push(c);
            } else {
                content.push(c);
            }
        }

        if depth != 0 {
            return Err(EngineError::FormulaError(format!(
                "Unclosed bracket in structured table reference: '{content}'"
            )));
        }

        // Parse content: e.g. "[#Headers],[Sales]" or "@Sales" or "Sales" or "[Col1]:[Col2]"
        let mut sections = Vec::new();
        let mut column = None;
        let mut end_column = None;
        let mut is_at_syntax = false;

        let trimmed_content = content.trim();
        if let Some(stripped) = trimmed_content.strip_prefix('@') {
            is_at_syntax = true;
            let col_name = stripped.trim().trim_matches('[').trim_matches(']');
            if !col_name.is_empty() {
                column = Some(col_name.to_string());
            }
        } else {
            // Split by comma at top bracket level
            let parts = split_structured_parts(trimmed_content);
            for part in parts {
                let part_trimmed = part.trim();
                if let Some(sec) = TableSection::from_str_opt(part_trimmed) {
                    sections.push(sec);
                } else if part_trimmed.contains("]:[") {
                    let colon_split: Vec<&str> = part_trimmed.split("]:[").collect();
                    if colon_split.len() == 2 {
                        let c1 = colon_split[0].trim_start_matches('[').trim();
                        let c2 = colon_split[1].trim_end_matches(']').trim();
                        column = Some(c1.to_string());
                        end_column = Some(c2.to_string());
                    }
                } else {
                    let clean_col = part_trimmed.trim_matches('[').trim_matches(']').trim();
                    if !clean_col.is_empty() {
                        column = Some(clean_col.to_string());
                    }
                }
            }
        }

        let table_ref = StructuredTableRef {
            table_name,
            sections,
            column,
            end_column,
            is_at_syntax,
        };

        // Check if followed by spill operator '#'
        if self.peek_char() == Some('#') {
            self.advance();
            return Ok(Token::Reference(Reference::Spill(Box::new(
                Reference::Table(table_ref),
            ))));
        }

        Ok(Token::Reference(Reference::Table(table_ref)))
    }

    fn lex_identifier_or_reference(&mut self) -> Result<Token> {
        let start_byte = self.current_byte_pos();
        let mut text = String::new();

        // Include leading '$' if any
        if self.peek_char() == Some('$') {
            text.push(self.advance().unwrap_or('$'));
        }

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if text.is_empty() {
            let unknown = self.advance().unwrap_or('?');
            return Err(EngineError::FormulaError(format!(
                "Unexpected character '{unknown}' in formula at position {start_byte}"
            )));
        }

        // 1. Check if followed by '[' -> Structured Table Reference
        if self.peek_char() == Some('[') {
            return self.lex_structured_table_ref(Some(text));
        }

        // 2. Check if followed by '!' -> Sheet Reference prefix
        if self.peek_char() == Some('!') {
            self.advance(); // consume '!'
            let sheet_ref = if let Some(colon_pos) = text.find(':') {
                let s1 = text[..colon_pos].trim().to_string();
                let s2 = text[colon_pos + 1..].trim().to_string();
                SheetRef::Span(s1, s2)
            } else {
                SheetRef::Single(text)
            };
            return self.lex_target_reference_after_sheet(Some(sheet_ref));
        }

        // 2b. Check for 3D sheet prefix e.g. Sheet1:Sheet3! or Jan:Dec!
        if self.peek_char() == Some(':') {
            let saved_colon_pos = self.pos;
            self.advance(); // consume ':'
            let mut text2 = String::new();
            while let Some(c) = self.peek_char() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                    text2.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if !text2.is_empty() && self.peek_char() == Some('!') {
                self.advance(); // consume '!'
                let sheet_ref = SheetRef::Span(text, text2);
                return self.lex_target_reference_after_sheet(Some(sheet_ref));
            } else {
                self.pos = saved_colon_pos;
            }
        }

        // 3. Check for Boolean literals TRUE / FALSE (when NOT a function call e.g. not followed by '(')
        if self.peek_char() != Some('(') {
            if text.eq_ignore_ascii_case("TRUE") {
                return Ok(Token::Boolean(true));
            }
            if text.eq_ignore_ascii_case("FALSE") {
                return Ok(Token::Boolean(false));
            }
        }

        // 4. Try parsing as R1C1 reference
        if is_r1c1_syntax(&text) {
            if let Ok(cell) = CellAddress::parse_r1c1(&text, None) {
                // Check if followed by colon ':' for R1C1 range
                if self.peek_char() == Some(':') {
                    let saved_pos = self.pos;
                    self.advance(); // consume ':'
                    let mut r2_text = String::new();
                    while let Some(c) = self.peek_char() {
                        if c.is_ascii_alphanumeric() || c == '[' || c == ']' || c == '-' || c == '+'
                        {
                            r2_text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if let Ok(cell2) = CellAddress::parse_r1c1(&r2_text, None) {
                        return Ok(Token::Reference(Reference::Range {
                            sheet: None,
                            range: RangeAddress::new(cell, cell2),
                        }));
                    } else {
                        self.pos = saved_pos;
                    }
                }
                return Ok(Token::Reference(Reference::Cell { sheet: None, cell }));
            }
        }

        // 5. Try parsing as A1 cell reference or whole column range (e.g. A1, $A$1, A:B, $A:$B)
        if let Ok(cell) = CellAddress::parse_a1(&text) {
            // Check if followed by spill operator '#'
            if self.peek_char() == Some('#') {
                self.advance();
                return Ok(Token::Reference(Reference::Spill(Box::new(
                    Reference::Cell { sheet: None, cell },
                ))));
            }
            // Check if followed by colon ':' for range
            if self.peek_char() == Some(':') {
                let saved_pos = self.pos;
                self.advance(); // consume ':'
                                // peek ahead for cell reference
                let mut target_str = String::new();
                if self.peek_char() == Some('$') {
                    target_str.push(self.advance().unwrap_or('$'));
                }
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_alphanumeric() || c == '$' {
                        target_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if let Ok(cell2) = CellAddress::parse_a1(&target_str) {
                    let mut range_ref = Reference::Range {
                        sheet: None,
                        range: RangeAddress::new(cell, cell2),
                    };
                    if self.peek_char() == Some('#') {
                        self.advance();
                        range_ref = Reference::Spill(Box::new(range_ref));
                    }
                    return Ok(Token::Reference(range_ref));
                } else {
                    // Not a cell, rollback colon
                    self.pos = saved_pos;
                }
            }
            return Ok(Token::Reference(Reference::Cell { sheet: None, cell }));
        }

        // Check if whole column range like "A:A" or "$A:$B"
        if text.chars().all(|c| c.is_ascii_alphabetic() || c == '$')
            && self.peek_char() == Some(':')
        {
            let saved_pos = self.pos;
            self.advance(); // consume ':'
            let mut col2_str = String::new();
            if self.peek_char() == Some('$') {
                col2_str.push(self.advance().unwrap_or('$'));
            }
            while let Some(c) = self.peek_char() {
                if c.is_ascii_alphabetic() {
                    col2_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let full_range_str = format!("{text}:{col2_str}");
            if let Ok(range) = RangeAddress::parse_a1(&full_range_str) {
                return Ok(Token::Reference(Reference::Range { sheet: None, range }));
            }
            self.pos = saved_pos;
        }

        // 6. Otherwise it's a function name or defined identifier (e.g. SUM, AVERAGE, _xlfn.LET)
        Ok(Token::Ident(text))
    }

    fn lex_target_reference_after_sheet(&mut self, sheet: Option<SheetRef>) -> Result<Token> {
        let mut target_str = String::new();
        if self.peek_char() == Some('$') {
            target_str.push(self.advance().unwrap_or('$'));
        }

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '$' {
                target_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for range ':'
        if self.peek_char() == Some(':') {
            self.advance(); // consume ':'
            let mut end_str = String::new();
            if self.peek_char() == Some('$') {
                end_str.push(self.advance().unwrap_or('$'));
            }
            while let Some(c) = self.peek_char() {
                if c.is_ascii_alphanumeric() || c == '$' {
                    end_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let range = RangeAddress::parse_a1(&format!("{target_str}:{end_str}"))?;
            let mut ref_val = Reference::Range { sheet, range };
            if self.peek_char() == Some('#') {
                self.advance();
                ref_val = Reference::Spill(Box::new(ref_val));
            }
            return Ok(Token::Reference(ref_val));
        }

        let cell = CellAddress::parse_a1(&target_str)?;
        let mut ref_val = Reference::Cell { sheet, cell };
        if self.peek_char() == Some('#') {
            self.advance();
            ref_val = Reference::Spill(Box::new(ref_val));
        }
        Ok(Token::Reference(ref_val))
    }
}

fn split_structured_parts(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for c in s.chars() {
        if c == '[' {
            depth += 1;
            current.push(c);
        } else if c == ']' {
            depth -= 1;
            current.push(c);
        } else if c == ',' && depth == 0 {
            if !current.trim().is_empty() {
                parts.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push(c);
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

fn is_r1c1_syntax(s: &str) -> bool {
    let upper = s.to_ascii_uppercase();
    (upper.starts_with('R') && upper.contains('C'))
        && (upper.contains('[') || upper.chars().skip(1).any(|c| c.is_ascii_digit()))
}

fn can_end_operand(t: &Token) -> bool {
    matches!(
        t,
        Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_)
            | Token::Error(_)
            | Token::Reference(_)
            | Token::RParen
            | Token::RBracket
            | Token::RBrace
            | Token::Percent
            | Token::Hash
    )
}
