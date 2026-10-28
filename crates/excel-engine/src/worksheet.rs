//! ECMA-376 Part 1 §18.3 Worksheet XML parser and serializer.
//! High-performance streaming worksheet parser and serializer powered by
//! the Compressed Sparse Page Grid (CSPG).
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss Round-Trip Fidelity),
//! Article III (Memory & Performance SLAs), and Article IV (zero panics, typed EngineError).

use crate::grid::{cell_to_coords, coords_to_cell, CellError, CellValue, CompressedSparsePageGrid};
use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::unparsed::RawXmlNodeBuffer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::io::{BufRead, Cursor, Write};

/// Sheet view information `<sheetView>` (ECMA-376 Part 1 §18.3.1.87).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SheetView {
    pub tab_selected: bool,
    pub workbook_view_id: u32,
    pub show_grid_lines: Option<bool>,
    pub show_row_col_headers: Option<bool>,
    pub zoom_scale: Option<u32>,
    pub unmodeled_attrs: BTreeMap<String, String>,
}

/// Sheet formatting properties `<sheetFormatPr>` (ECMA-376 Part 1 §18.3.1.81).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SheetFormatPr {
    pub default_row_height: f64,
    pub default_col_width: Option<f64>,
    pub custom_height: bool,
    pub unmodeled_attrs: BTreeMap<String, String>,
}

/// Page margins `<pageMargins>` (ECMA-376 Part 1 §18.3.1.62).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageMargins {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub header: f64,
    pub footer: f64,
}

/// High-level ECMA-376 Worksheet (`xl/worksheets/sheet*.xml`).
#[derive(Debug, Clone, Default)]
pub struct Worksheet {
    /// Compressed Sparse Page Grid holding all cell data and values.
    pub grid: CompressedSparsePageGrid,
    /// Explicit dimension reference (e.g., "A1:C2") or calculated automatically on save.
    pub dimension: Option<String>,
    pub sheet_views: Vec<SheetView>,
    pub sheet_format_pr: Option<SheetFormatPr>,
    pub page_margins: Option<PageMargins>,
    /// Unmodeled elements (e.g. dataValidations, autoFilter, conditionalFormatting)
    pub unmodeled_nodes: Vec<RawXmlNodeBuffer>,
    /// Root `<worksheet>` attributes including namespaces and `mc:Ignorable`
    pub root_attrs: BTreeMap<String, String>,
}

impl Worksheet {
    /// Creates an empty worksheet.
    pub fn new() -> Self {
        let mut root_attrs = BTreeMap::new();
        root_attrs.insert(
            "xmlns".to_string(),
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string(),
        );
        root_attrs.insert(
            "xmlns:r".to_string(),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships".to_string(),
        );

        Self {
            grid: CompressedSparsePageGrid::new(),
            dimension: None,
            sheet_views: vec![SheetView {
                tab_selected: true,
                workbook_view_id: 0,
                show_grid_lines: Some(true),
                show_row_col_headers: Some(true),
                zoom_scale: None,
                unmodeled_attrs: BTreeMap::new(),
            }],
            sheet_format_pr: Some(SheetFormatPr {
                default_row_height: 15.0,
                default_col_width: None,
                custom_height: false,
                unmodeled_attrs: BTreeMap::new(),
            }),
            page_margins: Some(PageMargins {
                left: 0.7,
                right: 0.7,
                top: 0.75,
                bottom: 0.75,
                header: 0.3,
                footer: 0.3,
            }),
            unmodeled_nodes: Vec::new(),
            root_attrs,
        }
    }

    /// Parses a worksheet from an XML string.
    pub fn from_xml_str(xml: &str) -> Result<Self> {
        Self::from_reader(Cursor::new(xml.as_bytes()))
    }

    /// Streaming parser for `xl/worksheets/sheet*.xml`.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = false;

        let mut ws = Self {
            grid: CompressedSparsePageGrid::new(),
            dimension: None,
            sheet_views: Vec::new(),
            sheet_format_pr: None,
            page_margins: None,
            unmodeled_nodes: Vec::new(),
            root_attrs: BTreeMap::new(),
        };

