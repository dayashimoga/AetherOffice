import { describe, it, expect, beforeEach } from "vitest";
import { PivotFieldListSidebar } from "../components/PivotFieldListSidebar";
import { UnifiedShellApp } from "../main";
import { calcState } from "../state/calcState";

describe("Interactive PivotTable & PivotChart Studio UI (GEC-v2.0 Subphase 18.1.3)", () => {
  beforeEach(() => {
    const mockCtx = {
      clearRect: () => {},
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      strokeText: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      save: () => {},
      restore: () => {},
      clip: () => {},
      scale: () => {},
      setTransform: () => {},
      measureText: (txt: string) => ({ width: txt.length * 7 }),
      setLineDash: () => {},
      font: "12px sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <nav id="tab-bar-container" class="tab-bar"></nav>
        <section id="ribbon-container" class="ribbon-bar"></section>
        <section id="sub-header-container" class="sub-header-bar"></section>
        <main class="workspace-layout">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <section class="canvas-viewport" id="viewport-container">
            <canvas id="render-surface"></canvas>
          </section>
        </main>
        <div id="bottom-sheet" class="bottom-sheet"></div>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info">Page 1 of 1</span>
            <span id="status-word-count">0 words</span>
          </div>
        </footer>
      </div>
    `;
  });

  describe("1. PivotFieldListSidebar Component Lifecycle & Controls", () => {
    it("initializes hidden and toggles visibility", () => {
      const sidebar = new PivotFieldListSidebar();
      expect(sidebar.isVisible()).toBe(false);

      sidebar.show();
      expect(sidebar.isVisible()).toBe(true);

      sidebar.hide();
      expect(sidebar.isVisible()).toBe(false);

      sidebar.toggle();
      expect(sidebar.isVisible()).toBe(true);
    });

    it("renders available fields pool and filters by search text", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.setAvailableFields(["Region", "Category", "Quarter", "Revenue", "Profit", "Units"]);

      sidebar.show();
      const fieldItems = document.querySelectorAll(".pivot-field-item");
      expect(fieldItems.length).toBe(6);

      const searchInput = document.querySelector(".pivot-field-search") as HTMLInputElement;
      expect(searchInput).not.toBeNull();

      // Filter by "Rev"
      searchInput.value = "Rev";
      searchInput.dispatchEvent(new Event("input"));

      const filteredItems = document.querySelectorAll(".pivot-field-item");
      expect(filteredItems.length).toBe(1);
      expect(filteredItems[0].textContent).toContain("Revenue");
    });
  });

  describe("2. Drag-and-Drop Quadrants & Field Allocation", () => {
    it("allocates fields to Rows, Columns, Values, and Filters quadrants", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      sidebar.addFieldToQuadrant("Region", "rows");
      sidebar.addFieldToQuadrant("Category", "columns");
      sidebar.addFieldToQuadrant("Sales", "values", "SUM");
      sidebar.addFieldToQuadrant("Quarter", "filters");

      const def = sidebar.getDefinition();
      expect(def.rowFields).toEqual(["Region"]);
      expect(def.colFields).toEqual(["Category"]);
      expect(def.valueFields[0].name).toBe("Sales");
      expect(def.valueFields[0].aggregation).toBe("SUM");
      expect(def.filterFields).toEqual(["Quarter"]);

      // Verify DOM quadrant pills
      const rowDrop = document.querySelector(".pivot-drop-rows");
      expect(rowDrop?.textContent).toContain("Region");

      const colDrop = document.querySelector(".pivot-drop-columns");
      expect(colDrop?.textContent).toContain("Category");

      const valDrop = document.querySelector(".pivot-drop-values");
      expect(valDrop?.textContent).toContain("SUM of Sales");

      const filterDrop = document.querySelector(".pivot-drop-filters");
      expect(filterDrop?.textContent).toContain("Quarter");
    });

    it("modifies aggregation function and updates measure pill label", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      sidebar.addFieldToQuadrant("Sales", "values", "SUM");
      sidebar.setAggregation("Sales", "AVERAGE");

      let def = sidebar.getDefinition();
      expect(def.valueFields[0].aggregation).toBe("AVERAGE");
      expect(def.valueFields[0].label).toBe("AVERAGE of Sales");

      sidebar.setAggregation("Sales", "PERCENT_OF_TOTAL");
      def = sidebar.getDefinition();
      expect(def.valueFields[0].aggregation).toBe("PERCENT_OF_TOTAL");
      expect(def.valueFields[0].label).toBe("PERCENT_OF_TOTAL of Sales");
    });

    it("removes fields from quadrant using remove pill button", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      sidebar.addFieldToQuadrant("Quarter", "filters");
      expect(sidebar.getDefinition().filterFields).toContain("Quarter");

      sidebar.removeFieldFromQuadrant("Quarter", "filters");
      expect(sidebar.getDefinition().filterFields).not.toContain("Quarter");
    });
  });

  describe("3. Dynamic Pivot Layout Calculation & Grid Ingestion", () => {
    it("renders PivotTable directly into Calc sheet grid with headers, row labels, and totals", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      sidebar.addFieldToQuadrant("Region", "rows");
      sidebar.addFieldToQuadrant("Category", "columns");
      sidebar.addFieldToQuadrant("Sales", "values", "SUM");

      sidebar.renderPivotTableToGrid("G1");

      // Verify Header at G1: "Region \ Category"
      expect(calcState.getCellValue("G1")).toBe("Region \\ Category");

      // Verify Column headers at H1, I1, J1
      expect(calcState.getCellValue("H1")).toContain("Electronics");
      expect(calcState.getCellValue("I1")).toContain("Furniture");
      expect(calcState.getCellValue("J1")).toBe("Total Sales");

      // Verify Row 1: G2 is "East", H2 is 1200, I2 is 800, J2 is 2000
      expect(calcState.getCellValue("G2")).toBe("East");
      expect(Number(calcState.getCellValue("H2"))).toBe(1200);
      expect(Number(calcState.getCellValue("I2"))).toBe(800);
      expect(Number(calcState.getCellValue("J2"))).toBe(2000);

      // Verify Grand Total Row at G6
      expect(calcState.getCellValue("G6")).toBe("Grand Total");
      expect(Number(calcState.getCellValue("J6"))).toBe(5850); // 1200+800 + 1000+500 + 750+250 + 950+400 = 5850
    });
  });

  describe("4. Interactive Slicers Point-and-Click Filtering", () => {
    it("adds slicer card with unique value buttons and filters data dynamically", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      sidebar.addSlicer("Region");
      expect(sidebar.getDefinition().slicers.length).toBe(1);

      const slicerCard = document.querySelector(".pivot-slicer-card");
      expect(slicerCard).not.toBeNull();
      expect(slicerCard?.textContent).toContain("Region");

      const northBtn = document.querySelector(".slicer-btn-North") as HTMLButtonElement;
      expect(northBtn).not.toBeNull();

      // Click "North" button to filter to only North
      northBtn.click();
      expect(sidebar.getDefinition().slicers[0].selectedValues.has("North")).toBe(true);

      // Verify PivotTable grid recomputed: only North rows present
      expect(calcState.getCellValue("G2")).toBe("North");
      expect(Number(calcState.getCellValue("H2"))).toBe(1000); // North Electronics
      expect(Number(calcState.getCellValue("I2"))).toBe(500);  // North Furniture
      expect(Number(calcState.getCellValue("J2"))).toBe(1500); // North Total

      // Clear filter using broom button
      const clearBtn = document.querySelector(".slicer-clear-btn") as HTMLElement;
      clearBtn.click();
      expect(sidebar.getDefinition().slicers[0].selectedValues.size).toBe(0);

      // Full dataset restored
      expect(Number(calcState.getCellValue("J6"))).toBe(5850);
    });
  });

  describe("5. Linked PivotChart Synchronization", () => {
    it("creates linked PivotChart and synchronizes range and title upon field changes", () => {
      const sidebar = new PivotFieldListSidebar();
      sidebar.show();

      const chart = sidebar.createLinkedPivotChart("column");
      expect(chart).not.toBeNull();
      expect(chart.type).toBe("column");
      expect(calcState.getCharts().length).toBe(1);

      // Modify aggregation -> chart title and summary sync
      sidebar.setAggregation("Sales", "AVERAGE");
      expect(chart.title).toContain("AVERAGE of Sales");
    });
  });

  describe("6. UnifiedShellApp Ribbon Action Wiring", () => {
    it("routes InsertPivotTable, InsertPivotChart, and InsertSlicer ribbon actions", () => {
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });

      const sidebar = app.getPivotFieldListSidebar();
      expect(sidebar.isVisible()).toBe(false);

      // 1. Dispatch InsertPivotTable
      app.handleRibbonAction("InsertPivotTable");
      expect(sidebar.isVisible()).toBe(true);

      // 2. Dispatch InsertPivotChart
      app.handleRibbonAction("InsertPivotChart");
      expect(sidebar.isVisible()).toBe(true);
      expect(calcState.getCharts().length).toBeGreaterThan(0);

      // 3. Dispatch InsertSlicer
      app.handleRibbonAction("InsertSlicer");
      expect(sidebar.getDefinition().slicers.some((s) => s.fieldName === "Region")).toBe(true);
    });
  });
});
