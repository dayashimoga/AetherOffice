//! Fundamental PDF data types defined in ISO 32000-1 and ISO 32000-2.

use crate::error::PdfError;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;

/// Indirect object identifier composed of an object number and a generation number.
/// In PDF syntax: `12 0 R` or `12 0 obj ... endobj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PdfObjectId {
    pub number: u32,
    pub generation: u16,
}

impl PdfObjectId {
    #[inline]
    pub const fn new(number: u32, generation: u16) -> Self {
        Self { number, generation }
    }
}

impl fmt::Display for PdfObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} R", self.number, self.generation)
    }
}

/// PDF Rectangle `[llx, lly, urx, ury]` in default user space units (1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PdfRect {
    pub llx: f64,
    pub lly: f64,
    pub urx: f64,
    pub ury: f64,
}

impl PdfRect {
    pub const fn new(llx: f64, lly: f64, urx: f64, ury: f64) -> Self {
        Self { llx, lly, urx, ury }
    }

    /// Standard Letter page size (8.5 x 11 inches = 612 x 792 points).
    pub const fn letter() -> Self {
        Self {
            llx: 0.0,
            lly: 0.0,
            urx: 612.0,
            ury: 792.0,
        }
    }

    /// Standard A4 page size (210 x 297 mm = 595.276 x 841.890 points).
    pub const fn a4() -> Self {
        Self {
            llx: 0.0,
            lly: 0.0,
            urx: 595.276,
            ury: 841.890,
        }
    }

    #[inline]
    pub fn width(&self) -> f64 {
        (self.urx - self.llx).abs()
    }

    #[inline]
    pub fn height(&self) -> f64 {
        (self.ury - self.lly).abs()
    }

    /// Parses a 4-element array into a `PdfRect`.
    pub fn from_array(arr: &[PdfObject]) -> Result<Self, PdfError> {
        if arr.len() < 4 {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: format!(
                    "PdfRect array requires at least 4 numbers, got {}",
                    arr.len()
                ),
            });
        }

        let llx = arr[0].as_f64().ok_or_else(|| PdfError::InvalidType {
            expected: "numeric",
            found: arr[0].type_name(),
        })?;
        let lly = arr[1].as_f64().ok_or_else(|| PdfError::InvalidType {
            expected: "numeric",
            found: arr[1].type_name(),
        })?;
        let urx = arr[2].as_f64().ok_or_else(|| PdfError::InvalidType {
            expected: "numeric",
            found: arr[2].type_name(),
        })?;
        let ury = arr[3].as_f64().ok_or_else(|| PdfError::InvalidType {
            expected: "numeric",
            found: arr[3].type_name(),
        })?;

        Ok(Self::new(llx, lly, urx, ury))
    }

    pub fn to_array(&self) -> PdfObject {
        PdfObject::Array(vec![
            PdfObject::Real(self.llx),
            PdfObject::Real(self.lly),
            PdfObject::Real(self.urx),
            PdfObject::Real(self.ury),
        ])
    }
}

/// PDF Stream object consisting of a dictionary descriptor and byte content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfStream {
    pub dict: BTreeMap<String, PdfObject>,
    pub content: Vec<u8>,
}

impl PdfStream {
    pub fn new(dict: BTreeMap<String, PdfObject>, content: Vec<u8>) -> Self {
        Self { dict, content }
    }

    pub fn length(&self) -> usize {
        self.content.len()
    }

    pub fn filter(&self) -> Option<&str> {
        self.dict.get("Filter").and_then(|obj| obj.as_name())
    }

    /// Decodes stream contents based on the `/Filter` dictionary entry.
    /// Supports `/FlateDecode` (Zlib/Deflate) or uncompressed raw streams.
    pub fn decode(&self) -> Result<Vec<u8>, PdfError> {
        match self.filter() {
            None => Ok(self.content.clone()),
            Some("FlateDecode") => {
                let mut decoder = ZlibDecoder::new(&self.content[..]);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(|e| PdfError::DecompressionError(e.to_string()))?;
                Ok(decompressed)
            }
            Some(unsupported) => Err(PdfError::DecompressionError(format!(
                "Unsupported stream filter: /{}",
                unsupported
            ))),
        }
    }
}

