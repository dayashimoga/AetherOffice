//! PDF Annotations Engine (ISO 32000-1 Clause 12.5).

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::{PdfObject, PdfObjectId, PdfRect};
use std::collections::BTreeMap;

/// Supported annotation subtypes under ISO 32000-1.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationSubtype {
    Text, // Sticky Note
    Link { uri: Option<String> },
    FreeText { default_appearance: String },
    Highlight { quad_points: Vec<f64> },
    Underline { quad_points: Vec<f64> },
    Squiggly { quad_points: Vec<f64> },
    StrikeOut { quad_points: Vec<f64> },
    Square,
    Circle,
    Line { start: (f64, f64), end: (f64, f64) },
    Ink { strokes: Vec<Vec<(f64, f64)>> },
    Stamp { name: String },
    Widget,
    Other(String),
}

/// Generic parsed PDF annotation record.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfAnnotation {
    pub id: PdfObjectId,
    pub page_index: usize,
    pub rect: PdfRect,
    pub subtype: AnnotationSubtype,
    pub contents: Option<String>,
    pub author: Option<String>,
    pub color: Option<[f32; 3]>,
    pub flags: u32,
}

impl PdfAnnotation {
    pub fn new(
        id: PdfObjectId,
        page_index: usize,
        rect: PdfRect,
        subtype: AnnotationSubtype,
    ) -> Self {
        Self {
            id,
            page_index,
            rect,
            subtype,
            contents: None,
            author: None,
            color: None,
            flags: 4, // Print by default
        }
    }

    /// Parses an annotation dictionary from a page.
    pub fn from_dict(
        id: PdfObjectId,
        page_index: usize,
        dict: &BTreeMap<String, PdfObject>,
    ) -> Result<Self, PdfError> {
        let subtype_name = dict
            .get("Subtype")
            .and_then(|v| v.as_name())
            .unwrap_or("Other");

        let rect = dict
            .get("Rect")
            .and_then(|v| v.as_array())
            .and_then(|arr| PdfRect::from_array(arr).ok())
            .unwrap_or_else(|| PdfRect::new(0.0, 0.0, 100.0, 100.0));

        let contents = dict.get("Contents").and_then(|v| v.to_string_lossy());
        let author = dict.get("T").and_then(|v| v.to_string_lossy());
        let flags = dict.get("F").and_then(|v| v.as_i64()).unwrap_or(0).max(0) as u32;

        let color = if let Some(PdfObject::Array(c_arr)) = dict.get("C") {
            if c_arr.len() >= 3 {
                let r = c_arr[0].as_f64().unwrap_or(0.0) as f32;
                let g = c_arr[1].as_f64().unwrap_or(0.0) as f32;
                let b = c_arr[2].as_f64().unwrap_or(0.0) as f32;
                Some([r, g, b])
            } else {
                None
            }
        } else {
            None
        };

        let subtype = match subtype_name {
            "Text" => AnnotationSubtype::Text,
            "Link" => {
                let uri = dict
                    .get("A")
                    .and_then(|a| a.as_dict())
                    .and_then(|a_dict| a_dict.get("URI").and_then(|u| u.to_string_lossy()));
                AnnotationSubtype::Link { uri }
            }
            "FreeText" => {
                let da = dict
                    .get("DA")
                    .and_then(|v| v.to_string_lossy())
                    .unwrap_or_else(|| "/Helv 12 Tf 0 g".into());
                AnnotationSubtype::FreeText {
                    default_appearance: da,
                }
            }
            "Highlight" | "Underline" | "Squiggly" | "StrikeOut" => {
                let mut quad_points = Vec::new();
                if let Some(PdfObject::Array(qp_arr)) = dict.get("QuadPoints") {
                    for item in qp_arr {
                        if let Some(f) = item.as_f64() {
                            quad_points.push(f);
                        }
                    }
                }
                match subtype_name {
                    "Highlight" => AnnotationSubtype::Highlight { quad_points },
                    "Underline" => AnnotationSubtype::Underline { quad_points },
                    "Squiggly" => AnnotationSubtype::Squiggly { quad_points },
                    _ => AnnotationSubtype::StrikeOut { quad_points },
                }
            }
            "Square" => AnnotationSubtype::Square,
            "Circle" => AnnotationSubtype::Circle,
            "Line" => {
                let mut start = (0.0, 0.0);
                let mut end = (0.0, 0.0);
                if let Some(PdfObject::Array(l_arr)) = dict.get("L") {
                    if l_arr.len() >= 4 {
                        start = (
                            l_arr[0].as_f64().unwrap_or(0.0),
                            l_arr[1].as_f64().unwrap_or(0.0),
                        );
                        end = (
                            l_arr[2].as_f64().unwrap_or(0.0),
                            l_arr[3].as_f64().unwrap_or(0.0),
                        );
                    }
                }
                AnnotationSubtype::Line { start, end }
            }
            "Ink" => {
                let mut strokes = Vec::new();
                if let Some(PdfObject::Array(il_arr)) = dict.get("InkList") {
                    for stroke_obj in il_arr {
                        if let Some(pts_arr) = stroke_obj.as_array() {
                            let mut stroke = Vec::with_capacity(pts_arr.len() / 2);
                            for chunk in pts_arr.chunks(2) {
                                if chunk.len() == 2 {
                                    let x = chunk[0].as_f64().unwrap_or(0.0);
                                    let y = chunk[1].as_f64().unwrap_or(0.0);
                                    stroke.push((x, y));
                                }
                            }
                            strokes.push(stroke);
                        }
                    }
                }
                AnnotationSubtype::Ink { strokes }
            }
            "Stamp" => {
                let name = dict
                    .get("Name")
                    .and_then(|v| v.to_string_lossy())
                    .unwrap_or_else(|| "Approved".into());
                AnnotationSubtype::Stamp { name }
            }
            "Widget" => AnnotationSubtype::Widget,
            other => AnnotationSubtype::Other(other.to_string()),
        };

        Ok(Self {
            id,
            page_index,
            rect,
            subtype,
            contents,
            author,
            color,
            flags,
        })
    }

