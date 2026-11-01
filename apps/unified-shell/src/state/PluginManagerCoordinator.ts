/**
 * PluginManagerCoordinator.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.2.1)
 *
 * Presentation Shell Coordinator for:
 * - WebAssembly Plugin Runtime lifecycle management (load, unload, init, suspend, resume)
 * - Gas metering, fuel replenishment, and security isolation
 * - Host capability and document action dispatching (Word, Calc, Slides)
 * - Dynamic custom calculation User-Defined Functions (UDF) registration and evaluation
 */

import {
  IPC,
  type ActionResultDto,
  type PluginActionDto,
  type PluginCellCoordDto,
  type PluginSelectionInfoDto,
  type PluginStatusDto,
  type PluginUdfCallResultDto,
  type PluginUdfMetaDto,
} from "../ipc";

export {
  type ActionResultDto,
  type PluginActionDto,
  type PluginCellCoordDto,
  type PluginSelectionInfoDto,
  type PluginStatusDto,
  type PluginUdfCallResultDto,
  type PluginUdfMetaDto,
};

export interface PluginExecutionRecord {
  pluginId: string;
  action: PluginActionDto;
  result: ActionResultDto;
  timestampMs: number;
}

export interface PluginManagerState {
  plugins: PluginStatusDto[];
  registeredUdfs: PluginUdfMetaDto[];
  selectedPluginId: string | null;
  isLoading: boolean;
  errorMessage: string | null;
  lastActionExecution: PluginExecutionRecord | null;
  lastUdfCall: PluginUdfCallResultDto | null;
}

export type PluginManagerListener = (state: Readonly<PluginManagerState>) => void;

export class PluginManagerCoordinator {
  private state: PluginManagerState;
  private listeners = new Set<PluginManagerListener>();

  constructor() {
    this.state = {
      plugins: [],
      registeredUdfs: [],
      selectedPluginId: null,
      isLoading: false,
      errorMessage: null,
      lastActionExecution: null,
      lastUdfCall: null,
    };
  }

  /**
   * Returns an immutable snapshot of the current coordinator state.
   */
  public getState(): Readonly<PluginManagerState> {
    return this.state;
  }

  /**
   * Subscribes a listener to reactive state modifications.
   */
  public subscribe(listener: PluginManagerListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snap = Object.freeze({ ...this.state });
    for (const listener of this.listeners) {
      try {
        listener(snap);
      } catch (err) {
        console.error("PluginManagerCoordinator listener error:", err);
      }
    }
  }

  /**
   * Refreshes the list of all installed plugins from the backend runtime.
   */
  public async refreshPlugins(): Promise<PluginStatusDto[]> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const plugins = await IPC.pluginList();
      this.state.plugins = plugins;
      this.state.isLoading = false;
      this.notify();
      return plugins;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Refreshes the list of registered dynamic calculation UDFs.
   */
  public async refreshUdfs(): Promise<PluginUdfMetaDto[]> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const udfs = await IPC.pluginListUdfs();
      this.state.registeredUdfs = udfs;
      this.state.isLoading = false;
      this.notify();
      return udfs;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Refreshes both installed plugins and registered UDFs concurrently.
   */
  public async refreshAll(): Promise<void> {
    await Promise.all([this.refreshPlugins(), this.refreshUdfs()]);
  }

  /**
   * Loads and sandboxes a new plugin from a JSON manifest specification.
   */
  public async loadPlugin(manifestJson: string): Promise<PluginStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.pluginLoad(manifestJson);
      await this.refreshPlugins();
      this.state.selectedPluginId = status.id;
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Unloads and destroys a sandboxed plugin instance.
   */
  public async unloadPlugin(pluginId: string): Promise<boolean> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const success = await IPC.pluginUnload(pluginId);
      if (this.state.selectedPluginId === pluginId) {
        this.state.selectedPluginId = null;
      }
      await this.refreshPlugins();
      this.state.isLoading = false;
      this.notify();
      return success;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Initializes a loaded plugin, invoking its `on_init` lifecycle hook.
   */
  public async initPlugin(pluginId: string): Promise<PluginStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.pluginInit(pluginId);
      await this.refreshPlugins();
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Suspends an active plugin instance, preventing hook executions.
   */
  public async suspendPlugin(pluginId: string): Promise<PluginStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.pluginSuspend(pluginId);
      await this.refreshPlugins();
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Resumes a suspended plugin instance back into active execution.
   */
  public async resumePlugin(pluginId: string): Promise<PluginStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.pluginResume(pluginId);
      await this.refreshPlugins();
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Replenishes the instruction fuel meter of a plugin to prevent gas exhaustion.
   */
  public async refillFuel(pluginId: string, additionalFuel: number): Promise<number> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const remaining = await IPC.pluginRefillFuel(pluginId, additionalFuel);
      await this.refreshPlugins();
      this.state.isLoading = false;
      this.notify();
      return remaining;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Dispatches a host mutation or query action on behalf of an active plugin.
   */
  public async executeAction(
    pluginId: string,
    action: PluginActionDto
  ): Promise<ActionResultDto> {
    this.state.errorMessage = null;

    try {
      const result = await IPC.pluginExecuteAction(pluginId, action);
      this.state.lastActionExecution = {
        pluginId,
        action,
        result,
        timestampMs: Date.now(),
      };
      await this.refreshPlugins();
      this.notify();
      return result;
    } catch (err: unknown) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Registers a new custom spreadsheet calculation function (UDF).
   */
  public async registerMockUdf(meta: PluginUdfMetaDto): Promise<boolean> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const success = await IPC.pluginRegisterMockUdf(meta);
      await this.refreshUdfs();
      this.state.isLoading = false;
      this.notify();
      return success;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Invokes a registered custom calculation function with string arguments.
   */
  public async callUdf(
    funcName: string,
    args: string[]
  ): Promise<PluginUdfCallResultDto> {
    this.state.errorMessage = null;

    try {
      const result = await IPC.pluginCallUdf(funcName, args);
      this.state.lastUdfCall = result;
      this.notify();
      return result;
    } catch (err: unknown) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Selects a plugin for detailed inspect pane view.
   */
  public selectPlugin(pluginId: string | null): void {
    this.state.selectedPluginId = pluginId;
    this.notify();
  }

  /**
   * Finds a plugin in the active local state by id.
   */
  public getPlugin(pluginId: string): PluginStatusDto | undefined {
    return this.state.plugins.find((p) => p.id === pluginId);
  }

  /**
   * Finds a registered UDF by case-insensitive name.
   */
  public getUdf(funcName: string): PluginUdfMetaDto | undefined {
    const target = funcName.toUpperCase();
    return this.state.registeredUdfs.find((u) => u.name.toUpperCase() === target);
  }

  /**
   * Clears any active error banner message.
   */
  public clearError(): void {
    this.state.errorMessage = null;
    this.notify();
  }
}

