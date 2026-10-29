//! PDF Cross-Reference (XRef) table and stream parser (ISO 32000-1 Clause 7.5).

use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::parser::PdfParser;
use crate::types::{PdfObject, PdfObjectId, PdfStream};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XrefEntry {
    Free {
        next_free_obj: u32,
        generation: u16,
    },
    Uncompressed {
        byte_offset: u64,
        generation: u16,
    },
    Compressed {
        stream_obj: u32,
        index_in_stream: u32,
    },
}

type XrefSection = (
    BTreeMap<u32, XrefEntry>,
    BTreeMap<String, PdfObject>,
    Option<usize>,
);
type XrefSectionResult = Result<XrefSection, PdfError>;

#[derive(Debug, Clone, PartialEq)]
pub struct XrefTable {
    pub entries: BTreeMap<u32, XrefEntry>,
    pub trailer: BTreeMap<String, PdfObject>,
}

impl XrefTable {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            trailer: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn get_entry(&self, obj_num: u32) -> Option<&XrefEntry> {
        self.entries.get(&obj_num)
    }

    #[inline]
    pub fn trailer(&self) -> &BTreeMap<String, PdfObject> {
        &self.trailer
    }

    pub fn root_id(&self) -> Result<PdfObjectId, PdfError> {
        self.trailer
            .get("Root")
            .and_then(|obj| obj.as_reference())
            .ok_or_else(|| {
                PdfError::InvalidTrailer("Missing /Root document catalog reference".into())
            })
    }

    pub fn info_id(&self) -> Option<PdfObjectId> {
        self.trailer.get("Info").and_then(|obj| obj.as_reference())
    }

    /// Finds `startxref` offset from the end of the PDF bytes.
    pub fn find_startxref_offset(data: &[u8]) -> Result<usize, PdfError> {
        let search_len = 2048.min(data.len());
        let search_slice = &data[data.len() - search_len..];

        let marker = b"startxref";
        let rel_pos = search_slice
            .windows(marker.len())
            .rposition(|w| w == marker)
            .ok_or_else(|| {
                PdfError::InvalidTrailer("Could not locate 'startxref' marker in PDF".into())
            })?;

        let abs_pos = data.len() - search_len + rel_pos + marker.len();
        let mut lexer = PdfLexer::with_pos(data, abs_pos);

        match lexer.next_token()? {
            Token::Integer(offset) if offset >= 0 => Ok(offset as usize),
            other => Err(PdfError::InvalidTrailer(format!(
                "Expected integer offset after 'startxref', got {:?}",
                other
            ))),
        }
    }

    /// Reads and parses the complete XRef hierarchy, following `/Prev` chains.
    pub fn parse_from_data(data: &[u8]) -> Result<Self, PdfError> {
        let start_offset = Self::find_startxref_offset(data)?;
        let mut table = Self::new();
        let mut current_offset = Some(start_offset);
        let mut visited_offsets = std::collections::BTreeSet::new();

        while let Some(offset) = current_offset {
            if !visited_offsets.insert(offset) {
                return Err(PdfError::InvalidXref(format!(
                    "Circular /Prev loop detected at offset {}",
                    offset
                )));
            }

            let (section_entries, section_trailer, prev_offset) =
                Self::parse_section_at(data, offset)?;

            // In incremental updates, later entries take precedence over earlier ones:
            for (obj_num, entry) in section_entries {
                table.entries.entry(obj_num).or_insert(entry);
            }

            // Merge trailer keys (earlier sections do not overwrite newer trailer values)
            for (k, v) in section_trailer {
                table.trailer.entry(k).or_insert(v);
            }

            current_offset = prev_offset;
        }

        Ok(table)
    }

