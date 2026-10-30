use plugin_runtime::{
    parse_wasm_module, ActionResult, CellCoordinate, DocumentQueryHandler, DocumentType,
    LifecycleEvent, MockDocumentHost, PluginAction, PluginCapability, PluginError, PluginManifest,
    PluginState, SelectionInfo, WasmBinaryBuilder, WasmPluginHost,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn make_test_manifest(id: &str, name: &str, caps: Vec<PluginCapability>) -> PluginManifest {
    PluginManifest {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "AetherOffice Engineering".to_string(),
        description: "Integration test plugin".to_string(),
        entrypoint: "plugin.wasm".to_string(),
        capabilities: caps,
        min_host_version: "0.1.0".to_string(),
    }
}

// 1. Wasm binary magic and version validation
#[test]
fn test_wasm_binary_magic_and_version_validation() -> TestResult {
    // Valid Core Wasm binary
    let core_wasm = WasmBinaryBuilder::new().component(false).build();
    let parsed_core = parse_wasm_module(&core_wasm)?;
    assert!(!parsed_core.is_component);
    assert_eq!(parsed_core.version, 1);

    // Valid Component Layer binary
    let component_wasm = WasmBinaryBuilder::new().component(true).build();
    let parsed_component = parse_wasm_module(&component_wasm)?;
    assert!(parsed_component.is_component);

    // Invalid magic
    let bad_magic = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
    let err = parse_wasm_module(&bad_magic);
    assert!(matches!(err, Err(PluginError::WasmParseError(_))));

    // Short binary (< 8 bytes)
    let short_bin = vec![0x00, 0x61, 0x73];
    assert!(matches!(
        parse_wasm_module(&short_bin),
        Err(PluginError::WasmParseError(_))
    ));

    Ok(())
}

// 2. Wasm section parsing and manifest extraction
#[test]
fn test_wasm_section_parsing_and_manifest_extraction() -> TestResult {
    let manifest = make_test_manifest(
        "com.aetheroffice.spellcheck",
        "Spell Check Plugin",
        vec![
            PluginCapability::DocumentRead,
            PluginCapability::DocumentMutate,
        ],
    );

    let wasm_bin = WasmBinaryBuilder::new()
        .with_manifest(&manifest)?
        .with_memory(2, Some(16))
        .with_export("on-init", 0, 0)
        .with_export("on-selection-changed", 0, 1)
        .with_import("document-query", "get-active-selection", 0)
        .build();

    let parsed = parse_wasm_module(&wasm_bin)?;

    assert_eq!(parsed.exports.len(), 2);
    assert_eq!(parsed.exports[0].name, "on-init");
    assert_eq!(parsed.exports[1].name, "on-selection-changed");

    assert_eq!(parsed.imports.len(), 1);
    assert_eq!(parsed.imports[0].module, "document-query");
    assert_eq!(parsed.imports[0].field, "get-active-selection");

    let mem = parsed.memory.ok_or_else(|| {
        PluginError::RuntimeFault("Expected memory limits in parsed wasm".to_string())
    })?;
    assert_eq!(mem.min_pages, 2);
    assert_eq!(mem.max_pages, Some(16));

    let embedded = parsed.embedded_manifest.ok_or_else(|| {
        PluginError::RuntimeFault("Expected embedded manifest in parsed wasm".to_string())
    })?;
    assert_eq!(embedded.id, "com.aetheroffice.spellcheck");
    assert_eq!(embedded.name, "Spell Check Plugin");
    assert!(embedded.has_capability(&PluginCapability::DocumentRead));
    assert!(embedded.has_capability(&PluginCapability::DocumentMutate));

    Ok(())
}

// 3. Plugin manifest validation and serialization
#[test]
fn test_plugin_manifest_validation() -> TestResult {
    let manifest = make_test_manifest(
        "org.aether.calc.tools",
        "Calc Productivity Tools",
        vec![PluginCapability::RibbonCommand],
    );
    assert!(manifest.validate().is_ok());

    let json_str = manifest.to_json_string()?;
    let roundtripped = PluginManifest::from_json_str(&json_str)?;
    assert_eq!(roundtripped.id, manifest.id);
    assert_eq!(roundtripped.name, manifest.name);
    assert!(roundtripped.has_capability(&PluginCapability::RibbonCommand));
    assert!(!roundtripped.has_capability(&PluginCapability::NetworkAccess));

    // Empty ID rejection
    let mut invalid = manifest.clone();
    invalid.id = "   ".to_string();
    assert!(matches!(
        invalid.validate(),
        Err(PluginError::InvalidManifest(_))
    ));

    // Empty Name rejection
    let mut invalid_name = manifest.clone();
    invalid_name.name = String::new();
    assert!(matches!(
        invalid_name.validate(),
        Err(PluginError::InvalidManifest(_))
    ));

    Ok(())
}

// 4. Plugin host loading and lifecycle state machine
#[test]
fn test_plugin_host_loading_and_lifecycle_state_machine() -> TestResult {
    let mut host = WasmPluginHost::default();

    let manifest = make_test_manifest(
        "com.aether.lifecycle.test",
        "Lifecycle Plugin",
        vec![PluginCapability::DocumentRead],
    );

    let bin = WasmBinaryBuilder::new()
        .with_manifest(&manifest)?
        .with_export("on-init", 0, 0)
        .build();

    let plugin_id = host.load_plugin(&bin, None)?;
    assert_eq!(plugin_id, "com.aether.lifecycle.test");
    assert_eq!(host.plugin_count(), 1);

    // Initial state is Loaded
    {
        let p = host
            .get_plugin(&plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.clone()))?;
        assert_eq!(p.state, PluginState::Loaded);
    }

    // Non-init event while Loaded is rejected
    let non_init_err = host.dispatch_event(
        &plugin_id,
        &LifecycleEvent::OnBeforeSave {
            doc_id: "doc-1".to_string(),
        },
    );
    assert!(matches!(
        non_init_err,
        Err(PluginError::LifecycleStateError { .. })
    ));

    // Calling init_plugin activates it
    host.init_plugin(&plugin_id)?;
    {
        let p = host
            .get_plugin(&plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.clone()))?;
        assert_eq!(p.state, PluginState::Active);
    }

    // Suspend
    host.suspend_plugin(&plugin_id)?;
    {
        let p = host
            .get_plugin(&plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.clone()))?;
        assert_eq!(p.state, PluginState::Suspended);
    }

    // Resume
    host.resume_plugin(&plugin_id)?;
    {
        let p = host
            .get_plugin(&plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.clone()))?;
        assert_eq!(p.state, PluginState::Active);
    }

    // Unload terminates and removes
    host.unload_plugin(&plugin_id)?;
    assert_eq!(host.plugin_count(), 0);
    assert!(host.get_plugin(&plugin_id).is_none());

    Ok(())
}

