export interface WorkbookStats {
  sheetCount: number;
  cellCount: number;
  formulasCount: number;
  chartsCount: number;
  tablesCount: number;
}

export class WorkbookStatisticsDialog {
  private element: HTMLElement;
  private isVisible = false;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "workbook-stats-modal";
    this.element.id = "workbook-stats-modal";
    this.element.setAttribute("data-testid", "workbook-stats-dialog");
    this.applyStyles();
    document.body.appendChild(this.element);
  }

  public show(stats: WorkbookStats): void {
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render(stats);
  }

  public hide(): void {
    this.isVisible = false;
    this.element.style.display = "none";
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  private applyStyles(): void {
    this.element.style.display = "none";
    this.element.style.position = "fixed";
    this.element.style.top = "0";
    this.element.style.left = "0";
    this.element.style.width = "100vw";
    this.element.style.height = "100vh";
    this.element.style.backgroundColor = "rgba(0, 0, 0, 0.4)";
    this.element.style.zIndex = "2000";
    this.element.style.alignItems = "center";
    this.element.style.justifyContent = "center";
    this.element.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
  }

  public render(stats: WorkbookStats): void {
    this.element.innerHTML = `
      <div class="workbook-stats-card" style="background: #ffffff; border-radius: 8px; width: 340px; box-shadow: 0 12px 32px rgba(0,0,0,0.25); overflow: hidden; border: 1px solid #e5e7eb;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; border-bottom: 1px solid #f3f4f6; background: #f9fafb;">
          <span style="font-weight: 600; font-size: 14px; color: #111827;">Workbook Statistics</span>
          <button class="btn-close-wb-stats" data-testid="btn-close-wb-stats" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
        </div>
        <div style="padding: 16px 20px; display: flex; flex-direction: column; gap: 10px; font-size: 13px; color: #374151;">
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Worksheets</span><strong class="wb-stats-sheets" data-testid="wb-stats-sheets">${stats.sheetCount}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Populated Cells</span><strong class="wb-stats-cells" data-testid="wb-stats-cells">${stats.cellCount}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Formulas</span><strong class="wb-stats-formulas" data-testid="wb-stats-formulas">${stats.formulasCount}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Charts</span><strong class="wb-stats-charts" data-testid="wb-stats-charts">${stats.chartsCount}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Tables</span><strong class="wb-stats-tables" data-testid="wb-stats-tables">${stats.tablesCount}</strong></div>
        </div>
        <div style="display: flex; justify-content: flex-end; padding: 12px 18px; border-top: 1px solid #f3f4f6; background: #f9fafb;">
          <button class="btn-wb-stats-ok" data-testid="btn-wb-stats-ok" style="padding: 6px 16px; border: 1px solid #d1d5db; border-radius: 4px; background: #107c41; color: #ffffff; cursor: pointer; font-size: 13px; font-weight: 500;">Close</button>
        </div>
      </div>
    `;

    const closeBtn = this.element.querySelector(".btn-close-wb-stats") as HTMLButtonElement | null;
    const okBtn = this.element.querySelector(".btn-wb-stats-ok") as HTMLButtonElement | null;
    closeBtn?.addEventListener("click", () => this.hide());
    okBtn?.addEventListener("click", () => this.hide());
  }
}

