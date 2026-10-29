import { SlideAnimation, SlideShape, SlidesState, slidesState } from "../state/slidesState";

export class AnimationPane {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private headerEl: HTMLElement;
  private toolbarEl: HTMLElement;
  private timingControlsEl: HTMLElement;
  private listEl: HTMLElement;
  private state: SlidesState;
  private visible = false;
  private onDirtyCallback?: () => void;
  private onPlayAllCallback?: () => void;
  private onPlaySelectedCallback?: (anim: SlideAnimation) => void;

  constructor(containerOrId: HTMLElement | string, state: SlidesState = slidesState) {
    this.state = state;
    if (typeof containerOrId === "string") {
      let el = document.getElementById(containerOrId);
      if (!el) {
        el = document.createElement("div");
        el.id = containerOrId;
        const main = document.querySelector(".workspace-layout") || document.body;
        main.appendChild(el);
      }
      this.container = el;
    } else {
      this.container = containerOrId;
    }

    this.rootEl = document.createElement("div");
    this.rootEl.className = "animation-pane";
    this.rootEl.style.width = "300px";
    this.rootEl.style.height = "100%";
    this.rootEl.style.backgroundColor = "#ffffff";
    this.rootEl.style.borderLeft = "1px solid #cbd5e1";
    this.rootEl.style.boxShadow = "-2px 0 8px rgba(0, 0, 0, 0.05)";
    this.rootEl.style.display = "none";
    this.rootEl.style.flexDirection = "column";
    this.rootEl.style.fontFamily = "Segoe UI, sans-serif";
    this.rootEl.style.zIndex = "40";
    this.rootEl.style.overflow = "hidden";

    // 1. Header
    this.headerEl = document.createElement("div");
    this.headerEl.className = "animation-pane-header";
    this.headerEl.style.display = "flex";
    this.headerEl.style.justifyContent = "space-between";
    this.headerEl.style.alignItems = "center";
    this.headerEl.style.padding = "8px 12px";
    this.headerEl.style.backgroundColor = "#f1f5f9";
    this.headerEl.style.borderBottom = "1px solid #e2e8f0";

    const titleEl = document.createElement("div");
    titleEl.style.fontWeight = "600";
    titleEl.style.fontSize = "13px";
    titleEl.style.color = "#1e293b";
    titleEl.innerHTML = "🎞️ Animation Pane";
    this.headerEl.appendChild(titleEl);

    const closeBtn = document.createElement("button");
    closeBtn.className = "anim-pane-close-btn";
    closeBtn.innerHTML = "✕";
    closeBtn.style.border = "none";
    closeBtn.style.background = "none";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.fontSize = "14px";
    closeBtn.style.color = "#64748b";
    closeBtn.onclick = () => this.close();
    this.headerEl.appendChild(closeBtn);
    this.rootEl.appendChild(this.headerEl);

    // 2. Action Toolbar
    this.toolbarEl = document.createElement("div");
    this.toolbarEl.className = "animation-pane-toolbar";
    this.toolbarEl.style.display = "flex";
    this.toolbarEl.style.flexWrap = "wrap";
    this.toolbarEl.style.gap = "4px";
    this.toolbarEl.style.padding = "6px 12px";
    this.toolbarEl.style.backgroundColor = "#f8fafc";
    this.toolbarEl.style.borderBottom = "1px solid #e2e8f0";

    const playAllBtn = this.createToolButton("Play All", "▶ Play All", "anim-btn-play-all", () => {
      if (this.onPlayAllCallback) this.onPlayAllCallback();
    });
    const playSelBtn = this.createToolButton("Play", "▶ Play", "anim-btn-play-sel", () => {
      const sel = this.state.getSelectedAnimation();
      if (sel && this.onPlaySelectedCallback) {
        this.onPlaySelectedCallback(sel);
      }
    });
    const upBtn = this.createToolButton("Move Up", "▲", "anim-btn-move-up", () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        this.state.reorderAnimation(sel.id, "up");
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    });
    const downBtn = this.createToolButton("Move Down", "▼", "anim-btn-move-down", () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        this.state.reorderAnimation(sel.id, "down");
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    });
    const removeBtn = this.createToolButton("Remove", "🗑", "anim-btn-remove", () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        this.state.removeAnimation(sel.id);
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    });

    this.toolbarEl.appendChild(playAllBtn);
    this.toolbarEl.appendChild(playSelBtn);
    this.toolbarEl.appendChild(upBtn);
    this.toolbarEl.appendChild(downBtn);
    this.toolbarEl.appendChild(removeBtn);
    this.rootEl.appendChild(this.toolbarEl);

    // 3. Timing Controls Row
    this.timingControlsEl = document.createElement("div");
    this.timingControlsEl.className = "animation-pane-timing";
    this.timingControlsEl.style.display = "flex";
    this.timingControlsEl.style.flexDirection = "column";
    this.timingControlsEl.style.gap = "6px";
    this.timingControlsEl.style.padding = "8px 12px";
    this.timingControlsEl.style.borderBottom = "1px solid #e2e8f0";
    this.timingControlsEl.style.fontSize = "12px";

    const triggerRow = document.createElement("div");
    triggerRow.style.display = "flex";
    triggerRow.style.alignItems = "center";
    triggerRow.style.justifyContent = "space-between";
    const triggerLabel = document.createElement("span");
    triggerLabel.textContent = "Start:";
    triggerLabel.style.color = "#475569";
    const triggerSelect = document.createElement("select");
    triggerSelect.className = "anim-select-trigger";
    triggerSelect.style.padding = "2px 6px";
    triggerSelect.style.borderRadius = "4px";
    triggerSelect.style.border = "1px solid #cbd5e1";
    triggerSelect.style.fontSize = "11px";

    const optClick = document.createElement("option");
    optClick.value = "click";
    optClick.textContent = "On Click";
    const optWith = document.createElement("option");
    optWith.value = "withPrev";
    optWith.textContent = "With Previous";
    const optAfter = document.createElement("option");
    optAfter.value = "afterPrev";
    optAfter.textContent = "After Previous";
    triggerSelect.appendChild(optClick);
    triggerSelect.appendChild(optWith);
    triggerSelect.appendChild(optAfter);

    triggerSelect.onchange = () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        this.state.setAnimationTrigger(sel.id, triggerSelect.value as any);
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    };
    triggerRow.appendChild(triggerLabel);
    triggerRow.appendChild(triggerSelect);
    this.timingControlsEl.appendChild(triggerRow);

    const timingRow = document.createElement("div");
    timingRow.style.display = "flex";
    timingRow.style.justifyContent = "space-between";
    timingRow.style.gap = "8px";

    const durWrap = document.createElement("div");
    durWrap.style.display = "flex";
    durWrap.style.alignItems = "center";
    durWrap.style.gap = "4px";
    durWrap.innerHTML = `<span style="color:#475569;">Duration:</span>`;
    const durInput = document.createElement("input");
    durInput.className = "anim-input-duration";
    durInput.type = "number";
    durInput.min = "0.1";
    durInput.max = "10";
    durInput.step = "0.25";
    durInput.value = "0.5";
    durInput.style.width = "52px";
    durInput.style.padding = "2px 4px";
    durInput.style.borderRadius = "4px";
    durInput.style.border = "1px solid #cbd5e1";
    durInput.onchange = () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        const val = parseFloat(durInput.value) || 0.5;
        this.state.setAnimationDuration(sel.id, val);
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    };
    durWrap.appendChild(durInput);

    const delayWrap = document.createElement("div");
    delayWrap.style.display = "flex";
    delayWrap.style.alignItems = "center";
    delayWrap.style.gap = "4px";
    delayWrap.innerHTML = `<span style="color:#475569;">Delay:</span>`;
    const delayInput = document.createElement("input");
    delayInput.className = "anim-input-delay";
    delayInput.type = "number";
    delayInput.min = "0";
    delayInput.max = "10";
    delayInput.step = "0.25";
    delayInput.value = "0";
    delayInput.style.width = "52px";
    delayInput.style.padding = "2px 4px";
    delayInput.style.borderRadius = "4px";
    delayInput.style.border = "1px solid #cbd5e1";
    delayInput.onchange = () => {
      const sel = this.state.getSelectedAnimation();
      if (sel) {
        const val = parseFloat(delayInput.value) || 0;
        this.state.setAnimationDelay(sel.id, val);
        this.refresh();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    };
    delayWrap.appendChild(delayInput);

    timingRow.appendChild(durWrap);
    timingRow.appendChild(delayWrap);
    this.timingControlsEl.appendChild(timingRow);
    this.rootEl.appendChild(this.timingControlsEl);

    // 4. Timeline Animation List
    this.listEl = document.createElement("div");
    this.listEl.className = "animation-pane-list";
    this.listEl.id = "animation-list";
    this.listEl.style.flex = "1";
    this.listEl.style.overflowY = "auto";
    this.listEl.style.padding = "8px 12px";
    this.rootEl.appendChild(this.listEl);

    this.container.appendChild(this.rootEl);
    this.refresh();
  }

  private createToolButton(title: string, text: string, className: string, onClick: () => void): HTMLButtonElement {
    const btn = document.createElement("button");
    btn.className = `anim-tool-btn ${className}`;
    btn.title = title;
    btn.textContent = text;
    btn.style.padding = "4px 8px";
    btn.style.fontSize = "11px";
    btn.style.border = "1px solid #cbd5e1";
    btn.style.borderRadius = "4px";
    btn.style.backgroundColor = "#ffffff";
    btn.style.cursor = "pointer";
    btn.style.transition = "background-color 0.15s ease";
    btn.onmouseenter = () => (btn.style.backgroundColor = "#e2e8f0");
    btn.onmouseleave = () => (btn.style.backgroundColor = "#ffffff");
    btn.onclick = onClick;
    return btn;
  }

  public setOnDirty(cb: () => void) {
    this.onDirtyCallback = cb;
  }

  public setCallbacks(callbacks: {
    onPlayAll?: () => void;
    onPlaySelected?: (anim: SlideAnimation) => void;
    onDirty?: () => void;
  }) {
    if (callbacks.onPlayAll) this.onPlayAllCallback = callbacks.onPlayAll;
    if (callbacks.onPlaySelected) this.onPlaySelectedCallback = callbacks.onPlaySelected;
    if (callbacks.onDirty) this.onDirtyCallback = callbacks.onDirty;
  }

  public open() {
    this.visible = true;
    this.rootEl.style.display = "flex";
    this.refresh();
  }

  public close() {
    this.visible = false;
    this.rootEl.style.display = "none";
  }

  public toggle(): boolean {
    if (this.visible) {
      this.close();
    } else {
      this.open();
    }
    return this.visible;
  }

  public isOpen(): boolean {
    return this.visible;
  }

  public getRoot(): HTMLElement {
    return this.rootEl;
  }

  public refresh() {
    const anims = this.state.getSlideAnimations();
    const selAnim = this.state.getSelectedAnimation();
    this.listEl.innerHTML = "";

    // Sync timing controls with selected animation
    const triggerSelect = this.rootEl.querySelector(".anim-select-trigger") as HTMLSelectElement | null;
    const durInput = this.rootEl.querySelector(".anim-input-duration") as HTMLInputElement | null;
    const delayInput = this.rootEl.querySelector(".anim-input-delay") as HTMLInputElement | null;
    if (selAnim) {
      if (triggerSelect) triggerSelect.value = selAnim.trigger;
      if (durInput) durInput.value = selAnim.duration.toString();
      if (delayInput) delayInput.value = selAnim.delay.toString();
    }

    if (anims.length === 0) {
      const emptyNotice = document.createElement("div");
      emptyNotice.className = "anim-empty-notice";
      emptyNotice.style.padding = "24px 12px";
      emptyNotice.style.textAlign = "center";
      emptyNotice.style.color = "#94a3b8";
      emptyNotice.style.fontSize = "12px";
      emptyNotice.innerHTML = `No animations on this slide.<br><span style="font-size:11px;color:#64748b;">Select a shape and click an effect in the Animations tab to add one.</span>`;
      this.listEl.appendChild(emptyNotice);
      return;
    }

    const currentSlide = this.state.getCurrentSlide();

    anims.forEach((anim, idx) => {
      const targetShape = currentSlide.shapes.find((s: SlideShape) => s.id === anim.shapeId);
      const isSelected = selAnim?.id === anim.id;

      const item = document.createElement("div");
      item.className = `anim-item ${isSelected ? "is-selected" : ""}`;
      item.dataset.animId = anim.id;
      item.style.display = "flex";
      item.style.alignItems = "center";
      item.style.justifyContent = "space-between";
      item.style.padding = "6px 8px";
      item.style.marginBottom = "4px";
      item.style.borderRadius = "4px";
      item.style.border = isSelected ? "1px solid #2563eb" : "1px solid #e2e8f0";
      item.style.backgroundColor = isSelected ? "#eff6ff" : "#ffffff";
      item.style.cursor = "pointer";
      item.style.fontSize = "12px";

      // Left info: order badge, trigger icon, effect badge, shape name
      const leftWrap = document.createElement("div");
      leftWrap.style.display = "flex";
      leftWrap.style.alignItems = "center";
      leftWrap.style.gap = "6px";
      leftWrap.style.overflow = "hidden";

      const orderBadge = document.createElement("span");
      orderBadge.style.width = "18px";
      orderBadge.style.height = "18px";
      orderBadge.style.display = "inline-flex";
      orderBadge.style.alignItems = "center";
      orderBadge.style.justifyContent = "center";
      orderBadge.style.backgroundColor = isSelected ? "#2563eb" : "#e2e8f0";
      orderBadge.style.color = isSelected ? "#ffffff" : "#475569";
      orderBadge.style.borderRadius = "50%";
      orderBadge.style.fontSize = "10px";
      orderBadge.style.fontWeight = "bold";
      orderBadge.textContent = `${idx + 1}`;

      const triggerIcon = document.createElement("span");
      triggerIcon.title = anim.trigger === "click" ? "On Click" : anim.trigger === "withPrev" ? "With Previous" : "After Previous";
      triggerIcon.textContent = anim.trigger === "click" ? "🖱️" : "⏱️";
      triggerIcon.style.fontSize = "11px";

      const effectBadge = document.createElement("span");
      const effectColor = anim.type === "entrance" ? "#16a34a" : anim.type === "emphasis" ? "#ca8a04" : "#dc2626";
      effectBadge.style.color = effectColor;
      effectBadge.style.fontWeight = "600";
      effectBadge.textContent = anim.effect;

      const shapeLabel = document.createElement("span");
      shapeLabel.style.color = "#64748b";
      shapeLabel.style.fontSize = "11px";
      shapeLabel.style.whiteSpace = "nowrap";
      shapeLabel.style.textOverflow = "ellipsis";
      shapeLabel.style.overflow = "hidden";
      shapeLabel.style.maxWidth = "90px";
      shapeLabel.textContent = targetShape
        ? targetShape.text
          ? `"${targetShape.text.slice(0, 14)}..."`
          : targetShape.type
        : anim.shapeId;

      leftWrap.appendChild(orderBadge);
      leftWrap.appendChild(triggerIcon);
      leftWrap.appendChild(effectBadge);
      leftWrap.appendChild(shapeLabel);

      // Right info: Duration bar
      const rightWrap = document.createElement("div");
      rightWrap.style.display = "flex";
      rightWrap.style.alignItems = "center";
      rightWrap.style.gap = "4px";

      const durBar = document.createElement("div");
      durBar.style.width = `${Math.min(48, Math.max(16, anim.duration * 24))}px`;
      durBar.style.height = "6px";
      durBar.style.backgroundColor = effectColor;
      durBar.style.borderRadius = "3px";
      durBar.title = `Duration: ${anim.duration}s, Delay: ${anim.delay}s`;

      const durText = document.createElement("span");
      durText.style.fontSize = "10px";
      durText.style.color = "#94a3b8";
      durText.textContent = `${anim.duration}s`;

      rightWrap.appendChild(durBar);
      rightWrap.appendChild(durText);

      item.appendChild(leftWrap);
      item.appendChild(rightWrap);

      item.onclick = () => {
        this.state.selectAnimation(anim.id);
        this.refresh();
      };

      item.ondblclick = () => {
        if (this.onPlaySelectedCallback) {
          this.onPlaySelectedCallback(anim);
        }
      };

      this.listEl.appendChild(item);
    });
  }
}
