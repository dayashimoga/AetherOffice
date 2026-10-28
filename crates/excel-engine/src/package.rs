//! High-Performance Multi-Threaded Parallel Excel Package Engine.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity),
//! Article III (Startup & Performance SLAs), and Article IV (zero panics, typed EngineError).

use crate::shared_strings::SharedStringTable;
use crate::styles::StyleSheet;
use crate::workbook::Workbook;
use crate::worksheet::Worksheet;
use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::package::{PackageReader, PackageWriter};
use ooxml_opc::unparsed::UnparsedPartStore;
use rayon::prelude::*;
use std::io::{Read, Seek, Write};

/// An opened Excel document package containing workbook metadata, styles, shared strings,
/// and parallel-loaded worksheets.
#[derive(Debug, Clone)]
pub struct ExcelWorkbookPackage {
    pub workbook: Workbook,
    pub shared_strings: SharedStringTable,
    pub styles: StyleSheet,
    pub worksheets: Vec<(String, Worksheet)>,
    pub unparsed_store: UnparsedPartStore,
}

impl ExcelWorkbookPackage {
    /// Opens an ECMA-376 `.xlsx` package and parses `workbook.xml`, `sharedStrings.xml`,
    /// `styles.xml`, and all `xl/worksheets/sheet*.xml` in parallel across available CPU cores using `rayon`.
    pub fn open<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut pkg_reader = PackageReader::open(reader)?;

        // 1. Read raw core XML payloads
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

        // 2. Parallel multi-threaded XML parsing of core parts via Rayon
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

        // 3. Collect worksheet part names from workbook relationships or package enumeration
        let mut sheet_payloads = Vec::new();
        let mut modeled_parts_vec = vec![
            "xl/workbook.xml".to_string(),
            "/xl/workbook.xml".to_string(),
            "xl/sharedStrings.xml".to_string(),
            "/xl/sharedStrings.xml".to_string(),
            "xl/styles.xml".to_string(),
            "/xl/styles.xml".to_string(),
        ];

        // Attempt relationship resolution or fallback to direct naming
        for (i, sheet) in workbook.sheets.iter().enumerate() {
            let direct_name = format!("xl/worksheets/sheet{}.xml", i + 1);
            let target_name = if pkg_reader.contains_part(&direct_name) {
                direct_name
            } else if pkg_reader.contains_part(&sheet.rel_id) {
                sheet.rel_id.clone()
            } else {
                direct_name
            };

            if pkg_reader.contains_part(&target_name) {
                let bytes = pkg_reader.read_part(&target_name)?;
                modeled_parts_vec.push(target_name.clone());
                modeled_parts_vec.push(format!("/{target_name}"));
                sheet_payloads.push((sheet.name.clone(), target_name, bytes));
            }
        }

        // 4. Parallel multi-threaded parsing of all worksheets via Rayon
        let worksheets: Result<Vec<(String, Worksheet)>> = sheet_payloads
            .into_par_iter()
            .map(|(name, _part_path, bytes)| {
                let s = std::str::from_utf8(&bytes)
                    .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                let ws = Worksheet::from_xml_str(s)?;
                Ok((name, ws))
            })
            .collect();

        let worksheets = worksheets?;

        // 5. Collect unparsed / unmodeled parts (vbaProject.bin, charts, drawings, unknown XML, etc.)
        let modeled_parts_refs: Vec<&str> = modeled_parts_vec.iter().map(|s| s.as_str()).collect();
        let unparsed_store = pkg_reader.extract_unparsed_store(&modeled_parts_refs)?;

        Ok(Self {
            workbook,
            shared_strings,
            styles,
            worksheets,
            unparsed_store,
        })
    }

    /// Retrieves a worksheet by name.
    pub fn worksheet_by_name(&self, name: &str) -> Option<&Worksheet> {
        self.worksheets
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, ws)| ws)
    }

    /// Retrieves a mutable reference to a worksheet by name.
    pub fn worksheet_by_name_mut(&mut self, name: &str) -> Option<&mut Worksheet> {
        self.worksheets
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, ws)| ws)
    }

    /// Serializes and saves the workbook and all worksheets back into an ECMA-376 `.xlsx` package,
    /// faithfully preserving all unmodeled parts, relationships, and extensions.
    pub fn save<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut pkg_writer = PackageWriter::new(writer);

        // 1. Write unmodeled parts from the store
        pkg_writer.write_unparsed_store(&self.unparsed_store)?;

        // 2. Parallel serialization of all worksheets via Rayon
        let serialized_sheets: Result<Vec<(String, String)>> = self
            .worksheets
            .par_iter()
            .enumerate()
            .map(|(idx, (_, ws))| {
                let part_name = format!("xl/worksheets/sheet{}.xml", idx + 1);
                let xml = ws.to_xml_string()?;
                Ok((part_name, xml))
            })
            .collect();

        let serialized_sheets = serialized_sheets?;

        for (part_name, xml) in serialized_sheets {
            pkg_writer.write_part(
                &part_name,
                xml.as_bytes(),
                Some("application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"),
            )?;
        }

        // 3. Serialize and write modeled core parts
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

        // 4. Finalize OPC package
        pkg_writer.finish()?;

        Ok(())
    }
}
