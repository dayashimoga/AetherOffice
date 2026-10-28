//! ECMA-376 Part 2 Unparsed Part Store & Markup Compatibility Engine.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity)
//! and Article IV (zero panics, typed EngineError).

use crate::relationships::Relationships;
use std::collections::BTreeMap;

/// Holds an unmodeled OPC part with its raw byte payload, optional content type, and relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsedPart {
    /// Normalized part name (e.g. `/word/vbaProject.bin`)
    pub name: String,
    /// Raw unmodeled bytes (preserved bit-for-bit without decompression degradation)
    pub content: Vec<u8>,
    /// Content type if registered in `[Content_Types].xml`
    pub content_type: Option<String>,
    /// Part-level relationships associated with this unmodeled part (e.g. `word/_rels/vbaProject.bin.rels`)
    pub relationships: Option<Relationships>,
}

impl UnparsedPart {
    /// Creates a new unparsed part container.
    pub fn new(
        name: &str,
        content: Vec<u8>,
        content_type: Option<String>,
        relationships: Option<Relationships>,
    ) -> Self {
        let normalized_name = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{name}")
        };
        Self {
            name: normalized_name,
            content,
            content_type,
            relationships,
        }
    }
}

/// In-memory repository retaining all unmodeled ZIP parts, macros, signatures, and custom streams.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnparsedPartStore {
    /// Internal map keyed by normalized part name (`/path/to/part`)
    parts: BTreeMap<String, UnparsedPart>,
}

impl UnparsedPartStore {
    /// Creates an empty `UnparsedPartStore`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of unmodeled parts stored.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// Checks if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Normalizes part names for keying.
    fn normalize_key(name: &str) -> String {
        let clean = name.trim();
        if clean.starts_with('/') {
            clean.to_string()
        } else {
            format!("/{clean}")
        }
    }

    /// Stores an unparsed part into the store.
    pub fn insert(&mut self, part: UnparsedPart) {
        let key = Self::normalize_key(&part.name);
        self.parts.insert(key, part);
    }

    /// Checks if a part exists in the unparsed store.
    pub fn contains(&self, name: &str) -> bool {
        let key = Self::normalize_key(name);
        self.parts.contains_key(&key)
    }

    /// Retrieves an unparsed part by name.
    pub fn get(&self, name: &str) -> Option<&UnparsedPart> {
        let key = Self::normalize_key(name);
        self.parts.get(&key)
    }

    /// Removes and returns an unparsed part from the store.
    pub fn remove(&mut self, name: &str) -> Option<UnparsedPart> {
        let key = Self::normalize_key(name);
        self.parts.remove(&key)
    }

    /// Returns an iterator over all stored unparsed parts sorted deterministically by name.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &UnparsedPart)> {
        self.parts.iter()
    }

    /// Returns an iterator of all part names in the store.
    pub fn part_names(&self) -> impl Iterator<Item = &String> {
        self.parts.keys()
    }

    /// Total byte size of all unmodeled payloads stored.
    pub fn total_bytes(&self) -> usize {
        self.parts.values().map(|p| p.content.len()).sum()
    }
}

/// Buffer retaining unmodeled XML elements, unknown attributes, and Markup Compatibility blocks.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawXmlNodeBuffer {
    /// Preserved raw XML markup slice
    pub raw_xml: String,
    /// Preserved `mc:Ignorable` namespace prefix list
    pub ignorable_prefixes: Vec<String>,
    /// Preserved unmodeled XML attributes (`name="value"`)
    pub unmodeled_attributes: BTreeMap<String, String>,
}

impl RawXmlNodeBuffer {
    /// Creates a new `RawXmlNodeBuffer`.
    pub fn new(raw_xml: &str) -> Self {
        Self {
            raw_xml: raw_xml.to_string(),
            ignorable_prefixes: Vec::new(),
            unmodeled_attributes: BTreeMap::new(),
        }
    }

    /// Adds an unmodeled XML attribute to be preserved on round-trip serialization.
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.unmodeled_attributes
            .insert(key.to_string(), value.to_string());
    }

    /// Registers an `mc:Ignorable` prefix.
    pub fn add_ignorable_prefix(&mut self, prefix: &str) {
        if !self.ignorable_prefixes.iter().any(|p| p == prefix) {
            self.ignorable_prefixes.push(prefix.to_string());
        }
    }

    /// Returns whether this node buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.raw_xml.is_empty()
            && self.ignorable_prefixes.is_empty()
            && self.unmodeled_attributes.is_empty()
    }
}
