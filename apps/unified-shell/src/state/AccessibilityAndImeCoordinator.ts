/**
 * AccessibilityAndImeCoordinator.ts
 * GEC-v7.0 Stage 52 (Sub-Phase 52.2.1)
 *
 * Presentation Shell Coordinator for:
 * - Synchronized virtualized ARIA Shadow DOM overlay for screen readers (NVDA, VoiceOver, Narrator, JAWS)
 * - Spatial and sequential keyboard navigation (`F6` pane cycling, arrow keys, Tab/Shift-Tab)
 * - CJK IME composition bridge, pre-edit string buffering, and candidate popup placement
 * - Bidirectional (BiDi) Unicode paragraph text analysis and RTL rendering alignment
 */

import {
  IPC,
  type AriaBoundingBoxDto,
  type AriaNodeDto,
  type AriaRole,
  type AriaTreeDto,
  type BidiAnalysisResultDto,
  type BidiDirection,
  type BidiRunDto,
  type ImeCompositionStateDto,
  type NavFocusPane,
} from "../ipc";

export {
  type AriaBoundingBoxDto,
  type AriaNodeDto,
  type AriaRole,
  type AriaTreeDto,
  type BidiAnalysisResultDto,
  type BidiDirection,
  type BidiRunDto,
  type ImeCompositionStateDto,
  type NavFocusPane,
};

export interface AccessibilityAndImeState {
  tree: AriaTreeDto;
  ime: ImeCompositionStateDto;
  activePane: NavFocusPane;
  focusedNodeId: string | null;
  lastBidiAnalysis: BidiAnalysisResultDto | null;
  isLoading: boolean;
  errorMessage: string | null;
}

export type AccessibilityListener = (state: Readonly<AccessibilityAndImeState>) => void;

function createDefaultRootNode(): AriaNodeDto {
  return {
    id: "root-doc",
    role: "document",
    label: "Document Canvas",
    value: null,
    level: null,
    selected: false,
    checked: null,
    disabled: false,
    expanded: null,
    live: null,
    bounds: { x: 0, y: 0, width: 1024, height: 768 },
    children: [],
  };
}

export class AccessibilityAndImeCoordinator {
  private state: AccessibilityAndImeState;
  private listeners = new Set<AccessibilityListener>();
  private shadowDomRoot: HTMLElement | null = null;
  private hiddenImeInput: HTMLInputElement | null = null;

  constructor() {
    const root = createDefaultRootNode();
    this.state = {
      tree: {
        root,
        total_nodes: 1,
      },
      ime: {
        is_composing: false,
        preedit_string: "",
        cursor_position: 0,
        selection_start: 0,
        selection_end: 0,
        caret_x: 0,
        caret_y: 0,
        caret_height: 18,
        candidate_box: { x: 0, y: 18, width: 220, height: 130 },
      },
      activePane: "canvas",
      focusedNodeId: "root-doc",
      lastBidiAnalysis: null,
      isLoading: false,
      errorMessage: null,
    };
  }

  /**
   * Returns an immutable snapshot of current accessibility coordinator state.
   */
  public getState(): Readonly<AccessibilityAndImeState> {
    return this.state;
  }

