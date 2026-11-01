/**
 * ConversionCoordinator.ts
 * GEC-v7.0 Stage 50 (Sub-Phase 50.2.1)
 *
 * Presentation Shell Coordinator for:
 * - Cross-Format Conversion Pipeline & Batch Exporter
 * - Task queue management, format detection, and duration estimation
 * - Single-file and batch execution with real-time status and progress tracking
 * - Format compatibility queries and result caching
 */

import {
  IPC,
  type BatchConversionReport,
  type ConversionEstimateDto,
  type ConversionFormat,
  type ConversionOptions,
  type ConversionResult,
  type ConversionTask,
} from "../ipc";

export {
  type BatchConversionReport,
  type ConversionEstimateDto,
  type ConversionFormat,
  type ConversionOptions,
  type ConversionResult,
  type ConversionTask,
};

export type ConversionTaskStatus =
  | "pending"
  | "converting"
  | "completed"
  | "failed"
  | "cancelled";

export interface QueuedConversionItem {
  id: string;
  sourceName: string;
  sourceFormat: ConversionFormat;
  targetFormat: ConversionFormat;
  inputBytes: Uint8Array;
  options: ConversionOptions;
  status: ConversionTaskStatus;
  result?: ConversionResult;
  errorMessage?: string;
  progressPercent: number;
}

export interface ConversionQueueSummary {
  total: number;
  pending: number;
  converting: number;
  completed: number;
  failed: number;
  cancelled: number;
}

export interface ConversionCoordinatorState {
  items: QueuedConversionItem[];
  isBusy: boolean;
  lastReport: BatchConversionReport | null;
  estimate: ConversionEstimateDto | null;
  errorMessage: string | null;
}

export type ConversionListener = (state: Readonly<ConversionCoordinatorState>) => void;

export class ConversionCoordinator {
  private state: ConversionCoordinatorState = {
    items: [],
    isBusy: false,
    lastReport: null,
    estimate: null,
    errorMessage: null,
  };

  private listeners = new Set<ConversionListener>();

  constructor() {}

  /**
   * Detects the ConversionFormat from a file name or path extension.
   */
  public detectFormat(fileName: string): ConversionFormat | null {
    const ext = fileName.split(".").pop()?.toLowerCase()?.trim();
    if (!ext) return null;
    switch (ext) {
      case "docx":
      case "doc":
        return "Docx";
      case "xlsx":
      case "xls":
        return "Xlsx";
      case "pptx":
      case "ppt":
        return "Pptx";
      case "pdf":
        return "Pdf";
      case "md":
      case "markdown":
        return "Markdown";
      case "csv":
        return "Csv";
      case "html":
      case "htm":
        return "Html";
      case "txt":
      case "text":
        return "PlainText";
      case "json":
        return "Json";
      default:
        return null;
    }
  }

  /**
   * Queries supported target formats for a given source format.
   */
  public async getSupportedTargets(from: ConversionFormat): Promise<ConversionFormat[]> {
    try {
      return await IPC.conversionSupportedFormats(from);
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      return [];
    }
  }

