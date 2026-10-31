export interface WordRun {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  subscript?: boolean;
  superscript?: boolean;
  font?: string;
  size?: number;
  color?: string;
  highlight?: string;
  linkUrl?: string;
  insertion?: boolean;
  deletion?: boolean;
}

export interface WordTableCell {
  text: string;
}

export interface WordTableRow {
  cells: WordTableCell[];
}

export interface WordTableData {
  rows: WordTableRow[];
  colCount: number;
}

export interface WordImageData {
  src: string;
  width: number;
  height: number;
  caption?: string;
}

export interface WordShapeData {
  shapeType: "rect" | "roundRect" | "ellipse" | "triangle" | "arrow" | "star" | "callout";
  width: number;
  height: number;
  color?: string;
  x?: number;
  y?: number;
  text?: string;
}

export interface WordComment {
  id: string;
  text: string;
  author: string;
  createdAt: string;
  paragraphId?: string;
}

export interface WordParagraph {
  id: string;
  style: string;
  runs: WordRun[];
  alignment: "left" | "center" | "right" | "justify";
  indent?: number;
  lineSpacing?: number;
  table?: WordTableData;
  image?: WordImageData;
  shape?: WordShapeData;
  commentId?: string;
  shading?: string;
  border?: string;
}

export interface DocPage {
  pageNumber: number;
  paragraphs: WordParagraph[];
  headerText?: string;
  bodyHtml?: string;
  footerText?: string;
}

export type WordPage = DocPage;

export interface WordDocumentModel {
  id: string;
  title: string;
  pages: DocPage[];
  pageOrientation?: "portrait" | "landscape";
  margins?: "normal" | "narrow" | "moderate" | "wide" | "custom";
  pageSize?: "letter" | "a4" | "legal";
  columns?: number;
  headerText?: string;
  footerText?: string;
  comments?: WordComment[];
  trackChanges?: boolean;
  viewMode?: "print" | "web" | "read";
  backgroundColor?: string;
  pageBorder?: string;
  watermark?: string;
  lineNumbers?: boolean;
  hyphenation?: boolean;
  showFormattingMarks?: boolean;
  readOnly?: boolean;
  gridlines?: boolean;
  darkCanvas?: boolean;
  citationStyle?: string;
  footnotes?: { id: string; num: number; text: string; pageIndex: number }[];
  citations?: { id: string; author: string; title: string; year: number; style: string }[];
}

export interface CaretPosition {
  pageIndex: number;
  paragraphIndex: number;
  runIndex: number;
  offset: number;
}

export interface SelectionRange {
  anchor: CaretPosition;
  focus: CaretPosition;
}

export function comparePositions(a: CaretPosition, b: CaretPosition): number {
  if (a.pageIndex !== b.pageIndex) return a.pageIndex - b.pageIndex;
  if (a.paragraphIndex !== b.paragraphIndex) return a.paragraphIndex - b.paragraphIndex;
  if (a.runIndex !== b.runIndex) return a.runIndex - b.runIndex;
  return a.offset - b.offset;
}

export interface WordFormatState {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  font: string;
  size: number;
  color: string;
  alignment: "left" | "center" | "right" | "justify";
  lineSpacing: number;
}

export class WriterState {
  doc: WordDocumentModel = {
    id: "doc-1",
    title: "Untitled 1.docx",
    pageOrientation: "portrait",
    margins: "normal",
    pageSize: "letter",
    headerText: "",
    footerText: "",
    pages: [
      {
        pageNumber: 1,
        headerText: "",
        bodyHtml: "",
        footerText: "",
        paragraphs: [
          {
            id: "p1",
            style: "Normal",
            alignment: "left",
            runs: [{ text: "", font: "Calibri", size: 11 }],
          },
        ],
      },
    ],
  };

  public editingZone: "body" | "header" | "footer" = "body";

  cursor: CaretPosition = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
  selection: SelectionRange | null = null;
  zoom: number = 1.0;
  activeFormat = {
    bold: false,
    italic: false,
    underline: false,
    strikethrough: false,
    font: "Calibri",
    size: 11,
    color: "#242424",
    alignment: "left" as "left" | "center" | "right" | "justify",
    lineSpacing: 1.15,
  };
  get activeFormatting() {
    return this.activeFormat;
  }

  get pages(): WordPage[] {
    return this.doc.pages;
  }

  public setEditingZone(zone: "body" | "header" | "footer"): void {
    this.editingZone = zone;
  }

  public getEditingZone(): "body" | "header" | "footer" {
    return this.editingZone;
  }

  public setHeaderText(text: string, pageIndex?: number): void {
    this.doc.headerText = text;
    if (pageIndex !== undefined && this.doc.pages[pageIndex]) {
      this.doc.pages[pageIndex].headerText = text;
    } else {
      this.doc.pages.forEach((p) => {
        p.headerText = text;
      });
    }
  }

  public setFooterText(text: string, pageIndex?: number): void {
    this.doc.footerText = text;
    if (pageIndex !== undefined && this.doc.pages[pageIndex]) {
      this.doc.pages[pageIndex].footerText = text;
    } else {
      this.doc.pages.forEach((p) => {
        p.footerText = text;
      });
    }
  }

  public get headerText(): string | undefined {
    return this.doc.pages[this.cursor.pageIndex]?.headerText ?? this.doc.headerText;
  }

  public get footerText(): string | undefined {
    return this.doc.pages[this.cursor.pageIndex]?.footerText ?? this.doc.footerText;
  }

  public getHeaderText(pageIndex = this.cursor.pageIndex): string {
    return this.doc.pages[pageIndex]?.headerText ?? this.doc.headerText ?? "";
  }

  public getFooterText(pageIndex = this.cursor.pageIndex): string {
    return this.doc.pages[pageIndex]?.footerText ?? this.doc.footerText ?? "";
  }

  public getBodyHtml(pageIndex = this.cursor.pageIndex): string {
    return this.doc.pages[pageIndex]?.bodyHtml ?? "";
  }

  public setBodyHtml(html: string, pageIndex = this.cursor.pageIndex): void {
    if (this.doc.pages[pageIndex]) {
      this.doc.pages[pageIndex].bodyHtml = html;
    }
  }

  public get fontFamily(): string {
    return this.activeFormat.font;
  }
  public set fontFamily(val: string) {
    this.activeFormat.font = val;
  }

  public get fontSize(): number {
    return this.activeFormat.size;
  }
  public set fontSize(val: number) {
    this.activeFormat.size = val;
  }

  get paragraphs(): WordParagraph[] {
    return this.doc.pages.flatMap((p) => p.paragraphs);
  }
  private history: string[] = [];
  private redoStack: string[] = [];

  constructor() {
    this.saveSnapshot();
  }

  public saveSnapshot() {
    if (this.history.length > 50) this.history.shift();
    this.history.push(JSON.stringify(this.doc));
    this.redoStack = [];
  }

  public undo() {
    if (this.history.length > 1) {
      const current = this.history.pop()!;
      this.redoStack.push(current);
      const prev = this.history[this.history.length - 1];
      if (prev) {
        this.doc = JSON.parse(prev);
      }
    }
  }

  public redo() {
    if (this.redoStack.length > 0) {
      const next = this.redoStack.pop()!;
      this.history.push(next);
      this.doc = JSON.parse(next);
    }
  }

  public getWordCount(): number {
    return this.getTotalWords();
  }

  public setZoom(zoom: number) {
    this.zoom = Math.max(0.5, Math.min(2.5, zoom));
  }

  public isSelectionCollapsed(): boolean {
    if (!this.selection) return true;
    const a = this.selection.anchor;
    const f = this.selection.focus;
    return (
      a.pageIndex === f.pageIndex &&
      a.paragraphIndex === f.paragraphIndex &&
      a.runIndex === f.runIndex &&
      a.offset === f.offset
    );
  }

