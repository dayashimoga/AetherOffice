//! OpenOfficeRS Excel Engine - Math & Statistical Functions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements SUMIFS, COUNTIFS, AVERAGEIFS, SUBTOTAL, AGGREGATE, ROUND, ROUNDUP,
//! ROUNDDOWN, MROUND, STDEV.S, and PERCENTILE.INC.

use crate::grid::{CellError, CellValue};

/// Evaluates wildcard matching (supports '*' for 0 or more characters, '?' for 1 character, and '~' escaping).
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p_chars: Vec<char> = pattern.chars().collect();
    let t_chars: Vec<char> = text.chars().collect();

    let mut dp = vec![vec![false; t_chars.len() + 1]; p_chars.len() + 1];
    dp[0][0] = true;

    // Build simplified pattern tokens dealing with ~ escapes
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < p_chars.len() {
        if p_chars[i] == '~' && i + 1 < p_chars.len() {
            tokens.push((p_chars[i + 1], false));
            i += 2;
        } else if p_chars[i] == '*' {
            tokens.push(('*', true));
            i += 1;
        } else if p_chars[i] == '?' {
            tokens.push(('?', true));
            i += 1;
        } else {
            tokens.push((p_chars[i], false));
            i += 1;
        }
    }

    let mut dp = vec![vec![false; t_chars.len() + 1]; tokens.len() + 1];
    dp[0][0] = true;

    for pi in 1..=tokens.len() {
        let (ch, is_wild) = tokens[pi - 1];
        if is_wild && ch == '*' {
            dp[pi][0] = dp[pi - 1][0];
        }
        for ti in 1..=t_chars.len() {
            let t_char = t_chars[ti - 1];
            if is_wild && ch == '*' {
                dp[pi][ti] = dp[pi - 1][ti] || dp[pi][ti - 1];
            } else if is_wild && ch == '?' {
                dp[pi][ti] = dp[pi - 1][ti - 1];
            } else {
                dp[pi][ti] = dp[pi - 1][ti - 1] && ch.eq_ignore_ascii_case(&t_char);
            }
        }
    }

    dp[tokens.len()][t_chars.len()]
}

/// Criterion comparator operator
#[derive(Debug, Clone, Copy, PartialEq)]
enum CriteriaOp {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
}

/// Parsed criteria structure
#[derive(Debug, Clone)]
enum Criteria {
    Number(CriteriaOp, f64),
    String(CriteriaOp, String),
    Boolean(bool),
}

