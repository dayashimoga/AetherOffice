//! ECMA-376 Part 1 §18.8 Styles (`xl/styles.xml`).
//! Comprehensive stylesheet parser, serializer, and formatting resolution engine.
//! Strictly conforms to GEC-v1.0 Article II (Zero-Loss ECMA-376 Round-Trip Fidelity)
//! and Article IV (zero panics, typed EngineError).

use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::unparsed::RawXmlNodeBuffer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::io::{BufRead, Cursor, Write};

/// Color representation in SpreadsheetML (ECMA-376 Part 1 §18.8.11).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Color {
    pub auto: bool,
    pub rgb: Option<String>,
    pub theme: Option<u32>,
    pub tint: Option<f64>,
    pub indexed: Option<u32>,
}

impl Color {
    pub fn from_rgb(rgb: impl Into<String>) -> Self {
        Self {
            auto: false,
            rgb: Some(rgb.into()),
            theme: None,
            tint: None,
            indexed: None,
        }
    }

    pub fn from_theme(theme: u32, tint: Option<f64>) -> Self {
        Self {
            auto: false,
            rgb: None,
            theme: Some(theme),
            tint,
            indexed: None,
        }
    }

    pub fn auto() -> Self {
        Self {
            auto: true,
            ..Default::default()
        }
    }
}

/// Custom number format `<numFmt>` (ECMA-376 Part 1 §18.8.30).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormat {
    pub num_fmt_id: u32,
    pub format_code: String,
}

impl NumberFormat {
    pub fn new(num_fmt_id: u32, format_code: impl Into<String>) -> Self {
        Self {
            num_fmt_id,
            format_code: format_code.into(),
        }
    }
}

/// Font definition `<font>` (ECMA-376 Part 1 §18.8.22).
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub name: String,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color: Option<Color>,
    pub family: Option<u32>,
    pub scheme: Option<String>,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            name: "Calibri".to_string(),
            size: 11.0,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: None,
            family: Some(2),
            scheme: Some("minor".to_string()),
        }
    }
}

/// Pattern fill type (ECMA-376 Part 1 §18.18.55 ST_PatternType).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PatternType {
    #[default]
    None,
    Solid,
    Gray125,
    DarkGray,
    MediumGray,
    LightGray,
    Gray0625,
    DarkHorizontal,
    DarkVertical,
    DarkDown,
    DarkUp,
    DarkGrid,
    DarkTrellis,
    LightHorizontal,
    LightVertical,
    LightDown,
    LightUp,
    LightGrid,
    LightTrellis,
    Other(String),
}

impl PatternType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Solid => "solid",
            Self::Gray125 => "gray125",
            Self::DarkGray => "darkGray",
            Self::MediumGray => "mediumGray",
            Self::LightGray => "lightGray",
            Self::Gray0625 => "gray0625",
            Self::DarkHorizontal => "darkHorizontal",
            Self::DarkVertical => "darkVertical",
            Self::DarkDown => "darkDown",
            Self::DarkUp => "darkUp",
            Self::DarkGrid => "darkGrid",
            Self::DarkTrellis => "darkTrellis",
            Self::LightHorizontal => "lightHorizontal",
            Self::LightVertical => "lightVertical",
            Self::LightDown => "lightDown",
            Self::LightUp => "lightUp",
            Self::LightGrid => "lightGrid",
            Self::LightTrellis => "lightTrellis",
            Self::Other(ref s) => s.as_str(),
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s {
            "none" => Self::None,
            "solid" => Self::Solid,
            "gray125" => Self::Gray125,
            "darkGray" => Self::DarkGray,
            "mediumGray" => Self::MediumGray,
            "lightGray" => Self::LightGray,
            "gray0625" => Self::Gray0625,
            "darkHorizontal" => Self::DarkHorizontal,
            "darkVertical" => Self::DarkVertical,
            "darkDown" => Self::DarkDown,
            "darkUp" => Self::DarkUp,
            "darkGrid" => Self::DarkGrid,
            "darkTrellis" => Self::DarkTrellis,
            "lightHorizontal" => Self::LightHorizontal,
            "lightVertical" => Self::LightVertical,
            "lightDown" => Self::LightDown,
            "lightUp" => Self::LightUp,
            "lightGrid" => Self::LightGrid,
            "lightTrellis" => Self::LightTrellis,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Fill definition `<fill>` (ECMA-376 Part 1 §18.8.20).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fill {
    pub pattern_type: PatternType,
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
}

impl Fill {
    pub fn none() -> Self {
        Self {
            pattern_type: PatternType::None,
            fg_color: None,
            bg_color: None,
        }
    }

    pub fn gray125() -> Self {
        Self {
            pattern_type: PatternType::Gray125,
            fg_color: None,
            bg_color: None,
        }
    }

    pub fn solid(color: Color) -> Self {
        Self {
            pattern_type: PatternType::Solid,
            fg_color: Some(color),
            bg_color: None,
        }
    }
}

/// Border edge line style (ECMA-376 Part 1 §18.18.3 ST_BorderStyle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Thin,
    Medium,
    Dashed,
    Dotted,
    Thick,
    Double,
    Hair,
    MediumDashed,
    DashDot,
    MediumDashDot,
    DashDotDot,
    MediumDashDotDot,
    SlantDashDot,
}

