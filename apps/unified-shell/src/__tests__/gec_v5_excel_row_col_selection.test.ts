import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { CalcState } from "../state/calcState";
import { FormulaBar } from "../components/FormulaBar";
import { UnifiedShellApp } from "../main";

describe("GEC-v5.0 Article III.2: Excel Multi-Row & Multi-Column Header Selection + Batch Row/Col Operations", () => {
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

  describe("1. Column Selection & Drag Selection (A..Z)", () => {
    it("selects entire column B on header click and reports B:B descriptor", () => {
      let reportedDesc = "";
      canvas.setOnRangeStatsChanged((_stats, desc) => {
        reportedDesc = desc;
      });

      // Click column B header (headerHeight = 26, col A is 50..146, col B is 146..242, center = 194)
      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(
        new MouseEvent("mousedown", {
          clientX: 194,
          clientY: 15,
          button: 0,
          bubbles: true,
        })
      );

      expect(state.selectedCell).toBe("B1");
      expect(state.getSelectionDescriptor()).toBe("B:B");
      expect(reportedDesc).toBe("B:B");

      const primary = state.selectedRanges[0];
      expect(primary.minCol).toBe(1);
      expect(primary.maxCol).toBe(1);
      expect(primary.minRow).toBe(0);
      expect(primary.maxRow).toBe(999);
    });

    it("selects columns B through D on drag and reports B:D descriptor", () => {
      let reportedDesc = "";
      canvas.setOnRangeStatsChanged((_stats, desc) => {
        reportedDesc = desc;
      });

      const canvasEl = container.querySelector("canvas")!;
      // Mouse down on Column B (x = 194, y = 15)
      canvasEl.dispatchEvent(
        new MouseEvent("mousedown", {
          clientX: 194,
          clientY: 15,
          button: 0,
          bubbles: true,
        })
      );

      // Mouse move to Column D (center = 386)
      canvasEl.dispatchEvent(
        new MouseEvent("mousemove", {
          clientX: 386,
          clientY: 15,
          buttons: 1,
          bubbles: true,
        })
      );

      window.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));

      expect(state.getSelectionDescriptor()).toBe("B:D");
      expect(reportedDesc).toBe("B:D");
      const primary = state.selectedRanges[0];
      expect(primary.minCol).toBe(1);
      expect(primary.maxCol).toBe(3);
      expect(primary.minRow).toBe(0);
      expect(primary.maxRow).toBe(999);
    });

    it("applies Bold and Currency format across all cells in columns B..D (including B1..D20)", () => {
      canvas.selectColumn(1); // Col B
      // Extend to Col D
      state.selectedRanges[0].maxCol = 3;

      // Apply Bold & Currency
      state.toggleBold();
      state.setNumberFormat("Currency");

      // Verify B1..D20 all receive bold and Currency format
      for (let r = 1; r <= 20; r++) {
        for (const col of ["B", "C", "D"]) {
          const coord = `${col}${r}`;
          const cell = state.getCell(coord);
          expect(cell.bold).toBe(true);
          expect(cell.format).toBe("Currency");
        }
      }

      // Assert Column A cells are untouched
      expect(state.getCell("A1").bold).toBeUndefined();
      expect(state.getCell("A10").format).toBeUndefined();
    });
  });

  describe("2. Row Selection & Drag Selection (1..N)", () => {
    it("selects entire row 3 on header click and reports 3:3 descriptor", () => {
      let reportedDesc = "";
      canvas.setOnRangeStatsChanged((_stats, desc) => {
        reportedDesc = desc;
      });

      // Row 3 header: headerWidth = 50, row 1 is 26..50, row 2 is 50..74, row 3 is 74..98, center = 86
      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(
        new MouseEvent("mousedown", {
          clientX: 25,
          clientY: 86,
          button: 0,
          bubbles: true,
        })
      );

      expect(state.selectedCell).toBe("A3");
      expect(state.getSelectionDescriptor()).toBe("3:3");
      expect(reportedDesc).toBe("3:3");

      const primary = state.selectedRanges[0];
      expect(primary.minRow).toBe(2);
      expect(primary.maxRow).toBe(2);
      expect(primary.minCol).toBe(0);
      expect(primary.maxCol).toBe(99);
    });

    it("selects rows 2 through 5 on drag and reports 2:5 descriptor", () => {
      let reportedDesc = "";
      canvas.setOnRangeStatsChanged((_stats, desc) => {
        reportedDesc = desc;
      });

      const canvasEl = container.querySelector("canvas")!;
      // Mouse down on Row 2 (y = 62)
      canvasEl.dispatchEvent(
        new MouseEvent("mousedown", {
          clientX: 25,
          clientY: 62,
          button: 0,
          bubbles: true,
        })
      );

      // Mouse move to Row 5 (y = 134)
      canvasEl.dispatchEvent(
        new MouseEvent("mousemove", {
          clientX: 25,
          clientY: 134,
          buttons: 1,
          bubbles: true,
        })
      );

      window.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));

      expect(state.getSelectionDescriptor()).toBe("2:5");
      expect(reportedDesc).toBe("2:5");
      const primary = state.selectedRanges[0];
      expect(primary.minRow).toBe(1);
      expect(primary.maxRow).toBe(4);
      expect(primary.minCol).toBe(0);
      expect(primary.maxCol).toBe(99);
    });

    it("applies Italic, Align Center and Fill Color across all cells in rows 2..5", () => {
      canvas.selectRow(1); // Row 2
      // Extend to Row 5
      state.selectedRanges[0].maxRow = 4;

      state.toggleItalic();
      state.setAlignment("center");
      state.setFillColor("#dcfce7");

      // Verify cells across columns A..J in rows 2..5
      for (let r = 2; r <= 5; r++) {
        for (const col of ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]) {
          const coord = `${col}${r}`;
          const cell = state.getCell(coord);
          expect(cell.italic).toBe(true);
          expect(cell.align).toBe("center");
          expect(cell.bgColor).toBe("#dcfce7");
        }
      }

      // Assert Row 1 and Row 6 are untouched
      expect(state.getCell("A1").italic).toBeUndefined();
      expect(state.getCell("B6").bgColor).toBeUndefined();
    });
  });

  describe("3. Select All Sheet Corner (◢)", () => {
    it("selects entire sheet on top-left corner click", () => {
      let reportedDesc = "";
      canvas.setOnRangeStatsChanged((_stats, desc) => {
        reportedDesc = desc;
      });

      // Top-left corner cell: x <= 50, y <= 26
      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(
        new MouseEvent("mousedown", {
          clientX: 20,
          clientY: 15,
          button: 0,
          bubbles: true,
        })
      );

      expect(state.selectedCell).toBe("A1");
      expect(state.getSelectionDescriptor()).toBe("1:1000");
      expect(reportedDesc).toBe("1:1000");

      const primary = state.selectedRanges[0];
      expect(primary.minRow).toBe(0);
      expect(primary.maxRow).toBe(999);
      expect(primary.minCol).toBe(0);
      expect(primary.maxCol).toBe(99);
    });

    it("clears all contents when Delete key is pressed on full sheet or row/col selection", () => {
      state.setCellValue("B2", "100");
      state.setCellValue("C3", "200");
      state.setCellValue("D4", "300");

      canvas.selectAll();

      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "Delete",
          bubbles: true,
        })
      );

      expect(state.getCellValue("B2")).toBe("");
      expect(state.getCellValue("C3")).toBe("");
      expect(state.getCellValue("D4")).toBe("");
    });
  });

  describe("4. Name Box Integration & Name Box Jump", () => {
    it("selects column range when typing B:D in Name Box and pressing Enter", () => {
      const formulaBar = new FormulaBar("sub-header-container");
      formulaBar.setCallbacks({
        onNameBoxJump: (coord) => {
          canvas.selectRangeByName(coord);
        },
      });

      const nameBox = formulaBar.getNameBox();
      nameBox.value = "B:D";
      nameBox.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

      expect(state.getSelectionDescriptor()).toBe("B:D");
      expect(state.selectedRanges[0].minCol).toBe(1);
      expect(state.selectedRanges[0].maxCol).toBe(3);
    });

    it("selects row range when typing 2:5 in Name Box and pressing Enter", () => {
      const formulaBar = new FormulaBar("sub-header-container");
      formulaBar.setCallbacks({
        onNameBoxJump: (coord) => {
          canvas.selectRangeByName(coord);
        },
      });

      const nameBox = formulaBar.getNameBox();
      nameBox.value = "2:5";
      nameBox.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

      expect(state.getSelectionDescriptor()).toBe("2:5");
      expect(state.selectedRanges[0].minRow).toBe(1);
      expect(state.selectedRanges[0].maxRow).toBe(4);
    });

    it("selects rectangular range when typing B2:D8 in Name Box and pressing Enter", () => {
      const formulaBar = new FormulaBar("sub-header-container");
      formulaBar.setCallbacks({
        onNameBoxJump: (coord) => {
          canvas.selectRangeByName(coord);
        },
      });

      const nameBox = formulaBar.getNameBox();
      nameBox.value = "B2:D8";
      nameBox.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

      expect(state.getSelectionDescriptor()).toBe("B2:D8");
      expect(state.selectedRanges[0].minCol).toBe(1);
      expect(state.selectedRanges[0].maxCol).toBe(3);
      expect(state.selectedRanges[0].minRow).toBe(1);
      expect(state.selectedRanges[0].maxRow).toBe(7);
    });
  });

  describe("5. End-to-End Ribbon Formatting on Selected Columns & Rows in UnifiedShellApp", () => {
    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app" class="desktop-shell">
          <header class="title-bar">
            <div class="app-branding">
              <span class="app-name">AetherOffice</span>
            </div>
            <div class="title-actions">
              <button id="btn-theme-toggle" class="theme-toggle-btn"></button>
            </div>
          </header>
          <div id="left-app-rail-container"></div>
          <div id="tab-bar-container"></div>
          <div id="ribbon-container"></div>
          <div id="sub-header-container" style="display: none;"></div>
          <div class="workbench">
            <div id="sidebar-container"></div>
            <main id="viewport-container" class="viewport"></main>
          </div>
          <footer id="status-bar-container">
            <span id="status-doc-info"></span>
            <input type="range" id="zoom-slider" min="50" max="200" value="100" />
            <span id="status-zoom-level">100%</span>
          </footer>
        </div>
      `;
    });

    it("formats selected columns B..D via Ribbon ToggleBold and SetNumberFormat", () => {
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });
      const gridCanvas = app.getCalcGridCanvas()!;
      const stateRef = gridCanvas.getState();

      // Select Columns B:D
      gridCanvas.selectColumn(1);
      stateRef.selectedRanges[0].maxCol = 3;
      expect(stateRef.getSelectionDescriptor()).toBe("B:D");

      // Dispatch Ribbon ToggleBold and SetNumberFormat
      (app as any).handleRibbonAction("ToggleBold");
      (app as any).handleRibbonAction("SetNumberFormat", "Currency");

      // Assert cells B1..D20 receive Bold and Currency format
      for (let r = 1; r <= 20; r++) {
        for (const col of ["B", "C", "D"]) {
          const coord = `${col}${r}`;
          const cell = stateRef.getCell(coord);
          expect(cell.bold).toBe(true);
          expect(cell.format).toBe("Currency");
        }
      }
    });

    it("formats selected rows 2..5 via Ribbon ToggleItalic and SetFillColor", () => {
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });
      const gridCanvas = app.getCalcGridCanvas()!;
      const stateRef = gridCanvas.getState();

      // Select Rows 2:5
      gridCanvas.selectRow(1);
      stateRef.selectedRanges[0].maxRow = 4;
      expect(stateRef.getSelectionDescriptor()).toBe("2:5");

      (app as any).handleRibbonAction("ToggleItalic");
      (app as any).handleRibbonAction("SetFillColor", "#fef08a");

      // Assert cells in rows 2..5 receive Italic and Fill color
      for (let r = 2; r <= 5; r++) {
        for (const col of ["A", "B", "C", "D", "E"]) {
          const coord = `${col}${r}`;
          const cell = stateRef.getCell(coord);
          expect(cell.italic).toBe(true);
          expect(cell.bgColor).toBe("#fef08a");
        }
      }
    });
  });
});
