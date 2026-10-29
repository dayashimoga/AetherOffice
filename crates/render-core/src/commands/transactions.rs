//! Transactional Office Commands & State Model
//!
//! Complies with ECMA-376, ISO 32000-1, and GEC-v1.0.
//! Provides atomic, reversible transactions across:
//! - WordprocessingML document text, ranges, and styles
//! - SpreadsheetML cell values, formulas, ranges, and sheets
//! - PresentationML vector shapes, bounds, text, and slides
//! - PDF form fields, text runs, page rotations, and objects

use crate::primitive::Rect2D;
use crate::shell::DocumentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================================
// 1. Unified Office State
// =========================================================================

/// Word inline range style formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordRangeStyle {
    pub start: usize,
    pub end: usize,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub color_hex: Option<String>,
}

/// PowerPoint shape model stored in global state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PptShapeData {
    pub id: u32,
    pub shape_type: String,
    pub rect: Rect2D,
    pub text: String,
}

/// Global office document state spanning Word, Excel, PowerPoint, and PDF.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOfficeState {
    // Word document state
    pub word_content: String,
    pub word_styles: Vec<WordRangeStyle>,

    // Excel worksheet state
    pub excel_cells: HashMap<String, String>,
    pub excel_sheets: Vec<String>,

    // PowerPoint slide state
    pub ppt_shapes: HashMap<u32, PptShapeData>,
    pub ppt_slides: Vec<u32>,

    // PDF document state
    pub pdf_text_runs: HashMap<(usize, usize), String>, // (page_index, run_index) -> text
    pub pdf_fields: HashMap<String, String>,            // field_name -> value
    pub pdf_page_rotations: HashMap<usize, u16>,        // page_index -> angle (0, 90, 180, 270)
    pub pdf_objects: HashMap<u32, Rect2D>,              // object_id -> rect
}

impl Default for GlobalOfficeState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalOfficeState {
    pub fn new() -> Self {
        Self {
            word_content: String::new(),
            word_styles: Vec::new(),
            excel_cells: HashMap::new(),
            excel_sheets: vec!["Sheet1".to_string()],
            ppt_shapes: HashMap::new(),
            ppt_slides: vec![1],
            pdf_text_runs: HashMap::new(),
            pdf_fields: HashMap::new(),
            pdf_page_rotations: HashMap::new(),
            pdf_objects: HashMap::new(),
        }
    }

    pub fn get_excel_cell(&self, coord: &str) -> Option<&str> {
        self.excel_cells.get(coord).map(|s| s.as_str())
    }

    pub fn set_excel_cell(&mut self, coord: impl Into<String>, val: impl Into<String>) {
        self.excel_cells.insert(coord.into(), val.into());
    }

    pub fn get_pdf_field(&self, field_name: &str) -> Option<&str> {
        self.pdf_fields.get(field_name).map(|s| s.as_str())
    }

    pub fn get_pdf_rotation(&self, page_index: usize) -> u16 {
        self.pdf_page_rotations
            .get(&page_index)
            .copied()
            .unwrap_or(0)
    }
}

// =========================================================================
// 2. Transaction Trait
// =========================================================================

/// Abstract reversible transaction executed by the Command Bus.
pub trait OfficeTransaction: std::fmt::Debug + std::any::Any + Send + Sync {
    /// Execute or redo the transaction against the global office state.
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String>;

    /// Revert the transaction against the global office state.
    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String>;

    /// Redo the transaction against the global office state. Defaults to `execute(state)`.
    fn redo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        self.execute(state)
    }

    /// User-facing description for tooltips and undo/redo stacks.
    fn description(&self) -> &str;

    /// Downcasting helpers.
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Document type affected by this transaction.
    fn target_app(&self) -> DocumentType;

    /// Attempt to merge the next transaction into this one (e.g. typing or drag coalescing).
    fn try_merge_next(&mut self, _next: &dyn OfficeTransaction) -> bool {
        false
    }
}

// =========================================================================
// 3. Word Transactions
// =========================================================================

/// Reversible transaction inserting text into a Word document with typing coalescing.
#[derive(Debug, Clone)]
pub struct WordInsertTextTransaction {
    pub offset: usize,
    pub text: String,
}

impl WordInsertTextTransaction {
    pub fn new(offset: usize, text: impl Into<String>) -> Self {
        Self {
            offset,
            text: text.into(),
        }
    }
}

