import { DocumentType } from "../../ipc";
import { pluginHost, PluginHost } from "../../plugins/pluginHost";
import {
  AppRibbonComponent,
  createRibbonButton,
  createRibbonGroup,
  RibbonActionEmitter,
} from "./types";

/**
 * ContextualRibbonRouter resolves active tabs and merges standard application
 * ribbon toolbars with dynamically injected plugin tabs and groups.
 */
export class ContextualRibbonRouter {
  constructor(private host: PluginHost = pluginHost) {}

  /**
   * Return the merged list of standard tabs and active plugin tabs for the document type.
   */
  public getVisibleTabs(docType: DocumentType, standardTabs: string[]): string[] {
    const injected = this.host.getInjectedTabs(docType).map((t) => t.label);
    return [...standardTabs, ...injected];
  }

  /**
   * Resolve keytip character for a tab (standard or plugin-injected).
   */
  public getKeytip(
    docType: DocumentType,
    tab: string,
    standardRibbon: AppRibbonComponent
  ): string {
    const injectedTab = this.host
      .getInjectedTabs(docType)
      .find((t) => t.label === tab);
    if (injectedTab?.keytip) {
      return injectedTab.keytip;
    }
    return standardRibbon.getKeytip(tab);
  }

  /**
   * Render either a plugin-injected custom tab or a standard tab with appended plugin groups.
   */
  public renderTab(
    docType: DocumentType,
    tab: string,
    body: HTMLElement,
    standardRibbon: AppRibbonComponent,
    emit: RibbonActionEmitter
  ): void {
    const injectedTab = this.host
      .getInjectedTabs(docType)
      .find((t) => t.label === tab);

    if (injectedTab) {
      // 1. Render third-party custom ribbon tab
      for (const grp of injectedTab.groups) {
        const groupEl = createRibbonGroup(grp.title);
        for (const btn of grp.buttons) {
          const btnEl = createRibbonButton(
            btn.label,
            btn.icon,
            () => {
              this.host.dispatchAction(btn.action);
              emit(btn.action);
            },
            btn.isToggle,
            btn.testId || `plugin-btn-${btn.id}`
          );
          if (btn.tooltip) {
            btnEl.title = btn.tooltip;
          }
          groupEl.appendChild(btnEl);
        }
        body.appendChild(groupEl);
      }
    } else {
      // 2. Render standard application tab
      standardRibbon.renderTab(tab, body, emit);

      // 3. Append any third-party groups injected into this standard tab
      const injectedGroups = this.host.getInjectedGroups(docType, tab);
      for (const grp of injectedGroups) {
        const groupEl = createRibbonGroup(grp.title);
        for (const btn of grp.buttons) {
          const btnEl = createRibbonButton(
            btn.label,
            btn.icon,
            () => {
              this.host.dispatchAction(btn.action);
              emit(btn.action);
            },
            btn.isToggle,
            btn.testId || `plugin-btn-${btn.id}`
          );
          if (btn.tooltip) {
            btnEl.title = btn.tooltip;
          }
          groupEl.appendChild(btnEl);
        }
        body.appendChild(groupEl);
      }
    }
  }
}

