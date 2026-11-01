/**
 * CalcValidationAndFormatCoordinator.ts
 *
 * Implements Sub-Phase 47.1.2: Data Validation, Conditional Formatting Rules Manager & Sparklines presentation coordinator.
 * Strictly conforms to GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - Reactive state management for cell data validation rules, dropdown extraction, and error alerts.
 * - Full Conditional Formatting Rules Manager (add, edit, priority reorder, stopIfTrue, color scales, data bars, icon sets).
 * - Micro-chart Sparklines SVG generation (Line, Column, Win/Loss).
 */

export interface CalcDataValidationRule {
  id: string;
  type: "whole" | "decimal" | "list" | "date" | "time" | "textLength" | "custom";
  operator: "between" | "notBetween" | "equal" | "notEqual" | "greaterThan" | "lessThan" | "greaterThanOrEqual" | "lessThanOrEqual";
  formula1: string;
  formula2?: string;
  sqref: string;
  allowBlank: boolean;
  showDropdown: boolean;
  errorStyle: "stop" | "warning" | "information";
  errorTitle?: string;
  errorMessage?: string;
}

export interface ConditionalRule {
  id: string;
  type: "cellIs" | "colorScale" | "dataBar" | "iconSet" | "top10" | "duplicateValues" | "uniqueValues" | "containsText" | "expression";
  priority: number;
  stopIfTrue: boolean;
  sqref: string;
  style?: {
    fontColor?: string;
    bold?: boolean;
    italic?: boolean;
    fillColor?: string;
    borderColor?: string;
  };
  cellIs?: {
    operator: string;
    formula1: string;
    formula2?: string;
  };
  colorScale?: {
    colors: string[];
    values: number[];
  };
  dataBar?: {
    color: string;
    showValue: boolean;
    minVal?: number;
    maxVal?: number;
  };
  iconSet?: {
    name: string;
    thresholds: number[];
  };
  top10?: {
    rank: number;
    percent: boolean;
    bottom: boolean;
  };
  text?: string;
}

export interface SparklineGroupConfig {
  id: string;
  type: "line" | "column" | "stacked";
  seriesColor: string;
  highColor?: string;
  lowColor?: string;
  negativeColor?: string;
  showMarkers: boolean;
  showHigh: boolean;
  showLow: boolean;
  showNegative: boolean;
  dataRange: string;
  targetRange: string;
}

export class CalcValidationAndFormatCoordinator {
  public validationRules: CalcDataValidationRule[] = [];
  public conditionalRules: ConditionalRule[] = [];
  public sparklineGroups: SparklineGroupConfig[] = [];

  private listeners: Set<() => void> = new Set();
  private nextId = 1;

