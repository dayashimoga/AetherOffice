import { TabBar, TabItem } from "./components/TabBar";
import { Ribbon, RibbonTabId } from "./components/Ribbon";
import { FormulaBar } from "./components/FormulaBar";
import { WordRuler } from "./components/WordRuler";
import { NavigationPane } from "./components/NavigationPane";
import { SlideSorter } from "./components/SlideSorter";
import { KeyboardShortcuts } from "./components/KeyboardShortcuts";
import { BottomSheet } from "./components/BottomSheet";
import { StylusInkLayer } from "./components/StylusInkLayer";
import { MobileReflowReader } from "./components/MobileReflowReader";
import { RowCardInspector } from "./components/RowCardInspector";
import { LaserPointer } from "./components/LaserPointer";
import { IPC, DocumentType, BootConfig } from "./ipc";

import { WriterCanvas } from "./components/viewports/WriterCanvas";
import { CalcGridCanvas } from "./components/viewports/CalcGridCanvas";
import { SlideStageCanvas } from "./components/viewports/SlideStageCanvas";
import { PdfViewerCanvas } from "./components/viewports/PdfViewerCanvas";

import { writerState } from "./state/writerState";
import { calcState } from "./state/calcState";
import { slidesState } from "./state/slidesState";
import { pdfState } from "./state/pdfState";
import { HubLauncher } from "./apps/hub/HubLauncher";
import { BackstageMenu } from "./components/BackstageMenu";
import { FindReplaceDialog } from "./components/FindReplaceDialog";
import { WordCountDialog } from "./components/WordCountDialog";
import { SheetTabBar } from "./components/SheetTabBar";
import { WorkbookSheetsPane } from "./components/WorkbookSheetsPane";

export class UnifiedShellApp {
  private tabBar!: TabBar;
  private ribbon!: Ribbon;
  private formulaBar!: FormulaBar;
  private wordRuler!: WordRuler;
  private navPane!: NavigationPane;
  private slideSorter!: SlideSorter;
  private shortcuts!: KeyboardShortcuts;
  private bottomSheet!: BottomSheet;
  private inkLayer!: StylusInkLayer;
  private reflowReader!: MobileReflowReader;
  private cardInspector!: RowCardInspector;
  private laserPointer!: LaserPointer;
  private currentDocType: DocumentType = "Word";

  private writerCanvas?: WriterCanvas;
  private calcGridCanvas?: CalcGridCanvas;
  private sheetTabBar?: SheetTabBar;
  private workbookSheetsPane?: WorkbookSheetsPane;
  private slideStageCanvas?: SlideStageCanvas;
  private pdfViewerCanvas?: PdfViewerCanvas;
  private hubLauncher?: HubLauncher;
  private backstageMenu!: BackstageMenu;
  private findReplaceDialog!: FindReplaceDialog;
  private wordCountDialog!: WordCountDialog;

  constructor(autoBoot = true) {
    this.init(autoBoot);
  }

