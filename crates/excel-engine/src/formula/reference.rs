//! ECMA-376 Formula Reference Representations.
//! Cell references (A1, $A$1, R1C1), 3D Sheet Spans, Structured Table References,
//! Dynamic Array Spill references, and Implicit Intersection.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).

use crate::grid::{col_to_name, name_to_col, MAX_COLS, MAX_ROWS};
use ooxml_opc::error::{EngineError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a single cell coordinate with absolute/relative flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellAddress {
    /// 0-based column index (0..16,383)
    pub col: u32,
    /// 0-based row index (0..1,048,575)
    pub row: u32,
    /// Whether column is absolute ($A vs A)
    pub col_abs: bool,
    /// Whether row is absolute ($1 vs 1)
    pub row_abs: bool,
}

impl CellAddress {
    /// Creates a new cell address.
    pub fn new(row: u32, col: u32, row_abs: bool, col_abs: bool) -> Result<Self> {
        if row >= MAX_ROWS {
            return Err(EngineError::FormulaError(format!(
                "Row {} exceeds maximum supported Excel row {}",
                row,
                MAX_ROWS - 1
            )));
        }
        if col >= MAX_COLS {
            return Err(EngineError::FormulaError(format!(
                "Column {} exceeds maximum supported Excel col {}",
                col,
                MAX_COLS - 1
            )));
        }
        Ok(Self {
            row,
            col,
            row_abs,
            col_abs,
        })
    }

    /// Parses an A1-style cell reference (e.g., "$B$12", "C5", "$A1", "XFD1048576").
    pub fn parse_a1(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(EngineError::FormulaError(
                "Empty cell reference string".to_string(),
            ));
        }

        let mut chars = trimmed.char_indices().peekable();
        let col_abs = if let Some(&(_, '$')) = chars.peek() {
            chars.next();
            true
        } else {
            false
        };

        let col_start = chars.peek().map(|&(i, _)| i).unwrap_or(0);
        let mut col_end = col_start;
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_alphabetic() {
                chars.next();
                col_end = chars.peek().map(|&(i, _)| i).unwrap_or(trimmed.len());
            } else {
                break;
            }
        }

        if col_start == col_end {
            return Err(EngineError::FormulaError(format!(
                "Missing column letters in cell reference: '{trimmed}'"
            )));
        }

        let col_str = &trimmed[col_start..col_end];
        let col = name_to_col(col_str)?;

        let row_abs = if let Some(&(_, '$')) = chars.peek() {
            chars.next();
            true
        } else {
            false
        };

        let row_start = chars.peek().map(|&(i, _)| i).unwrap_or(0);
        let mut row_end = row_start;
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_digit() {
                chars.next();
                row_end = chars.peek().map(|&(i, _)| i).unwrap_or(trimmed.len());
            } else {
                break;
            }
        }

        if row_start == row_end {
            return Err(EngineError::FormulaError(format!(
                "Missing row digits in cell reference: '{trimmed}'"
            )));
        }

        if chars.peek().is_some() {
            return Err(EngineError::FormulaError(format!(
                "Unexpected trailing characters in cell reference: '{trimmed}'"
            )));
        }

        let row_str = &trimmed[row_start..row_end];
        let row_num: u32 = row_str.parse().map_err(|e| {
            EngineError::FormulaError(format!("Invalid row number '{row_str}': {e}"))
        })?;

        if row_num == 0 || row_num > MAX_ROWS {
            return Err(EngineError::FormulaError(format!(
                "Row number {} out of bounds (1..{})",
                row_num, MAX_ROWS
            )));
        }

        Self::new(row_num - 1, col, row_abs, col_abs)
    }

    /// Formats as standard A1 notation.
    pub fn to_a1(&self) -> String {
        let col_prefix = if self.col_abs { "$" } else { "" };
        let row_prefix = if self.row_abs { "$" } else { "" };
        let col_name = col_to_name(self.col).unwrap_or_else(|_| "A".to_string());
        format!("{}{}{}{}", col_prefix, col_name, row_prefix, self.row + 1)
    }

    /// Formats as R1C1 notation relative to an optional base cell.
    pub fn to_r1c1(&self, base: Option<(u32, u32)>) -> String {
        let row_part = if self.row_abs || base.is_none() {
            format!("R{}", self.row + 1)
        } else {
            let base_row = base.map(|(r, _)| r).unwrap_or(0);
            let diff = self.row as i64 - base_row as i64;
            if diff == 0 {
                "R".to_string()
            } else {
                format!("R[{diff}]")
            }
        };

        let col_part = if self.col_abs || base.is_none() {
            format!("C{}", self.col + 1)
        } else {
            let base_col = base.map(|(_, c)| c).unwrap_or(0);
            let diff = self.col as i64 - base_col as i64;
            if diff == 0 {
                "C".to_string()
            } else {
                format!("C[{diff}]")
            }
        };

        format!("{row_part}{col_part}")
    }

    /// Parses an R1C1 notation string.
    pub fn parse_r1c1(s: &str, base: Option<(u32, u32)>) -> Result<Self> {
        let trimmed = s.trim();
        if !trimmed.starts_with('R') && !trimmed.starts_with('r') {
            return Err(EngineError::FormulaError(format!(
                "Invalid R1C1 reference, expected 'R' prefix: '{trimmed}'"
            )));
        }

        let mut rest = &trimmed[1..];
        let (row, row_abs, next_rest) = parse_r1c1_coordinate(rest, 'C', base.map(|(r, _)| r))?;
        rest = next_rest;

        if !rest.starts_with('C') && !rest.starts_with('c') {
            return Err(EngineError::FormulaError(format!(
                "Invalid R1C1 reference, expected 'C' part: '{trimmed}'"
            )));
        }
        rest = &rest[1..];
        let (col, col_abs, final_rest) = parse_r1c1_coordinate(rest, '\0', base.map(|(_, c)| c))?;

        if !final_rest.is_empty() {
            return Err(EngineError::FormulaError(format!(
                "Unexpected trailing characters in R1C1 reference: '{trimmed}'"
            )));
        }

        Self::new(row, col, row_abs, col_abs)
    }
}

