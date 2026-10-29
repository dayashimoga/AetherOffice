//! Multi-Revision Document Inspector for Incremental PDF Updates (ISO 32000-1 Clause 7.5.6).

use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::parser::PdfParser;
use crate::types::PdfObject;
use crate::xref::XrefTable;
use std::collections::{BTreeMap, BTreeSet};

type RawRevisionTuple = (BTreeMap<String, PdfObject>, Vec<u32>, Option<usize>, usize);
type RawRevisionResult = Result<RawRevisionTuple, PdfError>;

/// Historic document revision record.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfRevision {
    pub revision_index: usize,
    pub xref_offset: usize,
    pub file_end_offset: usize,
    pub trailer: BTreeMap<String, PdfObject>,
    pub modified_objects: Vec<u32>,
}

/// Ordered sequence of revisions making up the current document.
#[derive(Debug, Clone, Default)]
pub struct RevisionHistory {
    pub revisions: Vec<PdfRevision>,
}

impl RevisionHistory {
    /// Inspects and traces all historic revisions by traversing `/Prev` pointers.
    pub fn inspect(data: &[u8]) -> Result<Self, PdfError> {
        let mut raw_revs = Vec::new();
        let mut current_xref_offset = Some(XrefTable::find_startxref_offset(data)?);
        let mut visited = BTreeSet::new();

        while let Some(offset) = current_xref_offset {
            if !visited.insert(offset) {
                return Err(PdfError::InvalidXref(format!(
                    "Circular /Prev loop detected at offset {}",
                    offset
                )));
            }

            let (trailer_dict, modified_objs, prev_offset, file_end) =
                Self::parse_revision_at(data, offset)?;

            raw_revs.push((offset, file_end, trailer_dict, modified_objs));
            current_xref_offset = prev_offset;
        }

        // Reverse so that index 0 is the initial version, followed by updates:
        raw_revs.reverse();

        let mut revisions = Vec::with_capacity(raw_revs.len());
        for (idx, (xref_offset, file_end_offset, trailer, modified_objects)) in
            raw_revs.into_iter().enumerate()
        {
            revisions.push(PdfRevision {
                revision_index: idx,
                xref_offset,
                file_end_offset,
                trailer,
                modified_objects,
            });
        }

        Ok(Self { revisions })
    }

    pub fn count(&self) -> usize {
        self.revisions.len()
    }

    pub fn revision(&self, index: usize) -> Option<&PdfRevision> {
        self.revisions.get(index)
    }

    /// Slices the byte buffer up to the given revision's end, recovering the exact file state at that revision.
    pub fn extract_revision_bytes<'a>(
        &self,
        revision_index: usize,
        full_data: &'a [u8],
    ) -> Result<&'a [u8], PdfError> {
        let rev = self
            .revisions
            .get(revision_index)
            .ok_or(PdfError::PageOutOfRange {
                index: revision_index,
                count: self.revisions.len(),
            })?;

        let end = rev.file_end_offset.min(full_data.len());
        Ok(&full_data[..end])
    }

    fn parse_revision_at(data: &[u8], offset: usize) -> RawRevisionResult {
        if offset >= data.len() {
            return Err(PdfError::InvalidXref(format!(
                "XRef offset {} exceeds file length {}",
                offset,
                data.len()
            )));
        }

        let mut lexer = PdfLexer::with_pos(data, offset);
        lexer.skip_whitespace_and_comments();
        let tok = lexer.next_token()?;

        let mut modified_objs = Vec::new();
        let mut trailer_dict: BTreeMap<String, PdfObject> = BTreeMap::new();
        let prev_offset: Option<usize>;

        match tok {
            Token::Keyword(ref kw) if kw == "xref" => {
                // Classic xref table
                loop {
                    lexer.skip_whitespace_and_comments();
                    let t = lexer.peek_token()?;
                    match t {
                        Token::Keyword(ref k) if k == "trailer" => {
                            lexer.next_token()?;
                            let mut parser =
                                PdfParser::new(PdfLexer::with_pos(lexer.data(), lexer.pos()));
                            let obj = parser.parse_object()?;
                            lexer.set_pos(parser.pos());
                            trailer_dict = match obj {
                                PdfObject::Dictionary(d) => d,
                                other => {
                                    return Err(PdfError::InvalidTrailer(format!(
                                        "Expected trailer dictionary, got {:?}",
                                        other
                                    )))
                                }
                            };
                            break;
                        }
                        Token::Integer(start_num) => {
                            lexer.next_token()?;
                            let count_tok = lexer.next_token()?;
                            let count = match count_tok {
                                Token::Integer(c) => c.max(0) as u32,
                                _ => 0,
                            };
                            for current_obj in
                                (start_num.max(0) as u32)..(start_num.max(0) as u32 + count)
                            {
                                lexer.next_token()?; // offset
                                lexer.next_token()?; // gen
                                if let Ok(Token::Keyword(flag)) = lexer.next_token() {
                                    if flag == "n" {
                                        modified_objs.push(current_obj);
                                    }
                                }
                            }
                        }
                        _ => break,
                    }
                }

                prev_offset = trailer_dict
                    .get("Prev")
                    .and_then(|v| v.as_i64())
                    .map(|o| o.max(0) as usize);
            }
            Token::Integer(_) => {
                // XRef stream
                let mut parser = PdfParser::with_pos(data, offset);
                let (_id, obj) = parser.parse_indirect_object()?;
                lexer.set_pos(parser.pos());
                let dict = match obj {
                    PdfObject::Stream(ref s) => s.dict.clone(),
                    other => {
                        return Err(PdfError::InvalidXref(format!(
                            "Expected XRef stream object, found {:?}",
                            other
                        )))
                    }
                };

                prev_offset = dict
                    .get("Prev")
                    .and_then(|v| v.as_i64())
                    .map(|o| o.max(0) as usize);
                trailer_dict = dict;
            }
            other => {
                return Err(PdfError::InvalidXref(format!(
                    "Unexpected token at XRef offset {}: {:?}",
                    offset, other
                )))
            }
        }

        // Find %%EOF for this revision
        let search_start = lexer.pos();
        let target = b"%%EOF";
        let eof_rel = data[search_start..]
            .windows(target.len())
            .position(|w| w == target);

        let file_end = match eof_rel {
            Some(rel) => search_start + rel + target.len(),
            None => data.len(),
        };

        Ok((trailer_dict, modified_objs, prev_offset, file_end))
    }
}