  public init(autoBoot = true) {
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

    // 2.1. Initialize Backstage File Menu
    this.backstageMenu = new BackstageMenu("app");
    this.backstageMenu.setCallbacks({
      onClose: () => {
        console.log("Closed Backstage view");
      },
      onSave: async (filePath) => {
        const active = this.tabBar.getActiveTab();
        if (active) {
          const savedPath = await IPC.saveDocument(active.id, filePath);
          this.tabBar.updateTab(active.id, {
            isDirty: false,
            file_path: savedPath,
            title: savedPath.split(/[/\\]/).pop() || active.title,
          });
          const docInfo = document.getElementById("status-doc-info");
          if (docInfo) {
            docInfo.textContent = `Saved: ${savedPath.split(/[/\\]/).pop()}`;
          }
        }
      },
      onOpen: async (filePath) => {
        const summary = await IPC.openDocument(filePath);
        this.tabBar.addTab({
          id: summary.session_id,
          docType: summary.doc_type,
          title: summary.title,
          file_path: summary.file_path,
          isDirty: false,
        });
        this.tabBar.switchTab(summary.session_id);
      },
      onNew: (docType) => {
        const title =
          docType === "Word"
            ? "Untitled.docx"
            : docType === "Excel"
            ? "Book1.xlsx"
            : docType === "PowerPoint"
            ? "Presentation1.pptx"
            : "Document.pdf";
        const newId = `tab-${Date.now()}`;
        this.tabBar.addTab({
          id: newId,
          docType,
          title,
          isDirty: false,
        });
        this.tabBar.switchTab(newId);
      },
      onExportPdf: async (outputPath) => {
        const active = this.tabBar.getActiveTab();
        if (active) {
          await IPC.exportPdf(active.id, outputPath);
          const docInfo = document.getElementById("status-doc-info");
          if (docInfo) {
            docInfo.textContent = `Exported PDF: ${outputPath.split(/[/\\]/).pop()}`;
          }
        }
      },
      onCloseSession: () => {
        const active = this.tabBar.getActiveTab();
        if (active) {
          this.tabBar.closeTab(active.id);
        }
      },
    });

    // 3. Initialize Sub-headers
    this.formulaBar = new FormulaBar("sub-header-container");
    this.formulaBar.setCallbacks({
      onCommit: (val) => {
        calcState.commitFormula(val);
        this.calcGridCanvas?.render();
      },
      onCancel: () => console.log("Formula cancelled"),
      onNameBoxJump: (coord) => {
        calcState.selectCell(coord);
        this.calcGridCanvas?.render();
      },
    });

    this.wordRuler = new WordRuler("sub-header-container");
    this.wordRuler.setOnMarginsChange((left, right, indent) => {
      this.writerCanvas?.setMargins(left, right);
      if (indent !== 0) {
        writerState.execCommand("indent", indent);
      }
    });

    // 4. Initialize Sidebars
    this.navPane = new NavigationPane("sidebar-container");
    this.slideSorter = new SlideSorter("sidebar-container");
    this.slideSorter.setCallbacks({
      onSelectSlide: (id) => {
        this.slideStageCanvas?.selectSlide(id - 1);
        const docInfo = document.getElementById("status-doc-info");
        if (docInfo) docInfo.textContent = `Slide ${id} of 3`;
      },
    });

    // 5. Initialize Adaptive Mobile & Tablet Components
    this.bottomSheet = new BottomSheet("bottom-sheet");
    this.bottomSheet.setOnAction((action) => this.handleRibbonAction(action));

    this.inkLayer = new StylusInkLayer("stylus-ink-canvas");
    this.reflowReader = new MobileReflowReader("reflow-reader-container");
    this.reflowReader.setContent([
      "OpenOfficeRS features native high-performance Rust engines for Word, Excel, and PowerPoint documents.",
      "The Mobile Reflow Reader dynamically transforms rigid multi-column layouts into fluid, legible reading streams optimized for smartphone viewports.",
      "All document modifications continue to be governed by strict zero-loss ECMA-376 round-trip fidelity.",
    ]);

    this.cardInspector = new RowCardInspector("row-card-inspector-container");
    this.cardInspector.setRowData(2, 50, [
      { label: "Account ID", value: "ACC-10492", colIndex: 0 },
      { label: "Quarterly Revenue", value: "48290.00", colIndex: 1 },
      { label: "Growth Rate", value: "=B2/B1 - 1", colIndex: 2 },
      { label: "Audit Date", value: "2026-10-29", colIndex: 3 },
    ]);

    this.laserPointer = new LaserPointer("laser-pointer-dot");

    // 6. Initialize Find & Replace + Word Count Dialogs
    this.findReplaceDialog = new FindReplaceDialog();
    this.wordCountDialog = new WordCountDialog();

    const wordCountSpan = document.getElementById("status-word-count");
    if (wordCountSpan) {
      wordCountSpan.style.cursor = "pointer";
      wordCountSpan.title = "Click for detailed word count statistics";
      wordCountSpan.onclick = () => this.showWordCount();
    }

    const zoomSlider = document.getElementById("zoom-slider") as HTMLInputElement | null;
    const zoomLevel = document.getElementById("status-zoom-level");
    if (zoomSlider) {
      zoomSlider.oninput = (e) => {
        const val = parseInt((e.target as HTMLInputElement).value, 10) || 100;
        if (zoomLevel) zoomLevel.textContent = `${val}%`;
        const zoom = val / 100;
        if (this.currentDocType === "Word" && this.writerCanvas) {
          this.writerCanvas.getState().setZoom(zoom);
          this.writerCanvas.render();
        }
      };
    }

    // 7. Initialize Keyboard Shortcuts
    this.shortcuts = new KeyboardShortcuts();
    this.shortcuts.setCallbacks({
      onAction: (act) => this.handleShortcutAction(act),
      onToggleAlt: () => this.ribbon.toggleKeytips(),
      onAltKeytip: (code) => this.handleAltKeytip(code),
    });

    if (autoBoot) {
      void this.initBoot();
    }
  }

