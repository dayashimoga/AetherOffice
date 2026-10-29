//! PDF 1.5+ Object Stream parser for compressed objects (/Type /ObjStm) (ISO 32000-1 Clause 7.5.7).

use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::parser::PdfParser;
use crate::types::{PdfObject, PdfStream};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ObjectStream {
    objects: BTreeMap<u32, PdfObject>,
    index_map: Vec<u32>,
}

impl ObjectStream {
    pub fn parse_from_stream(stream: &PdfStream) -> Result<Self, PdfError> {
        let n = stream
            .dict
            .get("N")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PdfError::SyntaxError {
                offset: 0,
                message: "Object stream missing /N entry".into(),
            })?
            .max(0) as usize;

        let first = stream
            .dict
            .get("First")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PdfError::SyntaxError {
                offset: 0,
                message: "Object stream missing /First entry".into(),
            })?
            .max(0) as usize;

        let decoded = stream.decode()?;
        if first > decoded.len() {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: format!(
                    "Object stream /First offset {} exceeds decoded length {}",
                    first,
                    decoded.len()
                ),
            });
        }

        // Header parsing: N pairs of (object_number, offset)
        let header_slice = &decoded[..first];
        let mut header_lexer = PdfLexer::new(header_slice);
        let mut pairs = Vec::with_capacity(n);

        for _ in 0..n {
            let obj_tok = header_lexer.next_token()?;
            let off_tok = header_lexer.next_token()?;

            let obj_num = match obj_tok {
                Token::Integer(num) => num.max(0) as u32,
                other => {
                    return Err(PdfError::SyntaxError {
                        offset: header_lexer.pos(),
                        message: format!(
                            "Expected object number in ObjStm header, got {:?}",
                            other
                        ),
                    })
                }
            };

            let offset = match off_tok {
                Token::Integer(off) => off.max(0) as usize,
                other => {
                    return Err(PdfError::SyntaxError {
                        offset: header_lexer.pos(),
                        message: format!("Expected offset in ObjStm header, got {:?}", other),
                    })
                }
            };

            pairs.push((obj_num, offset));
        }

        // Parse each object starting at `first + offset`
        let mut objects = BTreeMap::new();
        let mut index_map = Vec::with_capacity(n);

        for (obj_num, offset) in pairs {
            let abs_offset = first + offset;
            if abs_offset >= decoded.len() {
                return Err(PdfError::SyntaxError {
                    offset: abs_offset,
                    message: "ObjStm object offset exceeds stream bounds".into(),
                });
            }

            let mut obj_parser = PdfParser::with_pos(&decoded, abs_offset);
            let object = obj_parser.parse_object()?;
            objects.insert(obj_num, object);
            index_map.push(obj_num);
        }

        Ok(Self { objects, index_map })
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<(u32, &PdfObject)> {
        self.index_map
            .get(index)
            .and_then(|&num| self.objects.get(&num).map(|obj| (num, obj)))
    }

    #[inline]
    pub fn get_by_number(&self, obj_num: u32) -> Option<&PdfObject> {
        self.objects.get(&obj_num)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}
