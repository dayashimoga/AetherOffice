# Sub-Phase 51.2.1: WASM Plugin Execution Bridge & Custom Spreadsheet UDF Wiring

## 1. Context & Objectives
As defined in GEC-v7.0 and Stage 51 of the master roadmap, Sub-Phase 51.2.1 integrates the WebAssembly sandboxed runtime (`plugin-runtime`) with the native spreadsheet formula evaluation engine (`excel-engine`) and unified presentation shell (`apps/unified-shell`).

Key objectives:
1. **Dynamic UDF Registration & Execution**:
   - Provide direct bidirectional bridge between `PluginUdfRegistry` and `excel-engine::functions::UdfHandler`.
   - Support arbitrary custom UDF functions (e.g. `MY_UDF`, `GEO_DISTANCE`, `TAX_CALC`) called directly from formula expressions with dynamic argument coercion, gas fuel consumption, and graceful error handling (`#VALUE!`, `#NAME?`).
2. **Plugin Lifecycle & Capability Management**:
   - Dynamic plugin loading from manifest and Wasm bytes with memory limits, gas metering, capability checks, and sandboxed state transitions (`Loaded` -> `Active` -> `Suspended` -> `Terminated`).
   - Plugin metadata query with detailed inspection (ID, name, version, status, capabilities, fuel consumption, registered UDF signatures).
3. **Tauri IPC Command Endpoints**:
   - `plugin_load`: Ingests plugin manifest and binary bytecode.
   - `plugin_unload`: Unloads plugin instance and purges associated UDFs.
   - `plugin_list`: Lists all installed plugins and their operational state.
   - `plugin_execute_action`: Executes sandboxed host actions (`InsertText`, `SetCellValue`, `QuerySelection`, etc.).
   - `plugin_call_udf`: Evaluates a registered UDF directly by name and serialized arguments.
   - `plugin_list_udfs`: Lists all registered calculation UDF definitions.
4. **Presentation Shell Coordinator (`PluginManagerCoordinator.ts`)**:
   - Frontend state manager for plugin installation, enabling/disabling, fuel limit monitoring, and live spreadsheet UDF discovery.
5. **Quality & Verification**:
   - 10 unit & integration tests in `crates/plugin-runtime/tests/subphase_51_2_1_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/plugin_manager_coordinator.test.ts`.
   - Gatekeeper `tools/verify-contract.ps1` exit code 0.