  public async initBoot() {
    try {
      const bootConfig = await IPC.getBootConfig();
      this.applyBootConfig(bootConfig);
    } catch {
      this.applyBootConfig({ app_mode: "writer" });
    }
  }

  public applyBootConfig(bootConfig: BootConfig) {
    const mode = bootConfig.app_mode || "writer";
    const initialFile = bootConfig.initial_file;

    switch (mode) {
      case "writer": {
        const title = initialFile
          ? initialFile.split(/[/\\]/).pop() || "Untitled 1.docx"
          : "Untitled 1.docx";
        this.tabBar.addTab({
          id: "tab-writer-1",
          docType: "Word",
          title,
          isDirty: false,
        });
        this.tabBar.switchTab("tab-writer-1");
        break;
      }
      case "calc": {
        const title = initialFile
          ? initialFile.split(/[/\\]/).pop() || "Book1.xlsx"
          : "Book1.xlsx";
        this.tabBar.addTab({
          id: "tab-calc-1",
          docType: "Excel",
          title,
          isDirty: false,
        });
        this.tabBar.switchTab("tab-calc-1");
        break;
      }
      case "slides": {
        const title = initialFile
          ? initialFile.split(/[/\\]/).pop() || "Presentation1.pptx"
          : "Presentation1.pptx";
        this.tabBar.addTab({
          id: "tab-slides-1",
          docType: "PowerPoint",
          title,
          isDirty: false,
        });
        this.tabBar.switchTab("tab-slides-1");
        break;
      }
      case "pdf": {
        const title = initialFile
          ? initialFile.split(/[/\\]/).pop() || "Document.pdf"
          : "Document.pdf";
        this.tabBar.addTab({
          id: "tab-pdf-1",
          docType: "PDF",
          title,
          isDirty: false,
        });
        this.tabBar.switchTab("tab-pdf-1");
        break;
      }
      case "hub":
      default: {
        this.showHub();
        break;
      }
    }
  }

  public showHub() {
    const appContainer = document.getElementById("app");
    if (appContainer) {
      appContainer.classList.add("shell-hub-mode");
    }

    const viewport = document.getElementById("viewport-container");
    if (viewport) {
      this.hubLauncher = new HubLauncher(viewport);
      this.hubLauncher.setCallbacks({
        onLaunchApp: (appMode, filePath) => {
          console.log(`Launched ${appMode} from Hub: ${filePath}`);
        },
      });
      this.hubLauncher.render();
    }

    const docInfo = document.getElementById("status-doc-info");
    if (docInfo) {
      docInfo.textContent = "AetherOffice Suite Hub";
    }
    const wordCount = document.getElementById("status-word-count");
    if (wordCount) {
      wordCount.textContent = "Ready • 4 Apps Available";
    }
  }

  public getHubLauncher(): HubLauncher | undefined {
    return this.hubLauncher;
  }

