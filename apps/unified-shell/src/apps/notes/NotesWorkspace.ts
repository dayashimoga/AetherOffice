import { NotesState, notesState } from "../../state/notesState";

export interface NotesWorkspaceCallbacks {
  onDirty?: () => void;
  onStatusChange?: (info: string) => void;
}

export class NotesWorkspace {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private sidebarEl: HTMLElement;
  private mainEl: HTMLElement;
  private editorPaneEl: HTMLElement;
  private lineGutterEl: HTMLElement;
  private textareaEl: HTMLTextAreaElement;
  private previewPaneEl: HTMLElement;
  private statusBarEl: HTMLElement;
  private state: NotesState;
  private callbacks: NotesWorkspaceCallbacks = {};

  constructor(containerOrId: HTMLElement | string, state: NotesState = notesState) {
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
    this.rootEl.className = "notes-workspace notes-app-root";
    this.rootEl.setAttribute("data-testid", "notes-workspace");
    this.rootEl.style.display = "flex";
    this.rootEl.style.width = "100%";
    this.rootEl.style.height = "100%";
    this.rootEl.style.backgroundColor = "var(--bg-primary, #ffffff)";
    this.rootEl.style.color = "var(--text-primary, #1e293b)";
    this.rootEl.style.overflow = "hidden";
    this.rootEl.style.fontFamily = "Segoe UI, -apple-system, sans-serif";

    // 1. Sidebar (Notes & Scratchpads)
    this.sidebarEl = document.createElement("aside");
    this.sidebarEl.className = "notes-sidebar";
    this.sidebarEl.setAttribute("data-testid", "notes-sidebar");
    this.sidebarEl.style.width = "220px";
    this.sidebarEl.style.height = "100%";
    this.sidebarEl.style.backgroundColor = "var(--bg-surface, #f8fafc)";
    this.sidebarEl.style.borderRight = "1px solid var(--border-color, #e2e8f0)";
    this.sidebarEl.style.display = "flex";
    this.sidebarEl.style.flexDirection = "column";
    this.sidebarEl.style.flexShrink = "0";

    // 2. Main Area (Toolbar + Editor / Preview Split + Status Bar)
    this.mainEl = document.createElement("main");
    this.mainEl.className = "notes-main";
    this.mainEl.style.flex = "1";
    this.mainEl.style.height = "100%";
    this.mainEl.style.display = "flex";
    this.mainEl.style.flexDirection = "column";
    this.mainEl.style.overflow = "hidden";

    // Sub-components
    this.editorPaneEl = document.createElement("div");
    this.lineGutterEl = document.createElement("div");
    this.textareaEl = document.createElement("textarea");
    this.previewPaneEl = document.createElement("div");
    this.statusBarEl = document.createElement("div");

    this.rootEl.appendChild(this.sidebarEl);
    this.rootEl.appendChild(this.mainEl);

    this.container.innerHTML = "";
    this.container.appendChild(this.rootEl);

    this.render();
  }

  public setCallbacks(callbacks: NotesWorkspaceCallbacks): void {
    this.callbacks = callbacks;
  }

  public getState(): NotesState {
    return this.state;
  }

  public getTextarea(): HTMLTextAreaElement {
    return this.textareaEl;
  }

  public render(): void {
    this.renderSidebar();
    this.renderMain();
  }

