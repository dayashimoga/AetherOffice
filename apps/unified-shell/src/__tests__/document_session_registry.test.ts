import { describe, it, expect, beforeEach } from "vitest";
import { DocumentSessionRegistry, documentSessionRegistry } from "../state/documentSessionRegistry";

describe("DocumentSessionRegistry (Sub-Phase 42.1.1)", () => {
  let registry: DocumentSessionRegistry;

  beforeEach(() => {
    registry = new DocumentSessionRegistry();
  });

  it("registers initial default sessions matching legacy singletons", () => {
    expect(documentSessionRegistry).toBeDefined();
    const sessions = registry.listSessions();
    expect(sessions.length).toBeGreaterThanOrEqual(5);

    const writerSession = registry.getSession("tab-writer-1");
    expect(writerSession).toBeDefined();
    expect(writerSession?.docType).toBe("Word");
    expect(writerSession?.instance.docType).toBe("Word");

    const calcSession = registry.getSession("tab-calc-1");
    expect(calcSession).toBeDefined();
    expect(calcSession?.docType).toBe("Excel");
  });

  it("creates independent concurrent instances of the same document format", () => {
    const s1 = registry.createSession("doc-word-1", "Word", "Proposal.docx");
    const s2 = registry.createSession("doc-word-2", "Word", "Contract.docx");

    expect(s1.id).toBe("doc-word-1");
    expect(s2.id).toBe("doc-word-2");

    const state1 = registry.getWriterState("doc-word-1")!;
    const state2 = registry.getWriterState("doc-word-2")!;

    expect(state1).toBeDefined();
    expect(state2).toBeDefined();
    expect(state1).not.toBe(state2);

    // Mutate state1 without affecting state2
    state1.doc.title = "Updated Proposal";
    state1.doc.pageSize = "a4";

    expect(state1.doc.title).toBe("Updated Proposal");
    expect(state1.doc.pageSize).toBe("a4");
    expect(state2.doc.title).not.toBe("Updated Proposal");
    expect(state2.doc.pageSize).toBe("letter");
  });

  it("creates independent concurrent spreadsheet instances", () => {
    const s1 = registry.createSession("calc-1", "Excel", "Budget.xlsx");
    const s2 = registry.createSession("calc-2", "Excel", "Forecast.xlsx");
    expect(s1.title).toBe("Budget.xlsx");
    expect(s2.title).toBe("Forecast.xlsx");

    const calc1 = registry.getCalcState("calc-1")!;
    const calc2 = registry.getCalcState("calc-2")!;

    expect(calc1).not.toBe(calc2);

    calc1.setCellValue("A1", "1000");
    calc1.setCellValue("A2", "2000");

    expect(Number(calc1.getCellValue("A1"))).toBe(1000);
    expect(calc2.getCellValue("A1")).toBeUndefined();
  });

  it("manages lifecycle states (active, warm, cold) on session switch", () => {
    registry.createSession("doc-1", "Word", "Doc1.docx");
    registry.createSession("doc-2", "Excel", "Doc2.xlsx");

    registry.setActiveSession("doc-1");
    expect(registry.getActiveSessionId()).toBe("doc-1");
    expect(registry.getSession("doc-1")?.lifecycleState).toBe("active");

    registry.setActiveSession("doc-2");
    expect(registry.getActiveSessionId()).toBe("doc-2");
    expect(registry.getSession("doc-2")?.lifecycleState).toBe("active");
    expect(registry.getSession("doc-1")?.lifecycleState).toBe("warm");
  });

  it("handles closing an active session and promotes remaining session", () => {
    registry.createSession("sess-a", "Word", "A.docx");
    registry.createSession("sess-b", "PowerPoint", "B.pptx");

    registry.setActiveSession("sess-a");
    expect(registry.getActiveSessionId()).toBe("sess-a");

    const closed = registry.closeSession("sess-a");
    expect(closed).toBe(true);
    expect(registry.getSession("sess-a")).toBeUndefined();
    expect(registry.getActiveSessionId()).toBe("sess-b");
    expect(registry.getSession("sess-b")?.lifecycleState).toBe("active");
  });

  it("filters sessions by document type", () => {
    registry.createSession("w1", "Word");
    registry.createSession("w2", "Word");
    registry.createSession("x1", "Excel");
    registry.createSession("p1", "PowerPoint");

    const words = registry.listSessionsByDocType("Word");
    expect(words.length).toBeGreaterThanOrEqual(2);
    expect(words.every((s) => s.docType === "Word")).toBe(true);

    const excels = registry.listSessionsByDocType("Excel");
    expect(excels.some((s) => s.id === "x1")).toBe(true);
  });

  it("supports getOrCreateSession idempotency", () => {
    const s1 = registry.getOrCreateSession("unique-id", "PDF", "Manual.pdf", "/docs/manual.pdf");
    expect(s1.title).toBe("Manual.pdf");
    expect(s1.filePath).toBe("/docs/manual.pdf");

    const s2 = registry.getOrCreateSession("unique-id", "PDF");
    expect(s2).toBe(s1);
  });
});
