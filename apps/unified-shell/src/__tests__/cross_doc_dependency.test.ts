import { describe, it, expect, beforeEach } from 'vitest';
import {
  CrossDocDependencyCoordinator,
  type DependencyEdge,
  type LiveEmbedSnapshot,
} from '../state/CrossDocDependencyCoordinator';

describe('CrossDocDependencyCoordinator - Cross-Document DAG & Live Embeds (Sub-Phase 50.1.1)', () => {
  let coordinator: CrossDocDependencyCoordinator;

  const mockEdge1: DependencyEdge = {
    id: 'edge-c1-w1',
    source_doc_id: 'calc-doc-1',
    source_anchor: {
      type: 'CalcRange',
      data: { sheet_name: 'Financials', range_a1: 'B2:E10' },
    },
    target_doc_id: 'word-doc-1',
    target_container: {
      type: 'WordInline',
      data: { container_id: 'table-container-1' },
    },
    kind: 'LiveRangeEmbed',
    created_at: 1000,
    version: 1,
  };

  const mockEdge2: DependencyEdge = {
    id: 'edge-c1-s1',
    source_doc_id: 'calc-doc-1',
    source_anchor: {
      type: 'CalcChart',
      data: { sheet_name: 'Charts', chart_id: 'Chart_Revenue' },
    },
    target_doc_id: 'slides-doc-1',
    target_container: {
      type: 'SlideShapeContainer',
      data: { slide_index: 0, shape_id: 101 },
    },
    kind: 'LiveChartEmbed',
    created_at: 1100,
    version: 1,
  };

  beforeEach(() => {
    coordinator = new CrossDocDependencyCoordinator();
  });

  it('1. Initializes cleanly with default state', () => {
    const state = coordinator.getState();
    expect(state.edges).toEqual([]);
    expect(state.snapshots).toEqual({});
    expect(state.staleEmbedIds).toEqual([]);
    expect(state.summary.total_edges).toBe(0);
    expect(state.selectedEdgeId).toBeNull();
    expect(state.activeDocId).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('2. Registers a dependency edge and updates summary', async () => {
    const edgeId = await coordinator.registerDependency(mockEdge1);
    expect(edgeId).toBe('edge-c1-w1');

    const state = coordinator.getState();
    expect(state.edges.length).toBe(1);
    expect(state.edges[0].id).toBe('edge-c1-w1');
    expect(state.summary.total_edges).toBeGreaterThanOrEqual(1);
    expect(state.error).toBeNull();
  });

  it('3. Handles circular dependency rejection gracefully', async () => {
    const selfEdge: DependencyEdge = {
      id: 'self-loop',
      source_doc_id: 'doc-x',
      source_anchor: { type: 'GenericAnchor', data: { key: 'k' } },
      target_doc_id: 'doc-x',
      target_container: { type: 'GenericTarget', data: { id: 't' } },
      kind: 'LiveTextEmbed',
      created_at: 1200,
      version: 1,
    };

    await expect(coordinator.registerDependency(selfEdge)).rejects.toThrow();
    const state = coordinator.getState();
    expect(state.error).toContain('Circular dependency');
  });

  it('4. Sets active document and filters incoming/outgoing edges', async () => {
    await coordinator.registerDependency(mockEdge1);
    await coordinator.registerDependency(mockEdge2);

    coordinator.setActiveDocument('calc-doc-1');
    let state = coordinator.getState();
    expect(state.activeDocId).toBe('calc-doc-1');
    expect(state.activeDocOutgoing.length).toBe(2);
    expect(state.activeDocIncoming.length).toBe(0);

    coordinator.setActiveDocument('word-doc-1');
    state = coordinator.getState();
    expect(state.activeDocId).toBe('word-doc-1');
    expect(state.activeDocOutgoing.length).toBe(0);
    expect(state.activeDocIncoming.length).toBe(1);
  });

  it('5. Caches and retrieves live embed snapshots', async () => {
    await coordinator.registerDependency(mockEdge1);

    const snapshot: LiveEmbedSnapshot = {
      embed_id: 'edge-c1-w1',
      source_doc_id: 'calc-doc-1',
      version: 1,
      rendered_html: '<table><tr><td>Revenue</td><td>$100k</td></tr></table>',
      rendered_svg: null,
      tabular_data: [['Revenue', '$100k']],
      scalar_value: null,
      last_updated_timestamp: Date.now(),
      is_stale: false,
    };

    await coordinator.updateSnapshot(snapshot);
    const retrieved = await coordinator.getSnapshot('edge-c1-w1');

    expect(retrieved).not.toBeNull();
    expect(retrieved?.embed_id).toBe('edge-c1-w1');
    expect(retrieved?.rendered_html).toContain('$100k');
    expect(coordinator.isEmbedStale('edge-c1-w1')).toBe(false);
  });

  it('6. Propagates invalidation and marks embed snapshots stale', async () => {
    await coordinator.registerDependency(mockEdge1);

    const snapshot: LiveEmbedSnapshot = {
      embed_id: 'edge-c1-w1',
      source_doc_id: 'calc-doc-1',
      version: 1,
      rendered_html: '<table><tr><td>Data</td></tr></table>',
      rendered_svg: null,
      tabular_data: null,
      scalar_value: null,
      last_updated_timestamp: Date.now(),
      is_stale: false,
    };
    await coordinator.updateSnapshot(snapshot);

    const affected = await coordinator.propagateInvalidation('calc-doc-1', mockEdge1.source_anchor);
    expect(affected).toContain('word-doc-1');
    expect(coordinator.isEmbedStale('edge-c1-w1')).toBe(true);

    const staleSnap = coordinator.getState().snapshots['edge-c1-w1'];
    expect(staleSnap.is_stale).toBe(true);
  });

  it('7. Manually marks embed stale and tracks badge status', () => {
    expect(coordinator.isEmbedStale('edge-manual-1')).toBe(false);
    coordinator.markEmbedStale('edge-manual-1');
    expect(coordinator.isEmbedStale('edge-manual-1')).toBe(true);
    expect(coordinator.getState().staleEmbedIds).toContain('edge-manual-1');
  });

  it('8. Selects edge for inspector viewing', () => {
    coordinator.selectEdge('edge-inspect-1');
    expect(coordinator.getState().selectedEdgeId).toBe('edge-inspect-1');
    coordinator.selectEdge(null);
    expect(coordinator.getState().selectedEdgeId).toBeNull();
  });

  it('9. Removes dependency edge and purges snapshot', async () => {
    await coordinator.registerDependency(mockEdge1);
    expect(coordinator.getState().edges.length).toBe(1);

    const removed = await coordinator.removeDependency('edge-c1-w1');
    expect(removed).toBe(true);
    expect(coordinator.getState().edges.length).toBe(0);
    expect(coordinator.getState().snapshots['edge-c1-w1']).toBeUndefined();
  });

  it('10. Clears coordinator state and notifies reactive subscribers', () => {
    let callCount = 0;
    const unsub = coordinator.subscribe(() => {
      callCount++;
    });

    coordinator.markEmbedStale('edge-stale-test');
    coordinator.clear();

    const state = coordinator.getState();
    expect(state.edges).toEqual([]);
    expect(state.staleEmbedIds).toEqual([]);
    expect(callCount).toBeGreaterThanOrEqual(2);

    unsub();
  });
});

