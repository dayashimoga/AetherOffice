//! OpenOfficeRS Excel Engine - Modern Dynamic Array Functions.
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements SEQUENCE, RANDARRAY, CHOOSEROWS, CHOOSECOLS, TOCOL, TOROW, TAKE, DROP,
//! EXPAND, and SORTBY.

use crate::functions::lookup::{compare_cell_values, to_flat_vector, to_matrix};
use crate::grid::{CellError, CellValue};

/// Evaluates `SEQUENCE(rows, [columns = 1], [start = 1], [step = 1])`.
pub fn fn_sequence(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let rows = match args[0].as_f64() {
        Some(r) => {
            if r < 1.0 || r.fract() != 0.0 {
                return CellValue::Error(CellError::Value);
            }
            r as usize
        }
        None => return CellValue::Error(CellError::Value),
    };

    let cols = match args.get(1) {
        Some(v) => match v.as_f64() {
            Some(c) => {
                if c < 1.0 || c.fract() != 0.0 {
                    return CellValue::Error(CellError::Value);
                }
                c as usize
            }
            None => return CellValue::Error(CellError::Value),
        },
        None => 1,
    };

    let start = args.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0);
    let step = args.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0);

    let mut matrix = Vec::with_capacity(rows);
    let mut current = start;

    for _ in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for _ in 0..cols {
            row.push(CellValue::Number(current));
            current += step;
        }
        matrix.push(row);
    }

    CellValue::Array(matrix)
}

/// Simple deterministic SplitMix64 pseudo-random generator state.
#[derive(Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

/// Evaluates `RANDARRAY([rows = 1], [columns = 1], [min = 0], [max = 1], [whole_number = false])`.
pub fn fn_randarray(args: &[CellValue]) -> CellValue {
    let rows = match args.first() {
        Some(v) => match v.as_f64() {
            Some(r) => {
                if r < 1.0 || r.fract() != 0.0 {
                    return CellValue::Error(CellError::Value);
                }
                r as usize
            }
            None => return CellValue::Error(CellError::Value),
        },
        None => 1,
    };

    let cols = match args.get(1) {
        Some(v) => match v.as_f64() {
            Some(c) => {
                if c < 1.0 || c.fract() != 0.0 {
                    return CellValue::Error(CellError::Value);
                }
                c as usize
            }
            None => return CellValue::Error(CellError::Value),
        },
        None => 1,
    };

    let min = args.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let max = args.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0);

    if min > max {
        return CellValue::Error(CellError::Value);
    }

    let whole_number = args
        .get(4)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(false);

    let mut rng = SplitMix64(0x853C49E6748FEA9B);
    let mut matrix = Vec::with_capacity(rows);

    for _ in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for _ in 0..cols {
            let u = rng.next_f64();
            let val = if whole_number {
                let lo = min.ceil();
                let hi = max.floor();
                if lo > hi {
                    return CellValue::Error(CellError::Value);
                }
                let span = hi - lo + 1.0;
                (lo + (u * span).floor()).min(hi)
            } else {
                min + u * (max - min)
            };
            row.push(CellValue::Number(val));
        }
        matrix.push(row);
    }

    CellValue::Array(matrix)
}

/// Evaluates `CHOOSEROWS(array, row_num1, [row_num2, ...])`.
pub fn fn_chooserows(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let total_rows = matrix.len() as i64;
    let mut chosen = Vec::with_capacity(args.len() - 1);

    for arg in &args[1..] {
        let row_idx = match arg.as_f64() {
            Some(v) => v as i64,
            None => return CellValue::Error(CellError::Value),
        };

        if row_idx == 0 {
            return CellValue::Error(CellError::Value);
        }

        let target_idx = if row_idx > 0 {
            row_idx - 1
        } else {
            total_rows + row_idx
        };

        if target_idx < 0 || target_idx >= total_rows {
            return CellValue::Error(CellError::Value);
        }

        chosen.push(matrix[target_idx as usize].clone());
    }

    CellValue::Array(chosen)
}

