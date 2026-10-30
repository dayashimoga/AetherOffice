import { DocumentType } from "../ipc";

export type PluginTargetApp = "Word" | "Excel" | "PowerPoint" | "PDF" | "All";

export interface PluginButtonDefinition {
  id: string;
  label: string;
  icon: string;
  action: string;
  tooltip?: string;
  isToggle?: boolean;
  testId?: string;
}

export interface PluginGroupDefinition {
  id: string;
  title: string;
  buttons: PluginButtonDefinition[];
}

export interface PluginTabDefinition {
  id: string;
  label: string;
  keytip?: string;
  groups: PluginGroupDefinition[];
}

export interface PluginInjectedGroupDefinition {
  targetTab: string;
  groups: PluginGroupDefinition[];
}

export interface PluginSidebarDefinition {
  id: string;
  title: string;
  icon: string;
  entrypointHtml?: string;
  entrypointUrl?: string;
  sandboxOptions?: string;
  width?: number;
}

export interface PluginShortcutDefinition {
  key: string;
  action: string;
  description?: string;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  targetApps?: PluginTargetApp[];
  ribbonContributions?: {
    tabs?: PluginTabDefinition[];
    injectedGroups?: PluginInjectedGroupDefinition[];
  };
  sidebarContributions?: PluginSidebarDefinition[];
  shortcuts?: PluginShortcutDefinition[];
  enabled?: boolean;
}

export type PluginActionListener = (action: string, payload?: unknown) => void;
export type PluginLifecycleListener = (event: string, pluginId: string, payload?: unknown) => void;

/**
 * Host manager responsible for plugin registration, manifest parsing,
 * Ribbon and sidebar UI contributions, and event dispatch.
 */
export class PluginHost {
  private plugins = new Map<string, PluginManifest>();
  private activeSidebarPaneId: string | null = null;
  private actionListeners = new Set<PluginActionListener>();
  private lifecycleListeners = new Set<PluginLifecycleListener>();

  /**
   * Parse and validate a raw plugin manifest JSON object or string.
   */
  public parseManifest(manifestOrJson: string | object): PluginManifest {
    let obj: unknown;
    if (typeof manifestOrJson === "string") {
      try {
        obj = JSON.parse(manifestOrJson);
      } catch (e) {
        throw new Error(`Failed to parse plugin JSON: ${(e as Error).message}`);
      }
    } else {
      obj = manifestOrJson;
    }

    if (!obj || typeof obj !== "object") {
      throw new Error("Invalid plugin manifest: root must be an object");
    }

    const m = obj as Partial<PluginManifest>;
    if (!m.id || typeof m.id !== "string" || m.id.trim() === "") {
      throw new Error("Invalid plugin manifest: missing required 'id'");
    }
    if (!m.name || typeof m.name !== "string" || m.name.trim() === "") {
      throw new Error("Invalid plugin manifest: missing required 'name'");
    }
    if (!m.version || typeof m.version !== "string" || m.version.trim() === "") {
      throw new Error("Invalid plugin manifest: missing required 'version'");
    }

    return {
      id: m.id.trim(),
      name: m.name.trim(),
      version: m.version.trim(),
      description: m.description,
      author: m.author,
      targetApps: m.targetApps || ["All"],
      ribbonContributions: m.ribbonContributions,
      sidebarContributions: m.sidebarContributions,
      shortcuts: m.shortcuts,
      enabled: m.enabled !== undefined ? m.enabled : true,
    };
  }

  /**
   * Register a new plugin in the host environment.
   */
  public registerPlugin(manifestOrJson: string | PluginManifest): PluginManifest {
    const manifest = this.parseManifest(manifestOrJson);
    this.plugins.set(manifest.id, manifest);
    this.emitLifecycle("plugin-registered", manifest.id, manifest);
    return manifest;
  }

  /**
   * Unregister an existing plugin.
   */
  public unregisterPlugin(id: string): boolean {
    if (this.plugins.has(id)) {
      if (this.activeSidebarPaneId) {
        const pane = this.getSidebarPanes().find((p) => p.id === this.activeSidebarPaneId);
        if (pane) {
          this.setActiveSidebarPane(null);
        }
      }
      this.plugins.delete(id);
      this.emitLifecycle("plugin-unregistered", id);
      return true;
    }
    return false;
  }

  /**
   * Retrieve a plugin manifest by ID.
   */
  public getPlugin(id: string): PluginManifest | undefined {
    return this.plugins.get(id);
  }

  /**
   * Get all registered plugins.
   */
  public getAllPlugins(): PluginManifest[] {
    return Array.from(this.plugins.values());
  }

