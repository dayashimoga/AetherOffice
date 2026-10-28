//! ECMA-376 Part 2 (ISO/IEC 29500-2) Content Types Engine.
//! Streaming parser and serializer for `[Content_Types].xml`.
//! Strictly conforms to GEC-v1.0 Article IV (zero panics, typed EngineError).

use crate::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::collections::BTreeMap;
use std::io::{BufRead, Cursor};

/// ECMA-376 Part 2 Content Types XML Namespace
pub const CONTENT_TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";

/// Default MIME types for common OpenXML formats
pub const RELATIONSHIPS_TYPE: &str = "application/vnd.openxmlformats-package.relationships+xml";
pub const XML_DEFAULT_TYPE: &str = "application/xml";

/// In-memory representation of an ECMA-376 `[Content_Types].xml` part.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentTypes {
    /// Mapping of file extension (lowercase, without leading dot) to content type.
    pub defaults: BTreeMap<String, String>,
    /// Mapping of normalized part name (starting with `/`) to content type.
    pub overrides: BTreeMap<String, String>,
}

impl ContentTypes {
    /// Creates a new empty `ContentTypes` repository.
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalizes a part name so it always starts with a leading `/`.
    pub fn normalize_part_name(part_name: &str) -> String {
        let trimmed = part_name.trim();
        if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        }
    }

    /// Normalizes an extension (lowercased, without leading dot).
    pub fn normalize_extension(ext: &str) -> String {
        let trimmed = ext.trim().trim_start_matches('.');
        trimmed.to_lowercase()
    }

    /// Adds a default extension mapping.
    pub fn add_default(&mut self, extension: &str, content_type: &str) {
        let norm_ext = Self::normalize_extension(extension);
        self.defaults
            .insert(norm_ext, content_type.trim().to_string());
    }

    /// Adds an override part mapping.
    pub fn add_override(&mut self, part_name: &str, content_type: &str) {
        let norm_part = Self::normalize_part_name(part_name);
        self.overrides
            .insert(norm_part, content_type.trim().to_string());
    }

    /// Resolves the content type for a given part name.
    /// In accordance with ECMA-376 Part 2 Section 9.1:
    /// 1. Checks `<Override>` elements (exact match on normalized part name).
    /// 2. If no override matches, extracts the file extension and checks `<Default>`.
    /// 3. If neither matches, returns `None`.
    pub fn get_content_type(&self, part_name: &str) -> Option<&str> {
        let norm_part = Self::normalize_part_name(part_name);
        if let Some(ct) = self.overrides.get(&norm_part) {
            return Some(ct.as_str());
        }

        // Extract extension from part name
        if let Some(dot_idx) = norm_part.rfind('.') {
            let ext = &norm_part[dot_idx + 1..];
            let norm_ext = ext.to_lowercase();
            if let Some(ct) = self.defaults.get(&norm_ext) {
                return Some(ct.as_str());
            }
        }

        None
    }

    /// Parses an ECMA-376 `[Content_Types].xml` stream using `quick-xml`.
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut content_types = ContentTypes::new();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"Default" {
                        let mut extension = None;
                        let mut content_type = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = attr.key.local_name();
                            let value = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;

                            if key.as_ref() == b"Extension" {
                                extension = Some(value.to_string());
                            } else if key.as_ref() == b"ContentType" {
                                content_type = Some(value.to_string());
                            }
                        }

                        if let (Some(ext), Some(ct)) = (extension, content_type) {
                            content_types.add_default(&ext, &ct);
                        }
                    } else if local_name.as_ref() == b"Override" {
                        let mut part_name = None;
                        let mut content_type = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = attr.key.local_name();
                            let value = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;

                            if key.as_ref() == b"PartName" {
                                part_name = Some(value.to_string());
                            } else if key.as_ref() == b"ContentType" {
                                content_type = Some(value.to_string());
                            }
                        }

                        if let (Some(pn), Some(ct)) = (part_name, content_type) {
                            content_types.add_override(&pn, &ct);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => return Err(EngineError::Xml(err)),
                _ => {}
            }
            buf.clear();
        }

        Ok(content_types)
    }

    /// Serializes the content types to a deterministic XML buffer.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(EngineError::Io)?;

        // <Types xmlns="...">
        let mut types_elem = BytesStart::new("Types");
        types_elem.push_attribute(("xmlns", CONTENT_TYPES_NS));
        writer
            .write_event(Event::Start(types_elem))
            .map_err(EngineError::Io)?;

        // Output defaults sorted by extension
        for (ext, ct) in &self.defaults {
            let mut def_elem = BytesStart::new("Default");
            def_elem.push_attribute(("Extension", ext.as_str()));
            def_elem.push_attribute(("ContentType", ct.as_str()));
            writer
                .write_event(Event::Empty(def_elem))
                .map_err(EngineError::Io)?;
        }

        // Output overrides sorted by part name
        for (part, ct) in &self.overrides {
            let mut ov_elem = BytesStart::new("Override");
            ov_elem.push_attribute(("PartName", part.as_str()));
            ov_elem.push_attribute(("ContentType", ct.as_str()));
            writer
                .write_event(Event::Empty(ov_elem))
                .map_err(EngineError::Io)?;
        }

        // </Types>
        writer
            .write_event(Event::End(BytesEnd::new("Types")))
            .map_err(EngineError::Io)?;

        Ok(writer.into_inner().into_inner())
    }
}
