import {
  PdfAnnotation,
  PdfPageModel,
  PdfSidebarTab,
  PdfState,
  PdfTextBlock,
  pdfState,
} from "../../state/pdfState";
import { SignDocumentModal } from "../../components/SignDocumentModal";

export interface PdfWorkspaceCallbacks {
  onDirty?: () => void;
  onPageChange?: (pageNumber: number) => void;
  onSave?: () => void;
}

export class PdfWorkspace {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private sidebarEl: HTMLElement;
  private sidebarHeaderEl: HTMLElement;
  private sidebarBodyEl: HTMLElement;
  private viewportEl: HTMLElement;
  private state: PdfState;
  private pageCanvases: Map<number, HTMLCanvasElement> = new Map();
  private callbacks: PdfWorkspaceCallbacks = {};

  // Interactive Tools & Modals
  private signModal: SignDocumentModal;
  private activePopover: HTMLElement | null = null;
  private isDrawingInk = false;
  private currentInkPoints: { x: number; y: number }[] = [];
  private activeInkPageNumber = 1;

  constructor(containerOrId: HTMLElement | string, state: PdfState = pdfState) {
    this.state = state;
    if (typeof containerOrId === "string") {
      let el = document.getElementById(containerOrId);
      if (!el) {
        el = document.createElement("div");
        el.id = containerOrId;
        document.body.appendChild(el);
      }
      this.container = el;
    } else {
      this.container = containerOrId;
    }

    this.rootEl = document.createElement("div");
    this.rootEl.className = "pdf-studio-workspace";
    this.rootEl.style.display = "flex";
    this.rootEl.style.width = "100%";
    this.rootEl.style.height = "100%";
    this.rootEl.style.backgroundColor = "#e2e8f0";
    this.rootEl.style.overflow = "hidden";
    this.rootEl.style.fontFamily = "Segoe UI, sans-serif";

    // 1. Left Sidebar (Thumbnails, Bookmarks, Annotations)
    this.sidebarEl = document.createElement("aside");
    this.sidebarEl.className = "pdf-sidebar";
    this.sidebarEl.setAttribute("data-testid", "pdf-sidebar");
    this.sidebarEl.style.width = "260px";
    this.sidebarEl.style.height = "100%";
    this.sidebarEl.style.backgroundColor = "#f8fafc";
    this.sidebarEl.style.borderRight = "1px solid #cbd5e1";
    this.sidebarEl.style.display = "flex";
    this.sidebarEl.style.flexDirection = "column";
    this.sidebarEl.style.flexShrink = "0";

    // Sidebar tab headers
    this.sidebarHeaderEl = document.createElement("div");
    this.sidebarHeaderEl.className = "pdf-sidebar-header";
    this.sidebarHeaderEl.style.display = "flex";
    this.sidebarHeaderEl.style.borderBottom = "1px solid #cbd5e1";
    this.sidebarHeaderEl.style.backgroundColor = "#f1f5f9";

    this.sidebarBodyEl = document.createElement("div");
    this.sidebarBodyEl.className = "pdf-sidebar-body";
    this.sidebarBodyEl.style.flex = "1";
    this.sidebarBodyEl.style.overflowY = "auto";
    this.sidebarBodyEl.style.padding = "8px";

    this.sidebarEl.appendChild(this.sidebarHeaderEl);
    this.sidebarEl.appendChild(this.sidebarBodyEl);
    this.rootEl.appendChild(this.sidebarEl);

    // 2. Central Multi-Page Scrollable Viewport
    this.viewportEl = document.createElement("section");
    this.viewportEl.className = "pdf-viewport";
    this.viewportEl.style.flex = "1";
    this.viewportEl.style.height = "100%";
    this.viewportEl.style.overflow = "auto";
    this.viewportEl.style.display = "flex";
    this.viewportEl.style.flexDirection = "column";
    this.viewportEl.style.alignItems = "center";
    this.viewportEl.style.padding = "24px";
    this.viewportEl.style.boxSizing = "border-box";
    this.rootEl.appendChild(this.viewportEl);

    this.container.innerHTML = "";
    this.container.appendChild(this.rootEl);

    // Digital Signature Modal
    this.signModal = new SignDocumentModal(this.state);

    this.renderSidebar();
    this.renderViewport();
  }

  public setCallbacks(cb: PdfWorkspaceCallbacks) {
    this.callbacks = cb;
  }

  public getState(): PdfState {
    return this.state;
  }

  public getRoot(): HTMLElement {
    return this.rootEl;
  }

  public getSidebar(): HTMLElement {
    return this.sidebarEl;
  }

  public getViewport(): HTMLElement {
    return this.viewportEl;
  }

  public getSignModal(): SignDocumentModal {
    return this.signModal;
  }

  public openSignModal(pageNumber = this.state.currentPage, targetFieldId?: string) {
    this.signModal.show({
      pageNumber,
      targetFieldId,
      onSigned: () => {
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      },
      onSave: () => {
        if (this.callbacks.onSave) this.callbacks.onSave();
      },
    });
  }

  public render() {
    this.renderSidebar();
    this.renderViewport();
  }

  // --- Sidebar Rendering ---

  private renderSidebar() {
    // 1. Render tab buttons
    this.sidebarHeaderEl.innerHTML = "";
    const tabs: { key: PdfSidebarTab; label: string; icon: string }[] = [
      { key: "thumbnails", label: "Pages", icon: "▦" },
      { key: "bookmarks", label: "Outline", icon: "📑" },
      { key: "annotations", label: "Notes", icon: "💬" },
    ];

    tabs.forEach((t) => {
      const btn = document.createElement("button");
      btn.className = `pdf-sidebar-tab-btn ${this.state.sidebarTab === t.key ? "is-active" : ""}`;
      btn.dataset.tab = t.key;
      btn.style.flex = "1";
      btn.style.padding = "8px 4px";
      btn.style.border = "none";
      btn.style.backgroundColor = this.state.sidebarTab === t.key ? "#ffffff" : "transparent";
      btn.style.borderBottom = this.state.sidebarTab === t.key ? "2px solid #b30b00" : "2px solid transparent";
      btn.style.cursor = "pointer";
      btn.style.fontSize = "12px";
      btn.style.fontWeight = this.state.sidebarTab === t.key ? "600" : "normal";
      btn.style.color = this.state.sidebarTab === t.key ? "#b30b00" : "#475569";
      btn.innerHTML = `<span>${t.icon}</span> ${t.label}`;
      btn.onclick = () => {
        this.state.setSidebarTab(t.key);
        this.renderSidebar();
      };
      this.sidebarHeaderEl.appendChild(btn);
    });

    // 2. Render tab body
    this.sidebarBodyEl.innerHTML = "";
    switch (this.state.sidebarTab) {
      case "thumbnails":
        this.renderThumbnailsTab();
        break;
      case "bookmarks":
        this.renderBookmarksTab();
        break;
      case "annotations":
        this.renderAnnotationsTab();
        break;
    }
  }

