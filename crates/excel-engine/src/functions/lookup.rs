//! OpenOfficeRS Excel Engine - Lookup & Reference Functions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements XLOOKUP, XMATCH, VLOOKUP, HLOOKUP, INDEX, MATCH, OFFSET, INDIRECT,
//! FILTER, UNIQUE, and SORT.

use crate::functions::math_stat::wildcard_match;
use crate::grid::{cell_to_coords, parse_range, CellError, CellValue, CompressedSparsePageGrid};
use std::collections::HashSet;

/// Compares two CellValues for equality (case-insensitive for strings, float epsilon for numbers).
pub fn cell_values_equal(a: &CellValue, b: &CellValue, wildcard: bool) -> bool {
    match (a, b) {
        (CellValue::Number(x), CellValue::Number(y)) => (x - y).abs() < 1e-12,
        (CellValue::Boolean(x), CellValue::Boolean(y)) => x == y,
        (CellValue::InlineString(x), CellValue::InlineString(y)) => {
            if wildcard {
                wildcard_match(y, x) || wildcard_match(x, y)
            } else {
                x.eq_ignore_ascii_case(y)
            }
        }
        (CellValue::Empty, CellValue::Empty) => true,
        _ => false,
    }
}

/// Helper comparing two CellValues: returns Some(Ordering) if comparable.
pub fn compare_cell_values(a: &CellValue, b: &CellValue) -> std::cmp::Ordering {
    match (a, b) {
        (CellValue::Number(x), CellValue::Number(y)) => {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (CellValue::InlineString(x), CellValue::InlineString(y)) => {
            x.to_ascii_uppercase().cmp(&y.to_ascii_uppercase())
        }
        (CellValue::Boolean(x), CellValue::Boolean(y)) => x.cmp(y),
        // Type precedence: Number < String < Boolean < Error
        (CellValue::Number(_), _) => std::cmp::Ordering::Less,
        (_, CellValue::Number(_)) => std::cmp::Ordering::Greater,
        (CellValue::InlineString(_), _) => std::cmp::Ordering::Less,
        (_, CellValue::InlineString(_)) => std::cmp::Ordering::Greater,
        (CellValue::Boolean(_), _) => std::cmp::Ordering::Less,
        (_, CellValue::Boolean(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    }
}

/// Helper converting a CellValue into a 2D matrix (rows x cols).
pub fn to_matrix(val: &CellValue) -> Vec<Vec<CellValue>> {
    match val {
        CellValue::Array(rows) => rows.clone(),
        other => vec![vec![other.clone()]],
    }
}

/// Helper converting a CellValue into a 1D vector.
pub fn to_flat_vector(val: &CellValue) -> Vec<CellValue> {
    match val {
        CellValue::Array(rows) => {
            let mut v = Vec::new();
            for r in rows {
                for c in r {
                    v.push(c.clone());
                }
            }
            v
        }
        other => vec![other.clone()],
    }
}

/// Evaluates `XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode = 0], [search_mode = 1])`.
pub fn fn_xlookup(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let target = &args[0];
    let lookup_vec = to_flat_vector(&args[1]);
    let return_vec = to_flat_vector(&args[2]);

    if lookup_vec.is_empty() {
        return CellValue::Error(CellError::NA);
    }

    let if_not_found = args.get(3).cloned();
    let match_mode = args.get(4).and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
    let search_mode = args.get(5).and_then(|v| v.as_f64()).unwrap_or(1.0) as i32;

    let matched_idx = match_in_slice(target, &lookup_vec, match_mode, search_mode);

    match matched_idx {
        Some(idx) => {
            if idx < return_vec.len() {
                return_vec[idx].clone()
            } else {
                CellValue::Error(CellError::Ref)
            }
        }
        None => if_not_found.unwrap_or(CellValue::Error(CellError::NA)),
    }
}

/// Evaluates `XMATCH(lookup_value, lookup_array, [match_mode = 0], [search_mode = 1])`.
pub fn fn_xmatch(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let target = &args[0];
    let lookup_vec = to_flat_vector(&args[1]);
    let match_mode = args.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
    let search_mode = args.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0) as i32;

    match match_in_slice(target, &lookup_vec, match_mode, search_mode) {
        Some(idx) => CellValue::Number((idx + 1) as f64), // 1-based index
        None => CellValue::Error(CellError::NA),
    }
}

/// Core matching logic for XLOOKUP/XMATCH.
fn match_in_slice(
    target: &CellValue,
    items: &[CellValue],
    match_mode: i32,
    search_mode: i32,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let wildcard = match_mode == 2;

    // Linear search forward (search_mode 1)
    if search_mode == 1 {
        let mut best_idx = None;
        let mut best_diff = None;

        for (i, item) in items.iter().enumerate() {
            if cell_values_equal(target, item, wildcard) {
                return Some(i);
            }
            if match_mode == -1 {
                // Next smaller
                if compare_cell_values(item, target) == std::cmp::Ordering::Less {
                    if let (Some(cur_val), Some(target_val)) = (item.as_f64(), target.as_f64()) {
                        let diff = target_val - cur_val;
                        if best_diff.is_none_or(|d| diff < d) {
                            best_diff = Some(diff);
                            best_idx = Some(i);
                        }
                    }
                }
            } else if match_mode == 1 {
                // Next larger
                if compare_cell_values(item, target) == std::cmp::Ordering::Greater {
                    if let (Some(cur_val), Some(target_val)) = (item.as_f64(), target.as_f64()) {
                        let diff = cur_val - target_val;
                        if best_diff.is_none_or(|d| diff < d) {
                            best_diff = Some(diff);
                            best_idx = Some(i);
                        }
                    }
                }
            }
        }
        return best_idx;
    }

    // Linear search backward (search_mode -1)
    if search_mode == -1 {
        for i in (0..items.len()).rev() {
            if cell_values_equal(target, &items[i], wildcard) {
                return Some(i);
            }
        }
        return None;
    }

    // Binary search ascending (search_mode 2)
    if search_mode == 2 {
        let mut low = 0;
        let mut high = items.len();
        while low < high {
            let mid = low + (high - low) / 2;
            match compare_cell_values(&items[mid], target) {
                std::cmp::Ordering::Equal => return Some(mid),
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
            }
        }
        if match_mode == -1 && low > 0 {
            return Some(low - 1);
        } else if match_mode == 1 && low < items.len() {
            return Some(low);
        }
        return None;
    }

    // Binary search descending (search_mode -2)
    if search_mode == -2 {
        let mut low = 0;
        let mut high = items.len();
        while low < high {
            let mid = low + (high - low) / 2;
            match compare_cell_values(&items[mid], target) {
                std::cmp::Ordering::Equal => return Some(mid),
                std::cmp::Ordering::Greater => low = mid + 1,
                std::cmp::Ordering::Less => high = mid,
            }
        }
        return None;
    }

    None
}

/// Evaluates `VLOOKUP(lookup_value, table_array, col_index, [range_lookup = true])`.
pub fn fn_vlookup(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let target = &args[0];
    let matrix = to_matrix(&args[1]);
    let col_idx = match args[2].as_f64() {
        Some(v) => v as usize,
        None => return CellValue::Error(CellError::Value),
    };
    let range_lookup = args
        .get(3)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(true);

    if matrix.is_empty() || col_idx < 1 || col_idx > matrix[0].len() {
        return CellValue::Error(CellError::Ref);
    }

    let first_col: Vec<CellValue> = matrix.iter().map(|row| row[0].clone()).collect();

    let matched_row = if range_lookup {
        // Approximate match: table must be sorted ascending; find largest value <= target
        let mut best = None;
        for (r, item) in first_col.iter().enumerate() {
            if compare_cell_values(item, target) != std::cmp::Ordering::Greater {
                best = Some(r);
            } else {
                break;
            }
        }
        best
    } else {
        // Exact match (with wildcards)
        first_col
            .iter()
            .position(|item| cell_values_equal(item, target, true))
    };

    match matched_row {
        Some(r) => matrix[r][col_idx - 1].clone(),
        None => CellValue::Error(CellError::NA),
    }
}

/// Evaluates `HLOOKUP(lookup_value, table_array, row_index, [range_lookup = true])`.
pub fn fn_hlookup(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let target = &args[0];
    let matrix = to_matrix(&args[1]);
    let row_idx = match args[2].as_f64() {
        Some(v) => v as usize,
        None => return CellValue::Error(CellError::Value),
    };
    let range_lookup = args
        .get(3)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(true);

    if matrix.is_empty() || row_idx < 1 || row_idx > matrix.len() {
        return CellValue::Error(CellError::Ref);
    }

    let first_row = &matrix[0];

    let matched_col = if range_lookup {
        let mut best = None;
        for (c, item) in first_row.iter().enumerate() {
            if compare_cell_values(item, target) != std::cmp::Ordering::Greater {
                best = Some(c);
            } else {
                break;
            }
        }
        best
    } else {
        first_row
            .iter()
            .position(|item| cell_values_equal(item, target, true))
    };

    match matched_col {
        Some(c) => matrix[row_idx - 1][c].clone(),
        None => CellValue::Error(CellError::NA),
    }
}

/// Evaluates `INDEX(array, row_num, [col_num])`.
pub fn fn_index(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() {
        return CellValue::Error(CellError::Ref);
    }

    let num_rows = matrix.len();
    let num_cols = matrix[0].len();

    let r_idx = match args[1].as_f64() {
        Some(v) => v as usize,
        None => return CellValue::Error(CellError::Value),
    };
    let c_idx = args
        .get(2)
        .and_then(|v| v.as_f64())
        .map(|v| v as usize)
        .unwrap_or(1);

    // If array is 1 row high, row_num acts as col_num
    if num_rows == 1 && args.len() == 2 && r_idx >= 1 && r_idx <= num_cols {
        return matrix[0][r_idx - 1].clone();
    }

    if r_idx < 1 || r_idx > num_rows || c_idx < 1 || c_idx > num_cols {
        return CellValue::Error(CellError::Ref);
    }

    matrix[r_idx - 1][c_idx - 1].clone()
}

/// Evaluates `MATCH(lookup_value, lookup_array, [match_type = 1])`.
pub fn fn_match(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let target = &args[0];
    let items = to_flat_vector(&args[1]);
    let match_type = args.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as i32;

    if items.is_empty() {
        return CellValue::Error(CellError::NA);
    }

    let idx = if match_type == 0 {
        // Exact match with wildcards
        items
            .iter()
            .position(|item| cell_values_equal(item, target, true))
    } else if match_type == 1 {
        // Largest value <= target (ascending)
        let mut best = None;
        for (i, item) in items.iter().enumerate() {
            if compare_cell_values(item, target) != std::cmp::Ordering::Greater {
                best = Some(i);
            } else {
                break;
            }
        }
        best
    } else if match_type == -1 {
        // Smallest value >= target (descending)
        let mut best = None;
        for (i, item) in items.iter().enumerate() {
            if compare_cell_values(item, target) != std::cmp::Ordering::Less {
                best = Some(i);
            } else {
                break;
            }
        }
        best
    } else {
        None
    };

    match idx {
        Some(i) => CellValue::Number((i + 1) as f64),
        None => CellValue::Error(CellError::NA),
    }
}

/// Evaluates `OFFSET(reference_origin_row, reference_origin_col, rows, cols, [height], [width])` in grid.
pub fn fn_offset(
    grid: &CompressedSparsePageGrid,
    origin_r: u32,
    origin_c: u32,
    args: &[CellValue],
) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let d_row = match args[0].as_f64() {
        Some(v) => v as i64,
        None => return CellValue::Error(CellError::Value),
    };
    let d_col = match args[1].as_f64() {
        Some(v) => v as i64,
        None => return CellValue::Error(CellError::Value),
    };
    let height = args
        .get(2)
        .and_then(|v| v.as_f64())
        .map(|v| v as usize)
        .unwrap_or(1);
    let width = args
        .get(3)
        .and_then(|v| v.as_f64())
        .map(|v| v as usize)
        .unwrap_or(1);

    let target_r = origin_r as i64 + d_row;
    let target_c = origin_c as i64 + d_col;

    if target_r < 0 || target_c < 0 || target_r >= 1_048_576 || target_c >= 16_384 {
        return CellValue::Error(CellError::Ref);
    }

    if height == 1 && width == 1 {
        grid.get_value(target_r as u32, target_c as u32)
    } else {
        let mut rows = Vec::new();
        for r in 0..height {
            let mut row = Vec::new();
            for c in 0..width {
                let cell_val = grid.get_value(
                    (target_r as usize + r) as u32,
                    (target_c as usize + c) as u32,
                );
                row.push(cell_val);
            }
            rows.push(row);
        }
        CellValue::Array(rows)
    }
}

/// Evaluates `INDIRECT(ref_text, [a1_style = true])` using the provided grid.
pub fn fn_indirect(grid: &CompressedSparsePageGrid, args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let ref_str = match &args[0] {
        CellValue::InlineString(s) => s.trim(),
        _ => return CellValue::Error(CellError::Value),
    };

    if ref_str.contains(':') {
        if let Ok(((r1, c1), (r2, c2))) = parse_range(ref_str) {
            let mut rows = Vec::new();
            for r in r1..=r2 {
                let mut row = Vec::new();
                for c in c1..=c2 {
                    row.push(grid.get_value(r, c));
                }
                rows.push(row);
            }
            return CellValue::Array(rows);
        }
    } else if let Ok((r, c)) = cell_to_coords(ref_str) {
        return grid.get_value(r, c);
    }

    CellValue::Error(CellError::Ref)
}

/// Evaluates `FILTER(array, include, [if_empty])`.
pub fn fn_filter(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let matrix = to_matrix(&args[0]);
    let include_vec = to_flat_vector(&args[1]);
    let if_empty = args
        .get(2)
        .cloned()
        .unwrap_or(CellValue::Error(CellError::Calc));

    if matrix.is_empty() || include_vec.len() != matrix.len() {
        return CellValue::Error(CellError::Value);
    }

    let mut filtered_rows = Vec::new();
    for (r, row) in matrix.iter().enumerate() {
        let inc = match &include_vec[r] {
            CellValue::Boolean(b) => *b,
            CellValue::Number(n) => *n != 0.0,
            _ => false,
        };
        if inc {
            filtered_rows.push(row.clone());
        }
    }

    if filtered_rows.is_empty() {
        if_empty
    } else {
        CellValue::Array(filtered_rows)
    }
}

/// Evaluates `UNIQUE(array, [by_col = false], [exactly_once = false])`.
pub fn fn_unique(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let matrix = to_matrix(&args[0]);
    let _by_col = args
        .get(1)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(false);
    let exactly_once = args
        .get(2)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(false);

    if matrix.is_empty() {
        return CellValue::Error(CellError::Calc);
    }

    // Hashable string representation for row deduplication
    let row_keys: Vec<String> = matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| match c {
                    CellValue::Number(n) => format!("N:{n}"),
                    CellValue::InlineString(s) => format!("S:{}", s.to_ascii_uppercase()),
                    CellValue::Boolean(b) => format!("B:{b}"),
                    CellValue::Error(e) => format!("E:{e}"),
                    CellValue::Empty => "EMPTY".to_string(),
                    _ => "OTHER".to_string(),
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();

    if exactly_once {
        let mut counts = std::collections::HashMap::new();
        for k in &row_keys {
            *counts.entry(k.clone()).or_insert(0) += 1;
        }
        let unique_rows: Vec<Vec<CellValue>> = matrix
            .iter()
            .enumerate()
            .filter(|(i, _)| counts.get(&row_keys[*i]) == Some(&1))
            .map(|(_, r)| r.clone())
            .collect();

        if unique_rows.is_empty() {
            CellValue::Error(CellError::Calc)
        } else {
            CellValue::Array(unique_rows)
        }
    } else {
        let mut seen = HashSet::new();
        let mut unique_rows = Vec::new();
        for (i, row) in matrix.iter().enumerate() {
            if seen.insert(row_keys[i].clone()) {
                unique_rows.push(row.clone());
            }
        }
        CellValue::Array(unique_rows)
    }
}

/// Evaluates `SORT(array, [sort_index = 1], [sort_order = 1], [by_col = false])`.
pub fn fn_sort(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let mut matrix = to_matrix(&args[0]);
    if matrix.is_empty() {
        return CellValue::Array(matrix);
    }

    let sort_index = args.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as usize;
    let sort_order = args.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as i32;

    if sort_index < 1 || sort_index > matrix[0].len() {
        return CellValue::Error(CellError::Value);
    }

    let col_idx = sort_index - 1;
    let asc = sort_order >= 0;

    matrix.sort_by(|row_a, row_b| {
        let ord = compare_cell_values(&row_a[col_idx], &row_b[col_idx]);
        if asc {
            ord
        } else {
            ord.reverse()
        }
    });

    CellValue::Array(matrix)
}
