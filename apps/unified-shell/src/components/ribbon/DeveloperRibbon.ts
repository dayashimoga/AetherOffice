import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
} from "./types";

export class DeveloperRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return ["Developer"];
  }

  public getKeytip(tab: string): string {
    return tab === "Developer" ? "L" : "H";
  }

  public renderTab(tab: string, body: HTMLElement, emit: RibbonActionEmitter): void {
    body.innerHTML = "";
    if (tab === "Developer") {
      this.renderDeveloperTab(body, emit);
    }
  }

  public renderDeveloperTab(body: HTMLElement, emit: RibbonActionEmitter): void {
    // 1. Code Group
    const codeGroup = createRibbonGroup("Code");

    const vbaBtn = createRibbonButton(
      "Visual Basic",
      "💻 Visual Basic",
      () => emit("OpenMacroEditor"),
      false,
      "btn-vba-editor"
    );

    const macrosBtn = createRibbonButton(
      "Macros",
      "⚡ Macros",
      () => emit("OpenMacrosDialog"),
      false,
      "btn-macros-dialog"
    );

    const recordBtn = createRibbonButton(
      "Record Macro",
      "🔴 Record Macro",
      () => emit("ToggleRecordMacro"),
      true,
      "btn-record-macro"
    );

    codeGroup.appendChild(vbaBtn);
    codeGroup.appendChild(macrosBtn);
    codeGroup.appendChild(recordBtn);
    body.appendChild(codeGroup);

    // 2. Add-ins Group
    const addinsGroup = createRibbonGroup("Add-ins");
    const addinsBtn = createRibbonButton(
      "Excel Add-ins",
      "🧩 Add-ins",
      () => emit("OpenAddinsDialog"),
      false,
      "btn-addins-dialog"
    );
    addinsGroup.appendChild(addinsBtn);
    body.appendChild(addinsGroup);

    // 3. Security Group
    const secGroup = createRibbonGroup("Security");
    const secBtn = createRibbonButton(
      "Macro Security",
      "🛡️ Macro Security",
      () => emit("OpenMacroSecurity"),
      false,
      "btn-macro-security"
    );
    secGroup.appendChild(secBtn);
    body.appendChild(secGroup);
  }
}

