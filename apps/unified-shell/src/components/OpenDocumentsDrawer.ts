import { DocumentType } from "../ipc";

export interface OpenDocEntry {
  id: string;
  docType: DocumentType;
  title: string;
  isDirty: boolean;
  filePath?: string;
  lastAutoSavedMs: number;
}

export class OpenDocumentsDrawer {
  private container: HTMLElement;
  private isOpen = false;
  private entries: OpenDocEntry[] = [];
  private onSelectDocCallback?: (id: string, docType: DocumentType) => void;
  private onPopoutDocCallback?: (entry: OpenDocEntry) => void;
  private onCloseDocCallback?: (id: string) => void;

  private static STORAGE_KEY = "aether_open_documents_registry";

  constructor(parentId = "app") {
    const parent = document.getElementById(parentId) || document.body;
    let el = document.getElementById("open-documents-drawer");
    if (!el) {
      el = document.createElement("div");
      el.id = "open-documents-drawer";
      el.className = "open-documents-drawer";
      parent.appendChild(el);
    }
    this.container = el;
    this.loadFromStorage();
    this.render();
  }

  public setCallbacks(callbacks: {
    onSelectDoc?: (id: string, docType: DocumentType) => void;
    onPopoutDoc?: (entry: OpenDocEntry) => void;
    onCloseDoc?: (id: string) => void;
  }) {
    this.onSelectDocCallback = callbacks.onSelectDoc;
    this.onPopoutDocCallback = callbacks.onPopoutDoc;
    this.onCloseDocCallback = callbacks.onCloseDoc;
  }

  public toggle(): boolean {
    this.isOpen = !this.isOpen;
    this.render();
    return this.isOpen;
  }

  public open() {
    this.isOpen = true;
    this.render();
  }

  public close() {
    this.isOpen = false;
    this.render();
  }

  public getIsOpen(): boolean {
    return this.isOpen;
  }

  public getEntries(): OpenDocEntry[] {
    return [...this.entries];
  }

  public registerDocument(entry: OpenDocEntry, contentSnapshot?: string) {
    const idx = this.entries.findIndex((e) => e.id === entry.id);
    const updated: OpenDocEntry = {
      ...entry,
      lastAutoSavedMs: Date.now(),
    };
    if (idx >= 0) {
      this.entries[idx] = updated;
    } else {
      this.entries.push(updated);
    }
    if (contentSnapshot !== undefined && typeof localStorage !== "undefined") {
      try {
        localStorage.setItem(`aether_autosave_${entry.id}`, contentSnapshot);
      } catch (err) {
        console.warn("Auto-save storage failed:", err);
      }
    }
    this.saveToStorage();
    if (this.isOpen) {
      this.render();
    }
  }

  public unregisterDocument(id: string) {
    this.entries = this.entries.filter((e) => e.id !== id);
    if (typeof localStorage !== "undefined") {
      try {
        localStorage.removeItem(`aether_autosave_${id}`);
      } catch {
        // ignore
      }
    }
    this.saveToStorage();
    if (this.isOpen) {
      this.render();
    }
  }

  public setDirty(id: string, isDirty: boolean) {
    const item = this.entries.find((e) => e.id === id);
    if (item) {
      item.isDirty = isDirty;
      item.lastAutoSavedMs = Date.now();
      this.saveToStorage();
      if (this.isOpen) {
        this.render();
      }
    }
  }

  private saveToStorage() {
    if (typeof localStorage !== "undefined") {
      try {
        localStorage.setItem(OpenDocumentsDrawer.STORAGE_KEY, JSON.stringify(this.entries));
      } catch {
        // ignore storage errors
      }
    }
  }

  private loadFromStorage() {
    if (typeof localStorage !== "undefined") {
      try {
        const raw = localStorage.getItem(OpenDocumentsDrawer.STORAGE_KEY);
        if (raw) {
          const parsed = JSON.parse(raw);
          if (Array.isArray(parsed)) {
            this.entries = parsed;
          }
        }
      } catch {
        this.entries = [];
      }
    }
  }

  private render() {
    this.container.style.display = this.isOpen ? "flex" : "none";
    if (!this.isOpen) return;

    this.container.innerHTML = `
      <div class="drawer-header">
        <div class="drawer-title-group">
          <span class="drawer-icon">📂</span>
          <h3 class="drawer-title">All Open Files (${this.entries.length})</h3>
        </div>
        <div class="drawer-header-actions">
          <span class="autosave-indicator" data-testid="autosave-indicator">● Auto-Save Active</span>
          <button class="drawer-close-btn" data-testid="drawer-close-btn" title="Close Drawer">×</button>
        </div>
      </div>
      <div class="drawer-body">
        ${
          this.entries.length === 0
            ? '<div class="drawer-empty-state">No open documents registered.</div>'
            : `<div class="drawer-cards-list">
                ${this.entries
                  .map((doc) => {
                    const badgeClass =
                      doc.docType === "Word"
                        ? "badge-word"
                        : doc.docType === "Excel"
                        ? "badge-excel"
                        : doc.docType === "PowerPoint"
                        ? "badge-ppt"
                        : doc.docType === "PDF"
                        ? "badge-pdf"
                        : "badge-notes";
                    const savedTime = new Date(doc.lastAutoSavedMs).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                      second: "2-digit",
                    });
                    return `
                      <div class="drawer-doc-card" data-doc-id="${doc.id}" data-testid="drawer-card-${doc.id}">
                        <div class="card-top-row">
                          <span class="card-badge ${badgeClass}">${doc.docType}</span>
                          <span class="card-dirty-pill">${doc.isDirty ? "Unsaved *" : "Saved"}</span>
                          <span class="card-autosave-time" title="Last auto-saved time">${savedTime}</span>
                        </div>
                        <div class="card-title-row">
                          <h4 class="card-doc-title">${doc.title}</h4>
                        </div>
                        <div class="card-actions-row">
                          <button class="btn-card-switch" data-doc-id="${doc.id}" data-doc-type="${doc.docType}" data-testid="btn-card-switch-${doc.id}">
                            Switch Here
                          </button>
                          <button class="btn-card-popout" data-doc-id="${doc.id}" data-testid="btn-card-popout-${doc.id}" title="Open in Dedicated Window ↗">
                            New Window ↗
                          </button>
                          <button class="btn-card-close" data-doc-id="${doc.id}" data-testid="btn-card-close-${doc.id}" title="Close Document">
                            ×
                          </button>
                        </div>
                      </div>
                    `;
                  })
                  .join("")}
              </div>`
        }
      </div>
    `;

    // Wire events
    const closeBtn = this.container.querySelector<HTMLButtonElement>(".drawer-close-btn");
    if (closeBtn) {
      closeBtn.onclick = () => this.close();
    }

    this.container.querySelectorAll<HTMLButtonElement>(".btn-card-switch").forEach((btn) => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const id = btn.dataset.docId;
        const type = btn.dataset.docType as DocumentType;
        if (id && type && this.onSelectDocCallback) {
          this.onSelectDocCallback(id, type);
          this.close();
        }
      };
    });

    this.container.querySelectorAll<HTMLButtonElement>(".btn-card-popout").forEach((btn) => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const id = btn.dataset.docId;
        const entry = this.entries.find((d) => d.id === id);
        if (entry && this.onPopoutDocCallback) {
          this.onPopoutDocCallback(entry);
        }
      };
    });

    this.container.querySelectorAll<HTMLButtonElement>(".btn-card-close").forEach((btn) => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const id = btn.dataset.docId;
        if (id && this.onCloseDocCallback) {
          this.onCloseDocCallback(id);
        }
      };
    });
  }
}

