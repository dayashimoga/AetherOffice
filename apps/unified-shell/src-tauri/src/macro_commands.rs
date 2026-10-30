//! Macro & VBA Engine Tauri IPC Commands (GEC-v2.0 Subphase 15.2.2)

use macro_engine::{
    Capability, MacroSandbox, SecurityAuditReport, SecurityScanner, VbaEvaluator, VbaParser,
    VirtualOfficeDom,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExecutionResult {
    pub success: bool,
    pub return_value: String,
    pub gas_consumed: u64,
    pub error_message: Option<String>,
}

pub fn scan_macro_security(code: &str) -> Result<SecurityAuditReport, String> {
    Ok(SecurityScanner::scan_source(code))
}

pub fn execute_macro(
    code: &str,
    routine_name: &str,
    granted_capabilities: &[String],
    gas_limit: Option<u64>,
) -> Result<MacroExecutionResult, String> {
    let mut parser = VbaParser::from_source(code).map_err(|e| format!("Parser error: {e}"))?;
    let program = parser
        .parse_program()
        .map_err(|e| format!("Grammar error: {e}"))?;

    let limit = gas_limit.unwrap_or(100_000);
    let mut sandbox = MacroSandbox::new(limit);

    // Apply granted capabilities from caller
    for cap_str in granted_capabilities {
        match cap_str.to_ascii_lowercase().as_str() {
            "fileio" => sandbox.grant(Capability::FileIo),
            "network" => sandbox.grant(Capability::Network),
            "systemcommand" | "syscmd" => sandbox.grant(Capability::SystemCommand),
            "environment" => sandbox.grant(Capability::Environment),
            "registry" => sandbox.grant(Capability::Registry),
            _ => {}
        }
    }

    let mut dom = VirtualOfficeDom::new();
    let mut evaluator = VbaEvaluator::with_sandbox(&mut dom, sandbox);

    let res = if routine_name.trim().is_empty() {
        evaluator
            .execute_program(&program)
            .map(|_| macro_engine::VbaValue::Empty)
    } else {
        evaluator.execute_routine(&program, routine_name, &[])
    };

    let remaining = evaluator.sandbox.remaining_gas();
    let gas_consumed = limit.saturating_sub(remaining);

    match res {
        Ok(val) => Ok(MacroExecutionResult {
            success: true,
            return_value: val.to_display_string(),
            gas_consumed,
            error_message: None,
        }),
        Err(err) => Ok(MacroExecutionResult {
            success: false,
            return_value: String::new(),
            gas_consumed,
            error_message: Some(err.to_string()),
        }),
    }
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_scan_macro_security(code: String) -> Result<SecurityAuditReport, String> {
    scan_macro_security(&code)
}

#[cfg(not(test))]
#[tauri::command]
pub fn cmd_execute_macro(
    code: String,
    routine_name: String,
    granted_capabilities: Vec<String>,
    gas_limit: Option<u64>,
) -> Result<MacroExecutionResult, String> {
    execute_macro(&code, &routine_name, &granted_capabilities, gas_limit)
}
