/**
 * AdaptiveTouchCoordinator.ts
 * GEC-v7.0 Stage 52 (Sub-Phase 52.2.2)
 *
 * Presentation Shell Coordinator for:
 * - High-precision multi-touch gesture processing (pinch-to-zoom, pan/scroll, double-tap, long-press, two-finger swipe)
 * - Hardware-aware stylus palm rejection (geometry heuristics, active stylus proximity, edge exclusion)
 * - Adaptive split-screen grid management with magnetic snap ratios and OS safe-area insets
 */

import {
  IPC,
  type AdaptiveSplitGridDto,
  type Rect2DDto,
  type SplitGridMode,
  type SwipeDirection,
  type TouchGestureDto,
  type TouchPointDto,
} from "../ipc";

export {
  type AdaptiveSplitGridDto,
  type Rect2DDto,
  type SplitGridMode,
  type SwipeDirection,
  type TouchGestureDto,
  type TouchPointDto,
};

export interface SafeAreaInsets {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface AdaptiveTouchState {
  activeTouches: Map<number, TouchPointDto>;
  activeStylus: TouchPointDto | null;
  lastGesture: TouchGestureDto;
  splitGrid: AdaptiveSplitGridDto;
  paneRects: Rect2DDto[];
  viewportWidth: number;
  viewportHeight: number;
  safeAreaInsets: SafeAreaInsets;
  zoomScale: number;
  scrollOffset: { x: number; y: number };
  rejectedPalmTouches: number[];
  errorMessage: string | null;
}

export type TouchListener = (state: Readonly<AdaptiveTouchState>) => void;

export class AdaptiveTouchCoordinator {
  private state: AdaptiveTouchState;
  private listeners = new Set<TouchListener>();

  constructor(initialWidth = 1200, initialHeight = 800) {
    this.state = {
      activeTouches: new Map(),
      activeStylus: null,
      lastGesture: { type: "none" },
      splitGrid: {
        mode: "single",
        ratio_x: 0.5,
        ratio_y: 0.5,
        snap_threshold: 0.04,
      },
      paneRects: [{ x: 0, y: 0, width: initialWidth, height: initialHeight }],
      viewportWidth: initialWidth,
      viewportHeight: initialHeight,
      safeAreaInsets: { top: 0, right: 0, bottom: 0, left: 0 },
      zoomScale: 1.0,
      scrollOffset: { x: 0, y: 0 },
      rejectedPalmTouches: [],
      errorMessage: null,
    };
  }

  /**
   * Returns an immutable snapshot of current touch and split view state.
   */
  public getState(): Readonly<AdaptiveTouchState> {
    return {
      ...this.state,
      activeTouches: new Map(this.state.activeTouches),
      rejectedPalmTouches: [...this.state.rejectedPalmTouches],
      paneRects: [...this.state.paneRects],
    };
  }