    /// Serializes this annotation into a standard PDF `/Type /Annot` dictionary.
    pub fn to_dict(&self) -> BTreeMap<String, PdfObject> {
        let mut dict = BTreeMap::new();
        dict.insert("Type".into(), PdfObject::Name("Annot".into()));
        dict.insert("Rect".into(), self.rect.to_array());
        dict.insert("F".into(), PdfObject::Integer(self.flags as i64));

        if let Some(ref c) = self.contents {
            dict.insert(
                "Contents".into(),
                PdfObject::LiteralString(c.as_bytes().to_vec()),
            );
        }
        if let Some(ref a) = self.author {
            dict.insert("T".into(), PdfObject::LiteralString(a.as_bytes().to_vec()));
        }
        if let Some(col) = self.color {
            dict.insert(
                "C".into(),
                PdfObject::Array(vec![
                    PdfObject::Real(col[0] as f64),
                    PdfObject::Real(col[1] as f64),
                    PdfObject::Real(col[2] as f64),
                ]),
            );
        }

        match &self.subtype {
            AnnotationSubtype::Text => {
                dict.insert("Subtype".into(), PdfObject::Name("Text".into()));
            }
            AnnotationSubtype::Link { uri } => {
                dict.insert("Subtype".into(), PdfObject::Name("Link".into()));
                if let Some(u) = uri {
                    let mut a_dict = BTreeMap::new();
                    a_dict.insert("S".into(), PdfObject::Name("URI".into()));
                    a_dict.insert(
                        "URI".into(),
                        PdfObject::LiteralString(u.as_bytes().to_vec()),
                    );
                    dict.insert("A".into(), PdfObject::Dictionary(a_dict));
                }
            }
            AnnotationSubtype::FreeText { default_appearance } => {
                dict.insert("Subtype".into(), PdfObject::Name("FreeText".into()));
                dict.insert(
                    "DA".into(),
                    PdfObject::LiteralString(default_appearance.as_bytes().to_vec()),
                );
            }
            AnnotationSubtype::Highlight { quad_points } => {
                dict.insert("Subtype".into(), PdfObject::Name("Highlight".into()));
                let qp_arr = quad_points.iter().copied().map(PdfObject::Real).collect();
                dict.insert("QuadPoints".into(), PdfObject::Array(qp_arr));
            }
            AnnotationSubtype::Underline { quad_points } => {
                dict.insert("Subtype".into(), PdfObject::Name("Underline".into()));
                let qp_arr = quad_points.iter().copied().map(PdfObject::Real).collect();
                dict.insert("QuadPoints".into(), PdfObject::Array(qp_arr));
            }
            AnnotationSubtype::Squiggly { quad_points } => {
                dict.insert("Subtype".into(), PdfObject::Name("Squiggly".into()));
                let qp_arr = quad_points.iter().copied().map(PdfObject::Real).collect();
                dict.insert("QuadPoints".into(), PdfObject::Array(qp_arr));
            }
            AnnotationSubtype::StrikeOut { quad_points } => {
                dict.insert("Subtype".into(), PdfObject::Name("StrikeOut".into()));
                let qp_arr = quad_points.iter().copied().map(PdfObject::Real).collect();
                dict.insert("QuadPoints".into(), PdfObject::Array(qp_arr));
            }
            AnnotationSubtype::Square => {
                dict.insert("Subtype".into(), PdfObject::Name("Square".into()));
            }
            AnnotationSubtype::Circle => {
                dict.insert("Subtype".into(), PdfObject::Name("Circle".into()));
            }
            AnnotationSubtype::Line { start, end } => {
                dict.insert("Subtype".into(), PdfObject::Name("Line".into()));
                dict.insert(
                    "L".into(),
                    PdfObject::Array(vec![
                        PdfObject::Real(start.0),
                        PdfObject::Real(start.1),
                        PdfObject::Real(end.0),
                        PdfObject::Real(end.1),
                    ]),
                );
            }
            AnnotationSubtype::Ink { strokes } => {
                dict.insert("Subtype".into(), PdfObject::Name("Ink".into()));
                let mut il_arr = Vec::new();
                for s in strokes {
                    let mut pts = Vec::new();
                    for &(x, y) in s {
                        pts.push(PdfObject::Real(x));
                        pts.push(PdfObject::Real(y));
                    }
                    il_arr.push(PdfObject::Array(pts));
                }
                dict.insert("InkList".into(), PdfObject::Array(il_arr));
            }
            AnnotationSubtype::Stamp { name } => {
                dict.insert("Subtype".into(), PdfObject::Name("Stamp".into()));
                dict.insert("Name".into(), PdfObject::Name(name.clone()));
            }
            AnnotationSubtype::Widget => {
                dict.insert("Subtype".into(), PdfObject::Name("Widget".into()));
            }
            AnnotationSubtype::Other(name) => {
                dict.insert("Subtype".into(), PdfObject::Name(name.clone()));
            }
        }

        dict
    }
}

