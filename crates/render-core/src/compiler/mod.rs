//! Unified Display List Compilers for Word, Excel, PowerPoint, and PDF

pub mod excel;
pub mod pdf;
pub mod pdf_editor;
pub mod ppt;
pub mod word;

pub use excel::{
    cell_coord_to_indices, col_to_name, indices_to_cell_coord, name_to_col, AutoFillEngine,
    ExcelDisplayCell, ExcelDisplayCompiler, ExcelSelectionRange, FreezePanesConfig,
    GridDimensionMap, GridViewportConfig, HeaderRenderConfig, MultiRangeSelectionModel,
    COL_HEADER_HEIGHT, DEFAULT_COL_WIDTH, DEFAULT_ROW_HEIGHT, MAX_EXCEL_COLS, MAX_EXCEL_ROWS,
    ROW_HEADER_WIDTH,
};
pub use pdf::{
    PdfCoordinateTransformer, PdfDisplayCompiler, PdfDocumentLayout, PdfHitResult, PdfHitTarget,
    PdfHitTester, PdfPageLayout, PdfPageLayoutMode, PdfSelectionHelper, PdfTextPoint,
    PdfTextSelection, PdfTile, PdfTileVirtualizer, PdfViewportConfig, DEFAULT_PAGE_GAP,
    DEFAULT_TILE_SIZE, MAX_PDF_ZOOM, MIN_PDF_ZOOM, ZOOM_PRESETS,
};
pub use pdf_editor::{
    PdfActiveTextState, PdfEditTextCommand, PdfEditorDisplayRenderer, PdfFormFiller, PdfGizmoHit,
    PdfMoveObjectCommand, PdfObjectEditor, PdfObjectGizmo, PdfObjectType, PdfPageDescriptor,
    PdfPageObject, PdfPageOrganiser, PdfRotatePageCommand, PdfSetFormFieldCommand, PdfTextEditor,
    PdfTextStyle, PdfThumbnailLayout,
};
pub use ppt::{
    generate_preset_path, MasterTheme, PptDisplayCompiler, PptGizmoHit, PptGizmoModel,
    PptStageController, PptViewportTransform, PresetShapeType, SlideAddShapeCommand,
    SlideAspectRatio, SlideCanvasConfig, SlideDeleteShapeCommand, SlideMetadata, SlideShapeModel,
    SlideShowAction, SlideShowStateMachine, SlideSorterCommand, SlideSorterModel, SlideStageModel,
    SlideTransformCommand, SlideTransformEngine, SwitchThemeCommand, ThemePreset, ZOrderAction,
};
pub use word::{
    CaretNavDirection, PageLayoutMode, PageMargins, PageSize, ParagraphAlignment,
    SubpixelTextLayout, UnderlineStyle, WordCaretModel, WordCaretPosition, WordDisplayCompiler,
    WordDisplayLine, WordDisplayPage, WordDisplayTableCell, WordHitResult, WordHitTester,
    WordNavigationEngine, WordSelectionRange, WordViewportConfig,
};