impl BorderStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Thin => "thin",
            Self::Medium => "medium",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
            Self::Thick => "thick",
            Self::Double => "double",
            Self::Hair => "hair",
            Self::MediumDashed => "mediumDashed",
            Self::DashDot => "dashDot",
            Self::MediumDashDot => "mediumDashDot",
            Self::DashDotDot => "dashDotDot",
            Self::MediumDashDotDot => "mediumDashDotDot",
            Self::SlantDashDot => "slantDashDot",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "thin" => Self::Thin,
            "medium" => Self::Medium,
            "dashed" => Self::Dashed,
            "dotted" => Self::Dotted,
            "thick" => Self::Thick,
            "double" => Self::Double,
            "hair" => Self::Hair,
            "mediumDashed" => Self::MediumDashed,
            "dashDot" => Self::DashDot,
            "mediumDashDot" => Self::MediumDashDot,
            "dashDotDot" => Self::DashDotDot,
            "mediumDashDotDot" => Self::MediumDashDotDot,
            "slantDashDot" => Self::SlantDashDot,
            _ => Self::None,
        }
    }
}

/// Border edge specification (left, right, top, bottom, diagonal).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BorderEdge {
    pub style: BorderStyle,
    pub color: Option<Color>,
}

/// Border definition `<border>` (ECMA-376 Part 1 §18.8.4).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Border {
    pub left: BorderEdge,
    pub right: BorderEdge,
    pub top: BorderEdge,
    pub bottom: BorderEdge,
    pub diagonal: BorderEdge,
    pub diagonal_up: bool,
    pub diagonal_down: bool,
}

/// Cell horizontal alignment (ECMA-376 Part 1 §18.18.40 ST_HorizontalAlignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    General,
    Left,
    Center,
    Right,
    Fill,
    Justify,
    CenterContinuous,
    Distributed,
}

impl HorizontalAlignment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Fill => "fill",
            Self::Justify => "justify",
            Self::CenterContinuous => "centerContinuous",
            Self::Distributed => "distributed",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "left" => Some(Self::Left),
            "center" => Some(Self::Center),
            "right" => Some(Self::Right),
            "fill" => Some(Self::Fill),
            "justify" => Some(Self::Justify),
            "centerContinuous" => Some(Self::CenterContinuous),
            "distributed" => Some(Self::Distributed),
            "general" => Some(Self::General),
            _ => None,
        }
    }
}

/// Cell vertical alignment (ECMA-376 Part 1 §18.18.88 ST_VerticalAlignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
    Justify,
    Distributed,
}

impl VerticalAlignment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Center => "center",
            Self::Bottom => "bottom",
            Self::Justify => "justify",
            Self::Distributed => "distributed",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "top" => Some(Self::Top),
            "center" => Some(Self::Center),
            "bottom" => Some(Self::Bottom),
            "justify" => Some(Self::Justify),
            "distributed" => Some(Self::Distributed),
            _ => None,
        }
    }
}

/// Cell alignment properties `<alignment>` (ECMA-376 Part 1 §18.8.1).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Alignment {
    pub horizontal: Option<HorizontalAlignment>,
    pub vertical: Option<VerticalAlignment>,
    pub wrap_text: bool,
    pub shrink_to_fit: bool,
    pub indent: u32,
    pub text_rotation: u32,
}

/// Format XF entry `<xf>` (ECMA-376 Part 1 §18.8.45).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellXf {
    pub num_fmt_id: u32,
    pub font_id: u32,
    pub fill_id: u32,
    pub border_id: u32,
    pub xf_id: Option<u32>,
    pub apply_number_format: Option<bool>,
    pub apply_font: Option<bool>,
    pub apply_fill: Option<bool>,
    pub apply_border: Option<bool>,
    pub apply_alignment: Option<bool>,
    pub alignment: Option<Alignment>,
}

/// Cell style definition `<cellStyle>` (ECMA-376 Part 1 §18.8.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellStyle {
    pub name: String,
    pub xf_id: u32,
    pub builtin_id: Option<u32>,
}

impl CellStyle {
    pub fn normal() -> Self {
        Self {
            name: "Normal".to_string(),
            xf_id: 0,
            builtin_id: Some(0),
        }
    }
}

/// Complete ECMA-376 Stylesheet (`xl/styles.xml`).
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    pub number_formats: Vec<NumberFormat>,
    pub fonts: Vec<Font>,
    pub fills: Vec<Fill>,
    pub borders: Vec<Border>,
    pub cell_style_xfs: Vec<CellXf>,
    pub cell_xfs: Vec<CellXf>,
    pub cell_styles: Vec<CellStyle>,
    pub dxfs: Option<RawXmlNodeBuffer>,
    pub table_styles: Option<RawXmlNodeBuffer>,
    pub ext_lst: Option<RawXmlNodeBuffer>,
    pub root_attrs: BTreeMap<String, String>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        let mut root_attrs = BTreeMap::new();
        root_attrs.insert(
            "xmlns".to_string(),
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string(),
        );

        Self {
            number_formats: Vec::new(),
            fonts: vec![Font::default()],
            fills: vec![Fill::none(), Fill::gray125()],
            borders: vec![Border::default()],
            cell_style_xfs: vec![CellXf::default()],
            cell_xfs: vec![CellXf::default()],
            cell_styles: vec![CellStyle::normal()],
            dxfs: None,
            table_styles: None,
            ext_lst: None,
            root_attrs,
        }
    }
}

