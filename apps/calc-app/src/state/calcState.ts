export interface CellData { raw: string; computed: string | number; format?: string; bold?: boolean; }
export class CalcState {
  cells: Map<string, CellData> = new Map([
    ["A1", { raw: "Revenue", computed: "Revenue", bold: true }],
    ["A2", { raw: "15000", computed: 15000 }],
    ["A3", { raw: "25000", computed: 25000 }],
    ["A4", { raw: "=SUM(A2:A3)", computed: 40000, bold: true }]
  ]);
  selectedCell: string = "A1";
  formulaBarText: string = "Revenue";

  selectCell(coord: string) {
    this.selectedCell = coord;
    this.formulaBarText = this.cells.get(coord)?.raw || "";
  }

  commitFormula(val: string) {
    this.cells.set(this.selectedCell, { raw: val, computed: val.startsWith("=") ? "EVAL_PENDING" : val });
    this.formulaBarText = val;
  }
}
export const calcState = new CalcState();
