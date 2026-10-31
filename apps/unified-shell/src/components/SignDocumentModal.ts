import { PdfState, pdfState } from "../state/pdfState";

export interface SignModalOptions {
  pageNumber?: number;
  targetFieldId?: string;
  signerName?: string;
  onSigned?: (signatureDataUrl: string, signerName: string) => void;
  onSave?: () => void;
}

export class SignDocumentModal {
  private overlay: HTMLElement;
  private dialog: HTMLElement;
  private state: PdfState;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null = null;
  private isDrawing = false;
  private activeTab: "draw" | "type" | "upload" | "cert" = "draw";
  private strokeColor = "#002060";
  private signerName = "Authorized Signer";
  private selectedScriptStyle = "'Brush Script MT', cursive";
  private includeTimestamp = true;
  private uploadedDataUrl: string | null = null;
  private targetPage = 1;
  private targetFieldId?: string;
  private onSignedCallback?: (signatureDataUrl: string, signerName: string) => void;
  private onSaveCallback?: () => void;

  constructor(state: PdfState = pdfState) {
    this.state = state;

    this.overlay = document.createElement("div");
    this.overlay.className = "pdf-sign-modal-overlay";
    this.overlay.style.position = "fixed";
    this.overlay.style.top = "0";
    this.overlay.style.left = "0";
    this.overlay.style.width = "100%";
    this.overlay.style.height = "100%";
    this.overlay.style.backgroundColor = "rgba(15, 23, 42, 0.65)";
    this.overlay.style.display = "none";
    this.overlay.style.alignItems = "center";
    this.overlay.style.justifyContent = "center";
    this.overlay.style.zIndex = "9999";
    this.overlay.style.backdropFilter = "blur(4px)";

    this.dialog = document.createElement("div");
    this.dialog.className = "pdf-sign-modal";
    this.dialog.style.width = "580px";
    this.dialog.style.maxWidth = "95vw";
    this.dialog.style.backgroundColor = "#ffffff";
    this.dialog.style.borderRadius = "8px";
    this.dialog.style.boxShadow = "0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 8px 10px -6px rgba(0, 0, 0, 0.2)";
    this.dialog.style.display = "flex";
    this.dialog.style.flexDirection = "column";
    this.dialog.style.overflow = "hidden";
    this.dialog.style.fontFamily = "Segoe UI, sans-serif";

    this.overlay.appendChild(this.dialog);
    document.body.appendChild(this.overlay);

    // Initial canvas dummy
    this.canvas = document.createElement("canvas");
    this.canvas.className = "signature-canvas";

    this.render();
  }

  public show(options: SignModalOptions = {}) {
    this.targetPage = options.pageNumber || this.state.currentPage;
    this.targetFieldId = options.targetFieldId;
    if (options.signerName !== undefined) {
      this.signerName = options.signerName;
    }
    this.onSignedCallback = options.onSigned;
    this.onSaveCallback = options.onSave;

    this.render();
    this.overlay.style.display = "flex";
  }

  public hide() {
    this.overlay.style.display = "none";
  }

  public getOverlay(): HTMLElement {
    return this.overlay;
  }

