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
    pub fn new(id: i64, text: &str) -> Self {
        use crate::model::{BodyElement, Paragraph, ParagraphChild, Run, RunContent};
        let mut p = Paragraph::default();
        let mut r = Run::default();
        r.content.push(RunContent::FootnoteRef);
        r.content
            .push(RunContent::Text(format!(" {}", text), false));
        p.children.push(ParagraphChild::Run(r));

        Self {
            id,
            footnote_type: None,
            body: Body {
                elements: vec![BodyElement::Paragraph(p)],
                sect_pr: None,
            },
        }
    }

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
    pub fn new(id: i64, text: &str) -> Self {
        use crate::model::{BodyElement, Paragraph, ParagraphChild, Run, RunContent};
        let mut p = Paragraph::default();
        let mut r = Run::default();
        r.content.push(RunContent::EndnoteRef);
        r.content
            .push(RunContent::Text(format!(" {}", text), false));
        p.children.push(ParagraphChild::Run(r));

        Self {
            id,
            endnote_type: None,
            body: Body {
                elements: vec![BodyElement::Paragraph(p)],
                sect_pr: None,
            },
        }
    }

    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }
}

/// Footnote and endnote numbering styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FootnoteNumberingFormat {
    #[default]
    Arabic,
    RomanLower,
    RomanUpper,
    LetterLower,
    LetterUpper,
    Symbol,
}

impl FootnoteNumberingFormat {
    pub fn format_index(&self, index: usize) -> String {
        match self {
            Self::Arabic => index.to_string(),
            Self::RomanLower => Self::to_roman(index, false),
            Self::RomanUpper => Self::to_roman(index, true),
            Self::LetterLower => Self::to_letter(index, false),
            Self::LetterUpper => Self::to_letter(index, true),
            Self::Symbol => {
                let symbols = ["*", "†", "‡", "§", "||", "¶"];
                if index == 0 {
                    return "*".to_string();
                }
                let sym_idx = (index.saturating_sub(1)) % symbols.len();
                let repeat = ((index.saturating_sub(1)) / symbols.len()).saturating_add(1);
                symbols[sym_idx].repeat(repeat)
            }
        }
    }

    fn to_roman(mut num: usize, upper: bool) -> String {
        if num == 0 {
            return if upper {
                "N".to_string()
            } else {
                "n".to_string()
            };
        }
        let roman_table = [
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
        let mut result = String::new();
        for &(val, sym) in &roman_table {
            while num >= val {
                result.push_str(sym);
                num = num.saturating_sub(val);
            }
        }
        if upper {
            result
        } else {
            result.to_lowercase()
        }
    }

    fn to_letter(mut num: usize, upper: bool) -> String {
        if num == 0 {
            return if upper {
                "A".to_string()
            } else {
                "a".to_string()
            };
        }
        let mut chars = Vec::new();
        while num > 0 {
            let rem = (num.saturating_sub(1)) % 26;
            let ch = ((if upper { b'A' } else { b'a' }).saturating_add(rem as u8)) as char;
            chars.push(ch);
            num = (num.saturating_sub(1)) / 26;
        }
        chars.into_iter().rev().collect()
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
    pub parent_id: Option<String>,
    pub is_resolved: bool,
    pub body: Body,
}

impl Comment {
    pub fn new(id: impl Into<String>, author: impl Into<String>, text: &str) -> Self {
        use crate::model::{BodyElement, Paragraph, ParagraphChild, Run, RunContent};
        let mut p = Paragraph::default();
        let mut r = Run::default();
        r.content.push(RunContent::Text(text.to_string(), false));
        p.children.push(ParagraphChild::Run(r));

        Self {
            id: id.into(),
            author: author.into(),
            initials: None,
            date: None,
            parent_id: None,
            is_resolved: false,
            body: Body {
                elements: vec![BodyElement::Paragraph(p)],
                sect_pr: None,
            },
        }
    }

    pub fn get_text(&self) -> String {
        self.body.get_full_text()
    }

    pub fn is_reply(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// Comments sub-document part: `<w:comments>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentsDocument {
    pub namespaces: Vec<(String, String)>,
    pub comments: Vec<Comment>,
}