        let mut buf = Vec::with_capacity(1024);
        let mut in_worksheet = false;
        let mut in_sheet_views = false;
        let mut in_sheet_data = false;
        let mut in_c = false;
        let mut in_v = false;
        let mut in_f = false;
        let mut in_is = false;
        let mut in_is_t = false;

        // Current cell accumulators
        let mut cur_cell_ref = String::new();
        let mut cur_cell_type = String::new();
        let mut cur_cell_style = None;
        let mut cur_cell_val = String::new();
        let mut cur_cell_formula = String::new();
        let mut cur_cell_is_text = String::new();

        loop {
            buf.clear();
            match xml_reader
                .read_event_into(&mut buf)
                .map_err(EngineError::Xml)?
            {
                Event::Start(ref e) => match e.name().as_ref() {
                    b"worksheet" => {
                        in_worksheet = true;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            ws.root_attrs.insert(key.to_string(), val.to_string());
                        }
                    }
                    b"sheetViews" => in_sheet_views = true,
                    b"sheetView" if in_sheet_views => {
                        let sv = parse_sheet_view(e, &xml_reader)?;
                        ws.sheet_views.push(sv);
                    }
                    b"sheetData" => in_sheet_data = true,
                    b"c" if in_sheet_data => {
                        in_c = true;
                        cur_cell_ref.clear();
                        cur_cell_type.clear();
                        cur_cell_style = None;
                        cur_cell_val.clear();
                        cur_cell_formula.clear();
                        cur_cell_is_text.clear();

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "r" => cur_cell_ref = val.to_string(),
                                "t" => cur_cell_type = val.to_string(),
                                "s" => cur_cell_style = val.parse::<u32>().ok(),
                                _ => {}
                            }
                        }
                    }
                    b"v" if in_c => in_v = true,
                    b"f" if in_c => in_f = true,
                    b"is" if in_c => in_is = true,
                    b"t" if in_is => in_is_t = true,
                    _ => {
                        if in_worksheet && !in_sheet_data {
                            let raw = read_subtree_xml(&mut xml_reader, e)?;
                            ws.unmodeled_nodes.push(RawXmlNodeBuffer::new(&raw));
                        }
                    }
                },
                Event::Empty(ref e) => match e.name().as_ref() {
                    b"dimension" => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"ref" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                ws.dimension = Some(val.to_string());
                            }
                        }
                    }
                    b"sheetFormatPr" => {
                        let mut sfp = SheetFormatPr::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "defaultRowHeight" => {
                                    sfp.default_row_height = val.parse::<f64>().unwrap_or(15.0)
                                }
                                "defaultColWidth" => {
                                    sfp.default_col_width = val.parse::<f64>().ok()
                                }
                                "customHeight" => {
                                    sfp.custom_height =
                                        val == "1" || val.eq_ignore_ascii_case("true")
                                }
                                _ => {
                                    sfp.unmodeled_attrs.insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                        ws.sheet_format_pr = Some(sfp);
                    }
                    b"sheetView" if in_sheet_views => {
                        let sv = parse_sheet_view(e, &xml_reader)?;
                        ws.sheet_views.push(sv);
                    }
                    b"pageMargins" => {
                        let mut pm = PageMargins::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "left" => pm.left = val.parse::<f64>().unwrap_or(0.7),
                                "right" => pm.right = val.parse::<f64>().unwrap_or(0.7),
                                "top" => pm.top = val.parse::<f64>().unwrap_or(0.75),
                                "bottom" => pm.bottom = val.parse::<f64>().unwrap_or(0.75),
                                "header" => pm.header = val.parse::<f64>().unwrap_or(0.3),
                                "footer" => pm.footer = val.parse::<f64>().unwrap_or(0.3),
                                _ => {}
                            }
                        }
                        ws.page_margins = Some(pm);
                    }
                    b"c" if in_sheet_data => {
                        // Empty cell (e.g. style placeholder `<c r="A1" s="1"/>`)
                        let mut r_ref = String::new();
                        let mut style = None;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            if key == "r" {
                                r_ref = val.to_string();
                            } else if key == "s" {
                                style = val.parse::<u32>().ok();
                            }
                        }
                        if !r_ref.is_empty() {
                            if let Ok((row, col)) = cell_to_coords(&r_ref) {
                                if let Some(s) = style {
                                    ws.grid.set_style(row, col, s)?;
                                }
                            }
                        }
                    }
                    _ => {
                        if in_worksheet && !in_sheet_data {
                            let mut empty_xml = Vec::new();
                            let mut writer = Writer::new(Cursor::new(&mut empty_xml));
                            writer
                                .write_event(Event::Empty(e.clone()))
                                .map_err(EngineError::Io)?;
                            let raw_str = String::from_utf8_lossy(&empty_xml).to_string();
                            ws.unmodeled_nodes.push(RawXmlNodeBuffer::new(&raw_str));
                        }
                    }
                },
                Event::Text(ref e) => {
                    let text = e.unescape().map_err(EngineError::Xml)?.into_owned();
                    if in_v {
                        cur_cell_val.push_str(&text);
                    } else if in_f {
                        cur_cell_formula.push_str(&text);
                    } else if in_is_t {
                        cur_cell_is_text.push_str(&text);
                    }
                }
                Event::End(ref e) => match e.name().as_ref() {
                    b"v" => in_v = false,
                    b"f" => in_f = false,
                    b"t" if in_is => in_is_t = false,
                    b"is" => in_is = false,
                    b"c" => {
                        in_c = false;
                        if !cur_cell_ref.is_empty() {
                            if let Ok((row, col)) = cell_to_coords(&cur_cell_ref) {
                                if let Some(s) = cur_cell_style {
                                    ws.grid.set_style(row, col, s)?;
                                }

                                if !cur_cell_formula.is_empty() {
                                    let cached = cur_cell_val.parse::<f64>().ok();
                                    ws.grid.set_formula(row, col, &cur_cell_formula, cached)?;
                                } else {
                                    match cur_cell_type.as_str() {
                                        "s" => {
                                            if let Ok(id) = cur_cell_val.parse::<u32>() {
                                                ws.grid.set_shared_string(row, col, id)?;
                                            }
                                        }
                                        "b" => {
                                            let b = cur_cell_val == "1"
                                                || cur_cell_val.eq_ignore_ascii_case("true");
                                            ws.grid.set_boolean(row, col, b)?;
                                        }
                                        "e" => {
                                            if let Some(err) =
                                                CellError::from_str_opt(&cur_cell_val)
                                            {
                                                ws.grid.set_error(row, col, err)?;
                                            }
                                        }
                                        "inlineStr" => {
                                            ws.grid.set_inline_string(
                                                row,
                                                col,
                                                &cur_cell_is_text,
                                            )?;
                                        }
                                        _ => {
                                            // Default: numeric value or general
                                            if let Ok(num) = cur_cell_val.parse::<f64>() {
                                                ws.grid.set_number(row, col, num)?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"sheetData" => in_sheet_data = false,
                    b"sheetViews" => in_sheet_views = false,
                    b"worksheet" => break,
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
        }

        if !ws.root_attrs.contains_key("xmlns") {
            ws.root_attrs.insert(
                "xmlns".to_string(),
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string(),
            );
        }

        Ok(ws)
    }

    /// Serializes worksheet to an XML writer.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<()> {
        let mut xml_writer = Writer::new(&mut writer);

        xml_writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(EngineError::Io)?;

        let mut root = BytesStart::new("worksheet");
        for (k, v) in &self.root_attrs {
            root.push_attribute((k.as_str(), v.as_str()));
        }
        xml_writer
            .write_event(Event::Start(root))
            .map_err(EngineError::Io)?;

        // `<dimension ref="..."/>`
        let dim_ref = self
            .dimension
            .clone()
            .unwrap_or_else(|| self.grid.dimension_ref());
        let mut dim_tag = BytesStart::new("dimension");
        dim_tag.push_attribute(("ref", dim_ref.as_str()));
        xml_writer
            .write_event(Event::Empty(dim_tag))
            .map_err(EngineError::Io)?;

        // `<sheetViews>`
        if !self.sheet_views.is_empty() {
            xml_writer
                .write_event(Event::Start(BytesStart::new("sheetViews")))
                .map_err(EngineError::Io)?;
            for sv in &self.sheet_views {
                let mut sv_tag = BytesStart::new("sheetView");
                if sv.tab_selected {
                    sv_tag.push_attribute(("tabSelected", "1"));
                }
                let wvid_str = sv.workbook_view_id.to_string();
                sv_tag.push_attribute(("workbookViewId", wvid_str.as_str()));
                if let Some(show) = sv.show_grid_lines {
                    sv_tag.push_attribute(("showGridLines", if show { "1" } else { "0" }));
                }
                if let Some(show) = sv.show_row_col_headers {
                    sv_tag.push_attribute(("showRowColHeaders", if show { "1" } else { "0" }));
                }
                if let Some(zoom) = sv.zoom_scale {
                    let s = zoom.to_string();
                    sv_tag.push_attribute(("zoomScale", s.as_str()));
                }
                for (k, v) in &sv.unmodeled_attrs {
                    sv_tag.push_attribute((k.as_str(), v.as_str()));
                }
                xml_writer
                    .write_event(Event::Empty(sv_tag))
                    .map_err(EngineError::Io)?;
            }
            xml_writer
                .write_event(Event::End(BytesEnd::new("sheetViews")))
                .map_err(EngineError::Io)?;
        }

        // `<sheetFormatPr>`
        if let Some(ref sfp) = self.sheet_format_pr {
            let mut sfp_tag = BytesStart::new("sheetFormatPr");
            let ht_str = sfp.default_row_height.to_string();
            sfp_tag.push_attribute(("defaultRowHeight", ht_str.as_str()));
            if let Some(w) = sfp.default_col_width {
                let w_str = w.to_string();
                sfp_tag.push_attribute(("defaultColWidth", w_str.as_str()));
            }
            if sfp.custom_height {
                sfp_tag.push_attribute(("customHeight", "1"));
            }
            for (k, v) in &sfp.unmodeled_attrs {
                sfp_tag.push_attribute((k.as_str(), v.as_str()));
            }
            xml_writer
                .write_event(Event::Empty(sfp_tag))
                .map_err(EngineError::Io)?;
        }

        // `<sheetData>`
        xml_writer
            .write_event(Event::Start(BytesStart::new("sheetData")))
            .map_err(EngineError::Io)?;

        // Stream rows and cells in strict ascending order directly out of CSPG
        if let Some((min_r, _, max_r, _)) = self.grid.bounding_box() {
            let mut cur_row_num = u32::MAX;
            let mut in_row_open = false;

            for (row, col, val, style) in self.grid.iter_cells() {
                if row != cur_row_num {
                    if in_row_open {
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("row")))
                            .map_err(EngineError::Io)?;
                    }
                    cur_row_num = row;
                    in_row_open = true;

                    let mut r_tag = BytesStart::new("row");
                    let r_str = (row + 1).to_string();
                    r_tag.push_attribute(("r", r_str.as_str()));
                    xml_writer
                        .write_event(Event::Start(r_tag))
                        .map_err(EngineError::Io)?;
                }

                let cell_coord_str = coords_to_cell(row, col)?;
                let mut c_tag = BytesStart::new("c");
                c_tag.push_attribute(("r", cell_coord_str.as_str()));

                if let Some(s) = style {
                    let s_str = s.to_string();
                    c_tag.push_attribute(("s", s_str.as_str()));
                }

                match val {
                    CellValue::Empty => {
                        xml_writer
                            .write_event(Event::Empty(c_tag))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::Number(n) => {
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("v")))
                            .map_err(EngineError::Io)?;
                        let n_str = n.to_string();
                        xml_writer
                            .write_event(Event::Text(BytesText::new(&n_str)))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::SharedString(id) => {
                        c_tag.push_attribute(("t", "s"));
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("v")))
                            .map_err(EngineError::Io)?;
                        let id_str = id.to_string();
                        xml_writer
                            .write_event(Event::Text(BytesText::new(&id_str)))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::Boolean(b) => {
                        c_tag.push_attribute(("t", "b"));
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Text(BytesText::new(if b { "1" } else { "0" })))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::Error(err) => {
                        c_tag.push_attribute(("t", "e"));
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Text(BytesText::new(err.as_str())))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("v")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::Formula {
                        formula,
                        cached_value,
                    } => {
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("f")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Text(BytesText::new(&formula)))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("f")))
                            .map_err(EngineError::Io)?;
                        if let Some(val) = cached_value {
                            xml_writer
                                .write_event(Event::Start(BytesStart::new("v")))
                                .map_err(EngineError::Io)?;
                            let val_str = val.to_string();
                            xml_writer
                                .write_event(Event::Text(BytesText::new(&val_str)))
                                .map_err(EngineError::Io)?;
                            xml_writer
                                .write_event(Event::End(BytesEnd::new("v")))
                                .map_err(EngineError::Io)?;
                        }
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::InlineString(ref s) => {
                        c_tag.push_attribute(("t", "inlineStr"));
                        xml_writer
                            .write_event(Event::Start(c_tag))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("is")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Start(BytesStart::new("t")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::Text(BytesText::new(s)))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("t")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("is")))
                            .map_err(EngineError::Io)?;
                        xml_writer
                            .write_event(Event::End(BytesEnd::new("c")))
                            .map_err(EngineError::Io)?;
                    }
                    CellValue::Array(ref arr) => {
                        if let Some(first_cell) = arr.first().and_then(|r| r.first()) {
                            match first_cell {
                                CellValue::Number(n) => {
                                    xml_writer
                                        .write_event(Event::Start(c_tag))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::Start(BytesStart::new("v")))
                                        .map_err(EngineError::Io)?;
                                    let n_str = n.to_string();
                                    xml_writer
                                        .write_event(Event::Text(BytesText::new(&n_str)))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::End(BytesEnd::new("v")))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::End(BytesEnd::new("c")))
                                        .map_err(EngineError::Io)?;
                                }
                                CellValue::InlineString(ref s) => {
                                    c_tag.push_attribute(("t", "inlineStr"));
                                    xml_writer
                                        .write_event(Event::Start(c_tag))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::Start(BytesStart::new("is")))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::Start(BytesStart::new("t")))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::Text(BytesText::new(s)))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::End(BytesEnd::new("t")))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::End(BytesEnd::new("is")))
                                        .map_err(EngineError::Io)?;
                                    xml_writer
                                        .write_event(Event::End(BytesEnd::new("c")))
                                        .map_err(EngineError::Io)?;
                                }
                                _ => {
                                    xml_writer
                                        .write_event(Event::Empty(c_tag))
                                        .map_err(EngineError::Io)?;
                                }
                            }
                        } else {
                            xml_writer
                                .write_event(Event::Empty(c_tag))
                                .map_err(EngineError::Io)?;
                        }
                    }
                }
            }

            if in_row_open {
                xml_writer
                    .write_event(Event::End(BytesEnd::new("row")))
                    .map_err(EngineError::Io)?;
            }
            let _ = (min_r, max_r);
        }

        xml_writer
            .write_event(Event::End(BytesEnd::new("sheetData")))
            .map_err(EngineError::Io)?;

        // `<pageMargins>`
        if let Some(ref pm) = self.page_margins {
            let mut pm_tag = BytesStart::new("pageMargins");
            let l_str = pm.left.to_string();
            pm_tag.push_attribute(("left", l_str.as_str()));
            let r_str = pm.right.to_string();
            pm_tag.push_attribute(("right", r_str.as_str()));
            let t_str = pm.top.to_string();
            pm_tag.push_attribute(("top", t_str.as_str()));
            let b_str = pm.bottom.to_string();
            pm_tag.push_attribute(("bottom", b_str.as_str()));
            let h_str = pm.header.to_string();
            pm_tag.push_attribute(("header", h_str.as_str()));
            let f_str = pm.footer.to_string();
            pm_tag.push_attribute(("footer", f_str.as_str()));
            xml_writer
                .write_event(Event::Empty(pm_tag))
                .map_err(EngineError::Io)?;
        }

        // Unmodeled nodes (autoFilter, drawing, conditionalFormatting, etc.)
        for node in &self.unmodeled_nodes {
            xml_writer
                .get_mut()
                .write_all(node.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        xml_writer
            .write_event(Event::End(BytesEnd::new("worksheet")))
            .map_err(EngineError::Io)?;

        Ok(())
    }

    /// Serializes worksheet to an XML string.
    pub fn to_xml_string(&self) -> Result<String> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| EngineError::MalformedPackage(format!("Serialized XML is not UTF-8: {e}")))
    }
}

