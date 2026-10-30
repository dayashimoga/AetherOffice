use crate::error::PluginError;
use crate::host::DocumentHost;
use crate::manifest::{PluginCapability, PluginManifest};
use crate::memory::LinearMemory;
use crate::wasm_binary::{parse_wasm_module, ParsedWasmModule};
use crate::wit_contract::{CellCoordinate, LifecycleEvent, SelectionInfo};
use std::collections::{HashMap, HashSet};

pub const DEFAULT_PLUGIN_FUEL_LIMIT: u64 = 100_000;
pub const DEFAULT_ACTION_FUEL_COST: u64 = 100;
pub const DEFAULT_EVENT_FUEL_COST: u64 = 500;

/// Operational lifecycle state of a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin binary parsed, memory allocated, awaiting initialization.
    Loaded,
    /// Plugin successfully initialized and handling document events.
    Active,
    /// Plugin temporarily suspended; events are suppressed.
    Suspended,
    /// Plugin terminated and shut down.
    Terminated,
}

impl PluginState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Terminated => "terminated",
        }
    }
}

/// Execution fuel meter tracking compute gas consumption to prevent runaway plugins.
#[derive(Debug, Clone)]
pub struct FuelMeter {
    pub limit: u64,
    pub remaining: u64,
}

impl FuelMeter {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            remaining: limit,
        }
    }

    pub fn consume(&mut self, units: u64) -> Result<(), PluginError> {
        if self.remaining < units {
            self.remaining = 0;
            return Err(PluginError::FuelExhausted { limit: self.limit });
        }
        self.remaining = self.remaining.saturating_sub(units);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.remaining = self.limit;
    }
}

/// Actions dispatched by a plugin to query or mutate host state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginAction {
    QuerySelection {
        doc_id: String,
    },
    QueryCellValue {
        coord: CellCoordinate,
    },
    QueryParagraph {
        doc_id: String,
        para_index: u32,
    },
    QuerySlideShapes {
        doc_id: String,
        slide_index: u32,
    },
    InsertText {
        doc_id: String,
        text: String,
    },
    SetCellValue {
        coord: CellCoordinate,
        value: String,
    },
    SetCellFormula {
        coord: CellCoordinate,
        formula: String,
    },
    ExecuteRibbonCommand {
        action: String,
        payload: String,
    },
}

/// Results returned from executing a `PluginAction`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    Selection(SelectionInfo),
    CellValue(String),
    ParagraphText(String),
    SlideShapes(Vec<String>),
    MutationSuccess,
    RibbonSuccess(bool),
}

/// An isolated, sandboxed instance of a WebAssembly plugin.
pub struct PluginInstance {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub memory: LinearMemory,
    pub fuel: FuelMeter,
    pub exported_hooks: HashSet<String>,
    pub parsed_module: ParsedWasmModule,
    pub dispatched_events_count: u64,
}

impl PluginInstance {
    pub fn new(
        manifest: PluginManifest,
        parsed_module: ParsedWasmModule,
        fuel_limit: u64,
    ) -> Result<Self, PluginError> {
        manifest.validate()?;

        let (min_pages, max_pages) = match parsed_module.memory {
            Some(limits) => (limits.min_pages, limits.max_pages),
            None => (1, Some(64)), // 1 page default (64KB), up to 4MB
        };

        let memory = LinearMemory::new(min_pages, max_pages)?;
        let mut exported_hooks = HashSet::new();
        for exp in &parsed_module.exports {
            exported_hooks.insert(exp.name.clone());
        }

        Ok(Self {
            manifest,
            state: PluginState::Loaded,
            memory,
            fuel: FuelMeter::new(fuel_limit),
            exported_hooks,
            parsed_module,
            dispatched_events_count: 0,
        })
    }

    /// Check if this plugin exports a specific lifecycle hook.
    pub fn exports_hook(&self, hook_name: &str) -> bool {
        self.exported_hooks.contains(hook_name)
    }

    /// Activate this plugin instance.
    pub fn activate(&mut self) -> Result<(), PluginError> {
        match self.state {
            PluginState::Loaded | PluginState::Suspended => {
                self.state = PluginState::Active;
                Ok(())
            }
            PluginState::Active => Ok(()),
            PluginState::Terminated => Err(PluginError::LifecycleStateError {
                current: self.state.as_str().to_string(),
                action: "activate".to_string(),
            }),
        }
    }