fn parse_r1c1_coordinate(
    s: &str,
    terminator: char,
    base: Option<u32>,
) -> Result<(u32, bool, &str)> {
    if s.starts_with('[') {
        let end_bracket = s.find(']').ok_or_else(|| {
            EngineError::FormulaError(format!("Unclosed bracket in R1C1 coordinate: '{s}'"))
        })?;
        let offset_str = &s[1..end_bracket];
        let offset: i64 = offset_str.parse().map_err(|e| {
            EngineError::FormulaError(format!("Invalid R1C1 offset '{offset_str}': {e}"))
        })?;
        let base_coord = base.unwrap_or(0) as i64;
        let final_coord = base_coord + offset;
        if final_coord < 0 {
            return Err(EngineError::FormulaError(format!(
                "R1C1 relative offset results in negative coordinate: {final_coord}"
            )));
        }
        let rest = &s[end_bracket + 1..];
        Ok((final_coord as u32, false, rest))
    } else {
        // Digits or empty until terminator
        let end_idx = if terminator == '\0' {
            s.len()
        } else {
            s.find(|c: char| c.to_ascii_uppercase() == terminator)
                .unwrap_or(s.len())
        };
        let num_str = &s[..end_idx];
        if num_str.is_empty() {
            // e.g. "RC" or "RC[1]" - means relative offset 0
            let base_coord = base.unwrap_or(0);
            Ok((base_coord, false, &s[end_idx..]))
        } else {
            let num: u32 = num_str.parse().map_err(|e| {
                EngineError::FormulaError(format!(
                    "Invalid absolute R1C1 coordinate '{num_str}': {e}"
                ))
            })?;
            if num == 0 {
                return Err(EngineError::FormulaError(
                    "R1C1 1-based coordinate cannot be 0".to_string(),
                ));
            }
            Ok((num - 1, true, &s[end_idx..]))
        }
    }
}

impl fmt::Display for CellAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a1())
    }
}

/// Represents a 2D rectangular cell range (e.g. `A1:B10`, `A:A`, `1:5`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RangeAddress {
    pub start: CellAddress,
    pub end: CellAddress,
    pub is_whole_col: bool,
    pub is_whole_row: bool,
}

impl RangeAddress {
    pub fn new(start: CellAddress, end: CellAddress) -> Self {
        Self {
            start,
            end,
            is_whole_col: false,
            is_whole_row: false,
        }
    }

    pub fn whole_columns(
        start_col: u32,
        end_col: u32,
        start_abs: bool,
        end_abs: bool,
    ) -> Result<Self> {
        let start = CellAddress::new(0, start_col, false, start_abs)?;
        let end = CellAddress::new(MAX_ROWS - 1, end_col, false, end_abs)?;
        Ok(Self {
            start,
            end,
            is_whole_col: true,
            is_whole_row: false,
        })
    }

    pub fn whole_rows(
        start_row: u32,
        end_row: u32,
        start_abs: bool,
        end_abs: bool,
    ) -> Result<Self> {
        let start = CellAddress::new(start_row, 0, start_abs, false)?;
        let end = CellAddress::new(end_row, MAX_COLS - 1, end_abs, false)?;
        Ok(Self {
            start,
            end,
            is_whole_col: false,
            is_whole_row: true,
        })
    }

