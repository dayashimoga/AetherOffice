#![forbid(unsafe_code)]
//! PresentationML & DrawingML Deterministic XML Serializer
//!
//! Emits compliant ECMA-376 XML for:
//! - `ppt/presentation.xml`
//! - `ppt/slideMasters/*.xml`
//! - `ppt/slideLayouts/*.xml`
//! - `ppt/slides/*.xml`
//! - `ppt/notesSlides/*.xml`
//! - `ppt/theme/*.xml`

use ooxml_opc::error::EngineError;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;

use crate::animation::*;
use crate::drawingml::{CustomGeometry, PathInstruction};
use crate::model::*;

pub const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";

// =========================================================================
// 1. Presentation Serializer (`ppt/presentation.xml`)
// =========================================================================

pub fn serialize_presentation(pres: &Presentation) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("p:presentation");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute((
        "xmlns:r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ));
    root.push_attribute((
        "xmlns:p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ));
    if pres.save_subset_fonts {
        root.push_attribute(("saveSubsetFonts", "1"));
    }
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    // sldMasterIdLst
    if !pres.slide_masters.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("p:sldMasterIdLst")))
            .map_err(EngineError::Io)?;
        for m in &pres.slide_masters {
            let mut elem = BytesStart::new("p:sldMasterId");
            elem.push_attribute(("id", m.id.to_string().as_str()));
            elem.push_attribute(("r:id", m.r_id.as_str()));
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("p:sldMasterIdLst")))
            .map_err(EngineError::Io)?;
    }

    // sldIdLst
    if !pres.slides.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("p:sldIdLst")))
            .map_err(EngineError::Io)?;
        for s in &pres.slides {
            let mut elem = BytesStart::new("p:sldId");
            elem.push_attribute(("id", s.id.to_string().as_str()));
            elem.push_attribute(("r:id", s.r_id.as_str()));
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("p:sldIdLst")))
            .map_err(EngineError::Io)?;
    }

    // sldSz
    let mut sld_sz = BytesStart::new("p:sldSz");
    sld_sz.push_attribute(("cx", pres.slide_size.cx.to_string().as_str()));
    sld_sz.push_attribute(("cy", pres.slide_size.cy.to_string().as_str()));
    writer
        .write_event(Event::Empty(sld_sz))
        .map_err(EngineError::Io)?;

    // notesSz
    let mut notes_sz = BytesStart::new("p:notesSz");
    notes_sz.push_attribute(("cx", pres.notes_size.cx.to_string().as_str()));
    notes_sz.push_attribute(("cy", pres.notes_size.cy.to_string().as_str()));
    writer
        .write_event(Event::Empty(notes_sz))
        .map_err(EngineError::Io)?;

    // defaultTextStyle
    if let Some(ref style) = pres.default_text_style {
        serialize_text_list_style(&mut writer, "p:defaultTextStyle", style)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:presentation")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

// =========================================================================
// 2. Slide Master Serializer (`ppt/slideMasters/*.xml`)
// =========================================================================

pub fn serialize_slide_master(master: &SlideMaster) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("p:sldMaster");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute((
        "xmlns:r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ));
    root.push_attribute((
        "xmlns:p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ));
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    // cSld
    serialize_common_slide_data(&mut writer, &master.common_slide_data)?;

    // clrMap
    serialize_color_map(&mut writer, &master.color_map)?;

    // sldLayoutIdLst
    if !master.slide_layouts.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("p:sldLayoutIdLst")))
            .map_err(EngineError::Io)?;
        for l in &master.slide_layouts {
            let mut elem = BytesStart::new("p:sldLayoutId");
            elem.push_attribute(("id", l.id.to_string().as_str()));
            elem.push_attribute(("r:id", l.r_id.as_str()));
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("p:sldLayoutIdLst")))
            .map_err(EngineError::Io)?;
    }

    // txStyles
    writer
        .write_event(Event::Start(BytesStart::new("p:txStyles")))
        .map_err(EngineError::Io)?;
    if let Some(ref title) = master.text_styles.title_style {
        serialize_text_list_style(&mut writer, "p:titleStyle", title)?;
    }
    if let Some(ref body) = master.text_styles.body_style {
        serialize_text_list_style(&mut writer, "p:bodyStyle", body)?;
    }
    if let Some(ref other) = master.text_styles.other_style {
        serialize_text_list_style(&mut writer, "p:otherStyle", other)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("p:txStyles")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("p:sldMaster")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

// =========================================================================
// 3. Slide Layout Serializer (`ppt/slideLayouts/*.xml`)
// =========================================================================

pub fn serialize_slide_layout(layout: &SlideLayout) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("p:sldLayout");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute((
        "xmlns:r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ));
    root.push_attribute((
        "xmlns:p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ));
    if let Some(ref t) = layout.layout_type {
        root.push_attribute(("type", t.as_str()));
    }
    if let Some(ref m) = layout.matching_name {
        root.push_attribute(("matchingName", m.as_str()));
    }
    if layout.preserve {
        root.push_attribute(("preserve", "1"));
    }
    if !layout.show_master_shapes {
        root.push_attribute(("showMasterSp", "0"));
    }
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    // cSld
    serialize_common_slide_data(&mut writer, &layout.common_slide_data)?;

    // clrMapOvr
    if let Some(ref ovr) = layout.color_map_override {
        serialize_color_map_override(&mut writer, ovr)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:sldLayout")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

// =========================================================================
// 4. Slide Serializer (`ppt/slides/*.xml`)
// =========================================================================

pub fn serialize_slide(slide: &Slide) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("p:sld");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute((
        "xmlns:r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ));
    root.push_attribute((
        "xmlns:p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ));
    if !slide.show_master_shapes {
        root.push_attribute(("showMasterSp", "0"));
    }
    if slide.show_master_ph_anim {
        root.push_attribute(("showMasterPhAnim", "1"));
    }
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    // cSld
    serialize_common_slide_data(&mut writer, &slide.common_slide_data)?;

    // clrMapOvr
    if let Some(ref ovr) = slide.color_map_override {
        serialize_color_map_override(&mut writer, ovr)?;
    }

    // timing
    if let Some(ref timing) = slide.timing {
        serialize_slide_timing(&mut writer, timing)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:sld")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

pub fn serialize_notes_slide(notes: &NotesSlide) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("p:notes");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute((
        "xmlns:r",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ));
    root.push_attribute((
        "xmlns:p",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ));
    if !notes.show_master_shapes {
        root.push_attribute(("showMasterSp", "0"));
    }
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    serialize_common_slide_data(&mut writer, &notes.common_slide_data)?;

    if let Some(ref ovr) = notes.color_map_override {
        serialize_color_map_override(&mut writer, ovr)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:notes")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

// =========================================================================
// 5. Common Slide Data Serializer (`p:cSld`, `p:spTree`, `p:sp`)
// =========================================================================

fn serialize_common_slide_data<W: std::io::Write>(
    writer: &mut Writer<W>,
    data: &CommonSlideData,
) -> Result<(), EngineError> {
    let mut csld = BytesStart::new("p:cSld");
    if let Some(ref n) = data.name {
        csld.push_attribute(("name", n.as_str()));
    }
    writer
        .write_event(Event::Start(csld))
        .map_err(EngineError::Io)?;

    // bg
    if let Some(ref bg) = data.background {
        writer
            .write_event(Event::Start(BytesStart::new("p:bg")))
            .map_err(EngineError::Io)?;
        if let Some(idx) = bg.bg_ref_idx {
            let mut bg_ref = BytesStart::new("p:bgRef");
            bg_ref.push_attribute(("idx", idx.to_string().as_str()));
            if let Some(ref color) = bg.bg_ref_color {
                writer
                    .write_event(Event::Start(bg_ref))
                    .map_err(EngineError::Io)?;
                serialize_color_choice(writer, color)?;
                writer
                    .write_event(Event::End(BytesEnd::new("p:bgRef")))
                    .map_err(EngineError::Io)?;
            } else {
                writer
                    .write_event(Event::Empty(bg_ref))
                    .map_err(EngineError::Io)?;
            }
        }
        if let Some(ref fill) = bg.fill {
            serialize_fill(writer, fill)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("p:bg")))
            .map_err(EngineError::Io)?;
    }

    // spTree
    writer
        .write_event(Event::Start(BytesStart::new("p:spTree")))
        .map_err(EngineError::Io)?;

    // nvGrpSpPr
    writer
        .write_event(Event::Start(BytesStart::new("p:nvGrpSpPr")))
        .map_err(EngineError::Io)?;
    let mut c_nv_pr = BytesStart::new("p:cNvPr");
    c_nv_pr.push_attribute(("id", "1"));
    c_nv_pr.push_attribute(("name", ""));
    writer
        .write_event(Event::Empty(c_nv_pr))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:cNvGrpSpPr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:nvPr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::End(BytesEnd::new("p:nvGrpSpPr")))
        .map_err(EngineError::Io)?;

    // grpSpPr
    writer
        .write_event(Event::Start(BytesStart::new("p:grpSpPr")))
        .map_err(EngineError::Io)?;
    let xfrm = data.shape_tree.group_transform.unwrap_or_default();
    serialize_group_transform(writer, &xfrm)?;
    writer
        .write_event(Event::End(BytesEnd::new("p:grpSpPr")))
        .map_err(EngineError::Io)?;

    // shapes
    for shape_elem in &data.shape_tree.shapes {
        match shape_elem {
            ShapeElement::Shape(ref s) => serialize_shape(writer, s)?,
            ShapeElement::Group(ref g) => serialize_group_shape(writer, g)?,
            ShapeElement::GraphicFrame(ref f) => serialize_graphic_frame(writer, f)?,
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:spTree")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("p:cSld")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_shape<W: std::io::Write>(
    writer: &mut Writer<W>,
    shape: &Shape,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:sp")))
        .map_err(EngineError::Io)?;

    // nvSpPr
    writer
        .write_event(Event::Start(BytesStart::new("p:nvSpPr")))
        .map_err(EngineError::Io)?;
    let mut c_nv_pr = BytesStart::new("p:cNvPr");
    c_nv_pr.push_attribute(("id", shape.id.to_string().as_str()));
    c_nv_pr.push_attribute(("name", shape.name.as_str()));
    writer
        .write_event(Event::Empty(c_nv_pr))
        .map_err(EngineError::Io)?;

    let mut c_nv_sp_pr = BytesStart::new("p:cNvSpPr");
    if shape.is_text_box {
        c_nv_sp_pr.push_attribute(("txBox", "1"));
    }
    writer
        .write_event(Event::Empty(c_nv_sp_pr))
        .map_err(EngineError::Io)?;

    // nvPr
    if let Some(ref ph) = shape.placeholder {
        writer
            .write_event(Event::Start(BytesStart::new("p:nvPr")))
            .map_err(EngineError::Io)?;
        let mut ph_elem = BytesStart::new("p:ph");
        if ph.ph_type != "body" {
            ph_elem.push_attribute(("type", ph.ph_type.as_str()));
        }
        if let Some(idx) = ph.index {
            ph_elem.push_attribute(("idx", idx.to_string().as_str()));
        }
        if let Some(ref o) = ph.orientation {
            ph_elem.push_attribute(("orient", o.as_str()));
        }
        if let Some(ref sz) = ph.size {
            ph_elem.push_attribute(("sz", sz.as_str()));
        }
        writer
            .write_event(Event::Empty(ph_elem))
            .map_err(EngineError::Io)?;
        writer
            .write_event(Event::End(BytesEnd::new("p:nvPr")))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Empty(BytesStart::new("p:nvPr")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:nvSpPr")))
        .map_err(EngineError::Io)?;

    // spPr
    serialize_shape_properties(writer, &shape.properties)?;

    // txBody
    if let Some(ref body) = shape.text_body {
        serialize_text_body(writer, body)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:sp")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_group_shape<W: std::io::Write>(
    writer: &mut Writer<W>,
    group: &GroupShape,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:grpSp")))
        .map_err(EngineError::Io)?;

    // nvGrpSpPr
    writer
        .write_event(Event::Start(BytesStart::new("p:nvGrpSpPr")))
        .map_err(EngineError::Io)?;
    let mut c_nv_pr = BytesStart::new("p:cNvPr");
    c_nv_pr.push_attribute(("id", group.id.to_string().as_str()));
    c_nv_pr.push_attribute(("name", group.name.as_str()));
    writer
        .write_event(Event::Empty(c_nv_pr))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:cNvGrpSpPr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:nvPr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::End(BytesEnd::new("p:nvGrpSpPr")))
        .map_err(EngineError::Io)?;

    // grpSpPr
    if let Some(ref xfrm) = group.transform {
        writer
            .write_event(Event::Start(BytesStart::new("p:grpSpPr")))
            .map_err(EngineError::Io)?;
        serialize_group_transform(writer, xfrm)?;
        writer
            .write_event(Event::End(BytesEnd::new("p:grpSpPr")))
            .map_err(EngineError::Io)?;
    }

    // children
    for child in &group.children {
        match child {
            ShapeElement::Shape(ref s) => serialize_shape(writer, s)?,
            ShapeElement::Group(ref g) => serialize_group_shape(writer, g)?,
            ShapeElement::GraphicFrame(ref f) => serialize_graphic_frame(writer, f)?,
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:grpSp")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_graphic_frame<W: std::io::Write>(
    writer: &mut Writer<W>,
    frame: &GraphicFrame,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:graphicFrame")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::Start(BytesStart::new("p:nvGraphicFramePr")))
        .map_err(EngineError::Io)?;
    let mut c_nv_pr = BytesStart::new("p:cNvPr");
    c_nv_pr.push_attribute(("id", frame.id.to_string().as_str()));
    c_nv_pr.push_attribute(("name", frame.name.as_str()));
    writer
        .write_event(Event::Empty(c_nv_pr))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:cNvGraphicFramePr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::Empty(BytesStart::new("p:nvPr")))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::End(BytesEnd::new("p:nvGraphicFramePr")))
        .map_err(EngineError::Io)?;

    if let Some(ref xfrm) = frame.transform {
        serialize_transform_2d(writer, xfrm)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:graphicFrame")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_shape_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    props: &ShapeProperties,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:spPr")))
        .map_err(EngineError::Io)?;

    if let Some(ref xfrm) = props.transform {
        serialize_transform_2d(writer, xfrm)?;
    }

    if let Some(ref geom) = props.geometry {
        match geom {
            Geometry::Preset(ref prst) => {
                let mut elem = BytesStart::new("a:prstGeom");
                elem.push_attribute(("prst", prst.as_str()));
                writer
                    .write_event(Event::Start(elem))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::Empty(BytesStart::new("a:avLst")))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::End(BytesEnd::new("a:prstGeom")))
                    .map_err(EngineError::Io)?;
            }
            Geometry::PresetWithAdjust {
                ref preset,
                ref adjust_values,
            } => {
                let mut elem = BytesStart::new("a:prstGeom");
                elem.push_attribute(("prst", preset.as_str()));
                writer
                    .write_event(Event::Start(elem))
                    .map_err(EngineError::Io)?;
                if adjust_values.is_empty() {
                    writer
                        .write_event(Event::Empty(BytesStart::new("a:avLst")))
                        .map_err(EngineError::Io)?;
                } else {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:avLst")))
                        .map_err(EngineError::Io)?;
                    for adj in adjust_values {
                        let mut gd = BytesStart::new("a:gd");
                        gd.push_attribute(("name", adj.name.as_str()));
                        gd.push_attribute(("fmla", adj.formula.as_str()));
                        writer
                            .write_event(Event::Empty(gd))
                            .map_err(EngineError::Io)?;
                    }
                    writer
                        .write_event(Event::End(BytesEnd::new("a:avLst")))
                        .map_err(EngineError::Io)?;
                }
                writer
                    .write_event(Event::End(BytesEnd::new("a:prstGeom")))
                    .map_err(EngineError::Io)?;
            }
            Geometry::Custom(ref cust) => {
                serialize_custom_geometry(writer, cust)?;
            }
        }
    }

    if let Some(ref fill) = props.fill {
        serialize_fill(writer, fill)?;
    }

    if let Some(ref line) = props.line {
        let mut ln = BytesStart::new("a:ln");
        if let Some(w) = line.width {
            ln.push_attribute(("w", w.to_string().as_str()));
        }
        writer
            .write_event(Event::Start(ln))
            .map_err(EngineError::Io)?;
        if let Some(ref fill) = line.fill {
            serialize_fill(writer, fill)?;
        }
        if let Some(ref dash) = line.dash_style {
            let mut prst_dash = BytesStart::new("a:prstDash");
            prst_dash.push_attribute(("val", dash.as_str()));
            writer
                .write_event(Event::Empty(prst_dash))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("a:ln")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:spPr")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_custom_geometry<W: std::io::Write>(
    writer: &mut Writer<W>,
    cust: &CustomGeometry,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("a:custGeom")))
        .map_err(EngineError::Io)?;

    // avLst
    if cust.adjust_values.is_empty() {
        writer
            .write_event(Event::Empty(BytesStart::new("a:avLst")))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Start(BytesStart::new("a:avLst")))
            .map_err(EngineError::Io)?;
        for adj in &cust.adjust_values {
            let mut gd = BytesStart::new("a:gd");
            gd.push_attribute(("name", adj.name.as_str()));
            gd.push_attribute(("fmla", adj.formula.as_str()));
            writer
                .write_event(Event::Empty(gd))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("a:avLst")))
            .map_err(EngineError::Io)?;
    }

    // gdLst
    if !cust.guides.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("a:gdLst")))
            .map_err(EngineError::Io)?;
        for gd in &cust.guides {
            let mut elem = BytesStart::new("a:gd");
            elem.push_attribute(("name", gd.name.as_str()));
            elem.push_attribute(("fmla", gd.formula.as_str()));
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("a:gdLst")))
            .map_err(EngineError::Io)?;
    }

    // pathLst
    writer
        .write_event(Event::Start(BytesStart::new("a:pathLst")))
        .map_err(EngineError::Io)?;
    for p in &cust.paths {
        let mut elem = BytesStart::new("a:path");
        let w_str;
        if let Some(w) = p.width {
            w_str = w.to_string();
            elem.push_attribute(("w", w_str.as_str()));
        }
        let h_str;
        if let Some(h) = p.height {
            h_str = h.to_string();
            elem.push_attribute(("h", h_str.as_str()));
        }
        writer
            .write_event(Event::Start(elem))
            .map_err(EngineError::Io)?;

        for inst in &p.instructions {
            match inst {
                PathInstruction::MoveTo { ref x, ref y } => {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:moveTo")))
                        .map_err(EngineError::Io)?;
                    let mut pt = BytesStart::new("a:pt");
                    pt.push_attribute(("x", x.as_str()));
                    pt.push_attribute(("y", y.as_str()));
                    writer
                        .write_event(Event::Empty(pt))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:moveTo")))
                        .map_err(EngineError::Io)?;
                }
                PathInstruction::LineTo { ref x, ref y } => {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:lnTo")))
                        .map_err(EngineError::Io)?;
                    let mut pt = BytesStart::new("a:pt");
                    pt.push_attribute(("x", x.as_str()));
                    pt.push_attribute(("y", y.as_str()));
                    writer
                        .write_event(Event::Empty(pt))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:lnTo")))
                        .map_err(EngineError::Io)?;
                }
                PathInstruction::ArcTo {
                    ref w_r,
                    ref h_r,
                    ref start_angle,
                    ref swing_angle,
                } => {
                    let mut arc = BytesStart::new("a:arcTo");
                    arc.push_attribute(("wR", w_r.as_str()));
                    arc.push_attribute(("hR", h_r.as_str()));
                    arc.push_attribute(("stAng", start_angle.as_str()));
                    arc.push_attribute(("swAng", swing_angle.as_str()));
                    writer
                        .write_event(Event::Empty(arc))
                        .map_err(EngineError::Io)?;
                }
                PathInstruction::QuadBezTo {
                    ref ctrl_x,
                    ref ctrl_y,
                    ref x,
                    ref y,
                } => {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:quadBezTo")))
                        .map_err(EngineError::Io)?;
                    let mut cp = BytesStart::new("a:pt");
                    cp.push_attribute(("x", ctrl_x.as_str()));
                    cp.push_attribute(("y", ctrl_y.as_str()));
                    writer
                        .write_event(Event::Empty(cp))
                        .map_err(EngineError::Io)?;
                    let mut end = BytesStart::new("a:pt");
                    end.push_attribute(("x", x.as_str()));
                    end.push_attribute(("y", y.as_str()));
                    writer
                        .write_event(Event::Empty(end))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:quadBezTo")))
                        .map_err(EngineError::Io)?;
                }
                PathInstruction::CubicBezTo {
                    ref ctrl1_x,
                    ref ctrl1_y,
                    ref ctrl2_x,
                    ref ctrl2_y,
                    ref x,
                    ref y,
                } => {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:cubicBezTo")))
                        .map_err(EngineError::Io)?;
                    let mut cp1 = BytesStart::new("a:pt");
                    cp1.push_attribute(("x", ctrl1_x.as_str()));
                    cp1.push_attribute(("y", ctrl1_y.as_str()));
                    writer
                        .write_event(Event::Empty(cp1))
                        .map_err(EngineError::Io)?;
                    let mut cp2 = BytesStart::new("a:pt");
                    cp2.push_attribute(("x", ctrl2_x.as_str()));
                    cp2.push_attribute(("y", ctrl2_y.as_str()));
                    writer
                        .write_event(Event::Empty(cp2))
                        .map_err(EngineError::Io)?;
                    let mut end = BytesStart::new("a:pt");
                    end.push_attribute(("x", x.as_str()));
                    end.push_attribute(("y", y.as_str()));
                    writer
                        .write_event(Event::Empty(end))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:cubicBezTo")))
                        .map_err(EngineError::Io)?;
                }
                PathInstruction::Close => {
                    writer
                        .write_event(Event::Empty(BytesStart::new("a:close")))
                        .map_err(EngineError::Io)?;
                }
            }
        }

        writer
            .write_event(Event::End(BytesEnd::new("a:path")))
            .map_err(EngineError::Io)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:pathLst")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("a:custGeom")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_transform_2d<W: std::io::Write>(
    writer: &mut Writer<W>,
    xfrm: &Transform2D,
) -> Result<(), EngineError> {
    let mut elem = BytesStart::new("a:xfrm");
    if xfrm.rot != 0 {
        elem.push_attribute(("rot", xfrm.rot.to_string().as_str()));
    }
    if xfrm.flip_h {
        elem.push_attribute(("flipH", "1"));
    }
    if xfrm.flip_v {
        elem.push_attribute(("flipV", "1"));
    }
    writer
        .write_event(Event::Start(elem))
        .map_err(EngineError::Io)?;

    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", xfrm.x.to_string().as_str()));
    off.push_attribute(("y", xfrm.y.to_string().as_str()));
    writer
        .write_event(Event::Empty(off))
        .map_err(EngineError::Io)?;

    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", xfrm.cx.to_string().as_str()));
    ext.push_attribute(("cy", xfrm.cy.to_string().as_str()));
    writer
        .write_event(Event::Empty(ext))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("a:xfrm")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_group_transform<W: std::io::Write>(
    writer: &mut Writer<W>,
    xfrm: &GroupTransform2D,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("a:xfrm")))
        .map_err(EngineError::Io)?;

    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", xfrm.x.to_string().as_str()));
    off.push_attribute(("y", xfrm.y.to_string().as_str()));
    writer
        .write_event(Event::Empty(off))
        .map_err(EngineError::Io)?;

    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", xfrm.cx.to_string().as_str()));
    ext.push_attribute(("cy", xfrm.cy.to_string().as_str()));
    writer
        .write_event(Event::Empty(ext))
        .map_err(EngineError::Io)?;

    let mut ch_off = BytesStart::new("a:chOff");
    ch_off.push_attribute(("x", xfrm.ch_x.to_string().as_str()));
    ch_off.push_attribute(("y", xfrm.ch_y.to_string().as_str()));
    writer
        .write_event(Event::Empty(ch_off))
        .map_err(EngineError::Io)?;

    let mut ch_ext = BytesStart::new("a:chExt");
    ch_ext.push_attribute(("cx", xfrm.ch_cx.to_string().as_str()));
    ch_ext.push_attribute(("cy", xfrm.ch_cy.to_string().as_str()));
    writer
        .write_event(Event::Empty(ch_ext))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("a:xfrm")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_fill<W: std::io::Write>(
    writer: &mut Writer<W>,
    fill: &Fill,
) -> Result<(), EngineError> {
    match fill {
        Fill::NoFill => {
            writer
                .write_event(Event::Empty(BytesStart::new("a:noFill")))
                .map_err(EngineError::Io)?;
        }
        Fill::Solid(ref c) => {
            writer
                .write_event(Event::Start(BytesStart::new("a:solidFill")))
                .map_err(EngineError::Io)?;
            serialize_color_choice(writer, c)?;
            writer
                .write_event(Event::End(BytesEnd::new("a:solidFill")))
                .map_err(EngineError::Io)?;
        }
        _ => {}
    }
    Ok(())
}

// =========================================================================
// 6. Text Body Serializer (`p:txBody`, `a:p`, `a:r`, `a:t`)
// =========================================================================

fn serialize_text_body<W: std::io::Write>(
    writer: &mut Writer<W>,
    body: &TextBody,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:txBody")))
        .map_err(EngineError::Io)?;

    // bodyPr
    let mut body_pr = BytesStart::new("a:bodyPr");
    if let Some(ref anch) = body.body_properties.anchor {
        body_pr.push_attribute(("anchor", anch.as_str()));
    }
    if let Some(ref v) = body.body_properties.vertical {
        body_pr.push_attribute(("vert", v.as_str()));
    }
    if let Some(ref w) = body.body_properties.wrap {
        body_pr.push_attribute(("wrap", w.as_str()));
    }
    if let Some(l) = body.body_properties.l_ins {
        body_pr.push_attribute(("lIns", l.to_string().as_str()));
    }
    if let Some(t) = body.body_properties.t_ins {
        body_pr.push_attribute(("tIns", t.to_string().as_str()));
    }
    if let Some(r) = body.body_properties.r_ins {
        body_pr.push_attribute(("rIns", r.to_string().as_str()));
    }
    if let Some(b) = body.body_properties.b_ins {
        body_pr.push_attribute(("bIns", b.to_string().as_str()));
    }
    writer
        .write_event(Event::Empty(body_pr))
        .map_err(EngineError::Io)?;

    // lstStyle
    if let Some(ref lst) = body.list_style {
        serialize_text_list_style(writer, "a:lstStyle", lst)?;
    } else {
        writer
            .write_event(Event::Empty(BytesStart::new("a:lstStyle")))
            .map_err(EngineError::Io)?;
    }

    // paragraphs
    for p in &body.paragraphs {
        serialize_paragraph(writer, p)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:txBody")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_paragraph<W: std::io::Write>(
    writer: &mut Writer<W>,
    p: &TextParagraph,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("a:p")))
        .map_err(EngineError::Io)?;

    // pPr
    if let Some(ref p_pr) = p.properties {
        serialize_paragraph_properties(writer, "a:pPr", p_pr)?;
    }

    // runs
    for run in &p.runs {
        match run {
            TextRunElement::Text(ref text, ref r_pr) => {
                writer
                    .write_event(Event::Start(BytesStart::new("a:r")))
                    .map_err(EngineError::Io)?;
                if let Some(ref props) = r_pr {
                    serialize_run_properties(writer, "a:rPr", props)?;
                }
                writer
                    .write_event(Event::Start(BytesStart::new("a:t")))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::Text(quick_xml::events::BytesText::new(text)))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::End(BytesEnd::new("a:t")))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::End(BytesEnd::new("a:r")))
                    .map_err(EngineError::Io)?;
            }
            TextRunElement::LineBreak(ref r_pr) => {
                let br = BytesStart::new("a:br");
                if let Some(ref props) = r_pr {
                    writer
                        .write_event(Event::Start(br))
                        .map_err(EngineError::Io)?;
                    serialize_run_properties(writer, "a:rPr", props)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:br")))
                        .map_err(EngineError::Io)?;
                } else {
                    writer
                        .write_event(Event::Empty(br))
                        .map_err(EngineError::Io)?;
                }
            }
            TextRunElement::Field {
                ref id,
                ref field_type,
                ref text,
                ref properties,
            } => {
                let mut fld = BytesStart::new("a:fld");
                fld.push_attribute(("id", id.as_str()));
                fld.push_attribute(("type", field_type.as_str()));
                writer
                    .write_event(Event::Start(fld))
                    .map_err(EngineError::Io)?;
                if let Some(ref props) = properties {
                    serialize_run_properties(writer, "a:rPr", props)?;
                }
                if let Some(ref t) = text {
                    writer
                        .write_event(Event::Start(BytesStart::new("a:t")))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::new(t)))
                        .map_err(EngineError::Io)?;
                    writer
                        .write_event(Event::End(BytesEnd::new("a:t")))
                        .map_err(EngineError::Io)?;
                }
                writer
                    .write_event(Event::End(BytesEnd::new("a:fld")))
                    .map_err(EngineError::Io)?;
            }
        }
    }

    // endParaRPr
    if let Some(ref end_r_pr) = p.end_paragraph_run_properties {
        serialize_run_properties(writer, "a:endParaRPr", end_r_pr)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("a:p")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_paragraph_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    props: &TextParagraphProperties,
) -> Result<(), EngineError> {
    let mut p_pr = BytesStart::new(tag);
    if let Some(lvl) = props.level {
        p_pr.push_attribute(("lvl", lvl.to_string().as_str()));
    }
    if let Some(ref al) = props.align {
        p_pr.push_attribute(("algn", al.as_str()));
    }
    if let Some(m) = props.mar_l {
        p_pr.push_attribute(("marL", m.to_string().as_str()));
    }
    if let Some(m) = props.mar_r {
        p_pr.push_attribute(("marR", m.to_string().as_str()));
    }
    if let Some(i) = props.indent {
        p_pr.push_attribute(("indent", i.to_string().as_str()));
    }
    if let Some(t) = props.def_tab_sz {
        p_pr.push_attribute(("defTabSz", t.to_string().as_str()));
    }

    let has_children = props.default_run_properties.is_some()
        || props.space_before.is_some()
        || props.space_after.is_some()
        || props.line_spacing.is_some();

    if has_children {
        writer
            .write_event(Event::Start(p_pr))
            .map_err(EngineError::Io)?;
        if let Some(ref def_r_pr) = props.default_run_properties {
            serialize_run_properties(writer, "a:defRPr", def_r_pr)?;
        }
        if let Some(spc) = props.space_before {
            writer
                .write_event(Event::Start(BytesStart::new("a:spcBef")))
                .map_err(EngineError::Io)?;
            let mut spc_pts = BytesStart::new("a:spcPts");
            spc_pts.push_attribute(("val", (spc / 20).to_string().as_str()));
            writer
                .write_event(Event::Empty(spc_pts))
                .map_err(EngineError::Io)?;
            writer
                .write_event(Event::End(BytesEnd::new("a:spcBef")))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Empty(p_pr))
            .map_err(EngineError::Io)?;
    }

    Ok(())
}

fn serialize_run_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    props: &TextRunProperties,
) -> Result<(), EngineError> {
    let mut r_pr = BytesStart::new(tag);
    if let Some(sz) = props.size {
        r_pr.push_attribute(("sz", sz.to_string().as_str()));
    }
    if let Some(b) = props.bold {
        r_pr.push_attribute(("b", if b { "1" } else { "0" }));
    }
    if let Some(i) = props.italic {
        r_pr.push_attribute(("i", if i { "1" } else { "0" }));
    }
    if let Some(ref u) = props.underline {
        r_pr.push_attribute(("u", u.as_str()));
    }
    if let Some(ref s) = props.strike {
        r_pr.push_attribute(("strike", s.as_str()));
    }
    if let Some(k) = props.kern {
        r_pr.push_attribute(("kern", k.to_string().as_str()));
    }
    if let Some(base) = props.baseline {
        r_pr.push_attribute(("baseline", base.to_string().as_str()));
    }
    if let Some(ref l) = props.lang {
        r_pr.push_attribute(("lang", l.as_str()));
    }

    let has_children = props.color.is_some()
        || props.font_latin.is_some()
        || props.font_ea.is_some()
        || props.font_cs.is_some();

    if has_children {
        writer
            .write_event(Event::Start(r_pr))
            .map_err(EngineError::Io)?;
        if let Some(ref c) = props.color {
            writer
                .write_event(Event::Start(BytesStart::new("a:solidFill")))
                .map_err(EngineError::Io)?;
            serialize_color_choice(writer, c)?;
            writer
                .write_event(Event::End(BytesEnd::new("a:solidFill")))
                .map_err(EngineError::Io)?;
        }
        if let Some(ref f) = props.font_latin {
            let mut latin = BytesStart::new("a:latin");
            latin.push_attribute(("typeface", f.as_str()));
            writer
                .write_event(Event::Empty(latin))
                .map_err(EngineError::Io)?;
        }
        if let Some(ref f) = props.font_ea {
            let mut ea = BytesStart::new("a:ea");
            ea.push_attribute(("typeface", f.as_str()));
            writer
                .write_event(Event::Empty(ea))
                .map_err(EngineError::Io)?;
        }
        if let Some(ref f) = props.font_cs {
            let mut cs = BytesStart::new("a:cs");
            cs.push_attribute(("typeface", f.as_str()));
            writer
                .write_event(Event::Empty(cs))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(EngineError::Io)?;
    } else {
        writer
            .write_event(Event::Empty(r_pr))
            .map_err(EngineError::Io)?;
    }

    Ok(())
}

fn serialize_text_list_style<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    style: &TextListStyle,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new(tag)))
        .map_err(EngineError::Io)?;

    if let Some(ref def) = style.default_paragraph_properties {
        serialize_paragraph_properties(writer, "a:defPPr", def)?;
    }

    for (idx, lvl_opt) in style.levels.iter().enumerate() {
        if let Some(ref lvl) = lvl_opt {
            let lvl_tag = format!("a:lvl{}pPr", idx + 1);
            serialize_paragraph_properties(writer, &lvl_tag, lvl)?;
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(EngineError::Io)?;

    Ok(())
}

// =========================================================================
// 7. Theme & Color Serializer (`ppt/theme/*.xml`)
// =========================================================================

pub fn serialize_theme(theme: &Theme) -> Result<Vec<u8>, EngineError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut root = BytesStart::new("a:theme");
    root.push_attribute((
        "xmlns:a",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ));
    root.push_attribute(("name", theme.name.as_str()));
    writer
        .write_event(Event::Start(root))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::Start(BytesStart::new("a:themeElements")))
        .map_err(EngineError::Io)?;

    // clrScheme
    let mut clr_scheme = BytesStart::new("a:clrScheme");
    clr_scheme.push_attribute(("name", theme.color_scheme.name.as_str()));
    writer
        .write_event(Event::Start(clr_scheme))
        .map_err(EngineError::Io)?;
    for (role, color) in &theme.color_scheme.colors {
        let tag = format!("a:{}", role);
        writer
            .write_event(Event::Start(BytesStart::new(&tag)))
            .map_err(EngineError::Io)?;
        serialize_color_choice(&mut writer, color)?;
        writer
            .write_event(Event::End(BytesEnd::new(&tag)))
            .map_err(EngineError::Io)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:clrScheme")))
        .map_err(EngineError::Io)?;

    // fontScheme
    let mut font_scheme = BytesStart::new("a:fontScheme");
    font_scheme.push_attribute(("name", theme.font_scheme.name.as_str()));
    writer
        .write_event(Event::Start(font_scheme))
        .map_err(EngineError::Io)?;
    serialize_font_collection(&mut writer, "a:majorFont", &theme.font_scheme.major_font)?;
    serialize_font_collection(&mut writer, "a:minorFont", &theme.font_scheme.minor_font)?;
    writer
        .write_event(Event::End(BytesEnd::new("a:fontScheme")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("a:themeElements")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("a:theme")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

fn serialize_font_collection<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    col: &FontCollection,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new(tag)))
        .map_err(EngineError::Io)?;

    let mut latin = BytesStart::new("a:latin");
    latin.push_attribute(("typeface", col.latin.as_deref().unwrap_or("")));
    writer
        .write_event(Event::Empty(latin))
        .map_err(EngineError::Io)?;

    let mut ea = BytesStart::new("a:ea");
    ea.push_attribute(("typeface", col.ea.as_deref().unwrap_or("")));
    writer
        .write_event(Event::Empty(ea))
        .map_err(EngineError::Io)?;

    let mut cs = BytesStart::new("a:cs");
    cs.push_attribute(("typeface", col.cs.as_deref().unwrap_or("")));
    writer
        .write_event(Event::Empty(cs))
        .map_err(EngineError::Io)?;

    for (script, tf) in &col.scripts {
        let mut font = BytesStart::new("a:font");
        font.push_attribute(("script", script.as_str()));
        font.push_attribute(("typeface", tf.as_str()));
        writer
            .write_event(Event::Empty(font))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_color_choice<W: std::io::Write>(
    writer: &mut Writer<W>,
    color: &ColorChoice,
) -> Result<(), EngineError> {
    match color {
        ColorChoice::Srgb(ref hex) => {
            let mut srgb = BytesStart::new("a:srgbClr");
            srgb.push_attribute(("val", hex.as_str()));
            writer
                .write_event(Event::Empty(srgb))
                .map_err(EngineError::Io)?;
        }
        ColorChoice::System(ref val, ref last_clr) => {
            let mut sys = BytesStart::new("a:sysClr");
            sys.push_attribute(("val", val.as_str()));
            if let Some(ref last) = last_clr {
                sys.push_attribute(("lastClr", last.as_str()));
            }
            writer
                .write_event(Event::Empty(sys))
                .map_err(EngineError::Io)?;
        }
        ColorChoice::Scheme(ref token, ref transforms) => {
            let mut scheme = BytesStart::new("a:schemeClr");
            scheme.push_attribute(("val", token.as_str()));
            if transforms.is_empty() {
                writer
                    .write_event(Event::Empty(scheme))
                    .map_err(EngineError::Io)?;
            } else {
                writer
                    .write_event(Event::Start(scheme))
                    .map_err(EngineError::Io)?;
                for t in transforms {
                    match t {
                        ColorTransform::Tint(v) => {
                            let mut tint = BytesStart::new("a:tint");
                            tint.push_attribute(("val", v.to_string().as_str()));
                            writer
                                .write_event(Event::Empty(tint))
                                .map_err(EngineError::Io)?;
                        }
                        ColorTransform::Shade(v) => {
                            let mut shade = BytesStart::new("a:shade");
                            shade.push_attribute(("val", v.to_string().as_str()));
                            writer
                                .write_event(Event::Empty(shade))
                                .map_err(EngineError::Io)?;
                        }
                        ColorTransform::LumMod(v) => {
                            let mut lum_mod = BytesStart::new("a:lumMod");
                            lum_mod.push_attribute(("val", v.to_string().as_str()));
                            writer
                                .write_event(Event::Empty(lum_mod))
                                .map_err(EngineError::Io)?;
                        }
                        ColorTransform::LumOff(v) => {
                            let mut lum_off = BytesStart::new("a:lumOff");
                            lum_off.push_attribute(("val", v.to_string().as_str()));
                            writer
                                .write_event(Event::Empty(lum_off))
                                .map_err(EngineError::Io)?;
                        }
                        ColorTransform::Alpha(v) => {
                            let mut alpha = BytesStart::new("a:alpha");
                            alpha.push_attribute(("val", v.to_string().as_str()));
                            writer
                                .write_event(Event::Empty(alpha))
                                .map_err(EngineError::Io)?;
                        }
                    }
                }
                writer
                    .write_event(Event::End(BytesEnd::new("a:schemeClr")))
                    .map_err(EngineError::Io)?;
            }
        }
    }
    Ok(())
}

fn serialize_color_map<W: std::io::Write>(
    writer: &mut Writer<W>,
    map: &ColorMap,
) -> Result<(), EngineError> {
    let mut clr_map = BytesStart::new("p:clrMap");
    clr_map.push_attribute(("bg1", map.bg1.as_str()));
    clr_map.push_attribute(("tx1", map.tx1.as_str()));
    clr_map.push_attribute(("bg2", map.bg2.as_str()));
    clr_map.push_attribute(("tx2", map.tx2.as_str()));
    clr_map.push_attribute(("accent1", map.accent1.as_str()));
    clr_map.push_attribute(("accent2", map.accent2.as_str()));
    clr_map.push_attribute(("accent3", map.accent3.as_str()));
    clr_map.push_attribute(("accent4", map.accent4.as_str()));
    clr_map.push_attribute(("accent5", map.accent5.as_str()));
    clr_map.push_attribute(("accent6", map.accent6.as_str()));
    clr_map.push_attribute(("hlink", map.hlink.as_str()));
    clr_map.push_attribute(("folHlink", map.fol_hlink.as_str()));
    writer
        .write_event(Event::Empty(clr_map))
        .map_err(EngineError::Io)?;
    Ok(())
}

fn serialize_color_map_override<W: std::io::Write>(
    writer: &mut Writer<W>,
    ovr: &ColorMapOverride,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:clrMapOvr")))
        .map_err(EngineError::Io)?;
    match ovr {
        ColorMapOverride::UseMaster => {
            writer
                .write_event(Event::Empty(BytesStart::new("a:masterClrMapping")))
                .map_err(EngineError::Io)?;
        }
        ColorMapOverride::Override(ref map) => {
            let mut ovr_map = BytesStart::new("a:overrideClrMapping");
            ovr_map.push_attribute(("bg1", map.bg1.as_str()));
            ovr_map.push_attribute(("tx1", map.tx1.as_str()));
            ovr_map.push_attribute(("bg2", map.bg2.as_str()));
            ovr_map.push_attribute(("tx2", map.tx2.as_str()));
            ovr_map.push_attribute(("accent1", map.accent1.as_str()));
            ovr_map.push_attribute(("accent2", map.accent2.as_str()));
            ovr_map.push_attribute(("accent3", map.accent3.as_str()));
            ovr_map.push_attribute(("accent4", map.accent4.as_str()));
            ovr_map.push_attribute(("accent5", map.accent5.as_str()));
            ovr_map.push_attribute(("accent6", map.accent6.as_str()));
            ovr_map.push_attribute(("hlink", map.hlink.as_str()));
            ovr_map.push_attribute(("folHlink", map.fol_hlink.as_str()));
            writer
                .write_event(Event::Empty(ovr_map))
                .map_err(EngineError::Io)?;
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new("p:clrMapOvr")))
        .map_err(EngineError::Io)?;
    Ok(())
}

fn serialize_slide_timing<W: std::io::Write>(
    writer: &mut Writer<W>,
    timing: &SlideTiming,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:timing")))
        .map_err(EngineError::Io)?;

    if let Some(ref root) = timing.root_node {
        writer
            .write_event(Event::Start(BytesStart::new("p:tnLst")))
            .map_err(EngineError::Io)?;
        serialize_time_node(writer, root)?;
        writer
            .write_event(Event::End(BytesEnd::new("p:tnLst")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:timing")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_time_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    node: &TimeNode,
) -> Result<(), EngineError> {
    match &node.kind {
        TimeNodeKind::Parallel(children) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:par")))
                .map_err(EngineError::Io)?;
            serialize_common_time_node(writer, &node.common)?;
            if !children.is_empty() {
                writer
                    .write_event(Event::Start(BytesStart::new("p:childTnLst")))
                    .map_err(EngineError::Io)?;
                for child in children {
                    serialize_time_node(writer, child)?;
                }
                writer
                    .write_event(Event::End(BytesEnd::new("p:childTnLst")))
                    .map_err(EngineError::Io)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("p:par")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Sequential(children) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:seq")))
                .map_err(EngineError::Io)?;
            serialize_common_time_node(writer, &node.common)?;
            if !children.is_empty() {
                writer
                    .write_event(Event::Start(BytesStart::new("p:childTnLst")))
                    .map_err(EngineError::Io)?;
                for child in children {
                    serialize_time_node(writer, child)?;
                }
                writer
                    .write_event(Event::End(BytesEnd::new("p:childTnLst")))
                    .map_err(EngineError::Io)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("p:seq")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Motion(motion) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:animMotion")))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, motion.target.shape_id)?;
            let mut elem = BytesStart::new("p:animMotion");
            elem.push_attribute(("path", motion.path.as_str()));
            if motion.rotate_to_path {
                elem.push_attribute(("rAng", "1"));
            }
            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:animMotion")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Rotation(rot) => {
            let mut elem = BytesStart::new("p:animRot");
            if let Some(by) = rot.by_deg {
                let by_em = (by * 60_000.0).round() as i64;
                elem.push_attribute(("by", by_em.to_string().as_str()));
            }
            writer
                .write_event(Event::Start(elem))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, rot.target.shape_id)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:animRot")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Scale(scale) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:animScale")))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, scale.target.shape_id)?;
            if let Some((sx, sy)) = scale.by_scale {
                let mut by_elem = BytesStart::new("p:by");
                let bx_str = ((sx * 100_000.0).round() as i64).to_string();
                let by_str = ((sy * 100_000.0).round() as i64).to_string();
                by_elem.push_attribute(("x", bx_str.as_str()));
                by_elem.push_attribute(("y", by_str.as_str()));
                writer
                    .write_event(Event::Empty(by_elem))
                    .map_err(EngineError::Io)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("p:animScale")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Color(clr) => {
            let mut elem = BytesStart::new("p:animClr");
            let clr_spc = match clr.color_space {
                ColorSpace::Rgb => "rgb",
                ColorSpace::Hsl => "hsl",
            };
            elem.push_attribute(("clrSpc", clr_spc));
            writer
                .write_event(Event::Start(elem))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, clr.target.shape_id)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:animClr")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Effect(eff) => {
            let mut elem = BytesStart::new("p:animEffect");
            elem.push_attribute(("type", eff.effect_type.as_str()));
            let trans = match eff.transition {
                EffectTransition::In => "in",
                EffectTransition::Out => "out",
            };
            elem.push_attribute(("transition", trans));
            writer
                .write_event(Event::Start(elem))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, eff.target.shape_id)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:animEffect")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Generic(generic) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:anim")))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, generic.target.shape_id)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:anim")))
                .map_err(EngineError::Io)?;
        }
        TimeNodeKind::Set(set) => {
            writer
                .write_event(Event::Start(BytesStart::new("p:set")))
                .map_err(EngineError::Io)?;
            serialize_common_behavior(writer, &node.common, set.target.shape_id)?;
            writer
                .write_event(Event::End(BytesEnd::new("p:set")))
                .map_err(EngineError::Io)?;
        }
    }
    Ok(())
}

