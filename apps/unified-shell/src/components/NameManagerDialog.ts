import { calcState } from "../state/calcState";

export class NameManagerDialog {
  private element: HTMLElement;
  private isVisible = false;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "name-manager-modal";
    this.element.id = "name-manager-modal";
    this.element.setAttribute("data-testid", "name-manager-dialog");
    this.applyStyles();
    document.body.appendChild(this.element);
  }

  public show(): void {
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render();
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

  public render(): void {
    const names = calcState.getAllNames();
    const rowsHtml =
      names.length > 0
        ? names
            .map(
              (item) => `
          <tr style="border-bottom: 1px solid #f3f4f6;">
            <td style="padding: 6px 12px; font-weight: 500;">${item.name}</td>
            <td style="padding: 6px 12px; color: #4b5563;">${item.value}</td>
            <td style="padding: 6px 12px; text-align: right;">
              <button class="btn-del-name" data-name="${item.name}" style="background: none; border: none; color: #dc2626; cursor: pointer; font-size: 12px;">Delete</button>
            </td>
          </tr>`
            )
            .join("")
        : `<tr><td colspan="3" style="padding: 16px; text-align: center; color: #9ca3af; font-size: 13px;">No defined names yet.</td></tr>`;

    this.element.innerHTML = `
      <div class="name-manager-card" style="background: #ffffff; border-radius: 8px; width: 440px; box-shadow: 0 12px 32px rgba(0,0,0,0.25); overflow: hidden; border: 1px solid #e5e7eb;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; border-bottom: 1px solid #f3f4f6; background: #f9fafb;">
          <span style="font-weight: 600; font-size: 14px; color: #111827;">Name Manager</span>
          <button class="btn-close-nm" data-testid="btn-close-name-manager" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
        </div>
        
        <div style="padding: 16px; display: flex; flex-direction: column; gap: 12px;">
          <!-- Defined Names List -->
          <div style="border: 1px solid #e5e7eb; border-radius: 6px; max-height: 180px; overflow-y: auto;">
            <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
              <thead>
                <tr style="background: #f8fafc; border-bottom: 1px solid #e5e7eb; text-align: left; color: #64748b;">
                  <th style="padding: 6px 12px;">Name</th>
                  <th style="padding: 6px 12px;">Refers To / Value</th>
                  <th style="padding: 6px 12px; text-align: right;">Action</th>
                </tr>
              </thead>
              <tbody class="nm-tbody">
                ${rowsHtml}
              </tbody>
            </table>
          </div>

          <!-- Add Name Form -->
          <div style="background: #f9fafb; padding: 12px; border-radius: 6px; border: 1px solid #e5e7eb; display: flex; flex-direction: column; gap: 8px;">
            <span style="font-size: 12px; font-weight: 600; color: #374151;">New Defined Name</span>
            <div style="display: flex; gap: 8px;">
              <input type="text" class="input-nm-name" data-testid="input-name-manager-name" placeholder="Name (e.g. TaxRate)" style="flex: 1; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 12px;" />
              <input type="text" class="input-nm-val" data-testid="input-name-manager-value" placeholder="Value (e.g. 0.18)" style="flex: 1; padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 12px;" />
              <button class="btn-add-nm" data-testid="btn-add-name" style="padding: 6px 14px; background: #107c41; color: #ffffff; border: none; border-radius: 4px; font-size: 12px; font-weight: 500; cursor: pointer;">Add</button>
            </div>
          </div>
        </div>

        <div style="display: flex; justify-content: flex-end; padding: 12px 18px; border-top: 1px solid #f3f4f6; background: #f9fafb;">
          <button class="btn-nm-close-bottom" data-testid="btn-close-name-manager-bottom" style="padding: 6px 16px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; color: #374151; cursor: pointer; font-size: 13px;">Close</button>
        </div>
      </div>
    `;

    // Wire events
    const closeBtn = this.element.querySelector(".btn-close-nm") as HTMLButtonElement | null;
    const closeBottomBtn = this.element.querySelector(".btn-nm-close-bottom") as HTMLButtonElement | null;
    closeBtn?.addEventListener("click", () => this.hide());
    closeBottomBtn?.addEventListener("click", () => this.hide());

    const addBtn = this.element.querySelector(".btn-add-nm") as HTMLButtonElement | null;
    const nameInput = this.element.querySelector(".input-nm-name") as HTMLInputElement | null;
    const valInput = this.element.querySelector(".input-nm-val") as HTMLInputElement | null;

    addBtn?.addEventListener("click", () => {
      const name = nameInput?.value.trim();
      const val = valInput?.value.trim();
      if (name && val !== undefined) {
        const num = Number(val);
        calcState.defineName(name, !isNaN(num) && val !== "" ? num : val);
        this.render();
      }
    });

    const delButtons = this.element.querySelectorAll(".btn-del-name");
    delButtons.forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const targetName = (e.currentTarget as HTMLElement).getAttribute("data-name");
        if (targetName) {
          calcState.deleteName(targetName);
          this.render();
        }
      });
    });
  }
}