  /**
   * Get all active (enabled) plugins applicable to the given document type.
   */
  public getActivePlugins(docType?: DocumentType): PluginManifest[] {
    return Array.from(this.plugins.values()).filter((p) => {
      if (p.enabled === false) return false;
      if (!docType) return true;
      const targets = p.targetApps || ["All"];
      return targets.includes("All") || targets.includes(docType as PluginTargetApp);
    });
  }

  /**
   * Enable a plugin by ID.
   */
  public enablePlugin(id: string): void {
    const plugin = this.plugins.get(id);
    if (plugin) {
      plugin.enabled = true;
      this.emitLifecycle("plugin-enabled", id);
    }
  }

  /**
   * Disable a plugin by ID.
   */
  public disablePlugin(id: string): void {
    const plugin = this.plugins.get(id);
    if (plugin) {
      plugin.enabled = false;
      this.emitLifecycle("plugin-disabled", id);
    }
  }

  /**
   * Check if a plugin is currently enabled.
   */
  public isPluginEnabled(id: string): boolean {
    const plugin = this.plugins.get(id);
    return plugin ? plugin.enabled !== false : false;
  }

  /**
   * Get all custom Ribbon tabs injected by active plugins for a document type.
   */
  public getInjectedTabs(docType: DocumentType): PluginTabDefinition[] {
    const active = this.getActivePlugins(docType);
    const tabs: PluginTabDefinition[] = [];
    for (const plugin of active) {
      if (plugin.ribbonContributions?.tabs) {
        tabs.push(...plugin.ribbonContributions.tabs);
      }
    }
    return tabs;
  }

  /**
   * Get all custom Ribbon groups injected into a standard tab (e.g. "Home").
   */
  public getInjectedGroups(docType: DocumentType, tabName: string): PluginGroupDefinition[] {
    const active = this.getActivePlugins(docType);
    const groups: PluginGroupDefinition[] = [];
    const normalizedTab = tabName.toLowerCase();

    for (const plugin of active) {
      if (plugin.ribbonContributions?.injectedGroups) {
        for (const inj of plugin.ribbonContributions.injectedGroups) {
          if (inj.targetTab.toLowerCase() === normalizedTab) {
            groups.push(...inj.groups);
          }
        }
      }
    }
    return groups;
  }

  /**
   * Get all sidebar panes registered by active plugins for a document type.
   */
  public getSidebarPanes(docType?: DocumentType): PluginSidebarDefinition[] {
    const active = this.getActivePlugins(docType);
    const sidebars: PluginSidebarDefinition[] = [];
    for (const plugin of active) {
      if (plugin.sidebarContributions) {
        sidebars.push(...plugin.sidebarContributions);
      }
    }
    return sidebars;
  }

  /**
   * Get currently active sidebar pane definition.
   */
  public getActiveSidebarPane(): PluginSidebarDefinition | null {
    if (!this.activeSidebarPaneId) return null;
    const all = this.getSidebarPanes();
    return all.find((p) => p.id === this.activeSidebarPaneId) || null;
  }

  /**
   * Set or toggle active sidebar pane.
   */
  public setActiveSidebarPane(paneId: string | null): void {
    if (this.activeSidebarPaneId === paneId) {
      return;
    }
    this.activeSidebarPaneId = paneId;
    this.emitLifecycle("sidebar-toggled", paneId || "none", { paneId });
  }

  /**
   * Dispatch an action on behalf of a plugin button or command.
   */
  public dispatchAction(action: string, payload?: unknown): void {
    for (const listener of this.actionListeners) {
      try {
        listener(action, payload);
      } catch (e) {
        console.error(`Error in plugin action listener for '${action}':`, e);
      }
    }
  }

  /**
   * Subscribe to dispatched plugin actions.
   */
  public onAction(listener: PluginActionListener): () => void {
    this.actionListeners.add(listener);
    return () => this.actionListeners.delete(listener);
  }

  /**
   * Subscribe to plugin lifecycle events.
   */
  public onLifecycle(listener: PluginLifecycleListener): () => void {
    this.lifecycleListeners.add(listener);
    return () => this.lifecycleListeners.delete(listener);
  }

  private emitLifecycle(event: string, pluginId: string, payload?: unknown): void {
    for (const listener of this.lifecycleListeners) {
      try {
        listener(event, pluginId, payload);
      } catch (e) {
        console.error(`Error in plugin lifecycle listener for '${event}':`, e);
      }
    }
  }

  /**
   * Reset all plugin states (useful for test isolation).
   */
  public clear(): void {
    this.plugins.clear();
    this.activeSidebarPaneId = null;
    this.actionListeners.clear();
    this.lifecycleListeners.clear();
  }
}

/** Global default plugin host singleton */
export const pluginHost = new PluginHost();
