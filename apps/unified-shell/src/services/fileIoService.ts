import { DocumentType, IPC } from "../ipc";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { binaryPackageService } from "./binaryPackageService";

export interface OpenedFileResult {
  file?: File;
  filePath: string;
  content: string;
  buffer?: ArrayBuffer;
  docType: DocumentType;
}

export class FileIoService {
  public detectDocType(fileName: string): DocumentType {
    const lower = fileName.toLowerCase();
    if (lower.endsWith(".xlsx") || lower.endsWith(".xls") || lower.endsWith(".csv") || lower.endsWith(".tsv")) {
      return "Excel";
    }
    if (lower.endsWith(".pptx") || lower.endsWith(".ppt")) {
      return "PowerPoint";
    }
    if (lower.endsWith(".pdf")) {
      return "PDF";
    }
    if (
      lower.endsWith(".txt") ||
      lower.endsWith(".md") ||
      lower.endsWith(".log") ||
      lower.endsWith(".json")
    ) {
      return "Notes";
    }
    return "Word";
  }

  public downloadFile(content: BlobPart, filename: string, mimeType = "application/octet-stream"): void {
    if (typeof window === "undefined" || !document) return;
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  public triggerOpenDialog(): Promise<OpenedFileResult | null> {
    return new Promise((resolve) => {
      let input = document.getElementById("universal-file-picker") as HTMLInputElement;
      if (!input) {
        input = document.createElement("input");
        input.type = "file";
        input.id = "universal-file-picker";
        input.style.display = "none";
        input.accept = ".docx,.doc,.html,.htm,.xlsx,.xls,.csv,.tsv,.pptx,.ppt,.pdf,.txt,.md,.json,.log";
        document.body.appendChild(input);
      }

      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) {
          resolve(null);
          return;
        }

        const docType = this.detectDocType(file.name);
        let content = "";
        let buffer: ArrayBuffer | undefined;
        try {
          buffer = await file.arrayBuffer();
          content = new TextDecoder().decode(buffer);
        } catch {
          content = "";
        }

        resolve({
          file,
          filePath: file.name,
          content,
          buffer,
          docType,
        });
      };

      input.click();
    });
  }

  public async saveDocument(
    sessionId: string,
    docType: DocumentType,
    title: string,
    existingPath?: string,
    isSaveAs = false
  ): Promise<string> {
    const targetPath = existingPath || title;
    let savedPath = targetPath;

    try {
      const res = await IPC.saveDocument(sessionId, isSaveAs ? targetPath : existingPath);
      if (res) savedPath = res;
    } catch {
      // IPC fallback to browser download
    }

    const hasTauri =
      typeof window !== "undefined" &&
      Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

    if (!hasTauri) {
      let content: BlobPart = "";
      let mime = "text/plain";

      if (docType === "Word") {
        content = (await binaryPackageService.buildDocx(writerState.getTextContent())) as unknown as BlobPart;
        mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
      } else if (docType === "Excel") {
        const activeSheet = calcState.getActiveSheet();
        const cells: Record<string, string> = {};
        if (activeSheet) {
          for (const [coord, c] of Object.entries(activeSheet.cells)) {
            if (c.value !== undefined && c.value !== null) cells[coord] = String(c.value);
          }
        }
        content = (await binaryPackageService.buildXlsx(cells)) as unknown as BlobPart;
        mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
      } else if (docType === "PowerPoint") {
        content = (await binaryPackageService.buildPptx(slidesState.slides)) as unknown as BlobPart;
        mime = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
      } else if (docType === "PDF") {
        content = pdfState.exportBinaryPdf() as unknown as BlobPart;
        mime = "application/pdf";
      } else {
        content = notesState.getContent();
        mime = targetPath.endsWith(".md") ? "text/markdown" : "text/plain";
      }

      const filename = targetPath.split(/[/\\]/).pop() || title;
      this.downloadFile(content, filename, mime);
    }

    return savedPath;
  }

  public async loadFileContent(
    bufferOrText: ArrayBuffer | Uint8Array | string,
    docType: DocumentType,
    fileName: string
  ): Promise<boolean> {
    const isBuffer = typeof bufferOrText !== "string";
    const uint8: Uint8Array = isBuffer
      ? bufferOrText instanceof Uint8Array
        ? bufferOrText
        : new Uint8Array(bufferOrText)
      : new TextEncoder().encode(bufferOrText);

    try {
      if (docType === "Word") {
        if (fileName.toLowerCase().endsWith(".docx")) {
          const parsed = await binaryPackageService.parseDocx(uint8);
          writerState.loadDocumentFromText(parsed.text, fileName);
        } else {
          const text = new TextDecoder().decode(uint8);
          writerState.loadDocumentFromText(text, fileName);
        }
      } else if (docType === "Excel") {
        if (fileName.toLowerCase().endsWith(".xlsx")) {
          const parsed = await binaryPackageService.parseXlsx(uint8);
          for (const [coord, val] of Object.entries(parsed.cells)) {
            calcState.setCellValue(coord, val);
          }
        } else {
          calcState.importFromCsv(new TextDecoder().decode(uint8));
        }
      } else if (docType === "PowerPoint") {
        if (fileName.toLowerCase().endsWith(".pptx")) {
          const parsed = await binaryPackageService.parsePptx(uint8);
          if (parsed.slides.length > 0) {
            slidesState.slides = parsed.slides.map((s, idx) => ({
              id: String(idx + 1),
              title: s.title,
              shapes: s.text.map((t, tIdx) => ({
                id: `shape-${idx}-${tIdx}`,
                type: "text" as const,
                x: 100,
                y: 100 + tIdx * 50,
                w: 400,
                h: 40,
                fill: "transparent",
                text: t,
              })),
            }));
            slidesState.currentSlideIndex = 0;
          }
        }
      } else if (docType === "PDF") {
        const parsed = binaryPackageService.parsePdf(uint8);
        if (parsed.pageCount > 1) {
          while (pdfState.pages.length < parsed.pageCount) {
            pdfState.insertBlankPage(pdfState.pages.length);
          }
        }
      } else {
        notesState.setContent(new TextDecoder().decode(uint8));
      }
      return true;
    } catch {
      return false;
    }
  }

  public async exportDocument(
    sessionId: string,
    docType: DocumentType,
    title: string,
    format: "pdf" | "csv" | "txt" | "html" | "md",
    outputPath?: string
  ): Promise<string> {
    const baseName = title.replace(/\.[^/.]+$/, "");
    const filename = outputPath || `${baseName}.${format}`;

    let content: BlobPart = "";
    let mime = "text/plain";

    if (format === "pdf") {
      try {
        await IPC.exportPdf(sessionId, filename);
        return filename;
      } catch {
        content = "%PDF-1.4\n%OpenOfficeRS Exported PDF\n";
        mime = "application/pdf";
      }
    } else if (format === "csv") {
      content = calcState.exportCSV();
      mime = "text/csv";
    } else if (format === "html") {
      content = `<!DOCTYPE html><html><head><title>${title}</title></head><body>${writerState.getTextContent()}</body></html>`;
      mime = "text/html";
    } else if (format === "md") {
      content = notesState.getContent();
      mime = "text/markdown";
    } else {
      content =
        docType === "Word"
          ? writerState.getTextContent()
          : docType === "Notes"
          ? notesState.getContent()
          : calcState.exportCSV();
      mime = "text/plain";
    }

    this.downloadFile(content, filename.split(/[/\\]/).pop() || filename, mime);
    return filename;
  }

  public printDocument(docType: DocumentType, title = "Document"): void {
    if (typeof document === "undefined" || typeof window === "undefined") return;

    const existing = document.getElementById("clean-print-frame");
    if (existing) existing.remove();

    const iframe = document.createElement("iframe");
    iframe.id = "clean-print-frame";
    iframe.style.position = "fixed";
    iframe.style.top = "-9999px";
    iframe.style.left = "-9999px";
    iframe.style.width = "0";
    iframe.style.height = "0";
    iframe.style.border = "none";
    try {
      document.body.appendChild(iframe);
    } catch {
      window.print();
      return;
    }

    let docHtml = "";
    if (docType === "Word") {
      const pages = writerState.doc.pages;
      docHtml = pages
        .map(
          (page) => `
        <div class="print-page" style="page-break-after: always; padding: 40px; font-family: Calibri, sans-serif; font-size: 11pt; line-height: 1.5;">
          ${page.paragraphs.map((p) => `<p style="margin: 0 0 10pt 0;">${p.runs.map((r) => r.text).join("") || "&nbsp;"}</p>`).join("")}
        </div>
      `
        )
        .join("");
    } else if (docType === "Excel") {
      const activeSheet = calcState.getActiveSheet();
      const cells = activeSheet?.cells;
      let tableRows = "";
      if (cells && cells.size > 0) {
        tableRows = Array.from(cells.entries())
          .map(
            ([coord, cell]) =>
              `<tr><td style="border: 1px solid #ccc; padding: 4px 8px; font-weight: bold; width: 60px;">${coord}</td><td style="border: 1px solid #ccc; padding: 4px 8px;">${cell.computed ?? cell.raw}</td></tr>`
          )
          .join("");
      } else {
        tableRows = `<tr><td style="border: 1px solid #ccc; padding: 8px;">Sheet is empty.</td></tr>`;
      }
      docHtml = `
        <div class="print-sheet" style="padding: 20px; font-family: Segoe UI, sans-serif; font-size: 10pt;">
          <h2 style="margin-bottom: 12px;">${activeSheet?.name || "Sheet1"}</h2>
          <table style="border-collapse: collapse; width: 100%;">${tableRows}</table>
        </div>
      `;
    } else if (docType === "PowerPoint") {
      docHtml = slidesState.slides
        .map(
          (s) => `
        <div class="print-slide" style="page-break-after: always; width: 100%; height: 600px; padding: 30px; border: 1px solid #ddd; margin-bottom: 20px; font-family: Segoe UI, sans-serif; background: ${s.background || "#ffffff"};">
          <h1 style="margin-bottom: 20px; font-size: 28pt;">${s.title}</h1>
          <div style="font-size: 14pt; color: #475569;">
            ${s.shapes.map((shape) => `<div style="margin: 8px 0;">${shape.text || `[Shape: ${shape.shapeType}]`}</div>`).join("")}
          </div>
        </div>
      `
        )
        .join("");
    } else if (docType === "PDF") {
      docHtml = pdfState.pages
        .map(
          (p, idx) => `
        <div class="print-pdf-page" style="page-break-after: always; width: ${p.width}px; height: ${p.height}px; padding: 40px; margin-bottom: 20px; border: 1px solid #ccc; font-family: sans-serif;">
          <div style="color: #64748b; font-size: 10pt; margin-bottom: 20px;">Page ${idx + 1} of ${pdfState.pages.length} (${(p.width / 96).toFixed(1)} × ${(p.height / 96).toFixed(1)} in)</div>
          ${(p.textBlocks || []).map((tb) => `<p style="margin: 4px 0;">${tb.text}</p>`).join("") || "<p>[PDF Page Content]</p>"}
        </div>
      `
        )
        .join("");
    } else {
      const content = notesState.getContent();
      docHtml = `
        <div class="print-notes" style="padding: 40px; font-family: monospace; font-size: 11pt; white-space: pre-wrap;">
          ${content || "<p>[Empty Note]</p>"}
        </div>
      `;
    }

    const frameDoc = iframe.contentWindow?.document;
    if (frameDoc) {
      frameDoc.open();
      frameDoc.write(`
        <!DOCTYPE html>
        <html>
          <head>
            <title>${title}</title>
            <style>
              @page { size: auto; margin: 15mm; }
              body { margin: 0; padding: 0; background: #fff; color: #000; }
            </style>
          </head>
          <body>
            ${docHtml}
          </body>
        </html>
      `);
      frameDoc.close();
    }

    try {
      window.print();
    } catch {
      // Fallback
    }
  }
}

export const fileIoService = new FileIoService();

export function downloadFile(content: BlobPart, filename: string, mimeType = "application/octet-stream"): void {
  fileIoService.downloadFile(content, filename, mimeType);
}
