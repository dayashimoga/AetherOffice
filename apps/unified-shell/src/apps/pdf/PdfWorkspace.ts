import {
  PdfPageModel,
  PdfSidebarTab,
  PdfState,
  pdfState,
} from "../../state/pdfState";

export interface PdfWorkspaceCallbacks {
  onDirty?: () => void;
  onPageChange?: (pageNumber: number) => void;
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
    const listWrap = document.createElement("div");
    listWrap.className = "pdf-thumbnails-list";
    listWrap.style.display = "flex";
    listWrap.style.flexDirection = "column";
    listWrap.style.gap = "12px";

    this.state.pages.forEach((page) => {
      const isSelected = page.pageNumber === this.state.currentPage;

      const card = document.createElement("div");
      card.className = `pdf-thumbnail-card ${isSelected ? "is-selected" : ""}`;
      card.dataset.pageNumber = page.pageNumber.toString();
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.alignItems = "center";
      card.style.padding = "8px";
      card.style.borderRadius = "6px";
      card.style.backgroundColor = isSelected ? "#eff6ff" : "#ffffff";
      card.style.border = isSelected ? "2px solid #2563eb" : "1px solid #cbd5e1";
      card.style.cursor = "pointer";
      card.style.transition = "all 0.15s ease";
      card.style.boxShadow = "0 2px 6px rgba(0, 0, 0, 0.05)";

      // Mini Canvas
      const thumbCanvas = document.createElement("canvas");
      thumbCanvas.className = "pdf-thumbnail-canvas";
      const scale = 0.18;
      const dpr = window.devicePixelRatio || 1;

      // Handle rotated dimensions
      const isSideways = page.rotation === 90 || page.rotation === 270;
      const effW = isSideways ? page.height : page.width;
      const effH = isSideways ? page.width : page.height;

      thumbCanvas.width = effW * scale * dpr;
      thumbCanvas.height = effH * scale * dpr;
      thumbCanvas.style.width = `${Math.round(effW * scale)}px`;
      thumbCanvas.style.height = `${Math.round(effH * scale)}px`;
      thumbCanvas.style.backgroundColor = "#ffffff";
      thumbCanvas.style.borderRadius = "2px";
      thumbCanvas.style.boxShadow = "0 1px 4px rgba(0, 0, 0, 0.15)";

      const ctx = thumbCanvas.getContext("2d");
      if (ctx) {
        ctx.setTransform(dpr * scale, 0, 0, dpr * scale, 0, 0);
        this.drawPageToContext(ctx, page);
      }
      card.appendChild(thumbCanvas);

      // Card Meta & Actions
      const metaRow = document.createElement("div");
      metaRow.style.display = "flex";
      metaRow.style.alignItems = "center";
      metaRow.style.justifyContent = "space-between";
      metaRow.style.width = "100%";
      metaRow.style.marginTop = "6px";
      metaRow.style.fontSize = "11px";

      const pageBadge = document.createElement("span");
      pageBadge.style.fontWeight = "600";
      pageBadge.style.color = isSelected ? "#2563eb" : "#475569";
      pageBadge.textContent = `Page ${page.pageNumber}${page.rotation ? ` (${page.rotation}°)` : ""}`;
      metaRow.appendChild(pageBadge);

      const actionWrap = document.createElement("div");
      actionWrap.style.display = "flex";
      actionWrap.style.gap = "4px";

      const btnCw = this.createMiniActionBtn("↷", "Rotate 90° CW", () => {
        this.state.rotatePageClockwise(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });
      const btnCcw = this.createMiniActionBtn("↶", "Rotate 90° CCW", () => {
        this.state.rotatePageCounterClockwise(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });
      const btnUp = this.createMiniActionBtn("▲", "Move Page Up", () => {
        this.state.movePageUp(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });
      const btnDown = this.createMiniActionBtn("▼", "Move Page Down", () => {
        this.state.movePageDown(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });
      const btnDel = this.createMiniActionBtn("🗑", "Delete Page", () => {
        this.state.deletePage(page.pageNumber);
        this.render();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      });

      actionWrap.appendChild(btnCw);
      actionWrap.appendChild(btnCcw);
      actionWrap.appendChild(btnUp);
      actionWrap.appendChild(btnDown);
      actionWrap.appendChild(btnDel);
      metaRow.appendChild(actionWrap);

      card.appendChild(metaRow);

      card.onclick = (e) => {
        const target = e.target as HTMLElement;
        if (target.tagName.toLowerCase() === "button") return;
        this.scrollToPage(page.pageNumber);
      };

      listWrap.appendChild(card);
    });

    this.sidebarBodyEl.appendChild(listWrap);
  }

  private renderBookmarksTab() {
    const wrap = document.createElement("div");
    wrap.className = "pdf-bookmarks-list";
    wrap.style.display = "flex";
    wrap.style.flexDirection = "column";
    wrap.style.gap = "6px";

    const topBar = document.createElement("div");
    topBar.style.display = "flex";
    topBar.style.justifyContent = "space-between";
    topBar.style.alignItems = "center";
    topBar.style.marginBottom = "8px";

    const titleSpan = document.createElement("span");
    titleSpan.style.fontSize = "12px";
    titleSpan.style.fontWeight = "600";
    titleSpan.textContent = "Document Outline";

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

      const icon = ann.type === "highlight" ? "🖍️" : ann.type === "note" ? "📝" : "✒️";
      header.innerHTML = `<span>${icon} <strong>${ann.type}</strong></span><span style="color:#64748b;font-size:11px;">p.${ann.pageNumber}</span>`;
      card.appendChild(header);

      if (ann.contents) {
        const body = document.createElement("div");
        body.style.color = "#334155";
        body.style.fontSize = "11px";
        body.textContent = ann.contents;
        card.appendChild(body);
      }

      card.onclick = () => this.scrollToPage(ann.pageNumber);
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

      // Rotation & Sizing
      const isSideways = page.rotation === 90 || page.rotation === 270;
      const effW = (isSideways ? page.height : page.width) * zoom;
      const effH = (isSideways ? page.width : page.height) * zoom;

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
      pageWrap.appendChild(canvas);

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

    // 2. Render Text Blocks
    if (page.textBlocks) {
      page.textBlocks.forEach((tb) => {
        ctx.fillStyle = tb.fontColor || "#1e293b";
        const weight = tb.bold ? "bold " : "";
        ctx.font = `${weight}${tb.fontSize || 14}px Segoe UI, sans-serif`;
        ctx.textAlign = "left";
        ctx.fillText(tb.text, tb.x, tb.y);
      });
    }

    // 3. Render AcroForms Fields
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
    }

    // 4. Render Annotations & Highlights
    if (page.annotations) {
      page.annotations.forEach((ann) => {
        const [ax, ay, aw, ah] = ann.rect;
        if (ann.type === "highlight") {
          ctx.fillStyle = ann.color;
          ctx.fillRect(ax, ay, aw, ah);
        } else if (ann.type === "note") {
          ctx.fillStyle = ann.color;
          ctx.fillRect(ax, ay, aw, ah);
          ctx.fillStyle = "#ffffff";
          ctx.font = "14px Segoe UI";
          ctx.textAlign = "center";
          ctx.fillText("📝", ax + aw / 2, ay + ah / 2 + 5);
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
