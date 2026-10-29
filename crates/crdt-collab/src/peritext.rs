use crate::id::OpId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatAttribute {
    Bold(bool),
    Italic(bool),
    Underline(bool),
    FontSize(u32),
    Color(String),
    Style(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeritextSpan {
    pub id: OpId,
    pub start_id: OpId,
    pub end_id: OpId,
    pub attr: FormatAttribute,
    pub lamport: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichTextRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_size: Option<u32>,
    pub color: Option<String>,
    pub style: Option<String>,
}
