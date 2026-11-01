/**
 * CrossDocDependencyCoordinator.ts
 * GEC-v7.0 Stage 50 (Sub-Phase 50.1.1)
 *
 * Presentation Shell Coordinator for:
 * - Cross-document Dependency Graph (DAG) management
 * - Real-time live embed synchronization (Calc cells in Word, charts in Slides, PDF regions in Notes)
 * - Topological invalidation propagation and staleness badge tracking
 * - Offline / suspended tab snapshot rendering (`LiveEmbedSnapshot`)
 * - Cycle detection and circular link prevention
 */

import {
  IPC,
  type CrossDocSummaryDto,
  type DependencyEdge,
  type DependencyKind,
  type EmbedAnchor,
  type EmbedTarget,
  type LiveEmbedSnapshot,
} from "../ipc";

export {
  type CrossDocSummaryDto,
  type DependencyEdge,
  type DependencyKind,
  type EmbedAnchor,
  type EmbedTarget,
  type LiveEmbedSnapshot,
};

export interface CrossDocState {
  edges: DependencyEdge[];
  snapshots: Record<string, LiveEmbedSnapshot>;
  staleEmbedIds: string[];
  summary: CrossDocSummaryDto;
  selectedEdgeId: string | null;
  activeDocId: string | null;
  activeDocIncoming: DependencyEdge[];
  activeDocOutgoing: DependencyEdge[];
  isLoading: boolean;
  error: string | null;
}

export type CrossDocListener = (state: Readonly<CrossDocState>) => void;

export class CrossDocDependencyCoordinator {
  private state: CrossDocState = {
    edges: [],
    snapshots: {},
    staleEmbedIds: [],
    summary: {
      total_nodes: 0,
      total_edges: 0,
      total_snapshots: 0,
      stale_snapshots: 0,
    },
    selectedEdgeId: null,
    activeDocId: null,
    activeDocIncoming: [],
    activeDocOutgoing: [],
    isLoading: false,
    error: null,
  };

  private listeners: Set<CrossDocListener> = new Set();

  constructor() {}

  public getState(): Readonly<CrossDocState> {
    return this.state;
  }

  public subscribe(listener: CrossDocListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    const currentState = this.getState();
    this.listeners.forEach((listener) => {
      try {
        listener(currentState);
      } catch (err) {
        console.error("Error in CrossDocDependencyCoordinator listener:", err);
      }
    });
  }

  public setActiveDocument(docId: string | null): void {
    this.state.activeDocId = docId;
    if (docId) {
      this.state.activeDocIncoming = this.state.edges.filter((e) => e.target_doc_id === docId);
      this.state.activeDocOutgoing = this.state.edges.filter((e) => e.source_doc_id === docId);
    } else {
      this.state.activeDocIncoming = [];
      this.state.activeDocOutgoing = [];
    }
    this.notify();
  }

  public selectEdge(edgeId: string | null): void {
    this.state.selectedEdgeId = edgeId;
    this.notify();
  }

  /**
   * Registers a new cross-document dependency edge.
   * Throws if a circular dependency is detected.
   */
  public async registerDependency(edge: DependencyEdge): Promise<string> {
    this.state.isLoading = true;
    this.state.error = null;
    this.notify();

    try {
      const edgeId = await IPC.depGraphRegisterDependency(edge);
      // Update local edge list
      const existingIdx = this.state.edges.findIndex((e) => e.id === edgeId);
      if (existingIdx >= 0) {
        this.state.edges[existingIdx] = edge;
      } else {
        this.state.edges.push(edge);
      }

      await this.refreshSummary();
      if (this.state.activeDocId) {
        this.setActiveDocument(this.state.activeDocId);
      }
      return edgeId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    } finally {
      this.state.isLoading = false;
      this.notify();
    }
  }

  /**
   * Removes an existing dependency edge by ID.
   */
  public async removeDependency(edgeId: string): Promise<boolean> {
    this.state.isLoading = true;
    this.state.error = null;
    this.notify();

    try {
      const removed = await IPC.depGraphRemoveDependency(edgeId);
      if (removed) {
        this.state.edges = this.state.edges.filter((e) => e.id !== edgeId);
        delete this.state.snapshots[edgeId];
        this.state.staleEmbedIds = this.state.staleEmbedIds.filter((id) => id !== edgeId);
        if (this.state.selectedEdgeId === edgeId) {
          this.state.selectedEdgeId = null;
        }
        await this.refreshSummary();
        if (this.state.activeDocId) {
          this.setActiveDocument(this.state.activeDocId);
        }
      }
      return removed;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    } finally {
      this.state.isLoading = false;
      this.notify();
    }
  }

