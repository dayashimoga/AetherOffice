import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC } from "../ipc";

describe("Sub-Phase 43.1.1: Complete Tauri Command Registration & Binary Buffer Bridge", () => {
  beforeEach(() => {
    IPC._resetMockState();
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  describe("Browser / Headless Fallback Mode", () => {
    it("reads binary file as Uint8Array via fallback", async () => {
      const result = await IPC.readFileBinary("test_document.docx");
      expect(result).toBeInstanceOf(Uint8Array);
      expect(result.length).toBe(4);
      expect(result[0]).toBe(0x50); // 'P'
      expect(result[1]).toBe(0x4b); // 'K'
      expect(result[2]).toBe(0x03);
      expect(result[3]).toBe(0x04);
    });

    it("saves binary file with custom path via fallback", async () => {
      const payload = new Uint8Array([1, 2, 3, 4, 5]);
      const savedPath = await IPC.saveFileBinary("archive.zip", payload, "/workspace/custom/archive.zip");
      expect(savedPath).toBe("/workspace/custom/archive.zip");
    });

    it("saves binary file with default path if customPath omitted", async () => {
      const payload = [10, 20, 30];
      const savedPath = await IPC.saveFileBinary("data.bin", payload);
      expect(savedPath).toBe("/mock/saved/data.bin");
    });

    it("renders PDF page binary buffer via fallback", async () => {
      const dummyPdf = new Uint8Array([37, 80, 68, 70]); // %PDF
      const rendered = await IPC.renderPdfPageBinary(dummyPdf, 0);
      expect(rendered).toBeInstanceOf(Uint8Array);
      const text = new TextDecoder().decode(rendered);
      expect(text).toContain("PDF_PAGE_MOCK_V1");
    });

    it("computes cryptographic hash via fallback", async () => {
      const data = new Uint8Array([1, 2, 3]);
      const digest = await IPC.cryptoHashBinary(data, "sha256");
      expect(digest).toBe("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    });

    it("syncs collab binary CRDT state vector via fallback", async () => {
      const stateVector = new Uint8Array([0, 1, 2]);
      const syncResult = await IPC.collabSyncBinary("session-collab-101", stateVector);
      expect(syncResult).toBeInstanceOf(Uint8Array);
      const header = new TextDecoder().decode(syncResult.slice(0, 4));
      expect(header).toBe("AECR");
    });

    it("executes sandboxed VBA macro code via fallback", async () => {
      const macroCode = `Sub TestMacro()\n  MsgBox "Hello"\nEnd Sub`;
      const res = await IPC.macroExecuteCode(macroCode, "TestMacro", ["FS_READ"], 5000);
      expect(res.success).toBe(true);
      expect(res.return_value).toBe("MockMacroOutput");
      expect(res.gas_consumed).toBe(120);
    });

    it("queries Copilot assistant for grounded document synthesis via fallback", async () => {
      const prompt = "Summarize the key deliverables of the quarterly review";
      const res = await IPC.copilotQuery("session-writer-99", prompt, "Word");
      expect(res.session_id).toBe("session-writer-99");
      expect(res.prompt).toBe(prompt);
      expect(res.response).toContain("[Aether AI] Verified document synthesis");
      expect(res.confidence).toBeGreaterThanOrEqual(0.9);
    });

    it("parses CSV binary data via fallback data connector", async () => {
      const csvBytes = new TextEncoder().encode("Col1,Col2,Col3\n1,2,3\n");
      const res = await IPC.dataConnectorParseCsv(csvBytes);
      expect(res.row_count).toBe(5);
      expect(res.column_count).toBe(3);
      expect(res.columns).toEqual(["Col1", "Col2", "Col3"]);
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

    it("bridges readFileBinary to tauri read_file_binary command", async () => {
      const rawBytes = [0x50, 0x4b, 0x05, 0x06];
      mockInvoke.mockResolvedValueOnce(rawBytes);

      const result = await IPC.readFileBinary("/documents/report.docx");

      expect(mockInvoke).toHaveBeenCalledWith(
        "read_file_binary",
        { file_path: "/documents/report.docx" },
        undefined
      );
      expect(result).toBeInstanceOf(Uint8Array);
      expect(Array.from(result)).toEqual(rawBytes);
    });

    it("bridges saveFileBinary to tauri save_file_binary command with byte conversion", async () => {
      mockInvoke.mockResolvedValueOnce("/saved/path/budget.xlsx");

      const buffer = new Uint8Array([11, 22, 33]);
      const res = await IPC.saveFileBinary("budget.xlsx", buffer, "/custom/dir/budget.xlsx");

      expect(mockInvoke).toHaveBeenCalledWith(
        "save_file_binary",
        {
          file_name: "budget.xlsx",
          bytes: [11, 22, 33],
          custom_path: "/custom/dir/budget.xlsx",
        },
        undefined
      );
      expect(res).toBe("/saved/path/budget.xlsx");
    });

    it("bridges renderPdfPageBinary passing bytes and page_index", async () => {
      const expectedRendered = [255, 0, 128, 64];
      mockInvoke.mockResolvedValueOnce(expectedRendered);

      const pdfBytes = new Uint8Array([1, 2, 3]);
      const result = await IPC.renderPdfPageBinary(pdfBytes, 2);

      expect(mockInvoke).toHaveBeenCalledWith(
        "render_pdf_page_binary",
        {
          bytes: [1, 2, 3],
          page_index: 2,
        },
        undefined
      );
      expect(result).toBeInstanceOf(Uint8Array);
      expect(Array.from(result)).toEqual(expectedRendered);
    });

    it("bridges cryptoHashBinary passing bytes and algorithm", async () => {
      mockInvoke.mockResolvedValueOnce("abc123hash");

      const bytes = new Uint8Array([4, 5, 6]);
      const digest = await IPC.cryptoHashBinary(bytes, "sha512");

      expect(mockInvoke).toHaveBeenCalledWith(
        "crypto_hash_binary",
        {
          bytes: [4, 5, 6],
          algorithm: "sha512",
        },
        undefined
      );
      expect(digest).toBe("abc123hash");
    });

    it("bridges collabSyncBinary passing session_id and state_vector", async () => {
      mockInvoke.mockResolvedValueOnce([65, 69, 67, 82]); // AECR

      const sv = new Uint8Array([1, 0, 1]);
      const delta = await IPC.collabSyncBinary("session-xyz", sv);

      expect(mockInvoke).toHaveBeenCalledWith(
        "collab_sync_binary",
        {
          session_id: "session-xyz",
          state_vector: [1, 0, 1],
        },
        undefined
      );
      expect(delta).toBeInstanceOf(Uint8Array);
      expect(Array.from(delta)).toEqual([65, 69, 67, 82]);
    });

    it("bridges macroExecuteCode with sandboxing parameters", async () => {
      const mockResult = {
        success: true,
        return_value: 42,
        gas_consumed: 350,
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const code = "Function Calc() As Integer\n  Calc = 42\nEnd Function";
      const res = await IPC.macroExecuteCode(code, "Calc", ["MATH_ONLY"], 10000);

      expect(mockInvoke).toHaveBeenCalledWith(
        "macro_execute_code",
        {
          code,
          routine_name: "Calc",
          capabilities: ["MATH_ONLY"],
          gas_limit: 10000,
        },
        undefined
      );
      expect(res).toEqual(mockResult);
    });

    it("bridges copilotQuery with session, prompt, and doc_type", async () => {
      const mockResult = {
        session_id: "sess-calc-1",
        prompt: "Explain formula in cell B4",
        response: "Cell B4 sums the range A1:A10",
        confidence: 0.99,
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const res = await IPC.copilotQuery("sess-calc-1", "Explain formula in cell B4", "Excel");

      expect(mockInvoke).toHaveBeenCalledWith(
        "copilot_query",
        {
          session_id: "sess-calc-1",
          prompt: "Explain formula in cell B4",
          doc_type: "Excel",
        },
        undefined
      );
      expect(res).toEqual(mockResult);
    });

    it("bridges dataConnectorParseCsv passing bytes and optional delimiter", async () => {
      const mockResult = {
        row_count: 100,
        column_count: 4,
        columns: ["ID", "Name", "Score", "Rank"],
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const csvBytes = new Uint8Array([65, 66, 67]);
      const res = await IPC.dataConnectorParseCsv(csvBytes, 44); // 44 is ','

      expect(mockInvoke).toHaveBeenCalledWith(
        "data_connector_parse_csv",
        {
          bytes: [65, 66, 67],
          delimiter: 44,
        },
        undefined
      );
      expect(res).toEqual(mockResult);
    });
  });
});