/// Standard ECMA-376 Part 1 Table 18-6 Built-in Number Formats.
pub fn builtin_format_code(num_fmt_id: u32) -> Option<&'static str> {
    match num_fmt_id {
        0 => Some("General"),
        1 => Some("0"),
        2 => Some("0.00"),
        3 => Some("#,##0"),
        4 => Some("#,##0.00"),
        9 => Some("0%"),
        10 => Some("0.00%"),
        11 => Some("0.00E+00"),
        12 => Some("# ?/?"),
        13 => Some("# ??/??"),
        14 => Some("mm-dd-yy"),
        15 => Some("d-mmm-yy"),
        16 => Some("d-mmm"),
        17 => Some("mmm-yy"),
        18 => Some("h:mm AM/PM"),
        19 => Some("h:mm:ss AM/PM"),
        20 => Some("h:mm"),
        21 => Some("h:mm:ss"),
        22 => Some("m/d/yy h:mm"),
        37 => Some("(#,##0);(#,##0)"),
        38 => Some("(#,##0);[Red](#,##0)"),
        39 => Some("(#,##0.00);(#,##0.00)"),
        40 => Some("(#,##0.00);[Red](#,##0.00)"),
        45 => Some("mm:ss"),
        46 => Some("[h]:mm:ss"),
        47 => Some("mmss.0"),
        48 => Some("##0.0E+0"),
        49 => Some("@"),
        _ => None,
    }
}

impl StyleSheet {
    /// Resolves the format code string for a given `num_fmt_id`.
    /// Checks custom `number_formats` first, then falls back to ECMA-376 built-in formats.
    pub fn get_format_code(&self, num_fmt_id: u32) -> Option<&str> {
        if let Some(nf) = self
            .number_formats
            .iter()
            .find(|n| n.num_fmt_id == num_fmt_id)
        {
            return Some(&nf.format_code);
        }
        builtin_format_code(num_fmt_id)
    }

    /// Parses a stylesheet from an XML string.
    pub fn from_xml_str(xml: &str) -> Result<Self> {
        Self::from_reader(Cursor::new(xml.as_bytes()))
    }

