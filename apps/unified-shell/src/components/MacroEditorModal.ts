export interface MacroSecurityFinding {
  severity: "Info" | "Low" | "Medium" | "High" | "Critical";
  capability?: string;
  message: string;
}

export interface MacroSecurityReport {
  risk_level: "Safe" | "Low" | "Medium" | "High" | "Critical";
  findings: MacroSecurityFinding[];
  required_capabilities: string[];
}

export interface MacroModalOptions {
  initialCode?: string;
  routineName?: string;
  onExecute?: (
    code: string,
    routineName: string,
    grantedCaps: string[]
  ) => Promise<{ success: boolean; return_value?: string; error_message?: string; gas_consumed?: number }>;
  onClose?: () => void;
}

export class MacroEditorModal {
  private container: HTMLElement;
  private backdrop: HTMLElement;
  private modal: HTMLElement;
  private codeInput!: HTMLTextAreaElement;
  private lineNumbersEl!: HTMLElement;
  private riskBadgeEl!: HTMLElement;
  private findingsListEl!: HTMLElement;
  private statusBarEl!: HTMLElement;

  private permFileIo!: HTMLInputElement;
  private permNetwork!: HTMLInputElement;
  private permSysCmd!: HTMLInputElement;

  private options: MacroModalOptions;

  constructor(options: MacroModalOptions = {}) {
    this.options = options;
    this.container = document.body;

    // Build Backdrop & Modal DOM
    this.backdrop = document.createElement("div");
    this.backdrop.className = "macro-modal-backdrop";
    this.backdrop.setAttribute("data-testid", "macro-modal-backdrop");
    this.backdrop.style.cssText =
      "position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.5);display:flex;align-items:center;justify-content:center;z-index:9999;";

    this.modal = document.createElement("div");
    this.modal.className = "macro-editor-modal";
    this.modal.setAttribute("data-testid", "macro-editor-modal");
    this.modal.style.cssText =
      "width:800px;max-width:92vw;height:550px;background:#2d2d2d;color:#f0f0f0;display:flex;flex-direction:column;border-radius:8px;box-shadow:0 12px 32px rgba(0,0,0,0.4);font-family:Segoe UI, sans-serif;";

    this.buildHeader();
    this.buildToolbar();
    this.buildEditorPane();
    this.buildSecurityPane();
    this.buildStatusBar();

    this.backdrop.appendChild(this.modal);
  }

  private buildHeader() {
    const header = document.createElement("div");
    header.className = "macro-modal-header";
    header.style.cssText =
      "padding:10px 16px;background:#1e1e1e;border-bottom:1px solid #3c3c3c;display:flex;align-items:center;justify-content:space-between;";

    const title = document.createElement("div");
    title.style.cssText = "font-weight:600;font-size:14px;display:flex;align-items:center;gap:8px;";
    title.innerHTML = "<span>💻</span><span>Microsoft Visual Basic for Applications - [AetherOffice]</span>";

    const closeBtn = document.createElement("button");
    closeBtn.setAttribute("data-testid", "macro-modal-close");
    closeBtn.textContent = "✕";
    closeBtn.style.cssText =
      "background:none;border:none;color:#aaa;font-size:16px;cursor:pointer;padding:4px 8px;";
    closeBtn.onclick = () => this.close();

    header.appendChild(title);
    header.appendChild(closeBtn);
    this.modal.appendChild(header);
  }

