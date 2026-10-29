import {
  CaretPosition,
  WriterState,
  writerState,
  comparePositions,
} from "../../state/writerState";

export interface LayoutLine {
  pageIndex: number;
  paragraphIndex: number;
  runIndex: number;
  text: string;
  x: number;
  y: number;
  height: number;
  fontSize: number;
  charAdvances: number[];
  startOffsetInRun: number;
  endOffsetInRun: number;
}

export interface LayoutPage {
  pageIndex: number;
  topY: number;
  lines: LayoutLine[];
}

export class WriterCanvas {
  private container: HTMLElement;
  private canvas!: HTMLCanvasElement;
  private canvases: HTMLCanvasElement[] = [];
  private state: WriterState;

  public readonly pageWidth = 816; // 8.5 inches at 96 dpi
  public readonly pageHeight = 1056; // 11 inches at 96 dpi
  public marginX = 96; // 1 inch
  public marginY = 96;
  public readonly pageGap = 32;

  private caretVisible = true;
  private caretTimer: number | null = null;
  private lastActivityTime = Date.now();
  private isMouseDown = false;
  private onStatusChange?: (pageText: string, words: number) => void;
  private onDocModifiedCallback?: (action: string, payload?: unknown) => void;

  // Cached layout lines for hit testing and subpixel caret placement
  private layoutPages: LayoutPage[] = [];

  constructor(container: HTMLElement, state: WriterState = writerState) {
    this.container = container;
    this.state = state;

    this.container.innerHTML = "";
    this.container.style.overflowY = "auto";
    this.container.style.overflowX = "auto";
    this.container.style.backgroundColor = "#eef2f5";
    this.container.style.display = "flex";
    this.container.style.flexDirection = "column";
    this.container.style.alignItems = "center";
    this.container.style.padding = "24px 0";

    this.startCaretBlink();
    this.render();
  }

  public setOnStatusChange(cb: (pageText: string, words: number) => void) {
    this.onStatusChange = cb;
    this.updateStatus();
  }

  public setOnDocModified(cb: (action: string, payload?: unknown) => void) {
    this.onDocModifiedCallback = cb;
  }

  public getCanvasElement(): HTMLCanvasElement {
    return this.canvas;
  }

  public getState(): WriterState {
    return this.state;
  }

  public setMargins(left: number, _right?: number) {
    this.marginX = Math.max(24, Math.min(left, 300));
    if (this.marginX === 48) {
      this.state.doc.margins = "narrow";
    } else if (this.marginX === 144) {
      this.state.doc.margins = "wide";
    } else if (this.marginX === 96) {
      this.state.doc.margins = "normal";
    }
    this.render();
  }

  public insertPageBreak() {
    this.state.insertPageBreak();
    this.emitDocChange("InsertPageBreak");
    this.render();
  }

  public destroy() {
    if (this.caretTimer) clearInterval(this.caretTimer);
  }

  private startCaretBlink() {
    if (this.caretTimer) clearInterval(this.caretTimer);
    if (typeof window !== "undefined") {
      this.caretTimer = window.setInterval(() => {
        const elapsed = Date.now() - this.lastActivityTime;
        if (elapsed < 500) {
          this.caretVisible = true;
        } else {
          const phase = Math.floor(elapsed / 530) % 2;
          this.caretVisible = phase === 0;
        }
        this.renderCaretOnly();
      }, 100);
    }
  }

  private resetBlink() {
    this.lastActivityTime = Date.now();
    this.caretVisible = true;
  }

  private emitDocChange(action: string, payload?: unknown) {
    if (this.onDocModifiedCallback) {
      this.onDocModifiedCallback(action, payload);
    }
  }

