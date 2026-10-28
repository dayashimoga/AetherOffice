//! ECMA-376 Part 2 (ISO/IEC 29500-2) Relationships Engine.
//! Streaming parser, serializer, and hierarchical URI graph resolver for `.rels` parts.
//! Strictly conforms to GEC-v1.0 Article IV (zero panics, typed EngineError).

use crate::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::collections::BTreeMap;
use std::io::{BufRead, Cursor};

/// ECMA-376 Part 2 Relationships XML Namespace
pub const RELATIONSHIPS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

/// Target mode for an OPC relationship (Internal or External).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetMode {
    #[default]
    Internal,
    External,
}

impl TargetMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "Internal",
            Self::External => "External",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        if s.eq_ignore_ascii_case("External") {
            Self::External
        } else {
            Self::Internal
        }
    }
}

/// Represents an individual ECMA-376 relationship entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub id: String,
    pub rel_type: String,
    pub target: String,
    pub target_mode: TargetMode,
}

/// In-memory collection of relationships for a package root or specific part.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Relationships {
    pub relationships: BTreeMap<String, Relationship>,
}

impl Relationships {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    pub fn add(&mut self, id: &str, rel_type: &str, target: &str, target_mode: TargetMode) {
        self.relationships.insert(
            id.to_string(),
            Relationship {
                id: id.to_string(),
                rel_type: rel_type.to_string(),
                target: target.to_string(),
                target_mode,
            },
        );
    }

    pub fn get(&self, id: &str) -> Option<&Relationship> {
        self.relationships.get(id)
    }

    pub fn find_by_type(&self, rel_type: &str) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| r.rel_type == rel_type)
            .collect()
    }

    /// Resolves the relationship part name for a given source part.
    /// - Root package (`None`) -> `_rels/.rels`
    /// - Specific part (`Some("word/document.xml")`) -> `word/_rels/document.xml.rels`
    pub fn rels_part_name_for(source_part: Option<&str>) -> String {
        match source_part {
            None | Some("") | Some("/") => "_rels/.rels".to_string(),
            Some(part) => {
                let clean = part.trim_start_matches('/');
                if let Some(slash_idx) = clean.rfind('/') {
                    let dir = &clean[..slash_idx];
                    let file = &clean[slash_idx + 1..];
                    format!("{dir}/_rels/{file}.rels")
                } else {
                    format!("_rels/{clean}.rels")
                }
            }
        }
    }

    /// Resolves the absolute package target path for a given relationship.
    /// Handles relative path resolution (including `../` traversal) and external targets.
    pub fn resolve_target(&self, id: &str, source_part: Option<&str>) -> Result<String> {
        let rel = self.get(id).ok_or_else(|| {
            EngineError::RelationshipNotFound(format!(
                "Relationship '{id}' not found in relationship set"
            ))
        })?;

        if rel.target_mode == TargetMode::External {
            return Ok(rel.target.clone());
        }

        let target = &rel.target;
        if target.starts_with('/') {
            return Ok(target.trim_start_matches('/').to_string());
        }

        // Relative target resolution based on source part directory
        let base_dir = match source_part {
            None | Some("") | Some("/") => "",
            Some(part) => {
                let clean = part.trim_start_matches('/');
                clean.rfind('/').map(|idx| &clean[..idx]).unwrap_or("")
            }
        };

        if base_dir.is_empty() {
            return Ok(normalize_relative_path(target));
        }

        let combined = format!("{base_dir}/{target}");
        Ok(normalize_relative_path(&combined))
    }

    /// Parses a `.rels` XML stream using `quick-xml`.
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut rels = Relationships::new();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    if e.local_name().as_ref() == b"Relationship" {
                        let mut id = None;
                        let mut rel_type = None;
                        let mut target = None;
                        let mut target_mode = TargetMode::Internal;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = attr.key.local_name();
                            let value = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;

                            if key.as_ref() == b"Id" {
                                id = Some(value.to_string());
                            } else if key.as_ref() == b"Type" {
                                rel_type = Some(value.to_string());
                            } else if key.as_ref() == b"Target" {
                                target = Some(value.to_string());
                            } else if key.as_ref() == b"TargetMode" {
                                target_mode = TargetMode::from_str_opt(&value);
                            }
                        }

                        if let (Some(id_val), Some(type_val), Some(target_val)) =
                            (id, rel_type, target)
                        {
                            rels.add(&id_val, &type_val, &target_val, target_mode);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => return Err(EngineError::Xml(err)),
                _ => {}
            }
            buf.clear();
        }

        Ok(rels)
    }

    /// Serializes the relationship collection to deterministic XML.
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

        // <Relationships xmlns="...">
        let mut rels_elem = BytesStart::new("Relationships");
        rels_elem.push_attribute(("xmlns", RELATIONSHIPS_NS));
        writer
            .write_event(Event::Start(rels_elem))
            .map_err(EngineError::Io)?;

        // Output sorted deterministically by Id
        for rel in self.relationships.values() {
            let mut elem = BytesStart::new("Relationship");
            elem.push_attribute(("Id", rel.id.as_str()));
            elem.push_attribute(("Type", rel.rel_type.as_str()));
            elem.push_attribute(("Target", rel.target.as_str()));
            if rel.target_mode == TargetMode::External {
                elem.push_attribute(("TargetMode", "External"));
            }
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }

        // </Relationships>
        writer
            .write_event(Event::End(BytesEnd::new("Relationships")))
            .map_err(EngineError::Io)?;

        Ok(writer.into_inner().into_inner())
    }
}

/// Normalizes path segments by resolving `.` and `..` without allocations where possible.
fn normalize_relative_path(path: &str) -> String {
    let mut segments = Vec::new();
    for seg in path.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        } else if seg == ".." {
            segments.pop();
        } else {
            segments.push(seg);
        }
    }
    segments.join("/")
}
