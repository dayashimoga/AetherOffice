/**
 * Office Backstage View (File Menu) Component
 * Complies with GEC-v2.0 & Subphase 8.2.1 Specifications
 */

import { DocumentType, DocumentMetadata, IPC } from "../ipc";

export type BackstageNavTab =
  | "Info"
  | "New"
  | "Open"
  | "Save"
  | "SaveAs"
  | "Export"
  | "Print"
  | "Plugins"
  | "Close";

export interface BackstageMenuCallbacks {
  onClose?: () => void;
  onNew?: (docType: DocumentType) => void;
  onOpen?: (filePath: string, fileContent?: string) => void;
  onOpenFile?: (file: File) => void;
  onSave?: (filePath?: string) => Promise<void>;
  onExportPdf?: (outputPath: string) => Promise<void>;
  onExport?: (format: "pdf" | "csv" | "txt" | "html" | "md", outputPath?: string) => Promise<void>;
  onCloseSession?: () => void;
  onOpenPlugins?: () => void;
  onInspectDocument?: () => void;
  onPrint?: () => void;
}

export class BackstageMenu {
  private overlay: HTMLElement;
  private callbacks: BackstageMenuCallbacks = {};
  private activeNav: BackstageNavTab = "Info";
  private visible = false;

  private currentDocType: DocumentType = "Word";
  private currentSessionId = "";
  private currentTitle = "";
  private currentFilePath?: string;
  private currentIsDirty = false;
  private currentMeta?: DocumentMetadata;

  constructor(parentContainerId = "app") {
    let parent = document.getElementById(parentContainerId);
    if (!parent) {
      parent = document.body;
    }

    const overlay = document.createElement("div");
    overlay.id = "backstage-overlay";
    overlay.className = "backstage-overlay hidden";
    parent.appendChild(overlay);
    this.overlay = overlay;

    // Listen for Escape key to close Backstage
    window.addEventListener("keydown", (e) => {
      if (this.visible && e.key === "Escape") {
        this.hide();
      }
    });
  }

  public setCallbacks(callbacks: BackstageMenuCallbacks): void {
    this.callbacks = callbacks;
  }

  public isOpen(): boolean {
    return this.visible;
  }

  public async show(
    docType: DocumentType = "Word",
    sessionId = "sess-1",
    title = "Untitled.docx",
    filePath?: string,
    isDirty = false
  ): Promise<void> {
    this.currentDocType = docType;
    this.currentSessionId = sessionId;
    this.currentTitle = title;
    this.currentFilePath = filePath;
    this.currentIsDirty = isDirty;
    this.visible = true;

    // Fetch live metadata if session ID present
    try {
      if (sessionId) {
        this.currentMeta = await IPC.getDocumentInfo(sessionId);
      }
    } catch (e) {
      console.warn("Unable to fetch document info via IPC:", e);
    }

    this.overlay.classList.remove("hidden");
    this.render();
  }

  public hide(): void {
    this.visible = false;
    this.overlay.classList.add("hidden");
    if (this.callbacks.onClose) {
      this.callbacks.onClose();
    }
  }

  public setActiveNav(tab: BackstageNavTab): void {
    this.activeNav = tab;
    this.render();
  }

