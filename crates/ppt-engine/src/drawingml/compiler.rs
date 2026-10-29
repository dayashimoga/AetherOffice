#![forbid(unsafe_code)]
//! DrawingML Geometry Compiler
//!
//! Evaluates and compiles preset shapes (`a:prstGeom`) and custom geometries (`a:custGeom`)
//! into fully resolved vector paths with normalized Bezier and arc coordinates.

use ooxml_opc::error::EngineError;
use serde::{Deserialize, Serialize};

use super::guide::{GeomGuide, GuideContext};
use super::path::{DrawingPath, PathCommand, PathFillMode};
use super::preset::get_preset_geometry;

/// Custom geometry representation: `<a:custGeom>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomGeometry {
    pub adjust_values: Vec<GeomGuide>,
    pub guides: Vec<GeomGuide>,
    pub paths: Vec<CustomPath>,
}

/// A path definition in `<a:pathLst>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomPath {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub fill: PathFillMode,
    pub stroke: bool,
    pub instructions: Vec<PathInstruction>,
}

impl Default for CustomPath {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            fill: PathFillMode::Norm,
            stroke: true,
            instructions: Vec::new(),
        }
    }
}

/// A drawing command within a custom path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathInstruction {
    MoveTo {
        x: String,
        y: String,
    },
    LineTo {
        x: String,
        y: String,
    },
    ArcTo {
        w_r: String,
        h_r: String,
        start_angle: String,
        swing_angle: String,
    },
    QuadBezTo {
        ctrl_x: String,
        ctrl_y: String,
        x: String,
        y: String,
    },
    CubicBezTo {
        ctrl1_x: String,
        ctrl1_y: String,
        ctrl2_x: String,
        ctrl2_y: String,
        x: String,
        y: String,
    },
    Close,
}

/// Geometry Compiler for compiling preset and custom DrawingML shapes into paths.
pub struct GeometryCompiler;

impl GeometryCompiler {
    /// Compile a standard preset geometry (e.g., `"roundRect"`, `"star5"`) into paths.
    pub fn compile_preset(
        preset_name: &str,
        x: i64,
        y: i64,
        w: i64,
        h: i64,
        custom_adjusts: &[GeomGuide],
    ) -> Result<Vec<DrawingPath>, EngineError> {
        let preset_def = get_preset_geometry(preset_name).ok_or_else(|| {
            EngineError::UnsupportedFeature(format!(
                "Unknown DrawingML preset geometry: '{preset_name}'"
            ))
        })?;

        let mut ctx = GuideContext::new(w, h);

        // Load default adjusts
        for &(adj_name, default_val) in preset_def.default_adjusts {
            ctx.set(adj_name, default_val);
        }

        // Apply custom adjust values
        for adj in custom_adjusts {
            let val = ctx.eval_formula(&adj.formula)?;
            ctx.set(&adj.name, val);
        }

        // Evaluate preset guides if any
        for &(gd_name, formula) in preset_def.guides {
            let val = ctx.eval_formula(formula)?;
            ctx.set(gd_name, val);
        }

        let mut paths = (preset_def.generate)(&ctx);

        // Offset paths by (x, y) if needed
        if x != 0 || y != 0 {
            for p in &mut paths {
                offset_path(p, x as f64, y as f64);
            }
        }

        Ok(paths)
    }