    /// Parses an A1-style range reference (e.g. "A1:B10", "$A:$C", "1:10").
    pub fn parse_a1(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() != 2 {
            return Err(EngineError::FormulaError(format!(
                "Invalid range string, expected single colon: '{trimmed}'"
            )));
        }

        let p1 = parts[0].trim();
        let p2 = parts[1].trim();

        // Check if whole column range: e.g. "A:B", "$A:$C"
        let is_col1 = p1
            .trim_start_matches('$')
            .chars()
            .all(|c| c.is_ascii_alphabetic());
        let is_col2 = p2
            .trim_start_matches('$')
            .chars()
            .all(|c| c.is_ascii_alphabetic());
        if is_col1 && is_col2 && !p1.is_empty() && !p2.is_empty() {
            let col1_abs = p1.starts_with('$');
            let col2_abs = p2.starts_with('$');
            let col1 = name_to_col(p1.trim_start_matches('$'))?;
            let col2 = name_to_col(p2.trim_start_matches('$'))?;
            return Self::whole_columns(col1, col2, col1_abs, col2_abs);
        }

        // Check if whole row range: e.g. "1:10", "$1:$5"
        let is_row1 = p1
            .trim_start_matches('$')
            .chars()
            .all(|c| c.is_ascii_digit());
        let is_row2 = p2
            .trim_start_matches('$')
            .chars()
            .all(|c| c.is_ascii_digit());
        if is_row1 && is_row2 && !p1.is_empty() && !p2.is_empty() {
            let row1_abs = p1.starts_with('$');
            let row2_abs = p2.starts_with('$');
            let r1: u32 = p1
                .trim_start_matches('$')
                .parse()
                .map_err(|e| EngineError::FormulaError(format!("Invalid row '{p1}': {e}")))?;
            let r2: u32 = p2
                .trim_start_matches('$')
                .parse()
                .map_err(|e| EngineError::FormulaError(format!("Invalid row '{p2}': {e}")))?;
            if r1 == 0 || r2 == 0 || r1 > MAX_ROWS || r2 > MAX_ROWS {
                return Err(EngineError::FormulaError(format!(
                    "Row range out of bounds: {r1}:{r2}"
                )));
            }
            return Self::whole_rows(r1 - 1, r2 - 1, row1_abs, row2_abs);
        }

        let start = CellAddress::parse_a1(p1)?;
        let end = CellAddress::parse_a1(p2)?;
        Ok(Self::new(start, end))
    }

    /// Formats as standard A1 notation.
    pub fn to_a1(&self) -> String {
        if self.is_whole_col {
            let c1_abs = if self.start.col_abs { "$" } else { "" };
            let c2_abs = if self.end.col_abs { "$" } else { "" };
            let c1 = col_to_name(self.start.col).unwrap_or_else(|_| "A".to_string());
            let c2 = col_to_name(self.end.col).unwrap_or_else(|_| "A".to_string());
            format!("{}{}:{}{}", c1_abs, c1, c2_abs, c2)
        } else if self.is_whole_row {
            let r1_abs = if self.start.row_abs { "$" } else { "" };
            let r2_abs = if self.end.row_abs { "$" } else { "" };
            format!(
                "{}{}:{}{}",
                r1_abs,
                self.start.row + 1,
                r2_abs,
                self.end.row + 1
            )
        } else {
            format!("{}:{}", self.start.to_a1(), self.end.to_a1())
        }
    }
}

impl fmt::Display for RangeAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a1())
    }
}

/// Represents a sheet reference or a 3D span across multiple sheets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SheetRef {
    /// Single worksheet name (e.g., `Sheet1`, `'Sales 2024'`)
    Single(String),
    /// 3D span across worksheets (e.g., `Sheet1:Sheet3`, `'Jan:Dec'`)
    Span(String, String),
}

impl SheetRef {
    /// Returns the formatted sheet reference prefix including the `!` suffix.
    pub fn to_prefix(&self) -> String {
        match self {
            Self::Single(s) => {
                if needs_quotes(s) {
                    format!("'{}'!", escape_single_quotes(s))
                } else {
                    format!("{s}!")
                }
            }
            Self::Span(start, end) => {
                let s = format!("{start}:{end}");
                if needs_quotes(start) || needs_quotes(end) || s.contains(':') {
                    format!("'{}'!", escape_single_quotes(&s))
                } else {
                    format!("{start}:{end}!")
                }
            }
        }
    }
}

fn needs_quotes(s: &str) -> bool {
    s.contains(' ')
        || s.contains('\'')
        || s.contains('-')
        || s.contains('+')
        || s.contains('*')
        || s.contains('/')
        || s.contains('&')
        || s.contains('#')
        || s.contains(':')
        || s.contains('(')
        || s.contains(')')
        || s.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
}

fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "''")
}

/// Structured Table section specifiers (ECMA-376 Part 1 §18.17.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableSection {
    All,
    Data,
    Headers,
    Totals,
    ThisRow,
}

