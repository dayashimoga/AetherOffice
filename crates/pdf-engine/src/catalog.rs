//! PDF Document Catalog and Hierarchical Page Tree Resolver (ISO 32000-1 Clause 7.7).

use crate::error::PdfError;
use crate::object_stream::ObjectStream;
use crate::page::PdfPage;
use crate::parser::PdfParser;
use crate::types::{LinearizationDict, PdfObject, PdfObjectId, PdfRect};
use crate::xref::{XrefEntry, XrefTable};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

/// Complete parsed PDF Document structure holding object tables and the resolved page tree.
pub struct PdfDocument {
    raw_data: Vec<u8>,
    version: (u8, u8),
    linearization: Option<LinearizationDict>,
    xref: XrefTable,
    catalog: BTreeMap<String, PdfObject>,
    pages: Vec<PdfPage>,
    cached_objects: RefCell<HashMap<PdfObjectId, PdfObject>>,
    cached_obj_streams: RefCell<HashMap<u32, ObjectStream>>,
}

#[derive(Clone, Default)]
struct InheritedPageAttributes {
    media_box: Option<PdfRect>,
    crop_box: Option<PdfRect>,
    rotate: Option<i32>,
    resources: Option<BTreeMap<String, PdfObject>>,
}

impl PdfDocument {
    /// Loads and verifies a PDF file from an in-memory byte buffer.
    pub fn parse(data: Vec<u8>) -> Result<Self, PdfError> {
        let version = Self::validate_header(&data)?;
        let linearization = Self::detect_linearization(&data);
        let xref = XrefTable::parse_from_data(&data)?;

        let mut doc = Self {
            raw_data: data,
            version,
            linearization,
            xref,
            catalog: BTreeMap::new(),
            pages: Vec::new(),
            cached_objects: RefCell::new(HashMap::new()),
            cached_obj_streams: RefCell::new(HashMap::new()),
        };

        // Resolve Document Catalog (/Root)
        let root_id = doc.xref.root_id()?;
        let root_obj = doc.resolve_object(root_id)?;
        let catalog_dict = match root_obj {
            PdfObject::Dictionary(d) => d,
            other => {
                return Err(PdfError::InvalidType {
                    expected: "dictionary for /Root",
                    found: other.type_name(),
                })
            }
        };
        doc.catalog = catalog_dict;

        // Traverse the hierarchical page tree (/Pages)
        let pages_ref = doc
            .catalog
            .get("Pages")
            .and_then(|v| v.as_reference())
            .ok_or_else(|| {
                PdfError::MalformedPageTree("Document Catalog missing /Pages reference".into())
            })?;

        let mut resolved_pages = Vec::new();
        let default_inherited = InheritedPageAttributes::default();
        doc.traverse_pages_node(pages_ref, &default_inherited, &mut resolved_pages)?;
        doc.pages = resolved_pages;

        Ok(doc)
    }

    #[inline]
    pub fn version(&self) -> (u8, u8) {
        self.version
    }

    #[inline]
    pub fn is_linearized(&self) -> bool {
        self.linearization.is_some()
    }

    #[inline]
    pub fn linearization_info(&self) -> Option<&LinearizationDict> {
        self.linearization.as_ref()
    }

    #[inline]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    #[inline]
    pub fn pages(&self) -> &[PdfPage] {
        &self.pages
    }

    pub fn page(&self, index: usize) -> Result<&PdfPage, PdfError> {
        self.pages.get(index).ok_or(PdfError::PageOutOfRange {
            index,
            count: self.pages.len(),
        })
    }

    #[inline]
    pub fn catalog(&self) -> &BTreeMap<String, PdfObject> {
        &self.catalog
    }

    #[inline]
    pub fn xref(&self) -> &XrefTable {
        &self.xref
    }

    #[inline]
    pub fn trailer(&self) -> &BTreeMap<String, PdfObject> {
        self.xref.trailer()
    }