    /// Parses either a traditional `xref` table or a PDF 1.5+ XRef stream at a specific offset.
    fn parse_section_at(data: &[u8], offset: usize) -> XrefSectionResult {
        if offset >= data.len() {
            return Err(PdfError::InvalidXref(format!(
                "XRef offset {} out of bounds (file size: {})",
                offset,
                data.len()
            )));
        }

        let mut lexer = PdfLexer::with_pos(data, offset);
        lexer.skip_whitespace_and_comments();

        let peeked = lexer.peek_token()?;
        match peeked {
            Token::Keyword(ref kw) if kw == "xref" => Self::parse_classic_xref(&mut lexer),
            Token::Integer(_) => Self::parse_xref_stream(data, offset),
            other => Err(PdfError::InvalidXref(format!(
                "Unexpected token at XRef offset {}: {:?}",
                offset, other
            ))),
        }
    }

    /// Parses a classic ASCII `xref` table with subsections and trailer dictionary.
    fn parse_classic_xref(lexer: &mut PdfLexer) -> XrefSectionResult {
        lexer.next_token()?; // consume 'xref'
        let mut entries = BTreeMap::new();

        loop {
            lexer.skip_whitespace_and_comments();
            let tok = lexer.peek_token()?;

            match tok {
                Token::Keyword(ref kw) if kw == "trailer" => {
                    lexer.next_token()?; // consume 'trailer'
                    let mut parser = PdfParser::new(PdfLexer::with_pos(lexer.data(), lexer.pos()));
                    let trailer_obj = parser.parse_object()?;
                    let trailer_dict = match trailer_obj {
                        PdfObject::Dictionary(d) => d,
                        other => {
                            return Err(PdfError::InvalidTrailer(format!(
                                "Expected dictionary for trailer, got {:?}",
                                other
                            )))
                        }
                    };

                    let prev_offset = trailer_dict
                        .get("Prev")
                        .and_then(|v| v.as_i64())
                        .map(|o| o.max(0) as usize);

                    return Ok((entries, trailer_dict, prev_offset));
                }
                Token::Integer(start_obj_num) => {
                    lexer.next_token()?; // consume start_obj_num
                    let count_tok = lexer.next_token()?;
                    let count = match count_tok {
                        Token::Integer(c) => c.max(0) as u32,
                        other => {
                            return Err(PdfError::InvalidXref(format!(
                                "Expected subsection count, got {:?}",
                                other
                            )))
                        }
                    };

                    let first_obj = start_obj_num.max(0) as u32;
                    for current_obj in first_obj..(first_obj.saturating_add(count)) {
                        lexer.skip_whitespace_and_comments();
                        let offset_tok = lexer.next_token()?;
                        let offset = match offset_tok {
                            Token::Integer(o) => o.max(0) as u64,
                            other => {
                                return Err(PdfError::InvalidXref(format!(
                                    "Expected entry offset, got {:?}",
                                    other
                                )))
                            }
                        };

                        let gen_tok = lexer.next_token()?;
                        let gen = match gen_tok {
                            Token::Integer(g) => g.max(0) as u16,
                            other => {
                                return Err(PdfError::InvalidXref(format!(
                                    "Expected entry generation, got {:?}",
                                    other
                                )))
                            }
                        };

                        let flag_tok = lexer.next_token()?;
                        let flag = match flag_tok {
                            Token::Keyword(k) => k,
                            other => {
                                return Err(PdfError::InvalidXref(format!(
                                    "Expected entry flag 'n' or 'f', got {:?}",
                                    other
                                )))
                            }
                        };

                        if flag == "n" {
                            entries.insert(
                                current_obj,
                                XrefEntry::Uncompressed {
                                    byte_offset: offset,
                                    generation: gen,
                                },
                            );
                        } else if flag == "f" {
                            entries.insert(
                                current_obj,
                                XrefEntry::Free {
                                    next_free_obj: offset as u32,
                                    generation: gen,
                                },
                            );
                        }
                    }
                }
                other => {
                    return Err(PdfError::InvalidXref(format!(
                        "Unexpected token in xref table: {:?}",
                        other
                    )))
                }
            }
        }
    }