  // --- 1. Sidebar Rendering ---
  private renderSidebar(): void {
    this.sidebarEl.innerHTML = "";

    // Sidebar Header
    const head = document.createElement("div");
    head.style.padding = "10px 12px";
    head.style.display = "flex";
    head.style.alignItems = "center";
    head.style.justifyContent = "space-between";
    head.style.borderBottom = "1px solid var(--border-color, #e2e8f0)";

    const title = document.createElement("span");
    title.textContent = "SCRATCHPADS";
    title.style.fontSize = "11px";
    title.style.fontWeight = "600";
    title.style.color = "var(--text-secondary, #64748b)";
    title.style.letterSpacing = "0.05em";

    const newBtn = document.createElement("button");
    newBtn.className = "btn-new-scratchpad";
    newBtn.setAttribute("data-testid", "btn-new-scratchpad");
    newBtn.textContent = "+ New";
    newBtn.style.padding = "2px 8px";
    newBtn.style.fontSize = "11px";
    newBtn.style.borderRadius = "4px";
    newBtn.style.border = "1px solid var(--border-color, #cbd5e1)";
    newBtn.style.backgroundColor = "var(--bg-primary, #ffffff)";
    newBtn.style.color = "var(--text-primary, #334155)";
    newBtn.style.cursor = "pointer";
    newBtn.onclick = () => {
      this.state.addNote();
      this.render();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    };

    head.appendChild(title);
    head.appendChild(newBtn);
    this.sidebarEl.appendChild(head);

    // Notes List
    const list = document.createElement("div");
    list.className = "notes-list";
    list.setAttribute("data-testid", "notes-list");
    list.style.flex = "1";
    list.style.overflowY = "auto";
    list.style.padding = "8px 6px";
    list.style.display = "flex";
    list.style.flexDirection = "column";
    list.style.gap = "4px";

    this.state.notes.forEach((note) => {
      const card = document.createElement("div");
      card.className = `note-card-item ${note.id === this.state.activeNoteId ? "is-active" : ""}`;
      card.setAttribute("data-testid", "note-card");
      card.dataset.noteId = note.id;
      card.style.padding = "8px 10px";
      card.style.borderRadius = "4px";
      card.style.cursor = "pointer";
      card.style.display = "flex";
      card.style.alignItems = "center";
      card.style.justifyContent = "space-between";
      card.style.fontSize = "12px";
      card.style.transition = "background-color 0.15s ease";

      const isActive = note.id === this.state.activeNoteId;
      card.style.backgroundColor = isActive ? "var(--bg-primary, #ffffff)" : "transparent";
      card.style.border = isActive ? "1px solid var(--border-color, #cbd5e1)" : "1px solid transparent";
      card.style.fontWeight = isActive ? "600" : "normal";

      const labelWrap = document.createElement("div");
      labelWrap.style.display = "flex";
      labelWrap.style.alignItems = "center";
      labelWrap.style.gap = "6px";
      labelWrap.style.overflow = "hidden";

      const icon = document.createElement("span");
      icon.textContent = note.title.endsWith(".md") ? "📝" : "📄";

      const nameSpan = document.createElement("span");
      nameSpan.textContent = note.title;
      nameSpan.style.whiteSpace = "nowrap";
      nameSpan.style.overflow = "hidden";
      nameSpan.style.textOverflow = "ellipsis";
      nameSpan.title = "Double-click to rename";
      nameSpan.ondblclick = (e) => {
        e.stopPropagation();
        const input = document.createElement("input");
        input.type = "text";
        input.value = note.title;
        input.className = "input-rename-scratchpad";
        input.setAttribute("data-testid", "input-rename-scratchpad");
        input.style.fontSize = "12px";
        input.style.width = "120px";
        input.style.padding = "1px 4px";
        input.onblur = () => {
          if (input.value.trim()) {
            this.state.renameNote(note.id, input.value.trim());
          }
          this.render();
          if (this.callbacks.onDirty) this.callbacks.onDirty();
        };
        input.onkeydown = (ke) => {
          if (ke.key === "Enter") {
            input.blur();
          } else if (ke.key === "Escape") {
            this.render();
          }
        };
        labelWrap.replaceChild(input, nameSpan);
        input.focus();
        input.select();
      };
      labelWrap.appendChild(icon);
      labelWrap.appendChild(nameSpan);
      card.appendChild(labelWrap);

      // Delete button (visible if > 1 note)
      if (this.state.notes.length > 1) {
        const delBtn = document.createElement("button");
        delBtn.className = "btn-delete-scratchpad";
        delBtn.setAttribute("data-testid", "btn-delete-scratchpad");
        delBtn.innerHTML = "&times;";
        delBtn.style.border = "none";
        delBtn.style.background = "none";
        delBtn.style.cursor = "pointer";
        delBtn.style.fontSize = "14px";
        delBtn.style.color = "#94a3b8";
        delBtn.onclick = (e) => {
          e.stopPropagation();
          this.state.deleteNote(note.id);
          this.render();
          if (this.callbacks.onDirty) this.callbacks.onDirty();
        };
        card.appendChild(delBtn);
      }

      card.onclick = () => {
        this.state.selectNote(note.id);
        this.render();
      };

      list.appendChild(card);
    });

    this.sidebarEl.appendChild(list);
  }