    /// Validates the `%PDF-M.m` magic byte header.
    fn validate_header(data: &[u8]) -> Result<(u8, u8), PdfError> {
        if data.len() < 8 {
            return Err(PdfError::InvalidHeader("PDF data too short".into()));
        }

        let magic = b"%PDF-";
        let header_slice = &data[..1024.min(data.len())];
        let offset = header_slice
            .windows(magic.len())
            .position(|w| w == magic)
            .ok_or_else(|| {
                PdfError::InvalidHeader("Missing '%PDF-' signature in file header".into())
            })?;

        let version_str = match std::str::from_utf8(&header_slice[offset + magic.len()..offset + 8])
        {
            Ok(s) => s,
            Err(_) => return Err(PdfError::InvalidHeader("Non-UTF-8 version string".into())),
        };

        let mut parts = version_str.split('.');
        let major = parts
            .next()
            .and_then(|s| s.chars().next()?.to_digit(10))
            .unwrap_or(1) as u8;
        let minor = parts
            .next()
            .and_then(|s| s.chars().next()?.to_digit(10))
            .unwrap_or(7) as u8;

        Ok((major, minor))
    }

    /// Fast scan for the `/Linearized` dictionary in the first 1024 bytes.
    fn detect_linearization(data: &[u8]) -> Option<LinearizationDict> {
        let scan_limit = 2048.min(data.len());
        let slice = &data[..scan_limit];
        let marker = b"/Linearized";

        if slice.windows(marker.len()).any(|w| w == marker) {
            let mut parser = PdfParser::new(crate::lexer::PdfLexer::new(slice));
            // Find first indirect object
            while let Ok((_, obj)) = parser.parse_indirect_object() {
                if let PdfObject::Dictionary(ref dict) = obj {
                    if dict.contains_key("Linearized") {
                        if let Ok(lin) = LinearizationDict::from_dict(dict) {
                            return Some(lin);
                        }
                    }
                }
            }
        }
        None
    }

    /// Resolves an indirect object reference `num gen R` to its `PdfObject` value.
    pub fn resolve_object(&self, id: PdfObjectId) -> Result<PdfObject, PdfError> {
        if let Some(cached) = self.cached_objects.borrow().get(&id) {
            return Ok(cached.clone());
        }

        let entry = self
            .xref
            .get_entry(id.number)
            .ok_or(PdfError::ObjectNotFound {
                number: id.number,
                generation: id.generation,
            })?;

        let resolved_obj = match entry {
            XrefEntry::Uncompressed { byte_offset, .. } => {
                let mut parser = PdfParser::with_pos(&self.raw_data, *byte_offset as usize);
                let (parsed_id, obj) = parser.parse_indirect_object()?;
                if parsed_id.number != id.number {
                    return Err(PdfError::InvalidXref(format!(
                        "Object mismatch at offset {}: expected {}, parsed {}",
                        byte_offset, id.number, parsed_id.number
                    )));
                }
                obj
            }
            XrefEntry::Compressed {
                stream_obj,
                index_in_stream,
            } => {
                let mut obj_streams = self.cached_obj_streams.borrow_mut();
                if !obj_streams.contains_key(stream_obj) {
                    let stream_ref_id = PdfObjectId::new(*stream_obj, 0);
                    let stream_obj_val = self.resolve_object(stream_ref_id)?;
                    let pdf_stream = match stream_obj_val {
                        PdfObject::Stream(s) => s,
                        other => {
                            return Err(PdfError::InvalidType {
                                expected: "stream for ObjStm",
                                found: other.type_name(),
                            })
                        }
                    };
                    let parsed_stream = ObjectStream::parse_from_stream(&pdf_stream)?;
                    obj_streams.insert(*stream_obj, parsed_stream);
                }

                let stream = obj_streams
                    .get(stream_obj)
                    .ok_or(PdfError::ObjectNotFound {
                        number: *stream_obj,
                        generation: 0,
                    })?;

                let (_, obj) = stream
                    .get_by_index(*index_in_stream as usize)
                    .ok_or_else(|| PdfError::SyntaxError {
                        offset: 0,
                        message: format!(
                            "Object index {} out of range in ObjStm {}",
                            index_in_stream, stream_obj
                        ),
                    })?;

                obj.clone()
            }
            XrefEntry::Free { .. } => PdfObject::Null,
        };

        self.cached_objects
            .borrow_mut()
            .insert(id, resolved_obj.clone());
        Ok(resolved_obj)
    }

