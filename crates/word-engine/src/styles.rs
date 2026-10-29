//! OpenOfficeRS Word Engine - ECMA-376 Part 1 §17.7 Style Hierarchy & Cascade Resolver.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Implements full style cascade: docDefaults -> table style -> paragraph style -> character style -> direct formatting,
//! cycle-safe `basedOn` inheritance chains, table conditional overrides, and toggle property inversion rules.

use crate::model::*;
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Cursor;

/// Style type as defined in ECMA-376 Part 1 §17.7.4.17 (`w:type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StyleType {
    #[default]
    Paragraph,
    Character,
    Table,
    Numbering,
}

impl StyleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Character => "character",
            Self::Table => "table",
            Self::Numbering => "numbering",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "character" => Self::Character,
            "table" => Self::Table,
            "numbering" => Self::Numbering,
            _ => Self::Paragraph,
        }
    }
}

/// Conditional table style formatting override type (`w:tblStylePr` `w:type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableStyleOverrideType {
    WholeTable,
    FirstRow,
    LastRow,
    FirstCol,
    LastCol,
    Band1Horz,
    Band2Horz,
    Band1Vert,
    Band2Vert,
    NeCell,
    NwCell,
    SeCell,
    SwCell,
}

impl TableStyleOverrideType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WholeTable => "wholeTable",
            Self::FirstRow => "firstRow",
            Self::LastRow => "lastRow",
            Self::FirstCol => "firstCol",
            Self::LastCol => "lastCol",
            Self::Band1Horz => "band1Horz",
            Self::Band2Horz => "band2Horz",
            Self::Band1Vert => "band1Vert",
            Self::Band2Vert => "band2Vert",
            Self::NeCell => "neCell",
            Self::NwCell => "nwCell",
            Self::SeCell => "seCell",
            Self::SwCell => "swCell",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "wholeTable" => Some(Self::WholeTable),
            "firstRow" => Some(Self::FirstRow),
            "lastRow" => Some(Self::LastRow),
            "firstCol" => Some(Self::FirstCol),
            "lastCol" => Some(Self::LastCol),
            "band1Horz" => Some(Self::Band1Horz),
            "band2Horz" => Some(Self::Band2Horz),
            "band1Vert" => Some(Self::Band1Vert),
            "band2Vert" => Some(Self::Band2Vert),
            "neCell" => Some(Self::NeCell),
            "nwCell" => Some(Self::NwCell),
            "seCell" => Some(Self::SeCell),
            "swCell" => Some(Self::SwCell),
            _ => None,
        }
    }
}

/// Conditional table style properties block: `<w:tblStylePr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableStyleOverride {
    pub override_type: Option<TableStyleOverrideType>,
    pub p_pr: Option<ParagraphProperties>,
    pub r_pr: Option<RunProperties>,
    pub tc_pr: Option<TableWidth>,
}

/// A style definition element: `<w:style>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Style {
    pub style_id: String,
    pub style_type: StyleType,
    pub name: Option<String>,
    pub based_on: Option<String>,
    pub next: Option<String>,
    pub link: Option<String>,
    pub is_default: bool,
    pub is_custom: bool,
    pub q_format: bool,
    pub p_pr: Option<ParagraphProperties>,
    pub r_pr: Option<RunProperties>,
    pub tbl_pr: Option<TableProperties>,
    pub tbl_style_pr: Vec<TableStyleOverride>,
    pub raw_elements: Vec<RawXmlNode>,
}

impl Style {
    pub fn new(style_id: impl Into<String>, style_type: StyleType) -> Self {
        Self {
            style_id: style_id.into(),
            style_type,
            is_default: false,
            is_custom: false,
            q_format: false,
            ..Default::default()
        }
    }
}

/// Document-wide formatting defaults: `<w:docDefaults>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DocDefaults {
    pub r_pr_default: Option<RunProperties>,
    pub p_pr_default: Option<ParagraphProperties>,
}

