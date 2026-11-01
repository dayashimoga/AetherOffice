/**
 * accessibility_ime_coordinator.test.ts
 * GEC-v7.0 Stage 52 (Sub-Phase 52.2.1)
 *
 * Vitest Unit & Integration Suite for AccessibilityAndImeCoordinator:
 * - Default state initialization and subscriber notification
 * - Fetching virtualized ARIA accessibility tree from backend
 * - Updating ARIA tree with nested document structure (headings, paragraphs, grid cells)
 * - Generating semantic HTML markup for virtual screen reader Shadow DOM overlay
 * - Attaching container and synchronizing DOM elements
 * - Intercepting CJK IME composition start, update, finish, and cancel lifecycle
 * - Positioning off-screen hidden IME input and candidate list popup at caret
 * - Unicode BiDi paragraph direction analysis (RTL vs LTR)
 * - Sequential F6 forward and Shift+F6 reverse keyboard pane navigation cycling
 * - Focus node synchronization and active tab index management
 */

import { describe, it, expect, beforeEach } from "vitest";
import {
  AccessibilityAndImeCoordinator,
  type AriaTreeDto,
} from "../state/AccessibilityAndImeCoordinator";

describe("Sub-Phase 52.2.1: AccessibilityAndImeCoordinator", () => {
  let coordinator: AccessibilityAndImeCoordinator;

  beforeEach(() => {
    coordinator = new AccessibilityAndImeCoordinator();
  });

  it("1. initializes with default document root, idle IME state, and canvas focus pane", () => {
    const state = coordinator.getState();
    expect(state.tree.total_nodes).toBe(1);
    expect(state.tree.root.role).toBe("document");
    expect(state.tree.root.id).toBe("root-doc");
    expect(state.ime.is_composing).toBe(false);
    expect(state.ime.preedit_string).toBe("");
    expect(state.activePane).toBe("canvas");
    expect(state.focusedNodeId).toBe("root-doc");
    expect(state.isLoading).toBe(false);
    expect(state.errorMessage).toBeNull();
  });

  it("2. fetches virtualized ARIA accessibility tree from backend", async () => {
    const tree = await coordinator.fetchAriaTree();
    expect(tree).toBeDefined();
    expect(tree.root.id).toBe("root-doc");
    expect(coordinator.getState().tree).toEqual(tree);
  });

  it("3. updates ARIA tree with nested document structure and node count", async () => {
    const customTree: AriaTreeDto = {
      root: {
        id: "doc-page-1",
        role: "document",
        label: "Word Page 1",
        selected: false,
        disabled: false,
        children: [
          {
            id: "heading-1",
            role: "heading",
            label: "Introduction",
            level: 1,
            selected: false,
            disabled: false,
            children: [],
          },
          {
            id: "para-1",
            role: "paragraph",
            label: "First accessible paragraph text.",
            selected: false,
            disabled: false,
            children: [],
          },
        ],
      },
      total_nodes: 3,
    };

    const count = await coordinator.updateAriaTree(customTree);
    expect(count).toBe(3);
    expect(coordinator.getState().tree.total_nodes).toBe(3);
    expect(coordinator.getState().tree.root.children.length).toBe(2);
  });

  it("4. generates semantic HTML markup for screen reader virtual overlay", async () => {
    const customTree: AriaTreeDto = {
      root: {
        id: "section-1",
        role: "document",
        label: "Section Container",
        selected: false,
        disabled: false,
        children: [
          {
            id: "h2-audit",
            role: "heading",
            label: "Audit Findings",
            level: 2,
            selected: false,
            disabled: false,
            children: [],
          },
          {
            id: "p-body",
            role: "paragraph",
            label: "All requirements met.",
            selected: false,
            disabled: false,
            children: [],
          },
        ],
      },
      total_nodes: 3,
    };

    await coordinator.updateAriaTree(customTree);
    coordinator.setFocusedNode("h2-audit");

    const html = coordinator.renderShadowDom();
    expect(html).toContain("id=\"aria-section-1\"");
    expect(html).toContain("role=\"document\"");
    expect(html).toContain("<h2 id=\"aria-h2-audit\" role=\"heading\" aria-label=\"Audit Findings\" aria-level=\"2\" tabindex=\"0\" autofocus>Audit Findings</h2>");
    expect(html).toContain("<p id=\"aria-p-body\" role=\"paragraph\" aria-label=\"All requirements met.\" tabindex=\"-1\">All requirements met.</p>");
  });

  it("5. mounts accessible DOM overlay and synchronizes generated HTML", async () => {
    const container = document.createElement("div");
    container.id = "aria-overlay-test";
    document.body.appendChild(container);

    coordinator.mountAriaOverlay(container);
    expect(container.innerHTML).toContain("id=\"aria-root-doc\"");

    // Update tree and verify immediate DOM sync
    const customTree: AriaTreeDto = {
      root: {
        id: "mounted-doc",
        role: "document",
        label: "Live Synced Document",
        selected: false,
        disabled: false,
        children: [],
      },
      total_nodes: 1,
    };
    await coordinator.updateAriaTree(customTree);

    expect(container.innerHTML).toContain("id=\"aria-mounted-doc\"");
    document.body.removeChild(container);
  });

  it("6. starts CJK IME composition and positions candidate box at caret", async () => {
    const hiddenInput = document.createElement("input");
    document.body.appendChild(hiddenInput);
    coordinator.attachHiddenImeInput(hiddenInput);

    const cbox = await coordinator.startIme(150, 220, 20);
    expect(coordinator.getState().ime.is_composing).toBe(true);
    expect(coordinator.getState().ime.caret_x).toBe(150);
    expect(coordinator.getState().ime.caret_y).toBe(220);
    expect(cbox.x).toBe(150);
    expect(cbox.y).toBe(240); // 220 + 20

    expect(hiddenInput.style.left).toBe("150px");
    expect(hiddenInput.style.top).toBe("220px");

    document.body.removeChild(hiddenInput);
  });

  it("7. updates intermediate CJK pre-edit text and cursor offset", async () => {
    await coordinator.startIme(100, 150, 18);
    const updatedBox = await coordinator.updateIme("nihon", 5, 0, 5, 130, 150, 18);

    expect(coordinator.getState().ime.is_composing).toBe(true);
    expect(coordinator.getState().ime.preedit_string).toBe("nihon");
    expect(coordinator.getState().ime.cursor_position).toBe(5);
    expect(updatedBox.x).toBe(130);
  });

  it("8. finishes CJK IME composition and returns committed string", async () => {
    await coordinator.startIme(100, 150, 18);
    await coordinator.updateIme("kanji", 5, 0, 5);

    const committed = await coordinator.finishIme();
    expect(committed).toBe("kanji");
    expect(coordinator.getState().ime.is_composing).toBe(false);
    expect(coordinator.getState().ime.preedit_string).toBe("");
  });

  it("9. cancels CJK IME composition without committing", async () => {
    await coordinator.startIme(100, 150, 18);
    await coordinator.updateIme("draft", 5);

    await coordinator.cancelIme();
    expect(coordinator.getState().ime.is_composing).toBe(false);
    expect(coordinator.getState().ime.preedit_string).toBe("");
  });

  it("10. analyzes BiDi Unicode direction and cycles keyboard focus panes (F6 / Shift+F6)", async () => {
    // BiDi analysis: Arabic -> RTL
    const arabicRes = await coordinator.analyzeBidi("مرحبا بكم");
    expect(arabicRes.base_direction).toBe("rtl");
    expect(coordinator.getState().lastBidiAnalysis).toEqual(arabicRes);

    // BiDi analysis: English -> LTR
    const englishRes = await coordinator.analyzeBidi("Antigravity AetherOffice");
    expect(englishRes.base_direction).toBe("ltr");

    // Keyboard Paning: start at canvas
    expect(coordinator.getState().activePane).toBe("canvas");

    // Forward cycle (F6): Canvas -> StatusBar -> TitleBar -> Ribbon -> SidePane -> Canvas
    const p1 = await coordinator.cycleFocusPane(false);
    expect(p1).toBe("status-bar");
    const p2 = await coordinator.cycleFocusPane(false);
    expect(p2).toBe("title-bar");
    const p3 = await coordinator.cycleFocusPane(false);
    expect(p3).toBe("ribbon");

    // Reverse cycle (Shift+F6): Ribbon -> TitleBar
    const pRev = await coordinator.cycleFocusPane(true);
    expect(pRev).toBe("title-bar");
  });
});

