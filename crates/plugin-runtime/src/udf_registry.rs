use crate::error::PluginError;
use excel_engine::functions::UdfHandler;
use excel_engine::grid::{CellError, CellValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Dynamic calculation function callback registered by a plugin.
pub type PluginUdfCallback =
    Arc<dyn Fn(&[CellValue]) -> Result<CellValue, PluginError> + Send + Sync>;

/// Metadata and execution handler for a dynamic calculation UDF.
#[derive(Clone)]
pub struct PluginUdfDefinition {
    pub name: String,
    pub description: String,
    pub min_args: usize,
    pub max_args: usize,
    pub callback: PluginUdfCallback,
}

/// Serializable metadata descriptor for a registered calculation UDF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginUdfMetaDto {
    pub name: String,
    pub description: String,
    pub min_args: usize,
    pub max_args: usize,
}

/// Serializable result descriptor for an interactive UDF call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginUdfCallResultDto {
    pub function_name: String,
    pub success: bool,
    pub result_value: Option<String>,
    pub error_message: Option<String>,
}

/// Registry managing dynamic User-Defined Functions contributed by plugins.
#[derive(Default, Clone)]
pub struct PluginUdfRegistry {
    functions: HashMap<String, PluginUdfDefinition>,
}

impl PluginUdfRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new calculation function. Function names are stored uppercase.
    pub fn register_udf(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        min_args: usize,
        max_args: usize,
        callback: impl Fn(&[CellValue]) -> Result<CellValue, PluginError> + Send + Sync + 'static,
    ) {
        let name_str = name.into();
        let key = name_str.to_ascii_uppercase();
        let def = PluginUdfDefinition {
            name: name_str,
            description: description.into(),
            min_args,
            max_args,
            callback: Arc::new(callback),
        };
        self.functions.insert(key, def);
    }

    /// Unregister a calculation function by name.
    pub fn unregister_udf(&mut self, name: &str) -> bool {
        self.functions.remove(&name.to_ascii_uppercase()).is_some()
    }

    /// Check if a calculation function is registered.
    pub fn has_udf(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_ascii_uppercase())
    }

    /// List all registered calculation function names.
    pub fn list_udf_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.functions.values().map(|f| f.name.clone()).collect();
        names.sort();
        names
    }

    /// List all registered calculation function metadata descriptors.
    pub fn list_udfs(&self) -> Vec<PluginUdfMetaDto> {
        let mut udfs: Vec<PluginUdfMetaDto> = self
            .functions
            .values()
            .map(|def| PluginUdfMetaDto {
                name: def.name.clone(),
                description: def.description.clone(),
                min_args: def.min_args,
                max_args: def.max_args,
            })
            .collect();
        udfs.sort_by(|a, b| a.name.cmp(&b.name));
        udfs
    }

    /// Number of registered calculation UDFs.
    pub fn count(&self) -> usize {
        self.functions.len()
    }

    /// Clear all registered UDFs.
    pub fn clear(&mut self) {
        self.functions.clear();
    }

    /// Execute a calculation function directly with arguments.
    pub fn execute_udf(&self, name: &str, args: &[CellValue]) -> Result<CellValue, PluginError> {
        let def = self
            .functions
            .get(&name.to_ascii_uppercase())
            .ok_or_else(|| {
                PluginError::HostCallError(format!("Unknown calculation UDF '{name}'"))
            })?;

        if args.len() < def.min_args || args.len() > def.max_args {
            return Err(PluginError::HostCallError(format!(
                "Function '{}' expects between {} and {} arguments, received {}",
                def.name,
                def.min_args,
                def.max_args,
                args.len()
            )));
        }

        (def.callback)(args)
    }

    /// Executes a calculation function from string arguments, returning a structured DTO.
    pub fn call_udf_with_strings(&self, name: &str, args: &[String]) -> PluginUdfCallResultDto {
        let cell_args: Vec<CellValue> = args
            .iter()
            .map(|s| {
                if let Ok(n) = s.parse::<f64>() {
                    CellValue::Number(n)
                } else if s.eq_ignore_ascii_case("true") {
                    CellValue::Boolean(true)
                } else if s.eq_ignore_ascii_case("false") {
                    CellValue::Boolean(false)
                } else {
                    CellValue::InlineString(s.clone())
                }
            })
            .collect();

        match self.execute_udf(name, &cell_args) {
            Ok(val) => {
                let s = match val {
                    CellValue::Number(n) => n.to_string(),
                    CellValue::InlineString(t) => t,
                    CellValue::SharedString(id) => format!("str#{id}"),
                    CellValue::Boolean(b) => b.to_string(),
                    CellValue::Error(e) => format!("{e:?}"),
                    CellValue::Formula {
                        formula,
                        cached_value,
                    } => {
                        format!("={formula} ({:?})", cached_value)
                    }
                    CellValue::Empty => String::new(),
                    CellValue::Array(_) => "[Array]".to_string(),
                };
                PluginUdfCallResultDto {
                    function_name: name.to_string(),
                    success: true,
                    result_value: Some(s),
                    error_message: None,
                }
            }
            Err(err) => PluginUdfCallResultDto {
                function_name: name.to_string(),
                success: false,
                result_value: None,
                error_message: Some(err.to_string()),
            },
        }
    }

    /// Create an `excel_engine::functions::UdfHandler` bridging this registry
    /// to the excel formula evaluation engine.
    pub fn create_excel_udf_handler(self: Arc<Self>) -> UdfHandler {
        Arc::new(
            move |name: &str, args: &[CellValue]| -> ooxml_opc::error::Result<CellValue> {
                match self.execute_udf(name, args) {
                    Ok(val) => Ok(val),
                    Err(_) => Ok(CellValue::Error(CellError::Value)),
                }
            },
        )
    }
}
