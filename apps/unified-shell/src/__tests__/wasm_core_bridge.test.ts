import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { IPC, WasmBridge } from "../ipc";

describe("Sub-Phase 43.2.2: WASM Core Compilation for Browser/Test Fallback", () => {
  beforeEach(() => {
    IPC._resetMockState();
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
  });

  describe("WasmCoreEngine Session Orchestration", () => {
    it("creates isolated sessions for Word, Excel, PowerPoint, and PDF", () => {
      const wordSess = WasmBridge.createSession("Word", "StrategicPlan.docx");
      expect(wordSess.session_id).toMatch(/^wasm-sess-/);
      expect(wordSess.doc_type).toBe("Word");
      expect(wordSess.title).toBe("StrategicPlan.docx");
      expect(wordSess.is_dirty).toBe(false);

      const excelSess = WasmBridge.createSession("Excel", "Ledger2026.xlsx");
      expect(excelSess.doc_type).toBe("Excel");

      const pptSess = WasmBridge.createSession("PowerPoint", "Keynote.pptx");
      expect(pptSess.doc_type).toBe("PowerPoint");

      const pdfSess = WasmBridge.createSession("PDF", "Compliance.pdf");
      expect(pdfSess.doc_type).toBe("PDF");
    });

    it("opens document from file path and decodes initial text bytes", () => {
      const textBytes = new TextEncoder().encode("Header line\nParagraph 1 text content.");
      const opened = WasmBridge.openDocument("C:/docs/Notes.md", textBytes);
      expect(opened.doc_type).toBe("Notes");
      expect(opened.title).toBe("Notes.md");
      expect(opened.file_path).toBe("C:/docs/Notes.md");

      const info = WasmBridge.getDocumentInfo(opened.session_id);
      expect(info.session_id).toBe(opened.session_id);
      expect(info.title).toBe("Notes.md");
      expect(info.doc_type).toBe("Notes");
    });

    it("dispatches actions and manages undo/redo history stacks", () => {
      const sess = WasmBridge.createSession("Word", "InteractiveDoc.docx");

      // Dispatch InsertText
      const r1 = WasmBridge.dispatchAction(sess.session_id, {
        type: "InsertText",
        payload: { text: "Hello WebAssembly!" },
      });
      expect(r1.undo_stack_depth).toBe(1);
      expect(r1.redo_stack_depth).toBe(0);
      expect(r1.is_dirty).toBe(true);

      // Dispatch ToggleBold
      const r2 = WasmBridge.dispatchAction(sess.session_id, { type: "ToggleBold" });
      expect(r2.undo_stack_depth).toBe(2);

      // Execute Undo
      WasmBridge.executeUndo(sess.session_id);
      // Execute Redo
      WasmBridge.executeRedo(sess.session_id);

      // Save document
      const savedPath = WasmBridge.saveDocument(sess.session_id, "C:/saved/InteractiveDoc.docx");
      expect(savedPath).toBe("C:/saved/InteractiveDoc.docx");

      const updatedInfo = WasmBridge.getDocumentInfo(sess.session_id);
      expect(updatedInfo.is_dirty).toBe(false);
    });

    it("dispatches spreadsheet grid actions (SetCellValue, SetCellFormula, AddSheet)", () => {
      const sess = WasmBridge.createSession("Excel", "Spreadsheet.xlsx");

      WasmBridge.dispatchAction(sess.session_id, {
        type: "SetCellValue",
        payload: { coord: "A1", value: "Revenue" },
      });
      WasmBridge.dispatchAction(sess.session_id, {
        type: "SetCellFormula",
        payload: { coord: "B1", formula: "SUM(A2:A10)" },
      });
      WasmBridge.dispatchAction(sess.session_id, {
        type: "AddSheet",
        payload: { title: "Q3_Summary" },
      });

      const svg = WasmBridge.exportSvg(sess.session_id);
      expect(svg).toContain("<svg");
      expect(svg).toContain(sess.session_id);
      expect(svg).toContain("Spreadsheet.xlsx [Excel]");
    });

    it("supports lossless OPC roundtrip preservation in browser/test mode", () => {
      const origBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0xde, 0xad]);
      const res = WasmBridge.roundtripOpc("docx", origBytes, "Updates text");
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual([0x50, 0x4b, 0x03, 0x04, 0xde, 0xad]);
    });

    it("closes session and purges from active registry", () => {
      const sess = WasmBridge.createSession("PowerPoint", "Slides.pptx");
      const closed = WasmBridge.closeSession(sess.session_id);
      expect(closed).toBe(true);

      expect(() => WasmBridge.getDocumentInfo(sess.session_id)).toThrowError();
    });
  });

  describe("IPC Engine Hookup", () => {
    it("provides access to WASM core engine via IPC.getWasmEngine()", () => {
      const engine = IPC.getWasmEngine();
      expect(engine).toBeDefined();
      expect(engine).toBe(WasmBridge);

      IPC._resetMockState();
      const newSess = engine.createSession("Word", "ResetTest.docx");
      expect(newSess.title).toBe("ResetTest.docx");
    });
  });
});

