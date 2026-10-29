//! Multi-Format W3C-Compliant Universal Clipboard & Cross-App Format Converters
//!
//! Complies with W3C Clipboard API and ECMA-376.
//! Provides:
//! - Multi-format MIME representation:
//!   - `text/plain`: Universal plaintext TSV or text
//!   - `text/html`: Styled HTML markup (`<table>`, `<tr>`, `<td>`, `<p>`, `<b>`, `<i>`)
//!   - `application/x-ooxml-*`: Lossless ECMA-376 XML fragments (`w:p`, `w:tbl`, `c`, `sheetData`)
//!   - `application/pdf`: Vector PDF data
//!   - `image/png`: Raster pixel data
//! - Cross-application converters:
//!   - Excel TSV <-> HTML Table <-> Word Table XML (`<w:tbl>`)
//!   - Word paragraph <-> HTML `<p>` <-> Plain text
//! - Ring-buffer clipboard history with bounded capacity

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================================
// 1. Multi-Format Clipboard Payload
// =========================================================================

/// Standardized multi-format office clipboard payload.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GlobalClipboardPayload {
    pub plain_text: Option<String>,
    pub html: Option<String>,
    pub ooxml: Option<String>,
    pub pdf_bytes: Option<Vec<u8>>,
    pub png_bytes: Option<Vec<u8>>,
    pub custom_formats: HashMap<String, Vec<u8>>,
}

impl GlobalClipboardPayload {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct payload from unformatted plain text.
    pub fn from_plain_text(text: impl Into<String>) -> Self {
        let t = text.into();
        let escaped = quick_xml::escape::escape(&t).to_string();
        Self {
            plain_text: Some(t),
            html: Some(format!("<p>{escaped}</p>")),
            ooxml: None,
            pdf_bytes: None,
            png_bytes: None,
            custom_formats: HashMap::new(),
        }
    }

