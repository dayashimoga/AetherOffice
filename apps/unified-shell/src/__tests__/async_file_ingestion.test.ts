import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC, IngestionProgress } from "../ipc";

describe("Sub-Phase 43.2.1: Non-Blocking Async File Ingestion Pipeline", () => {
  beforeEach(() => {
    IPC._resetMockState();
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  describe("Browser / Headless Fallback Pipeline", () => {
    it("reports real-time progress callbacks across discrete stages for docx", async () => {
      const progressUpdates: IngestionProgress[] = [];
      const session = await IPC.openDocumentAsync("C:/Documents/AnnualReport.docx", (p) => {
        progressUpdates.push(p);
      });

      expect(session).toBeDefined();
      expect(session.doc_type).toBe("Word");
      expect(session.title).toBe("AnnualReport.docx");
      expect(session.is_dirty).toBe(false);

      expect(progressUpdates.length).toBe(4);
      expect(progressUpdates[0].stage).toBe("ReadingCentralDirectory");
      expect(progressUpdates[0].percentage).toBe(25.0);
      expect(progressUpdates[1].stage).toBe("ExtractingParts");
      expect(progressUpdates[1].percentage).toBe(50.0);
      expect(progressUpdates[2].stage).toBe("BuildingDom");
      expect(progressUpdates[2].percentage).toBe(75.0);
      expect(progressUpdates[3].stage).toBe("Complete");
      expect(progressUpdates[3].percentage).toBe(100.0);
    });

    it("correctly identifies XLSX format and preserves custom session id", async () => {
      const progressUpdates: IngestionProgress[] = [];
      const session = await IPC.openDocumentAsync(
        "Financial_Q3.xlsx",
        (p) => progressUpdates.push(p),
        "custom-tracking-id-42"
      );

      expect(session.session_id).toBe("custom-tracking-id-42");
      expect(session.doc_type).toBe("Excel");
      expect(session.title).toBe("Financial_Q3.xlsx");
      expect(progressUpdates.length).toBe(4);
      expect(progressUpdates[3].stage).toBe("Complete");
    });

    it("correctly identifies PPTX and PDF formats", async () => {
      const pptSession = await IPC.openDocumentAsync("PitchDeck.pptx");
      expect(pptSession.doc_type).toBe("PowerPoint");
      expect(pptSession.title).toBe("PitchDeck.pptx");

      const pdfSession = await IPC.openDocumentAsync("Whitepaper.pdf");
      expect(pdfSession.doc_type).toBe("PDF");
      expect(pdfSession.title).toBe("Whitepaper.pdf");
    });

    it("supports cooperative cancellation request", async () => {
      const cancelled = await IPC.cancelIngestion("track-session-99");
      expect(cancelled).toBe(true);
    });
  });

  describe("Tauri Native Desktop IPC Invocation Bridge", () => {
    let mockInvoke: ReturnType<typeof vi.fn>;

    beforeEach(() => {
      mockInvoke = vi.fn();
      (window as unknown as { __TAURI_INTERNALS__: { invoke: typeof mockInvoke } }).__TAURI_INTERNALS__ = {
        invoke: mockInvoke,
      };
    });

    it("invokes open_document_async Tauri command with file_path and tracking id", async () => {
      const mockResult = {
        session_id: "sess-async-101",
        doc_type: "Word",
        title: "LargeSpec.docx",
        is_dirty: false,
        file_path: "C:/docs/LargeSpec.docx",
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const res = await IPC.openDocumentAsync("C:/docs/LargeSpec.docx", undefined, "track-spec-1");

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "open_document_async",
        {
          file_path: "C:/docs/LargeSpec.docx",
          session_id: "track-spec-1",
        },
        undefined
      );
      expect(res.session_id).toBe("sess-async-101");
      expect(res.doc_type).toBe("Word");
    });

    it("invokes cancel_ingestion Tauri command with target session id", async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const res = await IPC.cancelIngestion("track-cancel-target");

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "cancel_ingestion",
        {
          session_id: "track-cancel-target",
        },
        undefined
      );
      expect(res).toBe(true);
    });
  });
});