  private buildToolbar() {
    const toolbar = document.createElement("div");
    toolbar.className = "macro-modal-toolbar";
    toolbar.style.cssText =
      "padding:6px 12px;background:#252526;border-bottom:1px solid #3c3c3c;display:flex;align-items:center;gap:12px;font-size:12px;flex-wrap:wrap;";

    // Run Button
    const runBtn = document.createElement("button");
    runBtn.setAttribute("data-testid", "macro-btn-run");
    runBtn.innerHTML = "▶ <b>Run (F5)</b>";
    runBtn.style.cssText =
      "background:#0e639c;color:#fff;border:none;padding:5px 12px;border-radius:4px;cursor:pointer;";
    runBtn.onclick = () => this.run();

    // Step Into Button
    const stepBtn = document.createElement("button");
    stepBtn.setAttribute("data-testid", "macro-btn-step");
    stepBtn.innerHTML = "⏭ <b>Step Into (F8)</b>";
    stepBtn.style.cssText =
      "background:#3c3c3c;color:#eee;border:none;padding:5px 10px;border-radius:4px;cursor:pointer;";
    stepBtn.onclick = () => this.stepInto();

    // Reset Button
    const resetBtn = document.createElement("button");
    resetBtn.setAttribute("data-testid", "macro-btn-reset");
    resetBtn.innerHTML = "⏹ <b>Reset</b>";
    resetBtn.style.cssText =
      "background:#3c3c3c;color:#eee;border:none;padding:5px 10px;border-radius:4px;cursor:pointer;";
    resetBtn.onclick = () => this.reset();

    // Scan Button
    const scanBtn = document.createElement("button");
    scanBtn.setAttribute("data-testid", "macro-btn-scan");
    scanBtn.innerHTML = "🛡️ <b>Scan Security</b>";
    scanBtn.style.cssText =
      "background:#3c3c3c;color:#eee;border:none;padding:5px 10px;border-radius:4px;cursor:pointer;";
    scanBtn.onclick = () => this.scanSecurity();

    // Permissions toggles container
    const permsContainer = document.createElement("div");
    permsContainer.style.cssText = "display:flex;align-items:center;gap:8px;margin-left:auto;";

    this.permFileIo = this.createCheckbox("perm-toggle-fileio", "File I/O");
    this.permNetwork = this.createCheckbox("perm-toggle-network", "Network");
    this.permSysCmd = this.createCheckbox("perm-toggle-syscmd", "System CMD");

    permsContainer.appendChild(this.permFileIo.parentElement!);
    permsContainer.appendChild(this.permNetwork.parentElement!);
    permsContainer.appendChild(this.permSysCmd.parentElement!);

    toolbar.appendChild(runBtn);
    toolbar.appendChild(stepBtn);
    toolbar.appendChild(resetBtn);
    toolbar.appendChild(scanBtn);
    toolbar.appendChild(permsContainer);
    this.modal.appendChild(toolbar);
  }

  private createCheckbox(testId: string, labelText: string): HTMLInputElement {
    const label = document.createElement("label");
    label.style.cssText = "display:flex;align-items:center;gap:4px;cursor:pointer;font-size:11px;";
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.setAttribute("data-testid", testId);
    label.appendChild(checkbox);
    label.appendChild(document.createTextNode(labelText));
    return checkbox;
  }

  private buildEditorPane() {
    const pane = document.createElement("div");
    pane.className = "macro-editor-pane";
    pane.style.cssText = "flex:1;display:flex;position:relative;background:#1e1e1e;overflow:hidden;";

    // Line numbers gutter
    this.lineNumbersEl = document.createElement("div");
    this.lineNumbersEl.setAttribute("data-testid", "macro-line-numbers");
    this.lineNumbersEl.style.cssText =
      "width:40px;background:#1e1e1e;color:#858585;text-align:right;padding:8px 8px;font-family:Consolas, monospace;font-size:13px;line-height:20px;user-select:none;border-right:1px solid #333;";

    // Textarea input
    this.codeInput = document.createElement("textarea");
    this.codeInput.setAttribute("data-testid", "macro-code-input");
    this.codeInput.setAttribute("spellcheck", "false");
    this.codeInput.style.cssText =
      "flex:1;background:transparent;color:#d4d4d4;border:none;outline:none;padding:8px 12px;font-family:Consolas, monospace;font-size:13px;line-height:20px;resize:none;tab-size:4;white-space:pre;";

    const defaultCode =
      this.options.initialCode ||
      `Sub CalculateCommission()\n    Dim revenue As Double\n    Dim rate As Double\n    revenue = Range("A1").Value\n    rate = 0.15\n    Range("B1").Value = revenue * rate\nEnd Sub\n`;

    this.codeInput.value = defaultCode;
    this.updateLineNumbers();

    this.codeInput.addEventListener("input", () => {
      this.updateLineNumbers();
      this.scanSecurity();
    });

    pane.appendChild(this.lineNumbersEl);
    pane.appendChild(this.codeInput);
    this.modal.appendChild(pane);
  }

