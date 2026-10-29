//! PDF Font CMap and Unicode Translation Engine (ISO 32000-1 Clause 9.6 - 9.10).

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::types::PdfObject;
use std::collections::BTreeMap;

/// CMap mapping character codes (1-byte or 2-byte CIDs) to Unicode strings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontCMap {
    mappings: BTreeMap<u32, String>,
}

impl FontCMap {
    pub fn new() -> Self {
        Self {
            mappings: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, char_code: u32, unicode: String) {
        self.mappings.insert(char_code, unicode);
    }

    #[inline]
    pub fn lookup(&self, char_code: u32) -> Option<&str> {
        self.mappings.get(&char_code).map(|s| s.as_str())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Parses a `/ToUnicode` PostScript CMap stream.
    pub fn parse_to_unicode(data: &[u8]) -> Result<Self, PdfError> {
        let mut cmap = Self::new();
        let mut lexer = PdfLexer::new(data);

        while !lexer.is_eof() {
            lexer.skip_whitespace_and_comments();
            if lexer.is_eof() {
                break;
            }

            let tok = lexer.next_token()?;
            if let Token::Keyword(kw) = tok {
                match kw.as_str() {
                    "beginbfchar" => {
                        Self::parse_bfchar(&mut lexer, &mut cmap)?;
                    }
                    "beginbfrange" => {
                        Self::parse_bfrange(&mut lexer, &mut cmap)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(cmap)
    }

    fn parse_bfchar(lexer: &mut PdfLexer, cmap: &mut FontCMap) -> Result<(), PdfError> {
        loop {
            lexer.skip_whitespace_and_comments();
            if lexer.is_eof() {
                break;
            }

            let tok = lexer.peek_token()?;
            if let Token::Keyword(ref kw) = tok {
                if kw == "endbfchar" {
                    lexer.next_token()?;
                    break;
                }
            }

            // Pair: <srcCode> <dstUnicode>
            let src_tok = lexer.next_token()?;
            let dst_tok = lexer.next_token()?;

            if let (Token::HexString(src_bytes), Token::HexString(dst_bytes)) = (src_tok, dst_tok) {
                let src_code = Self::bytes_to_code(&src_bytes);
                let uni_str = Self::decode_utf16_be(&dst_bytes);
                cmap.insert(src_code, uni_str);
            }
        }
        Ok(())
    }

    fn parse_bfrange(lexer: &mut PdfLexer, cmap: &mut FontCMap) -> Result<(), PdfError> {
        loop {
            lexer.skip_whitespace_and_comments();
            if lexer.is_eof() {
                break;
            }

            let tok = lexer.peek_token()?;
            if let Token::Keyword(ref kw) = tok {
                if kw == "endbfrange" {
                    lexer.next_token()?;
                    break;
                }
            }

            let start_tok = lexer.next_token()?;
            let end_tok = lexer.next_token()?;
            let dest_tok = lexer.next_token()?;

            if let (Token::HexString(start_bytes), Token::HexString(end_bytes)) =
                (start_tok, end_tok)
            {
                let start_code = Self::bytes_to_code(&start_bytes);
                let end_code = Self::bytes_to_code(&end_bytes);

                match dest_tok {
                    Token::HexString(dst_bytes) => {
                        // Range mapping with sequential destination
                        let mut dst_code = Self::bytes_to_code(&dst_bytes);
                        for code in start_code..=end_code {
                            let uni_str = Self::code_to_unicode(dst_code);
                            cmap.insert(code, uni_str);
                            dst_code = dst_code.saturating_add(1);
                        }
                    }
                    Token::ArrayOpen => {
                        // Range mapping with explicit array of destinations
                        let mut cur_code = start_code;
                        loop {
                            lexer.skip_whitespace_and_comments();
                            let item_tok = lexer.next_token()?;
                            match item_tok {
                                Token::ArrayClose => break,
                                Token::HexString(b) if cur_code <= end_code => {
                                    let uni = Self::decode_utf16_be(&b);
                                    cmap.insert(cur_code, uni);
                                    cur_code = cur_code.saturating_add(1);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn bytes_to_code(bytes: &[u8]) -> u32 {
        let mut code = 0u32;
        for &b in bytes {
            code = (code << 8) | (b as u32);
        }
        code
    }

    /// Decodes big-endian UTF-16 bytes into a Rust String (handling surrogates).
    fn decode_utf16_be(bytes: &[u8]) -> String {
        if !bytes.len().is_multiple_of(2) {
            return String::from_utf8_lossy(bytes).into_owned();
        }

        let mut u16_words = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks(2) {
            u16_words.push(((chunk[0] as u16) << 8) | (chunk[1] as u16));
        }

        String::from_utf16_lossy(&u16_words)
    }

    fn code_to_unicode(code: u32) -> String {
        if let Some(ch) = char::from_u32(code) {
            ch.to_string()
        } else if code <= 0xFFFF {
            String::from_utf16_lossy(&[code as u16])
        } else {
            // Surrogate pair
            let h = (((code - 0x10000) >> 10) + 0xD800) as u16;
            let l = (((code - 0x10000) & 0x3FF) + 0xDC00) as u16;
            String::from_utf16_lossy(&[h, l])
        }
    }
}

/// Standard 8-bit WinAnsiEncoding translation table.
pub fn win_ansi_to_unicode(code: u8) -> char {
    match code {
        0x80 => '\u{20AC}', // Euro
        0x82 => '\u{201A}', // Single low-9 quotation mark
        0x83 => '\u{0192}', // Latin small letter f with hook
        0x84 => '\u{201E}', // Double low-9 quotation mark
        0x85 => '\u{2026}', // Horizontal ellipsis
        0x86 => '\u{2020}', // Dagger
        0x87 => '\u{2021}', // Double dagger
        0x88 => '\u{02C6}', // Modifier letter circumflex accent
        0x89 => '\u{2030}', // Per mille sign
        0x8A => '\u{0160}', // Latin capital letter S with caron
        0x8B => '\u{2039}', // Single left-pointing angle quotation mark
        0x8C => '\u{0152}', // Latin capital ligature OE
        0x8E => '\u{017D}', // Latin capital letter Z with caron
        0x91 => '\u{2018}', // Left single quotation mark
        0x92 => '\u{2019}', // Right single quotation mark
        0x93 => '\u{201C}', // Left double quotation mark
        0x94 => '\u{201D}', // Right double quotation mark
        0x95 => '\u{2022}', // Bullet
        0x96 => '\u{2013}', // En dash
        0x97 => '\u{2014}', // Em dash
        0x98 => '\u{02DC}', // Small tilde
        0x99 => '\u{2122}', // Trade mark sign
        0x9A => '\u{0161}', // Latin small letter s with caron
        0x9B => '\u{203A}', // Single right-pointing angle quotation mark
        0x9C => '\u{0153}', // Latin small ligature oe
        0x9E => '\u{017E}', // Latin small letter z with caron
        0x9F => '\u{0178}', // Latin capital letter Y with diaeresis
        c => c as char,     // ISO-8859-1 matches Unicode 1:1 for 0x00-0x7F and 0xA0-0xFF
    }
}

/// Resolved font model holding metrics, widths, and encoding tables.
#[derive(Debug, Clone)]
pub struct PdfFont {
    pub base_font: String,
    pub subtype: String,
    pub first_char: u32,
    pub last_char: u32,
    pub widths: BTreeMap<u32, f64>,
    pub default_width: f64,
    pub cmap: Option<FontCMap>,
    pub encoding: Option<String>,
}

impl PdfFont {
    pub fn new(base_font: String, subtype: String) -> Self {
        Self {
            base_font,
            subtype,
            first_char: 0,
            last_char: 255,
            widths: BTreeMap::new(),
            default_width: 1000.0,
            cmap: None,
            encoding: Some("WinAnsiEncoding".into()),
        }
    }

    /// Resolves font attributes and ToUnicode CMap from a `/Font` dictionary.
    pub fn from_dict(
        dict: &BTreeMap<String, PdfObject>,
        doc: &PdfDocument,
    ) -> Result<Self, PdfError> {
        let base_font = dict
            .get("BaseFont")
            .and_then(|v| v.as_name())
            .unwrap_or("Helvetica")
            .to_string();

        let subtype = dict
            .get("Subtype")
            .and_then(|v| v.as_name())
            .unwrap_or("Type1")
            .to_string();

        let first_char = dict
            .get("FirstChar")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(0) as u32;

        let last_char = dict
            .get("LastChar")
            .and_then(|v| v.as_i64())
            .unwrap_or(255)
            .max(0) as u32;

        let mut widths = BTreeMap::new();
        if let Some(w_obj) = dict.get("Widths") {
            let arr = match w_obj {
                PdfObject::Array(arr) => arr.clone(),
                PdfObject::Reference(ref_id) => {
                    if let Ok(PdfObject::Array(arr)) = doc.resolve_object(*ref_id) {
                        arr
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };

            for (idx, item) in arr.iter().enumerate() {
                if let Some(w) = item.as_f64() {
                    widths.insert(first_char + idx as u32, w);
                }
            }
        }

        let encoding = dict
            .get("Encoding")
            .and_then(|v| v.as_name())
            .map(String::from);

        // Resolve /ToUnicode CMap if present
        let mut cmap = None;
        if let Some(tu_obj) = dict.get("ToUnicode") {
            let stream = match tu_obj {
                PdfObject::Stream(s) => Some(s.clone()),
                PdfObject::Reference(ref_id) => {
                    if let Ok(PdfObject::Stream(s)) = doc.resolve_object(*ref_id) {
                        Some(s)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(ref stm) = stream {
                if let Ok(decoded) = stm.decode() {
                    if let Ok(parsed_cmap) = FontCMap::parse_to_unicode(&decoded) {
                        cmap = Some(parsed_cmap);
                    }
                }
            }
        }

        Ok(Self {
            base_font,
            subtype,
            first_char,
            last_char,
            widths,
            default_width: 1000.0,
            cmap,
            encoding,
        })
    }

    /// Decodes a sequence of character bytes into a Unicode string.
    pub fn decode_text(&self, bytes: &[u8]) -> String {
        if let Some(ref cmap) = self.cmap {
            // Check if 2-byte CIDFont encoding
            let is_2byte = self.subtype == "Type0"
                || bytes.len() > 1 && cmap.mappings.keys().any(|&k| k > 255);
            if is_2byte {
                let mut result = String::new();
                for chunk in bytes.chunks(2) {
                    let code = if chunk.len() == 2 {
                        ((chunk[0] as u32) << 8) | (chunk[1] as u32)
                    } else {
                        chunk[0] as u32
                    };
                    if let Some(uni) = cmap.lookup(code) {
                        result.push_str(uni);
                    } else {
                        result.push(win_ansi_to_unicode((code & 0xFF) as u8));
                    }
                }
                return result;
            }

            // 1-byte encoding with CMap lookup
            let mut result = String::new();
            for &b in bytes {
                let code = b as u32;
                if let Some(uni) = cmap.lookup(code) {
                    result.push_str(uni);
                } else {
                    result.push(win_ansi_to_unicode(b));
                }
            }
            result
        } else {
            // Standard WinAnsi / ASCII decoding
            let mut result = String::with_capacity(bytes.len());
            for &b in bytes {
                result.push(win_ansi_to_unicode(b));
            }
            result
        }
    }

    /// Returns glyph advance width in 1/1000th of text space.
    #[inline]
    pub fn glyph_width(&self, char_code: u32) -> f64 {
        self.widths
            .get(&char_code)
            .copied()
            .unwrap_or(self.default_width)
    }
}