impl OfficeTransaction for WordInsertTextTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if self.offset > state.word_content.len() {
            state.word_content.push_str(&self.text);
        } else {
            state.word_content.insert_str(self.offset, &self.text);
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        let len = self.text.len();
        if self.offset + len <= state.word_content.len() {
            state.word_content.drain(self.offset..self.offset + len);
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Word: Insert Text"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Word
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<WordInsertTextTransaction>() {
            if other.offset == self.offset + self.text.len() {
                self.text.push_str(&other.text);
                return true;
            }
        }
        false
    }
}

/// Reversible transaction deleting text from a Word document.
#[derive(Debug, Clone)]
pub struct WordDeleteTextTransaction {
    pub offset: usize,
    pub deleted_text: String,
}

impl WordDeleteTextTransaction {
    pub fn new(offset: usize, deleted_text: impl Into<String>) -> Self {
        Self {
            offset,
            deleted_text: deleted_text.into(),
        }
    }
}

impl OfficeTransaction for WordDeleteTextTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        let len = self.deleted_text.len();
        if self.offset + len <= state.word_content.len() {
            state.word_content.drain(self.offset..self.offset + len);
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if self.offset > state.word_content.len() {
            state.word_content.push_str(&self.deleted_text);
        } else {
            state
                .word_content
                .insert_str(self.offset, &self.deleted_text);
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Word: Delete Text"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Word
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<WordDeleteTextTransaction>() {
            // Backspace coalescing (leftward)
            if other.offset + other.deleted_text.len() == self.offset {
                self.deleted_text = format!("{}{}", other.deleted_text, self.deleted_text);
                self.offset = other.offset;
                return true;
            }
            // Delete key coalescing (rightward at same offset)
            if other.offset == self.offset {
                self.deleted_text.push_str(&other.deleted_text);
                return true;
            }
        }
        false
    }
}

/// Reversible transaction formatting a character range in Word.
#[derive(Debug, Clone)]
pub struct WordFormatRangeTransaction {
    pub style: WordRangeStyle,
    pub previous_styles: Vec<WordRangeStyle>,
}

impl WordFormatRangeTransaction {
    pub fn new(style: WordRangeStyle) -> Self {
        Self {
            style,
            previous_styles: Vec::new(),
        }
    }
}

impl OfficeTransaction for WordFormatRangeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        self.previous_styles = state.word_styles.clone();
        // Remove intersecting styles and append new style
        state
            .word_styles
            .retain(|s| s.end <= self.style.start || s.start >= self.style.end);
        state.word_styles.push(self.style.clone());
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.word_styles = self.previous_styles.clone();
        Ok(())
    }

    fn description(&self) -> &str {
        "Word: Format Range"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Word
    }
}

// =========================================================================
// 4. Excel Transactions
// =========================================================================

/// Reversible transaction setting a single cell formula or value.
#[derive(Debug, Clone)]
pub struct ExcelSetCellTransaction {
    pub coord: String,
    pub new_value: String,
    pub old_value: Option<String>,
}

impl ExcelSetCellTransaction {
    pub fn new(coord: impl Into<String>, new_value: impl Into<String>) -> Self {
        Self {
            coord: coord.into(),
            new_value: new_value.into(),
            old_value: None,
        }
    }
}

impl OfficeTransaction for ExcelSetCellTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        self.old_value = state
            .excel_cells
            .insert(self.coord.clone(), self.new_value.clone());
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        match &self.old_value {
            Some(v) => {
                state.excel_cells.insert(self.coord.clone(), v.clone());
            }
            None => {
                state.excel_cells.remove(&self.coord);
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Excel: Set Cell Value"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Excel
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<ExcelSetCellTransaction>() {
            if other.coord == self.coord {
                self.new_value = other.new_value.clone();
                return true;
            }
        }
        false
    }
}

/// Reversible transaction pasting multiple cells into Excel.
#[derive(Debug, Clone)]
pub struct ExcelPasteRangeTransaction {
    pub cells: Vec<(String, String)>,
    pub old_cells: Vec<(String, Option<String>)>,
}

impl ExcelPasteRangeTransaction {
    pub fn new(cells: Vec<(String, String)>) -> Self {
        Self {
            cells,
            old_cells: Vec::new(),
        }
    }
}

