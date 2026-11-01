use render_core::benchmarks::{
    AdversarialFuzzHarness, DifferentialAuditEngine, FuzzMutationType, SlaBenchmarkReport,
    SlaBenchmarkRunner, SlaBudgetThresholds, SlaMetricResult,
};
use render_core::primitive::{ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D};

#[test]
fn test_01_default_sla_budget_thresholds() {
    let thresholds = SlaBudgetThresholds::default();
    assert_eq!(thresholds.max_cold_start_ms, 100.0);
    assert_eq!(thresholds.max_idle_memory_mb, 35.0);
    assert_eq!(thresholds.max_active_session_memory_mb, 250.0);
    assert_eq!(thresholds.max_keystroke_latency_ms, 1.0);
    assert!((thresholds.max_frame_render_latency_ms - 8.333).abs() < 1e-3);
}

#[test]
fn test_02_cold_start_measurement_and_budget_pass() {
    let metric = SlaBenchmarkRunner::measure_cold_start(
        || {
            // Simulated fast cold start
            let mut v = Vec::with_capacity(100);
            v.push(42);
        },
        100.0,
    );

    assert_eq!(metric.metric_name, "cold_start_latency");
    assert_eq!(metric.unit, "ms");
    assert!(metric.passed);
    assert!(metric.measured_value >= 0.0);
    assert!(metric.measured_value <= 100.0);
    assert!(metric.p50_ms.is_some());
    assert!(metric.p99_ms.is_some());
}

#[test]
fn test_03_keystroke_cell_edit_latency_distribution() {
    let mut total_chars = 0;
    let metric = SlaBenchmarkRunner::measure_keystroke_latency(
        100,
        |i| {
            total_chars += i;
        },
        5.0, // generous debug threshold to prevent timer jitter
    );

    assert_eq!(metric.metric_name, "keystroke_cell_edit_latency");
    assert_eq!(metric.unit, "ms");
    assert!(metric.passed);
    assert!(total_chars > 0);
    assert!(metric.p50_ms.unwrap() <= metric.p99_ms.unwrap());
}

#[test]
fn test_04_120fps_frame_render_budget_evaluation() {
    let mut rendered_frames = 0;
    let metric = SlaBenchmarkRunner::measure_frame_render_latency(
        60,
        |_frame_idx| {
            rendered_frames += 1;
        },
        8.333,
    );

    assert_eq!(metric.metric_name, "frame_render_120fps_budget");
    assert_eq!(metric.unit, "ms");
    assert_eq!(rendered_frames, 60);
    assert!(metric.passed);
    assert!(metric.measured_value < 8.333);
}

#[test]
fn test_05_memory_budget_compliance_and_violation() {
    let thresholds = SlaBudgetThresholds::default();

    // 1. Compliant scenario
    let (idle_ok, sess_ok) = SlaBenchmarkRunner::evaluate_memory_metrics(28.5, 195.0, &thresholds);
    assert!(idle_ok.passed);
    assert_eq!(idle_ok.measured_value, 28.5);
    assert!(sess_ok.passed);
    assert_eq!(sess_ok.measured_value, 195.0);

    // 2. Non-compliant scenario
    let (idle_fail, sess_fail) =
        SlaBenchmarkRunner::evaluate_memory_metrics(42.0, 310.0, &thresholds);
    assert!(!idle_fail.passed);
    assert_eq!(idle_fail.measured_value, 42.0);
    assert!(!sess_fail.passed);
    assert_eq!(sess_fail.measured_value, 310.0);
}

#[test]
fn test_06_sla_benchmark_report_aggregation() {
    let metrics = vec![
        SlaMetricResult {
            metric_name: "cold_start".to_string(),
            measured_value: 45.0,
            budget_threshold: 100.0,
            unit: "ms".to_string(),
            passed: true,
            p50_ms: Some(45.0),
            p99_ms: Some(45.0),
        },
        SlaMetricResult {
            metric_name: "idle_memory".to_string(),
            measured_value: 30.0,
            budget_threshold: 35.0,
            unit: "MB".to_string(),
            passed: true,
            p50_ms: None,
            p99_ms: None,
        },
    ];

    let report = SlaBenchmarkRunner::build_report(metrics, 1_700_000_000_000);
    assert!(report.all_passed);
    assert_eq!(report.metrics.len(), 2);
    assert!(report.summary.contains("COMPLIANT"));
    assert!(report.summary.contains("2/2 metrics passed"));

    // With a failed metric
    let failing_metrics = vec![SlaMetricResult {
        metric_name: "keystroke".to_string(),
        measured_value: 2.5,
        budget_threshold: 1.0,
        unit: "ms".to_string(),
        passed: false,
        p50_ms: Some(1.2),
        p99_ms: Some(2.5),
    }];
    let fail_report = SlaBenchmarkRunner::build_report(failing_metrics, 1_700_000_000_000);
    assert!(!fail_report.all_passed);
    assert!(fail_report.summary.contains("NON-COMPLIANT"));
}

