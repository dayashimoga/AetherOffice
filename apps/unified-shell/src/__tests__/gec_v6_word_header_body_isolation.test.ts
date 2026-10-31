import { describe, it, expect, beforeEach, vi } from "vitest";
import { WriterState } from "../state/writerState";
import { WriterCanvas } from "../components/viewports/WriterCanvas";

describe("Subphase 36.1.1: Word Header/Body/Footer Strict Isolation & Non-Duplication (GEC-v6.0 Article I)", () => {
  let container: HTMLElement;
  let state: WriterState;
  let canvas: WriterCanvas;

  beforeEach(() => {
    document.body.innerHTML = '<div id="viewport-container"></div>';
    container = document.getElementById("viewport-container")!;
    state = new WriterState();
    state.resetToBlank("TestDoc.docx");

    // Mock Canvas 2D context for jsdom environment
    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = vi.fn().mockImplementation((contextId: string) => {
      if (contextId === "2d") {
        return {
          setTransform: vi.fn(),
          scale: vi.fn(),
          fillRect: vi.fn(),
          strokeRect: vi.fn(),
          fillText: vi.fn(),
          measureText: vi.fn().mockReturnValue({ width: 50 }),
          beginPath: vi.fn(),
          ellipse: vi.fn(),
          fill: vi.fn(),
          stroke: vi.fn(),
          setLineDash: vi.fn(),
          save: vi.fn(),
          restore: vi.fn(),
        } as unknown as CanvasRenderingContext2D;
      }
      return origGetContext.call(this, contextId);
    });

    canvas = new WriterCanvas(container, state);
  });

  it("1. Types 'Hello Body Text' into document body: headerText remains empty and text appears ONLY once on page", () => {
    // 1. Initial State assertions
    expect(canvas.editingZone).toBe("body");
    expect(state.editingZone).toBe("body");
    expect(state.doc.pages[0].headerText).toBe("");
    expect(state.doc.pages[0].footerText).toBe("");

    const bodyEl = container.querySelector<HTMLElement>(".word-page-body");
    expect(bodyEl).not.toBeNull();
    expect(bodyEl?.style.padding).toBe("24px 64px");
    expect(bodyEl?.style.outline).toContain("none");

    // 2. Type 'Hello Body Text' into body editor
    bodyEl!.textContent = "Hello Body Text";
    bodyEl!.dispatchEvent(new Event("input", { bubbles: true }));

    // 3. Assert headerText is empty and bodyHtml contains 'Hello Body Text'
    const page0 = state.doc.pages[0];
    expect(page0.headerText).toBe("");
    expect(page0.bodyHtml).toBe("Hello Body Text");
    expect(state.headerText).toBe("");
    expect(state.getBodyHtml(0)).toBe("Hello Body Text");

    // 4. Assert 'Hello Body Text' appears ONLY once across the entire page (inside .word-page-body)
    const pageSheet = container.querySelector<HTMLElement>(".word-page-sheet")!;
    expect(pageSheet).not.toBeNull();

    const headerEl = pageSheet.querySelector<HTMLElement>(".word-page-header")!;
    expect(headerEl.textContent).not.toContain("Hello Body Text");

    const headerInput = pageSheet.querySelector<HTMLElement>('[data-testid="word-header-input"]')!;
    expect(headerInput.textContent).toBe("Click to edit header");

    const footerEl = pageSheet.querySelector<HTMLElement>(".word-page-footer")!;
    expect(footerEl.textContent).not.toContain("Hello Body Text");

    // Verify all matching elements across container
    const matchingElements = Array.from(pageSheet.querySelectorAll("*")).filter(
      (el) => el.textContent === "Hello Body Text"
    );
    // Only the body element itself matches exact text
    expect(matchingElements.length).toBe(1);
    expect(matchingElements[0].classList.contains("word-page-body")).toBe(true);
  });

  it("2. Double-clicks Header zone, types 'Confidential Header', clicks Exit Header/Footer: headerText is saved while bodyHtml remains unchanged", () => {
    // Prepare body text first
    const bodyEl = container.querySelector<HTMLElement>(".word-page-body")!;
    bodyEl.textContent = "Hello Body Text";
    bodyEl.dispatchEvent(new Event("input", { bubbles: true }));
    expect(state.doc.pages[0].bodyHtml).toBe("Hello Body Text");

    // 1. Double-click Header zone
    const headerEl = container.querySelector<HTMLElement>('[data-testid="word-page-header"]')!;
    expect(headerEl).not.toBeNull();

    // Verify exit button is initially hidden
    const exitBtn = container.querySelector<HTMLElement>('[data-testid="btn-exit-header-footer"]')!;
    expect(exitBtn).not.toBeNull();
    expect(exitBtn.style.display).toBe("none");

    // Trigger double-click
    headerEl.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));

    // 2. Assert Header edit zone is active
    expect(canvas.editingZone).toBe("header");
    expect(state.editingZone).toBe("header");
    expect(exitBtn.style.display).toBe("inline-flex");
    expect(headerEl.classList.contains("editing-header")).toBe(true);
    expect(headerEl.style.opacity).toBe("1");

    // 3. Type 'Confidential Header' into header
    const headerInput = headerEl.querySelector<HTMLElement>('[data-testid="word-header-input"]')!;
    expect(headerInput).not.toBeNull();
    headerInput.textContent = "Confidential Header";
    headerInput.dispatchEvent(new Event("input", { bubbles: true }));

    // 4. Click Exit Header/Footer button
    exitBtn.click();

    // 5. Assert editingZone returned to body, exit button hidden, headerText saved and bodyHtml unchanged
    expect(canvas.editingZone).toBe("body");
    expect(state.editingZone).toBe("body");
    expect(exitBtn.style.display).toBe("none");

    const page0 = state.doc.pages[0];
    expect(page0.headerText).toBe("Confidential Header");
    expect(state.headerText).toBe("Confidential Header");
    expect(page0.bodyHtml).toBe("Hello Body Text");
    expect(state.getBodyHtml(0)).toBe("Hello Body Text");

    // 6. Verify DOM renders both correctly and independently
    const reloadedHeaderInput = container.querySelector<HTMLElement>('[data-testid="word-header-input"]')!;
    expect(reloadedHeaderInput.textContent).toBe("Confidential Header");

    const reloadedBody = container.querySelector<HTMLElement>(".word-page-body")!;
    expect(reloadedBody.textContent).toBe("Hello Body Text");
  });

  it("3. Double-clicks Footer zone, types 'Confidential Footer', exits via Escape: footerText is saved independently", () => {
    // 1. Double-click Footer zone
    const footerEl = container.querySelector<HTMLElement>('[data-testid="word-page-footer"]')!;
    expect(footerEl).not.toBeNull();
    footerEl.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));

    expect(canvas.editingZone).toBe("footer");
    expect(state.editingZone).toBe("footer");

    // 2. Type 'Confidential Footer'
    const footerInput = footerEl.querySelector<HTMLElement>('[data-testid="word-footer-input"]')!;
    footerInput.textContent = "Confidential Footer";
    footerInput.dispatchEvent(new Event("input", { bubbles: true }));

    // 3. Press Escape to exit
    container.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    expect(canvas.editingZone).toBe("body");
    expect(state.editingZone).toBe("body");
    expect(state.doc.pages[0].footerText).toBe("Confidential Footer");
    expect(state.footerText).toBe("Confidential Footer");
  });

  it("4. Clicking the main page sheet while editing header/footer immediately closes header/footer and focuses body", () => {
    // Enter header editing
    canvas.setEditingZone("header", 0);
    expect(canvas.editingZone).toBe("header");

    const pageSheet = container.querySelector<HTMLElement>(".word-page-sheet")!;
    // Click page sheet body
    pageSheet.click();

    expect(canvas.editingZone).toBe("body");
    expect(state.editingZone).toBe("body");
  });
});