  // --- 2. Main Workspace Rendering ---
  private renderMain(): void {
    this.mainEl.innerHTML = "";

    // Header Toolbar
    const toolbar = document.createElement("div");
    toolbar.className = "notes-toolbar";
    toolbar.style.display = "flex";
    toolbar.style.alignItems = "center";
    toolbar.style.justifyContent = "space-between";
    toolbar.style.padding = "6px 14px";
    toolbar.style.borderBottom = "1px solid var(--border-color, #e2e8f0)";
    toolbar.style.backgroundColor = "var(--bg-surface, #f8fafc)";
    toolbar.style.fontSize = "12px";

    const leftControls = document.createElement("div");
    leftControls.style.display = "flex";
    leftControls.style.alignItems = "center";
    leftControls.style.gap = "8px";

    // View Mode Toggle Group
    const modeGroup = document.createElement("div");
    modeGroup.style.display = "flex";
    modeGroup.style.border = "1px solid var(--border-color, #cbd5e1)";
    modeGroup.style.borderRadius = "4px";
    modeGroup.style.overflow = "hidden";

    const createModeBtn = (mode: "editor" | "split" | "preview", label: string, testId: string) => {
      const btn = document.createElement("button");
      btn.textContent = label;
      btn.setAttribute("data-testid", testId);
      btn.style.border = "none";
      btn.style.padding = "3px 10px";
      btn.style.fontSize = "11px";
      btn.style.cursor = "pointer";
      btn.style.backgroundColor = this.state.viewMode === mode ? "#0284c7" : "transparent";
      btn.style.color = this.state.viewMode === mode ? "#ffffff" : "var(--text-primary, #334155)";
      btn.onclick = () => {
        this.state.setViewMode(mode);
        this.renderMain();
      };
      return btn;
    };

    modeGroup.appendChild(createModeBtn("editor", "Editor", "btn-view-editor"));
    modeGroup.appendChild(createModeBtn("split", "Split", "btn-view-split"));
    modeGroup.appendChild(createModeBtn("preview", "Preview", "btn-view-preview"));
    leftControls.appendChild(modeGroup);

    // Font Toggle (Mono / Sans)
    const fontBtn = document.createElement("button");
    fontBtn.setAttribute("data-testid", "btn-font-toggle");
    fontBtn.textContent = this.state.fontFamily === "monospace" ? "Monospace" : "Sans";
    fontBtn.style.padding = "3px 8px";
    fontBtn.style.fontSize = "11px";
    fontBtn.style.borderRadius = "4px";
    fontBtn.style.border = "1px solid var(--border-color, #cbd5e1)";
    fontBtn.style.backgroundColor = "transparent";
    fontBtn.style.cursor = "pointer";
    fontBtn.onclick = () => {
      this.state.setFontFamily(this.state.fontFamily === "monospace" ? "sans" : "monospace");
      this.renderMain();
    };
    leftControls.appendChild(fontBtn);

    // Line Numbers Toggle
    const lineNumBtn = document.createElement("button");
    lineNumBtn.setAttribute("data-testid", "btn-toggle-linenums");
    lineNumBtn.textContent = this.state.showLineNumbers ? "Line # On" : "Line # Off";
    lineNumBtn.style.padding = "3px 8px";
    lineNumBtn.style.fontSize = "11px";
    lineNumBtn.style.borderRadius = "4px";
    lineNumBtn.style.border = "1px solid var(--border-color, #cbd5e1)";
    lineNumBtn.style.backgroundColor = "transparent";
    lineNumBtn.style.cursor = "pointer";
    lineNumBtn.onclick = () => {
      this.state.toggleLineNumbers();
      this.renderMain();
    };
    leftControls.appendChild(lineNumBtn);

    // Word Wrap Toggle
    const wrapBtn = document.createElement("button");
    wrapBtn.setAttribute("data-testid", "btn-toggle-wordwrap");
    wrapBtn.textContent = this.state.wordWrap ? "Wrap On" : "Wrap Off";
    wrapBtn.style.padding = "3px 8px";
    wrapBtn.style.fontSize = "11px";
    wrapBtn.style.borderRadius = "4px";
    wrapBtn.style.border = "1px solid var(--border-color, #cbd5e1)";
    wrapBtn.style.backgroundColor = "transparent";
    wrapBtn.style.cursor = "pointer";
    wrapBtn.onclick = () => {
      this.state.toggleWordWrap();
      this.renderMain();
    };
    leftControls.appendChild(wrapBtn);

    toolbar.appendChild(leftControls);

    // Right Controls (Insert Time/Date)
    const rightControls = document.createElement("div");
    rightControls.style.display = "flex";
    rightControls.style.alignItems = "center";
    rightControls.style.gap = "8px";

    const timeBtn = document.createElement("button");
    timeBtn.setAttribute("data-testid", "notes-btn-timedate");
    timeBtn.textContent = "⏱️ Time/Date (F5)";
    timeBtn.style.padding = "3px 8px";
    timeBtn.style.fontSize = "11px";
    timeBtn.style.borderRadius = "4px";
    timeBtn.style.border = "1px solid var(--border-color, #cbd5e1)";
    timeBtn.style.backgroundColor = "transparent";
    timeBtn.style.cursor = "pointer";
    timeBtn.onclick = () => {
      this.state.insertTimeDate();
      this.syncEditorContent();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    };
    rightControls.appendChild(timeBtn);

    toolbar.appendChild(rightControls);
    this.mainEl.appendChild(toolbar);

    // Optional Find & Replace Bar
    if (this.state.isFindBarVisible || this.state.isReplaceBarVisible) {
      const frBar = document.createElement("div");
      frBar.className = "notes-find-replace-bar";
      frBar.setAttribute("data-testid", "notes-find-replace-bar");
      frBar.style.display = "flex";
      frBar.style.alignItems = "center";
      frBar.style.flexWrap = "wrap";
      frBar.style.gap = "8px";
      frBar.style.padding = "6px 14px";
      frBar.style.backgroundColor = "var(--bg-surface, #f1f5f9)";
      frBar.style.borderBottom = "1px solid var(--border-color, #cbd5e1)";
      frBar.style.fontSize = "12px";

      const findInput = document.createElement("input");
      findInput.type = "text";
      findInput.placeholder = "Find...";
      findInput.className = "notes-input-find";
      findInput.setAttribute("data-testid", "notes-input-find");
      findInput.value = this.state.searchKeyword;
      findInput.style.padding = "3px 8px";
      findInput.style.borderRadius = "3px";
      findInput.style.border = "1px solid var(--border-color, #cbd5e1)";
      findInput.style.fontSize = "12px";
      findInput.oninput = () => {
        this.state.searchKeyword = findInput.value;
      };
      frBar.appendChild(findInput);

      const matchCaseCb = document.createElement("input");
      matchCaseCb.type = "checkbox";
      matchCaseCb.id = "notes-cb-match-case";
      matchCaseCb.setAttribute("data-testid", "notes-cb-match-case");
      const matchCaseLabel = document.createElement("label");
      matchCaseLabel.htmlFor = "notes-cb-match-case";
      matchCaseLabel.textContent = "Aa";
      matchCaseLabel.title = "Match Case";

      const regexCb = document.createElement("input");
      regexCb.type = "checkbox";
      regexCb.id = "notes-cb-regex";
      regexCb.setAttribute("data-testid", "notes-cb-regex");
      const regexLabel = document.createElement("label");
      regexLabel.htmlFor = "notes-cb-regex";
      regexLabel.textContent = ".*";
      regexLabel.title = "Regex";

      if (this.state.isReplaceBarVisible) {
        const replaceInput = document.createElement("input");
        replaceInput.type = "text";
        replaceInput.placeholder = "Replace with...";
        replaceInput.className = "notes-input-replace";
        replaceInput.setAttribute("data-testid", "notes-input-replace");
        replaceInput.style.padding = "3px 8px";
        replaceInput.style.borderRadius = "3px";
        replaceInput.style.border = "1px solid var(--border-color, #cbd5e1)";
        replaceInput.style.fontSize = "12px";
        frBar.appendChild(replaceInput);

        const btnReplace = document.createElement("button");
        btnReplace.textContent = "Replace";
        btnReplace.setAttribute("data-testid", "notes-btn-replace-one");
        btnReplace.style.padding = "3px 8px";
        btnReplace.style.cursor = "pointer";
        btnReplace.onclick = () => {
          this.state.replaceText(findInput.value, replaceInput.value, matchCaseCb.checked, regexCb.checked);
          this.syncEditorContent();
        };
        frBar.appendChild(btnReplace);

        const btnReplaceAll = document.createElement("button");
        btnReplaceAll.textContent = "Replace All";
        btnReplaceAll.setAttribute("data-testid", "notes-btn-replace-all");
        btnReplaceAll.style.padding = "3px 8px";
        btnReplaceAll.style.cursor = "pointer";
        btnReplaceAll.onclick = () => {
          this.state.replaceText(findInput.value, replaceInput.value, matchCaseCb.checked, regexCb.checked);
          this.syncEditorContent();
        };
        frBar.appendChild(btnReplaceAll);
      }

      frBar.appendChild(matchCaseCb);
      frBar.appendChild(matchCaseLabel);
      frBar.appendChild(regexCb);
      frBar.appendChild(regexLabel);

      const closeBtn = document.createElement("button");
      closeBtn.innerHTML = "&times;";
      closeBtn.setAttribute("data-testid", "notes-btn-close-find");
      closeBtn.style.border = "none";
      closeBtn.style.background = "none";
      closeBtn.style.cursor = "pointer";
      closeBtn.style.fontSize = "16px";
      closeBtn.style.color = "#64748b";
      closeBtn.onclick = () => {
        this.state.isFindBarVisible = false;
        this.state.isReplaceBarVisible = false;
        this.renderMain();
      };
      frBar.appendChild(closeBtn);

      this.mainEl.appendChild(frBar);
    }

    // Central Split Area
    const splitContainer = document.createElement("div");
    splitContainer.style.flex = "1";
    splitContainer.style.display = "flex";
    splitContainer.style.overflow = "hidden";
    splitContainer.style.position = "relative";

    // 1. Editor Pane
    if (this.state.viewMode === "editor" || this.state.viewMode === "split") {
      this.editorPaneEl = document.createElement("div");
      this.editorPaneEl.className = "notes-editor-pane";
      this.editorPaneEl.style.flex = "1";
      this.editorPaneEl.style.height = "100%";
      this.editorPaneEl.style.display = "flex";
      this.editorPaneEl.style.overflow = "hidden";
      this.editorPaneEl.style.borderRight = this.state.viewMode === "split" ? "1px solid var(--border-color, #e2e8f0)" : "none";

      // Line Number Gutter
      this.lineGutterEl = document.createElement("div");
      this.lineGutterEl.className = "notes-line-gutter";
      this.lineGutterEl.setAttribute("data-testid", "notes-line-gutter");
      this.lineGutterEl.style.display = this.state.showLineNumbers ? "block" : "none";
      this.lineGutterEl.style.width = "40px";
      this.lineGutterEl.style.padding = "16px 6px";
      this.lineGutterEl.style.textAlign = "right";
      this.lineGutterEl.style.color = "#94a3b8";
      this.lineGutterEl.style.backgroundColor = "var(--bg-surface, #f8fafc)";
      this.lineGutterEl.style.borderRight = "1px solid var(--border-color, #e2e8f0)";
      this.lineGutterEl.style.userSelect = "none";
      const effectiveSize = Math.round((this.state.fontSize * this.state.zoom) / 100);
      const ff = this.state.fontFamily;
      const effectiveFont =
        ff === "monospace" || ff === "Consolas" || ff === "JetBrains Mono" || ff === "Courier New"
          ? "'JetBrains Mono', 'Consolas', 'Courier New', monospace"
          : ff === "Serif"
          ? "'Georgia', 'Times New Roman', serif"
          : "'Inter', 'Segoe UI', -apple-system, sans-serif";

      this.lineGutterEl.style.fontFamily = effectiveFont;
      this.lineGutterEl.style.fontSize = `${effectiveSize}px`;
      this.lineGutterEl.style.lineHeight = "1.5";
      this.lineGutterEl.style.overflow = "hidden";

      // Textarea
      this.textareaEl = document.createElement("textarea");
      this.textareaEl.id = "notes-editor-textarea";
      this.textareaEl.className = "notes-editor-textarea";
      this.textareaEl.setAttribute("data-testid", "notes-editor-textarea");
      this.textareaEl.value = this.state.content;
      this.textareaEl.placeholder = "Type notes or Markdown here...";
      this.textareaEl.style.flex = "1";
      this.textareaEl.style.height = "100%";
      this.textareaEl.style.border = "none";
      this.textareaEl.style.outline = "none";
      this.textareaEl.style.padding = "16px";
      this.textareaEl.style.resize = "none";
      this.textareaEl.style.boxSizing = "border-box";
      this.textareaEl.style.backgroundColor = "transparent";
      this.textareaEl.style.color = "inherit";
      this.textareaEl.style.fontSize = `${effectiveSize}px`;
      this.textareaEl.style.lineHeight = "1.5";
      this.textareaEl.style.fontFamily = effectiveFont;
      this.textareaEl.style.whiteSpace = this.state.wordWrap ? "pre-wrap" : "pre";
      this.textareaEl.style.overflowWrap = this.state.wordWrap ? "break-word" : "normal";

      // Synchronize Scrolling between Textarea and Gutter
      this.textareaEl.onscroll = () => {
        this.lineGutterEl.scrollTop = this.textareaEl.scrollTop;
      };

      // Input & Cursor handling
      this.textareaEl.oninput = () => {
        this.state.setContent(this.textareaEl.value);
        this.updateLineNumbers();
        this.updateMarkdownPreview();
        this.updateStatusBar();
        if (this.callbacks.onDirty) this.callbacks.onDirty();
      };

      this.textareaEl.onkeydown = (e) => {
        if (e.key === "Tab") {
          e.preventDefault();
          const start = this.textareaEl.selectionStart;
          const end = this.textareaEl.selectionEnd;
          const val = this.textareaEl.value;
          const spaces = " ".repeat(this.state.tabSize);
          this.textareaEl.value = val.substring(0, start) + spaces + val.substring(end);
          this.textareaEl.selectionStart = this.textareaEl.selectionEnd = start + this.state.tabSize;
          this.state.setContent(this.textareaEl.value);
          this.updateLineNumbers();
          this.updateMarkdownPreview();
          this.updateStatusBar();
        } else if (e.key === "F5") {
          e.preventDefault();
          this.state.insertTimeDate();
          this.syncEditorContent();
          if (this.callbacks.onDirty) this.callbacks.onDirty();
        }
      };

      const updateCursor = () => {
        const selStart = this.textareaEl.selectionStart || 0;
        const textBefore = this.textareaEl.value.substring(0, selStart);
        const lines = textBefore.split("\n");
        const line = lines.length;
        const col = lines[lines.length - 1].length + 1;
        this.state.setCursor(line, col);
        this.updateStatusBar();
      };

      this.textareaEl.onkeyup = updateCursor;
      this.textareaEl.onclick = updateCursor;

      this.editorPaneEl.appendChild(this.lineGutterEl);
      this.editorPaneEl.appendChild(this.textareaEl);
      splitContainer.appendChild(this.editorPaneEl);
    }

    // 2. Markdown Preview Pane
    if (this.state.viewMode === "preview" || this.state.viewMode === "split") {
      this.previewPaneEl = document.createElement("div");
      this.previewPaneEl.className = "notes-markdown-preview";
      this.previewPaneEl.setAttribute("data-testid", "notes-markdown-preview");
      this.previewPaneEl.style.flex = "1";
      this.previewPaneEl.style.height = "100%";
      this.previewPaneEl.style.overflowY = "auto";
      this.previewPaneEl.style.padding = "20px 24px";
      this.previewPaneEl.style.boxSizing = "border-box";
      this.previewPaneEl.style.backgroundColor = "var(--bg-primary, #ffffff)";
      this.previewPaneEl.style.lineHeight = "1.6";
      splitContainer.appendChild(this.previewPaneEl);
    }

    this.mainEl.appendChild(splitContainer);

    // Bottom Status Bar
    this.statusBarEl = document.createElement("div");
    this.statusBarEl.className = "notes-status-bar";
    this.statusBarEl.setAttribute("data-testid", "notes-status-bar");
    this.statusBarEl.style.display = "flex";
    this.statusBarEl.style.alignItems = "center";
    this.statusBarEl.style.justifyContent = "space-between";
    this.statusBarEl.style.padding = "4px 16px";
    this.statusBarEl.style.borderTop = "1px solid var(--border-color, #e2e8f0)";
    this.statusBarEl.style.backgroundColor = "var(--bg-surface, #f8fafc)";
    this.statusBarEl.style.fontSize = "11px";
    this.statusBarEl.style.color = "var(--text-secondary, #64748b)";
    this.mainEl.appendChild(this.statusBarEl);

    this.updateLineNumbers();
    this.updateMarkdownPreview();
    this.updateStatusBar();
  }

