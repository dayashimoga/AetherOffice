//! OpenOfficeRS Word Engine - ECMA-376 Part 1 §17.9 Multi-Level Numbering Engine.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Models `word/numbering.xml`, manages abstract numbers (`w:abstractNum`), instances (`w:num`),
//! level overrides (`w:lvlOverride`), restart rules (`w:lvlRestart`), and produces formatted strings (1.a.i, Roman, bullets).

use crate::model::*;
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;

/// Numbering format type as defined in ECMA-376 Part 1 §17.18.59 (`w:numFmt`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NumberFormat {
    #[default]
    Decimal,
    LowerLetter,
    UpperLetter,
    LowerRoman,
    UpperRoman,
    Bullet,
    Ordinal,
    CardinalText,
    None,
}

impl NumberFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Decimal => "decimal",
            Self::LowerLetter => "lowerLetter",
            Self::UpperLetter => "upperLetter",
            Self::LowerRoman => "lowerRoman",
            Self::UpperRoman => "upperRoman",
            Self::Bullet => "bullet",
            Self::Ordinal => "ordinal",
            Self::CardinalText => "cardinalText",
            Self::None => "none",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "lowerLetter" => Self::LowerLetter,
            "upperLetter" => Self::UpperLetter,
            "lowerRoman" => Self::LowerRoman,
            "upperRoman" => Self::UpperRoman,
            "bullet" => Self::Bullet,
            "ordinal" => Self::Ordinal,
            "cardinalText" => Self::CardinalText,
            "none" => Self::None,
            _ => Self::Decimal,
        }
    }

    /// Formats a 1-based integer according to this numbering format.
    pub fn format_number(&self, val: u32) -> String {
        match self {
            Self::Decimal => val.to_string(),
            Self::LowerLetter => to_letter(val, false),
            Self::UpperLetter => to_letter(val, true),
            Self::LowerRoman => to_roman(val, false),
            Self::UpperRoman => to_roman(val, true),
            Self::Bullet => "\u{2022}".to_string(),
            Self::Ordinal => to_ordinal(val),
            Self::CardinalText => to_cardinal_text(val),
            Self::None => String::new(),
        }
    }
}

/// A single numbering level definition: `<w:lvl>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Level {
    pub ilvl: u32,
    pub start: u32,
    pub num_fmt: NumberFormat,
    pub lvl_restart: Option<u32>,
    pub p_style: Option<String>,
    pub is_lgl: bool,
    pub lvl_text: String,
    pub lvl_jc: Option<String>,
    pub p_pr: Option<ParagraphProperties>,
    pub r_pr: Option<RunProperties>,
}

impl Level {
    pub fn new(ilvl: u32, num_fmt: NumberFormat, lvl_text: impl Into<String>) -> Self {
        Self {
            ilvl,
            start: 1,
            num_fmt,
            lvl_text: lvl_text.into(),
            lvl_jc: Some("left".to_string()),
            ..Default::default()
        }
    }
}

/// Abstract numbering template: `<w:abstractNum>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AbstractNum {
    pub abstract_num_id: u32,
    pub multi_level_type: Option<String>,
    pub name: Option<String>,
    pub levels: BTreeMap<u32, Level>,
}

impl AbstractNum {
    pub fn new(abstract_num_id: u32) -> Self {
        Self {
            abstract_num_id,
            multi_level_type: Some("multilevel".to_string()),
            name: None,
            levels: BTreeMap::new(),
        }
    }

    pub fn get_level(&self, ilvl: u32) -> Option<&Level> {
        self.levels.get(&ilvl)
    }

    pub fn add_level(&mut self, level: Level) {
        self.levels.insert(level.ilvl, level);
    }
}

/// Level override definition: `<w:lvlOverride>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelOverride {
    pub ilvl: u32,
    pub start_override: Option<u32>,
    pub lvl: Option<Level>,
}

/// Numbering instance referencing an abstract template: `<w:num>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Num {
    pub num_id: u32,
    pub abstract_num_id: u32,
    pub lvl_overrides: BTreeMap<u32, LevelOverride>,
}

impl Num {
    pub fn new(num_id: u32, abstract_num_id: u32) -> Self {
        Self {
            num_id,
            abstract_num_id,
            lvl_overrides: BTreeMap::new(),
        }
    }
}

/// Entire `word/numbering.xml` part container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Numbering {
    pub namespaces: Vec<(String, String)>,
    pub abstract_nums: BTreeMap<u32, AbstractNum>,
    pub nums: BTreeMap<u32, Num>,
}

