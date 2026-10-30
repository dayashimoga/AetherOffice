import { calcState } from "../state/calcState";

export class GoalSeekDialog {
  private element: HTMLElement;
  private isVisible = false;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "goal-seek-modal";
    this.element.id = "goal-seek-modal";
    this.element.setAttribute("data-testid", "goal-seek-dialog");
    this.applyStyles();
    document.body.appendChild(this.element);
  }

  public show(defaultTarget?: string, defaultChanging?: string): void {
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render(defaultTarget || calcState.selectedCell, defaultChanging || "A1");
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

  public render(targetCell = "C1", changingCell = "A1"): void {
    this.element.innerHTML = `
      <div class="goal-seek-card" style="background: #ffffff; border-radius: 8px; width: 360px; box-shadow: 0 12px 32px rgba(0,0,0,0.25); overflow: hidden; border: 1px solid #e5e7eb;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; border-bottom: 1px solid #f3f4f6; background: #f9fafb;">
          <span style="font-weight: 600; font-size: 14px; color: #111827;">What-If Goal Seek</span>
          <button class="btn-close-gs" data-testid="btn-close-goal-seek" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
        </div>

        <div style="padding: 16px; display: flex; flex-direction: column; gap: 12px; font-size: 13px;">
          <div style="display: flex; align-items: center; justify-content: space-between;">
            <label style="color: #4b5563;">Set cell:</label>
            <input type="text" class="input-gs-target" data-testid="input-goal-seek-target" value="${targetCell}" style="width: 140px; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px;" />
          </div>

          <div style="display: flex; align-items: center; justify-content: space-between;">
            <label style="color: #4b5563;">To value:</label>
            <input type="text" class="input-gs-value" data-testid="input-goal-seek-value" value="100" style="width: 140px; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px;" />
          </div>

          <div style="display: flex; align-items: center; justify-content: space-between;">
            <label style="color: #4b5563;">By changing cell:</label>
            <input type="text" class="input-gs-changing" data-testid="input-goal-seek-changing" value="${changingCell}" style="width: 140px; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px;" />
          </div>

          <div class="gs-result-area" data-testid="goal-seek-result" style="margin-top: 4px; font-size: 12px; color: #059669; min-height: 18px;"></div>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px; padding: 12px 18px; border-top: 1px solid #f3f4f6; background: #f9fafb;">
          <button class="btn-gs-solve" data-testid="btn-goal-seek-solve" style="padding: 6px 16px; border: none; border-radius: 4px; background: #107c41; color: #ffffff; cursor: pointer; font-size: 13px; font-weight: 500;">OK</button>
          <button class="btn-gs-cancel" data-testid="btn-goal-seek-cancel" style="padding: 6px 16px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; color: #374151; cursor: pointer; font-size: 13px;">Cancel</button>
        </div>
      </div>
    `;

    const closeBtn = this.element.querySelector(".btn-close-gs") as HTMLButtonElement | null;
    const cancelBtn = this.element.querySelector(".btn-gs-cancel") as HTMLButtonElement | null;
    closeBtn?.addEventListener("click", () => this.hide());
    cancelBtn?.addEventListener("click", () => this.hide());

    const solveBtn = this.element.querySelector(".btn-gs-solve") as HTMLButtonElement | null;
    const targetInput = this.element.querySelector(".input-gs-target") as HTMLInputElement | null;
    const valInput = this.element.querySelector(".input-gs-value") as HTMLInputElement | null;
    const changingInput = this.element.querySelector(".input-gs-changing") as HTMLInputElement | null;
    const resultArea = this.element.querySelector(".gs-result-area") as HTMLElement | null;

    solveBtn?.addEventListener("click", () => {
      const tgt = targetInput?.value.trim().toUpperCase() || "C1";
      const val = Number(valInput?.value.trim() || 0);
      const chg = changingInput?.value.trim().toUpperCase() || "A1";

      const outcome = calcState.goalSeek(tgt, val, chg);
      if (resultArea) {
        if (outcome.success) {
          resultArea.style.color = "#059669";
          resultArea.textContent = `Goal Seek with cell ${tgt} found a solution: ${chg} = ${outcome.finalValue}`;
        } else {
          resultArea.style.color = "#dc2626";
          resultArea.textContent = `Goal Seek may not have found a solution. Best value: ${outcome.finalValue}`;
        }
      }
    });
  }
}
