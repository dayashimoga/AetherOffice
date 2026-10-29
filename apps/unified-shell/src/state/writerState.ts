export interface WordRun {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  font?: string;
  size?: number;
  color?: string;
}

export interface WordParagraph {
  id: string;
  style: string;
  runs: WordRun[];
  alignment: "left" | "center" | "right" | "justify";
}

export interface DocPage {
  pageNumber: number;
  paragraphs: WordParagraph[];
}

export interface WordDocumentModel {
  id: string;
  title: string;
  pages: DocPage[];
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

export class WriterState {
  doc: WordDocumentModel = {
    id: "doc-1",
    title: "Document 1.docx",
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
  activeFormat = { bold: false, italic: false, underline: false, font: "Calibri", size: 11, alignment: "left" as const };
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
    return a.pageIndex === f.pageIndex && a.paragraphIndex === f.paragraphIndex && a.runIndex === f.runIndex && a.offset === f.offset;
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

  execCommand(cmd: string, val?: any) {
    if (cmd === "toggleBold") this.activeFormat.bold = !this.activeFormat.bold;
    if (cmd === "toggleItalic") this.activeFormat.italic = !this.activeFormat.italic;
    if (cmd === "toggleUnderline") this.activeFormat.underline = !this.activeFormat.underline;
    if (cmd === "setFont") this.activeFormat.font = val;
    if (cmd === "setSize") this.activeFormat.size = Number(val) || 11;
    if (cmd === "setAlignment") this.activeFormat.alignment = val;
    if (cmd === "undo") this.undo();
    if (cmd === "zoomIn") this.setZoom(this.zoom + 0.1);
    if (cmd === "zoomOut") this.setZoom(this.zoom - 0.1);
    if (cmd === "selectAll") this.selectAll();
  }
}

export const writerState = new WriterState();