impl Numbering {
    pub fn new() -> Self {
        Self {
            namespaces: vec![(
                "xmlns:w".to_string(),
                "http://schemas.openxmlformats.org/wordprocessingml/2006/main".to_string(),
            )],
            abstract_nums: BTreeMap::new(),
            nums: BTreeMap::new(),
        }
    }

    pub fn add_abstract_num(&mut self, abs: AbstractNum) {
        self.abstract_nums.insert(abs.abstract_num_id, abs);
    }

    pub fn add_num(&mut self, num: Num) {
        self.nums.insert(num.num_id, num);
    }

    pub fn get_num(&self, num_id: u32) -> Option<&Num> {
        self.nums.get(&num_id)
    }

    pub fn get_abstract_num(&self, abstract_num_id: u32) -> Option<&AbstractNum> {
        self.abstract_nums.get(&abstract_num_id)
    }

    pub fn from_xml_str(xml: &str) -> Result<Self> {
        parse_numbering(xml.as_bytes())
    }

    pub fn to_xml_string(&self) -> Result<String> {
        let bytes = serialize_numbering(self)?;
        String::from_utf8(bytes).map_err(|e| EngineError::MalformedPackage(e.to_string()))
    }
}

/// Computed list item metadata resulting from the numbering state machine.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComputedNumberingItem {
    pub prefix_text: String,
    pub num_fmt: NumberFormat,
    pub counter: u32,
    pub p_pr: Option<ParagraphProperties>,
    pub r_pr: Option<RunProperties>,
}

/// Stateful list numbering engine tracking item counters and evaluating level prefixes.
#[derive(Debug, Clone, Default)]
pub struct NumberingEngine {
    /// Maps (num_id, ilvl) -> current integer counter value.
    counters: HashMap<(u32, u32), u32>,
}

impl NumberingEngine {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Resets all counters in the engine.
    pub fn reset(&mut self) {
        self.counters.clear();
    }

    /// Evaluates the next list item for a paragraph with `<w:numPr>`.
    pub fn next_item(
        &mut self,
        num_id: u32,
        ilvl: u32,
        numbering: &Numbering,
    ) -> Option<ComputedNumberingItem> {
        let num = numbering.get_num(num_id)?;
        let abs_num = numbering.get_abstract_num(num.abstract_num_id)?;

        // Check for level override
        let lvl_override = num.lvl_overrides.get(&ilvl);
        let level = if let Some(lo) = lvl_override {
            if let Some(ref custom_lvl) = lo.lvl {
                custom_lvl
            } else {
                abs_num.get_level(ilvl)?
            }
        } else {
            abs_num.get_level(ilvl)?
        };

        // Resolve initial start value
        let start_val = lvl_override
            .and_then(|lo| lo.start_override)
            .unwrap_or(level.start);

        let counter_key = (num_id, ilvl);
        let cur_val = match self.counters.get(&counter_key) {
            Some(&prev) => prev + 1,
            None => start_val,
        };
        self.counters.insert(counter_key, cur_val);

        // Reset child levels > ilvl if specified by lvlRestart or standard hierarchy
        for child_ilvl in (ilvl + 1)..=8 {
            self.counters.remove(&(num_id, child_ilvl));
        }

        // Interpolate level template (%1, %2, ...)
        let mut prefix_text = level.lvl_text.clone();
        for target_lvl in 0..=8 {
            let placeholder = format!("%{}", target_lvl + 1);
            if prefix_text.contains(&placeholder) {
                let lvl_def = abs_num.get_level(target_lvl);
                let num_fmt = lvl_def.map(|l| l.num_fmt).unwrap_or(NumberFormat::Decimal);
                let lvl_counter = self
                    .counters
                    .get(&(num_id, target_lvl))
                    .copied()
                    .unwrap_or(1);
                let formatted_num = num_fmt.format_number(lvl_counter);
                prefix_text = prefix_text.replace(&placeholder, &formatted_num);
            }
        }

        Some(ComputedNumberingItem {
            prefix_text,
            num_fmt: level.num_fmt,
            counter: cur_val,
            p_pr: level.p_pr.clone(),
            r_pr: level.r_pr.clone(),
        })
    }
}

// --- Number Formatting Utilities ---

fn to_roman(mut num: u32, upper: bool) -> String {
    if num == 0 {
        return "0".to_string();
    }
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut res = String::new();
    for &(val, sym) in &table {
        while num >= val {
            res.push_str(sym);
            num -= val;
        }
    }
    if upper {
        res
    } else {
        res.to_lowercase()
    }
}

