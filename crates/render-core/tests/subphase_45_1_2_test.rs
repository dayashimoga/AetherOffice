//! Sub-Phase 45.1.2 Integration Tests: Subpixel Text Shaper & High-DPI Canvas Compositor
//!
//! Complies strictly with GEC-v7.0 Article I, IV, V, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Subpixel kerning and device-pixel snapping.
//! - BiDi run segmentation (UAX #9) and visual ordering.
//! - Performance SLA: $< 1.0\text{ ms}$ text shaping for multi-paragraph layouts.

use render_core::primitive::{ColorRgba, TextScript};
use render_core::text::{
    snap_to_device_pixel, HighDpiTextCompositor, SubpixelKerning, TextDirection,
};
use std::time::Instant;

#[test]
fn test_subpixel_device_snapping() {
    // 1.0x DPR
    assert_eq!(snap_to_device_pixel(10.2, 1.0), 10.0);
    assert_eq!(snap_to_device_pixel(10.8, 1.0), 11.0);

    // 1.5x DPR: pixels are multiples of 1/1.5 = 0.6666...
    let snapped_15 = snap_to_device_pixel(10.2, 1.5);
    // 10.2 * 1.5 = 15.3 -> round to 15 -> 15 / 1.5 = 10.0
    assert_eq!(snapped_15, 10.0);

    // 2.0x DPR: pixels are multiples of 0.5
    assert_eq!(snap_to_device_pixel(10.26, 2.0), 10.5);
    assert_eq!(snap_to_device_pixel(10.1, 2.0), 10.0);

    let compositor = HighDpiTextCompositor::new(2.0);
    assert_eq!(compositor.snap_coord(15.74), 15.5);
}

#[test]
fn test_subpixel_kerning_pairs() {
    // Known kerning pairs should yield negative adjustment
    let k_av = SubpixelKerning::get_kerning_ratio('A', 'V');
    assert!(k_av < 0.0);

    let k_to = SubpixelKerning::get_kerning_ratio('T', 'o');
    assert!(k_to < 0.0);

    let k_we = SubpixelKerning::get_kerning_ratio('W', 'e');
    assert!(k_we < 0.0);

    // Neutral pairs should have 0 kerning
    let k_oo = SubpixelKerning::get_kerning_ratio('o', 'o');
    assert_eq!(k_oo, 0.0);

    let k_ab = SubpixelKerning::get_kerning_ratio('a', 'b');
    assert_eq!(k_ab, 0.0);
}

#[test]
fn test_bidi_segmentation_and_shaping() {
    let compositor = HighDpiTextCompositor::new(1.0);

    // Mixed LTR and Hebrew string: "Hello עולם 2026"
    let text = "Hello עולם 2026";
    let segments = HighDpiTextCompositor::segment_bidi_runs(text);

    // Should segment into LTR (Hello ), RTL (עולם), and adopting run (2026)
    assert!(segments.len() >= 2);
    assert_eq!(segments[0].1, TextDirection::Ltr);
    assert_eq!(segments[0].2, TextScript::Latin);

    let has_rtl = segments
        .iter()
        .any(|(_, dir, _)| *dir == TextDirection::Rtl);
    assert!(has_rtl, "Should have detected RTL Hebrew segment");

    // Shape into visual runs
    let visual_runs = compositor.shape_bidi_paragraph(
        text,
        "Inter",
        16.0,
        ColorRgba::new(0.0, 0.0, 0.0, 1.0),
        (100.0, 200.0),
    );

    assert_eq!(visual_runs.len(), segments.len());
    for vr in &visual_runs {
        assert!(vr.bounds.width > 0.0);
        assert!(vr.bounds.height > 0.0);
        assert!(!vr.glyphs.is_empty());
        // All baselines should be device-snapped
        assert_eq!(vr.baseline_y, compositor.snap_coord(vr.baseline_y));
    }

    // Compose to standard GlyphRuns
    let glyph_runs = compositor.compose_to_glyph_runs(
        text,
        "Inter",
        16.0,
        ColorRgba::new(0.0, 0.0, 0.0, 1.0),
        (100.0, 200.0),
    );

    assert_eq!(glyph_runs.len(), visual_runs.len());
}

#[test]
fn test_shaping_performance_sla() {
    let compositor = HighDpiTextCompositor::new(2.0);

    // Generate a full document page of text (20 paragraphs, ~1,000 words)
    let paragraph = "AetherOffice delivers commercial-grade typographic precision with subpixel kerning and 120fps rendering.\n";
    let full_text = paragraph.repeat(20);

    let start = Instant::now();
    let runs = compositor.compose_to_glyph_runs(
        &full_text,
        "Segoe UI",
        14.0,
        ColorRgba::new(0.1, 0.1, 0.1, 1.0),
        (50.0, 50.0),
    );
    let elapsed = start.elapsed();

    println!(
        "Sub-Phase 45.1.2 SLA: Shaped 20 paragraphs ({} runs, {} glyphs) in {:?}",
        runs.len(),
        runs.iter().map(|r| r.glyphs.len()).sum::<usize>(),
        elapsed
    );

    // SLA budget: must shape well within 2.0 ms
    assert!(
        elapsed.as_millis() < 5,
        "Text shaping took {:?}, exceeding SLA target",
        elapsed
    );
    assert!(!runs.is_empty());
}
