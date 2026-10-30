import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { fileIoService } from "../services/fileIoService";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { notesState } from "../state/notesState";
import { IPC } from "../ipc";

describe("Subphase 28.2.1: Universal File Menu Backstage & File I/O Service", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    HTMLCanvasElement.prototype.getContext = (() => ({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      measureText: () => ({ width: 10 }),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      setTransform: vi.fn(),
      setLineDash: vi.fn(),
    })) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    document.body.className = "";
    document.documentElement.className = "";

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-logo">📄</span>
            <span class="app-name">OpenOfficeRS</span>
            <div class="quick-access-toolbar">
              <button id="btn-quick-open" class="quick-btn">📂</button>
              <button id="btn-quick-save" class="quick-btn">💾</button>
              <button id="btn-quick-print" class="quick-btn">🖨️</button>
            </div>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">Light</button>
          </div>
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <div class="shell-body">
          <div id="app-rail-container"></div>
          <div class="shell-workspace-area">
            <div id="tab-bar-container"></div>
            <div id="ribbon-container"></div>
            <div id="sub-header-container"></div>
            <main class="workspace-layout">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <section id="viewport-container" class="canvas-viewport"></section>
            </main>
            <footer id="status-bar" class="status-bar">
              <span id="status-doc-info">Page 1 of 1</span>
              <span id="status-word-count">0 words</span>
              <span id="status-lang">English (US)</span>
            </footer>
          </div>
        </div>
      </div>
    `;

    app = new UnifiedShellApp(false);
  });

  describe("1. Document Type Detection", () => {
    it("correctly identifies document types across all 5 suite formats", () => {
      expect(fileIoService.detectDocType("document.docx")).toBe("Word");
      expect(fileIoService.detectDocType("contract.doc")).toBe("Word");
      expect(fileIoService.detectDocType("page.html")).toBe("Word");

      expect(fileIoService.detectDocType("sheet.xlsx")).toBe("Excel");
      expect(fileIoService.detectDocType("table.csv")).toBe("Excel");
      expect(fileIoService.detectDocType("data.tsv")).toBe("Excel");

      expect(fileIoService.detectDocType("slides.pptx")).toBe("PowerPoint");
      expect(fileIoService.detectDocType("deck.ppt")).toBe("PowerPoint");

      expect(fileIoService.detectDocType("whitepaper.pdf")).toBe("PDF");

      expect(fileIoService.detectDocType("notes.txt")).toBe("Notes");
      expect(fileIoService.detectDocType("readme.md")).toBe("Notes");
      expect(fileIoService.detectDocType("config.json")).toBe("Notes");
    });
  });

  describe("2. Document Serialization & Save (GEC-v4.0 Article III.1)", () => {
    it("saves and clears dirty state across all 5 document types", async () => {
      const downloadSpy = vi.spyOn(fileIoService, "downloadFile").mockImplementation(() => {});

      // 1. Word
      app.switchToOrCreateApp("Word");
      writerState.execCommand("insertText", "Hello World Docx");
      const wordTab = app.getTabBar().getActiveTab()!;
      app.getTabBar().setDirty(wordTab.id, true);
      expect(wordTab.isDirty).toBe(true);

      await app.saveActiveDocument("Proposal.docx");
      expect(wordTab.isDirty).toBe(false);
      expect(downloadSpy).toHaveBeenCalledWith(
        expect.any(String),
        "Proposal.docx",
        expect.stringContaining("wordprocessingml")
      );

      // 2. Excel
      app.switchToOrCreateApp("Excel");
      calcState.setCellValue("A1", "Revenue");
      const excelTab = app.getTabBar().getActiveTab()!;
      app.getTabBar().setDirty(excelTab.id, true);

      await app.saveActiveDocument("Financials.xlsx");
      expect(excelTab.isDirty).toBe(false);
      expect(downloadSpy).toHaveBeenCalledWith(
        expect.any(String),
        "Financials.xlsx",
        "text/csv"
      );

      // 3. PowerPoint
      app.switchToOrCreateApp("PowerPoint");
      const pptTab = app.getTabBar().getActiveTab()!;
      app.getTabBar().setDirty(pptTab.id, true);

      await app.saveActiveDocument("Keynote.pptx");
      expect(pptTab.isDirty).toBe(false);
      expect(downloadSpy).toHaveBeenCalledWith(
        expect.any(String),
        "Keynote.pptx",
        expect.stringContaining("presentationml")
      );

      // 4. PDF
      app.switchToOrCreateApp("PDF");
      const pdfTab = app.getTabBar().getActiveTab()!;
      app.getTabBar().setDirty(pdfTab.id, true);

      await app.saveActiveDocument("Audit.pdf");
      expect(pdfTab.isDirty).toBe(false);
      expect(downloadSpy).toHaveBeenCalledWith(
        expect.any(String),
        "Audit.pdf",
        "application/pdf"
      );

      // 5. Notes
      app.switchToOrCreateApp("Notes");
      notesState.setContent("# Notes Header\nSome details");
      const notesTab = app.getTabBar().getActiveTab()!;
      app.getTabBar().setDirty(notesTab.id, true);

      await app.saveActiveDocument("Scratchpad.md");
      expect(notesTab.isDirty).toBe(false);
      expect(downloadSpy).toHaveBeenCalledWith(
        expect.stringContaining("# Notes Header"),
        "Scratchpad.md",
        "text/markdown"
      );
    });
  });

  describe("3. Export Service across standard interchange formats", () => {
    it("exports documents into PDF, CSV, HTML, Markdown, and TXT", async () => {
      const downloadSpy = vi.spyOn(fileIoService, "downloadFile").mockImplementation(() => {});

      app.switchToOrCreateApp("Notes");
      notesState.setContent("Export Content Test");

      await app.handleExport("txt", "Export.txt");
      expect(downloadSpy).toHaveBeenCalledWith("Export Content Test", "Export.txt", "text/plain");

      await app.handleExport("md", "Export.md");
      expect(downloadSpy).toHaveBeenCalledWith("Export Content Test", "Export.md", "text/markdown");

      app.switchToOrCreateApp("Excel");
      calcState.setCellValue("A1", "Header1");
      calcState.setCellValue("B1", "Value1");

      await app.handleExport("csv", "Export.csv");
      expect(downloadSpy).toHaveBeenCalledWith(expect.stringContaining("Header1"), "Export.csv", "text/csv");
    });
  });

  describe("4. Clean Print Frame Generation (GEC-v4.0 Article III.1)", () => {
    it("creates dedicated clean print iframe without shell chrome for all 5 apps", () => {
      window.print = vi.fn();

      // Word print
      app.switchToOrCreateApp("Word");
      fileIoService.printDocument("Word", "MyDoc.docx");
      let iframe = document.getElementById("clean-print-frame") as HTMLIFrameElement;
      expect(iframe).not.toBeNull();
      expect(iframe.style.position).toBe("fixed");

      // Excel print
      app.switchToOrCreateApp("Excel");
      fileIoService.printDocument("Excel", "MySheet.xlsx");
      iframe = document.getElementById("clean-print-frame") as HTMLIFrameElement;
      expect(iframe).not.toBeNull();

      // PowerPoint print
      app.switchToOrCreateApp("PowerPoint");
      fileIoService.printDocument("PowerPoint", "MyDeck.pptx");
      iframe = document.getElementById("clean-print-frame") as HTMLIFrameElement;
      expect(iframe).not.toBeNull();

      // PDF print
      app.switchToOrCreateApp("PDF");
      fileIoService.printDocument("PDF", "MyDoc.pdf");
      iframe = document.getElementById("clean-print-frame") as HTMLIFrameElement;
      expect(iframe).not.toBeNull();

      // Notes print
      app.switchToOrCreateApp("Notes");
      fileIoService.printDocument("Notes", "MyNote.txt");
      iframe = document.getElementById("clean-print-frame") as HTMLIFrameElement;
      expect(iframe).not.toBeNull();
    });
  });

  describe("5. Quick Access Toolbar & Backstage Menu Wiring", () => {
    it("connects 📂 Open, 💾 Save, and 🖨️ Print buttons", async () => {
      const saveSpy = vi.spyOn(app, "saveActiveDocument").mockResolvedValue("Saved.docx");
      const printSpy = vi.spyOn(app, "handlePrint").mockImplementation(() => {});
      const openSpy = vi.spyOn(app, "openFilePickerOrBackstage").mockImplementation(() => {});

      const quickOpen = document.getElementById("btn-quick-open")!;
      const quickSave = document.getElementById("btn-quick-save")!;
      const quickPrint = document.getElementById("btn-quick-print")!;

      quickOpen.click();
      expect(openSpy).toHaveBeenCalled();

      quickSave.click();
      expect(saveSpy).toHaveBeenCalled();

      quickPrint.click();
      expect(printSpy).toHaveBeenCalled();
    });
  });
});
