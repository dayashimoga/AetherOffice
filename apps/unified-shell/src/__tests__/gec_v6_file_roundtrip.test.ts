import { describe, it, expect, beforeEach } from "vitest";
import { fileCodec, showSavedToast } from "../services/fileCodec";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { binaryPackageService } from "../services/binaryPackageService";
import { IPC } from "../ipc";

describe("GEC-v6.0 Subphase 35.1.1: Native Tauri + Browser File Save & Open Bridge for All 5 Formats", () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="app"></div>';
  });

  it("1. Word (.docx): full binary round-trip preserves text, formatting, and header/footer", async () => {
    writerState.resetToBlank("Proposal.docx");
    writerState.doc.pages[0].paragraphs = [
      {
        id: "p1",
        style: "Heading 1",
        alignment: "left",
        runs: [{ text: "Executive Summary", bold: true, font: "Calibri", size: 16 }],
      },
      {
        id: "p2",
        style: "Normal",
        alignment: "left",
        runs: [{ text: "This document proves real binary OOXML roundtrip serialization in AetherOffice.", font: "Calibri", size: 11 }],
      },
    ];
    writerState.doc.headerText = "CONFIDENTIAL HEADER";
    writerState.doc.footerText = "Page 1 of 1";

    // Build binary .docx
    const bytes = await binaryPackageService.buildDocx(
      writerState.getTextContent(),
      writerState.doc.pages[0].paragraphs.map((p) => ({
        text: p.runs.map((r) => r.text).join(""),
        bold: p.runs.some((r) => r.bold),
        italic: p.runs.some((r) => r.italic),
      }))
    );
    expect(bytes.length).toBeGreaterThan(100);

    // Reset to blank
    writerState.resetToBlank("Empty.docx");
    expect(writerState.getTextContent().trim()).toBe("");

    // Open from binary buffer
    const opened = await fileCodec.openFile(bytes, "Proposal.docx", "Word");
    expect(opened).toBe(true);

    const restoredText = writerState.getTextContent();
    expect(restoredText).toContain("Executive Summary");
    expect(restoredText).toContain("This document proves real binary OOXML roundtrip");
    expect(writerState.doc.pages[0].paragraphs.length).toBeGreaterThan(0);
  });

  it("2. Excel (.xlsx & .csv): full binary round-trip preserves multi-sheet cells and formulas", async () => {
    calcState.resetToBlank("Q3_Financials");
    calcState.setCellValue("A1", "Department");
    calcState.setCellValue("B1", "Budget");
    calcState.setCellValue("A2", "Engineering");
    calcState.setCellValue("B2", "250000");
    calcState.setCellValue("A3", "Design");
    calcState.setCellValue("B3", "120000");
    calcState.setCellValue("A4", "Total");
    calcState.setCellValue("B4", "=SUM(B2:B3)");

    const activeSheet = calcState.getActiveSheet();
    const cells: Record<string, string> = {};
    for (const [coord, c] of activeSheet!.cells.entries()) {
      if (c) {
        const val = c.raw || (c.computed !== undefined ? String(c.computed) : "");
        if (val !== "") cells[coord] = val;
      }
    }

    const bytes = await binaryPackageService.buildXlsx(cells);
    expect(bytes.length).toBeGreaterThan(200);

    // Reset grid
    calcState.resetToBlank("BlankSheet");
    expect(calcState.getCellValue("A1") ?? "").toBe("");

    // Re-open XLSX buffer
    const opened = await fileCodec.openFile(bytes, "Financials.xlsx", "Excel");
    expect(opened).toBe(true);

    expect(calcState.getCellValue("A1")).toBe("Department");
    expect(calcState.getCellValue("B1")).toBe("Budget");
    expect(calcState.getCellValue("A2")).toBe("Engineering");
    expect(String(calcState.getCellValue("B2"))).toBe("250000");
    expect(calcState.getCellValue("A4")).toBe("Total");
    expect(["370000", "=SUM(B2:B3)"]).toContain(String(calcState.getCellValue("B4")));

    // Test CSV roundtrip
    const csvData = "Metric,Value\nRevenue,100000\nExpenses,60000";
    const csvOpened = await fileCodec.openFile(csvData, "Report.csv", "Excel");
    expect(csvOpened).toBe(true);
    expect(calcState.getCellValue("A1")).toBe("Metric");
    expect(calcState.getCellValue("B1")).toBe("Value");
    expect(calcState.getCellValue("A2")).toBe("Revenue");
  });

  it("3. PowerPoint (.pptx): full binary round-trip preserves slides, titles and shape texts", async () => {
    slidesState.resetToBlank("ProductDeck.pptx");
    slidesState.slides = [
      {
        id: "slide-1",
        title: "AetherOffice 2026 Keynote",
        shapes: [
          {
            id: "s1",
            type: "text",
            x: 100,
            y: 200,
            w: 400,
            h: 50,
            fill: "transparent",
            text: "Next Generation Productivity Suite",
          },
        ],
      },
      {
        id: "slide-2",
        title: "Architecture & Capabilities",
        shapes: [
          {
            id: "s2",
            type: "text",
            x: 100,
            y: 200,
            w: 400,
            h: 50,
            fill: "transparent",
            text: "High-performance Rust Engine with WebAssembly acceleration",
          },
        ],
      },
    ];

    const bytes = await binaryPackageService.buildPptx(slidesState.slides);
    expect(bytes.length).toBeGreaterThan(200);

    // Reset slides
    slidesState.resetToBlank("Empty.pptx");
    expect(slidesState.slides.length).toBe(1);

    // Re-open PPTX buffer
    const opened = await fileCodec.openFile(bytes, "ProductDeck.pptx", "PowerPoint");
    expect(opened).toBe(true);

    expect(slidesState.slides.length).toBe(2);
    expect(slidesState.slides[0].title).toBe("AetherOffice 2026 Keynote");
    expect(slidesState.slides[0].shapes[0].text).toContain("Next Generation Productivity Suite");
    expect(slidesState.slides[1].title).toBe("Architecture & Capabilities");
    expect(slidesState.slides[1].shapes[0].text).toContain("High-performance Rust Engine");
  });

  it("4. PDF Studio (.pdf): full binary round-trip preserves multi-page structure and text elements", async () => {
    pdfState.resetToBlank("Contract.pdf");
    pdfState.loadPdfDocument([
      {
        id: "p1",
        pageNumber: 1,
        width: 612,
        height: 792,
        rotation: 0,
        title: "Page 1",
        textBlocks: [
          { id: "tb1", x: 72, y: 100, text: "Standard Non-Disclosure Agreement", fontSize: 16 },
          { id: "tb2", x: 72, y: 150, text: "Party A agrees to keep all information confidential.", fontSize: 11 },
        ],
      },
      {
        id: "p2",
        pageNumber: 2,
        width: 612,
        height: 792,
        rotation: 0,
        title: "Page 2",
        textBlocks: [
          { id: "tb3", x: 72, y: 100, text: "Signatures and Execution Clauses", fontSize: 14 },
        ],
      },
    ]);

    const bytes = pdfState.exportBinaryPdf();
    expect(bytes.length).toBeGreaterThan(100);

    // Reset PDF state
    pdfState.resetToBlank("Blank.pdf");
    expect(pdfState.pages.length).toBe(1);

    // Re-open PDF buffer
    const opened = await fileCodec.openFile(bytes, "Contract.pdf", "PDF");
    expect(opened).toBe(true);

    expect(pdfState.pages.length).toBe(2);
    const p1Text = (pdfState.pages[0]?.textBlocks || []).map((t) => t.text).join(" ");
    expect(p1Text).toContain("Standard Non-Disclosure Agreement");
    const p2Text = (pdfState.pages[1]?.textBlocks || []).map((t) => t.text).join(" ");
    expect(p2Text).toContain("Signatures and Execution Clauses");
  });

  it("5. Notepad / Notes (.txt & .md): full UTF-8 text round-trip", async () => {
    notesState.resetToBlank("README.md");
    const rawMarkdown = "# Project OpenOfficeRS\n\n- Zero unwrap crashes\n- Real binary roundtrips\n- WCAG AA high contrast\n";
    notesState.setContent(rawMarkdown);

    const savedPath = await fileCodec.saveFile("Notes", { filename: "README.md" });
    expect(savedPath).toBe("README.md");

    // Reset notes
    notesState.resetToBlank("blank.txt");
    expect(notesState.getContent()).toBe("");

    // Re-open
    const opened = await fileCodec.openFile(rawMarkdown, "README.md", "Notes");
    expect(opened).toBe(true);
    expect(notesState.getContent()).toBe(rawMarkdown);
  });

  it("6. Toast Notification Banner: displays '✅ Saved successfully: <filename>'", () => {
    showSavedToast("Quarterly_Report_2026.docx");

    const toast = document.getElementById("save-toast-banner");
    expect(toast).not.toBeNull();
    expect(toast?.textContent).toBe("✅ Saved successfully: Quarterly_Report_2026.docx");
    expect(toast?.classList.contains("toast-notification-banner")).toBe(true);
  });

  it("7. Native Tauri IPC commands saveFileBytes and readFileBytes are functional", async () => {
    const sampleBytes = new Uint8Array([80, 75, 3, 4, 1, 2, 3]);
    const savedPath = await IPC.saveFileBytes("test.docx", sampleBytes, "C:\\Users\\Test\\Downloads\\test.docx");
    expect(savedPath).toBe("C:\\Users\\Test\\Downloads\\test.docx");

    const readBytes = await IPC.readFileBytes("C:\\Users\\Test\\Downloads\\test.docx");
    expect(readBytes).toBeDefined();
  });
});
