#![forbid(unsafe_code)]
//! PresentationML SmartArt Builder & Visual Diagram Engine.
//!
//! Complies with ECMA-376 Part 1 §21.4 (DrawingML Diagrams & SmartArt).
//! Compiles logical item trees into styled shape clusters, connectors, and composite group shapes.

use ooxml_opc::error::EngineError;
use serde::{Deserialize, Serialize};

use crate::model::*;

/// Single node definition for SmartArt generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartArtItemDto {
    pub title: String,
    pub description: Option<String>,
    pub level: usize,
    pub color_hex: Option<String>,
}

/// Request DTO for generating a SmartArt diagram on a slide.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartArtBuildRequest {
    pub layout_type: String, // "process", "hierarchy", "cycle", "matrix", "pyramid"
    pub items: Vec<SmartArtItemDto>,
    pub x: i64,
    pub y: i64,
    pub cx: u32,
    pub cy: u32,
    pub palette: Option<Vec<String>>,
}

/// SmartArt generation and diagram synthesis engine.
pub struct SmartArtBuilder;

impl SmartArtBuilder {
    const DEFAULT_PALETTE: &'static [&'static str] = &[
        "#2B579A", // Office Blue
        "#107C41", // Excel Green
        "#D83B01", // Orange / Coral
        "#008272", // Teal
        "#5C2D91", // Purple
        "#0078D7", // Light Blue
    ];

    /// Builds a SmartArt diagram and adds it as a named `GroupShape` on the target slide.
    pub fn build_and_insert(
        slide: &mut Slide,
        request: SmartArtBuildRequest,
    ) -> Result<u32, EngineError> {
        if request.items.is_empty() {
            return Err(EngineError::InvalidOperation(
                "SmartArt generation requires at least one item".to_string(),
            ));
        }

        let shapes = &mut slide.common_slide_data.shape_tree.shapes;
        let mut max_id = 0u32;
        for s in shapes.iter() {
            max_id = max_id.max(s.id());
        }
        let group_id = max_id + 1;
        let start_child_id = group_id + 1;

        let palette: Vec<String> = request.palette.unwrap_or_else(|| {
            Self::DEFAULT_PALETTE
                .iter()
                .map(|&s| s.to_string())
                .collect()
        });

        let (children, group_name) = match request.layout_type.as_str() {
            "hierarchy" => (
                Self::build_hierarchy_layout(
                    &request.items,
                    start_child_id,
                    request.cx,
                    request.cy,
                    &palette,
                ),
                "SmartArt: Hierarchy",
            ),
            "cycle" => (
                Self::build_cycle_layout(
                    &request.items,
                    start_child_id,
                    request.cx,
                    request.cy,
                    &palette,
                ),
                "SmartArt: Cycle",
            ),
            "matrix" => (
                Self::build_matrix_layout(
                    &request.items,
                    start_child_id,
                    request.cx,
                    request.cy,
                    &palette,
                ),
                "SmartArt: Matrix",
            ),
            "pyramid" => (
                Self::build_pyramid_layout(
                    &request.items,
                    start_child_id,
                    request.cx,
                    request.cy,
                    &palette,
                ),
                "SmartArt: Pyramid",
            ),
            _ => (
                Self::build_process_layout(
                    &request.items,
                    start_child_id,
                    request.cx,
                    request.cy,
                    &palette,
                ),
                "SmartArt: Process",
            ),
        };

        let group_shape = GroupShape {
            id: group_id,
            name: group_name.to_string(),
            transform: Some(GroupTransform2D {
                x: request.x,
                y: request.y,
                cx: request.cx,
                cy: request.cy,
                ch_x: 0,
                ch_y: 0,
                ch_cx: request.cx,
                ch_cy: request.cy,
            }),
            children,
        };

        shapes.push(ShapeElement::Group(group_shape));
        Ok(group_id)
    }

    // Layout Implementations

    fn build_process_layout(
        items: &[SmartArtItemDto],
        mut current_id: u32,
        total_w: u32,
        total_h: u32,
        palette: &[String],
    ) -> Vec<ShapeElement> {
        let n = items.len();
        let mut elements = Vec::new();

        let arrow_w = (total_w as f64 * 0.05).clamp(200_000.0, 500_000.0) as u32;
        let spacing = (total_w as f64 * 0.02) as u32;
        let total_arrows = (n - 1) as u32;
        let available_w = total_w.saturating_sub(total_arrows * (arrow_w + spacing * 2));
        let card_w = (available_w / n as u32).max(400_000);
        let card_h = (total_h as f64 * 0.6).round() as u32;
        let card_y = (total_h - card_h) / 2;

        let mut cur_x = 0i64;

        for (i, item) in items.iter().enumerate() {
            let color = item
                .color_hex
                .clone()
                .unwrap_or_else(|| palette[i % palette.len()].clone());

            // Node Shape
            elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
                current_id,
                &format!("SmartArt_Node_{}", i + 1),
                (&item.title, item.description.as_deref()),
                (cur_x, card_y as i64, card_w, card_h),
                &color,
                "roundRect",
            ))));
            current_id += 1;
            cur_x += card_w as i64;

            // Connecting Arrow between cards
            if i + 1 < n {
                cur_x += spacing as i64;
                let arrow_y = card_y + (card_h / 2) - 150_000;
                elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_arrow(
                    current_id,
                    &format!("SmartArt_Arrow_{}", i + 1),
                    cur_x,
                    arrow_y as i64,
                    arrow_w,
                    300_000,
                    "#A0A0A0",
                ))));
                current_id += 1;
                cur_x += (arrow_w + spacing) as i64;
            }
        }

        elements
    }

    fn build_hierarchy_layout(
        items: &[SmartArtItemDto],
        mut current_id: u32,
        total_w: u32,
        total_h: u32,
        palette: &[String],
    ) -> Vec<ShapeElement> {
        let mut elements = Vec::new();
        let root = &items[0];
        let children = &items[1..];

        let root_w = (total_w as f64 * 0.4).round() as u32;
        let root_h = (total_h as f64 * 0.25).round() as u32;
        let root_x = ((total_w - root_w) / 2) as i64;
        let root_y = (total_h as f64 * 0.05).round() as i64;

        // Root shape
        elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
            current_id,
            "SmartArt_Root",
            (&root.title, root.description.as_deref()),
            (root_x, root_y, root_w, root_h),
            &palette[0],
            "roundRect",
        ))));
        current_id += 1;

        if children.is_empty() {
            return elements;
        }

        // Subordinate row
        let child_count = children.len() as u32;
        let child_spacing = (total_w as f64 * 0.03) as u32;
        let total_spacing = (child_count - 1) * child_spacing;
        let child_w = (total_w.saturating_sub(total_spacing) / child_count).max(300_000);
        let child_h = (total_h as f64 * 0.3).round() as u32;
        let child_y = (total_h as f64 * 0.55).round() as i64;

        let mut child_x = 0i64;
        for (i, child) in children.iter().enumerate() {
            let color = child
                .color_hex
                .clone()
                .unwrap_or_else(|| palette[(i + 1) % palette.len()].clone());

            elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
                current_id,
                &format!("SmartArt_Child_{}", i + 1),
                (&child.title, child.description.as_deref()),
                (child_x, child_y, child_w, child_h),
                &color,
                "roundRect",
            ))));
            current_id += 1;
            child_x += (child_w + child_spacing) as i64;
        }

        elements
    }

    fn build_cycle_layout(
        items: &[SmartArtItemDto],
        mut current_id: u32,
        total_w: u32,
        total_h: u32,
        palette: &[String],
    ) -> Vec<ShapeElement> {
        let n = items.len();
        let mut elements = Vec::new();

        let center_x = total_w as f64 / 2.0;
        let center_y = total_h as f64 / 2.0;
        let radius_x = center_x * 0.65;
        let radius_y = center_y * 0.65;
        let node_w = (total_w as f64 * 0.22).round() as u32;
        let node_h = (total_h as f64 * 0.22).round() as u32;

        for (i, item) in items.iter().enumerate() {
            let angle =
                (i as f64 / n as f64) * 2.0 * std::f64::consts::PI - (std::f64::consts::PI / 2.0);
            let nx = center_x + radius_x * angle.cos() - (node_w as f64 / 2.0);
            let ny = center_y + radius_y * angle.sin() - (node_h as f64 / 2.0);

            let color = item
                .color_hex
                .clone()
                .unwrap_or_else(|| palette[i % palette.len()].clone());

            elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
                current_id,
                &format!("SmartArt_Cycle_{}", i + 1),
                (&item.title, item.description.as_deref()),
                (nx.round() as i64, ny.round() as i64, node_w, node_h),
                &color,
                "ellipse",
            ))));
            current_id += 1;
        }

        elements
    }

    fn build_matrix_layout(
        items: &[SmartArtItemDto],
        mut current_id: u32,
        total_w: u32,
        total_h: u32,
        palette: &[String],
    ) -> Vec<ShapeElement> {
        let mut elements = Vec::new();
        let gap = (total_w as f64 * 0.04) as u32;
        let cell_w = (total_w - gap) / 2;
        let cell_h = (total_h - gap) / 2;

        let positions = [
            (0i64, 0i64),                                   // Top-Left
            ((cell_w + gap) as i64, 0i64),                  // Top-Right
            (0i64, (cell_h + gap) as i64),                  // Bottom-Left
            ((cell_w + gap) as i64, (cell_h + gap) as i64), // Bottom-Right
        ];

        for (i, item) in items.iter().take(4).enumerate() {
            let (x, y) = positions[i];
            let color = item
                .color_hex
                .clone()
                .unwrap_or_else(|| palette[i % palette.len()].clone());

            elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
                current_id,
                &format!("SmartArt_Matrix_{}", i + 1),
                (&item.title, item.description.as_deref()),
                (x, y, cell_w, cell_h),
                &color,
                "rect",
            ))));
            current_id += 1;
        }

        elements
    }

    fn build_pyramid_layout(
        items: &[SmartArtItemDto],
        mut current_id: u32,
        total_w: u32,
        total_h: u32,
        palette: &[String],
    ) -> Vec<ShapeElement> {
        let n = items.len();
        let mut elements = Vec::new();
        let tier_h = total_h / n as u32;

        for (i, item) in items.iter().enumerate() {
            // Apex (i = 0) is narrowest, base (i = n - 1) is widest
            let fraction = (i + 1) as f64 / n as f64;
            let tier_w = (total_w as f64 * (0.3 + 0.7 * fraction)).round() as u32;
            let tier_x = ((total_w - tier_w) / 2) as i64;
            let tier_y = (i as u32 * tier_h) as i64;

            let color = item
                .color_hex
                .clone()
                .unwrap_or_else(|| palette[i % palette.len()].clone());

            elements.push(ShapeElement::Shape(Box::new(Self::create_diagram_card(
                current_id,
                &format!("SmartArt_Pyramid_{}", i + 1),
                (&item.title, item.description.as_deref()),
                (tier_x, tier_y, tier_w, (tier_h as f64 * 0.9).round() as u32),
                &color,
                "trapezoid",
            ))));
            current_id += 1;
        }

        elements
    }

    // Shape Creation Primitives

    fn create_diagram_card(
        id: u32,
        name: &str,
        text: (&str, Option<&str>),
        bounds: (i64, i64, u32, u32),
        color_hex: &str,
        preset_geom: &str,
    ) -> Shape {
        let (title, desc) = text;
        let (x, y, cx, cy) = bounds;
        let clean_hex = color_hex.trim_start_matches('#');
        let mut runs = vec![TextRunElement::Text(title.to_string(), None)];
        if let Some(d) = desc {
            runs.push(TextRunElement::LineBreak(None));
            runs.push(TextRunElement::Text(d.to_string(), None));
        }

        Shape {
            id,
            name: name.to_string(),
            is_text_box: false,
            placeholder: None,
            properties: ShapeProperties {
                transform: Some(Transform2D::new(x, y, cx, cy)),
                geometry: Some(Geometry::Preset(preset_geom.to_string())),
                fill: Some(Fill::Solid(ColorChoice::Srgb(clean_hex.to_string()))),
                line: Some(LineProperties {
                    width: Some(12_700),
                    fill: Some(Fill::Solid(ColorChoice::Srgb("FFFFFF".to_string()))),
                    dash_style: None,
                }),
            },
            text_body: Some(TextBody {
                body_properties: BodyProperties::default(),
                list_style: None,
                paragraphs: vec![TextParagraph {
                    properties: None,
                    runs,
                    end_paragraph_run_properties: None,
                }],
            }),
            raw_elements: Vec::new(),
        }
    }

    fn create_diagram_arrow(
        id: u32,
        name: &str,
        x: i64,
        y: i64,
        cx: u32,
        cy: u32,
        color_hex: &str,
    ) -> Shape {
        let clean_hex = color_hex.trim_start_matches('#');
        Shape {
            id,
            name: name.to_string(),
            is_text_box: false,
            placeholder: None,
            properties: ShapeProperties {
                transform: Some(Transform2D::new(x, y, cx, cy)),
                geometry: Some(Geometry::Preset("rightArrow".to_string())),
                fill: Some(Fill::Solid(ColorChoice::Srgb(clean_hex.to_string()))),
                line: None,
            },
            text_body: None,
            raw_elements: Vec::new(),
        }
    }
}