  public render(): void {
    const docTypeClass =
      this.currentDocType === "Excel"
        ? "doc-type-excel"
        : this.currentDocType === "PowerPoint"
        ? "doc-type-ppt"
        : this.currentDocType === "PDF"
        ? "doc-type-pdf"
        : this.currentDocType === "Notes"
        ? "doc-type-notes"
        : "doc-type-word";

    this.overlay.innerHTML = `
      <!-- Left Navigation Rail -->
      <nav class="backstage-nav ${docTypeClass}" aria-label="Backstage Navigation">
        <div class="backstage-back-container">
          <button class="backstage-back-btn" id="btn-backstage-back" title="Back to Document (Esc)">
            ←
          </button>
        </div>

        <button class="backstage-nav-item ${this.activeNav === "Info" ? "active" : ""}" data-nav="Info">
          <span>ℹ️</span> <span>Info</span>
        </button>
        <button class="backstage-nav-item ${this.activeNav === "New" ? "active" : ""}" data-nav="New">
          <span>➕</span> <span>New</span>
        </button>
        <button class="backstage-nav-item ${this.activeNav === "Open" ? "active" : ""}" data-nav="Open">
          <span>📂</span> <span>Open</span>
        </button>
        <button class="backstage-nav-item ${this.activeNav === "Save" ? "active" : ""}" data-nav="Save">
          <span>💾</span> <span>Save</span>
        </button>
        <button class="backstage-nav-item ${this.activeNav === "SaveAs" ? "active" : ""}" data-nav="SaveAs">
          <span>📥</span> <span>Save As</span>
        </button>

        <div class="backstage-nav-separator"></div>

        <button class="backstage-nav-item ${this.activeNav === "Export" ? "active" : ""}" data-nav="Export">
          <span>📄</span> <span>Export</span>
        </button>
        <button class="backstage-nav-item ${this.activeNav === "Print" ? "active" : ""}" data-nav="Print">
          <span>🖨️</span> <span>Print</span>
        </button>

        <div class="backstage-nav-separator"></div>

        <button class="backstage-nav-item ${this.activeNav === "Plugins" ? "active" : ""}" data-nav="Plugins" data-testid="backstage-tab-plugins">
          <span>🧩</span> <span>Add-ins & Extensions</span>
        </button>

        <div class="backstage-nav-separator"></div>

        <button class="backstage-nav-item ${this.activeNav === "Close" ? "active" : ""}" data-nav="Close">
          <span>❌</span> <span>Close</span>
        </button>
      </nav>

      <!-- Right Content Panel -->
      <main class="backstage-content" id="backstage-content-panel">
        ${this.renderActivePanel()}
      </main>
    `;

    this.attachEventListeners();
  }

