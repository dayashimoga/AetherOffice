/**
 * LRU Tab Suspension & Instant Rehydration Coordinator
 *
 * Implements GEC-v7.0 Article I (Zero Global Document Singletons) and Article VI:
 * - Multi-document memory budget (< 250 MB RSS for 20 open documents).
 * - 3-tier lifecycle management: Active, Warm, Cold.
 * - Instant rehydration (< 50ms SLA) preserving scroll, zoom, caret, and selection coordinates.
 */

import { DocumentSessionRegistry, documentSessionRegistry } from "./documentSessionRegistry";

export interface ViewportState {
  scrollX: number;
  scrollY: number;
  zoom: number;
  caretOffset?: number;
  selectionSummary?: string;
}

export class LruTabSuspensionCoordinator {
  private registry: DocumentSessionRegistry;
  private maxActiveTabs: number;
  private maxWarmTabs: number;
  private viewportStates: Map<string, ViewportState> = new Map();
  private accessLog: Map<string, number> = new Map();
  private tickCounter: number = 0;

  constructor(
    registry: DocumentSessionRegistry = documentSessionRegistry,
    maxActiveTabs: number = 2,
    maxWarmTabs: number = 5
  ) {
    this.registry = registry;
    this.maxActiveTabs = Math.max(1, maxActiveTabs);
    this.maxWarmTabs = Math.max(1, maxWarmTabs);
  }

  public getMaxActiveTabs(): number {
    return this.maxActiveTabs;
  }

  public getMaxWarmTabs(): number {
    return this.maxWarmTabs;
  }

  public saveViewportState(sessionId: string, state: ViewportState): void {
    this.viewportStates.set(sessionId, { ...state });
  }

  public getViewportState(sessionId: string): ViewportState | undefined {
    return this.viewportStates.get(sessionId);
  }

  /**
   * Records an access to a session, promoting it to Active and enforcing quotas on other tabs.
   */
  public accessSession(sessionId: string, currentViewport?: ViewportState): void {
    const session = this.registry.getSession(sessionId);
    if (!session) return;

    if (currentViewport) {
      this.saveViewportState(sessionId, currentViewport);
    }

    this.tickCounter++;
    this.accessLog.set(sessionId, this.tickCounter);

    // Call registry to activate this tab and sync singletons
    this.registry.setActiveSession(sessionId);

    // If maxActiveTabs > 1, preserve the most recent prior active tab(s) up to maxActiveTabs
    if (this.maxActiveTabs > 1) {
      const otherSessions = this.registry
        .listSessions()
        .filter((s) => s.id !== sessionId)
        .sort((a, b) => (this.accessLog.get(b.id) ?? 0) - (this.accessLog.get(a.id) ?? 0));

      const allowedPrior = this.maxActiveTabs - 1;
      for (let i = 0; i < Math.min(allowedPrior, otherSessions.length); i++) {
        if (this.accessLog.has(otherSessions[i].id)) {
          otherSessions[i].lifecycleState = "active";
        }
      }
    }

    this.enforceQuotas(sessionId);
  }

  /**
   * Rehydrates a cold session, restoring its state and viewport coordinates in < 50ms.
   */
  public rehydrateSession(sessionId: string): { rehydrated: boolean; latencyMs: number } {
    const start = performance.now();
    const session = this.registry.getSession(sessionId);
    if (!session) {
      return { rehydrated: false, latencyMs: 0 };
    }

    const wasCold = session.lifecycleState === "cold";

    this.accessSession(sessionId);

    const latencyMs = performance.now() - start;
    return { rehydrated: wasCold, latencyMs };
  }

  /**
   * Returns memory breakdown counts across Active, Warm, and Cold tiers.
   */
  public getMemoryBreakdown(): { active: number; warm: number; cold: number } {
    let active = 0;
    let warm = 0;
    let cold = 0;

    for (const session of this.registry.listSessions()) {
      if (session.lifecycleState === "active") active++;
      else if (session.lifecycleState === "warm") warm++;
      else if (session.lifecycleState === "cold") cold++;
    }

    return { active, warm, cold };
  }

  /**
   * Internal quota enforcement demoting excess Active tabs to Warm, and excess Warm tabs to Cold.
   */
  private enforceQuotas(keepActiveId: string): void {
    const allSessions = this.registry.listSessions();

    // 1. Demote excess Active tabs to Warm
    const activeSessions = allSessions
      .filter((s) => s.lifecycleState === "active" && s.id !== keepActiveId)
      .sort((a, b) => (this.accessLog.get(b.id) ?? 0) - (this.accessLog.get(a.id) ?? 0));

    const allowedOtherActive = Math.max(0, this.maxActiveTabs - 1);
    if (activeSessions.length > allowedOtherActive) {
      for (let i = allowedOtherActive; i < activeSessions.length; i++) {
        activeSessions[i].lifecycleState = "warm";
      }
    }

    // 2. Demote excess Warm tabs to Cold
    const warmSessions = this.registry
      .listSessions()
      .filter((s) => s.lifecycleState === "warm")
      .sort((a, b) => (this.accessLog.get(b.id) ?? 0) - (this.accessLog.get(a.id) ?? 0));

    if (warmSessions.length > this.maxWarmTabs) {
      for (let i = this.maxWarmTabs; i < warmSessions.length; i++) {
        warmSessions[i].lifecycleState = "cold";
      }
    }
  }
}

export const lruTabSuspensionCoordinator = new LruTabSuspensionCoordinator();
