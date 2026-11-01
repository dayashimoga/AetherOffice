/**
 * ProjectExplorerSidebar.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.2.1)
 *
 * Project Explorer Sidebar for unified mixed-format multi-document workspaces:
 * - Mounts workspace directories and displays files in a natural tree structure
 * - Directly supports .docx, .xlsx, .pptx, .pdf, .md, .txt, .csv with format badges
 * - Instant file opening, search filtering, and active document tracking
 */

import { DocumentType } from "../ipc";

export interface ProjectFile {
  type: "file";
  name: string;
  path: string;
  docType: DocumentType;
  sizeBytes?: number;
}

export interface ProjectFolder {
  type: "folder";
  name: string;
  path: string;
  isExpanded: boolean;
  children: (ProjectFile | ProjectFolder)[];
}

export type ProjectNode = ProjectFile | ProjectFolder;

export interface ProjectExplorerCallbacks {
  onOpenFile?: (file: ProjectFile) => void;
  onMountFolder?: () => void;
  onRefresh?: () => void;
}

export class ProjectExplorerSidebar {
  private container: HTMLElement;
  private rootFolder: ProjectFolder;
  private activeFilePath: string | null = null;
  private searchQuery = "";
  private callbacks: ProjectExplorerCallbacks;

  constructor(
    containerIdOrElement: string | HTMLElement,
    callbacks: ProjectExplorerCallbacks = {}
  ) {
    if (typeof containerIdOrElement === "string") {
      const el = document.getElementById(containerIdOrElement);
      if (!el) throw new Error(`ProjectExplorerSidebar container #${containerIdOrElement} not found`);
      this.container = el;
    } else {
      this.container = containerIdOrElement;
    }
    this.callbacks = callbacks;
    this.rootFolder = this.createDefaultWorkspace();
    this.render();
  }

  public getRootFolder(): ProjectFolder {
    return this.rootFolder;
  }

  public setRootFolder(folder: ProjectFolder): void {
    this.rootFolder = folder;
    this.render();
  }

  public getActiveFilePath(): string | null {
    return this.activeFilePath;
  }

  public setActiveFilePath(filePath: string | null): void {
    this.activeFilePath = filePath;
    this.render();
  }

  public setSearchQuery(query: string): void {
    this.searchQuery = query.toLowerCase().trim();
    this.render();
  }

  public expandAll(): void {
    const expand = (folder: ProjectFolder) => {
      folder.isExpanded = true;
      for (const child of folder.children) {
        if (child.type === "folder") {
          expand(child);
        }
      }
    };
    expand(this.rootFolder);
    this.render();
  }

  public collapseAll(): void {
    const collapse = (folder: ProjectFolder) => {
      folder.isExpanded = false;
      for (const child of folder.children) {
        if (child.type === "folder") {
          collapse(child);
        }
      }
    };
    collapse(this.rootFolder);
    this.render();
  }

  public render(): void {
    this.container.innerHTML = "";
    this.container.className = "project-explorer-sidebar";
    this.container.dataset.testid = "project-explorer-sidebar";

    // 1. Header with Title & Action Controls
    const header = document.createElement("div");
    header.className = "project-explorer-header";
    header.innerHTML = `
      <div class="project-explorer-title-row" style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--border-color);">
        <span style="font-weight: 600; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-primary); display: flex; align-items: center; gap: 6px;">
          📁 Workspace Explorer
        </span>
        <div class="project-explorer-actions" style="display: flex; gap: 4px;">
          <button class="explorer-btn-expand" data-testid="explorer-btn-expand" title="Expand All Folders" style="background: none; border: none; cursor: pointer; padding: 2px 4px; font-size: 12px; color: var(--text-secondary);">⊞</button>
          <button class="explorer-btn-collapse" data-testid="explorer-btn-collapse" title="Collapse All Folders" style="background: none; border: none; cursor: pointer; padding: 2px 4px; font-size: 12px; color: var(--text-secondary);">⊟</button>
          <button class="explorer-btn-refresh" data-testid="explorer-btn-refresh" title="Refresh Workspace" style="background: none; border: none; cursor: pointer; padding: 2px 4px; font-size: 12px; color: var(--text-secondary);">↻</button>
        </div>
      </div>
    `;

    const expandBtn = header.querySelector(".explorer-btn-expand") as HTMLButtonElement;
    if (expandBtn) expandBtn.onclick = () => this.expandAll();

    const collapseBtn = header.querySelector(".explorer-btn-collapse") as HTMLButtonElement;
    if (collapseBtn) collapseBtn.onclick = () => this.collapseAll();

    const refreshBtn = header.querySelector(".explorer-btn-refresh") as HTMLButtonElement;
    if (refreshBtn) refreshBtn.onclick = () => this.callbacks.onRefresh?.();

    this.container.appendChild(header);

    // 2. Search Filter Box
    const searchWrapper = document.createElement("div");
    searchWrapper.className = "project-explorer-search";
    searchWrapper.style.padding = "6px 10px";
    searchWrapper.style.borderBottom = "1px solid var(--border-color)";

    const searchInput = document.createElement("input");
    searchInput.type = "text";
    searchInput.className = "project-explorer-filter-input";
    searchInput.dataset.testid = "project-explorer-filter-input";
    searchInput.placeholder = "Filter workspace files...";
    searchInput.value = this.searchQuery;
    searchInput.style.width = "100%";
    searchInput.style.boxSizing = "border-box";
    searchInput.style.padding = "4px 8px";
    searchInput.style.fontSize = "12px";
    searchInput.style.borderRadius = "4px";
    searchInput.style.border = "1px solid var(--border-color)";
    searchInput.style.backgroundColor = "var(--bg-canvas)";
    searchInput.style.color = "var(--text-primary)";

    searchInput.oninput = (e) => {
      this.searchQuery = (e.target as HTMLInputElement).value.toLowerCase().trim();
      this.renderTree();
    };

    searchWrapper.appendChild(searchInput);
    this.container.appendChild(searchWrapper);

    // 3. Tree Content Container
    const treeContainer = document.createElement("div");
    treeContainer.id = "project-explorer-tree";
    treeContainer.className = "project-explorer-tree";
    treeContainer.dataset.testid = "project-explorer-tree";
    treeContainer.style.flex = "1";
    treeContainer.style.overflowY = "auto";
    treeContainer.style.padding = "4px 0";
    this.container.appendChild(treeContainer);

    this.renderTree();
  }

