//! OpenOfficeRS Excel Engine - Modern Dynamic Lambda & Let Functions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements LET, LAMBDA closures, MAP, REDUCE, and SCAN.

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
    pub udf_handler: Option<UdfHandler>,
}

impl<'a> EvalContext<'a> {
    pub fn new(grid: Option<&'a crate::grid::CompressedSparsePageGrid>, date1904: bool) -> Self {
        Self {
            grid,
            date1904,
            env: HashMap::new(),
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

/// Evaluates `MAP(array, lambda)` applying a lambda to each element of array.
pub fn eval_map<F>(args: &[AstNode], ctx: &EvalContext, eval_fn: &F) -> Result<CellValue>
where
    F: Fn(&AstNode, &EvalContext) -> Result<CellValue>,
{
    if args.len() < 2 {
        return Ok(CellValue::Error(CellError::Value));
    }

    let arr_val = eval_fn(&args[0], ctx)?;
    let lambda_node = &args[1];

    let matrix = crate::functions::lookup::to_matrix(&arr_val);
    let mut out_matrix = Vec::new();

    for row in matrix {
        let mut out_row = Vec::new();
        for cell in row {
            let res = invoke_lambda(lambda_node, &[cell], ctx, eval_fn)?;
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
