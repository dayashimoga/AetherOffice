//! PDF Ink Annotation and Stylus Path Layer Compiler (ISO 32000-1 Clause 12.5.6.14).

use crate::types::{PdfObject, PdfRect, PdfStream};
use std::collections::BTreeMap;

/// Single freehand ink stroke consisting of consecutive `(x, y)` coordinate points.
#[derive(Debug, Clone, PartialEq)]
pub struct InkStroke {
    pub points: Vec<(f64, f64)>,
}

impl InkStroke {
    pub fn new(points: Vec<(f64, f64)>) -> Self {
        Self { points }
    }
}

/// Freehand ink drawing annotation (/Subtype /Ink) representing handwritten notes or signatures.
#[derive(Debug, Clone, PartialEq)]
pub struct InkAnnotation {
    pub rect: PdfRect,
    pub strokes: Vec<InkStroke>,
    pub color: [f32; 3],
    pub width: f64,
    pub opacity: f32,
    pub contents: Option<String>,
}

impl InkAnnotation {
    pub fn new(strokes: Vec<InkStroke>, color: [f32; 3], width: f64) -> Self {
        let rect = Self::calculate_bounding_rect(&strokes, width);
        Self {
            rect,
            strokes,
            color,
            width,
            opacity: 1.0,
            contents: None,
        }
    }

    /// Automatically computes the encompassing bounding rectangle for all strokes.
    pub fn calculate_bounding_rect(strokes: &[InkStroke], width: f64) -> PdfRect {
        if strokes.is_empty() {
            return PdfRect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for stroke in strokes {
            for &(x, y) in &stroke.points {
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x > max_x {
                    max_x = x;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }

        let pad = width.max(1.0);
        PdfRect::new(min_x - pad, min_y - pad, max_x + pad, max_y + pad)
    }

    /// Compiles ink path strokes into a standards-compliant `/AP /N` appearance stream.
    pub fn compile_appearance_stream(&self) -> PdfStream {
        let mut stream_content = Vec::new();

        // 1. Line width and stroke color
        let header = format!(
            "{:.2} w 1 J 1 j {:.3} {:.3} {:.3} RG\n",
            self.width, self.color[0], self.color[1], self.color[2]
        );
        stream_content.extend_from_slice(header.as_bytes());

        // 2. Draw path commands relative to annotation origin (self.rect.llx, self.rect.lly)
        let ox = self.rect.llx;
        let oy = self.rect.lly;

        for stroke in &self.strokes {
            if stroke.points.is_empty() {
                continue;
            }

            let p0 = stroke.points[0];
            let start_cmd = format!("{:.2} {:.2} m\n", p0.0 - ox, p0.1 - oy);
            stream_content.extend_from_slice(start_cmd.as_bytes());

            for pt in stroke.points.iter().skip(1) {
                let line_cmd = format!("{:.2} {:.2} l\n", pt.0 - ox, pt.1 - oy);
                stream_content.extend_from_slice(line_cmd.as_bytes());
            }

            stream_content.extend_from_slice(b"S\n");
        }

        // 3. Form XObject Dictionary descriptor
        let mut dict = BTreeMap::new();
        dict.insert("Type".into(), PdfObject::Name("XObject".into()));
        dict.insert("Subtype".into(), PdfObject::Name("Form".into()));
        dict.insert("FormType".into(), PdfObject::Integer(1));
        dict.insert(
            "BBox".into(),
            PdfObject::Array(vec![
                PdfObject::Real(0.0),
                PdfObject::Real(0.0),
                PdfObject::Real(self.rect.width()),
                PdfObject::Real(self.rect.height()),
            ]),
        );

        PdfStream::new(dict, stream_content)
    }

    /// Converts the ink annotation into a standard PDF `/Type /Annot /Subtype /Ink` dictionary.
    pub fn to_dict(&self) -> BTreeMap<String, PdfObject> {
        let mut dict = BTreeMap::new();
        dict.insert("Type".into(), PdfObject::Name("Annot".into()));
        dict.insert("Subtype".into(), PdfObject::Name("Ink".into()));
        dict.insert("Rect".into(), self.rect.to_array());

        // /InkList [[x1 y1 x2 y2 ...] ...]
        let mut ink_list = Vec::new();
        for stroke in &self.strokes {
            let mut pt_arr = Vec::with_capacity(stroke.points.len() * 2);
            for &(x, y) in &stroke.points {
                pt_arr.push(PdfObject::Real(x));
                pt_arr.push(PdfObject::Real(y));
            }
            ink_list.push(PdfObject::Array(pt_arr));
        }
        dict.insert("InkList".into(), PdfObject::Array(ink_list));

        // /C [r g b]
        dict.insert(
            "C".into(),
            PdfObject::Array(vec![
                PdfObject::Real(self.color[0] as f64),
                PdfObject::Real(self.color[1] as f64),
                PdfObject::Real(self.color[2] as f64),
            ]),
        );

        // /BS << /W width >>
        let mut bs = BTreeMap::new();
        bs.insert("W".into(), PdfObject::Real(self.width));
        dict.insert("BS".into(), PdfObject::Dictionary(bs));

        if let Some(ref c) = self.contents {
            dict.insert(
                "Contents".into(),
                PdfObject::LiteralString(c.as_bytes().to_vec()),
            );
        }

        dict
    }

    /// Converts the ink annotation to a dictionary referencing an external `/AP /N` appearance stream object.
    pub fn to_dict_with_ap(
        &self,
        ap_ref: crate::types::PdfObjectId,
    ) -> BTreeMap<String, PdfObject> {
        let mut dict = self.to_dict();
        let mut ap_dict = BTreeMap::new();
        ap_dict.insert("N".into(), PdfObject::Reference(ap_ref));
        dict.insert("AP".into(), PdfObject::Dictionary(ap_dict));
        dict
    }
}
