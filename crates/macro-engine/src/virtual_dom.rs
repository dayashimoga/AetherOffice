use crate::error::MacroError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VbaValue {
    Empty,
    Null,
    Nothing,
    Boolean(bool),
    Integer(i64),
    Double(f64),
    String(String),
}

impl VbaValue {
    pub fn to_display_string(&self) -> String {
        match self {
            VbaValue::Empty => String::new(),
            VbaValue::Null => "Null".to_string(),
            VbaValue::Nothing => "Nothing".to_string(),
            VbaValue::Boolean(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            VbaValue::Integer(i) => i.to_string(),
            VbaValue::Double(f) => f.to_string(),
            VbaValue::String(s) => s.clone(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            VbaValue::Integer(i) => Some(*i as f64),
            VbaValue::Double(f) => Some(*f),
            VbaValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            VbaValue::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            VbaValue::Integer(i) => Some(*i),
            VbaValue::Double(f) => Some(*f as i64),
            VbaValue::Boolean(b) => Some(if *b { 1 } else { 0 }),
            VbaValue::String(s) => s.trim().parse::<i64>().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            VbaValue::Boolean(b) => *b,
            VbaValue::Integer(i) => *i != 0,
            VbaValue::Double(f) => *f != 0.0,
            VbaValue::String(s) => !s.is_empty() && !s.eq_ignore_ascii_case("false"),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualCell {
    pub value: VbaValue,
    pub formula: Option<String>,
    pub interior_color: Option<u32>,
    pub font_bold: bool,
    pub font_italic: bool,
    pub font_color: Option<u32>,
    pub font_size: Option<f64>,
}

impl Default for VirtualCell {
    fn default() -> Self {
        Self {
            value: VbaValue::Empty,
            formula: None,
            interior_color: None,
            font_bold: false,
            font_italic: false,
            font_color: None,
            font_size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeAddress {
    pub start_row: u32, // 1-based
    pub start_col: u32, // 1-based
    pub end_row: u32,
    pub end_col: u32,
}

impl RangeAddress {
    pub fn single_cell(row: u32, col: u32) -> Self {
        Self {
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
        }
    }

    pub fn from_a1(a1: &str) -> Result<Self, MacroError> {
        let trimmed = a1.trim();
        if let Some((start_part, end_part)) = trimmed.split_once(':') {
            let (r1, c1) = parse_single_cell_ref(start_part)?;
            let (r2, c2) = parse_single_cell_ref(end_part)?;
            Ok(Self {
                start_row: u32::min(r1, r2),
                start_col: u32::min(c1, c2),
                end_row: u32::max(r1, r2),
                end_col: u32::max(c1, c2),
            })
        } else {
            let (r, c) = parse_single_cell_ref(trimmed)?;
            Ok(Self::single_cell(r, c))
        }
    }

    pub fn offset(&self, row_offset: i32, col_offset: i32) -> Result<Self, MacroError> {
        let new_r1 = (self.start_row as i32 + row_offset).max(1) as u32;
        let new_c1 = (self.start_col as i32 + col_offset).max(1) as u32;
        let new_r2 = (self.end_row as i32 + row_offset).max(1) as u32;
        let new_c2 = (self.end_col as i32 + col_offset).max(1) as u32;
        Ok(Self {
            start_row: new_r1,
            start_col: new_c1,
            end_row: new_r2,
            end_col: new_c2,
        })
    }
}

fn parse_single_cell_ref(s: &str) -> Result<(u32, u32), MacroError> {
    let clean = s.trim().trim_start_matches('$');
    let mut col_str = String::new();
    let mut row_str = String::new();

    for ch in clean.chars() {
        if ch == '$' {
            continue;
        }
        if ch.is_ascii_alphabetic() {
            col_str.push(ch.to_ascii_uppercase());
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }

    if col_str.is_empty() || row_str.is_empty() {
        return Err(MacroError::DomError(format!(
            "Invalid cell reference: '{s}'"
        )));
    }

    // Convert col letters A, B.. AA to 1-based index
    let mut col = 0u32;
    for c in col_str.chars() {
        let val = (c as u8 - b'A' + 1) as u32;
        col = col * 26 + val;
    }

    let row = row_str.parse::<u32>().map_err(|e| {
        MacroError::DomError(format!("Invalid row number in cell reference '{s}': {e}"))
    })?;

    Ok((row, col))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualWorksheet {
    pub name: String,
    pub cells: HashMap<(u32, u32), VirtualCell>,
    pub selected_range: RangeAddress,
}

impl VirtualWorksheet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
            selected_range: RangeAddress::single_cell(1, 1),
        }
    }

    pub fn get_cell(&self, row: u32, col: u32) -> VirtualCell {
        self.cells
            .get(&(row, col))
            .cloned()
            .unwrap_or_else(VirtualCell::default)
    }

    pub fn set_cell(&mut self, row: u32, col: u32, cell: VirtualCell) {
        self.cells.insert((row, col), cell);
    }

    pub fn get_value(&self, row: u32, col: u32) -> VbaValue {
        self.cells
            .get(&(row, col))
            .map(|c| c.value.clone())
            .unwrap_or(VbaValue::Empty)
    }

    pub fn set_value(&mut self, row: u32, col: u32, val: VbaValue) {
        let cell = self.cells.entry((row, col)).or_default();
        cell.value = val;
    }

    pub fn get_formula(&self, row: u32, col: u32) -> Option<String> {
        self.cells.get(&(row, col)).and_then(|c| c.formula.clone())
    }

    pub fn set_formula(&mut self, row: u32, col: u32, formula: String) {
        let cell = self.cells.entry((row, col)).or_default();
        cell.formula = Some(formula);
    }

    pub fn clear_range(&mut self, range: RangeAddress) {
        for r in range.start_row..=range.end_row {
            for c in range.start_col..=range.end_col {
                self.cells.remove(&(r, c));
            }
        }
    }

    pub fn set_range_value(&mut self, range: RangeAddress, val: VbaValue) {
        for r in range.start_row..=range.end_row {
            for c in range.start_col..=range.end_col {
                self.set_value(r, c, val.clone());
            }
        }
    }

    pub fn set_range_interior_color(&mut self, range: RangeAddress, color: u32) {
        for r in range.start_row..=range.end_row {
            for c in range.start_col..=range.end_col {
                let cell = self.cells.entry((r, c)).or_default();
                cell.interior_color = Some(color);
            }
        }
    }

    pub fn copy_range(&mut self, src: RangeAddress, dest_row: u32, dest_col: u32) {
        let mut cloned = Vec::new();
        for r in src.start_row..=src.end_row {
            for c in src.start_col..=src.end_col {
                if let Some(cell) = self.cells.get(&(r, c)) {
                    let r_off = r - src.start_row;
                    let c_off = c - src.start_col;
                    cloned.push((dest_row + r_off, dest_col + c_off, cell.clone()));
                }
            }
        }

        for (r, c, cell) in cloned {
            self.cells.insert((r, c), cell);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualWorkbook {
    pub name: String,
    pub sheets: Vec<VirtualWorksheet>,
    pub active_sheet_idx: usize,
}

impl VirtualWorkbook {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sheets: vec![VirtualWorksheet::new("Sheet1")],
            active_sheet_idx: 0,
        }
    }

    pub fn active_sheet(&self) -> Result<&VirtualWorksheet, MacroError> {
        self.sheets
            .get(self.active_sheet_idx)
            .ok_or_else(|| MacroError::DomError("No active sheet in workbook".to_string()))
    }

    pub fn active_sheet_mut(&mut self) -> Result<&mut VirtualWorksheet, MacroError> {
        self.sheets
            .get_mut(self.active_sheet_idx)
            .ok_or_else(|| MacroError::DomError("No active sheet in workbook".to_string()))
    }

    pub fn get_sheet_by_name(&self, name: &str) -> Option<&VirtualWorksheet> {
        self.sheets
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn get_sheet_by_name_mut(&mut self, name: &str) -> Option<&mut VirtualWorksheet> {
        self.sheets
            .iter_mut()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualApplication {
    pub screen_updating: bool,
    pub calculation: String,
    pub display_alerts: bool,
    pub status_bar: Option<String>,
}

impl Default for VirtualApplication {
    fn default() -> Self {
        Self {
            screen_updating: true,
            calculation: "Automatic".to_string(),
            display_alerts: true,
            status_bar: None,
        }
    }
}

/// Root Office Object Model Bridge Container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOfficeDom {
    pub application: VirtualApplication,
    pub workbooks: Vec<VirtualWorkbook>,
    pub active_workbook_idx: usize,
    pub clipboard_cell_data: Option<HashMap<(u32, u32), VirtualCell>>,
}

impl Default for VirtualOfficeDom {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualOfficeDom {
    pub fn new() -> Self {
        Self {
            application: VirtualApplication::default(),
            workbooks: vec![VirtualWorkbook::new("Book1")],
            active_workbook_idx: 0,
            clipboard_cell_data: None,
        }
    }

    pub fn active_workbook(&self) -> Result<&VirtualWorkbook, MacroError> {
        self.workbooks
            .get(self.active_workbook_idx)
            .ok_or_else(|| MacroError::DomError("No active workbook".to_string()))
    }

    pub fn active_workbook_mut(&mut self) -> Result<&mut VirtualWorkbook, MacroError> {
        self.workbooks
            .get_mut(self.active_workbook_idx)
            .ok_or_else(|| MacroError::DomError("No active workbook".to_string()))
    }

    pub fn active_sheet(&self) -> Result<&VirtualWorksheet, MacroError> {
        self.active_workbook()?.active_sheet()
    }

    pub fn active_sheet_mut(&mut self) -> Result<&mut VirtualWorksheet, MacroError> {
        self.active_workbook_mut()?.active_sheet_mut()
    }
}
