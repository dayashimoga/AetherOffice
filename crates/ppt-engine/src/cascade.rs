#![forbid(unsafe_code)]
//! PresentationML 3-Tier Inheritance Resolver: SlideMaster -> SlideLayout -> Slide
//!
//! Resolves:
//! - Placeholder shape property inheritance (transforms, geometry, fills, lines).
//! - Multi-level text paragraph & run formatting cascade (`txStyles`, `defaultTextStyle`, `defRPr`).
//! - Theme color scheme resolution (`a:clrScheme`), color mapping (`p:clrMap`, `p:clrMapOvr`),
//!   and color transformations (tint, shade, lumMod, lumOff, alpha).
//! - Theme font scheme resolution (`+mj-lt`, `+mn-lt`, `+mj-ea`, `+mn-ea`).

use crate::drawingml::{DrawingPath, GeometryCompiler};
use crate::model::*;
use ooxml_opc::error::EngineError;

// =========================================================================
// 1. Resolved Output Data Structures
// =========================================================================

/// A fully resolved shape ready for layout and rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedShape {
    pub id: u32,
    pub name: String,
    pub is_text_box: bool,
    pub placeholder_type: Option<String>,
    pub transform: Transform2D,
    pub geometry: Geometry,
    pub fill: Option<Fill>,
    pub line: Option<LineProperties>,
    pub paragraphs: Vec<ResolvedParagraph>,
}

impl ResolvedShape {
    /// Compiles the shape geometry into evaluated vector paths.
    pub fn evaluate_paths(&self) -> Result<Vec<DrawingPath>, EngineError> {
        let (x, y, w, h) = (
            self.transform.x,
            self.transform.y,
            self.transform.cx as i64,
            self.transform.cy as i64,
        );
        match &self.geometry {
            Geometry::Preset(preset) => GeometryCompiler::compile_preset(preset, x, y, w, h, &[]),
            Geometry::PresetWithAdjust {
                preset,
                adjust_values,
            } => GeometryCompiler::compile_preset(preset, x, y, w, h, adjust_values),
            Geometry::Custom(cust) => GeometryCompiler::compile_custom(cust, x, y, w, h),
        }
    }
}

/// A fully resolved text paragraph.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedParagraph {
    pub level: usize,
    pub align: String, // "l", "ctr", "r", "just", "dist"
    pub mar_l: i32,
    pub mar_r: i32,
    pub indent: i32,
    pub runs: Vec<ResolvedRun>,
}

/// A fully resolved text run.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRun {
    pub text: String,
    pub font_family: String,
    pub font_size_pt: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: Option<String>,
    pub color_rgb: [u8; 3],
    pub color_alpha: u8,
}

// =========================================================================
// 2. 3-Tier Inheritance Cascade Resolver
// =========================================================================

pub struct InheritanceResolver<'a> {
    pub presentation: Option<&'a Presentation>,
    pub master: &'a SlideMaster,
    pub layout: &'a SlideLayout,
    pub theme: Option<&'a Theme>,
}

impl<'a> InheritanceResolver<'a> {
    pub fn new(
        presentation: Option<&'a Presentation>,
        master: &'a SlideMaster,
        layout: &'a SlideLayout,
        theme: Option<&'a Theme>,
    ) -> Self {
        Self {
            presentation,
            master,
            layout,
            theme,
        }
    }

    /// Resolves all shapes on a slide canvas through the 3-tier cascade.
    pub fn resolve_slide(&self, slide: &Slide) -> Vec<ResolvedShape> {
        let mut out = Vec::new();

        // If show_master_shapes is true, inherit non-placeholder master/layout shapes
        if slide.show_master_shapes && self.layout.show_master_shapes {
            for s in &self.master.common_slide_data.shape_tree.shapes {
                if let ShapeElement::Shape(ref master_shape) = s {
                    if master_shape.placeholder.is_none() {
                        out.push(self.resolve_single_shape(master_shape, None, None, slide));
                    }
                }
            }
        }

        if slide.show_master_shapes {
            for s in &self.layout.common_slide_data.shape_tree.shapes {
                if let ShapeElement::Shape(ref layout_shape) = s {
                    if layout_shape.placeholder.is_none() {
                        out.push(self.resolve_single_shape(layout_shape, None, None, slide));
                    }
                }
            }
        }

        // Resolve slide shapes
        for s in &slide.common_slide_data.shape_tree.shapes {
            if let ShapeElement::Shape(ref slide_shape) = s {
                let layout_ph = self.find_layout_placeholder(slide_shape);
                let master_ph = layout_ph
                    .and_then(|l| self.find_master_placeholder(l))
                    .or_else(|| self.find_master_placeholder(slide_shape));

                out.push(self.resolve_single_shape(slide_shape, layout_ph, master_ph, slide));
            }
        }

        out
    }

