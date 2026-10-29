//! OpenOfficeRS Word Engine - ECMA-376 Part 1 §17.6 Section Geometry & Multi-Column Layout Solver.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Computes exact printable/display geometry, content box boundaries, page margins, and multi-column divisions.

use crate::model::SectionProperties;
use serde::{Deserialize, Serialize};

/// Geometry specification for an individual text column in a multi-column section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnGeometry {
    pub width: u32, // in twips (dxa)
    pub space: u32, // spacing after column in twips (dxa)
}

/// Resolved geometric dimensions and layout box boundaries for a document section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionGeometry {
    pub page_width: u32,      // twips (e.g. 12240 for US Letter, 11906 for A4)
    pub page_height: u32,     // twips (e.g. 15840 for US Letter, 16838 for A4)
    pub margin_top: i32,      // twips (dxa)
    pub margin_bottom: i32,   // twips (dxa)
    pub margin_left: i32,     // twips (dxa)
    pub margin_right: i32,    // twips (dxa)
    pub gutter: i32,          // twips (dxa)
    pub header_distance: i32, // twips (dxa)
    pub footer_distance: i32, // twips (dxa)
    pub orientation: String,  // "portrait" or "landscape"
    pub columns: Vec<ColumnGeometry>,
}

impl Default for SectionGeometry {
    fn default() -> Self {
        // Standard US Letter Portrait with 1-inch margins (1440 twips)
        Self {
            page_width: 12240,
            page_height: 15840,
            margin_top: 1440,
            margin_bottom: 1440,
            margin_left: 1440,
            margin_right: 1440,
            gutter: 0,
            header_distance: 720,
            footer_distance: 720,
            orientation: "portrait".to_string(),
            columns: vec![ColumnGeometry {
                width: 9360, // 12240 - 2880
                space: 0,
            }],
        }
    }
}

impl SectionGeometry {
    /// Computes resolved geometry from ECMA-376 `SectionProperties`.
    pub fn from_section_properties(sect: &SectionProperties) -> Self {
        let (page_width, page_height, orientation) = if let Some(ref ps) = sect.page_size {
            let w = ps.width;
            let h = ps.height;
            let orient = ps
                .orientation
                .clone()
                .unwrap_or_else(|| "portrait".to_string());
            (w, h, orient)
        } else {
            (12240, 15840, "portrait".to_string())
        };

        let (
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            gutter,
            header_distance,
            footer_distance,
        ) = if let Some(ref pm) = sect.page_margin {
            (
                pm.top, pm.bottom, pm.left, pm.right, pm.gutter, pm.header, pm.footer,
            )
        } else {
            (1440, 1440, 1440, 1440, 0, 720, 720)
        };

        // Compute available printable content width
        let net_width = page_width
            .saturating_sub(margin_left.max(0) as u32)
            .saturating_sub(margin_right.max(0) as u32)
            .saturating_sub(gutter.max(0) as u32);

        // Resolve columns layout
        let num_cols = sect
            .columns
            .as_ref()
            .and_then(|c| c.num)
            .unwrap_or(1)
            .max(1);

        let space_between = sect.columns.as_ref().and_then(|c| c.space).unwrap_or(720); // 0.5 in default column gap

        let mut columns = Vec::with_capacity(num_cols as usize);
        if num_cols == 1 {
            columns.push(ColumnGeometry {
                width: net_width,
                space: 0,
            });
        } else {
            let total_gaps = space_between.saturating_mul(num_cols - 1);
            let available_for_cols = net_width.saturating_sub(total_gaps);
            let col_w = available_for_cols / num_cols;

            for i in 0..num_cols {
                let space = if i < num_cols - 1 { space_between } else { 0 };
                columns.push(ColumnGeometry {
                    width: col_w,
                    space,
                });
            }
        }

        Self {
            page_width,
            page_height,
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            gutter,
            header_distance,
            footer_distance,
            orientation,
            columns,
        }
    }

    /// Computes net printable/display content box width in twips (dxa).
    #[inline]
    pub fn content_width(&self) -> u32 {
        self.page_width
            .saturating_sub(self.margin_left.max(0) as u32)
            .saturating_sub(self.margin_right.max(0) as u32)
            .saturating_sub(self.gutter.max(0) as u32)
    }

    /// Computes net printable/display content box height in twips (dxa).
    #[inline]
    pub fn content_height(&self) -> u32 {
        self.page_height
            .saturating_sub(self.margin_top.max(0) as u32)
            .saturating_sub(self.margin_bottom.max(0) as u32)
    }

    /// Returns the number of columns in this section.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.columns.len().max(1)
    }

    /// Returns the width of column `col_idx` in twips.
    pub fn column_width(&self, col_idx: usize) -> u32 {
        self.columns
            .get(col_idx)
            .map(|c| c.width)
            .unwrap_or_else(|| self.content_width())
    }

    /// Returns the X start coordinate of column `col_idx` from the left page boundary in twips.
    pub fn column_x(&self, col_idx: usize) -> u32 {
        let mut x = (self.margin_left.max(0) as u32).saturating_add(self.gutter.max(0) as u32);
        for i in 0..col_idx.min(self.columns.len()) {
            if let Some(col) = self.columns.get(i) {
                x = x.saturating_add(col.width).saturating_add(col.space);
            }
        }
        x
    }

    // --- Unit Conversions ---

    /// Converts twips (twentieths of a point) to typographic points ($pt = twips / 20.0$).
    #[inline]
    pub fn twips_to_points(twips: u32) -> f64 {
        twips as f64 / 20.0
    }

    /// Converts typographic points to twips ($twips = pt \cdot 20$).
    #[inline]
    pub fn points_to_twips(points: f64) -> u32 {
        (points * 20.0).round().max(0.0) as u32
    }

    /// Converts twips to millimeters ($mm = twips \cdot 25.4 / 1440.0$).
    #[inline]
    pub fn twips_to_mm(twips: u32) -> f64 {
        twips as f64 * 25.4 / 1440.0
    }

    /// Converts millimeters to twips ($twips = mm \cdot 1440.0 / 25.4$).
    #[inline]
    pub fn mm_to_twips(mm: f64) -> u32 {
        (mm * 1440.0 / 25.4).round().max(0.0) as u32
    }

    /// Converts twips to inches ($in = twips / 1440.0$).
    #[inline]
    pub fn twips_to_inches(twips: u32) -> f64 {
        twips as f64 / 1440.0
    }

    /// Converts inches to twips ($twips = in \cdot 1440$).
    #[inline]
    pub fn inches_to_twips(inches: f64) -> u32 {
        (inches * 1440.0).round().max(0.0) as u32
    }
}
