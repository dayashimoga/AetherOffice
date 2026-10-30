import {
  CaretPosition,
  WriterState,
  writerState,
  comparePositions,
} from "../../state/writerState";
import { collabState } from "../../state/collabState";

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

  public getPageDimensions(): { width: number; height: number } {
    let baseW = 816;
    let baseH = 1056;
    if (this.state.doc.pageSize === "a4") {
      baseW = 794;
      baseH = 1123;
    } else if (this.state.doc.pageSize === "legal") {
      baseW = 816;
      baseH = 1296;
    }
    if (this.state.doc.pageOrientation === "landscape") {
      return { width: baseH, height: baseW };
    }
    return { width: baseW, height: baseH };
  }

  public get pageWidth(): number {
    return this.getPageDimensions().width;
  }

  public get pageHeight(): number {
    return this.getPageDimensions().height;
  }

  private _customMarginX?: number;
  private _customMarginY?: number;

  public get marginX(): number {
    const m = this.state.doc.margins;
    if (m === "narrow") return 48;
    if (m === "moderate") return 72;
    if (m === "wide") return 144;
    return this._customMarginX ?? 96;
  }
  public set marginX(val: number) {
    this._customMarginX = val;
  }

  public get marginY(): number {
    const m = this.state.doc.margins;
    if (m === "narrow") return 48;
    if (m === "moderate") return 72;
    if (m === "wide") return 96;
    return this._customMarginY ?? 96;
  }
  public set marginY(val: number) {
    this._customMarginY = val;
  }

  public readonly pageGap = 32;

  private caretVisible = true;
  private caretTimer: number | null = null;
  private lastActivityTime = Date.now();
  private isMouseDown = false;
  private onStatusChange?: (pageText: string, words: number) => void;
  private onDocModifiedCallback?: (action: string, payload?: unknown) => void;
  private ghostText: string | null = null;

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
    collabState.subscribe(() => this.render());
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

  public insertNewPage() {
    this.insertPageBreak();
  }

  public deleteEmptyPage(pageIndex?: number): boolean {
    const res = this.state.deleteEmptyPage(pageIndex);
    if (res) {
      this.emitDocChange("DeleteEmptyPage");
      this.render();
    }
    return res;
  }

  public deletePage(pageIndex?: number, force = true): boolean {
    const res = this.state.deletePage(pageIndex, force);
    if (res) {
      this.emitDocChange("DeletePage");
      this.render();
    }
    return res;
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

  public typeText(text: string) {
    for (const char of text) {
      this.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
    }
    const pIdx = this.state.cursor.pageIndex;
    const bodyEl = this.container.querySelector(`.word-page-body[data-page-index="${pIdx}"]`) as HTMLElement | null;
    if (bodyEl) {
      const page = this.state.doc.pages[pIdx];
      if (page) {
        bodyEl.innerText = page.paragraphs
          .filter((p) => !p.shape && !p.image && !p.table)
          .map((p) => p.runs.map((r) => r.text).join(""))
          .join("\n");
      }
    }
  }

  public handleKeyDown(e: KeyboardEvent) {
    this.resetBlink();

    // Inline Ghost Text Handling (Subphase 20.1.3: Tab to accept, Esc to dismiss)
    if (this.ghostText) {
      if (e.key === "Tab") {
        e.preventDefault();
        this.acceptGhostText();
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        this.dismissGhostText();
        return;
      }
      if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        this.dismissGhostText();
      }
    }

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
      pageEl.style.backgroundColor = (this.state.doc as any).backgroundColor || "#ffffff";
      pageEl.style.boxShadow = "0 4px 16px rgba(0, 0, 0, 0.12)";
      pageEl.style.border = this.state.doc.pageBorder || "1px solid #d1d5db";
      pageEl.style.position = "relative";
      pageEl.style.boxSizing = "border-box";
      pageEl.style.overflow = "hidden";

      if (this.state.doc.watermark) {
        const wm = document.createElement("div");
        wm.className = "word-watermark";
        wm.setAttribute("data-testid", "word-watermark");
        wm.textContent = this.state.doc.watermark;
        wm.style.position = "absolute";
        wm.style.top = "50%";
        wm.style.left = "50%";
        wm.style.transform = "translate(-50%, -50%) rotate(-45deg)";
        wm.style.fontSize = `${54 * this.state.zoom}pt`;
        wm.style.fontWeight = "bold";
        wm.style.color = "rgba(156, 163, 175, 0.2)";
        wm.style.pointerEvents = "none";
        wm.style.zIndex = "2";
        wm.style.userSelect = "none";
        pageEl.appendChild(wm);
      }

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

      if (totalPages > 1) {
        const delPageBtn = document.createElement("button");
        delPageBtn.className = "btn-delete-page";
        delPageBtn.setAttribute("data-testid", "btn-delete-page");
        delPageBtn.title = "Delete Page";
        delPageBtn.textContent = "✕";
        delPageBtn.onclick = (e) => {
          e.stopPropagation();
          this.deletePage(pIdx, true);
        };
        headerEl.appendChild(delPageBtn);
      }
      headerEl.onclick = (e) => {
        e.stopPropagation();
        this.activateHeaderEditing(headerEl, pIdx);
      };
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
      footerEl.onclick = (e) => {
        e.stopPropagation();
        this.activateFooterEditing(footerEl, pIdx);
      };
      footerEl.ondblclick = (e) => {
        e.stopPropagation();
        this.activateFooterEditing(footerEl, pIdx);
      };
      pageEl.appendChild(footerEl);

      // 4. ContentEditable Body (Subphase 29.1.1)
      const bodyEl = document.createElement("div");
      bodyEl.className = "word-page-body";
      bodyEl.contentEditable = "true";
      bodyEl.setAttribute("data-page-index", String(pIdx));
      bodyEl.setAttribute("data-testid", "word-page-body");
      bodyEl.setAttribute("spellcheck", "false");
      bodyEl.setAttribute("data-placeholder", "Click here or type to add content...");
      bodyEl.style.position = "absolute";
      bodyEl.style.top = `${this.marginY * this.state.zoom}px`;
      bodyEl.style.left = `${this.marginX * this.state.zoom}px`;
      bodyEl.style.width = `${(this.pageWidth - this.marginX * 2) * this.state.zoom}px`;
      bodyEl.style.height = `${(this.pageHeight - this.marginY * 2) * this.state.zoom}px`;
      bodyEl.style.outline = "none";
      bodyEl.style.zIndex = "5";
      bodyEl.style.fontSize = `${11 * this.state.zoom}pt`;
      bodyEl.style.fontFamily = "Calibri, sans-serif";
      bodyEl.style.lineHeight = "1.5";
      bodyEl.style.boxSizing = "border-box";
      bodyEl.style.cursor = "text";
      bodyEl.style.whiteSpace = "pre-wrap";
      bodyEl.style.wordBreak = "break-word";
      bodyEl.style.padding = "4px";

      if (this.state.doc.columns && this.state.doc.columns > 1) {
        bodyEl.style.columnCount = String(this.state.doc.columns);
        bodyEl.style.columnGap = "24px";
        bodyEl.style.columnRule = "1px solid #e5e7eb";
      }

      const textOnly = page.paragraphs
        .filter((p) => !p.shape && !p.image && !p.table)
        .map((p) => p.runs.map((r) => r.text).join(""))
        .join("\n");
      bodyEl.textContent = textOnly;

      bodyEl.addEventListener("input", () => {
        const textCount = (bodyEl.textContent?.match(/\n/g) || []).length;
        const innerCount = (bodyEl.innerText?.match(/\n/g) || []).length;
        const text = textCount >= innerCount
          ? (bodyEl.textContent || bodyEl.innerText || "")
          : (bodyEl.innerText || bodyEl.textContent || "");
        const lines = text.split(/\r?\n/);
        this.syncPageTextFromDom(pIdx, lines);
      });

      bodyEl.addEventListener("focus", () => {
        this.state.cursor.pageIndex = pIdx;
        this.updateStatus();
      });

      pageEl.appendChild(bodyEl);

      // 5. Interactive Floating Shapes (Subphase 29.1.1)
      page.paragraphs.forEach((p) => {
        if (p.shape) {
          const shapeEl = this.createFloatingShapeElement(p, pIdx);
          pageEl.appendChild(shapeEl);
        }
      });

      pagesContainer.appendChild(pageEl);

      if (pIdx < totalPages - 1) {
        const divider = document.createElement("div");
        divider.className = "word-page-divider-action";
        const addBtn = document.createElement("button");
        addBtn.className = "btn-add-page-floating btn-add-page";
        addBtn.setAttribute("data-testid", "btn-add-page");
        addBtn.innerHTML = "<span>➕</span> <strong>Add Page</strong>";
        addBtn.title = "Add a new blank page";
        addBtn.onclick = () => this.insertNewPage();
        divider.appendChild(addBtn);
        pagesContainer.appendChild(divider);
      }

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

          // 4a. Render Table if present
          if (p.table) {
            const tableWidth = this.pageWidth - 2 * this.marginX - (p.indent || 0);
            const colWidth = tableWidth / Math.max(1, p.table.colCount);
            const rowHeight = 26;

            p.table.rows.forEach((row, rIdx) => {
              row.cells.forEach((cell, cIdx) => {
                const cellX = lineX + cIdx * colWidth;
                const cellY = lineY;
                ctx.strokeStyle = "#9ca3af";
                ctx.lineWidth = 1;
                ctx.strokeRect(cellX, cellY, colWidth, rowHeight);

                if (rIdx === 0) {
                  ctx.fillStyle = "#f3f4f6";
                  ctx.fillRect(cellX + 1, cellY + 1, colWidth - 2, rowHeight - 2);
                  ctx.fillStyle = "#111827";
                  ctx.font = "bold 11px Calibri, sans-serif";
                } else {
                  ctx.fillStyle = "#374151";
                  ctx.font = "11px Calibri, sans-serif";
                }
                ctx.fillText(cell.text, cellX + 6, cellY + 17);
              });
              lineY += rowHeight;
            });
            lineY += 12;
            return;
          }

          // 4b. Render Image if present
          if (p.image) {
            const imgWidth = Math.min(p.image.width, this.pageWidth - 2 * this.marginX);
            const imgHeight = p.image.height;
            const imgX = this.marginX + (this.pageWidth - 2 * this.marginX - imgWidth) / 2;

            ctx.fillStyle = "#f9fafb";
            ctx.fillRect(imgX, lineY, imgWidth, imgHeight);
            ctx.strokeStyle = "#cbd5e1";
            ctx.lineWidth = 1;
            ctx.strokeRect(imgX, lineY, imgWidth, imgHeight);

            ctx.fillStyle = "#475569";
            ctx.font = "12px Calibri, sans-serif";
            ctx.textAlign = "center";
            ctx.fillText(
              `🖼️ ${p.image.caption || "Image"} (${imgWidth}x${imgHeight})`,
              imgX + imgWidth / 2,
              lineY + imgHeight / 2
            );
            ctx.textAlign = "left";

            lineY += imgHeight + 14;
            return;
          }

          // 4c. Render Shape if present
          if (p.shape) {
            const shpWidth = p.shape.width;
            const shpHeight = p.shape.height;
            const shpX = this.marginX + (this.pageWidth - 2 * this.marginX - shpWidth) / 2;

            ctx.fillStyle = p.shape.color || "#2563eb";
            ctx.strokeStyle = "#1d4ed8";
            ctx.lineWidth = 2;

            if (p.shape.shapeType === "ellipse") {
              ctx.beginPath();
              ctx.ellipse(shpX + shpWidth / 2, lineY + shpHeight / 2, shpWidth / 2, shpHeight / 2, 0, 0, Math.PI * 2);
              ctx.fill();
              ctx.stroke();
            } else {
              ctx.fillRect(shpX, lineY, shpWidth, shpHeight);
              ctx.strokeRect(shpX, lineY, shpWidth, shpHeight);
            }

            lineY += shpHeight + 14;
            return;
          }

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

        // 6. Caret & Inline Ghost Text Rendering
        if (this.caretVisible && caretRenderCoords && caretRenderCoords.canvasIndex === pIdx) {
          ctx.fillStyle = "#000000";
          ctx.fillRect(caretRenderCoords.x, caretRenderCoords.y, 2, caretRenderCoords.height);

          // Inline Ghost Text Completion Overlay (Subphase 20.1.3)
          if (this.ghostText) {
            ctx.save();
            ctx.font = "italic 14px " + (this.state.activeFormatting?.font || "Calibri");
            ctx.fillStyle = "#9ca3af";
            ctx.fillText(
              this.ghostText,
              caretRenderCoords.x + 4,
              caretRenderCoords.y + caretRenderCoords.height * 0.8
            );
            ctx.restore();
          }
        }

        // 7. Remote Peer Awareness (Subphase 14.2.2)
        const remotePeers = collabState.getPeers();
        for (const peer of remotePeers) {
          const peerColor = peer.color_hex || "#1E88E5";

          // Remote Selection Highlight
          if (peer.selection && peer.selection.text_start != null && peer.selection.text_end != null) {
            const selStart = Math.min(peer.selection.text_start, peer.selection.text_end);
            const selEnd = Math.max(peer.selection.text_start, peer.selection.text_end);
            if (selStart < selEnd) {
              ctx.save();
              ctx.fillStyle = peerColor + "40"; // 25% opacity tint
              for (const l of pageLayout.lines) {
                const lineStartOffset = l.startOffsetInRun;
                const lineEndOffset = l.endOffsetInRun;
                if (lineEndOffset <= selStart || lineStartOffset >= selEnd) continue;
                const startCol = Math.max(0, selStart - lineStartOffset);
                const endCol = Math.min(l.text.length, selEnd - lineStartOffset);
                if (startCol < endCol) {
                  let leftX = l.x;
                  for (let i = 0; i < startCol; i++) leftX += l.charAdvances[i] || 0;
                  let width = 0;
                  for (let i = startCol; i < endCol; i++) width += l.charAdvances[i] || 0;
                  ctx.fillRect(leftX, l.y, Math.max(2, width), l.height);
                }
              }
              ctx.restore();
            }
          }

          // Remote Caret and Name Tag
          if (peer.cursor && peer.cursor.text_offset != null) {
            const offset = peer.cursor.text_offset;
            for (const l of pageLayout.lines) {
              if (offset >= l.startOffsetInRun && offset <= l.endOffsetInRun) {
                const col = offset - l.startOffsetInRun;
                let caretX = l.x;
                for (let i = 0; i < col; i++) caretX += l.charAdvances[i] || 0;
                const caretY = l.y;
                const caretH = l.height;

                // Vertical remote caret line
                ctx.fillStyle = peerColor;
                ctx.fillRect(caretX, caretY, 2, caretH);

                // Floating name badge
                ctx.save();
                ctx.font = "bold 10px sans-serif";
                const nameText = peer.username || "Anonymous";
                const textWidth = ctx.measureText(nameText).width;
                const badgeW = textWidth + 8;
                const badgeH = 16;
                const badgeX = caretX - 2;
                const badgeY = caretY - badgeH - 2;

                ctx.fillStyle = peerColor;
                ctx.beginPath();
                if (ctx.roundRect) {
                  ctx.roundRect(badgeX, badgeY, badgeW, badgeH, 3);
                } else {
                  ctx.rect(badgeX, badgeY, badgeW, badgeH);
                }
                ctx.fill();

                // Small triangular pointer
                ctx.beginPath();
                ctx.moveTo(caretX, caretY - 2);
                ctx.lineTo(caretX + 4, caretY - 2);
                ctx.lineTo(caretX, caretY);
                ctx.fill();

                ctx.fillStyle = "#ffffff";
                ctx.fillText(nameText, badgeX + 4, badgeY + 12);
                ctx.restore();
                break;
              }
            }
          }
        }
      }
    });

    // 5. "+ New Page" & "Delete Empty Page" Action Buttons at Bottom of Pages
    const pageActionsContainer = document.createElement("div");
    pageActionsContainer.className = "word-page-bottom-actions";
    pageActionsContainer.style.display = "flex";
    pageActionsContainer.style.alignItems = "center";
    pageActionsContainer.style.gap = "8px";
    pageActionsContainer.style.margin = "12px 0 32px 0";

    const addPageBtn = document.createElement("button");
    addPageBtn.className = "btn-add-page";
    addPageBtn.setAttribute("data-testid", "btn-add-new-page");
    addPageBtn.innerHTML = "<span>➕</span> <strong>Blank Page / New Page</strong>";
    addPageBtn.title = "Add a new blank page (Ctrl+Enter)";
    addPageBtn.style.padding = "8px 18px";
    addPageBtn.style.backgroundColor = "#ffffff";
    addPageBtn.style.border = "1px solid #d1d5db";
    addPageBtn.style.borderRadius = "4px";
    addPageBtn.style.cursor = "pointer";
    addPageBtn.style.fontWeight = "600";
    addPageBtn.style.fontSize = "12px";
    addPageBtn.style.color = "#374151";
    addPageBtn.style.boxShadow = "0 1px 3px rgba(0,0,0,0.08)";
    addPageBtn.onclick = () => this.insertNewPage();
    pageActionsContainer.appendChild(addPageBtn);

    if (this.state.doc.pages.length > 1) {
      const delPageBtn = document.createElement("button");
      delPageBtn.className = "btn-delete-empty-page";
      delPageBtn.setAttribute("data-testid", "btn-delete-empty-page");
      delPageBtn.innerHTML = "<span>🗑️</span> <strong>Delete Empty Page</strong>";
      delPageBtn.title = "Delete current page if empty";
      delPageBtn.style.padding = "8px 18px";
      delPageBtn.style.backgroundColor = "#ffffff";
      delPageBtn.style.border = "1px solid #d1d5db";
      delPageBtn.style.borderRadius = "4px";
      delPageBtn.style.cursor = "pointer";
      delPageBtn.style.fontWeight = "600";
      delPageBtn.style.fontSize = "12px";
      delPageBtn.style.color = "#dc2626";
      delPageBtn.style.boxShadow = "0 1px 3px rgba(0,0,0,0.08)";
      delPageBtn.onclick = () => this.deleteEmptyPage();
      pageActionsContainer.appendChild(delPageBtn);
    }

    pagesContainer.appendChild(pageActionsContainer);

    this.container.appendChild(pagesContainer);
    this.updateStatus();

    // Auto-focus active page canvas so user can type immediately
    if (this.canvas) {
      try {
        this.canvas.focus();
      } catch {
        // Ignored in test envs
      }
    }
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

  public setGhostText(text: string | null): void {
    this.ghostText = text;
    this.render();
  }

  public getGhostText(): string | null {
    return this.ghostText;
  }

  public acceptGhostText(): boolean {
    if (!this.ghostText) return false;
    const textToInsert = this.ghostText;
    this.ghostText = null;
    this.typeText(textToInsert);
    this.emitDocChange("AcceptGhostText", { text: textToInsert });
    return true;
  }

  public dismissGhostText(): void {
    if (this.ghostText) {
      this.ghostText = null;
      this.render();
    }
  }

  private syncPageTextFromDom(pageIndex: number, lines: string[]) {
    const page = this.state.doc.pages[pageIndex];
    if (!page) return;
    const nonTextParas = page.paragraphs.filter((p) => p.shape || p.image || p.table);
    const newParas = lines.map((line, idx) => ({
      id: `p-${pageIndex}-${idx}-${Date.now()}`,
      style: "Normal",
      alignment: "left" as const,
      runs: [{ text: line, font: "Calibri", size: 11 }],
    }));
    page.paragraphs = [...newParas, ...nonTextParas];
    this.emitDocChange("UpdateText", { pageIndex, lines });
    this.updateStatus();
  }

  private createFloatingShapeElement(p: import("../../state/writerState").WordParagraph, _pageIndex: number): HTMLElement {
    const s = p.shape!;
    const shapeEl = document.createElement("div");
    shapeEl.className = "word-floating-shape";
    shapeEl.setAttribute("data-shape-id", p.id);
    shapeEl.setAttribute("data-testid", "word-floating-shape");
    shapeEl.tabIndex = 0;

    const left = (s.x ?? 40) * this.state.zoom;
    const top = (this.marginY + (s.y ?? 40)) * this.state.zoom;
    const w = (s.width || 140) * this.state.zoom;
    const h = (s.height || 90) * this.state.zoom;

    shapeEl.style.left = `${left}px`;
    shapeEl.style.top = `${top}px`;
    shapeEl.style.width = `${w}px`;
    shapeEl.style.height = `${h}px`;

    // Shape Body
    const body = document.createElement("div");
    body.className = `shape-body ${s.shapeType}`;
    body.style.backgroundColor = s.color || "#2563eb";

    // Shape Text inside
    const textSpan = document.createElement("span");
    textSpan.className = "shape-text";
    textSpan.contentEditable = "true";
    textSpan.textContent = s.text || "";
    textSpan.addEventListener("input", (e) => {
      e.stopPropagation();
      s.text = textSpan.textContent || "";
      this.state.updateShape(p.id, { text: s.text });
    });
    body.appendChild(textSpan);
    shapeEl.appendChild(body);

    // Delete Button
    const delBtn = document.createElement("button");
    delBtn.className = "shape-delete-btn";
    delBtn.setAttribute("data-testid", "shape-delete-btn");
    delBtn.title = "Delete Shape";
    delBtn.textContent = "✕";
    delBtn.onclick = (e) => {
      e.stopPropagation();
      this.state.deleteShape(p.id);
      this.emitDocChange("DeleteShape", { shapeId: p.id });
      this.render();
    };
    shapeEl.appendChild(delBtn);

    // 8 Resize Handles
    const handlePositions = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
    handlePositions.forEach((pos) => {
      const hEl = document.createElement("div");
      hEl.className = `shape-handle handle-${pos}`;
      hEl.setAttribute("data-handle", pos);
      hEl.addEventListener("mousedown", (e) => {
        e.stopPropagation();
        e.preventDefault();
        this.startShapeResize(e, p.id, pos, shapeEl);
      });
      shapeEl.appendChild(hEl);
    });

    // Select on click
    shapeEl.addEventListener("click", (e) => {
      e.stopPropagation();
      document.querySelectorAll(".word-floating-shape.selected").forEach((el) => el.classList.remove("selected"));
      shapeEl.classList.add("selected");
    });

    // Delete on Del or Backspace key
    shapeEl.addEventListener("keydown", (e) => {
      if (e.key === "Delete" || e.key === "Backspace") {
        if (document.activeElement !== textSpan) {
          e.preventDefault();
          this.state.deleteShape(p.id);
          this.emitDocChange("DeleteShape", { shapeId: p.id });
          this.render();
        }
      }
    });

    // Drag to move
    shapeEl.addEventListener("mousedown", (e) => {
      if ((e.target as HTMLElement).classList.contains("shape-handle") || e.target === textSpan || e.target === delBtn) {
        return;
      }
      document.querySelectorAll(".word-floating-shape.selected").forEach((el) => el.classList.remove("selected"));
      shapeEl.classList.add("selected");
      this.startShapeDrag(e, p.id, shapeEl);
    });

    return shapeEl;
  }

  private startShapeDrag(e: MouseEvent, shapeId: string, shapeEl: HTMLElement) {
    const startX = e.clientX;
    const startY = e.clientY;
    const initLeft = parseFloat(shapeEl.style.left) || 0;
    const initTop = parseFloat(shapeEl.style.top) || 0;

    const onMouseMove = (ev: MouseEvent) => {
      const dx = (ev.clientX - startX) / this.state.zoom;
      const dy = (ev.clientY - startY) / this.state.zoom;
      const newLeft = Math.max(0, initLeft + dx);
      const newTop = Math.max(0, initTop + dy);
      shapeEl.style.left = `${newLeft}px`;
      shapeEl.style.top = `${newTop}px`;
    };

    const onMouseUp = () => {
      if (typeof window !== "undefined") {
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
      }
      const finalX = parseFloat(shapeEl.style.left) / this.state.zoom;
      const finalY = (parseFloat(shapeEl.style.top) - this.marginY * this.state.zoom) / this.state.zoom;
      this.state.updateShape(shapeId, { x: Math.max(0, Math.round(finalX)), y: Math.max(0, Math.round(finalY)) });
      this.emitDocChange("MoveShape", { shapeId, x: finalX, y: finalY });
    };

    if (typeof window !== "undefined") {
      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    }
  }

  private startShapeResize(e: MouseEvent, shapeId: string, handle: string, shapeEl: HTMLElement) {
    const startX = e.clientX;
    const startY = e.clientY;
    const initW = parseFloat(shapeEl.style.width) || 140;
    const initH = parseFloat(shapeEl.style.height) || 90;

    const onMouseMove = (ev: MouseEvent) => {
      const dx = (ev.clientX - startX) / this.state.zoom;
      const dy = (ev.clientY - startY) / this.state.zoom;
      let newW = initW;
      let newH = initH;

      if (handle.includes("e")) newW = Math.max(30, initW + dx);
      if (handle.includes("s")) newH = Math.max(30, initH + dy);
      if (handle.includes("w")) newW = Math.max(30, initW - dx);
      if (handle.includes("n")) newH = Math.max(30, initH - dy);

      shapeEl.style.width = `${newW}px`;
      shapeEl.style.height = `${newH}px`;
    };

    const onMouseUp = () => {
      if (typeof window !== "undefined") {
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
      }
      const finalW = Math.round(parseFloat(shapeEl.style.width) / this.state.zoom);
      const finalH = Math.round(parseFloat(shapeEl.style.height) / this.state.zoom);
      this.state.updateShape(shapeId, { width: finalW, height: finalH });
      this.emitDocChange("ResizeShape", { shapeId, width: finalW, height: finalH });
    };

    if (typeof window !== "undefined") {
      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    }
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
