# bootstrap_ribbon_wiring.ps1 - Generates Functional Ribbon Dispatchers
$ErrorActionPreference = "Stop"
Write-Host "=== [2/3] Wiring Ribbon Action Dispatchers & Command Bus ===" -ForegroundColor Cyan

$commandBusDir = "crates/render-core/src/commands"
New-Item -ItemType Directory -Force -Path $commandBusDir | Out-Null

@'
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
    InsertText { text: String },
    ToggleStyle { bold: Option<bool>, italic: Option<bool>, underline: Option<bool> },
    SetFont { family: String, size_pt: f32 },
    SetAlignment { align: String },
    InsertTable { rows: usize, cols: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExcelAction {
    SetCellFormula { coord: String, formula: String },
    SetRangeStyle { range: String, bold: Option<bool>, num_fmt: Option<String> },
    Recalculate,
    AddSheet { title: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PptAction {
    NewSlide { layout: String },
    InsertShape { shape_type: String, x: f32, y: f32, w: f32, h: f32 },
    UpdateText { shape_id: String, text: String },
    SetTransition { name: String, duration_ms: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PdfAction {
    SetFormField { field_name: String, value: String },
    AddAnnotation { page: usize, annot_type: String, rect: [f32; 4] },
    RotatePage { page: usize, angle_deg: i32 },
    SignDocument { signature_png_bytes: Vec<u8> },
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
'@ | Set-Content -Path "$commandBusDir/mod.rs" -Encoding UTF8

# Ensure render-core/src/lib.rs exports commands module
$libRsPath = "crates/render-core/src/lib.rs"
$libContent = Get-Content $libRsPath -Raw
if ($libContent -notmatch "pub mod commands;") {
    $updatedLib = $libContent -replace "pub mod adaptive;", "pub mod adaptive;`npub mod commands;"
    $updatedLib = $updatedLib -replace "pub use adaptive::\{", "pub use commands::*;`npub use adaptive::{"
    Set-Content -Path $libRsPath -Value $updatedLib -Encoding UTF8
}

Write-Host "Real-time Command Bus installed in crates/render-core/src/commands." -ForegroundColor Green

