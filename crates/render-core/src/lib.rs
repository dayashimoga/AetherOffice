#![forbid(unsafe_code)]
//! render-core - Unified 120fps WebGPU / Cosmic-Text Render Core
//!
//! Provides:
//! - GPU draw primitives (`QuadInstance`, `GlyphRun`, `BezierPathBatch`)
//! - Viewport culling and uniform 2D spatial grid indexing (`SpatialGrid`)
//! - Subpixel text shaping and BiDi / RTL / CJK handling (`ShapedTextEngine`)
//! - Retained display list compiler for Word, Excel, and PowerPoint
//! - Headless SVG exporter, software pixel rasterizer, and PDF vector fallback
//! - 120Hz frame pacing compositor and latency profiler

pub mod adaptive;
pub mod commands;
pub mod compiler;
pub mod compositor;
pub mod interaction;
pub mod primitive;
pub mod rasterizer;
pub mod shell;
pub mod spatial;
pub mod text;

pub use adaptive::{
    BottomSheetState, BottomSheetStateMachine, CardField, CardFieldType, FormFactor, InkPoint,
    InkStroke, LaserPointerEngine, RowCardInspectorEngine, SafeAreaInsets, TouchRibbonState,
    ViewportMetrics, WordReflowReaderEngine,
};
pub use commands::*;

pub use compiler::{
    excel::{
        cell_coord_to_indices, col_to_name, indices_to_cell_coord, name_to_col, AutoFillEngine,
        ExcelDisplayCell, ExcelDisplayCompiler, ExcelSelectionRange, FreezePanesConfig,
        GridDimensionMap, GridViewportConfig, HeaderRenderConfig, MultiRangeSelectionModel,
        COL_HEADER_HEIGHT, DEFAULT_COL_WIDTH, DEFAULT_ROW_HEIGHT, MAX_EXCEL_COLS, MAX_EXCEL_ROWS,
        ROW_HEADER_WIDTH,
    },
    ppt::{
        generate_preset_path, MasterTheme, PptDisplayCompiler, PptGizmoHit, PptGizmoModel,
        PptStageController, PptViewportTransform, PresetShapeType, SlideAddShapeCommand,
        SlideAspectRatio, SlideCanvasConfig, SlideDeleteShapeCommand, SlideMetadata,
        SlideShapeModel, SlideShowAction, SlideShowStateMachine, SlideSorterCommand,
        SlideSorterModel, SlideStageModel, SlideTransformCommand, SlideTransformEngine,
        SwitchThemeCommand, ThemePreset, ZOrderAction,
    },
    word::{
        CaretNavDirection, PageLayoutMode, PageMargins, PageSize, ParagraphAlignment,
        SubpixelTextLayout, UnderlineStyle, WordCaretModel, WordCaretPosition, WordDisplayCompiler,
        WordDisplayLine, WordDisplayPage, WordDisplayTableCell, WordHitResult, WordHitTester,
        WordNavigationEngine, WordSelectionRange, WordViewportConfig,
    },
};
pub use compositor::{Compositor, FrameStats, ViewportConfig};
pub use interaction::{
    ClipboardManager, ClipboardPayload, Command, CommandBus, CompoundCommand,
    CoordinateTransformer, DocumentState, ExcelHitTarget, ExcelRange, ExcelSelection, HitTester,
    InertiaPhysics, InsertTextCommand, PptHitTarget, PptSelection, ResizeHandle, WordHitTarget,
    WordSelection,
};
pub use primitive::{
    BezierPathBatch, ColorRgba, DisplayCommand, DisplayList, GlyphInstance, GlyphRun, QuadInstance,
    Rect2D, TextScript, TransformMatrix,
};
pub use rasterizer::SoftwareRasterizer;
pub use shell::{
    AltKeytipStateMachine, DesktopSessionManager, DocumentTab, DocumentType, FormatCommand,
    FormattingState, FormulaBarEngine, FormulaToken, FormulaTokenType, HeadingNode, KeyCombination,
    NavigationPaneEngine, PageOrientation, PageSetupModel, RibbonAction, RibbonModel, RibbonTabId,
    SessionSummary, ShellAction, ShortcutEngine, SlideSorterEngine, TabBarModel,
    WordRibbonController, WordRulerConfig,
};
pub use spatial::SpatialGrid;
pub use text::ShapedTextEngine;

pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
