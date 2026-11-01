#![forbid(unsafe_code)]
//! PresentationML Shape Grouping, Ungrouping & Z-Order Layer Management Engine.
//!
//! Complies with ECMA-376 Part 1 §19.3 & §20.1 (DrawingML Group Shapes & Z-Ordering).
//! Enforces GEC-v7.0 zero-panic and memory-safety invariants across all operations.

use ooxml_opc::error::EngineError;
use serde::{Deserialize, Serialize};

use crate::model::*;

/// Serializable metadata for an individual layer item in the presentation shell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerItemDto {
    pub id: u32,
    pub name: String,
    pub kind: String, // "shape", "group", "graphicFrame", "smartArt"
    pub z_index: usize,
    pub is_hidden: bool,
    pub is_locked: bool,
    pub x: i64,
    pub y: i64,
    pub cx: u32,
    pub cy: u32,
    pub children: Vec<LayerItemDto>,
}

/// Root slide layer hierarchy tree DTO for presentation shell sidebar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlideLayerTreeDto {
    pub slide_index: usize,
    pub items: Vec<LayerItemDto>,
}

/// Z-Order and grouping management coordinator.
pub struct SlideLayerEngine;

impl SlideLayerEngine {
    /// Extracts the full hierarchical layer tree of a slide for UI rendering.
    pub fn extract_layer_tree(slide: &Slide, slide_index: usize) -> SlideLayerTreeDto {
        let mut items = Vec::new();
        for (idx, elem) in slide.common_slide_data.shape_tree.shapes.iter().enumerate() {
            items.push(Self::convert_element_to_dto(elem, idx));
        }
        SlideLayerTreeDto { slide_index, items }
    }

    /// Groups two or more shapes into a composite `GroupShape`.
    pub fn group_shapes(slide: &mut Slide, shape_ids: &[u32]) -> Result<u32, EngineError> {
        if shape_ids.len() < 2 {
            return Err(EngineError::InvalidOperation(
                "Grouping requires at least two shapes".to_string(),
            ));
        }

        let shapes = &mut slide.common_slide_data.shape_tree.shapes;

        // Find indices of shapes to group
        let mut target_indices = Vec::new();
        for id in shape_ids {
            if let Some(pos) = shapes.iter().position(|s| s.id() == *id) {
                if !target_indices.contains(&pos) {
                    target_indices.push(pos);
                }
            } else {
                return Err(EngineError::PartNotFound(format!(
                    "Shape with id {id} not found on slide"
                )));
            }
        }

        target_indices.sort_unstable();
        let insert_idx = target_indices[0];

        // Find max shape id across slide for generating unique group id
        let mut max_id = 0u32;
        Self::compute_max_id_recursive(shapes, &mut max_id);
        let group_id = max_id + 1;

        // Extract selected shapes preserving their relative order
        let mut selected_elements = Vec::with_capacity(target_indices.len());
        // Remove from highest index to lowest index to keep lower indices valid
        for &idx in target_indices.iter().rev() {
            selected_elements.push(shapes.remove(idx));
        }
        selected_elements.reverse();

        // Calculate collective bounding box in parent coordinates
        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;

        for elem in &selected_elements {
            let (x, y, cx, cy) = Self::get_element_bounds(elem);
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + cx as i64);
            max_y = max_y.max(y + cy as i64);
        }

        if min_x == i64::MAX {
            min_x = 0;
            min_y = 0;
            max_x = 1_000_000;
            max_y = 1_000_000;
        }

        let group_w = (max_x - min_x).max(1) as u32;
        let group_h = (max_y - min_y).max(1) as u32;

        // Offset children coordinates so they are local to (0, 0) inside child space
        for elem in &mut selected_elements {
            Self::offset_element_transform(elem, -min_x, -min_y);
        }

