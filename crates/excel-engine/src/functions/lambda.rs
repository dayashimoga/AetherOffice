//! OpenOfficeRS Excel Engine - Modern Dynamic Lambda & Let Functions.
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements LET, LAMBDA closures, MAP, REDUCE, SCAN, MAKEARRAY, BYROW, BYCOL, and ISOMITTED.

use crate::formula::ast::{AstNode, Literal, Reference};
use crate::grid::{CellError, CellValue};
use ooxml_opc::error::Result;
use std::collections::HashMap;

/// UDF handler callback signature for dynamic User-Defined Functions (e.g. VBA macros).
pub type UdfHandler = std::sync::Arc<dyn Fn(&str, &[CellValue]) -> Result<CellValue> + Send + Sync>;

/// Evaluation context carrying workbook state and local lexical scope bindings.
#[derive(Clone, Default)]
pub struct EvalContext<'a> {
    pub grid: Option<&'a crate::grid::CompressedSparsePageGrid>,
    pub date1904: bool,
    pub env: HashMap<String, CellValue>,
    pub lambdas: HashMap<String, (Vec<String>, AstNode)>,
    pub udf_handler: Option<UdfHandler>,
}

impl<'a> EvalContext<'a> {
    pub fn new(grid: Option<&'a crate::grid::CompressedSparsePageGrid>, date1904: bool) -> Self {
        Self {
            grid,
            date1904,
            env: HashMap::new(),
            lambdas: HashMap::new(),
            udf_handler: None,
        }
    }

    pub fn with_env(&self, new_bindings: HashMap<String, CellValue>) -> Self {
        let mut merged = self.env.clone();
        for (k, v) in new_bindings {
            merged.insert(k, v);
        }
        Self {
            grid: self.grid,
            date1904: self.date1904,
            env: merged,
            lambdas: self.lambdas.clone(),
            udf_handler: self.udf_handler.clone(),
        }
    }

    pub fn with_lambda(&self, name: String, params: Vec<String>, body: AstNode) -> Self {
        let mut merged = self.lambdas.clone();
        merged.insert(name.to_ascii_uppercase(), (params, body));
        Self {
            grid: self.grid,
            date1904: self.date1904,
            env: self.env.clone(),
            lambdas: merged,
            udf_handler: self.udf_handler.clone(),
        }
    }

    pub fn with_udf_handler(mut self, handler: UdfHandler) -> Self {
        self.udf_handler = Some(handler);
        self
    }
}