// 5. Lifecycle event dispatch and broadcast
#[test]
fn test_lifecycle_event_dispatch_and_broadcast() -> TestResult {
    let mut host = WasmPluginHost::default();

    let m1 = make_test_manifest("plugin-a", "Plugin A", vec![PluginCapability::DocumentRead]);
    let b1 = WasmBinaryBuilder::new().with_manifest(&m1)?.build();
    host.load_plugin(&b1, None)?;
    host.init_plugin("plugin-a")?;

    let m2 = make_test_manifest("plugin-b", "Plugin B", vec![PluginCapability::DocumentRead]);
    let b2 = WasmBinaryBuilder::new().with_manifest(&m2)?.build();
    host.load_plugin(&b2, None)?;
    host.init_plugin("plugin-b")?;

    // Broadcast OnDocumentOpened
    let event = LifecycleEvent::OnDocumentOpened {
        doc_id: "report.docx".to_string(),
        doc_type: DocumentType::Word,
    };
    let results = host.broadcast_event(&event);
    assert_eq!(results.len(), 2);
    for (_, res) in results {
        assert!(res.is_ok());
    }

    // Verify both plugins incremented event counters
    let p_a = host
        .get_plugin("plugin-a")
        .ok_or_else(|| PluginError::PluginNotFound("plugin-a".to_string()))?;
    // 1 for on-init + 1 for on-document-opened
    assert_eq!(p_a.dispatched_events_count, 2);

    // Suspend plugin B, broadcast OnBeforeSave
    host.suspend_plugin("plugin-b")?;
    let save_event = LifecycleEvent::OnBeforeSave {
        doc_id: "report.docx".to_string(),
    };
    let broadcast_res = host.broadcast_event(&save_event);
    // Only active plugins receive broadcast
    assert_eq!(broadcast_res.len(), 1);
    assert_eq!(broadcast_res[0].0, "plugin-a");

    Ok(())
}

