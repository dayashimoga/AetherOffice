/**
 * BenchmarkCoordinator.ts
 * GEC-v7.0 Stage 53 (Sub-Phase 53.2.1)
 *
 * Presentation Shell Coordinator for:
 * - GEC-v7.0 Article VI SLA performance budgets & metrics
 * - Cold-start latency, memory footprint, keystroke/cell edit latency, and 120fps frame rendering
 * - Adversarial fuzz testing invariants & error safety
 * - Differential display list audit comparison
 */

import {
  IPC,
  type DifferentialAuditResultDto,
  type FuzzMutationTypeDto,
  type SlaBenchmarkReportDto,
  type SlaBudgetThresholdsDto,
  type SlaMetricResultDto,
} from "../ipc";

export {
  type DifferentialAuditResultDto,
  type FuzzMutationTypeDto,
  type SlaBenchmarkReportDto,
  type SlaBudgetThresholdsDto,
  type SlaMetricResultDto,
};

export interface BenchmarkState {
  lastReport: SlaBenchmarkReportDto | null;
  isRunning: boolean;
  thresholds: SlaBudgetThresholdsDto;
  lastDifferentialResult: DifferentialAuditResultDto | null;
  fuzzExecutionCount: number;
  errorMessage: string | null;
}

export type BenchmarkListener = (state: Readonly<BenchmarkState>) => void;

export class BenchmarkCoordinator {
  private state: BenchmarkState;
  private listeners = new Set<BenchmarkListener>();

  constructor() {
    this.state = {
      lastReport: null,
      isRunning: false,
      thresholds: {
        max_cold_start_ms: 100.0,
        max_idle_memory_mb: 35.0,
        max_active_session_memory_mb: 250.0,
        max_keystroke_latency_ms: 1.0,
        max_frame_render_latency_ms: 8.333,
      },
      lastDifferentialResult: null,
      fuzzExecutionCount: 0,
      errorMessage: null,
    };
  }

  public getState(): Readonly<BenchmarkState> {
    return this.state;
  }

  public subscribe(listener: BenchmarkListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  /**
   * Executes the full Rust SLA benchmark suite and evaluates against thresholds.
   */
  public async runBenchmarks(): Promise<SlaBenchmarkReportDto> {
    this.state = { ...this.state, isRunning: true, errorMessage: null };
    this.notify();

    try {
      const report = await IPC.slaRunBenchmarks();
      const compliant = await IPC.slaVerifyBudgets(report, this.state.thresholds);

      const finalReport: SlaBenchmarkReportDto = {
        ...report,
        all_passed: compliant,
      };

      this.state = {
        ...this.state,
        lastReport: finalReport,
        isRunning: false,
        errorMessage: null,
      };
      this.notify();
      return finalReport;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, isRunning: false, errorMessage: msg };
      this.notify();
      throw err;
    }
  }

  /**
   * Verifies an SLA benchmark report against active thresholds.
   */
  public async verifyReport(report?: SlaBenchmarkReportDto): Promise<boolean> {
    const target = report ?? this.state.lastReport;
    if (!target) return false;
    return IPC.slaVerifyBudgets(target, this.state.thresholds);
  }

  /**
   * Mutates a document buffer using the adversarial fuzzing harness.
   */
  public async fuzzBuffer(
    payload: Uint8Array | number[],
    mutation: FuzzMutationTypeDto | string
  ): Promise<number[]> {
    try {
      const mutated = await IPC.fuzzTestDocumentBuffer(payload, mutation);
      this.state = {
        ...this.state,
        fuzzExecutionCount: this.state.fuzzExecutionCount + 1,
        errorMessage: null,
      };
      this.notify();
      return mutated;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, errorMessage: msg };
      this.notify();
      throw err;
    }
  }

  /**
   * Evaluates differential parity between two display lists.
   */
  public async compareDisplays(
    baseDisplay: unknown,
    candidateDisplay: unknown
  ): Promise<DifferentialAuditResultDto> {
    try {
      const result = await IPC.differentialAuditEvaluate(baseDisplay, candidateDisplay);
      this.state = {
        ...this.state,
        lastDifferentialResult: result,
        errorMessage: null,
      };
      this.notify();
      return result;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, errorMessage: msg };
      this.notify();
      throw err;
    }
  }

  /**
   * Updates SLA budget thresholds.
   */
  public setCustomThresholds(thresholds: Partial<SlaBudgetThresholdsDto>): void {
    this.state = {
      ...this.state,
      thresholds: {
        ...this.state.thresholds,
        ...thresholds,
      },
    };
    this.notify();
  }
}
