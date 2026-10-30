export interface NoteItem {
  id: string;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
}

export type NotesViewMode = "editor" | "split" | "preview";
export type NotesFontFamily = "monospace" | "sans" | "JetBrains Mono" | "Consolas" | "Inter" | "Serif";

export class NotesState {
  public notes: NoteItem[] = [];
  public activeNoteId = "note-1";
  public content = "";
  public title = "Untitled 1.txt";
  public viewMode: NotesViewMode = "split";
  public showLineNumbers = true;
  public wordWrap = true;
  public fontFamily: NotesFontFamily = "monospace";
  public fontSize = 14;
  public tabSize = 4;
  public cursorLine = 1;
  public cursorCol = 1;
  public encoding = "UTF-8";

  public zoom = 100;
  public searchKeyword = "";
  public isFindBarVisible = false;
  public isReplaceBarVisible = false;

  private undoStack: string[] = [];
  private redoStack: string[] = [];
  private nextNoteCounter = 1;

  constructor() {
    this.resetToBlank("Untitled 1.txt");
  }

  public resetToBlank(title = "Untitled 1.txt"): void {
    const id = `note-${++this.nextNoteCounter}`;
    this.activeNoteId = id;
    this.title = title;
    this.content = "";
    this.notes = [
      {
        id,
        title,
        content: "",
        createdAt: Date.now(),
        updatedAt: Date.now(),
      },
    ];
    this.viewMode = "split";
    this.showLineNumbers = true;
    this.wordWrap = true;
    this.fontFamily = "monospace";
    this.fontSize = 14;
    this.tabSize = 4;
    this.cursorLine = 1;
    this.cursorCol = 1;
    this.zoom = 100;
    this.searchKeyword = "";
    this.isFindBarVisible = false;
    this.isReplaceBarVisible = false;
    this.undoStack = [];
    this.redoStack = [];
  }

  public getActiveNote(): NoteItem | undefined {
    return this.notes.find((n) => n.id === this.activeNoteId) || this.notes[0];
  }

  public getContent(): string {
    return this.content;
  }

  public renderMarkdownToHtml(raw: string): string {
    if (!raw.trim()) return "<p></p>";
    return raw
      .split("\n")
      .map((line) => {
        if (line.startsWith("# ")) return `<h1>${line.slice(2)}</h1>`;
        if (line.startsWith("## ")) return `<h2>${line.slice(3)}</h2>`;
        if (line.startsWith("### ")) return `<h3>${line.slice(4)}</h3>`;
        if (line.startsWith("- [x] ")) return `<p><input type="checkbox" checked disabled> ${line.slice(6)}</p>`;
        if (line.startsWith("- [ ] ")) return `<p><input type="checkbox" disabled> ${line.slice(6)}</p>`;
        if (line.startsWith("- ")) return `<li>${line.slice(2)}</li>`;
        return `<p>${line}</p>`;
      })
      .join("");
  }

  public setContent(newContent: string): void {
    if (this.content !== newContent) {
      this.undoStack.push(this.content);
      if (this.undoStack.length > 50) this.undoStack.shift();
      this.redoStack = [];
      this.content = newContent;

      const active = this.getActiveNote();
      if (active) {
        active.content = newContent;
        active.updatedAt = Date.now();
      }
    }
  }

  public setCursor(line: number, col: number): void {
    this.cursorLine = Math.max(1, line);
    this.cursorCol = Math.max(1, col);
  }

  public getStats(): { lines: number; words: number; chars: number } {
    const text = this.content;
    const lines = text === "" ? 1 : text.split("\n").length;
    const chars = text.length;
    const words = text.trim() === "" ? 0 : text.trim().split(/\s+/).length;
    return { lines, words, chars };
  }

