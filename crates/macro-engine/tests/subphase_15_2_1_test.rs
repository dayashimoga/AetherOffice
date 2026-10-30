use macro_engine::{
    Capability, MacroError, MacroSandbox, RiskLevel, SecurityScanner, Severity, VbaEvaluator,
    VbaParser, VirtualOfficeDom,
};

#[test]
fn test_01_scanner_flags_shell_execution() {
    let source = "
    Sub RunExploit()
        Shell \"powershell.exe -Command Write-Host PWNED\", 0
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let report = SecurityScanner::scan_program(&program);
    assert_eq!(report.risk_level, RiskLevel::Critical);
    assert!(report
        .required_capabilities
        .contains(&Capability::SystemCommand));
    assert!(report.findings.iter().any(
        |f| f.severity == Severity::Critical && f.capability == Some(Capability::SystemCommand)
    ));
    assert!(!report.is_safe_to_execute());
}

#[test]
fn test_02_scanner_flags_createobject_wscript() {
    let source = "
    Sub RunPayload()
        Dim wsh As Object
        Set wsh = CreateObject(\"WScript.Shell\")
    End Sub
    ";

    let report = SecurityScanner::scan_source(source);
    assert_eq!(report.risk_level, RiskLevel::Critical);
    assert!(report
        .required_capabilities
        .contains(&Capability::SystemCommand));
    assert!(report
        .findings
        .iter()
        .any(|f| f.message.contains("WScript.Shell")));
}

#[test]
fn test_03_scanner_flags_network_com_object() {
    let source = "
    Sub DownloadData()
        Dim http As Object
        Set http = CreateObject(\"MSXML2.XMLHTTP\")
    End Sub
    ";

    let report = SecurityScanner::scan_source(source);
    assert!(report.risk_level >= RiskLevel::High);
    assert!(report.required_capabilities.contains(&Capability::Network));
    assert!(report
        .findings
        .iter()
        .any(|f| f.capability == Some(Capability::Network)));
}

#[test]
fn test_04_scanner_flags_filesystem_and_environ() {
    let source = "
    Sub ProbeSystem()
        Dim fso As Object
        Set fso = CreateObject(\"Scripting.FileSystemObject\")
        Dim u As String
        u = Environ(\"USERNAME\")
    End Sub
    ";

    let report = SecurityScanner::scan_source(source);
    assert!(report.risk_level >= RiskLevel::High);
    assert!(report.required_capabilities.contains(&Capability::FileIo));
    assert!(report
        .required_capabilities
        .contains(&Capability::Environment));
}

#[test]
fn test_05_scanner_clears_safe_macro() {
    let source = "
    Sub CalculateMonthlyTotal()
        Dim subtotal As Double
        subtotal = 1000 * 1.08
        Range(\"A1\").Value = subtotal
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let report = SecurityScanner::scan_program(&program);
    assert_eq!(report.risk_level, RiskLevel::Safe);
    assert!(report.required_capabilities.is_empty());
    assert!(report.is_safe_to_execute());
}

#[test]
fn test_06_gas_meter_terminates_infinite_loop() {
    let source = "
    Sub InfiniteLoop()
        Dim x As Integer
        Do While True
            x = x + 1
        Loop
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let sandbox = MacroSandbox::new(200); // 200 fuel units
    let mut evaluator = VbaEvaluator::with_sandbox(&mut dom, sandbox);

    let res = evaluator.execute_routine(&program, "InfiniteLoop", &[]);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), MacroError::GasExhausted(_)));
    assert_eq!(evaluator.sandbox.remaining_gas(), 0);
}

#[test]
fn test_07_gas_meter_permits_bounded_loop() {
    let source = "
    Sub BoundedLoop()
        Dim sum As Integer
        Dim i As Integer
        For i = 1 To 20
            sum = sum + i
        Next i
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let sandbox = MacroSandbox::new(50_000);
    let mut evaluator = VbaEvaluator::with_sandbox(&mut dom, sandbox);

    evaluator
        .execute_routine(&program, "BoundedLoop", &[])
        .unwrap();
    // Gas should have been consumed but remaining > 0
    assert!(evaluator.sandbox.remaining_gas() < 50_000);
    assert!(evaluator.sandbox.remaining_gas() > 40_000);
}

#[test]
fn test_08_capability_gatekeeper_blocks_unauthorized_shell() {
    let source = "
    Sub MaliciousCode()
        Shell \"cmd.exe /c whoami\"
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let mut evaluator = VbaEvaluator::new(&mut dom); // Default sandbox denies all external caps

    let res = evaluator.execute_routine(&program, "MaliciousCode", &[]);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), MacroError::PermissionDenied(_)));

    let logs = evaluator.sandbox.audit_log();
    let blocked_entry = logs
        .entries
        .iter()
        .find(|e| e.capability == Capability::SystemCommand && !e.allowed);
    assert!(blocked_entry.is_some());
}

#[test]
fn test_09_capability_gatekeeper_permits_authorized_access() {
    let source = "
    Sub ReadEnvVar()
        Dim user As String
        user = Environ(\"USER\")
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let mut sandbox = MacroSandbox::default();
    sandbox.grant(Capability::Environment); // User explicitly grants Environment permission

    let mut evaluator = VbaEvaluator::with_sandbox(&mut dom, sandbox);
    evaluator
        .execute_routine(&program, "ReadEnvVar", &[])
        .unwrap();

    let logs = evaluator.sandbox.audit_log();
    let allowed_entry = logs
        .entries
        .iter()
        .find(|e| e.capability == Capability::Environment && e.allowed);
    assert!(allowed_entry.is_some());
}

#[test]
fn test_10_audit_log_traceability_and_clearing() {
    let mut sandbox = MacroSandbox::default();

    // 1. Initial check fails
    let res = sandbox.check_capability(Capability::FileIo, "ReadFile");
    assert!(res.is_err());

    // 2. Grant and recheck succeeds
    sandbox.grant(Capability::FileIo);
    let res = sandbox.check_capability(Capability::FileIo, "ReadFile");
    assert!(res.is_ok());

    // 3. Deny and check fails
    sandbox.deny(Capability::FileIo);
    let res = sandbox.check_capability(Capability::FileIo, "ReadFile");
    assert!(res.is_err());

    let log = sandbox.audit_log();
    assert!(log.entries.len() >= 4);

    // Verify chronological timestamps
    for i in 1..log.entries.len() {
        assert!(log.entries[i].timestamp_millis >= log.entries[i - 1].timestamp_millis);
    }

    // Clear
    sandbox.audit_log_mut().clear();
    assert!(sandbox.audit_log().entries.is_empty());
}
