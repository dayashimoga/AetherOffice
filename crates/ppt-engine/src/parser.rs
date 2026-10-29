#![forbid(unsafe_code)]
//! PresentationML & DrawingML Streaming XML Parser
//!
//! Fast, memory-efficient streaming parser for:
//! - `ppt/presentation.xml`
//! - `ppt/slideMasters/*.xml`
//! - `ppt/slideLayouts/*.xml`
//! - `ppt/slides/*.xml`
//! - `ppt/notesSlides/*.xml`
//! - `ppt/theme/*.xml`

use ooxml_opc::error::EngineError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::BTreeMap;

use crate::animation::*;
use crate::drawingml::{CustomGeometry, CustomPath, GeomGuide, PathFillMode, PathInstruction};
use crate::model::*;

// =========================================================================
// XML Helper Utilities
// =========================================================================

fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    if let Some(pos) = s.find(':') {
        &s[pos + 1..]
    } else {
        s
    }
}

fn get_attr_val(e: &BytesStart, name: &str) -> Option<String> {
    for a in e.attributes().flatten() {
        if local_name(a.key.into_inner()) == name {
            return String::from_utf8(a.value.into_owned()).ok();
        }
    }
    None
}

fn parse_u32_attr(e: &BytesStart, name: &str) -> Option<u32> {
    get_attr_val(e, name).and_then(|s| s.parse::<u32>().ok())
}

fn parse_i64_attr(e: &BytesStart, name: &str) -> Option<i64> {
    get_attr_val(e, name).and_then(|s| s.parse::<i64>().ok())
}

fn parse_i32_attr(e: &BytesStart, name: &str) -> Option<i32> {
    get_attr_val(e, name).and_then(|s| s.parse::<i32>().ok())
}

