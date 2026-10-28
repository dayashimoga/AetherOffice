//! Deterministic synthetic ECMA-376 Part 2 fixture generator for OpenOfficeRS.
//! Conforms strictly to GEC-v1.0 Article IV (zero panics, typed EngineError).

use crate::error::{EngineError, Result};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Summary manifest of an inspected ECMA-376 OPC package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticPackageManifest {
    /// List of all part paths contained within the package
    pub parts: Vec<String>,
    /// Whether `[Content_Types].xml` is present at the package root
    pub has_content_types: bool,
    /// Whether `_rels/.rels` is present at the package root
    pub has_root_rels: bool,
    /// Total byte size of the raw package buffer
    pub total_bytes: usize,
}

/// Helper to write an entry to an in-memory ZIP package.
fn write_zip_entry<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    filename: &str,
    content: &[u8],
) -> Result<()> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(filename, options)
        .map_err(EngineError::Zip)?;
    zip.write_all(content).map_err(EngineError::Io)?;
    Ok(())
}

/// Generates a deterministic, compliant minimal ECMA-376 Part 2 `.docx` synthetic fixture.
pub fn create_synthetic_docx() -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);

        let content_types = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
        write_zip_entry(&mut zip, "[Content_Types].xml", content_types)?;

        let root_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
        write_zip_entry(&mut zip, "_rels/.rels", root_rels)?;

        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>OpenOfficeRS Synthetic Word Document</w:t></w:r>
    </w:p>
    <w:p>
      <w:r><w:t>Clean-room deterministic fixture generated under GEC-v1.0.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;
        write_zip_entry(&mut zip, "word/document.xml", doc_xml)?;

        zip.finish().map_err(EngineError::Zip)?;
    }
    Ok(buffer.into_inner())
}

/// Generates a deterministic, compliant minimal ECMA-376 Part 2 `.xlsx` synthetic fixture.
pub fn create_synthetic_xlsx() -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);

        let content_types = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#;
        write_zip_entry(&mut zip, "[Content_Types].xml", content_types)?;

        let root_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
        write_zip_entry(&mut zip, "_rels/.rels", root_rels)?;

        let workbook_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#;
        write_zip_entry(&mut zip, "xl/workbook.xml", workbook_xml)?;

        let workbook_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#;
        write_zip_entry(&mut zip, "xl/_rels/workbook.xml.rels", workbook_rels)?;

        let sheet_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Synthetic Value A1</t></is></c>
      <c r="B1"><v>42</v></c>
    </row>
  </sheetData>
</worksheet>"#;
        write_zip_entry(&mut zip, "xl/worksheets/sheet1.xml", sheet_xml)?;

        zip.finish().map_err(EngineError::Zip)?;
    }
    Ok(buffer.into_inner())
}

/// Generates a deterministic, compliant minimal ECMA-376 Part 2 `.pptx` synthetic fixture.
pub fn create_synthetic_pptx() -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);

        let content_types = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
</Types>"#;
        write_zip_entry(&mut zip, "[Content_Types].xml", content_types)?;

        let root_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;
        write_zip_entry(&mut zip, "_rels/.rels", root_rels)?;

        let pres_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId1"/>
  </p:sldIdLst>
</p:presentation>"#;
        write_zip_entry(&mut zip, "ppt/presentation.xml", pres_xml)?;

        let pres_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#;
        write_zip_entry(&mut zip, "ppt/_rels/presentation.xml.rels", pres_rels)?;

        let slide_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
</p:sld>"#;
        write_zip_entry(&mut zip, "ppt/slides/slide1.xml", slide_xml)?;

        zip.finish().map_err(EngineError::Zip)?;
    }
    Ok(buffer.into_inner())
}

/// Inspects an in-memory byte buffer to verify ECMA-376 Part 2 OPC compliance.
pub fn inspect_opc_package(bytes: &[u8]) -> Result<SyntheticPackageManifest> {
    if bytes.is_empty() {
        return Err(EngineError::MalformedPackage(
            "Package buffer is empty".to_string(),
        ));
    }

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(EngineError::Zip)?;

    let mut parts = Vec::new();
    let mut has_content_types = false;
    let mut has_root_rels = false;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(EngineError::Zip)?;
        let name = file.name().to_string();
        if name == "[Content_Types].xml" {
            has_content_types = true;
        } else if name == "_rels/.rels" {
            has_root_rels = true;
        }
        parts.push(name);
    }

    if !has_content_types {
        return Err(EngineError::PartNotFound(
            "Required ECMA-376 Part 2 root '[Content_Types].xml' missing".to_string(),
        ));
    }
    if !has_root_rels {
        return Err(EngineError::RelationshipNotFound(
            "Required ECMA-376 Part 2 root relationship '_rels/.rels' missing".to_string(),
        ));
    }

    Ok(SyntheticPackageManifest {
        parts,
        has_content_types,
        has_root_rels,
        total_bytes: bytes.len(),
    })
}

/// Reads the raw text content of an individual part in an OPC package.
pub fn read_part_content(bytes: &[u8], part_name: &str) -> Result<String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(EngineError::Zip)?;
    let mut file = archive
        .by_name(part_name)
        .map_err(|_| EngineError::PartNotFound(part_name.to_string()))?;

    let mut content = String::new();
    file.read_to_string(&mut content).map_err(EngineError::Io)?;
    Ok(content)
}
