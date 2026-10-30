//! OpenOfficeRS Excel Engine - Complete Standard & Dynamic Functions Registry.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, Clean-Room Standards).
//! Dispatches over 40+ Excel functions across Math, Statistical, Financial, Date/Time,
//! Lookup & Reference, Logical, and Modern Dynamic Lambda/Let.

pub mod datetime;
pub mod financial;
pub mod lambda;
pub mod lookup;
pub mod math_stat;

pub use datetime::*;
pub use financial::*;
pub use lambda::*;
pub use lookup::*;
pub use math_stat::*;

use crate::formula::ast::{AstNode, Reference};
use crate::grid::{CellError, CellValue};
use ooxml_opc::error::Result;

/// Dispatches formula function evaluation, supporting short-circuiting forms (IF, LET, LAMBDA, MAP, REDUCE, SCAN)
/// as well as standard eagerly-evaluated functions.
pub fn dispatch_function<F>(
    name: &str,
    args: &[AstNode],
    ctx: &EvalContext,
    eval_fn: &F,
) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    let clean_name = name
        .trim_start_matches("_xlfn.")
        .trim_start_matches("_xlws.")
        .to_ascii_uppercase();

    // 1. Special-form functions requiring unevaluated AST arguments
    match clean_name.as_str() {
        "IF" => {
            if args.is_empty() {
                return Ok(CellValue::Error(CellError::Value));
            }
            let cond_val = eval_fn(&args[0], ctx)?;
            let is_true = match cond_val {
                CellValue::Boolean(b) => b,
                CellValue::Number(n) => n != 0.0,
                _ => false,
            };
            if is_true {
                if args.len() > 1 {
                    eval_fn(&args[1], ctx)
                } else {
                    Ok(CellValue::Boolean(true))
                }
            } else if args.len() > 2 {
                eval_fn(&args[2], ctx)
            } else {
                Ok(CellValue::Boolean(false))
            }
        }
        "LET" => eval_let(args, ctx, eval_fn),
        "MAP" => eval_map(args, ctx, eval_fn),
        "REDUCE" => eval_reduce(args, ctx, eval_fn),
        "SCAN" => eval_scan(args, ctx, eval_fn),
        "LAMBDA" => {
            // Return unevaluated lambda definition
            Ok(CellValue::InlineString(format!("LAMBDA:{}", args.len())))
        }
        "OFFSET" => {
            if let Some(grid) = ctx.grid {
                if args.is_empty() {
                    return Ok(CellValue::Error(CellError::Value));
                }
                // First argument must be a cell reference to get origin
                let (origin_r, origin_c) = match &args[0] {
                    AstNode::Reference(Reference::Cell { cell, .. }) => (cell.row, cell.col),
                    _ => (0, 0),
                };
                let mut evaluated_args = Vec::new();
                for arg in &args[1..] {
                    evaluated_args.push(eval_fn(arg, ctx)?);
                }
                Ok(fn_offset(grid, origin_r, origin_c, &evaluated_args))
            } else {
                Ok(CellValue::Error(CellError::Ref))
            }
        }
        "INDIRECT" => {
            if let Some(grid) = ctx.grid {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(eval_fn(arg, ctx)?);
                }
                Ok(fn_indirect(grid, &evaluated_args))
            } else {
                Ok(CellValue::Error(CellError::Ref))
            }
        }
        _ => {
            // 2. Standard eager functions: evaluate arguments first
            let mut eval_args = Vec::new();
            for arg in args {
                // If argument is a range reference, evaluate it to a full matrix
                if let AstNode::Reference(Reference::Range { range, .. }) = arg {
                    if let Some(grid) = ctx.grid {
                        let r1 = range.start.row.min(range.end.row);
                        let r2 = range.start.row.max(range.end.row);
                        let c1 = range.start.col.min(range.end.col);
                        let c2 = range.start.col.max(range.end.col);
                        let mut matrix = Vec::new();
                        for r in r1..=r2 {
                            let mut row = Vec::new();
                            for c in c1..=c2 {
                                row.push(grid.get_value(r, c));
                            }
                            matrix.push(row);
                        }
                        eval_args.push(CellValue::Array(matrix));
                        continue;
                    }
                }

                let val = eval_fn(arg, ctx)?;
                eval_args.push(val);
            }

            let eager_res = dispatch_eager_function(&clean_name, &eval_args, ctx.date1904)?;
            if let CellValue::Error(CellError::Name) = eager_res {
                if let Some(ref handler) = ctx.udf_handler {
                    match handler(&clean_name, &eval_args) {
                        Ok(val) => Ok(val),
                        Err(_) => Ok(CellValue::Error(CellError::Value)),
                    }
                } else {
                    Ok(CellValue::Error(CellError::Name))
                }
            } else {
                Ok(eager_res)
            }
        }
    }
}