  public getWriterCanvas(): WriterCanvas | undefined {
    return this.writerCanvas;
  }

  public getWordRuler(): WordRuler {
    return this.wordRuler;
  }

  public getNavPane(): NavigationPane | undefined {
    return this.navPane;
  }

  public updateNavigationOutline() {
    if (!this.navPane || this.currentDocType !== "Word") return;
    const headings: { level: number; text: string; pageIndex: number; paragraphIndex: number }[] = [];
    writerState.doc.pages.forEach((page, pIdx) => {
      page.paragraphs.forEach((p, paraIdx) => {
        if (p.style.startsWith("Heading") || p.style === "Title" || p.style === "Subtitle") {
          let level = 1;
          if (p.style === "Heading 2" || p.style === "Heading2") level = 2;
          else if (p.style === "Heading 3" || p.style === "Heading3") level = 3;
          else if (p.style === "Subtitle") level = 2;
          const text = p.runs.map((r) => r.text).join("").trim();
          if (text) {
            headings.push({ level, text, pageIndex: pIdx, paragraphIndex: paraIdx });
          }
        }
      });
    });

    this.navPane.setHeadings(headings.map((h) => ({ level: h.level, text: h.text })));
    this.navPane.setOnSelectHeading((idx) => {
      const target = headings[idx];
      if (target) {
        writerState.cursor = {
          pageIndex: target.pageIndex,
          paragraphIndex: target.paragraphIndex,
          runIndex: 0,
          offset: 0,
        };
        writerState.clearSelection();
        this.writerCanvas?.render();
      }
    });
  }

  public showFindReplace(mode: "find" | "replace" = "find") {
    this.findReplaceDialog.setCallbacks({
      onFind: (query, matchCase) => writerState.findMatches(query, matchCase),
      onSelectMatch: (m) => {
        writerState.cursor = {
          pageIndex: m.pageIndex,
          paragraphIndex: m.paragraphIndex,
          runIndex: m.runIndex,
          offset: m.offset,
        };
        writerState.selection = {
          anchor: { ...writerState.cursor },
          focus: { ...writerState.cursor, offset: m.offset + m.length },
        };
        this.writerCanvas?.render();
      },
      onReplace: (m, rep) => {
        writerState.replaceMatch(m, rep);
        this.writerCanvas?.render();
        const active = this.tabBar.getActiveTab();
        if (active) this.tabBar.setDirty(active.id, true);
      },
      onReplaceAll: (query, rep, matchCase) => {
        const count = writerState.replaceAllMatches(query, rep, matchCase);
        this.writerCanvas?.render();
        const active = this.tabBar.getActiveTab();
        if (active) this.tabBar.setDirty(active.id, true);
        return count;
      },
      onClose: () => {
        writerState.clearSelection();
        this.writerCanvas?.render();
      },
    });
    this.findReplaceDialog.show(mode);
  }

  public showWordCount() {
    const stats = writerState.getDetailedWordCount();
    this.wordCountDialog.show(stats);
  }

  public getFindReplaceDialog(): FindReplaceDialog {
    return this.findReplaceDialog;
  }

  public getWordCountDialog(): WordCountDialog {
    return this.wordCountDialog;
  }

  public toggleBackstage() {
    if (this.backstageMenu.isOpen()) {
      this.backstageMenu.hide();
    } else {
      const active = this.tabBar.getActiveTab();
      if (active) {
        this.backstageMenu.show(
          active.docType,
          active.id,
          active.title,
          active.file_path,
          active.isDirty
        );
      } else {
        this.backstageMenu.show("Word", "sess-1", "Untitled.docx", undefined, false);
      }
    }
  }

  public getBackstageMenu(): BackstageMenu {
    return this.backstageMenu;
  }

  public getTabBar(): TabBar {
    return this.tabBar;
  }

  public getCurrentDocType(): DocumentType {
    return this.currentDocType;
  }