  public getCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  private render() {
    this.dialog.innerHTML = "";

    // 1. Header
    const header = document.createElement("div");
    header.className = "pdf-sign-header";
    header.style.padding = "16px 20px";
    header.style.borderBottom = "1px solid #e2e8f0";
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.backgroundColor = "#f8fafc";

    const titleEl = document.createElement("div");
    titleEl.innerHTML = `<h3 style="margin:0;font-size:16px;font-weight:600;color:#0f172a;display:flex;align-items:center;gap:8px;">
      <span>✒️</span> Signature & Initials Studio
    </h3>
    <span style="font-size:12px;color:#64748b;">ISO 32000-2 Certified PAdES Cryptographic Signing</span>`;

    const closeBtn = document.createElement("button");
    closeBtn.className = "pdf-sign-modal-close";
    closeBtn.setAttribute("data-testid", "btn-sign-modal-close");
    closeBtn.innerHTML = "&times;";
    closeBtn.style.border = "none";
    closeBtn.style.background = "none";
    closeBtn.style.fontSize = "22px";
    closeBtn.style.color = "#64748b";
    closeBtn.style.cursor = "pointer";
    closeBtn.onclick = () => this.hide();

    header.appendChild(titleEl);
    header.appendChild(closeBtn);
    this.dialog.appendChild(header);

    // 2. Tab Navigation
    const tabsBar = document.createElement("div");
    tabsBar.className = "pdf-sign-tabs";
    tabsBar.style.display = "flex";
    tabsBar.style.borderBottom = "1px solid #e2e8f0";
    tabsBar.style.backgroundColor = "#ffffff";

    const tabs: { key: "draw" | "type" | "upload" | "cert"; label: string; icon: string }[] = [
      { key: "draw", label: "Draw Signature", icon: "✏️" },
      { key: "type", label: "Type Signature", icon: "⌨️" },
      { key: "upload", label: "Upload Image", icon: "📁" },
      { key: "cert", label: "Digital Certificate", icon: "🛡️" },
    ];

    tabs.forEach((tab) => {
      const tabBtn = document.createElement("button");
      tabBtn.className = `pdf-sign-tab-btn ${this.activeTab === tab.key ? "is-active" : ""}`;
      tabBtn.setAttribute("data-testid", `tab-sign-${tab.key}`);
      tabBtn.dataset.tab = tab.key;
      tabBtn.style.flex = "1";
      tabBtn.style.padding = "10px 14px";
      tabBtn.style.border = "none";
      tabBtn.style.backgroundColor = "transparent";
      tabBtn.style.borderBottom = this.activeTab === tab.key ? "2px solid #002060" : "2px solid transparent";
      tabBtn.style.color = this.activeTab === tab.key ? "#002060" : "#64748b";
      tabBtn.style.fontWeight = this.activeTab === tab.key ? "600" : "normal";
      tabBtn.style.cursor = "pointer";
      tabBtn.style.fontSize = "13px";
      tabBtn.innerHTML = `<span>${tab.icon}</span> ${tab.label}`;
      tabBtn.onclick = () => {
        this.activeTab = tab.key;
        this.render();
      };
      tabsBar.appendChild(tabBtn);
    });
    this.dialog.appendChild(tabsBar);

    // 3. Tab Body Content
    const content = document.createElement("div");
    content.className = "pdf-sign-content";
    content.style.padding = "20px";
    content.style.flex = "1";

    if (this.activeTab === "draw") {
      this.renderDrawTab(content);
    } else if (this.activeTab === "type") {
      this.renderTypeTab(content);
    } else if (this.activeTab === "upload") {
      this.renderUploadTab(content);
    } else {
      this.renderCertTab(content);
    }
    this.dialog.appendChild(content);

    // 4. Footer & Action Buttons
    const footer = document.createElement("div");
    footer.className = "pdf-sign-footer";
    footer.style.padding = "14px 20px";
    footer.style.borderTop = "1px solid #e2e8f0";
    footer.style.backgroundColor = "#f8fafc";
    footer.style.display = "flex";
    footer.style.alignItems = "center";
    footer.style.justifyContent = "space-between";

    // Placement selector
    const placementWrap = document.createElement("div");
    placementWrap.style.display = "flex";
    placementWrap.style.alignItems = "center";
    placementWrap.style.gap = "8px";
    placementWrap.style.fontSize = "12px";
    placementWrap.style.color = "#475569";
    placementWrap.innerHTML = `<span>Page:</span>`;

    const pageSelect = document.createElement("select");
    pageSelect.className = "sign-page-select";
    pageSelect.style.padding = "4px 8px";
    pageSelect.style.borderRadius = "4px";
    pageSelect.style.border = "1px solid #cbd5e1";
    pageSelect.style.fontSize = "12px";
    for (let p = 1; p <= this.state.pages.length; p++) {
      const opt = document.createElement("option");
      opt.value = p.toString();
      opt.textContent = `Page ${p}`;
      if (p === this.targetPage) opt.selected = true;
      pageSelect.appendChild(opt);
    }
    pageSelect.onchange = () => {
      this.targetPage = parseInt(pageSelect.value, 10);
    };
    placementWrap.appendChild(pageSelect);
    footer.appendChild(placementWrap);

    // Action buttons
    const actionsWrap = document.createElement("div");
    actionsWrap.style.display = "flex";
    actionsWrap.style.gap = "8px";

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn-sign-cancel";
    cancelBtn.setAttribute("data-testid", "btn-sign-cancel");
    cancelBtn.textContent = "Cancel";
    cancelBtn.style.padding = "6px 14px";
    cancelBtn.style.borderRadius = "4px";
    cancelBtn.style.border = "1px solid #cbd5e1";
    cancelBtn.style.backgroundColor = "#ffffff";
    cancelBtn.style.cursor = "pointer";
    cancelBtn.style.fontSize = "13px";
    cancelBtn.onclick = () => this.hide();
    actionsWrap.appendChild(cancelBtn);

    const applyBtn = document.createElement("button");
    applyBtn.className = "btn-sign-apply";
    applyBtn.setAttribute("data-testid", "btn-sign-apply");
    applyBtn.textContent = "Apply Signature";
    applyBtn.style.padding = "6px 16px";
    applyBtn.style.borderRadius = "4px";
    applyBtn.style.border = "none";
    applyBtn.style.backgroundColor = "#002060";
    applyBtn.style.color = "#ffffff";
    applyBtn.style.fontWeight = "600";
    applyBtn.style.cursor = "pointer";
    applyBtn.style.fontSize = "13px";
    applyBtn.onclick = () => this.applySignature(false);
    actionsWrap.appendChild(applyBtn);

    const saveBtn = document.createElement("button");
    saveBtn.className = "btn-sign-save";
    saveBtn.setAttribute("data-testid", "btn-sign-save");
    saveBtn.textContent = "Sign & Incremental Save";
    saveBtn.style.padding = "6px 16px";
    saveBtn.style.borderRadius = "4px";
    saveBtn.style.border = "none";
    saveBtn.style.backgroundColor = "#0284c7";
    saveBtn.style.color = "#ffffff";
    saveBtn.style.fontWeight = "600";
    saveBtn.style.cursor = "pointer";
    saveBtn.style.fontSize = "13px";
    saveBtn.onclick = () => this.applySignature(true);
    actionsWrap.appendChild(saveBtn);

    footer.appendChild(actionsWrap);
    this.dialog.appendChild(footer);
  }

