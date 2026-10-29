//! OpenOfficeRS Word Engine - Word Package Integration.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity),
//! Article III (Startup & Performance SLAs), and Article IV (zero panics, typed EngineError, #[forbid(unsafe_code)]).
//! Integrates `ooxml-opc` `PackageReader`/`PackageWriter` with WordprocessingML DOM and sub-documents.

use crate::model::Document;
use crate::piece_tree::PieceTree;
use crate::serializer::{
    serialize_comments, serialize_document, serialize_endnotes, serialize_footer,
    serialize_footnotes, serialize_header,
};
use crate::sub_documents::{
    CommentsDocument, EndnotesDocument, FooterDocument, FootnotesDocument, HeaderDocument,
};
use ooxml_opc::content_types::ContentTypes;
use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::package::{PackageReader, PackageWriter};
use ooxml_opc::relationships::Relationships;
use ooxml_opc::unparsed::UnparsedPartStore;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

/// ECMA-376 WordprocessingML Content Types
pub const DOCUMENT_MAIN_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
pub const HEADER_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
pub const FOOTER_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
pub const FOOTNOTES_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml";
pub const ENDNOTES_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml";
pub const COMMENTS_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";

/// ECMA-376 Relationship Types
pub const DOCUMENT_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";

/// Represents an opened or newly created WordprocessingML `.docx` package.
#[derive(Debug, Clone)]
pub struct WordPackage {
    pub document: Document,
    pub headers: Vec<(String, HeaderDocument)>,
    pub footers: Vec<(String, FooterDocument)>,
    pub footnotes: Option<FootnotesDocument>,
    pub endnotes: Option<EndnotesDocument>,
    pub comments: Option<CommentsDocument>,
    pub unparsed_store: UnparsedPartStore,
    pub content_types: ContentTypes,
    pub package_relationships: Relationships,
    pub part_relationships: BTreeMap<String, Relationships>,
}

impl Default for WordPackage {
    fn default() -> Self {
        Self::new()
    }
}

impl WordPackage {
    /// Creates a new, blank WordprocessingML package with standard defaults.
    pub fn new() -> Self {
        let mut content_types = ContentTypes::new();
        content_types.add_default("rels", ooxml_opc::content_types::RELATIONSHIPS_TYPE);
        content_types.add_default("xml", ooxml_opc::content_types::XML_DEFAULT_TYPE);
        content_types.add_override("word/document.xml", DOCUMENT_MAIN_TYPE);

        let mut package_relationships = Relationships::new();
        package_relationships.add(
            "rId1",
            DOCUMENT_REL_TYPE,
            "word/document.xml",
            ooxml_opc::relationships::TargetMode::Internal,
        );

        let document = Document {
            namespaces: vec![
                (
                    "xmlns:w".to_string(),
                    "http://schemas.openxmlformats.org/wordprocessingml/2006/main".to_string(),
                ),
                (
                    "xmlns:r".to_string(),
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
                        .to_string(),
                ),
                (
                    "xmlns:m".to_string(),
                    "http://schemas.openxmlformats.org/officeDocument/2006/math".to_string(),
                ),
                (
                    "xmlns:mc".to_string(),
                    "http://schemas.openxmlformats.org/markup-compatibility/2006".to_string(),
                ),
            ],
            ignorable: None,
            body: crate::model::Body::default(),
            unparsed_nodes: Vec::new(),
        };

        Self {
            document,
            headers: Vec::new(),
            footers: Vec::new(),
            footnotes: None,
            endnotes: None,
            comments: None,
            unparsed_store: UnparsedPartStore::new(),
            content_types,
            package_relationships,
            part_relationships: BTreeMap::new(),
        }
    }

    /// Opens an ECMA-376 `.docx` package from a seekable reader.
    pub fn open<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut pkg_reader = PackageReader::open(reader)?;

        // 1. Read and parse root document.xml
        let doc_part_name = if pkg_reader.contains_part("word/document.xml") {
            "word/document.xml"
        } else if pkg_reader.contains_part("/word/document.xml") {
            "/word/document.xml"
        } else {
            return Err(EngineError::PartNotFound("word/document.xml".to_string()));
        };

        let doc_bytes = pkg_reader.read_part(doc_part_name)?;
        let document = crate::parser::parse_document(&doc_bytes)?;

        // 2. Discover and parse auxiliary sub-documents (headers, footers, footnotes, endnotes, comments)
        let part_names = pkg_reader.part_names().to_vec();
        let mut headers = Vec::new();
        let mut footers = Vec::new();
        let mut footnotes = None;
        let mut endnotes = None;
        let mut comments = None;

        let mut modeled_parts = vec![
            "word/document.xml".to_string(),
            "/word/document.xml".to_string(),
        ];