  /**
   * Adds a single conversion item to the queue.
   */
  public enqueue(item: {
    id?: string;
    sourceName: string;
    sourceFormat?: ConversionFormat;
    targetFormat: ConversionFormat;
    inputBytes: Uint8Array;
    options?: Partial<ConversionOptions>;
  }): string {
    const detected = item.sourceFormat ?? this.detectFormat(item.sourceName) ?? "PlainText";
    const id = item.id || `conv-task-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
    const fullOptions: ConversionOptions = {
      csv_delimiter: item.options?.csv_delimiter ?? ",",
      include_headers: item.options?.include_headers ?? true,
      pretty_print: item.options?.pretty_print ?? true,
      page_orientation: item.options?.page_orientation ?? "portrait",
      author: item.options?.author ?? "AetherOffice Universal Converter",
      title: item.options?.title ?? item.sourceName,
    };

    const queuedItem: QueuedConversionItem = {
      id,
      sourceName: item.sourceName,
      sourceFormat: detected,
      targetFormat: item.targetFormat,
      inputBytes: item.inputBytes,
      options: fullOptions,
      status: "pending",
      progressPercent: 0,
    };

    this.state.items.push(queuedItem);
    this.notify();
    return id;
  }

  /**
   * Adds multiple conversion items to the queue.
   */
  public enqueueMultiple(
    items: Array<{
      id?: string;
      sourceName: string;
      sourceFormat?: ConversionFormat;
      targetFormat: ConversionFormat;
      inputBytes: Uint8Array;
      options?: Partial<ConversionOptions>;
    }>
  ): string[] {
    const ids: string[] = [];
    for (const item of items) {
      ids.push(this.enqueue(item));
    }
    return ids;
  }

  /**
   * Cancels a pending item in the queue.
   */
  public cancel(id: string): boolean {
    const item = this.state.items.find((i) => i.id === id);
    if (!item || item.status === "completed" || item.status === "converting") {
      return false;
    }
    item.status = "cancelled";
    this.notify();
    return true;
  }

  /**
   * Clears the entire queue.
   */
  public clearQueue(): void {
    this.state.items = [];
    this.state.estimate = null;
    this.notify();
  }

  /**
   * Clears only completed, failed, or cancelled tasks from the queue.
   */
  public clearCompleted(): void {
    this.state.items = this.state.items.filter(
      (i) => i.status === "pending" || i.status === "converting"
    );
    this.notify();
  }

  /**
   * Retrieves the current snapshot of queued items.
   */
  public getQueue(): QueuedConversionItem[] {
    return [...this.state.items];
  }

  /**
   * Finds a queued conversion item by ID.
   */
  public getItem(id: string): QueuedConversionItem | undefined {
    return this.state.items.find((i) => i.id === id);
  }

  /**
   * Calculates duration and payload estimates for currently pending tasks.
   */
  public async estimateQueue(): Promise<ConversionEstimateDto | null> {
    const pending = this.state.items.filter((i) => i.status === "pending");
    if (pending.length === 0) {
      this.state.estimate = null;
      this.notify();
      return null;
    }

    const totalBytes = pending.reduce((sum, item) => sum + item.inputBytes.byteLength, 0);
    try {
      const estimate = await IPC.conversionEstimateJob(pending.length, totalBytes);
      this.state.estimate = estimate;
      this.notify();
      return estimate;
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      return null;
    }
  }

  /**
   * Executes a single conversion task by item ID.
   */
  public async convertSingle(id: string): Promise<ConversionResult> {
    const item = this.state.items.find((i) => i.id === id);
    if (!item) {
      throw new Error(`Conversion task with ID "${id}" not found`);
    }

    item.status = "converting";
    item.progressPercent = 25;
    this.notify();

    const task: ConversionTask = {
      id: item.id,
      source_format: item.sourceFormat,
      target_format: item.targetFormat,
      input_bytes: Array.from(item.inputBytes),
      options: item.options,
    };

    try {
      const result = await IPC.conversionConvertSingle(task);
      item.result = result;
      item.progressPercent = 100;
      if (result.success) {
        item.status = "completed";
      } else {
        item.status = "failed";
        item.errorMessage = result.error_message || "Conversion failed";
      }
      this.notify();
      return result;
    } catch (err) {
      item.status = "failed";
      item.errorMessage = err instanceof Error ? err.message : String(err);
      item.progressPercent = 100;
      this.notify();
      throw err;
    }
  }

  /**
   * Executes all pending conversion tasks in batch.
   */
  public async convertAll(): Promise<BatchConversionReport> {
    const pendingItems = this.state.items.filter((i) => i.status === "pending");
    if (pendingItems.length === 0) {
      return {
        total_tasks: 0,
        succeeded: 0,
        failed: 0,
        results: [],
        duration_total_ms: 0,
      };
    }

    this.state.isBusy = true;
    for (const item of pendingItems) {
      item.status = "converting";
      item.progressPercent = 10;
    }
    this.notify();

    const tasks: ConversionTask[] = pendingItems.map((item) => ({
      id: item.id,
      source_format: item.sourceFormat,
      target_format: item.targetFormat,
      input_bytes: Array.from(item.inputBytes),
      options: item.options,
    }));

    try {
      const report = await IPC.conversionBatchConvert(tasks);
      this.state.lastReport = report;

      for (const res of report.results) {
        const item = this.state.items.find((i) => i.id === res.task_id);
        if (item) {
          item.result = res;
          item.progressPercent = 100;
          if (res.success) {
            item.status = "completed";
          } else {
            item.status = "failed";
            item.errorMessage = res.error_message || "Conversion failed";
          }
        }
      }

      this.state.isBusy = false;
      this.notify();
      return report;
    } catch (err) {
      this.state.isBusy = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      for (const item of pendingItems) {
        if (item.status === "converting") {
          item.status = "failed";
          item.errorMessage = this.state.errorMessage;
        }
      }
      this.notify();
      throw err;
    }
  }

  /**
   * Gets a summary breakdown of tasks by status.
   */
  public getSummary(): ConversionQueueSummary {
    let pending = 0;
    let converting = 0;
    let completed = 0;
    let failed = 0;
    let cancelled = 0;

    for (const item of this.state.items) {
      switch (item.status) {
        case "pending":
          pending++;
          break;
        case "converting":
          converting++;
          break;
        case "completed":
          completed++;
          break;
        case "failed":
          failed++;
          break;
        case "cancelled":
          cancelled++;
          break;
      }
    }

    return {
      total: this.state.items.length,
      pending,
      converting,
      completed,
      failed,
      cancelled,
    };
  }

  /**
   * Subscribes to coordinator state updates.
   */
  public subscribe(listener: ConversionListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Gets the current immutable state.
   */
  public getState(): Readonly<ConversionCoordinatorState> {
    return { ...this.state };
  }

  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("ConversionCoordinator subscriber error:", err);
      }
    }
  }
}

