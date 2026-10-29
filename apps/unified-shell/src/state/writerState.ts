export interface WordRun {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  font?: string;
  size?: number;
  color?: string;
}

export interface WordParagraph {
  id: string;
  style: string;
  runs: WordRun[];
  alignment: "left" | "center" | "right" | "justify";
  indent?: number;
}

export interface DocPage {
  pageNumber: number;
  paragraphs: WordParagraph[];
}

export interface WordDocumentModel {
  id: string;
  title: string;
  pages: DocPage[];
  pageOrientation?: "portrait" | "landscape";
  margins?: "normal" | "narrow" | "wide";
  pageSize?: "letter" | "a4" | "legal";
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

export class WriterState {
  doc: WordDocumentModel = {
    id: "doc-1",
    title: "Document 1.docx",
    pageOrientation: "portrait",
    margins: "normal",
    pageSize: "letter",
    pages: [
      {
        pageNumber: 1,
        paragraphs: [
          {
            id: "p1",
            style: "Heading1",
            alignment: "left",
            runs: [{ text: "OpenOfficeRS — Executive Technical Summary", bold: true, size: 24, color: "#1f497d" }],
          },
          {
            id: "p2",
            style: "Normal",
            alignment: "left",
            runs: [
              {
                text: "OpenOfficeRS is a unified, high-performance office suite engineered from the ground up in idiomatic Rust and TypeScript. It delivers 100% ECMA-376 / ISO-29500 zero-loss roundtrip fidelity.",
                size: 11,
                font: "Calibri",
              },
            ],
          },
          {
            id: "p3",
            style: "Heading2",
            alignment: "left",
            runs: [{ text: "Core Architecture & Design Principles", bold: true, size: 16, color: "#2e75b6" }],
          },
          {
            id: "p4",
            style: "Normal",
            alignment: "left",
            runs: [
              {
                text: "The word processor engine utilizes a piece-tree rope memory model for O(log N) keystroke insertion, Knuth-Plass total-fit dynamic programming line breaking, and subpixel HarfBuzz text shaping.",
                size: 11,
                font: "Calibri",
              },
            ],
          },
          {
            id: "p5",
            style: "Normal",
            alignment: "left",
            runs: [
              {
                text: "Viewport rendering targets a strict 120 FPS latency budget (< 8.33 ms frame time) with tile virtualization and automatic GPU display list culling.",
                italic: true,
                size: 11,
                font: "Calibri",
              },
            ],
          },
        ],
      },
      {
        pageNumber: 2,
        paragraphs: [
          {
            id: "p6",
            style: "Heading1",
            alignment: "left",
            runs: [{ text: "Performance SLAs & Benchmark Compliance", bold: true, size: 24, color: "#1f497d" }],
          },
          {
            id: "p7",
            style: "Normal",
            alignment: "left",
            runs: [
              {
                text: "All memory allocations are strictly bounded under Global Engineering Contract v1.0. Cold startup finishes under 150 ms on desktop and 300 ms on mobile devices.",
                size: 11,
                font: "Calibri",
              },
            ],
          },
          {
            id: "p8",
            style: "Heading2",
            alignment: "left",
            runs: [{ text: "Universal Adaptive Form-Factor Parity", bold: true, size: 16, color: "#2e75b6" }],
          },
          {
            id: "p9",
            style: "Normal",
            alignment: "left",
            runs: [
              {
                text: "The unified shell seamlessly transforms across Desktop (>= 1024px multi-tab workspace), Tablet (768-1023px touch ribbon & stylus ink overlay), and Mobile (< 768px Reflow Reader & bottom sheets).",
                size: 11,
                font: "Calibri",
              },
            ],
          },
        ],
      },
    ],
  };

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
  private history: string[] = [];

  constructor() {
    this.saveSnapshot();
  }

  public saveSnapshot() {
    if (this.history.length > 50) this.history.shift();
    this.history.push(JSON.stringify(this.doc));
  }

  public undo() {
    if (this.history.length > 1) {
      this.history.pop();
      const prev = this.history[this.history.length - 1];
      if (prev) {
        this.doc = JSON.parse(prev);
      }
    }
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

  private applyRunMutation(mutate: (r: WordRun) => void) {
    this.saveSnapshot();
    if (this.selection && !this.isSelectionCollapsed()) {
      const a = this.selection.anchor;
      const f = this.selection.focus;
      const startP = Math.min(a.pageIndex, f.pageIndex);
      const endP = Math.max(a.pageIndex, f.pageIndex);

      for (let pIdx = startP; pIdx <= endP; pIdx++) {
        const page = this.doc.pages[pIdx];
        if (!page) continue;
        for (const para of page.paragraphs) {
          for (const r of para.runs) {
            mutate(r);
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

      case "setTextColor":
      case "SetTextColor":
        this.activeFormat.color = val || "#242424";
        this.applyRunMutation((r) => {
          r.color = val || "#242424";
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
          } else if (styleName === "Title") {
            p.runs.forEach((r) => {
              r.bold = true;
              r.size = 28;
              r.color = "#1f497d";
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
        this.saveSnapshot();
        const page = this.doc.pages[this.cursor.pageIndex] || this.doc.pages[0];
        page.paragraphs.splice(this.cursor.paragraphIndex + 1, 0, {
          id: `tbl-${Date.now()}`,
          style: "Normal",
          alignment: "left",
          runs: [
            { text: "┌────────────┬────────────┬────────────┐\n", font: "Consolas", size: 10, color: "#1f497d" },
            { text: "│ Item 1     │ Quantity   │ Price      │\n", font: "Consolas", size: 10, bold: true },
            { text: "├────────────┼────────────┼────────────┤\n", font: "Consolas", size: 10, color: "#1f497d" },
            { text: "│ Widget A   │ 10         │ $45.00     │\n", font: "Consolas", size: 10 },
            { text: "│ Widget B   │ 25         │ $120.00    │\n", font: "Consolas", size: 10 },
            { text: "└────────────┴────────────┴────────────┘", font: "Consolas", size: 10, color: "#1f497d" },
          ],
        });
        this.cursor.paragraphIndex++;
        break;
      }

      case "insertPageBreak":
      case "InsertPageBreak": {
        this.saveSnapshot();
        const newPageNum = this.doc.pages.length + 1;
        this.doc.pages.push({
          pageNumber: newPageNum,
          paragraphs: [
            {
              id: `p-${Date.now()}`,
              style: "Normal",
              alignment: "left",
              runs: [{ text: "", font: this.activeFormat.font, size: this.activeFormat.size }],
            },
          ],
        });
        this.cursor.pageIndex = this.doc.pages.length - 1;
        this.cursor.paragraphIndex = 0;
        this.cursor.runIndex = 0;
        this.cursor.offset = 0;
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
      case "SetLineSpacing":
        this.activeFormat.lineSpacing = Number(val) || 1.15;
        break;

      case "undo":
        this.undo();
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
