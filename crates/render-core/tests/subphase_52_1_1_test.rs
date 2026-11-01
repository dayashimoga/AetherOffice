//! Integration Test Suite for Sub-Phase 52.1.1: Semantic Design Tokens & Ribbon Switcher
//!
//! GEC-v7.0 Stage 52 (Sub-Phase 52.1.1)
//!
//! Verifies:
//! - Default Light theme and Classic ribbon layout
//! - Theme switching across Light, Dark, HighContrastDark, HighContrastLight
//! - Ribbon layout modes (Classic, Compact, Collapsed) and row configurations
//! - WCAG 2.1 AA contrast ratio validation for standard themes (>= 4.5:1)
//! - High-contrast contrast ratio validation (>= 7.0:1)
//! - Semantic CSS variable string generation
//! - Dark mode surface token coverage (zero unstyled panels)
//! - Compact density metrics (button height, icon size)
//! - Contrast ratio mathematical precision
//! - Serialization and deserialization roundtrip for ThemeTokens and RibbonLayoutConfig

use render_core::theme::{
    DesignTokenEngine, RibbonLayoutConfig, RibbonLayoutMode, ThemeMode, ThemeTokens,
};

#[test]
fn test_default_theme_and_ribbon_mode() {
    let engine = DesignTokenEngine::new();
    assert_eq!(engine.active_theme(), ThemeMode::Light);
    assert_eq!(engine.ribbon_config().mode, RibbonLayoutMode::Classic);
    assert_eq!(engine.ribbon_config().height_px, 96.0);
    assert!(!engine.ribbon_config().is_single_row);
    assert!(engine.ribbon_config().show_group_headers);
}

#[test]
fn test_theme_switching_light_dark_high_contrast() {
    let mut engine = DesignTokenEngine::new();

    engine.set_theme(ThemeMode::Dark);
    assert_eq!(engine.active_theme(), ThemeMode::Dark);
    assert!(engine.active_theme().is_dark());
    assert!(!engine.active_theme().is_high_contrast());

    engine.set_theme(ThemeMode::HighContrastDark);
    assert_eq!(engine.active_theme(), ThemeMode::HighContrastDark);
    assert!(engine.active_theme().is_dark());
    assert!(engine.active_theme().is_high_contrast());

    engine.set_theme(ThemeMode::HighContrastLight);
    assert_eq!(engine.active_theme(), ThemeMode::HighContrastLight);
    assert!(!engine.active_theme().is_dark());
    assert!(engine.active_theme().is_high_contrast());
}

#[test]
fn test_ribbon_layout_modes_heights_and_rows() {
    let mut engine = DesignTokenEngine::new();

    // Classic: 96px, multi-row, headers visible
    engine.set_ribbon_mode(RibbonLayoutMode::Classic);
    let cfg_classic = engine.ribbon_config();
    assert_eq!(cfg_classic.height_px, 96.0);
    assert!(!cfg_classic.is_single_row);
    assert!(cfg_classic.show_group_headers);
    assert!(!cfg_classic.collapsed);

    // Compact: 44px, single-row, streamlined
    engine.set_ribbon_mode(RibbonLayoutMode::Compact);
    let cfg_compact = engine.ribbon_config();
    assert_eq!(cfg_compact.height_px, 44.0);
    assert!(cfg_compact.is_single_row);
    assert!(!cfg_compact.show_group_headers);
    assert!(!cfg_compact.collapsed);

    // Collapsed: 32px, tab-only
    engine.set_ribbon_mode(RibbonLayoutMode::Collapsed);
    let cfg_collapsed = engine.ribbon_config();
    assert_eq!(cfg_collapsed.height_px, 32.0);
    assert!(cfg_collapsed.collapsed);
}

#[test]
fn test_wcag_2_1_aa_contrast_ratios_standard() {
    let engine = DesignTokenEngine::new();

    let light_tokens = engine.evaluate_tokens(ThemeMode::Light, RibbonLayoutMode::Classic);
    assert!(
        light_tokens.contrast_ratio >= 4.5,
        "Light theme contrast ratio {} must be >= 4.5:1",
        light_tokens.contrast_ratio
    );
    assert!(engine.validate_wcag_aa(ThemeMode::Light));

    let dark_tokens = engine.evaluate_tokens(ThemeMode::Dark, RibbonLayoutMode::Classic);
    assert!(
        dark_tokens.contrast_ratio >= 4.5,
        "Dark theme contrast ratio {} must be >= 4.5:1",
        dark_tokens.contrast_ratio
    );
    assert!(engine.validate_wcag_aa(ThemeMode::Dark));
}