// 6. Capability gatekeeper enforcement
#[test]
fn test_capability_gatekeeper_enforcement() -> TestResult {
    let mut host = WasmPluginHost::default();
    let mut mock_host = MockDocumentHost::new().with_default_doc();

    // Plugin with ONLY DocumentRead (no DocumentMutate, no RibbonCommand)
    let read_only_manifest = make_test_manifest(
        "read-only-plugin",
        "Read Only Plugin",
        vec![PluginCapability::DocumentRead],
    );
    let bin = WasmBinaryBuilder::new()
        .with_manifest(&read_only_manifest)?
        .build();
    host.load_plugin(&bin, None)?;
    host.init_plugin("read-only-plugin")?;

    // Read query succeeds
    let read_res = host.execute_plugin_action(
        "read-only-plugin",
        PluginAction::QueryCellValue {
            coord: CellCoordinate::new("Sheet1", 1, 1),
        },
        &mut mock_host,
    )?;
    assert_eq!(read_res, ActionResult::CellValue("4200".to_string()));

    // Mutation fails with PermissionDenied
    let mutate_res = host.execute_plugin_action(
        "read-only-plugin",
        PluginAction::SetCellValue {
            coord: CellCoordinate::new("Sheet1", 1, 1),
            value: "9999".to_string(),
        },
        &mut mock_host,
    );
    assert_eq!(
        mutate_res,
        Err(PluginError::PermissionDenied {
            required_capability: "document-mutate".to_string(),
        })
    );

    // Ribbon command fails with PermissionDenied
    let ribbon_res = host.execute_plugin_action(
        "read-only-plugin",
        PluginAction::ExecuteRibbonCommand {
            action: "format-bold".to_string(),
            payload: "{}".to_string(),
        },
        &mut mock_host,
    );
    assert_eq!(
        ribbon_res,
        Err(PluginError::PermissionDenied {
            required_capability: "ribbon-command".to_string(),
        })
    );

    Ok(())
}

// 7. Host document query and mutation APIs
#[test]
fn test_host_document_query_and_mutation_apis() -> TestResult {
    let mut host = WasmPluginHost::default();
    let mut mock_doc = MockDocumentHost::new().with_default_doc();

    let full_manifest = make_test_manifest(
        "full-access-plugin",
        "Full Access Plugin",
        vec![
            PluginCapability::DocumentRead,
            PluginCapability::DocumentMutate,
            PluginCapability::RibbonCommand,
        ],
    );
    let bin = WasmBinaryBuilder::new()
        .with_manifest(&full_manifest)?
        .build();
    host.load_plugin(&bin, None)?;
    host.init_plugin("full-access-plugin")?;

    // Query active selection
    let sel_res = host.execute_plugin_action(
        "full-access-plugin",
        PluginAction::QuerySelection {
            doc_id: "doc-1".to_string(),
        },
        &mut mock_doc,
    )?;
    assert_eq!(
        sel_res,
        ActionResult::Selection(SelectionInfo {
            doc_id: "doc-1".to_string(),
            doc_type: DocumentType::Word,
            start_offset: 0,
            end_offset: 12,
            selected_text: "Hello World!".to_string(),
        })
    );

    // Set cell value
    let coord = CellCoordinate::new("Sheet1", 5, 2);
    let set_val = host.execute_plugin_action(
        "full-access-plugin",
        PluginAction::SetCellValue {
            coord: coord.clone(),
            value: "Gross Revenue".to_string(),
        },
        &mut mock_doc,
    )?;
    assert_eq!(set_val, ActionResult::MutationSuccess);

    // Query updated cell value
    let get_val = host.execute_plugin_action(
        "full-access-plugin",
        PluginAction::QueryCellValue { coord },
        &mut mock_doc,
    )?;
    assert_eq!(
        get_val,
        ActionResult::CellValue("Gross Revenue".to_string())
    );

    // Execute Ribbon command
    let cmd_res = host.execute_plugin_action(
        "full-access-plugin",
        PluginAction::ExecuteRibbonCommand {
            action: "calc:recalculate".to_string(),
            payload: "all".to_string(),
        },
        &mut mock_doc,
    )?;
    assert_eq!(cmd_res, ActionResult::RibbonSuccess(true));
    assert_eq!(mock_doc.executed_commands.len(), 1);
    assert_eq!(
        mock_doc.executed_commands[0],
        ("calc:recalculate".to_string(), "all".to_string())
    );

    Ok(())
}

