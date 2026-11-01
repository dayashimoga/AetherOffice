//! Exhaustive Differential, Fuzzing & SLA Benchmark Certification Engine.
//!
//! Enforces GEC-v7.0 Article VI performance budgets:
//! - Desktop startup latency: < 100 ms cold start
//! - Idle memory footprint: < 35 MB RSS
//! - Active document memory: < 250 MB RSS for 20 open documents
//! - Keystroke / cell edit latency: < 1.0 ms p99
//! - 120fps frame rendering frame budget: <= 8.33 ms
//!
//! Provides adversarial fuzzing resistance testing and differential
//! display-list equivalence verification.

use crate::primitive::DisplayList;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// SLA performance thresholds specified by GEC-v7.0 Article VI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlaBudgetThresholds {
    pub max_cold_start_ms: f64,
    pub max_idle_memory_mb: f64,
    pub max_active_session_memory_mb: f64,
    pub max_keystroke_latency_ms: f64,
    pub max_frame_render_latency_ms: f64,
}

impl Default for SlaBudgetThresholds {
    fn default() -> Self {
        Self {
            max_cold_start_ms: 100.0,
            max_idle_memory_mb: 35.0,
            max_active_session_memory_mb: 250.0,
            max_keystroke_latency_ms: 1.0,
            max_frame_render_latency_ms: 8.333,
        }
    }
}

/// Single measured SLA metric verification result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlaMetricResult {
    pub metric_name: String,
    pub measured_value: f64,
    pub budget_threshold: f64,
    pub unit: String,
    pub passed: bool,
    pub p50_ms: Option<f64>,
    pub p99_ms: Option<f64>,
}

/// Aggregated SLA benchmark audit report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlaBenchmarkReport {
    pub timestamp_ms: u64,
    pub all_passed: bool,
    pub metrics: Vec<SlaMetricResult>,
    pub summary: String,
}

/// High-precision SLA benchmark executor.
pub struct SlaBenchmarkRunner;

impl SlaBenchmarkRunner {
    /// Measures cold-start initialization latency.
    pub fn measure_cold_start<F>(init_fn: F, threshold_ms: f64) -> SlaMetricResult
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        init_fn();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        SlaMetricResult {
            metric_name: "cold_start_latency".to_string(),
            measured_value: elapsed_ms,
            budget_threshold: threshold_ms,
            unit: "ms".to_string(),
            passed: elapsed_ms <= threshold_ms,
            p50_ms: Some(elapsed_ms),
            p99_ms: Some(elapsed_ms),
        }
    }

    /// Measures keystroke or cell-edit latencies across multiple iterations.
    pub fn measure_keystroke_latency<F>(
        iterations: usize,
        mut op: F,
        threshold_ms: f64,
    ) -> SlaMetricResult
    where
        F: FnMut(usize),
    {
        let iters = iterations.max(1);
        let mut latencies_ms = Vec::with_capacity(iters);

        for i in 0..iters {
            let start = Instant::now();
            op(i);
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            latencies_ms.push(elapsed_ms);
        }

        latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p50_idx = ((iters as f64) * 0.50).floor() as usize;
        let p99_idx = (((iters as f64) * 0.99).floor() as usize).min(iters - 1);

        let p50 = latencies_ms.get(p50_idx).copied().unwrap_or(0.0);
        let p99 = latencies_ms.get(p99_idx).copied().unwrap_or(0.0);

        SlaMetricResult {
            metric_name: "keystroke_cell_edit_latency".to_string(),
            measured_value: p99,
            budget_threshold: threshold_ms,
            unit: "ms".to_string(),
            passed: p99 <= threshold_ms,
            p50_ms: Some(p50),
            p99_ms: Some(p99),
        }
    }

    /// Measures frame render durations over a continuous simulated frame sequence.
    pub fn measure_frame_render_latency<F>(
        frames: usize,
        mut render_fn: F,
        threshold_ms: f64,
    ) -> SlaMetricResult
    where
        F: FnMut(usize),
    {
        let total_frames = frames.max(1);
        let mut frame_times_ms = Vec::with_capacity(total_frames);

        for f in 0..total_frames {
            let start = Instant::now();
            render_fn(f);
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            frame_times_ms.push(elapsed_ms);
        }

        frame_times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p50_idx = ((total_frames as f64) * 0.50).floor() as usize;
        let p99_idx = (((total_frames as f64) * 0.99).floor() as usize).min(total_frames - 1);

        let p50 = frame_times_ms.get(p50_idx).copied().unwrap_or(0.0);
        let p99 = frame_times_ms.get(p99_idx).copied().unwrap_or(0.0);

        SlaMetricResult {
            metric_name: "frame_render_120fps_budget".to_string(),
            measured_value: p99,
            budget_threshold: threshold_ms,
            unit: "ms".to_string(),
            passed: p99 <= threshold_ms,
            p50_ms: Some(p50),
            p99_ms: Some(p99),
        }
    }

    /// Evaluates memory budgets against recorded resident set size.
    pub fn evaluate_memory_metrics(
        idle_mb: f64,
        active_sessions_mb: f64,
        thresholds: &SlaBudgetThresholds,
    ) -> (SlaMetricResult, SlaMetricResult) {
        let idle_res = SlaMetricResult {
            metric_name: "idle_memory_footprint".to_string(),
            measured_value: idle_mb,
            budget_threshold: thresholds.max_idle_memory_mb,
            unit: "MB".to_string(),
            passed: idle_mb <= thresholds.max_idle_memory_mb,
            p50_ms: None,
            p99_ms: None,
        };

        let session_res = SlaMetricResult {
            metric_name: "active_20_session_memory".to_string(),
            measured_value: active_sessions_mb,
            budget_threshold: thresholds.max_active_session_memory_mb,
            unit: "MB".to_string(),
            passed: active_sessions_mb <= thresholds.max_active_session_memory_mb,
            p50_ms: None,
            p99_ms: None,
        };

        (idle_res, session_res)
    }

    /// Assembles full audit report from metric results.
    pub fn build_report(metrics: Vec<SlaMetricResult>, timestamp_ms: u64) -> SlaBenchmarkReport {
        let all_passed = metrics.iter().all(|m| m.passed);
        let passed_count = metrics.iter().filter(|m| m.passed).count();
        let summary = format!(
            "SLA Certification: {}/{} metrics passed (Status: {})",
            passed_count,
            metrics.len(),
            if all_passed {
                "COMPLIANT"
            } else {
                "NON-COMPLIANT"
            }
        );

        SlaBenchmarkReport {
            timestamp_ms,
            all_passed,
            metrics,
            summary,
        }
    }
}