pub struct AnnotationEngine;

impl AnnotationEngine {
    /// Extracts all annotations from across all pages of a document.
    pub fn extract_annotations(doc: &PdfDocument) -> Result<Vec<PdfAnnotation>, PdfError> {
        let mut annotations = Vec::new();

        for (page_idx, page) in doc.pages().iter().enumerate() {
            for &annot_id in &page.annotations {
                if let Ok(PdfObject::Dictionary(dict)) = doc.resolve_object(annot_id) {
                    if let Ok(annot) = PdfAnnotation::from_dict(annot_id, page_idx, &dict) {
                        annotations.push(annot);
                    }
                }
            }
        }

        Ok(annotations)
    }

    /// Creates a new Sticky Note Text Annotation.
    pub fn create_text_annotation(
        id: PdfObjectId,
        page_index: usize,
        rect: PdfRect,
        contents: String,
        author: Option<String>,
    ) -> PdfAnnotation {
        let mut annot = PdfAnnotation::new(id, page_index, rect, AnnotationSubtype::Text);
        annot.contents = Some(contents);
        annot.author = author;
        annot.color = Some([1.0, 0.9, 0.0]); // Yellow sticky note
        annot
    }

    /// Creates a new Highlight Annotation with quad points.
    pub fn create_highlight_annotation(
        id: PdfObjectId,
        page_index: usize,
        rect: PdfRect,
        quad_points: Vec<f64>,
        color: [f32; 3],
    ) -> PdfAnnotation {
        let mut annot = PdfAnnotation::new(
            id,
            page_index,
            rect,
            AnnotationSubtype::Highlight { quad_points },
        );
        annot.color = Some(color);
        annot
    }
}