fn serialize_common_time_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    ctn: &CommonTimeNode,
) -> Result<(), EngineError> {
    let mut elem = BytesStart::new("p:cTn");
    elem.push_attribute(("id", ctn.id.to_string().as_str()));
    if let Some(dur) = ctn.duration_ms {
        elem.push_attribute(("dur", dur.to_string().as_str()));
    } else {
        elem.push_attribute(("dur", "indefinite"));
    }
    if ctn.auto_reverse {
        elem.push_attribute(("autoRev", "1"));
    }
    if ctn.accel_ratio > 0.0 {
        let a = ((ctn.accel_ratio * 100_000.0).round() as i64).to_string();
        elem.push_attribute(("accel", a.as_str()));
    }
    if ctn.decel_ratio > 0.0 {
        let d = ((ctn.decel_ratio * 100_000.0).round() as i64).to_string();
        elem.push_attribute(("decel", d.as_str()));
    }
    let node_type = match ctn.trigger_type {
        TriggerType::OnClick => "clickEffect",
        TriggerType::WithPrev => "withEffect",
        TriggerType::AfterPrev => "afterEffect",
    };
    elem.push_attribute(("nodeType", node_type));

    writer
        .write_event(Event::Start(elem))
        .map_err(EngineError::Io)?;

    if ctn.delay_ms != 0 {
        writer
            .write_event(Event::Start(BytesStart::new("p:stCondLst")))
            .map_err(EngineError::Io)?;
        let mut cond = BytesStart::new("p:cond");
        cond.push_attribute(("delay", ctn.delay_ms.to_string().as_str()));
        writer
            .write_event(Event::Empty(cond))
            .map_err(EngineError::Io)?;
        writer
            .write_event(Event::End(BytesEnd::new("p:stCondLst")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:cTn")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_common_behavior<W: std::io::Write>(
    writer: &mut Writer<W>,
    ctn: &CommonTimeNode,
    shape_id: u32,
) -> Result<(), EngineError> {
    writer
        .write_event(Event::Start(BytesStart::new("p:cBhvr")))
        .map_err(EngineError::Io)?;

    serialize_common_time_node(writer, ctn)?;

    writer
        .write_event(Event::Start(BytesStart::new("p:tgtEl")))
        .map_err(EngineError::Io)?;
    let mut sp_tgt = BytesStart::new("p:spTgt");
    sp_tgt.push_attribute(("spid", shape_id.to_string().as_str()));
    writer
        .write_event(Event::Empty(sp_tgt))
        .map_err(EngineError::Io)?;
    writer
        .write_event(Event::End(BytesEnd::new("p:tgtEl")))
        .map_err(EngineError::Io)?;

    writer
        .write_event(Event::End(BytesEnd::new("p:cBhvr")))
        .map_err(EngineError::Io)?;

    Ok(())
}