impl OfficeTransaction for ExcelPasteRangeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        self.old_cells.clear();
        for (coord, val) in &self.cells {
            let prev = state.excel_cells.insert(coord.clone(), val.clone());
            self.old_cells.push((coord.clone(), prev));
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        for (coord, prev) in &self.old_cells {
            match prev {
                Some(v) => {
                    state.excel_cells.insert(coord.clone(), v.clone());
                }
                None => {
                    state.excel_cells.remove(coord);
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Excel: Paste Range"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Excel
    }
}

/// Reversible transaction adding a worksheet.
#[derive(Debug, Clone)]
pub struct ExcelAddSheetTransaction {
    pub sheet_name: String,
}

impl ExcelAddSheetTransaction {
    pub fn new(sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
        }
    }
}

impl OfficeTransaction for ExcelAddSheetTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if !state.excel_sheets.contains(&self.sheet_name) {
            state.excel_sheets.push(self.sheet_name.clone());
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.excel_sheets.retain(|s| s != &self.sheet_name);
        Ok(())
    }

    fn description(&self) -> &str {
        "Excel: Add Sheet"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Excel
    }
}

// =========================================================================
// 5. PowerPoint Transactions
// =========================================================================

/// Reversible transaction adding a shape to a slide.
#[derive(Debug, Clone)]
pub struct PptAddShapeTransaction {
    pub shape: PptShapeData,
}

impl PptAddShapeTransaction {
    pub fn new(shape: PptShapeData) -> Self {
        Self { shape }
    }
}

impl OfficeTransaction for PptAddShapeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.ppt_shapes.insert(self.shape.id, self.shape.clone());
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.ppt_shapes.remove(&self.shape.id);
        Ok(())
    }

    fn description(&self) -> &str {
        "PowerPoint: Add Shape"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::PowerPoint
    }
}

/// Reversible transaction moving or resizing a PowerPoint shape.
#[derive(Debug, Clone)]
pub struct PptMoveShapeTransaction {
    pub shape_id: u32,
    pub old_rect: Rect2D,
    pub new_rect: Rect2D,
}

impl PptMoveShapeTransaction {
    pub fn new(shape_id: u32, old_rect: Rect2D, new_rect: Rect2D) -> Self {
        Self {
            shape_id,
            old_rect,
            new_rect,
        }
    }
}

impl OfficeTransaction for PptMoveShapeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if let Some(shape) = state.ppt_shapes.get_mut(&self.shape_id) {
            shape.rect = self.new_rect;
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        if let Some(shape) = state.ppt_shapes.get_mut(&self.shape_id) {
            shape.rect = self.old_rect;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "PowerPoint: Transform Shape"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::PowerPoint
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<PptMoveShapeTransaction>() {
            if other.shape_id == self.shape_id {
                self.new_rect = other.new_rect;
                return true;
            }
        }
        false
    }
}

/// Reversible transaction deleting a PowerPoint shape.
#[derive(Debug, Clone)]
pub struct PptDeleteShapeTransaction {
    pub shape: PptShapeData,
}

impl PptDeleteShapeTransaction {
    pub fn new(shape: PptShapeData) -> Self {
        Self { shape }
    }
}

impl OfficeTransaction for PptDeleteShapeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.ppt_shapes.remove(&self.shape.id);
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.ppt_shapes.insert(self.shape.id, self.shape.clone());
        Ok(())
    }

    fn description(&self) -> &str {
        "PowerPoint: Delete Shape"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::PowerPoint
    }
}

// =========================================================================
// 6. PDF Transactions
// =========================================================================

/// Reversible transaction editing text on a PDF page with typing coalescing.
#[derive(Debug, Clone)]
pub struct PdfEditTextTransaction {
    pub page_index: usize,
    pub run_index: usize,
    pub old_text: String,
    pub new_text: String,
}

impl PdfEditTextTransaction {
    pub fn new(
        page_index: usize,
        run_index: usize,
        old_text: impl Into<String>,
        new_text: impl Into<String>,
    ) -> Self {
        Self {
            page_index,
            run_index,
            old_text: old_text.into(),
            new_text: new_text.into(),
        }
    }
}

impl OfficeTransaction for PdfEditTextTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state
            .pdf_text_runs
            .insert((self.page_index, self.run_index), self.new_text.clone());
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state
            .pdf_text_runs
            .insert((self.page_index, self.run_index), self.old_text.clone());
        Ok(())
    }

    fn description(&self) -> &str {
        "PDF: Edit Text"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Pdf
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<PdfEditTextTransaction>() {
            if other.page_index == self.page_index && other.run_index == self.run_index {
                self.new_text = other.new_text.clone();
                return true;
            }
        }
        false
    }
}