fn parse_criteria(val: &CellValue) -> Criteria {
    match val {
        CellValue::Number(n) => Criteria::Number(CriteriaOp::Equal, *n),
        CellValue::Boolean(b) => Criteria::Boolean(*b),
        CellValue::InlineString(s) => {
            let trimmed = s.trim();
            if let Some(rest) = trimmed.strip_prefix(">=") {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::GreaterEqual, num)
                } else {
                    Criteria::String(CriteriaOp::GreaterEqual, rest.trim().to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix("<=") {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::LessEqual, num)
                } else {
                    Criteria::String(CriteriaOp::LessEqual, rest.trim().to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix("<>") {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::NotEqual, num)
                } else {
                    Criteria::String(CriteriaOp::NotEqual, rest.trim().to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix('>') {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::GreaterThan, num)
                } else {
                    Criteria::String(CriteriaOp::GreaterThan, rest.trim().to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix('<') {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::LessThan, num)
                } else {
                    Criteria::String(CriteriaOp::LessThan, rest.trim().to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix('=') {
                if let Ok(num) = rest.trim().parse::<f64>() {
                    Criteria::Number(CriteriaOp::Equal, num)
                } else {
                    Criteria::String(CriteriaOp::Equal, rest.trim().to_string())
                }
            } else if let Ok(num) = trimmed.parse::<f64>() {
                Criteria::Number(CriteriaOp::Equal, num)
            } else {
                Criteria::String(CriteriaOp::Equal, trimmed.to_string())
            }
        }
        _ => Criteria::String(CriteriaOp::Equal, String::new()),
    }
}

fn matches_criteria(cell: &CellValue, crit: &Criteria) -> bool {
    match crit {
        Criteria::Number(op, target) => {
            if let Some(val) = cell.as_f64() {
                match op {
                    CriteriaOp::Equal => (val - target).abs() < 1e-12,
                    CriteriaOp::NotEqual => (val - target).abs() >= 1e-12,
                    CriteriaOp::GreaterThan => val > *target,
                    CriteriaOp::GreaterEqual => val >= *target,
                    CriteriaOp::LessThan => val < *target,
                    CriteriaOp::LessEqual => val <= *target,
                }
            } else {
                false
            }
        }
        Criteria::String(op, pattern) => {
            let cell_str = match cell {
                CellValue::InlineString(s) => s.as_str(),
                CellValue::Number(_n) => return false,
                CellValue::Boolean(b) => {
                    if *b {
                        "TRUE"
                    } else {
                        "FALSE"
                    }
                }
                CellValue::Empty => "",
                _ => return false,
            };

            match op {
                CriteriaOp::Equal => wildcard_match(pattern, cell_str),
                CriteriaOp::NotEqual => !wildcard_match(pattern, cell_str),
                CriteriaOp::GreaterThan => {
                    cell_str.to_ascii_uppercase() > pattern.to_ascii_uppercase()
                }
                CriteriaOp::GreaterEqual => {
                    cell_str.to_ascii_uppercase() >= pattern.to_ascii_uppercase()
                }
                CriteriaOp::LessThan => {
                    cell_str.to_ascii_uppercase() < pattern.to_ascii_uppercase()
                }
                CriteriaOp::LessEqual => {
                    cell_str.to_ascii_uppercase() <= pattern.to_ascii_uppercase()
                }
            }
        }
        Criteria::Boolean(b) => match cell {
            CellValue::Boolean(cb) => cb == b,
            _ => false,
        },
    }
}

/// Flattens a CellValue array or scalar into a flat vector.
pub fn flatten_cell_values(val: &CellValue) -> Vec<CellValue> {
    match val {
        CellValue::Array(rows) => {
            let mut res = Vec::new();
            for r in rows {
                for c in r {
                    res.push(c.clone());
                }
            }
            res
        }
        other => vec![other.clone()],
    }
}

/// Evaluates `SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])`.
pub fn fn_sumifs(args: &[CellValue]) -> CellValue {
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return CellValue::Error(CellError::Value);
    }

    let sum_values = flatten_cell_values(&args[0]);
    let n = sum_values.len();

    let mut criteria_pairs = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let range_vals = flatten_cell_values(&args[i]);
        if range_vals.len() != n {
            return CellValue::Error(CellError::Value);
        }
        let crit = parse_criteria(&args[i + 1]);
        criteria_pairs.push((range_vals, crit));
        i += 2;
    }

    let mut total = 0.0;
    for idx in 0..n {
        let mut all_match = true;
        for (range_vals, crit) in &criteria_pairs {
            if !matches_criteria(&range_vals[idx], crit) {
                all_match = false;
                break;
            }
        }
        if all_match {
            if let Some(v) = sum_values[idx].as_f64() {
                total += v;
            }
        }
    }

    CellValue::Number(total)
}

/// Evaluates `COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2, ...])`.
pub fn fn_countifs(args: &[CellValue]) -> CellValue {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return CellValue::Error(CellError::Value);
    }

    let first_range = flatten_cell_values(&args[0]);
    let n = first_range.len();

    let mut criteria_pairs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let range_vals = flatten_cell_values(&args[i]);
        if range_vals.len() != n {
            return CellValue::Error(CellError::Value);
        }
        let crit = parse_criteria(&args[i + 1]);
        criteria_pairs.push((range_vals, crit));
        i += 2;
    }

    let mut count = 0i64;
    for idx in 0..n {
        let mut all_match = true;
        for (range_vals, crit) in &criteria_pairs {
            if !matches_criteria(&range_vals[idx], crit) {
                all_match = false;
                break;
            }
        }
        if all_match {
            count += 1;
        }
    }

    CellValue::Number(count as f64)
}

/// Evaluates `AVERAGEIFS(average_range, criteria_range1, criteria1, ...)`.
pub fn fn_averageifs(args: &[CellValue]) -> CellValue {
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return CellValue::Error(CellError::Value);
    }

    let avg_values = flatten_cell_values(&args[0]);
    let n = avg_values.len();

    let mut criteria_pairs = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let range_vals = flatten_cell_values(&args[i]);
        if range_vals.len() != n {
            return CellValue::Error(CellError::Value);
        }
        let crit = parse_criteria(&args[i + 1]);
        criteria_pairs.push((range_vals, crit));
        i += 2;
    }

    let mut sum = 0.0;
    let mut count = 0i64;
    for idx in 0..n {
        let mut all_match = true;
        for (range_vals, crit) in &criteria_pairs {
            if !matches_criteria(&range_vals[idx], crit) {
                all_match = false;
                break;
            }
        }
        if all_match {
            if let Some(v) = avg_values[idx].as_f64() {
                sum += v;
                count += 1;
            }
        }
    }

    if count == 0 {
        CellValue::Error(CellError::Div0)
    } else {
        CellValue::Number(sum / count as f64)
    }
}