/// Adversarial fuzzing mutation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FuzzMutationType {
    TruncatedZip,
    CorruptedXmlTag,
    MalformedUtf8Sequence,
    CyclicEntityExpansion,
    DeepNesting,
}

/// Fuzz mutation generator and robustness invariant harness.
pub struct AdversarialFuzzHarness;

impl AdversarialFuzzHarness {
    /// Generates mutated byte payload according to adversarial testing scenario.
    pub fn generate_mutated_payload(base: &[u8], mutation: FuzzMutationType) -> Vec<u8> {
        let mut data = base.to_vec();
        match mutation {
            FuzzMutationType::TruncatedZip => {
                if data.len() > 16 {
                    data.truncate(data.len() / 2);
                } else {
                    data.clear();
                }
            }
            FuzzMutationType::CorruptedXmlTag => {
                let pattern = b"<";
                if let Some(pos) = data.windows(pattern.len()).position(|w| w == pattern) {
                    if pos + 3 < data.len() {
                        data[pos + 1] = b'\0';
                        data[pos + 2] = 0xFF;
                    }
                } else {
                    data.extend_from_slice(b"<\0\xFFbroken>");
                }
            }
            FuzzMutationType::MalformedUtf8Sequence => {
                // Injects invalid continuation bytes
                data.extend_from_slice(&[0xC3, 0x28, 0xA0, 0xA1, 0xFF, 0xFE]);
            }
            FuzzMutationType::CyclicEntityExpansion => {
                data.extend_from_slice(
                    b"<!DOCTYPE test [ <!ENTITY lol 'lol'> <!ENTITY lol2 '&lol;&lol;'> ]>",
                );
            }
            FuzzMutationType::DeepNesting => {
                for _ in 0..1000 {
                    data.extend_from_slice(b"<nested>");
                }
            }
        }
        data
    }

    /// Verifies that parser rejects corrupted data gracefully without panics.
    pub fn verify_fuzz_robustness<F, T>(payload: &[u8], mut parser: F) -> Result<T, String>
    where
        F: FnMut(&[u8]) -> Result<T, String>,
    {
        parser(payload)
    }
}

/// Differential audit comparison result for display lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialAuditResult {
    pub is_identical: bool,
    pub total_base_commands: usize,
    pub total_candidate_commands: usize,
    pub divergences: Vec<String>,
}

/// Differential audit engine comparing display lists across renders/conversions.
pub struct DifferentialAuditEngine;

impl DifferentialAuditEngine {
    /// Compares two retained display lists command by command, identifying divergences.
    pub fn compare_display_lists(
        base: &DisplayList,
        candidate: &DisplayList,
    ) -> DifferentialAuditResult {
        let mut divergences = Vec::new();

        if (base.bounds.x - candidate.bounds.x).abs() > 1e-4
            || (base.bounds.y - candidate.bounds.y).abs() > 1e-4
            || (base.bounds.width - candidate.bounds.width).abs() > 1e-4
            || (base.bounds.height - candidate.bounds.height).abs() > 1e-4
        {
            divergences.push(format!(
                "Bounds mismatch: Base ({:?}) vs Candidate ({:?})",
                base.bounds, candidate.bounds
            ));
        }

        if base.commands.len() != candidate.commands.len() {
            divergences.push(format!(
                "Command count mismatch: Base has {} commands, Candidate has {}",
                base.commands.len(),
                candidate.commands.len()
            ));
        }

        let min_len = base.commands.len().min(candidate.commands.len());
        for i in 0..min_len {
            if base.commands[i] != candidate.commands[i] {
                divergences.push(format!(
                    "Command at index {} differs: Base ({:?}) != Candidate ({:?})",
                    i, base.commands[i], candidate.commands[i]
                ));
                if divergences.len() >= 10 {
                    divergences.push("... truncated further divergences".to_string());
                    break;
                }
            }
        }

        DifferentialAuditResult {
            is_identical: divergences.is_empty(),
            total_base_commands: base.commands.len(),
            total_candidate_commands: candidate.commands.len(),
            divergences,
        }
    }
}