  private renderThumbnailsTab() {
    const list = document.createElement("div");
    list.className = "pdf-thumbnail-list";
    list.setAttribute("data-testid", "pdf-thumbnail-list");
    list.style.display = "flex";
    list.style.flexDirection = "column";
    list.style.gap = "12px";

    this.state.pages.forEach((page) => {
      const item = document.createElement("div");
      item.className = `pdf-thumbnail-card pdf-thumbnail-item ${page.pageNumber === this.state.currentPage ? "is-selected" : ""}`;
      item.setAttribute("data-testid", "pdf-thumb-card");
      item.dataset.pageNumber = page.pageNumber.toString();
      item.style.display = "flex";
      item.style.flexDirection = "column";
      item.style.alignItems = "center";
      item.style.padding = "8px";
      item.style.borderRadius = "6px";
      item.style.cursor = "pointer";
      item.style.backgroundColor = page.pageNumber === this.state.currentPage ? "#e2e8f0" : "#ffffff";
      item.style.border = page.pageNumber === this.state.currentPage ? "2px solid #b30b00" : "1px solid #cbd5e1";
      item.style.position = "relative";

      // Mini Canvas Thumbnail
      const thumbCanvas = document.createElement("canvas");
      thumbCanvas.className = "pdf-thumbnail-canvas";
      const scale = 0.16;
      const isSideways = page.rotation === 90 || page.rotation === 270;
      const w = (isSideways ? page.height : page.width) * scale;
      const h = (isSideways ? page.width : page.height) * scale;
      thumbCanvas.width = w;
      thumbCanvas.height = h;
      thumbCanvas.style.width = `${Math.round(w)}px`;
      thumbCanvas.style.height = `${Math.round(h)}px`;
      thumbCanvas.style.boxShadow = "0 2px 4px rgba(0,0,0,0.1)";

      const ctx = thumbCanvas.getContext("2d");
      if (ctx) {
        ctx.save();
        ctx.scale(scale, scale);
        this.drawPageToContext(ctx, page);
        ctx.restore();
      }
      item.appendChild(thumbCanvas);

      // Label & Action bar
      const infoRow = document.createElement("div");
      infoRow.style.display = "flex";
      infoRow.style.width = "100%";
      infoRow.style.justifyContent = "space-between";
      infoRow.style.alignItems = "center";
      infoRow.style.marginTop = "6px";
      infoRow.style.fontSize = "11px";
      infoRow.style.color = "#334155";

      const label = document.createElement("span");
      label.textContent = `Page ${page.pageNumber}`;
      if (page.rotation !== 0) {
        label.textContent += ` (${page.rotation}°)`;
      }
      infoRow.appendChild(label);

      // Mini actions (Rotate CW, Delete)
      const actions = document.createElement("div");
      actions.style.display = "flex";
      actions.style.gap = "4px";

      const rotBtn = this.createMiniActionBtn("⟳", "Rotate 90° CW", () => {
        this.state.rotatePageClockwise(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });
      rotBtn.className = "btn-thumb-rotate";
      rotBtn.setAttribute("data-testid", "btn-thumb-rotate");

      const delBtn = this.createMiniActionBtn("✕", "Delete Page", () => {
        if (confirm(`Delete page ${page.pageNumber}?`)) {
          this.state.deletePage(page.pageNumber);
          this.render();
          if (this.callbacks.onDirty) this.callbacks.onDirty();
        }
      });
      delBtn.className = "btn-thumb-delete";
      delBtn.setAttribute("data-testid", "btn-thumb-delete");

      actions.appendChild(rotBtn);
      actions.appendChild(delBtn);
      infoRow.appendChild(actions);

      item.appendChild(infoRow);

      item.onclick = (e) => {
        if ((e.target as HTMLElement).tagName === "BUTTON") return;
        this.scrollToPage(page.pageNumber);
      };

      list.appendChild(item);
    });

    this.sidebarBodyEl.appendChild(list);
  }

  private renderBookmarksTab() {
    const wrap = document.createElement("div");
    wrap.className = "pdf-bookmarks-tree";
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "4px";

    const topBar = document.createElement("div");
    topBar.style.display = "flex";
    topBar.style.justifyContent = "space-between";
    topBar.style.alignItems = "center";
    topBar.style.marginBottom = "8px";

    const titleSpan = document.createElement("span");
    titleSpan.style.fontSize = "12px";
    titleSpan.style.fontWeight = "600";
    titleSpan.style.color = "#475569";
    titleSpan.textContent = "DOCUMENT OUTLINE";

    const addBtn = document.createElement("button");
    addBtn.className = "btn-add-bookmark";
    addBtn.textContent = "+ Bookmark";
    addBtn.style.padding = "2px 6px";
    addBtn.style.fontSize = "11px";
    addBtn.style.cursor = "pointer";
    addBtn.onclick = () => {
      const title = window.prompt("Bookmark Title:", `Page ${this.state.currentPage}`);
      if (title) {
        this.state.addBookmark(title, this.state.currentPage);
        this.renderSidebar();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      }
    };
    topBar.appendChild(titleSpan);
    topBar.appendChild(addBtn);
    wrap.appendChild(topBar);

    this.state.bookmarks.forEach((bm) => {
      const row = document.createElement("div");
      row.className = "pdf-bookmark-item";
      row.style.display = "flex";
      row.style.alignItems = "center";
      row.style.padding = "6px 8px";
      row.style.paddingLeft = `${8 + bm.level * 16}px`;
      row.style.borderRadius = "4px";
      row.style.cursor = "pointer";
      row.style.fontSize = "12px";
      row.style.color = "#1e293b";
      row.style.transition = "background-color 0.15s ease";
      row.onmouseenter = () => (row.style.backgroundColor = "#e2e8f0");
      row.onmouseleave = () => (row.style.backgroundColor = "transparent");

      row.innerHTML = `<span style="margin-right:6px;">📑</span><span style="flex:1;">${bm.title}</span><span style="color:#64748b;font-size:11px;">p.${bm.pageNumber}</span>`;
      row.onclick = () => this.scrollToPage(bm.pageNumber);
      wrap.appendChild(row);
    });

    this.sidebarBodyEl.appendChild(wrap);
  }

  private renderAnnotationsTab() {
    const wrap = document.createElement("div");
    wrap.className = "pdf-annotations-list";
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "6px";

    const allAnnots = this.state.getAllAnnotations();
    if (allAnnots.length === 0) {
      wrap.innerHTML = `<div style="text-align:center;padding:24px 8px;color:#94a3b8;font-size:12px;">No annotations in this document.</div>`;
      this.sidebarBodyEl.appendChild(wrap);
      return;
    }

    allAnnots.forEach((ann) => {
      const card = document.createElement("div");
      card.className = "pdf-annotation-item";
      card.dataset.id = ann.id;
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.padding = "6px 8px";
      card.style.borderRadius = "4px";
      card.style.backgroundColor = "#ffffff";
      card.style.border = "1px solid #cbd5e1";
      card.style.cursor = "pointer";
      card.style.fontSize = "12px";

      const header = document.createElement("div");
      header.style.display = "flex";
      header.style.justifyContent = "space-between";
      header.style.alignItems = "center";
      header.style.marginBottom = "4px";

      let icon = "📝";
      if (ann.type === "highlight") icon = "🖍️";
      if (ann.type === "underline") icon = "_";
      if (ann.type === "strike") icon = "~";
      if (ann.type === "ink") icon = "✏️";
      if (ann.type === "redact") icon = "⬛";
      if (ann.type === "signature") icon = "✒️";

      header.innerHTML = `<span>${icon} <strong>${ann.type}</strong></span><span style="color:#64748b;font-size:11px;">p.${ann.pageNumber}</span>`;
      card.appendChild(header);

      if (ann.contents) {
        const body = document.createElement("div");
        body.style.color = "#334155";
        body.style.fontSize = "11px";
        body.textContent = ann.contents;
        card.appendChild(body);
      }

      card.onclick = () => {
        this.scrollToPage(ann.pageNumber);
        if (ann.type === "note") {
          this.showCommentPopover(ann);
        }
      };
      wrap.appendChild(card);
    });

    this.sidebarBodyEl.appendChild(wrap);
  }

  private createMiniActionBtn(text: string, title: string, onClick: () => void): HTMLButtonElement {
    const btn = document.createElement("button");
    btn.textContent = text;
    btn.title = title;
    btn.style.border = "none";
    btn.style.backgroundColor = "transparent";
    btn.style.cursor = "pointer";
    btn.style.padding = "2px 4px";
    btn.style.fontSize = "11px";
    btn.style.borderRadius = "2px";
    btn.style.color = "#64748b";
    btn.onmouseenter = () => (btn.style.backgroundColor = "#cbd5e1");
    btn.onmouseleave = () => (btn.style.backgroundColor = "transparent");
    btn.onclick = (e) => {
      e.stopPropagation();
      onClick();
    };
    return btn;
  }

  // --- Central Viewport Rendering ---

  private renderViewport() {
    this.viewportEl.innerHTML = "";
    this.pageCanvases.clear();
    this.closeCommentPopover();

    if (this.state.isEncrypted) {
      const lockOverlay = document.createElement("div");
      lockOverlay.className = "pdf-lock-screen";
      lockOverlay.setAttribute("data-testid", "pdf-lock-screen");
      lockOverlay.style.display = "flex";
      lockOverlay.style.flexDirection = "column";
      lockOverlay.style.alignItems = "center";
      lockOverlay.style.justifyContent = "center";
      lockOverlay.style.height = "100%";
      lockOverlay.style.padding = "40px";
      lockOverlay.style.textAlign = "center";
      lockOverlay.innerHTML = `
        <div style="font-size: 48px; margin-bottom: 16px;">🔒</div>
        <h2 style="font-size: 20px; font-weight: 600; margin-bottom: 8px; color: #0f172a;">Document is Password Protected</h2>
        <p style="font-size: 14px; color: #64748b; margin-bottom: 20px;">Enter password to unlock and view contents.</p>
        <div style="display: flex; gap: 8px;">
          <input type="password" id="pdf-unlock-password-input" data-testid="pdf-unlock-input" placeholder="Enter password..." style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 14px; width: 220px;" />
          <button id="pdf-unlock-btn" data-testid="pdf-unlock-btn" style="padding: 8px 16px; background: #2563eb; color: #fff; border: none; border-radius: 4px; font-weight: 600; cursor: pointer;">Unlock</button>
        </div>
        <div id="pdf-unlock-error" data-testid="pdf-unlock-error" style="color: #ef4444; font-size: 13px; margin-top: 8px; display: none;">Incorrect password</div>
      `;
      const input = lockOverlay.querySelector<HTMLInputElement>("#pdf-unlock-password-input");
      const btn = lockOverlay.querySelector<HTMLButtonElement>("#pdf-unlock-btn");
      const err = lockOverlay.querySelector<HTMLElement>("#pdf-unlock-error");
      const tryUnlock = () => {
        const pass = input?.value || "";
        if (this.state.decryptPassword(pass)) {
          this.render();
          if (this.callbacks.onDirty) this.callbacks.onDirty();
        } else {
          if (err) err.style.display = "block";
        }
      };
      if (btn) btn.onclick = tryUnlock;
      if (input) {
        input.onkeydown = (e) => {
          if (e.key === "Enter") tryUnlock();
        };
      }
      this.viewportEl.appendChild(lockOverlay);
      return;
    }

    const zoom = this.state.zoom;
    const viewMode = this.state.viewMode;

    let containerWrap = this.viewportEl;
    if (viewMode === "twoPage") {
      const spreadRow = document.createElement("div");
      spreadRow.className = "pdf-spread-container";
      spreadRow.style.display = "flex";
      spreadRow.style.flexWrap = "wrap";
      spreadRow.style.justifyContent = "center";
      spreadRow.style.gap = "24px";
      this.viewportEl.appendChild(spreadRow);
      containerWrap = spreadRow;
    }

    const pagesToRender =
      viewMode === "single"
        ? this.state.pages.filter((p) => p.pageNumber === this.state.currentPage)
        : this.state.pages;

    pagesToRender.forEach((page) => {
      const pageWrap = document.createElement("div");
      pageWrap.className = `pdf-page-container ${page.pageNumber === this.state.currentPage ? "is-current-page" : ""}`;
      pageWrap.dataset.pageNumber = page.pageNumber.toString();
      pageWrap.style.marginBottom = "24px";
      pageWrap.style.display = "flex";
      pageWrap.style.flexDirection = "column";
      pageWrap.style.alignItems = "center";
      pageWrap.style.position = "relative";

      // Rotation & Sizing
      const isSideways = page.rotation === 90 || page.rotation === 270;
      const effW = (isSideways ? page.height : page.width) * zoom;
      const effH = (isSideways ? page.width : page.height) * zoom;

      // Canvas wrapper for layering interactive DOM overlays
      const canvasWrap = document.createElement("div");
      canvasWrap.className = "pdf-canvas-wrap";
      canvasWrap.style.position = "relative";
      canvasWrap.style.width = `${Math.round(effW)}px`;
      canvasWrap.style.height = `${Math.round(effH)}px`;

      const canvas = document.createElement("canvas");
      canvas.className = "pdf-page-canvas";
      canvas.id = `pdf-canvas-p${page.pageNumber}`;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = effW * dpr;
      canvas.height = effH * dpr;
      canvas.style.width = `${Math.round(effW)}px`;
      canvas.style.height = `${Math.round(effH)}px`;
      canvas.style.backgroundColor = "#ffffff";
      canvas.style.boxShadow = "0 4px 16px rgba(0, 0, 0, 0.15)";
      canvas.style.borderRadius = "2px";

      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.setTransform(dpr * zoom, 0, 0, dpr * zoom, 0, 0);
        this.drawPageToContext(ctx, page);
      }

      this.pageCanvases.set(page.pageNumber, canvas);
      canvasWrap.appendChild(canvas);

      // Interactive Overlay (Form fields, In-place Text Editing, Ink Drawing, Sticky Notes)
      const overlay = this.createPageOverlay(page, effW, effH, zoom);
      canvasWrap.appendChild(overlay);

      pageWrap.appendChild(canvasWrap);

      // Page Number Footer indicator
      const footer = document.createElement("div");
      footer.style.marginTop = "6px";
      footer.style.fontSize = "11px";
      footer.style.color = "#64748b";
      footer.textContent = `Page ${page.pageNumber} of ${this.state.pages.length}`;
      pageWrap.appendChild(footer);

      containerWrap.appendChild(pageWrap);
    });
  }