/// Linearization dictionary parameters (ISO 32000 Annex F).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearizationDict {
    pub version: f64,
    pub file_length: u64,
    pub primary_hint_stream: (u64, u64),
    pub first_page_object: u32,
    pub first_page_end_offset: u64,
    pub page_count: u32,
    pub first_page_xref_offset: u64,
}

impl LinearizationDict {
    pub fn from_dict(dict: &BTreeMap<String, PdfObject>) -> Result<Self, PdfError> {
        let version = dict
            .get("Linearized")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let file_length = dict
            .get("L")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PdfError::LinearizationError("Missing /L (file length)".into()))?
            as u64;

        let primary_hint_stream = if let Some(PdfObject::Array(arr)) = dict.get("H") {
            if arr.len() >= 2 {
                let off = arr[0].as_i64().unwrap_or(0) as u64;
                let len = arr[1].as_i64().unwrap_or(0) as u64;
                (off, len)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        let first_page_object = dict.get("O").and_then(|v| v.as_i64()).unwrap_or(0) as u32;

        let first_page_end_offset = dict.get("E").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        let page_count = dict.get("N").and_then(|v| v.as_i64()).unwrap_or(1) as u32;

        let first_page_xref_offset = dict.get("T").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        Ok(Self {
            version,
            file_length,
            primary_hint_stream,
            first_page_object,
            first_page_end_offset,
            page_count,
            first_page_xref_offset,
        })
    }
}

/// Core PDF object model corresponding to ISO 32000-1 Clause 7.3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Name(String),
    LiteralString(Vec<u8>),
    HexString(Vec<u8>),
    Array(Vec<PdfObject>),
    Dictionary(BTreeMap<String, PdfObject>),
    Stream(PdfStream),
    Reference(PdfObjectId),
}

impl PdfObject {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, PdfObject::Null)
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PdfObject::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PdfObject::Real(f) => Some(*f),
            PdfObject::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    #[inline]
    pub fn as_name(&self) -> Option<&str> {
        match self {
            PdfObject::Name(s) => Some(s.as_str()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            PdfObject::LiteralString(b) | PdfObject::HexString(b) => Some(b.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn to_string_lossy(&self) -> Option<String> {
        match self {
            PdfObject::LiteralString(b) | PdfObject::HexString(b) => {
                Some(String::from_utf8_lossy(b).into_owned())
            }
            PdfObject::Name(s) => Some(s.clone()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&[PdfObject]> {
        match self {
            PdfObject::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<PdfObject>> {
        match self {
            PdfObject::Array(arr) => Some(arr),
            _ => None,
        }
    }

    #[inline]
    pub fn as_dict(&self) -> Option<&BTreeMap<String, PdfObject>> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            PdfObject::Stream(s) => Some(&s.dict),
            _ => None,
        }
    }

    #[inline]
    pub fn as_dict_mut(&mut self) -> Option<&mut BTreeMap<String, PdfObject>> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            PdfObject::Stream(s) => Some(&mut s.dict),
            _ => None,
        }
    }

    #[inline]
    pub fn as_stream(&self) -> Option<&PdfStream> {
        match self {
            PdfObject::Stream(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn as_reference(&self) -> Option<PdfObjectId> {
        match self {
            PdfObject::Reference(id) => Some(*id),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            PdfObject::Null => "null",
            PdfObject::Boolean(_) => "boolean",
            PdfObject::Integer(_) => "integer",
            PdfObject::Real(_) => "real",
            PdfObject::Name(_) => "name",
            PdfObject::LiteralString(_) => "literal_string",
            PdfObject::HexString(_) => "hex_string",
            PdfObject::Array(_) => "array",
            PdfObject::Dictionary(_) => "dictionary",
            PdfObject::Stream(_) => "stream",
            PdfObject::Reference(_) => "reference",
        }
    }
}