  private renderActivePanel(): string {
    switch (this.activeNav) {
      case "Info":
        return `
          <h2 class="backstage-panel-title">Document Information</h2>
          <div class="info-card-group">
            <div class="info-card">
              <div class="info-card-header">Document Title</div>
              <div class="info-card-value">${this.currentTitle}</div>
            </div>
            <div class="info-card">
              <div class="info-card-header">Engine & Format</div>
              <div class="info-card-value">${this.currentDocType} (ECMA-376 / ISO)</div>
            </div>
            <div class="info-card">
              <div class="info-card-header">File Path</div>
              <div class="info-card-value" style="font-size: 14px; font-family: monospace;">
                ${this.currentFilePath ?? "Unsaved Document"}
              </div>
            </div>
            <div class="info-card">
              <div class="info-card-header">Save Status</div>
              <div class="info-card-value">
                ${
                  this.currentIsDirty
                    ? `<span class="info-badge-dirty">Modified (Unsaved Changes)</span>`
                    : `<span class="info-badge-clean">Saved to Disk</span>`
                }
              </div>
            </div>
            <div class="info-card">
              <div class="info-card-header">Session & Size</div>
              <div class="info-card-value" style="font-size: 14px;">
                <code>${this.currentSessionId || "N/A"}</code> • ${
                  this.currentMeta?.file_size_bytes ? `${this.currentMeta.file_size_bytes} bytes` : "In-Memory"
                }
              </div>
            </div>
          </div>
          <button class="btn-primary-backstage" id="btn-quick-save">Quick Save Now</button>
          <button class="btn-primary-backstage" id="btn-backstage-inspect" data-testid="btn-backstage-inspect" style="margin-left: 10px; background-color: #605e5c;">Inspect & Sanitize Document</button>
        `;

      case "New":
        return `
          <h2 class="backstage-panel-title">New Document</h2>
          <p style="color: #6a737d;">Select a blank document or template format to create:</p>
          <div class="backstage-templates-grid">
            <div class="backstage-template-card" data-new-type="Word" data-testid="backstage-new-word">
              <span class="backstage-template-icon">📄</span>
              <div class="backstage-template-name">Blank Document</div>
              <div class="backstage-template-desc">ECMA-376 WordprocessingML document (.docx)</div>
            </div>
            <div class="backstage-template-card" data-new-type="Excel" data-testid="backstage-new-excel">
              <span class="backstage-template-icon">📊</span>
              <div class="backstage-template-name">Blank Spreadsheet</div>
              <div class="backstage-template-desc">High performance SpreadsheetML workbook (.xlsx)</div>
            </div>
            <div class="backstage-template-card" data-new-type="PowerPoint" data-testid="backstage-new-ppt">
              <span class="backstage-template-icon">📽️</span>
              <div class="backstage-template-name">Blank Presentation</div>
              <div class="backstage-template-desc">PresentationML slide deck (.pptx)</div>
            </div>
            <div class="backstage-template-card" data-new-type="PDF" data-testid="backstage-new-pdf">
              <span class="backstage-template-icon">📕</span>
              <div class="backstage-template-name">Blank PDF Publication</div>
              <div class="backstage-template-desc">ISO 32000-1 vector publication document (.pdf)</div>
            </div>
            <div class="backstage-template-card" data-new-type="Notes" data-testid="backstage-new-notes">
              <span class="backstage-template-icon">📝</span>
              <div class="backstage-template-name">Blank Text / Notes</div>
              <div class="backstage-template-desc">Plain text (.txt) and Markdown notes (.md)</div>
            </div>
          </div>
        `;

      case "Open":
        return `
          <h2 class="backstage-panel-title">Open Document</h2>
          <div class="backstage-form-group">
            <label style="font-weight: 600;">Choose File from Computer:</label>
            <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
              <input
                type="file"
                id="input-file-picker"
                data-testid="input-file-picker"
                style="display: none;"
                accept=".docx,.xlsx,.xls,.csv,.pptx,.ppt,.pdf,.txt,.md,.json,.log"
              />
              <button class="btn-primary-backstage" id="btn-browse-file" data-testid="btn-browse-file" type="button">
                📂 Browse Local Files...
              </button>
              <span id="label-chosen-file" style="font-size: 13px; color: var(--text-secondary, #6a737d);">No file chosen</span>
            </div>
            <label style="font-weight: 600;">Or Enter File Path from Disk:</label>
            <input type="text" class="backstage-input" id="input-open-file-path" data-testid="input-open-file-path" placeholder="e.g. documents/Report.docx or G:\\budget.csv" />
            <button class="btn-primary-backstage" id="btn-submit-open-file" data-testid="btn-submit-open-file">Open Document</button>
          </div>
        `;

      case "Save":
        return `
          <h2 class="backstage-panel-title">Save Document</h2>
          <p style="color: #6a737d; margin-bottom: 20px;">
            Write all pending modifications directly to disk with ECMA-376 round-trip fidelity.
          </p>
          <div class="backstage-form-group">
            <div style="font-size: 14px;"><strong>Target File:</strong> ${this.currentFilePath ?? "Unsaved"}</div>
            <button class="btn-primary-backstage" id="btn-submit-save">Save Now</button>
          </div>
        `;

      case "SaveAs":
        return `
          <h2 class="backstage-panel-title">Save As Copy</h2>
          <p style="color: #6a737d; margin-bottom: 16px;">
            Save a copy of this document to a new file location on disk:
          </p>
          <div class="backstage-form-group">
            <label style="font-weight: 600;">Destination File Path:</label>
            <input
              type="text"
              class="backstage-input"
              id="input-save-as-path"
              value="${this.currentFilePath ?? `Saved_${this.currentTitle}`}"
              placeholder="e.g. C:\\Documents\\Copy.docx"
            />
            <button class="btn-primary-backstage" id="btn-submit-save-as">Save Copy to Disk</button>
          </div>
        `;

      case "Export":
        return `
          <h2 class="backstage-panel-title">Export Document</h2>
          <p style="color: #6a737d; margin-bottom: 16px;">
            Render or convert the active document into standard interchange formats:
          </p>
          <div class="backstage-templates-grid" style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); margin-bottom: 24px;">
            <div class="backstage-template-card" id="btn-export-pdf" data-testid="btn-export-pdf" data-export-format="pdf">
              <span class="backstage-template-icon">📕</span>
              <div class="backstage-template-name">Export to PDF</div>
              <div class="backstage-template-desc">Certified ISO 32000 publication (.pdf)</div>
            </div>
            ${this.currentDocType === "Excel" ? `
            <div class="backstage-template-card" id="btn-export-csv" data-testid="btn-export-csv" data-export-format="csv">
              <span class="backstage-template-icon">📊</span>
              <div class="backstage-template-name">Export to CSV</div>
              <div class="backstage-template-desc">Comma-separated values spreadsheet (.csv)</div>
            </div>` : ""}
            ${this.currentDocType === "Word" || this.currentDocType === "Notes" ? `
            <div class="backstage-template-card" id="btn-export-txt" data-testid="btn-export-txt" data-export-format="txt">
              <span class="backstage-template-icon">📄</span>
              <div class="backstage-template-name">Export Plain Text</div>
              <div class="backstage-template-desc">UTF-8 plain text file (.txt)</div>
            </div>
            <div class="backstage-template-card" id="btn-export-html" data-testid="btn-export-html" data-export-format="html">
              <span class="backstage-template-icon">🌐</span>
              <div class="backstage-template-name">Export HTML</div>
              <div class="backstage-template-desc">Standalone Web HTML document (.html)</div>
            </div>` : ""}
            ${this.currentDocType === "Notes" ? `
            <div class="backstage-template-card" id="btn-export-md" data-testid="btn-export-md" data-export-format="md">
              <span class="backstage-template-icon">📝</span>
              <div class="backstage-template-name">Export Markdown</div>
              <div class="backstage-template-desc">Formatted CommonMark note (.md)</div>
            </div>` : ""}
          </div>
          <div class="backstage-form-group">
            <label style="font-weight: 600;">Export Destination / Custom Path:</label>
            <input
              type="text"
              class="backstage-input"
              id="input-export-pdf-path"
              value="${this.currentTitle.replace(/\.[^/.]+$/, "")}.pdf"
              placeholder="e.g. exports/Exported.pdf"
            />
            <button class="btn-primary-backstage" id="btn-submit-export-pdf">Export PDF</button>
          </div>
        `;

      case "Print":
        return `
          <h2 class="backstage-panel-title">Print Document</h2>
          <p style="color: #6a737d; margin-bottom: 20px;">
            Send document pages directly to system printer or print preview:
          </p>
          <button class="btn-primary-backstage" id="btn-submit-print" data-testid="btn-submit-print">Print Document</button>
        `;

      case "Plugins":
        return `
          <h2 class="backstage-panel-title">Add-ins & Extensions</h2>
          <div style="padding: 16px 0; max-width: 600px; display: flex; flex-direction: column; gap: 16px;">
            <p style="color: #4b5563; font-size: 14px; line-height: 1.5;">
              Extend AetherOffice with dynamic third-party plugins, custom ribbon tabs, calculation formulas (UDFs), and sandboxed sidebars.
            </p>
            <div>
              <button class="btn-primary-backstage" id="btn-open-plugin-manager" data-testid="btn-open-plugin-manager" style="background-color: var(--primary-accent, #2b579a); padding: 10px 20px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 8px;">
                <span>🧩</span> <span>Open Plugin Manager & Marketplace</span>
              </button>
            </div>
          </div>
        `;
      case "Close":
        return `
          <h2 class="backstage-panel-title">Close Document</h2>
          <p style="color: #6a737d; margin-bottom: 20px;">
            Close the current document window. If you have unsaved changes, you will be prompted.
          </p>
          <button class="btn-primary-backstage" id="btn-submit-close-session" style="background-color: #d73a49;">
            Close Document
          </button>
        `;
    }
  }

