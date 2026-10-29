//! OpenOfficeRS Word Engine - WordprocessingML DOM Model.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Models ECMA-376 Part 1 §17 WordprocessingML elements:
//! w:document, w:body, w:p, w:r, w:t, w:tbl, w:sectPr, bookmarks, hyperlinks,
//! tracked changes (w:ins, w:del), comments, footnotes, and endnotes.

use serde::{Deserialize, Serialize};

/// Generic raw XML node buffer retaining unmodeled elements and extensions losslessly.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RawXmlNode {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub content: Option<String>,
    pub children: Vec<RawXmlNode>,
}

impl RawXmlNode {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
            content: None,
            children: Vec::new(),
        }
    }
}

/// Root document element: `<w:document>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Document {
    pub namespaces: Vec<(String, String)>,
    pub ignorable: Option<String>,
    pub body: Body,
    pub unparsed_nodes: Vec<RawXmlNode>,
}

impl Document {
    /// Extracts full plaintext from all paragraphs and table cells in the document body.
    pub fn get_full_text(&self) -> String {
        self.body.get_full_text()
    }

    /// Creates an immutable PieceTree containing the full document text.
    pub fn create_piece_tree(&self) -> crate::piece_tree::PieceTree {
        crate::piece_tree::PieceTree::new(&self.get_full_text())
    }
}

/// Document body container: `<w:body>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Body {
    pub elements: Vec<BodyElement>,
    pub sect_pr: Option<SectionProperties>,
}

impl Body {
    /// Extracts full plaintext from all contained elements.
    pub fn get_full_text(&self) -> String {
        let mut out = String::new();
        for (i, elem) in self.elements.iter().enumerate() {
            let t = elem.get_text();
            if !t.is_empty() {
                if i > 0 && !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&t);
            }
        }
        out
    }
}

/// Top-level block element within `<w:body>` or table cells.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodyElement {
    Paragraph(Paragraph),
    Table(Table),
    SectionProperties(SectionProperties),
    Raw(RawXmlNode),
}

impl BodyElement {
    pub fn get_text(&self) -> String {
        match self {
            Self::Paragraph(p) => p.get_text(),
            Self::Table(tbl) => tbl.get_text(),
            _ => String::new(),
        }
    }
}

/// Paragraph element: `<w:p>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Paragraph {
    pub para_id: Option<String>,
    pub text_id: Option<String>,
    pub rsid_r: Option<String>,
    pub rsid_r_default: Option<String>,
    pub properties: Option<ParagraphProperties>,
    pub children: Vec<ParagraphChild>,
    pub raw_attrs: Vec<(String, String)>,
}

impl Paragraph {
    /// Returns the concatenated text of all runs, hyperlinks, and tracked revisions.
    pub fn get_text(&self) -> String {
        let mut out = String::new();
        for child in &self.children {
            child.write_text(&mut out);
        }
        out
    }
}

/// Numbering reference in paragraph properties: `<w:numPr>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NumberingProperty {
    pub num_id: u32,
    pub ilvl: u32,
}

/// Paragraph formatting properties: `<w:pPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ParagraphProperties {
    pub style: Option<String>,
    pub justification: Option<String>,
    pub spacing: Option<Spacing>,
    pub indentation: Option<Indentation>,
    pub r_pr: Option<RunProperties>,
    pub num_pr: Option<NumberingProperty>,
    pub widow_control: Option<bool>,
    pub keep_next: bool,
    pub keep_lines: bool,
    pub page_break_before: bool,
    pub kinsoku: Option<bool>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Paragraph spacing: `<w:spacing>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Spacing {
    pub before: Option<u32>,
    pub after: Option<u32>,
    pub line: Option<u32>,
    pub line_rule: Option<String>,
}

/// Paragraph indentation: `<w:ind>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Indentation {
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub first_line: Option<i32>,
    pub hanging: Option<i32>,
}

/// In-line child element within a paragraph: `<w:p>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParagraphChild {
    Run(Run),
    BookmarkStart(BookmarkStart),
    BookmarkEnd(BookmarkEnd),
    Hyperlink(Hyperlink),
    Inserted(TrackedChange),
    Deleted(TrackedChange),
    CommentRangeStart(String),
    CommentRangeEnd(String),
    CommentReference(String),
    FootnoteReference(String),
    EndnoteReference(String),
    Raw(RawXmlNode),
}

impl ParagraphChild {
    pub fn write_text(&self, out: &mut String) {
        match self {
            Self::Run(r) => r.write_text(out),
            Self::Hyperlink(h) => {
                for c in &h.children {
                    c.write_text(out);
                }
            }
            Self::Inserted(ins) => {
                for c in &ins.children {
                    c.write_text(out);
                }
            }
            Self::Deleted(_) => {
                // Deleted text omitted from active plaintext
            }
            _ => {}
        }
    }
}

/// Text run element: `<w:r>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Run {
    pub rsid_r_pr: Option<String>,
    pub rsid_r: Option<String>,
    pub properties: Option<RunProperties>,
    pub content: Vec<RunContent>,
    pub raw_attrs: Vec<(String, String)>,
}

