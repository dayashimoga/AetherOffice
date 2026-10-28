//! ECMA-376 Part 2 (ISO/IEC 29500-2) Lazy OPC Package Reader.
//! Efficient streaming reader capable of resolving central directories, content types,
//! and relationship graphs in < 15ms without decompressing all streams upfront.
//! Strictly conforms to GEC-v1.0 Article IV (zero panics, typed EngineError).

use crate::content_types::ContentTypes;
use crate::error::{EngineError, Result};
use crate::relationships::Relationships;
use std::collections::{BTreeMap, HashMap};
use std::io::{BufReader, Read, Seek};
use zip::ZipArchive;

/// Lazy ECMA-376 Part 2 Open Packaging Conventions (OPC) package reader.
pub struct PackageReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    /// List of all raw part names in the archive.
    part_names: Vec<String>,
    /// Case-preserving and normalized part index mapping to ZIP entry numbers.
    part_index: HashMap<String, usize>,
    /// Cached ContentTypes parsed lazily from `[Content_Types].xml`.
    content_types: Option<ContentTypes>,
    /// Cached Relationships parsed lazily by relationship part name.
    relationship_cache: BTreeMap<String, Relationships>,
}

impl<R: Read + Seek> PackageReader<R> {
    /// Opens an OPC package from any seekable reader.
    /// Fast initialization: indexes ZIP central directory without decompressing content.
    pub fn open(reader: R) -> Result<Self> {
        let mut archive = ZipArchive::new(reader).map_err(EngineError::Zip)?;
        let mut part_index = HashMap::with_capacity(archive.len());
        let mut part_names = Vec::with_capacity(archive.len());

        for i in 0..archive.len() {
            let file = archive.by_index_raw(i).map_err(EngineError::Zip)?;
            let name = file.name().to_string();
            // Store both raw name and normalized leading-slash name
            let norm_name = if name.starts_with('/') {
                name.clone()
            } else {
                format!("/{name}")
            };
            part_index.insert(norm_name, i);
            part_index.insert(name.clone(), i);
            part_names.push(name);
        }

        Ok(Self {
            archive,
            part_names,
            part_index,
            content_types: None,
            relationship_cache: BTreeMap::new(),
        })
    }

    /// Returns the total number of parts in the package.
    pub fn len(&self) -> usize {
        self.part_names.len()
    }

    /// Returns whether the package is empty.
    pub fn is_empty(&self) -> bool {
        self.part_names.is_empty()
    }

    /// Checks if a part exists in the package (supports with or without leading slash).
    pub fn contains_part(&self, part_name: &str) -> bool {
        let clean = part_name.trim();
        let norm = if clean.starts_with('/') {
            clean.to_string()
        } else {
            format!("/{clean}")
        };
        self.part_index.contains_key(clean) || self.part_index.contains_key(&norm)
    }

    /// Returns a slice of all part names in the package.
    pub fn part_names(&self) -> &[String] {
        &self.part_names
    }

    /// Reads and returns the raw decompressed byte stream of a specific part.
    pub fn read_part(&mut self, part_name: &str) -> Result<Vec<u8>> {
        let clean = part_name.trim();
        let norm = if clean.starts_with('/') {
            clean.to_string()
        } else {
            format!("/{clean}")
        };

        let index = self
            .part_index
            .get(clean)
            .or_else(|| self.part_index.get(&norm))
            .copied()
            .ok_or_else(|| EngineError::PartNotFound(clean.to_string()))?;

        let mut file = self.archive.by_index(index).map_err(EngineError::Zip)?;
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data).map_err(EngineError::Io)?;
        Ok(data)
    }

    /// Lazily reads and parses `[Content_Types].xml`.
    pub fn get_content_types(&mut self) -> Result<&ContentTypes> {
        if self.content_types.is_none() {
            let data = self.read_part("[Content_Types].xml")?;
            let parsed = ContentTypes::parse(BufReader::new(std::io::Cursor::new(data)))?;
            self.content_types = Some(parsed);
        }

        self.content_types
            .as_ref()
            .ok_or_else(|| EngineError::PartNotFound("[Content_Types].xml".to_string()))
    }

    /// Resolves the content type for a given part name.
    pub fn get_part_content_type(&mut self, part_name: &str) -> Result<String> {
        let ct = self.get_content_types()?;
        ct.get_content_type(part_name)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                EngineError::InvalidContentType(format!(
                    "No content type defined for part '{part_name}'"
                ))
            })
    }

    /// Lazily reads and parses the relationship set for a specific source part or root package.
    pub fn get_relationships(&mut self, source_part: Option<&str>) -> Result<&Relationships> {
        let rels_part = Relationships::rels_part_name_for(source_part);

        if !self.relationship_cache.contains_key(&rels_part) {
            let rels = if self.contains_part(&rels_part) {
                let data = self.read_part(&rels_part)?;
                Relationships::parse(BufReader::new(std::io::Cursor::new(data)))?
            } else {
                Relationships::new()
            };
            self.relationship_cache.insert(rels_part.clone(), rels);
        }

        self.relationship_cache
            .get(&rels_part)
            .ok_or(EngineError::RelationshipNotFound(rels_part))
    }

    /// Resolves the target package part path for a given relationship ID.
    pub fn resolve_relationship_target(
        &mut self,
        source_part: Option<&str>,
        rel_id: &str,
    ) -> Result<String> {
        let rels = self.get_relationships(source_part)?;
        rels.resolve_target(rel_id, source_part)
    }
}
