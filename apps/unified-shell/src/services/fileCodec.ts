import { DocumentType, IPC } from "../ipc";
import { writerState, WordPage } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { binaryPackageService } from "./binaryPackageService";
import { downloadFile } from "./fileIoService";
import JSZip from "jszip";

export interface SaveFileOptions {
  filename: string;
  customPath?: string;
}

export function showSavedToast(fileName: string): void {
  if (typeof document === "undefined") return;

  let toast = document.getElementById("save-toast-banner");
  if (!toast) {
    toast = document.createElement("div");
    toast.id = "save-toast-banner";
    toast.className = "toast-notification-banner save-toast-banner";
    toast.style.position = "fixed";
    toast.style.bottom = "24px";
    toast.style.right = "24px";
    toast.style.zIndex = "99999";
    toast.style.backgroundColor = "var(--bg-surface, #1e293b)";
    toast.style.color = "var(--text-primary, #f8fafc)";
    toast.style.border = "1px solid #10b981";
    toast.style.borderRadius = "8px";
    toast.style.boxShadow = "0 10px 25px -5px rgba(0, 0, 0, 0.4)";
    toast.style.padding = "12px 20px";
    toast.style.fontSize = "13px";
    toast.style.fontWeight = "600";
    toast.style.display = "flex";
    toast.style.alignItems = "center";
    toast.style.gap = "8px";
    toast.style.transition = "opacity 0.3s ease, transform 0.3s ease";
    document.body.appendChild(toast);
  }

  const cleanName = fileName.split(/[/\\]/).pop() || fileName;
  toast.textContent = `✅ Saved successfully: ${cleanName}`;
  toast.style.opacity = "1";
  toast.style.transform = "translateY(0)";

  const existingTimer = (toast as unknown as { _hideTimer?: ReturnType<typeof setTimeout> })._hideTimer;
  if (existingTimer) clearTimeout(existingTimer);

  (toast as unknown as { _hideTimer?: ReturnType<typeof setTimeout> })._hideTimer = setTimeout(() => {
    if (toast) {
      toast.style.opacity = "0";
      toast.style.transform = "translateY(10px)";
    }
  }, 3000);
}