  // --- Interactive Page Overlay ---

  private createPageOverlay(page: PdfPageModel, effW: number, effH: number, zoom: number): HTMLElement {
    const overlay = document.createElement("div");
    overlay.className = "pdf-page-overlay";
    overlay.setAttribute("data-testid", "pdf-page-overlay");
    overlay.dataset.pageNumber = page.pageNumber.toString();
    overlay.style.position = "absolute";
    overlay.style.top = "0";
    overlay.style.left = "0";
    overlay.style.width = `${Math.round(effW)}px`;
    overlay.style.height = `${Math.round(effH)}px`;
    overlay.style.pointerEvents = "auto";
    overlay.style.cursor = this.getCursorForTool();

    // 1. Text Blocks (Edit & Add Text)
    if (page.textBlocks) {
      page.textBlocks.forEach((tb) => {
        const tbBox = document.createElement("div");
        tbBox.className = "pdf-text-block-editable";
        tbBox.setAttribute("data-testid", "pdf-text-block-editable");
        tbBox.dataset.blockId = tb.id;
        tbBox.style.position = "absolute";
        tbBox.style.left = `${tb.x * zoom}px`;
        tbBox.style.top = `${(tb.y - (tb.fontSize || 14)) * zoom}px`;
        tbBox.style.cursor = this.state.activeTool === "edit_text" ? "text" : "default";

        if (this.state.activeTool === "edit_text") {
          tbBox.style.border = "1px dashed #0078d4";
          tbBox.style.padding = "2px 4px";
          tbBox.style.backgroundColor = "rgba(0, 120, 212, 0.05)";
          tbBox.title = "Click to edit text";
        }

        tbBox.onclick = (e) => {
          if (this.state.activeTool === "edit_text") {
            e.stopPropagation();
            this.openInlineTextEditor(tbBox, page, tb, zoom);
          }
        };

        overlay.appendChild(tbBox);
      });
    }

    // 1b. Image Blocks
    if (page.imageBlocks) {
      page.imageBlocks.forEach((img) => {
        const imgBox = document.createElement("div");
        imgBox.className = "pdf-image-block";
        imgBox.setAttribute("data-testid", "pdf-image-block");
        imgBox.dataset.blockId = img.id;
        imgBox.style.position = "absolute";
        imgBox.style.left = `${img.x * zoom}px`;
        imgBox.style.top = `${img.y * zoom}px`;
        imgBox.style.width = `${img.width * zoom}px`;
        imgBox.style.height = `${img.height * zoom}px`;
        imgBox.style.border = "1px dashed #94a3b8";
        imgBox.style.boxSizing = "border-box";
        overlay.appendChild(imgBox);
      });
    }

    // 2. Interactive AcroForm Fields Overlay
    if (page.formFields) {
      page.formFields.forEach((field) => {
        const [fx, fy, fw, fh] = field.rect;
        const fieldEl = document.createElement("div");
        fieldEl.className = `pdf-form-field-interactive pdf-field-${field.type}`;
        fieldEl.setAttribute("data-testid", `pdf-field-${field.type}`);
        fieldEl.dataset.fieldId = field.id;
        fieldEl.dataset.fieldName = field.name;
        fieldEl.style.position = "absolute";
        fieldEl.style.left = `${fx * zoom}px`;
        fieldEl.style.top = `${fy * zoom}px`;
        fieldEl.style.width = `${fw * zoom}px`;
        fieldEl.style.height = `${fh * zoom}px`;
        fieldEl.style.cursor = "pointer";

        if (this.state.highlightFields) {
          fieldEl.style.backgroundColor = "rgba(59, 130, 246, 0.25)";
          fieldEl.style.outline = "2px dashed #2563eb";
        }

        if (field.type === "checkbox") {
          fieldEl.onclick = (e) => {
            e.stopPropagation();
            this.state.setFormFieldValue(field.id, !field.value);
            this.render();
            if (this.callbacks.onDirty) this.callbacks.onDirty();
          };
        } else if (field.type === "radio") {
          fieldEl.onclick = (e) => {
            e.stopPropagation();
            this.state.setFormFieldValue(field.id, true);
            this.render();
            if (this.callbacks.onDirty) this.callbacks.onDirty();
          };
        } else if (field.type === "text") {
          fieldEl.onclick = (e) => {
            e.stopPropagation();
            const newVal = prompt(`Edit ${field.name}:`, String(field.value));
            if (newVal !== null) {
              this.state.setFormFieldValue(field.id, newVal);
              this.render();
              if (this.callbacks.onDirty) this.callbacks.onDirty();
            }
          };
        } else if (field.type === "dropdown") {
          fieldEl.onclick = (e) => {
            e.stopPropagation();
            const opts = field.options || ["Option 1", "Option 2", "Option 3"];
            const currentIdx = opts.indexOf(String(field.value));
            const nextIdx = (currentIdx + 1) % opts.length;
            this.state.setFormFieldValue(field.id, opts[nextIdx]);
            this.render();
            if (this.callbacks.onDirty) this.callbacks.onDirty();
          };
        } else if (field.type === "signature") {
          fieldEl.onclick = (e) => {
            e.stopPropagation();
            this.openSignModal(page.pageNumber, field.id);
          };
        }

        overlay.appendChild(fieldEl);
      });
    }

    // 3. Annotations Overlay (Sticky Notes & Eraser)
    if (page.annotations) {
      page.annotations.forEach((ann) => {
        const [ax, ay, aw, ah] = ann.rect;
        const annEl = document.createElement("div");
        annEl.className = `pdf-annot-hitbox pdf-annot-${ann.type}`;
        annEl.setAttribute("data-testid", `pdf-annot-${ann.type}`);
        annEl.dataset.annotId = ann.id;
        annEl.style.position = "absolute";
        annEl.style.left = `${ax * zoom}px`;
        annEl.style.top = `${ay * zoom}px`;
        annEl.style.width = `${Math.max(aw, 20) * zoom}px`;
        annEl.style.height = `${Math.max(ah, 20) * zoom}px`;
        annEl.style.cursor = this.state.activeTool === "eraser" ? "crosshair" : "pointer";

        annEl.onclick = (e) => {
          e.stopPropagation();
          if (this.state.activeTool === "eraser") {
            this.state.removeAnnotation(ann.id);
            this.render();
            if (this.callbacks.onDirty) this.callbacks.onDirty();
          } else if (ann.type === "note") {
            this.showCommentPopover(ann);
          }
        };

        overlay.appendChild(annEl);
      });
    }

    // 4. Ink Drawing & Placement Events on Overlay
    overlay.addEventListener("mousedown", (e) => {
      const rect = overlay.getBoundingClientRect();
      const x = (e.clientX - rect.left) / zoom;
      const y = (e.clientY - rect.top) / zoom;

      if (this.state.activeTool === "pencil") {
        this.isDrawingInk = true;
        this.activeInkPageNumber = page.pageNumber;
        this.currentInkPoints = [{ x, y }];
      } else if (this.state.activeTool === "add_text") {
        this.state.addTextBlock(page.pageNumber, Math.round(x), Math.round(y));
        this.state.setTool("edit_text");
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "add_image") {
        this.state.addImageBlock(page.pageNumber, {
          x: Math.round(x),
          y: Math.round(y),
          width: 220,
          height: 140,
          title: "Inserted Graphic",
        });
        this.state.setTool("select");
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "note") {
        const note = this.state.addAnnotation({
          type: "note",
          pageNumber: page.pageNumber,
          rect: [Math.round(x), Math.round(y), 24, 24],
          color: "#eab308",
          author: "User",
          date: new Date().toISOString().slice(0, 10),
          contents: "New sticky comment",
        });
        this.render();
        this.showCommentPopover(note);
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "highlight") {
        this.state.addAnnotation({
          type: "highlight",
          pageNumber: page.pageNumber,
          rect: [Math.round(x), Math.round(y), 200, 20],
          color: this.state.highlightColor,
          contents: "Highlighted text",
        });
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "underline") {
        this.state.addAnnotation({
          type: "underline",
          pageNumber: page.pageNumber,
          rect: [Math.round(x), Math.round(y), 180, 2],
          color: "#0078d4",
        });
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "strike") {
        this.state.addAnnotation({
          type: "strike",
          pageNumber: page.pageNumber,
          rect: [Math.round(x), Math.round(y), 180, 2],
          color: "#dc2626",
        });
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "redact") {
        this.state.markRedaction(page.pageNumber, [Math.round(x), Math.round(y), 220, 36]);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "add_form_field") {
        this.state.addFormField(page.pageNumber, {
          rect: [Math.round(x), Math.round(y), 180, 28],
        });
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      } else if (this.state.activeTool === "sign") {
        this.openSignModal(page.pageNumber);
      }
    });

    overlay.addEventListener("mousemove", (e) => {
      if (!this.isDrawingInk || this.activeInkPageNumber !== page.pageNumber) return;
      const rect = overlay.getBoundingClientRect();
      const x = (e.clientX - rect.left) / zoom;
      const y = (e.clientY - rect.top) / zoom;
      this.currentInkPoints.push({ x, y });

      // Draw real-time stroke on page canvas
      const canvas = this.pageCanvases.get(page.pageNumber);
      if (canvas) {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          const dpr = window.devicePixelRatio || 1;
          ctx.save();
          ctx.setTransform(dpr * zoom, 0, 0, dpr * zoom, 0, 0);
          ctx.strokeStyle = this.state.inkColor;
          ctx.lineWidth = this.state.inkThickness;
          ctx.lineCap = "round";
          ctx.lineJoin = "round";
          const pts = this.currentInkPoints;
          if (pts.length > 1) {
            ctx.beginPath();
            ctx.moveTo(pts[pts.length - 2].x, pts[pts.length - 2].y);
            ctx.lineTo(pts[pts.length - 1].x, pts[pts.length - 1].y);
            ctx.stroke();
          }
          ctx.restore();
        }
      }
    });