    /// Suspend this plugin instance.
    pub fn suspend(&mut self) -> Result<(), PluginError> {
        match self.state {
            PluginState::Active => {
                self.state = PluginState::Suspended;
                Ok(())
            }
            PluginState::Suspended => Ok(()),
            _ => Err(PluginError::LifecycleStateError {
                current: self.state.as_str().to_string(),
                action: "suspend".to_string(),
            }),
        }
    }

    /// Terminate this plugin instance.
    pub fn terminate(&mut self) -> Result<(), PluginError> {
        self.state = PluginState::Terminated;
        Ok(())
    }

    /// Execute a host query or mutation on behalf of this plugin, enforcing capability gates and gas limits.
    pub fn execute_action(
        &mut self,
        action: PluginAction,
        host: &mut dyn DocumentHost,
    ) -> Result<ActionResult, PluginError> {
        if self.state != PluginState::Active {
            return Err(PluginError::LifecycleStateError {
                current: self.state.as_str().to_string(),
                action: "execute_action".to_string(),
            });
        }

        // Consume fuel for the action
        self.fuel.consume(DEFAULT_ACTION_FUEL_COST)?;

        match action {
            PluginAction::QuerySelection { doc_id } => {
                self.check_capability(&PluginCapability::DocumentRead)?;
                let res = host.get_active_selection(&doc_id)?;
                Ok(ActionResult::Selection(res))
            }
            PluginAction::QueryCellValue { coord } => {
                self.check_capability(&PluginCapability::DocumentRead)?;
                let res = host.get_cell_value(&coord)?;
                Ok(ActionResult::CellValue(res))
            }
            PluginAction::QueryParagraph { doc_id, para_index } => {
                self.check_capability(&PluginCapability::DocumentRead)?;
                let res = host.get_paragraph_text(&doc_id, para_index)?;
                Ok(ActionResult::ParagraphText(res))
            }
            PluginAction::QuerySlideShapes {
                doc_id,
                slide_index,
            } => {
                self.check_capability(&PluginCapability::DocumentRead)?;
                let res = host.get_slide_shapes(&doc_id, slide_index)?;
                Ok(ActionResult::SlideShapes(res))
            }
            PluginAction::InsertText { doc_id, text } => {
                self.check_capability(&PluginCapability::DocumentMutate)?;
                host.insert_text(&doc_id, &text)?;
                Ok(ActionResult::MutationSuccess)
            }
            PluginAction::SetCellValue { coord, value } => {
                self.check_capability(&PluginCapability::DocumentMutate)?;
                host.set_cell_value(&coord, &value)?;
                Ok(ActionResult::MutationSuccess)
            }
            PluginAction::SetCellFormula { coord, formula } => {
                self.check_capability(&PluginCapability::DocumentMutate)?;
                host.set_cell_formula(&coord, &formula)?;
                Ok(ActionResult::MutationSuccess)
            }
            PluginAction::ExecuteRibbonCommand { action, payload } => {
                self.check_capability(&PluginCapability::RibbonCommand)?;
                let res = host.execute_ribbon_command(&action, &payload)?;
                Ok(ActionResult::RibbonSuccess(res))
            }
        }
    }

    /// Verify that this plugin possesses the required capability permission.
    fn check_capability(&self, capability: &PluginCapability) -> Result<(), PluginError> {
        if self.manifest.has_capability(capability) {
            Ok(())
        } else {
            Err(PluginError::PermissionDenied {
                required_capability: capability.as_str().to_string(),
            })
        }
    }