  private renderTree(): void {
    const treeContainer = this.container.querySelector("#project-explorer-tree");
    if (!treeContainer) return;
    treeContainer.innerHTML = "";

    const filtered = this.filterFolder(this.rootFolder);
    if (!filtered) {
      treeContainer.innerHTML = `
        <div style="padding: 16px; font-size: 12px; color: var(--text-muted); text-align: center;">
          No matching documents found.
        </div>
      `;
      return;
    }

    this.renderFolder(treeContainer as HTMLElement, filtered, 0);
  }

  private filterFolder(folder: ProjectFolder): ProjectFolder | null {
    if (!this.searchQuery) return folder;

    const filteredChildren: (ProjectFile | ProjectFolder)[] = [];
    for (const child of folder.children) {
      if (child.type === "file") {
        if (child.name.toLowerCase().includes(this.searchQuery)) {
          filteredChildren.push(child);
        }
      } else {
        const sub = this.filterFolder(child);
        if (sub && sub.children.length > 0) {
          filteredChildren.push(sub);
        }
      }
    }

    if (filteredChildren.length === 0 && !folder.name.toLowerCase().includes(this.searchQuery)) {
      return null;
    }

    return {
      ...folder,
      isExpanded: true, // auto-expand matching searches
      children: filteredChildren,
    };
  }

  private renderFolder(parentEl: HTMLElement, folder: ProjectFolder, level: number): void {
    const folderRow = document.createElement("div");
    folderRow.className = "project-folder-row";
    folderRow.dataset.testid = "project-folder-item";
    folderRow.style.display = "flex";
    folderRow.style.alignItems = "center";
    folderRow.style.padding = "4px 8px";
    folderRow.style.paddingLeft = `${level * 16 + 8}px`;
    folderRow.style.cursor = "pointer";
    folderRow.style.fontSize = "12px";
    folderRow.style.fontWeight = "600";
    folderRow.style.color = "var(--text-primary)";
    folderRow.style.userSelect = "none";

    const arrow = document.createElement("span");
    arrow.className = "folder-arrow";
    arrow.style.marginRight = "4px";
    arrow.style.fontSize = "10px";
    arrow.textContent = folder.isExpanded ? "▼" : "▶";

    const icon = document.createElement("span");
    icon.style.marginRight = "6px";
    icon.textContent = folder.isExpanded ? "📂" : "📁";

    const label = document.createElement("span");
    label.textContent = folder.name;

    folderRow.appendChild(arrow);
    folderRow.appendChild(icon);
    folderRow.appendChild(label);

    folderRow.onclick = (e) => {
      e.stopPropagation();
      folder.isExpanded = !folder.isExpanded;
      this.render();
    };

    parentEl.appendChild(folderRow);

    if (folder.isExpanded) {
      for (const child of folder.children) {
        if (child.type === "folder") {
          this.renderFolder(parentEl, child, level + 1);
        } else {
          this.renderFile(parentEl, child, level + 1);
        }
      }
    }
  }

