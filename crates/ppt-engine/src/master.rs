#![forbid(unsafe_code)]
//! PresentationML Slide Master & Layout Designer View Engine.
//!
//! Implements:
//! - Complete Master -> Layout -> Slide template hierarchy.
//! - Standard layout templates (Title, Title & Content, Two Content, Comparison, Blank, etc.).
//! - Placeholder creation, positioning, transform updates, and deletion.
//! - Layout duplication, creation, deletion, and slide layout reassignment.
//! - Extraction of serializable MasterLayoutTreeDto for reactive UI presentation.

use ooxml_opc::error::EngineError;
use ooxml_opc::relationships::TargetMode;
use serde::{Deserialize, Serialize};

use crate::model::*;
use crate::package::{
    resolve_target_path, PresentationPackage, SLIDE_LAYOUT_REL, SLIDE_LAYOUT_TYPE, SLIDE_MASTER_REL,
};

/// Serializable placeholder metadata DTO for UI presentation and manipulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaceholderDto {
    pub ph_type: String,
    pub index: Option<u32>,
    pub shape_id: u32,
    pub name: String,
    pub x: i64,
    pub y: i64,
    pub cx: u32,
    pub cy: u32,
    pub prompt_text: String,
}

/// Serializable slide layout metadata DTO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlideLayoutDto {
    pub part_name: String,
    pub master_part_name: String,
    pub layout_type: String,
    pub name: String,
    pub show_master_shapes: bool,
    pub placeholders: Vec<PlaceholderDto>,
}

/// Serializable slide master metadata DTO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlideMasterDto {
    pub part_name: String,
    pub id: u32,
    pub name: String,
    pub theme_name: String,
    pub background_color: Option<String>,
    pub placeholders: Vec<PlaceholderDto>,
    pub layouts: Vec<SlideLayoutDto>,
}

/// Root presentation master hierarchy tree DTO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MasterLayoutTreeDto {
    pub masters: Vec<SlideMasterDto>,
    pub slide_size: SlideSize,
}

/// Interactive Slide Master and Layout Designer engine.
pub struct SlideMasterDesigner;

impl SlideMasterDesigner {
    /// Extracts the full hierarchical master and layout tree from a presentation package.
    pub fn extract_master_tree(pkg: &PresentationPackage) -> MasterLayoutTreeDto {
        let mut masters = Vec::new();

        for (master_part, master) in &pkg.slide_masters {
            let mut theme_name = "Office Theme".to_string();

            // Find theme name from master relationships
            if let Some(master_rels) = pkg.part_relationships.get(master_part) {
                for rel in master_rels.relationships.values() {
                    if rel.rel_type.ends_with("/theme") {
                        let theme_part = resolve_target_path(master_part, &rel.target);
                        if let Some(theme) = pkg.themes.get(&theme_part) {
                            theme_name = theme.name.clone();
                        }
                    }
                }
            }

            // Extract background color if solid
            let background_color =
                master
                    .common_slide_data
                    .background
                    .as_ref()
                    .and_then(|bg| match &bg.fill {
                        Some(Fill::Solid(ColorChoice::Srgb(hex))) => Some(format!("#{hex}")),
                        _ => None,
                    });

            let master_placeholders =
                Self::extract_placeholders(&master.common_slide_data.shape_tree.shapes);

            // Discover child layouts linked to this master
            let mut layouts = Vec::new();
            if let Some(master_rels) = pkg.part_relationships.get(master_part) {
                for rel in master_rels.relationships.values() {
                    if rel.rel_type == SLIDE_LAYOUT_REL {
                        let layout_part = resolve_target_path(master_part, &rel.target);
                        if let Some(layout) = pkg.slide_layouts.get(&layout_part) {
                            let layout_placeholders = Self::extract_placeholders(
                                &layout.common_slide_data.shape_tree.shapes,
                            );
                            let layout_type = layout
                                .layout_type
                                .clone()
                                .unwrap_or_else(|| "custom".to_string());
                            let layout_name = layout
                                .matching_name
                                .clone()
                                .or_else(|| layout.common_slide_data.name.clone())
                                .unwrap_or_else(|| Self::friendly_layout_name(&layout_type));

                            layouts.push(SlideLayoutDto {
                                part_name: layout_part,
                                master_part_name: master_part.clone(),
                                layout_type,
                                name: layout_name,
                                show_master_shapes: layout.show_master_shapes,
                                placeholders: layout_placeholders,
                            });
                        }
                    }
                }
            }

            // If relationships didn't list layouts, attach any slide_layouts present in package
            if layouts.is_empty() {
                for (layout_part, layout) in &pkg.slide_layouts {
                    let layout_placeholders =
                        Self::extract_placeholders(&layout.common_slide_data.shape_tree.shapes);
                    let layout_type = layout
                        .layout_type
                        .clone()
                        .unwrap_or_else(|| "titleAndContent".to_string());
                    let layout_name = layout
                        .matching_name
                        .clone()
                        .or_else(|| layout.common_slide_data.name.clone())
                        .unwrap_or_else(|| Self::friendly_layout_name(&layout_type));

                    layouts.push(SlideLayoutDto {
                        part_name: layout_part.clone(),
                        master_part_name: master_part.clone(),
                        layout_type,
                        name: layout_name,
                        show_master_shapes: layout.show_master_shapes,
                        placeholders: layout_placeholders,
                    });
                }
            }

            masters.push(SlideMasterDto {
                part_name: master_part.clone(),
                id: 1,
                name: master
                    .common_slide_data
                    .name
                    .clone()
                    .unwrap_or_else(|| "Master Slide".to_string()),
                theme_name,
                background_color,
                placeholders: master_placeholders,
                layouts,
            });
        }

        MasterLayoutTreeDto {
            masters,
            slide_size: pkg.presentation.slide_size,
        }
    }