impl TableSection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "#All",
            Self::Data => "#Data",
            Self::Headers => "#Headers",
            Self::Totals => "#Totals",
            Self::ThisRow => "#This Row",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        let clean = s.trim().trim_start_matches('[').trim_end_matches(']');
        if clean.eq_ignore_ascii_case("#All") {
            Some(Self::All)
        } else if clean.eq_ignore_ascii_case("#Data") {
            Some(Self::Data)
        } else if clean.eq_ignore_ascii_case("#Headers") {
            Some(Self::Headers)
        } else if clean.eq_ignore_ascii_case("#Totals") {
            Some(Self::Totals)
        } else if clean.eq_ignore_ascii_case("#This Row") || clean == "@" {
            Some(Self::ThisRow)
        } else {
            None
        }
    }
}

impl std::str::FromStr for TableSection {
    type Err = EngineError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_opt(s).ok_or_else(|| {
            EngineError::FormulaError(format!("Unrecognized table section specifier: '{s}'"))
        })
    }
}

/// ECMA-376 Structured Table Reference (e.g. `DeptSales[[#Headers],[Sales]]`, `[@Amount]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructuredTableRef {
    /// Optional table name (e.g., `Table1`). Omitted for intra-table reference `[@Amount]`.
    pub table_name: Option<String>,
    /// Table section specifiers (`#All`, `#Headers`, `#Data`, `#Totals`, `#This Row`).
    pub sections: Vec<TableSection>,
    /// Primary column name or start column of range.
    pub column: Option<String>,
    /// Optional end column for structured column spans `[Col1]:[Col2]`.
    pub end_column: Option<String>,
    /// Whether implicit '@' was used (e.g. `Table1[@Sales]`).
    pub is_at_syntax: bool,
}

impl StructuredTableRef {
    /// Formats as standard Excel structured reference string.
    pub fn to_formula_string(&self) -> String {
        let mut out = String::new();
        if let Some(ref tbl) = self.table_name {
            out.push_str(tbl);
        }

        out.push('[');

        if self.is_at_syntax && self.sections.is_empty() {
            out.push('@');
            if let Some(ref col) = self.column {
                if needs_column_brackets(col) {
                    out.push('[');
                    out.push_str(col);
                    out.push(']');
                } else {
                    out.push_str(col);
                }
            }
        } else {
            let mut parts = Vec::new();
            for sec in &self.sections {
                parts.push(format!("[{}]", sec.as_str()));
            }

            if let Some(ref col) = self.column {
                if let Some(ref end) = self.end_column {
                    parts.push(format!("[{col}]:[{end}]"));
                } else if parts.is_empty() {
                    // Single column shorthand without outer bracket duplication
                    if needs_column_brackets(col) {
                        parts.push(format!("[{col}]"));
                    } else {
                        out.push_str(col);
                        out.push(']');
                        return out;
                    }
                } else {
                    parts.push(format!("[{col}]"));
                }
            }

            out.push_str(&parts.join(","));
        }

        out.push(']');
        out
    }
}

fn needs_column_brackets(col: &str) -> bool {
    col.contains(' ')
        || col.contains('#')
        || col.contains('[')
        || col.contains(']')
        || col.contains('-')
        || col.contains(',')
        || col.contains(':')
}

/// Unified reference representation in ECMA-376 formula AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reference {
    /// Single cell reference (optional sheet / 3D span)
    Cell {
        sheet: Option<SheetRef>,
        cell: CellAddress,
    },
    /// Range reference (optional sheet / 3D span)
    Range {
        sheet: Option<SheetRef>,
        range: RangeAddress,
    },
    /// Structured table reference
    Table(StructuredTableRef),
    /// Defined name or named range reference (e.g. `TaxRate`, `TotalSales`)
    Named(String),
    /// Dynamic array spill operator postfix reference (e.g. `A1#`, `Sheet1!A1#`)
    Spill(Box<Reference>),
    /// Dynamic array implicit intersection prefix reference (e.g. `@A1:A10`, `@Table1[Col]`)
    ImplicitIntersection(Box<Reference>),
}

impl Reference {
    /// Formats as canonical formula reference string.
    pub fn to_formula_string(&self) -> String {
        match self {
            Self::Cell { sheet, cell } => {
                let prefix = sheet.as_ref().map(|s| s.to_prefix()).unwrap_or_default();
                format!("{}{}", prefix, cell.to_a1())
            }
            Self::Range { sheet, range } => {
                let prefix = sheet.as_ref().map(|s| s.to_prefix()).unwrap_or_default();
                format!("{}{}", prefix, range.to_a1())
            }
            Self::Table(tbl) => tbl.to_formula_string(),
            Self::Named(name) => name.clone(),
            Self::Spill(base) => format!("{}#", base.to_formula_string()),
            Self::ImplicitIntersection(base) => format!("@{}", base.to_formula_string()),
        }
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_formula_string())
    }
}