  private updateLineNumbers(): void {
    if (!this.lineGutterEl) return;
    const stats = this.state.getStats();
    const lineNums = Array.from({ length: stats.lines }, (_, i) => i + 1).join("\n");
    this.lineGutterEl.textContent = lineNums;
  }

  private syncEditorContent(): void {
    if (this.textareaEl) {
      this.textareaEl.value = this.state.content;
    }
    this.updateLineNumbers();
    this.updateMarkdownPreview();
    this.updateStatusBar();
  }

  private updateMarkdownPreview(): void {
    if (!this.previewPaneEl) return;
    this.previewPaneEl.innerHTML = this.renderMarkdown(this.state.content);

    // Wire interactive task checkboxes in preview
    const cbs = this.previewPaneEl.querySelectorAll<HTMLInputElement>('input[type="checkbox"]');
    cbs.forEach((cb) => {
      cb.onclick = (e) => {
        e.stopPropagation();
        const lineIdx = parseInt(cb.dataset.line || "0", 10);
        this.toggleMarkdownCheckbox(lineIdx, cb.checked);
      };
    });
  }

  private toggleMarkdownCheckbox(lineIdx: number, checked: boolean): void {
    const lines = this.state.content.split("\n");
    if (lines[lineIdx] !== undefined) {
      if (checked) {
        lines[lineIdx] = lines[lineIdx].replace(/- \[\s*\]/, "- [x]");
      } else {
        lines[lineIdx] = lines[lineIdx].replace(/- \[[xX]\]/, "- [ ]");
      }
      this.state.setContent(lines.join("\n"));
      this.syncEditorContent();
      if (this.callbacks.onDirty) this.callbacks.onDirty();
    }
  }

