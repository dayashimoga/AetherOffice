import { describe, it, expect, beforeEach } from "vitest";
import { Ribbon } from "../components/Ribbon";
import { BottomSheet } from "../components/BottomSheet";

describe("Strict Per-App Contextual Ribbon Router & Mobile Bar Cleanup (GEC-v2.0 Subphase 8.2.2)", () => {
  let ribbonContainer: HTMLElement;
  let ribbon: Ribbon;

  beforeEach(() => {
    document.body.innerHTML = `
      <section id="ribbon-container" class="ribbon-bar"></section>
      <div id="bottom-sheet" class="bottom-sheet"></div>
    `;
    ribbonContainer = document.getElementById("ribbon-container")!;
    ribbon = new Ribbon("ribbon-container");
  });

  describe("1. Ribbon Tab State & Active Indicator Bugfix", () => {
    it("renders File as a backstage button without active underline indicator", () => {
      const fileBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      expect(fileBtn).not.toBeNull();
      expect(fileBtn?.classList.contains("ribbon-file-tab")).toBe(true);
      expect(fileBtn?.classList.contains("active")).toBe(false);

      const homeBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-home"]');
      expect(homeBtn).not.toBeNull();
      expect(homeBtn?.classList.contains("active")).toBe(true);
    });

    it("maintains strict activeRibbonTab state: clicking tab updates active class and switches toolbar panel", () => {
      const insertBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-insert"]');
      expect(insertBtn).not.toBeNull();

      // Click Insert tab (causes re-render)
      insertBtn?.click();

      // Query freshly rendered DOM elements
      const homeBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-home"]');
      const updatedInsertBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-insert"]');
      const fileBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      expect(homeBtn?.classList.contains("active")).toBe(false);
      expect(updatedInsertBtn?.classList.contains("active")).toBe(true);
      expect(fileBtn?.classList.contains("active")).toBe(false);
      expect(ribbon.getActiveTab()).toBe("Insert");

      // Body should now contain Insert groups (Tables, Illustrations, Pages)
      const body = document.getElementById("ribbon-tools-body");
      expect(body?.textContent).toContain("Tables");
      expect(body?.textContent).toContain("Illustrations");
      expect(body?.textContent).toContain("Pages");
    });

    it("clicking File tab triggers Backstage toggle action without making File tab active or underlined", () => {
      let triggeredAction = "";
      ribbon.setOnAction((action) => {
        triggeredAction = action;
      });

      const fileBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      fileBtn?.click();

      expect(triggeredAction).toBe("ToggleBackstage");
      expect(fileBtn?.classList.contains("active")).toBe(false);
      expect(ribbon.getActiveTab()).toBe("Home");
    });

    it("cycles through all Writer tabs and verifies every tab switches active indicator and body", () => {
      const tabs = ribbon.getVisibleTabs();
      expect(tabs).toEqual(["Home", "Insert", "Layout", "References", "Review", "View"]);

      tabs.forEach((tab) => {
        ribbon.setActiveTab(tab);
        expect(ribbon.getActiveTab()).toBe(tab);

        const tabBtn = ribbonContainer.querySelector<HTMLButtonElement>(
          `[data-testid="ribbon-tab-${tab.toLowerCase().replace(/\s+/g, "-")}"]`
        );
        expect(tabBtn?.classList.contains("active")).toBe(true);

        const fileBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
        expect(fileBtn?.classList.contains("active")).toBe(false);
      });
    });
  });

  describe("2. Dedicated Writer Ribbon Schema", () => {
    beforeEach(() => {
      ribbon.setDocumentType("Word");
    });

    it("has only Writer tabs: Home, Insert, Layout, References, Review, View", () => {
      expect(ribbon.getVisibleTabs()).toEqual([
        "Home",
        "Insert",
        "Layout",
        "References",
        "Review",
        "View",
      ]);
      expect(ribbon.getVisibleTabs()).not.toContain("Formulas");
      expect(ribbon.getVisibleTabs()).not.toContain("Slide Show");
      expect(ribbon.getVisibleTabs()).not.toContain("Edit PDF");
    });

    it("renders Home tab with Word typography styles (Normal, Heading 1, Heading 2)", () => {
      ribbon.setActiveTab("Home");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Normal");
      expect(body.textContent).toContain("Heading 1");
      expect(body.textContent).toContain("Heading 2");
      expect(body.textContent).toContain("Styles");
      expect(body.textContent).toContain("Paragraph");
    });

    it("renders References tab with TOC, Footnotes, Citations, and Captions", () => {
      ribbon.setActiveTab("References");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Table of Contents");
      expect(body.textContent).toContain("Footnotes");
      expect(body.textContent).toContain("Citations & Bibliography");
      expect(body.textContent).toContain("Captions");
    });

    it("renders Review tab with Proofing, Comments, and Tracking", () => {
      ribbon.setActiveTab("Review");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Proofing");
      expect(body.textContent).toContain("Comments");
      expect(body.textContent).toContain("Tracking");
    });
  });

  describe("3. Dedicated Calc Ribbon Schema", () => {
    beforeEach(() => {
      ribbon.setDocumentType("Excel");
    });

    it("has only Calc tabs: Home, Insert, Page Layout, Formulas, Data, Review, View", () => {
      expect(ribbon.getVisibleTabs()).toEqual([
        "Home",
        "Insert",
        "Page Layout",
        "Formulas",
        "Data",
        "Review",
        "View",
      ]);
    });

    it("renders Formulas tab with Function Library, Defined Names, and Formula Auditing", () => {
      ribbon.setActiveTab("Formulas");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Function Library");
      expect(body.textContent).toContain("Defined Names");
      expect(body.textContent).toContain("Formula Auditing");
      expect(body.textContent).toContain("Calculation");
    });

    it("renders Data tab with Get & Transform, Sort & Filter, and Data Tools", () => {
      ribbon.setActiveTab("Data");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Get & Transform Data");
      expect(body.textContent).toContain("Sort & Filter");
      expect(body.textContent).toContain("Data Tools");
      expect(body.textContent).toContain("Outline");
    });

    it("renders Page Layout tab with Sheet Options (Gridlines View/Print)", () => {
      ribbon.setActiveTab("Page Layout");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Sheet Options");
      expect(body.textContent).toContain("Page Setup");
    });
  });

  describe("4. Dedicated Slides Ribbon Schema (Strict Elimination of Word Styles)", () => {
    beforeEach(() => {
      ribbon.setDocumentType("PowerPoint");
    });

    it("has only Slides tabs: Home, Insert, Design, Transitions, Animations, Slide Show, Review, View", () => {
      expect(ribbon.getVisibleTabs()).toEqual([
        "Home",
        "Insert",
        "Design",
        "Transitions",
        "Animations",
        "Slide Show",
        "Review",
        "View",
      ]);
    });

    it("STRICT GEC REQUIREMENT: Slides Home tab does NOT contain Word Normal/Heading styles", () => {
      ribbon.setActiveTab("Home");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).not.toContain("Heading 1");
      expect(body.textContent).not.toContain("Heading 2");
      expect(body.textContent).not.toContain("Heading 3");

      // Must contain PowerPoint Drawing & Arrange tools
      expect(body.textContent).toContain("Drawing");
      expect(body.textContent).toContain("Arrange");
      expect(body.textContent).toContain("Slides");
    });

    it("renders Design tab with Themes, Variants, and Customize (4:3 and 16:9)", () => {
      ribbon.setActiveTab("Design");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Themes");
      expect(body.textContent).toContain("Variants");
      expect(body.textContent).toContain("Customize");
    });

    it("renders Transitions and Animations tabs with timing and animation pane controls", () => {
      ribbon.setActiveTab("Transitions");
      let body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Transition to This Slide");
      expect(body.textContent).toContain("Effect Options");
      expect(body.textContent).toContain("Timing");

      ribbon.setActiveTab("Animations");
      body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Animation");
      expect(body.textContent).toContain("Advanced Animation");
      expect(body.textContent).toContain("Timing");
    });

    it("renders Slide Show tab with From Beginning F5 and Presenter View", () => {
      ribbon.setActiveTab("Slide Show");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Start Slide Show");
      expect(body.textContent).toContain("Set Up");
      expect(body.textContent).toContain("Monitors");
    });
  });

  describe("5. Dedicated PDF Ribbon Schema", () => {
    beforeEach(() => {
      ribbon.setDocumentType("PDF");
    });

    it("has only PDF tabs: Home, Edit PDF, Forms, Comment & Annotate, Page Tools, Sign & Protect, View", () => {
      expect(ribbon.getVisibleTabs()).toEqual([
        "Home",
        "Edit PDF",
        "Forms",
        "Comment & Annotate",
        "Page Tools",
        "Sign & Protect",
        "View",
      ]);
    });

    it("renders Edit PDF tab with Text Editing, Image Editing, and Links", () => {
      ribbon.setActiveTab("Edit PDF");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Text Editing");
      expect(body.textContent).toContain("Image Editing");
      expect(body.textContent).toContain("Links");
    });

    it("renders Forms tab with Text Field, Checkbox, Dropdown, and Signatures", () => {
      ribbon.setActiveTab("Forms");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Form Fields");
      expect(body.textContent).toContain("Field Tools");
    });

    it("renders Comment & Annotate tab with Annotations, Pencil, Stamps, and Drawing", () => {
      ribbon.setActiveTab("Comment & Annotate");
      const body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Annotations");
      expect(body.textContent).toContain("Drawing");
    });

    it("renders Page Tools and Sign & Protect tabs", () => {
      ribbon.setActiveTab("Page Tools");
      let body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Page Management");

      ribbon.setActiveTab("Sign & Protect");
      body = document.getElementById("ribbon-tools-body")!;
      expect(body.textContent).toContain("Fill & Sign");
      expect(body.textContent).toContain("Security");
      expect(body.textContent).toContain("Redaction");
    });
  });

  describe("6. Action Emissions and Non-Placeholder Interactivity", () => {
    it("emits correct action when clicking ribbon action buttons", () => {
      const actions: string[] = [];
      ribbon.setOnAction((action) => {
        actions.push(action);
      });

      ribbon.setDocumentType("Word");
      ribbon.setActiveTab("Home");

      const pasteBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="writer-btn-paste"]');
      pasteBtn?.click();
      expect(actions).toContain("Paste");

      ribbon.setDocumentType("Excel");
      ribbon.setActiveTab("Home");
      const calcPasteBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="calc-btn-paste"]');
      calcPasteBtn?.click();
      expect(actions).toContain("Paste");

      ribbon.setDocumentType("PowerPoint");
      ribbon.setActiveTab("Home");
      const slidesPasteBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="slides-btn-paste"]');
      slidesPasteBtn?.click();
      expect(actions).toContain("Paste");

      ribbon.setDocumentType("PDF");
      ribbon.setActiveTab("Home");
      const handBtn = ribbonContainer.querySelector<HTMLButtonElement>('[data-testid="pdf-btn-hand"]');
      handBtn?.click();
      expect(actions).toContain("SetPdfTool");
    });
  });

  describe("7. Mobile Bottom Bar Cleanup on Desktop Viewport (>= 768px)", () => {
    it("hides bottom sheet handle and bar whenever window.innerWidth >= 768px", () => {
      // Set desktop window innerWidth
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        configurable: true,
        value: 1280,
      });

      const bottomSheet = new BottomSheet("bottom-sheet");
      bottomSheet.updateVisibility();

      expect(bottomSheet.isVisible()).toBe(false);
      expect(bottomSheet.getElement().style.display).toBe("none");
    });

    it("shows bottom sheet on mobile screens (< 768px)", () => {
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        configurable: true,
        value: 390,
      });

      const bottomSheet = new BottomSheet("bottom-sheet");
      bottomSheet.updateVisibility();

      expect(bottomSheet.isVisible()).toBe(true);
      expect(bottomSheet.getElement().style.display).not.toBe("none");
    });
  });
});
