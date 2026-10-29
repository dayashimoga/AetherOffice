#![forbid(unsafe_code)]
//! DrawingML Geometry, Guide Evaluator, and Path Engine (ECMA-376 Part 1, Section 20.1.9)
//!
//! Provides full support for:
//! - All 17 DrawingML guide formula operations (`val`, `+-`, `*/`, `+/`, `?:`, `abs`, `at2`, `cat2`, `sat2`, `cos`, `sin`, `tan`, `sqrt`, `min`, `max`, `pin`, `mod`)
//! - All 187 standard ECMA-376 `a:prstGeom` preset shapes
//! - Custom geometries (`a:custGeom`) with `a:avLst`, `a:gdLst`, and `a:pathLst`
//! - Normalized vector path commands (`MoveTo`, `LineTo`, `QuadTo`, `CubicTo`, `ArcTo`, `Close`)
//! - Elliptical arc to cubic Bezier conversion and SVG path data export

pub mod compiler;
pub mod guide;
pub mod path;
pub mod preset;

pub use compiler::{CustomGeometry, CustomPath, GeometryCompiler, PathInstruction};
pub use guide::{GeomGuide, GuideContext};
pub use path::{DrawingPath, PathCommand, PathFillMode, Point2D};
pub use preset::{get_preset_geometry, list_preset_names, PresetDefinition};
