#![forbid(unsafe_code)]
//! PresentationML and DrawingML Document Object Model (DOM)
//!
//! Strictly conforms to ECMA-376 Part 1:
//! - §19: PresentationML (Presentation, SlideMaster, SlideLayout, Slide, NotesSlide)
//! - §20: DrawingML (Shapes, Transform2D, Geometry, Fills, Lines, TextBody, TextStyles, Theme)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// =========================================================================
// 1. Raw XML Preservation (Zero-Loss Round-Trip Fidelity)
// =========================================================================

/// Preserves unrecognized XML elements and attributes for zero-loss round-trip fidelity.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RawXmlNode {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<RawXmlNode>,
}

// =========================================================================
// 2. Presentation Root (`p:presentation`)
// =========================================================================

/// Presentation document root: `<p:presentation>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Presentation {
    pub slide_masters: Vec<SlideMasterIdRef>,
    pub slides: Vec<SlideIdRef>,
    pub slide_size: SlideSize,
    pub notes_size: NotesSize,
    pub default_text_style: Option<TextListStyle>,
    pub save_subset_fonts: bool,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Slide master relationship reference: `<p:sldMasterId>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SlideMasterIdRef {
    pub id: u32,
    pub r_id: String,
}

/// Slide relationship reference: `<p:sldId>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SlideIdRef {
    pub id: u32,
    pub r_id: String,
}

/// Slide canvas dimensions: `<p:sldSz>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlideSize {
    pub cx: u32, // in EMUs (default 12,192,000 for 16:9 widescreen = 13.33 in)
    pub cy: u32, // in EMUs (default 6,858,000 for 16:9 widescreen = 7.5 in)
}

impl Default for SlideSize {
    fn default() -> Self {
        Self {
            cx: 12_192_000,
            cy: 6_858_000,
        }
    }
}

/// Notes canvas dimensions: `<p:notesSz>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotesSize {
    pub cx: u32,
    pub cy: u32,
}

impl Default for NotesSize {
    fn default() -> Self {
        Self {
            cx: 6_858_000,
            cy: 9_144_000,
        }
    }
}

// =========================================================================
// 3. Slide Master (`p:sldMaster`)
// =========================================================================

/// Slide master part root: `<p:sldMaster>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideMaster {
    pub common_slide_data: CommonSlideData,
    pub color_map: ColorMap,
    pub slide_layouts: Vec<SlideLayoutIdRef>,
    pub text_styles: MasterTextStyles,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Slide layout relationship reference in master: `<p:sldLayoutId>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SlideLayoutIdRef {
    pub id: u32,
    pub r_id: String,
}

/// Master text styling hierarchy: `<p:txStyles>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MasterTextStyles {
    pub title_style: Option<TextListStyle>,
    pub body_style: Option<TextListStyle>,
    pub other_style: Option<TextListStyle>,
}

// =========================================================================
// 4. Slide Layout (`p:sldLayout`)
// =========================================================================

/// Slide layout part root: `<p:sldLayout>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideLayout {
    pub layout_type: Option<String>, // e.g. "title", "ctrTitle", "subTitle", "body", "blank"
    pub matching_name: Option<String>,
    pub preserve: bool,
    pub show_master_shapes: bool,
    pub common_slide_data: CommonSlideData,
    pub color_map_override: Option<ColorMapOverride>,
    pub raw_elements: Vec<RawXmlNode>,
}

// =========================================================================
// 5. Slide (`p:sld`) & Notes Slide (`p:notes`)
// =========================================================================

/// Slide part root: `<p:sld>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Slide {
    pub show_master_shapes: bool,
    pub show_master_ph_anim: bool,
    pub common_slide_data: CommonSlideData,
    pub color_map_override: Option<ColorMapOverride>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Notes slide part root: `<p:notes>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotesSlide {
    pub show_master_shapes: bool,
    pub common_slide_data: CommonSlideData,
    pub color_map_override: Option<ColorMapOverride>,
    pub raw_elements: Vec<RawXmlNode>,
}

// =========================================================================
// 6. Common Slide Data (`p:cSld`) & Background
// =========================================================================

/// Common slide data present in masters, layouts, slides, and notes: `<p:cSld>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommonSlideData {
    pub name: Option<String>,
    pub background: Option<Background>,
    pub shape_tree: ShapeTree,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Background formatting: `<p:bg>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Background {
    pub fill: Option<Fill>,
    pub bg_ref_idx: Option<u32>,
    pub bg_ref_color: Option<ColorChoice>,
}

// =========================================================================
// 7. Shape Tree & Elements (`p:spTree`, `p:sp`, `p:grpSp`)
// =========================================================================

/// Hierarchical container of shapes on a slide canvas: `<p:spTree>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ShapeTree {
    pub group_transform: Option<GroupTransform2D>,
    pub shapes: Vec<ShapeElement>,
}

/// Element inside a shape tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeElement {
    Shape(Box<Shape>),
    Group(GroupShape),
    GraphicFrame(GraphicFrame),
}

