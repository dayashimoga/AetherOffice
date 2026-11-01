import { describe, it, expect, beforeEach } from "vitest";
import {
  BenchmarkCoordinator,
  type BenchmarkState,
} from "../state/BenchmarkCoordinator";

describe("BenchmarkCoordinator (Sub-Phase 53.2.1)", () => {
  let coordinator: BenchmarkCoordinator;

  beforeEach(() => {
    coordinator = new BenchmarkCoordinator();
  });

  it("test_01_default_state_and_thresholds", () => {
    const state = coordinator.getState();
    expect(state.lastReport).toBeNull();
    expect(state.isRunning).toBe(false);
    expect(state.thresholds.max_cold_start_ms).toBe(100.0);
    expect(state.thresholds.max_idle_memory_mb).toBe(35.0);
    expect(state.thresholds.max_active_session_memory_mb).toBe(250.0);
    expect(state.thresholds.max_keystroke_latency_ms).toBe(1.0);
    expect(state.thresholds.max_frame_render_latency_ms).toBe(8.333);
    expect(state.fuzzExecutionCount).toBe(0);
    expect(state.lastDifferentialResult).toBeNull();
    expect(state.errorMessage).toBeNull();
  });

  it("test_02_run_benchmarks_populates_report", async () => {
    const report = await coordinator.runBenchmarks();
    expect(report.all_passed).toBe(true);
    expect(report.metrics.length).toBe(5);

    const state = coordinator.getState();
    expect(state.isRunning).toBe(false);
    expect(state.lastReport).toEqual(report);

    const metricNames = report.metrics.map((m) => m.metric_name);
    expect(metricNames).toContain("cold_start_latency");
    expect(metricNames).toContain("keystroke_cell_edit_latency");
    expect(metricNames).toContain("frame_render_120fps_budget");
    expect(metricNames).toContain("idle_memory_footprint");
    expect(metricNames).toContain("active_20_session_memory");
  });

  it("test_03_verify_report_against_active_thresholds", async () => {
    const report = await coordinator.runBenchmarks();
    const isCompliant = await coordinator.verifyReport(report);
    expect(isCompliant).toBe(true);
  });

  it("test_04_verify_report_detects_violations_when_tightened", async () => {
    const report = await coordinator.runBenchmarks();

    // Tighten cold start budget below measured value (e.g. 5ms vs measured 42ms)
    coordinator.setCustomThresholds({ max_cold_start_ms: 5.0 });
    const isCompliant = await coordinator.verifyReport(report);
    expect(isCompliant).toBe(false);
  });

  it("test_05_fuzz_buffer_truncated_zip", async () => {
    const input = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    const mutated = await coordinator.fuzzBuffer(input, "truncated-zip");

    expect(mutated.length).toBeLessThan(input.length);
    expect(coordinator.getState().fuzzExecutionCount).toBe(1);
  });

  it("test_06_fuzz_buffer_corrupted_tag", async () => {
    const input = [60, 114, 111, 111, 116, 62]; // "<root>"
    const mutated = await coordinator.fuzzBuffer(input, "corrupted-xml-tag");

    expect(mutated.length).toBeGreaterThan(input.length);
    expect(coordinator.getState().fuzzExecutionCount).toBe(1);
  });

  it("test_07_compare_displays_identical", async () => {
    const display1 = {
      bounds: { x: 0, y: 0, width: 800, height: 600 },
      commands: [{ type: "PushClip" }, { type: "PopClip" }],
    };
    const display2 = { ...display1 };

    const result = await coordinator.compareDisplays(display1, display2);
    expect(result.is_identical).toBe(true);
    expect(result.divergences.length).toBe(0);
    expect(coordinator.getState().lastDifferentialResult).toEqual(result);
  });

  it("test_08_compare_displays_divergent", async () => {
    const display1 = {
      bounds: { x: 0, y: 0, width: 800, height: 600 },
      commands: [{ type: "PushClip" }],
    };
    const display2 = {
      bounds: { x: 0, y: 0, width: 800, height: 600 },
      commands: [{ type: "PushClip" }, { type: "DrawQuad" }],
    };

    const result = await coordinator.compareDisplays(display1, display2);
    expect(result.is_identical).toBe(false);
    expect(result.divergences.length).toBeGreaterThan(0);
    expect(result.divergences[0]).toContain("Command count mismatch");
  });

  it("test_09_set_custom_thresholds", () => {
    coordinator.setCustomThresholds({
      max_idle_memory_mb: 25.0,
      max_keystroke_latency_ms: 0.8,
    });

    const state = coordinator.getState();
    expect(state.thresholds.max_idle_memory_mb).toBe(25.0);
    expect(state.thresholds.max_keystroke_latency_ms).toBe(0.8);
    // Unaltered thresholds remain default
    expect(state.thresholds.max_cold_start_ms).toBe(100.0);
    expect(state.thresholds.max_active_session_memory_mb).toBe(250.0);
  });

  it("test_10_subscribe_receives_state_updates", async () => {
    const history: BenchmarkState[] = [];
    const unsubscribe = coordinator.subscribe((st) => {
      history.push({ ...st });
    });

    expect(history.length).toBe(1);

    await coordinator.runBenchmarks();
    expect(history.length).toBeGreaterThanOrEqual(2);

    unsubscribe();
    const countAfterUnsub = history.length;
    coordinator.setCustomThresholds({ max_cold_start_ms: 50.0 });
    expect(history.length).toBe(countAfterUnsub);
  });
});