#[test]
fn test_07_adversarial_fuzz_mutations_and_no_panics() {
    let base_xml = b"<?xml version=\"1.0\"?><root><item id=\"1\">Hello World</item></root>";

    let mutations = [
        FuzzMutationType::TruncatedZip,
        FuzzMutationType::CorruptedXmlTag,
        FuzzMutationType::MalformedUtf8Sequence,
        FuzzMutationType::CyclicEntityExpansion,
        FuzzMutationType::DeepNesting,
    ];

    for mutation in mutations {
        let mutated = AdversarialFuzzHarness::generate_mutated_payload(base_xml, mutation);
        assert!(!mutated.is_empty() || mutation == FuzzMutationType::TruncatedZip);

        // Simulated robust parser that returns Err on corrupted bytes
        let result = AdversarialFuzzHarness::verify_fuzz_robustness(&mutated, |bytes| {
            if bytes.is_empty() || !bytes.ends_with(b"</root>") {
                return Err("Truncated payload or unclosed root tag".to_string());
            }
            if bytes.contains(&0xFF) || bytes.contains(&b'\0') {
                return Err("Invalid byte sequence".to_string());
            }
            if bytes.starts_with(b"<!DOCTYPE") {
                return Err("Forbidden entity expansion".to_string());
            }
            if bytes.len() > 1000 {
                return Err("Exceeded nesting depth limit".to_string());
            }
            // Parse as UTF-8
            std::str::from_utf8(bytes)
                .map(|s| s.to_string())
                .map_err(|e| e.to_string())
        });

        // Must return Err without unhandled panic
        assert!(
            result.is_err(),
            "Mutation {:?} should be rejected by parser",
            mutation
        );
    }
}

#[test]
fn test_08_differential_audit_identical_display_lists() {
    let mut dl1 = DisplayList {
        bounds: Rect2D::new(0.0, 0.0, 800.0, 600.0),
        ..Default::default()
    };
    dl1.commands.push(DisplayCommand::PushClip(Rect2D::new(
        0.0, 0.0, 800.0, 600.0,
    )));
    dl1.commands
        .push(DisplayCommand::DrawQuad(QuadInstance::solid(
            Rect2D::new(0.0, 0.0, 100.0, 100.0),
            ColorRgba::BLACK,
        )));
    dl1.commands.push(DisplayCommand::PopClip);

    let dl2 = dl1.clone();

    let diff = DifferentialAuditEngine::compare_display_lists(&dl1, &dl2);
    assert!(diff.is_identical);
    assert_eq!(diff.total_base_commands, 3);
    assert_eq!(diff.total_candidate_commands, 3);
    assert!(diff.divergences.is_empty());
}

#[test]
fn test_09_differential_audit_detects_divergences() {
    let mut dl1 = DisplayList {
        bounds: Rect2D::new(0.0, 0.0, 800.0, 600.0),
        ..Default::default()
    };
    dl1.commands.push(DisplayCommand::PushClip(Rect2D::new(
        0.0, 0.0, 800.0, 600.0,
    )));

    let mut dl2 = DisplayList {
        bounds: Rect2D::new(0.0, 0.0, 1024.0, 768.0),
        ..Default::default()
    };
    // Command count and content divergence
    dl2.commands.push(DisplayCommand::PopClip);

    let diff = DifferentialAuditEngine::compare_display_lists(&dl1, &dl2);
    assert!(!diff.is_identical);
    assert_eq!(diff.divergences.len(), 2);
    assert!(diff.divergences[0].contains("Bounds mismatch"));
    assert!(diff.divergences[1].contains("Command at index 0 differs"));
}

#[test]
fn test_10_serde_roundtrip_all_benchmark_structures() {
    let thresholds = SlaBudgetThresholds::default();
    let json_thresh = serde_json::to_string(&thresholds).expect("Serialize thresholds");
    let de_thresh: SlaBudgetThresholds =
        serde_json::from_str(&json_thresh).expect("Deserialize thresholds");
    assert_eq!(thresholds, de_thresh);

    let metric = SlaMetricResult {
        metric_name: "test_metric".to_string(),
        measured_value: 12.34,
        budget_threshold: 50.0,
        unit: "ms".to_string(),
        passed: true,
        p50_ms: Some(10.0),
        p99_ms: Some(12.34),
    };
    let json_metric = serde_json::to_string(&metric).expect("Serialize metric");
    let de_metric: SlaMetricResult =
        serde_json::from_str(&json_metric).expect("Deserialize metric");
    assert_eq!(metric, de_metric);

    let report = SlaBenchmarkReport {
        timestamp_ms: 1_700_000_000_000,
        all_passed: true,
        metrics: vec![metric],
        summary: "Audit Report Pass".to_string(),
    };
    let json_report = serde_json::to_string(&report).expect("Serialize report");
    let de_report: SlaBenchmarkReport =
        serde_json::from_str(&json_report).expect("Deserialize report");
    assert_eq!(report, de_report);
}