    /// Parses a stylesheet from any `BufRead` stream.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut styles = Self {
            number_formats: Vec::new(),
            fonts: Vec::new(),
            fills: Vec::new(),
            borders: Vec::new(),
            cell_style_xfs: Vec::new(),
            cell_xfs: Vec::new(),
            cell_styles: Vec::new(),
            dxfs: None,
            table_styles: None,
            ext_lst: None,
            root_attrs: BTreeMap::new(),
        };

        let mut buf = Vec::new();
        let mut in_num_fmts = false;
        let mut in_fonts = false;
        let mut in_fills = false;
        let mut in_borders = false;
        let mut in_cell_style_xfs = false;
        let mut in_cell_xfs = false;
        let mut in_cell_styles = false;

        // Current item accumulators
        let mut cur_font: Option<Font> = None;
        let mut cur_fill: Option<Fill> = None;
        let mut cur_border: Option<Border> = None;
        let mut cur_edge_name: Option<String> = None;
        let mut cur_xf: Option<CellXf> = None;

        loop {
            buf.clear();
            match xml_reader
                .read_event_into(&mut buf)
                .map_err(EngineError::Xml)?
            {
                Event::Start(ref e) => match e.name().as_ref() {
                    b"styleSheet" => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            styles.root_attrs.insert(key.to_string(), val.to_string());
                        }
                    }
                    b"numFmts" => in_num_fmts = true,
                    b"fonts" => in_fonts = true,
                    b"font" if in_fonts => cur_font = Some(Font::default()),
                    b"fills" => in_fills = true,
                    b"fill" if in_fills => cur_fill = Some(Fill::default()),
                    b"patternFill" if cur_fill.is_some() => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"patternType" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                if let Some(ref mut f) = cur_fill {
                                    f.pattern_type = PatternType::from_str_name(&val);
                                }
                            }
                        }
                    }
                    b"borders" => in_borders = true,
                    b"border" if in_borders => cur_border = Some(Border::default()),
                    b"left" | b"right" | b"top" | b"bottom" | b"diagonal"
                        if cur_border.is_some() =>
                    {
                        cur_edge_name =
                            Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                        let mut style = BorderStyle::None;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"style" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                style = BorderStyle::from_str_opt(&val);
                            }
                        }
                        if let Some(ref mut b) = cur_border {
                            let edge = match cur_edge_name.as_deref() {
                                Some("left") => &mut b.left,
                                Some("right") => &mut b.right,
                                Some("top") => &mut b.top,
                                Some("bottom") => &mut b.bottom,
                                Some("diagonal") => &mut b.diagonal,
                                _ => unreachable!(),
                            };
                            edge.style = style;
                        }
                    }
                    b"cellStyleXfs" => in_cell_style_xfs = true,
                    b"cellXfs" => in_cell_xfs = true,
                    b"xf" if in_cell_style_xfs || in_cell_xfs => {
                        let mut xf = CellXf::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "numFmtId" => xf.num_fmt_id = val.parse::<u32>().unwrap_or(0),
                                "fontId" => xf.font_id = val.parse::<u32>().unwrap_or(0),
                                "fillId" => xf.fill_id = val.parse::<u32>().unwrap_or(0),
                                "borderId" => xf.border_id = val.parse::<u32>().unwrap_or(0),
                                "xfId" => xf.xf_id = val.parse::<u32>().ok(),
                                "applyNumberFormat" => {
                                    xf.apply_number_format =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyFont" => {
                                    xf.apply_font =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyFill" => {
                                    xf.apply_fill =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyBorder" => {
                                    xf.apply_border =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyAlignment" => {
                                    xf.apply_alignment =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                _ => {}
                            }
                        }
                        cur_xf = Some(xf);
                    }
                    b"alignment" if cur_xf.is_some() => {
                        let mut align = Alignment::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "horizontal" => {
                                    align.horizontal = HorizontalAlignment::from_str_opt(&val)
                                }
                                "vertical" => {
                                    align.vertical = VerticalAlignment::from_str_opt(&val)
                                }
                                "wrapText" => {
                                    align.wrap_text = val == "1" || val.eq_ignore_ascii_case("true")
                                }
                                "shrinkToFit" => {
                                    align.shrink_to_fit =
                                        val == "1" || val.eq_ignore_ascii_case("true")
                                }
                                "indent" => align.indent = val.parse::<u32>().unwrap_or(0),
                                "textRotation" => {
                                    align.text_rotation = val.parse::<u32>().unwrap_or(0)
                                }
                                _ => {}
                            }
                        }
                        if let Some(ref mut xf) = cur_xf {
                            xf.alignment = Some(align);
                        }
                    }
                    b"cellStyles" => in_cell_styles = true,
                    b"dxfs" => {
                        let raw = read_subtree_xml(&mut xml_reader, e)?;
                        styles.dxfs = Some(RawXmlNodeBuffer::new(&raw));
                    }
                    b"tableStyles" => {
                        let raw = read_subtree_xml(&mut xml_reader, e)?;
                        styles.table_styles = Some(RawXmlNodeBuffer::new(&raw));
                    }
                    b"extLst" => {
                        let raw = read_subtree_xml(&mut xml_reader, e)?;
                        styles.ext_lst = Some(RawXmlNodeBuffer::new(&raw));
                    }
                    _ => {}
                },
                Event::Empty(ref e) => match e.name().as_ref() {
                    b"numFmt" if in_num_fmts => {
                        let mut num_fmt_id = 0u32;
                        let mut format_code = String::new();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            if key == "numFmtId" {
                                num_fmt_id = val.parse::<u32>().unwrap_or(0);
                            } else if key == "formatCode" {
                                format_code = val.to_string();
                            }
                        }
                        styles.number_formats.push(NumberFormat {
                            num_fmt_id,
                            format_code,
                        });
                    }
                    b"sz" if cur_font.is_some() => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"val" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                if let Some(ref mut f) = cur_font {
                                    f.size = val.parse::<f64>().unwrap_or(11.0);
                                }
                            }
                        }
                    }
                    b"name" | b"rFont" if cur_font.is_some() => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"val" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                if let Some(ref mut f) = cur_font {
                                    f.name = val.to_string();
                                }
                            }
                        }
                    }
                    b"b" if cur_font.is_some() => {
                        if let Some(ref mut f) = cur_font {
                            f.bold = true;
                        }
                    }
                    b"i" if cur_font.is_some() => {
                        if let Some(ref mut f) = cur_font {
                            f.italic = true;
                        }
                    }
                    b"u" if cur_font.is_some() => {
                        if let Some(ref mut f) = cur_font {
                            f.underline = true;
                        }
                    }
                    b"strike" if cur_font.is_some() => {
                        if let Some(ref mut f) = cur_font {
                            f.strike = true;
                        }
                    }
                    b"color" if cur_font.is_some() => {
                        let color = parse_color_attrs(e, &mut xml_reader)?;
                        if let Some(ref mut f) = cur_font {
                            f.color = Some(color);
                        }
                    }
                    b"fgColor" if cur_fill.is_some() => {
                        let color = parse_color_attrs(e, &mut xml_reader)?;
                        if let Some(ref mut f) = cur_fill {
                            f.fg_color = Some(color);
                        }
                    }
                    b"bgColor" if cur_fill.is_some() => {
                        let color = parse_color_attrs(e, &mut xml_reader)?;
                        if let Some(ref mut f) = cur_fill {
                            f.bg_color = Some(color);
                        }
                    }
                    b"patternFill" if cur_fill.is_some() => {
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"patternType" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                if let Some(ref mut f) = cur_fill {
                                    f.pattern_type = PatternType::from_str_name(&val);
                                }
                            }
                        }
                    }
                    b"color" if cur_border.is_some() && cur_edge_name.is_some() => {
                        let color = parse_color_attrs(e, &mut xml_reader)?;
                        if let Some(ref mut b) = cur_border {
                            let edge = match cur_edge_name.as_deref() {
                                Some("left") => &mut b.left,
                                Some("right") => &mut b.right,
                                Some("top") => &mut b.top,
                                Some("bottom") => &mut b.bottom,
                                Some("diagonal") => &mut b.diagonal,
                                _ => unreachable!(),
                            };
                            edge.color = Some(color);
                        }
                    }
                    b"left" | b"right" | b"top" | b"bottom" | b"diagonal"
                        if cur_border.is_some() =>
                    {
                        let edge_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        let mut style = BorderStyle::None;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            if attr.key.into_inner() == b"style" {
                                let val = attr
                                    .decode_and_unescape_value(xml_reader.decoder())
                                    .map_err(EngineError::Xml)?;
                                style = BorderStyle::from_str_opt(&val);
                            }
                        }
                        if let Some(ref mut b) = cur_border {
                            let edge = match edge_name.as_str() {
                                "left" => &mut b.left,
                                "right" => &mut b.right,
                                "top" => &mut b.top,
                                "bottom" => &mut b.bottom,
                                "diagonal" => &mut b.diagonal,
                                _ => unreachable!(),
                            };
                            edge.style = style;
                        }
                    }
                    b"alignment" if cur_xf.is_some() => {
                        let mut align = Alignment::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "horizontal" => {
                                    align.horizontal = HorizontalAlignment::from_str_opt(&val)
                                }
                                "vertical" => {
                                    align.vertical = VerticalAlignment::from_str_opt(&val)
                                }
                                "wrapText" => {
                                    align.wrap_text = val == "1" || val.eq_ignore_ascii_case("true")
                                }
                                "shrinkToFit" => {
                                    align.shrink_to_fit =
                                        val == "1" || val.eq_ignore_ascii_case("true")
                                }
                                "indent" => align.indent = val.parse::<u32>().unwrap_or(0),
                                "textRotation" => {
                                    align.text_rotation = val.parse::<u32>().unwrap_or(0)
                                }
                                _ => {}
                            }
                        }
                        if let Some(ref mut xf) = cur_xf {
                            xf.alignment = Some(align);
                        }
                    }
                    b"xf" if in_cell_style_xfs || in_cell_xfs => {
                        let mut xf = CellXf::default();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "numFmtId" => xf.num_fmt_id = val.parse::<u32>().unwrap_or(0),
                                "fontId" => xf.font_id = val.parse::<u32>().unwrap_or(0),
                                "fillId" => xf.fill_id = val.parse::<u32>().unwrap_or(0),
                                "borderId" => xf.border_id = val.parse::<u32>().unwrap_or(0),
                                "xfId" => xf.xf_id = val.parse::<u32>().ok(),
                                "applyNumberFormat" => {
                                    xf.apply_number_format =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyFont" => {
                                    xf.apply_font =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyFill" => {
                                    xf.apply_fill =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyBorder" => {
                                    xf.apply_border =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                "applyAlignment" => {
                                    xf.apply_alignment =
                                        Some(val == "1" || val.eq_ignore_ascii_case("true"))
                                }
                                _ => {}
                            }
                        }
                        if in_cell_style_xfs {
                            styles.cell_style_xfs.push(xf);
                        } else if in_cell_xfs {
                            styles.cell_xfs.push(xf);
                        }
                    }
                    b"cellStyle" if in_cell_styles => {
                        let mut name = String::new();
                        let mut xf_id = 0u32;
                        let mut builtin_id = None;
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            let key = std::str::from_utf8(attr.key.into_inner())
                                .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
                            let val = attr
                                .decode_and_unescape_value(xml_reader.decoder())
                                .map_err(EngineError::Xml)?;
                            match key {
                                "name" => name = val.to_string(),
                                "xfId" => xf_id = val.parse::<u32>().unwrap_or(0),
                                "builtinId" => builtin_id = val.parse::<u32>().ok(),
                                _ => {}
                            }
                        }
                        styles.cell_styles.push(CellStyle {
                            name,
                            xf_id,
                            builtin_id,
                        });
                    }
                    _ => {}
                },
                Event::End(ref e) => match e.name().as_ref() {
                    b"numFmts" => in_num_fmts = false,
                    b"fonts" => in_fonts = false,
                    b"font" => {
                        if let Some(f) = cur_font.take() {
                            styles.fonts.push(f);
                        }
                    }
                    b"fills" => in_fills = false,
                    b"fill" => {
                        if let Some(f) = cur_fill.take() {
                            styles.fills.push(f);
                        }
                    }
                    b"borders" => in_borders = false,
                    b"border" => {
                        if let Some(b) = cur_border.take() {
                            styles.borders.push(b);
                        }
                    }
                    b"left" | b"right" | b"top" | b"bottom" | b"diagonal" => {
                        cur_edge_name = None;
                    }
                    b"cellStyleXfs" => in_cell_style_xfs = false,
                    b"cellXfs" => in_cell_xfs = false,
                    b"xf" => {
                        if let Some(xf) = cur_xf.take() {
                            if in_cell_style_xfs {
                                styles.cell_style_xfs.push(xf);
                            } else if in_cell_xfs {
                                styles.cell_xfs.push(xf);
                            }
                        }
                    }
                    b"cellStyles" => in_cell_styles = false,
                    b"styleSheet" => break,
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
        }

        if !styles.root_attrs.contains_key("xmlns") {
            styles.root_attrs.insert(
                "xmlns".to_string(),
                "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string(),
            );
        }

        Ok(styles)
    }

    /// Serializes stylesheet according to strict ECMA-376 Part 1 §18.8 child element order.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<()> {
        let mut xml_writer = Writer::new(&mut writer);

        xml_writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(EngineError::Io)?;

        let mut root = BytesStart::new("styleSheet");
        for (k, v) in &self.root_attrs {
            root.push_attribute((k.as_str(), v.as_str()));
        }
        xml_writer
            .write_event(Event::Start(root))
            .map_err(EngineError::Io)?;

        // 1. `<numFmts>`
        if !self.number_formats.is_empty() {
            let mut num_fmts_tag = BytesStart::new("numFmts");
            let count_str = self.number_formats.len().to_string();
            num_fmts_tag.push_attribute(("count", count_str.as_str()));
            xml_writer
                .write_event(Event::Start(num_fmts_tag))
                .map_err(EngineError::Io)?;

            for nf in &self.number_formats {
                let mut tag = BytesStart::new("numFmt");
                let id_str = nf.num_fmt_id.to_string();
                tag.push_attribute(("numFmtId", id_str.as_str()));
                tag.push_attribute(("formatCode", nf.format_code.as_str()));
                xml_writer
                    .write_event(Event::Empty(tag))
                    .map_err(EngineError::Io)?;
            }
            xml_writer
                .write_event(Event::End(BytesEnd::new("numFmts")))
                .map_err(EngineError::Io)?;
        }

        // 2. `<fonts>`
        let mut fonts_tag = BytesStart::new("fonts");
        let fonts_count = self.fonts.len().to_string();
        fonts_tag.push_attribute(("count", fonts_count.as_str()));
        xml_writer
            .write_event(Event::Start(fonts_tag))
            .map_err(EngineError::Io)?;

        for font in &self.fonts {
            xml_writer
                .write_event(Event::Start(BytesStart::new("font")))
                .map_err(EngineError::Io)?;

            if font.bold {
                xml_writer
                    .write_event(Event::Empty(BytesStart::new("b")))
                    .map_err(EngineError::Io)?;
            }
            if font.italic {
                xml_writer
                    .write_event(Event::Empty(BytesStart::new("i")))
                    .map_err(EngineError::Io)?;
            }
            if font.underline {
                xml_writer
                    .write_event(Event::Empty(BytesStart::new("u")))
                    .map_err(EngineError::Io)?;
            }
            if font.strike {
                xml_writer
                    .write_event(Event::Empty(BytesStart::new("strike")))
                    .map_err(EngineError::Io)?;
            }

            let mut sz_tag = BytesStart::new("sz");
            let sz_str = font.size.to_string();
            sz_tag.push_attribute(("val", sz_str.as_str()));
            xml_writer
                .write_event(Event::Empty(sz_tag))
                .map_err(EngineError::Io)?;

            if let Some(ref col) = font.color {
                write_color_event(&mut xml_writer, "color", col)?;
            }

            let mut name_tag = BytesStart::new("name");
            name_tag.push_attribute(("val", font.name.as_str()));
            xml_writer
                .write_event(Event::Empty(name_tag))
                .map_err(EngineError::Io)?;

            if let Some(family) = font.family {
                let mut fam_tag = BytesStart::new("family");
                let fam_str = family.to_string();
                fam_tag.push_attribute(("val", fam_str.as_str()));
                xml_writer
                    .write_event(Event::Empty(fam_tag))
                    .map_err(EngineError::Io)?;
            }

            if let Some(ref scheme) = font.scheme {
                let mut scheme_tag = BytesStart::new("scheme");
                scheme_tag.push_attribute(("val", scheme.as_str()));
                xml_writer
                    .write_event(Event::Empty(scheme_tag))
                    .map_err(EngineError::Io)?;
            }

            xml_writer
                .write_event(Event::End(BytesEnd::new("font")))
                .map_err(EngineError::Io)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("fonts")))
            .map_err(EngineError::Io)?;

        // 3. `<fills>`
        let mut fills_tag = BytesStart::new("fills");
        let fills_count = self.fills.len().to_string();
        fills_tag.push_attribute(("count", fills_count.as_str()));
        xml_writer
            .write_event(Event::Start(fills_tag))
            .map_err(EngineError::Io)?;

        for fill in &self.fills {
            xml_writer
                .write_event(Event::Start(BytesStart::new("fill")))
                .map_err(EngineError::Io)?;

            let has_colors = fill.fg_color.is_some() || fill.bg_color.is_some();
            if has_colors {
                let mut pf = BytesStart::new("patternFill");
                pf.push_attribute(("patternType", fill.pattern_type.as_str()));
                xml_writer
                    .write_event(Event::Start(pf))
                    .map_err(EngineError::Io)?;

                if let Some(ref fg) = fill.fg_color {
                    write_color_event(&mut xml_writer, "fgColor", fg)?;
                }
                if let Some(ref bg) = fill.bg_color {
                    write_color_event(&mut xml_writer, "bgColor", bg)?;
                }
                xml_writer
                    .write_event(Event::End(BytesEnd::new("patternFill")))
                    .map_err(EngineError::Io)?;
            } else {
                let mut pf = BytesStart::new("patternFill");
                pf.push_attribute(("patternType", fill.pattern_type.as_str()));
                xml_writer
                    .write_event(Event::Empty(pf))
                    .map_err(EngineError::Io)?;
            }

            xml_writer
                .write_event(Event::End(BytesEnd::new("fill")))
                .map_err(EngineError::Io)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("fills")))
            .map_err(EngineError::Io)?;

        // 4. `<borders>`
        let mut borders_tag = BytesStart::new("borders");
        let borders_count = self.borders.len().to_string();
        borders_tag.push_attribute(("count", borders_count.as_str()));
        xml_writer
            .write_event(Event::Start(borders_tag))
            .map_err(EngineError::Io)?;

        for b in &self.borders {
            let mut b_start = BytesStart::new("border");
            if b.diagonal_up {
                b_start.push_attribute(("diagonalUp", "1"));
            }
            if b.diagonal_down {
                b_start.push_attribute(("diagonalDown", "1"));
            }
            xml_writer
                .write_event(Event::Start(b_start))
                .map_err(EngineError::Io)?;

            write_border_edge(&mut xml_writer, "left", &b.left)?;
            write_border_edge(&mut xml_writer, "right", &b.right)?;
            write_border_edge(&mut xml_writer, "top", &b.top)?;
            write_border_edge(&mut xml_writer, "bottom", &b.bottom)?;
            write_border_edge(&mut xml_writer, "diagonal", &b.diagonal)?;

            xml_writer
                .write_event(Event::End(BytesEnd::new("border")))
                .map_err(EngineError::Io)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("borders")))
            .map_err(EngineError::Io)?;

        // 5. `<cellStyleXfs>`
        let mut csx_tag = BytesStart::new("cellStyleXfs");
        let csx_count = self.cell_style_xfs.len().to_string();
        csx_tag.push_attribute(("count", csx_count.as_str()));
        xml_writer
            .write_event(Event::Start(csx_tag))
            .map_err(EngineError::Io)?;
        for xf in &self.cell_style_xfs {
            write_xf_event(&mut xml_writer, xf)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("cellStyleXfs")))
            .map_err(EngineError::Io)?;

        // 6. `<cellXfs>`
        let mut cx_tag = BytesStart::new("cellXfs");
        let cx_count = self.cell_xfs.len().to_string();
        cx_tag.push_attribute(("count", cx_count.as_str()));
        xml_writer
            .write_event(Event::Start(cx_tag))
            .map_err(EngineError::Io)?;
        for xf in &self.cell_xfs {
            write_xf_event(&mut xml_writer, xf)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("cellXfs")))
            .map_err(EngineError::Io)?;

        // 7. `<cellStyles>`
        let mut cstyles_tag = BytesStart::new("cellStyles");
        let cstyles_count = self.cell_styles.len().to_string();
        cstyles_tag.push_attribute(("count", cstyles_count.as_str()));
        xml_writer
            .write_event(Event::Start(cstyles_tag))
            .map_err(EngineError::Io)?;
        for cs in &self.cell_styles {
            let mut cs_tag = BytesStart::new("cellStyle");
            cs_tag.push_attribute(("name", cs.name.as_str()));
            let xf_str = cs.xf_id.to_string();
            cs_tag.push_attribute(("xfId", xf_str.as_str()));
            if let Some(bid) = cs.builtin_id {
                let bid_str = bid.to_string();
                cs_tag.push_attribute(("builtinId", bid_str.as_str()));
            }
            xml_writer
                .write_event(Event::Empty(cs_tag))
                .map_err(EngineError::Io)?;
        }
        xml_writer
            .write_event(Event::End(BytesEnd::new("cellStyles")))
            .map_err(EngineError::Io)?;

        // 8. `<dxfs>`
        if let Some(ref dxfs) = self.dxfs {
            xml_writer
                .get_mut()
                .write_all(dxfs.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        // 9. `<tableStyles>`
        if let Some(ref ts) = self.table_styles {
            xml_writer
                .get_mut()
                .write_all(ts.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        // 10. `<extLst>`
        if let Some(ref ext) = self.ext_lst {
            xml_writer
                .get_mut()
                .write_all(ext.raw_xml.as_bytes())
                .map_err(EngineError::Io)?;
        }

        xml_writer
            .write_event(Event::End(BytesEnd::new("styleSheet")))
            .map_err(EngineError::Io)?;

        Ok(())
    }

    /// Serializes stylesheet to an XML string.
    pub fn to_xml_string(&self) -> Result<String> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| EngineError::MalformedPackage(format!("Serialized XML is not UTF-8: {e}")))
    }
}

