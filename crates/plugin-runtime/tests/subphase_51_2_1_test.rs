use excel_engine::calc_dag::evaluate_ast_node_with_ctx;
use excel_engine::formula::parse_formula;
use excel_engine::functions::lambda::EvalContext;
use excel_engine::grid::{CellError, CellValue};
use plugin_runtime::{
    CellCoordinate, DocumentQueryHandler, MockDocumentHost, PluginAction, PluginCapability,
    PluginError, PluginManifest, PluginState, PluginUdfRegistry, WasmBinaryBuilder, WasmPluginHost,
};
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn make_manifest(id: &str, name: &str, caps: Vec<PluginCapability>) -> PluginManifest {
    PluginManifest {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "AetherOffice Plugin Lab".to_string(),
        description: "Integration test plugin for Sub-Phase 51.2.1".to_string(),
        entrypoint: "plugin.wasm".to_string(),
        capabilities: caps,
        min_host_version: "0.1.0".to_string(),
    }
}

// 1. Plugin manifest deserialization and validation
#[test]
fn test_plugin_manifest_deserialization_and_validation() -> TestResult {
    let json = r#"{
        "id": "com.aetheroffice.finance-tools",
        "name": "Financial Analytics",
        "version": "2.1.0",
        "author": "Aether Corp",
        "description": "Custom financial UDFs and ledger tools",
        "capabilities": ["document-read", "document-mutate", "ribbon-command"]
    }"#;

    let manifest = PluginManifest::from_json_str(json)?;
    assert_eq!(manifest.id, "com.aetheroffice.finance-tools");
    assert_eq!(manifest.name, "Financial Analytics");
    assert_eq!(manifest.version, "2.1.0");
    assert!(manifest.has_capability(&PluginCapability::DocumentRead));
    assert!(manifest.has_capability(&PluginCapability::DocumentMutate));
    assert!(manifest.has_capability(&PluginCapability::RibbonCommand));
    assert!(!manifest.has_capability(&PluginCapability::NetworkAccess));

    // Invalid manifest: empty id
    let bad_json = r#"{"id": "", "name": "No ID", "version": "1.0"}"#;
    assert!(PluginManifest::from_json_str(bad_json).is_err());

    Ok(())
}

// 2. Plugin host load and list plugins DTO inspection
#[test]
fn test_plugin_host_load_and_list_plugins() -> TestResult {
    let mut host = WasmPluginHost::new(50_000);
    let wasm = WasmBinaryBuilder::new().component(false).build();
    let manifest = make_manifest(
        "com.aetheroffice.geo",
        "Geographic Calculator",
        vec![PluginCapability::DocumentRead],
    );

    let plugin_id = host.load_plugin(&wasm, Some(manifest))?;
    assert_eq!(plugin_id, "com.aetheroffice.geo");
    assert_eq!(host.plugin_count(), 1);

    let list = host.list_plugins();
    assert_eq!(list.len(), 1);
    let item = &list[0];
    assert_eq!(item.id, "com.aetheroffice.geo");
    assert_eq!(item.name, "Geographic Calculator");
    assert_eq!(item.state, "loaded");
    assert_eq!(item.fuel_limit, 50_000);
    assert_eq!(item.fuel_remaining, 50_000);
    assert!(item.capabilities.contains(&"document-read".to_string()));

    Ok(())
}

// 3. Plugin lifecycle state machine transitions
#[test]
fn test_plugin_lifecycle_state_machine() -> TestResult {
    let mut host = WasmPluginHost::new(100_000);
    let wasm = WasmBinaryBuilder::new().component(false).build();
    let manifest = make_manifest("com.test.lifecycle", "Lifecycle Test", vec![]);
    let pid = host.load_plugin(&wasm, Some(manifest))?;

    let p = host.get_plugin(&pid).unwrap();
    assert_eq!(p.state, PluginState::Loaded);
    assert!(!host.is_active(&pid));

    // Initialize -> Active
    host.init_plugin(&pid)?;
    assert_eq!(host.get_plugin(&pid).unwrap().state, PluginState::Active);
    assert!(host.is_active(&pid));

    // Suspend -> Suspended
    host.suspend_plugin(&pid)?;
    assert_eq!(host.get_plugin(&pid).unwrap().state, PluginState::Suspended);
    assert!(!host.is_active(&pid));

    // Resume -> Active
    host.resume_plugin(&pid)?;
    assert_eq!(host.get_plugin(&pid).unwrap().state, PluginState::Active);

    // Unload -> Terminated & Removed
    host.unload_plugin(&pid)?;
    assert_eq!(host.plugin_count(), 0);
    assert!(host.get_plugin(&pid).is_none());

    Ok(())
}