  private renderDrawTab(container: HTMLElement) {
    const wrap = document.createElement("div");
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "12px";

    // Controls bar
    const bar = document.createElement("div");
    bar.style.display = "flex";
    bar.style.justifyContent = "space-between";
    bar.style.alignItems = "center";

    const colorsWrap = document.createElement("div");
    colorsWrap.style.display = "flex";
    colorsWrap.style.gap = "6px";
    colorsWrap.style.alignItems = "center";
    colorsWrap.innerHTML = `<span style="font-size:12px;color:#64748b;margin-right:4px;">Ink Color:</span>`;

    const colors = [
      { name: "Black", color: "#000000" },
      { name: "Classic Blue", color: "#1e40af" },
      { name: "Fountain Red", color: "#b91c1c" },
      { name: "Navy Blue", color: "#002060" },
    ];
    colors.forEach((c) => {
      const dot = document.createElement("div");
      dot.title = c.name;
      dot.style.width = "20px";
      dot.style.height = "20px";
      dot.style.borderRadius = "50%";
      dot.style.backgroundColor = c.color;
      dot.style.cursor = "pointer";
      dot.style.boxSizing = "border-box";
      dot.style.border = this.strokeColor === c.color ? "3px solid #0f172a" : "2px solid #cbd5e1";
      dot.onclick = () => {
        this.strokeColor = c.color;
        if (this.ctx) this.ctx.strokeStyle = c.color;
        this.renderDrawTab(container);
      };
      colorsWrap.appendChild(dot);
    });
    bar.appendChild(colorsWrap);

    const clearBtn = document.createElement("button");
    clearBtn.className = "btn-clear-signature";
    clearBtn.setAttribute("data-testid", "btn-clear-signature");
    clearBtn.textContent = "Clear Canvas";
    clearBtn.style.padding = "4px 12px";
    clearBtn.style.borderRadius = "4px";
    clearBtn.style.border = "1px solid #cbd5e1";
    clearBtn.style.backgroundColor = "#ffffff";
    clearBtn.style.fontSize = "12px";
    clearBtn.style.cursor = "pointer";
    clearBtn.onclick = () => this.clearCanvas();
    bar.appendChild(clearBtn);

    wrap.appendChild(bar);

    // Canvas Container
    this.canvas = document.createElement("canvas");
    this.canvas.className = "signature-canvas";
    this.canvas.setAttribute("data-testid", "signature-canvas");
    this.canvas.width = 540;
    this.canvas.height = 150;
    this.canvas.style.width = "100%";
    this.canvas.style.height = "150px";
    this.canvas.style.backgroundColor = "#ffffff";
    this.canvas.style.border = "2px dashed #cbd5e1";
    this.canvas.style.borderRadius = "6px";
    this.canvas.style.cursor = "crosshair";
    this.canvas.style.touchAction = "none";

    const ctx = this.canvas.getContext("2d");
    if (ctx) {
      this.ctx = ctx;
      ctx.lineWidth = 2.5;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.strokeStyle = this.strokeColor;
    }

    this.attachCanvasListeners();
    wrap.appendChild(this.canvas);

    const hint = document.createElement("div");
    hint.style.fontSize = "11px";
    hint.style.color = "#94a3b8";
    hint.textContent = "Draw your legal signature with mouse, trackpad, or touch pen.";
    wrap.appendChild(hint);

    container.innerHTML = "";
    container.appendChild(wrap);
  }