  private updateStatusBar(): void {
    if (!this.statusBarEl) return;
    const stats = this.state.getStats();
    const line = this.state.cursorLine;
    const col = this.state.cursorCol;
    const info = `Ln ${line}, Col ${col}  •  ${stats.lines} lines  •  ${stats.words} words  •  ${stats.chars} chars  •  ${this.state.encoding}`;
    this.statusBarEl.textContent = info;
    if (this.callbacks.onStatusChange) {
      this.callbacks.onStatusChange(info);
    }
  }

  // --- Clean Client-Side Markdown Parser ---
  public renderMarkdown(raw: string): string {
    if (!raw.trim()) {
      return `<p style="color:#94a3b8;font-style:italic;">Live Markdown Preview will appear here as you type...</p>`;
    }

    const lines = raw.split("\n");
    let html = "";
    let inCodeBlock = false;
    let codeBuffer = "";
    let inTable = false;
    let tableBuffer: string[] = [];

    const flushTable = () => {
      if (tableBuffer.length > 0) {
        let tbl = `<table style="border-collapse:collapse;width:100%;margin:12px 0;">`;
        tableBuffer.forEach((row, rIdx) => {
          if (row.trim().match(/^\|?[-: ]+\|?$/)) return;
          const cells = row.split("|").filter((_c, i, a) => i !== 0 || a.length === 1).map((c) => c.trim());
          const tag = rIdx === 0 ? "th" : "td";
          tbl += `<tr>`;
          cells.forEach((cell) => {
            tbl += `<${tag} style="border:1px solid #cbd5e1;padding:6px 10px;text-align:left;">${this.parseInline(cell)}</${tag}>`;
          });
          tbl += `</tr>`;
        });
        tbl += `</table>`;
        html += tbl;
        tableBuffer = [];
      }
      inTable = false;
    };

    lines.forEach((line, idx) => {
      // Code Block Toggle
      if (line.trim().startsWith("```")) {
        if (inCodeBlock) {
          html += `<pre style="background:#f1f5f9;border:1px solid #cbd5e1;padding:12px;border-radius:6px;overflow-x:auto;"><code>${this.escapeHtml(codeBuffer)}</code></pre>`;
          codeBuffer = "";
          inCodeBlock = false;
        } else {
          if (inTable) flushTable();
          inCodeBlock = true;
        }
        return;
      }

      if (inCodeBlock) {
        codeBuffer += (codeBuffer ? "\n" : "") + line;
        return;
      }

      // Tables
      if (line.trim().startsWith("|") && line.trim().endsWith("|")) {
        inTable = true;
        tableBuffer.push(line);
        return;
      } else if (inTable) {
        flushTable();
      }

      // Horizontal Rule
      if (line.trim() === "---" || line.trim() === "***") {
        html += `<hr style="border:none;border-top:1px solid #cbd5e1;margin:16px 0;" />`;
        return;
      }

      // Headings
      if (line.startsWith("# ")) {
        html += `<h1 style="font-size:24px;font-weight:700;border-bottom:1px solid #e2e8f0;padding-bottom:6px;margin:16px 0 8px;">${this.parseInline(line.slice(2))}</h1>`;
      } else if (line.startsWith("## ")) {
        html += `<h2 style="font-size:20px;font-weight:600;border-bottom:1px solid #e2e8f0;padding-bottom:4px;margin:14px 0 8px;">${this.parseInline(line.slice(3))}</h2>`;
      } else if (line.startsWith("### ")) {
        html += `<h3 style="font-size:16px;font-weight:600;margin:12px 0 6px;">${this.parseInline(line.slice(4))}</h3>`;
      } else if (line.startsWith("> ")) {
        html += `<blockquote style="border-left:3px solid #0284c7;margin:8px 0;padding:4px 12px;background:#f8fafc;color:#475569;">${this.parseInline(line.slice(2))}</blockquote>`;
      } else if (line.trim().startsWith("- [ ]") || line.trim().startsWith("- [x]") || line.trim().startsWith("- [X]")) {
        // Interactive Checklist item
        const checked = line.trim().startsWith("- [x]") || line.trim().startsWith("- [X]");
        const text = line.replace(/^[ \t]*- \[[ xX]\]/, "").trim();
        html += `<div style="display:flex;align-items:center;gap:8px;margin:4px 0;">
          <input type="checkbox" data-line="${idx}" ${checked ? "checked" : ""} style="cursor:pointer;" />
          <span style="${checked ? "text-decoration:line-through;color:#94a3b8;" : ""}">${this.parseInline(text)}</span>
        </div>`;
      } else if (line.trim().startsWith("- ") || line.trim().startsWith("* ")) {
        html += `<li style="margin:2px 0 2px 20px;">${this.parseInline(line.replace(/^[ \t]*[-*] /, ""))}</li>`;
      } else if (line.trim() === "") {
        html += `<div style="height:8px;"></div>`;
      } else {
        html += `<p style="margin:4px 0;">${this.parseInline(line)}</p>`;
      }
    });

    if (inTable) flushTable();
    if (inCodeBlock) {
      html += `<pre style="background:#f1f5f9;border:1px solid #cbd5e1;padding:12px;border-radius:6px;overflow-x:auto;"><code>${this.escapeHtml(codeBuffer)}</code></pre>`;
    }

    return html;
  }

  private parseInline(text: string): string {
    let res = this.escapeHtml(text);
    // Bold
    res = res.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    // Italic
    res = res.replace(/\*(.+?)\*/g, "<em>$1</em>");
    // Strike
    res = res.replace(/~~(.+?)~~/g, "<del>$1</del>");
    // Inline code
    res = res.replace(/`([^`]+)`/g, "<code style=\"background:#f1f5f9;padding:2px 4px;border-radius:3px;font-family:monospace;font-size:0.9em;\">$1</code>");
    return res;
  }

  private escapeHtml(str: string): string {
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }
}
