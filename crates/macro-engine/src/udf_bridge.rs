use crate::ast::VbaProgram;
use crate::evaluator::VbaEvaluator;
use crate::sandbox::MacroSandbox;
use crate::virtual_dom::{VbaValue, VirtualOfficeDom};
use excel_engine::functions::UdfHandler;
use excel_engine::grid::{CellError, CellValue};
use std::sync::{Arc, Mutex};

/// Bridges a parsed VBA program to excel-engine's formula evaluator as a dynamic UDF handler.
pub fn create_vba_udf_handler(program: VbaProgram, sandbox_fuel: u64) -> UdfHandler {
    let program = Arc::new(program);
    let dom = Arc::new(Mutex::new(VirtualOfficeDom::new()));

    Arc::new(
        move |name: &str, args: &[CellValue]| -> ooxml_opc::error::Result<CellValue> {
            let vba_args: Vec<VbaValue> = args.iter().map(cell_value_to_vba_value).collect();
            let mut dom_guard = match dom.lock() {
                Ok(g) => g,
                Err(_) => return Ok(CellValue::Error(CellError::Value)),
            };

            let sandbox = MacroSandbox::new(sandbox_fuel);
            let mut evaluator = VbaEvaluator::with_sandbox(&mut dom_guard, sandbox);

            match evaluator.execute_routine(&program, name, &vba_args) {
                Ok(ret) => Ok(vba_value_to_cell_value(&ret)),
                Err(_) => Ok(CellValue::Error(CellError::Value)),
            }
        },
    )
}

pub fn cell_value_to_vba_value(cv: &CellValue) -> VbaValue {
    match cv {
        CellValue::Empty => VbaValue::Empty,
        CellValue::Boolean(b) => VbaValue::Boolean(*b),
        CellValue::Number(n) => VbaValue::Double(*n),
        CellValue::InlineString(s) => VbaValue::String(s.clone()),
        CellValue::SharedString(_) => VbaValue::Empty,
        CellValue::Error(_) => VbaValue::Null,
        CellValue::Formula { cached_value, .. } => {
            if let Some(n) = cached_value {
                VbaValue::Double(*n)
            } else {
                VbaValue::Empty
            }
        }
        CellValue::Array(matrix) => {
            // Flatten first element for scalar UDF param
            if let Some(first_row) = matrix.first() {
                if let Some(first_cell) = first_row.first() {
                    return cell_value_to_vba_value(first_cell);
                }
            }
            VbaValue::Empty
        }
    }
}

pub fn vba_value_to_cell_value(vv: &VbaValue) -> CellValue {
    match vv {
        VbaValue::Empty => CellValue::Empty,
        VbaValue::Boolean(b) => CellValue::Boolean(*b),
        VbaValue::Integer(i) => CellValue::Number(*i as f64),
        VbaValue::Double(d) => CellValue::Number(*d),
        VbaValue::String(s) => CellValue::InlineString(s.clone()),
        VbaValue::Null | VbaValue::Nothing => CellValue::Error(CellError::Null),
    }
}