  /**
   * Propagates invalidation from a source document along all dependent paths.
   */
  public async propagateInvalidation(
    sourceDocId: string,
    affectedAnchor?: EmbedAnchor
  ): Promise<string[]> {
    this.state.isLoading = true;
    this.state.error = null;
    this.notify();

    try {
      const affectedDocs = await IPC.depGraphPropagateInvalidation(
        sourceDocId,
        affectedAnchor
      );

      // Mark matching snapshots as stale locally
      for (const edge of this.state.edges) {
        if (edge.source_doc_id === sourceDocId) {
          const anchorMatches =
            !affectedAnchor || JSON.stringify(edge.source_anchor) === JSON.stringify(affectedAnchor);
          if (anchorMatches) {
            if (!this.state.staleEmbedIds.includes(edge.id)) {
              this.state.staleEmbedIds.push(edge.id);
            }
            if (this.state.snapshots[edge.id]) {
              this.state.snapshots[edge.id].is_stale = true;
            }
          }
        }
      }

      await this.refreshSummary();
      return affectedDocs;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    } finally {
      this.state.isLoading = false;
      this.notify();
    }
  }

  /**
   * Caches or updates a live embed snapshot.
   */
  public async updateSnapshot(snapshot: LiveEmbedSnapshot): Promise<void> {
    try {
      await IPC.depGraphUpdateSnapshot(snapshot);
      this.state.snapshots[snapshot.embed_id] = snapshot;
      if (!snapshot.is_stale) {
        this.state.staleEmbedIds = this.state.staleEmbedIds.filter((id) => id !== snapshot.embed_id);
      } else if (!this.state.staleEmbedIds.includes(snapshot.embed_id)) {
        this.state.staleEmbedIds.push(snapshot.embed_id);
      }
      await this.refreshSummary();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    }
  }

  /**
   * Retrieves a live embed snapshot by ID.
   */
  public async getSnapshot(embedId: string): Promise<LiveEmbedSnapshot | null> {
    if (this.state.snapshots[embedId]) {
      return this.state.snapshots[embedId];
    }
    try {
      const snap = await IPC.depGraphGetSnapshot(embedId);
      if (snap) {
        this.state.snapshots[embedId] = snap;
        if (snap.is_stale && !this.state.staleEmbedIds.includes(embedId)) {
          this.state.staleEmbedIds.push(embedId);
        }
        this.notify();
      }
      return snap;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      return null;
    }
  }

  /**
   * Refreshes the high-level dependency graph summary.
   */
  public async refreshSummary(): Promise<CrossDocSummaryDto> {
    try {
      const summary = await IPC.depGraphSummary();
      this.state.summary = summary;
      this.notify();
      return summary;
    } catch (err) {
      console.warn("Failed to refresh dependency graph summary:", err);
      return this.state.summary;
    }
  }

  public isEmbedStale(embedId: string): boolean {
    return this.state.staleEmbedIds.includes(embedId);
  }

  public markEmbedStale(embedId: string): void {
    if (!this.state.staleEmbedIds.includes(embedId)) {
      this.state.staleEmbedIds.push(embedId);
      if (this.state.snapshots[embedId]) {
        this.state.snapshots[embedId].is_stale = true;
      }
      this.state.summary.stale_snapshots = this.state.staleEmbedIds.length;
      this.notify();
    }
  }

  public clear(): void {
    this.state = {
      edges: [],
      snapshots: {},
      staleEmbedIds: [],
      summary: {
        total_nodes: 0,
        total_edges: 0,
        total_snapshots: 0,
        stale_snapshots: 0,
      },
      selectedEdgeId: null,
      activeDocId: null,
      activeDocIncoming: [],
      activeDocOutgoing: [],
      isLoading: false,
      error: null,
    };
    this.notify();
  }
}

export const crossDocCoordinator = new CrossDocDependencyCoordinator();

