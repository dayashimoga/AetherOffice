/**
 * plugin_manager_coordinator.test.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.2.1)
 *
 * Vitest Unit & Integration Suite for PluginManagerCoordinator:
 * - Default state initialization and listener subscription
 * - Loading sandboxed plugins from JSON manifest
 * - Refreshing and listing installed plugins
 * - Initializing plugins, verifying lifecycle transition to Active and fuel deduction
 * - Suspending and resuming plugin instances
 * - Refilling instruction fuel meters to prevent gas exhaustion
 * - Dispatched host document actions (queries and mutations)
 * - Error isolation when executing actions on non-active plugins
 * - Registering custom spreadsheet calculation UDFs
 * - Evaluating custom UDFs with dynamic arguments
 */

import { describe, it, expect, beforeEach } from "vitest";
import { PluginManagerCoordinator } from "../state/PluginManagerCoordinator";

describe("Sub-Phase 51.2.1: PluginManagerCoordinator", () => {
  let coordinator: PluginManagerCoordinator;

  beforeEach(() => {
    coordinator = new PluginManagerCoordinator();
  });

  it("1. initializes with empty plugins, udfs, and null selection", () => {
    const state = coordinator.getState();
    expect(state.plugins).toEqual([]);
    expect(state.registeredUdfs).toEqual([]);
    expect(state.selectedPluginId).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.errorMessage).toBeNull();
    expect(state.lastActionExecution).toBeNull();
    expect(state.lastUdfCall).toBeNull();
  });

  it("2. loads a new plugin from manifest json and sets it as selected", async () => {
    const manifest = JSON.stringify({
      id: "org.aetheroffice.sample-ext",
      name: "Sample Extension",
      version: "1.0.0",
      author: "Test Author",
      description: "A test WebAssembly plugin",
      capabilities: ["read-document", "calc-udf"],
      fuel_limit: 8000,
    });

    const status = await coordinator.loadPlugin(manifest);
    expect(status.id).toBe("org.aetheroffice.sample-ext");
    expect(status.name).toBe("Sample Extension");
    expect(status.state).toBe("Loaded");
    expect(status.fuel_remaining).toBe(8000);

    const state = coordinator.getState();
    expect(state.plugins.length).toBe(1);
    expect(state.selectedPluginId).toBe("org.aetheroffice.sample-ext");
    expect(coordinator.getPlugin("org.aetheroffice.sample-ext")).toBeDefined();
  });

  it("3. lists and refreshes installed plugins", async () => {
    const plugins = await coordinator.refreshPlugins();
    expect(Array.isArray(plugins)).toBe(true);
    expect(coordinator.getState().isLoading).toBe(false);
  });

  it("4. initializes a plugin, advancing state to Active and consuming init fuel", async () => {
    const pluginId = "org.aetheroffice.sample-ext";
    const status = await coordinator.initPlugin(pluginId);

    expect(status.id).toBe(pluginId);
    expect(status.state).toBe("Active");
    expect(status.dispatched_events).toBeGreaterThan(0);
    expect(status.fuel_remaining).toBeLessThan(8000);
  });

  it("5. suspends an active plugin and resumes it back to active", async () => {
    const pluginId = "org.aetheroffice.sample-ext";

    const suspended = await coordinator.suspendPlugin(pluginId);
    expect(suspended.state).toBe("Suspended");
    expect(coordinator.getPlugin(pluginId)?.state).toBe("Suspended");

    const resumed = await coordinator.resumePlugin(pluginId);
    expect(resumed.state).toBe("Active");
    expect(coordinator.getPlugin(pluginId)?.state).toBe("Active");
  });

  it("6. refills instruction fuel for a plugin", async () => {
    const pluginId = "org.aetheroffice.sample-ext";
    await coordinator.refreshPlugins();
    const oldFuel = coordinator.getPlugin(pluginId)?.fuel_remaining || 0;

    const newFuel = await coordinator.refillFuel(pluginId, 2500);
    expect(newFuel).toBe(oldFuel + 2500);
    expect(coordinator.getPlugin(pluginId)?.fuel_remaining).toBe(newFuel);
  });

  it("7. executes host action (QuerySelection and SetCellValue) on active plugin", async () => {
    const pluginId = "org.aetheroffice.sample-ext";

    const selRes = await coordinator.executeAction(pluginId, {
      QuerySelection: { doc_id: "doc-sample-1" },
    });
    expect(typeof selRes === "object" && selRes !== null).toBe(true);
    if (typeof selRes === "object" && selRes !== null) {
      expect("Selection" in selRes).toBe(true);
    }

    const mutRes = await coordinator.executeAction(pluginId, {
      SetCellValue: {
        coord: { sheet: "Sheet1", row: 1, col: 1 },
        value: "Custom computed value",
      },
    });
    expect(mutRes).toBe("MutationSuccess");

    const state = coordinator.getState();
    expect(state.lastActionExecution).not.toBeNull();
    expect(state.lastActionExecution?.pluginId).toBe(pluginId);
  });

  it("8. throws error when executing action on suspended plugin", async () => {
    const pluginId = "org.aetheroffice.sample-ext";
    await coordinator.suspendPlugin(pluginId);

    await expect(
      coordinator.executeAction(pluginId, {
        QueryCellValue: { coord: { sheet: "Sheet1", row: 1, col: 1 } },
      })
    ).rejects.toThrow();

    expect(coordinator.getState().errorMessage).not.toBeNull();

    // Re-activate for subsequent tests
    await coordinator.resumePlugin(pluginId);
    coordinator.clearError();
    expect(coordinator.getState().errorMessage).toBeNull();
  });

  it("9. registers custom calculation UDF and lists registered UDFs", async () => {
    const registered = await coordinator.registerMockUdf({
      name: "GEO_DISTANCE",
      description: "Computes geodesic distance between two coordinate pairs",
      min_args: 4,
      max_args: 4,
    });
    expect(registered).toBe(true);

    const udfs = await coordinator.refreshUdfs();
    expect(udfs.some((u) => u.name === "GEO_DISTANCE")).toBe(true);

    const udf = coordinator.getUdf("geo_distance");
    expect(udf).toBeDefined();
    expect(udf?.min_args).toBe(4);
  });

  it("10. calls custom calculation UDF and verifies evaluated result", async () => {
    const res = await coordinator.callUdf("GEO_DISTANCE", ["40.7128", "-74.0060", "51.5074", "-0.1278"]);
    expect(res.success).toBe(true);
    expect(res.function_name).toBe("GEO_DISTANCE");
    expect(res.result_value).toBe("415.82");
    expect(res.error_message).toBeNull();

    const state = coordinator.getState();
    expect(state.lastUdfCall).toEqual(res);
  });
});