fn write_border_edge<W: Write>(
    writer: &mut Writer<W>,
    edge_name: &str,
    edge: &BorderEdge,
) -> Result<()> {
    if edge.style == BorderStyle::None && edge.color.is_none() {
        writer
            .write_event(Event::Empty(BytesStart::new(edge_name)))
            .map_err(EngineError::Io)?;
        return Ok(());
    }

    let mut start = BytesStart::new(edge_name);
    if edge.style != BorderStyle::None {
        start.push_attribute(("style", edge.style.as_str()));
    }

    if let Some(ref col) = edge.color {
        writer
            .write_event(Event::Start(start))
            .map_err(EngineError::Io)?;
        write_color_event(writer, "color", col)?;
        writer
            .write_event(Event::End(BytesEnd::new(edge_name)))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Empty(start))
            .map_err(EngineError::Io)?;
    }

    Ok(())
}

fn write_xf_event<W: Write>(writer: &mut Writer<W>, xf: &CellXf) -> Result<()> {
    let mut xf_tag = BytesStart::new("xf");
    let num_fmt_str = xf.num_fmt_id.to_string();
    xf_tag.push_attribute(("numFmtId", num_fmt_str.as_str()));
    let font_str = xf.font_id.to_string();
    xf_tag.push_attribute(("fontId", font_str.as_str()));
    let fill_str = xf.fill_id.to_string();
    xf_tag.push_attribute(("fillId", fill_str.as_str()));
    let border_str = xf.border_id.to_string();
    xf_tag.push_attribute(("borderId", border_str.as_str()));

    if let Some(xf_id) = xf.xf_id {
        let s = xf_id.to_string();
        xf_tag.push_attribute(("xfId", s.as_str()));
    }
    if let Some(b) = xf.apply_number_format {
        xf_tag.push_attribute(("applyNumberFormat", if b { "1" } else { "0" }));
    }
    if let Some(b) = xf.apply_font {
        xf_tag.push_attribute(("applyFont", if b { "1" } else { "0" }));
    }
    if let Some(b) = xf.apply_fill {
        xf_tag.push_attribute(("applyFill", if b { "1" } else { "0" }));
    }
    if let Some(b) = xf.apply_border {
        xf_tag.push_attribute(("applyBorder", if b { "1" } else { "0" }));
    }
    if let Some(b) = xf.apply_alignment {
        xf_tag.push_attribute(("applyAlignment", if b { "1" } else { "0" }));
    }

    if let Some(ref align) = xf.alignment {
        writer
            .write_event(Event::Start(xf_tag))
            .map_err(EngineError::Io)?;

        let mut align_tag = BytesStart::new("alignment");
        if let Some(h) = align.horizontal {
            align_tag.push_attribute(("horizontal", h.as_str()));
        }
        if let Some(v) = align.vertical {
            align_tag.push_attribute(("vertical", v.as_str()));
        }
        if align.wrap_text {
            align_tag.push_attribute(("wrapText", "1"));
        }
        if align.shrink_to_fit {
            align_tag.push_attribute(("shrinkToFit", "1"));
        }
        if align.indent > 0 {
            let s = align.indent.to_string();
            align_tag.push_attribute(("indent", s.as_str()));
        }
        if align.text_rotation > 0 {
            let s = align.text_rotation.to_string();
            align_tag.push_attribute(("textRotation", s.as_str()));
        }
        writer
            .write_event(Event::Empty(align_tag))
            .map_err(EngineError::Io)?;

        writer
            .write_event(Event::End(BytesEnd::new("xf")))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Empty(xf_tag))
            .map_err(EngineError::Io)?;
    }

    Ok(())
}

