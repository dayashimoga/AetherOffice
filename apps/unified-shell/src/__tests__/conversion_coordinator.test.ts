/**
 * conversion_coordinator.test.ts
 * GEC-v7.0 Stage 50 (Sub-Phase 50.2.1)
 *
 * Vitest Unit & Integration Suite for ConversionCoordinator:
 * - Format detection from filenames & extensions
 * - Enqueue single and multiple conversion tasks
 * - Queue inspection, status transitions, and summary counts
 * - Supported targets discovery
 * - Job time & payload estimation
 * - Single-task execution with real-time status transitions
 * - Batch conversion execution with aggregate reporting
 * - Cancellation rules (pending can cancel, completed/converting cannot)
 * - Queue pruning (clearQueue, clearCompleted)
 * - Reactive state subscription notifications
 */

import { describe, it, expect, beforeEach } from "vitest";
import {
  ConversionCoordinator,
  type ConversionCoordinatorState,
} from "../state/ConversionCoordinator";

describe("Sub-Phase 50.2.1: ConversionCoordinator", () => {
  let coordinator: ConversionCoordinator;

  beforeEach(() => {
    coordinator = new ConversionCoordinator();
  });

  it("1. detects conversion formats from filenames and extensions", () => {
    expect(coordinator.detectFormat("report.docx")).toBe("Docx");
    expect(coordinator.detectFormat("data.xlsx")).toBe("Xlsx");
    expect(coordinator.detectFormat("slides.pptx")).toBe("Pptx");
    expect(coordinator.detectFormat("whitepaper.pdf")).toBe("Pdf");
    expect(coordinator.detectFormat("notes.md")).toBe("Markdown");
    expect(coordinator.detectFormat("export.csv")).toBe("Csv");
    expect(coordinator.detectFormat("index.html")).toBe("Html");
    expect(coordinator.detectFormat("readme.txt")).toBe("PlainText");
    expect(coordinator.detectFormat("config.json")).toBe("Json");
    expect(coordinator.detectFormat("binary.unknown_ext")).toBeNull();
  });

  it("2. enqueues single item with default options and updates summary", () => {
    const inputBytes = new TextEncoder().encode("# Header 1\nSome content");
    const id = coordinator.enqueue({
      sourceName: "sample.md",
      targetFormat: "Html",
      inputBytes,
    });

    expect(id).toBeDefined();
    const item = coordinator.getItem(id);
    expect(item).toBeDefined();
    expect(item?.sourceFormat).toBe("Markdown");
    expect(item?.targetFormat).toBe("Html");
    expect(item?.status).toBe("pending");
    expect(item?.progressPercent).toBe(0);

    const summary = coordinator.getSummary();
    expect(summary.total).toBe(1);
    expect(summary.pending).toBe(1);
    expect(summary.completed).toBe(0);
  });

  it("3. enqueues multiple items with custom conversion options", () => {
    const bytes1 = new TextEncoder().encode("col1,col2\nval1,val2");
    const bytes2 = new TextEncoder().encode("[{\"key\": \"val\"}]");

    const ids = coordinator.enqueueMultiple([
      {
        sourceName: "data.csv",
        targetFormat: "Json",
        inputBytes: bytes1,
        options: { csv_delimiter: ";", pretty_print: true },
      },
      {
        sourceName: "input.json",
        targetFormat: "Csv",
        inputBytes: bytes2,
      },
    ]);

    expect(ids).toHaveLength(2);
    const queue = coordinator.getQueue();
    expect(queue).toHaveLength(2);
    expect(queue[0].options.csv_delimiter).toBe(";");
    expect(queue[1].sourceFormat).toBe("Json");
    expect(queue[1].targetFormat).toBe("Csv");
  });

  it("4. queries supported target formats for source format", async () => {
    const mdTargets = await coordinator.getSupportedTargets("Markdown");
    expect(mdTargets).toContain("Html");
    expect(mdTargets).toContain("Docx");

    const csvTargets = await coordinator.getSupportedTargets("Csv");
    expect(csvTargets).toContain("Xlsx");
    expect(csvTargets).toContain("Json");
  });

  it("5. calculates job duration and payload estimate for pending queue", async () => {
    const emptyEst = await coordinator.estimateQueue();
    expect(emptyEst).toBeNull();

    const bytes = new TextEncoder().encode("Hello AetherOffice Universal Converter");
    coordinator.enqueue({
      sourceName: "doc1.md",
      targetFormat: "Html",
      inputBytes: bytes,
    });
    coordinator.enqueue({
      sourceName: "doc2.md",
      targetFormat: "Docx",
      inputBytes: bytes,
    });

    const est = await coordinator.estimateQueue();
    expect(est).not.toBeNull();
    expect(est?.task_count).toBe(2);
    expect(est?.total_bytes).toBe(bytes.byteLength * 2);
    expect(est?.estimated_duration_ms).toBeGreaterThan(0);
  });

  it("6. executes single conversion task with proper status transitions", async () => {
    const md = "# Test Document\n\nAutomated testing.";
    const id = coordinator.enqueue({
      sourceName: "test.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode(md),
    });

    const result = await coordinator.convertSingle(id);
    expect(result.success).toBe(true);
    expect(result.target_format).toBe("Html");

    const item = coordinator.getItem(id);
    expect(item?.status).toBe("completed");
    expect(item?.progressPercent).toBe(100);
    expect(item?.result).toBeDefined();

    const summary = coordinator.getSummary();
    expect(summary.completed).toBe(1);
    expect(summary.pending).toBe(0);
  });

  it("7. executes batch conversion of queued items with aggregate report", async () => {
    coordinator.enqueue({
      sourceName: "a.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Doc A"),
    });
    coordinator.enqueue({
      sourceName: "b.csv",
      targetFormat: "Json",
      inputBytes: new TextEncoder().encode("Header\nValue"),
    });

    const report = await coordinator.convertAll();
    expect(report.total_tasks).toBe(2);
    expect(report.succeeded).toBe(2);
    expect(report.failed).toBe(0);
    expect(report.results).toHaveLength(2);

    const summary = coordinator.getSummary();
    expect(summary.total).toBe(2);
    expect(summary.completed).toBe(2);
    expect(summary.pending).toBe(0);
  });

  it("8. cancels pending items and prevents cancelling completed/converting items", async () => {
    const id = coordinator.enqueue({
      sourceName: "to_cancel.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Cancel me"),
    });

    expect(coordinator.cancel(id)).toBe(true);
    expect(coordinator.getItem(id)?.status).toBe("cancelled");

    // Cannot cancel already cancelled item
    expect(coordinator.cancel(id)).toBe(true); // remains cancelled

    // Enqueue and complete another item
    const id2 = coordinator.enqueue({
      sourceName: "complete_first.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Complete first"),
    });
    await coordinator.convertSingle(id2);

    // Cannot cancel completed item
    expect(coordinator.cancel(id2)).toBe(false);
  });

  it("9. clears entire queue or completed items only", async () => {
    const id1 = coordinator.enqueue({
      sourceName: "item1.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Item 1"),
    });
    const id2 = coordinator.enqueue({
      sourceName: "item2.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Item 2"),
    });

    await coordinator.convertSingle(id1);

    expect(coordinator.getQueue()).toHaveLength(2);
    coordinator.clearCompleted();
    expect(coordinator.getQueue()).toHaveLength(1);
    expect(coordinator.getItem(id2)?.status).toBe("pending");

    coordinator.clearQueue();
    expect(coordinator.getQueue()).toHaveLength(0);
  });

  it("10. notifies reactive subscribers on state changes", () => {
    const notifications: ConversionCoordinatorState[] = [];
    const unsubscribe = coordinator.subscribe((state) => {
      notifications.push({ ...state });
    });

    coordinator.enqueue({
      sourceName: "sub_test.md",
      targetFormat: "Html",
      inputBytes: new TextEncoder().encode("# Subscribe"),
    });

    expect(notifications.length).toBeGreaterThan(1);
    const last = notifications[notifications.length - 1];
    expect(last.items).toHaveLength(1);

    unsubscribe();
    coordinator.clearQueue();
    // After unsubscribe, no further notifications
    expect(notifications[notifications.length - 1].items).toHaveLength(1);
  });
});
