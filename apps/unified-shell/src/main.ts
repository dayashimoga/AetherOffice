import { TabBar, TabItem } from "./components/TabBar";
import { Ribbon, RibbonTabId } from "./components/Ribbon";
import { FormulaBar } from "./components/FormulaBar";
import { WordRuler } from "./components/WordRuler";
import { NavigationPane } from "./components/NavigationPane";
import { SlideSorter } from "./components/SlideSorter";
import { KeyboardShortcuts } from "./components/KeyboardShortcuts";
import { IPC, DocumentType } from "./ipc";

class UnifiedShellApp {
  private tabBar!: TabBar;
  private ribbon!: Ribbon;
  private formulaBar!: FormulaBar;
  private wordRuler!: WordRuler;
  private navPane!: NavigationPane;
  private slideSorter!: SlideSorter;
  private shortcuts!: KeyboardShortcuts;
  private currentDocType: DocumentType = "Word";

  constructor() {
    this.init();
  }

  private init() {
    // 1. Initialize Tab Bar
    this.tabBar = new TabBar("tab-bar-container");
    this.tabBar.setCallbacks({
      onTabChange: (tab) => this.handleTabChange(tab),
      onTabClose: (id) => this.handleTabClose(id),
      onNewTab: () => this.handleNewTab(),
    });

    // 2. Initialize Ribbon
    this.ribbon = new Ribbon("ribbon-container");
    this.ribbon.setOnAction((action, payload) => this.handleRibbonAction(action, payload));

    // 3. Initialize Sub-headers
    this.formulaBar = new FormulaBar("sub-header-container");
    this.formulaBar.setCallbacks({
      onCommit: (val) => console.log("Formula committed:", val),
      onCancel: () => console.log("Formula cancelled"),
    });

    this.wordRuler = new WordRuler("sub-header-container");

    // 4. Initialize Sidebars
    this.navPane = new NavigationPane("sidebar-container");
    this.slideSorter = new SlideSorter("sidebar-container");

    // 5. Initialize Keyboard Shortcuts
    this.shortcuts = new KeyboardShortcuts();
    this.shortcuts.setCallbacks({
      onAction: (act) => this.handleShortcutAction(act),
      onToggleAlt: () => this.ribbon.toggleKeytips(),
      onAltKeytip: (code) => this.handleAltKeytip(code),
    });

    // Default Document Tabs
    this.tabBar.addTab({
      id: "tab-1",
      docType: "Word",
      title: "Document 1",
      isDirty: false,
    });
    this.tabBar.addTab({
      id: "tab-2",
      docType: "Excel",
      title: "Financials.xlsx",
      isDirty: true,
    });
    this.tabBar.addTab({
      id: "tab-3",
      docType: "PowerPoint",
      title: "Pitch.pptx",
      isDirty: false,
    });

    // Switch to first tab
    this.tabBar.switchTab("tab-1");
  }

  private handleTabChange(tab: TabItem) {
    this.currentDocType = tab.docType;
    this.ribbon.setDocumentType(tab.docType);

    // Update Sub-header bar
    const subHeader = document.getElementById("sub-header-container");
    if (subHeader) {
      if (tab.docType === "Excel") {
        subHeader.style.display = "flex";
        this.formulaBar = new FormulaBar("sub-header-container");
        this.formulaBar.setActiveCell("A1", "=SUM(B2:B10)");
      } else if (tab.docType === "Word") {
        subHeader.style.display = "flex";
        this.wordRuler = new WordRuler("sub-header-container");
        this.wordRuler.setMargins(72, 72, 36);
      } else {
        subHeader.style.display = "none";
      }
    }

    // Update Sidebar
    if (tab.docType === "Word") {
      this.navPane = new NavigationPane("sidebar-container");
      this.navPane.setHeadings([
        { level: 1, text: "Executive Summary" },
        { level: 2, text: "Market Opportunity" },
        { level: 1, text: "Technical Architecture" },
        { level: 2, text: "Rust Engine Core" },
      ]);
    } else if (tab.docType === "PowerPoint") {
      this.slideSorter = new SlideSorter("sidebar-container");
      this.slideSorter.setSlides([
        { id: 1, slideNumber: 1, title: "Title Slide" },
        { id: 2, slideNumber: 2, title: "Problem Statement" },
        { id: 3, slideNumber: 3, title: "Solution & Demo" },
      ]);
    } else {
      const sidebar = document.getElementById("sidebar-container");
      if (sidebar) sidebar.innerHTML = '<div class="sidebar-heading">Workbook Sheets</div>';
    }

    // Update Status Bar
    const docInfo = document.getElementById("status-doc-info");
    if (docInfo) {
      docInfo.textContent = tab.docType === "Excel" ? "Sheet 1 of 3" : tab.docType === "PowerPoint" ? "Slide 1 of 3" : "Page 1 of 4";
    }
  }

  private handleTabClose(id: string) {
    IPC.closeSession(id, false).catch((err) => console.error("Close error:", err));
  }

  private handleNewTab() {
    const newId = `tab-${Date.now()}`;
    this.tabBar.addTab({
      id: newId,
      docType: this.currentDocType,
      title: `New ${this.currentDocType}`,
      isDirty: false,
    });
  }

  private handleRibbonAction(action: string, payload?: unknown) {
    console.log("Ribbon Action:", action, payload);
    const active = this.tabBar.getActiveTab();
    if (!active) return;

    if (action === "Paste" || action === "Copy" || action === "Cut") {
      console.log(`Clipboard action ${action}`);
    } else if (action === "ToggleBold" || action === "ToggleItalic" || action === "ToggleUnderline") {
      this.tabBar.setDirty(active.id, true);
    }
  }

  private handleShortcutAction(action: string) {
    console.log("Shortcut Action:", action);
    const active = this.tabBar.getActiveTab();
    if (!active) return;

    switch (action) {
      case "NextTab":
        this.tabBar.nextTab();
        break;
      case "PrevTab":
        this.tabBar.prevTab();
        break;
      case "Undo":
        IPC.executeUndo(active.id);
        break;
      case "Redo":
        IPC.executeRedo(active.id);
        break;
      case "Save":
        IPC.saveDocument(active.id);
        this.tabBar.setDirty(active.id, false);
        break;
      case "New":
        this.handleNewTab();
        break;
      case "Close":
        this.tabBar.closeTab(active.id);
        break;
      case "CellEdit":
        if (this.currentDocType === "Excel") {
          this.formulaBar.focus();
        }
        break;
    }
  }

  private handleAltKeytip(code: string) {
    const keyMap: Record<string, RibbonTabId> = {
      F: "File",
      H: "Home",
      N: "Insert",
      P: "Layout",
      M: "Formulas",
      A: "Data",
      T: "Transitions",
      K: "Animations",
      S: "SlideShow",
      R: "Review",
      W: "View",
    };
    const target = keyMap[code];
    if (target) {
      this.ribbon.setActiveTab(target);
    }
  }
}

// Start application
window.addEventListener("DOMContentLoaded", () => {
  new UnifiedShellApp();
});
