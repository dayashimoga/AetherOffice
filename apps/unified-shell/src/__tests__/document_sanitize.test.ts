import { describe, it, expect, beforeEach, vi } from "vitest";
import { SanitizeModal } from "../components/SanitizeModal";
import { BackstageMenu } from "../components/BackstageMenu";
import { UnifiedShellApp } from "../main";

describe("Sanitize, Privacy Audit & Forensic Redaction Engine (GEC-v2.0 Subphase 17.1.3)", () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <nav id="tab-bar-container" class="tab-bar"></nav>
        <section id="ribbon-container" class="ribbon-bar"></section>
        <section id="sub-header-container" class="sub-header-bar"></section>
        <main class="workspace-layout">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <section class="canvas-viewport" id="viewport-container">
            <canvas id="render-surface"></canvas>
          </section>
        </main>
        <div id="bottom-sheet" class="bottom-sheet"></div>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info">Page 1 of 1</span>
            <span id="status-word-count">0 words</span>
          </div>
        </footer>
      </div>
    `;
  });

  describe("1. SanitizeModal Component Lifecycle & Controls", () => {
    it("initializes hidden and sets default sanitize options", () => {
      const modal = new SanitizeModal("app");
      expect(modal.isVisible()).toBe(false);

      const overlay = document.getElementById("sanitize-modal-overlay");
      expect(overlay).not.toBeNull();
      expect(overlay?.style.display).toBe("none");

      const settings = modal.getSettings();
      expect(settings.stripMetadata).toBe(true);
      expect(settings.stripComments).toBe(true);
      expect(settings.stripRevisions).toBe(true);
      expect(settings.removeHiddenContent).toBe(true);
      expect(settings.redactPii).toBe(true);
    });

    it("opens modal and renders inspection findings", () => {
      const modal = new SanitizeModal("app");
      modal.open("QuarterlyFinancials.xlsx", "Excel");

      expect(modal.isVisible()).toBe(true);
      const overlay = document.getElementById("sanitize-modal-overlay");
      expect(overlay?.style.display).toBe("flex");

      const dialog = document.querySelector('[data-testid="sanitize-dialog"]');
      expect(dialog).not.toBeNull();

      const findings = document.querySelector('[data-testid="sanitize-findings-list"]');
      expect(findings).not.toBeNull();
      expect(findings?.textContent).toContain("author/company");
      expect(findings?.textContent).toContain("Comments");
      expect(findings?.textContent).toContain("PII Matches");

      const report = modal.getReport();
      expect(report).not.toBeNull();
      expect(report?.authorMetadata.length).toBeGreaterThan(0);
      expect(report?.commentsCount).toBe(3);
      expect(report?.trackedChangesCount).toBe(5);
    });

    it("allows updating settings checkboxes dynamically", () => {
      const modal = new SanitizeModal("app");
      modal.open("Document.docx", "Word");

      const optComments = document.querySelector<HTMLInputElement>(
        '[data-testid="sanitize-opt-comments"]'
      );
      expect(optComments).not.toBeNull();
      expect(optComments?.checked).toBe(true);

      optComments!.checked = false;
      optComments!.dispatchEvent(new Event("change"));

      expect(modal.getSettings().stripComments).toBe(false);
    });

    it("executes sanitization and renders success status banner", () => {
      const modal = new SanitizeModal("app");
      const onSanitized = vi.fn();
      modal.setOnSanitized(onSanitized);

      modal.open("M_and_A_Strategy.docx", "Word");

      const cleanBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="sanitize-btn-clean"]'
      );
      expect(cleanBtn).not.toBeNull();

      cleanBtn!.click();

      expect(modal.isSanitized()).toBe(true);
      expect(onSanitized).toHaveBeenCalled();

      const banner = document.querySelector('[data-testid="sanitize-status-banner"]');
      expect(banner).not.toBeNull();
      expect(banner?.textContent).toContain("Document Successfully Sanitized");
    });

    it("closes modal upon clicking close button or pressing Escape", () => {
      const modal = new SanitizeModal("app");
      modal.open("Report.docx", "Word");
      expect(modal.isVisible()).toBe(true);

      const closeBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="sanitize-btn-close"]'
      );
      expect(closeBtn).not.toBeNull();
      closeBtn!.click();
      expect(modal.isVisible()).toBe(false);

      modal.open("Report.docx", "Word");
      expect(modal.isVisible()).toBe(true);
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      expect(modal.isVisible()).toBe(false);
    });
  });

  describe("2. Backstage File Menu & UnifiedShellApp Integration", () => {
    it("renders Inspect & Sanitize button in Backstage Info tab and invokes modal", async () => {
      const backstage = new BackstageMenu("app");
      const onInspect = vi.fn();
      backstage.setCallbacks({ onInspectDocument: onInspect });

      await backstage.show("Word", "sess-1", "Confidential.docx");

      const inspectBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="btn-backstage-inspect"]'
      );
      expect(inspectBtn).not.toBeNull();

      inspectBtn!.click();
      expect(onInspect).toHaveBeenCalled();
    });

    it("routes InspectDocument and SanitizeDocument ribbon actions to SanitizeModal", () => {
      const app = new UnifiedShellApp(false);
      const modal = app.getSanitizeModal();

      expect(modal.isVisible()).toBe(false);

      const ribbon = (app as unknown as { ribbon: { onActionCallback?: (action: string) => void } }).ribbon;
      if (ribbon.onActionCallback) {
        ribbon.onActionCallback("InspectDocument");
      }
      expect(modal.isVisible()).toBe(true);

      modal.close();
      expect(modal.isVisible()).toBe(false);

      if (ribbon.onActionCallback) {
        ribbon.onActionCallback("SanitizeDocument");
      }
      expect(modal.isVisible()).toBe(true);
    });
  });
});