    /// Resolves a single shape through Slide -> SlideLayout -> SlideMaster tiers.
    pub fn resolve_single_shape(
        &self,
        slide_shape: &Shape,
        layout_shape: Option<&Shape>,
        master_shape: Option<&Shape>,
        slide: &Slide,
    ) -> ResolvedShape {
        // 1. Resolve Transform2D (xfrm)
        let transform = slide_shape
            .properties
            .transform
            .or_else(|| layout_shape.and_then(|l| l.properties.transform))
            .or_else(|| master_shape.and_then(|m| m.properties.transform))
            .unwrap_or_default();

        // 2. Resolve Geometry
        let geometry = slide_shape
            .properties
            .geometry
            .clone()
            .or_else(|| layout_shape.and_then(|l| l.properties.geometry.clone()))
            .or_else(|| master_shape.and_then(|m| m.properties.geometry.clone()))
            .unwrap_or(Geometry::Preset("rect".to_string()));

        // 3. Resolve Fill
        let fill = slide_shape
            .properties
            .fill
            .clone()
            .or_else(|| layout_shape.and_then(|l| l.properties.fill.clone()))
            .or_else(|| master_shape.and_then(|m| m.properties.fill.clone()));

        // 4. Resolve Line
        let line = slide_shape
            .properties
            .line
            .clone()
            .or_else(|| layout_shape.and_then(|l| l.properties.line.clone()))
            .or_else(|| master_shape.and_then(|m| m.properties.line.clone()));

        // 5. Resolve Text Body
        let ph_type = slide_shape
            .placeholder
            .as_ref()
            .map(|p| p.ph_type.clone())
            .or_else(|| {
                layout_shape.and_then(|l| l.placeholder.as_ref().map(|p| p.ph_type.clone()))
            })
            .or_else(|| {
                master_shape.and_then(|m| m.placeholder.as_ref().map(|p| p.ph_type.clone()))
            });

        let paragraphs = self.resolve_text_body(
            slide_shape.text_body.as_ref(),
            layout_shape.and_then(|l| l.text_body.as_ref()),
            master_shape.and_then(|m| m.text_body.as_ref()),
            ph_type.as_deref(),
            slide,
        );

        ResolvedShape {
            id: slide_shape.id,
            name: slide_shape.name.clone(),
            is_text_box: slide_shape.is_text_box,
            placeholder_type: ph_type,
            transform,
            geometry,
            fill,
            line,
            paragraphs,
        }
    }

    // =====================================================================
    // Placeholder Matching Logic
    // =====================================================================

    /// Finds matching placeholder on SlideLayout.
    pub fn find_layout_placeholder(&self, shape: &Shape) -> Option<&Shape> {
        let ph = shape.placeholder.as_ref()?;
        self.match_placeholder_in_shapes(ph, &self.layout.common_slide_data.shape_tree.shapes)
    }

    /// Finds matching placeholder on SlideMaster.
    pub fn find_master_placeholder(&self, shape: &Shape) -> Option<&Shape> {
        let ph = shape.placeholder.as_ref()?;
        self.match_placeholder_in_shapes(ph, &self.master.common_slide_data.shape_tree.shapes)
    }

