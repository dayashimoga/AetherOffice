//! Transactional Command Bus, Global Undo/Redo & Universal Clipboard
//!
//! Complies with ECMA-376, ISO 32000-1, W3C Clipboard API, and GEC-v1.0.

pub mod bus;
pub mod clipboard;
pub mod transactions;

pub use bus::GlobalCommandBus;
pub use clipboard::{
    grid_to_tsv, html_table_to_tsv, tsv_to_grid, tsv_to_html_table, tsv_to_word_table_xml,
    word_table_xml_to_tsv, GlobalClipboard, GlobalClipboardPayload,
};
pub use transactions::{
    CompoundOfficeTransaction, ExcelAddSheetTransaction, ExcelPasteRangeTransaction,
    ExcelSetCellTransaction, GlobalOfficeState, OfficeTransaction, PdfEditTextTransaction,
    PdfMoveObjectTransaction, PdfRotatePageTransaction, PdfSetFormFieldTransaction,
    PptAddShapeTransaction, PptDeleteShapeTransaction, PptMoveShapeTransaction, PptShapeData,
    WordDeleteTextTransaction, WordFormatRangeTransaction, WordInsertTextTransaction,
    WordRangeStyle,
};

use crate::primitive::Rect2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "app", content = "payload")]
pub enum OfficeCommand {
    Word(WordAction),
    Excel(ExcelAction),
    PowerPoint(PptAction),
    Pdf(PdfAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WordAction {
    InsertText {
        text: String,
    },
    ToggleStyle {
        bold: Option<bool>,
        italic: Option<bool>,
        underline: Option<bool>,
    },
    SetFont {
        family: String,
        size_pt: f32,
    },
    SetAlignment {
        align: String,
    },
    InsertTable {
        rows: usize,
        cols: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExcelAction {
    SetCellFormula {
        coord: String,
        formula: String,
    },
    SetRangeStyle {
        range: String,
        bold: Option<bool>,
        num_fmt: Option<String>,
    },
    Recalculate,
    AddSheet {
        title: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PptAction {
    NewSlide {
        layout: String,
    },
    InsertShape {
        shape_type: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    UpdateText {
        shape_id: String,
        text: String,
    },
    SetTransition {
        name: String,
        duration_ms: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PdfAction {
    SetFormField {
        field_name: String,
        value: String,
    },
    AddAnnotation {
        page: usize,
        annot_type: String,
        rect: [f32; 4],
    },
    RotatePage {
        page: usize,
        angle_deg: i32,
    },
    SignDocument {
        signature_png_bytes: Vec<u8>,
    },
}

pub struct CommandDispatcher;

impl CommandDispatcher {
    /// String-formatted immediate dispatch summary (backward-compatible).
    pub fn dispatch(cmd: OfficeCommand) -> Result<String, String> {
        match cmd {
            OfficeCommand::Word(a) => Ok(format!("Dispatched Word Action: {:?}", a)),
            OfficeCommand::Excel(a) => Ok(format!("Dispatched Excel Action: {:?}", a)),
            OfficeCommand::PowerPoint(a) => Ok(format!("Dispatched PowerPoint Action: {:?}", a)),
            OfficeCommand::Pdf(a) => Ok(format!("Dispatched PDF Action: {:?}", a)),
        }
    }

    /// Dispatch an OfficeCommand to the GlobalCommandBus and execute against GlobalOfficeState.
    pub fn dispatch_to_bus(
        bus: &mut GlobalCommandBus,
        state: &mut GlobalOfficeState,
        cmd: OfficeCommand,
    ) -> Result<(), String> {
        match cmd {
            OfficeCommand::Word(WordAction::InsertText { text }) => {
                let offset = state.word_content.len();
                let tx = Box::new(WordInsertTextTransaction::new(offset, text));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Word(WordAction::SetFont { family, size_pt }) => {
                let len = state.word_content.len();
                let style = WordRangeStyle {
                    start: 0,
                    end: len,
                    font_name: Some(family),
                    font_size: Some(size_pt),
                    bold: None,
                    italic: None,
                    underline: None,
                    color_hex: None,
                };
                let tx = Box::new(WordFormatRangeTransaction::new(style));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Word(WordAction::ToggleStyle {
                bold,
                italic,
                underline,
            }) => {
                let len = state.word_content.len();
                let style = WordRangeStyle {
                    start: 0,
                    end: len,
                    font_name: None,
                    font_size: None,
                    bold,
                    italic,
                    underline,
                    color_hex: None,
                };
                let tx = Box::new(WordFormatRangeTransaction::new(style));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Excel(ExcelAction::SetCellFormula { coord, formula }) => {
                let tx = Box::new(ExcelSetCellTransaction::new(coord, formula));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Excel(ExcelAction::AddSheet { title }) => {
                let tx = Box::new(ExcelAddSheetTransaction::new(title));
                bus.execute(state, tx)?;
            }
            OfficeCommand::PowerPoint(PptAction::InsertShape {
                shape_type,
                x,
                y,
                w,
                h,
            }) => {
                let id = (state.ppt_shapes.len() as u32) + 1;
                let shape = PptShapeData {
                    id,
                    shape_type,
                    rect: Rect2D::new(x as f64, y as f64, w as f64, h as f64),
                    text: String::new(),
                };
                let tx = Box::new(PptAddShapeTransaction::new(shape));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Pdf(PdfAction::SetFormField { field_name, value }) => {
                let old_val = state
                    .pdf_fields
                    .get(&field_name)
                    .cloned()
                    .unwrap_or_default();
                let tx = Box::new(PdfSetFormFieldTransaction::new(field_name, old_val, value));
                bus.execute(state, tx)?;
            }
            OfficeCommand::Pdf(PdfAction::RotatePage { page, angle_deg }) => {
                let tx = Box::new(PdfRotatePageTransaction::new(page, angle_deg));
                bus.execute(state, tx)?;
            }
            _ => {
                // Non-state-mutating commands (e.g. Recalculate, NewSlide, SetTransition)
            }
        }
        Ok(())
    }
}