  public selectAll() {
    const lastPage = this.doc.pages.length - 1;
    const lastP = this.doc.pages[lastPage].paragraphs.length - 1;
    const lastR = this.doc.pages[lastPage].paragraphs[lastP].runs.length - 1;
    const lastOffset = this.doc.pages[lastPage].paragraphs[lastP].runs[lastR].text.length;

    this.selection = {
      anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 },
      focus: { pageIndex: lastPage, paragraphIndex: lastP, runIndex: lastR, offset: lastOffset },
    };
  }

  public clearSelection() {
    this.selection = null;
  }

  public deleteSelection(): boolean {
    if (!this.selection || this.isSelectionCollapsed()) return false;
    this.saveSnapshot();

    const isForward = comparePositions(this.selection.anchor, this.selection.focus) <= 0;
    const start = isForward ? this.selection.anchor : this.selection.focus;
    const end = isForward ? this.selection.focus : this.selection.anchor;

    const startPage = this.doc.pages[start.pageIndex];
    const endPage = this.doc.pages[end.pageIndex];
    if (!startPage || !endPage) return false;

    const startPara = startPage.paragraphs[start.paragraphIndex];
    const endPara = endPage.paragraphs[end.paragraphIndex];
    if (!startPara || !endPara) return false;

    if (start.pageIndex === end.pageIndex && start.paragraphIndex === end.paragraphIndex) {
      if (start.runIndex === end.runIndex) {
        const run = startPara.runs[start.runIndex];
        if (run) {
          run.text = run.text.slice(0, start.offset) + run.text.slice(end.offset);
        }
      } else {
        const startRun = startPara.runs[start.runIndex];
        const endRun = startPara.runs[end.runIndex];
        const remainingEndText = endRun ? endRun.text.slice(end.offset) : "";
        if (startRun) {
          startRun.text = startRun.text.slice(0, start.offset) + remainingEndText;
        }
        startPara.runs.splice(start.runIndex + 1, end.runIndex - start.runIndex);
      }
    } else {
      const startRun = startPara.runs[start.runIndex];
      const endRemainingRuns = endPara.runs.slice(end.runIndex);
      if (endRemainingRuns.length > 0) {
        endRemainingRuns[0].text = endRemainingRuns[0].text.slice(end.offset);
      }
      if (startRun) {
        startRun.text = startRun.text.slice(0, start.offset);
      }
      startPara.runs = startPara.runs.slice(0, start.runIndex + 1).concat(endRemainingRuns);

      if (start.pageIndex === end.pageIndex) {
        startPage.paragraphs.splice(start.paragraphIndex + 1, end.paragraphIndex - start.paragraphIndex);
      } else {
        startPage.paragraphs.splice(start.paragraphIndex + 1);
        for (let p = start.pageIndex + 1; p < end.pageIndex; p++) {
          this.doc.pages[p].paragraphs = [];
        }
        endPage.paragraphs.splice(0, end.paragraphIndex + 1);
      }
    }

    this.cursor = { ...start };
    this.clearSelection();
    return true;
  }

  public insertText(text: string): void {
    if (this.selection && !this.isSelectionCollapsed()) {
      this.deleteSelection();
    }
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    let para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) {
      para = {
        id: `p-${Date.now()}`,
        style: "Normal",
        alignment: this.activeFormat.alignment,
        runs: [],
      };
      page.paragraphs.push(para);
      this.cursor.paragraphIndex = page.paragraphs.length - 1;
    }

    if (this.doc.trackChanges) {
      const trackedRun: WordRun = {
        text,
        font: this.activeFormat.font,
        size: this.activeFormat.size,
        bold: this.activeFormat.bold,
        italic: this.activeFormat.italic,
        color: "#2563eb",
        underline: true,
        insertion: true,
      };
      para.runs.splice(this.cursor.runIndex + 1, 0, trackedRun);
      this.cursor.runIndex++;
      this.cursor.offset = text.length;
      this.clearSelection();
      this.saveSnapshot();
      return;
    }

    let run = para.runs[this.cursor.runIndex];
    if (!run) {
      run = {
        text: "",
        font: this.activeFormat.font,
        size: this.activeFormat.size,
        bold: this.activeFormat.bold,
        italic: this.activeFormat.italic,
        color: this.activeFormat.color,
      };
      para.runs.push(run);
      this.cursor.runIndex = para.runs.length - 1;
    }

    run.text = run.text.slice(0, this.cursor.offset) + text + run.text.slice(this.cursor.offset);
    this.cursor.offset += text.length;
    this.clearSelection();
    this.saveSnapshot();
  }

  public deleteBackward(): void {
    if (this.selection && !this.isSelectionCollapsed()) {
      this.deleteSelection();
      return;
    }
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.cursor.runIndex];
    if (!run) return;

    if (this.doc.trackChanges) {
      if (this.cursor.offset > 0) {
        const charToDelete = run.text.slice(this.cursor.offset - 1, this.cursor.offset);
        run.text = run.text.slice(0, this.cursor.offset - 1) + run.text.slice(this.cursor.offset);
        const delRun: WordRun = {
          text: charToDelete,
          font: run.font,
          size: run.size,
          color: "#dc2626",
          strikethrough: true,
          deletion: true,
        };
        para.runs.splice(this.cursor.runIndex + 1, 0, delRun);
        this.cursor.offset--;
        this.saveSnapshot();
        return;
      }
    }

    if (this.cursor.offset > 0) {
      run.text = run.text.slice(0, this.cursor.offset - 1) + run.text.slice(this.cursor.offset);
      this.cursor.offset--;
      this.saveSnapshot();
    } else if (this.cursor.offset === 0) {
      if (this.cursor.runIndex > 0) {
        this.cursor.runIndex--;
        const prevRun = para.runs[this.cursor.runIndex];
        this.cursor.offset = prevRun.text.length;
        if (prevRun.text.length > 0) {
          prevRun.text = prevRun.text.slice(0, -1);
          this.cursor.offset--;
          this.saveSnapshot();
        }
      } else if (this.cursor.paragraphIndex > 0) {
        const prevP = page.paragraphs[this.cursor.paragraphIndex - 1];
        const lastRunIdx = prevP.runs.length > 0 ? prevP.runs.length - 1 : 0;
        const lastRun = prevP.runs[lastRunIdx];
        const mergeOffset = lastRun ? lastRun.text.length : 0;

        prevP.runs.push(...para.runs);
        page.paragraphs.splice(this.cursor.paragraphIndex, 1);

        this.cursor.paragraphIndex--;
        this.cursor.runIndex = lastRunIdx;
        this.cursor.offset = mergeOffset;
        this.saveSnapshot();
      } else if (this.cursor.pageIndex > 0) {
        this.cursor.pageIndex--;
        const prevPage = this.doc.pages[this.cursor.pageIndex];
        this.cursor.paragraphIndex = prevPage.paragraphs.length - 1;
        const prevP = prevPage.paragraphs[this.cursor.paragraphIndex];
        this.cursor.runIndex = prevP.runs.length - 1;
        this.cursor.offset = prevP.runs[this.cursor.runIndex].text.length;
      }
    }
  }

  public deleteForward(): void {
    if (this.selection && !this.isSelectionCollapsed()) {
      this.deleteSelection();
      return;
    }
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.cursor.runIndex];
    if (!run) return;

    if (this.cursor.offset < run.text.length) {
      run.text = run.text.slice(0, this.cursor.offset) + run.text.slice(this.cursor.offset + 1);
      this.saveSnapshot();
    } else if (this.cursor.runIndex + 1 < para.runs.length) {
      const nextRun = para.runs[this.cursor.runIndex + 1];
      if (nextRun.text.length > 0) {
        nextRun.text = nextRun.text.slice(1);
        this.saveSnapshot();
      }
    } else if (this.cursor.paragraphIndex + 1 < page.paragraphs.length) {
      const nextP = page.paragraphs[this.cursor.paragraphIndex + 1];
      para.runs.push(...nextP.runs);
      page.paragraphs.splice(this.cursor.paragraphIndex + 1, 1);
      this.saveSnapshot();
    }
  }

  public splitParagraph(): void {
    if (this.selection && !this.isSelectionCollapsed()) {
      this.deleteSelection();
    }
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.cursor.runIndex];
    if (!run) return;

    const textBefore = run.text.slice(0, this.cursor.offset);
    const textAfter = run.text.slice(this.cursor.offset);
    run.text = textBefore;

    const remainingRuns = para.runs.slice(this.cursor.runIndex + 1);
    para.runs = para.runs.slice(0, this.cursor.runIndex + 1);

    const newParaRuns: WordRun[] = [
      {
        text: textAfter,
        font: run.font || this.activeFormat.font,
        size: run.size || this.activeFormat.size,
        bold: run.bold,
        italic: run.italic,
        color: run.color,
      },
      ...remainingRuns,
    ];

    const newPara: WordParagraph = {
      id: `p-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
      style: para.style === "Title" || para.style.startsWith("Heading") ? "Normal" : para.style,
      alignment: para.alignment,
      runs: newParaRuns,
    };

    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, newPara);
    this.cursor.paragraphIndex++;
    this.cursor.runIndex = 0;
    this.cursor.offset = 0;
    this.clearSelection();
    this.saveSnapshot();
  }

  public resetToBlank(title: string = "Untitled.docx"): void {
    this.zoom = 1.0;
    this.doc = {
      id: `doc-${Date.now()}`,
      title,
      pageOrientation: "portrait",
      margins: "normal",
      pageSize: "letter",
      headerText: "",
      footerText: "",
      pages: [
        {
          pageNumber: 1,
          headerText: "",
          bodyHtml: "",
          footerText: "",
          paragraphs: [
            {
              id: "p1",
              style: "Normal",
              alignment: "left",
              runs: [{ text: "", font: "Calibri", size: 11 }],
            },
          ],
        },
      ],
    };
    this.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
    this.clearSelection();
    this.saveSnapshot();
  }

  public getTextContent(): string {
    return this.doc.pages
      .map((p) =>
        p.paragraphs
          .map((para) => para.runs.map((r) => r.text).join(""))
          .join("\n")
      )
      .join("\n\n");
  }

  public clipboardText = "";
  public getSelectedText(): string {
    if (!this.selection || this.isSelectionCollapsed()) return "";
    const isForward = comparePositions(this.selection.anchor, this.selection.focus) <= 0;
    const start = isForward ? this.selection.anchor : this.selection.focus;
    const end = isForward ? this.selection.focus : this.selection.anchor;
    let result = "";
    for (let pIdx = start.pageIndex; pIdx <= end.pageIndex; pIdx++) {
      const page = this.doc.pages[pIdx];
      if (!page) continue;
      const startP = pIdx === start.pageIndex ? start.paragraphIndex : 0;
      const endP = pIdx === end.pageIndex ? end.paragraphIndex : page.paragraphs.length - 1;
      for (let p = startP; p <= endP; p++) {
        const para = page.paragraphs[p];
        if (!para) continue;
        const startR = (pIdx === start.pageIndex && p === start.paragraphIndex) ? start.runIndex : 0;
        const endR = (pIdx === end.pageIndex && p === end.paragraphIndex) ? end.runIndex : para.runs.length - 1;
        for (let r = startR; r <= endR; r++) {
          const run = para.runs[r];
          if (!run) continue;
          const sOff = (pIdx === start.pageIndex && p === start.paragraphIndex && r === start.runIndex) ? start.offset : 0;
          const eOff = (pIdx === end.pageIndex && p === end.paragraphIndex && r === end.runIndex) ? end.offset : run.text.length;
          result += run.text.slice(sOff, eOff);
        }
        if (p < endP) result += "\n";
      }
    }
    return result;
  }

  public copy(): string {
    const text = this.getSelectedText();
    if (text) {
      this.clipboardText = text;
    }
    return this.clipboardText;
  }

  public paste(text?: string): void {
    const textToInsert = text !== undefined ? text : this.clipboardText;
    if (textToInsert) {
      this.insertText(textToInsert);
    }
  }

  public loadDocumentFromText(text: string, title = "Document.docx"): void {
    this.resetToBlank(title);
    const lines = text.split(/\r?\n/);
    this.doc.pages[0].paragraphs = lines.map((line, idx) => ({
      id: `p-${idx + 1}`,
      style: "Normal",
      alignment: "left",
      runs: [{ text: line, font: "Calibri", size: 11 }],
    }));
    this.doc.pages[0].bodyHtml = text;
    this.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
    this.saveSnapshot();
  }

  public insertNewPage(): void {
    this.insertPageBreak();
  }

  public deleteEmptyPage(pageIndex?: number): boolean {
    const idx = pageIndex !== undefined ? pageIndex : this.cursor.pageIndex;
    if (this.doc.pages.length <= 1 || idx < 0 || idx >= this.doc.pages.length) {
      return false;
    }
    const page = this.doc.pages[idx];
    const isEmpty = page.paragraphs.every(
      (p) => !p.table && !p.image && !p.shape && p.runs.every((r) => !r.text || r.text.trim() === "")
    );
    if (isEmpty) {
      this.saveSnapshot();
      this.doc.pages.splice(idx, 1);
      this.doc.pages.forEach((p, i) => {
        p.pageNumber = i + 1;
      });
      if (this.cursor.pageIndex >= this.doc.pages.length) {
        this.cursor.pageIndex = this.doc.pages.length - 1;
      }
      const curPage = this.doc.pages[this.cursor.pageIndex];
      this.cursor.paragraphIndex = Math.min(this.cursor.paragraphIndex, curPage.paragraphs.length - 1);
      const curPara = curPage.paragraphs[this.cursor.paragraphIndex];
      this.cursor.runIndex = Math.min(this.cursor.runIndex, curPara.runs.length - 1);
      this.cursor.offset = Math.min(this.cursor.offset, curPara.runs[this.cursor.runIndex].text.length);
      return true;
    }
    return false;
  }

  public deletePage(pageIndex?: number, force = true): boolean {
    const idx = pageIndex !== undefined ? pageIndex : this.cursor.pageIndex;
    if (this.doc.pages.length <= 1 || idx < 0 || idx >= this.doc.pages.length) {
      return false;
    }
    if (!force) {
      return this.deleteEmptyPage(idx);
    }
    this.saveSnapshot();
    this.doc.pages.splice(idx, 1);
    this.doc.pages.forEach((p, i) => {
      p.pageNumber = i + 1;
    });
    if (this.cursor.pageIndex >= this.doc.pages.length) {
      this.cursor.pageIndex = this.doc.pages.length - 1;
    }
    const curPage = this.doc.pages[this.cursor.pageIndex];
    this.cursor.paragraphIndex = Math.min(this.cursor.paragraphIndex, curPage.paragraphs.length - 1);
    const curPara = curPage.paragraphs[this.cursor.paragraphIndex];
    this.cursor.runIndex = Math.min(this.cursor.runIndex, curPara.runs.length - 1);
    this.cursor.offset = Math.min(this.cursor.offset, curPara.runs[this.cursor.runIndex].text.length);
    return true;
  }

  public insertPageBreak(): void {
    this.saveSnapshot();
    const newPageNum = this.doc.pages.length + 1;
    const newPage: DocPage = {
      pageNumber: newPageNum,
      headerText: this.doc.headerText || "",
      bodyHtml: "",
      footerText: this.doc.footerText || "",
      paragraphs: [
        {
          id: `p-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
          style: "Normal",
          alignment: this.activeFormat.alignment,
          runs: [{ text: "", font: this.activeFormat.font, size: this.activeFormat.size }],
        },
      ],
    };
    this.doc.pages.push(newPage);
    this.cursor.pageIndex = this.doc.pages.length - 1;
    this.cursor.paragraphIndex = 0;
    this.cursor.runIndex = 0;
    this.cursor.offset = 0;
    this.clearSelection();
  }

  public autoPaginate(maxHeightPx: number = 864): boolean {
    let pagesChanged = false;
    for (let pIdx = 0; pIdx < this.doc.pages.length; pIdx++) {
      const page = this.doc.pages[pIdx];
      let currentHeight = 0;
      let overflowStartIndex = -1;

      for (let i = 0; i < page.paragraphs.length; i++) {
        const p = page.paragraphs[i];
        const fontSize = p.runs[0]?.size || (p.style === "Heading1" ? 22 : 11);
        const lineSpacing = fontSize * 1.35;
        const totalChars = p.runs.reduce((acc, r) => acc + r.text.length, 0);
        const estimatedLines = Math.max(1, Math.ceil(totalChars / 75));
        const pHeight = estimatedLines * lineSpacing + (p.style.startsWith("Heading") ? 14 : 8);

        if (currentHeight + pHeight > maxHeightPx && i > 0) {
          overflowStartIndex = i;
          break;
        }
        currentHeight += pHeight;
      }

      if (overflowStartIndex !== -1) {
        pagesChanged = true;
        const overflowParas = page.paragraphs.splice(overflowStartIndex);
        if (pIdx + 1 < this.doc.pages.length) {
          this.doc.pages[pIdx + 1].paragraphs.unshift(...overflowParas);
        } else {
          this.doc.pages.push({
            pageNumber: pIdx + 2,
            headerText: page.headerText || this.doc.headerText || "",
            bodyHtml: "",
            footerText: page.footerText || this.doc.footerText || "",
            paragraphs: overflowParas,
          });
        }
      }
    }
    return pagesChanged;
  }

  public insertLineBreak(): void {
    this.insertText("\n");
  }

  public selectWord(pos: CaretPosition): void {
    const page = this.doc.pages[pos.pageIndex];
    if (!page) return;
    const para = page.paragraphs[pos.paragraphIndex];
    if (!para) return;
    const run = para.runs[pos.runIndex];
    if (!run) return;

    const text = run.text;
    let start = Math.min(pos.offset, text.length);
    let end = Math.min(pos.offset, text.length);

    while (start > 0 && /\w/.test(text[start - 1])) {
      start--;
    }
    while (end < text.length && /\w/.test(text[end])) {
      end++;
    }

    this.selection = {
      anchor: { ...pos, offset: start },
      focus: { ...pos, offset: end },
    };
    this.cursor = { ...pos, offset: end };
  }

  public selectParagraph(pos: CaretPosition): void {
    const page = this.doc.pages[pos.pageIndex];
    if (!page) return;
    const para = page.paragraphs[pos.paragraphIndex];
    if (!para) return;

    const lastRunIdx = Math.max(0, para.runs.length - 1);
    const lastRun = para.runs[lastRunIdx];
    const lastOffset = lastRun ? lastRun.text.length : 0;

    this.selection = {
      anchor: { pageIndex: pos.pageIndex, paragraphIndex: pos.paragraphIndex, runIndex: 0, offset: 0 },
      focus: { pageIndex: pos.pageIndex, paragraphIndex: pos.paragraphIndex, runIndex: lastRunIdx, offset: lastOffset },
    };
    this.cursor = { pageIndex: pos.pageIndex, paragraphIndex: pos.paragraphIndex, runIndex: lastRunIdx, offset: lastOffset };
  }

  public moveWordLeft(): void {
    const page = this.doc.pages[this.cursor.pageIndex];
    if (!page) return;
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.cursor.runIndex];
    if (!run) return;

    let offset = this.cursor.offset;
    const text = run.text;
    while (offset > 0 && /\s/.test(text[offset - 1])) {
      offset--;
    }
    while (offset > 0 && !/\s/.test(text[offset - 1])) {
      offset--;
    }
    this.cursor.offset = offset;
  }

  public moveWordRight(): void {
    const page = this.doc.pages[this.cursor.pageIndex];
    if (!page) return;
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.cursor.runIndex];
    if (!run) return;

    let offset = this.cursor.offset;
    const text = run.text;
    while (offset < text.length && !/\s/.test(text[offset])) {
      offset++;
    }
    while (offset < text.length && /\s/.test(text[offset])) {
      offset++;
    }
    this.cursor.offset = offset;
  }

  public getTotalWords(): number {
    let words = 0;
    for (const page of this.doc.pages) {
      for (const p of page.paragraphs) {
        for (const r of p.runs) {
          words += r.text.trim().split(/\s+/).filter(Boolean).length;
        }
      }
    }
    return words;
  }

  public getTotalPages(): number {
    return this.doc.pages.length;
  }

  public getDetailedWordCount() {
    let words = 0;
    let charactersNoSpaces = 0;
    let charactersWithSpaces = 0;
    let paragraphs = 0;
    let lines = 0;

    this.doc.pages.forEach((page) => {
      paragraphs += page.paragraphs.length;
      page.paragraphs.forEach((p) => {
        const fullText = p.runs.map((r) => r.text).join("");
        const pWords = fullText.trim().split(/\s+/).filter(Boolean);
        words += pWords.length;
        charactersWithSpaces += fullText.length;
        charactersNoSpaces += fullText.replace(/\s+/g, "").length;
        lines += Math.max(1, Math.ceil(fullText.length / 75));
      });
    });

    return {
      pages: Math.max(1, this.doc.pages.length),
      words,
      charactersNoSpaces,
      charactersWithSpaces,
      paragraphs,
      lines,
    };
  }

  public insertTable(rowCount = 3, colCount = 3): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const rows: WordTableRow[] = [];
    for (let r = 0; r < rowCount; r++) {
      const cells: WordTableCell[] = [];
      for (let c = 0; c < colCount; c++) {
        cells.push({ text: r === 0 ? `Header ${c + 1}` : `Data ${r},${c + 1}` });
      }
      rows.push({ cells });
    }

    const tablePara: WordParagraph = {
      id: `tbl-${Date.now()}`,
      style: "Normal",
      alignment: "left",
      runs: [{ text: "", font: "Calibri", size: 11 }],
      table: { rows, colCount },
    };

    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, tablePara);
    this.cursor.paragraphIndex++;
  }

  public addTableRow(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex];
    const para = page?.paragraphs[this.cursor.paragraphIndex];
    if (para?.table) {
      const newCells: WordTableCell[] = [];
      for (let c = 0; c < para.table.colCount; c++) {
        newCells.push({ text: `Row ${para.table.rows.length + 1}, Col ${c + 1}` });
      }
      para.table.rows.push({ cells: newCells });
    }
  }

  public deleteTableRow(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex];
    const para = page?.paragraphs[this.cursor.paragraphIndex];
    if (para?.table && para.table.rows.length > 1) {
      para.table.rows.pop();
    }
  }

  public addTableColumn(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex];
    const para = page?.paragraphs[this.cursor.paragraphIndex];
    if (para?.table) {
      para.table.colCount++;
      para.table.rows.forEach((row, rIdx) => {
        row.cells.push({
          text: rIdx === 0 ? `Header ${para.table!.colCount}` : `Cell ${rIdx + 1},${para.table!.colCount}`,
        });
      });
    }
  }

  public deleteTableColumn(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex];
    const para = page?.paragraphs[this.cursor.paragraphIndex];
    if (para?.table && para.table.colCount > 1) {
      para.table.colCount--;
      para.table.rows.forEach((row) => {
        row.cells.pop();
      });
    }
  }

  public insertPicture(
    src = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='100' height='60'></svg>",
    width = 320,
    height = 180,
    caption = "Embedded Picture"
  ): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const imgPara: WordParagraph = {
      id: `img-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      runs: [{ text: `[Picture: ${caption}]`, font: "Calibri", size: 10, italic: true, color: "#6b7280" }],
      image: { src, width, height, caption },
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, imgPara);
    this.cursor.paragraphIndex++;
  }

  public insertShape(shapeType: "rect" | "roundRect" | "ellipse" | "triangle" | "arrow" | "star" | "callout" = "rect"): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const shapePara: WordParagraph = {
      id: `shp-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      runs: [{ text: `[Shape: ${shapeType}]`, font: "Calibri", size: 10, italic: true, color: "#6b7280" }],
      shape: { shapeType, width: 140, height: 90, color: "#2563eb", x: 40, y: 40, text: "" },
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, shapePara);
    this.cursor.paragraphIndex++;
  }

  public sortParagraphs(pageIndex: number = this.cursor.pageIndex): void {
    this.saveSnapshot();
    const page = this.doc.pages[pageIndex] || this.doc.pages[0];
    if (!page) return;
    const textParas = page.paragraphs.filter((p) => !p.shape && !p.image && !p.table);
    const nonText = page.paragraphs.filter((p) => p.shape || p.image || p.table);
    textParas.sort((a, b) => {
      const textA = a.runs.map((r) => r.text).join("").trim();
      const textB = b.runs.map((r) => r.text).join("").trim();
      return textA.localeCompare(textB);
    });
    page.paragraphs = [...textParas, ...nonText];
  }

  public changeCase(mode: "uppercase" | "lowercase" | "titlecase" | "toggle" = "uppercase"): void {
    this.saveSnapshot();
    this.applyRunMutation((r) => {
      if (mode === "uppercase") {
        r.text = r.text.toUpperCase();
      } else if (mode === "lowercase") {
        r.text = r.text.toLowerCase();
      } else if (mode === "titlecase") {
        r.text = r.text.replace(/\b\w/g, (c) => c.toUpperCase());
      } else if (mode === "toggle") {
        r.text = r.text.split("").map((c) => (c === c.toUpperCase() ? c.toLowerCase() : c.toUpperCase())).join("");
      }
    });
  }

  public insertCoverPage(): void {
    this.saveSnapshot();
    const coverPage: DocPage = {
      pageNumber: 1,
      headerText: "",
      bodyHtml: "",
      footerText: "",
      paragraphs: [
        {
          id: `cov-title-${Date.now()}`,
          style: "Title",
          alignment: "left",
          runs: [{ text: "Document Title", font: "Calibri", size: 36, bold: true, color: "#1f497d" }],
        },
        {
          id: `cov-sub-${Date.now()}`,
          style: "Subtitle",
          alignment: "left",
          runs: [{ text: "Subtitle / Executive Abstract", font: "Calibri", size: 14, italic: true, color: "#595959" }],
        },
        {
          id: `cov-meta-${Date.now()}`,
          style: "Normal",
          alignment: "left",
          runs: [{ text: `Author: AetherOffice User  |  Date: ${new Date().toLocaleDateString()}`, font: "Calibri", size: 10, color: "#9ca3af" }],
        },
      ],
    };
    this.doc.pages.unshift(coverPage);
    this.doc.pages.forEach((p, idx) => (p.pageNumber = idx + 1));
    this.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
  }

  public insertSymbol(symbol: string = "©"): void {
    this.insertText(symbol);
  }

  public insertChart(chartType: "column" | "bar" | "line" | "pie" = "column"): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const chartPara: WordParagraph = {
      id: `chart-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      runs: [{ text: `[Chart: ${chartType.toUpperCase()}]`, font: "Calibri", size: 11, bold: true, color: "#2563eb" }],
      shape: {
        shapeType: "rect",
        width: 300,
        height: 180,
        color: "#f8fafc",
        text: `Chart: ${chartType.toUpperCase()}`,
      },
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, chartPara);
    this.cursor.paragraphIndex++;
  }

  public insertTextBox(text: string = "Text Box Content"): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const boxPara: WordParagraph = {
      id: `txtbox-${Date.now()}`,
      style: "Normal",
      alignment: "left",
      runs: [{ text, font: "Calibri", size: 11, color: "#111827" }],
      shape: {
        shapeType: "rect",
        width: 180,
        height: 100,
        color: "#ffffff",
        text,
      },
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, boxPara);
    this.cursor.paragraphIndex++;
  }

  public insertWordArt(text: string = "WordArt Title"): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const artPara: WordParagraph = {
      id: `wordart-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      runs: [{ text, font: "Impact", size: 28, bold: true, color: "#2563eb" }],
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, artPara);
    this.cursor.paragraphIndex++;
  }

  public insertDropCap(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const para = page.paragraphs[this.cursor.paragraphIndex];
    if (para && para.runs.length > 0 && para.runs[0].text.length > 0) {
      const firstChar = para.runs[0].text.slice(0, 1);
      const rest = para.runs[0].text.slice(1);
      para.runs[0].text = rest;
      para.runs.unshift({
        text: firstChar,
        font: "Georgia",
        size: 32,
        bold: true,
        color: "#1e3a8a",
      });
    }
  }

  private copiedFormat: Partial<WordFormatState> | null = null;
  public copyFormat(): void {
    this.copiedFormat = { ...this.activeFormat };
  }
  public pasteFormat(): void {
    if (this.copiedFormat) {
      this.activeFormat = { ...this.activeFormat, ...this.copiedFormat };
      this.applyRunMutation((r) => {
        Object.assign(r, this.copiedFormat);
      });
    }
  }

  public updateShape(shapeId: string, updates: Partial<WordShapeData>): boolean {
    this.saveSnapshot();
    for (const page of this.doc.pages) {
      for (const p of page.paragraphs) {
        if (p.id === shapeId && p.shape) {
          p.shape = { ...p.shape, ...updates };
          return true;
        }
      }
    }
    return false;
  }

  public deleteShape(shapeId: string): boolean {
    this.saveSnapshot();
    for (const page of this.doc.pages) {
      const idx = page.paragraphs.findIndex((p) => p.id === shapeId);
      if (idx !== -1) {
        page.paragraphs.splice(idx, 1);
        return true;
      }
    }
    return false;
  }

  public insertHyperlink(text: string, url: string): void {
    this.saveSnapshot();
    this.insertText(text);
    this.applyRunMutation((r) => {
      r.color = "#2563eb";
      r.underline = true;
      r.linkUrl = url;
    });
  }

  public insertHorizontalLine(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const para: WordParagraph = {
      id: `line-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      shape: {
        shapeType: "rect",
        width: 624,
        height: 2,
        color: "#94a3b8",
      },
      runs: [{ text: "" }],
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, para);
    this.cursor.paragraphIndex++;
    this.cursor.runIndex = 0;
    this.cursor.offset = 0;
  }

  public insertDateTime(): void {
    const now = new Date();
    const str = now.toLocaleDateString() + " " + now.toLocaleTimeString();
    this.insertText(str);
  }

  public insertComment(text: string, author = "Reviewer"): WordComment {
    this.saveSnapshot();
    if (!this.doc.comments) this.doc.comments = [];
    const comment: WordComment = {
      id: `c-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
      text,
      author,
      createdAt: new Date().toLocaleTimeString(),
    };
    const page = this.doc.pages[this.cursor.pageIndex];
    const para = page?.paragraphs[this.cursor.paragraphIndex];
    if (para) {
      para.commentId = comment.id;
      comment.paragraphId = para.id;
    }
    this.doc.comments.push(comment);
    return comment;
  }

  public deleteComment(id?: string): void {
    if (!this.doc.comments) return;
    this.saveSnapshot();
    if (id) {
      this.doc.comments = this.doc.comments.filter((c) => c.id !== id);
    } else {
      const page = this.doc.pages[this.cursor.pageIndex];
      const para = page?.paragraphs[this.cursor.paragraphIndex];
      if (para?.commentId) {
        const commentId = para.commentId;
        this.doc.comments = this.doc.comments.filter((c) => c.id !== commentId);
        delete para.commentId;
      }
    }
  }

  public toggleTrackChanges(): boolean {
    this.doc.trackChanges = !this.doc.trackChanges;
    return !!this.doc.trackChanges;
  }

  public acceptChanges(): void {
    this.saveSnapshot();
    this.doc.pages.forEach((page) => {
      page.paragraphs.forEach((p) => {
        p.runs = p.runs.filter((r) => !r.deletion);
        p.runs.forEach((r) => {
          if (r.insertion) {
            r.insertion = false;
            r.color = "#242424";
            r.underline = false;
          }
        });
      });
    });
  }

  public rejectChanges(): void {
    this.saveSnapshot();
    this.doc.pages.forEach((page) => {
      page.paragraphs.forEach((p) => {
        p.runs = p.runs.filter((r) => !r.insertion);
        p.runs.forEach((r) => {
          if (r.deletion) {
            r.deletion = false;
            r.strikethrough = false;
            r.color = "#242424";
          }
        });
      });
    });
  }

  public footnotes: { id: string; num: number; text: string; pageIndex: number }[] = [];
  public citations: { id: string; author: string; title: string; year: number; style: string }[] = [];

  public setStyle(styleName: string): void {
    this.execCommand("SetStyle", styleName);
  }

  public insertParagraph(): void {
    this.splitParagraph();
  }

  public insertTOC(): void {
    this.saveSnapshot();
    const entries: { title: string; pageNum: number; level: number }[] = [];
    this.doc.pages.forEach((page, pIdx) => {
      page.paragraphs.forEach((p) => {
        if (p.style.startsWith("Heading")) {
          const text = p.runs.map((r) => r.text).join("").trim();
          if (text) {
            const level = p.style === "Heading 1" ? 1 : p.style === "Heading 2" ? 2 : 3;
            entries.push({ title: text, pageNum: pIdx + 1, level });
          }
        }
      });
    });

    const tocRuns: WordRun[] = [
      { text: "TABLE OF CONTENTS\n", font: "Calibri", size: 14, bold: true, color: "#1f497d" },
    ];
    if (entries.length === 0) {
      tocRuns.push({ text: "  (No headings found in document)\n", font: "Calibri", size: 10, italic: true, color: "#6b7280" });
    } else {
      entries.forEach((e) => {
        const indent = e.level === 1 ? "" : e.level === 2 ? "    " : "        ";
        tocRuns.push({
          text: `${indent}${e.title} ......................... ${e.pageNum}\n`,
          font: "Calibri",
          size: 11,
          color: "#2563eb",
          underline: true,
        });
      });
    }

    const tocPara: WordParagraph = {
      id: `toc-${Date.now()}`,
      style: "Normal",
      alignment: "left",
      runs: tocRuns,
      shading: "#f8fafc",
      border: "1px solid #cbd5e1",
    };

    const page0 = this.doc.pages[0];
    if (page0) {
      page0.paragraphs.unshift(tocPara);
    }
  }

  public updateTOC(): void {
    const page0 = this.doc.pages[0];
    if (page0) {
      const idx = page0.paragraphs.findIndex((p) => p.id.startsWith("toc-"));
      if (idx !== -1) {
        page0.paragraphs.splice(idx, 1);
      }
    }
    this.insertTOC();
  }

  public insertFootnote(text = "Footnote reference"): void {
    this.saveSnapshot();
    const num = this.footnotes.length + 1;
    this.insertText(`[${num}]`);
    this.applyRunMutation((r) => {
      r.superscript = true;
      r.size = 9;
      r.color = "#2563eb";
    });
    const entry = {
      id: `fn-${Date.now()}`,
      num,
      text,
      pageIndex: this.cursor.pageIndex,
    };
    this.footnotes.push(entry);
    if (!this.doc.footnotes) this.doc.footnotes = [];
    this.doc.footnotes.push(entry);
  }

  public insertEndnote(text = "Endnote reference"): void {
    this.saveSnapshot();
    const num = this.footnotes.length + 1;
    this.insertText(`[i${num}]`);
    this.applyRunMutation((r) => {
      r.superscript = true;
      r.size = 9;
      r.color = "#7c3aed";
    });
    const entry = {
      id: `en-${Date.now()}`,
      num,
      text: `Endnote: ${text}`,
      pageIndex: this.doc.pages.length - 1,
    };
    this.footnotes.push(entry);
    if (!this.doc.footnotes) this.doc.footnotes = [];
    this.doc.footnotes.push(entry);
  }

  public insertCitation(author = "Smith, J.", title = "AetherOffice Architecture", year = 2026): void {
    this.saveSnapshot();
    const entry = {
      id: `cite-${Date.now()}`,
      author,
      title,
      year,
      style: this.doc.citationStyle || "APA",
    };
    this.citations.push(entry);
    if (!this.doc.citations) this.doc.citations = [];
    this.doc.citations.push(entry);
    this.insertText(`(${author}, ${year})`);
  }

  public insertBibliography(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const bibRuns: WordRun[] = [
      { text: "\nBIBLIOGRAPHY\n", font: "Calibri", size: 14, bold: true, color: "#1f497d" },
    ];
    if (this.citations.length === 0) {
      bibRuns.push({ text: "Smith, J. (2026). AetherOffice Architecture and ECMA-376 Standards. Aether Press.\n", font: "Calibri", size: 11 });
    } else {
      this.citations.forEach((c) => {
        bibRuns.push({ text: `${c.author} (${c.year}). ${c.title}. [${c.style} Style].\n`, font: "Calibri", size: 11 });
      });
    }
    const bibPara: WordParagraph = {
      id: `bib-${Date.now()}`,
      style: "Normal",
      alignment: "left",
      runs: bibRuns,
    };
    page.paragraphs.push(bibPara);
  }

  public insertCaption(caption = "Figure 1: Diagram Overview"): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const capPara: WordParagraph = {
      id: `cap-${Date.now()}`,
      style: "Normal",
      alignment: "center",
      runs: [{ text: caption, font: "Calibri", size: 9, italic: true, color: "#6b7280" }],
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, capPara);
    this.cursor.paragraphIndex++;
  }

  public tableOfFigures(): void {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
    const figPara: WordParagraph = {
      id: `tof-${Date.now()}`,
      style: "Normal",
      alignment: "left",
      runs: [
        { text: "TABLE OF FIGURES\n", font: "Calibri", size: 12, bold: true, color: "#1f497d" },
        { text: "Figure 1: Architecture Overview ......................... 1\n", font: "Calibri", size: 10, color: "#2563eb" },
      ],
    };
    page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, figPara);
    this.cursor.paragraphIndex++;
  }

  public protectDocument(readOnly?: boolean): boolean {
    this.saveSnapshot();
    this.doc.readOnly = readOnly !== undefined ? readOnly : !this.doc.readOnly;
    return !!this.doc.readOnly;
  }

  public findMatches(query: string, matchCase = false): { pageIndex: number; paragraphIndex: number; runIndex: number; offset: number; length: number; text: string }[] {
    const matches: { pageIndex: number; paragraphIndex: number; runIndex: number; offset: number; length: number; text: string }[] = [];
    if (!query) return matches;

    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(matchCase ? query : escaped, matchCase ? "g" : "gi");

    this.doc.pages.forEach((page, pIdx) => {
      page.paragraphs.forEach((para, paraIdx) => {
        para.runs.forEach((run, rIdx) => {
          regex.lastIndex = 0;
          let m: RegExpExecArray | null;
          while ((m = regex.exec(run.text)) !== null) {
            matches.push({
              pageIndex: pIdx,
              paragraphIndex: paraIdx,
              runIndex: rIdx,
              offset: m.index,
              length: m[0].length,
              text: m[0],
            });
          }
        });
      });
    });
    return matches;
  }

  public replaceMatch(match: { pageIndex: number; paragraphIndex: number; runIndex: number; offset: number; length: number }, replacement: string): void {
    this.saveSnapshot();
    const page = this.doc.pages[match.pageIndex];
    const para = page?.paragraphs[match.paragraphIndex];
    const run = para?.runs[match.runIndex];
    if (run) {
      run.text = run.text.slice(0, match.offset) + replacement + run.text.slice(match.offset + match.length);
    }
  }

  public replaceAllMatches(query: string, replacement: string, matchCase = false): number {
    this.saveSnapshot();
    let count = 0;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(matchCase ? query : escaped, matchCase ? "g" : "gi");

    this.doc.pages.forEach((page) => {
      page.paragraphs.forEach((para) => {
        para.runs.forEach((run) => {
          const newText = run.text.replace(regex, () => {
            count++;
            return replacement;
          });
          run.text = newText;
        });
      });
    });
    return count;
  }

  private applyRunMutation(mutate: (r: WordRun) => void) {
    this.saveSnapshot();
    if (this.selection && !this.isSelectionCollapsed()) {
      const isStartFirst = comparePositions(this.selection.anchor, this.selection.focus) <= 0;
      const start = isStartFirst ? this.selection.anchor : this.selection.focus;
      const end = isStartFirst ? this.selection.focus : this.selection.anchor;

      if (
        start.pageIndex === end.pageIndex &&
        start.paragraphIndex === end.paragraphIndex &&
        start.runIndex === end.runIndex
      ) {
        const page = this.doc.pages[start.pageIndex];
        const para = page?.paragraphs[start.paragraphIndex];
        const run = para?.runs[start.runIndex];
        if (run && start.offset !== end.offset) {
          const beforeText = run.text.slice(0, start.offset);
          const selectedText = run.text.slice(start.offset, end.offset);
          const afterText = run.text.slice(end.offset);

          const mutatedRun: WordRun = { ...run, text: selectedText };
          mutate(mutatedRun);

          const replacementRuns: WordRun[] = [];
          if (beforeText) replacementRuns.push({ ...run, text: beforeText });
          replacementRuns.push(mutatedRun);
          if (afterText) replacementRuns.push({ ...run, text: afterText });

          para.runs.splice(start.runIndex, 1, ...replacementRuns);
          const newSelectedRunIdx = beforeText ? start.runIndex + 1 : start.runIndex;
          this.selection = {
            anchor: { ...start, runIndex: newSelectedRunIdx, offset: 0 },
            focus: { ...end, runIndex: newSelectedRunIdx, offset: selectedText.length },
          };
          this.cursor = { ...end, runIndex: newSelectedRunIdx, offset: selectedText.length };
          return;
        }
      }

      for (let pIdx = start.pageIndex; pIdx <= end.pageIndex; pIdx++) {
        const page = this.doc.pages[pIdx];
        if (!page) continue;
        const startPara = pIdx === start.pageIndex ? start.paragraphIndex : 0;
        const endPara = pIdx === end.pageIndex ? end.paragraphIndex : page.paragraphs.length - 1;
        for (let paraIdx = startPara; paraIdx <= endPara; paraIdx++) {
          const para = page.paragraphs[paraIdx];
          if (!para) continue;
          const firstRunIdx =
            pIdx === start.pageIndex && paraIdx === start.paragraphIndex
              ? start.offset >= (para.runs[start.runIndex]?.text.length || 0) && start.runIndex < para.runs.length - 1
                ? start.runIndex + 1
                : start.runIndex
              : 0;
          const lastRunIdx =
            pIdx === end.pageIndex && paraIdx === end.paragraphIndex
              ? end.runIndex
              : para.runs.length - 1;
          for (let rIdx = firstRunIdx; rIdx <= lastRunIdx; rIdx++) {
            const r = para.runs[rIdx];
            if (r) mutate(r);
          }
        }
      }
    } else {
      const page = this.doc.pages[this.cursor.pageIndex];
      if (page) {
        const para = page.paragraphs[this.cursor.paragraphIndex];
        if (para) {
          const run = para.runs[this.cursor.runIndex];
          if (run) mutate(run);
        }
      }
    }
  }

  private applyParagraphMutation(mutate: (p: WordParagraph) => void) {
    this.saveSnapshot();
    const page = this.doc.pages[this.cursor.pageIndex];
    if (page) {
      const para = page.paragraphs[this.cursor.paragraphIndex];
      if (para) mutate(para);
    }
  }

  execCommand(cmd: string, val?: any) {
    switch (cmd) {
      case "cut":
      case "Cut":
        if (this.selection && !this.isSelectionCollapsed()) {
          this.copy();
          this.deleteSelection();
        }
        break;

      case "copy":
      case "Copy":
        this.copy();
        break;

      case "paste":
      case "Paste":
        this.paste(typeof val === "string" ? val : undefined);
        break;

      case "formatPainter":
      case "FormatPainter":
        if (this.copiedFormat) {
          this.pasteFormat();
        } else {
          this.copyFormat();
        }
        break;

      case "changeCase":
      case "ChangeCase":
        this.changeCase(val || "uppercase");
        break;

      case "toggleMultilevelList":
      case "ToggleMultilevelList":
        this.applyParagraphMutation((p) => {
          if (p.runs.length > 0) {
            const firstRun = p.runs[0];
            if (firstRun.text.startsWith("1.1 ")) {
              firstRun.text = firstRun.text.slice(4);
            } else {
              firstRun.text = `1.1 ${firstRun.text}`;
            }
          }
        });
        break;

      case "sortParagraphs":
      case "SortParagraphs":
        this.sortParagraphs();
        break;

      case "toggleFormattingMarks":
      case "ToggleFormattingMarks":
        this.doc.showFormattingMarks = !this.doc.showFormattingMarks;
        break;

      case "setShading":
      case "SetShading":
        this.applyParagraphMutation((p) => {
          p.shading = val || "#f3f4f6";
        });
        break;

      case "setParagraphBorder":
      case "SetParagraphBorder":
        this.applyParagraphMutation((p) => {
          p.border = val || "1px solid #94a3b8";
        });
        break;

      case "insertCoverPage":
      case "InsertCoverPage":
        this.insertCoverPage();
        break;

      case "insertChart":
      case "InsertChart":
        this.insertChart(val || "column");
        break;

      case "insertHeader":
      case "InsertHeader": {
        this.editingZone = "header";
        const text = typeof val === "string" ? val : val && typeof val === "object" && "text" in val ? (val as any).text : "Document Header";
        this.setHeaderText(text);
        break;
      }

      case "insertFooter":
      case "InsertFooter": {
        this.editingZone = "footer";
        const text = typeof val === "string" ? val : val && typeof val === "object" && "text" in val ? (val as any).text : "Document Footer";
        this.setFooterText(text);
        break;
      }

      case "CloseHeaderFooter":
      case "ExitHeaderFooter":
        this.editingZone = "body";
        break;

      case "insertPageNumber":
      case "InsertPageNumber":
        this.doc.footerText = "Page 1 of 1";
        break;

      case "insertTextBox":
      case "InsertTextBox":
        this.insertTextBox(typeof val === "string" ? val : undefined);
        break;

      case "insertWordArt":
      case "InsertWordArt":
        this.insertWordArt(typeof val === "string" ? val : undefined);
        break;

      case "insertDropCap":
      case "InsertDropCap":
        this.insertDropCap();
        break;

      case "insertSymbol":
      case "InsertSymbol":
        this.insertSymbol(typeof val === "string" ? val : "©");
        break;

      case "insertBookmark":
      case "InsertBookmark":
        this.insertText(`[Bookmark: ${val || "BM-1"}]`);
        break;

      case "toggleLineNumbers":
      case "ToggleLineNumbers":
        this.doc.lineNumbers = !this.doc.lineNumbers;
        break;

      case "toggleHyphenation":
      case "ToggleHyphenation":
        this.doc.hyphenation = !this.doc.hyphenation;
        break;

      case "setPageBorder":
      case "SetPageBorder":
        this.doc.pageBorder = val || "1px solid #d1d5db";
        break;

      case "setWatermark":
      case "SetWatermark":
        this.doc.watermark = typeof val === "string" ? val : "CONFIDENTIAL";
        break;

      case "insertTOC":
      case "InsertTOC":
        this.insertTOC();
        break;

      case "updateTOC":
      case "UpdateTOC":
        this.updateTOC();
        break;

      case "insertFootnote":
      case "InsertFootnote":
        this.insertFootnote(typeof val === "string" ? val : undefined);
        break;

      case "insertEndnote":
      case "InsertEndnote":
        this.insertEndnote(typeof val === "string" ? val : undefined);
        break;

      case "nextFootnote":
      case "NextFootnote":
        if (this.footnotes.length > 0) {
          const fn = this.footnotes[0];
          this.cursor.pageIndex = fn.pageIndex;
        }
        break;

      case "insertCitation":
      case "InsertCitation":
        this.insertCitation();
        break;

      case "manageSources":
      case "ManageSources":
        this.insertCitation("Darwin, C.", "Origin of Species", 1859);
        break;

      case "setCitationStyle":
      case "SetCitationStyle":
        this.doc.citationStyle = val || "APA";
        break;

      case "insertBibliography":
      case "InsertBibliography":
        this.insertBibliography();
        break;

      case "insertCaption":
      case "InsertCaption":
        this.insertCaption(typeof val === "string" ? val : undefined);
        break;

      case "tableOfFigures":
      case "TableOfFigures":
        this.tableOfFigures();
        break;

      case "checkSpelling":
      case "CheckSpelling":
      case "thesaurus":
      case "Thesaurus":
      case "translate":
      case "Translate":
      case "setLanguage":
      case "SetLanguage":
        (this.doc as any).lastProofingAction = cmd;
        break;

      case "protectDocument":
      case "ProtectDocument":
        this.protectDocument(typeof val === "boolean" ? val : undefined);
        break;

      case "toggleGridlines":
      case "ToggleGridlines":
        this.doc.gridlines = !this.doc.gridlines;
        break;

      case "toggleDarkCanvas":
      case "ToggleDarkCanvas":
        this.doc.darkCanvas = !this.doc.darkCanvas;
        break;

      case "toggleFocusMode":
      case "ToggleFocusMode":
        (this.doc as any).focusMode = !(this.doc as any).focusMode;
        break;

      case "toggleBold":
      case "ToggleBold":
        this.activeFormat.bold = !this.activeFormat.bold;
        this.applyRunMutation((r) => {
          r.bold = !r.bold;
        });
        break;

      case "toggleItalic":
      case "ToggleItalic":
        this.activeFormat.italic = !this.activeFormat.italic;
        this.applyRunMutation((r) => {
          r.italic = !r.italic;
        });
        break;

      case "toggleUnderline":
      case "ToggleUnderline":
        this.activeFormat.underline = !this.activeFormat.underline;
        this.applyRunMutation((r) => {
          r.underline = !r.underline;
        });
        break;

      case "toggleStrikethrough":
      case "ToggleStrikethrough":
        this.activeFormat.strikethrough = !this.activeFormat.strikethrough;
        this.applyRunMutation((r) => {
          r.strikethrough = !r.strikethrough;
        });
        break;

      case "toggleSubscript":
      case "ToggleSubscript":
        this.applyRunMutation((r) => {
          r.subscript = !r.subscript;
          if (r.subscript) r.superscript = false;
        });
        break;

      case "toggleSuperscript":
      case "ToggleSuperscript":
        this.applyRunMutation((r) => {
          r.superscript = !r.superscript;
          if (r.superscript) r.subscript = false;
        });
        break;

      case "setFont":
      case "SetFontFamily":
        this.activeFormat.font = val;
        this.applyRunMutation((r) => {
          r.font = val;
        });
        break;

      case "setSize":
      case "SetFontSize":
        this.activeFormat.size = Number(val) || 11;
        this.applyRunMutation((r) => {
          r.size = Number(val) || 11;
        });
        break;

      case "increaseFontSize":
      case "IncreaseFontSize": {
        const nextSize = (this.activeFormat.size || 11) + 2;
        this.activeFormat.size = nextSize;
        this.applyRunMutation((r) => {
          r.size = (r.size || 11) + 2;
        });
        break;
      }

      case "decreaseFontSize":
      case "DecreaseFontSize": {
        const prevSize = Math.max(8, (this.activeFormat.size || 11) - 2);
        this.activeFormat.size = prevSize;
        this.applyRunMutation((r) => {
          r.size = Math.max(8, (r.size || 11) - 2);
        });
        break;
      }

      case "setFontColor":
      case "SetFontColor":
      case "setTextColor":
      case "SetTextColor":
        this.activeFormat.color = val || "#242424";
        this.applyRunMutation((r) => {
          r.color = val || "#242424";
        });
        break;

      case "setHighlightColor":
      case "SetHighlightColor":
        this.applyRunMutation((r) => {
          r.highlight = val || "#fef08a";
        });
        break;

      case "clearFormatting":
      case "ClearFormatting":
        this.activeFormat = {
          bold: false,
          italic: false,
          underline: false,
          strikethrough: false,
          font: "Calibri",
          size: 11,
          color: "#242424",
          alignment: "left",
          lineSpacing: 1.15,
        };
        this.applyRunMutation((r) => {
          r.bold = false;
          r.italic = false;
          r.underline = false;
          r.strikethrough = false;
          r.font = "Calibri";
          r.size = 11;
          r.color = "#242424";
        });
        break;

      case "setAlignment":
      case "AlignLeft":
      case "AlignCenter":
      case "AlignRight":
      case "AlignJustify": {
        const align = (cmd.startsWith("Align") ? cmd.replace("Align", "").toLowerCase() : val) as
          | "left"
          | "center"
          | "right"
          | "justify";
        this.activeFormat.alignment = align;
        this.applyParagraphMutation((p) => {
          p.alignment = align;
        });
        break;
      }

      case "increaseIndent":
      case "IncreaseIndent":
        this.applyParagraphMutation((p) => {
          p.indent = (p.indent || 0) + 36;
        });
        break;

      case "decreaseIndent":
      case "DecreaseIndent":
        this.applyParagraphMutation((p) => {
          p.indent = Math.max(0, (p.indent || 0) - 36);
        });
        break;

      case "toggleBulletList":
      case "ToggleBulletList":
        this.applyParagraphMutation((p) => {
          if (p.runs.length > 0) {
            const firstRun = p.runs[0];
            if (firstRun.text.startsWith("• ")) {
              firstRun.text = firstRun.text.slice(2);
            } else {
              firstRun.text = `• ${firstRun.text}`;
            }
          }
        });
        break;

      case "toggleNumberedList":
      case "ToggleNumberedList":
        this.applyParagraphMutation((p) => {
          if (p.runs.length > 0) {
            const firstRun = p.runs[0];
            if (firstRun.text.startsWith("1. ")) {
              firstRun.text = firstRun.text.slice(3);
            } else {
              firstRun.text = `1. ${firstRun.text}`;
            }
          }
        });
        break;

      case "setStyle":
      case "SetStyle": {
        const styleName = String(val);
        this.applyParagraphMutation((p) => {
          p.style = styleName;
          if (styleName === "Heading 1" || styleName === "Heading1") {
            p.runs.forEach((r) => {
              r.bold = true;
              r.size = 24;
              r.color = "#1f497d";
            });
          } else if (styleName === "Heading 2" || styleName === "Heading2") {
            p.runs.forEach((r) => {
              r.bold = true;
              r.size = 16;
              r.color = "#2e75b6";
            });
          } else if (styleName === "Heading 3" || styleName === "Heading3") {
            p.runs.forEach((r) => {
              r.bold = true;
              r.size = 13;
              r.color = "#595959";
            });
          } else if (styleName === "Title") {
            p.runs.forEach((r) => {
              r.bold = true;
              r.size = 28;
              r.color = "#1f497d";
            });
          } else if (styleName === "Subtitle") {
            p.runs.forEach((r) => {
              r.italic = true;
              r.size = 12;
              r.color = "#595959";
            });
          } else if (styleName === "Quote") {
            p.indent = 24;
            p.runs.forEach((r) => {
              r.italic = true;
              r.font = "Georgia";
              r.size = 12;
              r.color = "#475569";
            });
          } else {
            p.runs.forEach((r) => {
              r.bold = false;
              r.italic = false;
              r.size = 11;
              r.color = "#242424";
            });
          }
        });
        break;
      }

      case "insertTable":
      case "InsertTable": {
        const rows = val?.rows || 3;
        const cols = val?.cols || 3;
        this.insertTable(rows, cols);
        break;
      }

      case "addTableRow":
      case "AddTableRow":
        this.addTableRow();
        break;

      case "deleteTableRow":
      case "DeleteTableRow":
        this.deleteTableRow();
        break;

      case "addTableColumn":
      case "AddTableColumn":
        this.addTableColumn();
        break;

      case "deleteTableColumn":
      case "DeleteTableColumn":
        this.deleteTableColumn();
        break;

      case "insertPicture":
      case "InsertPicture": {
        const src = typeof val === "string" ? val : val?.src;
        const width = val?.width || 320;
        const height = val?.height || 180;
        const caption = val?.caption || "Embedded Image";
        this.insertPicture(src, width, height, caption);
        break;
      }

      case "insertShape":
      case "InsertShape":
        this.insertShape((val as any) || "rect");
        break;

      case "deleteShape":
      case "DeleteShape":
        if (typeof val === "string") this.deleteShape(val);
        break;

      case "updateShape":
      case "UpdateShape":
        if (val && typeof val === "object") {
          const { shapeId, updates } = val as { shapeId: string; updates: Partial<WordShapeData> };
          this.updateShape(shapeId, updates);
        }
        break;

      case "deletePage":
      case "DeletePage":
        this.deletePage(typeof val === "number" ? val : undefined, true);
        break;

      case "insertHorizontalLine":
      case "InsertHorizontalLine":
        this.insertHorizontalLine();
        break;

      case "insertDateTime":
      case "InsertDateTime":
        this.insertDateTime();
        break;

      case "setPageBackgroundColor":
      case "SetPageBackgroundColor":
        this.saveSnapshot();
        (this.doc as any).backgroundColor = val || "#ffffff";
        break;

      case "insertEquation":
      case "InsertEquation": {
        const eqText = typeof val === "string" ? val : "∑(x) = ∫ 2π r dr";
        this.insertText(eqText);
        break;
      }

      case "insertHyperlink":
      case "InsertHyperlink":
      case "insertLink":
      case "InsertLink": {
        const text = typeof val === "string" ? val : val?.text || "AetherOffice Link";
        const url = val?.url || "https://aetheroffice.org";
        this.insertHyperlink(text, url);
        break;
      }

      case "insertComment":
      case "InsertComment":
      case "newComment":
      case "NewComment":
        this.insertComment(typeof val === "string" ? val : val?.text || "Needs review", val?.author);
        break;

      case "deleteComment":
      case "DeleteComment":
        this.deleteComment(typeof val === "string" ? val : val?.id);
        break;

      case "nextComment":
      case "NextComment":
        if (this.doc.comments && this.doc.comments.length > 0) {
          const nextIdx = (((this as any).selectedCommentIndex ?? -1) + 1) % this.doc.comments.length;
          (this as any).selectedCommentIndex = nextIdx;
        }
        break;

      case "prevComment":
      case "PrevComment":
        if (this.doc.comments && this.doc.comments.length > 0) {
          const cur = (this as any).selectedCommentIndex ?? 0;
          (this as any).selectedCommentIndex = cur <= 0 ? this.doc.comments.length - 1 : cur - 1;
        }
        break;

      case "toggleTrackChanges":
      case "ToggleTrackChanges":
        this.toggleTrackChanges();
        break;

      case "acceptAll":
      case "AcceptAll":
      case "acceptChange":
      case "AcceptChange":
      case "acceptChanges":
      case "AcceptChanges":
        this.acceptChanges();
        break;

      case "rejectAll":
      case "RejectAll":
      case "rejectChange":
      case "RejectChange":
      case "rejectChanges":
      case "RejectChanges":
        this.rejectChanges();
        break;

      case "viewPrintLayout":
      case "ViewPrintLayout":
        this.doc.viewMode = "print";
        break;

      case "viewWebLayout":
      case "ViewWebLayout":
        this.doc.viewMode = "web";
        break;

      case "viewReadMode":
      case "ViewReadMode":
        this.doc.viewMode = "read";
        break;

      case "zoom100":
      case "Zoom100":
        this.setZoom(1.0);
        break;

      case "zoomOnePage":
      case "ZoomOnePage":
        this.setZoom(0.85);
        break;

      case "zoomMultiplePages":
      case "ZoomMultiplePages":
        this.setZoom(0.65);
        break;

      case "zoomPageWidth":
      case "ZoomPageWidth":
        this.setZoom(1.25);
        break;

      case "insertNewPage":
      case "InsertNewPage":
      case "insertBlankPage":
      case "InsertBlankPage":
      case "insertPageBreak":
      case "InsertPageBreak": {
        this.insertPageBreak();
        break;
      }

      case "deleteEmptyPage":
      case "DeleteEmptyPage": {
        this.deleteEmptyPage();
        break;
      }

      case "setMargins":
      case "SetMargins":
        this.saveSnapshot();
        this.doc.margins = val || "normal";
        break;

      case "setOrientation":
      case "SetOrientation":
        this.saveSnapshot();
        this.doc.pageOrientation = val === "landscape" ? "landscape" : "portrait";
        break;

      case "setPageSize":
      case "SetPageSize":
        this.saveSnapshot();
        this.doc.pageSize = val || "letter";
        break;

      case "setLineSpacing":
      case "SetLineSpacing": {
        const spacing = Number(val) || 1.15;
        this.activeFormat.lineSpacing = spacing;
        this.applyParagraphMutation((p) => {
          p.lineSpacing = spacing;
        });
        break;
      }

      case "setColumns":
      case "SetColumns":
        this.saveSnapshot();
        this.doc.columns = Number(val) || 1;
        break;

      case "undo":
        this.undo();
        break;

      case "redo":
        this.redo();
        break;

      case "setZoom":
      case "SetZoom":
        this.setZoom(Number(val) || 1.0);
        break;

      case "zoomIn":
        this.setZoom(this.zoom + 0.1);
        break;

      case "zoomOut":
        this.setZoom(this.zoom - 0.1);
        break;

      case "selectAll":
        this.selectAll();
        break;
    }
  }
}

export const writerState = new WriterState();
