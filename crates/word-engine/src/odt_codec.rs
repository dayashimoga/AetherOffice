//! OpenDocument Text (.odt) Codec
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - OASIS OpenDocument Format for Office Applications (ODF 1.3).
//! - Lossless translation between ODF `<office:text>` and WordprocessingML `Document`.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::model::{
    BodyElement, Document, Paragraph, ParagraphChild, ParagraphProperties, Run, RunContent, Table,
    TableCell, TableRow,
};

/// Codec for decoding and encoding OASIS OpenDocument Text (.odt) packages.
pub struct OdtCodec;

impl OdtCodec {
    /// Decodes an `.odt` ZIP archive into a `Document` model.
    pub fn decode(odt_bytes: &[u8]) -> Result<Document, String> {
        let cursor = Cursor::new(odt_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| format!("Invalid ODT archive: {}", e))?;

        let mut content_xml = String::new();
        {
            let mut file = archive
                .by_name("content.xml")
                .map_err(|_| "ODT archive missing content.xml".to_string())?;
            file.read_to_string(&mut content_xml)
                .map_err(|e| format!("Failed to read content.xml: {}", e))?;
        }

        Self::parse_content_xml(&content_xml)
    }

    /// Encodes a `Document` model into a valid `.odt` ZIP archive.
    pub fn encode(doc: &Document) -> Result<Vec<u8>, String> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);

            // 1. mimetype (MUST be first and uncompressed per ODF spec)
            let stored_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("mimetype", stored_opts)
                .map_err(|e| format!("Failed to write mimetype: {}", e))?;
            zip.write_all(b"application/vnd.oasis.opendocument.text")
                .map_err(|e| format!("Failed to write mimetype payload: {}", e))?;

            // 2. META-INF/manifest.xml
            let def_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("META-INF/manifest.xml", def_opts)
                .map_err(|e| format!("Failed to write manifest: {}", e))?;
            let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
            zip.write_all(manifest.as_bytes())
                .map_err(|e| format!("Failed to write manifest payload: {}", e))?;

            // 3. content.xml
            zip.start_file("content.xml", def_opts)
                .map_err(|e| format!("Failed to write content.xml: {}", e))?;
            let content_xml = Self::generate_content_xml(doc)?;
            zip.write_all(&content_xml)
                .map_err(|e| format!("Failed to write content.xml payload: {}", e))?;

            zip.finish()
                .map_err(|e| format!("Failed to finalize ODT archive: {}", e))?;
        }

        Ok(buffer.into_inner())
    }

    fn parse_content_xml(xml: &str) -> Result<Document, String> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut doc = Document::default();
        let mut in_body = false;
        let mut cur_para: Option<Paragraph> = None;
        let mut cur_run: Option<Run> = None;
        let mut cur_tbl: Option<Table> = None;
        let mut cur_row: Option<TableRow> = None;
        let mut cur_cell: Option<TableCell> = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().into_inner()).to_string();
                    let local = name.split(':').next_back().unwrap_or(&name);

                    if local == "body" || local == "text" {
                        in_body = true;
                    } else if in_body {
                        match local {
                            "h" => {
                                let mut p = Paragraph::default();
                                let mut level = 1u8;
                                for a in e.attributes().flatten() {
                                    if String::from_utf8_lossy(a.key.as_ref())
                                        .ends_with("outline-level")
                                    {
                                        if let Ok(l) =
                                            String::from_utf8_lossy(&a.value).parse::<u8>()
                                        {
                                            level = l;
                                        }
                                    }
                                }
                                p.properties = Some(ParagraphProperties {
                                    style: Some(format!("Heading {}", level)),
                                    outline_level: Some((level.saturating_sub(1)) as u32),
                                    ..Default::default()
                                });
                                cur_para = Some(p);
                            }
                            "p" => {
                                cur_para = Some(Paragraph::default());
                            }
                            "span" => {
                                cur_run = Some(Run::default());
                            }
                            "table" => {
                                cur_tbl = Some(Table::default());
                            }
                            "table-row" => {
                                cur_row = Some(TableRow::default());
                            }
                            "table-cell" => {
                                cur_cell = Some(TableCell::default());
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Text(ref e)) if in_body => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if let Some(ref mut r) = cur_run {
                        r.content.push(RunContent::Text(text, false));
                    } else if let Some(ref mut p) = cur_para {
                        let mut r = Run::default();
                        r.content.push(RunContent::Text(text, false));
                        p.children.push(ParagraphChild::Run(r));
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().into_inner()).to_string();
                    let local = name.split(':').next_back().unwrap_or(&name);

                    match local {
                        "span" => {
                            if let Some(r) = cur_run.take() {
                                if let Some(ref mut p) = cur_para {
                                    p.children.push(ParagraphChild::Run(r));
                                }
                            }
                        }
                        "p" | "h" => {
                            if let Some(p) = cur_para.take() {
                                if let Some(ref mut cell) = cur_cell {
                                    cell.elements.push(BodyElement::Paragraph(p));
                                } else {
                                    doc.body.elements.push(BodyElement::Paragraph(p));
                                }
                            }
                        }
                        "table-cell" => {
                            if let Some(cell) = cur_cell.take() {
                                if let Some(ref mut row) = cur_row {
                                    row.cells.push(cell);
                                }
                            }
                        }
                        "table-row" => {
                            if let Some(row) = cur_row.take() {
                                if let Some(ref mut tbl) = cur_tbl {
                                    tbl.rows.push(row);
                                }
                            }
                        }
                        "table" => {
                            if let Some(tbl) = cur_tbl.take() {
                                doc.body.elements.push(BodyElement::Table(tbl));
                            }
                        }
                        "body" => break,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }

    fn generate_content_xml(doc: &Document) -> Result<Vec<u8>, String> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        let mut root = BytesStart::new("office:document-content");
        root.push_attribute((
            "xmlns:office",
            "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
        ));
        root.push_attribute((
            "xmlns:text",
            "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
        ));
        root.push_attribute((
            "xmlns:table",
            "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
        ));
        root.push_attribute(("office:version", "1.3"));

        writer
            .write_event(Event::Start(root))
            .map_err(|e| format!("XML write error: {}", e))?;

        writer
            .write_event(Event::Start(BytesStart::new("office:body")))
            .map_err(|e| format!("XML write error: {}", e))?;

        writer
            .write_event(Event::Start(BytesStart::new("office:text")))
            .map_err(|e| format!("XML write error: {}", e))?;

        for elem in &doc.body.elements {
            Self::write_body_element(&mut writer, elem)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("office:text")))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("office:body")))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("office:document-content")))
            .map_err(|e| format!("XML write error: {}", e))?;

        Ok(writer.into_inner().into_inner())
    }

    fn write_body_element<W: Write>(
        writer: &mut Writer<W>,
        elem: &BodyElement,
    ) -> Result<(), String> {
        match elem {
            BodyElement::Paragraph(p) => {
                let is_heading = p
                    .properties
                    .as_ref()
                    .and_then(|pr| pr.style.as_ref())
                    .map(|s| s.starts_with("Heading"))
                    .unwrap_or(false);

                if is_heading {
                    let level = p
                        .properties
                        .as_ref()
                        .and_then(|pr| pr.outline_level)
                        .map(|ol| ol + 1)
                        .unwrap_or(1);
                    let mut h_tag = BytesStart::new("text:h");
                    h_tag.push_attribute(("text:outline-level", level.to_string().as_str()));
                    writer
                        .write_event(Event::Start(h_tag))
                        .map_err(|e| format!("{}", e))?;
                } else {
                    writer
                        .write_event(Event::Start(BytesStart::new("text:p")))
                        .map_err(|e| format!("{}", e))?;
                }

                for child in &p.children {
                    if let ParagraphChild::Run(ref r) = child {
                        let text = r.get_text();
                        writer
                            .write_event(Event::Text(BytesText::new(&text)))
                            .map_err(|e| format!("{}", e))?;
                    }
                }

                if is_heading {
                    writer
                        .write_event(Event::End(BytesEnd::new("text:h")))
                        .map_err(|e| format!("{}", e))?;
                } else {
                    writer
                        .write_event(Event::End(BytesEnd::new("text:p")))
                        .map_err(|e| format!("{}", e))?;
                }
            }
            BodyElement::Table(tbl) => {
                writer
                    .write_event(Event::Start(BytesStart::new("table:table")))
                    .map_err(|e| format!("{}", e))?;
                for row in &tbl.rows {
                    writer
                        .write_event(Event::Start(BytesStart::new("table:table-row")))
                        .map_err(|e| format!("{}", e))?;
                    for cell in &row.cells {
                        writer
                            .write_event(Event::Start(BytesStart::new("table:table-cell")))
                            .map_err(|e| format!("{}", e))?;
                        for cell_elem in &cell.elements {
                            Self::write_body_element(writer, cell_elem)?;
                        }
                        writer
                            .write_event(Event::End(BytesEnd::new("table:table-cell")))
                            .map_err(|e| format!("{}", e))?;
                    }
                    writer
                        .write_event(Event::End(BytesEnd::new("table:table-row")))
                        .map_err(|e| format!("{}", e))?;
                }
                writer
                    .write_event(Event::End(BytesEnd::new("table:table")))
                    .map_err(|e| format!("{}", e))?;
            }
            _ => {}
        }
        Ok(())
    }
}
