# bootstrap_standalone_apps.ps1 - Scaffolds 4 Independent Apps + Launcher
$ErrorActionPreference = "Stop"
Write-Host "=== [1/3] Partitioning Monolithic UI into 4 Standalone Engines ===" -ForegroundColor Cyan

$apps = @("writer-app", "calc-app", "slides-app", "pdf-app", "launcher-shell")
foreach ($app in $apps) {
    New-Item -ItemType Directory -Force -Path "apps/$app/src/components" | Out-Null
    New-Item -ItemType Directory -Force -Path "apps/$app/src/state" | Out-Null
    New-Item -ItemType Directory -Force -Path "apps/$app/src-tauri/src" | Out-Null
}

# 1. Writer (Word) Dedicated State & Canvas Engine
@'
export interface WordRun { text: string; bold?: boolean; italic?: boolean; underline?: boolean; font?: string; size?: number; }
export interface WordParagraph { id: string; style: string; runs: WordRun[]; alignment: "left" | "center" | "right" | "justify"; }
export interface WordDocumentModel { id: string; title: string; sections: { paragraphs: WordParagraph[] }[]; }

export class WriterState {
  doc: WordDocumentModel = { id: "doc-1", title: "Document 1.docx", sections: [{ paragraphs: [
    { id: "p1", style: "Heading1", alignment: "left", runs: [{ text: "Executive Summary", bold: true, size: 24 }] },
    { id: "p2", style: "Normal", alignment: "left", runs: [{ text: "Production-grade open-source document authoring powered by Rust." }] }
  ]}] };
  cursor = { paragraphIndex: 0, runIndex: 0, offset: 0 };
  activeFormat = { bold: false, italic: false, underline: false, font: "Calibri", size: 11, alignment: "left" };

  execCommand(cmd: string, val?: any) {
    if (cmd === "toggleBold") this.activeFormat.bold = !this.activeFormat.bold;
    if (cmd === "toggleItalic") this.activeFormat.italic = !this.activeFormat.italic;
    if (cmd === "setFont") this.activeFormat.font = val;
    if (cmd === "setAlignment") this.activeFormat.alignment = val;
    console.log(`[WriterCommand] Executed: ${cmd}`, val, this.activeFormat);
  }
}
export const writerState = new WriterState();
'@ | Set-Content -Path "apps/writer-app/src/state/writerState.ts" -Encoding UTF8

# 2. Calc (Excel) Dedicated Grid State & Virtualized Math Engine
@'
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
'@ | Set-Content -Path "apps/calc-app/src/state/calcState.ts" -Encoding UTF8

# 3. Slides (PowerPoint) Dedicated Shape Tree & Slide Canvas State
@'
export interface SlideShape { id: string; type: "text" | "rect" | "ellipse"; x: number; y: number; w: number; h: number; fill: string; text?: string; }
export interface SlideData { id: string; title: string; shapes: SlideShape[]; }

export class SlidesState {
  slides: SlideData[] = [
    { id: "s1", title: "Title Slide", shapes: [
      { id: "sh1", type: "text", x: 100, y: 150, w: 800, h: 80, fill: "transparent", text: "Kinetiq Presentation" },
      { id: "sh2", type: "rect", x: 100, y: 250, w: 400, h: 4, fill: "#2563eb" }
    ]},
    { id: "s2", title: "Problem Statement", shapes: [] },
    { id: "s3", title: "Solution & Demo", shapes: [] }
  ];
  currentSlideIndex = 0;
  selectedShapeId: string | null = null;

  selectSlide(idx: number) { this.currentSlideIndex = idx; this.selectedShapeId = null; }
  selectShape(id: string) { this.selectedShapeId = id; }
}
export const slidesState = new SlidesState();
'@ | Set-Content -Path "apps/slides-app/src/state/slidesState.ts" -Encoding UTF8

# 4. PDF Studio Dedicated Form & Document State
@'
export interface PdfFormField { id: string; name: string; type: "text" | "checkbox"; value: string | boolean; rect: [number, number, number, number]; }
export class PdfState {
  pageCount = 1;
  currentPage = 1;
  zoom = 1.0;
  formFields: PdfFormField[] = [
    { id: "f1", name: "Full_Name", type: "text", value: "John Doe", rect: [100, 200, 250, 30] },
    { id: "f2", name: "Agreed", type: "checkbox", value: true, rect: [100, 250, 20, 20] }
  ];
  activeTool: "select" | "edit_text" | "highlight" | "form_fill" | "sign" = "form_fill";

  setTool(t: typeof this.activeTool) { this.activeTool = t; }
}
export const pdfState = new PdfState();
'@ | Set-Content -Path "apps/pdf-app/src/state/pdfState.ts" -Encoding UTF8

Write-Host "4 Independent application engines created with dedicated state stores." -ForegroundColor Green