#[test]
fn test_wcag_2_1_aa_contrast_ratios_high_contrast() {
    let engine = DesignTokenEngine::new();

    let hc_dark_tokens =
        engine.evaluate_tokens(ThemeMode::HighContrastDark, RibbonLayoutMode::Classic);
    assert!(
        hc_dark_tokens.contrast_ratio >= 7.0,
        "High-contrast dark contrast ratio {} must be >= 7.0:1",
        hc_dark_tokens.contrast_ratio
    );
    assert!(engine.validate_wcag_aa(ThemeMode::HighContrastDark));

    let hc_light_tokens =
        engine.evaluate_tokens(ThemeMode::HighContrastLight, RibbonLayoutMode::Classic);
    assert!(
        hc_light_tokens.contrast_ratio >= 7.0,
        "High-contrast light contrast ratio {} must be >= 7.0:1",
        hc_light_tokens.contrast_ratio
    );
    assert!(engine.validate_wcag_aa(ThemeMode::HighContrastLight));
}

#[test]
fn test_css_variables_generation() {
    let engine = DesignTokenEngine::new();
    let css = engine.to_css_variables(ThemeMode::Dark, RibbonLayoutMode::Compact);

    assert!(css.contains(":root {"));
    assert!(css.contains("--bg-primary: #0f172a;"));
    assert!(css.contains("--bg-surface: #1e293b;"));
    assert!(css.contains("--text-primary: #f8fafc;"));
    assert!(css.contains("--primary-accent: #38bdf8;"));
    assert!(css.contains("--ribbon-height: 44px;"));
    assert!(css.contains("--button-height: 26px;"));
    assert!(css.contains("--icon-size: 16px;"));
}

#[test]
fn test_semantic_color_tokens_contrast_dark_mode() {
    let engine = DesignTokenEngine::new();
    let tokens = engine.evaluate_tokens(ThemeMode::Dark, RibbonLayoutMode::Classic);

    // Verify dark mode panels are not white (#ffffff)
    assert_ne!(tokens.bg_surface, "#ffffff");
    assert_ne!(tokens.bg_panel, "#ffffff");
    assert_ne!(tokens.bg_canvas_backdrop, "#ffffff");
    assert_ne!(tokens.bg_toolbar, "#ffffff");
    assert_eq!(tokens.focus_ring, "#38bdf8");
}

#[test]
fn test_ribbon_compact_density_metrics() {
    let engine = DesignTokenEngine::new();

    let classic = engine.evaluate_tokens(ThemeMode::Light, RibbonLayoutMode::Classic);
    let compact = engine.evaluate_tokens(ThemeMode::Light, RibbonLayoutMode::Compact);

    assert!(classic.ribbon_height > compact.ribbon_height);
    assert!(classic.button_height > compact.button_height);
    assert!(classic.icon_size > compact.icon_size);
    assert_eq!(compact.button_height, 26.0);
    assert_eq!(compact.icon_size, 16.0);
}

#[test]
fn test_hex_luminance_and_contrast_ratio_math() {
    // Pure black (#000000) and pure white (#ffffff) must have 21.0:1 contrast ratio
    let ratio = DesignTokenEngine::calculate_contrast_ratio("#000000", "#ffffff");
    assert!((ratio - 21.0).abs() < 0.1);

    // Same color must have 1.0:1 contrast ratio
    let same = DesignTokenEngine::calculate_contrast_ratio("#1e293b", "#1e293b");
    assert!((same - 1.0).abs() < 0.01);
}

#[test]
fn test_design_token_engine_state_roundtrip() {
    let engine = DesignTokenEngine::new();
    let tokens = engine.evaluate_tokens(ThemeMode::Dark, RibbonLayoutMode::Compact);

    let json = serde_json::to_string(&tokens).expect("Serialize ThemeTokens");
    let deserialized: ThemeTokens = serde_json::from_str(&json).expect("Deserialize ThemeTokens");

    assert_eq!(tokens, deserialized);

    let cfg = RibbonLayoutConfig {
        mode: RibbonLayoutMode::Compact,
        collapsed: false,
        height_px: 44.0,
        is_single_row: true,
        show_group_headers: false,
    };
    let cfg_json = serde_json::to_string(&cfg).expect("Serialize RibbonLayoutConfig");
    let cfg_deserialized: RibbonLayoutConfig =
        serde_json::from_str(&cfg_json).expect("Deserialize RibbonLayoutConfig");

    assert_eq!(cfg, cfg_deserialized);
}