export class FileCodecService {
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
      lower.endsWith(".json") ||
      lower.endsWith(".ini")
    ) {
      return "Notes";
    }
    return "Word";
  }

  public async saveFile(
    docTypeOrFilename: DocumentType | string,
    optionsOrDocType?: SaveFileOptions | DocumentType
  ): Promise<string> {
    let docType: DocumentType;
    let filename: string;
    let customPath: string | undefined;

    if (typeof optionsOrDocType === "object" && optionsOrDocType !== null) {
      docType = docTypeOrFilename as DocumentType;
      filename = optionsOrDocType.filename;
      customPath = optionsOrDocType.customPath;
    } else if (typeof optionsOrDocType === "string") {
      filename = docTypeOrFilename;
      docType = optionsOrDocType as DocumentType;
    } else {
      filename = docTypeOrFilename;
      docType = this.detectDocType(filename);
    }
    let bytes: Uint8Array;
    let mime = "application/octet-stream";

    if (docType === "Word") {
      const text = writerState.getTextContent();
      const paras = writerState.doc.pages[0]?.paragraphs?.map((p) => ({
        text: p.runs.map((r) => r.text).join(""),
        bold: p.runs.some((r) => r.bold),
        italic: p.runs.some((r) => r.italic),
      })) || [{ text }];
      bytes = await binaryPackageService.buildDocx(text, paras);
      mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    } else if (docType === "Excel") {
      const isCsv = filename.toLowerCase().endsWith(".csv") || filename.toLowerCase().endsWith(".tsv");
      if (isCsv) {
        const csvContent = calcState.exportCSV();
        bytes = new TextEncoder().encode(csvContent);
        mime = "text/csv";
      } else {
        const sheetsPayload = calcState.sheets.map((s) => {
          const cells: Record<string, string> = {};
          for (const [coord, cell] of s.cells.entries()) {
            if (cell) {
              const val = cell.raw || (cell.computed !== undefined ? String(cell.computed) : "");
              if (val !== "") cells[coord] = val;
            }
          }
          return { name: s.name, cells };
        });
        bytes = await binaryPackageService.buildXlsx(sheetsPayload);
        mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
      }
    } else if (docType === "PowerPoint") {
      bytes = await binaryPackageService.buildPptx(slidesState.slides);
      mime = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
    } else if (docType === "PDF") {
      bytes = pdfState.exportBinaryPdf();
      mime = "application/pdf";
    } else {
      const content = notesState.getContent();
      bytes = new TextEncoder().encode(content);
      mime = filename.endsWith(".md") ? "text/markdown" : "text/plain";
    }

    let downloadPayload: BlobPart = bytes as unknown as BlobPart;
    if (docType === "Notes") {
      downloadPayload = notesState.getContent();
    }

    let savedPath = customPath || filename;

    const hasTauri =
      typeof window !== "undefined" &&
      Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

    if (hasTauri) {
      try {
        const res = await IPC.saveFileBytes(filename, bytes, customPath);
        if (res) savedPath = res;
      } catch (err) {
        console.warn("IPC.saveFileBytes error, fallback to browser download:", err);
        downloadFile(downloadPayload, filename, mime);
      }
    } else {
      downloadFile(downloadPayload, filename, mime);
    }

    showSavedToast(savedPath);
    return savedPath;
  }

  public async openFile(
    bufferOrText: ArrayBuffer | Uint8Array | string,
    fileName: string,
    forcedDocType?: DocumentType
  ): Promise<boolean> {
    const docType = forcedDocType || this.detectDocType(fileName);
    const isBuffer = typeof bufferOrText !== "string";
    const uint8: Uint8Array = isBuffer
      ? bufferOrText instanceof Uint8Array
        ? bufferOrText
        : new Uint8Array(bufferOrText)
      : new TextEncoder().encode(bufferOrText);

    try {
      if (docType === "Word") {
        let extractedText = "";
        let headerText = "";
        let footerText = "";
        const isDocx = fileName.toLowerCase().endsWith(".docx");

        if (isDocx) {
          try {
            const zip = await JSZip.loadAsync(uint8);
            const docFile = zip.file("word/document.xml");
            if (docFile) {
              const xml = await docFile.async("string");
              const pMatches = xml.match(/<w:p[\s>].*?<\/w:p>/gs) || [];
              const paras: string[] = [];
              for (const p of pMatches) {
                const tMatches = p.match(/<w:t[^>]*>(.*?)<\/w:t>/gs) || [];
                const line = tMatches
                  .map((t) => t.replace(/<w:t[^>]*>/, "").replace(/<\/w:t>/, ""))
                  .join("")
                  .replace(/&amp;/g, "&")
                  .replace(/&lt;/g, "<")
                  .replace(/&gt;/g, ">");
                paras.push(line);
              }
              extractedText = paras.join("\n");
            }

            // Check for header
            const headerFile = zip.file("word/header1.xml") || zip.file("word/header.xml");
            if (headerFile) {
              const hXml = await headerFile.async("string");
              const ht = (hXml.match(/<w:t[^>]*>(.*?)<\/w:t>/gs) || [])
                .map((t) => t.replace(/<w:t[^>]*>/, "").replace(/<\/w:t>/, ""))
                .join("");
              headerText = ht.trim();
            }

            // Check for footer
            const footerFile = zip.file("word/footer1.xml") || zip.file("word/footer.xml");
            if (footerFile) {
              const fXml = await footerFile.async("string");
              const ft = (fXml.match(/<w:t[^>]*>(.*?)<\/w:t>/gs) || [])
                .map((t) => t.replace(/<w:t[^>]*>/, "").replace(/<\/w:t>/, ""))
                .join("");
              footerText = ft.trim();
            }
          } catch {
            extractedText = new TextDecoder().decode(uint8);
          }
        } else {
          extractedText = new TextDecoder().decode(uint8);
        }

        writerState.loadDocumentFromText(extractedText || " ", fileName);
        if (headerText) writerState.doc.headerText = headerText;
        if (footerText) writerState.doc.footerText = footerText;

        const p0 = writerState.doc.pages[0];
        if (p0) {
          (p0 as WordPage & { bodyHtml?: string; headerHtml?: string; footerHtml?: string }).bodyHtml = extractedText;
          (p0 as WordPage & { bodyHtml?: string; headerHtml?: string; footerHtml?: string }).headerHtml = headerText;
          (p0 as WordPage & { bodyHtml?: string; headerHtml?: string; footerHtml?: string }).footerHtml = footerText;
        }
        return true;
      }

      if (docType === "Excel") {
        const isXlsx = fileName.toLowerCase().endsWith(".xlsx") || fileName.toLowerCase().endsWith(".xls");
        if (isXlsx) {
          const parsed = await binaryPackageService.parseXlsx(uint8);
          if (parsed.sheets && parsed.sheets.length > 0) {
            calcState.resetToBlank(parsed.sheets[0].name);
            for (const [coord, val] of Object.entries(parsed.sheets[0].cells)) {
              calcState.setCellValue(coord, val);
            }
            for (let i = 1; i < parsed.sheets.length; i++) {
              const sh = parsed.sheets[i];
              calcState.addSheet(sh.name);
              for (const [coord, val] of Object.entries(sh.cells)) {
                calcState.setCellValue(coord, val);
              }
            }
            calcState.switchSheet(calcState.sheets[0].id);
          } else {
            calcState.resetToBlank("Sheet1");
            for (const [coord, val] of Object.entries(parsed.cells)) {
              calcState.setCellValue(coord, val);
            }
          }
        } else {
          const text = new TextDecoder().decode(uint8);
          calcState.resetToBlank("Sheet1");
          calcState.importFromCsv(text);
        }
        calcState.recalculateAll();
        return true;
      }

      if (docType === "PowerPoint") {
        const isPptx = fileName.toLowerCase().endsWith(".pptx");
        if (isPptx) {
          const parsed = await binaryPackageService.parsePptx(uint8);
          if (parsed.slides && parsed.slides.length > 0) {
            slidesState.slides = parsed.slides.map((s, idx) => {
              if (s.shapes && s.shapes.length > 0) {
                return {
                  id: String(idx + 1),
                  title: s.title || `Slide ${idx + 1}`,
                  shapes: s.shapes,
                  notes: s.notes,
                  transition: s.transition,
                  transitionConfig: s.transitionConfig,
                  background: s.background,
                  animations: s.animations,
                  layout: s.layout,
                };
              }
              const shapeTexts = s.text && s.text.length > 1 ? s.text.slice(1) : (s.text || [s.title || ""]);
              return {
                id: String(idx + 1),
                title: s.title || `Slide ${idx + 1}`,
                notes: s.notes,
                transition: s.transition,
                transitionConfig: s.transitionConfig,
                background: s.background,
                shapes: shapeTexts.map((t, tIdx) => ({
                  id: `shape-${idx}-${tIdx}`,
                  type: "text" as const,
                  x: 80,
                  y: 100 + tIdx * 50,
                  w: 500,
                  h: 40,
                  fill: "transparent",
                  text: t,
                })),
              };
            });
            slidesState.currentSlideIndex = 0;
          }
        } else {
          slidesState.resetToBlank(fileName);
        }
        return true;
      }

      if (docType === "PDF") {
        const parsed = binaryPackageService.parsePdf(uint8);
        if (parsed.pages && parsed.pages.length > 0) {
          pdfState.loadPdfDocument(
            parsed.pages.map((p, i) => ({
              id: `page-${i + 1}`,
              pageNumber: i + 1,
              width: p.width,
              height: p.height,
              rotation: 0,
              title: `Page ${i + 1}`,
              textBlocks: p.textBlocks.map((tb, tbIdx) => ({
                id: `tb-${i + 1}-${tbIdx + 1}`,
                x: tb.x,
                y: tb.y,
                text: tb.text,
                fontSize: tb.fontSize || 12,
              })),
            }))
          );
        } else if (parsed.pageCount > 0) {
          pdfState.resetToBlank(fileName);
          while (pdfState.pages.length < parsed.pageCount) {
            pdfState.insertBlankPage(pdfState.pages.length);
          }
          if (pdfState.pages[0]) {
            pdfState.pages[0].textBlocks = [
              {
                id: "tb-1",
                x: 72,
                y: 120,
                text: parsed.text,
                fontSize: 14,
              },
            ];
          }
        }
        return true;
      }

      // Notes (.txt, .md, .log, .json)
      const text = new TextDecoder().decode(uint8);
      notesState.resetToBlank(fileName);
      notesState.setContent(text);
      return true;
    } catch (e) {
      console.warn("Failed to open file via fileCodec:", e);
      return false;
    }
  }

  public async openFileFromDisk(filePath: string): Promise<boolean> {
    try {
      const bytes = await IPC.readFileBytes(filePath);
      const fileName = filePath.split(/[/\\]/).pop() || filePath;
      return await this.openFile(bytes, fileName);
    } catch (err) {
      console.warn("openFileFromDisk failed:", err);
      return false;
    }
  }
}

export const fileCodec = new FileCodecService();