    /// Compile a custom geometry (`<a:custGeom>`) into paths.
    pub fn compile_custom(
        geom: &CustomGeometry,
        x: i64,
        y: i64,
        w: i64,
        h: i64,
    ) -> Result<Vec<DrawingPath>, EngineError> {
        let mut ctx = GuideContext::new(w, h);

        // Evaluate custom adjust values
        ctx.evaluate_guides(&geom.adjust_values)?;

        // Evaluate guides
        ctx.evaluate_guides(&geom.guides)?;

        let mut result_paths = Vec::new();

        for cpath in &geom.paths {
            let path_w = cpath.width.unwrap_or(w);
            let path_h = cpath.height.unwrap_or(h);

            let scale_x = if path_w > 0 {
                w as f64 / path_w as f64
            } else {
                1.0
            };
            let scale_y = if path_h > 0 {
                h as f64 / path_h as f64
            } else {
                1.0
            };

            let mut path = DrawingPath::new();
            path.fill = cpath.fill;
            path.stroke = cpath.stroke;
            path.width = Some(w);
            path.height = Some(h);

            for inst in &cpath.instructions {
                match inst {
                    PathInstruction::MoveTo { x: px, y: py } => {
                        let rx = ctx.resolve_operand(px)? as f64 * scale_x + x as f64;
                        let ry = ctx.resolve_operand(py)? as f64 * scale_y + y as f64;
                        path.move_to(rx, ry);
                    }
                    PathInstruction::LineTo { x: px, y: py } => {
                        let rx = ctx.resolve_operand(px)? as f64 * scale_x + x as f64;
                        let ry = ctx.resolve_operand(py)? as f64 * scale_y + y as f64;
                        path.line_to(rx, ry);
                    }
                    PathInstruction::ArcTo {
                        w_r,
                        h_r,
                        start_angle,
                        swing_angle,
                    } => {
                        let wr = ctx.resolve_operand(w_r)? as f64 * scale_x;
                        let hr = ctx.resolve_operand(h_r)? as f64 * scale_y;
                        let st = ctx.resolve_operand(start_angle)? as f64 / 60_000.0;
                        let sw = ctx.resolve_operand(swing_angle)? as f64 / 60_000.0;
                        path.arc_to(wr, hr, st, sw);
                    }
                    PathInstruction::QuadBezTo {
                        ctrl_x,
                        ctrl_y,
                        x: px,
                        y: py,
                    } => {
                        let cx = ctx.resolve_operand(ctrl_x)? as f64 * scale_x + x as f64;
                        let cy = ctx.resolve_operand(ctrl_y)? as f64 * scale_y + y as f64;
                        let rx = ctx.resolve_operand(px)? as f64 * scale_x + x as f64;
                        let ry = ctx.resolve_operand(py)? as f64 * scale_y + y as f64;
                        path.quad_to(cx, cy, rx, ry);
                    }
                    PathInstruction::CubicBezTo {
                        ctrl1_x,
                        ctrl1_y,
                        ctrl2_x,
                        ctrl2_y,
                        x: px,
                        y: py,
                    } => {
                        let c1x = ctx.resolve_operand(ctrl1_x)? as f64 * scale_x + x as f64;
                        let c1y = ctx.resolve_operand(ctrl1_y)? as f64 * scale_y + y as f64;
                        let c2x = ctx.resolve_operand(ctrl2_x)? as f64 * scale_x + x as f64;
                        let c2y = ctx.resolve_operand(ctrl2_y)? as f64 * scale_y + y as f64;
                        let rx = ctx.resolve_operand(px)? as f64 * scale_x + x as f64;
                        let ry = ctx.resolve_operand(py)? as f64 * scale_y + y as f64;
                        path.cubic_to(c1x, c1y, c2x, c2y, rx, ry);
                    }
                    PathInstruction::Close => {
                        path.close();
                    }
                }
            }

            result_paths.push(path);
        }

        Ok(result_paths)
    }
}

fn offset_path(path: &mut DrawingPath, dx: f64, dy: f64) {
    for cmd in &mut path.commands {
        match cmd {
            PathCommand::MoveTo(pt) | PathCommand::LineTo(pt) => {
                pt.x += dx;
                pt.y += dy;
            }
            PathCommand::QuadTo { ctrl, to } => {
                ctrl.x += dx;
                ctrl.y += dy;
                to.x += dx;
                to.y += dy;
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                ctrl1.x += dx;
                ctrl1.y += dy;
                ctrl2.x += dx;
                ctrl2.y += dy;
                to.x += dx;
                to.y += dy;
            }
            _ => {}
        }
    }
}