/// DrawingML shape: `<p:sp>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Shape {
    pub id: u32,
    pub name: String,
    pub is_text_box: bool,
    pub placeholder: Option<Placeholder>,
    pub properties: ShapeProperties,
    pub text_body: Option<TextBody>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Group shape container: `<p:grpSp>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GroupShape {
    pub id: u32,
    pub name: String,
    pub transform: Option<GroupTransform2D>,
    pub children: Vec<ShapeElement>,
}

/// Graphic frame containing tables, charts, or smartart: `<p:graphicFrame>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GraphicFrame {
    pub id: u32,
    pub name: String,
    pub transform: Option<Transform2D>,
    pub raw_elements: Vec<RawXmlNode>,
}

/// Placeholder shape configuration: `<p:ph>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Placeholder {
    pub ph_type: String, // "title", "ctrTitle", "subTitle", "body", "dt", "ftr", "sldNum", etc.
    pub index: Option<u32>,
    pub orientation: Option<String>,
    pub size: Option<String>,
}

/// Shape visual formatting properties: `<p:spPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ShapeProperties {
    pub transform: Option<Transform2D>,
    pub geometry: Option<Geometry>,
    pub fill: Option<Fill>,
    pub line: Option<LineProperties>,
}

/// 2D affine transformation in EMUs: `<a:xfrm>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Transform2D {
    pub x: i64,
    pub y: i64,
    pub cx: u32,
    pub cy: u32,
    pub rot: i32, // in 60,000ths of a degree (default 0)
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Transform2D {
    pub fn new(x: i64, y: i64, cx: u32, cy: u32) -> Self {
        Self {
            x,
            y,
            cx,
            cy,
            rot: 0,
            flip_h: false,
            flip_v: false,
        }
    }
}

/// Group 2D coordinate transformation: `<a:grpSpPr><a:xfrm>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupTransform2D {
    pub x: i64,
    pub y: i64,
    pub cx: u32,
    pub cy: u32,
    pub ch_x: i64,
    pub ch_y: i64,
    pub ch_cx: u32,
    pub ch_cy: u32,
}

use crate::drawingml::{CustomGeometry, GeomGuide};

/// Shape geometry: `<a:prstGeom>` or `<a:custGeom>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    Preset(String), // e.g. "rect", "roundRect", "ellipse"
    PresetWithAdjust {
        preset: String,
        adjust_values: Vec<GeomGuide>,
    },
    Custom(Box<CustomGeometry>),
}

impl Geometry {
    pub fn preset_name(&self) -> Option<&str> {
        match self {
            Self::Preset(name) => Some(name.as_str()),
            Self::PresetWithAdjust { preset, .. } => Some(preset.as_str()),
            Self::Custom(_) => None,
        }
    }
}

/// Fill formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Fill {
    NoFill,
    Solid(ColorChoice),
    Gradient,
    Pattern,
    Blip(String), // relationship id for image
}

/// Line stroke formatting: `<a:ln>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LineProperties {
    pub width: Option<u32>, // in EMUs (1pt = 12700 EMUs)
    pub fill: Option<Fill>,
    pub dash_style: Option<String>,
}

// =========================================================================
// 8. DrawingML Text Body & Typography (`p:txBody`, `a:p`, `a:r`)
// =========================================================================

/// Rich text body of a shape: `<p:txBody>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextBody {
    pub body_properties: BodyProperties,
    pub list_style: Option<TextListStyle>,
    pub paragraphs: Vec<TextParagraph>,
}

/// Text body level properties: `<a:bodyPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BodyProperties {
    pub anchor: Option<String>, // "t" (top), "ctr" (center), "b" (bottom)
    pub vertical: Option<String>,
    pub wrap: Option<String>, // "none", "square"
    pub l_ins: Option<i32>,   // left inset in EMUs
    pub t_ins: Option<i32>,   // top inset in EMUs
    pub r_ins: Option<i32>,   // right inset in EMUs
    pub b_ins: Option<i32>,   // bottom inset in EMUs
    pub num_cols: Option<u32>,
}

/// Paragraph within a text body: `<a:p>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextParagraph {
    pub properties: Option<TextParagraphProperties>,
    pub runs: Vec<TextRunElement>,
    pub end_paragraph_run_properties: Option<TextRunProperties>,
}

/// Run-level text element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextRunElement {
    Text(String, Option<TextRunProperties>),
    LineBreak(Option<TextRunProperties>),
    Field {
        id: String,
        field_type: String, // e.g. "slNum", "datetime"
        text: Option<String>,
        properties: Option<TextRunProperties>,
    },
}

