import { WriterState, writerState } from "../state/writerState";

export class WriterCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: WriterState;
  private pageWidth = 816; // 8.5 inches at 96 dpi
  private pageHeight = 1056; // 11 inches at 96 dpi
  private marginX = 72; // 1 inch
  private marginY = 72;
  private caretVisible = true;
  private caretTimer: number | null = null;
  private onStatusChange?: (pageText: string, words: number) => void;

  constructor(container: HTMLElement, state: WriterState = writerState) {
    this.container = container;
    this.state = state;

    this.canvas = document.createElement("canvas");
    this.canvas.className = "writer-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.margin = "32px auto";
    this.canvas.style.boxShadow = "0 4px 16px rgba(0, 0, 0, 0.15)";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "text";

    this.container.innerHTML = "";
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
  }

  private initCanvasSize() {
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = this.pageWidth * dpr;
    this.canvas.height = this.pageHeight * dpr;
    this.canvas.style.width = `${this.pageWidth}px`;
    this.canvas.style.height = `${this.pageHeight}px`;
    this.ctx.scale(dpr, dpr);
  }

  private startCaretBlink() {
    if (this.caretTimer) clearInterval(this.caretTimer);
    this.caretTimer = window.setInterval(() => {
      this.caretVisible = !this.caretVisible;
      this.render();
    }, 500);
  }

  public destroy() {
    if (this.caretTimer) clearInterval(this.caretTimer);
  }

  private attachEvents() {
    this.canvas.addEventListener("click", () => {
      this.canvas.focus();
      this.caretVisible = true;
      this.render();
    });

    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      const activeSec = this.state.doc.sections[0];
      if (!activeSec || activeSec.paragraphs.length === 0) return;

      const p = activeSec.paragraphs[this.state.cursor.paragraphIndex] || activeSec.paragraphs[0];
      const r = p.runs[this.state.cursor.runIndex] || p.runs[0];

      if (e.key === "Backspace") {
        e.preventDefault();
        if (r.text.length > 0) {
          r.text = r.text.slice(0, -1);
          this.render();
        }
      } else if (e.key === "Enter") {
        e.preventDefault();
        activeSec.paragraphs.splice(this.state.cursor.paragraphIndex + 1, 0, {
          id: `p-${Date.now()}`,
          style: "Normal",
          alignment: "left",
          runs: [{ text: "", font: this.state.activeFormat.font, size: this.state.activeFormat.size }],
        });
        this.state.cursor.paragraphIndex++;
        this.state.cursor.runIndex = 0;
        this.render();
      } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        r.text += e.key;
        this.caretVisible = true;
        this.render();
      }
    });
  }

  public render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.pageWidth, this.pageHeight);

    // 1. Page Background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, this.pageWidth, this.pageHeight);

    // 2. Page Border / Margin guide
    ctx.strokeStyle = "#ebebeb";
    ctx.setLineDash([4, 4]);
    ctx.strokeRect(this.marginX, this.marginY, this.pageWidth - 2 * this.marginX, this.pageHeight - 2 * this.marginY);
    ctx.setLineDash([]);

    // 3. Document Content Layout
    let curY = this.marginY + 20;
    let totalWords = 0;
    let caretX = this.marginX;
    let caretY = curY - 14;
    let caretH = 18;

    const activeSec = this.state.doc.sections[0];
    if (activeSec) {
      activeSec.paragraphs.forEach((p, pIdx) => {
        let curX = this.marginX;

        p.runs.forEach((r, rIdx) => {
          const fontSize = r.size || (p.style === "Heading1" ? 22 : 11);
          const fontFamily = r.font || "Calibri";
          const isBold = r.bold !== undefined ? r.bold : p.style === "Heading1";
          const isItalic = r.italic || false;

          let fontStyle = "";
          if (isItalic) fontStyle += "italic ";
          if (isBold) fontStyle += "bold ";
          ctx.font = `${fontStyle}${fontSize}px ${fontFamily}, sans-serif`;

          ctx.fillStyle = p.style === "Heading1" ? "#1f497d" : "#242424";
          ctx.textAlign = (p.alignment === "justify" ? "left" : p.alignment) || "left";

          let drawX = curX;
          if (p.alignment === "center") drawX = this.pageWidth / 2;
          else if (p.alignment === "right") drawX = this.pageWidth - this.marginX;

          ctx.fillText(r.text, drawX, curY);

          if (r.underline) {
            const metrics = ctx.measureText(r.text);
            ctx.beginPath();
            ctx.moveTo(drawX, curY + 2);
            ctx.lineTo(drawX + metrics.width, curY + 2);
            ctx.strokeStyle = ctx.fillStyle;
            ctx.lineWidth = 1;
            ctx.stroke();
          }

          const wordsInRun = r.text.trim().split(/\s+/).filter(Boolean).length;
          totalWords += wordsInRun;

          if (pIdx === this.state.cursor.paragraphIndex && rIdx === this.state.cursor.runIndex) {
            const textBefore = r.text;
            const metrics = ctx.measureText(textBefore);
            caretX = drawX + metrics.width;
            caretY = curY - fontSize + 2;
            caretH = fontSize + 2;
          }

          curX += ctx.measureText(r.text).width;
        });

        curY += (p.style === "Heading1" ? 36 : 22);
      });
    }

    // 4. Caret rendering
    if (this.caretVisible) {
      ctx.fillStyle = "#000000";
      ctx.fillRect(caretX, caretY, 2, caretH);
    }

    // 5. Update Status
    if (this.onStatusChange) {
      this.onStatusChange("Page 1 of 1", totalWords);
    }
  }
}