fn parse_bool_attr(e: &BytesStart, name: &str) -> bool {
    matches!(
        get_attr_val(e, name).as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

// =========================================================================
// 1. Presentation Parser (`ppt/presentation.xml`)
// =========================================================================

pub fn parse_presentation(xml: &[u8]) -> Result<Presentation, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut presentation = Presentation::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "presentation" => {
                    presentation.save_subset_fonts = parse_bool_attr(e, "saveSubsetFonts");
                }
                "sldMasterIdLst" => {
                    presentation.slide_masters = parse_slide_master_id_list(&mut reader)?;
                }
                "sldIdLst" => {
                    presentation.slides = parse_slide_id_list(&mut reader)?;
                }
                "defaultTextStyle" => {
                    presentation.default_text_style =
                        Some(parse_text_list_style(&mut reader, "defaultTextStyle")?);
                }
                _ => {}
            },
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "sldSz" => {
                    let cx = parse_u32_attr(e, "cx").unwrap_or(12_192_000);
                    let cy = parse_u32_attr(e, "cy").unwrap_or(6_858_000);
                    presentation.slide_size = SlideSize { cx, cy };
                }
                "notesSz" => {
                    let cx = parse_u32_attr(e, "cx").unwrap_or(6_858_000);
                    let cy = parse_u32_attr(e, "cy").unwrap_or(9_144_000);
                    presentation.notes_size = NotesSize { cx, cy };
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(presentation)
}

fn parse_slide_master_id_list(
    reader: &mut Reader<&[u8]>,
) -> Result<Vec<SlideMasterIdRef>, EngineError> {
    let mut masters = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "sldMasterId" => {
                let id = parse_u32_attr(e, "id").unwrap_or(2_147_483_648);
                let r_id = get_attr_val(e, "id")
                    .or_else(|| get_attr_val(e, "rId"))
                    .unwrap_or_default();
                masters.push(SlideMasterIdRef { id, r_id });
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "sldMasterIdLst" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(masters)
}

fn parse_slide_id_list(reader: &mut Reader<&[u8]>) -> Result<Vec<SlideIdRef>, EngineError> {
    let mut slides = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "sldId" => {
                let id = parse_u32_attr(e, "id").unwrap_or(256);
                let r_id = get_attr_val(e, "id")
                    .or_else(|| get_attr_val(e, "rId"))
                    .unwrap_or_default();
                slides.push(SlideIdRef { id, r_id });
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "sldIdLst" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(slides)
}

// =========================================================================
// 2. Slide Master Parser (`ppt/slideMasters/*.xml`)
// =========================================================================

pub fn parse_slide_master(xml: &[u8]) -> Result<SlideMaster, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = false;

    let mut master = SlideMaster::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "cSld" => {
                    master.common_slide_data = parse_common_slide_data(&mut reader)?;
                }
                "sldLayoutIdLst" => {
                    master.slide_layouts = parse_slide_layout_id_list(&mut reader)?;
                }
                "txStyles" => {
                    master.text_styles = parse_master_text_styles(&mut reader)?;
                }
                _ => {}
            },
            Event::Empty(ref e) => {
                if local_name(e.name().into_inner()) == "clrMap" {
                    master.color_map = parse_color_map_tag(e);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(master)
}

fn parse_slide_layout_id_list(
    reader: &mut Reader<&[u8]>,
) -> Result<Vec<SlideLayoutIdRef>, EngineError> {
    let mut layouts = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "sldLayoutId" => {
                let id = parse_u32_attr(e, "id").unwrap_or(2_147_483_649);
                let r_id = get_attr_val(e, "id")
                    .or_else(|| get_attr_val(e, "rId"))
                    .unwrap_or_default();
                layouts.push(SlideLayoutIdRef { id, r_id });
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "sldLayoutIdLst" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(layouts)
}

fn parse_master_text_styles(reader: &mut Reader<&[u8]>) -> Result<MasterTextStyles, EngineError> {
    let mut styles = MasterTextStyles::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "titleStyle" => {
                    styles.title_style = Some(parse_text_list_style(reader, "titleStyle")?);
                }
                "bodyStyle" => {
                    styles.body_style = Some(parse_text_list_style(reader, "bodyStyle")?);
                }
                "otherStyle" => {
                    styles.other_style = Some(parse_text_list_style(reader, "otherStyle")?);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "txStyles" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(styles)
}

// =========================================================================
// 3. Slide Layout Parser (`ppt/slideLayouts/*.xml`)
// =========================================================================

pub fn parse_slide_layout(xml: &[u8]) -> Result<SlideLayout, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut layout = SlideLayout {
        show_master_shapes: true,
        ..Default::default()
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "sldLayout" => {
                    layout.layout_type = get_attr_val(e, "type");
                    layout.matching_name = get_attr_val(e, "matchingName");
                    layout.preserve = parse_bool_attr(e, "preserve");
                    if let Some(val) = get_attr_val(e, "showMasterSp") {
                        layout.show_master_shapes = val == "1" || val == "true";
                    }
                }
                "cSld" => {
                    layout.common_slide_data = parse_common_slide_data(&mut reader)?;
                }
                "clrMapOvr" => {
                    layout.color_map_override = Some(parse_color_map_override(&mut reader)?);
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(layout)
}

// =========================================================================
// 4. Slide Parser (`ppt/slides/*.xml`) & Notes Slide
// =========================================================================

pub fn parse_slide(xml: &[u8]) -> Result<Slide, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut slide = Slide {
        show_master_shapes: true,
        ..Default::default()
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "sld" => {
                    if let Some(val) = get_attr_val(e, "showMasterSp") {
                        slide.show_master_shapes = val == "1" || val == "true";
                    }
                    if let Some(val) = get_attr_val(e, "showMasterPhAnim") {
                        slide.show_master_ph_anim = val == "1" || val == "true";
                    }
                }
                "cSld" => {
                    slide.common_slide_data = parse_common_slide_data(&mut reader)?;
                }
                "clrMapOvr" => {
                    slide.color_map_override = Some(parse_color_map_override(&mut reader)?);
                }
                "transition" => {
                    slide.transition = Some(parse_slide_transition(&mut reader, e, false)?);
                }
                "timing" => {
                    slide.timing = Some(parse_slide_timing(&mut reader)?);
                }
                _ => {}
            },
            Event::Empty(ref e) => {
                if local_name(e.name().into_inner()) == "transition" {
                    slide.transition = Some(parse_slide_transition(&mut reader, e, true)?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(slide)
}

pub fn parse_notes_slide(xml: &[u8]) -> Result<NotesSlide, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut notes = NotesSlide {
        show_master_shapes: true,
        ..Default::default()
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "notes" => {
                    if let Some(val) = get_attr_val(e, "showMasterSp") {
                        notes.show_master_shapes = val == "1" || val == "true";
                    }
                }
                "cSld" => {
                    notes.common_slide_data = parse_common_slide_data(&mut reader)?;
                }
                "clrMapOvr" => {
                    notes.color_map_override = Some(parse_color_map_override(&mut reader)?);
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(notes)
}

// =========================================================================
// 5. Common Slide Data Parser (`p:cSld`)
// =========================================================================

fn parse_common_slide_data(reader: &mut Reader<&[u8]>) -> Result<CommonSlideData, EngineError> {
    let mut data = CommonSlideData::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "bg" => {
                    data.background = Some(parse_background(reader)?);
                }
                "spTree" => {
                    data.shape_tree = parse_shape_tree(reader)?;
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "cSld" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(data)
}

fn parse_background(reader: &mut Reader<&[u8]>) -> Result<Background, EngineError> {
    let mut bg = Background::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "bgRef" => {
                    bg.bg_ref_idx = parse_u32_attr(e, "idx");
                    bg.bg_ref_color = parse_child_color_choice(reader, "bgRef")?;
                }
                "solidFill" => {
                    bg.fill = parse_solid_fill(reader)?;
                }
                _ => {}
            },
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "bgRef" => {
                    bg.bg_ref_idx = parse_u32_attr(e, "idx");
                }
                "noFill" => {
                    bg.fill = Some(Fill::NoFill);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "bg" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(bg)
}

// =========================================================================
// 6. Shape Tree Parser (`p:spTree`, `p:sp`)
// =========================================================================

fn parse_shape_tree(reader: &mut Reader<&[u8]>) -> Result<ShapeTree, EngineError> {
    let mut tree = ShapeTree::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "sp" => {
                    tree.shapes
                        .push(ShapeElement::Shape(Box::new(parse_shape(reader)?)));
                }
                "grpSp" => {
                    tree.shapes
                        .push(ShapeElement::Group(parse_group_shape(reader)?));
                }
                "graphicFrame" => {
                    tree.shapes
                        .push(ShapeElement::GraphicFrame(parse_graphic_frame(reader)?));
                }
                "grpSpPr" => {
                    tree.group_transform = parse_group_transform(reader)?;
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "spTree" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(tree)
}

fn parse_shape(reader: &mut Reader<&[u8]>) -> Result<Shape, EngineError> {
    let mut shape = Shape::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "nvSpPr" => {
                    parse_non_visual_shape_properties(reader, &mut shape)?;
                }
                "spPr" => {
                    shape.properties = parse_shape_properties(reader)?;
                }
                "txBody" => {
                    shape.text_body = Some(parse_text_body(reader)?);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "sp" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(shape)
}

fn parse_group_shape(reader: &mut Reader<&[u8]>) -> Result<GroupShape, EngineError> {
    let mut group = GroupShape::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "nvGrpSpPr" => {
                    parse_non_visual_group_properties(reader, &mut group)?;
                }
                "grpSpPr" => {
                    group.transform = parse_group_transform(reader)?;
                }
                "sp" => {
                    group
                        .children
                        .push(ShapeElement::Shape(Box::new(parse_shape(reader)?)));
                }
                "grpSp" => {
                    group
                        .children
                        .push(ShapeElement::Group(parse_group_shape(reader)?));
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "grpSp" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(group)
}

fn parse_graphic_frame(reader: &mut Reader<&[u8]>) -> Result<GraphicFrame, EngineError> {
    let mut frame = GraphicFrame::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "nvGraphicFramePr" => {
                    let mut b2 = Vec::new();
                    loop {
                        match reader.read_event_into(&mut b2).map_err(EngineError::Xml)? {
                            Event::Empty(ref e2)
                                if local_name(e2.name().into_inner()) == "cNvPr" =>
                            {
                                frame.id = parse_u32_attr(e2, "id").unwrap_or(0);
                                frame.name = get_attr_val(e2, "name").unwrap_or_default();
                            }
                            Event::End(ref e2)
                                if local_name(e2.name().into_inner()) == "nvGraphicFramePr" =>
                            {
                                break
                            }
                            Event::Eof => break,
                            _ => {}
                        }
                        b2.clear();
                    }
                }
                "xfrm" => {
                    frame.transform = Some(parse_transform_2d(e, reader)?);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "graphicFrame" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(frame)
}

fn parse_non_visual_shape_properties(
    reader: &mut Reader<&[u8]>,
    shape: &mut Shape,
) -> Result<(), EngineError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "cNvPr" => {
                    shape.id = parse_u32_attr(e, "id").unwrap_or(0);
                    shape.name = get_attr_val(e, "name").unwrap_or_default();
                }
                "cNvSpPr" => {
                    shape.is_text_box = parse_bool_attr(e, "txBox");
                }
                "ph" => {
                    shape.placeholder = Some(Placeholder {
                        ph_type: get_attr_val(e, "type").unwrap_or_else(|| "body".to_string()),
                        index: parse_u32_attr(e, "idx"),
                        orientation: get_attr_val(e, "orient"),
                        size: get_attr_val(e, "sz"),
                    });
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "nvSpPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn parse_non_visual_group_properties(
    reader: &mut Reader<&[u8]>,
    group: &mut GroupShape,
) -> Result<(), EngineError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "cNvPr" => {
                group.id = parse_u32_attr(e, "id").unwrap_or(0);
                group.name = get_attr_val(e, "name").unwrap_or_default();
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "nvGrpSpPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

// =========================================================================
// 7. Shape Properties & Transforms (`p:spPr`, `a:xfrm`)
// =========================================================================

fn parse_shape_properties(reader: &mut Reader<&[u8]>) -> Result<ShapeProperties, EngineError> {
    let mut props = ShapeProperties::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "xfrm" => {
                    props.transform = Some(parse_transform_2d(e, reader)?);
                }
                "prstGeom" => {
                    let prst = get_attr_val(e, "prst").unwrap_or_else(|| "rect".to_string());
                    let adjusts = parse_adjust_list(reader, "prstGeom")?;
                    if adjusts.is_empty() {
                        props.geometry = Some(Geometry::Preset(prst));
                    } else {
                        props.geometry = Some(Geometry::PresetWithAdjust {
                            preset: prst,
                            adjust_values: adjusts,
                        });
                    }
                }
                "custGeom" => {
                    let cust = parse_custom_geometry(reader)?;
                    props.geometry = Some(Geometry::Custom(Box::new(cust)));
                }
                "solidFill" => {
                    props.fill = parse_solid_fill(reader)?;
                }
                "ln" => {
                    props.line = Some(parse_line_properties(e, reader)?);
                }
                _ => {}
            },
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "noFill" => {
                    props.fill = Some(Fill::NoFill);
                }
                "prstGeom" => {
                    let prst = get_attr_val(e, "prst").unwrap_or_else(|| "rect".to_string());
                    props.geometry = Some(Geometry::Preset(prst));
                }
                "custGeom" => {
                    props.geometry = Some(Geometry::Custom(Box::default()));
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "spPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(props)
}

fn parse_transform_2d(
    e_xfrm: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Transform2D, EngineError> {
    let mut xfrm = Transform2D {
        rot: parse_i32_attr(e_xfrm, "rot").unwrap_or(0),
        flip_h: parse_bool_attr(e_xfrm, "flipH"),
        flip_v: parse_bool_attr(e_xfrm, "flipV"),
        ..Default::default()
    };

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "off" => {
                    xfrm.x = parse_i64_attr(e, "x").unwrap_or(0);
                    xfrm.y = parse_i64_attr(e, "y").unwrap_or(0);
                }
                "ext" => {
                    xfrm.cx = parse_u32_attr(e, "cx").unwrap_or(0);
                    xfrm.cy = parse_u32_attr(e, "cy").unwrap_or(0);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "xfrm" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(xfrm)
}

fn parse_group_transform(
    reader: &mut Reader<&[u8]>,
) -> Result<Option<GroupTransform2D>, EngineError> {
    let mut grp_xfrm: Option<GroupTransform2D> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "xfrm" => {
                let mut gx = GroupTransform2D::default();
                let mut b2 = Vec::new();
                loop {
                    match reader.read_event_into(&mut b2).map_err(EngineError::Xml)? {
                        Event::Empty(ref e2) => match local_name(e2.name().into_inner()) {
                            "off" => {
                                gx.x = parse_i64_attr(e2, "x").unwrap_or(0);
                                gx.y = parse_i64_attr(e2, "y").unwrap_or(0);
                            }
                            "ext" => {
                                gx.cx = parse_u32_attr(e2, "cx").unwrap_or(0);
                                gx.cy = parse_u32_attr(e2, "cy").unwrap_or(0);
                            }
                            "chOff" => {
                                gx.ch_x = parse_i64_attr(e2, "x").unwrap_or(0);
                                gx.ch_y = parse_i64_attr(e2, "y").unwrap_or(0);
                            }
                            "chExt" => {
                                gx.ch_cx = parse_u32_attr(e2, "cx").unwrap_or(0);
                                gx.ch_cy = parse_u32_attr(e2, "cy").unwrap_or(0);
                            }
                            _ => {}
                        },
                        Event::End(ref e2) if local_name(e2.name().into_inner()) == "xfrm" => break,
                        Event::Eof => break,
                        _ => {}
                    }
                    b2.clear();
                }
                grp_xfrm = Some(gx);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "grpSpPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(grp_xfrm)
}

fn parse_line_properties(
    e_ln: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<LineProperties, EngineError> {
    let mut line = LineProperties {
        width: parse_u32_attr(e_ln, "w"),
        ..Default::default()
    };

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                if local_name(e.name().into_inner()) == "solidFill" {
                    line.fill = parse_solid_fill(reader)?;
                }
            }
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "noFill" => {
                    line.fill = Some(Fill::NoFill);
                }
                "prstDash" => {
                    line.dash_style = get_attr_val(e, "val");
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "ln" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(line)
}

fn parse_solid_fill(reader: &mut Reader<&[u8]>) -> Result<Option<Fill>, EngineError> {
    let color = parse_child_color_choice(reader, "solidFill")?;
    Ok(color.map(Fill::Solid))
}

// =========================================================================
// 8. Text Body & Paragraph Parser (`p:txBody`, `a:p`, `a:r`)
// =========================================================================

fn parse_text_body(reader: &mut Reader<&[u8]>) -> Result<TextBody, EngineError> {
    let mut body = TextBody::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "bodyPr" => {
                    body.body_properties = parse_body_properties(e);
                }
                "lstStyle" => {
                    body.list_style = Some(parse_text_list_style(reader, "lstStyle")?);
                }
                "p" => {
                    body.paragraphs.push(parse_text_paragraph(reader)?);
                }
                _ => {}
            },
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "bodyPr" => {
                body.body_properties = parse_body_properties(e);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "txBody" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(body)
}

fn parse_body_properties(e: &BytesStart) -> BodyProperties {
    BodyProperties {
        anchor: get_attr_val(e, "anchor"),
        vertical: get_attr_val(e, "vert"),
        wrap: get_attr_val(e, "wrap"),
        l_ins: parse_i32_attr(e, "lIns"),
        t_ins: parse_i32_attr(e, "tIns"),
        r_ins: parse_i32_attr(e, "rIns"),
        b_ins: parse_i32_attr(e, "bIns"),
        num_cols: parse_u32_attr(e, "numCol"),
    }
}

fn parse_text_paragraph(reader: &mut Reader<&[u8]>) -> Result<TextParagraph, EngineError> {
    let mut para = TextParagraph::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "pPr" => {
                    para.properties = Some(parse_paragraph_properties(e, reader, "pPr")?);
                }
                "r" => {
                    para.runs.push(parse_text_run(reader)?);
                }
                "fld" => {
                    para.runs.push(parse_text_field(e, reader)?);
                }
                "endParaRPr" => {
                    para.end_paragraph_run_properties =
                        Some(parse_run_properties(e, reader, "endParaRPr")?);
                }
                _ => {}
            },
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "br" => {
                    para.runs.push(TextRunElement::LineBreak(None));
                }
                "pPr" => {
                    para.properties = Some(parse_paragraph_properties_empty(e));
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "p" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(para)
}

fn parse_paragraph_properties_empty(e: &BytesStart) -> TextParagraphProperties {
    TextParagraphProperties {
        level: parse_u32_attr(e, "lvl").map(|l| l as usize),
        align: get_attr_val(e, "algn"),
        mar_l: parse_i32_attr(e, "marL"),
        mar_r: parse_i32_attr(e, "marR"),
        indent: parse_i32_attr(e, "indent"),
        def_tab_sz: parse_u32_attr(e, "defTabSz"),
        ..Default::default()
    }
}

fn parse_paragraph_properties(
    e_p_pr: &BytesStart,
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<TextParagraphProperties, EngineError> {
    let mut props = parse_paragraph_properties_empty(e_p_pr);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "defRPr" => {
                    props.default_run_properties = Some(parse_run_properties(e, reader, "defRPr")?);
                }
                "spcBef" => {
                    props.space_before = parse_spacing(reader, "spcBef")?;
                }
                "spcAft" => {
                    props.space_after = parse_spacing(reader, "spcAft")?;
                }
                "lnSpc" => {
                    props.line_spacing = parse_spacing(reader, "lnSpc")?;
                }
                _ => {}
            },
            Event::Empty(ref e) => {
                if local_name(e.name().into_inner()) == "defRPr" {
                    props.default_run_properties = Some(parse_run_properties_empty(e));
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(props)
}

fn parse_spacing(reader: &mut Reader<&[u8]>, end_tag: &str) -> Result<Option<i32>, EngineError> {
    let mut val = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "spcPts" => {
                    val = parse_i32_attr(e, "val").map(|v| v * 20); // convert 1/100 pt to twips
                }
                "spcPct" => {
                    val = parse_i32_attr(e, "val");
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(val)
}

fn parse_text_run(reader: &mut Reader<&[u8]>) -> Result<TextRunElement, EngineError> {
    let mut text = String::new();
    let mut props = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "rPr" => {
                    props = Some(parse_run_properties(e, reader, "rPr")?);
                }
                "t" => {
                    let mut b_t = Vec::new();
                    if let Ok(Event::Text(t)) = reader.read_event_into(&mut b_t) {
                        text = t.unescape().unwrap_or_default().into_owned();
                    }
                }
                _ => {}
            },
            Event::Empty(ref e) => {
                if local_name(e.name().into_inner()) == "rPr" {
                    props = Some(parse_run_properties_empty(e));
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "r" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(TextRunElement::Text(text, props))
}

fn parse_text_field(
    e_fld: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<TextRunElement, EngineError> {
    let id = get_attr_val(e_fld, "id").unwrap_or_default();
    let field_type = get_attr_val(e_fld, "type").unwrap_or_else(|| "slNum".to_string());
    let mut text = None;
    let mut props = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "rPr" => {
                    props = Some(parse_run_properties(e, reader, "rPr")?);
                }
                "t" => {
                    let mut b_t = Vec::new();
                    if let Ok(Event::Text(t)) = reader.read_event_into(&mut b_t) {
                        text = Some(t.unescape().unwrap_or_default().into_owned());
                    }
                }
                _ => {}
            },
            Event::Empty(ref e) => {
                if local_name(e.name().into_inner()) == "rPr" {
                    props = Some(parse_run_properties_empty(e));
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "fld" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(TextRunElement::Field {
        id,
        field_type,
        text,
        properties: props,
    })
}

fn parse_run_properties_empty(e: &BytesStart) -> TextRunProperties {
    TextRunProperties {
        size: parse_u32_attr(e, "sz"),
        bold: get_attr_val(e, "b").map(|v| v == "1" || v == "true"),
        italic: get_attr_val(e, "i").map(|v| v == "1" || v == "true"),
        underline: get_attr_val(e, "u"),
        strike: get_attr_val(e, "strike"),
        kern: parse_u32_attr(e, "kern"),
        baseline: parse_i32_attr(e, "baseline"),
        lang: get_attr_val(e, "lang"),
        ..Default::default()
    }
}

fn parse_run_properties(
    e_r_pr: &BytesStart,
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<TextRunProperties, EngineError> {
    let mut props = parse_run_properties_empty(e_r_pr);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                if local_name(e.name().into_inner()) == "solidFill" {
                    props.color = parse_child_color_choice(reader, "solidFill")?;
                }
            }
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "latin" => {
                    props.font_latin = get_attr_val(e, "typeface");
                }
                "ea" => {
                    props.font_ea = get_attr_val(e, "typeface");
                }
                "cs" => {
                    props.font_cs = get_attr_val(e, "typeface");
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(props)
}

fn parse_text_list_style(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<TextListStyle, EngineError> {
    let mut style = TextListStyle {
        default_paragraph_properties: None,
        levels: vec![None; 9],
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                if name == "defPPr" {
                    style.default_paragraph_properties =
                        Some(parse_paragraph_properties(e, reader, "defPPr")?);
                } else if name.starts_with("lvl") && name.ends_with("pPr") {
                    if let Ok(lvl_num) = name[3..4].parse::<usize>() {
                        if (1..=9).contains(&lvl_num) {
                            style.levels[lvl_num - 1] =
                                Some(parse_paragraph_properties(e, reader, name)?);
                        }
                    }
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                if name == "defPPr" {
                    style.default_paragraph_properties = Some(parse_paragraph_properties_empty(e));
                } else if name.starts_with("lvl") && name.ends_with("pPr") {
                    if let Ok(lvl_num) = name[3..4].parse::<usize>() {
                        if (1..=9).contains(&lvl_num) {
                            style.levels[lvl_num - 1] = Some(parse_paragraph_properties_empty(e));
                        }
                    }
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(style)
}

// =========================================================================
// 9. DrawingML Theme & Colors Parser (`ppt/theme/*.xml`)
// =========================================================================

pub fn parse_theme(xml: &[u8]) -> Result<Theme, EngineError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut theme = Theme::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "theme" => {
                    theme.name =
                        get_attr_val(e, "name").unwrap_or_else(|| "Office Theme".to_string());
                }
                "clrScheme" => {
                    theme.color_scheme = parse_color_scheme(e, &mut reader)?;
                }
                "fontScheme" => {
                    theme.font_scheme = parse_font_scheme(e, &mut reader)?;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(theme)
}

fn parse_color_scheme(
    e_scheme: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<ColorScheme, EngineError> {
    let name = get_attr_val(e_scheme, "name").unwrap_or_else(|| "Office".to_string());
    let mut colors = BTreeMap::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let color_role = local_name(e.name().into_inner()).to_string();
                if let Some(color) = parse_child_color_choice(reader, &color_role)? {
                    colors.insert(color_role, color);
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "clrScheme" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ColorScheme { name, colors })
}

fn parse_child_color_choice(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<Option<ColorChoice>, EngineError> {
    let mut choice = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let tag = local_name(e.name().into_inner());
                match tag {
                    "srgbClr" => {
                        let hex = get_attr_val(e, "val").unwrap_or_else(|| "000000".to_string());
                        choice = Some(ColorChoice::Srgb(hex));
                    }
                    "sysClr" => {
                        let val =
                            get_attr_val(e, "val").unwrap_or_else(|| "windowText".to_string());
                        let last_clr = get_attr_val(e, "lastClr");
                        choice = Some(ColorChoice::System(val, last_clr));
                    }
                    "schemeClr" => {
                        let val = get_attr_val(e, "val").unwrap_or_else(|| "tx1".to_string());
                        let transforms = parse_color_transforms(reader, "schemeClr")?;
                        choice = Some(ColorChoice::Scheme(val, transforms));
                    }
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                let tag = local_name(e.name().into_inner());
                match tag {
                    "srgbClr" => {
                        let hex = get_attr_val(e, "val").unwrap_or_else(|| "000000".to_string());
                        choice = Some(ColorChoice::Srgb(hex));
                    }
                    "sysClr" => {
                        let val =
                            get_attr_val(e, "val").unwrap_or_else(|| "windowText".to_string());
                        let last_clr = get_attr_val(e, "lastClr");
                        choice = Some(ColorChoice::System(val, last_clr));
                    }
                    "schemeClr" => {
                        let val = get_attr_val(e, "val").unwrap_or_else(|| "tx1".to_string());
                        choice = Some(ColorChoice::Scheme(val, Vec::new()));
                    }
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(choice)
}

fn parse_color_transforms(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<Vec<ColorTransform>, EngineError> {
    let mut transforms = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "tint" => {
                    if let Some(v) = parse_u32_attr(e, "val") {
                        transforms.push(ColorTransform::Tint(v));
                    }
                }
                "shade" => {
                    if let Some(v) = parse_u32_attr(e, "val") {
                        transforms.push(ColorTransform::Shade(v));
                    }
                }
                "lumMod" => {
                    if let Some(v) = parse_u32_attr(e, "val") {
                        transforms.push(ColorTransform::LumMod(v));
                    }
                }
                "lumOff" => {
                    if let Some(v) = parse_u32_attr(e, "val") {
                        transforms.push(ColorTransform::LumOff(v));
                    }
                }
                "alpha" => {
                    if let Some(v) = parse_u32_attr(e, "val") {
                        transforms.push(ColorTransform::Alpha(v));
                    }
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(transforms)
}

fn parse_font_scheme(
    e_font_scheme: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<FontScheme, EngineError> {
    let name = get_attr_val(e_font_scheme, "name").unwrap_or_else(|| "Office".to_string());
    let mut major_font = FontCollection::default();
    let mut minor_font = FontCollection::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "majorFont" => {
                    major_font = parse_font_collection(reader, "majorFont")?;
                }
                "minorFont" => {
                    minor_font = parse_font_collection(reader, "minorFont")?;
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "fontScheme" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(FontScheme {
        name,
        major_font,
        minor_font,
    })
}

fn parse_font_collection(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<FontCollection, EngineError> {
    let mut col = FontCollection::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "latin" => {
                    col.latin = get_attr_val(e, "typeface");
                }
                "ea" => {
                    col.ea = get_attr_val(e, "typeface");
                }
                "cs" => {
                    col.cs = get_attr_val(e, "typeface");
                }
                "font" => {
                    if let (Some(script), Some(tf)) =
                        (get_attr_val(e, "script"), get_attr_val(e, "typeface"))
                    {
                        col.scripts.insert(script, tf);
                    }
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(col)
}

// =========================================================================
// 10. Color Map & Override Parsers
// =========================================================================

fn parse_color_map_tag(e: &BytesStart) -> ColorMap {
    ColorMap {
        bg1: get_attr_val(e, "bg1").unwrap_or_else(|| "lt1".to_string()),
        tx1: get_attr_val(e, "tx1").unwrap_or_else(|| "dk1".to_string()),
        bg2: get_attr_val(e, "bg2").unwrap_or_else(|| "lt2".to_string()),
        tx2: get_attr_val(e, "tx2").unwrap_or_else(|| "dk2".to_string()),
        accent1: get_attr_val(e, "accent1").unwrap_or_else(|| "accent1".to_string()),
        accent2: get_attr_val(e, "accent2").unwrap_or_else(|| "accent2".to_string()),
        accent3: get_attr_val(e, "accent3").unwrap_or_else(|| "accent3".to_string()),
        accent4: get_attr_val(e, "accent4").unwrap_or_else(|| "accent4".to_string()),
        accent5: get_attr_val(e, "accent5").unwrap_or_else(|| "accent5".to_string()),
        accent6: get_attr_val(e, "accent6").unwrap_or_else(|| "accent6".to_string()),
        hlink: get_attr_val(e, "hlink").unwrap_or_else(|| "hlink".to_string()),
        fol_hlink: get_attr_val(e, "folHlink").unwrap_or_else(|| "folHlink".to_string()),
    }
}

fn parse_color_map_override(reader: &mut Reader<&[u8]>) -> Result<ColorMapOverride, EngineError> {
    let mut ovr = ColorMapOverride::UseMaster;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "masterClrMapping" => {
                    ovr = ColorMapOverride::UseMaster;
                }
                "overrideClrMapping" => {
                    ovr = ColorMapOverride::Override(Box::new(parse_color_map_tag(e)));
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "clrMapOvr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ovr)
}

fn parse_adjust_list(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<Vec<GeomGuide>, EngineError> {
    let mut adjusts = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "gd" => {
                if let (Some(name), Some(formula)) =
                    (get_attr_val(e, "name"), get_attr_val(e, "fmla"))
                {
                    adjusts.push(GeomGuide::new(name, formula));
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(adjusts)
}

fn parse_custom_geometry(reader: &mut Reader<&[u8]>) -> Result<CustomGeometry, EngineError> {
    let mut geom = CustomGeometry::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "avLst" => {
                    geom.adjust_values = parse_adjust_list(reader, "avLst")?;
                }
                "gdLst" => {
                    geom.guides = parse_adjust_list(reader, "gdLst")?;
                }
                "pathLst" => {
                    geom.paths = parse_path_list(reader)?;
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "custGeom" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(geom)
}

fn parse_path_list(reader: &mut Reader<&[u8]>) -> Result<Vec<CustomPath>, EngineError> {
    let mut paths = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "path" => {
                let w = parse_i64_attr(e, "w");
                let h = parse_i64_attr(e, "h");
                let stroke = parse_bool_attr(e, "stroke");
                let fill = match get_attr_val(e, "fill").as_deref() {
                    Some("none") => PathFillMode::None,
                    Some("lighten") => PathFillMode::Lighten,
                    Some("lightenLess") => PathFillMode::LightenLess,
                    Some("darken") => PathFillMode::Darken,
                    Some("darkenLess") => PathFillMode::DarkenLess,
                    _ => PathFillMode::Norm,
                };
                let instructions = parse_path_instructions(reader)?;
                paths.push(CustomPath {
                    width: w,
                    height: h,
                    fill,
                    stroke,
                    instructions,
                });
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "pathLst" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(paths)
}

fn parse_path_instructions(
    reader: &mut Reader<&[u8]>,
) -> Result<Vec<PathInstruction>, EngineError> {
    let mut instructions = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => match local_name(e.name().into_inner()) {
                "moveTo" => {
                    let (x, y) = parse_point(reader, "moveTo")?;
                    instructions.push(PathInstruction::MoveTo { x, y });
                }
                "lnTo" => {
                    let (x, y) = parse_point(reader, "lnTo")?;
                    instructions.push(PathInstruction::LineTo { x, y });
                }
                "quadBezTo" => {
                    let pts = parse_points(reader, "quadBezTo")?;
                    if pts.len() >= 2 {
                        instructions.push(PathInstruction::QuadBezTo {
                            ctrl_x: pts[0].0.clone(),
                            ctrl_y: pts[0].1.clone(),
                            x: pts[1].0.clone(),
                            y: pts[1].1.clone(),
                        });
                    }
                }
                "cubicBezTo" => {
                    let pts = parse_points(reader, "cubicBezTo")?;
                    if pts.len() >= 3 {
                        instructions.push(PathInstruction::CubicBezTo {
                            ctrl1_x: pts[0].0.clone(),
                            ctrl1_y: pts[0].1.clone(),
                            ctrl2_x: pts[1].0.clone(),
                            ctrl2_y: pts[1].1.clone(),
                            x: pts[2].0.clone(),
                            y: pts[2].1.clone(),
                        });
                    }
                }
                _ => {}
            },
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "arcTo" => {
                    let w_r = get_attr_val(e, "wR").unwrap_or_else(|| "0".to_string());
                    let h_r = get_attr_val(e, "hR").unwrap_or_else(|| "0".to_string());
                    let start_angle = get_attr_val(e, "stAng").unwrap_or_else(|| "0".to_string());
                    let swing_angle = get_attr_val(e, "swAng").unwrap_or_else(|| "0".to_string());
                    instructions.push(PathInstruction::ArcTo {
                        w_r,
                        h_r,
                        start_angle,
                        swing_angle,
                    });
                }
                "close" => {
                    instructions.push(PathInstruction::Close);
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "path" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(instructions)
}

fn parse_point(reader: &mut Reader<&[u8]>, end_tag: &str) -> Result<(String, String), EngineError> {
    let mut x = "0".to_string();
    let mut y = "0".to_string();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "pt" => {
                if let Some(val) = get_attr_val(e, "x") {
                    x = val;
                }
                if let Some(val) = get_attr_val(e, "y") {
                    y = val;
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok((x, y))
}

fn parse_points(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<Vec<(String, String)>, EngineError> {
    let mut pts = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "pt" => {
                let x = get_attr_val(e, "x").unwrap_or_else(|| "0".to_string());
                let y = get_attr_val(e, "y").unwrap_or_else(|| "0".to_string());
                pts.push((x, y));
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(pts)
}

fn parse_slide_timing(reader: &mut Reader<&[u8]>) -> Result<SlideTiming, EngineError> {
    let mut timing = SlideTiming::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                if local_name(e.name().into_inner()) == "tnLst" {
                    timing.root_node = parse_time_node_list(reader)?;
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "timing" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(timing)
}

fn parse_time_node_list(reader: &mut Reader<&[u8]>) -> Result<Option<TimeNode>, EngineError> {
    let mut root = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let tag = local_name(e.name().into_inner()).to_string();
                if tag == "par" || tag == "seq" {
                    root = Some(parse_time_node(reader, &tag)?);
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tnLst" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(root)
}

fn parse_time_node(
    reader: &mut Reader<&[u8]>,
    container_tag: &str,
) -> Result<TimeNode, EngineError> {
    let mut common = CommonTimeNode::default();
    let mut children = Vec::new();
    let mut motion = None;
    let mut rotation = None;
    let mut scale = None;
    let mut color = None;
    let mut effect = None;
    let mut generic = None;
    let mut set = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let tag = local_name(e.name().into_inner()).to_string();
                match tag.as_str() {
                    "cTn" => {
                        common = parse_common_time_node(e, reader)?;
                    }
                    "par" | "seq" => {
                        children.push(parse_time_node(reader, &tag)?);
                    }
                    "animMotion" => {
                        motion = Some(parse_anim_motion(reader)?);
                    }
                    "animRot" => {
                        rotation = Some(parse_anim_rot(e, reader)?);
                    }
                    "animScale" => {
                        scale = Some(parse_anim_scale(reader)?);
                    }
                    "animClr" => {
                        color = Some(parse_anim_color(e, reader)?);
                    }
                    "animEffect" => {
                        effect = Some(parse_anim_effect(e, reader)?);
                    }
                    "anim" => {
                        generic = Some(parse_generic_anim(reader)?);
                    }
                    "set" => {
                        set = Some(parse_set_anim(reader)?);
                    }
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == container_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    let kind = if let Some(m) = motion {
        TimeNodeKind::Motion(m)
    } else if let Some(r) = rotation {
        TimeNodeKind::Rotation(r)
    } else if let Some(s) = scale {
        TimeNodeKind::Scale(s)
    } else if let Some(c) = color {
        TimeNodeKind::Color(c)
    } else if let Some(eff) = effect {
        TimeNodeKind::Effect(eff)
    } else if let Some(g) = generic {
        TimeNodeKind::Generic(g)
    } else if let Some(st) = set {
        TimeNodeKind::Set(st)
    } else if container_tag == "seq" {
        TimeNodeKind::Sequential(children)
    } else {
        TimeNodeKind::Parallel(children)
    };

    Ok(TimeNode { common, kind })
}

fn parse_common_time_node(
    e_ctn: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<CommonTimeNode, EngineError> {
    let id = parse_u32_attr(e_ctn, "id").unwrap_or(0);
    let duration_ms = get_attr_val(e_ctn, "dur").and_then(|s| {
        if s == "indefinite" {
            None
        } else {
            s.parse::<u64>().ok()
        }
    });
    let auto_reverse = parse_bool_attr(e_ctn, "autoRev");
    let accel_ratio = parse_i64_attr(e_ctn, "accel")
        .map(|v| v as f32 / 100_000.0)
        .unwrap_or(0.0);
    let decel_ratio = parse_i64_attr(e_ctn, "decel")
        .map(|v| v as f32 / 100_000.0)
        .unwrap_or(0.0);
    let trigger_type = match get_attr_val(e_ctn, "nodeType").as_deref() {
        Some("clickEffect") => TriggerType::OnClick,
        Some("withEffect") => TriggerType::WithPrev,
        Some("afterEffect") => TriggerType::AfterPrev,
        _ => TriggerType::WithPrev,
    };

    let mut delay_ms = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "cond" => {
                if let Some(d) = parse_i64_attr(e, "delay") {
                    delay_ms = d;
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "cTn" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(CommonTimeNode {
        id,
        duration_ms,
        delay_ms,
        repeat_count: Some(1.0),
        auto_reverse,
        accel_ratio,
        decel_ratio,
        trigger_type,
        fill: TimeNodeFill::Hold,
        restart: TimeNodeRestart::Always,
    })
}

fn parse_anim_motion(reader: &mut Reader<&[u8]>) -> Result<MotionAnimation, EngineError> {
    let mut target = TargetElement::default();
    let mut path = String::new();
    let mut rotate_to_path = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "spTgt" => {
                    target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
                }
                "animMotion" => {
                    if let Some(p) = get_attr_val(e, "path") {
                        path = p;
                    }
                    if let Some(r) = get_attr_val(e, "rAng") {
                        rotate_to_path = r == "1" || r == "true";
                    }
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "animMotion" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(MotionAnimation {
        target,
        path,
        rotate_to_path,
    })
}

fn parse_anim_rot(
    e_rot: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<RotationAnimation, EngineError> {
    let by_deg = parse_i64_attr(e_rot, "by").map(|by| by as f64 / 60_000.0);
    let mut target = TargetElement::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "spTgt" => {
                target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "animRot" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(RotationAnimation {
        target,
        from_deg: None,
        to_deg: None,
        by_deg,
    })
}

fn parse_anim_scale(reader: &mut Reader<&[u8]>) -> Result<ScaleAnimation, EngineError> {
    let mut target = TargetElement::default();
    let mut by_scale = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "spTgt" => {
                    target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
                }
                "by" => {
                    let bx = parse_i64_attr(e, "x")
                        .map(|v| v as f64 / 100_000.0)
                        .unwrap_or(1.0);
                    let by = parse_i64_attr(e, "y")
                        .map(|v| v as f64 / 100_000.0)
                        .unwrap_or(1.0);
                    by_scale = Some((bx, by));
                }
                _ => {}
            },
            Event::End(ref e) if local_name(e.name().into_inner()) == "animScale" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ScaleAnimation {
        target,
        from_scale: Some((1.0, 1.0)),
        to_scale: None,
        by_scale,
    })
}

fn parse_anim_color(
    e_clr: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<ColorAnimation, EngineError> {
    let color_space = match get_attr_val(e_clr, "clrSpc").as_deref() {
        Some("hsl") => ColorSpace::Hsl,
        _ => ColorSpace::Rgb,
    };
    let direction = match get_attr_val(e_clr, "dir").as_deref() {
        Some("ccw") => ColorDirection::CounterClockwise,
        _ => ColorDirection::Clockwise,
    };

    let mut target = TargetElement::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "spTgt" => {
                target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "animClr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ColorAnimation {
        target,
        color_space,
        direction,
        from_rgb: None,
        to_rgb: None,
    })
}

fn parse_anim_effect(
    e_eff: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<EffectAnimation, EngineError> {
    let effect_type = get_attr_val(e_eff, "type").unwrap_or_else(|| "fade".to_string());
    let transition = match get_attr_val(e_eff, "transition").as_deref() {
        Some("out") => EffectTransition::Out,
        _ => EffectTransition::In,
    };

    let mut target = TargetElement::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "spTgt" => {
                target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "animEffect" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(EffectAnimation {
        target,
        effect_type,
        transition,
    })
}

fn parse_generic_anim(reader: &mut Reader<&[u8]>) -> Result<GenericAnimation, EngineError> {
    let mut target = TargetElement::default();
    let mut attribute_name = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "spTgt" => {
                target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "attrName" => {
                let mut b = Vec::new();
                if let Ok(Event::Text(t)) = reader.read_event_into(&mut b) {
                    attribute_name = t.unescape().unwrap_or_default().into_owned();
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "anim" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(GenericAnimation {
        target,
        attribute_name,
        from_val: None,
        to_val: None,
        by_val: None,
    })
}

fn parse_set_anim(reader: &mut Reader<&[u8]>) -> Result<SetAnimation, EngineError> {
    let mut target = TargetElement::default();
    let mut attribute_name = String::new();
    let mut value = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) => match local_name(e.name().into_inner()) {
                "spTgt" => {
                    target.shape_id = parse_u32_attr(e, "spid").unwrap_or(0);
                }
                "strVal" => {
                    value = get_attr_val(e, "val").unwrap_or_default();
                }
                _ => {}
            },
            Event::Start(ref e) if local_name(e.name().into_inner()) == "attrName" => {
                let mut b = Vec::new();
                if let Ok(Event::Text(t)) = reader.read_event_into(&mut b) {
                    attribute_name = t.unescape().unwrap_or_default().into_owned();
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "set" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(SetAnimation {
        target,
        attribute_name,
        value,
    })
}

fn parse_slide_transition(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    is_empty: bool,
) -> Result<SlideTransition, EngineError> {
    let mut transition = SlideTransition {
        speed: get_attr_val(start, "spd"),
        advance_on_click: get_attr_val(start, "advClick")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(true),
        advance_after_ms: get_attr_val(start, "advTm").and_then(|v| v.parse::<u64>().ok()),
        duration_ms: get_attr_val(start, "dur").and_then(|v| v.parse::<u64>().ok()),
        ..Default::default()
    };

    if is_empty {
        return Ok(transition);
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let tag = local_name(e.name().into_inner());
                if tag == "morph" {
                    let opt_str = get_attr_val(e, "option").unwrap_or_default();
                    let option = match opt_str.as_str() {
                        "byWord" => MorphOption::ByWord,
                        "byChar" => MorphOption::ByChar,
                        _ => MorphOption::ByObject,
                    };
                    transition.morph = Some(MorphTransition { option });
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "transition" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(transition)
}
