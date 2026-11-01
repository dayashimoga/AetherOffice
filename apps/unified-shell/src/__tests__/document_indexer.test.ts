import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC } from "../ipc";

describe("Sub-Phase 44.1.1: Incremental Multi-Format Document Indexer", () => {
  beforeEach(() => {
    IPC._resetMockState();
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  describe("Browser / Headless Fallback Indexer Mode", () => {
    it("indexes plain text documents via fallback mock", async () => {
      const indexedCount = await IPC.workspaceIndexText(
        "doc-notes-101",
        "Architecture Notes",
        "High performance zero-loss document indexing and hybrid search."
      );
      expect(indexedCount).toBe(1);
    });

    it("performs hybrid workspace search returning scored matches", async () => {
      const results = await IPC.workspaceSearch("zero-loss indexing", 5);
      expect(results).toBeDefined();
      expect(results.length).toBeGreaterThan(0);

      const top = results[0];
      expect(top.chunk).toBeDefined();
      expect(top.chunk.raw_text).toContain("zero-loss indexing");
      expect(top.semantic_score).toBeGreaterThan(0);
      expect(top.lexical_score).toBeGreaterThan(0);
      expect(top.score).toBeGreaterThan(0);
    });

    it("retrieves workspace index summary statistics", async () => {
      const summary = await IPC.workspaceGetIndexSummary();
      expect(summary.total_documents).toBe(1);
      expect(summary.total_chunks).toBe(1);
      expect(summary.total_estimated_tokens).toBeGreaterThan(0);
      expect(summary.chunks_by_kind.Word).toBe(1);
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

    it("bridges workspaceSearch to Tauri command with typed query parameters", async () => {
      const mockMatches = [
        {
          chunk: {
            id: "c1",
            doc_id: "doc-tauri-1",
            kind: "Word",
            raw_text: "Target match content",
            structural_context: "Doc > Section 1",
            composite_text: "Target match content",
            token_count_estimate: 10,
          },
          semantic_score: 0.92,
          lexical_score: 0.85,
          score: 0.895,
        },
      ];
      mockInvoke.mockResolvedValueOnce(mockMatches);

      const res = await IPC.workspaceSearch("target query", 10);
      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "workspace_search",
        {
          query: "target query",
          top_k: 10,
        },
        undefined
      );
      expect(res).toEqual(mockMatches);
    });

    it("bridges workspaceIndexText to Tauri command", async () => {
      mockInvoke.mockResolvedValueOnce(3);

      const res = await IPC.workspaceIndexText("doc-2", "Spec", "Body content");
      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "workspace_index_text",
        {
          doc_id: "doc-2",
          title: "Spec",
          text: "Body content",
        },
        undefined
      );
      expect(res).toBe(3);
    });

    it("bridges workspaceGetIndexSummary to Tauri command", async () => {
      const mockSummary = {
        total_documents: 5,
        total_chunks: 42,
        chunks_by_kind: { Word: 20, Calc: 12, Slides: 10 },
        total_estimated_tokens: 4200,
      };
      mockInvoke.mockResolvedValueOnce(mockSummary);

      const res = await IPC.workspaceGetIndexSummary();
      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith("workspace_get_index_summary", expect.anything(), undefined);
      expect(res).toEqual(mockSummary);
    });
  });
});