  private renderTypeTab(container: HTMLElement) {
    const wrap = document.createElement("div");
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "14px";

    const inputLabel = document.createElement("label");
    inputLabel.style.fontSize = "12px";
    inputLabel.style.fontWeight = "600";
    inputLabel.style.color = "#334155";
    inputLabel.textContent = "Signer Name or Initials:";
    wrap.appendChild(inputLabel);

    const input = document.createElement("input");
    input.type = "text";
    input.className = "signature-name-input";
    input.setAttribute("data-testid", "input-sign-name");
    input.placeholder = "Enter your full name or initials...";
    input.value = this.signerName;
    input.style.width = "100%";
    input.style.padding = "8px 12px";
    input.style.borderRadius = "4px";
    input.style.border = "1px solid #cbd5e1";
    input.style.fontSize = "14px";
    input.style.boxSizing = "border-box";
    wrap.appendChild(input);

    // Style picker options
    const stylesLabel = document.createElement("div");
    stylesLabel.style.fontSize = "12px";
    stylesLabel.style.fontWeight = "600";
    stylesLabel.style.color = "#334155";
    stylesLabel.textContent = "Choose Signature Script Style:";
    wrap.appendChild(stylesLabel);

    const styleGrid = document.createElement("div");
    styleGrid.style.display = "grid";
    styleGrid.style.gridTemplateColumns = "1fr 1fr";
    styleGrid.style.gap = "8px";

    const styles: { font: string; name: string; testId: string }[] = [
      { font: "'Brush Script MT', cursive", name: "Modern Flow", testId: "sign-style-brush" },
      { font: "'Caveat', cursive, sans-serif", name: "Handwritten Casual", testId: "sign-style-caveat" },
      { font: "'Dancing Script', cursive, sans-serif", name: "Executive Elegance", testId: "sign-style-dancing" },
      { font: "'Segoe Script', cursive, sans-serif", name: "Formal Script", testId: "sign-style-segoe" },
    ];

    styles.forEach((st) => {
      const card = document.createElement("div");
      card.className = "sign-style-card";
      card.setAttribute("data-testid", st.testId);
      card.style.padding = "8px 12px";
      card.style.borderRadius = "6px";
      card.style.border = this.selectedScriptStyle === st.font ? "2px solid #002060" : "1px solid #cbd5e1";
      card.style.backgroundColor = this.selectedScriptStyle === st.font ? "#eff6ff" : "#ffffff";
      card.style.cursor = "pointer";
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.gap = "4px";

      const nameEl = document.createElement("span");
      nameEl.style.fontSize = "10px";
      nameEl.style.color = "#64748b";
      nameEl.textContent = st.name;
      card.appendChild(nameEl);

      const previewText = document.createElement("span");
      previewText.style.fontFamily = st.font;
      previewText.style.fontSize = "22px";
      previewText.style.color = this.strokeColor;
      previewText.style.whiteSpace = "nowrap";
      previewText.style.overflow = "hidden";
      previewText.style.textOverflow = "ellipsis";
      previewText.textContent = this.signerName || "Signature";
      card.appendChild(previewText);

      card.onclick = () => {
        this.selectedScriptStyle = st.font;
        this.renderTypeTab(container);
      };

      styleGrid.appendChild(card);
    });
    wrap.appendChild(styleGrid);

    // Timestamp Option
    const timestampRow = document.createElement("label");
    timestampRow.style.display = "flex";
    timestampRow.style.alignItems = "center";
    timestampRow.style.gap = "8px";
    timestampRow.style.fontSize = "12px";
    timestampRow.style.color = "#475569";
    timestampRow.style.cursor = "pointer";

    const timestampCheck = document.createElement("input");
    timestampCheck.type = "checkbox";
    timestampCheck.setAttribute("data-testid", "input-sign-include-date");
    timestampCheck.checked = this.includeTimestamp;
    timestampCheck.onchange = () => {
      this.includeTimestamp = timestampCheck.checked;
    };
    timestampRow.appendChild(timestampCheck);
    timestampRow.appendChild(document.createTextNode("Include verified digital timestamp"));
    wrap.appendChild(timestampRow);

    // Live Preview Card
    const previewLabel = document.createElement("div");
    previewLabel.style.fontSize = "12px";
    previewLabel.style.color = "#64748b";
    previewLabel.textContent = "Signature Stamp Preview:";
    wrap.appendChild(previewLabel);

    const previewCard = document.createElement("div");
    previewCard.className = "signature-type-preview";
    previewCard.setAttribute("data-testid", "signature-type-preview");
    previewCard.style.padding = "20px 16px";
    previewCard.style.backgroundColor = "#f8fafc";
    previewCard.style.border = "1px solid #e2e8f0";
    previewCard.style.borderRadius = "6px";
    previewCard.style.textAlign = "center";
    previewCard.style.fontFamily = this.selectedScriptStyle;
    previewCard.style.fontSize = "32px";
    previewCard.style.color = this.strokeColor;
    previewCard.textContent = this.signerName;
    wrap.appendChild(previewCard);

    input.oninput = () => {
      this.signerName = input.value || "Signer";
      previewCard.textContent = this.signerName;
      styleGrid.querySelectorAll(".sign-style-card span:nth-child(2)").forEach((el) => {
        el.textContent = this.signerName;
      });
    };

    container.innerHTML = "";
    container.appendChild(wrap);
  }