/// Evaluates `SUBTOTAL(function_num, ref1, [ref2, ...])`.
pub fn fn_subtotal(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let func_code = match args[0].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    // Extract all numbers, skipping nested subtotals (already skipped by caller if tagged)
    let mut numbers = Vec::new();
    for arg in &args[1..] {
        for cell in flatten_cell_values(arg) {
            if let Some(n) = cell.as_f64() {
                numbers.push(n);
            }
        }
    }

    let op = func_code % 100;
    match op {
        1 => {
            // AVERAGE
            if numbers.is_empty() {
                CellValue::Error(CellError::Div0)
            } else {
                let sum: f64 = numbers.iter().sum();
                CellValue::Number(sum / numbers.len() as f64)
            }
        }
        2 => CellValue::Number(numbers.len() as f64), // COUNT
        3 => {
            // COUNTA
            let mut count = 0;
            for arg in &args[1..] {
                for cell in flatten_cell_values(arg) {
                    if !cell.is_empty() {
                        count += 1;
                    }
                }
            }
            CellValue::Number(count as f64)
        }
        4 => {
            // MAX
            if numbers.is_empty() {
                CellValue::Number(0.0)
            } else {
                let max = numbers.iter().fold(f64::MIN, |a, &b| a.max(b));
                CellValue::Number(max)
            }
        }
        5 => {
            // MIN
            if numbers.is_empty() {
                CellValue::Number(0.0)
            } else {
                let min = numbers.iter().fold(f64::MAX, |a, &b| a.min(b));
                CellValue::Number(min)
            }
        }
        6 => {
            // PRODUCT
            let prod = numbers.iter().fold(1.0, |a, &b| a * b);
            CellValue::Number(prod)
        }
        7 => fn_stdev_s(&[CellValue::Array(vec![numbers
            .into_iter()
            .map(CellValue::Number)
            .collect()])]), // STDEV.S
        9 => {
            // SUM
            let sum: f64 = numbers.iter().sum();
            CellValue::Number(sum)
        }
        _ => CellValue::Error(CellError::Value),
    }
}

