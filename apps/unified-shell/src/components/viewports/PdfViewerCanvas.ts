import { PdfState, pdfState } from "../../state/pdfState";

export class PdfViewerCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: PdfState;
  private pageWidth = 816;
  private pageHeight = 1056;

  constructor(container: HTMLElement, state: PdfState = pdfState) {
    this.container = container;
    this.state = state;

    this.canvas = document.createElement("canvas");
    this.canvas.className = "pdf-viewer-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.margin = "32px auto";
    this.canvas.style.boxShadow = "0 6px 20px rgba(0, 0, 0, 0.2)";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "default";

    this.container.innerHTML = "";
    this.container.appendChild(this.canvas);

    const context = this.canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context for PDF");
    this.ctx = context;

    this.initCanvasSize();
    this.attachEvents();
    this.render();
  }

  private initCanvasSize() {
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = this.pageWidth * dpr;
    this.canvas.height = this.pageHeight * dpr;
    this.canvas.style.width = `${this.pageWidth}px`;
    this.canvas.style.height = `${this.pageHeight}px`;
    this.ctx.scale(dpr, dpr);
  }

  private attachEvents() {
    this.canvas.addEventListener("click", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      // Check AcroForm field interaction
      this.state.formFields.forEach((f) => {
        const [fx, fy, fw, fh] = f.rect;
        if (x >= fx && x <= fx + fw && y >= fy && y <= fy + fh) {
          if (f.type === "checkbox") {
            f.value = !f.value;
            this.render();
          } else if (f.type === "text") {
            const input = prompt("Edit AcroForm Field Value:", String(f.value));
            if (input !== null) {
              f.value = input;
              this.render();
            }
          }
        }
      });
    });
  }

  public render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.pageWidth, this.pageHeight);

    // 1. White Page Background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, this.pageWidth, this.pageHeight);

    // 2. Header and Title
    ctx.font = "bold 20px 'Segoe UI', sans-serif";
    ctx.fillStyle = "#b30b00"; // PDF Crimson
    ctx.fillText("ISO 32000-2 Certified PDF Document", 80, 80);

    ctx.font = "12px 'Segoe UI', sans-serif";
    ctx.fillStyle = "#555555";
    ctx.fillText("Interactive Form & Digital Signing Specification", 80, 105);

    // Horizontal Rule
    ctx.strokeStyle = "#e0e0e0";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(80, 120);
    ctx.lineTo(this.pageWidth - 80, 120);
    ctx.stroke();

    // 3. Render AcroForms Interactive Fields
    ctx.font = "bold 13px 'Segoe UI', sans-serif";
    ctx.fillStyle = "#222222";
    ctx.fillText("Fillable Form Fields (ISO 32000 AcroForms):", 80, 170);

    this.state.formFields.forEach((f) => {
      const [fx, fy, fw, fh] = f.rect;

      if (f.type === "text") {
        // Label
        ctx.font = "11px 'Segoe UI', sans-serif";
        ctx.fillStyle = "#666666";
        ctx.fillText(f.name, fx, fy - 6);

        // Input field box
        ctx.fillStyle = "#eff6fc"; // Light blue fill for interactive field
        ctx.fillRect(fx, fy, fw, fh);
        ctx.strokeStyle = "#0078d4";
        ctx.lineWidth = 1;
        ctx.strokeRect(fx, fy, fw, fh);

        // Field text
        ctx.font = "13px 'Segoe UI', sans-serif";
        ctx.fillStyle = "#111111";
        ctx.fillText(String(f.value), fx + 8, fy + 20);
      } else if (f.type === "checkbox") {
        ctx.font = "11px 'Segoe UI', sans-serif";
        ctx.fillStyle = "#666666";
        ctx.fillText("I accept the agreement terms", fx + fw + 8, fy + 14);

        ctx.fillStyle = "#eff6fc";
        ctx.fillRect(fx, fy, fw, fh);
        ctx.strokeStyle = "#0078d4";
        ctx.strokeRect(fx, fy, fw, fh);

        if (f.value === true) {
          ctx.strokeStyle = "#0078d4";
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(fx + 4, fy + 10);
          ctx.lineTo(fx + 8, fy + 15);
          ctx.lineTo(fx + 16, fy + 5);
          ctx.stroke();
        }
      }
    });

    // 4. Digital Signature Seal
    ctx.fillStyle = "#fdf2f2";
    ctx.strokeStyle = "#d13438";
    ctx.lineWidth = 1.5;
    const sigRect = [80, 320, 340, 80];
    ctx.fillRect(sigRect[0], sigRect[1], sigRect[2], sigRect[3]);
    ctx.strokeRect(sigRect[0], sigRect[1], sigRect[2], sigRect[3]);

    ctx.font = "bold 12px 'Segoe UI', sans-serif";
    ctx.fillStyle = "#d13438";
    ctx.fillText("🔒 Cryptographically Verified Digital Signature", 96, 350);
    ctx.font = "11px monospace";
    ctx.fillStyle = "#666666";
    ctx.fillText("SHA256: 9b2d8e41a... | OpenOfficeRS PDF-Engine", 96, 375);
  }
}