    /// Construct payload from styled Word text.
    pub fn from_word(
        text: &str,
        is_bold: bool,
        is_italic: bool,
        font_family: Option<&str>,
        font_size_pt: Option<f32>,
    ) -> Self {
        let escaped = quick_xml::escape::escape(text);

        // Build HTML style string
        let mut css_parts = Vec::new();
        if is_bold {
            css_parts.push("font-weight:bold;".to_string());
        }
        if is_italic {
            css_parts.push("font-style:italic;".to_string());
        }
        if let Some(font) = font_family {
            css_parts.push(format!("font-family:'{font}';"));
        }
        if let Some(size) = font_size_pt {
            css_parts.push(format!("font-size:{size}pt;"));
        }

        let html = if css_parts.is_empty() {
            format!("<p>{escaped}</p>")
        } else {
            let style_str = css_parts.join(" ");
            format!(r#"<p><span style="{style_str}">{escaped}</span></p>"#)
        };

        // Build ECMA-376 WordprocessingML XML
        let mut rpr = String::new();
        if is_bold {
            rpr.push_str("<w:b/>");
        }
        if is_italic {
            rpr.push_str("<w:i/>");
        }
        if let Some(font) = font_family {
            rpr.push_str(&format!(r#"<w:rFonts w:ascii="{font}" w:hAnsi="{font}"/>"#,));
        }
        if let Some(size) = font_size_pt {
            let half_pts = (size * 2.0).round() as u32;
            rpr.push_str(&format!(r#"<w:sz w:val="{half_pts}"/>"#));
        }

        let rpr_xml = if rpr.is_empty() {
            String::new()
        } else {
            format!("<w:rPr>{rpr}</w:rPr>")
        };

        let ooxml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r>{rpr_xml}<w:t>{escaped}</w:t></w:r></w:p>"#
        );

        Self {
            plain_text: Some(text.to_string()),
            html: Some(html),
            ooxml: Some(ooxml),
            pdf_bytes: None,
            png_bytes: None,
            custom_formats: HashMap::new(),
        }
    }

    /// Construct payload from a 2D Excel spreadsheet grid.
    pub fn from_excel_grid(grid: &[Vec<String>]) -> Self {
        let plain = grid_to_tsv(grid);
        let html = tsv_to_html_table(&plain);
        let ooxml = tsv_to_word_table_xml(&plain);

        Self {
            plain_text: Some(plain),
            html: Some(html),
            ooxml: Some(ooxml),
            pdf_bytes: None,
            png_bytes: None,
            custom_formats: HashMap::new(),
        }
    }

    /// Construct payload from PDF extracted text.
    pub fn from_pdf_text(text: &str) -> Self {
        let escaped = quick_xml::escape::escape(text);
        Self {
            plain_text: Some(text.to_string()),
            html: Some(format!(
                r#"<div class="pdf-extracted"><p>{escaped}</p></div>"#
            )),
            ooxml: None,
            pdf_bytes: None,
            png_bytes: None,
            custom_formats: HashMap::new(),
        }
    }

    /// Construct payload from raster PNG bytes.
    pub fn from_image_png(png_bytes: Vec<u8>) -> Self {
        Self {
            plain_text: None,
            html: Some(
                r#"<img src="data:image/png;base64,..." alt="Clipboard Image"/>"#.to_string(),
            ),
            ooxml: None,
            pdf_bytes: None,
            png_bytes: Some(png_bytes),
            custom_formats: HashMap::new(),
        }
    }
}

// =========================================================================
// 2. Cross-App Format Converters
// =========================================================================

/// Convert a 2D string grid to tab-separated values (TSV).
pub fn grid_to_tsv(grid: &[Vec<String>]) -> String {
    let mut out = String::new();
    for (i, row) in grid.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&row.join("\t"));
    }
    out
}

/// Convert tab-separated values (TSV) to a 2D string grid.
pub fn tsv_to_grid(tsv: &str) -> Vec<Vec<String>> {
    if tsv.is_empty() {
        return Vec::new();
    }
    tsv.lines()
        .map(|line| line.split('\t').map(|s| s.to_string()).collect())
        .collect()
}

/// Convert tab-separated values (TSV) to HTML table markup.
pub fn tsv_to_html_table(tsv: &str) -> String {
    let mut html = String::from("<table>\n");
    for line in tsv.lines() {
        html.push_str("  <tr>\n");
        for cell in line.split('\t') {
            let escaped = quick_xml::escape::escape(cell);
            html.push_str(&format!("    <td>{escaped}</td>\n"));
        }
        html.push_str("  </tr>\n");
    }
    html.push_str("</table>");
    html
}

/// Convert HTML table markup to tab-separated values (TSV).
pub fn html_table_to_tsv(html: &str) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();

    let mut in_td = false;
    let mut in_th = false;
    let mut cell_buf = String::new();

    let mut reader = quick_xml::Reader::from_str(html);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            quick_xml::events::Event::Start(ref e) => {
                let qname = e.name();
                match qname.as_ref() {
                    b"tr" => {
                        current_row.clear();
                    }
                    b"td" => {
                        in_td = true;
                        cell_buf.clear();
                    }
                    b"th" => {
                        in_th = true;
                        cell_buf.clear();
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::End(ref e) => {
                let qname = e.name();
                match qname.as_ref() {
                    b"tr" => {
                        if !current_row.is_empty() {
                            rows.push(current_row.clone());
                        }
                    }
                    b"td" => {
                        in_td = false;
                        current_row.push(cell_buf.trim().to_string());
                        cell_buf.clear();
                    }
                    b"th" => {
                        in_th = false;
                        current_row.push(cell_buf.trim().to_string());
                        cell_buf.clear();
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Text(ref e) => {
                if in_td || in_th {
                    if let Ok(t) = e.unescape() {
                        cell_buf.push_str(&t);
                    }
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    grid_to_tsv(&rows)
}

/// Convert tab-separated values (TSV) to ECMA-376 `<w:tbl>` WordprocessingML markup.
pub fn tsv_to_word_table_xml(tsv: &str) -> String {
    let mut xml = String::from(
        r#"<w:tbl xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );

    for line in tsv.lines() {
        xml.push_str("<w:tr>");
        for cell in line.split('\t') {
            let escaped = quick_xml::escape::escape(cell);
            xml.push_str(&format!(
                "<w:tc><w:p><w:r><w:t>{escaped}</w:t></w:r></w:p></w:tc>"
            ));
        }
        xml.push_str("</w:tr>");
    }

    xml.push_str("</w:tbl>");
    xml
}

/// Convert ECMA-376 `<w:tbl>` WordprocessingML markup to tab-separated values (TSV).
pub fn word_table_xml_to_tsv(xml: &str) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut cell_buf = String::new();
    let mut in_tc = false;
    let mut in_t = false;

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut buf = Vec::new();

    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            quick_xml::events::Event::Start(ref e) => {
                let qname = e.name();
                match qname.as_ref() {
                    b"w:tr" => {
                        current_row.clear();
                    }
                    b"w:tc" => {
                        in_tc = true;
                        cell_buf.clear();
                    }
                    b"w:t" => {
                        in_t = true;
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::End(ref e) => {
                let qname = e.name();
                match qname.as_ref() {
                    b"w:tr" => {
                        if !current_row.is_empty() {
                            rows.push(current_row.clone());
                        }
                    }
                    b"w:tc" => {
                        in_tc = false;
                        current_row.push(cell_buf.clone());
                        cell_buf.clear();
                    }
                    b"w:t" => {
                        in_t = false;
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Text(ref e) => {
                if in_tc && in_t {
                    if let Ok(t) = e.unescape() {
                        cell_buf.push_str(&t);
                    }
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    grid_to_tsv(&rows)
}

// =========================================================================
// 3. Global Clipboard Manager
// =========================================================================

/// Universal office clipboard manager maintaining active payload and history ring buffer.
#[derive(Debug, Clone)]
pub struct GlobalClipboard {
    current: Option<GlobalClipboardPayload>,
    history: Vec<GlobalClipboardPayload>,
    capacity: usize,
}

impl Default for GlobalClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalClipboard {
    pub const DEFAULT_CAPACITY: usize = 25;

    /// Create a new clipboard with default 25-item history capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Create a new clipboard with specified history capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            current: None,
            history: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    /// Copy a new payload into the clipboard, pushing into history.
    pub fn copy(&mut self, payload: GlobalClipboardPayload) {
        if self.history.len() >= self.capacity {
            self.history.remove(0);
        }
        self.history.push(payload.clone());
        self.current = Some(payload);
    }

    /// Current active clipboard payload.
    pub fn current(&self) -> Option<&GlobalClipboardPayload> {
        self.current.as_ref()
    }

    /// Clipboard history in chronological order (oldest first).
    pub fn history(&self) -> &[GlobalClipboardPayload] {
        &self.history
    }

    /// Clear current payload and history.
    pub fn clear(&mut self) {
        self.current = None;
        self.history.clear();
    }

    /// Extract clipboard content as plain text.
    pub fn paste_plain_text(&self) -> Option<String> {
        self.current.as_ref().and_then(|p| p.plain_text.clone())
    }

    /// Extract clipboard content as a 2D Excel grid.
    ///
    /// Parses plain text TSV if available, or HTML table markup.
    pub fn paste_excel_grid(&self) -> Option<Vec<Vec<String>>> {
        let p = self.current.as_ref()?;
        if let Some(plain) = &p.plain_text {
            return Some(tsv_to_grid(plain));
        }
        if let Some(html) = &p.html {
            let tsv = html_table_to_tsv(html);
            return Some(tsv_to_grid(&tsv));
        }
        None
    }

    /// Extract clipboard content formatted as Word table XML (`<w:tbl>`).
    pub fn paste_as_word_table_xml(&self) -> Option<String> {
        let p = self.current.as_ref()?;
        if let Some(ooxml) = &p.ooxml {
            if ooxml.contains("<w:tbl") {
                return Some(ooxml.clone());
            }
        }
        if let Some(plain) = &p.plain_text {
            return Some(tsv_to_word_table_xml(plain));
        }
        None
    }
}