/// Reversible transaction moving a PDF page object with drag coalescing.
#[derive(Debug, Clone)]
pub struct PdfMoveObjectTransaction {
    pub object_id: u32,
    pub old_rect: Rect2D,
    pub new_rect: Rect2D,
}

impl PdfMoveObjectTransaction {
    pub fn new(object_id: u32, old_rect: Rect2D, new_rect: Rect2D) -> Self {
        Self {
            object_id,
            old_rect,
            new_rect,
        }
    }
}

impl OfficeTransaction for PdfMoveObjectTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.pdf_objects.insert(self.object_id, self.new_rect);
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state.pdf_objects.insert(self.object_id, self.old_rect);
        Ok(())
    }

    fn description(&self) -> &str {
        "PDF: Move Object"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Pdf
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<PdfMoveObjectTransaction>() {
            if other.object_id == self.object_id {
                self.new_rect = other.new_rect;
                return true;
            }
        }
        false
    }
}

/// Reversible transaction rotating a PDF page by +/-90 or 180 degrees.
#[derive(Debug, Clone)]
pub struct PdfRotatePageTransaction {
    pub page_index: usize,
    pub delta_deg: i32,
}

impl PdfRotatePageTransaction {
    pub fn new(page_index: usize, delta_deg: i32) -> Self {
        Self {
            page_index,
            delta_deg,
        }
    }
}

impl OfficeTransaction for PdfRotatePageTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        let curr = state.pdf_page_rotations.entry(self.page_index).or_insert(0);
        *curr = ((*curr as i32 + self.delta_deg).rem_euclid(360)) as u16;
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        let curr = state.pdf_page_rotations.entry(self.page_index).or_insert(0);
        *curr = ((*curr as i32 - self.delta_deg).rem_euclid(360)) as u16;
        Ok(())
    }

    fn description(&self) -> &str {
        "PDF: Rotate Page"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Pdf
    }
}

/// Reversible transaction updating an AcroForm field value.
#[derive(Debug, Clone)]
pub struct PdfSetFormFieldTransaction {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

impl PdfSetFormFieldTransaction {
    pub fn new(
        field_name: impl Into<String>,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            old_value: old_value.into(),
            new_value: new_value.into(),
        }
    }
}

impl OfficeTransaction for PdfSetFormFieldTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state
            .pdf_fields
            .insert(self.field_name.clone(), self.new_value.clone());
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        state
            .pdf_fields
            .insert(self.field_name.clone(), self.old_value.clone());
        Ok(())
    }

    fn description(&self) -> &str {
        "PDF: Set Form Field"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::Pdf
    }

    fn try_merge_next(&mut self, next: &dyn OfficeTransaction) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<PdfSetFormFieldTransaction>() {
            if other.field_name == self.field_name {
                self.new_value = other.new_value.clone();
                return true;
            }
        }
        false
    }
}

// =========================================================================
// 7. Compound Atomic Office Transaction
// =========================================================================

/// Compound transaction executing multiple child transactions atomically.
///
/// If any child transaction fails during execution, all previously executed child
/// transactions are rolled back in reverse order.
#[derive(Debug)]
pub struct CompoundOfficeTransaction {
    pub description_str: String,
    pub target_application: DocumentType,
    pub transactions: Vec<Box<dyn OfficeTransaction>>,
}

impl CompoundOfficeTransaction {
    pub fn new(
        description: impl Into<String>,
        target_app: DocumentType,
        transactions: Vec<Box<dyn OfficeTransaction>>,
    ) -> Self {
        Self {
            description_str: description.into(),
            target_application: target_app,
            transactions,
        }
    }
}

impl OfficeTransaction for CompoundOfficeTransaction {
    fn execute(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        for (executed_count, tx) in self.transactions.iter_mut().enumerate() {
            if let Err(e) = tx.execute(state) {
                // Roll back already-executed transactions in reverse
                for rollback in self.transactions[..executed_count].iter_mut().rev() {
                    let _ = rollback.undo(state);
                }
                return Err(format!(
                    "Compound transaction failed at step {executed_count}: {e}"
                ));
            }
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        for tx in self.transactions.iter_mut().rev() {
            tx.undo(state)?;
        }
        Ok(())
    }

    fn redo(&mut self, state: &mut GlobalOfficeState) -> Result<(), String> {
        for tx in &mut self.transactions {
            tx.redo(state)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description_str
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        self.target_application
    }
}
