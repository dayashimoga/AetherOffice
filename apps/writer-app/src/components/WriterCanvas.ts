import { CaretPosition, WriterState, writerState } from "../state/writerState";

export class WriterCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: WriterState;

  public readonly pageWidth = 816; // 8.5 inches at 96 dpi
  public readonly pageHeight = 1056; // 11 inches at 96 dpi
  public readonly marginX = 96; // 1 inch
  public readonly marginY = 96;
  public readonly pageGap = 32;

  private caretVisible = true;
  private caretTimer: number | null = null;
  private lastActivityTime = Date.now();
  private isMouseDown = false;
  private onStatusChange?: (pageText: string, words: number) => void;

  // Cached layout lines for hit testing and subpixel caret placement
  private layoutPages: {
    pageIndex: number;
    topY: number;
    lines: {
      paragraphIndex: number;
      runIndex: number;
      text: string;
      x: number;
      y: number;
      height: number;
      fontSize: number;
      charAdvances: number[];
    }[];
  }[] = [];

  constructor(container: HTMLElement, state: WriterState = writerState) {
    this.container = container;
    this.state = state;

    this.container.innerHTML = "";
    this.container.style.overflowY = "auto";
    this.container.style.overflowX = "auto";
    this.container.style.backgroundColor = "#eef2f5";
    this.container.style.display = "flex";
    this.container.style.justifyContent = "center";
    this.container.style.alignItems = "flex-start";
    this.container.style.padding = "24px 0";

    this.canvas = document.createElement("canvas");
    this.canvas.className = "writer-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "text";

    this.container.appendChild(this.canvas);

    const context = this.canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context for Writer");
    this.ctx = context;

    this.initCanvasSize();
    this.startCaretBlink();
    this.attachEvents();
    this.render();
  }

  public setOnStatusChange(cb: (pageText: string, words: number) => void) {
    this.onStatusChange = cb;
    this.updateStatus();
  }

  public destroy() {
    if (this.caretTimer) clearInterval(this.caretTimer);
  }

  private initCanvasSize() {
    const dpr = window.devicePixelRatio || 1;
    const totalPages = this.state.doc.pages.length;
    const totalHeight = totalPages * this.pageHeight + (totalPages - 1) * this.pageGap + 48;
    const totalWidth = this.pageWidth + 64;

    this.canvas.width = totalWidth * dpr * this.state.zoom;
    this.canvas.height = totalHeight * dpr * this.state.zoom;
    this.canvas.style.width = `${totalWidth * this.state.zoom}px`;
    this.canvas.style.height = `${totalHeight * this.state.zoom}px`;

    this.ctx.setTransform(1, 0, 0, 1, 0, 0);
    this.ctx.scale(dpr * this.state.zoom, dpr * this.state.zoom);
  }

  private startCaretBlink() {
    if (this.caretTimer) clearInterval(this.caretTimer);
    this.caretTimer = window.setInterval(() => {
      const elapsed = Date.now() - this.lastActivityTime;
      // Solid on for first 500ms after typing/action
      if (elapsed < 500) {
        this.caretVisible = true;
      } else {
        const phase = Math.floor(elapsed / 530) % 2;
        this.caretVisible = phase === 0;
      }
      this.render();
    }, 100);
  }

  private resetBlink() {
    this.lastActivityTime = Date.now();
    this.caretVisible = true;
  }

  private attachEvents() {
    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      this.canvas.focus();
      this.isMouseDown = true;
      this.resetBlink();

      const pt = this.getCanvasLocalCoords(e);
      const hit = this.hitTest(pt.x, pt.y);
      if (hit) {
        this.state.cursor = hit;
        if (e.shiftKey) {
          const anchor = this.state.selection ? this.state.selection.anchor : this.state.cursor;
          this.state.selection = { anchor, focus: hit };
        } else {
          this.state.selection = { anchor: hit, focus: hit };
        }
        this.render();
      }
    });

    window.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isMouseDown) return;
      const pt = this.getCanvasLocalCoords(e);
      const hit = this.hitTest(pt.x, pt.y);
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

    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      this.handleKeyDown(e);
    });

    this.canvas.addEventListener("wheel", (e: WheelEvent) => {
      if (e.ctrlKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.1 : -0.1;
        this.state.setZoom(this.state.zoom + delta);
        this.initCanvasSize();
        this.render();
      }
    });
  }

  private getCanvasLocalCoords(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left) / this.state.zoom;
    const y = (e.clientY - rect.top) / this.state.zoom;
    return { x, y };
  }

  private hitTest(clickX: number, clickY: number): CaretPosition | null {
    if (this.layoutPages.length === 0) return null;

    // 1. Find target page
    let targetPage = this.layoutPages[0];
    for (const p of this.layoutPages) {
      if (clickY >= p.topY && clickY <= p.topY + this.pageHeight + this.pageGap) {
        targetPage = p;
        break;
      }
      if (clickY > p.topY) targetPage = p;
    }

    if (targetPage.lines.length === 0) {
      return { pageIndex: targetPage.pageIndex, paragraphIndex: 0, runIndex: 0, offset: 0 };
    }

    // 2. Find closest line
    let closestLine = targetPage.lines[0];
    let minDist = Infinity;
    for (const l of targetPage.lines) {
      const midY = l.y + l.height / 2;
      const dist = Math.abs(clickY - midY);
      if (dist < minDist) {
        minDist = dist;
        closestLine = l;
      }
    }

    // 3. Subpixel advance hit test
    const relX = clickX - closestLine.x;
    let offset = 0;
    let currX = 0;
    for (let i = 0; i < closestLine.charAdvances.length; i++) {
      const adv = closestLine.charAdvances[i];
      if (relX < currX + adv / 2) {
        offset = i;
        break;
      } else if (relX < currX + adv) {
        offset = i + 1;
        break;
      }
      currX += adv;
      offset = i + 1;
    }

    return {
      pageIndex: targetPage.pageIndex,
      paragraphIndex: closestLine.paragraphIndex,
      runIndex: closestLine.runIndex,
      offset: Math.min(offset, closestLine.text.length),
    };
  }

  private handleKeyDown(e: KeyboardEvent) {
    const page = this.state.doc.pages[this.state.cursor.pageIndex] || this.state.doc.pages[0];
    const para = page.paragraphs[this.state.cursor.paragraphIndex] || page.paragraphs[0];
    const run = para.runs[this.state.cursor.runIndex] || para.runs[0];

    this.resetBlink();

    if (e.ctrlKey || e.metaKey) {
      if (e.key === "a" || e.key === "A") {
        e.preventDefault();
        this.state.selectAll();
        this.render();
        return;
      } else if (e.key === "z" || e.key === "Z") {
        e.preventDefault();
        this.state.undo();
        this.render();
        return;
      }
    }

    if (e.key === "ArrowLeft") {
      e.preventDefault();
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
      this.state.clearSelection();
      this.render();
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
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
      this.state.clearSelection();
      this.render();
    } else if (e.key === "Home") {
      e.preventDefault();
      this.state.cursor.offset = 0;
      this.state.clearSelection();
      this.render();
    } else if (e.key === "End") {
      e.preventDefault();
      this.state.cursor.offset = run.text.length;
      this.state.clearSelection();
      this.render();
    } else if (e.key === "Backspace") {
      e.preventDefault();
      if (this.state.cursor.offset > 0) {
        run.text = run.text.slice(0, this.state.cursor.offset - 1) + run.text.slice(this.state.cursor.offset);
        this.state.cursor.offset--;
        this.state.saveSnapshot();
        this.render();
      }
    } else if (e.key === "Delete") {
      e.preventDefault();
      if (this.state.cursor.offset < run.text.length) {
        run.text = run.text.slice(0, this.state.cursor.offset) + run.text.slice(this.state.cursor.offset + 1);
        this.state.saveSnapshot();
        this.render();
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      const textAfter = run.text.slice(this.state.cursor.offset);
      run.text = run.text.slice(0, this.state.cursor.offset);

      page.paragraphs.splice(this.state.cursor.paragraphIndex + 1, 0, {
        id: `p-${Date.now()}`,
        style: "Normal",
        alignment: "left",
        runs: [{ text: textAfter, font: this.state.activeFormat.font, size: this.state.activeFormat.size }],
      });
      this.state.cursor.paragraphIndex++;
      this.state.cursor.runIndex = 0;
      this.state.cursor.offset = 0;
      this.state.saveSnapshot();
      this.render();
    } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      run.text = run.text.slice(0, this.state.cursor.offset) + e.key + run.text.slice(this.state.cursor.offset);
      this.state.cursor.offset++;
      this.render();
    }
  }

  public render() {
    const ctx = this.ctx;
    const totalPages = this.state.doc.pages.length;
    const totalHeight = totalPages * this.pageHeight + (totalPages - 1) * this.pageGap + 48;
    const totalWidth = this.pageWidth + 64;

    ctx.clearRect(0, 0, totalWidth, totalHeight);
    this.layoutPages = [];

    let currentY = 24;
    let caretRenderCoords: { x: number; y: number; height: number } | null = null;

    // Render Each Document Page
    this.state.doc.pages.forEach((page, pIdx) => {
      const pageX = 32;
      const pageY = currentY;

      // 1. Page Drop Shadow
      ctx.save();
      ctx.shadowColor = "rgba(0, 0, 0, 0.14)";
      ctx.shadowBlur = 16;
      ctx.shadowOffsetY = 4;
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(pageX, pageY, this.pageWidth, this.pageHeight);
      ctx.restore();

      // 2. Page Border
      ctx.strokeStyle = "#d1d5db";
      ctx.lineWidth = 1;
      ctx.strokeRect(pageX, pageY, this.pageWidth, this.pageHeight);

      // 3. Margin Guide (Light dashed)
      ctx.strokeStyle = "#f3f4f6";
      ctx.setLineDash([4, 4]);
      ctx.strokeRect(pageX + this.marginX, pageY + this.marginY, this.pageWidth - 2 * this.marginX, this.pageHeight - 2 * this.marginY);
      ctx.setLineDash([]);

      // 4. Page Header & Number
      ctx.font = "10px Calibri, sans-serif";
      ctx.fillStyle = "#9ca3af";
      ctx.textAlign = "right";
      ctx.fillText(`Page ${pIdx + 1} of ${totalPages}`, pageX + this.pageWidth - this.marginX, pageY + 48);

      const pageLayout = { pageIndex: pIdx, topY: pageY, lines: [] as any[] };

      // 5. Layout & Render Paragraphs
      let lineY = pageY + this.marginY + 10;
      page.paragraphs.forEach((p, pIndex) => {
        let lineX = pageX + this.marginX;

        p.runs.forEach((r, rIndex) => {
          const fontSize = r.size || (p.style === "Heading1" ? 22 : p.style === "Heading2" ? 15 : 11);
          const fontFamily = r.font || "Calibri";
          const isBold = r.bold !== undefined ? r.bold : p.style.startsWith("Heading");
          const isItalic = r.italic || false;

          let fontStyle = "";
          if (isItalic) fontStyle += "italic ";
          if (isBold) fontStyle += "bold ";
          ctx.font = `${fontStyle}${fontSize}px ${fontFamily}, sans-serif`;

          ctx.fillStyle = r.color || (p.style === "Heading1" ? "#1f497d" : p.style === "Heading2" ? "#2e75b6" : "#242424");
          ctx.textAlign = "left";

          // Word-wrap long paragraphs into multiple lines
          const words = r.text.split(" ");
          let currentLineText = "";

          for (let w = 0; w < words.length; w++) {
            const testLine = currentLineText ? `${currentLineText} ${words[w]}` : words[w];
            const testWidth = ctx.measureText(testLine).width;
            const maxLineWidth = this.pageWidth - 2 * this.marginX;

            if (testWidth > maxLineWidth && currentLineText) {
              // Draw line
              this.renderTextLine(ctx, currentLineText, lineX, lineY, fontSize, r.underline || false);
              const charAdv = this.computeCharAdvances(ctx, currentLineText);
              pageLayout.lines.push({
                paragraphIndex: pIndex,
                runIndex: rIndex,
                text: currentLineText,
                x: lineX,
                y: lineY - fontSize + 2,
                height: fontSize * 1.25,
                fontSize,
                charAdvances: charAdv,
              });

              lineY += fontSize * 1.35;
              currentLineText = words[w];
            } else {
              currentLineText = testLine;
            }
          }

          if (currentLineText) {
            this.renderTextLine(ctx, currentLineText, lineX, lineY, fontSize, r.underline || false);
            const charAdv = this.computeCharAdvances(ctx, currentLineText);
            pageLayout.lines.push({
              paragraphIndex: pIndex,
              runIndex: rIndex,
              text: currentLineText,
              x: lineX,
              y: lineY - fontSize + 2,
              height: fontSize * 1.25,
              fontSize,
              charAdvances: charAdv,
            });

            // Caret tracking
            if (pIdx === this.state.cursor.pageIndex && pIndex === this.state.cursor.paragraphIndex && rIndex === this.state.cursor.runIndex) {
              const textBefore = r.text.slice(0, this.state.cursor.offset);
              const subpixelWidth = ctx.measureText(textBefore).width;
              caretRenderCoords = {
                x: lineX + subpixelWidth,
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
      currentY += this.pageHeight + this.pageGap;
    });

    // 6. Text Selection Highlighting
    if (this.state.selection && !this.state.isSelectionCollapsed()) {
      ctx.fillStyle = "rgba(51, 153, 255, 0.35)";
      const sel = this.state.selection;
      for (const pl of this.layoutPages) {
        if (pl.pageIndex >= sel.anchor.pageIndex && pl.pageIndex <= sel.focus.pageIndex) {
          for (const l of pl.lines) {
            ctx.fillRect(l.x, l.y, ctx.measureText(l.text).width, l.height);
          }
        }
      }
    }

    // 7. Caret Rendering
    if (this.caretVisible && caretRenderCoords) {
      ctx.fillStyle = "#000000";
      ctx.fillRect((caretRenderCoords as any).x, (caretRenderCoords as any).y, 2, (caretRenderCoords as any).height);
    }

    this.updateStatus();
  }

  private renderTextLine(
    ctx: CanvasRenderingContext2D,
    text: string,
    x: number,
    y: number,
    _fontSize: number,
    underline: boolean,
  ) {
    ctx.fillText(text, x, y);
    if (underline) {
      const width = ctx.measureText(text).width;
      ctx.beginPath();
      ctx.moveTo(x, y + 2);
      ctx.lineTo(x + width, y + 2);
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
      const totalPages = this.state.doc.pages.length;
      const words = this.state.getTotalWords();
      this.onStatusChange(`Page ${pageNum} of ${totalPages}`, words);
    }
  }
}
