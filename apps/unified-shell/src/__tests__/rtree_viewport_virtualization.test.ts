import { describe, it, expect, beforeEach } from "vitest";
import {
  ViewportTileVirtualizer,
  ViewportRect,
} from "../components/ViewportTileVirtualizer";

describe("Sub-Phase 45.1.1: ViewportTileVirtualizer & Tile Virtualization", () => {
  let virtualizer: ViewportTileVirtualizer;

  beforeEach(() => {
    virtualizer = new ViewportTileVirtualizer(512, 1);
  });

  it("initializes with correct defaults and properties", () => {
    expect(virtualizer.getTileSize()).toBe(512);
    expect(virtualizer.getOverscanMarginTiles()).toBe(1);
    expect(virtualizer.getCachedTileCount()).toBe(0);

    const custom = new ViewportTileVirtualizer(256, 2);
    expect(custom.getTileSize()).toBe(256);
    expect(custom.getOverscanMarginTiles()).toBe(2);
  });

  it("computes accurate tile bounds for positive and negative coordinates", () => {
    const b0 = virtualizer.computeTileBounds({ col: 0, row: 0, lod: 0 });
    expect(b0).toEqual({ x: 0, y: 0, width: 512, height: 512 });

    const b1 = virtualizer.computeTileBounds({ col: 2, row: 3, lod: 0 });
    expect(b1).toEqual({ x: 1024, y: 1536, width: 512, height: 512 });

    const bNeg = virtualizer.computeTileBounds({ col: -1, row: -2, lod: 0 });
    expect(bNeg).toEqual({ x: -512, y: -1024, width: 512, height: 512 });
  });

  it("calculates exact visible tiles for viewport", () => {
    // Single tile viewport entirely inside [0, 0, 512, 512]
    const vp1: ViewportRect = { x: 100, y: 100, width: 200, height: 200 };
    const vis1 = virtualizer.getVisibleTileCoords(vp1);
    expect(vis1).toHaveLength(1);
    expect(vis1[0]).toEqual({ col: 0, row: 0, lod: 0 });

    // Viewport crossing 4 tiles: [400, 400] to [700, 700]
    const vp2: ViewportRect = { x: 400, y: 400, width: 300, height: 300 };
    const vis2 = virtualizer.getVisibleTileCoords(vp2);
    expect(vis2).toHaveLength(4);
    expect(vis2).toContainEqual({ col: 0, row: 0, lod: 0 });
    expect(vis2).toContainEqual({ col: 1, row: 0, lod: 0 });
    expect(vis2).toContainEqual({ col: 0, row: 1, lod: 0 });
    expect(vis2).toContainEqual({ col: 1, row: 1, lod: 0 });
  });

  it("applies overscan preloading margins", () => {
    const vp: ViewportRect = { x: 100, y: 100, width: 200, height: 200 };
    // Margin is 1 tile: col -1..1, row -1..1 => 3 x 3 = 9 tiles
    const buffered = virtualizer.getBufferedTileCoords(vp);
    expect(buffered).toHaveLength(9);

    expect(buffered).toContainEqual({ col: -1, row: -1, lod: 0 });
    expect(buffered).toContainEqual({ col: 0, row: 0, lod: 0 });
    expect(buffered).toContainEqual({ col: 1, row: 1, lod: 0 });
  });

  it("tracks viewport frames and caches tiles across movements", () => {
    const vp1: ViewportRect = { x: 0, y: 0, width: 500, height: 500 };
    const res1 = virtualizer.updateViewport(vp1, 0, 1);

    expect(res1.visible).toHaveLength(1);
    expect(res1.newlyNeeded).toHaveLength(9);
    expect(virtualizer.getCachedTileCount()).toBe(9);
    expect(virtualizer.isTileVisible({ col: 0, row: 0, lod: 0 })).toBe(true);
    expect(virtualizer.isTileVisible({ col: 1, row: 1, lod: 0 })).toBe(false);

    // Frame 2: same viewport => newlyNeeded is empty
    const res2 = virtualizer.updateViewport(vp1, 0, 2);
    expect(res2.newlyNeeded).toHaveLength(0);
    expect(virtualizer.getCachedTileCount()).toBe(9);

    // Pan to tile (10, 10) on Frame 15
    const vpFar: ViewportRect = { x: 5120, y: 5120, width: 500, height: 500 };
    const resFar = virtualizer.updateViewport(vpFar, 0, 15);
    expect(resFar.newlyNeeded).toHaveLength(9);
    expect(virtualizer.getCachedTileCount()).toBe(18);

    // Evict cache to max 10 tiles
    const evicted = virtualizer.evictCache(10);
    expect(evicted).toHaveLength(8);
    expect(virtualizer.getCachedTileCount()).toBe(10);

    // The evicted tiles should have been the oldest ones from Frame 1/2
    expect(virtualizer.isTileVisible({ col: 10, row: 10, lod: 0 })).toBe(true);
  });

  it("satisfies 120fps budget: 1,000 viewport queries under 10ms", () => {
    const start = performance.now();
    for (let i = 0; i < 1000; i++) {
      const vp: ViewportRect = {
        x: (i * 17) % 5000,
        y: (i * 23) % 5000,
        width: 1920,
        height: 1080,
      };
      virtualizer.getVisibleTileCoords(vp);
    }
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThan(50);
  });
});