fn to_letter(mut num: u32, upper: bool) -> String {
    if num == 0 {
        return String::new();
    }
    let mut chars = Vec::new();
    while num > 0 {
        let rem = ((num - 1) % 26) as u8;
        let c = if upper { b'A' + rem } else { b'a' + rem };
        chars.push(c as char);
        num = (num - 1) / 26;
    }
    chars.reverse();
    chars.into_iter().collect()
}

fn to_ordinal(num: u32) -> String {
    let suffix = match num % 100 {
        11..=13 => "th",
        _ => match num % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{num}{suffix}")
}

fn to_cardinal_text(num: u32) -> String {
    match num {
        1 => "one".to_string(),
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        6 => "six".to_string(),
        7 => "seven".to_string(),
        8 => "eight".to_string(),
        9 => "nine".to_string(),
        10 => "ten".to_string(),
        _ => num.to_string(),
    }
}

// --- Streaming XML Parser for word/numbering.xml ---

pub fn parse_numbering(xml_bytes: &[u8]) -> Result<Numbering> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut numbering = Numbering::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "numbering" => {
                for attr in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(attr.key.into_inner()).to_string();
                    let v = String::from_utf8_lossy(&attr.value).to_string();
                    numbering.namespaces.push((k, v));
                }
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "abstractNum" => {
                let abs = parse_abstract_num(&mut reader, e)?;
                numbering.add_abstract_num(abs);
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "num" => {
                let num = parse_num(&mut reader, e)?;
                numbering.add_num(num);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(numbering)
}

fn parse_abstract_num<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<AbstractNum> {
    let id = get_attr_val(start_tag, "abstractNumId")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let mut abs = AbstractNum::new(id);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "lvl" => {
                let lvl = parse_level(reader, e)?;
                abs.add_level(lvl);
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "multiLevelType" => abs.multi_level_type = get_attr_val(e, "val"),
                    "name" => abs.name = get_attr_val(e, "val"),
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "abstractNum" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(abs)
}

fn parse_level<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<Level> {
    let ilvl = get_attr_val(start_tag, "ilvl")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let mut lvl = Level {
        ilvl,
        start: 1,
        ..Default::default()
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pPr" => lvl.p_pr = Some(crate::parser::parse_paragraph_properties(reader)?),
                    "rPr" => lvl.r_pr = Some(crate::parser::parse_run_properties(reader)?),
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "start" => {
                        lvl.start = get_attr_val(e, "val")
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(1);
                    }
                    "numFmt" => {
                        if let Some(val) = get_attr_val(e, "val") {
                            lvl.num_fmt = NumberFormat::from_str_opt(&val);
                        }
                    }
                    "lvlRestart" => {
                        lvl.lvl_restart =
                            get_attr_val(e, "val").and_then(|s| s.parse::<u32>().ok());
                    }
                    "pStyle" => lvl.p_style = get_attr_val(e, "val"),
                    "isLgl" => lvl.is_lgl = true,
                    "lvlText" => {
                        if let Some(val) = get_attr_val(e, "val") {
                            lvl.lvl_text = val;
                        }
                    }
                    "lvlJc" => lvl.lvl_jc = get_attr_val(e, "val"),
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "lvl" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(lvl)
}

fn parse_num<R: std::io::BufRead>(reader: &mut Reader<R>, start_tag: &BytesStart) -> Result<Num> {
    let num_id = get_attr_val(start_tag, "numId")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let mut num = Num::new(num_id, 0);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "lvlOverride" => {
                let lo = parse_lvl_override(reader, e)?;
                num.lvl_overrides.insert(lo.ilvl, lo);
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                if name == "abstractNumId" {
                    num.abstract_num_id = get_attr_val(e, "val")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "num" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(num)
}

fn parse_lvl_override<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<LevelOverride> {
    let ilvl = get_attr_val(start_tag, "ilvl")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let mut lo = LevelOverride {
        ilvl,
        start_override: None,
        lvl: None,
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "lvl" => {
                lo.lvl = Some(parse_level(reader, e)?);
            }
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "startOverride" => {
                lo.start_override = get_attr_val(e, "val").and_then(|s| s.parse::<u32>().ok());
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "lvlOverride" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(lo)
}

// --- Serializer for word/numbering.xml ---

pub fn serialize_numbering(numbering: &Numbering) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root_tag = BytesStart::new("w:numbering");
    for (k, v) in &numbering.namespaces {
        root_tag.push_attribute((k.as_str(), v.as_str()));
    }
    writer
        .write_event(Event::Start(root_tag))
        .map_err(EngineError::Io)?;

    // 1. AbstractNums
    for abs in numbering.abstract_nums.values() {
        serialize_abstract_num(&mut writer, abs)?;
    }

    // 2. Nums
    for num in numbering.nums.values() {
        serialize_num(&mut writer, num)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:numbering")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

fn serialize_abstract_num<W: std::io::Write>(
    writer: &mut Writer<W>,
    abs: &AbstractNum,
) -> Result<()> {
    let mut abs_tag = BytesStart::new("w:abstractNum");
    let id_str = abs.abstract_num_id.to_string();
    abs_tag.push_attribute(("w:abstractNumId", id_str.as_str()));
    writer
        .write_event(Event::Start(abs_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref mlt) = abs.multi_level_type {
        let mut t = BytesStart::new("w:multiLevelType");
        t.push_attribute(("w:val", mlt.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref name) = abs.name {
        let mut t = BytesStart::new("w:name");
        t.push_attribute(("w:val", name.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }

    for lvl in abs.levels.values() {
        serialize_level(writer, lvl)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:abstractNum")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_level<W: std::io::Write>(writer: &mut Writer<W>, lvl: &Level) -> Result<()> {
    let mut lvl_tag = BytesStart::new("w:lvl");
    let ilvl_str = lvl.ilvl.to_string();
    lvl_tag.push_attribute(("w:ilvl", ilvl_str.as_str()));
    writer
        .write_event(Event::Start(lvl_tag))
        .map_err(EngineError::Io)?;

    let mut start_tag = BytesStart::new("w:start");
    let start_str = lvl.start.to_string();
    start_tag.push_attribute(("w:val", start_str.as_str()));
    writer
        .write_event(Event::Empty(start_tag))
        .map_err(EngineError::Io)?;

    let mut fmt_tag = BytesStart::new("w:numFmt");
    fmt_tag.push_attribute(("w:val", lvl.num_fmt.as_str()));
    writer
        .write_event(Event::Empty(fmt_tag))
        .map_err(EngineError::Io)?;

    if let Some(r) = lvl.lvl_restart {
        let mut r_tag = BytesStart::new("w:lvlRestart");
        let s = r.to_string();
        r_tag.push_attribute(("w:val", s.as_str()));
        writer
            .write_event(Event::Empty(r_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref pstyle) = lvl.p_style {
        let mut t = BytesStart::new("w:pStyle");
        t.push_attribute(("w:val", pstyle.as_str()));
        writer
            .write_event(Event::Empty(t))
            .map_err(EngineError::Io)?;
    }

    if lvl.is_lgl {
        writer
            .write_event(Event::Empty(BytesStart::new("w:isLgl")))
            .map_err(EngineError::Io)?;
    }

    let mut txt_tag = BytesStart::new("w:lvlText");
    txt_tag.push_attribute(("w:val", lvl.lvl_text.as_str()));
    writer
        .write_event(Event::Empty(txt_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref jc) = lvl.lvl_jc {
        let mut jc_tag = BytesStart::new("w:lvlJc");
        jc_tag.push_attribute(("w:val", jc.as_str()));
        writer
            .write_event(Event::Empty(jc_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref p_pr) = lvl.p_pr {
        crate::serializer::serialize_paragraph_properties(writer, p_pr)?;
    }
    if let Some(ref r_pr) = lvl.r_pr {
        crate::serializer::serialize_run_properties(writer, r_pr)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:lvl")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_num<W: std::io::Write>(writer: &mut Writer<W>, num: &Num) -> Result<()> {
    let mut num_tag = BytesStart::new("w:num");
    let num_str = num.num_id.to_string();
    num_tag.push_attribute(("w:numId", num_str.as_str()));
    writer
        .write_event(Event::Start(num_tag))
        .map_err(EngineError::Io)?;

    let mut abs_tag = BytesStart::new("w:abstractNumId");
    let abs_str = num.abstract_num_id.to_string();
    abs_tag.push_attribute(("w:val", abs_str.as_str()));
    writer
        .write_event(Event::Empty(abs_tag))
        .map_err(EngineError::Io)?;

    for lo in num.lvl_overrides.values() {
        let mut lo_tag = BytesStart::new("w:lvlOverride");
        let ilvl_str = lo.ilvl.to_string();
        lo_tag.push_attribute(("w:ilvl", ilvl_str.as_str()));
        writer
            .write_event(Event::Start(lo_tag))
            .map_err(EngineError::Io)?;

        if let Some(start) = lo.start_override {
            let mut so_tag = BytesStart::new("w:startOverride");
            let s = start.to_string();
            so_tag.push_attribute(("w:val", s.as_str()));
            writer
                .write_event(Event::Empty(so_tag))
                .map_err(EngineError::Io)?;
        }

        if let Some(ref lvl) = lo.lvl {
            serialize_level(writer, lvl)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("w:lvlOverride")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:num")))
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