  /**
   * Subscribes a listener to touch state mutations.
   * Returns an unsubscription function.
   */
  public subscribe(listener: TouchListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("AdaptiveTouchCoordinator listener error:", err);
      }
    }
  }

  /**
   * Initializes the coordinator with viewport dimensions and safe area insets.
   */
  public async init(
    width = 1200,
    height = 800,
    insets?: Partial<SafeAreaInsets>
  ): Promise<void> {
    this.state.viewportWidth = width;
    this.state.viewportHeight = height;
    if (insets) {
      this.state.safeAreaInsets = {
        top: insets.top ?? 0,
        right: insets.right ?? 0,
        bottom: insets.bottom ?? 0,
        left: insets.left ?? 0,
      };
    }
    this.state.activeTouches.clear();
    this.state.rejectedPalmTouches = [];
    this.state.activeStylus = null;
    await IPC.gestureProcessTouchCancel(0);
    await this.refreshPaneRects();
  }

  /**
   * Evaluates incoming touch start contact.
   * Runs palm rejection filter; if accepted, passes to gesture recognizer.
   */
  public async handleTouchStart(
    touch: TouchPointDto
  ): Promise<{ isPalm: boolean; gesture: TouchGestureDto }> {
    try {
      if (touch.is_stylus) {
        this.state.activeStylus = touch;
      }

      // Check palm rejection filter
      const isPalm = await IPC.palmRejectionFilterContact(
        touch,
        this.state.activeStylus,
        this.state.viewportWidth,
        this.state.viewportHeight
      );

      if (isPalm) {
        if (!this.state.rejectedPalmTouches.includes(touch.id)) {
          this.state.rejectedPalmTouches.push(touch.id);
        }
        this.notify();
        return { isPalm: true, gesture: { type: "none" } };
      }

      this.state.activeTouches.set(touch.id, touch);
      const gesture = await IPC.gestureProcessTouchStart(touch);
      this.state.lastGesture = gesture;
      this.applyGesture(gesture);
      this.notify();
      return { isPalm: false, gesture };
    } catch (err) {
      this.state.errorMessage = `Touch start failed: ${String(err)}`;
      this.notify();
      return { isPalm: false, gesture: { type: "none" } };
    }
  }

  /**
   * Handles touch movement.
   * Dispatches updates to gesture recognizer and updates pan/zoom states.
   */
  public async handleTouchMove(touch: TouchPointDto): Promise<TouchGestureDto> {
    try {
      if (this.state.rejectedPalmTouches.includes(touch.id)) {
        return { type: "none" };
      }

      if (touch.is_stylus) {
        this.state.activeStylus = touch;
      }

      this.state.activeTouches.set(touch.id, touch);
      const gesture = await IPC.gestureProcessTouchMove(touch);
      this.state.lastGesture = gesture;
      this.applyGesture(gesture);
      this.notify();
      return gesture;
    } catch (err) {
      this.state.errorMessage = `Touch move failed: ${String(err)}`;
      this.notify();
      return { type: "none" };
    }
  }

  /**
   * Handles touch release / completion.
   */
  public async handleTouchEnd(
    touchId: number,
    timestampMs?: number
  ): Promise<TouchGestureDto> {
    try {
      this.state.activeTouches.delete(touchId);
      const palmIdx = this.state.rejectedPalmTouches.indexOf(touchId);
      if (palmIdx >= 0) {
        this.state.rejectedPalmTouches.splice(palmIdx, 1);
      }

      if (this.state.activeStylus?.id === touchId) {
        this.state.activeStylus = null;
      }

      const gesture = await IPC.gestureProcessTouchEnd(
        touchId,
        timestampMs ?? Date.now()
      );
      this.state.lastGesture = gesture;
      this.applyGesture(gesture);
      this.notify();
      return gesture;
    } catch (err) {
      this.state.errorMessage = `Touch end failed: ${String(err)}`;
      this.notify();
      return { type: "none" };
    }
  }

  /**
   * Handles touch cancellation (e.g., interrupted by system alert or palm rejection).
   */
  public async handleTouchCancel(touchId: number): Promise<void> {
    try {
      this.state.activeTouches.delete(touchId);
      const palmIdx = this.state.rejectedPalmTouches.indexOf(touchId);
      if (palmIdx >= 0) {
        this.state.rejectedPalmTouches.splice(palmIdx, 1);
      }

      if (this.state.activeStylus?.id === touchId) {
        this.state.activeStylus = null;
      }

      await IPC.gestureProcessTouchCancel(touchId);
      this.notify();
    } catch (err) {
      this.state.errorMessage = `Touch cancel failed: ${String(err)}`;
      this.notify();
    }
  }

  /**
   * Configures split-grid mode and proportions, recomputing sub-pane rectangles.
   */
  public async setSplitLayout(
    mode: SplitGridMode | string,
    ratioX?: number,
    ratioY?: number
  ): Promise<Rect2DDto[]> {
    try {
      const grid = await IPC.adaptiveSplitSetLayout(mode, ratioX, ratioY);
      this.state.splitGrid = grid;
      return await this.refreshPaneRects();
    } catch (err) {
      this.state.errorMessage = `Set split layout failed: ${String(err)}`;
      this.notify();
      return this.state.paneRects;
    }
  }

  /**
   * Updates viewport dimensions and recalculates active subpane rectangles.
   */
  public async updateViewport(
    width: number,
    height: number,
    insets?: Partial<SafeAreaInsets>
  ): Promise<Rect2DDto[]> {
    this.state.viewportWidth = width;
    this.state.viewportHeight = height;
    if (insets) {
      this.state.safeAreaInsets = {
        top: insets.top ?? this.state.safeAreaInsets.top,
        right: insets.right ?? this.state.safeAreaInsets.right,
        bottom: insets.bottom ?? this.state.safeAreaInsets.bottom,
        left: insets.left ?? this.state.safeAreaInsets.left,
      };
    }
    return await this.refreshPaneRects();
  }

  /**
   * Refreshes subpane bounding boxes from backend split grid.
   */
  public async refreshPaneRects(): Promise<Rect2DDto[]> {
    try {
      const rects = await IPC.adaptiveSplitGetRects(
        this.state.viewportWidth,
        this.state.viewportHeight,
        this.state.safeAreaInsets.top,
        this.state.safeAreaInsets.bottom
      );
      this.state.paneRects = rects;
      this.notify();
      return rects;
    } catch (err) {
      this.state.errorMessage = `Refresh pane rects failed: ${String(err)}`;
      this.notify();
      return this.state.paneRects;
    }
  }

  /**
   * Resets accumulated gestures and offsets.
   */
  public async resetTransforms(): Promise<void> {
    this.state.zoomScale = 1.0;
    this.state.scrollOffset = { x: 0, y: 0 };
    this.state.lastGesture = { type: "none" };
    this.state.activeTouches.clear();
    this.state.rejectedPalmTouches = [];
    this.state.activeStylus = null;
    await IPC.gestureProcessTouchCancel(0);
    this.notify();
  }

  private applyGesture(gesture: TouchGestureDto): void {
    if (gesture.type === "pinch-zoom") {
      const newScale = this.state.zoomScale * gesture.payload.scale;
      // Clamp zoom between 25% and 500%
      this.state.zoomScale = Math.min(5.0, Math.max(0.25, newScale));
    } else if (gesture.type === "pan-scroll") {
      this.state.scrollOffset.x += gesture.payload.delta_x;
      this.state.scrollOffset.y += gesture.payload.delta_y;
    }
  }
}