    const stopInk = () => {
      if (this.isDrawingInk && this.currentInkPoints.length > 1) {
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        this.currentInkPoints.forEach((p) => {
          if (p.x < minX) minX = p.x;
          if (p.y < minY) minY = p.y;
          if (p.x > maxX) maxX = p.x;
          if (p.y > maxY) maxY = p.y;
        });

        this.state.addAnnotation({
          type: "ink",
          pageNumber: this.activeInkPageNumber,
          rect: [minX, minY, maxX - minX, maxY - minY],
          color: this.state.inkColor,
          strokeWidth: this.state.inkThickness,
          points: [...this.currentInkPoints],
          contents: "Stylus / Freehand Ink Stroke",
        });
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      }
      this.isDrawingInk = false;
      this.currentInkPoints = [];
    };

    overlay.addEventListener("mouseup", stopInk);
    overlay.addEventListener("mouseleave", stopInk);

    return overlay;
  }

  // --- Inline Text Editor ---

  private openInlineTextEditor(
    anchor: HTMLElement,
    page: PdfPageModel,
    tb: PdfTextBlock,
    zoom: number
  ) {
    const input = document.createElement("input");
    input.type = "text";
    input.className = "pdf-inline-text-editor";
    input.value = tb.text;
    input.style.position = "absolute";
    input.style.left = `${tb.x * zoom}px`;
    input.style.top = `${(tb.y - (tb.fontSize || 14)) * zoom}px`;
    input.style.fontSize = `${(tb.fontSize || 14) * zoom}px`;
    input.style.fontFamily = tb.fontFamily || "Segoe UI, sans-serif";
    input.style.color = tb.fontColor || "#1e293b";
    input.style.fontWeight = tb.bold ? "bold" : "normal";
    input.style.padding = "2px 4px";
    input.style.border = "1px solid #0078d4";
    input.style.borderRadius = "2px";
    input.style.backgroundColor = "#ffffff";
    input.style.boxShadow = "0 2px 8px rgba(0,0,0,0.2)";
    input.style.zIndex = "100";
    input.style.outline = "none";

    const commit = () => {
      const val = input.value.trim();
      if (val !== "") {
        this.state.updateTextBlock(page.pageNumber, tb.id, val);
      }
      if (input.parentElement) {
        input.parentElement.removeChild(input);
      }
      this.render();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    };

    input.onkeydown = (e) => {
      if (e.key === "Enter") {
        commit();
      } else if (e.key === "Escape") {
        if (input.parentElement) input.parentElement.removeChild(input);
      }
    };
    input.onblur = commit;

    if (anchor.parentElement) {
      anchor.parentElement.appendChild(input);
      input.focus();
      input.select();
    }
  }

  // --- Sticky Note Comment Popover ---

  private showCommentPopover(ann: PdfAnnotation) {
    this.closeCommentPopover();

    const popover = document.createElement("div");
    popover.className = "pdf-comment-popover";
    popover.setAttribute("data-testid", "pdf-comment-popover");
    popover.style.position = "fixed";
    popover.style.zIndex = "1000";
    popover.style.width = "260px";
    popover.style.backgroundColor = "#ffffff";
    popover.style.border = "1px solid #cbd5e1";
    popover.style.borderRadius = "6px";
    popover.style.boxShadow = "0 10px 15px -3px rgba(0,0,0,0.2)";
    popover.style.padding = "12px";
    popover.style.display = "flex";
    popover.style.flexDirection = "column";
    popover.style.gap = "8px";

    // Header
    const head = document.createElement("div");
    head.style.display = "flex";
    head.style.justifyContent = "space-between";
    head.style.alignItems = "center";
    head.style.fontSize = "12px";
    head.style.color = "#475569";
    head.innerHTML = `<span><strong>${ann.author || "User"}</strong></span><span style="font-size:11px;">${ann.date || ""}</span>`;

    const closeBtn = document.createElement("button");
    closeBtn.className = "btn-comment-close";
    closeBtn.setAttribute("data-testid", "btn-comment-close");
    closeBtn.innerHTML = "&times;";
    closeBtn.style.border = "none";
    closeBtn.style.background = "none";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.fontSize = "16px";
    closeBtn.onclick = () => this.closeCommentPopover();
    head.appendChild(closeBtn);
    popover.appendChild(head);

    // Textarea
    const textarea = document.createElement("textarea");
    textarea.className = "pdf-comment-textarea";
    textarea.setAttribute("data-testid", "pdf-comment-textarea");
    textarea.value = ann.contents || "";
    textarea.style.width = "100%";
    textarea.style.height = "70px";
    textarea.style.boxSizing = "border-box";
    textarea.style.padding = "6px";
    textarea.style.borderRadius = "4px";
    textarea.style.border = "1px solid #cbd5e1";
    textarea.style.fontSize = "12px";
    textarea.style.fontFamily = "Segoe UI, sans-serif";
    popover.appendChild(textarea);

    // Actions
    const actions = document.createElement("div");
    actions.style.display = "flex";
    actions.style.justifyContent = "space-between";
    actions.style.alignItems = "center";

    const delBtn = document.createElement("button");
    delBtn.className = "btn-comment-delete";
    delBtn.setAttribute("data-testid", "btn-comment-delete");
    delBtn.textContent = "Delete Note";
    delBtn.style.border = "none";
    delBtn.style.background = "none";
    delBtn.style.color = "#dc2626";
    delBtn.style.fontSize = "11px";
    delBtn.style.cursor = "pointer";
    delBtn.onclick = () => {
      this.state.removeAnnotation(ann.id);
      this.closeCommentPopover();
      this.render();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    };
    actions.appendChild(delBtn);

    const saveBtn = document.createElement("button");
    saveBtn.className = "btn-comment-save";
    saveBtn.setAttribute("data-testid", "btn-comment-save");
    saveBtn.textContent = "Save";
    saveBtn.style.padding = "4px 12px";
    saveBtn.style.backgroundColor = "#b30b00";
    saveBtn.style.color = "#ffffff";
    saveBtn.style.border = "none";
    saveBtn.style.borderRadius = "4px";
    saveBtn.style.fontSize = "12px";
    saveBtn.style.cursor = "pointer";
    saveBtn.onclick = () => {
      this.state.updateAnnotation(ann.id, { contents: textarea.value });
      this.closeCommentPopover();
      this.render();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    };
    actions.appendChild(saveBtn);

    popover.appendChild(actions);

    // Center popover
    popover.style.top = "180px";
    popover.style.right = "80px";

    this.activePopover = popover;
    document.body.appendChild(popover);
  }

  private closeCommentPopover() {
    if (this.activePopover && this.activePopover.parentElement) {
      this.activePopover.parentElement.removeChild(this.activePopover);
      this.activePopover = null;
    }
  }

  private getCursorForTool(): string {
    switch (this.state.activeTool) {
      case "hand":
        return "grab";
      case "edit_text":
      case "add_text":
        return "text";
      case "pencil":
        return "crosshair";
      case "eraser":
        return "crosshair";
      case "highlight":
      case "underline":
      case "strike":
      case "redact":
        return "crosshair";
      case "note":
        return "cell";
      default:
        return "default";
    }
  }

  // --- Canvas Rendering Pipeline ---

  public drawPageToContext(ctx: CanvasRenderingContext2D, page: PdfPageModel) {
    const isSideways = page.rotation === 90 || page.rotation === 270;
    const effW = isSideways ? page.height : page.width;
    const effH = isSideways ? page.width : page.height;

    ctx.save();
    ctx.clearRect(0, 0, effW, effH);

    // Apply Rotation Transform around center
    if (page.rotation !== 0) {
      ctx.translate(effW / 2, effH / 2);
      ctx.rotate((page.rotation * Math.PI) / 180);
      ctx.translate(-page.width / 2, -page.height / 2);
    }

    // 1. White Page Background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, page.width, page.height);

    // 2. Render Watermark (if present)
    if (page.watermark) {
      ctx.save();
      ctx.translate(page.width / 2, page.height / 2);
      ctx.rotate((page.watermark.rotation * Math.PI) / 180);
      ctx.fillStyle = page.watermark.color;
      ctx.globalAlpha = page.watermark.opacity;
      ctx.font = `bold ${page.watermark.fontSize}px Segoe UI, sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(page.watermark.text, 0, 0);
      ctx.restore();
    }

    // 3. Render Text Blocks
    if (page.textBlocks) {
      page.textBlocks.forEach((tb) => {
        ctx.fillStyle = tb.fontColor || "#1e293b";
        const weight = tb.bold ? "bold " : "";
        const italic = tb.italic ? "italic " : "";
        ctx.font = `${weight}${italic}${tb.fontSize || 14}px ${tb.fontFamily || "Segoe UI, sans-serif"}`;
        if (tb.align === "center") {
          ctx.textAlign = "center";
        } else if (tb.align === "right") {
          ctx.textAlign = "right";
        } else {
          ctx.textAlign = "left";
        }
        ctx.textBaseline = "alphabetic";
        ctx.fillText(tb.text, tb.x, tb.y);

        if (tb.underline) {
          const metrics = ctx.measureText(tb.text);
          ctx.strokeStyle = tb.fontColor || "#1e293b";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(tb.x, tb.y + 2);
          ctx.lineTo(tb.x + metrics.width, tb.y + 2);
          ctx.stroke();
        }

        if (this.state.searchQuery && tb.text.toLowerCase().includes(this.state.searchQuery)) {
          const metrics = ctx.measureText(tb.text);
          ctx.save();
          ctx.fillStyle = "rgba(253, 224, 71, 0.4)";
          ctx.fillRect(tb.x - 2, tb.y - (tb.fontSize || 14), metrics.width + 4, (tb.fontSize || 14) + 4);
          ctx.restore();
        }
      });
    }

    // 4. Render Image Blocks
    if (page.imageBlocks) {
      page.imageBlocks.forEach((img) => {
        ctx.save();
        ctx.strokeStyle = "#cbd5e1";
        ctx.lineWidth = 1;
        ctx.strokeRect(img.x, img.y, img.width, img.height);
        ctx.fillStyle = "#f1f5f9";
        ctx.fillRect(img.x, img.y, img.width, img.height);

        ctx.fillStyle = "#64748b";
        ctx.font = "12px Segoe UI, sans-serif";
        ctx.textAlign = "center";
        ctx.fillText(`🖼️ ${img.title || "Graphic"}`, img.x + img.width / 2, img.y + img.height / 2);
        ctx.restore();
      });
    }

    // 5. Render AcroForms Fields
    if (page.formFields) {
      page.formFields.forEach((f) => {
        const [fx, fy, fw, fh] = f.rect;
        if (f.type === "text") {
          ctx.font = "11px Segoe UI, sans-serif";
          ctx.fillStyle = "#64748b";
          ctx.fillText(f.name, fx, fy - 6);

          ctx.fillStyle = "#eff6fc";
          ctx.fillRect(fx, fy, fw, fh);
          ctx.strokeStyle = "#0078d4";
          ctx.lineWidth = 1;
          ctx.strokeRect(fx, fy, fw, fh);

          ctx.font = "13px Segoe UI, sans-serif";
          ctx.fillStyle = "#111111";
          ctx.fillText(String(f.value), fx + 8, fy + 20);
        } else if (f.type === "checkbox") {
          ctx.font = "11px Segoe UI, sans-serif";
          ctx.fillStyle = "#64748b";
          ctx.fillText(f.name, fx + fw + 8, fy + 14);

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
        } else if (f.type === "radio") {
          ctx.font = "11px Segoe UI, sans-serif";
          ctx.fillStyle = "#64748b";
          ctx.fillText(f.name, fx + fw + 8, fy + 14);

          ctx.beginPath();
          ctx.arc(fx + fw / 2, fy + fh / 2, fw / 2, 0, Math.PI * 2);
          ctx.fillStyle = "#eff6fc";
          ctx.fill();
          ctx.strokeStyle = "#0078d4";
          ctx.stroke();

          if (f.value === true) {
            ctx.beginPath();
            ctx.arc(fx + fw / 2, fy + fh / 2, fw / 4, 0, Math.PI * 2);
            ctx.fillStyle = "#0078d4";
            ctx.fill();
          }
        } else if (f.type === "dropdown") {
          ctx.font = "11px Segoe UI, sans-serif";
          ctx.fillStyle = "#64748b";
          ctx.fillText(f.name, fx, fy - 6);

          ctx.fillStyle = "#eff6fc";
          ctx.fillRect(fx, fy, fw, fh);
          ctx.strokeStyle = "#0078d4";
          ctx.strokeRect(fx, fy, fw, fh);

          ctx.font = "12px Segoe UI, sans-serif";
          ctx.fillStyle = "#1e293b";
          ctx.fillText(String(f.value), fx + 8, fy + 18);

          ctx.font = "10px Segoe UI, sans-serif";
          ctx.fillText("▼", fx + fw - 16, fy + 18);
        } else if (f.type === "signature") {
          ctx.strokeStyle = "#002060";
          ctx.lineWidth = 1.5;
          ctx.strokeRect(fx, fy, fw, fh);
          ctx.fillStyle = "rgba(0, 32, 96, 0.05)";
          ctx.fillRect(fx, fy, fw, fh);

          if (f.value) {
            ctx.font = "20px 'Brush Script MT', cursive, sans-serif";
            ctx.fillStyle = "#002060";
            ctx.fillText(String(f.value), fx + 12, fy + fh / 2 + 6);
          } else {
            ctx.font = "12px Segoe UI, sans-serif";
            ctx.fillStyle = "#002060";
            ctx.fillText("✒️ Click to Sign", fx + 12, fy + fh / 2 + 4);
          }
        }
      });
    }

    // 6. Render Annotations & Highlights
    if (page.annotations) {
      page.annotations.forEach((ann) => {
        const [ax, ay, aw, ah] = ann.rect;
        if (ann.type === "highlight") {
          ctx.fillStyle = ann.color;
          ctx.fillRect(ax, ay, aw, ah);
        } else if (ann.type === "underline") {
          ctx.strokeStyle = ann.color;
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(ax, ay + ah);
          ctx.lineTo(ax + aw, ay + ah);
          ctx.stroke();
        } else if (ann.type === "strike") {
          ctx.strokeStyle = ann.color;
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.moveTo(ax, ay + ah / 2);
          ctx.lineTo(ax + aw, ay + ah / 2);
          ctx.stroke();
        } else if (ann.type === "note") {
          ctx.fillStyle = ann.color;
          ctx.fillRect(ax, ay, aw, ah);
          ctx.fillStyle = "#ffffff";
          ctx.font = "14px Segoe UI";
          ctx.textAlign = "center";
          ctx.fillText("📝", ax + aw / 2, ay + ah / 2 + 5);
        } else if (ann.type === "ink" && ann.points && ann.points.length > 0) {
          ctx.strokeStyle = ann.color;
          ctx.lineWidth = ann.strokeWidth || 2;
          ctx.lineCap = "round";
          ctx.lineJoin = "round";
          ctx.beginPath();
          ctx.moveTo(ann.points[0].x, ann.points[0].y);
          for (let i = 1; i < ann.points.length; i++) {
            ctx.lineTo(ann.points[i].x, ann.points[i].y);
          }
          ctx.stroke();
        } else if (ann.type === "stamp") {
          ctx.save();
          ctx.strokeStyle = ann.color;
          ctx.lineWidth = 2.5;
          ctx.strokeRect(ax, ay, aw, ah);
          ctx.fillStyle = ann.color;
          ctx.font = "bold 14px Segoe UI, sans-serif";
          ctx.textAlign = "center";
          ctx.fillText(ann.contents || "STAMP", ax + aw / 2, ay + ah / 2 + 5);
          ctx.restore();
        } else if (ann.type === "redact") {
          if (ann.isAppliedRedaction) {
            ctx.fillStyle = "#000000";
            ctx.fillRect(ax, ay, aw, ah);
          } else {
            ctx.strokeStyle = "#dc2626";
            ctx.lineWidth = 1.5;
            ctx.strokeRect(ax, ay, aw, ah);
            ctx.fillStyle = "rgba(220, 38, 38, 0.2)";
            ctx.fillRect(ax, ay, aw, ah);
            ctx.fillStyle = "#dc2626";
            ctx.font = "bold 11px Segoe UI";
            ctx.textAlign = "center";
            ctx.fillText("REDACT", ax + aw / 2, ay + ah / 2 + 4);
          }
        } else if (ann.type === "signature") {
          ctx.save();
          ctx.strokeStyle = "#002060";
          ctx.lineWidth = 1;
          ctx.strokeRect(ax, ay, aw, ah);
          ctx.fillStyle = "rgba(0, 32, 96, 0.04)";
          ctx.fillRect(ax, ay, aw, ah);
          ctx.font = "24px 'Brush Script MT', cursive, sans-serif";
          ctx.fillStyle = "#002060";
          ctx.fillText(ann.signerName || "Signed", ax + 12, ay + 36);
          ctx.font = "10px Segoe UI, sans-serif";
          ctx.fillStyle = "#64748b";
          ctx.fillText(`Verified Digital Signature • ${ann.date?.slice(0, 10) || ""}`, ax + 12, ay + 52);
          ctx.restore();
        }
      });
    }

    ctx.restore();
  }

  public scrollToPage(pageNumber: number) {
    if (this.state.setCurrentPage(pageNumber)) {
      this.renderSidebar();
      if (this.state.viewMode === "single") {
        this.renderViewport();
      } else {
        const targetCanvas = this.pageCanvases.get(pageNumber);
        if (targetCanvas) {
          targetCanvas.scrollIntoView({ behavior: "smooth", block: "center" });
        }
      }
      if (this.callbacks.onPageChange) {
        this.callbacks.onPageChange(pageNumber);
      }
    }
  }
}