        let group_shape = GroupShape {
            id: group_id,
            name: format!("Group {group_id}"),
            transform: Some(GroupTransform2D {
                x: min_x,
                y: min_y,
                cx: group_w,
                cy: group_h,
                ch_x: 0,
                ch_y: 0,
                ch_cx: group_w,
                ch_cy: group_h,
            }),
            children: selected_elements,
        };

        shapes.insert(insert_idx, ShapeElement::Group(group_shape));
        Ok(group_id)
    }

    /// Ungroups a composite `GroupShape` into individual shapes with restored parent coordinates.
    pub fn ungroup_shapes(slide: &mut Slide, group_id: u32) -> Result<Vec<u32>, EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;

        let group_pos = shapes
            .iter()
            .position(|s| match s {
                ShapeElement::Group(grp) => grp.id == group_id,
                _ => false,
            })
            .ok_or_else(|| {
                EngineError::PartNotFound(format!("Group shape with id {group_id} not found"))
            })?;

        let removed = shapes.remove(group_pos);
        let grp = match removed {
            ShapeElement::Group(g) => g,
            _ => unreachable!(),
        };

        let (grp_x, grp_y) = grp.transform.as_ref().map(|t| (t.x, t.y)).unwrap_or((0, 0));

        let mut child_ids = Vec::with_capacity(grp.children.len());

        for (insert_offset, mut child) in grp.children.into_iter().enumerate() {
            child_ids.push(child.id());
            Self::offset_element_transform(&mut child, grp_x, grp_y);
            shapes.insert(group_pos + insert_offset, child);
        }

        Ok(child_ids)
    }

    /// Moves a shape to the top of the rendering stack (rendered last, visually in front).
    pub fn bring_to_front(slide: &mut Slide, shape_id: u32) -> Result<(), EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let pos = shapes
            .iter()
            .position(|s| s.id() == shape_id)
            .ok_or_else(|| EngineError::PartNotFound(format!("Shape {shape_id} not found")))?;

        let elem = shapes.remove(pos);
        shapes.push(elem);
        Ok(())
    }

    /// Moves a shape to the bottom of the rendering stack (rendered first, visually in back).
    pub fn send_to_back(slide: &mut Slide, shape_id: u32) -> Result<(), EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let pos = shapes
            .iter()
            .position(|s| s.id() == shape_id)
            .ok_or_else(|| EngineError::PartNotFound(format!("Shape {shape_id} not found")))?;

        let elem = shapes.remove(pos);
        shapes.insert(0, elem);
        Ok(())
    }

    /// Moves a shape one step forward in the Z-order hierarchy.
    pub fn bring_forward(slide: &mut Slide, shape_id: u32) -> Result<(), EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let pos = shapes
            .iter()
            .position(|s| s.id() == shape_id)
            .ok_or_else(|| EngineError::PartNotFound(format!("Shape {shape_id} not found")))?;

        if pos + 1 < shapes.len() {
            shapes.swap(pos, pos + 1);
        }
        Ok(())
    }

    /// Moves a shape one step backward in the Z-order hierarchy.
    pub fn send_backward(slide: &mut Slide, shape_id: u32) -> Result<(), EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let pos = shapes
            .iter()
            .position(|s| s.id() == shape_id)
            .ok_or_else(|| EngineError::PartNotFound(format!("Shape {shape_id} not found")))?;

        if pos > 0 {
            shapes.swap(pos, pos - 1);
        }
        Ok(())
    }

    /// Reorders the entire slide shape tree matching a specified list of shape IDs from back to front.
    pub fn reorder_shapes(slide: &mut Slide, ordered_ids: &[u32]) -> Result<(), EngineError> {
        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let mut reordered = Vec::with_capacity(shapes.len());

        for &id in ordered_ids {
            if let Some(pos) = shapes.iter().position(|s| s.id() == id) {
                reordered.push(shapes.remove(pos));
            }
        }

        // Retain any remaining shapes that were omitted from ordered_ids at the end
        reordered.append(shapes);
        *shapes = reordered;
        Ok(())
    }

    // Helper functions

    fn convert_element_to_dto(elem: &ShapeElement, z_index: usize) -> LayerItemDto {
        match elem {
            ShapeElement::Shape(sh) => {
                let (x, y, cx, cy) = sh
                    .properties
                    .transform
                    .as_ref()
                    .map(|t| (t.x, t.y, t.cx, t.cy))
                    .unwrap_or((0, 0, 0, 0));
                let kind = if sh.name.starts_with("SmartArt") {
                    "smartArt"
                } else if sh.is_text_box {
                    "text"
                } else {
                    "shape"
                };
                LayerItemDto {
                    id: sh.id,
                    name: sh.name.clone(),
                    kind: kind.to_string(),
                    z_index,
                    is_hidden: false,
                    is_locked: false,
                    x,
                    y,
                    cx,
                    cy,
                    children: Vec::new(),
                }
            }
            ShapeElement::Group(grp) => {
                let (x, y, cx, cy) = grp
                    .transform
                    .as_ref()
                    .map(|t| (t.x, t.y, t.cx, t.cy))
                    .unwrap_or((0, 0, 0, 0));
                let children = grp
                    .children
                    .iter()
                    .enumerate()
                    .map(|(c_idx, c)| Self::convert_element_to_dto(c, c_idx))
                    .collect();
                LayerItemDto {
                    id: grp.id,
                    name: grp.name.clone(),
                    kind: "group".to_string(),
                    z_index,
                    is_hidden: false,
                    is_locked: false,
                    x,
                    y,
                    cx,
                    cy,
                    children,
                }
            }
            ShapeElement::GraphicFrame(gf) => {
                let (x, y, cx, cy) = gf
                    .transform
                    .as_ref()
                    .map(|t| (t.x, t.y, t.cx, t.cy))
                    .unwrap_or((0, 0, 0, 0));
                LayerItemDto {
                    id: gf.id,
                    name: gf.name.clone(),
                    kind: "graphicFrame".to_string(),
                    z_index,
                    is_hidden: false,
                    is_locked: false,
                    x,
                    y,
                    cx,
                    cy,
                    children: Vec::new(),
                }
            }
        }
    }

    fn get_element_bounds(elem: &ShapeElement) -> (i64, i64, u32, u32) {
        match elem {
            ShapeElement::Shape(sh) => sh
                .properties
                .transform
                .as_ref()
                .map(|t| (t.x, t.y, t.cx, t.cy))
                .unwrap_or((0, 0, 0, 0)),
            ShapeElement::Group(grp) => grp
                .transform
                .as_ref()
                .map(|t| (t.x, t.y, t.cx, t.cy))
                .unwrap_or((0, 0, 0, 0)),
            ShapeElement::GraphicFrame(gf) => gf
                .transform
                .as_ref()
                .map(|t| (t.x, t.y, t.cx, t.cy))
                .unwrap_or((0, 0, 0, 0)),
        }
    }

    fn offset_element_transform(elem: &mut ShapeElement, dx: i64, dy: i64) {
        match elem {
            ShapeElement::Shape(sh) => {
                if let Some(ref mut xf) = sh.properties.transform {
                    xf.x += dx;
                    xf.y += dy;
                }
            }
            ShapeElement::Group(grp) => {
                if let Some(ref mut xf) = grp.transform {
                    xf.x += dx;
                    xf.y += dy;
                }
            }
            ShapeElement::GraphicFrame(gf) => {
                if let Some(ref mut xf) = gf.transform {
                    xf.x += dx;
                    xf.y += dy;
                }
            }
        }
    }

    fn compute_max_id_recursive(shapes: &[ShapeElement], max_id: &mut u32) {
        for s in shapes {
            *max_id = (*max_id).max(s.id());
            if let ShapeElement::Group(grp) = s {
                Self::compute_max_id_recursive(&grp.children, max_id);
            }
        }
    }
}
