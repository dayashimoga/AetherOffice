/**
 * calc_validation_and_format.test.ts
 *
 * Test suite for Sub-Phase 47.1.2: Data Validation, Conditional Formatting & Sparklines.
 * Strictly conforms to GEC-v7.0.
 */

import { describe, it, expect, vi } from "vitest";
import { CalcValidationAndFormatCoordinator } from "../state/CalcValidationAndFormatCoordinator";

describe("Sub-Phase 47.1.2: CalcValidationAndFormatCoordinator", () => {
  // =========================================================================
  // 1. DATA VALIDATION
  // =========================================================================

  it("manages data validation rules and matches target cell ranges", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    const ruleId = coordinator.addValidationRule({
      type: "whole",
      operator: "between",
      formula1: "1",
      formula2: "100",
      sqref: "A1:B10 C15",
      allowBlank: true,
      showDropdown: false,
      errorStyle: "stop",
      errorTitle: "Invalid Number",
      errorMessage: "Number must be between 1 and 100",
    });

    // A1 and B10 should match
    expect(coordinator.getValidationForCell("A1")?.id).toBe(ruleId);
    expect(coordinator.getValidationForCell("B10")?.id).toBe(ruleId);
    expect(coordinator.getValidationForCell("C15")?.id).toBe(ruleId);

    // C1 or D1 should not match
    expect(coordinator.getValidationForCell("C1")).toBeNull();
    expect(coordinator.getValidationForCell("D1")).toBeNull();

    // Removing rule
    coordinator.removeValidationRule(ruleId);
    expect(coordinator.getValidationForCell("A1")).toBeNull();
  });

  it("extracts list dropdown options from literal strings and cell ranges", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    coordinator.addValidationRule({
      type: "list",
      operator: "equal",
      formula1: '"Red,Green,Blue"',
      sqref: "A1:A5",
      allowBlank: false,
      showDropdown: true,
      errorStyle: "warning",
    });

    const options = coordinator.getListOptions("A1");
    expect(options).toEqual(["Red", "Green", "Blue"]);

    // Range reference
    coordinator.addValidationRule({
      type: "list",
      operator: "equal",
      formula1: "=$C$1:$C$3",
      sqref: "B1:B5",
      allowBlank: false,
      showDropdown: true,
      errorStyle: "stop",
    });

    const mockGrid = { C1: "Alpha", C2: "Beta", C3: "Gamma" };
    const rangeOptions = coordinator.getListOptions("B2", mockGrid);
    expect(rangeOptions).toEqual(["Alpha", "Beta", "Gamma"]);
  });

  it("validates cell values for whole numbers, decimals, text length, and blanks", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    coordinator.addValidationRule({
      type: "whole",
      operator: "greaterThan",
      formula1: "10",
      sqref: "A1",
      allowBlank: false,
      showDropdown: false,
      errorStyle: "stop",
    });

    // Valid integer > 10
    expect(coordinator.validateCell("A1", 15).isValid).toBe(true);
    // Invalid integer <= 10
    expect(coordinator.validateCell("A1", 8).isValid).toBe(false);
    // Invalid non-integer
    expect(coordinator.validateCell("A1", 15.5).isValid).toBe(false);
    // Reject blank
    expect(coordinator.validateCell("A1", "").isValid).toBe(false);

    // Text length rule
    coordinator.addValidationRule({
      type: "textLength",
      operator: "lessThanOrEqual",
      formula1: "6",
      sqref: "B1",
      allowBlank: true,
      showDropdown: false,
      errorStyle: "information",
    });

    expect(coordinator.validateCell("B1", "Pass").isValid).toBe(true);
    expect(coordinator.validateCell("B1", "TooLongString").isValid).toBe(false);
    // Allow blank
    expect(coordinator.validateCell("B1", null).isValid).toBe(true);
  });

  // =========================================================================
  // 2. CONDITIONAL FORMATTING RULES MANAGER
  // =========================================================================

  it("manages conditional formatting rules with priority reordering and stopIfTrue", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    const r1 = coordinator.addConditionalRule({
      type: "cellIs",
      priority: 1,
      stopIfTrue: true,
      sqref: "A1:A10",
      cellIs: { operator: "greaterThan", formula1: "50" },
      style: { fillColor: "#00FF00", bold: true },
    });

    const r2 = coordinator.addConditionalRule({
      type: "cellIs",
      priority: 2,
      stopIfTrue: false,
      sqref: "A1:A10",
      cellIs: { operator: "greaterThan", formula1: "20" },
      style: { fillColor: "#FFFF00" },
    });

    // Reorder rule priority
    coordinator.reorderRule(r2, 0);
    expect(coordinator.conditionalRules[0].id).toBe(r2);
    expect(coordinator.conditionalRules[1].id).toBe(r1);

    // Toggle stopIfTrue
    coordinator.toggleStopIfTrue(r2);
    expect(coordinator.conditionalRules[0].stopIfTrue).toBe(true);
  });

  it("evaluates cellIs and containsText rules applying styles", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    coordinator.addConditionalRule({
      type: "cellIs",
      priority: 1,
      stopIfTrue: false,
      sqref: "A1:A10",
      cellIs: { operator: "between", formula1: "10", formula2: "20" },
      style: { fontColor: "#0000FF", bold: true },
    });

    coordinator.addConditionalRule({
      type: "containsText",
      priority: 2,
      stopIfTrue: false,
      sqref: "B1:B10",
      text: "URGENT",
      style: { fillColor: "#FF0000" },
    });

    const resA1 = coordinator.getAppliedFormatting("A1", 15);
    expect(resA1.style?.fontColor).toBe("#0000FF");
    expect(resA1.style?.bold).toBe(true);

    const resB1 = coordinator.getAppliedFormatting("B1", "Notice: URGENT action needed");
    expect(resB1.style?.fillColor).toBe("#FF0000");
  });

  it("computes color scale gradients and data bar fill ratios", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    coordinator.addConditionalRule({
      type: "colorScale",
      priority: 1,
      stopIfTrue: false,
      sqref: "C1:C10",
      colorScale: {
        colors: ["#FF0000", "#00FF00"],
        values: [0, 100],
      },
    });

    coordinator.addConditionalRule({
      type: "dataBar",
      priority: 2,
      stopIfTrue: false,
      sqref: "D1:D10",
      dataBar: {
        color: "#5B9BD5",
        showValue: true,
        minVal: 0,
        maxVal: 200,
      },
    });

    const resC1 = coordinator.getAppliedFormatting("C1", 50);
    // Midpoint between #FF0000 and #00FF00 -> #808000
    expect(resC1.colorScale).toBe("#808000");

    const resD1 = coordinator.getAppliedFormatting("D1", 100, [0, 50, 100, 200]);
    expect(resD1.dataBar?.ratio).toBe(0.5);
    expect(resD1.dataBar?.color).toBe("#5B9BD5");
  });

  // =========================================================================
  // 3. SPARKLINES
  // =========================================================================

  it("manages sparkline groups and renders SVG micro-charts", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();

    const lineId = coordinator.addSparklineGroup({
      type: "line",
      seriesColor: "#2E75B6",
      highColor: "#00B050",
      lowColor: "#C00000",
      showMarkers: true,
      showHigh: true,
      showLow: true,
      showNegative: false,
      dataRange: "A1:E1",
      targetRange: "F1",
    });

    const lineSvg = coordinator.renderSparklineSvg(lineId, [10, 40, 20, 80, 30], 120, 24);
    expect(lineSvg).toContain("<polyline");
    expect(lineSvg).toContain('stroke="#2E75B6"');
    // High point circle in green
    expect(lineSvg).toContain('fill="#00B050"');
    // Low point circle in red
    expect(lineSvg).toContain('fill="#C00000"');

    // Column sparkline
    const colId = coordinator.addSparklineGroup({
      type: "column",
      seriesColor: "#4472C4",
      negativeColor: "#ED7D31",
      showMarkers: false,
      showHigh: false,
      showLow: false,
      showNegative: true,
      dataRange: "A2:E2",
      targetRange: "F2",
    });

    const colSvg = coordinator.renderSparklineSvg(colId, [10, -5, 20], 100, 24);
    expect(colSvg).toContain("<rect");
    expect(colSvg).toContain('fill="#ED7D31"');

    // Remove group
    coordinator.removeSparklineGroup(colId);
    expect(coordinator.sparklineGroups.length).toBe(1);
  });

  it("dispatches notifications to reactive subscribers on state changes", () => {
    const coordinator = new CalcValidationAndFormatCoordinator();
    const listener = vi.fn();
    const unsubscribe = coordinator.subscribe(listener);

    const valId = coordinator.addValidationRule({
      type: "whole",
      operator: "equal",
      formula1: "42",
      sqref: "Z1",
      allowBlank: true,
      showDropdown: false,
      errorStyle: "stop",
    });
    expect(listener).toHaveBeenCalledTimes(1);

    coordinator.removeValidationRule(valId);
    expect(listener).toHaveBeenCalledTimes(2);

    unsubscribe();
    coordinator.addValidationRule({
      type: "whole",
      operator: "equal",
      formula1: "1",
      sqref: "Z2",
      allowBlank: true,
      showDropdown: false,
      errorStyle: "stop",
    });
    expect(listener).toHaveBeenCalledTimes(2);
  });
});
