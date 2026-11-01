/**
 * FocusRouter.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.1.2)
 *
 * Modular focus and shortcut routing controller responsible for:
 * - Tracking focused pane, session, and active document type
 * - Routing global keyboard shortcuts across multi-format viewports
 * - Dynamic Alt keytip resolution and contextual ribbon/copilot synchronization
 */

import { DocumentType, IPC } from "../ipc";
import { RibbonTabId } from "../components/Ribbon";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";

export interface ShortcutRouterDelegate {
  getActiveTabId: () => string | null;
  getCurrentDocType: () => DocumentType;
  onNextTab: () => void;
  onPrevTab: () => void;
  onSave: () => Promise<void> | void;
  onSaveAs: () => void;
  onOpen: () => void;
  onPrint: () => void;
  onNew: () => void;
  onClose: (tabId: string) => void;
  onFind: () => void;
  onReplace: () => void;
  onFocusFormulaBar?: () => void;
  onCalcCopy?: () => void;
  onCalcCut?: () => void;
  onCalcPaste?: () => void;
  onRenderCurrent?: () => void;
  onSaveWorkspace?: () => void;
  onOpenWorkspace?: () => void;
}

export type FocusChangeListener = (paneId: string, sessionId: string | null, docType: DocumentType) => void;

export class FocusRouter {
  private focusedPaneId = "primary-pane";
  private focusedSessionId: string | null = null;
  private focusedDocType: DocumentType = "Word";
  private listeners: FocusChangeListener[] = [];

  constructor(initialDocType: DocumentType = "Word") {
    this.focusedDocType = initialDocType;
  }

  public getFocusedPaneId(): string {
    return this.focusedPaneId;
  }

  public getFocusedSessionId(): string | null {
    return this.focusedSessionId;
  }

  public getFocusedDocType(): DocumentType {
    return this.focusedDocType;
  }

  public setFocus(paneId: string, sessionId: string | null, docType: DocumentType): void {
    const changed =
      this.focusedPaneId !== paneId ||
      this.focusedSessionId !== sessionId ||
      this.focusedDocType !== docType;

    this.focusedPaneId = paneId;
    this.focusedSessionId = sessionId;
    this.focusedDocType = docType;

    if (changed) {
      for (const listener of this.listeners) {
        listener(paneId, sessionId, docType);
      }
    }
  }

  public onFocusChange(listener: FocusChangeListener): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  /**
   * Dispatches keyboard shortcut actions to the currently focused application/document.
   */
  public handleShortcutAction(action: string, delegate: ShortcutRouterDelegate): void {
    if (action === "SaveWorkspace") {
      delegate.onSaveWorkspace?.();
      return;
    }
    if (action === "OpenWorkspace") {
      delegate.onOpenWorkspace?.();
      return;
    }

    const activeTabId = delegate.getActiveTabId();
    if (!activeTabId) return;

    const docType = delegate.getCurrentDocType();

    switch (action) {
      case "NextTab":
        delegate.onNextTab();
        break;
      case "PrevTab":
        delegate.onPrevTab();
        break;
      case "Undo":
        if (docType === "PowerPoint") {
          slidesState.undo();
        } else if (docType === "PDF") {
          pdfState.undo();
        } else if (docType === "Notes") {
          notesState.undo();
        } else if (docType === "Excel") {
          calcState.undo();
        } else {
          writerState.undo();
          IPC.executeUndo(activeTabId);
        }
        delegate.onRenderCurrent?.();
        break;
      case "Redo":
        if (docType === "PowerPoint") {
          slidesState.redo();
        } else if (docType === "PDF") {
          pdfState.redo();
        } else if (docType === "Notes") {
          notesState.redo();
        } else if (docType === "Excel") {
          calcState.redo();
        } else {
          writerState.redo();
          IPC.executeRedo(activeTabId);
        }
        delegate.onRenderCurrent?.();
        break;
      case "Save":
        void delegate.onSave();
        break;
      case "SaveAs":
        delegate.onSaveAs();
        break;
      case "Open":
        delegate.onOpen();
        break;
      case "Print":
        delegate.onPrint();
        break;
      case "New":
        delegate.onNew();
        break;
      case "Close":
        delegate.onClose(activeTabId);
        break;
      case "Find":
        delegate.onFind();
        break;
      case "Replace":
        delegate.onReplace();
        break;
      case "SaveWorkspace":
        delegate.onSaveWorkspace?.();
        break;
      case "OpenWorkspace":
        delegate.onOpenWorkspace?.();
        break;
      case "CellEdit":
        if (docType === "Excel") {
          delegate.onFocusFormulaBar?.();
        }
        break;
      case "Copy":
        if (docType === "Excel") {
          delegate.onCalcCopy?.();
        }
        break;
      case "Cut":
        if (docType === "Excel") {
          delegate.onCalcCut?.();
        }
        break;
      case "Paste":
        if (docType === "Excel") {
          delegate.onCalcPaste?.();
        }
        break;
    }
  }

  /**
   * Resolves the target ribbon tab or backstage modal for an Alt keytip.
   */
  public resolveAltKeytip(
    code: string,
    currentDocType: DocumentType
  ): { type: "tab"; tabId: RibbonTabId } | { type: "backstage" } | null {
    if (code === "F") {
      return { type: "backstage" };
    }

    const keyMap: Record<string, RibbonTabId> = {
      H: "Home",
      N: "Insert",
      P:
        currentDocType === "Excel"
          ? "Page Layout"
          : currentDocType === "PDF"
          ? "Page Tools"
          : "Layout",
      G: "Design",
      M: "Formulas",
      A: "Data",
      T: "Transitions",
      K: "Animations",
      S:
        currentDocType === "PowerPoint"
          ? "Slide Show"
          : currentDocType === "Word"
          ? "References"
          : "Sign & Protect",
      E: "Edit PDF",
      C: "Comment & Annotate",
      R: "Review",
      W: "View",
    };

    const target = keyMap[code];
    if (target) {
      return { type: "tab", tabId: target };
    }
    return null;
  }

  /**
   * Maps document type to Copilot AI mode.
   */
  public getCopilotMode(docType: DocumentType): "writer" | "calc" | "slides" {
    const dt = docType.toLowerCase();
    if (dt === "excel" || dt === "calc") return "calc";
    if (dt === "powerpoint" || dt === "slides") return "slides";
    return "writer";
  }

  /**
   * Maps document type to CollabBar presence mode.
   */
  public getCollabMode(docType: DocumentType): "writer" | "calc" | "slides" {
    return this.getCopilotMode(docType);
  }

  public destroy(): void {
    this.listeners = [];
  }
}

