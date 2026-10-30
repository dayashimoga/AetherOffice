import { describe, it, expect, beforeEach, vi } from "vitest";
import { CopilotSidebar, CopilotCitation, CopilotDiff } from "../components/CopilotSidebar";
import { DiffPreviewModal } from "../components/DiffPreviewModal";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { UnifiedShellApp } from "../main";

describe("Interactive AetherCopilot Assistant UI & Inline Ghost Text (GEC-v2.0 Subphase 20.1.3)", () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app">
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content" style="display:flex;">
          <div id="sidebar-container"></div>
          <div id="viewport-container">
            <canvas id="render-surface"></canvas>
          </div>
          <div id="copilot-sidebar-container"></div>
        </div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
        </div>
      </div>
    `;
  });

  describe("1. CopilotSidebar Lifecycle & Mode Switching", () => {
    it("initializes hidden and toggles visibility", () => {
      const sidebar = new CopilotSidebar("copilot-sidebar-container");
      expect(sidebar.isOpen()).toBe(false);

      sidebar.show();
      expect(sidebar.isOpen()).toBe(true);

      sidebar.hide();
      expect(sidebar.isOpen()).toBe(false);

      sidebar.toggle();
      expect(sidebar.isOpen()).toBe(true);
    });

    it("dynamically updates suggested prompt chips based on active document mode", () => {
      const sidebar = new CopilotSidebar("copilot-sidebar-container");

      // Mode: Writer
      sidebar.setMode("writer");
      const chipsEl = document.querySelector(".copilot-chips-container") as HTMLElement;
      expect(chipsEl.textContent).toContain("Summarize Section");
      expect(chipsEl.textContent).toContain("Rewrite for Conciseness");
      expect(chipsEl.textContent).toContain("Fix Tone & Grammar");

      // Mode: Calc
      sidebar.setMode("calc");
      expect(chipsEl.textContent).toContain("Explain Formula");
      expect(chipsEl.textContent).toContain("Synthesize Formula");
      expect(chipsEl.textContent).toContain("Detect Data Anomalies");

      // Mode: Slides
      sidebar.setMode("slides");
      expect(chipsEl.textContent).toContain("Generate Slide from Outline");
      expect(chipsEl.textContent).toContain("Suggest Layout");
      expect(chipsEl.textContent).toContain("Create Speaker Notes");
    });
  });

  describe("2. Conversation Messages & Citation Interactions", () => {
    it("submits prompt, appends user & assistant bubbles, and clears conversation", () => {
      const sidebar = new CopilotSidebar("copilot-sidebar-container");
      const input = document.querySelector(".copilot-input-textarea") as HTMLTextAreaElement;
      const sendBtn = document.querySelector(".copilot-btn-send") as HTMLButtonElement;

      input.value = "Summarize the financial highlights";
      sendBtn.click();

      const messages = sidebar.getMessages();
      expect(messages.some((m) => m.sender === "user" && m.text.includes("financial highlights"))).toBe(true);

      sidebar.clearMessages();
      expect(sidebar.getMessages().length).toBe(1); // Only welcome message remains
    });

    it("renders clickable citation pills and invokes citation callback", () => {
      const onCitationClick = vi.fn();
      const sidebar = new CopilotSidebar("copilot-sidebar-container", { onCitationClick });

      const citations: CopilotCitation[] = [
        {
          marker: "[1]",
          snippet: "Q3 subscription revenue surged by 24%",
          location: "Section 1 > Heading 2",
        },
      ];

      sidebar.appendMessage(
        "assistant",
        "Operating momentum remained strong with revenue surging 24% YoY [1].",
        { citations }
      );

      const pill = document.querySelector(".copilot-citation-link") as HTMLButtonElement;
      expect(pill).not.toBeNull();
      expect(pill.textContent).toBe("[1]");

      pill.click();
      expect(onCitationClick).toHaveBeenCalledTimes(1);
      expect(onCitationClick).toHaveBeenCalledWith(citations[0]);
    });
  });

  describe("3. Inline Ghost-Text Completion in WriterCanvas", () => {
    it("renders ghost text, dismisses on [Esc], and accepts on [Tab]", () => {
      const container = document.getElementById("viewport-container") as HTMLElement;
      const writerState = new WriterState();
      writerState.doc.pages[0].paragraphs[0].runs[0].text = "Quarterly revenue report";
      const canvas = new WriterCanvas(container, writerState);

      // Initially no ghost text
      expect(canvas.getGhostText()).toBeNull();

      // Set ghost text completion
      canvas.setGhostText(" showing 24% growth across enterprise clients");
      expect(canvas.getGhostText()).toBe(" showing 24% growth across enterprise clients");

      // Press Escape -> Dismisses ghost text
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Escape" }));
      expect(canvas.getGhostText()).toBeNull();

      // Re-set ghost text and press Tab -> Accepts and inserts text
      canvas.setGhostText(" for corporate audit");
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Tab" }));

      expect(canvas.getGhostText()).toBeNull();
      const fullText = writerState.doc.pages[0].paragraphs[0].runs.map((r) => r.text).join("");
      expect(fullText).toContain("for corporate audit");
    });
  });

  describe("4. DiffPreviewModal Visual Diffs & Application", () => {
    it("renders side-by-side original and proposed changes, invokes onApply callback", () => {
      const modal = new DiffPreviewModal();
      expect(modal.isOpen()).toBe(false);

      const onApply = vi.fn();
      const onCancel = vi.fn();

      modal.open({
        title: "Review Formula Optimization",
        targetDescription: "Cell D2",
        originalText: "=VLOOKUP(A2, Products!A:C, 3, FALSE)",
        proposedText: "=XLOOKUP(A2, Products!A:A, Products!C:C, 0)",
        onApply,
        onCancel,
      });

      expect(modal.isOpen()).toBe(true);

      const origEl = document.querySelector(".diff-view-original") as HTMLElement;
      const propEl = document.querySelector(".diff-view-proposed") as HTMLElement;
      expect(origEl.textContent).toBe("=VLOOKUP(A2, Products!A:C, 3, FALSE)");
      expect(propEl.textContent).toBe("=XLOOKUP(A2, Products!A:A, Products!C:C, 0)");

      // Click Accept & Apply
      const applyBtn = document.querySelector(".diff-modal-btn-apply") as HTMLButtonElement;
      applyBtn.click();

      expect(onApply).toHaveBeenCalledTimes(1);
      expect(onApply).toHaveBeenCalledWith("=XLOOKUP(A2, Products!A:A, Products!C:C, 0)");
      expect(modal.isOpen()).toBe(false);
    });

    it("cancels diff without applying changes", () => {
      const modal = new DiffPreviewModal();
      const onApply = vi.fn();
      const onCancel = vi.fn();

      modal.open({
        originalText: "Draft v1",
        proposedText: "Draft v2",
        onApply,
        onCancel,
      });

      const cancelBtn = document.querySelector(".diff-modal-btn-cancel") as HTMLButtonElement;
      cancelBtn.click();

      expect(onCancel).toHaveBeenCalledTimes(1);
      expect(onApply).not.toHaveBeenCalled();
      expect(modal.isOpen()).toBe(false);
    });
  });

  describe("5. UnifiedShellApp Ribbon Wiring for Copilot & Diff Workflow", () => {
    it("toggles Copilot sidebar via ToggleCopilot ribbon action", () => {
      const app = new UnifiedShellApp(false);
      const copilot = app.getCopilotSidebar();

      expect(copilot.isOpen()).toBe(false);

      // Trigger ToggleCopilot
      app.handleRibbonAction("ToggleCopilot");
      expect(copilot.isOpen()).toBe(true);

      app.handleRibbonAction("ToggleCopilot");
      expect(copilot.isOpen()).toBe(false);
    });

    it("triggers DiffPreviewModal when clicking Review Diff button on Copilot message", () => {
      const app = new UnifiedShellApp(false);
      const copilot = app.getCopilotSidebar();
      const diffModal = app.getDiffPreviewModal();

      copilot.show();

      const diff: CopilotDiff = {
        original: "In order to facilitate the completion...",
        modified: "To complete the project...",
        targetRangeOrElement: "Paragraph 1",
      };

      copilot.appendMessage("assistant", "Here is a concise rewrite [1].", { diff });

      const reviewBtn = document.querySelector(".copilot-btn-review-diff") as HTMLButtonElement;
      expect(reviewBtn).not.toBeNull();

      reviewBtn.click();
      expect(diffModal.isOpen()).toBe(true);
    });
  });
});