/// Dispatches standard eager functions taking evaluated arguments.
fn dispatch_eager_function(name: &str, args: &[CellValue], date1904: bool) -> Result<CellValue> {
    match name {
        // --- Math & Statistical ---
        "SUM" => {
            let mut sum = 0.0;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if let Some(n) = cell.as_f64() {
                        sum += n;
                    }
                }
            }
            Ok(CellValue::Number(sum))
        }
        "AVERAGE" => {
            let mut sum = 0.0;
            let mut count = 0;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if let Some(n) = cell.as_f64() {
                        sum += n;
                        count += 1;
                    }
                }
            }
            if count == 0 {
                Ok(CellValue::Error(CellError::Div0))
            } else {
                Ok(CellValue::Number(sum / count as f64))
            }
        }
        "MIN" => {
            let mut min = f64::MAX;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if let Some(n) = cell.as_f64() {
                        if n < min {
                            min = n;
                        }
                    }
                }
            }
            Ok(CellValue::Number(if min == f64::MAX { 0.0 } else { min }))
        }
        "MAX" => {
            let mut max = f64::MIN;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if let Some(n) = cell.as_f64() {
                        if n > max {
                            max = n;
                        }
                    }
                }
            }
            Ok(CellValue::Number(if max == f64::MIN { 0.0 } else { max }))
        }
        "COUNT" => {
            let mut count = 0;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if matches!(cell, CellValue::Number(_)) {
                        count += 1;
                    }
                }
            }
            Ok(CellValue::Number(count as f64))
        }
        "COUNTA" => {
            let mut count = 0;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    if !cell.is_empty() {
                        count += 1;
                    }
                }
            }
            Ok(CellValue::Number(count as f64))
        }
        "ABS" => {
            if let Some(first) = args.first() {
                match first.as_f64() {
                    Some(n) => Ok(CellValue::Number(n.abs())),
                    None => Ok(CellValue::Error(CellError::Value)),
                }
            } else {
                Ok(CellValue::Error(CellError::Value))
            }
        }
        "ROUND" => Ok(fn_round(args)),
        "ROUNDUP" => Ok(fn_roundup(args)),
        "ROUNDDOWN" => Ok(fn_rounddown(args)),
        "MROUND" => Ok(fn_mround(args)),
        "STDEV.S" | "STDEVS" => Ok(fn_stdev_s(args)),
        "PERCENTILE.INC" | "PERCENTILE" => Ok(fn_percentile_inc(args)),
        "SUMIFS" => Ok(fn_sumifs(args)),
        "COUNTIFS" => Ok(fn_countifs(args)),
        "AVERAGEIFS" => Ok(fn_averageifs(args)),
        "SUBTOTAL" => Ok(fn_subtotal(args)),
        "AGGREGATE" => Ok(fn_aggregate(args)),

        // --- Financial ---
        "PMT" => Ok(fn_pmt(args)),
        "IPMT" => Ok(fn_ipmt(args)),
        "PPMT" => Ok(fn_ppmt(args)),
        "NPV" => Ok(fn_npv(args)),
        "XNPV" => Ok(fn_xnpv(args)),
        "IRR" => Ok(fn_irr(args)),
        "XIRR" => Ok(fn_xirr(args)),

        // --- Date & Time ---
        "DATE" => Ok(fn_date(args, date1904)),
        "YEAR" => Ok(fn_year(args, date1904)),
        "MONTH" => Ok(fn_month(args, date1904)),
        "DAY" => Ok(fn_day(args, date1904)),
        "EDATE" => Ok(fn_edate(args, date1904)),
        "EOMONTH" => Ok(fn_eomonth(args, date1904)),
        "WORKDAY.INTL" | "WORKDAYINTL" => Ok(fn_workday_intl(args, date1904)),
        "NETWORKDAYS.INTL" | "NETWORKDAYSINTL" => Ok(fn_networkdays_intl(args, date1904)),

        // --- Lookup & Reference ---
        "XLOOKUP" => Ok(fn_xlookup(args)),
        "XMATCH" => Ok(fn_xmatch(args)),
        "VLOOKUP" => Ok(fn_vlookup(args)),
        "HLOOKUP" => Ok(fn_hlookup(args)),
        "INDEX" => Ok(fn_index(args)),
        "MATCH" => Ok(fn_match(args)),
        "FILTER" => Ok(fn_filter(args)),
        "UNIQUE" => Ok(fn_unique(args)),
        "SORT" => Ok(fn_sort(args)),

        // --- Logical ---
        "AND" => {
            let mut res = true;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    match cell {
                        CellValue::Boolean(b) => {
                            if !b {
                                res = false;
                            }
                        }
                        CellValue::Number(n) => {
                            if n == 0.0 {
                                res = false;
                            }
                        }
                        CellValue::Error(e) => return Ok(CellValue::Error(e)),
                        _ => {}
                    }
                }
            }
            Ok(CellValue::Boolean(res))
        }
        "OR" => {
            let mut res = false;
            for arg in args {
                for cell in flatten_cell_values(arg) {
                    match cell {
                        CellValue::Boolean(b) => {
                            if b {
                                res = true;
                            }
                        }
                        CellValue::Number(n) => {
                            if n != 0.0 {
                                res = true;
                            }
                        }
                        CellValue::Error(e) => return Ok(CellValue::Error(e)),
                        _ => {}
                    }
                }
            }
            Ok(CellValue::Boolean(res))
        }
        "NOT" => {
            if let Some(first) = args.first() {
                match first {
                    CellValue::Boolean(b) => Ok(CellValue::Boolean(!b)),
                    CellValue::Number(n) => Ok(CellValue::Boolean(*n == 0.0)),
                    _ => Ok(CellValue::Error(CellError::Value)),
                }
            } else {
                Ok(CellValue::Error(CellError::Value))
            }
        }

        _ => Ok(CellValue::Error(CellError::Name)),
    }
}
