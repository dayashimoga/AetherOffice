/**
 * DiffPreviewModal Component.
 * Displays before-and-after visual diffs comparing original document content
 * with AI-generated text or formula proposals before committing changes to the active document.
 */

export interface DiffModalOptions {
  title?: string;
  targetDescription?: string;
  originalText: string;
  proposedText: string;
  onApply: (appliedText: string) => void;
  onCancel?: () => void;
}

export class DiffPreviewModal {
  private backdropEl: HTMLElement;
  private modalEl: HTMLElement;
  private titleEl: HTMLElement;
  private targetDescEl: HTMLElement;
  private originalViewEl: HTMLElement;
  private proposedViewEl: HTMLElement;
  private applyBtn: HTMLButtonElement;
  private cancelBtn: HTMLButtonElement;

  private currentOptions: DiffModalOptions | null = null;
  private visible = false;

  constructor(parent: HTMLElement = document.body) {
    this.backdropEl = document.createElement("div");
    this.backdropEl.className = "diff-modal-backdrop";
    this.backdropEl.style.display = "none";
    this.backdropEl.style.position = "fixed";
    this.backdropEl.style.top = "0";
    this.backdropEl.style.left = "0";
    this.backdropEl.style.width = "100vw";
    this.backdropEl.style.height = "100vh";
    this.backdropEl.style.backgroundColor = "rgba(15, 23, 42, 0.45)";
    this.backdropEl.style.backdropFilter = "blur(2px)";
    this.backdropEl.style.zIndex = "10000";
    this.backdropEl.style.alignItems = "center";
    this.backdropEl.style.justifyContent = "center";

    this.modalEl = document.createElement("div");
    this.modalEl.className = "diff-modal-container";
    this.modalEl.style.width = "640px";
    this.modalEl.style.maxWidth = "90vw";
    this.modalEl.style.maxHeight = "85vh";
    this.modalEl.style.backgroundColor = "#ffffff";
    this.modalEl.style.borderRadius = "12px";
    this.modalEl.style.boxShadow = "0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)";
    this.modalEl.style.display = "flex";
    this.modalEl.style.flexDirection = "column";
    this.modalEl.style.overflow = "hidden";
    this.modalEl.style.fontFamily = "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

    // 1. Header
    const header = document.createElement("div");
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.padding = "16px 20px";
    header.style.borderBottom = "1px solid #e2e8f0";
    header.style.backgroundColor = "#f8fafc";

    const titleContainer = document.createElement("div");
    this.titleEl = document.createElement("h3");
    this.titleEl.className = "diff-modal-title";
    this.titleEl.textContent = "Review Proposed AI Changes";
    this.titleEl.style.margin = "0";
    this.titleEl.style.fontSize = "16px";
    this.titleEl.style.fontWeight = "700";
    this.titleEl.style.color = "#0f172a";

    this.targetDescEl = document.createElement("p");
    this.targetDescEl.className = "diff-modal-target-desc";
    this.targetDescEl.textContent = "Target: Active selection";
    this.targetDescEl.style.margin = "4px 0 0 0";
    this.targetDescEl.style.fontSize = "12px";
    this.targetDescEl.style.color = "#64748b";

    titleContainer.appendChild(this.titleEl);
    titleContainer.appendChild(this.targetDescEl);

    const closeBtn = document.createElement("button");
    closeBtn.className = "diff-modal-btn-close";
    closeBtn.textContent = "✕";
    closeBtn.style.border = "none";
    closeBtn.style.background = "transparent";
    closeBtn.style.fontSize = "16px";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.color = "#64748b";
    closeBtn.onclick = () => this.close();

    header.appendChild(titleContainer);
    header.appendChild(closeBtn);
    this.modalEl.appendChild(header);

    // 2. Diff Body (Side by Side comparison)
    const bodyContainer = document.createElement("div");
    bodyContainer.className = "diff-modal-body";
    bodyContainer.style.display = "grid";
    bodyContainer.style.gridTemplateColumns = "1fr 1fr";
    bodyContainer.style.gap = "16px";
    bodyContainer.style.padding = "20px";
    bodyContainer.style.overflowY = "auto";
    bodyContainer.style.flex = "1";

    // Original Pane
    const origBox = document.createElement("div");
    origBox.style.display = "flex";
    origBox.style.flexDirection = "column";
    origBox.style.gap = "6px";

    const origLabel = document.createElement("span");
    origLabel.textContent = "Original Content";
    origLabel.style.fontSize = "11px";
    origLabel.style.fontWeight = "700";
    origLabel.style.textTransform = "uppercase";
    origLabel.style.letterSpacing = "0.05em";
    origLabel.style.color = "#991b1b";

    this.originalViewEl = document.createElement("div");
    this.originalViewEl.className = "diff-view-original";
    this.originalViewEl.style.flex = "1";
    this.originalViewEl.style.padding = "12px";
    this.originalViewEl.style.backgroundColor = "#fee2e2";
    this.originalViewEl.style.border = "1px solid #fecaca";
    this.originalViewEl.style.borderRadius = "8px";
    this.originalViewEl.style.color = "#991b1b";
    this.originalViewEl.style.fontSize = "13px";
    this.originalViewEl.style.lineHeight = "1.5";
    this.originalViewEl.style.whiteSpace = "pre-wrap";
    this.originalViewEl.style.wordBreak = "break-word";

    origBox.appendChild(origLabel);
    origBox.appendChild(this.originalViewEl);

    // Proposed Pane
    const propBox = document.createElement("div");
    propBox.style.display = "flex";
    propBox.style.flexDirection = "column";
    propBox.style.gap = "6px";

    const propLabel = document.createElement("span");
    propLabel.textContent = "Proposed Replacement";
    propLabel.style.fontSize = "11px";
    propLabel.style.fontWeight = "700";
    propLabel.style.textTransform = "uppercase";
    propLabel.style.letterSpacing = "0.05em";
    propLabel.style.color = "#166534";

    this.proposedViewEl = document.createElement("div");
    this.proposedViewEl.className = "diff-view-proposed";
    this.proposedViewEl.style.flex = "1";
    this.proposedViewEl.style.padding = "12px";
    this.proposedViewEl.style.backgroundColor = "#dcfce7";
    this.proposedViewEl.style.border = "1px solid #bbf7d0";
    this.proposedViewEl.style.borderRadius = "8px";
    this.proposedViewEl.style.color = "#166534";
    this.proposedViewEl.style.fontSize = "13px";
    this.proposedViewEl.style.lineHeight = "1.5";
    this.proposedViewEl.style.whiteSpace = "pre-wrap";
    this.proposedViewEl.style.wordBreak = "break-word";

    propBox.appendChild(propLabel);
    propBox.appendChild(this.proposedViewEl);

    bodyContainer.appendChild(origBox);
    bodyContainer.appendChild(propBox);
    this.modalEl.appendChild(bodyContainer);

    // 3. Footer Actions
    const footer = document.createElement("div");
    footer.style.display = "flex";
    footer.style.justifyContent = "flex-end";
    footer.style.gap = "10px";
    footer.style.padding = "14px 20px";
    footer.style.borderTop = "1px solid #e2e8f0";
    footer.style.backgroundColor = "#f8fafc";

    this.cancelBtn = document.createElement("button");
    this.cancelBtn.className = "diff-modal-btn-cancel";
    this.cancelBtn.textContent = "Cancel";
    this.cancelBtn.style.padding = "8px 16px";
    this.cancelBtn.style.borderRadius = "6px";
    this.cancelBtn.style.border = "1px solid #cbd5e1";
    this.cancelBtn.style.backgroundColor = "#ffffff";
    this.cancelBtn.style.color = "#334155";
    this.cancelBtn.style.fontWeight = "500";
    this.cancelBtn.style.fontSize = "13px";
    this.cancelBtn.style.cursor = "pointer";
    this.cancelBtn.onclick = () => this.close();

    this.applyBtn = document.createElement("button");
    this.applyBtn.className = "diff-modal-btn-apply";
    this.applyBtn.textContent = "Accept & Apply Changes";
    this.applyBtn.style.padding = "8px 18px";
    this.applyBtn.style.borderRadius = "6px";
    this.applyBtn.style.border = "none";
    this.applyBtn.style.backgroundColor = "#16a34a";
    this.applyBtn.style.color = "#ffffff";
    this.applyBtn.style.fontWeight = "600";
    this.applyBtn.style.fontSize = "13px";
    this.applyBtn.style.cursor = "pointer";
    this.applyBtn.onclick = () => this.apply();

    footer.appendChild(this.cancelBtn);
    footer.appendChild(this.applyBtn);
    this.modalEl.appendChild(footer);

    this.backdropEl.appendChild(this.modalEl);
    parent.appendChild(this.backdropEl);
  }

  public open(options: DiffModalOptions): void {
    this.currentOptions = options;
    if (options.title) {
      this.titleEl.textContent = options.title;
    } else {
      this.titleEl.textContent = "Review Proposed AI Changes";
    }

    if (options.targetDescription) {
      this.targetDescEl.textContent = `Target: ${options.targetDescription}`;
      this.targetDescEl.style.display = "block";
    } else {
      this.targetDescEl.style.display = "none";
    }

    this.originalViewEl.textContent = options.originalText;
    this.proposedViewEl.textContent = options.proposedText;

    this.visible = true;
    this.backdropEl.style.display = "flex";
  }

  public close(): void {
    if (this.currentOptions?.onCancel) {
      this.currentOptions.onCancel();
    }
    this.visible = false;
    this.backdropEl.style.display = "none";
    this.currentOptions = null;
  }

  public apply(): void {
    if (this.currentOptions) {
      this.currentOptions.onApply(this.currentOptions.proposedText);
    }
    this.visible = false;
    this.backdropEl.style.display = "none";
    this.currentOptions = null;
  }

  public isOpen(): boolean {
    return this.visible;
  }
}

