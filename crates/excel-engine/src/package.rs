//! High-Performance Multi-Threaded Parallel Excel Package Engine.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity),
//! Article III (Startup & Performance SLAs), and Article IV (zero panics, typed EngineError).

use crate::shared_strings::SharedStringTable;
use crate::styles::StyleSheet;
use crate::workbook::Workbook;
use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::package::{PackageReader, PackageWriter};
use ooxml_opc::unparsed::UnparsedPartStore;
use std::io::{Read, Seek, Write};

/// An opened Excel document package containing workbook metadata, styles, and shared strings.
#[derive(Debug, Clone)]
pub struct ExcelWorkbookPackage {
    pub workbook: Workbook,
    pub shared_strings: SharedStringTable,
    pub styles: StyleSheet,
    pub unparsed_store: UnparsedPartStore,
}

impl ExcelWorkbookPackage {
    /// Opens an ECMA-376 `.xlsx` package and parses `workbook.xml`, `sharedStrings.xml`,
    /// and `styles.xml` in parallel across available CPU cores using `rayon`.
    pub fn open<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut pkg_reader = PackageReader::open(reader)?;

        // 1. Read raw XML payloads
        let wb_bytes = pkg_reader.read_part("xl/workbook.xml")?;
        let sst_bytes = if pkg_reader.contains_part("xl/sharedStrings.xml") {
            Some(pkg_reader.read_part("xl/sharedStrings.xml")?)
        } else {
            None
        };
        let styles_bytes = if pkg_reader.contains_part("xl/styles.xml") {
            Some(pkg_reader.read_part("xl/styles.xml")?)
        } else {
            None
        };

        // 2. Parallel multi-threaded XML parsing via Rayon
        let (wb_res, (sst_res, styles_res)) = rayon::join(
            || -> Result<Workbook> {
                let s = std::str::from_utf8(&wb_bytes)
                    .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                Workbook::from_xml_str(s)
            },
            || {
                rayon::join(
                    || -> Result<SharedStringTable> {
                        if let Some(bytes) = sst_bytes {
                            let s = std::str::from_utf8(&bytes)
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            SharedStringTable::from_xml_str(s)
                        } else {
                            Ok(SharedStringTable::new())
                        }
                    },
                    || -> Result<StyleSheet> {
                        if let Some(bytes) = styles_bytes {
                            let s = std::str::from_utf8(&bytes)
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            StyleSheet::from_xml_str(s)
                        } else {
                            Ok(StyleSheet::default())
                        }
                    },
                )
            },
        );

        let workbook = wb_res?;
        let shared_strings = sst_res?;
        let styles = styles_res?;

        // 3. Collect unparsed / unmodeled parts (vbaProject.bin, charts, drawings, unknown XML, etc.)
        let modeled_parts = [
            "xl/workbook.xml",
            "/xl/workbook.xml",
            "xl/sharedStrings.xml",
            "/xl/sharedStrings.xml",
            "xl/styles.xml",
            "/xl/styles.xml",
        ];

        let unparsed_store = pkg_reader.extract_unparsed_store(&modeled_parts)?;

        Ok(Self {
            workbook,
            shared_strings,
            styles,
            unparsed_store,
        })
    }

    /// Serializes and saves the workbook back into an ECMA-376 `.xlsx` package,
    /// faithfully preserving all unmodeled parts, relationships, and extensions.
    pub fn save<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut pkg_writer = PackageWriter::new(writer);

        // 1. Write unmodeled parts from the store
        pkg_writer.write_unparsed_store(&self.unparsed_store)?;

        // 2. Serialize and write modeled parts
        let wb_xml = self.workbook.to_xml_string()?;
        pkg_writer.write_part(
            "xl/workbook.xml",
            wb_xml.as_bytes(),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"),
        )?;

        if !self.shared_strings.is_empty() {
            let sst_xml = self.shared_strings.to_xml_string()?;
            pkg_writer.write_part(
                "xl/sharedStrings.xml",
                sst_xml.as_bytes(),
                Some(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml",
                ),
            )?;
        }

        let styles_xml = self.styles.to_xml_string()?;
        pkg_writer.write_part(
            "xl/styles.xml",
            styles_xml.as_bytes(),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"),
        )?;

        // 3. Finalize OPC package
        pkg_writer.finish()?;

        Ok(())
    }
}
