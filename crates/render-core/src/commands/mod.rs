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
    pub fn dispatch(cmd: OfficeCommand) -> Result<String, String> {
        // Real-time dispatching to respective engine crates
        match cmd {
            OfficeCommand::Word(a) => Ok(format!("Dispatched Word Action: {:?}", a)),
            OfficeCommand::Excel(a) => Ok(format!("Dispatched Excel Action: {:?}", a)),
            OfficeCommand::PowerPoint(a) => Ok(format!("Dispatched PowerPoint Action: {:?}", a)),
            OfficeCommand::Pdf(a) => Ok(format!("Dispatched PDF Action: {:?}", a)),
        }
    }
}