// 8. Linear memory isolation and bounds checking
#[test]
fn test_linear_memory_isolation_and_bounds_checking() -> TestResult {
    let mut mem = plugin_runtime::LinearMemory::new(1, Some(4))?;
    assert_eq!(mem.page_count(), 1);
    assert_eq!(mem.byte_len(), 65_536);

    // Write and read within bounds
    let write_len = mem.write_utf8(100, "WebAssembly Component Model")?;
    let read_back = mem.read_utf8(100, write_len)?;
    assert_eq!(read_back, "WebAssembly Component Model");

    // Out of bounds write
    let bad_write = mem.write_bytes(65_530, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert!(matches!(
        bad_write,
        Err(PluginError::MemoryOutOfBounds { .. })
    ));

    // Grow memory by 1 page (to 128KB)
    let old_pages = mem.grow(1)?;
    assert_eq!(old_pages, 1);
    assert_eq!(mem.page_count(), 2);
    assert_eq!(mem.byte_len(), 131_072);

    // Now write to offset 65_530 succeeds
    let ok_write = mem.write_bytes(65_530, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert!(ok_write.is_ok());

    // Exceeding max_pages (4)
    let exceed_err = mem.grow(5);
    assert!(matches!(
        exceed_err,
        Err(PluginError::MemoryOutOfBounds { .. })
    ));

    Ok(())
}

// 9. Gas / fuel metering exhaustion trap
#[test]
fn test_gas_fuel_metering_exhaustion_trap() -> TestResult {
    // Host with strict fuel limit of 1000 units
    let mut host = WasmPluginHost::new(1_000);
    let mut mock_doc = MockDocumentHost::new().with_default_doc();

    let manifest = make_test_manifest(
        "gas-limited-plugin",
        "Gas Limited",
        vec![PluginCapability::DocumentRead],
    );
    let bin = WasmBinaryBuilder::new().with_manifest(&manifest)?.build();
    host.load_plugin(&bin, None)?;

    // on-init event consumes 500 units (1000 -> 500)
    host.init_plugin("gas-limited-plugin")?;

    {
        let p = host
            .get_plugin("gas-limited-plugin")
            .ok_or_else(|| PluginError::PluginNotFound("gas-limited-plugin".to_string()))?;
        assert_eq!(p.fuel.remaining, 500);
    }

    // 4 actions consume 100 each (500 -> 100)
    for _ in 0..4 {
        host.execute_plugin_action(
            "gas-limited-plugin",
            PluginAction::QueryCellValue {
                coord: CellCoordinate::new("Sheet1", 1, 1),
            },
            &mut mock_doc,
        )?;
    }

    {
        let p = host
            .get_plugin("gas-limited-plugin")
            .ok_or_else(|| PluginError::PluginNotFound("gas-limited-plugin".to_string()))?;
        assert_eq!(p.fuel.remaining, 100);
    }

    // 5th action succeeds (100 -> 0)
    host.execute_plugin_action(
        "gas-limited-plugin",
        PluginAction::QueryCellValue {
            coord: CellCoordinate::new("Sheet1", 1, 1),
        },
        &mut mock_doc,
    )?;

    // 6th action fails with FuelExhausted!
    let trapped = host.execute_plugin_action(
        "gas-limited-plugin",
        PluginAction::QueryCellValue {
            coord: CellCoordinate::new("Sheet1", 1, 1),
        },
        &mut mock_doc,
    );
    assert_eq!(trapped, Err(PluginError::FuelExhausted { limit: 1_000 }));

    Ok(())
}

// 10. Bidirectional roundtrip WIT flow
#[test]
fn test_bidirectional_roundtrip_wit_flow() -> TestResult {
    let mut host = WasmPluginHost::default();
    let mut mock_doc = MockDocumentHost::new().with_default_doc();

    let manifest = make_test_manifest(
        "com.aether.autocorrect",
        "Auto Correct Assistant",
        vec![
            PluginCapability::DocumentRead,
            PluginCapability::DocumentMutate,
            PluginCapability::RibbonCommand,
        ],
    );

    let bin = WasmBinaryBuilder::new()
        .with_manifest(&manifest)?
        .with_export("on-init", 0, 0)
        .with_export("on-selection-changed", 0, 1)
        .build();

    let plugin_id = host.load_plugin(&bin, None)?;
    host.init_plugin(&plugin_id)?;

    // 1. Host triggers OnSelectionChanged
    let sel = mock_doc.get_active_selection("doc-1")?;
    host.dispatch_event(
        &plugin_id,
        &LifecycleEvent::OnSelectionChanged {
            selection: sel.clone(),
        },
    )?;

    // 2. Plugin queries active selection
    let query_sel = host.execute_plugin_action(
        &plugin_id,
        PluginAction::QuerySelection {
            doc_id: "doc-1".to_string(),
        },
        &mut mock_doc,
    )?;

    if let ActionResult::Selection(info) = query_sel {
        assert_eq!(info.selected_text, "Hello World!");

        // 3. Plugin mutates document by appending corrected text
        host.execute_plugin_action(
            &plugin_id,
            PluginAction::InsertText {
                doc_id: info.doc_id.clone(),
                text: " [Checked by AetherPlugin]".to_string(),
            },
            &mut mock_doc,
        )?;

        // 4. Plugin triggers ribbon status notification
        host.execute_plugin_action(
            &plugin_id,
            PluginAction::ExecuteRibbonCommand {
                action: "status:update".to_string(),
                payload: "Selection formatted".to_string(),
            },
            &mut mock_doc,
        )?;
    } else {
        return Err("Unexpected action result".into());
    }

    // Verify document mutation in mock host
    let para_text = mock_doc.get_paragraph_text("doc-1", 0)?;
    assert!(para_text.ends_with("[Checked by AetherPlugin]"));

    // Verify ribbon command dispatch
    assert_eq!(mock_doc.executed_commands.len(), 1);
    assert_eq!(
        mock_doc.executed_commands[0],
        (
            "status:update".to_string(),
            "Selection formatted".to_string()
        )
    );

    Ok(())
}

// 11. Plugin UDF registry and excel engine formula evaluation bridge
#[test]
fn test_plugin_udf_registry_and_formula_bridge() -> TestResult {
    use excel_engine::formula::ast::{AstNode, Literal};
    use excel_engine::functions::{dispatch_function, EvalContext};
    use excel_engine::grid::CellValue;
    use plugin_runtime::PluginUdfRegistry;
    use std::sync::Arc;

    let mut registry = PluginUdfRegistry::new();

    // Register a custom function: ADD_TAX(amount, rate)
    registry.register_udf(
        "ADD_TAX",
        "Adds tax percentage to an amount",
        2,
        2,
        |args| {
            let amount = match args[0] {
                CellValue::Number(n) => n,
                _ => {
                    return Err(PluginError::HostCallError(
                        "Expected number for amount".into(),
                    ))
                }
            };
            let rate = match args[1] {
                CellValue::Number(n) => n,
                _ => {
                    return Err(PluginError::HostCallError(
                        "Expected number for rate".into(),
                    ))
                }
            };
            Ok(CellValue::Number(amount * (1.0 + rate)))
        },
    );

    assert!(registry.has_udf("ADD_TAX"));
    assert!(registry.has_udf("add_tax"));
    assert_eq!(registry.list_udf_names(), vec!["ADD_TAX".to_string()]);

    let udf_handler = Arc::new(registry).create_excel_udf_handler();
    let ctx = EvalContext::new(None, false).with_udf_handler(udf_handler);

    let args = vec![
        AstNode::Literal(Literal::Number(100.0)),
        AstNode::Literal(Literal::Number(0.08)),
    ];

    let eval_fn = |node: &AstNode, _c: &EvalContext| -> ooxml_opc::error::Result<CellValue> {
        match node {
            AstNode::Literal(Literal::Number(n)) => Ok(CellValue::Number(*n)),
            _ => Ok(CellValue::Empty),
        }
    };

    // Call dynamic function through excel engine dispatch_function
    let res = dispatch_function("ADD_TAX", &args, &ctx, &eval_fn)?;

    assert_eq!(res, CellValue::Number(108.0));

    Ok(())
}