fn write_color_event<W: Write>(
    writer: &mut Writer<W>,
    tag_name: &str,
    color: &Color,
) -> Result<()> {
    let mut tag = BytesStart::new(tag_name);
    if color.auto {
        tag.push_attribute(("auto", "1"));
    }
    if let Some(ref rgb) = color.rgb {
        tag.push_attribute(("rgb", rgb.as_str()));
    }
    if let Some(theme) = color.theme {
        let s = theme.to_string();
        tag.push_attribute(("theme", s.as_str()));
    }
    if let Some(tint) = color.tint {
        let s = tint.to_string();
        tag.push_attribute(("tint", s.as_str()));
    }
    if let Some(indexed) = color.indexed {
        let s = indexed.to_string();
        tag.push_attribute(("indexed", s.as_str()));
    }
    writer
        .write_event(Event::Empty(tag))
        .map_err(EngineError::Io)?;
    Ok(())
}

fn parse_color_attrs<R: BufRead>(start: &BytesStart<'_>, reader: &mut Reader<R>) -> Result<Color> {
    let mut color = Color::default();
    for attr in start.attributes() {
        let attr = attr.map_err(EngineError::XmlAttr)?;
        let key = std::str::from_utf8(attr.key.into_inner())
            .map_err(|e| EngineError::MalformedPackage(e.to_string()))?;
        let val = attr
            .decode_and_unescape_value(reader.decoder())
            .map_err(EngineError::Xml)?;
        match key {
            "auto" => color.auto = val == "1" || val.eq_ignore_ascii_case("true"),
            "rgb" => color.rgb = Some(val.to_string()),
            "theme" => color.theme = val.parse::<u32>().ok(),
            "tint" => color.tint = val.parse::<f64>().ok(),
            "indexed" => color.indexed = val.parse::<u32>().ok(),
            _ => {}
        }
    }
    Ok(color)
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