    fn match_placeholder_in_shapes<'b>(
        &self,
        target_ph: &Placeholder,
        shapes: &'b [ShapeElement],
    ) -> Option<&'b Shape> {
        // Step 1: Match by idx if present on target
        if let Some(target_idx) = target_ph.index {
            for elem in shapes {
                if let ShapeElement::Shape(ref s) = elem {
                    if let Some(ref ph) = s.placeholder {
                        if ph.index == Some(target_idx) {
                            return Some(s);
                        }
                    }
                }
            }
        }

        // Step 2: Match by exact ph_type
        for elem in shapes {
            if let ShapeElement::Shape(ref s) = elem {
                if let Some(ref ph) = s.placeholder {
                    if ph.ph_type == target_ph.ph_type {
                        return Some(s);
                    }
                }
            }
        }

        // Step 3: ECMA-376 Fallback matching
        // ctrTitle -> title
        // subTitle -> body
        let fallback_type = match target_ph.ph_type.as_str() {
            "ctrTitle" => Some("title"),
            "subTitle" => Some("body"),
            _ => None,
        };

        if let Some(fb) = fallback_type {
            for elem in shapes {
                if let ShapeElement::Shape(ref s) = elem {
                    if let Some(ref ph) = s.placeholder {
                        if ph.ph_type == fb {
                            return Some(s);
                        }
                    }
                }
            }
        }

        None
    }

    // =====================================================================
    // 3. Text Formatting Cascade
    // =====================================================================

    fn resolve_text_body(
        &self,
        slide_body: Option<&TextBody>,
        layout_body: Option<&TextBody>,
        master_body: Option<&TextBody>,
        ph_type: Option<&str>,
        slide: &Slide,
    ) -> Vec<ResolvedParagraph> {
        let body = match slide_body.or(layout_body).or(master_body) {
            Some(b) => b,
            None => return Vec::new(),
        };

        let mut out = Vec::new();

        for p in &body.paragraphs {
            let lvl = p.properties.as_ref().and_then(|pr| pr.level).unwrap_or(0);

            // Cascade paragraph properties:
            // 1. Direct paragraph formatting
            // 2. Shape-level lstStyle (if any)
            // 3. Master txStyles (titleStyle, bodyStyle, otherStyle)
            // 4. Presentation defaultTextStyle
            let align = self.resolve_para_align(p, body, ph_type, lvl);
            let mar_l = self.resolve_para_mar_l(p, body, ph_type, lvl);
            let mar_r = self.resolve_para_mar_r(p, body, ph_type, lvl);
            let indent = self.resolve_para_indent(p, body, ph_type, lvl);

            // Resolve runs
            let mut resolved_runs = Vec::new();
            for run in &p.runs {
                if let TextRunElement::Text(ref text, ref r_pr) = run {
                    resolved_runs.push(self.resolve_run(
                        text,
                        r_pr.as_ref(),
                        p,
                        ph_type,
                        lvl,
                        slide,
                    ));
                } else if let TextRunElement::Field {
                    ref text,
                    ref properties,
                    ..
                } = run
                {
                    let display_text = text.as_deref().unwrap_or("1");
                    resolved_runs.push(self.resolve_run(
                        display_text,
                        properties.as_ref(),
                        p,
                        ph_type,
                        lvl,
                        slide,
                    ));
                }
            }

            out.push(ResolvedParagraph {
                level: lvl,
                align,
                mar_l,
                mar_r,
                indent,
                runs: resolved_runs,
            });
        }

        out
    }

    fn resolve_para_align(
        &self,
        p: &TextParagraph,
        body: &TextBody,
        ph_type: Option<&str>,
        lvl: usize,
    ) -> String {
        p.properties
            .as_ref()
            .and_then(|pr| pr.align.clone())
            .or_else(|| {
                body.list_style.as_ref().and_then(|l| {
                    l.levels
                        .get(lvl)
                        .and_then(|opt| opt.as_ref().and_then(|pr| pr.align.clone()))
                })
            })
            .or_else(|| self.get_master_level_prop(ph_type, lvl, |pr| pr.align.clone()))
            .or_else(|| {
                self.presentation
                    .and_then(|pres| pres.default_text_style.as_ref())
                    .and_then(|d| {
                        d.levels
                            .get(lvl)
                            .and_then(|opt| opt.as_ref().and_then(|pr| pr.align.clone()))
                    })
            })
            .unwrap_or_else(|| "l".to_string())
    }

    fn resolve_para_mar_l(
        &self,
        p: &TextParagraph,
        body: &TextBody,
        ph_type: Option<&str>,
        lvl: usize,
    ) -> i32 {
        p.properties
            .as_ref()
            .and_then(|pr| pr.mar_l)
            .or_else(|| {
                body.list_style.as_ref().and_then(|l| {
                    l.levels
                        .get(lvl)
                        .and_then(|opt| opt.as_ref().and_then(|pr| pr.mar_l))
                })
            })
            .or_else(|| self.get_master_level_prop(ph_type, lvl, |pr| pr.mar_l))
            .or_else(|| {
                self.presentation
                    .and_then(|pres| pres.default_text_style.as_ref())
                    .and_then(|d| {
                        d.levels
                            .get(lvl)
                            .and_then(|opt| opt.as_ref().and_then(|pr| pr.mar_l))
                    })
            })
            .unwrap_or(0)
    }

    fn resolve_para_mar_r(
        &self,
        p: &TextParagraph,
        body: &TextBody,
        ph_type: Option<&str>,
        lvl: usize,
    ) -> i32 {
        p.properties
            .as_ref()
            .and_then(|pr| pr.mar_r)
            .or_else(|| {
                body.list_style.as_ref().and_then(|l| {
                    l.levels
                        .get(lvl)
                        .and_then(|opt| opt.as_ref().and_then(|pr| pr.mar_r))
                })
            })
            .or_else(|| self.get_master_level_prop(ph_type, lvl, |pr| pr.mar_r))
            .unwrap_or(0)
    }

    fn resolve_para_indent(
        &self,
        p: &TextParagraph,
        body: &TextBody,
        ph_type: Option<&str>,
        lvl: usize,
    ) -> i32 {
        p.properties
            .as_ref()
            .and_then(|pr| pr.indent)
            .or_else(|| {
                body.list_style.as_ref().and_then(|l| {
                    l.levels
                        .get(lvl)
                        .and_then(|opt| opt.as_ref().and_then(|pr| pr.indent))
                })
            })
            .or_else(|| self.get_master_level_prop(ph_type, lvl, |pr| pr.indent))
            .unwrap_or(0)
    }

    fn get_master_level_prop<T, F>(&self, ph_type: Option<&str>, lvl: usize, f: F) -> Option<T>
    where
        F: Fn(&TextParagraphProperties) -> Option<T>,
    {
        let style = match ph_type {
            Some("title") | Some("ctrTitle") | Some("subTitle") => {
                self.master.text_styles.title_style.as_ref()
            }
            Some("body") | Some("obj") => self.master.text_styles.body_style.as_ref(),
            _ => self.master.text_styles.other_style.as_ref().or(self
                .master
                .text_styles
                .body_style
                .as_ref()),
        };

        style.and_then(|s| {
            s.levels
                .get(lvl)
                .and_then(|opt| opt.as_ref().and_then(&f))
                .or_else(|| s.default_paragraph_properties.as_ref().and_then(&f))
        })
    }

    fn resolve_run(
        &self,
        text: &str,
        r_pr: Option<&TextRunProperties>,
        p: &TextParagraph,
        ph_type: Option<&str>,
        lvl: usize,
        slide: &Slide,
    ) -> ResolvedRun {
        // Font size in pt (1800 hundredths of pt = 18.0 pt)
        let sz_100th = r_pr
            .and_then(|r| r.size)
            .or_else(|| {
                p.properties
                    .as_ref()
                    .and_then(|pr| pr.default_run_properties.as_ref().and_then(|r| r.size))
            })
            .or_else(|| {
                self.get_master_level_prop(ph_type, lvl, |pr| {
                    pr.default_run_properties.as_ref().and_then(|r| r.size)
                })
            })
            .or_else(|| {
                self.presentation
                    .and_then(|pres| pres.default_text_style.as_ref())
                    .and_then(|d| {
                        d.levels.get(lvl).and_then(|opt| {
                            opt.as_ref().and_then(|pr| {
                                pr.default_run_properties.as_ref().and_then(|r| r.size)
                            })
                        })
                    })
            })
            .unwrap_or(1800);
        let font_size_pt = sz_100th as f32 / 100.0;

        // Bold
        let bold = r_pr
            .and_then(|r| r.bold)
            .or_else(|| {
                p.properties
                    .as_ref()
                    .and_then(|pr| pr.default_run_properties.as_ref().and_then(|r| r.bold))
            })
            .or_else(|| {
                self.get_master_level_prop(ph_type, lvl, |pr| {
                    pr.default_run_properties.as_ref().and_then(|r| r.bold)
                })
            })
            .unwrap_or(false);

        // Italic
        let italic = r_pr
            .and_then(|r| r.italic)
            .or_else(|| {
                p.properties
                    .as_ref()
                    .and_then(|pr| pr.default_run_properties.as_ref().and_then(|r| r.italic))
            })
            .or_else(|| {
                self.get_master_level_prop(ph_type, lvl, |pr| {
                    pr.default_run_properties.as_ref().and_then(|r| r.italic)
                })
            })
            .unwrap_or(false);

        // Underline
        let underline = r_pr.and_then(|r| r.underline.clone());

        // Font Family (resolving +mj-lt, +mn-lt)
        let raw_font = r_pr
            .and_then(|r| r.font_latin.clone())
            .or_else(|| {
                p.properties.as_ref().and_then(|pr| {
                    pr.default_run_properties
                        .as_ref()
                        .and_then(|r| r.font_latin.clone())
                })
            })
            .or_else(|| {
                self.get_master_level_prop(ph_type, lvl, |pr| {
                    pr.default_run_properties
                        .as_ref()
                        .and_then(|r| r.font_latin.clone())
                })
            })
            .unwrap_or_else(|| "+mn-lt".to_string());

        let font_family = self.resolve_theme_font(&raw_font);

        // Color
        let color_choice = r_pr
            .and_then(|r| r.color.clone())
            .or_else(|| {
                p.properties.as_ref().and_then(|pr| {
                    pr.default_run_properties
                        .as_ref()
                        .and_then(|r| r.color.clone())
                })
            })
            .or_else(|| {
                self.get_master_level_prop(ph_type, lvl, |pr| {
                    pr.default_run_properties
                        .as_ref()
                        .and_then(|r| r.color.clone())
                })
            });

        let (color_rgb, color_alpha) = self.resolve_color(color_choice.as_ref(), slide);

        ResolvedRun {
            text: text.to_string(),
            font_family,
            font_size_pt,
            bold,
            italic,
            underline,
            color_rgb,
            color_alpha,
        }
    }

    // =====================================================================
    // 4. Theme Font Scheme Resolution
    // =====================================================================

    pub fn resolve_theme_font(&self, typeface: &str) -> String {
        if let Some(theme) = self.theme {
            match typeface {
                "+mj-lt" => {
                    if let Some(ref latin) = theme.font_scheme.major_font.latin {
                        if !latin.is_empty() {
                            return latin.clone();
                        }
                    }
                    "Calibri".to_string()
                }
                "+mn-lt" => {
                    if let Some(ref latin) = theme.font_scheme.minor_font.latin {
                        if !latin.is_empty() {
                            return latin.clone();
                        }
                    }
                    "Calibri".to_string()
                }
                "+mj-ea" => theme
                    .font_scheme
                    .major_font
                    .ea
                    .clone()
                    .unwrap_or_else(|| "MS Gothic".to_string()),
                "+mn-ea" => theme
                    .font_scheme
                    .minor_font
                    .ea
                    .clone()
                    .unwrap_or_else(|| "MS Gothic".to_string()),
                _ => typeface.to_string(),
            }
        } else {
            match typeface {
                "+mj-lt" | "+mn-lt" => "Calibri".to_string(),
                _ => typeface.to_string(),
            }
        }
    }

    // =====================================================================
    // 5. Theme Color Scheme Resolution & Transforms
    // =====================================================================

    pub fn resolve_color(&self, choice: Option<&ColorChoice>, slide: &Slide) -> ([u8; 3], u8) {
        let choice = match choice {
            Some(c) => c,
            None => return ([0, 0, 0], 255),
        };

        match choice {
            ColorChoice::Srgb(ref hex) => {
                let rgb = Self::parse_hex_rgb(hex);
                (rgb, 255)
            }
            ColorChoice::System(_, ref last_clr) => {
                let hex = last_clr.as_deref().unwrap_or("000000");
                (Self::parse_hex_rgb(hex), 255)
            }
            ColorChoice::Scheme(ref token, ref transforms) => {
                // 1. Map scheme token through effective ColorMap
                let mapped_token = self.get_effective_color_map(slide).map_token(token);

                // 2. Resolve mapped token from Theme
                let base_rgb = self.resolve_theme_scheme_token(mapped_token);

                // 3. Apply color transformations in order
                Self::apply_transforms(base_rgb, transforms)
            }
        }
    }

    fn get_effective_color_map<'b>(&'b self, slide: &'b Slide) -> &'b ColorMap {
        if let Some(ColorMapOverride::Override(ref map)) = slide.color_map_override {
            return map;
        }
        if let Some(ColorMapOverride::Override(ref map)) = self.layout.color_map_override {
            return map;
        }
        &self.master.color_map
    }

    fn resolve_theme_scheme_token(&self, token: &str) -> [u8; 3] {
        if let Some(theme) = self.theme {
            if let Some(c) = theme.color_scheme.colors.get(token) {
                match c {
                    ColorChoice::Srgb(ref hex) => return Self::parse_hex_rgb(hex),
                    ColorChoice::System(_, ref last) => {
                        return Self::parse_hex_rgb(last.as_deref().unwrap_or("000000"))
                    }
                    _ => {}
                }
            }
        }

        // Clean-room default Office theme fallback
        match token {
            "dk1" => [0, 0, 0],
            "lt1" => [255, 255, 255],
            "dk2" => [14, 40, 65],
            "lt2" => [232, 232, 232],
            "accent1" => [21, 96, 130],
            "accent2" => [233, 113, 50],
            "accent3" => [25, 107, 36],
            "accent4" => [15, 158, 213],
            "accent5" => [160, 43, 147],
            "accent6" => [78, 167, 46],
            "hlink" => [70, 120, 134],
            "folHlink" => [150, 96, 125],
            _ => [0, 0, 0],
        }
    }

    pub fn parse_hex_rgb(hex: &str) -> [u8; 3] {
        let clean = hex.trim_start_matches('#');
        if clean.len() >= 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
            [r, g, b]
        } else {
            [0, 0, 0]
        }
    }

    /// Applies DrawingML color transformations (tint, shade, lumMod, lumOff, alpha).
    pub fn apply_transforms(mut rgb: [u8; 3], transforms: &[ColorTransform]) -> ([u8; 3], u8) {
        let mut alpha = 255u8;

        for t in transforms {
            match *t {
                ColorTransform::Tint(val) => {
                    // Linear interpolation towards white: (1 - t) * C + t * 255
                    let t_ratio = (val as f32 / 100_000.0).clamp(0.0, 1.0);
                    rgb[0] = (rgb[0] as f32 + (255.0 - rgb[0] as f32) * t_ratio).round() as u8;
                    rgb[1] = (rgb[1] as f32 + (255.0 - rgb[1] as f32) * t_ratio).round() as u8;
                    rgb[2] = (rgb[2] as f32 + (255.0 - rgb[2] as f32) * t_ratio).round() as u8;
                }
                ColorTransform::Shade(val) => {
                    // Linear interpolation towards black: s * C
                    let s_ratio = (val as f32 / 100_000.0).clamp(0.0, 1.0);
                    rgb[0] = (rgb[0] as f32 * s_ratio).round() as u8;
                    rgb[1] = (rgb[1] as f32 * s_ratio).round() as u8;
                    rgb[2] = (rgb[2] as f32 * s_ratio).round() as u8;
                }
                ColorTransform::LumMod(m) => {
                    let (h, s, l) = Self::rgb_to_hsl(rgb);
                    let m_ratio = m as f32 / 100_000.0;
                    let new_l = (l * m_ratio).clamp(0.0, 1.0);
                    rgb = Self::hsl_to_rgb(h, s, new_l);
                }
                ColorTransform::LumOff(o) => {
                    let (h, s, l) = Self::rgb_to_hsl(rgb);
                    let o_ratio = o as f32 / 100_000.0;
                    let new_l = (l + o_ratio).clamp(0.0, 1.0);
                    rgb = Self::hsl_to_rgb(h, s, new_l);
                }
                ColorTransform::Alpha(val) => {
                    alpha = ((val as u64 * 255) / 100_000).min(255) as u8;
                }
            }
        }

        (rgb, alpha)
    }

    fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
        let r = rgb[0] as f32 / 255.0;
        let g = rgb[1] as f32 / 255.0;
        let b = rgb[2] as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0.0, 0.0, l);
        }

        let s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        let mut h = if max == r {
            (g - b) / delta + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };
        h /= 6.0;

        (h, s, l)
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
        if s == 0.0 {
            let gray = (l * 255.0).round() as u8;
            return [gray, gray, gray];
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = Self::hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = Self::hue_to_rgb(p, q, h);
        let b = Self::hue_to_rgb(p, q, h - 1.0 / 3.0);

        [
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        ]
    }

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }
}