  public subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener();
    }
  }

  // --- Helpers ---

  public parseCoord(coord: string): { row: number; col: number } {
    const match = coord.trim().toUpperCase().match(/^([A-Z]+)([0-9]+)$/);
    if (!match) return { row: 0, col: 0 };
    const colStr = match[1];
    let col = 0;
    for (let i = 0; i < colStr.length; i++) {
      col = col * 26 + (colStr.charCodeAt(i) - 64);
    }
    const row = parseInt(match[2], 10) - 1;
    return { row, col: col - 1 };
  }

  public cellMatchesSqref(sqref: string, coord: string): boolean {
    const target = this.parseCoord(coord);
    for (const part of sqref.trim().toUpperCase().split(/\s+/)) {
      if (part.includes(":")) {
        const [c1, c2] = part.split(":");
        const p1 = this.parseCoord(c1);
        const p2 = this.parseCoord(c2);
        const rMin = Math.min(p1.row, p2.row);
        const rMax = Math.max(p1.row, p2.row);
        const cMin = Math.min(p1.col, p2.col);
        const cMax = Math.max(p1.col, p2.col);
        if (target.row >= rMin && target.row <= rMax && target.col >= cMin && target.col <= cMax) {
          return true;
        }
      } else {
        const p = this.parseCoord(part);
        if (target.row === p.row && target.col === p.col) {
          return true;
        }
      }
    }
    return false;
  }

  // --- Data Validation ---

  public addValidationRule(rule: Omit<CalcDataValidationRule, "id">): string {
    const id = `val-rule-${this.nextId++}`;
    this.validationRules.push({ id, ...rule });
    this.notify();
    return id;
  }

  public removeValidationRule(id: string): void {
    this.validationRules = this.validationRules.filter((r) => r.id !== id);
    this.notify();
  }

  public getValidationForCell(cellCoord: string): CalcDataValidationRule | null {
    return this.validationRules.find((r) => this.cellMatchesSqref(r.sqref, cellCoord)) || null;
  }

  public getListOptions(cellCoord: string, gridValues?: Record<string, any>): string[] {
    const rule = this.getValidationForCell(cellCoord);
    if (!rule || rule.type !== "list") return [];

    const f1 = rule.formula1.trim();
    if (f1.startsWith('"') && f1.endsWith('"') && f1.length >= 2) {
      return f1.slice(1, -1).split(",").map((s) => s.trim());
    }
    if (f1.includes(",") && !f1.includes(":") && !f1.startsWith("=")) {
      return f1.split(",").map((s) => s.trim());
    }

    if (gridValues) {
      const cleanRef = f1.replace(/^=/, "").replace(/\$/g, "");
      if (cleanRef.includes(":")) {
        const [start, end] = cleanRef.split(":");
        const p1 = this.parseCoord(start);
        const p2 = this.parseCoord(end);
        const options: string[] = [];
        for (let r = Math.min(p1.row, p2.row); r <= Math.max(p1.row, p2.row); r++) {
          for (let c = Math.min(p1.col, p2.col); c <= Math.max(p1.col, p2.col); c++) {
            const coord = `${String.fromCharCode(65 + c)}${r + 1}`;
            if (gridValues[coord] !== undefined && gridValues[coord] !== null && gridValues[coord] !== "") {
              options.push(String(gridValues[coord]));
            }
          }
        }
        return options;
      }
    }

    return [];
  }

  public validateCell(
    cellCoord: string,
    val: any,
    gridValues?: Record<string, any>
  ): { isValid: boolean; error?: { style: string; title: string; message: string } } {
    const rule = this.getValidationForCell(cellCoord);
    if (!rule) return { isValid: true };

    if (val === null || val === undefined || val === "") {
      if (rule.allowBlank) return { isValid: true };
      return {
        isValid: false,
        error: {
          style: rule.errorStyle,
          title: rule.errorTitle || "Validation Error",
          message: rule.errorMessage || "This cell cannot be blank.",
        },
      };
    }

    let isValid = true;

    if (rule.type === "list") {
      const options = this.getListOptions(cellCoord, gridValues);
      const strVal = String(val).trim().toUpperCase();
      isValid = options.some((opt) => opt.trim().toUpperCase() === strVal);
    } else if (rule.type === "whole") {
      const num = Number(val);
      if (isNaN(num) || !Number.isInteger(num)) {
        isValid = false;
      } else {
        isValid = this.checkNumericBounds(num, rule.operator, rule.formula1, rule.formula2);
      }
    } else if (rule.type === "decimal") {
      const num = Number(val);
      if (isNaN(num)) {
        isValid = false;
      } else {
        isValid = this.checkNumericBounds(num, rule.operator, rule.formula1, rule.formula2);
      }
    } else if (rule.type === "textLength") {
      const len = String(val).length;
      isValid = this.checkNumericBounds(len, rule.operator, rule.formula1, rule.formula2);
    }

    if (isValid) {
      return { isValid: true };
    }

    return {
      isValid: false,
      error: {
        style: rule.errorStyle,
        title: rule.errorTitle || "Invalid Value",
        message: rule.errorMessage || "The value entered violates cell validation rules.",
      },
    };
  }

  private checkNumericBounds(
    val: number,
    operator: string,
    f1Str: string,
    f2Str?: string
  ): boolean {
    const f1 = parseFloat(f1Str) || 0;
    const f2 = f2Str ? parseFloat(f2Str) : 0;

    switch (operator) {
      case "between": {
        const min = Math.min(f1, f2);
        const max = Math.max(f1, f2);
        return val >= min && val <= max;
      }
      case "notBetween": {
        const min = Math.min(f1, f2);
        const max = Math.max(f1, f2);
        return val < min || val > max;
      }
      case "equal":
        return Math.abs(val - f1) < 1e-9;
      case "notEqual":
        return Math.abs(val - f1) >= 1e-9;
      case "greaterThan":
        return val > f1;
      case "lessThan":
        return val < f1;
      case "greaterThanOrEqual":
        return val >= f1;
      case "lessThanOrEqual":
        return val <= f1;
      default:
        return true;
    }
  }

  // --- Conditional Formatting ---

  public addConditionalRule(rule: Omit<ConditionalRule, "id">): string {
    const id = `cf-rule-${this.nextId++}`;
    this.conditionalRules.push({ id, ...rule });
    this.conditionalRules.sort((a, b) => a.priority - b.priority);
    this.notify();
    return id;
  }

  public removeConditionalRule(id: string): void {
    this.conditionalRules = this.conditionalRules.filter((r) => r.id !== id);
    this.notify();
  }

  public reorderRule(id: string, newPriority: number): void {
    const target = this.conditionalRules.find((r) => r.id === id);
    if (target) {
      target.priority = newPriority;
      this.conditionalRules.sort((a, b) => a.priority - b.priority);
      this.notify();
    }
  }

  public toggleStopIfTrue(id: string): void {
    const target = this.conditionalRules.find((r) => r.id === id);
    if (target) {
      target.stopIfTrue = !target.stopIfTrue;
      this.notify();
    }
  }

  public getAppliedFormatting(
    cellCoord: string,
    val: any,
    rangeData: number[] = []
  ): {
    style?: { fontColor?: string; bold?: boolean; italic?: boolean; fillColor?: string };
    colorScale?: string;
    dataBar?: { ratio: number; color: string };
    icon?: { set: string; index: number };
  } {
    const matchingRules = this.conditionalRules.filter((r) =>
      this.cellMatchesSqref(r.sqref, cellCoord)
    );

    let mergedStyle: any = null;
    let colorScale: string | undefined;
    let dataBar: { ratio: number; color: string } | undefined;
    let icon: { set: string; index: number } | undefined;

    for (const rule of matchingRules) {
      let matched = false;
      const num = Number(val);

      if (rule.type === "cellIs" && rule.cellIs) {
        if (!isNaN(num)) {
          matched = this.checkNumericBounds(
            num,
            rule.cellIs.operator,
            rule.cellIs.formula1,
            rule.cellIs.formula2
          );
        }
      } else if (rule.type === "containsText" && rule.text) {
        matched = String(val).toUpperCase().includes(rule.text.toUpperCase());
      } else if (rule.type === "colorScale" && rule.colorScale && !isNaN(num)) {
        colorScale = this.interpolateColorScale(
          num,
          rule.colorScale.values,
          rule.colorScale.colors
        );
        matched = true;
      } else if (rule.type === "dataBar" && rule.dataBar && !isNaN(num)) {
        const min = rule.dataBar.minVal !== undefined ? rule.dataBar.minVal : Math.min(...rangeData, 0);
        const max = rule.dataBar.maxVal !== undefined ? rule.dataBar.maxVal : Math.max(...rangeData, 1);
        const ratio = max > min ? Math.max(0, Math.min(1, (num - min) / (max - min))) : 1;
        dataBar = { ratio, color: rule.dataBar.color };
        matched = true;
      } else if (rule.type === "iconSet" && rule.iconSet && !isNaN(num)) {
        let idx = 0;
        for (const th of rule.iconSet.thresholds) {
          if (num >= th) idx++;
        }
        icon = { set: rule.iconSet.name, index: idx };
        matched = true;
      }

      if (matched) {
        if (rule.style) {
          mergedStyle = { ...(mergedStyle || {}), ...rule.style };
        }
        if (rule.stopIfTrue) break;
      }
    }

    return {
      style: mergedStyle || undefined,
      colorScale,
      dataBar,
      icon,
    };
  }

  private interpolateColorScale(val: number, values: number[], hexColors: string[]): string {
    if (hexColors.length < 2 || values.length < 2) return hexColors[0] || "#FFFFFF";
    const minVal = values[0];
    const maxVal = values[values.length - 1];
    const clamped = Math.max(minVal, Math.min(maxVal, val));

    if (hexColors.length === 2 || values.length === 2) {
      const t = maxVal > minVal ? (clamped - minVal) / (maxVal - minVal) : 0;
      return this.lerpHex(hexColors[0], hexColors[1], t);
    }

    const midVal = values[1];
    if (clamped <= midVal) {
      const t = midVal > minVal ? (clamped - minVal) / (midVal - minVal) : 0;
      return this.lerpHex(hexColors[0], hexColors[1], t);
    } else {
      const t = maxVal > midVal ? (clamped - midVal) / (maxVal - midVal) : 0;
      return this.lerpHex(hexColors[1], hexColors[2], t);
    }
  }

  private lerpHex(c1: string, c2: string, t: number): string {
    const parse = (hex: string) => {
      const clean = hex.replace(/^#/, "");
      const r = parseInt(clean.slice(0, 2), 16);
      const g = parseInt(clean.slice(2, 4), 16);
      const b = parseInt(clean.slice(4, 6), 16);
      return [isNaN(r) ? 255 : r, isNaN(g) ? 255 : g, isNaN(b) ? 255 : b];
    };
    const [r1, g1, b1] = parse(c1);
    const [r2, g2, b2] = parse(c2);
    const r = Math.round((1 - t) * r1 + t * r2);
    const g = Math.round((1 - t) * g1 + t * g2);
    const b = Math.round((1 - t) * b1 + t * b2);
    const toHex = (n: number) => n.toString(16).padStart(2, "0").toUpperCase();
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  }

  // --- Sparklines Engine ---

  public addSparklineGroup(group: Omit<SparklineGroupConfig, "id">): string {
    const id = `spark-${this.nextId++}`;
    this.sparklineGroups.push({ id, ...group });
    this.notify();
    return id;
  }

  public removeSparklineGroup(id: string): void {
    this.sparklineGroups = this.sparklineGroups.filter((g) => g.id !== id);
    this.notify();
  }

  public renderSparklineSvg(
    groupId: string,
    data: number[],
    width = 100,
    height = 24
  ): string {
    const group = this.sparklineGroups.find((g) => g.id === groupId);
    if (!group || data.length === 0) {
      return `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg"></svg>`;
    }

    const padding = 2;
    const w = Math.max(1, width - padding * 2);
    const h = Math.max(1, height - padding * 2);
    const min = Math.min(...data);
    const max = Math.max(...data);
    const span = Math.max(1e-6, max - min);
    const n = data.length;

    if (group.type === "line") {
      const points = data.map((val, i) => {
        const x = n > 1 ? padding + (i / (n - 1)) * w : padding + w / 2;
        const y = padding + h - ((val - min) / span) * h;
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      });
      let svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg"><polyline fill="none" stroke="${group.seriesColor}" stroke-width="1.5" points="${points.join(" ")}" />`;
      if (group.showMarkers) {
        data.forEach((val, i) => {
          const x = n > 1 ? padding + (i / (n - 1)) * w : padding + w / 2;
          const y = padding + h - ((val - min) / span) * h;
          let fill = group.seriesColor;
          if (group.showHigh && Math.abs(val - max) < 1e-9 && group.highColor) {
            fill = group.highColor;
          } else if (group.showLow && Math.abs(val - min) < 1e-9 && group.lowColor) {
            fill = group.lowColor;
          } else if (group.showNegative && val < 0 && group.negativeColor) {
            fill = group.negativeColor;
          }
          svg += `<circle cx="${x.toFixed(1)}" cy="${y.toFixed(1)}" r="2" fill="${fill}" />`;
        });
      }
      svg += `</svg>`;
      return svg;
    }

    if (group.type === "column") {
      let svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">`;
      const barW = Math.max(1, (w / n) * 0.75);
      const gap = Math.max(0.5, (w / n) * 0.25);
      data.forEach((val, i) => {
        const x = padding + i * (barW + gap);
        const barH = Math.max(2, (Math.abs(val) / span) * h);
        const y = val >= 0 ? padding + h - barH : padding + h;
        let fill = group.seriesColor;
        if (group.showNegative && val < 0 && group.negativeColor) {
          fill = group.negativeColor;
        } else if (group.showHigh && Math.abs(val - max) < 1e-9 && group.highColor) {
          fill = group.highColor;
        } else if (group.showLow && Math.abs(val - min) < 1e-9 && group.lowColor) {
          fill = group.lowColor;
        }
        svg += `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${barW.toFixed(1)}" height="${barH.toFixed(1)}" fill="${fill}" />`;
      });
      svg += `</svg>`;
      return svg;
    }

    // Stacked (Win/Loss)
    let svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">`;
    const barW = Math.max(1, (w / n) * 0.75);
    const gap = Math.max(0.5, (w / n) * 0.25);
    const midY = padding + h / 2;
    data.forEach((val, i) => {
      const x = padding + i * (barW + gap);
      const isPos = val > 0;
      const y = isPos ? padding : midY + 1;
      const barH = Math.max(1, h / 2 - 1);
      const fill = !isPos && group.negativeColor ? group.negativeColor : group.seriesColor;
      svg += `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${barW.toFixed(1)}" height="${barH.toFixed(1)}" fill="${fill}" />`;
    });
    svg += `</svg>`;
    return svg;
  }
}