    /// Recursively traverses a `/Pages` node or `/Page` leaf, resolving inherited attributes.
    fn traverse_pages_node(
        &self,
        node_id: PdfObjectId,
        inherited: &InheritedPageAttributes,
        resolved_pages: &mut Vec<PdfPage>,
    ) -> Result<(), PdfError> {
        let node_obj = self.resolve_object(node_id)?;
        let dict = match node_obj {
            PdfObject::Dictionary(d) => d,
            other => {
                return Err(PdfError::InvalidType {
                    expected: "dictionary for page tree node",
                    found: other.type_name(),
                })
            }
        };

        let node_type = dict
            .get("Type")
            .and_then(|v| v.as_name())
            .ok_or_else(|| PdfError::MalformedPageTree("Page tree node missing /Type".into()))?;

        // Update inherited attributes
        let mut cur_inherited = inherited.clone();
        if let Some(PdfObject::Array(arr)) = dict.get("MediaBox") {
            if let Ok(rect) = PdfRect::from_array(arr) {
                cur_inherited.media_box = Some(rect);
            }
        }
        if let Some(PdfObject::Array(arr)) = dict.get("CropBox") {
            if let Ok(rect) = PdfRect::from_array(arr) {
                cur_inherited.crop_box = Some(rect);
            }
        }
        if let Some(rot) = dict.get("Rotate").and_then(|v| v.as_i64()) {
            cur_inherited.rotate = Some(rot as i32);
        }
        if let Some(PdfObject::Dictionary(res)) = dict.get("Resources") {
            cur_inherited.resources = Some(res.clone());
        }

        match node_type {
            "Pages" => {
                let kids = dict.get("Kids").and_then(|v| v.as_array()).ok_or_else(|| {
                    PdfError::MalformedPageTree("/Pages dictionary missing /Kids array".into())
                })?;

                for kid_obj in kids {
                    let kid_id = kid_obj.as_reference().ok_or_else(|| {
                        PdfError::MalformedPageTree("Item in /Kids is not an indirect ref".into())
                    })?;
                    self.traverse_pages_node(kid_id, &cur_inherited, resolved_pages)?;
                }
                Ok(())
            }
            "Page" => {
                let media_box = cur_inherited.media_box.unwrap_or_else(PdfRect::letter);
                let crop_box = cur_inherited.crop_box.unwrap_or(media_box);
                let rotate = cur_inherited.rotate.unwrap_or(0);

                let mut resources = cur_inherited.resources.unwrap_or_default();
                if let Some(res_obj) = dict.get("Resources") {
                    let res_dict = match res_obj {
                        PdfObject::Dictionary(d) => d.clone(),
                        PdfObject::Reference(ref_id) => {
                            if let Ok(PdfObject::Dictionary(d)) = self.resolve_object(*ref_id) {
                                d
                            } else {
                                BTreeMap::new()
                            }
                        }
                        _ => BTreeMap::new(),
                    };
                    resources.extend(res_dict);
                }

                // Collect content streams (/Contents may be an indirect ref or an array of refs)
                let mut contents = Vec::new();
                if let Some(c_obj) = dict.get("Contents") {
                    match c_obj {
                        PdfObject::Reference(ref_id) => contents.push(*ref_id),
                        PdfObject::Array(arr) => {
                            for item in arr {
                                if let Some(ref_id) = item.as_reference() {
                                    contents.push(ref_id);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Collect annotations (/Annots)
                let mut annotations = Vec::new();
                if let Some(a_obj) = dict.get("Annots") {
                    let annot_arr = match a_obj {
                        PdfObject::Array(arr) => arr.clone(),
                        PdfObject::Reference(ref_id) => {
                            if let Ok(PdfObject::Array(arr)) = self.resolve_object(*ref_id) {
                                arr
                            } else {
                                Vec::new()
                            }
                        }
                        _ => Vec::new(),
                    };
                    for item in annot_arr {
                        if let Some(ref_id) = item.as_reference() {
                            annotations.push(ref_id);
                        }
                    }
                }

                let page_idx = resolved_pages.len();
                let page = PdfPage {
                    index: page_idx,
                    page_number: page_idx + 1,
                    object_id: node_id,
                    media_box,
                    crop_box,
                    rotate,
                    contents,
                    resources,
                    annotations,
                };

                resolved_pages.push(page);
                Ok(())
            }
            other => Err(PdfError::MalformedPageTree(format!(
                "Unexpected /Type '/{}' in page tree hierarchy",
                other
            ))),
        }
    }
}
