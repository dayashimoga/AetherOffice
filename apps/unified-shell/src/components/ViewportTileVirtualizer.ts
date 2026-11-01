/**
 * Viewport Tile Virtualizer & Overscan Culling Engine
 *
 * Implements GEC-v7.0 Article I, V, and VI:
 * - 120fps rendering budget (<= 8.33ms per frame) with sub-0.1ms tile coordinate calculation.
 * - Dynamic 2D tile virtualization with LOD support, overscan buffering, and LRU cache eviction.
 * - Eliminates memory spikes and excessive draw calls across large documents (Word, Calc, Slides, PDF).
 */

export interface ViewportRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface TileCoord {
  col: number;
  row: number;
  lod: number;
}

export interface VirtualTile {
  coord: TileCoord;
  bounds: ViewportRect;
  isVisible: boolean;
  lastFrameSeen: number;
}

export class ViewportTileVirtualizer {
  private tileSize: number;
  private overscanMarginTiles: number;
  private cachedTiles: Map<string, VirtualTile> = new Map();

  constructor(tileSize: number = 512, overscanMarginTiles: number = 1) {
    this.tileSize = tileSize > 0 ? tileSize : 512;
    this.overscanMarginTiles = Math.max(0, overscanMarginTiles);
  }

  public getTileSize(): number {
    return this.tileSize;
  }

  public getOverscanMarginTiles(): number {
    return this.overscanMarginTiles;
  }

  public static tileKey(coord: TileCoord): string {
    return `${coord.lod}:${coord.col}:${coord.row}`;
  }

  /**
   * Computes exact 2D bounding rectangle in DIPs for a given tile coordinate.
   */
  public computeTileBounds(coord: TileCoord): ViewportRect {
    return {
      x: coord.col * this.tileSize,
      y: coord.row * this.tileSize,
      width: this.tileSize,
      height: this.tileSize,
    };
  }

  /**
   * Computes all tile coordinates strictly intersecting the current viewport.
   */
  public getVisibleTileCoords(viewport: ViewportRect, lod: number = 0): TileCoord[] {
    const right = viewport.x + viewport.width;
    const bottom = viewport.y + viewport.height;

    const colMin = Math.floor(viewport.x / this.tileSize);
    const colMax = Math.floor(right / this.tileSize);
    const rowMin = Math.floor(viewport.y / this.tileSize);
    const rowMax = Math.floor(bottom / this.tileSize);

    const coords: TileCoord[] = [];
    for (let c = colMin; c <= colMax; c++) {
      for (let r = rowMin; r <= rowMax; r++) {
        coords.push({ col: c, row: r, lod });
      }
    }
    return coords;
  }

  /**
   * Computes tile coordinates intersecting viewport expanded by overscan preloading margins.
   */
  public getBufferedTileCoords(viewport: ViewportRect, lod: number = 0): TileCoord[] {
    const right = viewport.x + viewport.width;
    const bottom = viewport.y + viewport.height;
    const m = this.overscanMarginTiles;

    const colMin = Math.floor(viewport.x / this.tileSize) - m;
    const colMax = Math.floor(right / this.tileSize) + m;
    const rowMin = Math.floor(viewport.y / this.tileSize) - m;
    const rowMax = Math.floor(bottom / this.tileSize) + m;

    const coords: TileCoord[] = [];
    for (let c = colMin; c <= colMax; c++) {
      for (let r = rowMin; r <= rowMax; r++) {
        coords.push({ col: c, row: r, lod });
      }
    }
    return coords;
  }

  /**
   * Updates viewport state for the current frame.
   * Returns visible tile coordinates and newly requested tiles that were not previously cached.
   */
  public updateViewport(
    viewport: ViewportRect,
    lod: number = 0,
    frameNum: number = 1
  ): { visible: TileCoord[]; newlyNeeded: TileCoord[] } {
    const visible = this.getVisibleTileCoords(viewport, lod);
    const buffered = this.getBufferedTileCoords(viewport, lod);

    const visibleKeySet = new Set<string>();
    for (const v of visible) {
      visibleKeySet.add(ViewportTileVirtualizer.tileKey(v));
    }

    const newlyNeeded: TileCoord[] = [];

    for (const coord of buffered) {
      const key = ViewportTileVirtualizer.tileKey(coord);
      const isVisible = visibleKeySet.has(key);

      const existing = this.cachedTiles.get(key);
      if (existing) {
        existing.isVisible = isVisible;
        existing.lastFrameSeen = frameNum;
      } else {
        const bounds = this.computeTileBounds(coord);
        this.cachedTiles.set(key, {
          coord,
          bounds,
          isVisible,
          lastFrameSeen: frameNum,
        });
        newlyNeeded.push(coord);
      }
    }

    return { visible, newlyNeeded };
  }

  /**
   * Evicts least-recently-used tiles from the cache when capacity is exceeded.
   */
  public evictCache(maxCached: number): TileCoord[] {
    if (this.cachedTiles.size <= maxCached) {
      return [];
    }

    const entries = Array.from(this.cachedTiles.entries()).sort(
      (a, b) => a[1].lastFrameSeen - b[1].lastFrameSeen
    );

    const countToEvict = this.cachedTiles.size - maxCached;
    const evicted: TileCoord[] = [];

    for (let i = 0; i < countToEvict; i++) {
      const [key, tile] = entries[i];
      this.cachedTiles.delete(key);
      evicted.push(tile.coord);
    }

    return evicted;
  }

  public isTileVisible(coord: TileCoord): boolean {
    const key = ViewportTileVirtualizer.tileKey(coord);
    return this.cachedTiles.get(key)?.isVisible ?? false;
  }

  public getCachedTileCount(): number {
    return this.cachedTiles.size;
  }

  public clear(): void {
    this.cachedTiles.clear();
  }
}

