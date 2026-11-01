import { describe, it, expect, beforeEach } from "vitest";
import { DocumentSessionRegistry } from "../state/documentSessionRegistry";
import {
  LruTabSuspensionCoordinator,
  ViewportState,
} from "../state/LruTabSuspensionCoordinator";

describe("Sub-Phase 45.2.1: LRU Tab Suspension & Instant Rehydration", () => {
  let registry: DocumentSessionRegistry;
  let coordinator: LruTabSuspensionCoordinator;

  beforeEach(() => {
    registry = new DocumentSessionRegistry();
    registry.clearAll();
    // Configure max 2 active, max 3 warm
    coordinator = new LruTabSuspensionCoordinator(registry, 2, 3);
  });

  it("initializes with configured quotas and zero sessions", () => {
    expect(coordinator.getMaxActiveTabs()).toBe(2);
    expect(coordinator.getMaxWarmTabs()).toBe(3);
    const breakdown = coordinator.getMemoryBreakdown();
    expect(breakdown).toEqual({ active: 0, warm: 0, cold: 0 });
  });

  it("preserves viewport scroll, zoom, and caret positions", () => {
    registry.createSession("tab-writer-1", "Word", "Proposal.docx");
    const viewport: ViewportState = {
      scrollX: 0,
      scrollY: 850,
      zoom: 1.5,
      caretOffset: 120,
      selectionSummary: "Financial Estimates",
    };

    coordinator.saveViewportState("tab-writer-1", viewport);
    const retrieved = coordinator.getViewportState("tab-writer-1");
    expect(retrieved).toEqual(viewport);
  });

  it("enforces multi-tier active, warm, and cold quotas upon tab switching", () => {
    // Create 6 tabs across formats
    registry.createSession("tab-1", "Word", "Doc 1.docx");
    registry.createSession("tab-2", "Excel", "Calc 1.xlsx");
    registry.createSession("tab-3", "PowerPoint", "Slides 1.pptx");
    registry.createSession("tab-4", "PDF", "Manual.pdf");
    registry.createSession("tab-5", "Notes", "Notes.txt");
    registry.createSession("tab-6", "Word", "Doc 2.docx");

    // Initially all 6 are warm
    expect(coordinator.getMemoryBreakdown()).toEqual({
      active: 0,
      warm: 6,
      cold: 0,
    });

    // Access tab-1 and tab-2 -> both active, 2 warm, 2 cold (maxWarm = 3)
    coordinator.accessSession("tab-1");
    coordinator.accessSession("tab-2");

    let breakdown = coordinator.getMemoryBreakdown();
    expect(breakdown.active).toBe(2);
    expect(breakdown.warm).toBe(2);
    expect(breakdown.cold).toBe(2);

    // Access tab-3, tab-4, tab-5, tab-6 sequentially
    coordinator.accessSession("tab-3");
    coordinator.accessSession("tab-4");
    coordinator.accessSession("tab-5");
    coordinator.accessSession("tab-6");

    breakdown = coordinator.getMemoryBreakdown();
    expect(breakdown.active).toBe(2);
    expect(breakdown.warm).toBe(3);
    expect(breakdown.cold).toBe(1);

    // tab-6 should be active
    expect(registry.getSession("tab-6")?.lifecycleState).toBe("active");
  });

  it("rehydrates cold suspended tabs within 50ms SLA", () => {
    // Create 6 tabs and access them to force tab-1 into cold state
    registry.createSession("tab-1", "Word", "Doc 1.docx");
    registry.createSession("tab-2", "Excel", "Calc 1.xlsx");
    registry.createSession("tab-3", "PowerPoint", "Slides 1.pptx");
    registry.createSession("tab-4", "PDF", "Manual.pdf");
    registry.createSession("tab-5", "Notes", "Notes.txt");
    registry.createSession("tab-6", "Word", "Doc 2.docx");

    coordinator.accessSession("tab-1", {
      scrollX: 0,
      scrollY: 1200,
      zoom: 1.0,
      caretOffset: 45,
    });
    coordinator.accessSession("tab-2");
    coordinator.accessSession("tab-3");
    coordinator.accessSession("tab-4");
    coordinator.accessSession("tab-5");
    coordinator.accessSession("tab-6");

    // tab-1 should now be cold
    const tab1 = registry.getSession("tab-1");
    expect(tab1?.lifecycleState).toBe("cold");

    // Rehydrate tab-1
    const res = coordinator.rehydrateSession("tab-1");
    expect(res.rehydrated).toBe(true);
    expect(res.latencyMs).toBeLessThan(50);
    expect(tab1?.lifecycleState).toBe("active");

    // Verify viewport state was preserved
    const vp = coordinator.getViewportState("tab-1");
    expect(vp?.scrollY).toBe(1200);
    expect(vp?.caretOffset).toBe(45);
  });
});