/// Entire `word/styles.xml` part container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StyleSheet {
    pub namespaces: Vec<(String, String)>,
    pub doc_defaults: DocDefaults,
    pub styles: BTreeMap<String, Style>,
    pub latent_styles: Option<RawXmlNode>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
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
            ],
            doc_defaults: DocDefaults {
                r_pr_default: Some(RunProperties {
                    size_half_points: Some(22), // 11pt default
                    font_ascii: Some("Calibri".to_string()),
                    ..Default::default()
                }),
                p_pr_default: None,
            },
            styles: BTreeMap::new(),
            latent_styles: None,
        }
    }

    pub fn add_style(&mut self, style: Style) {
        self.styles.insert(style.style_id.clone(), style);
    }

    pub fn get_style(&self, style_id: &str) -> Option<&Style> {
        self.styles.get(style_id)
    }

    pub fn default_paragraph_style(&self) -> Option<&Style> {
        self.styles
            .values()
            .find(|s| s.style_type == StyleType::Paragraph && s.is_default)
            .or_else(|| self.get_style("Normal"))
    }

    pub fn default_character_style(&self) -> Option<&Style> {
        self.styles
            .values()
            .find(|s| s.style_type == StyleType::Character && s.is_default)
    }

    pub fn from_xml_str(xml: &str) -> Result<Self> {
        parse_stylesheet(xml.as_bytes())
    }

    pub fn to_xml_string(&self) -> Result<String> {
        let bytes = serialize_stylesheet(self)?;
        String::from_utf8(bytes).map_err(|e| EngineError::MalformedPackage(e.to_string()))
    }
}

/// Fully computed run properties after evaluating the complete style cascade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRunProperties {
    pub bold: bool,
    pub italic: bool,
    pub underline: Option<String>,
    pub strike: bool,
    pub double_strike: bool,
    pub caps: bool,
    pub small_caps: bool,
    pub color: Option<String>,
    pub size_half_points: u32,
    pub font_ascii: String,
    pub font_h_ansi: Option<String>,
    pub font_east_asia: Option<String>,
    pub font_cs: Option<String>,
    pub highlight: Option<String>,
    pub vertical_align: Option<String>,
}

impl Default for ResolvedRunProperties {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: None,
            strike: false,
            double_strike: false,
            caps: false,
            small_caps: false,
            color: None,
            size_half_points: 20, // 10pt standard fallback
            font_ascii: "Calibri".to_string(),
            font_h_ansi: None,
            font_east_asia: None,
            font_cs: None,
            highlight: None,
            vertical_align: None,
        }
    }
}

/// Fully computed paragraph properties after evaluating the complete style cascade.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResolvedParagraphProperties {
    pub style_id: Option<String>,
    pub justification: Option<String>,
    pub spacing_before: u32,
    pub spacing_after: u32,
    pub line_spacing: u32,
    pub line_rule: String,
    pub indent_left: i32,
    pub indent_right: i32,
    pub indent_first_line: i32,
    pub indent_hanging: i32,
    pub outline_level: Option<u32>,
    pub num_id: Option<u32>,
    pub num_level: Option<u32>,
    pub run_defaults: ResolvedRunProperties,
}

/// High-performance ECMA-376 Part 1 §17.7 style cascade resolver.
pub struct StyleCascadeResolver<'a> {
    stylesheet: &'a StyleSheet,
}

impl<'a> StyleCascadeResolver<'a> {
    pub fn new(stylesheet: &'a StyleSheet) -> Self {
        Self { stylesheet }
    }

    /// Resolves the full `basedOn` inheritance chain for a style, avoiding cycles.
    /// Returns style references from root ancestor down to the specified style.
    pub fn get_inheritance_chain(&self, style_id: &str) -> Vec<&'a Style> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut curr_id = Some(style_id.to_string());