impl Run {
    pub fn write_text(&self, out: &mut String) {
        for c in &self.content {
            match c {
                RunContent::Text(s, _) => out.push_str(s),
                RunContent::Tab => out.push('\t'),
                RunContent::Break(_) => out.push('\n'),
                RunContent::SoftHyphen => out.push('\u{00AD}'),
                RunContent::NoBreakHyphen => out.push('\u{2011}'),
                _ => {}
            }
        }
    }

    pub fn get_text(&self) -> String {
        let mut s = String::new();
        self.write_text(&mut s);
        s
    }
}

/// Formatting properties for a text run: `<w:rPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RunProperties {
    pub bold: bool,
    pub italic: bool,
    pub underline: Option<String>,
    pub strike: bool,
    pub double_strike: bool,
    pub color: Option<String>,
    pub size_half_points: Option<u32>,
    pub font_ascii: Option<String>,
    pub font_h_ansi: Option<String>,
    pub font_east_asia: Option<String>,
    pub font_cs: Option<String>,
    pub highlight: Option<String>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Discrete content item within a run: `<w:r>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunContent {
    /// `<w:t xml:space="preserve">`: text string and xml:space preservation flag
    Text(String, bool),
    /// `<w:delText>`: deleted text within a revision
    DeletedText(String, bool),
    /// `<w:br w:type="...">`: page, column, or line break
    Break(Option<String>),
    /// `<w:tab/>`: tab stop character
    Tab,
    /// `<w:softHyphen/>`
    SoftHyphen,
    /// `<w:noBreakHyphen/>`
    NoBreakHyphen,
    /// `<w:separator/>`: footnote/endnote separator
    Separator,
    /// `<w:continuationSeparator/>`: footnote/endnote continuation separator
    ContinuationSeparator,
    /// `<w:footnoteRef/>`: reference mark in footnote
    FootnoteRef,
    /// `<w:endnoteRef/>`: reference mark in endnote
    EndnoteRef,
    /// `<w:drawing>` or other unmodeled inline drawing/part
    Raw(RawXmlNode),
}

/// Bookmark start: `<w:bookmarkStart>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BookmarkStart {
    pub id: String,
    pub name: String,
    pub col_first: Option<u32>,
    pub col_last: Option<u32>,
}

/// Bookmark end: `<w:bookmarkEnd>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BookmarkEnd {
    pub id: String,
}

/// Hyperlink element: `<w:hyperlink>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Hyperlink {
    pub id: Option<String>,
    pub anchor: Option<String>,
    pub tooltip: Option<String>,
    pub history: bool,
    pub children: Vec<ParagraphChild>,
}

/// Tracked revision element (`<w:ins>` or `<w:del>`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackedChange {
    pub id: String,
    pub author: String,
    pub date: String,
    pub children: Vec<ParagraphChild>,
}

/// Table element: `<w:tbl>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Table {
    pub properties: Option<TableProperties>,
    pub grid: Option<TableGrid>,
    pub rows: Vec<TableRow>,
    pub raw_attrs: Vec<(String, String)>,
}

impl Table {
    pub fn get_text(&self) -> String {
        let mut out = String::new();
        for r in &self.rows {
            for c in &r.cells {
                let cell_text = c.get_text();
                if !cell_text.is_empty() {
                    out.push_str(&cell_text);
                    out.push('\t');
                }
            }
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }
}

/// Table formatting properties: `<w:tblPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableProperties {
    pub width: Option<TableWidth>,
    pub justification: Option<String>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Table column grid definition: `<w:tblGrid>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableGrid {
    pub columns: Vec<u32>,
}

/// Table row: `<w:tr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableRow {
    pub rsid_r: Option<String>,
    pub rsid_tr: Option<String>,
    pub cells: Vec<TableCell>,
    pub raw_attrs: Vec<(String, String)>,
}

/// Table cell: `<w:tc>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableCell {
    pub width: Option<TableWidth>,
    pub elements: Vec<BodyElement>,
    pub raw_attrs: Vec<(String, String)>,
}

impl TableCell {
    pub fn get_text(&self) -> String {
        let mut out = String::new();
        for elem in &self.elements {
            let t = elem.get_text();
            if !t.is_empty() {
                out.push_str(&t);
            }
        }
        out
    }
}

/// Width measurement used in tables and cells: `<w:tblW>` / `<w:tcW>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TableWidth {
    pub width: u32,
    pub width_type: String,
}

/// Section formatting properties: `<w:sectPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SectionProperties {
    pub header_refs: Vec<HeaderFooterRef>,
    pub footer_refs: Vec<HeaderFooterRef>,
    pub page_size: Option<PageSize>,
    pub page_margin: Option<PageMargin>,
    pub columns: Option<Columns>,
    pub doc_grid: Option<DocGrid>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Header or footer relationship reference: `<w:headerReference>` / `<w:footerReference>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HeaderFooterRef {
    pub ref_type: String, // "default", "first", "even"
    pub rel_id: String,
}

/// Page dimensions: `<w:pgSz>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PageSize {
    pub width: u32,
    pub height: u32,
    pub orientation: Option<String>,
}

/// Page margins in dxa: `<w:pgMar>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PageMargin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
    pub header: i32,
    pub footer: i32,
    pub gutter: i32,
}

/// Column layout: `<w:cols>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Columns {
    pub num: Option<u32>,
    pub space: Option<u32>,
}

/// Document line grid: `<w:docGrid>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DocGrid {
    pub line_pitch: Option<u32>,
}
