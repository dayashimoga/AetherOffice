//! PDF Object Serializer and Incremental Update Writer (ISO 32000-1 Clause 7.5.6).

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::{PdfObject, PdfObjectId};
use crate::xref::XrefTable;
use std::collections::BTreeMap;

/// Serializes any `PdfObject` into standard PDF byte syntax.
pub fn serialize_object(obj: &PdfObject, out: &mut Vec<u8>) {
    match obj {
        PdfObject::Null => out.extend_from_slice(b"null"),
        PdfObject::Boolean(true) => out.extend_from_slice(b"true"),
        PdfObject::Boolean(false) => out.extend_from_slice(b"false"),
        PdfObject::Integer(i) => out.extend_from_slice(i.to_string().as_bytes()),
        PdfObject::Real(f) => {
            if f.is_finite() {
                // Strip unnecessary trailing zeroes
                let s = format!("{:.4}", f);
                let trimmed = s.trim_end_matches('0').trim_end_matches('.');
                if trimmed.is_empty() || trimmed == "-" {
                    out.extend_from_slice(b"0");
                } else {
                    out.extend_from_slice(trimmed.as_bytes());
                }
            } else {
                out.extend_from_slice(b"0");
            }
        }
        PdfObject::Name(name) => {
            out.push(b'/');
            for &b in name.as_bytes() {
                if b.is_ascii_whitespace()
                    || matches!(
                        b,
                        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%' | b'#'
                    )
                {
                    out.extend_from_slice(format!("#{:02X}", b).as_bytes());
                } else {
                    out.push(b);
                }
            }
        }
        PdfObject::LiteralString(bytes) => {
            out.push(b'(');
            for &b in bytes {
                match b {
                    b'(' => out.extend_from_slice(b"\\("),
                    b')' => out.extend_from_slice(b"\\)"),
                    b'\\' => out.extend_from_slice(b"\\\\"),
                    b'\r' => out.extend_from_slice(b"\\r"),
                    b'\n' => out.extend_from_slice(b"\\n"),
                    b'\t' => out.extend_from_slice(b"\\t"),
                    other => out.push(other),
                }
            }
            out.push(b')');
        }
        PdfObject::HexString(bytes) => {
            out.push(b'<');
            for &b in bytes {
                out.extend_from_slice(format!("{:02X}", b).as_bytes());
            }
            out.push(b'>');
        }
        PdfObject::Array(arr) => {
            out.push(b'[');
            for (idx, item) in arr.iter().enumerate() {
                if idx > 0 {
                    out.push(b' ');
                }
                serialize_object(item, out);
            }
            out.push(b']');
        }
        PdfObject::Dictionary(dict) => {
            out.extend_from_slice(b"<<\n");
            for (key, val) in dict {
                out.push(b'/');
                out.extend_from_slice(key.as_bytes());
                out.push(b' ');
                serialize_object(val, out);
                out.push(b'\n');
            }
            out.extend_from_slice(b">>");
        }
        PdfObject::Stream(stream) => {
            let mut dict = stream.dict.clone();
            dict.insert(
                "Length".into(),
                PdfObject::Integer(stream.content.len() as i64),
            );
            serialize_object(&PdfObject::Dictionary(dict), out);
            out.extend_from_slice(b"\nstream\r\n");
            out.extend_from_slice(&stream.content);
            out.extend_from_slice(b"\r\nendstream");
        }
        PdfObject::Reference(id) => {
            out.extend_from_slice(format!("{} {} R", id.number, id.generation).as_bytes());
        }
    }
}

/// Builder for non-destructive incremental updates to existing PDF documents.
pub struct PdfIncrementalWriter {
    original_bytes: Vec<u8>,
    last_xref_offset: usize,
    max_object_number: u32,
    root_id: PdfObjectId,
    info_id: Option<PdfObjectId>,
    pending_objects: BTreeMap<u32, (u16, PdfObject)>,
}

