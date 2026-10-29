import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { CalcState, shiftFormula } from "../state/calcState";
import { UnifiedShellApp } from "../main";

describe("Range Selection, Fill Handle Auto-Series & Clipboard Cut/Copy/Paste (GEC-v2.0 Subphase 10.1.2)", () => {
  let container: HTMLElement;
  let state: CalcState;
  let canvas: CalcGridCanvas;

  beforeEach(() => {
    const mockCtx = {
      clearRect: () => {},
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      measureText: (text: string) => ({ width: text.length * 7 }),
      beginPath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      arc: () => {},
      closePath: () => {},
      fill: () => {},
      stroke: () => {},
      save: () => {},
      restore: () => {},
      setTransform: () => {},
      scale: () => {},
      setLineDash: () => {},
      rect: () => {},
      clip: () => {},
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="sub-header-container"></div>
      <div id="sidebar-container"></div>
      <div id="viewport-container" style="width: 800px; height: 600px; position: relative;"></div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new CalcState();
    canvas = new CalcGridCanvas(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Rectangular Range Selection & Live Status Bar Statistics", () => {
    it("selects rectangular cell range and computes count, sum, and average", () => {
      // Setup numbers in A1:B3
      state.cells.set("A1", { raw: "10", computed: 10 });
      state.cells.set("A2", { raw: "20", computed: 20 });
      state.cells.set("A3", { raw: "30", computed: 30 });
      state.cells.set("B1", { raw: "40", computed: 40 });
      state.cells.set("B2", { raw: "50", computed: 50 });
      state.cells.set("B3", { raw: "60", computed: 60 });

      // Select A1:B3
      canvas.selectCell("A1", false, false);
      canvas.selectCell("B3", true, false);

      const stats = state.getSelectedRangeStats();
      expect(stats.count).toBe(6);
      expect(stats.numbersCount).toBe(6);
      expect(stats.sum).toBe(210);
      expect(stats.average).toBe(35);
      expect(stats.min).toBe(10);
      expect(stats.max).toBe(60);
    });

    it("triggers onRangeStatsChanged and updates status bar on multi-cell selection", () => {
      let emittedRange = "";
      let emittedStats: any = null;

      canvas.setOnRangeStatsChanged((stats, rangeText) => {
        emittedStats = stats;
        emittedRange = rangeText;
      });

      state.cells.set("A1", { raw: "100", computed: 100 });
      state.cells.set("A2", { raw: "200", computed: 200 });

      canvas.selectCell("A1", false, false);
      canvas.selectCell("A2", true, false);

      expect(emittedRange).toBe("A1:A2");
      expect(emittedStats?.sum).toBe(300);
      expect(emittedStats?.average).toBe(150);
    });

    it("expands range selection using Shift + Arrow keys", () => {
      canvas.selectCell("A1");
      const canvasEl = container.querySelector("canvas")!;

      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", shiftKey: true }));
      let range = state.selectedRanges[0];
      expect(range.minRow).toBe(0);
      expect(range.maxRow).toBe(1);

      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", shiftKey: true }));
      range = state.selectedRanges[0];
      expect(range.minCol).toBe(0);
      expect(range.maxCol).toBe(1);
    });
  });

  describe("2. Fill Handle Auto-Series & Dynamic Formula Shifting", () => {
    it("shifts relative formula coordinates correctly down and right", () => {
      expect(shiftFormula("=A1+B1", 1, 0)).toBe("=A2+B2");
      expect(shiftFormula("=A1+B1", 0, 1)).toBe("=B1+C1");
      expect(shiftFormula("=$A$1+B1", 3, 0)).toBe("=$A$1+B4");
      expect(shiftFormula("=A$1+$B1", 2, 2)).toBe("=C$1+$B3");
      expect(shiftFormula("=SUM(A1:A5)", 2, 0)).toBe("=SUM(A3:A7)");
    });

    it("auto-fills linear numerical series", () => {
      const series1 = state.predictSeries(["1", "2"], 3, true);
      expect(series1).toEqual(["3", "4", "5"]);

      const series2 = state.predictSeries(["100", "200"], 2, true);
      expect(series2).toEqual(["300", "400"]);
    });

    it("auto-fills days of week (short and full names)", () => {
      const shortDays = state.predictSeries(["Mon", "Tue"], 3, true);
      expect(shortDays).toEqual(["Wed", "Thu", "Fri"]);

      const fullDays = state.predictSeries(["Friday"], 2, true);
      expect(fullDays).toEqual(["Saturday", "Sunday"]);
    });

    it("auto-fills months of year (short and full names)", () => {
      const shortMonths = state.predictSeries(["Jan", "Feb"], 3, true);
      expect(shortMonths).toEqual(["Mar", "Apr", "May"]);

      const fullMonths = state.predictSeries(["November"], 2, true);
      expect(fullMonths).toEqual(["December", "January"]);
    });

    it("applies auto-fill on formula cells shifting references and recalculating", () => {
      state.cells.set("A1", { raw: "10", computed: 10 });
      state.cells.set("B1", { raw: "20", computed: 20 });
      state.cells.set("C1", { raw: "=A1+B1", computed: 30 });

      state.cells.set("A2", { raw: "50", computed: 50 });
      state.cells.set("B2", { raw: "70", computed: 70 });

      // Drag fill C1 down to C2
      state.applyAutoFill(
        { minRow: 0, maxRow: 0, minCol: 2, maxCol: 2 }, // C1
        { minRow: 0, maxRow: 1, minCol: 2, maxCol: 2 }  // C1:C2
      );

      expect(state.cells.get("C2")?.raw).toBe("=A2+B2");
      expect(state.cells.get("C2")?.computed).toBe(120);
    });
  });

  describe("3. Multi-Cell Clipboard Cut / Copy / Paste (TSV Interoperability)", () => {
    it("copies range to TSV format and sets clipboard marching ants state", () => {
      state.cells.set("A1", { raw: "100", computed: 100 });
      state.cells.set("B1", { raw: "200", computed: 200 });
      state.cells.set("A2", { raw: "300", computed: 300 });
      state.cells.set("B2", { raw: "400", computed: 400 });

      state.selectedRanges = [{ minRow: 0, maxRow: 1, minCol: 0, maxCol: 1 }];
      const tsv = canvas.copy();

      expect(tsv).toBe("100\t200\n300\t400");
      expect(state.clipboardData).not.toBeNull();
      expect(state.clipboardData?.isCut).toBe(false);
      expect(state.clipboardData?.range).toEqual({ minRow: 0, maxRow: 1, minCol: 0, maxCol: 1 });
    });

    it("pastes TSV data starting from active target cell", () => {
      state.selectedCell = "C1";
      canvas.paste("Hello\tWorld\nFoo\tBar");

      expect(state.cells.get("C1")?.raw).toBe("Hello");
      expect(state.cells.get("D1")?.raw).toBe("World");
      expect(state.cells.get("C2")?.raw).toBe("Foo");
      expect(state.cells.get("D2")?.raw).toBe("Bar");
    });

    it("executes Cut operation, removing source cells upon paste", () => {
      state.cells.set("A10", { raw: "MoveMe", computed: "MoveMe" });
      state.selectedRanges = [{ minRow: 9, maxRow: 9, minCol: 0, maxCol: 0 }];

      canvas.cut();
      expect(state.clipboardData?.isCut).toBe(true);

      state.selectedCell = "B10";
      canvas.paste();

      // Pasted to B10
      expect(state.cells.get("B10")?.raw).toBe("MoveMe");
      // Cleared from A10
      expect(state.cells.has("A10")).toBe(false);
      // Clipboard cleared after cut paste
      expect(state.clipboardData).toBeNull();
    });

    it("clears clipboard marching ants on Escape", () => {
      state.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }];
      canvas.copy();
      expect(state.clipboardData).not.toBeNull();

      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

      expect(state.clipboardData).toBeNull();
    });
  });

  describe("4. UnifiedShellApp Ribbon & Keyboard Integration for Calc Clipboard", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app">
          <div id="titlebar">
            <span id="titlebar-filename"></span>
            <span id="titlebar-dirty-indicator"></span>
          </div>
          <div id="tab-bar-container"></div>
          <div id="ribbon-container"></div>
          <div id="sub-header-container"></div>
          <div id="sidebar-container"></div>
          <div id="viewport-container" style="width: 800px; height: 600px;"></div>
          <div id="bottom-sheet"></div>
          <canvas id="stylus-ink-canvas"></canvas>
          <div id="reflow-reader-container"></div>
          <div id="row-card-inspector-container"></div>
          <div id="laser-pointer-dot"></div>
          <div id="status-bar">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </div>
        </div>
      `;

      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });
    });

    it("dispatches Copy, Cut, and Paste through Ribbon actions", () => {
      const calcCv = app.getCalcGridCanvas()!;
      expect(calcCv).toBeDefined();

      // Trigger Copy
      (app as any).handleRibbonAction("Copy");
      expect(calcCv.getState().clipboardData).not.toBeNull();

      // Trigger Cut
      (app as any).handleRibbonAction("Cut");
      expect(calcCv.getState().clipboardData?.isCut).toBe(true);

      // Trigger Paste
      (app as any).handleRibbonAction("Paste");
    });

    it("dispatches Copy, Cut, and Paste through global keyboard shortcuts", () => {
      const calcCv = app.getCalcGridCanvas()!;
      expect(calcCv).toBeDefined();

      (app as any).handleShortcutAction("Copy");
      expect(calcCv.getState().clipboardData).not.toBeNull();

      (app as any).handleShortcutAction("Cut");
      expect(calcCv.getState().clipboardData?.isCut).toBe(true);

      (app as any).handleShortcutAction("Paste");
    });
  });
});

