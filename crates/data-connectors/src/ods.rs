//! OASIS OpenDocument Spreadsheet (.ods) Codec.
//! Implements pure safe-Rust reading and writing of standard ODF 1.3 spreadsheet packages.

use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Field, Schema};
use crate::value::DataValue;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// ODS mimetype string required by OASIS ODF 1.3 specification.
pub const ODS_MIMETYPE: &[u8] = b"application/vnd.oasis.opendocument.spreadsheet";

/// Parsed data for a single sheet: (sheet_name, schema, records).
pub type OdsSheetData = (String, Schema, Vec<Vec<DataValue>>);

/// OpenDocument Spreadsheet archive reader.
pub struct OdsReader;

impl OdsReader {
    /// Reads and parses all sheets from an `.ods` ZIP buffer.
    /// Returns a list of sheet data.
    pub fn read_sheets(data: &[u8]) -> ConnectorResult<Vec<OdsSheetData>> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| ConnectorError::InvalidData(format!("Invalid ODS ZIP package: {e}")))?;

        let mut content_xml = String::new();
        {
            let mut file = archive.by_name("content.xml").map_err(|_| {
                ConnectorError::InvalidData("ODS archive missing content.xml".to_string())
            })?;
            file.read_to_string(&mut content_xml)
                .map_err(|e| ConnectorError::Io(format!("Failed to read content.xml: {e}")))?;
        }

        Self::parse_content_xml(&content_xml)
    }

    /// Parses the XML string of `content.xml`.
    pub fn parse_content_xml(xml: &str) -> ConnectorResult<Vec<OdsSheetData>> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut sheets = Vec::new();
        let mut cur_sheet_name: Option<String> = None;
        let mut cur_rows: Vec<Vec<DataValue>> = Vec::new();
        let mut cur_row: Vec<DataValue> = Vec::new();

        let mut cur_cell_val_type: Option<String> = None;
        let mut cur_cell_val_attr: Option<String> = None;
        let mut cur_cell_repeat: usize = 1;
        let mut in_cell = false;
        let mut in_text_p = false;
        let mut cell_text_buf = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let local_name = Self::local_name(qname.as_ref());
                    match local_name {
                        b"table" => {
                            let name = Self::extract_attr(e, b"name")
                                .unwrap_or_else(|| format!("Sheet{}", sheets.len() + 1));
                            cur_sheet_name = Some(name);
                            cur_rows.clear();
                        }
                        b"table-row" => {
                            cur_row.clear();
                        }
                        b"table-cell" | b"covered-table-cell" => {
                            in_cell = true;
                            cur_cell_val_type = Self::extract_attr(e, b"value-type");
                            cur_cell_val_attr = Self::extract_attr(e, b"value")
                                .or_else(|| Self::extract_attr(e, b"boolean-value"))
                                .or_else(|| Self::extract_attr(e, b"date-value"));
                            cur_cell_repeat = Self::extract_attr(e, b"number-columns-repeated")
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(1)
                                .min(1024); // Limit repeat count to avoid memory explosion
                            cell_text_buf.clear();
                        }
                        b"p" if in_cell => {
                            in_text_p = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) if in_text_p => {
                    if let Ok(txt) = e.unescape() {
                        cell_text_buf.push_str(&txt);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let local_name = Self::local_name(qname.as_ref());
                    match local_name {
                        b"p" => {
                            in_text_p = false;
                        }
                        b"table-cell" | b"covered-table-cell" => {
                            let val = Self::resolve_cell_value(
                                cur_cell_val_type.as_deref(),
                                cur_cell_val_attr.as_deref(),
                                &cell_text_buf,
                            );
                            for _ in 0..cur_cell_repeat {
                                cur_row.push(val.clone());
                            }
                            in_cell = false;
                            cur_cell_val_type = None;
                            cur_cell_val_attr = None;
                            cur_cell_repeat = 1;
                        }
                        b"table-row" => {
                            // Trim trailing nulls from row
                            while let Some(last) = cur_row.last() {
                                if last.is_null() {
                                    cur_row.pop();
                                } else {
                                    break;
                                }
                            }
                            if !cur_row.is_empty() {
                                cur_rows.push(cur_row.clone());
                            }
                            cur_row.clear();
                        }
                        b"table" => {
                            if let Some(name) = cur_sheet_name.take() {
                                let (schema, records) =
                                    Self::infer_sheet_schema_and_records(&cur_rows);
                                sheets.push((name, schema, records));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ConnectorError::InvalidData(format!(
                        "Error parsing ODS content.xml: {e}"
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(sheets)
    }

    fn resolve_cell_value(val_type: Option<&str>, val_attr: Option<&str>, text: &str) -> DataValue {
        match val_type {
            Some("float") | Some("currency") | Some("percentage") => {
                if let Some(attr) = val_attr {
                    if let Ok(f) = attr.parse::<f64>() {
                        if f.fract() == 0.0 && f >= (i64::MIN as f64) && f <= (i64::MAX as f64) {
                            return DataValue::Int64(f as i64);
                        } else {
                            return DataValue::Float64(f);
                        }
                    }
                }
                if let Ok(f) = text.trim().parse::<f64>() {
                    return DataValue::Float64(f);
                }
                DataValue::Text(text.to_string())
            }
            Some("boolean") => {
                let b = val_attr
                    .map(|a| a == "true")
                    .unwrap_or_else(|| text.trim().eq_ignore_ascii_case("true"));
                DataValue::Boolean(b)
            }
            Some("date") => {
                if let Some(attr) = val_attr {
                    DataValue::Text(attr.to_string())
                } else {
                    DataValue::Text(text.to_string())
                }
            }
            Some("string") => {
                if text.is_empty() {
                    DataValue::Null
                } else {
                    DataValue::Text(text.to_string())
                }
            }
            _ => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    DataValue::Null
                } else if let Ok(i) = trimmed.parse::<i64>() {
                    DataValue::Int64(i)
                } else if let Ok(f) = trimmed.parse::<f64>() {
                    DataValue::Float64(f)
                } else if trimmed.eq_ignore_ascii_case("true") {
                    DataValue::Boolean(true)
                } else if trimmed.eq_ignore_ascii_case("false") {
                    DataValue::Boolean(false)
                } else {
                    DataValue::Text(trimmed.to_string())
                }
            }
        }
    }

    fn infer_sheet_schema_and_records(rows: &[Vec<DataValue>]) -> (Schema, Vec<Vec<DataValue>>) {
        if rows.is_empty() {
            return (Schema::default(), Vec::new());
        }

        let first_row = &rows[0];
        let has_headers = first_row.iter().all(|v| matches!(v, DataValue::Text(_)));

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut header_names = Vec::with_capacity(num_cols);

        let data_start_row = if has_headers && rows.len() > 1 {
            for v in first_row {
                header_names.push(v.to_string_lossy());
            }
            while header_names.len() < num_cols {
                header_names.push(format!("Column_{}", header_names.len() + 1));
            }
            1
        } else {
            for i in 0..num_cols {
                header_names.push(format!("Column_{}", i + 1));
            }
            0
        };

        let records: Vec<Vec<DataValue>> = rows[data_start_row..].to_vec();

        let mut fields = Vec::with_capacity(num_cols);
        for (col_idx, name) in header_names.into_iter().enumerate() {
            let mut inferred = DataType::Utf8;
            for r in &records {
                if let Some(v) = r.get(col_idx) {
                    match v {
                        DataValue::Int64(_) => inferred = DataType::Int64,
                        DataValue::Float64(_) => inferred = DataType::Float64,
                        DataValue::Boolean(_) => inferred = DataType::Boolean,
                        DataValue::Date(_) => inferred = DataType::Date,
                        _ => {}
                    }
                }
            }
            fields.push(Field::new(name, inferred, true));
        }

        (Schema::new(fields), records)
    }

    fn local_name(name: &[u8]) -> &[u8] {
        if let Some(pos) = name.iter().position(|&b| b == b':') {
            &name[pos + 1..]
        } else {
            name
        }
    }

    fn extract_attr(e: &BytesStart, attr_local: &[u8]) -> Option<String> {
        for a in e.attributes().flatten() {
            let key = a.key.as_ref();
            let local = Self::local_name(key);
            if local == attr_local {
                return String::from_utf8(a.value.to_vec()).ok();
            }
        }
        None
    }
}

/// OpenDocument Spreadsheet archive generator.
pub struct OdsWriter;

impl OdsWriter {
    /// Serializes a single sheet into a standard OASIS ODF 1.3 `.ods` package.
    pub fn write_single_sheet(
        sheet_name: &str,
        schema: &Schema,
        rows: &[Vec<DataValue>],
    ) -> ConnectorResult<Vec<u8>> {
        Self::write_package(&[(sheet_name, schema, rows)])
    }

    /// Serializes multiple sheets into a standard OASIS ODF 1.3 `.ods` package.
    pub fn write_package(
        sheets: &[(&str, &Schema, &[Vec<DataValue>])],
    ) -> ConnectorResult<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);

            // 1. mimetype (MUST be first and uncompressed per ODF spec)
            let stored_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("mimetype", stored_opts)
                .map_err(|e| ConnectorError::Io(format!("Failed to write ODS mimetype: {e}")))?;
            zip.write_all(ODS_MIMETYPE).map_err(|e| {
                ConnectorError::Io(format!("Failed to write ODS mimetype payload: {e}"))
            })?;

            // 2. META-INF/manifest.xml
            let def_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("META-INF/manifest.xml", def_opts)
                .map_err(|e| ConnectorError::Io(format!("Failed to write manifest: {e}")))?;
            let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
            zip.write_all(manifest.as_bytes()).map_err(|e| {
                ConnectorError::Io(format!("Failed to write manifest payload: {e}"))
            })?;

            // 3. content.xml
            zip.start_file("content.xml", def_opts)
                .map_err(|e| ConnectorError::Io(format!("Failed to write content.xml: {e}")))?;
            let content_xml = Self::generate_content_xml(sheets);
            zip.write_all(content_xml.as_bytes()).map_err(|e| {
                ConnectorError::Io(format!("Failed to write content.xml payload: {e}"))
            })?;

            zip.finish()
                .map_err(|e| ConnectorError::Io(format!("Failed to finalize ODS archive: {e}")))?;
        }

        Ok(buffer.into_inner())
    }

    /// Generates standard ODF 1.3 `content.xml` for spreadsheets.
    pub fn generate_content_xml(sheets: &[(&str, &Schema, &[Vec<DataValue>])]) -> String {
        let mut xml = String::with_capacity(4096);
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<office:document-content xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\" ");
        xml.push_str("xmlns:table=\"urn:oasis:names:tc:opendocument:xmlns:table:1.0\" ");
        xml.push_str("xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\" ");
        xml.push_str("xmlns:xlink=\"http://www.w3.org/1999/xlink\" office:version=\"1.3\">\n");
        xml.push_str("  <office:body>\n");
        xml.push_str("    <office:spreadsheet>\n");

        for (sheet_name, schema, rows) in sheets {
            xml.push_str(&format!(
                "      <table:table table:name=\"{}\">\n",
                quick_xml::escape::escape(*sheet_name)
            ));

            // Headers row
            if !schema.fields.is_empty() {
                xml.push_str("        <table:table-row>\n");
                for f in &schema.fields {
                    let esc = quick_xml::escape::escape(&f.name);
                    xml.push_str(&format!(
                        "          <table:table-cell office:value-type=\"string\"><text:p>{esc}</text:p></table:table-cell>\n"
                    ));
                }
                xml.push_str("        </table:table-row>\n");
            }

            // Data rows
            for row in *rows {
                xml.push_str("        <table:table-row>\n");
                for val in row {
                    match val {
                        DataValue::Null => {
                            xml.push_str("          <table:table-cell/>\n");
                        }
                        DataValue::Boolean(b) => {
                            let b_str = if *b { "true" } else { "false" };
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"boolean\" office:boolean-value=\"{b_str}\"><text:p>{b_str}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::Int64(i) => {
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"float\" office:value=\"{i}\"><text:p>{i}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::Float64(f) => {
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"float\" office:value=\"{f}\"><text:p>{f}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::Date(d) => {
                            let text = val.to_string_lossy();
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"float\" office:value=\"{d}\"><text:p>{text}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::DateTime(dt) => {
                            let days = *dt as f64 / 86400000.0;
                            let text = val.to_string_lossy();
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"float\" office:value=\"{days}\"><text:p>{text}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::Text(s) => {
                            let esc = quick_xml::escape::escape(s);
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"string\"><text:p>{esc}</text:p></table:table-cell>\n"
                            ));
                        }
                        DataValue::Binary(b) => {
                            let desc = format!("<BLOB {} bytes>", b.len());
                            xml.push_str(&format!(
                                "          <table:table-cell office:value-type=\"string\"><text:p>{desc}</text:p></table:table-cell>\n"
                            ));
                        }
                    }
                }
                xml.push_str("        </table:table-row>\n");
            }

            xml.push_str("      </table:table>\n");
        }

        xml.push_str("    </office:spreadsheet>\n");
        xml.push_str("  </office:body>\n");
        xml.push_str("</office:document-content>\n");

        xml
    }
}
