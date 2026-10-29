//! Recursive descent PDF object parser (ISO 32000-1 Clause 7.3).

use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::types::{PdfObject, PdfObjectId, PdfStream};
use std::collections::BTreeMap;

pub struct PdfParser<'a> {
    lexer: PdfLexer<'a>,
}

impl<'a> PdfParser<'a> {
    pub fn new(lexer: PdfLexer<'a>) -> Self {
        Self { lexer }
    }

    pub fn from_data(data: &'a [u8]) -> Self {
        Self {
            lexer: PdfLexer::new(data),
        }
    }

    pub fn with_pos(data: &'a [u8], pos: usize) -> Self {
        Self {
            lexer: PdfLexer::with_pos(data, pos),
        }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.lexer.pos()
    }

    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.lexer.set_pos(pos);
    }

    #[inline]
    pub fn lexer(&mut self) -> &mut PdfLexer<'a> {
        &mut self.lexer
    }

    /// Parses the next PDF object from the token stream.
    pub fn parse_object(&mut self) -> Result<PdfObject, PdfError> {
        let token = self.lexer.next_token()?;
        self.parse_object_from_token(token)
    }

    /// Internal recursive dispatcher from a peeked or consumed token.
    pub fn parse_object_from_token(&mut self, token: Token) -> Result<PdfObject, PdfError> {
        match token {
            Token::Keyword(kw) => match kw.as_str() {
                "null" => Ok(PdfObject::Null),
                "true" => Ok(PdfObject::Boolean(true)),
                "false" => Ok(PdfObject::Boolean(false)),
                other => Err(PdfError::SyntaxError {
                    offset: self.lexer.pos(),
                    message: format!("Unexpected keyword '{}' as object", other),
                }),
            },
            Token::Integer(num) => {
                // Lookahead to check if this is the start of an indirect reference: `num gen R`
                let saved_pos = self.lexer.pos();
                if let Ok(Token::Integer(gen)) = self.lexer.next_token() {
                    if let Ok(Token::Keyword(kw)) = self.lexer.next_token() {
                        if kw == "R" {
                            return Ok(PdfObject::Reference(PdfObjectId::new(
                                num.max(0) as u32,
                                gen.max(0) as u16,
                            )));
                        }
                    }
                }
                // Backtrack if not an indirect reference
                self.lexer.set_pos(saved_pos);
                Ok(PdfObject::Integer(num))
            }
            Token::Real(f) => Ok(PdfObject::Real(f)),
            Token::Name(name) => Ok(PdfObject::Name(name)),
            Token::LiteralString(s) => Ok(PdfObject::LiteralString(s)),
            Token::HexString(s) => Ok(PdfObject::HexString(s)),
            Token::ArrayOpen => self.parse_array(),
            Token::DictOpen => self.parse_dictionary(),
            Token::DictClose | Token::ArrayClose => Err(PdfError::SyntaxError {
                offset: self.lexer.pos(),
                message: "Unexpected closing delimiter".into(),
            }),
            Token::Eof => Err(PdfError::UnexpectedEof),
        }
    }

    /// Parses an array `[ obj1 obj2 ... objN ]`.
    pub fn parse_array(&mut self) -> Result<PdfObject, PdfError> {
        let mut items = Vec::new();
        loop {
            self.lexer.skip_whitespace_and_comments();
            if self.lexer.is_eof() {
                return Err(PdfError::UnexpectedEof);
            }

            let peeked = self.lexer.peek_token()?;
            if peeked == Token::ArrayClose {
                self.lexer.next_token()?; // consume ']'
                break;
            }

            let obj = self.parse_object()?;
            items.push(obj);
        }
        Ok(PdfObject::Array(items))
    }

    /// Parses a dictionary `<< /Key1 Val1 /Key2 Val2 ... >>`.
    pub fn parse_dictionary(&mut self) -> Result<PdfObject, PdfError> {
        let mut dict = BTreeMap::new();
        loop {
            self.lexer.skip_whitespace_and_comments();
            if self.lexer.is_eof() {
                return Err(PdfError::UnexpectedEof);
            }

            let tok = self.lexer.next_token()?;
            match tok {
                Token::DictClose => break,
                Token::Name(key) => {
                    let val = self.parse_object()?;
                    dict.insert(key, val);
                }
                other => {
                    return Err(PdfError::SyntaxError {
                        offset: self.lexer.pos(),
                        message: format!("Expected dictionary key name, found {:?}", other),
                    });
                }
            }
        }
        Ok(PdfObject::Dictionary(dict))
    }

    /// Parses an indirect object definition `12 0 obj ... endobj`.
    pub fn parse_indirect_object(&mut self) -> Result<(PdfObjectId, PdfObject), PdfError> {
        let num_tok = self.lexer.next_token()?;
        let num = match num_tok {
            Token::Integer(n) => n.max(0) as u32,
            other => {
                return Err(PdfError::SyntaxError {
                    offset: self.lexer.pos(),
                    message: format!("Expected object number, got {:?}", other),
                })
            }
        };

        let gen_tok = self.lexer.next_token()?;
        let gen = match gen_tok {
            Token::Integer(g) => g.max(0) as u16,
            other => {
                return Err(PdfError::SyntaxError {
                    offset: self.lexer.pos(),
                    message: format!("Expected generation number, got {:?}", other),
                })
            }
        };

        let obj_tok = self.lexer.next_token()?;
        match obj_tok {
            Token::Keyword(ref kw) if kw == "obj" => {}
            other => {
                return Err(PdfError::SyntaxError {
                    offset: self.lexer.pos(),
                    message: format!("Expected 'obj' keyword, got {:?}", other),
                })
            }
        }

        let mut object = self.parse_object()?;

        // If the parsed object is a Dictionary, check if a stream follows:
        if let PdfObject::Dictionary(dict) = object {
            self.lexer.skip_whitespace_and_comments();
            let saved_pos = self.lexer.pos();
            let is_stream = if let Ok(Token::Keyword(ref kw)) = self.lexer.peek_token() {
                kw == "stream"
            } else {
                false
            };

            if is_stream {
                self.lexer.next_token()?; // consume "stream"
                let stream = self.parse_stream_body(dict)?;
                object = PdfObject::Stream(stream);
            } else {
                self.lexer.set_pos(saved_pos);
                object = PdfObject::Dictionary(dict);
            }
        }

        // Consume `endobj` if present
        self.lexer.skip_whitespace_and_comments();
        if let Ok(Token::Keyword(ref kw)) = self.lexer.peek_token() {
            if kw == "endobj" {
                self.lexer.next_token()?;
            }
        }

        Ok((PdfObjectId::new(num, gen), object))
    }

    /// Reads raw stream data after the `stream` keyword up to `endstream`.
    fn parse_stream_body(
        &mut self,
        dict: BTreeMap<String, PdfObject>,
    ) -> Result<PdfStream, PdfError> {
        // According to ISO 32000-1 Clause 7.3.8.1:
        // The keyword 'stream' should be followed by either CRLF or LF.
        let data = self.lexer.data();
        let mut cur = self.lexer.pos();

        if cur < data.len() && data[cur] == b'\r' {
            cur += 1;
        }
        if cur < data.len() && data[cur] == b'\n' {
            cur += 1;
        }
        let stream_start = cur;

        // Try reading length from dictionary if available as direct integer
        let declared_len = dict.get("Length").and_then(|obj| obj.as_i64());

        let stream_bytes = if let Some(len) = declared_len {
            let ulen = len.max(0) as usize;
            if stream_start + ulen <= data.len() {
                let candidate = &data[stream_start..stream_start + ulen];
                // Check if endstream is directly after candidate
                let mut check_pos = stream_start + ulen;
                while check_pos < data.len() && PdfLexer::is_whitespace(data[check_pos]) {
                    check_pos += 1;
                }
                if check_pos + 9 <= data.len() && &data[check_pos..check_pos + 9] == b"endstream" {
                    self.lexer.set_pos(check_pos + 9);
                    candidate.to_vec()
                } else {
                    self.find_endstream(stream_start)?
                }
            } else {
                self.find_endstream(stream_start)?
            }
        } else {
            self.find_endstream(stream_start)?
        };

        Ok(PdfStream::new(dict, stream_bytes))
    }

    fn find_endstream(&mut self, stream_start: usize) -> Result<Vec<u8>, PdfError> {
        let data = self.lexer.data();
        let target = b"endstream";
        let endstream_idx = data[stream_start..]
            .windows(target.len())
            .position(|window| window == target)
            .map(|idx| stream_start + idx)
            .ok_or_else(|| PdfError::SyntaxError {
                offset: stream_start,
                message: "Missing 'endstream' marker".into(),
            })?;

        // Trim trailing CR, LF or CRLF right before 'endstream'
        let mut stream_end = endstream_idx;
        if stream_end > stream_start && data[stream_end - 1] == b'\n' {
            stream_end -= 1;
        }
        if stream_end > stream_start && data[stream_end - 1] == b'\r' {
            stream_end -= 1;
        }

        self.lexer.set_pos(endstream_idx + target.len());
        Ok(data[stream_start..stream_end].to_vec())
    }
}