    /// Dispatch a lifecycle event to this plugin.
    pub fn dispatch_event(&mut self, event: &LifecycleEvent) -> Result<(), PluginError> {
        let hook_name = event.hook_name();

        match self.state {
            PluginState::Loaded => {
                // Loaded plugins can only receive OnInit
                if hook_name != "on-init" {
                    return Err(PluginError::LifecycleStateError {
                        current: self.state.as_str().to_string(),
                        action: format!("dispatch_{hook_name}"),
                    });
                }
            }
            PluginState::Active => {}
            PluginState::Suspended => {
                // Silently skip events when suspended
                return Ok(());
            }
            PluginState::Terminated => {
                return Err(PluginError::LifecycleStateError {
                    current: self.state.as_str().to_string(),
                    action: format!("dispatch_{hook_name}"),
                });
            }
        }

        // Consume fuel for event dispatch
        self.fuel.consume(DEFAULT_EVENT_FUEL_COST)?;

        if hook_name == "on-init" {
            self.state = PluginState::Active;
        }

        self.dispatched_events_count = self.dispatched_events_count.saturating_add(1);
        Ok(())
    }
}

/// The AetherOffice WebAssembly Plugin Host managing plugins and isolation boundaries.
pub struct WasmPluginHost {
    plugins: HashMap<String, PluginInstance>,
    default_fuel_limit: u64,
}

impl Default for WasmPluginHost {
    fn default() -> Self {
        Self::new(DEFAULT_PLUGIN_FUEL_LIMIT)
    }
}

impl WasmPluginHost {
    pub fn new(default_fuel_limit: u64) -> Self {
        Self {
            plugins: HashMap::new(),
            default_fuel_limit,
        }
    }

    /// Load a WebAssembly plugin from raw binary bytes.
    /// Uses manifest embedded in custom section `"aetheroffice:manifest"` if manifest_override is None.
    pub fn load_plugin(
        &mut self,
        binary: &[u8],
        manifest_override: Option<PluginManifest>,
    ) -> Result<String, PluginError> {
        let parsed = parse_wasm_module(binary)?;

        let manifest = match manifest_override {
            Some(m) => m,
            None => parsed.embedded_manifest.clone().ok_or_else(|| {
                PluginError::InvalidManifest(
                    "No embedded manifest found and no override provided".to_string(),
                )
            })?,
        };

        manifest.validate()?;
        let plugin_id = manifest.id.clone();

        let instance = PluginInstance::new(manifest, parsed, self.default_fuel_limit)?;
        self.plugins.insert(plugin_id.clone(), instance);

        Ok(plugin_id)
    }

    /// Get a reference to a plugin instance by ID.
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.plugins.get(plugin_id)
    }

    /// Get a mutable reference to a plugin instance by ID.
    pub fn get_plugin_mut(&mut self, plugin_id: &str) -> Option<&mut PluginInstance> {
        self.plugins.get_mut(plugin_id)
    }

    /// Total count of loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// List all loaded plugin IDs.
    pub fn list_plugin_ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Initialize a plugin by ID (`on-init` event).
    pub fn init_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let instance = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

        instance.dispatch_event(&LifecycleEvent::OnInit {
            plugin_id: plugin_id.to_string(),
        })
    }

    /// Suspend a plugin.
    pub fn suspend_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let instance = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;
        instance.suspend()
    }

    /// Resume / activate a plugin.
    pub fn resume_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let instance = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;
        instance.activate()
    }

    /// Terminate and unload a plugin.
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        if let Some(mut instance) = self.plugins.remove(plugin_id) {
            instance.terminate()?;
            Ok(())
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Dispatch an event to a single plugin.
    pub fn dispatch_event(
        &mut self,
        plugin_id: &str,
        event: &LifecycleEvent,
    ) -> Result<(), PluginError> {
        let instance = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;
        instance.dispatch_event(event)
    }

    /// Broadcast a document lifecycle event to all active plugins.
    pub fn broadcast_event(
        &mut self,
        event: &LifecycleEvent,
    ) -> Vec<(String, Result<(), PluginError>)> {
        let mut results = Vec::new();
        for (id, instance) in &mut self.plugins {
            if instance.state == PluginState::Active {
                let res = instance.dispatch_event(event);
                results.push((id.clone(), res));
            }
        }
        results
    }

    /// Execute a plugin-driven action through the host.
    pub fn execute_plugin_action(
        &mut self,
        plugin_id: &str,
        action: PluginAction,
        host: &mut dyn DocumentHost,
    ) -> Result<ActionResult, PluginError> {
        let instance = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;
        instance.execute_action(action, host)
    }
}