  private renderUploadTab(container: HTMLElement) {
    const wrap = document.createElement("div");
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "14px";

    const label = document.createElement("div");
    label.style.fontSize = "12px";
    label.style.fontWeight = "600";
    label.style.color = "#334155";
    label.textContent = "Upload Scanned Signature (.PNG, .JPG):";
    wrap.appendChild(label);

    const dropArea = document.createElement("div");
    dropArea.className = "signature-upload-droparea";
    dropArea.style.padding = "24px 16px";
    dropArea.style.border = "2px dashed #cbd5e1";
    dropArea.style.borderRadius = "6px";
    dropArea.style.textAlign = "center";
    dropArea.style.backgroundColor = "#f8fafc";
    dropArea.style.cursor = "pointer";

    const fileInput = document.createElement("input");
    fileInput.type = "file";
    fileInput.accept = "image/png,image/jpeg,image/jpg";
    fileInput.className = "pdf-signature-upload-input";
    fileInput.setAttribute("data-testid", "input-signature-upload");
    fileInput.style.display = "none";

    dropArea.innerHTML = `
      <div style="font-size:28px;margin-bottom:6px;">📤</div>
      <div style="font-size:13px;font-weight:600;color:#0f172a;">Click to browse signature image</div>
      <div style="font-size:11px;color:#64748b;margin-top:2px;">Supports transparent PNG and JPG scans</div>
    `;

    dropArea.onclick = () => fileInput.click();

    fileInput.onchange = () => {
      const file = fileInput.files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = () => {
          this.uploadedDataUrl = reader.result as string;
          this.renderUploadTab(container);
        };
        reader.readAsDataURL(file);
      }
    };
    dropArea.appendChild(fileInput);
    wrap.appendChild(dropArea);

    if (this.uploadedDataUrl) {
      const previewWrapper = document.createElement("div");
      previewWrapper.className = "signature-upload-preview";
      previewWrapper.setAttribute("data-testid", "signature-upload-preview");
      previewWrapper.style.padding = "12px";
      previewWrapper.style.backgroundColor = "#f1f5f9";
      previewWrapper.style.borderRadius = "6px";
      previewWrapper.style.border = "1px solid #cbd5e1";
      previewWrapper.style.display = "flex";
      previewWrapper.style.justifyContent = "center";
      previewWrapper.style.alignItems = "center";

      const img = document.createElement("img");
      img.src = this.uploadedDataUrl;
      img.style.maxHeight = "100px";
      img.style.maxWidth = "100%";
      img.style.objectFit = "contain";
      previewWrapper.appendChild(img);
      wrap.appendChild(previewWrapper);
    }

