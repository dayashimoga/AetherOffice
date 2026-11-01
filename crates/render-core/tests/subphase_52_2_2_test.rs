//! subphase_52_2_2_test.rs
//! GEC-v7.0 Stage 52 (Sub-Phase 52.2.2)
//!
//! Rust Integration Tests for:
//! - Touch contact geometry and ellipse area calculation
//! - Stylus palm rejection filtering (contact radius, ellipse area, stylus proximity, screen edge exclusion)
//! - Multi-touch gesture recognition (PinchZoom, PanScroll, DoubleTap)
//! - Adaptive split grid sub-pane rectangle compilation with safe area avoidance
//! - Magnetic ratio divider snapping (1/3, 1/2, 2/3)

use render_core::adaptive::SafeAreaInsets;
use render_core::primitive::Rect2D;
use render_core::{
    AdaptiveSplitGrid, PalmRejectionFilter, SplitGridMode, TouchGesture, TouchGestureRecognizer,
    TouchPoint,
};

#[test]
fn test_touch_point_creation_and_contact_area() {
    let p = TouchPoint::new(1, 100.0, 200.0, 1000).with_geometry(10.0, 10.0, 0.6, false);
    assert_eq!(p.id, 1);
    assert_eq!(p.x, 100.0);
    assert_eq!(p.y, 200.0);
    assert_eq!(p.pressure, 0.6);
    assert!(!p.is_stylus);

    // Area: pi * 10 * 10 ~ 314.159
    let area = p.contact_area();
    assert!((area - 314.159).abs() < 0.1);
}

#[test]
fn test_palm_rejection_geometry_filter() {
    let filter = PalmRejectionFilter::new();

    // Normal finger touch (radius 8px, area ~201px^2) -> Not palm
    let finger = TouchPoint::new(1, 200.0, 200.0, 1000).with_geometry(8.0, 8.0, 0.5, false);
    assert!(!filter.is_palm(&finger, None, 1024.0, 768.0));

    // Large palm contact (radius 25px, area ~1963px^2) -> Palm!
    let palm = TouchPoint::new(2, 200.0, 200.0, 1000).with_geometry(25.0, 25.0, 0.8, false);
    assert!(filter.is_palm(&palm, None, 1024.0, 768.0));

    // Stylus with large contact radius -> Stylus is never rejected!
    let stylus = TouchPoint::new(3, 200.0, 200.0, 1000).with_geometry(30.0, 30.0, 0.9, true);
    assert!(!filter.is_palm(&stylus, None, 1024.0, 768.0));
}

#[test]
fn test_palm_rejection_stylus_proximity_masking() {
    let filter = PalmRejectionFilter::new();
    let stylus = TouchPoint::new(10, 300.0, 300.0, 1000).with_geometry(2.0, 2.0, 0.9, true);

    // Finger resting 50px away from stylus -> Suppressed as supporting palm!
    let nearby_finger = TouchPoint::new(11, 340.0, 330.0, 1000).with_geometry(9.0, 9.0, 0.5, false);
    assert!(filter.is_palm(&nearby_finger, Some(&stylus), 1024.0, 768.0));

    // Other hand touching 400px away -> Intentional 2nd touch, not suppressed
    let distant_finger =
        TouchPoint::new(12, 700.0, 300.0, 1000).with_geometry(9.0, 9.0, 0.5, false);
    assert!(!filter.is_palm(&distant_finger, Some(&stylus), 1024.0, 768.0));
}

#[test]
fn test_palm_rejection_edge_exclusion() {
    let filter = PalmRejectionFilter::new();

    // Touch at x = 5 (within 15px screen margin) -> Rejected
    let edge_touch = TouchPoint::new(1, 5.0, 300.0, 1000).with_geometry(8.0, 8.0, 0.5, false);
    assert!(filter.is_palm(&edge_touch, None, 1024.0, 768.0));

    // Touch at center (x = 500) -> Accepted
    let center_touch = TouchPoint::new(2, 500.0, 300.0, 1000).with_geometry(8.0, 8.0, 0.5, false);
    assert!(!filter.is_palm(&center_touch, None, 1024.0, 768.0));
}

#[test]
fn test_gesture_pinch_to_zoom() {
    let mut recognizer = TouchGestureRecognizer::new();

    // Start 2 touches at (100, 100) and (200, 100) -> distance = 100
    recognizer.touch_start(TouchPoint::new(1, 100.0, 100.0, 100));
    recognizer.touch_start(TouchPoint::new(2, 200.0, 100.0, 100));
    assert_eq!(recognizer.active_touch_count(), 2);

    // Move touch 2 outwards to (250, 100) -> distance = 150 -> zoom in!
    let gesture = recognizer.touch_move(TouchPoint::new(2, 250.0, 100.0, 120));

    match gesture {
        TouchGesture::PinchZoom {
            scale,
            focal_x,
            focal_y,
        } => {
            assert!(scale > 1.2);
            assert_eq!(focal_x, 175.0); // (100 + 250) / 2
            assert_eq!(focal_y, 100.0);
        }
        other => panic!("Expected PinchZoom, got {:?}", other),
    }
}