  private attachCanvasEvents(cv: HTMLCanvasElement, pageIdx: number) {
    cv.addEventListener("mousedown", (e: MouseEvent) => {
      cv.focus();
      this.isMouseDown = true;
      this.resetBlink();

      const pt = this.getCanvasLocalCoords(cv, e);
      const hit = this.hitTestPage(pt.x, pt.y, pageIdx);
      if (hit) {
        if (e.detail === 2) {
          this.state.selectWord(hit);
        } else if (e.detail >= 3) {
          this.state.selectParagraph(hit);
        } else {
          this.state.cursor = hit;
          if (e.shiftKey) {
            const anchor = this.state.selection ? this.state.selection.anchor : hit;
            this.state.selection = { anchor, focus: hit };
          } else {
            this.state.selection = { anchor: hit, focus: hit };
          }
        }
        this.render();
      }
    });

    cv.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isMouseDown) return;
      const pt = this.getCanvasLocalCoords(cv, e);
      const hit = this.hitTestPage(pt.x, pt.y, pageIdx);
      if (hit) {
        this.state.cursor = hit;
        const anchor = this.state.selection ? this.state.selection.anchor : hit;
        this.state.selection = { anchor, focus: hit };
        this.render();
      }
    });

    window.addEventListener("mouseup", () => {
      this.isMouseDown = false;
      if (this.state.selection && this.state.isSelectionCollapsed()) {
        this.state.clearSelection();
      }
    });

    cv.addEventListener("keydown", (e: KeyboardEvent) => {
      this.handleKeyDown(e);
    });

    cv.addEventListener("beforeinput", (e: Event) => {
      const inputEvent = e as InputEvent;
      if (inputEvent.data) {
        e.preventDefault();
        this.state.insertText(inputEvent.data);
        this.emitDocChange("InsertText", { text: inputEvent.data });
        this.render();
      }
    });

    cv.addEventListener("compositionend", (e: CompositionEvent) => {
      if (e.data) {
        this.state.insertText(e.data);
        this.emitDocChange("InsertText", { text: e.data });
        this.render();
      }
    });

    cv.addEventListener("wheel", (e: WheelEvent) => {
      if (e.ctrlKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.1 : -0.1;
        this.state.setZoom(this.state.zoom + delta);
        this.render();
      }
    });
  }

  private getCanvasLocalCoords(cv: HTMLCanvasElement, e: MouseEvent): { x: number; y: number } {
    const rect = cv.getBoundingClientRect();
    const x = (e.clientX - rect.left) / this.state.zoom;
    const y = (e.clientY - rect.top) / this.state.zoom;
    return { x, y };
  }

  public hitTest(clickX: number, clickY: number): CaretPosition | null {
    if (this.layoutPages.length === 0) return null;
    let targetPage = this.layoutPages[0];
    for (const p of this.layoutPages) {
      if (clickY >= p.topY && clickY <= p.topY + this.pageHeight + this.pageGap) {
        targetPage = p;
        break;
      }
      if (clickY > p.topY) targetPage = p;
    }
    return this.hitTestPage(clickX, clickY - targetPage.topY, targetPage.pageIndex);
  }

  private hitTestPage(clickX: number, clickY: number, pageIndex: number): CaretPosition | null {
    const pageLayout = this.layoutPages.find((p) => p.pageIndex === pageIndex);
    if (!pageLayout || pageLayout.lines.length === 0) {
      return { pageIndex, paragraphIndex: 0, runIndex: 0, offset: 0 };
    }

    let closestLine = pageLayout.lines[0];
    let minDist = Infinity;
    for (const l of pageLayout.lines) {
      const midY = l.y + l.height / 2;
      const dist = Math.abs(clickY - midY);
      if (dist < minDist) {
        minDist = dist;
        closestLine = l;
      }
    }

    const relX = clickX - closestLine.x;
    let offsetInLine = 0;
    let currX = 0;
    for (let i = 0; i < closestLine.charAdvances.length; i++) {
      const adv = closestLine.charAdvances[i];
      if (relX < currX + adv / 2) {
        offsetInLine = i;
        break;
      } else if (relX < currX + adv) {
        offsetInLine = i + 1;
        break;
      }
      currX += adv;
      offsetInLine = i + 1;
    }

    const clampedOffsetInLine = Math.min(offsetInLine, closestLine.text.length);
    const finalOffset = closestLine.startOffsetInRun + clampedOffsetInLine;

    return {
      pageIndex,
      paragraphIndex: closestLine.paragraphIndex,
      runIndex: closestLine.runIndex,
      offset: finalOffset,
    };
  }

  public handleKeyDown(e: KeyboardEvent) {
    this.resetBlink();

    // 1. Page Break via Ctrl+Enter
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      this.insertPageBreak();
      return;
    }

    // 2. Undo / Redo / Select All
    if (e.ctrlKey || e.metaKey) {
      if (e.key === "a" || e.key === "A") {
        e.preventDefault();
        this.state.selectAll();
        this.render();
        return;
      } else if (e.key === "z" || e.key === "Z") {
        e.preventDefault();
        if (e.shiftKey) {
          this.state.execCommand("redo");
          this.emitDocChange("Redo");
        } else {
          this.state.undo();
          this.emitDocChange("Undo");
        }
        this.render();
        return;
      } else if (e.key === "y" || e.key === "Y") {
        e.preventDefault();
        this.state.execCommand("redo");
        this.emitDocChange("Redo");
        this.render();
        return;
      }
    }

    // 3. Cursor Navigation
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      if (e.ctrlKey || e.altKey) {
        this.state.moveWordLeft();
      } else {
        this.moveCursorLeft();
      }
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "ArrowRight") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      if (e.ctrlKey || e.altKey) {
        this.state.moveWordRight();
      } else {
        this.moveCursorRight();
      }
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      this.moveCursorVertical(-1);
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      this.moveCursorVertical(1);
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "Home") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      if (e.ctrlKey) {
        this.state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
      } else {
        this.state.cursor.offset = 0;
      }
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "End") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      if (e.ctrlKey) {
        const lastPage = this.state.doc.pages.length - 1;
        const lastPara = this.state.doc.pages[lastPage].paragraphs.length - 1;
        const lastRun = this.state.doc.pages[lastPage].paragraphs[lastPara].runs.length - 1;
        const lastOffset = this.state.doc.pages[lastPage].paragraphs[lastPara].runs[lastRun].text.length;
        this.state.cursor = { pageIndex: lastPage, paragraphIndex: lastPara, runIndex: lastRun, offset: lastOffset };
      } else {
        const page = this.state.doc.pages[this.state.cursor.pageIndex];
        const para = page?.paragraphs[this.state.cursor.paragraphIndex];
        const run = para?.runs[this.state.cursor.runIndex];
        if (run) this.state.cursor.offset = run.text.length;
      }
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "PageUp") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      this.state.cursor.pageIndex = Math.max(0, this.state.cursor.pageIndex - 1);
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    if (e.key === "PageDown") {
      e.preventDefault();
      const anchor = this.state.selection ? this.state.selection.anchor : { ...this.state.cursor };
      this.state.cursor.pageIndex = Math.min(this.state.doc.pages.length - 1, this.state.cursor.pageIndex + 1);
      if (e.shiftKey) {
        this.state.selection = { anchor, focus: { ...this.state.cursor } };
      } else {
        this.state.clearSelection();
      }
      this.render();
      return;
    }

    // 4. Deletions & Typing
    if (e.key === "Backspace") {
      e.preventDefault();
      this.state.deleteBackward();
      this.emitDocChange("DeleteText");
      this.render();
      return;
    }

    if (e.key === "Delete") {
      e.preventDefault();
      this.state.deleteForward();
      this.emitDocChange("DeleteText");
      this.render();
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) {
        this.state.insertLineBreak();
      } else {
        this.state.splitParagraph();
      }
      this.emitDocChange("InsertText", { text: "\n" });
      this.render();
      return;
    }

    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      this.state.insertText(e.key);
      this.emitDocChange("InsertText", { text: e.key });
      this.render();
      return;
    }
  }

  private moveCursorLeft() {
    const page = this.state.doc.pages[this.state.cursor.pageIndex];
    if (!page) return;
    const para = page.paragraphs[this.state.cursor.paragraphIndex];
    if (!para) return;

    if (this.state.cursor.offset > 0) {
      this.state.cursor.offset--;
    } else if (this.state.cursor.runIndex > 0) {
      this.state.cursor.runIndex--;
      this.state.cursor.offset = para.runs[this.state.cursor.runIndex].text.length;
    } else if (this.state.cursor.paragraphIndex > 0) {
      this.state.cursor.paragraphIndex--;
      const prevP = page.paragraphs[this.state.cursor.paragraphIndex];
      this.state.cursor.runIndex = prevP.runs.length - 1;
      this.state.cursor.offset = prevP.runs[this.state.cursor.runIndex].text.length;
    } else if (this.state.cursor.pageIndex > 0) {
      this.state.cursor.pageIndex--;
      const prevPage = this.state.doc.pages[this.state.cursor.pageIndex];
      this.state.cursor.paragraphIndex = prevPage.paragraphs.length - 1;
      const prevP = prevPage.paragraphs[this.state.cursor.paragraphIndex];
      this.state.cursor.runIndex = prevP.runs.length - 1;
      this.state.cursor.offset = prevP.runs[this.state.cursor.runIndex].text.length;
    }
  }

  private moveCursorRight() {
    const page = this.state.doc.pages[this.state.cursor.pageIndex];
    if (!page) return;
    const para = page.paragraphs[this.state.cursor.paragraphIndex];
    if (!para) return;
    const run = para.runs[this.state.cursor.runIndex];
    if (!run) return;

    if (this.state.cursor.offset < run.text.length) {
      this.state.cursor.offset++;
    } else if (this.state.cursor.runIndex + 1 < para.runs.length) {
      this.state.cursor.runIndex++;
      this.state.cursor.offset = 0;
    } else if (this.state.cursor.paragraphIndex + 1 < page.paragraphs.length) {
      this.state.cursor.paragraphIndex++;
      this.state.cursor.runIndex = 0;
      this.state.cursor.offset = 0;
    } else if (this.state.cursor.pageIndex + 1 < this.state.doc.pages.length) {
      this.state.cursor.pageIndex++;
      this.state.cursor.paragraphIndex = 0;
      this.state.cursor.runIndex = 0;
      this.state.cursor.offset = 0;
    }
  }

  private moveCursorVertical(direction: -1 | 1) {
    const allLines: LayoutLine[] = [];
    for (const pl of this.layoutPages) {
      allLines.push(...pl.lines);
    }
    if (allLines.length === 0) return;

    const cur = this.state.cursor;
    const currentLineIdx = allLines.findIndex(
      (l) =>
        l.pageIndex === cur.pageIndex &&
        l.paragraphIndex === cur.paragraphIndex &&
        l.runIndex === cur.runIndex &&
        cur.offset >= l.startOffsetInRun &&
        cur.offset <= l.endOffsetInRun
    );

    if (currentLineIdx === -1) {
      if (direction === -1) this.moveCursorLeft();
      else this.moveCursorRight();
      return;
    }

    const currentLine = allLines[currentLineIdx];
    const colInLine = cur.offset - currentLine.startOffsetInRun;
    let caretX = currentLine.x;
    for (let i = 0; i < Math.min(colInLine, currentLine.charAdvances.length); i++) {
      caretX += currentLine.charAdvances[i];
    }

    const targetLineIdx = currentLineIdx + direction;
    if (targetLineIdx < 0 || targetLineIdx >= allLines.length) return;

    const targetLine = allLines[targetLineIdx];
    const relX = caretX - targetLine.x;
    let offsetInTarget = 0;
    let currX = 0;
    for (let i = 0; i < targetLine.charAdvances.length; i++) {
      const adv = targetLine.charAdvances[i];
      if (relX < currX + adv / 2) {
        offsetInTarget = i;
        break;
      } else if (relX < currX + adv) {
        offsetInTarget = i + 1;
        break;
      }
      currX += adv;
      offsetInTarget = i + 1;
    }

    this.state.cursor = {
      pageIndex: targetLine.pageIndex,
      paragraphIndex: targetLine.paragraphIndex,
      runIndex: targetLine.runIndex,
      offset: targetLine.startOffsetInRun + Math.min(offsetInTarget, targetLine.text.length),
    };
  }

  private activateHeaderEditing(headerEl: HTMLElement, pageIndex: number) {
    headerEl.contentEditable = "true";
    headerEl.classList.add("editing-header");
    headerEl.style.border = "1px dashed #2563eb";
    headerEl.style.backgroundColor = "rgba(37, 99, 235, 0.05)";
    headerEl.focus();

    const finish = () => {
      headerEl.contentEditable = "false";
      headerEl.classList.remove("editing-header");
      headerEl.style.border = "none";
      headerEl.style.backgroundColor = "transparent";
      const cleanText = headerEl.innerText.replace(/Page \d+ of \d+/g, "").trim();
      if (cleanText) {
        this.state.doc.headerText = cleanText;
      }
      this.emitDocChange("SetHeader", { text: this.state.doc.headerText, pageIndex });
      this.render();
    };

    headerEl.onblur = finish;
    headerEl.onkeydown = (ev) => {
      if (ev.key === "Enter" || ev.key === "Escape") {
        ev.preventDefault();
        headerEl.blur();
      }
    };
  }

  private activateFooterEditing(footerEl: HTMLElement, pageIndex: number) {
    footerEl.contentEditable = "true";
    footerEl.classList.add("editing-footer");
    footerEl.style.border = "1px dashed #2563eb";
    footerEl.style.backgroundColor = "rgba(37, 99, 235, 0.05)";
    footerEl.focus();

    const finish = () => {
      footerEl.contentEditable = "false";
      footerEl.classList.remove("editing-footer");
      footerEl.style.border = "none";
      footerEl.style.backgroundColor = "transparent";
      const cleanText = footerEl.innerText.replace(/Page \d+ of \d+/g, "").trim();
      if (cleanText) {
        this.state.doc.footerText = cleanText;
      }
      this.emitDocChange("SetFooter", { text: this.state.doc.footerText, pageIndex });
      this.render();
    };

    footerEl.onblur = finish;
    footerEl.onkeydown = (ev) => {
      if (ev.key === "Enter" || ev.key === "Escape") {
        ev.preventDefault();
        footerEl.blur();
      }
    };
  }

  public render() {
    // 1. Run auto-pagination if content height exceeds page boundary
    this.state.autoPaginate(this.pageHeight - this.marginY * 2);

    this.container.innerHTML = "";
    this.canvases = [];
    this.layoutPages = [];

    const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
    const totalPages = Math.max(1, this.state.doc.pages.length);

    const pagesContainer = document.createElement("div");
    pagesContainer.className = "word-pages-container";
    pagesContainer.style.display = "flex";
    pagesContainer.style.flexDirection = "column";
    pagesContainer.style.alignItems = "center";
    pagesContainer.style.gap = `${this.pageGap}px`;

    let caretRenderCoords: {
      canvasIndex: number;
      x: number;
      y: number;
      height: number;
    } | null = null;

    // Render Each Visual Paper Sheet (.word-page)
    this.state.doc.pages.forEach((page, pIdx) => {
      const pageEl = document.createElement("div");
      pageEl.className = "word-page";
      pageEl.setAttribute("data-page-index", String(pIdx));
      pageEl.setAttribute("data-page-number", String(pIdx + 1));
      pageEl.style.width = `${this.pageWidth * this.state.zoom}px`;
      pageEl.style.height = `${this.pageHeight * this.state.zoom}px`;
      pageEl.style.backgroundColor = "#ffffff";
      pageEl.style.boxShadow = "0 4px 16px rgba(0, 0, 0, 0.12)";
      pageEl.style.border = "1px solid #d1d5db";
      pageEl.style.position = "relative";
      pageEl.style.boxSizing = "border-box";
      pageEl.style.overflow = "hidden";

      // 1. Header Zone
      const headerEl = document.createElement("div");
      headerEl.className = "word-page-header";
      headerEl.setAttribute("data-page-index", String(pIdx));
      headerEl.style.position = "absolute";
      headerEl.style.top = "0";
      headerEl.style.left = `${this.marginX * this.state.zoom}px`;
      headerEl.style.right = `${this.marginX * this.state.zoom}px`;
      headerEl.style.height = `${this.marginY * this.state.zoom}px`;
      headerEl.style.display = "flex";
      headerEl.style.justifyContent = "space-between";
      headerEl.style.alignItems = "center";
      headerEl.style.fontSize = `${10 * this.state.zoom}px`;
      headerEl.style.color = "#9ca3af";
      headerEl.style.cursor = "pointer";
      headerEl.title = "Double-click to edit header";

      const headerTitle = document.createElement("span");
      headerTitle.className = "header-text";
      headerTitle.textContent = this.state.doc.headerText || "OpenOfficeRS Technical Document";

      const headerPageNum = document.createElement("span");
      headerPageNum.className = "page-number-display";
      headerPageNum.textContent = `Page ${pIdx + 1} of ${totalPages}`;

      headerEl.appendChild(headerTitle);
      headerEl.appendChild(headerPageNum);
      headerEl.ondblclick = (e) => {
        e.stopPropagation();
        this.activateHeaderEditing(headerEl, pIdx);
      };
      pageEl.appendChild(headerEl);

      // 2. Middle Content Canvas
      const cv = document.createElement("canvas");
      cv.className = "writer-canvas word-page-canvas";
      cv.tabIndex = 0;
      cv.setAttribute("data-page-index", String(pIdx));
      cv.style.position = "absolute";
      cv.style.top = `${this.marginY * this.state.zoom}px`;
      cv.style.left = "0";
      cv.style.width = `${this.pageWidth * this.state.zoom}px`;
      cv.style.height = `${(this.pageHeight - this.marginY * 2) * this.state.zoom}px`;
      cv.style.outline = "none";
      cv.style.cursor = "text";

      const contentHeight = this.pageHeight - this.marginY * 2;
      cv.width = this.pageWidth * dpr * this.state.zoom;
      cv.height = contentHeight * dpr * this.state.zoom;

      const ctx = cv.getContext("2d");
      if (ctx) {
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.scale(dpr * this.state.zoom, dpr * this.state.zoom);
      }

      this.attachCanvasEvents(cv, pIdx);
      pageEl.appendChild(cv);
      this.canvases.push(cv);
      if (pIdx === 0 || pIdx === this.state.cursor.pageIndex) {
        this.canvas = cv;
      }

      // 3. Footer Zone
      const footerEl = document.createElement("div");
      footerEl.className = "word-page-footer";
      footerEl.setAttribute("data-page-index", String(pIdx));
      footerEl.style.position = "absolute";
      footerEl.style.bottom = "0";
      footerEl.style.left = `${this.marginX * this.state.zoom}px`;
      footerEl.style.right = `${this.marginX * this.state.zoom}px`;
      footerEl.style.height = `${this.marginY * this.state.zoom}px`;
      footerEl.style.display = "flex";
      footerEl.style.justifyContent = "space-between";
      footerEl.style.alignItems = "center";
      footerEl.style.fontSize = `${10 * this.state.zoom}px`;
      footerEl.style.color = "#9ca3af";
      footerEl.style.cursor = "pointer";
      footerEl.title = "Double-click to edit footer";

      const footerTitle = document.createElement("span");
      footerTitle.className = "footer-text";
      footerTitle.textContent = this.state.doc.footerText || "AetherOffice Suite";

      const footerPageNum = document.createElement("span");
      footerPageNum.className = "page-number-display";
      footerPageNum.textContent = `Page ${pIdx + 1} of ${totalPages}`;

      footerEl.appendChild(footerTitle);
      footerEl.appendChild(footerPageNum);
      footerEl.ondblclick = (e) => {
        e.stopPropagation();
        this.activateFooterEditing(footerEl, pIdx);
      };
      pageEl.appendChild(footerEl);

      pagesContainer.appendChild(pageEl);

      // 4. Layout Paragraphs on this page's Canvas
      if (ctx) {
        // Draw Margin Guide (dashed)
        ctx.strokeStyle = "#f3f4f6";
        ctx.setLineDash([4, 4]);
        ctx.strokeRect(this.marginX, 0, this.pageWidth - 2 * this.marginX, contentHeight);
        ctx.setLineDash([]);

        const pageLayout: LayoutPage = { pageIndex: pIdx, topY: pIdx * (this.pageHeight + this.pageGap), lines: [] };
        let lineY = 16;

        page.paragraphs.forEach((p, pIndex) => {
          const lineX = this.marginX + (p.indent || 0);

          p.runs.forEach((r, rIndex) => {
            const fontSize = r.size || (p.style === "Heading1" ? 22 : p.style === "Heading2" ? 15 : 11);
            const fontFamily = r.font || "Calibri";
            const isBold = r.bold !== undefined ? r.bold : p.style.startsWith("Heading");
            const isItalic = r.italic || false;

            let fontStyle = "";
            if (isItalic) fontStyle += "italic ";
            if (isBold) fontStyle += "bold ";
            ctx.font = `${fontStyle}${fontSize}px ${fontFamily}, sans-serif`;
            ctx.fillStyle =
              r.color ||
              (p.style === "Heading1" ? "#1f497d" : p.style === "Heading2" ? "#2e75b6" : "#242424");
            ctx.textAlign = "left";

            const words = r.text.split(" ");
            const maxLineWidth = this.pageWidth - 2 * this.marginX - (p.indent || 0);
            let currentLineText = "";
            let lineStartOffset = 0;

            for (let w = 0; w < words.length; w++) {
              const word = words[w];
              const testLine = currentLineText ? `${currentLineText} ${word}` : word;
              const testWidth = ctx.measureText(testLine).width;

              if (testWidth > maxLineWidth && currentLineText) {
                let actualLineX = lineX;
                const curWidth = ctx.measureText(currentLineText).width;
                if (p.alignment === "center") {
                  actualLineX = this.marginX + Math.max(0, (maxLineWidth - curWidth) / 2);
                } else if (p.alignment === "right") {
                  actualLineX = this.pageWidth - this.marginX - curWidth;
                }

                this.renderTextLine(
                  ctx,
                  currentLineText,
                  actualLineX,
                  lineY,
                  fontSize,
                  r.underline || false,
                  r.strikethrough || false,
                  r.highlight
                );
                const charAdv = this.computeCharAdvances(ctx, currentLineText);
                const lineEndOffset = lineStartOffset + currentLineText.length;
                pageLayout.lines.push({
                  pageIndex: pIdx,
                  paragraphIndex: pIndex,
                  runIndex: rIndex,
                  text: currentLineText,
                  x: actualLineX,
                  y: lineY - fontSize + 2,
                  height: fontSize * 1.25,
                  fontSize,
                  charAdvances: charAdv,
                  startOffsetInRun: lineStartOffset,
                  endOffsetInRun: lineEndOffset,
                });

                if (
                  pIdx === this.state.cursor.pageIndex &&
                  pIndex === this.state.cursor.paragraphIndex &&
                  rIndex === this.state.cursor.runIndex &&
                  this.state.cursor.offset >= lineStartOffset &&
                  this.state.cursor.offset <= lineEndOffset
                ) {
                  const textBefore = currentLineText.slice(0, this.state.cursor.offset - lineStartOffset);
                  const subpixelWidth = ctx.measureText(textBefore).width;
                  caretRenderCoords = {
                    canvasIndex: pIdx,
                    x: actualLineX + subpixelWidth,
                    y: lineY - fontSize + 2,
                    height: fontSize * 1.25,
                  };
                }

                const spacingMultiplier = p.lineSpacing || this.state.activeFormat.lineSpacing || 1.15;
                lineY += fontSize * 1.35 * (spacingMultiplier / 1.15);
                lineStartOffset = lineEndOffset + 1;
                currentLineText = word;
              } else {
                currentLineText = testLine;
              }
            }

            if (currentLineText !== "") {
              let actualLineX = lineX;
              const curWidth = ctx.measureText(currentLineText).width;
              if (p.alignment === "center") {
                actualLineX = this.marginX + Math.max(0, (maxLineWidth - curWidth) / 2);
              } else if (p.alignment === "right") {
                actualLineX = this.pageWidth - this.marginX - curWidth;
              }

              this.renderTextLine(
                ctx,
                currentLineText,
                actualLineX,
                lineY,
                fontSize,
                r.underline || false,
                r.strikethrough || false,
                r.highlight
              );
              const charAdv = this.computeCharAdvances(ctx, currentLineText);
              const lineEndOffset = lineStartOffset + currentLineText.length;
              pageLayout.lines.push({
                pageIndex: pIdx,
                paragraphIndex: pIndex,
                runIndex: rIndex,
                text: currentLineText,
                x: actualLineX,
                y: lineY - fontSize + 2,
                height: fontSize * 1.25,
                fontSize,
                charAdvances: charAdv,
                startOffsetInRun: lineStartOffset,
                endOffsetInRun: lineEndOffset,
              });

              if (
                pIdx === this.state.cursor.pageIndex &&
                pIndex === this.state.cursor.paragraphIndex &&
                rIndex === this.state.cursor.runIndex &&
                this.state.cursor.offset >= lineStartOffset &&
                this.state.cursor.offset <= lineEndOffset
              ) {
                const textBefore = currentLineText.slice(0, this.state.cursor.offset - lineStartOffset);
                const subpixelWidth = ctx.measureText(textBefore).width;
                caretRenderCoords = {
                  canvasIndex: pIdx,
                  x: actualLineX + subpixelWidth,
                  y: lineY - fontSize + 2,
                  height: fontSize * 1.25,
                };
              }

              const spacingMultiplier = p.lineSpacing || this.state.activeFormat.lineSpacing || 1.15;
              lineY += fontSize * 1.35 * (spacingMultiplier / 1.15);
            } else if (r.text === "") {
              const charAdv = [0];
              pageLayout.lines.push({
                pageIndex: pIdx,
                paragraphIndex: pIndex,
                runIndex: rIndex,
                text: "",
                x: lineX,
                y: lineY - fontSize + 2,
                height: fontSize * 1.25,
                fontSize,
                charAdvances: charAdv,
                startOffsetInRun: 0,
                endOffsetInRun: 0,
              });

              if (
                pIdx === this.state.cursor.pageIndex &&
                pIndex === this.state.cursor.paragraphIndex &&
                rIndex === this.state.cursor.runIndex
              ) {
                caretRenderCoords = {
                  canvasIndex: pIdx,
                  x: lineX,
                  y: lineY - fontSize + 2,
                  height: fontSize * 1.25,
                };
              }

              lineY += fontSize * 1.35;
            }
          });

          lineY += p.style.startsWith("Heading") ? 14 : 8;
        });

        this.layoutPages.push(pageLayout);

        // 5. Text Selection Highlighting on this page's Canvas
        if (this.state.selection && !this.state.isSelectionCollapsed()) {
          ctx.fillStyle = "rgba(51, 153, 255, 0.35)";
          const isForward = comparePositions(this.state.selection.anchor, this.state.selection.focus) <= 0;
          const start = isForward ? this.state.selection.anchor : this.state.selection.focus;
          const end = isForward ? this.state.selection.focus : this.state.selection.anchor;

          for (const l of pageLayout.lines) {
            const lineStartPos: CaretPosition = {
              pageIndex: l.pageIndex,
              paragraphIndex: l.paragraphIndex,
              runIndex: l.runIndex,
              offset: l.startOffsetInRun,
            };
            const lineEndPos: CaretPosition = {
              pageIndex: l.pageIndex,
              paragraphIndex: l.paragraphIndex,
              runIndex: l.runIndex,
              offset: l.endOffsetInRun,
            };

            if (comparePositions(lineEndPos, start) <= 0 || comparePositions(lineStartPos, end) >= 0) {
              continue;
            }

            const startCol =
              comparePositions(lineStartPos, start) < 0
                ? Math.max(0, start.offset - l.startOffsetInRun)
                : 0;
            const endCol =
              comparePositions(lineEndPos, end) > 0
                ? Math.min(l.text.length, end.offset - l.startOffsetInRun)
                : l.text.length;

            if (startCol < endCol) {
              let leftX = l.x;
              for (let i = 0; i < startCol; i++) {
                leftX += l.charAdvances[i] || 0;
              }
              let width = 0;
              for (let i = startCol; i < endCol; i++) {
                width += l.charAdvances[i] || 0;
              }
              ctx.fillRect(leftX, l.y, Math.max(2, width), l.height);
            }
          }
        }

        // 6. Caret Rendering
        if (this.caretVisible && caretRenderCoords && caretRenderCoords.canvasIndex === pIdx) {
          ctx.fillStyle = "#000000";
          ctx.fillRect(caretRenderCoords.x, caretRenderCoords.y, 2, caretRenderCoords.height);
        }
      }
    });

    // 5. "+ New Page" Action Button at Bottom
    const addPageBtn = document.createElement("button");
    addPageBtn.className = "btn-add-page";
    addPageBtn.textContent = "➕ Blank Page";
    addPageBtn.style.padding = "8px 18px";
    addPageBtn.style.margin = "8px 0 24px 0";
    addPageBtn.style.backgroundColor = "#ffffff";
    addPageBtn.style.border = "1px solid #d1d5db";
    addPageBtn.style.borderRadius = "4px";
    addPageBtn.style.cursor = "pointer";
    addPageBtn.style.fontWeight = "600";
    addPageBtn.style.fontSize = "12px";
    addPageBtn.style.color = "#374151";
    addPageBtn.style.boxShadow = "0 1px 3px rgba(0,0,0,0.08)";
    addPageBtn.onclick = () => this.insertPageBreak();
    pagesContainer.appendChild(addPageBtn);

    this.container.appendChild(pagesContainer);
    this.updateStatus();
  }

  private renderCaretOnly() {
    // When blinking, if no selection is active, simply toggle caret
    if (!this.state.selection || this.state.isSelectionCollapsed()) {
      // Re-rendering active canvas keeps it fast
      const activeIdx = this.state.cursor.pageIndex;
      if (this.canvases[activeIdx]) {
        // Redraw page
        this.render();
      }
    }
  }

  private renderTextLine(
    ctx: CanvasRenderingContext2D,
    text: string,
    x: number,
    y: number,
    fontSize: number,
    underline: boolean,
    strikethrough: boolean = false,
    highlight?: string
  ) {
    const width = ctx.measureText(text).width;
    if (highlight) {
      const prevFill = ctx.fillStyle;
      ctx.fillStyle = highlight;
      ctx.fillRect(x, y - fontSize + 2, width, fontSize * 1.25);
      ctx.fillStyle = prevFill;
    }
    ctx.fillText(text, x, y);
    if (underline) {
      ctx.beginPath();
      ctx.moveTo(x, y + 2);
      ctx.lineTo(x + width, y + 2);
      ctx.strokeStyle = ctx.fillStyle;
      ctx.lineWidth = 1;
      ctx.stroke();
    }
    if (strikethrough) {
      ctx.beginPath();
      ctx.moveTo(x, y - fontSize / 3);
      ctx.lineTo(x + width, y - fontSize / 3);
      ctx.strokeStyle = ctx.fillStyle;
      ctx.lineWidth = 1;
      ctx.stroke();
    }
  }

  private computeCharAdvances(ctx: CanvasRenderingContext2D, text: string): number[] {
    const advances: number[] = [];
    for (const char of text) {
      advances.push(ctx.measureText(char).width);
    }
    return advances;
  }

  private updateStatus() {
    if (this.onStatusChange) {
      const pageNum = this.state.cursor.pageIndex + 1;
      const totalPages = Math.max(1, this.state.doc.pages.length);
      const words = this.state.getTotalWords();
      this.onStatusChange(`Page ${pageNum} of ${totalPages}`, words);
    }
  }
}