  private renderFile(parentEl: HTMLElement, file: ProjectFile, level: number): void {
    const fileRow = document.createElement("div");
    const isActive = this.activeFilePath === file.path;
    fileRow.className = `project-file-row ${isActive ? "active" : ""}`;
    fileRow.dataset.testid = "project-file-item";
    fileRow.dataset.filePath = file.path;
    fileRow.dataset.docType = file.docType;
    fileRow.style.display = "flex";
    fileRow.style.alignItems = "center";
    fileRow.style.padding = "4px 8px";
    fileRow.style.paddingLeft = `${level * 16 + 8}px`;
    fileRow.style.cursor = "pointer";
    fileRow.style.fontSize = "12px";
    fileRow.style.color = isActive ? "var(--accent-primary)" : "var(--text-secondary)";
    fileRow.style.backgroundColor = isActive ? "var(--bg-active)" : "transparent";
    fileRow.style.borderRadius = "4px";

    const badge = this.createBadge(file.docType);
    const label = document.createElement("span");
    label.className = "project-file-name";
    label.textContent = file.name;
    label.style.marginLeft = "6px";
    label.style.overflow = "hidden";
    label.style.textOverflow = "ellipsis";
    label.style.whiteSpace = "nowrap";

    fileRow.appendChild(badge);
    fileRow.appendChild(label);

    fileRow.onclick = (e) => {
      e.stopPropagation();
      this.activeFilePath = file.path;
      this.render();
      this.callbacks.onOpenFile?.(file);
    };

    parentEl.appendChild(fileRow);
  }

  private createBadge(docType: DocumentType): HTMLElement {
    const badge = document.createElement("span");
    badge.className = `project-badge project-badge-${docType.toLowerCase()}`;
    badge.style.display = "inline-flex";
    badge.style.alignItems = "center";
    badge.style.justifyContent = "center";
    badge.style.minWidth = "18px";
    badge.style.height = "16px";
    badge.style.padding = "0 3px";
    badge.style.borderRadius = "3px";
    badge.style.fontSize = "9px";
    badge.style.fontWeight = "700";
    badge.style.color = "#ffffff";

    switch (docType) {
      case "Word":
        badge.style.backgroundColor = "#2b579a"; // Word Blue
        badge.textContent = "W";
        break;
      case "Excel":
        badge.style.backgroundColor = "#217346"; // Excel Green
        badge.textContent = "X";
        break;
      case "PowerPoint":
        badge.style.backgroundColor = "#d24726"; // PPT Orange
        badge.textContent = "P";
        break;
      case "PDF":
        badge.style.backgroundColor = "#b30b00"; // PDF Red
        badge.textContent = "PDF";
        break;
      case "Notes":
        badge.style.backgroundColor = "#475569"; // Notes Slate
        badge.textContent = "MD";
        break;
    }
    return badge;
  }

  public detectDocType(fileName: string): DocumentType {
    const lower = fileName.toLowerCase();
    if (lower.endsWith(".docx") || lower.endsWith(".doc")) return "Word";
    if (lower.endsWith(".xlsx") || lower.endsWith(".xls") || lower.endsWith(".csv")) return "Excel";
    if (lower.endsWith(".pptx") || lower.endsWith(".ppt")) return "PowerPoint";
    if (lower.endsWith(".pdf")) return "PDF";
    return "Notes";
  }

  private createDefaultWorkspace(): ProjectFolder {
    return {
      type: "folder",
      name: "Aether Project Workspace",
      path: "/workspace",
      isExpanded: true,
      children: [
        {
          type: "folder",
          name: "Executive Briefings",
          path: "/workspace/Executive Briefings",
          isExpanded: true,
          children: [
            {
              type: "file",
              name: "Q4_Proposal.docx",
              path: "/workspace/Executive Briefings/Q4_Proposal.docx",
              docType: "Word",
            },
            {
              type: "file",
              name: "Pitch_Deck.pptx",
              path: "/workspace/Executive Briefings/Pitch_Deck.pptx",
              docType: "PowerPoint",
            },
          ],
        },
        {
          type: "folder",
          name: "Financial Models",
          path: "/workspace/Financial Models",
          isExpanded: true,
          children: [
            {
              type: "file",
              name: "Budget_Model_2026.xlsx",
              path: "/workspace/Financial Models/Budget_Model_2026.xlsx",
              docType: "Excel",
            },
            {
              type: "file",
              name: "Sales_Forecast.csv",
              path: "/workspace/Financial Models/Sales_Forecast.csv",
              docType: "Excel",
            },
          ],
        },
        {
          type: "folder",
          name: "Legal & Compliance",
          path: "/workspace/Legal & Compliance",
          isExpanded: true,
          children: [
            {
              type: "file",
              name: "Enterprise_Contract.pdf",
              path: "/workspace/Legal & Compliance/Enterprise_Contract.pdf",
              docType: "PDF",
            },
            {
              type: "file",
              name: "Meeting_Notes.md",
              path: "/workspace/Legal & Compliance/Meeting_Notes.md",
              docType: "Notes",
            },
          ],
        },
      ],
    };
  }

  public destroy(): void {
    this.container.innerHTML = "";
  }
}