    /// Parses a PDF 1.5+ compressed Cross-Reference Stream (`/Type /XRef`).
    fn parse_xref_stream(data: &[u8], offset: usize) -> XrefSectionResult {
        let mut parser = PdfParser::with_pos(data, offset);
        let (_id, obj) = parser.parse_indirect_object()?;

        let stream: &PdfStream = match obj {
            PdfObject::Stream(ref s) => s,
            other => {
                return Err(PdfError::InvalidXref(format!(
                    "Expected XRef stream object at offset {}, found {:?}",
                    offset, other
                )))
            }
        };

        let dict = &stream.dict;
        let is_xref = dict
            .get("Type")
            .and_then(|v| v.as_name())
            .map(|n| n == "XRef")
            .unwrap_or(false);

        if !is_xref {
            return Err(PdfError::InvalidXref(
                "Indirect object is not of /Type /XRef".into(),
            ));
        }

        let size = dict
            .get("Size")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PdfError::InvalidXref("XRef stream missing /Size".into()))?
            .max(0) as u32;

        // /W array [w0, w1, w2]
        let w_arr = dict
            .get("W")
            .and_then(|v| v.as_array())
            .ok_or_else(|| PdfError::InvalidXref("XRef stream missing /W array".into()))?;

        if w_arr.len() < 3 {
            return Err(PdfError::InvalidXref(
                "XRef stream /W array must have at least 3 elements".into(),
            ));
        }

        let w0 = w_arr[0].as_i64().unwrap_or(0).max(0) as usize;
        let w1 = w_arr[1].as_i64().unwrap_or(0).max(0) as usize;
        let w2 = w_arr[2].as_i64().unwrap_or(0).max(0) as usize;
        let entry_len = w0 + w1 + w2;

        if entry_len == 0 {
            return Err(PdfError::InvalidXref("XRef stream /W sum is zero".into()));
        }

        // /Index array [start1, count1, start2, count2, ...] defaults to [0, Size]
        let mut subsections = Vec::new();
        if let Some(index_obj) = dict.get("Index").and_then(|v| v.as_array()) {
            for chunk in index_obj.chunks(2) {
                if chunk.len() == 2 {
                    let start = chunk[0].as_i64().unwrap_or(0).max(0) as u32;
                    let count = chunk[1].as_i64().unwrap_or(0).max(0) as u32;
                    subsections.push((start, count));
                }
            }
        } else {
            subsections.push((0, size));
        }

        let decoded_bytes = stream.decode()?;
        let mut byte_pos = 0;
        let mut entries = BTreeMap::new();

        for (start_obj, count) in subsections {
            for obj_id in start_obj..(start_obj.saturating_add(count)) {
                if byte_pos + entry_len > decoded_bytes.len() {
                    break;
                }

                let f0 = Self::read_uint(&decoded_bytes[byte_pos..byte_pos + w0]);
                byte_pos += w0;
                let f1 = Self::read_uint(&decoded_bytes[byte_pos..byte_pos + w1]);
                byte_pos += w1;
                let f2 = Self::read_uint(&decoded_bytes[byte_pos..byte_pos + w2]);
                byte_pos += w2;

                // If w0 == 0, the type defaults to 1
                let entry_type = if w0 == 0 { 1 } else { f0 };

                match entry_type {
                    0 => {
                        entries.insert(
                            obj_id,
                            XrefEntry::Free {
                                next_free_obj: f1 as u32,
                                generation: f2 as u16,
                            },
                        );
                    }
                    1 => {
                        entries.insert(
                            obj_id,
                            XrefEntry::Uncompressed {
                                byte_offset: f1,
                                generation: f2 as u16,
                            },
                        );
                    }
                    2 => {
                        entries.insert(
                            obj_id,
                            XrefEntry::Compressed {
                                stream_obj: f1 as u32,
                                index_in_stream: f2 as u32,
                            },
                        );
                    }
                    _ => {} // ignore unrecognized types per spec
                }
            }
        }

        let prev_offset = dict
            .get("Prev")
            .and_then(|v| v.as_i64())
            .map(|o| o.max(0) as usize);

        Ok((entries, dict.clone(), prev_offset))
    }

    #[inline]
    fn read_uint(bytes: &[u8]) -> u64 {
        let mut res = 0u64;
        for &b in bytes {
            res = (res << 8) | (b as u64);
        }
        res
    }
}

impl Default for XrefTable {
    fn default() -> Self {
        Self::new()
    }
}
