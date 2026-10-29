//! High-performance streaming PDF lexical analyzer adhering to ISO 32000-1 Clause 7.2.

use crate::error::PdfError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Integer(i64),
    Real(f64),
    Name(String),
    LiteralString(Vec<u8>),
    HexString(Vec<u8>),
    ArrayOpen,
    ArrayClose,
    DictOpen,
    DictClose,
    Eof,
}

pub struct PdfLexer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PdfLexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn with_pos(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }

    #[inline]
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn peek_byte(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], PdfError> {
        if self.pos + count <= self.data.len() {
            let slice = &self.data[self.pos..self.pos + count];
            self.pos += count;
            Ok(slice)
        } else {
            Err(PdfError::UnexpectedEof)
        }
    }

    pub fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if Self::is_whitespace(b) {
                self.pos += 1;
            } else if b == b'%' {
                // PDF Comment: skip until end of line (LF, CR, or CRLF)
                self.pos += 1;
                while self.pos < self.data.len() {
                    let cb = self.data[self.pos];
                    if cb == b'\n' || cb == b'\r' {
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn is_whitespace(b: u8) -> bool {
        matches!(b, 0x00 | 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
    }

    #[inline]
    pub fn is_delimiter(b: u8) -> bool {
        matches!(
            b,
            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
        )
    }

    pub fn peek_token(&mut self) -> Result<Token, PdfError> {
        let saved_pos = self.pos;
        let tok = self.next_token()?;
        self.pos = saved_pos;
        Ok(tok)
    }

    pub fn next_token(&mut self) -> Result<Token, PdfError> {
        self.skip_whitespace_and_comments();
        if self.is_eof() {
            return Ok(Token::Eof);
        }

        let start_pos = self.pos;
        let b = self.data[self.pos];

        match b {
            b'[' => {
                self.pos += 1;
                Ok(Token::ArrayOpen)
            }
            b']' => {
                self.pos += 1;
                Ok(Token::ArrayClose)
            }
            b'<' => {
                if self.pos + 1 < self.data.len() && self.data[self.pos + 1] == b'<' {
                    self.pos += 2;
                    Ok(Token::DictOpen)
                } else {
                    self.read_hex_string()
                }
            }
            b'>' => {
                if self.pos + 1 < self.data.len() && self.data[self.pos + 1] == b'>' {
                    self.pos += 2;
                    Ok(Token::DictClose)
                } else {
                    Err(PdfError::SyntaxError {
                        offset: start_pos,
                        message: "Unexpected standalone '>' delimiter".into(),
                    })
                }
            }
            b'(' => self.read_literal_string(),
            b'/' => self.read_name(),
            _ => self.read_regular_token(),
        }
    }

    fn read_name(&mut self) -> Result<Token, PdfError> {
        self.pos += 1; // skip leading '/'
        let mut name = String::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if Self::is_whitespace(b) || Self::is_delimiter(b) {
                break;
            }

            if b == b'#' {
                // Hex escape sequence: #xx
                if self.pos + 2 < self.data.len() {
                    let h1 = self.data[self.pos + 1];
                    let h2 = self.data[self.pos + 2];
                    if let (Some(d1), Some(d2)) = (Self::hex_digit(h1), Self::hex_digit(h2)) {
                        let val = (d1 << 4) | d2;
                        name.push(val as char);
                        self.pos += 3;
                        continue;
                    }
                }
            }

            name.push(b as char);
            self.pos += 1;
        }

        Ok(Token::Name(name))
    }

    fn read_literal_string(&mut self) -> Result<Token, PdfError> {
        let start_offset = self.pos;
        self.pos += 1; // skip '('
        let mut depth = 1usize;
        let mut bytes = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;

            match b {
                b'(' => {
                    depth += 1;
                    bytes.push(b'(');
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Token::LiteralString(bytes));
                    }
                    bytes.push(b')');
                }
                b'\\' => {
                    if self.pos >= self.data.len() {
                        break;
                    }
                    let esc = self.data[self.pos];
                    self.pos += 1;
                    match esc {
                        b'n' => bytes.push(b'\n'),
                        b'r' => bytes.push(b'\r'),
                        b't' => bytes.push(b'\t'),
                        b'b' => bytes.push(0x08),
                        b'f' => bytes.push(0x0C),
                        b'(' => bytes.push(b'('),
                        b')' => bytes.push(b')'),
                        b'\\' => bytes.push(b'\\'),
                        b'\r' => {
                            // Check for \r\n line continuation
                            if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
                                self.pos += 1;
                            }
                            // line continuation is ignored
                        }
                        b'\n' => {
                            // line continuation is ignored
                        }
                        b'0'..=b'7' => {
                            // Octal escape \ddd (1 to 3 octal digits)
                            let mut oct_val = (esc - b'0') as u16;
                            let mut digits = 1;
                            while digits < 3 && self.pos < self.data.len() {
                                let ob = self.data[self.pos];
                                if (b'0'..=b'7').contains(&ob) {
                                    oct_val = (oct_val << 3) + (ob - b'0') as u16;
                                    self.pos += 1;
                                    digits += 1;
                                } else {
                                    break;
                                }
                            }
                            bytes.push((oct_val & 0xFF) as u8);
                        }
                        other => {
                            bytes.push(other);
                        }
                    }
                }
                b'\r' => {
                    if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                    bytes.push(b'\n');
                }
                other => {
                    bytes.push(other);
                }
            }
        }

        Err(PdfError::SyntaxError {
            offset: start_offset,
            message: "Unterminated literal string".into(),
        })
    }

    fn read_hex_string(&mut self) -> Result<Token, PdfError> {
        let start_offset = self.pos;
        self.pos += 1; // skip '<'
        let mut digits = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;

            if b == b'>' {
                break;
            }
            if Self::is_whitespace(b) {
                continue;
            }
            if let Some(digit) = Self::hex_digit(b) {
                digits.push(digit);
            } else {
                return Err(PdfError::SyntaxError {
                    offset: self.pos - 1,
                    message: format!("Invalid character '{}' in hex string", b as char),
                });
            }
        }

        // If odd number of digits, append a trailing 0 (ISO 32000-1 Clause 7.3.4.3)
        if digits.len() % 2 != 0 {
            digits.push(0);
        }

        let mut bytes = Vec::with_capacity(digits.len() / 2);
        for chunk in digits.chunks(2) {
            bytes.push((chunk[0] << 4) | chunk[1]);
        }

        if self.pos > self.data.len() && self.data.last() != Some(&b'>') {
            return Err(PdfError::SyntaxError {
                offset: start_offset,
                message: "Unterminated hexadecimal string".into(),
            });
        }

        Ok(Token::HexString(bytes))
    }

    fn read_regular_token(&mut self) -> Result<Token, PdfError> {
        let start = self.pos;
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if Self::is_whitespace(b) || Self::is_delimiter(b) {
                break;
            }
            self.pos += 1;
        }

        let slice = &self.data[start..self.pos];
        let text = match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => {
                return Err(PdfError::SyntaxError {
                    offset: start,
                    message: "Non-UTF-8 token in PDF stream".into(),
                })
            }
        };

        // Check if integer
        if let Ok(i) = text.parse::<i64>() {
            return Ok(Token::Integer(i));
        }

        // Check if real number
        if let Ok(f) = text.parse::<f64>() {
            return Ok(Token::Real(f));
        }

        // Check special keyword tokens
        Ok(Token::Keyword(text.to_string()))
    }

    #[inline]
    fn hex_digit(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
}