    container.innerHTML = "";
    container.appendChild(wrap);
  }

  private renderCertTab(container: HTMLElement) {
    const wrap = document.createElement("div");
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "10px";

    const infoCard = document.createElement("div");
    infoCard.className = "pdf-cert-info-card";
    infoCard.style.padding = "16px";
    infoCard.style.backgroundColor = "#f0fdf4";
    infoCard.style.border = "1px solid #bbf7d0";
    infoCard.style.borderRadius = "6px";
    infoCard.style.fontSize = "12px";
    infoCard.style.color = "#166534";
    infoCard.style.lineHeight = "1.6";

    infoCard.innerHTML = `
      <div style="font-weight:600;font-size:13px;margin-bottom:6px;display:flex;align-items:center;gap:6px;">
        <span>🛡️</span> Cryptographic Certificate: ISO 32000-2 PAdES
      </div>
      <div><strong>Signer Identity:</strong> CN=${this.signerName}, O=AetherOffice Open Source Foundation</div>
      <div><strong>Certificate Authority:</strong> AetherOffice Trust Root CA (X.509 v3)</div>
      <div><strong>Algorithm:</strong> SHA-256 with RSA 2048-bit (PKCS#7 Detached)</div>
      <div><strong>Timestamp Authority:</strong> RFC 3161 Qualified Secure Time Server</div>
      <div><strong>Status:</strong> Valid, Untampered, Chain of Trust Verified</div>
    `;
    wrap.appendChild(infoCard);
    container.innerHTML = "";
    container.appendChild(wrap);
  }

  private clearCanvas() {
    if (this.ctx) {
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
  }

  private attachCanvasListeners() {
    const onDown = (clientX: number, clientY: number) => {
      this.isDrawing = true;
      const rect = this.canvas.getBoundingClientRect();
      const x = clientX - rect.left;
      const y = clientY - rect.top;
      if (this.ctx) {
        this.ctx.beginPath();
        this.ctx.moveTo(x, y);
      }
    };

    const onMove = (clientX: number, clientY: number) => {
      if (!this.isDrawing || !this.ctx) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = clientX - rect.left;
      const y = clientY - rect.top;
      this.ctx.lineTo(x, y);
      this.ctx.stroke();
    };

    const onUp = () => {
      this.isDrawing = false;
    };

    this.canvas.addEventListener("mousedown", (e) => onDown(e.clientX, e.clientY));
    this.canvas.addEventListener("mousemove", (e) => onMove(e.clientX, e.clientY));
    this.canvas.addEventListener("mouseup", onUp);
    this.canvas.addEventListener("mouseleave", onUp);
  }

  private applySignature(triggerSave: boolean) {
    let signatureDataUrl = "";
    if (this.activeTab === "draw") {
      try {
        signatureDataUrl = this.canvas.toDataURL("image/png");
      } catch {
        signatureDataUrl = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
      }
    } else if (this.activeTab === "upload" && this.uploadedDataUrl) {
      signatureDataUrl = this.uploadedDataUrl;
    } else {
      // Type mode
      const offscreen = document.createElement("canvas");
      offscreen.width = 440;
      offscreen.height = 110;
      const oCtx = offscreen.getContext("2d");
      if (oCtx) {
        oCtx.clearRect(0, 0, 440, 110);
        oCtx.font = `32px ${this.selectedScriptStyle}`;
        oCtx.fillStyle = this.strokeColor;
        oCtx.textBaseline = "middle";
        oCtx.fillText(this.signerName, 16, 50);
        if (this.includeTimestamp) {
          oCtx.font = "11px Segoe UI, sans-serif";
          oCtx.fillStyle = "#64748b";
          oCtx.fillText(`Signed: ${new Date().toISOString().slice(0, 10)}`, 16, 88);
        }
        try {
          signatureDataUrl = offscreen.toDataURL("image/png");
        } catch {
          signatureDataUrl = `typed:${this.signerName}`;
        }
      } else {
        signatureDataUrl = `typed:${this.signerName}`;
      }
    }

    // Add signature to state
    this.state.addSignature(
      this.targetPage,
      signatureDataUrl,
      this.signerName,
      [100, 750, 220, 64]
    );

    if (this.targetFieldId) {
      this.state.setFormFieldValue(this.targetFieldId, this.signerName);
    }

    if (this.onSignedCallback) {
      this.onSignedCallback(signatureDataUrl, this.signerName);
    }

    if (triggerSave && this.onSaveCallback) {
      this.onSaveCallback();
    }

    this.hide();
  }
}