        for name in &part_names {
            let clean = name.trim_start_matches('/');
            if clean.starts_with("word/header") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let header = crate::parser::parse_header(&bytes)?;
                headers.push((clean.to_string(), header));
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean.starts_with("word/footer") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let footer = crate::parser::parse_footer(&bytes)?;
                footers.push((clean.to_string(), footer));
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean == "word/footnotes.xml" {
                let bytes = pkg_reader.read_part(name)?;
                footnotes = Some(crate::parser::parse_footnotes(&bytes)?);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean == "word/endnotes.xml" {
                let bytes = pkg_reader.read_part(name)?;
                endnotes = Some(crate::parser::parse_endnotes(&bytes)?);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean == "word/comments.xml" {
                let bytes = pkg_reader.read_part(name)?;
                comments = Some(crate::parser::parse_comments(&bytes)?);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            }
        }

        // 3. Preserve root content types
        let content_types = pkg_reader.get_content_types()?.clone();

        // 4. Preserve package relationships (_rels/.rels)
        let package_relationships = match pkg_reader.get_relationships(None) {
            Ok(rels) => rels.clone(),
            Err(_) => Relationships::new(),
        };

        // 5. Preserve part-level relationships (e.g. word/_rels/document.xml.rels and sub-document rels)
        let mut part_relationships = BTreeMap::new();
        for name in &part_names {
            let clean = name.trim_start_matches('/');
            if clean.ends_with(".rels") && clean != "_rels/.rels" {
                // Determine source part
                let source_part = if let Some(idx) = clean.find("/_rels/") {
                    let dir = &clean[..idx];
                    let file = &clean[idx + 7..clean.len() - 5];
                    format!("{dir}/{file}")
                } else if let Some(stripped) = clean.strip_prefix("_rels/") {
                    stripped.trim_end_matches(".rels").to_string()
                } else {
                    continue;
                };

                if let Ok(rels) = pkg_reader.get_relationships(Some(&source_part)) {
                    part_relationships.insert(source_part, rels.clone());
                }
            }
        }

        // 6. Extract unmodeled parts (styles, settings, fontTable, webSettings, docProps, etc.)
        let modeled_refs: Vec<&str> = modeled_parts.iter().map(|s| s.as_str()).collect();
        let unparsed_store = pkg_reader.extract_unparsed_store(&modeled_refs)?;

        Ok(Self {
            document,
            headers,
            footers,
            footnotes,
            endnotes,
            comments,
            unparsed_store,
            content_types,
            package_relationships,
            part_relationships,
        })
    }

    /// Loads a `.docx` package directly from a file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path).map_err(EngineError::Io)?;
        Self::open(file)
    }

    /// Loads a `.docx` package directly from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::open(Cursor::new(bytes))
    }

    /// Extracts plaintext from the entire main document body.
    pub fn get_full_text(&self) -> String {
        self.document.get_full_text()
    }

    /// Creates a high-performance Piece-Tree text buffer initialized with the document text.
    pub fn create_piece_tree(&self) -> PieceTree {
        self.document.create_piece_tree()
    }

    /// Serializes and saves the WordprocessingML document and all parts back into a `.docx` package.
    pub fn save<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut pkg_writer = PackageWriter::new(writer);

        // 1. Restore root content types
        pkg_writer.set_content_types(self.content_types.clone());

        // 2. Restore package-level relationships
        pkg_writer.set_package_relationships(self.package_relationships.clone());

        // 3. Restore part-level relationships
        for (source_part, rels) in &self.part_relationships {
            pkg_writer.set_part_relationships(source_part, rels.clone());
        }

        // 4. Write all unmodeled parts and extensions
        pkg_writer.write_unparsed_store(&self.unparsed_store)?;

        // 5. Serialize and write root document.xml
        let doc_xml = serialize_document(&self.document)?;
        pkg_writer.write_part("word/document.xml", &doc_xml, Some(DOCUMENT_MAIN_TYPE))?;

        // 6. Serialize and write headers
        for (part_name, header) in &self.headers {
            let header_xml = serialize_header(header)?;
            let clean = part_name.trim_start_matches('/');
            pkg_writer.write_part(clean, &header_xml, Some(HEADER_TYPE))?;
        }

        // 7. Serialize and write footers
        for (part_name, footer) in &self.footers {
            let footer_xml = serialize_footer(footer)?;
            let clean = part_name.trim_start_matches('/');
            pkg_writer.write_part(clean, &footer_xml, Some(FOOTER_TYPE))?;
        }

        // 8. Serialize and write footnotes
        if let Some(ref fn_doc) = self.footnotes {
            let fn_xml = serialize_footnotes(fn_doc)?;
            pkg_writer.write_part("word/footnotes.xml", &fn_xml, Some(FOOTNOTES_TYPE))?;
        }

        // 9. Serialize and write endnotes
        if let Some(ref en_doc) = self.endnotes {
            let en_xml = serialize_endnotes(en_doc)?;
            pkg_writer.write_part("word/endnotes.xml", &en_xml, Some(ENDNOTES_TYPE))?;
        }

        // 10. Serialize and write comments
        if let Some(ref cm_doc) = self.comments {
            let cm_xml = serialize_comments(cm_doc)?;
            pkg_writer.write_part("word/comments.xml", &cm_xml, Some(COMMENTS_TYPE))?;
        }

        // 11. Finalize OPC package
        pkg_writer.finish()?;
        Ok(())
    }

    /// Saves the package to a file on disk.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path).map_err(EngineError::Io)?;
        self.save(file)
    }

    /// Serializes the package into an in-memory byte buffer.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        self.save(&mut cursor)?;
        Ok(cursor.into_inner())
    }
}
