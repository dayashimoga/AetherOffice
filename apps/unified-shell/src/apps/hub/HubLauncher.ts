/**
 * AetherHub Main Suite Launcher & Process Manager
 * Complies with GEC-v2.0 & Subphase 8.1.2 Specifications
 */

import { IPC, AppInstance } from "../../ipc";

export interface RecentFileItem {
  name: string;
  appMode: "writer" | "calc" | "slides" | "pdf" | "notes";
  path: string;
  lastOpened: string;
}

export interface TemplateItem {
  id: string;
  name: string;
  appMode: "writer" | "calc" | "slides" | "pdf" | "notes";
  icon: string;
  description: string;
  defaultFilename: string;
}

export interface HubLauncherCallbacks {
  onLaunchApp?: (appMode: "writer" | "calc" | "slides" | "pdf" | "notes", filePath?: string) => void;
  onSelectRecent?: (item: RecentFileItem) => void;
  onSelectTemplate?: (template: TemplateItem) => void;
}

export const DEFAULT_TEMPLATES: TemplateItem[] = [
  {
    id: "tpl-writer-blank",
    name: "Blank Document",
    appMode: "writer",
    icon: "📄",
    description: "Start clean with standard margins and paragraph typography.",
    defaultFilename: "Untitled 1.docx",
  },
  {
    id: "tpl-calc-budget",
    name: "Financial Budget Sheet",
    appMode: "calc",
    icon: "📊",
    description: "Pre-structured multi-tab annual income and expense matrix.",
    defaultFilename: "Annual_Budget.xlsx",
  },
  {
    id: "tpl-slides-pitch",
    name: "Pitch Deck",
    appMode: "slides",
    icon: "📽️",
    description: "Modern slide layout with title, agenda, and product feature canvas.",
    defaultFilename: "Pitch_Deck.pptx",
  },
  {
    id: "tpl-pdf-blank",
    name: "Blank / Form PDF",
    appMode: "pdf",
    icon: "📕",
    description: "Standard US-Letter vector page ready for form fill & review.",
    defaultFilename: "Form_Template.pdf",
  },
];

export const INITIAL_RECENT_FILES: RecentFileItem[] = [
  {
    name: "Quarterly_Financial_Report_2026.docx",
    appMode: "writer",
    path: "documents/Quarterly_Financial_Report_2026.docx",
    lastOpened: "Today at 09:15 AM",
  },
  {
    name: "Enterprise_Resource_Budget.xlsx",
    appMode: "calc",
    path: "spreadsheets/Enterprise_Resource_Budget.xlsx",
    lastOpened: "Yesterday at 04:30 PM",
  },
  {
    name: "AetherOffice_Architecture_Overview.pptx",
    appMode: "slides",
    path: "presentations/AetherOffice_Architecture_Overview.pptx",
    lastOpened: "Oct 28, 2026",
  },
  {
    name: "ECMA-376_Specification_Digest.pdf",
    appMode: "pdf",
    path: "manuals/ECMA-376_Specification_Digest.pdf",
    lastOpened: "Oct 25, 2026",
  },
];

export class HubLauncher {
  private container: HTMLElement;
  private callbacks: HubLauncherCallbacks = {};
  private activeInstances: AppInstance[] = [];
  private static readonly STORAGE_KEY = "aether_recent_files";

  constructor(container: string | HTMLElement) {
    if (typeof container === "string") {
      const el = document.getElementById(container);
      if (!el) {
        throw new Error(`HubLauncher container element '#${container}' not found.`);
      }
      this.container = el;
    } else {
      this.container = container;
    }
  }

  public setCallbacks(callbacks: HubLauncherCallbacks) {
    this.callbacks = callbacks;
  }

