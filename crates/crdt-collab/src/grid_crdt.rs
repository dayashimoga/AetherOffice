use crate::error::CrdtError;
use crate::fugue::StateVector;
use crate::id::OpId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellCoord {
    pub sheet: String,
    pub row: u32,
    pub col: u32,
}

impl CellCoord {
    pub fn new(sheet: impl Into<String>, row: u32, col: u32) -> Self {
        Self {
            sheet: sheet.into(),
            row,
            col,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridCellValue {
    Empty,
    Number(f64),
    Text(String),
    Boolean(bool),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellRegister {
    pub id: OpId,
    pub value: GridCellValue,
    pub formula: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridDelta {
    SetCell {
        id: OpId,
        coord: CellCoord,
        value: GridCellValue,
        formula: Option<String>,
        format: Option<String>,
    },
    InsertRow {
        id: OpId,
        sheet: String,
        at_row: u32,
    },
    DeleteRow {
        id: OpId,
        sheet: String,
        at_row: u32,
    },
    InsertCol {
        id: OpId,
        sheet: String,
        at_col: u32,
    },
    DeleteCol {
        id: OpId,
        sheet: String,
        at_col: u32,
    },
}

pub struct CoordinateShifter;

impl CoordinateShifter {
    pub fn shift_coord(
        coord: &CellCoord,
        sheet: &str,
        is_row: bool,
        at_index: u32,
        delta: i32,
    ) -> Option<CellCoord> {
        if coord.sheet != sheet {
            return Some(coord.clone());
        }

        let mut row = coord.row;
        let mut col = coord.col;

        if is_row {
            if delta < 0 && row == at_index {
                return None; // Deleted cell
            }
            if row >= at_index {
                let new_row = row as i64 + delta as i64;
                if new_row < 0 {
                    return None;
                }
                row = new_row as u32;
            }
        } else {
            if delta < 0 && col == at_index {
                return None; // Deleted cell
            }
            if col >= at_index {
                let new_col = col as i64 + delta as i64;
                if new_col < 0 {
                    return None;
                }
                col = new_col as u32;
            }
        }

        Some(CellCoord {
            sheet: coord.sheet.clone(),
            row,
            col,
        })
    }

    /// Rewrites cell coordinates in Excel formula strings (e.g. `=SUM(A1:B10)`).
    pub fn shift_formula_references(
        formula: &str,
        _sheet: &str,
        is_row: bool,
        at_index: u32,
        delta: i32,
    ) -> String {
        let mut result = String::with_capacity(formula.len() + 16);
        let chars: Vec<char> = formula.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            let ch = chars[idx];

            // Check if this looks like start of a column reference (letters A-Z)
            if ch.is_ascii_alphabetic() {
                let mut col_str = String::new();
                while idx < chars.len() && chars[idx].is_ascii_alphabetic() {
                    col_str.push(chars[idx]);
                    idx += 1;
                }

                // Followed immediately by digits (row)
                let mut row_str = String::new();
                while idx < chars.len() && chars[idx].is_ascii_digit() {
                    row_str.push(chars[idx]);
                    idx += 1;
                }

                if !row_str.is_empty() {
                    // Try to parse row number (1-based in Excel formula)
                    if let Ok(row_1based) = row_str.parse::<u32>() {
                        let row_0based = row_1based.saturating_sub(1);
                        let mut new_row = row_0based;
                        let mut valid = true;

                        if is_row {
                            if delta < 0 && row_0based == at_index {
                                valid = false; // #REF!
                            } else if row_0based >= at_index {
                                let shifted = row_0based as i64 + delta as i64;
                                if shifted >= 0 {
                                    new_row = shifted as u32;
                                } else {
                                    valid = false;
                                }
                            }
                        }

                        if valid {
                            result.push_str(&col_str);
                            result.push_str(&(new_row + 1).to_string());
                        } else {
                            result.push_str("#REF!");
                        }
                    } else {
                        // Not a valid cell reference, restore characters
                        result.push_str(&col_str);
                        result.push_str(&row_str);
                    }
                } else {
                    // Just identifier / function name like SUM or AVERAGE
                    result.push_str(&col_str);
                }
            } else {
                result.push(ch);
                idx += 1;
            }
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralOp {
    pub id: OpId,
    pub sheet: String,
    pub is_row: bool,
    pub at_index: u32,
    pub delta: i32,
}

#[derive(Debug, Clone)]
pub struct GridMatrixCrdt {
    pub client_id: u64,
    pub lamport: u64,
    pub seq: u32,
    cells: HashMap<CellCoord, CellRegister>,
    structural_history: Vec<StructuralOp>,
    state_vector: StateVector,
}

impl GridMatrixCrdt {
    pub fn new(client_id: u64) -> Self {
        Self {
            client_id,
            lamport: 0,
            seq: 0,
            cells: HashMap::new(),
            structural_history: Vec::new(),
            state_vector: StateVector::new(),
        }
    }

    pub fn get_cell(&self, sheet: &str, row: u32, col: u32) -> Option<&CellRegister> {
        let coord = CellCoord::new(sheet, row, col);
        self.cells.get(&coord)
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn set_cell_value(
        &mut self,
        sheet: &str,
        row: u32,
        col: u32,
        val: GridCellValue,
    ) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let coord = CellCoord::new(sheet, row, col);

        let delta = GridDelta::SetCell {
            id,
            coord,
            value: val,
            formula: None,
            format: None,
        };

        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn set_cell_formula(
        &mut self,
        sheet: &str,
        row: u32,
        col: u32,
        formula: &str,
        cached_val: GridCellValue,
    ) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let coord = CellCoord::new(sheet, row, col);

        let delta = GridDelta::SetCell {
            id,
            coord,
            value: cached_val,
            formula: Some(formula.to_string()),
            format: None,
        };

        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn insert_row(&mut self, sheet: &str, at_row: u32) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let delta = GridDelta::InsertRow {
            id,
            sheet: sheet.to_string(),
            at_row,
        };
        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn delete_row(&mut self, sheet: &str, at_row: u32) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let delta = GridDelta::DeleteRow {
            id,
            sheet: sheet.to_string(),
            at_row,
        };
        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn insert_col(&mut self, sheet: &str, at_col: u32) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let delta = GridDelta::InsertCol {
            id,
            sheet: sheet.to_string(),
            at_col,
        };
        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn delete_col(&mut self, sheet: &str, at_col: u32) -> GridDelta {
        self.lamport += 1;
        self.seq += 1;
        let id = OpId::new(self.lamport, self.client_id, self.seq);
        let delta = GridDelta::DeleteCol {
            id,
            sheet: sheet.to_string(),
            at_col,
        };
        let _ = self.apply_delta(delta.clone());
        self.state_vector.set(self.client_id, self.seq);
        delta
    }

    pub fn apply_delta(&mut self, delta: GridDelta) -> Result<(), CrdtError> {
        match delta {
            GridDelta::SetCell {
                id,
                coord,
                value,
                formula,
                format,
            } => {
                if id.lamport > self.lamport {
                    self.lamport = id.lamport;
                }
                self.state_vector.set(id.client_id, id.seq);

                // Transform coord against any structural operations that occurred after this edit
                let mut target_coord = Some(coord);
                for op in &self.structural_history {
                    if op.id > id {
                        if let Some(c) = target_coord {
                            target_coord = CoordinateShifter::shift_coord(
                                &c,
                                &op.sheet,
                                op.is_row,
                                op.at_index,
                                op.delta,
                            );
                        }
                    }
                }

                let effective_coord = match target_coord {
                    Some(c) => c,
                    None => return Ok(()), // Cell was deleted by concurrent structural deletion
                };

                // LWW Conflict Resolution: higher OpId wins
                let should_write = match self.cells.get(&effective_coord) {
                    Some(existing) => id > existing.id,
                    None => true,
                };

                if should_write {
                    self.cells.insert(
                        effective_coord,
                        CellRegister {
                            id,
                            value,
                            formula,
                            format,
                        },
                    );
                }
                Ok(())
            }
            GridDelta::InsertRow { id, sheet, at_row } => {
                self.handle_structural_shift(id, &sheet, true, at_row, 1);
                Ok(())
            }
            GridDelta::DeleteRow { id, sheet, at_row } => {
                self.handle_structural_shift(id, &sheet, true, at_row, -1);
                Ok(())
            }
            GridDelta::InsertCol { id, sheet, at_col } => {
                self.handle_structural_shift(id, &sheet, false, at_col, 1);
                Ok(())
            }
            GridDelta::DeleteCol { id, sheet, at_col } => {
                self.handle_structural_shift(id, &sheet, false, at_col, -1);
                Ok(())
            }
        }
    }

    fn handle_structural_shift(
        &mut self,
        id: OpId,
        sheet: &str,
        is_row: bool,
        at_index: u32,
        delta: i32,
    ) {
        if id.lamport > self.lamport {
            self.lamport = id.lamport;
        }
        self.state_vector.set(id.client_id, id.seq);

        self.structural_history.push(StructuralOp {
            id,
            sheet: sheet.to_string(),
            is_row,
            at_index,
            delta,
        });

        let mut old_cells = std::mem::take(&mut self.cells);
        for (coord, mut reg) in old_cells.drain() {
            // Rewrite formula references if present
            if let Some(ref f) = reg.formula {
                reg.formula = Some(CoordinateShifter::shift_formula_references(
                    f, sheet, is_row, at_index, delta,
                ));
            }

            // Shift coordinate
            if let Some(new_coord) =
                CoordinateShifter::shift_coord(&coord, sheet, is_row, at_index, delta)
            {
                self.cells.insert(new_coord, reg);
            }
        }
    }

    pub fn to_cspg(
        &self,
        sheet: &str,
    ) -> Result<excel_engine::CompressedSparsePageGrid, CrdtError> {
        let mut grid = excel_engine::CompressedSparsePageGrid::new();

        for (coord, reg) in &self.cells {
            if coord.sheet == sheet {
                match &reg.value {
                    GridCellValue::Number(n) => {
                        let _ = grid.set_number(coord.row, coord.col, *n);
                    }
                    GridCellValue::Text(t) => {
                        let _ = grid.set_inline_string(coord.row, coord.col, t);
                    }
                    GridCellValue::Boolean(b) => {
                        let _ = grid.set_boolean(coord.row, coord.col, *b);
                    }
                    GridCellValue::Error(_) => {
                        let _ =
                            grid.set_error(coord.row, coord.col, excel_engine::CellError::Value);
                    }
                    GridCellValue::Empty => {}
                }

                if let Some(ref f) = reg.formula {
                    let cached_num = match reg.value {
                        GridCellValue::Number(n) => Some(n),
                        _ => None,
                    };
                    let _ = grid.set_formula(coord.row, coord.col, f, cached_num);
                }
            }
        }

        Ok(grid)
    }
}