        while let Some(id) = curr_id {
            if visited.contains(&id) {
                // Break cyclic inheritance
                break;
            }
            visited.insert(id.clone());

            if let Some(style) = self.stylesheet.get_style(&id) {
                chain.push(style);
                curr_id = style.based_on.clone();
            } else {
                break;
            }
        }

        // Reverse chain so base ancestor is at index 0 and target style is last
        chain.reverse();
        chain
    }

    /// Resolves effective paragraph properties for a paragraph.
    pub fn resolve_paragraph(&self, p: &Paragraph) -> ResolvedParagraphProperties {
        let mut resolved = ResolvedParagraphProperties {
            line_rule: "auto".to_string(),
            line_spacing: 240, // 1 line default (240 twips)
            ..Default::default()
        };

        // 1. Layer docDefaults
        if let Some(ref d_p) = self.stylesheet.doc_defaults.p_pr_default {
            self.apply_p_pr(&mut resolved, d_p);
        }
        if let Some(ref d_r) = self.stylesheet.doc_defaults.r_pr_default {
            self.apply_r_pr(&mut resolved.run_defaults, d_r, false);
        }

        // 2. Identify paragraph style
        let style_id = p
            .properties
            .as_ref()
            .and_then(|p_pr| p_pr.style.as_deref())
            .unwrap_or("Normal");

        resolved.style_id = Some(style_id.to_string());

        // 3. Apply style inheritance chain (root ancestor -> derived style)
        let chain = self.get_inheritance_chain(style_id);
        for style in chain {
            if let Some(ref p_pr) = style.p_pr {
                self.apply_p_pr(&mut resolved, p_pr);
            }
            if let Some(ref r_pr) = style.r_pr {
                self.apply_r_pr(&mut resolved.run_defaults, r_pr, false);
            }
        }

        // 4. Apply direct paragraph properties
        if let Some(ref dir_p) = p.properties {
            self.apply_p_pr(&mut resolved, dir_p);
            if let Some(ref dir_r) = dir_p.r_pr {
                self.apply_r_pr(&mut resolved.run_defaults, dir_r, false);
            }
        }

        resolved
    }

    /// Resolves effective run properties for a text run inside a paragraph.
    pub fn resolve_run(&self, r: &Run, p: &Paragraph) -> ResolvedRunProperties {
        let p_resolved = self.resolve_paragraph(p);
        let mut resolved = p_resolved.run_defaults;

        // 1. Check for character style (w:rStyle)
        let char_style_id = self.extract_character_style_id(r);
        if let Some(cs_id) = char_style_id {
            let chain = self.get_inheritance_chain(&cs_id);
            for style in chain {
                if let Some(ref r_pr) = style.r_pr {
                    self.apply_r_pr(&mut resolved, r_pr, false);
                }
            }
        }

        // 2. Apply direct run formatting with toggle property rules
        if let Some(ref dir_r) = r.properties {
            self.apply_r_pr(&mut resolved, dir_r, true);
        }

        resolved
    }

    /// Resolves effective run properties within a table cell, applying table style conditional blocks.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_table_cell_run(
        &self,
        r: &Run,
        p: &Paragraph,
        table_style_id: Option<&str>,
        row_idx: usize,
        col_idx: usize,
        num_rows: usize,
        num_cols: usize,
    ) -> ResolvedRunProperties {
        let mut resolved = self.resolve_run(r, p);

        if let Some(ts_id) = table_style_id {
            let chain = self.get_inheritance_chain(ts_id);
            for style in chain {
                for cond in &style.tbl_style_pr {
                    if let Some(override_type) = cond.override_type {
                        if self.is_table_condition_met(
                            override_type,
                            row_idx,
                            col_idx,
                            num_rows,
                            num_cols,
                        ) {
                            if let Some(ref r_pr) = cond.r_pr {
                                self.apply_r_pr(&mut resolved, r_pr, false);
                            }
                        }
                    }
                }
            }
        }

        resolved
    }

    #[allow(clippy::too_many_arguments)]
    fn is_table_condition_met(
        &self,
        cond: TableStyleOverrideType,
        row: usize,
        col: usize,
        num_rows: usize,
        num_cols: usize,
    ) -> bool {
        match cond {
            TableStyleOverrideType::WholeTable => true,
            TableStyleOverrideType::FirstRow => row == 0,
            TableStyleOverrideType::LastRow => row == num_rows.saturating_sub(1),
            TableStyleOverrideType::FirstCol => col == 0,
            TableStyleOverrideType::LastCol => col == num_cols.saturating_sub(1),
            TableStyleOverrideType::Band1Horz => row > 0 && row % 2 == 1,
            TableStyleOverrideType::Band2Horz => row > 0 && row.is_multiple_of(2),
            TableStyleOverrideType::Band1Vert => col > 0 && col % 2 == 1,
            TableStyleOverrideType::Band2Vert => col > 0 && col.is_multiple_of(2),
            TableStyleOverrideType::NwCell => row == 0 && col == 0,
            TableStyleOverrideType::NeCell => row == 0 && col == num_cols.saturating_sub(1),
            TableStyleOverrideType::SwCell => row == num_rows.saturating_sub(1) && col == 0,
            TableStyleOverrideType::SeCell => {
                row == num_rows.saturating_sub(1) && col == num_cols.saturating_sub(1)
            }
        }
    }

    fn extract_character_style_id(&self, r: &Run) -> Option<String> {
        if let Some(ref r_pr) = r.properties {
            for raw in &r_pr.raw_elements {
                if raw.tag.ends_with("rStyle") {
                    for (k, v) in &raw.attributes {
                        if k.ends_with("val") {
                            return Some(v.clone());
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_p_pr(&self, target: &mut ResolvedParagraphProperties, src: &ParagraphProperties) {
        if let Some(ref jc) = src.justification {
            target.justification = Some(jc.clone());
        }
        if let Some(ref sp) = src.spacing {
            if let Some(b) = sp.before {
                target.spacing_before = b;
            }
            if let Some(a) = sp.after {
                target.spacing_after = a;
            }
            if let Some(l) = sp.line {
                target.line_spacing = l;
            }
            if let Some(ref lr) = sp.line_rule {
                target.line_rule = lr.clone();
            }
        }
        if let Some(ref ind) = src.indentation {
            if let Some(l) = ind.left {
                target.indent_left = l;
            }
            if let Some(r) = ind.right {
                target.indent_right = r;
            }
            if let Some(fl) = ind.first_line {
                target.indent_first_line = fl;
            }
            if let Some(h) = ind.hanging {
                target.indent_hanging = h;
            }
        }
        if let Some(ref num) = src.num_pr {
            target.num_id = Some(num.num_id);
            target.num_level = Some(num.ilvl);
        }
    }

    fn apply_r_pr(&self, target: &mut ResolvedRunProperties, src: &RunProperties, is_direct: bool) {
        // Toggle property resolution (ECMA-376 Part 1 §17.7.3)
        // If direct run formatting specifies bold/italic/strike:
        // A direct <w:b/> element without val or with val="1" toggles the inherited style value!
        if is_direct {
            if src.bold {
                target.bold = !target.bold;
            }
            if src.italic {
                target.italic = !target.italic;
            }
            if src.strike {
                target.strike = !target.strike;
            }
        } else {
            if src.bold {
                target.bold = true;
            }
            if src.italic {
                target.italic = true;
            }
            if src.strike {
                target.strike = true;
            }
        }

        if src.double_strike {
            target.double_strike = true;
        }
        if let Some(ref u) = src.underline {
            target.underline = Some(u.clone());
        }
        if let Some(ref c) = src.color {
            target.color = Some(c.clone());
        }
        if let Some(sz) = src.size_half_points {
            target.size_half_points = sz;
        }
        if let Some(ref font) = src.font_ascii {
            target.font_ascii = font.clone();
        }
        if let Some(ref font) = src.font_h_ansi {
            target.font_h_ansi = Some(font.clone());
        }
        if let Some(ref font) = src.font_east_asia {
            target.font_east_asia = Some(font.clone());
        }
        if let Some(ref font) = src.font_cs {
            target.font_cs = Some(font.clone());
        }
        if let Some(ref hl) = src.highlight {
            target.highlight = Some(hl.clone());
        }
    }
}

// --- Streaming XML Parser for word/styles.xml ---

pub fn parse_stylesheet(xml_bytes: &[u8]) -> Result<StyleSheet> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut sheet = StyleSheet::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "styles" => {
                for attr in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(attr.key.into_inner()).to_string();
                    let v = String::from_utf8_lossy(&attr.value).to_string();
                    sheet.namespaces.push((k, v));
                }
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "docDefaults" => {
                sheet.doc_defaults = parse_doc_defaults(&mut reader)?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "style" => {
                let style = parse_style(&mut reader, e)?;
                sheet.add_style(style);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(sheet)
}

fn parse_doc_defaults<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<DocDefaults> {
    let mut defaults = DocDefaults::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "rPrDefault" => {
                defaults.r_pr_default = parse_r_pr_default(reader)?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "pPrDefault" => {
                defaults.p_pr_default = parse_p_pr_default(reader)?;
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "docDefaults" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(defaults)
}

fn parse_r_pr_default<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<Option<RunProperties>> {
    let mut r_pr = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "rPr" => {
                r_pr = Some(crate::parser::parse_run_properties(reader)?);
            }
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "rPr" => {
                r_pr = Some(RunProperties::default());
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "rPrDefault" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(r_pr)
}

fn parse_p_pr_default<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<Option<ParagraphProperties>> {
    let mut p_pr = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "pPr" => {
                p_pr = Some(crate::parser::parse_paragraph_properties(reader)?);
            }
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "pPr" => {
                p_pr = Some(ParagraphProperties::default());
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "pPrDefault" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(p_pr)
}

fn parse_style<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<Style> {
    let mut style = Style::default();

    for attr in start_tag.attributes().flatten() {
        let k = local_name(attr.key.into_inner());
        let v = String::from_utf8_lossy(&attr.value).to_string();
        match k {
            "type" => style.style_type = StyleType::from_str_opt(&v),
            "styleId" => style.style_id = v,
            "default" => style.is_default = v == "1" || v == "true" || v == "on",
            "customStyle" => style.is_custom = v == "1" || v == "true",
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pPr" => style.p_pr = Some(crate::parser::parse_paragraph_properties(reader)?),
                    "rPr" => style.r_pr = Some(crate::parser::parse_run_properties(reader)?),
                    "tblStylePr" => {
                        let override_type = get_attr_val(e, "type")
                            .and_then(|t| TableStyleOverrideType::from_str_opt(&t));
                        let cond = parse_table_style_override(reader, override_type)?;
                        style.tbl_style_pr.push(cond);
                    }
                    _ => style
                        .raw_elements
                        .push(crate::parser::parse_raw_xml_node(reader, e)?),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "name" => style.name = get_attr_val(e, "val"),
                    "basedOn" => style.based_on = get_attr_val(e, "val"),
                    "next" => style.next = get_attr_val(e, "val"),
                    "link" => style.link = get_attr_val(e, "val"),
                    "qFormat" => style.q_format = true,
                    _ => style
                        .raw_elements
                        .push(crate::parser::parse_empty_raw_node(e)?),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "style" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(style)
}

fn parse_table_style_override<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    override_type: Option<TableStyleOverrideType>,
) -> Result<TableStyleOverride> {
    let mut cond = TableStyleOverride {
        override_type,
        ..Default::default()
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pPr" => cond.p_pr = Some(crate::parser::parse_paragraph_properties(reader)?),
                    "rPr" => cond.r_pr = Some(crate::parser::parse_run_properties(reader)?),
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tblStylePr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(cond)
}

// --- Serializer for word/styles.xml ---

pub fn serialize_stylesheet(sheet: &StyleSheet) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root_tag = BytesStart::new("w:styles");
    for (k, v) in &sheet.namespaces {
        root_tag.push_attribute((k.as_str(), v.as_str()));
    }
    writer
        .write_event(Event::Start(root_tag))
        .map_err(EngineError::Io)?;

    // 1. DocDefaults
    serialize_doc_defaults(&mut writer, &sheet.doc_defaults)?;

    // 2. Styles
    for style in sheet.styles.values() {
        serialize_style(&mut writer, style)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:styles")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

fn serialize_doc_defaults<W: std::io::Write>(
    writer: &mut Writer<W>,
    defaults: &DocDefaults,
) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("w:docDefaults")))
        .map_err(EngineError::Io)?;

    if let Some(ref r_pr) = defaults.r_pr_default {
        writer
            .write_event(Event::Start(BytesStart::new("w:rPrDefault")))
            .map_err(EngineError::Io)?;
        crate::serializer::serialize_run_properties(writer, r_pr)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:rPrDefault")))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref p_pr) = defaults.p_pr_default {
        writer
            .write_event(Event::Start(BytesStart::new("w:pPrDefault")))
            .map_err(EngineError::Io)?;
        crate::serializer::serialize_paragraph_properties(writer, p_pr)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:pPrDefault")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:docDefaults")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_style<W: std::io::Write>(writer: &mut Writer<W>, style: &Style) -> Result<()> {
    let mut style_tag = BytesStart::new("w:style");
    style_tag.push_attribute(("w:type", style.style_type.as_str()));
    style_tag.push_attribute(("w:styleId", style.style_id.as_str()));
    if style.is_default {
        style_tag.push_attribute(("w:default", "1"));
    }
    if style.is_custom {
        style_tag.push_attribute(("w:customStyle", "1"));
    }
    writer
        .write_event(Event::Start(style_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref name) = style.name {
        let mut t = BytesStart::new("w:name");
        t.push_attribute(("w:val", name.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref based) = style.based_on {
        let mut t = BytesStart::new("w:basedOn");
        t.push_attribute(("w:val", based.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref next) = style.next {
        let mut t = BytesStart::new("w:next");
        t.push_attribute(("w:val", next.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref link) = style.link {
        let mut t = BytesStart::new("w:link");
        t.push_attribute(("w:val", link.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }
    if style.q_format {
        writer
            .write_event(Event::Empty(BytesStart::new("w:qFormat")))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref p_pr) = style.p_pr {
        crate::serializer::serialize_paragraph_properties(writer, p_pr)?;
    }
    if let Some(ref r_pr) = style.r_pr {
        crate::serializer::serialize_run_properties(writer, r_pr)?;
    }

    for cond in &style.tbl_style_pr {
        if let Some(override_type) = cond.override_type {
            let mut tag = BytesStart::new("w:tblStylePr");
            tag.push_attribute(("w:type", override_type.as_str()));
            writer
                .write_event(Event::Start(tag))
                .map_err(EngineError::Io)?;
            if let Some(ref p_pr) = cond.p_pr {
                crate::serializer::serialize_paragraph_properties(writer, p_pr)?;
            }
            if let Some(ref r_pr) = cond.r_pr {
                crate::serializer::serialize_run_properties(writer, r_pr)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("w:tblStylePr")))
                .map_err(EngineError::Io)?;
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:style")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn local_name(qname: &[u8]) -> &str {
    if let Some(pos) = qname.iter().rposition(|&b| b == b':') {
        std::str::from_utf8(&qname[pos + 1..]).unwrap_or("")
    } else {
        std::str::from_utf8(qname).unwrap_or("")
    }
}

fn get_attr_val(e: &BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.into_inner();
        if local_name(key) == name {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}
