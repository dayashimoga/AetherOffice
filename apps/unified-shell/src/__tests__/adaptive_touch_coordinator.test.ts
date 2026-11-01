import { describe, it, expect, beforeEach } from "vitest";
import {
  AdaptiveTouchCoordinator,
  type TouchPointDto,
} from "../state/AdaptiveTouchCoordinator";

describe("AdaptiveTouchCoordinator (Sub-Phase 52.2.2)", () => {
  let coordinator: AdaptiveTouchCoordinator;

  beforeEach(async () => {
    coordinator = new AdaptiveTouchCoordinator(1200, 800);
    await coordinator.init(1200, 800, { top: 24, bottom: 34 });
  });

  it("1. should initialize with default single-pane layout and 1.0 zoom", () => {
    const state = coordinator.getState();
    expect(state.viewportWidth).toBe(1200);
    expect(state.viewportHeight).toBe(800);
    expect(state.safeAreaInsets.top).toBe(24);
    expect(state.safeAreaInsets.bottom).toBe(34);
    expect(state.zoomScale).toBe(1.0);
    expect(state.scrollOffset).toEqual({ x: 0, y: 0 });
    expect(state.splitGrid.mode).toBe("single");
    expect(state.paneRects.length).toBe(1);
    expect(state.paneRects[0].y).toBe(24);
    expect(state.paneRects[0].height).toBe(800 - 24 - 34);
  });

  it("2. should accept normal touch contacts and dispatch touch start", async () => {
    const touch: TouchPointDto = {
      id: 101,
      x: 300,
      y: 400,
      radius_x: 6,
      radius_y: 6,
      pressure: 0.8,
      is_stylus: false,
      timestamp_ms: 1000,
    };

    const res = await coordinator.handleTouchStart(touch);
    expect(res.isPalm).toBe(false);
    expect(res.gesture.type).toBe("none");

    const state = coordinator.getState();
    expect(state.activeTouches.has(101)).toBe(true);
    expect(state.rejectedPalmTouches).not.toContain(101);
  });

  it("3. should reject palm contacts based on large geometry heuristics", async () => {
    const palmTouch: TouchPointDto = {
      id: 102,
      x: 500,
      y: 500,
      radius_x: 28,
      radius_y: 24,
      pressure: 1.0,
      is_stylus: false,
      timestamp_ms: 1000,
    };

    const res = await coordinator.handleTouchStart(palmTouch);
    expect(res.isPalm).toBe(true);
    expect(res.gesture.type).toBe("none");

    const state = coordinator.getState();
    expect(state.activeTouches.has(102)).toBe(false);
    expect(state.rejectedPalmTouches).toContain(102);

    // Any move from a rejected palm touch should also be ignored
    const moveRes = await coordinator.handleTouchMove(palmTouch);
    expect(moveRes.type).toBe("none");
  });

  it("4. should never reject stylus touches regardless of contact size", async () => {
    const stylusTouch: TouchPointDto = {
      id: 103,
      x: 400,
      y: 400,
      radius_x: 35,
      radius_y: 35,
      pressure: 0.95,
      is_stylus: true,
      timestamp_ms: 1000,
    };

    const res = await coordinator.handleTouchStart(stylusTouch);
    expect(res.isPalm).toBe(false);

    const state = coordinator.getState();
    expect(state.activeTouches.has(103)).toBe(true);
    expect(state.activeStylus).not.toBeNull();
    expect(state.activeStylus?.id).toBe(103);
  });

  it("5. should reject touches within active stylus proximity threshold (120px)", async () => {
    // 1. Register active stylus at (300, 300)
    const stylus: TouchPointDto = {
      id: 201,
      x: 300,
      y: 300,
      radius_x: 4,
      radius_y: 4,
      pressure: 0.9,
      is_stylus: true,
      timestamp_ms: 1000,
    };
    await coordinator.handleTouchStart(stylus);

    // 2. Palm rests nearby at (350, 320) -> distance ~ 53.8px < 120px
    const restingHand: TouchPointDto = {
      id: 202,
      x: 350,
      y: 320,
      radius_x: 10,
      radius_y: 10,
      pressure: 0.5,
      is_stylus: false,
      timestamp_ms: 1010,
    };

    const res = await coordinator.handleTouchStart(restingHand);
    expect(res.isPalm).toBe(true);
    expect(coordinator.getState().rejectedPalmTouches).toContain(202);
  });

  it("6. should reject touches within screen edge exclusion zone (< 15px)", async () => {
    const edgeTouch: TouchPointDto = {
      id: 301,
      x: 8, // < 15px from left edge
      y: 400,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.7,
      is_stylus: false,
      timestamp_ms: 1000,
    };

    const res = await coordinator.handleTouchStart(edgeTouch);
    expect(res.isPalm).toBe(true);
    expect(coordinator.getState().rejectedPalmTouches).toContain(301);
  });

  it("7. should dispatch pan-scroll gesture on single finger move and update scroll offset", async () => {
    const tStart: TouchPointDto = {
      id: 401,
      x: 200,
      y: 200,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1000,
    };
    await coordinator.handleTouchStart(tStart);

    const tMove: TouchPointDto = {
      id: 401,
      x: 235,
      y: 220,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1020,
    };
    const gesture = await coordinator.handleTouchMove(tMove);
    expect(gesture.type).toBe("pan-scroll");
    if (gesture.type === "pan-scroll") {
      expect(gesture.payload.delta_x).toBe(35);
      expect(gesture.payload.delta_y).toBe(20);
    }

    const state = coordinator.getState();
    expect(state.scrollOffset.x).toBe(35);
    expect(state.scrollOffset.y).toBe(20);
  });

  it("8. should dispatch pinch-to-zoom gesture on two finger move and update zoom scale", async () => {
    const t1: TouchPointDto = {
      id: 501,
      x: 200,
      y: 200,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1000,
    };
    const t2: TouchPointDto = {
      id: 502,
      x: 300,
      y: 200,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1000,
    };

    await coordinator.handleTouchStart(t1);
    await coordinator.handleTouchStart(t2);

    const t1Move: TouchPointDto = {
      ...t1,
      x: 180,
      timestamp_ms: 1020,
    };
    const gesture = await coordinator.handleTouchMove(t1Move);
    expect(gesture.type).toBe("pinch-zoom");
    if (gesture.type === "pinch-zoom") {
      expect(gesture.payload.scale).toBeGreaterThan(1.0);
    }

    const state = coordinator.getState();
    expect(state.zoomScale).toBeGreaterThan(1.0);
  });

  it("9. should clean up touch state on touch end and cancel", async () => {
    const touch: TouchPointDto = {
      id: 601,
      x: 300,
      y: 300,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1000,
    };
    await coordinator.handleTouchStart(touch);
    expect(coordinator.getState().activeTouches.has(601)).toBe(true);

    await coordinator.handleTouchEnd(601, 1050);
    expect(coordinator.getState().activeTouches.has(601)).toBe(false);

    // Cancel test
    const touch2: TouchPointDto = {
      id: 602,
      x: 400,
      y: 400,
      radius_x: 5,
      radius_y: 5,
      pressure: 0.6,
      is_stylus: false,
      timestamp_ms: 1000,
    };
    await coordinator.handleTouchStart(touch2);
    expect(coordinator.getState().activeTouches.has(602)).toBe(true);

    await coordinator.handleTouchCancel(602);
    expect(coordinator.getState().activeTouches.has(602)).toBe(false);
  });

  it("10. should configure adaptive split-screen layouts with magnetic snapping and safe-area insets", async () => {
    // 1. Two-way horizontal with magnetic snap (0.49 -> 0.500)
    const rects2Way = await coordinator.setSplitLayout("two-way-horizontal", 0.49);
    expect(rects2Way.length).toBe(2);
    expect(coordinator.getState().splitGrid.ratio_x).toBe(0.500);
    expect(rects2Way[0].width).toBe(600);
    expect(rects2Way[1].width).toBe(600);
    expect(rects2Way[0].y).toBe(24);
    expect(rects2Way[0].height).toBe(800 - 24 - 34);

    // 2. Four-way grid layout
    const rects4Way = await coordinator.setSplitLayout("four-way-grid", 0.5, 0.5);
    expect(rects4Way.length).toBe(4);
    expect(rects4Way[0].width).toBe(600);
    expect(rects4Way[0].height).toBe((800 - 24 - 34) / 2);

    // 3. Reset transforms
    await coordinator.resetTransforms();
    const resetState = coordinator.getState();
    expect(resetState.zoomScale).toBe(1.0);
    expect(resetState.scrollOffset).toEqual({ x: 0, y: 0 });
    expect(resetState.lastGesture.type).toBe("none");
  });
});
