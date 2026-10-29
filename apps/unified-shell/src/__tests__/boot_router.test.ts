import { describe, it, expect, beforeEach } from "vitest";
import { UnifiedShellApp } from "../main";

describe("Standalone Binary App-Mode Boot Router (GEC-v2.0 Subphase 8.1.1)", () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <div id="tab-bar-container"></div>
      <div id="ribbon-container"></div>
      <div id="sub-header-container"></div>
      <div id="sidebar-container"></div>
      <div id="bottom-sheet"></div>
      <canvas id="stylus-ink-canvas"></canvas>
      <div id="reflow-reader-container"></div>
      <div id="row-card-inspector-container"></div>
      <div id="laser-pointer-dot"></div>
      <span id="status-doc-info"></span>
      <span id="status-word-count"></span>
      <span id="status-zoom-level"></span>
      <input type="range" id="zoom-slider" />
    `;
  });

  it("boots isolated Writer app with clean Untitled 1.docx and zero cross-format tabs", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    const tabBar = app.getTabBar();
    expect(tabBar.getTabs().length).toBe(1);
    expect(tabBar.getTabs()[0].docType).toBe("Word");
    expect(tabBar.getTabs()[0].title).toBe("Untitled 1.docx");
    expect(app.getCurrentDocType()).toBe("Word");

    // Zero cross-format tabs
    const docTypes = tabBar.getTabs().map((t) => t.docType);
    expect(docTypes).not.toContain("Excel");
    expect(docTypes).not.toContain("PowerPoint");
    expect(docTypes).not.toContain("PDF");
  });

  it("boots isolated Calc app with Book1.xlsx and zero cross-format tabs", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "calc" });

    const tabBar = app.getTabBar();
    expect(tabBar.getTabs().length).toBe(1);
    expect(tabBar.getTabs()[0].docType).toBe("Excel");
    expect(tabBar.getTabs()[0].title).toBe("Book1.xlsx");
    expect(app.getCurrentDocType()).toBe("Excel");

    const docTypes = tabBar.getTabs().map((t) => t.docType);
    expect(docTypes).not.toContain("Word");
    expect(docTypes).not.toContain("PowerPoint");
    expect(docTypes).not.toContain("PDF");
  });

  it("boots isolated Slides app with Presentation1.pptx and zero cross-format tabs", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "slides" });

    const tabBar = app.getTabBar();
    expect(tabBar.getTabs().length).toBe(1);
    expect(tabBar.getTabs()[0].docType).toBe("PowerPoint");
    expect(tabBar.getTabs()[0].title).toBe("Presentation1.pptx");
    expect(app.getCurrentDocType()).toBe("PowerPoint");

    const docTypes = tabBar.getTabs().map((t) => t.docType);
    expect(docTypes).not.toContain("Word");
    expect(docTypes).not.toContain("Excel");
    expect(docTypes).not.toContain("PDF");
  });

  it("boots isolated PDF Studio app with Document.pdf and zero cross-format tabs", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "pdf" });

    const tabBar = app.getTabBar();
    expect(tabBar.getTabs().length).toBe(1);
    expect(tabBar.getTabs()[0].docType).toBe("PDF");
    expect(tabBar.getTabs()[0].title).toBe("Document.pdf");
    expect(app.getCurrentDocType()).toBe("PDF");

    const docTypes = tabBar.getTabs().map((t) => t.docType);
    expect(docTypes).not.toContain("Word");
    expect(docTypes).not.toContain("Excel");
    expect(docTypes).not.toContain("PowerPoint");
  });

  it("boots isolated app with CLI provided initial file", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({
      app_mode: "writer",
      initial_file: "/home/user/documents/QuarterlyReport.docx",
    });

    const tabBar = app.getTabBar();
    expect(tabBar.getTabs().length).toBe(1);
    expect(tabBar.getTabs()[0].title).toBe("QuarterlyReport.docx");
    expect(tabBar.getTabs()[0].docType).toBe("Word");
  });
});