/// Extracts a variable identifier name from an AstNode (handles Named reference or String literal).
pub fn extract_identifier_name(node: &AstNode) -> Option<String> {
    match node {
        AstNode::Reference(Reference::Named(name)) => Some(name.clone()),
        AstNode::Literal(Literal::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Evaluates `LET(name1, val1, [name2, val2, ...], calculation)`.
pub fn eval_let<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return Ok(CellValue::Error(CellError::Value));
    }

    let mut local_ctx = ctx.clone();
    let num_pairs = (args.len() - 1) / 2;

    for i in 0..num_pairs {
        let name_node = &args[2 * i];
        let val_node = &args[2 * i + 1];

        let var_name = match extract_identifier_name(name_node) {
            Some(n) => n.to_ascii_uppercase(),
            None => return Ok(CellValue::Error(CellError::Value)),
        };

        // If val_node is a LAMBDA call, extract parameters and body to register as closure
        if let AstNode::FunctionCall { name, args } = val_node {
            if name
                .trim_start_matches("_xlfn.")
                .trim_start_matches("_xlws.")
                .eq_ignore_ascii_case("LAMBDA")
                && !args.is_empty()
            {
                let param_count = args.len() - 1;
                let body = args[args.len() - 1].clone();
                let mut params = Vec::with_capacity(param_count);
                for arg in args.iter().take(param_count) {
                    if let Some(param_name) = extract_identifier_name(arg) {
                        params.push(param_name.to_ascii_uppercase());
                    }
                }
                local_ctx.lambdas.insert(var_name.clone(), (params, body));
                local_ctx
                    .env
                    .insert(var_name, CellValue::InlineString("LAMBDA".to_string()));
                continue;
            }
        }

        let val = eval_fn(val_node, &local_ctx)?;
        local_ctx.env.insert(var_name, val);
    }

    // Evaluate final calculation
    let final_calc = &args[args.len() - 1];
    eval_fn(final_calc, &local_ctx)
}

/// Evaluates a user-defined LAMBDA invocation: `LAMBDA(param1, ..., body)(arg1, ...)`.
pub fn invoke_lambda<F>(
    lambda_node: &AstNode,
    passed_args: &[CellValue],
    ctx: &EvalContext,
    eval_fn: &F,
) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    match lambda_node {
        AstNode::Parenthesized(inner) => invoke_lambda(inner, passed_args, ctx, eval_fn),
        AstNode::Reference(Reference::Named(name)) => {
            if let Some((params, body)) = ctx.lambdas.get(&name.to_ascii_uppercase()) {
                let mut call_env = HashMap::new();
                for (i, param_name) in params.iter().enumerate() {
                    let arg_val = passed_args.get(i).cloned().unwrap_or(CellValue::Empty);
                    call_env.insert(param_name.clone(), arg_val);
                }
                let call_ctx = ctx.with_env(call_env);
                eval_fn(body, &call_ctx)
            } else {
                Ok(CellValue::Error(CellError::Name))
            }
        }
        AstNode::FunctionCall { name, args }
            if name
                .trim_start_matches("_xlfn.")
                .trim_start_matches("_xlws.")
                .eq_ignore_ascii_case("LAMBDA") =>
        {
            if args.is_empty() {
                return Ok(CellValue::Error(CellError::Value));
            }
            let param_count = args.len() - 1;
            let body = &args[args.len() - 1];

            let mut call_env = HashMap::new();
            for (i, arg) in args.iter().enumerate().take(param_count) {
                if let Some(param_name) = extract_identifier_name(arg) {
                    let arg_val = passed_args.get(i).cloned().unwrap_or(CellValue::Empty);
                    call_env.insert(param_name.to_ascii_uppercase(), arg_val);
                }
            }

            let call_ctx = ctx.with_env(call_env);
            eval_fn(body, &call_ctx)
        }
        _ => Ok(CellValue::Error(CellError::Value)),
    }
}

/// Evaluates `MAP(array1, [array2, ...], lambda)` applying a lambda to elements.
pub fn eval_map<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 2 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let lambda_node = &args[args.len() - 1];
    let arr_count = args.len() - 1;

    let mut matrices = Vec::with_capacity(arr_count);
    for arg in &args[0..arr_count] {
        let val = eval_fn(arg, ctx)?;
        matrices.push(crate::functions::lookup::to_matrix(&val));
    }

    let rows = matrices[0].len();
    let cols = if rows > 0 { matrices[0][0].len() } else { 0 };

    for m in &matrices[1..] {
        if m.len() != rows || (rows > 0 && m[0].len() != cols) {
            return Ok(CellValue::Error(CellError::Value));
        }
    }

    let mut out_matrix = Vec::with_capacity(rows);

    for r in 0..rows {
        let mut out_row = Vec::with_capacity(cols);
        for c in 0..cols {
            let mut element_args = Vec::with_capacity(arr_count);
            for m in &matrices {
                element_args.push(m[r][c].clone());
            }
            let res = invoke_lambda(lambda_node, &element_args, ctx, eval_fn)?;
            out_row.push(res);
        }
        out_matrix.push(out_row);
    }

    Ok(CellValue::Array(out_matrix))
}

/// Evaluates `REDUCE(initial_value, array, lambda)`.
pub fn eval_reduce<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 3 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let mut accumulator = eval_fn(&args[0], ctx)?;
    let arr_val = eval_fn(&args[1], ctx)?;
    let lambda_node = &args[2];

    let flat_items = crate::functions::lookup::to_flat_vector(&arr_val);

    for item in flat_items {
        accumulator = invoke_lambda(lambda_node, &[accumulator, item], ctx, eval_fn)?;
    }

    Ok(accumulator)
}