/// Evaluates `CHOOSECOLS(array, col_num1, [col_num2, ...])`.
pub fn fn_choosecols(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() || matrix[0].is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let total_cols = matrix[0].len() as i64;
    let mut col_indices = Vec::with_capacity(args.len() - 1);

    for arg in &args[1..] {
        let col_idx = match arg.as_f64() {
            Some(v) => v as i64,
            None => return CellValue::Error(CellError::Value),
        };

        if col_idx == 0 {
            return CellValue::Error(CellError::Value);
        }

        let target_idx = if col_idx > 0 {
            col_idx - 1
        } else {
            total_cols + col_idx
        };

        if target_idx < 0 || target_idx >= total_cols {
            return CellValue::Error(CellError::Value);
        }

        col_indices.push(target_idx as usize);
    }

    let mut result = Vec::with_capacity(matrix.len());
    for row in &matrix {
        let mut new_row = Vec::with_capacity(col_indices.len());
        for &idx in &col_indices {
            new_row.push(row[idx].clone());
        }
        result.push(new_row);
    }

    CellValue::Array(result)
}

/// Evaluates `TOCOL(array, [ignore = 0], [scan_by_column = false])`.
pub fn fn_tocol(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() || matrix[0].is_empty() {
        return CellValue::Error(CellError::Calc);
    }

    let ignore = args.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
    let scan_by_column = args
        .get(2)
        .and_then(|v| match v {
            CellValue::Boolean(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            _ => None,
        })
        .unwrap_or(false);

    let _rows = matrix.len();
    let cols = matrix[0].len();
    let mut flat = Vec::new();

    let check_include = |cell: &CellValue| -> bool {
        match ignore {
            1 => !cell.is_empty(),
            2 => !matches!(cell, CellValue::Error(_)),
            3 => !cell.is_empty() && !matches!(cell, CellValue::Error(_)),
            _ => true,
        }
    };

    if scan_by_column {
        for c in 0..cols {
            for row in &matrix {
                let cell = &row[c];
                if check_include(cell) {
                    flat.push(cell.clone());
                }
            }
        }
    } else {
        for row in &matrix {
            for cell in row {
                if check_include(cell) {
                    flat.push(cell.clone());
                }
            }
        }
    }

    if flat.is_empty() {
        CellValue::Error(CellError::Calc)
    } else {
        CellValue::Array(flat.into_iter().map(|item| vec![item]).collect())
    }
}

/// Evaluates `TOROW(array, [ignore = 0], [scan_by_column = false])`.
pub fn fn_torow(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let col_res = fn_tocol(args);
    match col_res {
        CellValue::Array(rows) => {
            let single_row: Vec<CellValue> = rows.into_iter().flat_map(|r| r.into_iter()).collect();
            CellValue::Array(vec![single_row])
        }
        err => err,
    }
}

/// Evaluates `TAKE(array, rows, [columns])`.
pub fn fn_take(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() || matrix[0].is_empty() {
        return CellValue::Error(CellError::Calc);
    }

    let total_rows = matrix.len();
    let total_cols = matrix[0].len();

    let rows_arg = match args[1].as_f64() {
        Some(v) => v as i64,
        None => return CellValue::Error(CellError::Value),
    };

    if rows_arg == 0 {
        return CellValue::Error(CellError::Calc);
    }

    let cols_arg = match args.get(2) {
        Some(v) => match v.as_f64() {
            Some(c) => {
                let c_int = c as i64;
                if c_int == 0 {
                    return CellValue::Error(CellError::Calc);
                }
                Some(c_int)
            }
            None => return CellValue::Error(CellError::Value),
        },
        None => None,
    };

    let (r_start, r_end) = if rows_arg > 0 {
        (0usize, (rows_arg as usize).min(total_rows))
    } else {
        let count = ((-rows_arg) as usize).min(total_rows);
        (total_rows - count, total_rows)
    };

    let (c_start, c_end) = match cols_arg {
        Some(c) if c > 0 => (0usize, (c as usize).min(total_cols)),
        Some(c) => {
            let count = ((-c) as usize).min(total_cols);
            (total_cols - count, total_cols)
        }
        None => (0usize, total_cols),
    };

    let mut result = Vec::with_capacity(r_end - r_start);
    for row in &matrix[r_start..r_end] {
        result.push(row[c_start..c_end].to_vec());
    }

    CellValue::Array(result)
}

/// Evaluates `DROP(array, rows, [columns])`.
pub fn fn_drop(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() || matrix[0].is_empty() {
        return CellValue::Error(CellError::Calc);
    }

    let total_rows = matrix.len();
    let total_cols = matrix[0].len();

    let rows_drop = match args[1].as_f64() {
        Some(v) => v as i64,
        None => return CellValue::Error(CellError::Value),
    };

    let cols_drop = match args.get(2) {
        Some(v) => match v.as_f64() {
            Some(c) => c as i64,
            None => return CellValue::Error(CellError::Value),
        },
        None => 0,
    };

    let (r_start, r_end) = if rows_drop >= 0 {
        let start = rows_drop as usize;
        if start >= total_rows {
            return CellValue::Error(CellError::Calc);
        }
        (start, total_rows)
    } else {
        let count = (-rows_drop) as usize;
        if count >= total_rows {
            return CellValue::Error(CellError::Calc);
        }
        (0, total_rows - count)
    };

    let (c_start, c_end) = if cols_drop >= 0 {
        let start = cols_drop as usize;
        if start >= total_cols {
            return CellValue::Error(CellError::Calc);
        }
        (start, total_cols)
    } else {
        let count = (-cols_drop) as usize;
        if count >= total_cols {
            return CellValue::Error(CellError::Calc);
        }
        (0, total_cols - count)
    };

    let mut result = Vec::with_capacity(r_end - r_start);
    for row in &matrix[r_start..r_end] {
        result.push(row[c_start..c_end].to_vec());
    }

    CellValue::Array(result)
}

/// Evaluates `EXPAND(array, rows, [columns], [pad_with = #N/A])`.
pub fn fn_expand(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() || matrix[0].is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let cur_rows = matrix.len();
    let cur_cols = matrix[0].len();

    let target_rows = match args[1].as_f64() {
        Some(r) => {
            let r_int = r as usize;
            if r_int < cur_rows {
                return CellValue::Error(CellError::Value);
            }
            r_int
        }
        None => return CellValue::Error(CellError::Value),
    };

    let target_cols = match args.get(2) {
        Some(v) => match v.as_f64() {
            Some(c) => {
                let c_int = c as usize;
                if c_int < cur_cols {
                    return CellValue::Error(CellError::Value);
                }
                c_int
            }
            None => return CellValue::Error(CellError::Value),
        },
        None => cur_cols,
    };

    let pad_with = args
        .get(3)
        .cloned()
        .unwrap_or(CellValue::Error(CellError::NA));

    let mut result = Vec::with_capacity(target_rows);

    for src_row in &matrix {
        let mut row = Vec::with_capacity(target_cols);
        for item in src_row.iter().take(target_cols) {
            row.push(item.clone());
        }
        while row.len() < target_cols {
            row.push(pad_with.clone());
        }
        result.push(row);
    }
    while result.len() < target_rows {
        result.push(vec![pad_with.clone(); target_cols]);
    }

    CellValue::Array(result)
}

/// Evaluates `SORTBY(array, by_array1, [order1 = 1], [by_array2, order2 = 1], ...)`.
pub fn fn_sortby(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let matrix = to_matrix(&args[0]);
    if matrix.is_empty() {
        return CellValue::Array(matrix);
    }

    let num_rows = matrix.len();

    // Parse sort keys: pairs of (by_vector, is_asc)
    let mut sort_keys: Vec<(Vec<CellValue>, bool)> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let by_vec = to_flat_vector(&args[i]);
        if by_vec.len() != num_rows {
            return CellValue::Error(CellError::Value);
        }

        let is_asc = if i + 1 < args.len() {
            let order_val = args[i + 1].as_f64().unwrap_or(1.0);
            i += 2;
            order_val >= 0.0
        } else {
            i += 1;
            true
        };

        sort_keys.push((by_vec, is_asc));
    }

    let mut row_indices: Vec<usize> = (0..num_rows).collect();

    row_indices.sort_by(|&a, &b| {
        for (key_vec, is_asc) in &sort_keys {
            let ord = compare_cell_values(&key_vec[a], &key_vec[b]);
            if ord != std::cmp::Ordering::Equal {
                return if *is_asc { ord } else { ord.reverse() };
            }
        }
        std::cmp::Ordering::Equal
    });

    let mut sorted_matrix = Vec::with_capacity(num_rows);
    for idx in row_indices {
        sorted_matrix.push(matrix[idx].clone());
    }

    CellValue::Array(sorted_matrix)
}