  private buildSecurityPane() {
    const secPane = document.createElement("div");
    secPane.setAttribute("data-testid", "macro-audit-pane");
    secPane.style.cssText =
      "height:100px;background:#252526;border-top:1px solid #3c3c3c;padding:8px 12px;display:flex;flex-direction:column;font-size:12px;overflow-y:auto;";

    const headerRow = document.createElement("div");
    headerRow.style.cssText = "display:flex;align-items:center;justify-content:space-between;margin-bottom:6px;";

    const label = document.createElement("span");
    label.style.fontWeight = "600";
    label.textContent = "Security Sandbox Audit:";

    this.riskBadgeEl = document.createElement("span");
    this.riskBadgeEl.setAttribute("data-testid", "macro-risk-badge");
    this.riskBadgeEl.style.cssText =
      "padding:2px 8px;border-radius:12px;font-weight:600;font-size:11px;background:#2e7d32;color:#fff;";
    this.riskBadgeEl.textContent = "Safe";

    headerRow.appendChild(label);
    headerRow.appendChild(this.riskBadgeEl);
    secPane.appendChild(headerRow);

    this.findingsListEl = document.createElement("div");
    this.findingsListEl.setAttribute("data-testid", "macro-findings-list");
    this.findingsListEl.style.cssText = "color:#aaa;font-size:11px;";
    this.findingsListEl.textContent = "No suspicious external API invocations detected.";
    secPane.appendChild(this.findingsListEl);

    this.modal.appendChild(secPane);
  }

  private buildStatusBar() {
    this.statusBarEl = document.createElement("div");
    this.statusBarEl.setAttribute("data-testid", "macro-status-bar");
    this.statusBarEl.style.cssText =
      "padding:4px 12px;background:#007acc;color:#fff;font-size:11px;display:flex;align-items:center;justify-content:space-between;";
    this.statusBarEl.textContent = "Ready - Gas Limit: 100,000";
    this.modal.appendChild(this.statusBarEl);
  }

  private updateLineNumbers() {
    const lines = this.codeInput.value.split("\n").length;
    let numbers = "";
    for (let i = 1; i <= lines; i++) {
      numbers += `${i}\n`;
    }
    this.lineNumbersEl.textContent = numbers;
  }

  public scanSecurity(): MacroSecurityReport {
    const code = this.codeInput.value.toLowerCase();
    const findings: MacroSecurityFinding[] = [];
    const requiredCaps: string[] = [];

    if (code.includes("shell(") || code.includes("shell ")) {
      findings.push({
        severity: "Critical",
        capability: "SystemCommand",
        message: "Dangerous invocation of system Shell command",
      });
      requiredCaps.push("SystemCommand");
    }

    if (code.includes("wscript.shell")) {
      findings.push({
        severity: "Critical",
        capability: "SystemCommand",
        message: "Creation of WScript.Shell OS automation component",
      });
      requiredCaps.push("SystemCommand");
    }

    if (code.includes("xmlhttp") || code.includes("winhttprequest")) {
      findings.push({
        severity: "High",
        capability: "Network",
        message: "Outbound network connection object requested",
      });
      requiredCaps.push("Network");
    }

    if (code.includes("filesystemobject") || code.includes("adodb.stream")) {
      findings.push({
        severity: "High",
        capability: "FileIo",
        message: "Direct disk file system I/O object referenced",
      });
      requiredCaps.push("FileIo");
    }

    if (code.includes("environ(")) {
      findings.push({
        severity: "Medium",
        capability: "Environment",
        message: "Access to system environment variables",
      });
      requiredCaps.push("Environment");
    }

    const risk_level: "Safe" | "Low" | "Medium" | "High" | "Critical" =
      findings.some((f) => f.severity === "Critical")
        ? "Critical"
        : findings.some((f) => f.severity === "High")
        ? "High"
        : findings.some((f) => f.severity === "Medium")
        ? "Medium"
        : "Safe";

    this.renderSecurityReport({ risk_level, findings, required_capabilities: requiredCaps });
    return { risk_level, findings, required_capabilities: requiredCaps };
  }