// 4. Plugin action execution and fuel consumption
#[test]
fn test_plugin_action_execution_and_fuel_consumption() -> TestResult {
    let mut host = WasmPluginHost::new(10_000);
    let wasm = WasmBinaryBuilder::new().component(false).build();
    let manifest = make_manifest(
        "com.test.actions",
        "Action Test",
        vec![PluginCapability::DocumentMutate],
    );
    let pid = host.load_plugin(&wasm, Some(manifest))?;
    host.init_plugin(&pid)?;

    let mut doc_host = MockDocumentHost::new();

    let action = PluginAction::SetCellValue {
        coord: CellCoordinate::new("Sheet1", 0, 0),
        value: "42.5".to_string(),
    };

    let res = host.execute_plugin_action(&pid, action, &mut doc_host)?;
    assert_eq!(res, plugin_runtime::ActionResult::MutationSuccess);
    assert_eq!(
        doc_host.get_cell_value(&CellCoordinate::new("Sheet1", 0, 0))?,
        "42.5"
    );

    // Fuel consumed (100 units per action)
    let p = host.get_plugin(&pid).unwrap();
    assert!(p.fuel.remaining < 10_000);

    Ok(())
}

// 5. Plugin permission denial without required capability
#[test]
fn test_plugin_permission_denial_without_capability() -> TestResult {
    let mut host = WasmPluginHost::new(10_000);
    let wasm = WasmBinaryBuilder::new().component(false).build();
    // Only DocumentRead, missing DocumentMutate
    let manifest = make_manifest(
        "com.test.readonly",
        "Readonly Test",
        vec![PluginCapability::DocumentRead],
    );
    let pid = host.load_plugin(&wasm, Some(manifest))?;
    host.init_plugin(&pid)?;

    let mut doc_host = MockDocumentHost::new();
    let action = PluginAction::InsertText {
        doc_id: "doc-1".to_string(),
        text: "Unauthorized edit".to_string(),
    };

    let err = host.execute_plugin_action(&pid, action, &mut doc_host);
    assert!(matches!(err, Err(PluginError::PermissionDenied { .. })));

    Ok(())
}

// 6. Plugin fuel exhaustion and refill
#[test]
fn test_plugin_fuel_exhaustion_and_refill() -> TestResult {
    let mut host = WasmPluginHost::new(700);
    let wasm = WasmBinaryBuilder::new().component(false).build();
    let manifest = make_manifest(
        "com.test.fuel",
        "Fuel Test",
        vec![PluginCapability::DocumentMutate],
    );
    let pid = host.load_plugin(&wasm, Some(manifest))?;
    // init_plugin consumes 500 fuel -> remaining: 200
    host.init_plugin(&pid)?;

    let mut doc_host = MockDocumentHost::new();

    // 1st action consumes 100 fuel -> remaining: 100
    host.execute_plugin_action(
        &pid,
        PluginAction::InsertText {
            doc_id: "doc-1".to_string(),
            text: "Hello".to_string(),
        },
        &mut doc_host,
    )?;

    // 2nd action consumes 100 fuel -> remaining: 0
    host.execute_plugin_action(
        &pid,
        PluginAction::InsertText {
            doc_id: "doc-1".to_string(),
            text: " World".to_string(),
        },
        &mut doc_host,
    )?;

    // 3rd action fails with FuelExhausted
    let err = host.execute_plugin_action(
        &pid,
        PluginAction::InsertText {
            doc_id: "doc-1".to_string(),
            text: "!".to_string(),
        },
        &mut doc_host,
    );
    assert!(matches!(err, Err(PluginError::FuelExhausted { .. })));

    // Refill fuel
    host.refill_fuel(&pid)?;
    assert_eq!(host.get_plugin(&pid).unwrap().fuel.remaining, 700);

    // Can execute again
    let ok = host.execute_plugin_action(
        &pid,
        PluginAction::InsertText {
            doc_id: "doc-1".to_string(),
            text: " Success".to_string(),
        },
        &mut doc_host,
    );
    assert!(ok.is_ok());

    Ok(())
}

