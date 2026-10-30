use crate::error::PluginError;
use serde::{Deserialize, Serialize};

/// Granular capability permissions granted to a WebAssembly plugin.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCapability {
    /// Read document contents (selections, text, cells, shapes).
    DocumentRead,
    /// Mutate document contents (insert text, update cells, set formulas).
    DocumentMutate,
    /// Execute or dispatch ribbon and menu commands.
    RibbonCommand,
    /// Connect to remote networks or APIs.
    NetworkAccess,
    /// Read or write the system clipboard.
    ClipboardAccess,
    /// Export files to the local file system.
    FileSystemExport,
    /// Custom vendor-defined capability.
    Custom(String),
}

impl PluginCapability {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DocumentRead => "document-read",
            Self::DocumentMutate => "document-mutate",
            Self::RibbonCommand => "ribbon-command",
            Self::NetworkAccess => "network-access",
            Self::ClipboardAccess => "clipboard-access",
            Self::FileSystemExport => "file-system-export",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Metadata and capability manifest for an AetherOffice WebAssembly plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_entrypoint")]
    pub entrypoint: String,
    #[serde(default)]
    pub capabilities: Vec<PluginCapability>,
    #[serde(default = "default_min_host_version")]
    pub min_host_version: String,
}

fn default_entrypoint() -> String {
    "plugin.wasm".to_string()
}

fn default_min_host_version() -> String {
    "0.1.0".to_string()
}

impl PluginManifest {
    /// Validate manifest fields according to specification rules.
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.id.trim().is_empty() {
            return Err(PluginError::InvalidManifest(
                "Plugin ID cannot be empty".to_string(),
            ));
        }
        if self.name.trim().is_empty() {
            return Err(PluginError::InvalidManifest(
                "Plugin Name cannot be empty".to_string(),
            ));
        }
        if self.version.trim().is_empty() {
            return Err(PluginError::InvalidManifest(
                "Plugin Version cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Check whether this manifest requests a specific capability.
    pub fn has_capability(&self, capability: &PluginCapability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Parse manifest from JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, PluginError> {
        let manifest: Self =
            serde_json::from_str(json).map_err(|e| PluginError::InvalidManifest(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serialize manifest to JSON string.
    pub fn to_json_string(&self) -> Result<String, PluginError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PluginError::SerializationError(e.to_string()))
    }
}