  private renderSecurityReport(report: MacroSecurityReport) {
    this.riskBadgeEl.textContent = report.risk_level;
    switch (report.risk_level) {
      case "Critical":
        this.riskBadgeEl.style.background = "#d32f2f";
        break;
      case "High":
        this.riskBadgeEl.style.background = "#e65100";
        break;
      case "Medium":
        this.riskBadgeEl.style.background = "#f57f17";
        break;
      case "Low":
        this.riskBadgeEl.style.background = "#0288d1";
        break;
      default:
        this.riskBadgeEl.style.background = "#2e7d32";
        break;
    }

    if (report.findings.length === 0) {
      this.findingsListEl.textContent = "No suspicious external API invocations detected.";
      this.findingsListEl.style.color = "#aaa";
    } else {
      this.findingsListEl.innerHTML = report.findings
        .map((f) => `<div style="color:#ff8a80;margin:2px 0;">⚠ [${f.severity}] ${f.message}</div>`)
        .join("");
    }
  }

  public async run(): Promise<void> {
    this.statusBarEl.textContent = "Running macro in isolation sandbox...";
    this.statusBarEl.style.background = "#007acc";

    const grantedCaps: string[] = [];
    if (this.permFileIo.checked) grantedCaps.push("FileIo");
    if (this.permNetwork.checked) grantedCaps.push("Network");
    if (this.permSysCmd.checked) grantedCaps.push("SystemCommand");

    const code = this.codeInput.value;
    const routineName = this.options.routineName || "";

    try {
      if (this.options.onExecute) {
        const result = await this.options.onExecute(code, routineName, grantedCaps);
        if (result.success) {
          const gas = result.gas_consumed || 12;
          this.statusBarEl.textContent = `Completed successfully. Gas consumed: ${gas}`;
          this.statusBarEl.style.background = "#2e7d32";
        } else {
          this.statusBarEl.textContent = `Execution failed: ${result.error_message || "Unknown error"}`;
          this.statusBarEl.style.background = "#d32f2f";
        }
      } else {
        // Direct local simulated execution
        const report = this.scanSecurity();
        if (report.risk_level === "Critical" && !this.permSysCmd.checked) {
          this.statusBarEl.textContent = "Execution blocked: SystemCommand capability denied by sandbox";
          this.statusBarEl.style.background = "#d32f2f";
          return;
        }
        this.statusBarEl.textContent = "Completed successfully. Gas consumed: 15";
        this.statusBarEl.style.background = "#2e7d32";
      }
    } catch (err: unknown) {
      this.statusBarEl.textContent = `Execution error: ${String(err)}`;
      this.statusBarEl.style.background = "#d32f2f";
    }
  }

  public stepInto() {
    this.statusBarEl.textContent = "Single-step execution paused at line 1";
    this.statusBarEl.style.background = "#f57f17";
  }

  public reset() {
    this.statusBarEl.textContent = "Execution halted. State reset to initial.";
    this.statusBarEl.style.background = "#007acc";
  }

  public open() {
    this.container.appendChild(this.backdrop);
    this.scanSecurity();
  }

  public close() {
    if (this.backdrop.parentElement) {
      this.backdrop.parentElement.removeChild(this.backdrop);
    }
    if (this.options.onClose) {
      this.options.onClose();
    }
  }

  public getCode(): string {
    return this.codeInput.value;
  }

  public setCode(code: string) {
    this.codeInput.value = code;
    this.updateLineNumbers();
    this.scanSecurity();
  }

  public setPermission(cap: "FileIo" | "Network" | "SystemCommand", granted: boolean) {
    if (cap === "FileIo") this.permFileIo.checked = granted;
    if (cap === "Network") this.permNetwork.checked = granted;
    if (cap === "SystemCommand") this.permSysCmd.checked = granted;
  }

  public static open(options: MacroModalOptions = {}): MacroEditorModal {
    const modal = new MacroEditorModal(options);
    modal.open();
    return modal;
  }
}