/// Evaluates `AGGREGATE(function_num, options, ref1, [k])`.
pub fn fn_aggregate(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let func_num = match args[0].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };
    let options = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let ignore_errors = matches!(options, 2 | 3 | 6 | 7);

    let mut numbers = Vec::new();
    for cell in flatten_cell_values(&args[2]) {
        match cell {
            CellValue::Number(n) => numbers.push(n),
            CellValue::Error(_) if !ignore_errors => {
                return cell;
            }
            _ => {}
        }
    }

    match func_num {
        1 => {
            // AVERAGE
            if numbers.is_empty() {
                CellValue::Error(CellError::Div0)
            } else {
                let sum: f64 = numbers.iter().sum();
                CellValue::Number(sum / numbers.len() as f64)
            }
        }
        2 => CellValue::Number(numbers.len() as f64), // COUNT
        4 => {
            // MAX
            if numbers.is_empty() {
                CellValue::Number(0.0)
            } else {
                let max = numbers.iter().fold(f64::MIN, |a, &b| a.max(b));
                CellValue::Number(max)
            }
        }
        5 => {
            // MIN
            if numbers.is_empty() {
                CellValue::Number(0.0)
            } else {
                let min = numbers.iter().fold(f64::MAX, |a, &b| a.min(b));
                CellValue::Number(min)
            }
        }
        9 => {
            // SUM
            let sum: f64 = numbers.iter().sum();
            CellValue::Number(sum)
        }
        14 => {
            // LARGE (requires k in args[3])
            let k = match args.get(3).and_then(|v| v.as_f64()) {
                Some(v) => v as usize,
                None => return CellValue::Error(CellError::Value),
            };
            if k < 1 || k > numbers.len() {
                return CellValue::Error(CellError::Num);
            }
            numbers.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            CellValue::Number(numbers[k - 1])
        }
        15 => {
            // SMALL (requires k in args[3])
            let k = match args.get(3).and_then(|v| v.as_f64()) {
                Some(v) => v as usize,
                None => return CellValue::Error(CellError::Value),
            };
            if k < 1 || k > numbers.len() {
                return CellValue::Error(CellError::Num);
            }
            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            CellValue::Number(numbers[k - 1])
        }
        16 => {
            // PERCENTILE.INC
            let k_val = match args.get(3) {
                Some(v) => v.clone(),
                None => return CellValue::Error(CellError::Value),
            };
            fn_percentile_inc(&[
                CellValue::Array(vec![numbers.into_iter().map(CellValue::Number).collect()]),
                k_val,
            ])
        }
        _ => CellValue::Error(CellError::Value),
    }
}

/// Evaluates `ROUND(number, num_digits)` with exact Excel 1e-10 epsilon.
pub fn fn_round(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let n = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let d = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let factor = 10f64.powi(d);
    let scaled = n * factor;
    let eps = 1e-10;
    let rounded = if scaled >= 0.0 {
        (scaled + eps).round()
    } else {
        (scaled - eps).round()
    };
    CellValue::Number(rounded / factor)
}

/// Evaluates `ROUNDUP(number, num_digits)` (rounds away from zero).
pub fn fn_roundup(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let n = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let d = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let factor = 10f64.powi(d);
    let scaled = n * factor;
    let rounded = if scaled >= 0.0 {
        scaled.ceil()
    } else {
        scaled.floor()
    };
    CellValue::Number(rounded / factor)
}

/// Evaluates `ROUNDDOWN(number, num_digits)` (rounds toward zero / truncation).
pub fn fn_rounddown(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let n = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let d = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let factor = 10f64.powi(d);
    let scaled = n * factor;
    let rounded = scaled.trunc();
    CellValue::Number(rounded / factor)
}

/// Evaluates `MROUND(number, multiple)`.
pub fn fn_mround(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let n = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let m = match args[1].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };

    if m == 0.0 {
        return CellValue::Number(0.0);
    }

    if (n > 0.0 && m < 0.0) || (n < 0.0 && m > 0.0) {
        return CellValue::Error(CellError::Num);
    }

    let rounded = (n / m).round();
    CellValue::Number(rounded * m)
}

/// Evaluates `STDEV.S(number1, [number2, ...])` (sample standard deviation).
pub fn fn_stdev_s(args: &[CellValue]) -> CellValue {
    let mut values = Vec::new();
    for arg in args {
        for cell in flatten_cell_values(arg) {
            if let Some(n) = cell.as_f64() {
                values.push(n);
            }
        }
    }

    if values.len() < 2 {
        return CellValue::Error(CellError::Div0);
    }

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);

    CellValue::Number(variance.sqrt())
}

/// Evaluates `PERCENTILE.INC(array, k)`.
pub fn fn_percentile_inc(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let mut values = Vec::new();
    for cell in flatten_cell_values(&args[0]) {
        if let Some(n) = cell.as_f64() {
            values.push(n);
        }
    }

    let k = match args[1].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };

    if !(0.0..=1.0).contains(&k) || values.is_empty() {
        return CellValue::Error(CellError::Num);
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    if n == 1 {
        return CellValue::Number(values[0]);
    }

    let r = (n - 1) as f64 * k;
    let idx = r.floor() as usize;
    let frac = r - idx as f64;

    if idx >= n - 1 {
        CellValue::Number(values[n - 1])
    } else {
        let val = values[idx] + frac * (values[idx + 1] - values[idx]);
        CellValue::Number(val)
    }
}