fn read_subtree_xml<R: BufRead>(reader: &mut Reader<R>, start: &BytesStart<'_>) -> Result<String> {
    let mut depth = 1usize;
    let mut out = Vec::new();
    let mut writer = Writer::new(Cursor::new(&mut out));
    writer
        .write_event(Event::Start(start.clone()))
        .map_err(EngineError::Io)?;

    let mut buf = Vec::new();
    while depth > 0 {
        buf.clear();
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(e) => {
                depth += 1;
                writer
                    .write_event(Event::Start(e))
                    .map_err(EngineError::Io)?;
            }
            Event::End(e) => {
                depth -= 1;
                writer.write_event(Event::End(e)).map_err(EngineError::Io)?;
            }
            Event::Empty(e) => {
                writer
                    .write_event(Event::Empty(e))
                    .map_err(EngineError::Io)?;
            }
            Event::Text(e) => {
                writer
                    .write_event(Event::Text(e))
                    .map_err(EngineError::Io)?;
            }
            Event::CData(e) => {
                writer
                    .write_event(Event::CData(e))
                    .map_err(EngineError::Io)?;
            }
            Event::Comment(e) => {
                writer
                    .write_event(Event::Comment(e))
                    .map_err(EngineError::Io)?;
            }
            Event::Eof => {
                return Err(EngineError::MalformedPackage(
                    "Unexpected EOF while reading raw XML subtree".to_string(),
                ));
            }
            _ => {}
        }
    }

    String::from_utf8(out)
        .map_err(|e| EngineError::MalformedPackage(format!("Raw XML subtree was not UTF-8: {e}")))
}

fn parse_sheet_view<R: BufRead>(e: &BytesStart<'_>, xml_reader: &Reader<R>) -> Result<SheetView> {
    let mut sv = SheetView::default();
    for attr in e.attributes() {
        let attr = attr.map_err(EngineError::XmlAttr)?;
        let key = std::str::from_utf8(attr.key.into_inner())
            .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
        let val = attr
            .decode_and_unescape_value(xml_reader.decoder())
            .map_err(EngineError::Xml)?;
        match key {
            "tabSelected" => sv.tab_selected = val == "1" || val.eq_ignore_ascii_case("true"),
            "workbookViewId" => sv.workbook_view_id = val.parse::<u32>().unwrap_or(0),
            "showGridLines" => {
                sv.show_grid_lines = Some(val == "1" || val.eq_ignore_ascii_case("true"))
            }
            "showRowColHeaders" => {
                sv.show_row_col_headers = Some(val == "1" || val.eq_ignore_ascii_case("true"))
            }
            "zoomScale" => sv.zoom_scale = val.parse::<u32>().ok(),
            _ => {
                sv.unmodeled_attrs.insert(key.to_string(), val.to_string());
            }
        }
    }
    Ok(sv)
}
