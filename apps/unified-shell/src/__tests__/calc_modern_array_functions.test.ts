/**
 * calc_modern_array_functions.test.ts
 *
 * Test suite for Sub-Phase 47.1.1: Modern Dynamic Array & Lambda Functions (Frontend Coordinator & Autocomplete).
 * Strictly conforms to GEC-v7.0.
 */

import { describe, it, expect, vi } from "vitest";
import {
  CalcFormulaCoordinator,
  MODERN_FUNCTIONS_CATALOG,
} from "../state/CalcFormulaCoordinator";

describe("Sub-Phase 47.1.1: Modern Dynamic Array & Lambda Functions Coordinator", () => {
  it("maintains a complete catalog of modern dynamic array and lambda functions", () => {
    const names = MODERN_FUNCTIONS_CATALOG.map((f) => f.name);
    expect(names).toContain("SEQUENCE");
    expect(names).toContain("RANDARRAY");
    expect(names).toContain("CHOOSEROWS");
    expect(names).toContain("CHOOSECOLS");
    expect(names).toContain("TOCOL");
    expect(names).toContain("TOROW");
    expect(names).toContain("TAKE");
    expect(names).toContain("DROP");
    expect(names).toContain("EXPAND");
    expect(names).toContain("SORTBY");
    expect(names).toContain("FILTER");
    expect(names).toContain("UNIQUE");
    expect(names).toContain("SORT");
    expect(names).toContain("XLOOKUP");
    expect(names).toContain("XMATCH");
    expect(names).toContain("LET");
    expect(names).toContain("LAMBDA");
    expect(names).toContain("MAKEARRAY");
    expect(names).toContain("BYROW");
    expect(names).toContain("BYCOL");
    expect(names).toContain("MAP");
    expect(names).toContain("REDUCE");
    expect(names).toContain("SCAN");
    expect(names).toContain("ISOMITTED");
  });

  it("provides autocomplete suggestions based on formula bar prefix", () => {
    const coordinator = new CalcFormulaCoordinator();

    const seqSuggestions = coordinator.getSuggestions("=SEQ");
    expect(seqSuggestions.length).toBe(1);
    expect(seqSuggestions[0].name).toBe("SEQUENCE");

    const xSuggestions = coordinator.getSuggestions("X");
    const xNames = xSuggestions.map((s) => s.name);
    expect(xNames).toContain("XLOOKUP");
    expect(xNames).toContain("XMATCH");

    const empty = coordinator.getSuggestions("");
    expect(empty).toEqual([]);
  });

  it("tracks active parameter index for contextual signature tooltips", () => {
    const coordinator = new CalcFormulaCoordinator();

    // =SEQUENCE(4, | -> index 1 (columns)
    const formula1 = "=SEQUENCE(4, ";
    const hint1 = coordinator.getActiveParameterHint(formula1, formula1.length);
    expect(hint1).not.toBeNull();
    expect(hint1?.func.name).toBe("SEQUENCE");
    expect(hint1?.activeParamIndex).toBe(1);

    // =XLOOKUP("Target", A1:A10, B1:B10, "Default", | -> index 4 (match_mode)
    const formula2 = '=XLOOKUP("Target", A1:A10, B1:B10, "Default", ';
    const hint2 = coordinator.getActiveParameterHint(formula2, formula2.length);
    expect(hint2?.func.name).toBe("XLOOKUP");
    expect(hint2?.activeParamIndex).toBe(4);
  });

  it("computes dynamic array spill range correctly", () => {
    const coordinator = new CalcFormulaCoordinator();

    // 3 rows x 2 columns rooted at A1
    const zone = coordinator.calculateSpillRange("A1", 3, 2);
    expect(zone.rootCell).toBe("A1");
    expect(zone.endCell).toBe("B3");
    expect(zone.spilledCells).toEqual(["B1", "A2", "B2", "A3", "B3"]);
    expect(zone.hasCollision).toBe(false);
    expect(zone.collisionCells).toEqual([]);
  });

  it("detects #SPILL! collisions with occupied cells", () => {
    const coordinator = new CalcFormulaCoordinator();
    const occupied = new Set(["B2"]);

    const zone = coordinator.calculateSpillRange("A1", 3, 2, occupied);
    expect(zone.hasCollision).toBe(true);
    expect(zone.collisionCells).toEqual(["B2"]);
  });

  it("notifies subscribers when spill zones change or are cleared", () => {
    const coordinator = new CalcFormulaCoordinator();
    const listener = vi.fn();
    const unsubscribe = coordinator.subscribe(listener);

    coordinator.calculateSpillRange("C5", 2, 2);
    expect(listener).toHaveBeenCalledTimes(1);

    coordinator.clearSpillZone("C5");
    expect(listener).toHaveBeenCalledTimes(2);

    unsubscribe();
    coordinator.calculateSpillRange("D1", 1, 3);
    expect(listener).toHaveBeenCalledTimes(2);
  });

  it("evaluates SEQUENCE client-side in fallback mode", () => {
    const coordinator = new CalcFormulaCoordinator();
    const seq = coordinator.evalSequence(2, 3, 10, 5);
    expect(seq).toEqual([
      [10, 15, 20],
      [25, 30, 35],
    ]);
  });

  it("evaluates TOCOL with scan order and blank/error filtering", () => {
    const coordinator = new CalcFormulaCoordinator();
    const matrix = [
      [1, null, "#N/A"],
      [2, 3, 4],
    ];

    // Keep all, row-by-row
    const all = coordinator.evalToCol(matrix, 0, false);
    expect(all).toEqual([[1], [null], ["#N/A"], [2], [3], [4]]);

    // Ignore blanks and errors (ignore = 3), scan by column
    const filtered = coordinator.evalToCol(matrix, 3, true);
    expect(filtered).toEqual([[1], [2], [3], [4]]);
  });
});

