//! PDF Page model representing a resolved page with inherited attributes (ISO 32000-1 Clause 7.7.3).

use crate::types::{PdfObjectId, PdfRect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPage {
    /// 0-based index in the document
    pub index: usize,
    /// 1-based page number for human display
    pub page_number: usize,
    /// Indirect object reference of this /Type /Page
    pub object_id: PdfObjectId,
    /// Physical paper dimensions (/MediaBox)
    pub media_box: PdfRect,
    /// Viewable / printable region (/CropBox, defaults to MediaBox)
    pub crop_box: PdfRect,
    /// Rotation in degrees clockwise (0, 90, 180, 270)
    pub rotate: i32,
    /// Content stream indirect object references
    pub contents: Vec<PdfObjectId>,
    /// Resource dictionary (fonts, images, colorspaces)
    pub resources: BTreeMap<String, crate::types::PdfObject>,
    /// Annotation object references (/Annots)
    pub annotations: Vec<PdfObjectId>,
}

impl PdfPage {
    #[inline]
    pub fn width_pts(&self) -> f64 {
        self.crop_box.width()
    }

    #[inline]
    pub fn height_pts(&self) -> f64 {
        self.crop_box.height()
    }

    /// Effective display width in points taking page rotation into account.
    #[inline]
    pub fn effective_width(&self) -> f64 {
        if self.rotate == 90 || self.rotate == 270 {
            self.height_pts()
        } else {
            self.width_pts()
        }
    }

    /// Effective display height in points taking page rotation into account.
    #[inline]
    pub fn effective_height(&self) -> f64 {
        if self.rotate == 90 || self.rotate == 270 {
            self.width_pts()
        } else {
            self.height_pts()
        }
    }
}
