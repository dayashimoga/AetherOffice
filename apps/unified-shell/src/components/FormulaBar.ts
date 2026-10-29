export class FormulaBar {
  private container: HTMLElement;
  private nameBoxEl!: HTMLInputElement;
  private formulaInputEl!: HTMLInputElement;
  private onCommitCallback?: (val: string) => void;
  private onCancelCallback?: () => void;
  private activeCellText = "";

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`FormulaBar container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setCallbacks(callbacks: {
    onCommit?: (val: string) => void;
    onCancel?: () => void;
  }) {
    this.onCommitCallback = callbacks.onCommit;
    this.onCancelCallback = callbacks.onCancel;
  }

  public setActiveCell(a1Coord: string, formulaOrValue: string) {
    this.activeCellText = formulaOrValue;
    if (this.nameBoxEl) this.nameBoxEl.value = a1Coord;
    if (this.formulaInputEl) this.formulaInputEl.value = formulaOrValue;
  }

  public focus() {
    if (this.formulaInputEl) {
      this.formulaInputEl.focus();
      this.formulaInputEl.select();
    }
  }

  private render() {
    this.container.innerHTML = "";

    const bar = document.createElement("div");
    bar.className = "formula-bar-container";

    // 1. Name Box
    this.nameBoxEl = document.createElement("input");
    this.nameBoxEl.className = "name-box";
    this.nameBoxEl.value = "A1";
    this.nameBoxEl.readOnly = true;

    // 2. Commit / Cancel / fx Buttons
    const btnBox = document.createElement("div");
    btnBox.className = "formula-buttons";

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "formula-btn";
    cancelBtn.textContent = "✕";
    cancelBtn.title = "Cancel (Esc)";
    cancelBtn.onclick = () => {
      this.formulaInputEl.value = this.activeCellText;
      if (this.onCancelCallback) this.onCancelCallback();
    };

    const commitBtn = document.createElement("button");
    commitBtn.className = "formula-btn";
    commitBtn.textContent = "✓";
    commitBtn.title = "Enter (Commit)";
    commitBtn.onclick = () => {
      if (this.onCommitCallback) this.onCommitCallback(this.formulaInputEl.value);
    };

    const fxLabel = document.createElement("button");
    fxLabel.className = "formula-btn";
    fxLabel.textContent = "fx";
    fxLabel.style.fontWeight = "bold";
    fxLabel.style.fontStyle = "italic";

    btnBox.appendChild(cancelBtn);
    btnBox.appendChild(commitBtn);
    btnBox.appendChild(fxLabel);

    // 3. Formula Input Field
    this.formulaInputEl = document.createElement("input");
    this.formulaInputEl.className = "formula-input";
    this.formulaInputEl.placeholder = "Enter text or formula (=SUM, etc.)...";
    this.formulaInputEl.onkeydown = (e) => {
      if (e.key === "Enter") {
        if (this.onCommitCallback) this.onCommitCallback(this.formulaInputEl.value);
      } else if (e.key === "Escape") {
        this.formulaInputEl.value = this.activeCellText;
        if (this.onCancelCallback) this.onCancelCallback();
      }
    };

    bar.appendChild(this.nameBoxEl);
    bar.appendChild(btnBox);
    bar.appendChild(this.formulaInputEl);

    this.container.appendChild(bar);
  }
}