#[test]
fn test_gesture_single_touch_pan_scroll() {
    let mut recognizer = TouchGestureRecognizer::new();

    recognizer.touch_start(TouchPoint::new(1, 100.0, 100.0, 100));
    let gesture = recognizer.touch_move(TouchPoint::new(1, 120.0, 130.0, 120));

    match gesture {
        TouchGesture::PanScroll { delta_x, delta_y } => {
            assert_eq!(delta_x, 20.0);
            assert_eq!(delta_y, 30.0);
        }
        other => panic!("Expected PanScroll, got {:?}", other),
    }
}

#[test]
fn test_gesture_double_tap_detection() {
    let mut recognizer = TouchGestureRecognizer::new();

    // First tap
    recognizer.touch_start(TouchPoint::new(1, 200.0, 200.0, 100));
    let g1 = recognizer.touch_end(1, 180);
    assert_eq!(g1, TouchGesture::None);

    // Second tap 150ms later at same coordinate
    recognizer.touch_start(TouchPoint::new(2, 202.0, 201.0, 250));
    let g2 = recognizer.touch_end(2, 320);

    match g2 {
        TouchGesture::DoubleTap { tap_x, tap_y } => {
            assert_eq!(tap_x, 202.0);
            assert_eq!(tap_y, 201.0);
        }
        other => panic!("Expected DoubleTap, got {:?}", other),
    }
}

#[test]
fn test_adaptive_split_grid_single_and_two_way() {
    let grid = AdaptiveSplitGrid::new(SplitGridMode::Single);
    let total = Rect2D::new(0.0, 0.0, 1000.0, 600.0);
    let insets = SafeAreaInsets::default();

    let single_rects = grid.compute_subpane_rects(&total, &insets);
    assert_eq!(single_rects.len(), 1);
    assert_eq!(single_rects[0], Rect2D::new(0.0, 0.0, 1000.0, 600.0));

    // Two-Way Horizontal (50/50)
    let grid_h = AdaptiveSplitGrid::new(SplitGridMode::TwoWayHorizontal);
    let h_rects = grid_h.compute_subpane_rects(&total, &insets);
    assert_eq!(h_rects.len(), 2);
    assert_eq!(h_rects[0], Rect2D::new(0.0, 0.0, 500.0, 600.0));
    assert_eq!(h_rects[1], Rect2D::new(500.0, 0.0, 500.0, 600.0));

    // Two-Way Vertical (50/50)
    let grid_v = AdaptiveSplitGrid::new(SplitGridMode::TwoWayVertical);
    let v_rects = grid_v.compute_subpane_rects(&total, &insets);
    assert_eq!(v_rects.len(), 2);
    assert_eq!(v_rects[0], Rect2D::new(0.0, 0.0, 1000.0, 300.0));
    assert_eq!(v_rects[1], Rect2D::new(0.0, 300.0, 1000.0, 300.0));
}

#[test]
fn test_adaptive_split_grid_four_way_quadrant() {
    let grid_4 = AdaptiveSplitGrid::new(SplitGridMode::FourWayGrid);
    let total = Rect2D::new(0.0, 0.0, 800.0, 600.0);
    let insets = SafeAreaInsets {
        top: 20.0,
        right: 0.0,
        bottom: 20.0,
        left: 0.0,
    };

    let rects = grid_4.compute_subpane_rects(&total, &insets);
    assert_eq!(rects.len(), 4);

    // Safe height = 600 - 40 = 560; half = 280
    // Safe width = 800; half = 400
    assert_eq!(rects[0], Rect2D::new(0.0, 20.0, 400.0, 280.0)); // Top-Left
    assert_eq!(rects[1], Rect2D::new(400.0, 20.0, 400.0, 280.0)); // Top-Right
    assert_eq!(rects[2], Rect2D::new(0.0, 300.0, 400.0, 280.0)); // Bottom-Left
    assert_eq!(rects[3], Rect2D::new(400.0, 300.0, 400.0, 280.0)); // Bottom-Right
}

#[test]
fn test_split_grid_ratio_magnetic_snap() {
    let mut grid = AdaptiveSplitGrid::new(SplitGridMode::TwoWayHorizontal);

    // 0.49 is within snap threshold (0.04) of 0.500 -> snaps to 0.500
    grid.set_ratios_with_snap(0.49, 0.51);
    assert_eq!(grid.ratio_x, 0.500);
    assert_eq!(grid.ratio_y, 0.500);

    // 0.34 is within snap threshold of 1/3 (0.333) -> snaps to 0.333
    grid.set_ratios_with_snap(0.34, 0.65);
    assert_eq!(grid.ratio_x, 0.333);
    assert_eq!(grid.ratio_y, 0.667);

    // 0.20 is not near any snap point -> kept at 0.20
    grid.set_ratios_with_snap(0.20, 0.80);
    assert_eq!(grid.ratio_x, 0.20);
    assert_eq!(grid.ratio_y, 0.80);
}
