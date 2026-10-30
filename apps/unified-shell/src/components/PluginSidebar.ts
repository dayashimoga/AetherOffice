import { pluginHost, PluginHost, PluginSidebarDefinition } from "../plugins/pluginHost";

export interface PluginIframeMessage {
  type: string;
  payload?: unknown;
}

/**
 * Sandboxed sidebar pane renderer hosting third-party plugin webviews in an isolated iframe.
 */
export class PluginSidebar {
  private container: HTMLElement;
  private currentPane: PluginSidebarDefinition | null = null;
  private iframeEl: HTMLIFrameElement | null = null;
  private messageHandler?: (event: MessageEvent) => void;

  constructor(
    containerId = "sidebar-container",
    private host: PluginHost = pluginHost
  ) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`Sidebar container #${containerId} not found`);
    this.container = el;
  }

  public isVisible(): boolean {
    return this.currentPane !== null;
  }

  public getActivePaneId(): string | null {
    return this.currentPane ? this.currentPane.id : null;
  }

  /**
   * Render and open the specified plugin sidebar pane.
   */
  public show(pane: PluginSidebarDefinition): void {
    this.currentPane = pane;
    this.container.classList.remove("collapsed");
    this.container.style.display = "flex";
    if (pane.width) {
      this.container.style.width = `${pane.width}px`;
    }

    this.container.innerHTML = "";

    // 1. Sidebar Header
    const header = document.createElement("div");
    header.className = "plugin-sidebar-header";
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.padding = "8px 12px";
    header.style.borderBottom = "1px solid var(--border-color, #e0e0e0)";
    header.style.backgroundColor = "var(--bg-secondary, #f8f9fa)";

    const titleContainer = document.createElement("div");
    titleContainer.style.display = "flex";
    titleContainer.style.alignItems = "center";
    titleContainer.style.gap = "8px";

    const iconSpan = document.createElement("span");
    iconSpan.className = "plugin-sidebar-icon";
    iconSpan.textContent = pane.icon || "🧩";
    titleContainer.appendChild(iconSpan);

    const titleEl = document.createElement("span");
    titleEl.className = "plugin-sidebar-title";
    titleEl.style.fontWeight = "600";
    titleEl.textContent = pane.title;
    titleContainer.appendChild(titleEl);

    header.appendChild(titleContainer);

    // Close button
    const closeBtn = document.createElement("button");
    closeBtn.className = "plugin-sidebar-close-btn";
    closeBtn.setAttribute("data-testid", "plugin-sidebar-close");
    closeBtn.textContent = "✕";
    closeBtn.title = "Close sidebar";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.background = "none";
    closeBtn.style.border = "none";
    closeBtn.style.fontSize = "14px";
    closeBtn.onclick = () => {
      this.hide();
    };
    header.appendChild(closeBtn);

    this.container.appendChild(header);

    // 2. Sandboxed Iframe Container
    const body = document.createElement("div");
    body.className = "plugin-sidebar-body";
    body.style.flex = "1";
    body.style.display = "flex";
    body.style.overflow = "hidden";

    const iframe = document.createElement("iframe");
    iframe.className = "plugin-sidebar-iframe";
    iframe.setAttribute("data-testid", "plugin-sidebar-iframe");
    iframe.setAttribute(
      "sandbox",
      pane.sandboxOptions || "allow-scripts allow-forms"
    );
    iframe.style.width = "100%";
    iframe.style.height = "100%";
    iframe.style.border = "none";

    if (pane.entrypointHtml) {
      iframe.srcdoc = pane.entrypointHtml;
    } else if (pane.entrypointUrl) {
      iframe.src = pane.entrypointUrl;
    } else {
      iframe.srcdoc = `<!DOCTYPE html><html><body><p>Plugin ${pane.title} loaded.</p></body></html>`;
    }

    this.iframeEl = iframe;
    body.appendChild(iframe);
    this.container.appendChild(body);

    // 3. PostMessage bridge setup
    this.setupMessageBridge();
  }

  /**
   * Hide and tear down the active plugin sidebar pane.
   */
  public hide(): void {
    if (!this.currentPane && !this.iframeEl) {
      return;
    }
    if (this.messageHandler) {
      window.removeEventListener("message", this.messageHandler);
      this.messageHandler = undefined;
    }
    this.currentPane = null;
    this.iframeEl = null;
    this.container.innerHTML = "";
    this.container.classList.add("collapsed");
    this.container.style.display = "none";
    this.host.setActiveSidebarPane(null);
  }

  /**
   * Post a structured message to the running plugin iframe.
   */
  public postMessageToPlugin(message: unknown): void {
    if (this.iframeEl && this.iframeEl.contentWindow) {
      this.iframeEl.contentWindow.postMessage(message, "*");
    }
  }

  private setupMessageBridge(): void {
    if (this.messageHandler) {
      window.removeEventListener("message", this.messageHandler);
    }

    this.messageHandler = (event: MessageEvent) => {
      // Allow messages if they originate from our iframe
      if (
        this.iframeEl &&
        event.source === this.iframeEl.contentWindow &&
        event.data
      ) {
        const data = event.data as PluginIframeMessage;
        if (data.type) {
          this.host.dispatchAction(`sidebar:${data.type}`, data.payload);
        }
      }
    };

    window.addEventListener("message", this.messageHandler);
  }
}
