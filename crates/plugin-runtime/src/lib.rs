#![forbid(unsafe_code)]

pub mod error;
pub mod host;
pub mod manifest;
pub mod memory;
pub mod runtime;
pub mod wasm_binary;
pub mod wit_contract;

pub use error::PluginError;
pub use host::{DocumentHost, DocumentMutationHandler, DocumentQueryHandler, MockDocumentHost};
pub use manifest::{PluginCapability, PluginManifest};
pub use memory::{LinearMemory, WASM_PAGE_SIZE};
pub use runtime::{
    ActionResult, FuelMeter, PluginAction, PluginInstance, PluginState, WasmPluginHost,
};
pub use wasm_binary::{parse_wasm_module, ParsedWasmModule, WasmBinaryBuilder};
pub use wit_contract::{CellCoordinate, DocumentType, LifecycleEvent, SelectionInfo};