// 7. Dynamic UDF registration and argument checking
#[test]
fn test_udf_registry_custom_udf_registration_and_execution() -> TestResult {
    let mut registry = PluginUdfRegistry::new();

    // Register a hypotenuse UDF: HYPOT(a, b) = sqrt(a^2 + b^2)
    registry.register_udf(
        "HYPOT",
        "Calculates hypotenuse of right-angle triangle",
        2,
        2,
        |args| {
            let a = args[0]
                .as_f64()
                .ok_or_else(|| PluginError::HostCallError("Arg 1 must be a number".to_string()))?;
            let b = args[1]
                .as_f64()
                .ok_or_else(|| PluginError::HostCallError("Arg 2 must be a number".to_string()))?;
            Ok(CellValue::Number((a * a + b * b).sqrt()))
        },
    );

    assert!(registry.has_udf("hypot"));
    assert_eq!(registry.count(), 1);

    let res = registry.execute_udf("HYPOT", &[CellValue::Number(3.0), CellValue::Number(4.0)])?;
    assert_eq!(res, CellValue::Number(5.0));

    // Wrong argument count
    let err = registry.execute_udf("HYPOT", &[CellValue::Number(3.0)]);
    assert!(err.is_err());

    Ok(())
}

// 8. UDF execution via string arguments DTO
#[test]
fn test_udf_call_with_strings_dto() -> TestResult {
    let mut registry = PluginUdfRegistry::new();

    // Register TAX_CALC(amount, rate) = amount * (1.0 + rate)
    registry.register_udf(
        "TAX_CALC",
        "Calculates total amount with tax",
        2,
        2,
        |args| {
            let amount = args[0]
                .as_f64()
                .ok_or_else(|| PluginError::HostCallError("Invalid amount".to_string()))?;
            let rate = args[1]
                .as_f64()
                .ok_or_else(|| PluginError::HostCallError("Invalid rate".to_string()))?;
            Ok(CellValue::Number(amount * (1.0 + rate)))
        },
    );

    let result =
        registry.call_udf_with_strings("TAX_CALC", &["100.0".to_string(), "0.08".to_string()]);
    assert!(result.success);
    assert_eq!(result.result_value, Some("108".to_string()));
    assert_eq!(result.error_message, None);

    // Call unknown UDF
    let bad = registry.call_udf_with_strings("UNKNOWN_FUNC", &[]);
    assert!(!bad.success);
    assert!(bad.error_message.is_some());

    Ok(())
}

// 9. UDF integration with Excel engine formula evaluator
#[test]
fn test_udf_integration_with_excel_formula_evaluator() -> TestResult {
    let mut registry = PluginUdfRegistry::new();

    // Register MULTIPLY_ADD(a, b, c) = a * b + c
    registry.register_udf(
        "MULTIPLY_ADD",
        "Multiplies a and b then adds c",
        3,
        3,
        |args| {
            let a = args[0].as_f64().unwrap_or(0.0);
            let b = args[1].as_f64().unwrap_or(0.0);
            let c = args[2].as_f64().unwrap_or(0.0);
            Ok(CellValue::Number(a * b + c))
        },
    );

    let udf_handler = Arc::new(registry).create_excel_udf_handler();
    let ctx = EvalContext::new(None, false).with_udf_handler(udf_handler);

    let ast = parse_formula("=MULTIPLY_ADD(4, 5, 2)")?;
    let evaluated = evaluate_ast_node_with_ctx(&ast, &ctx, true)?;
    assert_eq!(evaluated, CellValue::Number(22.0));

    Ok(())
}

// 10. UDF deregistration and formula error handling
#[test]
fn test_udf_deregistration_and_excel_formula_error() -> TestResult {
    let mut registry = PluginUdfRegistry::new();
    registry.register_udf("TEMP_FUNC", "Temporary UDF", 1, 1, |args| {
        let v = args[0].as_f64().unwrap_or(0.0);
        Ok(CellValue::Number(v * 2.0))
    });

    let udfs = registry.list_udfs();
    assert_eq!(udfs.len(), 1);
    assert_eq!(udfs[0].name, "TEMP_FUNC");

    // Unregister
    assert!(registry.unregister_udf("TEMP_FUNC"));
    assert!(!registry.has_udf("TEMP_FUNC"));
    assert_eq!(registry.count(), 0);

    let udf_handler = Arc::new(registry).create_excel_udf_handler();
    let ctx = EvalContext::new(None, false).with_udf_handler(udf_handler);

    let ast = parse_formula("=TEMP_FUNC(10)")?;
    let evaluated = evaluate_ast_node_with_ctx(&ast, &ctx, true)?;
    // Unknown function returns #NAME? or #VALUE!
    assert!(matches!(
        evaluated,
        CellValue::Error(CellError::Name) | CellValue::Error(CellError::Value)
    ));

    Ok(())
}