  public addNote(title?: string, content = ""): NoteItem {
    const id = `note-${++this.nextNoteCounter}`;
    const defaultTitle = title || `Untitled ${this.notes.length + 1}.txt`;
    const note: NoteItem = {
      id,
      title: defaultTitle,
      content,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    this.notes.push(note);
    this.activeNoteId = id;
    this.title = defaultTitle;
    this.content = content;
    this.undoStack = [];
    this.redoStack = [];
    return note;
  }

  public deleteNote(id: string): boolean {
    if (this.notes.length <= 1) return false;
    const idx = this.notes.findIndex((n) => n.id === id);
    if (idx === -1) return false;

    this.notes.splice(idx, 1);
    if (this.activeNoteId === id) {
      const nextActive = this.notes[Math.max(0, idx - 1)];
      this.activeNoteId = nextActive.id;
      this.title = nextActive.title;
      this.content = nextActive.content;
      this.undoStack = [];
      this.redoStack = [];
    }
    return true;
  }

  public selectNote(id: string): boolean {
    const target = this.notes.find((n) => n.id === id);
    if (!target) return false;

    this.activeNoteId = target.id;
    this.title = target.title;
    this.content = target.content;
    this.undoStack = [];
    this.redoStack = [];
    return true;
  }

  public renameNote(id: string, newTitle: string): boolean {
    const note = this.notes.find((n) => n.id === id);
    if (!note) return false;
    note.title = newTitle;
    if (this.activeNoteId === id) {
      this.title = newTitle;
    }
    return true;
  }

  public setViewMode(mode: NotesViewMode): void {
    this.viewMode = mode;
  }

  public toggleLineNumbers(): boolean {
    this.showLineNumbers = !this.showLineNumbers;
    return this.showLineNumbers;
  }

  public toggleWordWrap(): boolean {
    this.wordWrap = !this.wordWrap;
    return this.wordWrap;
  }

  public setFontFamily(font: NotesFontFamily): void {
    this.fontFamily = font;
  }

  public setFontSize(size: number): void {
    this.fontSize = Math.min(36, Math.max(8, size));
  }

  public setTabSize(size: number): void {
    this.tabSize = Math.max(2, Math.min(8, size));
  }

  public insertTimeDate(): string {
    const now = new Date();
    const formatted = `${now.toLocaleTimeString()} ${now.toLocaleDateString()}`;
    this.setContent(this.content ? `${this.content}\n${formatted}` : formatted);
    return formatted;
  }

  public transformText(
    type: "upper" | "lower" | "title" | "sort_lines" | "sort_az" | "sort_za" | "reverse_lines" | "remove_duplicates" | "trim_whitespace"
  ): void {
    let result = this.content;
    switch (type) {
      case "upper":
        result = this.content.toUpperCase();
        break;
      case "lower":
        result = this.content.toLowerCase();
        break;
      case "title":
        result = this.content.replace(/\b\w/g, (c) => c.toUpperCase());
        break;
      case "sort_lines":
      case "sort_az":
        result = this.content.split("\n").sort().join("\n");
        break;
      case "sort_za":
      case "reverse_lines":
        result = this.content.split("\n").sort().reverse().join("\n");
        break;
      case "remove_duplicates":
        result = this.content
          .split("\n")
          .filter((line, idx, arr) => arr.indexOf(line) === idx)
          .join("\n");
        break;
      case "trim_whitespace":
        result = this.content
          .split("\n")
          .map((line) => line.trimEnd())
          .join("\n");
        break;
    }
    this.setContent(result);
  }

  public insertMarkdownSnippet(
    kind: "h1" | "h2" | "h3" | "bold" | "italic" | "code" | "checklist" | "table" | "quote" | "inline_code" | "link" | "hr"
  ): void {
    let snippet = "";
    switch (kind) {
      case "h1":
        snippet = "\n# Heading 1\n";
        break;
      case "h2":
        snippet = "\n## Heading 2\n";
        break;
      case "h3":
        snippet = "\n### Heading 3\n";
        break;
      case "bold":
        snippet = " **bold text** ";
        break;
      case "italic":
        snippet = " *italic text* ";
        break;
      case "inline_code":
        snippet = " `code` ";
        break;
      case "code":
        snippet = "\n```ts\nconsole.log('code');\n```\n";
        break;
      case "checklist":
        snippet = "\n- [ ] Task item\n";
        break;
      case "table":
        snippet = "\n| Item | Description | Status |\n|---|---|---|\n| Task 1 | Implement module | Done |\n";
        break;
      case "quote":
        snippet = "\n> Important note\n";
        break;
      case "link":
        snippet = " [Link](https://example.com) ";
        break;
      case "hr":
        snippet = "\n\n---\n\n";
        break;
    }
    this.setContent(this.content ? `${this.content}${snippet}` : snippet.trimStart());
  }

  public goToLine(line: number): number {
    const lines = this.content.split("\n");
    const targetLine = Math.max(1, Math.min(lines.length, line));
    let offset = 0;
    for (let i = 0; i < targetLine - 1; i++) {
      offset += lines[i].length + 1; // +1 for newline
    }
    this.setCursor(targetLine, 1);
    return offset;
  }

  public zoomIn(): number {
    this.zoom = Math.min(300, this.zoom + 10);
    return this.zoom;
  }

  public zoomOut(): number {
    this.zoom = Math.max(50, this.zoom - 10);
    return this.zoom;
  }

  public zoom100(): number {
    this.zoom = 100;
    return this.zoom;
  }

  public replaceText(findText: string, replaceText: string, matchCase = false, useRegex = false): number {
    if (!findText) return 0;
    try {
      let regex: RegExp;
      if (useRegex) {
        regex = new RegExp(findText, matchCase ? "g" : "gi");
      } else {
        const escaped = findText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        regex = new RegExp(escaped, matchCase ? "g" : "gi");
      }
      const matches = this.content.match(regex);
      const count = matches ? matches.length : 0;
      if (count > 0) {
        this.setContent(this.content.replace(regex, replaceText));
      }
      return count;
    } catch {
      return 0;
    }
  }

  public canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  public canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  public undo(): boolean {
    if (this.undoStack.length === 0) return false;
    const prev = this.undoStack.pop()!;
    this.redoStack.push(this.content);
    this.content = prev;
    const active = this.getActiveNote();
    if (active) active.content = prev;
    return true;
  }

  public redo(): boolean {
    if (this.redoStack.length === 0) return false;
    const next = this.redoStack.pop()!;
    this.undoStack.push(this.content);
    this.content = next;
    const active = this.getActiveNote();
    if (active) active.content = next;
    return true;
  }

  public execCommand(action: string, payload?: unknown): boolean {
    switch (action) {
      case "NewNote":
      case "AddNote":
        this.addNote(typeof payload === "string" ? payload : undefined);
        return true;
      case "DeleteNote":
        return this.deleteNote(typeof payload === "string" ? payload : this.activeNoteId);
      case "SelectNote":
        return typeof payload === "string" ? this.selectNote(payload) : false;
      case "RenameNote":
        if (payload && typeof payload === "object") {
          const { id, title } = payload as { id: string; title: string };
          return this.renameNote(id, title);
        }
        return false;
      case "SetNotesViewMode":
      case "SetViewMode":
        if (payload === "editor" || payload === "split" || payload === "preview") {
          this.setViewMode(payload);
          return true;
        }
        return false;
      case "ToggleLineNumbers":
        this.toggleLineNumbers();
        return true;
      case "ToggleWordWrap":
        this.toggleWordWrap();
        return true;
      case "SetFontFamily":
        this.setFontFamily(payload as NotesFontFamily);
        return true;
      case "SetFontSize":
        if (typeof payload === "number") {
          this.setFontSize(payload);
          return true;
        }
        return false;
      case "InsertTimeDate":
        this.insertTimeDate();
        return true;
      case "TransformText":
        if (typeof payload === "string") {
          this.transformText(payload as any);
          return true;
        }
        return false;
      case "InsertMarkdownSnippet":
        if (typeof payload === "string") {
          this.insertMarkdownSnippet(payload as any);
          return true;
        }
        return false;
      case "ReplaceText":
        if (payload && typeof payload === "object") {
          const { find, replace, matchCase, useRegex } = payload as any;
          return this.replaceText(find, replace, !!matchCase, !!useRegex) > 0;
        }
        return false;
      case "Find":
        this.isFindBarVisible = true;
        return true;
      case "Replace":
        this.isReplaceBarVisible = true;
        return true;
      case "CloseFind":
        this.isFindBarVisible = false;
        return true;
      case "CloseReplace":
        this.isReplaceBarVisible = false;
        return true;
      case "GoToLine":
        if (typeof payload === "number") {
          this.goToLine(payload);
          return true;
        }
        return false;
      case "ZoomIn":
        this.zoomIn();
        return true;
      case "ZoomOut":
        this.zoomOut();
        return true;
      case "Zoom100":
        this.zoom100();
        return true;
      case "SaveDocumentMd": {
        const active = this.getActiveNote();
        if (active && !active.title.endsWith(".md")) {
          active.title = active.title.replace(/\.[^/.]+$/, "") + ".md";
          this.title = active.title;
        }
        return true;
      }
      case "Undo":
        return this.undo();
      case "Redo":
        return this.redo();
      default:
        return false;
    }
  }
}

export const notesState = new NotesState();