  public async render(): Promise<void> {
    // Fetch currently running standalone instances
    try {
      this.activeInstances = await IPC.listAppInstances();
    } catch (e) {
      console.warn("Unable to fetch running instances:", e);
      this.activeInstances = [];
    }

    const recentFiles = this.getRecentFiles();

    this.container.innerHTML = `
      <div class="hub-launcher" data-testid="hub-launcher">
        <!-- 1. Hub Header -->
        <header class="hub-header">
          <div class="hub-title-group">
            <div class="hub-logo-badge">⚡</div>
            <div>
              <h1>AetherOffice Suite Hub</h1>
              <p class="hub-subtitle">Universal Native Office Suite • Word • Excel • PowerPoint • PDF</p>
            </div>
          </div>
          <div class="hub-meta-info">
            <span class="hub-badge">ECMA-376 Native</span>
            <span class="hub-badge">Rust 2024 Engine</span>
          </div>
        </header>

        <!-- 2. Prominent Standalone App Launch Cards -->
        <section class="hub-section" aria-label="Standalone Applications">
          <h2 class="hub-section-title">Standalone Applications</h2>
          <div class="hub-apps-grid">
            <!-- Writer Card -->
            <div class="hub-app-card" data-app="writer" data-testid="app-card-writer">
              <div class="hub-card-header">
                <span class="hub-card-icon">📄</span>
                <div>
                  <div class="hub-card-name">AetherWriter</div>
                  <div class="hub-card-format">Word Processor (.docx, .doc)</div>
                </div>
              </div>
              <p class="hub-card-desc">
                High-fidelity ECMA-376 word processing, dynamic tables, typography, and page pagination.
              </p>
              <div class="hub-card-actions">
                <button class="btn-launch" data-launch-app="writer">
                  <span>New Document</span>
                </button>
              </div>
            </div>

            <!-- Calc Card -->
            <div class="hub-app-card" data-app="calc" data-testid="app-card-calc">
              <div class="hub-card-header">
                <span class="hub-card-icon">📊</span>
                <div>
                  <div class="hub-card-name">AetherCalc</div>
                  <div class="hub-card-format">Spreadsheet Engine (.xlsx, .csv)</div>
                </div>
              </div>
              <p class="hub-card-desc">
                Fast multi-sheet calculations, 400+ formula functions, and virtualized 1,000,000+ cell grid.
              </p>
              <div class="hub-card-actions">
                <button class="btn-launch" data-launch-app="calc">
                  <span>New Spreadsheet</span>
                </button>
              </div>
            </div>

            <!-- Slides Card -->
            <div class="hub-app-card" data-app="slides" data-testid="app-card-slides">
              <div class="hub-card-header">
                <span class="hub-card-icon">📽️</span>
                <div>
                  <div class="hub-card-name">AetherSlides</div>
                  <div class="hub-card-format">Presentation Studio (.pptx, .ppt)</div>
                </div>
              </div>
              <p class="hub-card-desc">
                Slide masters, vector shapes, animations, and presentation projection mode.
              </p>
              <div class="hub-card-actions">
                <button class="btn-launch" data-launch-app="slides">
                  <span>New Presentation</span>
                </button>
              </div>
            </div>

            <!-- PDF Card -->
            <div class="hub-app-card" data-app="pdf" data-testid="app-card-pdf">
              <div class="hub-card-header">
                <span class="hub-card-icon">📕</span>
                <div>
                  <div class="hub-card-name">AetherPDF</div>
                  <div class="hub-card-format">Document Reader (.pdf)</div>
                </div>
              </div>
              <p class="hub-card-desc">
                Crisp vector document rendering, page rotation, text markup, and interactive form fields.
              </p>
              <div class="hub-card-actions">
                <button class="btn-launch" data-launch-app="pdf">
                  <span>Open PDF Studio</span>
                </button>
              </div>
            </div>

            <!-- Notes Card -->
            <div class="hub-app-card" data-app="notes" data-testid="app-card-notes">
              <div class="hub-card-header">
                <span class="hub-card-icon">📝</span>
                <div>
                  <div class="hub-card-name">AetherNotes</div>
                  <div class="hub-card-format">Notepad & Markdown (.txt, .md)</div>
                </div>
              </div>
              <p class="hub-card-desc">
                Ultra-fast plain text editor, Markdown live preview, line numbering, and quick notes.
              </p>
              <div class="hub-card-actions">
                <button class="btn-launch" data-launch-app="notes">
                  <span>New Note</span>
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- 3. Template Gallery -->
        <section class="hub-section" aria-label="Template Gallery">
          <h2 class="hub-section-title">New from Template</h2>
          <div class="hub-templates-grid">
            ${DEFAULT_TEMPLATES.map(
              (tpl) => `
              <div class="hub-template-card" data-template-id="${tpl.id}" data-app="${tpl.appMode}">
                <span class="hub-template-icon">${tpl.icon}</span>
                <div class="hub-template-info">
                  <span class="hub-template-name">${tpl.name}</span>
                  <span class="hub-template-app">${tpl.description}</span>
                </div>
              </div>
            `
            ).join("")}
          </div>
        </section>

        <!-- 4. Recent Files List -->
        <section class="hub-section" aria-label="Recent Documents">
          <div class="hub-section-title">
            <span>Recent Documents</span>
            <div class="hub-table-actions">
              <button class="btn-secondary" id="btn-open-local-file">Open Local File...</button>
              <button class="btn-secondary" id="btn-clear-recent">Clear Recent</button>
            </div>
          </div>
          <div class="hub-recent-panel">
            ${
              recentFiles.length === 0
                ? `<div style="text-align:center; padding: 24px; color: #8a8a8a;">No recent documents found.</div>`
                : `
              <table class="hub-recent-table" data-testid="recent-files-table">
                <thead>
                  <tr>
                    <th>Document Name</th>
                    <th>Application</th>
                    <th>Last Modified</th>
                    <th>File Path</th>
                    <th style="text-align:right">Action</th>
                  </tr>
                </thead>
                <tbody>
                  ${recentFiles
                    .map(
                      (f, idx) => `
                    <tr data-recent-index="${idx}">
                      <td>
                        <span class="recent-file-name" data-open-recent="${idx}">
                          ${f.appMode === "writer" ? "📄" : f.appMode === "calc" ? "📊" : f.appMode === "slides" ? "📽️" : "📕"}
                          ${f.name}
                        </span>
                      </td>
                      <td>
                        <span class="recent-type-pill pill-${f.appMode}">
                          ${f.appMode.toUpperCase()}
                        </span>
                      </td>
                      <td>${f.lastOpened}</td>
                      <td class="recent-file-path">${f.path}</td>
                      <td style="text-align:right">
                        <button class="btn-secondary" data-open-recent="${idx}">Open</button>
                        <button class="btn-secondary" data-remove-recent="${idx}">×</button>
                      </td>
                    </tr>
                  `
                    )
                    .join("")}
                </tbody>
              </table>
            `
            }
          </div>
        </section>

        <!-- 5. Running Standalone Instances & OS Windows -->
        <section class="hub-section" aria-label="Running Processes">
          <div class="hub-section-title">
            <span>Running Standalone Instances</span>
            <button class="btn-secondary" id="btn-refresh-instances">Refresh List</button>
          </div>
          <div class="hub-instances-panel">
            ${
              this.activeInstances.length === 0
                ? `<div style="text-align:center; padding: 20px; color: #8a8a8a;">No external child instances currently running.</div>`
                : `
              <table class="hub-recent-table" data-testid="running-instances-table">
                <thead>
                  <tr>
                    <th>Status</th>
                    <th>Instance ID</th>
                    <th>Application</th>
                    <th>Window Title</th>
                    <th>PID</th>
                    <th style="text-align:right">Control</th>
                  </tr>
                </thead>
                <tbody>
                  ${this.activeInstances
                    .map(
                      (inst) => `
                    <tr>
                      <td><span class="instance-status-dot"></span>Active</td>
                      <td><code>${inst.instance_id}</code></td>
                      <td><span class="recent-type-pill pill-${inst.app_mode}">${inst.app_mode.toUpperCase()}</span></td>
                      <td>${inst.title}</td>
                      <td><code>${inst.process_id ?? "N/A"}</code></td>
                      <td style="text-align:right">
                        <button class="btn-secondary" data-focus-instance="${inst.instance_id}">Focus</button>
                        <button class="btn-danger-sm" data-close-instance="${inst.instance_id}">Close</button>
                      </td>
                    </tr>
                  `
                    )
                    .join("")}
                </tbody>
              </table>
            `
            }
          </div>
        </section>
      </div>
    `;

    this.attachEventListeners();
  }