    /// Extracts all placeholder metadata from a list of shape elements.
    pub fn extract_placeholders(shapes: &[ShapeElement]) -> Vec<PlaceholderDto> {
        let mut out = Vec::new();
        for s in shapes {
            if let ShapeElement::Shape(ref shape) = s {
                if let Some(ref ph) = shape.placeholder {
                    let xfrm = shape.properties.transform.unwrap_or_default();
                    let prompt = shape
                        .text_body
                        .as_ref()
                        .and_then(|tb| tb.paragraphs.first())
                        .and_then(|p| p.runs.first())
                        .map(|r| r.get_text().to_string())
                        .unwrap_or_else(|| Self::default_prompt_text(&ph.ph_type));

                    out.push(PlaceholderDto {
                        ph_type: ph.ph_type.clone(),
                        index: ph.index,
                        shape_id: shape.id,
                        name: shape.name.clone(),
                        x: xfrm.x,
                        y: xfrm.y,
                        cx: xfrm.cx,
                        cy: xfrm.cy,
                        prompt_text: prompt,
                    });
                }
            }
        }
        out
    }

    /// Creates a new slide layout under the specified master.
    pub fn create_layout(
        pkg: &mut PresentationPackage,
        master_part_name: &str,
        layout_type: &str,
        name: &str,
    ) -> Result<String, EngineError> {
        if !pkg.slide_masters.contains_key(master_part_name) {
            return Err(EngineError::PartNotFound(master_part_name.to_string()));
        }

        let next_idx = Self::next_layout_index(pkg);
        let new_layout_part = format!("ppt/slideLayouts/slideLayout{next_idx}.xml");

        // 1. Content Types override
        pkg.content_types
            .add_override(&new_layout_part, SLIDE_LAYOUT_TYPE);

        // 2. Master -> Layout relationship
        let master_rels = pkg
            .part_relationships
            .entry(master_part_name.to_string())
            .or_default();
        let rel_id = format!("rId{}", master_rels.relationships.len() + 1);
        let target_layout_path = format!("../slideLayouts/slideLayout{next_idx}.xml");
        master_rels.add(
            &rel_id,
            SLIDE_LAYOUT_REL,
            &target_layout_path,
            TargetMode::Internal,
        );

        // 3. Layout -> Master relationship
        let mut layout_rels = ooxml_opc::relationships::Relationships::new();
        let master_filename = match master_part_name.rfind('/') {
            Some(pos) => &master_part_name[pos + 1..],
            None => master_part_name,
        };
        let target_master_path = format!("../slideMasters/{master_filename}");
        layout_rels.add(
            "rId1",
            SLIDE_MASTER_REL,
            &target_master_path,
            TargetMode::Internal,
        );
        pkg.part_relationships
            .insert(new_layout_part.clone(), layout_rels);

        // 4. Generate layout shapes according to standard layout type
        let shapes = Self::generate_default_layout_shapes(layout_type);

        let layout = SlideLayout {
            layout_type: Some(layout_type.to_string()),
            matching_name: Some(name.to_string()),
            preserve: true,
            show_master_shapes: true,
            common_slide_data: CommonSlideData {
                name: Some(name.to_string()),
                shape_tree: ShapeTree {
                    shapes,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        pkg.slide_layouts.insert(new_layout_part.clone(), layout);

        Ok(new_layout_part)
    }

    /// Duplicates an existing slide layout.
    pub fn duplicate_layout(
        pkg: &mut PresentationPackage,
        layout_part_name: &str,
        new_name: &str,
    ) -> Result<String, EngineError> {
        let original_layout = pkg
            .slide_layouts
            .get(layout_part_name)
            .ok_or_else(|| EngineError::PartNotFound(layout_part_name.to_string()))?
            .clone();

        // Discover parent master from layout relationships
        let mut master_part = "ppt/slideMasters/slideMaster1.xml".to_string();
        if let Some(rels) = pkg.part_relationships.get(layout_part_name) {
            for rel in rels.relationships.values() {
                if rel.rel_type == SLIDE_MASTER_REL {
                    master_part = resolve_target_path(layout_part_name, &rel.target);
                    break;
                }
            }
        }

        let next_idx = Self::next_layout_index(pkg);
        let new_layout_part = format!("ppt/slideLayouts/slideLayout{next_idx}.xml");

        pkg.content_types
            .add_override(&new_layout_part, SLIDE_LAYOUT_TYPE);

        // Link in master
        let master_rels = pkg
            .part_relationships
            .entry(master_part.clone())
            .or_default();
        let rel_id = format!("rId{}", master_rels.relationships.len() + 1);
        let target_layout_path = format!("../slideLayouts/slideLayout{next_idx}.xml");
        master_rels.add(
            &rel_id,
            SLIDE_LAYOUT_REL,
            &target_layout_path,
            TargetMode::Internal,
        );

        // Link to master in layout rels
        let mut layout_rels = ooxml_opc::relationships::Relationships::new();
        let master_filename = match master_part.rfind('/') {
            Some(pos) => &master_part[pos + 1..],
            None => &master_part,
        };
        let target_master_path = format!("../slideMasters/{master_filename}");
        layout_rels.add(
            "rId1",
            SLIDE_MASTER_REL,
            &target_master_path,
            TargetMode::Internal,
        );
        pkg.part_relationships
            .insert(new_layout_part.clone(), layout_rels);

        let mut cloned = original_layout;
        cloned.matching_name = Some(new_name.to_string());
        cloned.common_slide_data.name = Some(new_name.to_string());

        pkg.slide_layouts.insert(new_layout_part.clone(), cloned);
        Ok(new_layout_part)
    }

    /// Deletes an unused slide layout. Returns error if assigned to existing slides.
    pub fn delete_layout(
        pkg: &mut PresentationPackage,
        layout_part_name: &str,
    ) -> Result<(), EngineError> {
        if !pkg.slide_layouts.contains_key(layout_part_name) {
            return Err(EngineError::PartNotFound(layout_part_name.to_string()));
        }

        // Check whether any slide currently uses this layout
        for (slide_part, _) in &pkg.slides {
            if let Some(rels) = pkg.part_relationships.get(slide_part) {
                for rel in rels.relationships.values() {
                    if rel.rel_type == SLIDE_LAYOUT_REL {
                        let target = resolve_target_path(slide_part, &rel.target);
                        if target == layout_part_name {
                            return Err(EngineError::InvalidOperation(format!(
                                "Cannot delete layout '{layout_part_name}' because it is in use by slide '{slide_part}'"
                            )));
                        }
                    }
                }
            }
        }

        pkg.slide_layouts.remove(layout_part_name);
        pkg.part_relationships.remove(layout_part_name);
        pkg.content_types.remove_override(layout_part_name);

        // Clean up references in masters
        for rels in pkg.part_relationships.values_mut() {
            rels.relationships.retain(|_, rel| {
                if rel.rel_type == SLIDE_LAYOUT_REL {
                    !rel.target.contains(layout_part_name)
                } else {
                    true
                }
            });
        }

        Ok(())
    }

    /// Adds a placeholder shape to a master or layout part.
    pub fn add_placeholder(
        pkg: &mut PresentationPackage,
        target_part: &str,
        ph_type: &str,
        x: i64,
        y: i64,
        cx: u32,
        cy: u32,
    ) -> Result<u32, EngineError> {
        let shape_tree = Self::get_shape_tree_mut(pkg, target_part)?;

        let mut max_id = 0u32;
        for s in &shape_tree.shapes {
            if let ShapeElement::Shape(ref sh) = s {
                max_id = max_id.max(sh.id);
            }
        }
        let next_id = max_id + 1;

        let prompt = Self::default_prompt_text(ph_type);
        let ph_shape = Shape {
            id: next_id,
            name: format!("{ph_type} Placeholder {next_id}"),
            is_text_box: true,
            placeholder: Some(Placeholder {
                ph_type: ph_type.to_string(),
                index: None,
                orientation: None,
                size: None,
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(x, y, cx, cy)),
                geometry: Some(Geometry::Preset("rect".to_string())),
                fill: None,
                line: None,
            },
            text_body: Some(Self::create_text_body(&prompt)),
            raw_elements: Vec::new(),
        };

        shape_tree
            .shapes
            .push(ShapeElement::Shape(Box::new(ph_shape)));
        Ok(next_id)
    }

    /// Removes a placeholder shape from a master or layout part.
    pub fn remove_placeholder(
        pkg: &mut PresentationPackage,
        target_part: &str,
        shape_id: u32,
    ) -> Result<(), EngineError> {
        let shape_tree = Self::get_shape_tree_mut(pkg, target_part)?;
        shape_tree.shapes.retain(|s| match s {
            ShapeElement::Shape(ref sh) => sh.id != shape_id,
            _ => true,
        });
        Ok(())
    }

    /// Updates the 2D bounding transform of a placeholder shape.
    pub fn update_placeholder_transform(
        pkg: &mut PresentationPackage,
        target_part: &str,
        shape_id: u32,
        x: i64,
        y: i64,
        cx: u32,
        cy: u32,
    ) -> Result<(), EngineError> {
        let shape_tree = Self::get_shape_tree_mut(pkg, target_part)?;
        for s in &mut shape_tree.shapes {
            if let ShapeElement::Shape(ref mut sh) = s {
                if sh.id == shape_id {
                    sh.properties.transform = Some(Transform2D::new(x, y, cx, cy));
                    return Ok(());
                }
            }
        }
        Err(EngineError::InvalidOperation(format!(
            "Shape ID {shape_id} not found in part '{target_part}'"
        )))
    }

    /// Applies a slide layout to an existing slide, updating relationships and placeholders.
    pub fn apply_layout_to_slide(
        pkg: &mut PresentationPackage,
        slide_idx: usize,
        layout_part_name: &str,
    ) -> Result<(), EngineError> {
        if !pkg.slide_layouts.contains_key(layout_part_name) {
            return Err(EngineError::PartNotFound(layout_part_name.to_string()));
        }
        if slide_idx >= pkg.slides.len() {
            return Err(EngineError::InvalidOperation(format!(
                "Slide index {slide_idx} out of bounds"
            )));
        }

        let (slide_part, _) = &pkg.slides[slide_idx];
        let slide_part = slide_part.clone();

        // Update relationship in slide rels
        let slide_rels = pkg
            .part_relationships
            .entry(slide_part.clone())
            .or_default();
        let mut rel_key = None;
        for (k, rel) in &slide_rels.relationships {
            if rel.rel_type == SLIDE_LAYOUT_REL {
                rel_key = Some(k.clone());
                break;
            }
        }

        let layout_filename = match layout_part_name.rfind('/') {
            Some(pos) => &layout_part_name[pos + 1..],
            None => layout_part_name,
        };
        let target_relative = format!("../slideLayouts/{layout_filename}");

        if let Some(k) = rel_key {
            slide_rels.add(&k, SLIDE_LAYOUT_REL, &target_relative, TargetMode::Internal);
        } else {
            let next_r_id = format!("rId{}", slide_rels.relationships.len() + 1);
            slide_rels.add(
                &next_r_id,
                SLIDE_LAYOUT_REL,
                &target_relative,
                TargetMode::Internal,
            );
        }

        // Harmonize slide placeholders: ensure placeholders in target layout exist on slide
        let target_layout = pkg
            .slide_layouts
            .get(layout_part_name)
            .cloned()
            .ok_or_else(|| EngineError::PartNotFound(layout_part_name.to_string()))?;
        let (_, slide) = &mut pkg.slides[slide_idx];

        let mut existing_ph_types = std::collections::HashSet::new();
        let mut max_id = 0u32;
        for s in &slide.common_slide_data.shape_tree.shapes {
            if let ShapeElement::Shape(ref sh) = s {
                max_id = max_id.max(sh.id);
                if let Some(ref ph) = sh.placeholder {
                    existing_ph_types.insert(ph.ph_type.clone());
                }
            }
        }

        for s in &target_layout.common_slide_data.shape_tree.shapes {
            if let ShapeElement::Shape(ref layout_sh) = s {
                if let Some(ref ph) = layout_sh.placeholder {
                    if !existing_ph_types.contains(&ph.ph_type)
                        && ph.ph_type != "dt"
                        && ph.ph_type != "ftr"
                        && ph.ph_type != "sldNum"
                    {
                        max_id += 1;
                        let prompt = Self::default_prompt_text(&ph.ph_type);
                        let new_slide_shape = Shape {
                            id: max_id,
                            name: format!("{} {}", Self::friendly_layout_name(&ph.ph_type), max_id),
                            is_text_box: true,
                            placeholder: Some(ph.clone()),
                            properties: layout_sh.properties.clone(),
                            text_body: Some(Self::create_text_body(&prompt)),
                            raw_elements: Vec::new(),
                        };
                        slide
                            .common_slide_data
                            .shape_tree
                            .shapes
                            .push(ShapeElement::Shape(Box::new(new_slide_shape)));
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolves the slide layout part currently assigned to a slide.
    pub fn get_slide_layout_part(pkg: &PresentationPackage, slide_idx: usize) -> Option<String> {
        let (slide_part, _) = pkg.slides.get(slide_idx)?;
        let slide_rels = pkg.part_relationships.get(slide_part)?;
        for rel in slide_rels.relationships.values() {
            if rel.rel_type == SLIDE_LAYOUT_REL {
                return Some(resolve_target_path(slide_part, &rel.target));
            }
        }
        None
    }

    /// Sets `show_master_shapes` on a master, layout, or slide part.
    pub fn set_show_master_shapes(
        pkg: &mut PresentationPackage,
        target_part: &str,
        show: bool,
    ) -> Result<(), EngineError> {
        if let Some(layout) = pkg.slide_layouts.get_mut(target_part) {
            layout.show_master_shapes = show;
            return Ok(());
        }
        for (part, slide) in &mut pkg.slides {
            if part == target_part {
                slide.show_master_shapes = show;
                return Ok(());
            }
        }
        Err(EngineError::PartNotFound(target_part.to_string()))
    }

    /// Generates canonical DrawingML placeholder shapes for standard layout templates.
    pub fn generate_default_layout_shapes(layout_type: &str) -> Vec<ShapeElement> {
        let mut shapes = Vec::new();

        match layout_type {
            "title" => {
                // Title
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Title",
                        "title",
                        None,
                        (838_200, 1_828_800, 10_515_600, 1_828_800),
                    ),
                )));
                // Subtitle
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        2,
                        "Subtitle",
                        "subTitle",
                        Some(1),
                        (838_200, 3_886_200, 10_515_600, 1_524_000),
                    ),
                )));
            }
            "twoContent" => {
                // Title
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Title",
                        "title",
                        None,
                        (838_200, 457_200, 10_515_600, 1_143_000),
                    ),
                )));
                // Left Body
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        2,
                        "Left Content",
                        "body",
                        Some(1),
                        (838_200, 1_828_800, 5_029_200, 4_572_000),
                    ),
                )));
                // Right Body
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        3,
                        "Right Content",
                        "body",
                        Some(2),
                        (6_324_600, 1_828_800, 5_029_200, 4_572_000),
                    ),
                )));
            }
            "sectionHeader" => {
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Section Title",
                        "title",
                        None,
                        (838_200, 2_057_400, 10_515_600, 1_371_600),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        2,
                        "Section Text",
                        "body",
                        Some(1),
                        (838_200, 3_657_600, 10_515_600, 1_371_600),
                    ),
                )));
            }
            "comparison" => {
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Title",
                        "title",
                        None,
                        (838_200, 457_200, 10_515_600, 1_143_000),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        2,
                        "Left Header",
                        "body",
                        Some(1),
                        (838_200, 1_828_800, 5_029_200, 685_800),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        3,
                        "Right Header",
                        "body",
                        Some(2),
                        (6_324_600, 1_828_800, 5_029_200, 685_800),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        4,
                        "Left Content",
                        "body",
                        Some(3),
                        (838_200, 2_667_000, 5_029_200, 3_733_800),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        5,
                        "Right Content",
                        "body",
                        Some(4),
                        (6_324_600, 2_667_000, 5_029_200, 3_733_800),
                    ),
                )));
            }
            "titleOnly" => {
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Title",
                        "title",
                        None,
                        (838_200, 457_200, 10_515_600, 1_143_000),
                    ),
                )));
            }
            "blank" => {
                // Blank layout has no default title or body
            }
            _ => {
                // Default: titleAndContent
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        1,
                        "Title",
                        "title",
                        None,
                        (838_200, 457_200, 10_515_600, 1_143_000),
                    ),
                )));
                shapes.push(ShapeElement::Shape(Box::new(
                    Self::create_placeholder_shape(
                        2,
                        "Content",
                        "body",
                        Some(1),
                        (838_200, 1_828_800, 10_515_600, 4_572_000),
                    ),
                )));
            }
        }

        // Standard footer placeholders (date/time, footer, slide number)
        shapes.push(ShapeElement::Shape(Box::new(
            Self::create_placeholder_shape(
                10,
                "Date Placeholder",
                "dt",
                None,
                (838_200, 6_400_800, 2_438_400, 365_760),
            ),
        )));
        shapes.push(ShapeElement::Shape(Box::new(
            Self::create_placeholder_shape(
                11,
                "Footer Placeholder",
                "ftr",
                None,
                (3_810_000, 6_400_800, 4_572_000, 365_760),
            ),
        )));
        shapes.push(ShapeElement::Shape(Box::new(
            Self::create_placeholder_shape(
                12,
                "Slide Number Placeholder",
                "sldNum",
                None,
                (8_915_400, 6_400_800, 2_438_400, 365_760),
            ),
        )));

        shapes
    }

    /// List of standard layout preset identifiers and human-readable names.
    pub fn standard_layout_presets() -> Vec<(&'static str, &'static str)> {
        vec![
            ("title", "Title Slide"),
            ("titleAndContent", "Title and Content"),
            ("sectionHeader", "Section Header"),
            ("twoContent", "Two Content"),
            ("comparison", "Comparison"),
            ("titleOnly", "Title Only"),
            ("blank", "Blank"),
            ("contentWithCaption", "Content with Caption"),
            ("pictureWithCaption", "Picture with Caption"),
        ]
    }

    // Helper functions

    fn next_layout_index(pkg: &PresentationPackage) -> usize {
        let mut max_idx = 0usize;
        for k in pkg.slide_layouts.keys() {
            if let Some(s) = k.strip_prefix("ppt/slideLayouts/slideLayout") {
                if let Some(num_str) = s.strip_suffix(".xml") {
                    if let Ok(num) = num_str.parse::<usize>() {
                        max_idx = max_idx.max(num);
                    }
                }
            }
        }
        max_idx + 1
    }

    fn get_shape_tree_mut<'a>(
        pkg: &'a mut PresentationPackage,
        target_part: &str,
    ) -> Result<&'a mut ShapeTree, EngineError> {
        if let Some(master) = pkg.slide_masters.get_mut(target_part) {
            return Ok(&mut master.common_slide_data.shape_tree);
        }
        if let Some(layout) = pkg.slide_layouts.get_mut(target_part) {
            return Ok(&mut layout.common_slide_data.shape_tree);
        }
        Err(EngineError::PartNotFound(target_part.to_string()))
    }

    fn create_placeholder_shape(
        id: u32,
        name: &str,
        ph_type: &str,
        index: Option<u32>,
        bounds: (i64, i64, u32, u32),
    ) -> Shape {
        let (x, y, cx, cy) = bounds;
        let prompt = Self::default_prompt_text(ph_type);
        Shape {
            id,
            name: name.to_string(),
            is_text_box: true,
            placeholder: Some(Placeholder {
                ph_type: ph_type.to_string(),
                index,
                orientation: None,
                size: None,
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(x, y, cx, cy)),
                geometry: Some(Geometry::Preset("rect".to_string())),
                fill: None,
                line: None,
            },
            text_body: Some(Self::create_text_body(&prompt)),
            raw_elements: Vec::new(),
        }
    }

    fn create_text_body(text: &str) -> TextBody {
        TextBody {
            body_properties: BodyProperties::default(),
            list_style: None,
            paragraphs: vec![TextParagraph {
                properties: None,
                runs: vec![TextRunElement::Text(text.to_string(), None)],
                end_paragraph_run_properties: None,
            }],
        }
    }

    fn default_prompt_text(ph_type: &str) -> String {
        match ph_type {
            "title" | "ctrTitle" => "Click to edit Master title style".to_string(),
            "subTitle" => "Click to edit Master subtitle style".to_string(),
            "body" => "Click to edit Master text styles".to_string(),
            "dt" => "<date/time>".to_string(),
            "ftr" => "<footer>".to_string(),
            "sldNum" => "<#>".to_string(),
            "pic" => "Click icon to add picture".to_string(),
            "chart" => "Click icon to add chart".to_string(),
            "tbl" => "Click icon to add table".to_string(),
            _ => "Click to edit placeholder".to_string(),
        }
    }

    fn friendly_layout_name(layout_type: &str) -> String {
        match layout_type {
            "title" => "Title Slide".to_string(),
            "titleAndContent" => "Title and Content".to_string(),
            "sectionHeader" => "Section Header".to_string(),
            "twoContent" => "Two Content".to_string(),
            "comparison" => "Comparison".to_string(),
            "titleOnly" => "Title Only".to_string(),
            "blank" => "Blank".to_string(),
            "contentWithCaption" => "Content with Caption".to_string(),
            "pictureWithCaption" => "Picture with Caption".to_string(),
            other => other.to_string(),
        }
    }
}