  /**
   * Subscribes a listener to reactive accessibility and IME updates.
   */
  public subscribe(listener: AccessibilityListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snapshot = Object.freeze({ ...this.state });
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("[AccessibilityAndImeCoordinator] Listener error:", err);
      }
    }
  }

  /**
   * Attaches an off-screen container element to host the accessible Shadow DOM overlay.
   */
  public mountAriaOverlay(container: HTMLElement): void {
    this.shadowDomRoot = container;
    this.syncDomOverlay();
  }

  /**
   * Attaches an off-screen hidden input element for native OS IME interception.
   */
  public attachHiddenImeInput(input: HTMLInputElement): void {
    this.hiddenImeInput = input;
    this.positionHiddenImeInput();
  }

  private positionHiddenImeInput(): void {
    if (!this.hiddenImeInput) return;
    const { caret_x, caret_y, caret_height } = this.state.ime;
    this.hiddenImeInput.style.position = "absolute";
    this.hiddenImeInput.style.left = `${caret_x}px`;
    this.hiddenImeInput.style.top = `${caret_y}px`;
    this.hiddenImeInput.style.height = `${caret_height}px`;
    this.hiddenImeInput.style.width = "1px";
    this.hiddenImeInput.style.opacity = "0";
    this.hiddenImeInput.style.pointerEvents = "none";
    this.hiddenImeInput.style.zIndex = "-1";
  }

  /**
   * Refreshes the virtualized accessibility tree from native IPC backend.
   */
  public async fetchAriaTree(): Promise<AriaTreeDto> {
    this.state = { ...this.state, isLoading: true, errorMessage: null };
    this.notify();

    try {
      const tree = await IPC.accessibilityGetAriaTree();
      this.state = {
        ...this.state,
        tree,
        isLoading: false,
        errorMessage: null,
      };
      this.syncDomOverlay();
      this.notify();
      return tree;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, isLoading: false, errorMessage: message };
      this.notify();
      throw err;
    }
  }

  /**
   * Synchronizes an updated accessibility tree into the coordinator and backend.
   */
  public async updateAriaTree(tree: AriaTreeDto): Promise<number> {
    this.state = {
      ...this.state,
      tree,
    };
    this.syncDomOverlay();
    this.notify();

    try {
      return await IPC.accessibilitySetAriaTree(tree);
    } catch {
      return tree.total_nodes;
    }
  }

  /**
   * Sets the active focused element ID for virtual cursor synchronization.
   */
  public setFocusedNode(nodeId: string | null): void {
    this.state = {
      ...this.state,
      focusedNodeId: nodeId,
    };
    this.syncDomOverlay();
    this.notify();
  }

  /**
   * Begins a CJK composition session at specified canvas caret coordinates.
   */
  public async startIme(caretX: number, caretY: number, caretH: number): Promise<AriaBoundingBoxDto> {
    const candidateBox = await IPC.imeStartComposition(caretX, caretY, caretH);
    this.state = {
      ...this.state,
      ime: {
        is_composing: true,
        preedit_string: "",
        cursor_position: 0,
        selection_start: 0,
        selection_end: 0,
        caret_x: caretX,
        caret_y: caretY,
        caret_height: caretH,
        candidate_box: candidateBox,
      },
    };
    this.positionHiddenImeInput();
    this.notify();
    return candidateBox;
  }

  /**
   * Updates intermediate CJK pre-edit text during active composition.
   */
  public async updateIme(
    text: string,
    cursor: number,
    selStart: number = 0,
    selEnd: number = text.length,
    caretX?: number,
    caretY?: number,
    caretH?: number
  ): Promise<AriaBoundingBoxDto> {
    const cx = caretX ?? this.state.ime.caret_x;
    const cy = caretY ?? this.state.ime.caret_y;
    const ch = caretH ?? this.state.ime.caret_height;

    const candidateBox = await IPC.imeUpdateComposition(
      text,
      cursor,
      selStart,
      selEnd,
      cx,
      cy,
      ch
    );

    this.state = {
      ...this.state,
      ime: {
        is_composing: true,
        preedit_string: text,
        cursor_position: cursor,
        selection_start: selStart,
        selection_end: selEnd,
        caret_x: cx,
        caret_y: cy,
        caret_height: ch,
        candidate_box: candidateBox,
      },
    };
    this.positionHiddenImeInput();
    this.notify();
    return candidateBox;
  }

  /**
   * Commits current CJK composition and returns committed string.
   */
  public async finishIme(): Promise<string> {
    const committed = await IPC.imeFinishComposition();
    this.state = {
      ...this.state,
      ime: {
        ...this.state.ime,
        is_composing: false,
        preedit_string: "",
        cursor_position: 0,
      },
    };
    this.notify();
    return committed;
  }

  /**
   * Cancels active CJK composition without committing text.
   */
  public async cancelIme(): Promise<void> {
    await IPC.imeCancelComposition();
    this.state = {
      ...this.state,
      ime: {
        ...this.state.ime,
        is_composing: false,
        preedit_string: "",
        cursor_position: 0,
      },
    };
    this.notify();
  }

  /**
   * Analyzes text reading direction and computes contiguous BiDi runs.
   */
  public async analyzeBidi(text: string): Promise<BidiAnalysisResultDto> {
    const result = await IPC.bidiAnalyzeText(text);
    this.state = {
      ...this.state,
      lastBidiAnalysis: result,
    };
    this.notify();
    return result;
  }

  /**
   * Cycles focus between main presentation shell panes (F6 / Shift+F6 order).
   */
  public async cycleFocusPane(reverse: boolean = false): Promise<NavFocusPane> {
    const nextPane = await IPC.keyboardNavCyclePane(reverse);
    this.state = {
      ...this.state,
      activePane: nextPane,
    };
    this.notify();
    return nextPane;
  }

  /**
   * Manually sets the active focus pane.
   */
  public setFocusPane(pane: NavFocusPane): void {
    this.state = {
      ...this.state,
      activePane: pane,
    };
    this.notify();
  }

  /**
   * Generates clean semantic HTML markup for virtual screen reader overlay.
   */
  public renderShadowDom(): string {
    return this.renderNodeHtml(this.state.tree.root);
  }

  private renderNodeHtml(node: AriaNodeDto): string {
    const roleAttr = `role="${node.role}"`;
    const idAttr = `id="aria-${node.id}"`;
    const labelAttr = `aria-label="${node.label}"`;

    let extra = "";
    if (node.level !== null && node.level !== undefined) {
      extra += ` aria-level="${node.level}"`;
    }
    if (node.selected) {
      extra += ` aria-selected="true"`;
    }
    if (node.disabled) {
      extra += ` aria-disabled="true"`;
    }
    if (node.live) {
      extra += ` aria-live="${node.live}"`;
    }
    if (node.id === this.state.focusedNodeId) {
      extra += ` tabindex="0" autofocus`;
    } else {
      extra += ` tabindex="-1"`;
    }

    const tag = node.role === "heading" ? "h2" : node.role === "paragraph" ? "p" : "div";

    if (!node.children || node.children.length === 0) {
      const text = node.value || node.label;
      return `<${tag} ${idAttr} ${roleAttr} ${labelAttr}${extra}>${text}</${tag}>`;
    }

    const childrenHtml = node.children.map((child) => this.renderNodeHtml(child)).join("");
    return `<${tag} ${idAttr} ${roleAttr} ${labelAttr}${extra}>${childrenHtml}</${tag}>`;
  }

  /**
   * Synchronizes the generated ARIA tree into the mounted DOM container.
   */
  public syncDomOverlay(): void {
    if (!this.shadowDomRoot) return;
    this.shadowDomRoot.innerHTML = this.renderShadowDom();
  }
}

export const accessibilityAndImeCoordinator = new AccessibilityAndImeCoordinator();

