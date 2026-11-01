import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC } from "../ipc";

describe("Sub-Phase 43.1.2: Zero-Loss OPC Round-Trip Bridge", () => {
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
    it("preserves package binary buffer via fallback for docx", async () => {
      const originalBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x10, 0x20]);
      const res = await IPC.roundtripOpcBinary("docx", originalBytes, "Updated line of text");
      expect(res).toBeInstanceOf(Uint8Array);
      expect(res.length).toBe(6);
      expect(res[0]).toBe(0x50);
      expect(res[1]).toBe(0x4b);
      expect(res[2]).toBe(0x03);
      expect(res[3]).toBe(0x04);
      expect(res[4]).toBe(0x10);
      expect(res[5]).toBe(0x20);
    });

    it("preserves package binary buffer via fallback for xlsx with json cell map", async () => {
      const originalBytes = [0x50, 0x4b, 0x03, 0x04, 0x99];
      const cellUpdates = JSON.stringify({ A1: "Quarterly Target", B1: "150000" });
      const res = await IPC.roundtripOpcBinary("xlsx", originalBytes, cellUpdates);
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual(originalBytes);
    });

    it("preserves package binary buffer via fallback for pptx", async () => {
      const originalBytes = new Uint8Array([0x50, 0x4b, 0x07, 0x08]);
      const res = await IPC.roundtripOpcBinary("pptx", originalBytes, "{}");
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual([0x50, 0x4b, 0x07, 0x08]);
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

    it("bridges roundtripOpcBinary for docx to tauri command with typed payload", async () => {
      const expectedOut = [0x50, 0x4b, 0x03, 0x04, 0xAA, 0xBB];
      mockInvoke.mockResolvedValueOnce(expectedOut);

      const inputBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x01, 0x02]);
      const updates = "New section header\nUpdated paragraph content.";
      const res = await IPC.roundtripOpcBinary("docx", inputBytes, updates);

      expect(mockInvoke).toHaveBeenCalledWith(
        "roundtrip_opc_binary",
        {
          format: "docx",
          original_bytes: [0x50, 0x4b, 0x03, 0x04, 0x01, 0x02],
          updates_json: updates,
        },
        undefined
      );
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual(expectedOut);
    });

    it("bridges roundtripOpcBinary for xlsx with JSON cell dictionary", async () => {
      const expectedOut = [0x50, 0x4b, 0x03, 0x04, 0xCC, 0xDD];
      mockInvoke.mockResolvedValueOnce(expectedOut);

      const inputBytes = [0x50, 0x4b, 0x03, 0x04];
      const cellMap = JSON.stringify({ C4: "=SUM(C1:C3)", D4: "Certified" });
      const res = await IPC.roundtripOpcBinary("xlsx", inputBytes, cellMap);

      expect(mockInvoke).toHaveBeenCalledWith(
        "roundtrip_opc_binary",
        {
          format: "xlsx",
          original_bytes: inputBytes,
          updates_json: cellMap,
        },
        undefined
      );
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual(expectedOut);
    });

    it("bridges roundtripOpcBinary for pptx with zero data loss guarantee", async () => {
      const expectedOut = [0x50, 0x4b, 0x05, 0x06];
      mockInvoke.mockResolvedValueOnce(expectedOut);

      const inputBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x55, 0x66]);
      const res = await IPC.roundtripOpcBinary("pptx", inputBytes, "");

      expect(mockInvoke).toHaveBeenCalledWith(
        "roundtrip_opc_binary",
        {
          format: "pptx",
          original_bytes: [0x50, 0x4b, 0x03, 0x04, 0x55, 0x66],
          updates_json: "",
        },
        undefined
      );
      expect(res).toBeInstanceOf(Uint8Array);
      expect(Array.from(res)).toEqual(expectedOut);
    });
  });
});

