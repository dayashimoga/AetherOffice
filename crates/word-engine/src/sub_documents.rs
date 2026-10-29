//! OpenOfficeRS Word Engine - Sub-Documents & Auxiliary Parts Model.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Models headers (`word/header*.xml`), footers (`word/footer*.xml`), footnotes (`word/footnotes.xml`),
//! endnotes (`word/endnotes.xml`), and comments (`word/comments.xml`).

use crate::model::Body;
use serde::{Deserialize, Serialize};

/// Header sub-document part: `<w:hdr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HeaderDocument {
    pub namespaces: Vec<(String, String)>,
    pub body: Body,
}

impl HeaderDocument {
    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Footer sub-document part: `<w:ftr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FooterDocument {
    pub namespaces: Vec<(String, String)>,
    pub body: Body,
}

impl FooterDocument {
    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Footnote item within `<w:footnotes>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Footnote {
    pub id: i64,
    pub footnote_type: Option<String>, // "separator", "continuationSeparator", or normal (None)
    pub body: Body,
}

impl Footnote {
    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Footnotes sub-document part: `<w:footnotes>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FootnotesDocument {
    pub namespaces: Vec<(String, String)>,
    pub footnotes: Vec<Footnote>,
}

/// Endnote item within `<w:endnotes>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Endnote {
    pub id: i64,
    pub endnote_type: Option<String>,
    pub body: Body,
}

impl Endnote {
    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Endnotes sub-document part: `<w:endnotes>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EndnotesDocument {
    pub namespaces: Vec<(String, String)>,
    pub endnotes: Vec<Endnote>,
}

/// Review comment within `<w:comments>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub initials: Option<String>,
    pub date: Option<String>,
    pub body: Body,
}

impl Comment {
    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Comments sub-document part: `<w:comments>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentsDocument {
    pub namespaces: Vec<(String, String)>,
    pub comments: Vec<Comment>,
}
