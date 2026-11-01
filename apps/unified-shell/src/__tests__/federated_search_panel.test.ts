import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { IPC, FederatedSearchMatch, FederatedReplaceSummary } from "../ipc";
import { FederatedSearchPanel } from "../components/FederatedSearchPanel";
import { KeyboardShortcuts } from "../components/KeyboardShortcuts";

describe("Sub-Phase 44.1.2: Cross-Document Federated Search & Replace Panel", () => {
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
    it("executes federated search in fallback mode", async () => {
      const hits = await IPC.federatedSearch("mock query");
      expect(hits).toBeDefined();
      expect(hits.length).toBeGreaterThan(0);
      expect(hits[0].match_id).toBe("match-mock-1");
      expect(hits[0].doc_type).toBe("Word");
      expect(hits[0].snippet).toContain("mock query");
    });

    it("executes federated replace single match in fallback mode", async () => {
      const replaced = await IPC.federatedReplaceMatch(
        "sess-mock-1",
        "mock query",
        "replacement text"
      );
      expect(replaced).toBe(1);
    });

    it("executes federated replace all across workspace in fallback mode", async () => {
      const summary: FederatedReplaceSummary = await IPC.federatedReplaceAll(
        "old term",
        "new term",
        { case_sensitive: true, whole_word: true }
      );
      expect(summary.total_replacements).toBe(2);
      expect(summary.documents_affected).toBe(1);
      expect(summary.replacements_by_session["sess-mock-1"]).toBe(2);
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

    it("bridges federated_search to Tauri command with options", async () => {
      const mockMatches: FederatedSearchMatch[] = [
        {
          match_id: "match-1",
          session_id: "sess-word-1",
          doc_type: "Word",
          document_title: "Policy.docx",
          location: { type: "Word", details: { line: 5, char_offset: 12 } },
          snippet: "Compliance standards for 2026",
          matched_text: "Compliance",
          start_offset: 0,
          end_offset: 10,
        },
      ];
      mockInvoke.mockResolvedValueOnce(mockMatches);

      const hits = await IPC.federatedSearch("Compliance", {
        case_sensitive: true,
        whole_word: true,
        use_regex: false,
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "federated_search",
        {
          query: "Compliance",
          case_sensitive: true,
          whole_word: true,
          use_regex: false,
        },
        undefined
      );
      expect(hits).toEqual(mockMatches);
    });

    it("bridges federated_replace_match to Tauri command", async () => {
      mockInvoke.mockResolvedValueOnce(1);

      const count = await IPC.federatedReplaceMatch(
        "sess-1",
        "old_value",
        "new_value",
        { case_sensitive: false }
      );

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "federated_replace_match",
        {
          session_id: "sess-1",
          query: "old_value",
          replacement: "new_value",
          case_sensitive: false,
          whole_word: false,
          use_regex: false,
        },
        undefined
      );
      expect(count).toBe(1);
    });

    it("bridges federated_replace_all to Tauri command with target session filtering", async () => {
      const mockSummary: FederatedReplaceSummary = {
        total_replacements: 5,
        documents_affected: 2,
        replacements_by_session: { "sess-1": 3, "sess-2": 2 },
      };
      mockInvoke.mockResolvedValueOnce(mockSummary);

      const summary = await IPC.federatedReplaceAll(
        "Target",
        "Replacement",
        { whole_word: true },
        ["sess-1", "sess-2"]
      );

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith(
        "federated_replace_all",
        {
          query: "Target",
          replacement: "Replacement",
          case_sensitive: false,
          whole_word: true,
          use_regex: false,
          target_session_ids: ["sess-1", "sess-2"],
        },
        undefined
      );
      expect(summary).toEqual(mockSummary);
    });
  });

  describe("FederatedSearchPanel Component Lifecycle & Interactivity", () => {
    it("initializes DOM elements and begins in hidden state", () => {
      const panel = new FederatedSearchPanel();
      expect(panel.isOpen()).toBe(false);

      const el = document.getElementById("federated-search-panel");
      expect(el).not.toBeNull();
      expect(el?.getAttribute("role")).toBe("dialog");
      expect(el?.getAttribute("aria-label")).toContain("Federated Search");
    });

    it("shows, hides, and toggles panel visibility", () => {
      const panel = new FederatedSearchPanel();
      panel.show("test-query", "find");
      expect(panel.isOpen()).toBe(true);

      const input = document.querySelector<HTMLInputElement>(".federated-query-input");
      expect(input?.value).toBe("test-query");

      panel.hide();
      expect(panel.isOpen()).toBe(false);

      panel.toggle("toggle-query");
      expect(panel.isOpen()).toBe(true);

      panel.toggle();
      expect(panel.isOpen()).toBe(false);
    });

    it("switches between Find and Replace modes", () => {
      const panel = new FederatedSearchPanel();
      panel.show("", "find");

      const replaceRow = document.querySelector<HTMLElement>(".federated-replace-row");
      expect(replaceRow?.style.display).toBe("none");

      const tabReplace = document.querySelector<HTMLButtonElement>(".federated-tab-replace");
      tabReplace?.click();

      const updatedReplaceRow = document.querySelector<HTMLElement>(".federated-replace-row");
      expect(updatedReplaceRow?.style.display).toBe("flex");
    });

    it("executes search and renders grouped results by document", async () => {
      const panel = new FederatedSearchPanel();
      panel.show("searchable text");

      const searchBtn = document.querySelector<HTMLButtonElement>(".federated-search-btn");
      searchBtn?.click();

      // Wait for async execution
      await new Promise((r) => setTimeout(r, 50));

      const matches = panel.getMatches();
      expect(matches.length).toBeGreaterThan(0);

      const results = document.querySelectorAll(".federated-match-item");
      expect(results.length).toBeGreaterThan(0);

      const docGroup = document.querySelector(".federated-doc-group");
      expect(docGroup).not.toBeNull();
    });

    it("triggers callbacks when match is clicked or tab switch requested", async () => {
      const onSelect = vi.fn();
      const onTabSwitch = vi.fn();

      const panel = new FederatedSearchPanel();
      panel.setCallbacks({
        onSelectMatch: onSelect,
        onTabSwitchRequested: onTabSwitch,
      });

      panel.show("sample text");
      await panel.executeSearch();

      const matchItem = document.querySelector<HTMLElement>(".federated-match-item");
      matchItem?.click();

      expect(onSelect).toHaveBeenCalledTimes(1);
      expect(onTabSwitch).toHaveBeenCalledTimes(1);
      expect(panel.getSelectedMatch()).not.toBeNull();
    });

    it("executes single replace and replace-all workflows", async () => {
      const panel = new FederatedSearchPanel();
      panel.show("term", "replace");
      await panel.executeSearch();

      const replaceBtn = document.querySelector<HTMLButtonElement>(".federated-replace-btn");
      replaceBtn?.click();
      await new Promise((r) => setTimeout(r, 50));

      const replaceAllBtn = document.querySelector<HTMLButtonElement>(".federated-replace-all-btn");
      replaceAllBtn?.click();
      await new Promise((r) => setTimeout(r, 50));

      const statusBar = document.querySelector(".federated-status-text");
      expect(statusBar?.textContent).toContain("Replaced");
    });
  });

  describe("Keyboard Shortcuts Integration", () => {
    it("emits FederatedFind on Ctrl+Shift+F and FederatedReplace on Ctrl+Shift+H", () => {
      const onAction = vi.fn();
      const shortcuts = new KeyboardShortcuts();
      shortcuts.setCallbacks({ onAction });

      // Simulate Ctrl+Shift+F
      const findEvt = new KeyboardEvent("keydown", {
        key: "f",
        ctrlKey: true,
        shiftKey: true,
      });
      window.dispatchEvent(findEvt);
      expect(onAction).toHaveBeenCalledWith("FederatedFind");

      // Simulate Ctrl+Shift+H
      const replaceEvt = new KeyboardEvent("keydown", {
        key: "h",
        ctrlKey: true,
        shiftKey: true,
      });
      window.dispatchEvent(replaceEvt);
      expect(onAction).toHaveBeenCalledWith("FederatedReplace");
    });
  });
});

