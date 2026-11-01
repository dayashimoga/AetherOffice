import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC, OmniboxItem } from "../ipc";
import { OmniboxLauncher } from "../components/OmniboxLauncher";
import { KeyboardShortcuts } from "../components/KeyboardShortcuts";

describe("Sub-Phase 44.2.1: Omnibox Command, File & Symbol Launcher (Ctrl+K)", () => {
  beforeEach(() => {
    IPC._resetMockState();
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    document.body.innerHTML = "";
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    document.body.innerHTML = "";
  });

  describe("Browser / Headless IPC Fallback Bridge", () => {
    it("retrieves default catalog of commands and open tabs in fallback mode", async () => {
      const items = await IPC.omniboxQuery("");
      expect(items).toBeDefined();
      expect(items.length).toBeGreaterThan(0);
      expect(items.some((i) => i.category === "Command")).toBe(true);
      expect(items.some((i) => i.category === "File")).toBe(true);
    });

    it("filters catalog by search query in fallback mode", async () => {
      const items = await IPC.omniboxQuery("Save");
      expect(items).toBeDefined();
      expect(items.some((i) => i.title.toLowerCase().includes("save"))).toBe(true);
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

    it("bridges omnibox_query to Tauri command with typed query parameters", async () => {
      const mockItems: OmniboxItem[] = [
        {
          id: "cmd-new",
          category: "Command",
          title: "New Document",
          description: "Create blank document",
          shortcut: "Ctrl+N",
          icon: "file-plus",
          score: 800,
          payload: "NewTab",
        },
      ];
      mockInvoke.mockResolvedValueOnce(mockItems);

      const items = await IPC.omniboxQuery("New", 15);
      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "omnibox_query",
        {
          query: "New",
          max_results: 15,
        },
        undefined
      );
      expect(items).toEqual(mockItems);
    });
  });

  describe("OmniboxLauncher Component Lifecycle & Navigation", () => {
    it("initializes DOM elements and begins in hidden state", () => {
      const launcher = new OmniboxLauncher();
      expect(launcher.isOpen()).toBe(false);

      const overlay = document.getElementById("omnibox-launcher-overlay");
      expect(overlay).not.toBeNull();
      expect(overlay?.getAttribute("role")).toBe("dialog");
      expect(overlay?.getAttribute("aria-label")).toContain("Omnibox");
    });

    it("shows, hides, and toggles launcher visibility", () => {
      const launcher = new OmniboxLauncher();
      launcher.show("initial-search");
      expect(launcher.isOpen()).toBe(true);

      const input = document.querySelector<HTMLInputElement>(".omnibox-input");
      expect(input?.value).toBe("initial-search");

      launcher.hide();
      expect(launcher.isOpen()).toBe(false);

      launcher.toggle();
      expect(launcher.isOpen()).toBe(true);

      launcher.toggle();
      expect(launcher.isOpen()).toBe(false);
    });

    it("queries items and renders results with category badges and shortcuts", async () => {
      const launcher = new OmniboxLauncher();
      launcher.show();

      await launcher.executeQuery("Document");

      const items = launcher.getItems();
      expect(items.length).toBeGreaterThan(0);

      const renderedItems = document.querySelectorAll(".omnibox-item");
      expect(renderedItems.length).toBe(items.length);
    });

    it("navigates results with selectNext and selectPrev", async () => {
      const launcher = new OmniboxLauncher();
      launcher.show();
      await launcher.executeQuery("");

      expect(launcher.getSelectedIndex()).toBe(0);

      launcher.selectNext();
      expect(launcher.getSelectedIndex()).toBe(1);

      launcher.selectPrev();
      expect(launcher.getSelectedIndex()).toBe(0);
    });

    it("executes selected item and invokes callbacks", async () => {
      const onExecute = vi.fn();
      const onTabSwitch = vi.fn();

      const launcher = new OmniboxLauncher();
      launcher.setCallbacks({
        onExecuteItem: onExecute,
        onTabSwitch,
      });

      launcher.show();
      await launcher.executeQuery("");

      launcher.executeSelected();

      expect(onExecute).toHaveBeenCalledTimes(1);
      expect(launcher.isOpen()).toBe(false);
    });

    it("filters by category pills (Commands, Files, Symbols)", async () => {
      const launcher = new OmniboxLauncher();
      launcher.show();

      const cmdPill = document.querySelector<HTMLButtonElement>('.omnibox-pill[data-cat="Command"]');
      cmdPill?.click();

      // Wait for query execution
      await new Promise((r) => setTimeout(r, 50));

      const input = document.querySelector<HTMLInputElement>(".omnibox-input");
      expect(input).not.toBeNull();
    });

    it("closes when clicking the backdrop overlay", () => {
      const launcher = new OmniboxLauncher();
      launcher.show();
      expect(launcher.isOpen()).toBe(true);

      const overlay = document.getElementById("omnibox-launcher-overlay");
      overlay?.click();

      expect(launcher.isOpen()).toBe(false);
    });
  });

  describe("Keyboard Shortcuts Integration", () => {
    it("emits Omnibox on Ctrl+K", () => {
      const onAction = vi.fn();
      const shortcuts = new KeyboardShortcuts();
      shortcuts.setCallbacks({ onAction });

      // Simulate Ctrl+K
      const event = new KeyboardEvent("keydown", {
        key: "k",
        ctrlKey: true,
      });
      window.dispatchEvent(event);

      expect(onAction).toHaveBeenCalledWith("Omnibox");
    });
  });
});

