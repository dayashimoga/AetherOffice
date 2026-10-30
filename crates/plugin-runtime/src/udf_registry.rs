use crate::error::PluginError;
use excel_engine::functions::UdfHandler;
use excel_engine::grid::{CellError, CellValue};
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
        self.functions.values().map(|f| f.name.clone()).collect()
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