  private attachEventListeners(): void {
    // 1. Back button
    const backBtn = this.overlay.querySelector<HTMLButtonElement>("#btn-backstage-back");
    if (backBtn) {
      backBtn.addEventListener("click", () => this.hide());
    }

    // 2. Navigation items
    const navItems = this.overlay.querySelectorAll<HTMLButtonElement>("[data-nav]");
    navItems.forEach((btn) => {
      btn.addEventListener("click", () => {
        const nav = btn.getAttribute("data-nav") as BackstageNavTab;
        if (nav) {
          this.setActiveNav(nav);
        }
      });
    });

    // 3. New Document templates
    const newCards = this.overlay.querySelectorAll<HTMLElement>("[data-new-type]");
    newCards.forEach((card) => {
      card.addEventListener("click", () => {
        const docType = card.getAttribute("data-new-type") as DocumentType;
        if (docType && this.callbacks.onNew) {
          this.callbacks.onNew(docType);
          this.hide();
        }
      });
    });

    // 4. Open Document
    const btnBrowse = this.overlay.querySelector<HTMLButtonElement>("#btn-browse-file");
    const inputFilePicker = this.overlay.querySelector<HTMLInputElement>("#input-file-picker");
    const labelChosen = this.overlay.querySelector<HTMLSpanElement>("#label-chosen-file");

    if (btnBrowse && inputFilePicker) {
      btnBrowse.addEventListener("click", () => {
        inputFilePicker.click();
      });
      inputFilePicker.addEventListener("change", () => {
        const file = inputFilePicker.files?.[0];
        if (file) {
          if (labelChosen) {
            labelChosen.textContent = file.name;
          }
          if (this.callbacks.onOpenFile) {
            this.callbacks.onOpenFile(file);
            this.hide();
          } else if (this.callbacks.onOpen) {
            const reader = new FileReader();
            reader.onload = () => {
              this.callbacks.onOpen?.(file.name, typeof reader.result === "string" ? reader.result : "");
              this.hide();
            };
            reader.readAsText(file);
          }
        }
      });
    }

    const btnOpen = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-open-file");
    const inputOpen = this.overlay.querySelector<HTMLInputElement>("#input-open-file-path");
    if (btnOpen && inputOpen) {
      btnOpen.addEventListener("click", () => {
        const path = inputOpen.value.trim();
        if (path && this.callbacks.onOpen) {
          this.callbacks.onOpen(path);
          this.hide();
        }
      });
    }

    // 5. Quick Save / Save
    const btnQuickSave = this.overlay.querySelector<HTMLButtonElement>("#btn-quick-save");
    const btnSave = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-save");
    const handleSave = async () => {
      if (this.callbacks.onSave) {
        await this.callbacks.onSave(this.currentFilePath);
        this.currentIsDirty = false;
        this.render();
      }
    };
    if (btnQuickSave) btnQuickSave.addEventListener("click", handleSave);
    if (btnSave) btnSave.addEventListener("click", handleSave);

    const btnInspect = this.overlay.querySelector<HTMLButtonElement>("#btn-backstage-inspect");
    if (btnInspect) {
      btnInspect.addEventListener("click", () => {
        if (this.callbacks.onInspectDocument) {
          this.callbacks.onInspectDocument();
        }
      });
    }

    // 6. Save As
    const btnSaveAs = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-save-as");
    const inputSaveAs = this.overlay.querySelector<HTMLInputElement>("#input-save-as-path");
    if (btnSaveAs && inputSaveAs) {
      btnSaveAs.addEventListener("click", async () => {
        const path = inputSaveAs.value.trim();
        if (path && this.callbacks.onSave) {
          await this.callbacks.onSave(path);
          this.currentFilePath = path;
          this.currentIsDirty = false;
          this.render();
        }
      });
    }

    // 7. Export Options
    const exportCards = this.overlay.querySelectorAll<HTMLElement>("[data-export-format]");
    exportCards.forEach((card) => {
      card.addEventListener("click", async () => {
        const fmt = card.getAttribute("data-export-format") as "pdf" | "csv" | "txt" | "html" | "md";
        const customPath = (this.overlay.querySelector<HTMLInputElement>("#input-export-pdf-path")?.value || "").trim();
        if (fmt === "pdf" && this.callbacks.onExportPdf) {
          await this.callbacks.onExportPdf(customPath || `${this.currentTitle.replace(/\.[^/.]+$/, "")}.pdf`);
          this.hide();
        } else if (fmt && this.callbacks.onExport) {
          await this.callbacks.onExport(fmt, customPath);
          this.hide();
        }
      });
    });

    const btnExportPdf = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-export-pdf");
    const inputExportPdf = this.overlay.querySelector<HTMLInputElement>("#input-export-pdf-path");
    if (btnExportPdf && inputExportPdf) {
      btnExportPdf.addEventListener("click", async () => {
        const path = inputExportPdf.value.trim();
        if (path && this.callbacks.onExportPdf) {
          await this.callbacks.onExportPdf(path);
          this.hide();
        }
      });
    }

    // 8. Print
    const btnPrint = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-print");
    if (btnPrint) {
      btnPrint.addEventListener("click", () => {
        if (this.callbacks.onPrint) {
          this.callbacks.onPrint();
        } else {
          window.print();
        }
        this.hide();
      });
    }

    // 8.1 Open Plugin Manager
    const btnOpenPlugins = this.overlay.querySelector<HTMLButtonElement>("#btn-open-plugin-manager");
    if (btnOpenPlugins) {
      btnOpenPlugins.addEventListener("click", () => {
        this.hide();
        if (this.callbacks.onOpenPlugins) {
          this.callbacks.onOpenPlugins();
        }
      });
    }

    // 9. Close
    const btnClose = this.overlay.querySelector<HTMLButtonElement>("#btn-submit-close-session");
    if (btnClose) {
      btnClose.addEventListener("click", () => {
        if (this.callbacks.onCloseSession) {
          this.callbacks.onCloseSession();
        }
        this.hide();
      });
    }
  }
}
