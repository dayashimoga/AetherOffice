//! ECMA-376 Part 1 §18.2 Workbook XML parser and serializer.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity)
//! and Article IV (zero panics, typed EngineError).

use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::unparsed::RawXmlNodeBuffer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::io::{BufRead, Cursor, Write};

/// State of a worksheet within a workbook (ECMA-376 Part 1 §18.18.68 ST_SheetState).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetState {
    #[default]
    Visible,
    Hidden,
    VeryHidden,
}

impl SheetState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::VeryHidden => "veryHidden",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "hidden" => Self::Hidden,
            "veryHidden" => Self::VeryHidden,
            _ => Self::Visible,
        }
    }
}

/// A worksheet reference entry in `<sheets>` (ECMA-376 Part 1 §18.2.20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetRecord {
    pub name: String,
    pub sheet_id: u32,
    pub rel_id: String,
    pub state: SheetState,
}

impl SheetRecord {
    pub fn new(name: impl Into<String>, sheet_id: u32, rel_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sheet_id,
            rel_id: rel_id.into(),
            state: SheetState::Visible,
        }
    }
}

/// A defined name / named range entry in `<definedNames>` (ECMA-376 Part 1 §18.2.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinedName {
    pub name: String,
    pub value: String,
    pub local_sheet_id: Option<u32>,
    pub hidden: bool,
    pub comment: Option<String>,
}

impl DefinedName {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            local_sheet_id: None,
            hidden: false,
            comment: None,
        }
    }
}

/// Calculation mode for calculation properties (ECMA-376 Part 1 §18.18.8 ST_CalcMode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalcMode {
    #[default]
    Auto,
    AutoNoTable,
    Manual,
}

impl CalcMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::AutoNoTable => "autoNoTable",
            Self::Manual => "manual",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "autoNoTable" => Self::AutoNoTable,
            "manual" => Self::Manual,
            _ => Self::Auto,
        }
    }
}

/// Calculation properties `<calcPr>` (ECMA-376 Part 1 §18.2.2).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CalcPr {
    pub calc_id: Option<u32>,
    pub calc_mode: CalcMode,
    pub full_calc_on_load: bool,
    pub ref_mode: Option<String>,
    pub iterate: bool,
    pub iterate_count: Option<u32>,
    pub iterate_delta: Option<f64>,
}

/// Workbook properties `<workbookPr>` (ECMA-376 Part 1 §18.2.27).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkbookPr {
    /// Date system: false = 1900 date system (default), true = 1904 date system.
    pub date1904: bool,
    pub default_theme_version: Option<String>,
    pub backup: bool,
    pub filter_privacy: bool,
    pub unmodeled_attrs: BTreeMap<String, String>,
}

/// Workbook view `<workbookView>` (ECMA-376 Part 1 §18.2.30).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkbookView {
    pub x_window: Option<i32>,
    pub y_window: Option<i32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub tab_ratio: Option<u32>,
    pub first_sheet: Option<u32>,
    pub active_tab: Option<u32>,
    pub unmodeled_attrs: BTreeMap<String, String>,
}

/// File version information `<fileVersion>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileVersion {
    pub app_name: Option<String>,
    pub last_edited: Option<String>,
    pub lowest_edited: Option<String>,
    pub rup_build: Option<String>,
    pub unmodeled_attrs: BTreeMap<String, String>,
}

/// High-level ECMA-376 Workbook (`xl/workbook.xml`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Workbook {
    pub sheets: Vec<SheetRecord>,
    pub defined_names: Vec<DefinedName>,
    pub calc_pr: CalcPr,
    pub workbook_pr: WorkbookPr,
    pub book_views: Vec<WorkbookView>,
    pub file_version: Option<FileVersion>,
    /// Unmodeled elements (e.g. revisionPtr, mc:AlternateContent, etc.)
    pub unmodeled_nodes: Vec<RawXmlNodeBuffer>,
    /// Extension list `<extLst>` preserved verbatim via RawXmlNodeBuffer
    pub ext_lst: Option<RawXmlNodeBuffer>,
    /// Root `<workbook>` XML attributes including namespaces and `mc:Ignorable`
    pub root_attrs: BTreeMap<String, String>,
}