  private attachEventListeners(): void {
    // 1. App Launch Cards
    const launchButtons = this.container.querySelectorAll<HTMLButtonElement>("[data-launch-app]");
    launchButtons.forEach((btn) => {
      btn.addEventListener("click", () => {
        const app = btn.getAttribute("data-launch-app") as "writer" | "calc" | "slides" | "pdf";
        if (app) {
          this.launchApp(app);
        }
      });
    });

    // 2. Template Cards
    const tplCards = this.container.querySelectorAll<HTMLElement>("[data-template-id]");
    tplCards.forEach((card) => {
      card.addEventListener("click", () => {
        const tplId = card.getAttribute("data-template-id");
        const tpl = DEFAULT_TEMPLATES.find((t) => t.id === tplId);
        if (tpl) {
          if (this.callbacks.onSelectTemplate) {
            this.callbacks.onSelectTemplate(tpl);
          }
          this.launchApp(tpl.appMode, tpl.defaultFilename);
        }
      });
    });

    // 3. Open Recent File
    const openRecentButtons = this.container.querySelectorAll<HTMLElement>("[data-open-recent]");
    openRecentButtons.forEach((el) => {
      el.addEventListener("click", () => {
        const idxStr = el.getAttribute("data-open-recent");
        if (idxStr !== null) {
          const idx = parseInt(idxStr, 10);
          const files = this.getRecentFiles();
          const target = files[idx];
          if (target) {
            if (this.callbacks.onSelectRecent) {
              this.callbacks.onSelectRecent(target);
            }
            this.launchApp(target.appMode, target.path);
          }
        }
      });
    });

    // 4. Remove Recent File
    const removeRecentButtons = this.container.querySelectorAll<HTMLElement>("[data-remove-recent]");
    removeRecentButtons.forEach((btn) => {
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const idxStr = btn.getAttribute("data-remove-recent");
        if (idxStr !== null) {
          const idx = parseInt(idxStr, 10);
          const files = this.getRecentFiles();
          if (files[idx]) {
            this.removeRecentFile(files[idx].path);
            this.render();
          }
        }
      });
    });

    // 5. Open Local File Dialog Button
    const btnOpenLocal = this.container.querySelector<HTMLButtonElement>("#btn-open-local-file");
    if (btnOpenLocal) {
      btnOpenLocal.addEventListener("click", () => {
        this.promptOpenLocalFile();
      });
    }

    // 6. Clear Recent Button
    const btnClearRecent = this.container.querySelector<HTMLButtonElement>("#btn-clear-recent");
    if (btnClearRecent) {
      btnClearRecent.addEventListener("click", () => {
        this.clearRecentFiles();
        this.render();
      });
    }

    // 7. Refresh Instances Button
    const btnRefresh = this.container.querySelector<HTMLButtonElement>("#btn-refresh-instances");
    if (btnRefresh) {
      btnRefresh.addEventListener("click", () => {
        this.render();
      });
    }

    // 8. Focus / Close Instances
    const focusButtons = this.container.querySelectorAll<HTMLElement>("[data-focus-instance]");
    focusButtons.forEach((btn) => {
      btn.addEventListener("click", async () => {
        const instId = btn.getAttribute("data-focus-instance");
        if (instId) {
          await IPC.focusAppInstance(instId);
        }
      });
    });

    const closeButtons = this.container.querySelectorAll<HTMLElement>("[data-close-instance]");
    closeButtons.forEach((btn) => {
      btn.addEventListener("click", async () => {
        const instId = btn.getAttribute("data-close-instance");
        if (instId) {
          await IPC.closeAppInstance(instId);
          this.render();
        }
      });
    });
  }

  public async launchApp(appMode: "writer" | "calc" | "slides" | "pdf" | "notes", filePath?: string): Promise<AppInstance | null> {
    try {
      const inst = await IPC.launchStandaloneApp(appMode, filePath);
      if (filePath) {
        const filename = filePath.split(/[/\\]/).pop() || filePath;
        this.addRecentFile({
          name: filename,
          appMode,
          path: filePath,
          lastOpened: "Just now",
        });
      }
      if (this.callbacks.onLaunchApp) {
        this.callbacks.onLaunchApp(appMode, filePath);
      }
      await this.render();
      return inst;
    } catch (e) {
      console.error(`Failed to launch standalone app '${appMode}':`, e);
      return null;
    }
  }

  public promptOpenLocalFile(): void {
    const inputPath = window.prompt("Enter local document path to open (docx, xlsx, pptx, pdf):");
    if (inputPath && inputPath.trim()) {
      const path = inputPath.trim();
      const lower = path.toLowerCase();
      let mode: "writer" | "calc" | "slides" | "pdf" = "writer";
      if (lower.endsWith(".xlsx") || lower.endsWith(".xls") || lower.endsWith(".csv")) {
        mode = "calc";
      } else if (lower.endsWith(".pptx") || lower.endsWith(".ppt")) {
        mode = "slides";
      } else if (lower.endsWith(".pdf")) {
        mode = "pdf";
      }
      this.launchApp(mode, path);
    }
  }

  public getRecentFiles(): RecentFileItem[] {
    try {
      const raw = localStorage.getItem(HubLauncher.STORAGE_KEY);
      if (raw !== null) {
        return JSON.parse(raw);
      }
    } catch (e) {
      console.warn("Error reading recent files from localStorage:", e);
    }
    // Initialize defaults if key never existed
    this.saveRecentFiles(INITIAL_RECENT_FILES);
    return INITIAL_RECENT_FILES;
  }

  public saveRecentFiles(files: RecentFileItem[]): void {
    try {
      localStorage.setItem(HubLauncher.STORAGE_KEY, JSON.stringify(files));
    } catch (e) {
      console.warn("Error writing recent files to localStorage:", e);
    }
  }

  public addRecentFile(item: RecentFileItem): void {
    const files = this.getRecentFiles().filter((f) => f.path !== item.path);
    files.unshift(item);
    // Limit to 20 most recent
    this.saveRecentFiles(files.slice(0, 20));
  }

  public removeRecentFile(path: string): void {
    const files = this.getRecentFiles().filter((f) => f.path !== path);
    this.saveRecentFiles(files);
  }

  public clearRecentFiles(): void {
    this.saveRecentFiles([]);
  }
}
