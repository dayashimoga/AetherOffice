import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { DeveloperRibbon } from "../components/ribbon/DeveloperRibbon";
import { CalcRibbon } from "../components/ribbon/CalcRibbon";
import { MacroEditorModal } from "../components/MacroEditorModal";

describe("Developer Ribbon & Macro Editor Modal (GEC-v2.0 Subphase 15.2.2)", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.restoreAllMocks();
  });

  describe("1. Developer Ribbon Component", () => {
    it("provides the Developer tab and Keytip 'L'", () => {
      const devRibbon = new DeveloperRibbon();
      expect(devRibbon.getTabs()).toEqual(["Developer"]);
      expect(devRibbon.getKeytip("Developer")).toBe("L");
    });

    it("renders Code, Add-ins, and Security ribbon groups with buttons", () => {
      const devRibbon = new DeveloperRibbon();
      const body = document.createElement("div");
      const emit = vi.fn();

      devRibbon.renderTab("Developer", body, emit);

      const vbaBtn = body.querySelector('[data-testid="btn-vba-editor"]') as HTMLButtonElement;
      const macrosBtn = body.querySelector('[data-testid="btn-macros-dialog"]') as HTMLButtonElement;
      const recordBtn = body.querySelector('[data-testid="btn-record-macro"]') as HTMLButtonElement;
      const addinsBtn = body.querySelector('[data-testid="btn-addins-dialog"]') as HTMLButtonElement;
      const secBtn = body.querySelector('[data-testid="btn-macro-security"]') as HTMLButtonElement;

      expect(vbaBtn).toBeTruthy();
      expect(macrosBtn).toBeTruthy();
      expect(recordBtn).toBeTruthy();
      expect(addinsBtn).toBeTruthy();
      expect(secBtn).toBeTruthy();

      // Trigger actions
      vbaBtn.click();
      expect(emit).toHaveBeenCalledWith("OpenMacroEditor");

      macrosBtn.click();
      expect(emit).toHaveBeenCalledWith("OpenMacrosDialog");

      recordBtn.click();
      expect(emit).toHaveBeenCalledWith("ToggleRecordMacro");
      expect(recordBtn.classList.contains("active")).toBe(true);

      addinsBtn.click();
      expect(emit).toHaveBeenCalledWith("OpenAddinsDialog");

      secBtn.click();
      expect(emit).toHaveBeenCalledWith("OpenMacroSecurity");
    });

    it("CalcRibbon includes Developer tab when enabled and renders via DeveloperRibbon", () => {
      const calcRibbon = new CalcRibbon();
      calcRibbon.setDeveloperTabVisible(true);
      const tabs = calcRibbon.getTabs();
      expect(tabs).toContain("Developer");
      expect(calcRibbon.getKeytip("Developer")).toBe("L");

      const body = document.createElement("div");
      const emit = vi.fn();
      calcRibbon.renderTab("Developer", body, emit);

      const vbaBtn = body.querySelector('[data-testid="btn-vba-editor"]');
      expect(vbaBtn).toBeTruthy();
    });
  });

  describe("2. MacroEditorModal Lifecycle & UI", () => {
    it("mounts into DOM with header, editor, gutter, security pane, and status bar", () => {
      const modal = new MacroEditorModal();
      modal.open();

      const modalEl = document.querySelector('[data-testid="macro-editor-modal"]');
      expect(modalEl).toBeTruthy();

      const codeInput = document.querySelector('[data-testid="macro-code-input"]') as HTMLTextAreaElement;
      const lineNumbers = document.querySelector('[data-testid="macro-line-numbers"]');
      const statusBar = document.querySelector('[data-testid="macro-status-bar"]');
      const riskBadge = document.querySelector('[data-testid="macro-risk-badge"]');

      expect(codeInput).toBeTruthy();
      expect(lineNumbers).toBeTruthy();
      expect(statusBar).toBeTruthy();
      expect(riskBadge).toBeTruthy();
      expect(riskBadge?.textContent).toBe("Safe");

      modal.close();
      expect(document.querySelector('[data-testid="macro-editor-modal"]')).toBeNull();
    });

    it("updates line numbers gutter dynamically when lines are added", () => {
      const modal = new MacroEditorModal({ initialCode: "Sub Test()\nEnd Sub" });
      modal.open();

      const lineNumbers = document.querySelector('[data-testid="macro-line-numbers"]');
      expect(lineNumbers?.textContent).toContain("1\n2\n");

      modal.setCode("Line1\nLine2\nLine3\nLine4\nLine5");
      expect(lineNumbers?.textContent).toContain("1\n2\n3\n4\n5\n");

      modal.close();
    });

    it("handles toolbar Step Into, Reset, and Close buttons", () => {
      const modal = new MacroEditorModal();
      modal.open();

      const statusBar = document.querySelector('[data-testid="macro-status-bar"]') as HTMLElement;
      const stepBtn = document.querySelector('[data-testid="macro-btn-step"]') as HTMLButtonElement;
      const resetBtn = document.querySelector('[data-testid="macro-btn-reset"]') as HTMLButtonElement;
      const closeBtn = document.querySelector('[data-testid="macro-modal-close"]') as HTMLButtonElement;

      stepBtn.click();
      expect(statusBar.textContent).toContain("Single-step execution paused");

      resetBtn.click();
      expect(statusBar.textContent).toContain("Execution halted. State reset");

      closeBtn.click();
      expect(document.querySelector('[data-testid="macro-editor-modal"]')).toBeNull();
    });
  });

  describe("3. Static Security Analysis & Sandbox Gatekeeping in UI", () => {
    it("flags dangerous Shell execution as Critical and updates UI badge", () => {
      const modal = new MacroEditorModal();
      modal.open();

      modal.setCode("Sub Exploit()\n    Shell \"cmd.exe /c calc.exe\"\nEnd Sub");

      const riskBadge = document.querySelector('[data-testid="macro-risk-badge"]');
      const findingsList = document.querySelector('[data-testid="macro-findings-list"]');

      expect(riskBadge?.textContent).toBe("Critical");
      expect(findingsList?.textContent).toContain("Dangerous invocation of system Shell command");

      modal.close();
    });

    it("flags WScript.Shell and MSXML2.XMLHTTP COM objects", () => {
      const modal = new MacroEditorModal();
      modal.open();

      modal.setCode(`
        Sub NetworkAndShell()
            Set wsh = CreateObject("WScript.Shell")
            Set http = CreateObject("MSXML2.XMLHTTP")
        End Sub
      `);

      const riskBadge = document.querySelector('[data-testid="macro-risk-badge"]');
      const findingsList = document.querySelector('[data-testid="macro-findings-list"]');

      expect(riskBadge?.textContent).toBe("Critical");
      expect(findingsList?.textContent).toContain("WScript.Shell");
      expect(findingsList?.textContent).toContain("network connection");

      modal.close();
    });

    it("blocks execution of Critical code when SystemCommand capability is not granted", async () => {
      const modal = new MacroEditorModal();
      modal.open();

      modal.setCode("Sub Pwn()\n    Shell \"calc.exe\"\nEnd Sub");

      const runBtn = document.querySelector('[data-testid="macro-btn-run"]') as HTMLButtonElement;
      const statusBar = document.querySelector('[data-testid="macro-status-bar"]') as HTMLElement;

      await runBtn.click();

      expect(statusBar.textContent).toContain("Execution blocked: SystemCommand capability denied");

      modal.close();
    });

    it("permits execution when capability checkbox is toggled", async () => {
      const modal = new MacroEditorModal();
      modal.open();

      modal.setCode("Sub Pwn()\n    Shell \"calc.exe\"\nEnd Sub");
      modal.setPermission("SystemCommand", true);

      const runBtn = document.querySelector('[data-testid="macro-btn-run"]') as HTMLButtonElement;
      const statusBar = document.querySelector('[data-testid="macro-status-bar"]') as HTMLElement;

      await runBtn.click();

      expect(statusBar.textContent).toContain("Completed successfully");
      expect(statusBar.textContent).toContain("Gas consumed");

      modal.close();
    });

    it("invokes custom onExecute handler with granted capabilities", async () => {
      const onExecute = vi.fn().mockResolvedValue({
        success: true,
        gas_consumed: 42,
        return_value: "100",
      });

      const modal = new MacroEditorModal({
        initialCode: "Sub MyCode()\nEnd Sub",
        routineName: "MyCode",
        onExecute,
      });
      modal.open();

      modal.setPermission("FileIo", true);
      modal.setPermission("Network", true);

      const runBtn = document.querySelector('[data-testid="macro-btn-run"]') as HTMLButtonElement;
      const statusBar = document.querySelector('[data-testid="macro-status-bar"]') as HTMLElement;

      await runBtn.click();

      expect(onExecute).toHaveBeenCalledWith(
        "Sub MyCode()\nEnd Sub",
        "MyCode",
        ["FileIo", "Network"]
      );
      expect(statusBar.textContent).toContain("Completed successfully. Gas consumed: 42");

      modal.close();
    });
  });
});