impl TextRunElement {
    pub fn get_text(&self) -> &str {
        match self {
            Self::Text(ref t, _) => t.as_str(),
            Self::LineBreak(_) => "\n",
            Self::Field { ref text, .. } => text.as_deref().unwrap_or(""),
        }
    }
}

/// Text paragraph formatting: `<a:pPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextParagraphProperties {
    pub level: Option<usize>,  // 0-based list level (0..8)
    pub align: Option<String>, // "l", "ctr", "r", "just", "dist"
    pub mar_l: Option<i32>,    // left margin in EMUs
    pub mar_r: Option<i32>,
    pub indent: Option<i32>, // first line indent in EMUs
    pub def_tab_sz: Option<u32>,
    pub default_run_properties: Option<TextRunProperties>,
    pub space_before: Option<i32>,
    pub space_after: Option<i32>,
    pub line_spacing: Option<i32>,
}

/// Text run formatting: `<a:rPr>` or `<a:defRPr>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextRunProperties {
    pub size: Option<u32>, // in 100ths of a point (e.g. 1800 = 18pt)
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<String>,
    pub strike: Option<String>,
    pub kern: Option<u32>,
    pub baseline: Option<i32>,
    pub color: Option<ColorChoice>,
    pub font_latin: Option<String>,
    pub font_ea: Option<String>,
    pub font_cs: Option<String>,
    pub lang: Option<String>,
}

/// Multi-level list paragraph formatting style: `<p:defaultTextStyle>`, `<p:titleStyle>`, `<p:bodyStyle>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextListStyle {
    pub default_paragraph_properties: Option<TextParagraphProperties>,
    pub levels: Vec<Option<TextParagraphProperties>>, // indexed 0..8 for lvl1pPr..lvl9pPr
}

// =========================================================================
// 9. DrawingML Theme & Colors (`a:theme`)
// =========================================================================

/// Theme part root: `<a:theme>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub color_scheme: ColorScheme,
    pub font_scheme: FontScheme,
}

/// Theme color scheme: `<a:clrScheme>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorScheme {
    pub name: String,
    pub colors: BTreeMap<String, ColorChoice>, // "dk1", "lt1", "dk2", "lt2", "accent1".."accent6", "hlink", "folHlink"
}

/// Theme font scheme: `<a:fontScheme>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FontScheme {
    pub name: String,
    pub major_font: FontCollection,
    pub minor_font: FontCollection,
}

/// Font collection for a theme role (major or minor): `<a:majorFont>` / `<a:minorFont>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FontCollection {
    pub latin: Option<String>,
    pub ea: Option<String>,
    pub cs: Option<String>,
    pub scripts: BTreeMap<String, String>,
}

/// DrawingML Color representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorChoice {
    Srgb(String),                        // 6-hex RGB e.g. "156082"
    System(String, Option<String>),      // system color name, lastClr hex
    Scheme(String, Vec<ColorTransform>), // scheme token (e.g. "tx1", "bg1", "accent1") + transforms
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Srgb("000000".to_string())
    }
}

/// DrawingML color transformation modifying luminance, tint, shade, or opacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorTransform {
    Tint(u32),   // 0..100,000 (percentage)
    Shade(u32),  // 0..100,000
    LumMod(u32), // 0..100,000
    LumOff(u32), // 0..100,000
    Alpha(u32),  // 0..100,000
}

// =========================================================================
// 10. Color Mapping (`p:clrMap` & `p:clrMapOvr`)
// =========================================================================

/// Maps semantic presentation color roles to theme color scheme tokens: `<p:clrMap>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ColorMap {
    pub bg1: String,
    pub tx1: String,
    pub bg2: String,
    pub tx2: String,
    pub accent1: String,
    pub accent2: String,
    pub accent3: String,
    pub accent4: String,
    pub accent5: String,
    pub accent6: String,
    pub hlink: String,
    pub fol_hlink: String,
}

impl ColorMap {
    /// Maps a scheme color token (e.g. "bg1", "tx1") to the target theme token.
    pub fn map_token<'a>(&'a self, token: &'a str) -> &'a str {
        match token {
            "bg1" => &self.bg1,
            "tx1" => &self.tx1,
            "bg2" => &self.bg2,
            "tx2" => &self.tx2,
            "accent1" => &self.accent1,
            "accent2" => &self.accent2,
            "accent3" => &self.accent3,
            "accent4" => &self.accent4,
            "accent5" => &self.accent5,
            "accent6" => &self.accent6,
            "hlink" => &self.hlink,
            "folHlink" => &self.fol_hlink,
            _ => token,
        }
    }
}

/// Color map override on layout or slide: `<p:clrMapOvr>`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorMapOverride {
    #[default]
    UseMaster,
    Override(Box<ColorMap>),
}
