export class KeyboardShortcuts {
  private onActionCallback?: (action: string) => void;
  private onAltKeytipCallback?: (code: string) => void;
  private onToggleAltCallback?: () => boolean;
  private altActive = false;

  constructor() {
    this.attach();
  }

  public setCallbacks(callbacks: {
    onAction?: (action: string) => void;
    onToggleAlt?: () => boolean;
    onAltKeytip?: (code: string) => void;
  }) {
    this.onActionCallback = callbacks.onAction;
    this.onToggleAltCallback = callbacks.onToggleAlt;
    this.onAltKeytipCallback = callbacks.onAltKeytip;
  }

  private attach() {
    window.addEventListener("keydown", (e: KeyboardEvent) => this.handleKeyDown(e));
  }

  public handleKeyDown(e: KeyboardEvent) {
    // 1. Alt Keytip toggle
    if (e.key === "Alt") {
      e.preventDefault();
      if (this.onToggleAltCallback) {
        this.altActive = this.onToggleAltCallback();
      }
      return;
    }

      // If Alt Keytips are active and an alphanumeric key is pressed
      if (this.altActive && e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        if (this.onAltKeytipCallback) {
          this.onAltKeytipCallback(e.key.toUpperCase());
        }
        this.altActive = false;
        return;
      }

      // Escape exits Alt Keytips
      if (e.key === "Escape" && this.altActive) {
        e.preventDefault();
        this.altActive = false;
        if (this.onToggleAltCallback) {
          this.onToggleAltCallback();
        }
        return;
      }

      // 2. Control / Command Accelerators
      const isCtrl = e.ctrlKey || e.metaKey;

      if (isCtrl && !e.altKey) {
        const k = e.key.toLowerCase();
        switch (k) {
          case "c":
            this.emit("Copy");
            break;
          case "v":
            this.emit("Paste");
            break;
          case "x":
            this.emit("Cut");
            break;
          case "z":
            if (e.shiftKey) {
              this.emit("Redo");
            } else {
              this.emit("Undo");
            }
            e.preventDefault();
            break;
          case "y":
            this.emit("Redo");
            e.preventDefault();
            break;
          case "s":
            e.preventDefault();
            if (e.shiftKey) {
              this.emit("SaveAs");
            } else {
              this.emit("Save");
            }
            break;
          case "o":
            e.preventDefault();
            this.emit("Open");
            break;
          case "n":
            e.preventDefault();
            this.emit("New");
            break;
          case "p":
            e.preventDefault();
            this.emit("Print");
            break;
          case "w":
            e.preventDefault();
            this.emit("Close");
            break;
          case "f":
            e.preventDefault();
            this.emit("Find");
            break;
          case "h":
            e.preventDefault();
            this.emit("Replace");
            break;
          case "tab":
            e.preventDefault();
            if (e.shiftKey) {
              this.emit("PrevTab");
            } else {
              this.emit("NextTab");
            }
            break;
        }
      }

      // 3. Excel Function Keys (F2, F4) & PowerPoint Slide Show (F5, Shift+F5)
      if (!isCtrl && !e.altKey) {
        if (e.shiftKey && e.key === "F5") {
          e.preventDefault();
          this.emit("StartShowCurrent");
        } else if (!e.shiftKey) {
          if (e.key === "F2") {
            e.preventDefault();
            this.emit("CellEdit");
          } else if (e.key === "F4") {
            e.preventDefault();
            this.emit("ToggleF4");
          } else if (e.key === "F5") {
            e.preventDefault();
            this.emit("StartShowBeginning");
          }
        }
      }
    }

  private emit(action: string) {
    if (this.onActionCallback) {
      this.onActionCallback(action);
    }
  }
}