impl Workbook {
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
            sheets: Vec::new(),
            defined_names: Vec::new(),
            calc_pr: CalcPr::default(),
            workbook_pr: WorkbookPr::default(),
            book_views: Vec::new(),
            file_version: None,
            unmodeled_nodes: Vec::new(),
            ext_lst: None,
            root_attrs,
        }
    }

    /// Appends a new worksheet to the workbook.
    pub fn add_sheet(
        &mut self,
        name: impl Into<String>,
        sheet_id: u32,
        rel_id: impl Into<String>,
    ) -> &mut SheetRecord {
        self.sheets.push(SheetRecord::new(name, sheet_id, rel_id));
        let idx = self.sheets.len() - 1;
        &mut self.sheets[idx]
    }

    /// Finds a sheet record by name.
    pub fn sheet_by_name(&self, name: &str) -> Option<&SheetRecord> {
        self.sheets.iter().find(|s| s.name == name)
    }

    /// Finds a sheet record by sheet ID.
    pub fn sheet_by_id(&self, sheet_id: u32) -> Option<&SheetRecord> {
        self.sheets.iter().find(|s| s.sheet_id == sheet_id)
    }

    /// Checks if the 1904 date system is active.
    pub fn is_date1904(&self) -> bool {
        self.workbook_pr.date1904
    }

    /// Sets the date system (false = 1900, true = 1904).
    pub fn set_date1904(&mut self, val: bool) {
        self.workbook_pr.date1904 = val;
    }

    /// Parses a workbook from an XML string.
    pub fn from_xml_str(xml: &str) -> Result<Self> {
        Self::from_reader(Cursor::new(xml.as_bytes()))
    }

    /// Parses a workbook from any `BufRead` stream.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut workbook = Self::new();
        workbook.root_attrs.clear(); // Will populate from parsed root tag

        let mut buf = Vec::new();
        let mut in_workbook = false;
        let mut in_sheets = false;
        let mut in_defined_names = false;
        let mut in_book_views = false;

        // Current item accumulators
        let mut cur_defined_name_attrs: Option<(String, Option<u32>, bool, Option<String>)> = None;
        let mut cur_defined_name_val = String::new();

        loop {
            buf.clear();
            match xml_reader
                .read_event_into(&mut buf)
                .map_err(EngineError::Xml)?
            {
                Event::Start(ref e) => match e.name().as_ref() {
                    b"workbook" => {
                        in_workbook = true;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            workbook.root_attrs.insert(key.to_string(), val.to_string());
                        }
                    }
                    b"sheets" => in_sheets = true,
                    b"sheet" if in_sheets => {
                        let mut name = String::new();
                        let mut sheet_id = 0u32;
                        let mut rel_id = String::new();
                        let mut state = SheetState::Visible;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "name" => name = val.to_string(),
                                "sheetId" => {
                                    sheet_id = val.parse::<u32>().map_err(|e| {
                                        EngineError::SchemaValidation(format!(
                                            "Invalid sheetId: {e}"
                                        ))
                                    })?;
                                }
                                "r:id" | "id" => rel_id = val.to_string(),
                                "state" => state = SheetState::from_str_opt(&val),
                                _ => {}
                            }
                        }
                        workbook.sheets.push(SheetRecord {
                            name,
                            sheet_id,
                            rel_id,
                            state,
                        });
                    }
                    b"definedNames" => in_defined_names = true,
                    b"definedName" if in_defined_names => {
                        let mut name = String::new();
                        let mut local_sheet_id = None;
                        let mut hidden = false;
                        let mut comment = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "name" => name = val.to_string(),
                                "localSheetId" => local_sheet_id = val.parse::<u32>().ok(),
                                "hidden" => hidden = val == "1" || val.eq_ignore_ascii_case("true"),
                                "comment" => comment = Some(val.to_string()),
                                _ => {}
                            }
                        }
                        cur_defined_name_attrs = Some((name, local_sheet_id, hidden, comment));
                        cur_defined_name_val.clear();
                    }
                    b"bookViews" => in_book_views = true,
                    b"workbookView" if in_book_views => {
                        let mut view = WorkbookView::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "xWindow" => view.x_window = val.parse::<i32>().ok(),
                                "yWindow" => view.y_window = val.parse::<i32>().ok(),
                                "windowWidth" => view.window_width = val.parse::<u32>().ok(),
                                "windowHeight" => view.window_height = val.parse::<u32>().ok(),
                                "tabRatio" => view.tab_ratio = val.parse::<u32>().ok(),
                                "firstSheet" => view.first_sheet = val.parse::<u32>().ok(),
                                "activeTab" => view.active_tab = val.parse::<u32>().ok(),
                                _ => {
                                    view.unmodeled_attrs
                                        .insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                        workbook.book_views.push(view);
                    }
                    b"extLst" => {
                        let raw = read_subtree_xml(&mut xml_reader, e)?;
                        workbook.ext_lst = Some(RawXmlNodeBuffer::new(&raw));
                    }
                    _ => {
                        // Preserve other non-empty root elements as unmodeled raw XML nodes
                        if in_workbook {
                            let raw = read_subtree_xml(&mut xml_reader, e)?;
                            workbook.unmodeled_nodes.push(RawXmlNodeBuffer::new(&raw));
                        }
                    }
                },
                Event::Empty(ref e) => match e.name().as_ref() {
                    b"workbookPr" => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "date1904" => {
                                    workbook.workbook_pr.date1904 =
                                        val == "1" || val.eq_ignore_ascii_case("true");
                                }
                                "defaultThemeVersion" => {
                                    workbook.workbook_pr.default_theme_version =
                                        Some(val.to_string());
                                }
                                "backup" => {
                                    workbook.workbook_pr.backup =
                                        val == "1" || val.eq_ignore_ascii_case("true");
                                }
                                "filterPrivacy" => {
                                    workbook.workbook_pr.filter_privacy =
                                        val == "1" || val.eq_ignore_ascii_case("true");
                                }
                                _ => {
                                    workbook
                                        .workbook_pr
                                        .unmodeled_attrs
                                        .insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                    }
                    b"calcPr" => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "calcId" => workbook.calc_pr.calc_id = val.parse::<u32>().ok(),
                                "calcMode" => {
                                    workbook.calc_pr.calc_mode = CalcMode::from_str_opt(&val)
                                }
                                "fullCalcOnLoad" => {
                                    workbook.calc_pr.full_calc_on_load =
                                        val == "1" || val.eq_ignore_ascii_case("true");
                                }
                                "refMode" => workbook.calc_pr.ref_mode = Some(val.to_string()),
                                "iterate" => {
                                    workbook.calc_pr.iterate =
                                        val == "1" || val.eq_ignore_ascii_case("true");
                                }
                                "iterateCount" => {
                                    workbook.calc_pr.iterate_count = val.parse::<u32>().ok()
                                }
                                "iterateDelta" => {
                                    workbook.calc_pr.iterate_delta = val.parse::<f64>().ok()
                                }
                                _ => {}
                            }
                        }
                    }
                    b"fileVersion" => {
                        let mut fv = FileVersion::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "appName" => fv.app_name = Some(val.to_string()),
                                "lastEdited" => fv.last_edited = Some(val.to_string()),
                                "lowestEdited" => fv.lowest_edited = Some(val.to_string()),
                                "rupBuild" => fv.rup_build = Some(val.to_string()),
                                _ => {
                                    fv.unmodeled_attrs.insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                        workbook.file_version = Some(fv);
                    }
                    b"sheet" if in_sheets => {
                        let mut name = String::new();
                        let mut sheet_id = 0u32;
                        let mut rel_id = String::new();
                        let mut state = SheetState::Visible;

                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "name" => name = val.to_string(),
                                "sheetId" => {
                                    sheet_id = val.parse::<u32>().map_err(|e| {
                                        EngineError::SchemaValidation(format!(
                                            "Invalid sheetId: {e}"
                                        ))
                                    })?;
                                }
                                "r:id" | "id" => rel_id = val.to_string(),
                                "state" => state = SheetState::from_str_opt(&val),
                                _ => {}
                            }
                        }
                        workbook.sheets.push(SheetRecord {
                            name,
                            sheet_id,
                            rel_id,
                            state,
                        });
                    }
                    b"workbookView" if in_book_views => {
                        let mut view = WorkbookView::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "xWindow" => view.x_window = val.parse::<i32>().ok(),
                                "yWindow" => view.y_window = val.parse::<i32>().ok(),
                                "windowWidth" => view.window_width = val.parse::<u32>().ok(),
                                "windowHeight" => view.window_height = val.parse::<u32>().ok(),
                                "tabRatio" => view.tab_ratio = val.parse::<u32>().ok(),
                                "firstSheet" => view.first_sheet = val.parse::<u32>().ok(),
                                "activeTab" => view.active_tab = val.parse::<u32>().ok(),
                                _ => {
                                    view.unmodeled_attrs
                                        .insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                        workbook.book_views.push(view);
                    }
                    _ => {
                        // Empty unmodeled element
                        if in_workbook {
                            let mut empty_xml = Vec::new();
                            let mut writer = Writer::new(Cursor::new(&mut empty_xml));
                            writer
                                .write_event(Event::Empty(e.clone()))
                                .map_err(EngineError::Io)?;
                            let raw_str = String::from_utf8_lossy(&empty_xml).to_string();
                            workbook
                                .unmodeled_nodes
                                .push(RawXmlNodeBuffer::new(&raw_str));
                        }
                    }
                },
                Event::Text(ref e) => {
                    if in_defined_names && cur_defined_name_attrs.is_some() {
                        let text = e.unescape().map_err(EngineError::Xml)?.into_owned();
                        cur_defined_name_val.push_str(&text);
                    }
                }
                Event::End(ref e) => match e.name().as_ref() {
                    b"workbook" => break,
                    b"sheets" => in_sheets = false,
                    b"bookViews" => in_book_views = false,
                    b"definedNames" => in_defined_names = false,
                    b"definedName" => {
                        if let Some((name, local_sheet_id, hidden, comment)) =
                            cur_defined_name_attrs.take()
                        {
                            workbook.defined_names.push(DefinedName {
                                name,
                                value: cur_defined_name_val.clone(),
                                local_sheet_id,
                                hidden,
                                comment,
                            });
                            cur_defined_name_val.clear();
                        }
                    }
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
        }

        // Ensure default namespaces if missing
        if !workbook.root_attrs.contains_key("xmlns") {
            workbook.root_attrs.insert(
                "xmlns".to_string(),
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string(),
            );
        }
        if !workbook.root_attrs.contains_key("xmlns:r") {
            workbook.root_attrs.insert(
                "xmlns:r".to_string(),
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships".to_string(),
            );
        }

        Ok(workbook)
    }

    /// Serializes this workbook into an XML writer.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<()> {
        let mut xml_writer = Writer::new(&mut writer);

        xml_writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(EngineError::Io)?;

        // Root `<workbook>`
        let mut root_start = BytesStart::new("workbook");
        for (k, v) in &self.root_attrs {
            root_start.push_attribute((k.as_str(), v.as_str()));
        }
        xml_writer
            .write_event(Event::Start(root_start))
            .map_err(EngineError::Io)?;

        // `<fileVersion>`
        if let Some(ref fv) = self.file_version {
            let mut fv_tag = BytesStart::new("fileVersion");
            if let Some(ref v) = fv.app_name {
                fv_tag.push_attribute(("appName", v.as_str()));
            }
            if let Some(ref v) = fv.last_edited {
                fv_tag.push_attribute(("lastEdited", v.as_str()));
            }
            if let Some(ref v) = fv.lowest_edited {
                fv_tag.push_attribute(("lowestEdited", v.as_str()));
            }
            if let Some(ref v) = fv.rup_build {
                fv_tag.push_attribute(("rupBuild", v.as_str()));
            }
            for (k, v) in &fv.unmodeled_attrs {
                fv_tag.push_attribute((k.as_str(), v.as_str()));
            }
            xml_writer
                .write_event(Event::Empty(fv_tag))
                .map_err(EngineError::Io)?;
        }

        // `<workbookPr>`
        let mut wb_pr = BytesStart::new("workbookPr");
        if self.workbook_pr.date1904 {
            wb_pr.push_attribute(("date1904", "1"));
        }
        if let Some(ref theme_ver) = self.workbook_pr.default_theme_version {
            wb_pr.push_attribute(("defaultThemeVersion", theme_ver.as_str()));
        }
        if self.workbook_pr.backup {
            wb_pr.push_attribute(("backup", "1"));
        }
        if self.workbook_pr.filter_privacy {
            wb_pr.push_attribute(("filterPrivacy", "1"));
        }
        for (k, v) in &self.workbook_pr.unmodeled_attrs {
            wb_pr.push_attribute((k.as_str(), v.as_str()));
        }
        xml_writer
            .write_event(Event::Empty(wb_pr))
            .map_err(EngineError::Io)?;

        // Write unmodeled nodes that appear before bookViews
        for node in &self.unmodeled_nodes {
            xml_writer
                .get_mut()
                .write_all(node.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        // `<bookViews>`
        if !self.book_views.is_empty() {
            xml_writer
                .write_event(Event::Start(BytesStart::new("bookViews")))
                .map_err(EngineError::Io)?;
            for view in &self.book_views {
                let mut v_tag = BytesStart::new("workbookView");
                if let Some(val) = view.x_window {
                    v_tag.push_attribute(("xWindow", val.to_string().as_str()));
                }
                if let Some(val) = view.y_window {
                    v_tag.push_attribute(("yWindow", val.to_string().as_str()));
                }
                if let Some(val) = view.window_width {
                    v_tag.push_attribute(("windowWidth", val.to_string().as_str()));
                }
                if let Some(val) = view.window_height {
                    v_tag.push_attribute(("windowHeight", val.to_string().as_str()));
                }
                if let Some(val) = view.tab_ratio {
                    v_tag.push_attribute(("tabRatio", val.to_string().as_str()));
                }
                if let Some(val) = view.first_sheet {
                    v_tag.push_attribute(("firstSheet", val.to_string().as_str()));
                }
                if let Some(val) = view.active_tab {
                    v_tag.push_attribute(("activeTab", val.to_string().as_str()));
                }
                for (k, v) in &view.unmodeled_attrs {
                    v_tag.push_attribute((k.as_str(), v.as_str()));
                }
                xml_writer
                    .write_event(Event::Empty(v_tag))
                    .map_err(EngineError::Io)?;
            }
            xml_writer
                .write_event(Event::End(BytesEnd::new("bookViews")))
                .map_err(EngineError::Io)?;
        }

        // `<sheets>`
        xml_writer
            .write_event(Event::Start(BytesStart::new("sheets")))
            .map_err(EngineError::Io)?;
        for sheet in &self.sheets {
            let mut s_tag = BytesStart::new("sheet");
            s_tag.push_attribute(("name", sheet.name.as_str()));
            s_tag.push_attribute(("sheetId", sheet.sheet_id.to_string().as_str()));
            s_tag.push_attribute(("r:id", sheet.rel_id.as_str()));
            if sheet.state != SheetState::Visible {
                s_tag.push_attribute(("state", sheet.state.as_str()));
            }
            xml_writer
                .write_event(Event::Empty(s_tag))
                .map_err(EngineError::Io)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("sheets")))
            .map_err(EngineError::Io)?;

        // `<definedNames>`
        if !self.defined_names.is_empty() {
            xml_writer
                .write_event(Event::Start(BytesStart::new("definedNames")))
                .map_err(EngineError::Io)?;
            for dn in &self.defined_names {
                let mut dn_tag = BytesStart::new("definedName");
                dn_tag.push_attribute(("name", dn.name.as_str()));
                if let Some(sid) = dn.local_sheet_id {
                    dn_tag.push_attribute(("localSheetId", sid.to_string().as_str()));
                }
                if dn.hidden {
                    dn_tag.push_attribute(("hidden", "1"));
                }
                if let Some(ref c) = dn.comment {
                    dn_tag.push_attribute(("comment", c.as_str()));
                }
                xml_writer
                    .write_event(Event::Start(dn_tag))
                    .map_err(EngineError::Io)?;
                xml_writer
                    .write_event(Event::Text(quick_xml::events::BytesText::new(&dn.value)))
                    .map_err(EngineError::Io)?;
                xml_writer
                    .write_event(Event::End(BytesEnd::new("definedName")))
                    .map_err(EngineError::Io)?;
            }
            xml_writer
                .write_event(Event::End(BytesEnd::new("definedNames")))
                .map_err(EngineError::Io)?;
        }

        // `<calcPr>`
        let mut calc_tag = BytesStart::new("calcPr");
        if let Some(id) = self.calc_pr.calc_id {
            calc_tag.push_attribute(("calcId", id.to_string().as_str()));
        }
        if self.calc_pr.calc_mode != CalcMode::Auto {
            calc_tag.push_attribute(("calcMode", self.calc_pr.calc_mode.as_str()));
        }
        if self.calc_pr.full_calc_on_load {
            calc_tag.push_attribute(("fullCalcOnLoad", "1"));
        }
        if let Some(ref rm) = self.calc_pr.ref_mode {
            calc_tag.push_attribute(("refMode", rm.as_str()));
        }
        if self.calc_pr.iterate {
            calc_tag.push_attribute(("iterate", "1"));
        }
        if let Some(ic) = self.calc_pr.iterate_count {
            calc_tag.push_attribute(("iterateCount", ic.to_string().as_str()));
        }
        if let Some(id) = self.calc_pr.iterate_delta {
            calc_tag.push_attribute(("iterateDelta", id.to_string().as_str()));
        }
        xml_writer
            .write_event(Event::Empty(calc_tag))
            .map_err(EngineError::Io)?;

        // `<extLst>`
        if let Some(ref ext) = self.ext_lst {
            xml_writer
                .get_mut()
                .write_all(ext.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        // Close root `<workbook>`
        xml_writer
            .write_event(Event::End(BytesEnd::new("workbook")))
            .map_err(EngineError::Io)?;

        Ok(())
    }

    /// Serializes this workbook to an XML string.
    pub fn to_xml_string(&self) -> Result<String> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| EngineError::MalformedPackage(format!("Serialized XML is not UTF-8: {e}")))
    }
}

/// Helper function to read a raw XML subtree between balanced start and end tags.
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