  public getCalcGridCanvas(): CalcGridCanvas | undefined {
    return this.calcGridCanvas;
  }

  public getSheetTabBar(): SheetTabBar | undefined {
    return this.sheetTabBar;
  }

  public getWorkbookSheetsPane(): WorkbookSheetsPane | undefined {
    return this.workbookSheetsPane;
  }

  public getFormulaBar(): FormulaBar {
    return this.formulaBar;
  }

  public updateCalcStatusBar() {
    const docInfo = document.getElementById("status-doc-info");
    if (docInfo && this.currentDocType === "Excel") {
      const current = calcState.getActiveSheetIndex() + 1;
      const total = calcState.sheets.length;
      docInfo.textContent = `Sheet ${current} of ${total}`;
    }
  }

  private handleTabChange(tab: TabItem) {
    const appContainer = document.getElementById("app");
    if (appContainer) {
      appContainer.classList.remove("shell-hub-mode");
    }

    this.currentDocType = tab.docType;
    this.ribbon.setDocumentType(tab.docType);

    // Update Sub-header bar
    const subHeader = document.getElementById("sub-header-container");
    if (subHeader) {
      if (tab.docType === "Excel") {
        subHeader.style.display = "flex";
        this.formulaBar = new FormulaBar("sub-header-container");
        this.formulaBar.setCallbacks({
          onCommit: (val) => {
            calcState.commitFormula(val);
            this.calcGridCanvas?.render();
            this.tabBar.setDirty(tab.id, true);
          },
          onCancel: () => {
            this.calcGridCanvas?.cancelEditing();
          },
          onNameBoxJump: (coord) => {
            calcState.selectCell(coord);
            this.calcGridCanvas?.render();
          },
          onInput: (val) => {
            this.calcGridCanvas?.setEditingValue(val);
          },
        });
        this.formulaBar.setActiveCell(calcState.selectedCell, calcState.formulaBarText);
      } else if (tab.docType === "Word") {
        subHeader.style.display = "flex";
        this.wordRuler = new WordRuler("sub-header-container");
        this.wordRuler.setOnMarginsChange((left, right, indent) => {
          this.writerCanvas?.setMargins(left, right);
          if (indent !== 0) {
            writerState.execCommand("indent", indent);
          }
        });
        this.wordRuler.setMargins(96, 96, 0);
      } else {
        subHeader.style.display = "none";
      }
    }

    // Update Sidebar
    if (tab.docType === "Word") {
      this.navPane = new NavigationPane("sidebar-container");
      this.updateNavigationOutline();
    } else if (tab.docType === "PowerPoint") {
      this.slideSorter = new SlideSorter("sidebar-container");
      this.slideSorter.setCallbacks({
        onSelectSlide: (id) => {
          slidesState.selectSlide(id - 1);
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
        },
        onReorder: (fromIdx, toIdx) => {
          slidesState.reorderSlide(fromIdx, toIdx);
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
          this.tabBar.setDirty(tab.id, true);
        },
        onAddSlide: () => {
          slidesState.addSlide();
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
          this.tabBar.setDirty(tab.id, true);
        },
        onDuplicateSlide: (idx) => {
          slidesState.duplicateSlide(idx);
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
          this.tabBar.setDirty(tab.id, true);
        },
        onDeleteSlide: (idx) => {
          slidesState.deleteSlide(idx);
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
          this.tabBar.setDirty(tab.id, true);
        },
        onToggleHide: (idx) => {
          slidesState.toggleHideSlide(idx);
          this.updateSlideSorter();
          this.tabBar.setDirty(tab.id, true);
        },
      });
      this.updateSlideSorter();
    } else if (tab.docType === "Excel") {
      this.workbookSheetsPane = new WorkbookSheetsPane("sidebar-container", calcState);
      this.workbookSheetsPane.setCallbacks({
        onSheetSwitched: () => {
          this.calcGridCanvas?.render();
          this.sheetTabBar?.refresh();
          this.updateCalcStatusBar();
          this.formulaBar.setActiveCell(calcState.selectedCell, calcState.formulaBarText);
        },
        onSheetAdded: () => {
          this.calcGridCanvas?.render();
          this.sheetTabBar?.refresh();
          this.updateCalcStatusBar();
          this.tabBar.setDirty(tab.id, true);
        },
        onSheetRenamed: () => {
          this.sheetTabBar?.refresh();
          this.tabBar.setDirty(tab.id, true);
        },
        onSheetDeleted: () => {
          this.calcGridCanvas?.render();
          this.sheetTabBar?.refresh();
          this.updateCalcStatusBar();
          this.tabBar.setDirty(tab.id, true);
        },
      });
    } else {
      const sidebar = document.getElementById("sidebar-container");
      if (sidebar) sidebar.innerHTML = '<div class="sidebar-heading">Navigation</div>';
    }

    // Update Active Viewport Controller
    const viewportContainer = document.getElementById("viewport-container");
    if (viewportContainer) {
      if (this.writerCanvas) {
        this.writerCanvas.destroy();
        this.writerCanvas = undefined;
      }
      this.calcGridCanvas = undefined;
      this.sheetTabBar = undefined;
      this.slideStageCanvas = undefined;
      this.pdfViewerCanvas = undefined;

      if (tab.docType === "Word") {
        this.writerCanvas = new WriterCanvas(viewportContainer, writerState);
        this.writerCanvas.setOnStatusChange((pInfo, words) => {
          const docInfo = document.getElementById("status-doc-info");
          const wordInfo = document.getElementById("status-word-count");
          if (docInfo) docInfo.textContent = pInfo;
          if (wordInfo) wordInfo.textContent = `${words} words`;
        });
        this.writerCanvas.setOnDocModified((action, payload) => {
          this.tabBar.setDirty(tab.id, true);
          if (action === "InsertText") {
            const text = (payload as { text?: string })?.text || "";
            IPC.dispatchUiAction(tab.id, { type: "InsertText", payload: { text } }).catch(() => {});
          } else if (action === "DeleteText") {
            IPC.dispatchUiAction(tab.id, {
              type: "DeleteTextRange",
              payload: { offset: 0, length: 1 },
            }).catch(() => {});
          }
        });
      } else if (tab.docType === "Excel") {
        viewportContainer.innerHTML = `
          <div class="calc-viewport-wrapper">
            <div class="calc-grid-container" id="calc-grid-container"></div>
            <div id="calc-sheet-tab-bar-container"></div>
          </div>
        `;
        const gridContainer = document.getElementById("calc-grid-container")!;
        const sheetTabContainer = document.getElementById("calc-sheet-tab-bar-container")!;

        this.calcGridCanvas = new CalcGridCanvas(gridContainer, calcState);
        this.calcGridCanvas.setOnCellSelected((coord, val) => {
          this.formulaBar.setActiveCell(coord, val);
        });
        this.calcGridCanvas.setOnCellEditing((coord, val) => {
          this.formulaBar.setActiveCell(coord, val);
        });
        this.calcGridCanvas.setOnCellModified((_coord, _val) => {
          this.tabBar.setDirty(tab.id, true);
          this.workbookSheetsPane?.refresh();
        });

        this.sheetTabBar = new SheetTabBar(sheetTabContainer, calcState);
        this.sheetTabBar.setCallbacks({
          onSheetSwitched: () => {
            this.calcGridCanvas?.render();
            this.workbookSheetsPane?.refresh();
            this.updateCalcStatusBar();
            this.formulaBar.setActiveCell(calcState.selectedCell, calcState.formulaBarText);
          },
          onSheetAdded: () => {
            this.calcGridCanvas?.render();
            this.workbookSheetsPane?.refresh();
            this.updateCalcStatusBar();
            this.tabBar.setDirty(tab.id, true);
          },
          onSheetRenamed: () => {
            this.workbookSheetsPane?.refresh();
            this.tabBar.setDirty(tab.id, true);
          },
          onSheetDeleted: () => {
            this.calcGridCanvas?.render();
            this.workbookSheetsPane?.refresh();
            this.updateCalcStatusBar();
            this.tabBar.setDirty(tab.id, true);
          },
        });
      } else if (tab.docType === "PowerPoint") {
        this.slideStageCanvas = new SlideStageCanvas(viewportContainer, slidesState);
        this.slideStageCanvas.setOnDirty(() => this.tabBar.setDirty(tab.id, true));
      } else if (tab.docType === "PDF") {
        this.pdfViewerCanvas = new PdfViewerCanvas(viewportContainer, pdfState);
      }
    }

    // Update Status Bar
    const docInfo = document.getElementById("status-doc-info");
    if (docInfo) {
      if (tab.docType === "Excel") {
        this.updateCalcStatusBar();
      } else {
        docInfo.textContent = tab.docType === "PowerPoint" ? "Slide 1 of 3" : "Page 1 of 1";
      }
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

  private updateSlideSorter() {
    if (!this.slideSorter) return;
    const thumbs = slidesState.slides.map((s, idx) => ({
      id: idx + 1,
      slideNumber: idx + 1,
      title: s.title,
      shapeCount: s.shapes.length,
      isHidden: s.isHidden,
    }));
    this.slideSorter.setSlides(thumbs, slidesState.currentSlideIndex + 1);
    const docInfo = document.getElementById("status-doc-info");
    if (docInfo && this.currentDocType === "PowerPoint") {
      docInfo.textContent = `Slide ${slidesState.currentSlideIndex + 1} of ${slidesState.slides.length}`;
    }
  }

  private handleRibbonAction(action: string, payload?: unknown) {
    console.log("Ribbon Action:", action, payload);

    if (action === "ToggleBackstage") {
      this.toggleBackstage();
      return;
    }
    if (action === "Find") {
      this.showFindReplace("find");
      return;
    }
    if (action === "Replace") {
      this.showFindReplace("replace");
      return;
    }
    if (action === "WordCount") {
      this.showWordCount();
      return;
    }
    if (action === "ToggleRuler") {
      const subHeader = document.getElementById("sub-header-container");
      if (subHeader) {
        subHeader.style.display = subHeader.style.display === "none" ? "flex" : "none";
      }
      return;
    }
    if (action === "ToggleNavPane") {
      const sidebar = document.getElementById("sidebar-container");
      if (sidebar) {
        sidebar.style.display = sidebar.style.display === "none" ? "block" : "none";
      }
      return;
    }
    if (action === "ViewWebLayout") {
      writerState.doc.viewMode = "web";
      this.writerCanvas?.render();
      return;
    }
    if (action === "ViewPrintLayout") {
      writerState.doc.viewMode = "print";
      this.writerCanvas?.render();
      return;
    }
    if (action === "ViewReadMode") {
      this.reflowReader.toggle();
      return;
    }

    const active = this.tabBar.getActiveTab();
    if (!active) return;

    if (this.currentDocType === "Word") {
      if (action === "InsertBlankPage" || action === "InsertPageBreak") {
        this.writerCanvas?.insertPageBreak();
      } else {
        writerState.execCommand(action, payload);
        this.writerCanvas?.render();
        if (action === "SetStyle" || action === "setStyle") {
          this.updateNavigationOutline();
        } else if (action === "SetMargins" || action === "setMargins") {
          const leftMargin = payload === "narrow" ? 48 : payload === "wide" ? 144 : 96;
          this.wordRuler.setMargins(leftMargin, leftMargin, 0);
          this.writerCanvas?.setMargins(leftMargin, leftMargin);
        }
      }
      this.tabBar.setDirty(active.id, true);
    } else if (this.currentDocType === "PowerPoint") {
      if (action === "StartShowBeginning") {
        slidesState.startShow(true);
        if (!document.fullscreenElement) {
          document.documentElement.requestFullscreen().catch(() => {});
        }
        this.slideStageCanvas?.render();
        this.updateSlideSorter();
      } else if (action === "StartShowCurrent") {
        slidesState.startShow(false);
        if (!document.fullscreenElement) {
          document.documentElement.requestFullscreen().catch(() => {});
        }
        this.slideStageCanvas?.render();
        this.updateSlideSorter();
      } else {
        const changed = slidesState.execCommand(action, payload);
        if (changed) {
          this.slideStageCanvas?.render();
          this.updateSlideSorter();
          this.tabBar.setDirty(active.id, true);
        }
      }
    } else if (this.currentDocType === "Excel") {
      if (action === "ToggleBold") {
        const cell = calcState.cells.get(calcState.selectedCell);
        if (cell) cell.bold = !cell.bold;
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "ToggleItalic") {
        const cell = calcState.cells.get(calcState.selectedCell);
        if (cell) cell.italic = !cell.italic;
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "ToggleUnderline") {
        const cell = calcState.cells.get(calcState.selectedCell);
        if (cell) cell.underline = !cell.underline;
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "SetFontColor") {
        const cell = calcState.cells.get(calcState.selectedCell);
        if (cell) cell.color = String(payload || "#15803d");
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "SetFillColor") {
        const cell = calcState.cells.get(calcState.selectedCell);
        if (cell) cell.bgColor = String(payload || "#dcfce7");
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "InsertRow") {
        const cr = calcState.coordToColRow(calcState.selectedCell);
        calcState.insertRow(cr.row);
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "DeleteRow") {
        const cr = calcState.coordToColRow(calcState.selectedCell);
        calcState.deleteRow(cr.row);
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "InsertColumn") {
        const cr = calcState.coordToColRow(calcState.selectedCell);
        calcState.insertColumn(cr.col);
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "DeleteColumn") {
        const cr = calcState.coordToColRow(calcState.selectedCell);
        calcState.deleteColumn(cr.col);
        this.calcGridCanvas?.render();
        this.tabBar.setDirty(active.id, true);
      } else if (action === "Recalculate") {
        calcState.recalculateAll();
        this.calcGridCanvas?.render();
      }
    } else if (action === "SetFormField") {
      if (this.currentDocType === "PDF") {
        this.pdfViewerCanvas?.render();
      }
    } else if (action === "ToggleReflow") {
      this.reflowReader.toggle();
    } else if (action === "ToggleInk") {
      this.inkLayer.resize();
      console.log("Ink layer activated for stylus freehand annotations");
    } else if (action === "ToggleLaserPointer") {
      this.laserPointer.setActive(true);
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
        if (this.currentDocType === "PowerPoint") {
          slidesState.undo();
          this.slideStageCanvas?.render();
        } else {
          IPC.executeUndo(active.id);
        }
        break;
      case "Redo":
        if (this.currentDocType === "PowerPoint") {
          slidesState.redo();
          this.slideStageCanvas?.render();
        } else {
          IPC.executeRedo(active.id);
        }
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
      case "Find":
        this.showFindReplace("find");
        break;
      case "Replace":
        this.showFindReplace("replace");
        break;
      case "CellEdit":
        if (this.currentDocType === "Excel") {
          this.formulaBar.focus();
        }
        break;
    }
  }

  private handleAltKeytip(code: string) {
    if (code === "F") {
      this.toggleBackstage();
      return;
    }
    const keyMap: Record<string, RibbonTabId> = {
      H: "Home",
      N: "Insert",
      P:
        this.currentDocType === "Excel"
          ? "Page Layout"
          : this.currentDocType === "PDF"
          ? "Page Tools"
          : "Layout",
      G: "Design",
      M: "Formulas",
      A: "Data",
      T: "Transitions",
      K: "Animations",
      S:
        this.currentDocType === "PowerPoint"
          ? "Slide Show"
          : this.currentDocType === "Word"
          ? "References"
          : "Sign & Protect",
      E: "Edit PDF",
      C: "Comment & Annotate",
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