impl PdfIncrementalWriter {
    /// Initializes an incremental writer from a loaded `PdfDocument`.
    pub fn from_document(doc: &PdfDocument, original_bytes: Vec<u8>) -> Result<Self, PdfError> {
        let last_xref_offset = XrefTable::find_startxref_offset(&original_bytes)?;
        let root_id = doc.xref().root_id()?;
        let info_id = doc.xref().info_id();

        let max_object_number = doc
            .xref()
            .entries
            .keys()
            .copied()
            .max()
            .unwrap_or(root_id.number);

        Ok(Self {
            original_bytes,
            last_xref_offset,
            max_object_number,
            root_id,
            info_id,
            pending_objects: BTreeMap::new(),
        })
    }

    /// Initializes directly from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PdfError> {
        let doc = PdfDocument::parse(bytes.to_vec())?;
        Self::from_document(&doc, bytes.to_vec())
    }

    /// Allocates a new unique object ID for this revision.
    pub fn allocate_object_id(&mut self) -> PdfObjectId {
        self.max_object_number = self.max_object_number.saturating_add(1);
        PdfObjectId::new(self.max_object_number, 0)
    }

    /// Sets or replaces an object in the pending incremental update.
    pub fn set_object(&mut self, id: PdfObjectId, object: PdfObject) {
        if id.number > self.max_object_number {
            self.max_object_number = id.number;
        }
        self.pending_objects
            .insert(id.number, (id.generation, object));
    }

    /// Convenience method to allocate and set a new object in one call.
    pub fn add_object(&mut self, object: PdfObject) -> PdfObjectId {
        let id = self.allocate_object_id();
        self.set_object(id, object);
        id
    }

    pub fn set_root(&mut self, root_id: PdfObjectId) {
        self.root_id = root_id;
    }

    pub fn pending_count(&self) -> usize {
        self.pending_objects.len()
    }

    /// Finalizes the incremental update, appending only new/modified objects and trailer.
    pub fn finish(self) -> Result<Vec<u8>, PdfError> {
        if self.pending_objects.is_empty() {
            return Ok(self.original_bytes);
        }

        let mut out = self.original_bytes;
        // Ensure separation from previous revision
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }

        let mut new_offsets: BTreeMap<u32, (u64, u16)> = BTreeMap::new();

        // 1. Serialize all new / modified indirect objects
        for (num, (gen, obj)) in self.pending_objects {
            let obj_start = out.len() as u64;
            new_offsets.insert(num, (obj_start, gen));

            out.extend_from_slice(format!("{} {} obj\n", num, gen).as_bytes());
            serialize_object(&obj, &mut out);
            out.extend_from_slice(b"\nendobj\n");
        }

        // 2. Append incremental XRef table
        let new_xref_offset = out.len();
        out.extend_from_slice(b"xref\n");

        // Group into contiguous subsections
        let mut subsections: Vec<Vec<u32>> = Vec::new();
        for &num in new_offsets.keys() {
            if let Some(last_group) = subsections.last_mut() {
                if let Some(&last_num) = last_group.last() {
                    if last_num + 1 == num {
                        last_group.push(num);
                        continue;
                    }
                }
            }
            subsections.push(vec![num]);
        }

        for group in subsections {
            let start_num = group[0];
            let count = group.len();
            out.extend_from_slice(format!("{} {}\n", start_num, count).as_bytes());

            for &num in &group {
                let (offset, gen) = new_offsets.get(&num).copied().unwrap_or((0, 0));
                out.extend_from_slice(format!("{:010} {:05} n \r\n", offset, gen).as_bytes());
            }
        }

        // 3. Append incremental Trailer dictionary
        out.extend_from_slice(b"trailer\n<<\n");
        out.extend_from_slice(format!("/Size {}\n", self.max_object_number + 1).as_bytes());
        out.extend_from_slice(format!("/Root {}\n", self.root_id).as_bytes());
        if let Some(info) = self.info_id {
            out.extend_from_slice(format!("/Info {}\n", info).as_bytes());
        }
        out.extend_from_slice(format!("/Prev {}\n", self.last_xref_offset).as_bytes());
        out.extend_from_slice(b">>\nstartxref\n");
        out.extend_from_slice(format!("{}\n%%EOF", new_xref_offset).as_bytes());

        Ok(out)
    }
}
