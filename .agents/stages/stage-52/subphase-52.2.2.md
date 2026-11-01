# Sub-Phase 52.2.2: Android & iOS Touch Gestures, Stylus Palm Rejection & Adaptive Split Views

## Objective
Establish high-performance touch interactions, stylus palm rejection filtering, and flexible adaptive split views across mobile, tablet, and desktop environments:
1. **Authoritative Rust Core (`crates/render-core/src/gesture.rs`, `adaptive.rs`, `lib.rs`)**:
   - `TouchGesture`: `PinchZoom(scale, focal_x, focal_y)`, `PanScroll(dx, dy)`, `DoubleTap(x, y)`, `LongPress(x, y)`, `TwoFingerSwipe(direction)`.
   - `TouchGestureRecognizer`: Multi-touch tracking and gesture classification.
   - `PalmRejectionFilter`: Mathematical palm suppression using contact geometry, ellipse contact area, stylus proximity masking, and edge exclusion.
   - `AdaptiveSplitGrid`: 2-way horizontal/vertical and 4-way grid layout compilation with divider drag-handle snapping and safe area avoidance.
2. **Native Tauri Desktop IPC (`apps/unified-shell/src-tauri`)**:
   - Commands:
     - `gesture_process_touch_event`: Ingests raw pointer/touch points and yields classified gesture events.
     - `palm_rejection_filter_contact`: Evaluates whether a touch contact constitutes palm interference.
     - `adaptive_split_set_layout`: Sets split configuration (Single, TwoWayHorizontal, TwoWayVertical, FourWayGrid) and ratio.
     - `adaptive_split_get_rects`: Calculates sub-pane layout bounds for given container dimensions.
3. **TypeScript Presentation Shell Coordinator (`apps/unified-shell/src/state/AdaptiveTouchCoordinator.ts`)**:
   - Handles multi-touch gestures (pinch-to-zoom on canvas, pan scrolling, swipe gestures).
   - Filters out resting palm contacts during Apple Pencil / S-Pen / Surface Pen stylus inking.
   - Manages responsive split-view drag divider and responsive multi-pane layout.
4. **Verification & Invariants**:
   - 10 integration tests in `crates/render-core/tests/subphase_52_2_2_test.rs`.
   - 10 unit/integration tests in `apps/unified-shell/src/__tests__/adaptive_touch_coordinator.test.ts`.
   - 100% pass on `tools/verify-contract.ps1`.