/// Evaluates `SCAN(initial_value, array, lambda)` returning intermediate accumulation states.
pub fn eval_scan<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 3 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let mut accumulator = eval_fn(&args[0], ctx)?;
    let arr_val = eval_fn(&args[1], ctx)?;
    let lambda_node = &args[2];

    let matrix = crate::functions::lookup::to_matrix(&arr_val);
    let mut out_matrix = Vec::new();

    for row in matrix {
        let mut out_row = Vec::new();
        for cell in row {
            accumulator = invoke_lambda(lambda_node, &[accumulator, cell], ctx, eval_fn)?;
            out_row.push(accumulator.clone());
        }
        out_matrix.push(out_row);
    }

    Ok(CellValue::Array(out_matrix))
}

/// Evaluates `MAKEARRAY(rows, cols, lambda)`.
pub fn eval_makearray<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 3 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let rows_val = eval_fn(&args[0], ctx)?;
    let cols_val = eval_fn(&args[1], ctx)?;
    let lambda_node = &args[2];

    let rows = match rows_val.as_f64() {
        Some(r) => {
            if r < 1.0 || r.fract() != 0.0 {
                return Ok(CellValue::Error(CellError::Value));
            }
            r as usize
        }
        None => return Ok(CellValue::Error(CellError::Value)),
    };

    let cols = match cols_val.as_f64() {
        Some(c) => {
            if c < 1.0 || c.fract() != 0.0 {
                return Ok(CellValue::Error(CellError::Value));
            }
            c as usize
        }
        None => return Ok(CellValue::Error(CellError::Value)),
    };

    let mut out_matrix = Vec::with_capacity(rows);

    for r in 1..=rows {
        let mut out_row = Vec::with_capacity(cols);
        for c in 1..=cols {
            let res = invoke_lambda(
                lambda_node,
                &[CellValue::Number(r as f64), CellValue::Number(c as f64)],
                ctx,
                eval_fn,
            )?;
            out_row.push(res);
        }
        out_matrix.push(out_row);
    }

    Ok(CellValue::Array(out_matrix))
}

/// Evaluates `BYROW(array, lambda)`.
pub fn eval_byrow<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 2 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let arr_val = eval_fn(&args[0], ctx)?;
    let lambda_node = &args[1];

    let matrix = crate::functions::lookup::to_matrix(&arr_val);
    let mut out_matrix = Vec::with_capacity(matrix.len());

    for row in matrix {
        let row_val = CellValue::Array(vec![row]);
        let res = invoke_lambda(lambda_node, &[row_val], ctx, eval_fn)?;
        out_matrix.push(vec![res]);
    }

    Ok(CellValue::Array(out_matrix))
}

/// Evaluates `BYCOL(array, lambda)`.
pub fn eval_bycol<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 2 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let arr_val = eval_fn(&args[0], ctx)?;
    let lambda_node = &args[1];

    let matrix = crate::functions::lookup::to_matrix(&arr_val);
    if matrix.is_empty() || matrix[0].is_empty() {
        return Ok(CellValue::Error(CellError::Calc));
    }

    let num_cols = matrix[0].len();
    let mut out_row = Vec::with_capacity(num_cols);

    for c in 0..num_cols {
        let col_cells = matrix.iter().map(|r| vec![r[c].clone()]).collect();
        let col_val = CellValue::Array(col_cells);
        let res = invoke_lambda(lambda_node, &[col_val], ctx, eval_fn)?;
        out_row.push(res);
    }

    Ok(CellValue::Array(vec![out_row]))
}

/// Evaluates `ISOMITTED(parameter)`.
pub fn eval_isomitted<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.is_empty() {
        return Ok(CellValue::Error(CellError::Value));
    }

    match &args[0] {
        AstNode::Literal(Literal::Empty) => Ok(CellValue::Boolean(true)),
        AstNode::Reference(Reference::Named(name)) => {
            if let Some(val) = ctx.env.get(&name.to_ascii_uppercase()) {
                Ok(CellValue::Boolean(val.is_empty()))
            } else {
                Ok(CellValue::Boolean(true))
            }
        }
        other => {
            let val = eval_fn(other, ctx)?;
            Ok(CellValue::Boolean(val.is_empty()))
        }
    }
}
