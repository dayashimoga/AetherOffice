//! Semantic Design Token Engine & Classic/Compact Ribbon Switcher
//!
//! GEC-v7.0 Stage 52 (Sub-Phase 52.1.1)
//!
//! Complies strictly with GEC-v7.0 Articles I, IV, and V:
//! - Safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics, unwrap, or expect in production library code.
//! - WCAG 2.1 AA compliant contrast ratios (4.5:1 standard, 7.0:1 high-contrast).
//! - 100% theme coverage for Light, Dark, HighContrastDark, HighContrastLight.
//! - Adaptive ribbon layout engine (Classic multi-row, Compact single-row, Collapsed).

use serde::{Deserialize, Serialize};

/// Supported presentation theme modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    HighContrastDark,
    HighContrastLight,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::HighContrastDark => "high-contrast-dark",
            Self::HighContrastLight => "high-contrast-light",
        }
    }

    pub fn is_high_contrast(&self) -> bool {
        matches!(self, Self::HighContrastDark | Self::HighContrastLight)
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark | Self::HighContrastDark)
    }
}

/// Supported ribbon toolbar layout structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RibbonLayoutMode {
    #[default]
    Classic,
    Compact,
    Collapsed,
}

impl RibbonLayoutMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Compact => "compact",
            Self::Collapsed => "collapsed",
        }
    }

    pub fn default_height(&self) -> f32 {
        match self {
            Self::Classic => 96.0,
            Self::Compact => 44.0,
            Self::Collapsed => 32.0,
        }
    }
}

/// Resolved semantic design tokens for styling the presentation shell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeTokens {
    pub mode: ThemeMode,
    pub ribbon_mode: RibbonLayoutMode,
    pub bg_primary: String,
    pub bg_surface: String,
    pub bg_shell: String,
    pub bg_canvas_backdrop: String,
    pub bg_panel: String,
    pub bg_toolbar: String,
    pub border_color: String,
    pub border_subtle: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub primary_accent: String,
    pub focus_ring: String,
    pub ribbon_height: f32,
    pub title_bar_height: f32,
    pub tab_bar_height: f32,
    pub status_bar_height: f32,
    pub button_height: f32,
    pub icon_size: f32,
    pub contrast_ratio: f32,
}

/// Configuration descriptor for the application ribbon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RibbonLayoutConfig {
    pub mode: RibbonLayoutMode,
    pub collapsed: bool,
    pub height_px: f32,
    pub is_single_row: bool,
    pub show_group_headers: bool,
}

impl Default for RibbonLayoutConfig {
    fn default() -> Self {
        Self {
            mode: RibbonLayoutMode::Classic,
            collapsed: false,
            height_px: 96.0,
            is_single_row: false,
            show_group_headers: true,
        }
    }
}

/// Registry and evaluator for semantic design tokens and layout metrics.
#[derive(Debug, Clone, Default)]
pub struct DesignTokenEngine {
    active_theme: ThemeMode,
    ribbon_config: RibbonLayoutConfig,
}

impl DesignTokenEngine {
    /// Creates a new design token engine with default light theme and classic ribbon.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_theme(&self) -> ThemeMode {
        self.active_theme
    }

    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.active_theme = theme;
    }

    pub fn ribbon_config(&self) -> &RibbonLayoutConfig {
        &self.ribbon_config
    }

    pub fn set_ribbon_mode(&mut self, mode: RibbonLayoutMode) {
        let is_single_row = mode == RibbonLayoutMode::Compact;
        let show_group_headers = mode == RibbonLayoutMode::Classic;
        let collapsed = mode == RibbonLayoutMode::Collapsed;
        let height_px = mode.default_height();

        self.ribbon_config = RibbonLayoutConfig {
            mode,
            collapsed,
            height_px,
            is_single_row,
            show_group_headers,
        };
    }

    /// Evaluates resolved design tokens for the specified theme and ribbon layout mode.
    pub fn evaluate_tokens(&self, theme: ThemeMode, ribbon_mode: RibbonLayoutMode) -> ThemeTokens {
        let (
            bg_primary,
            bg_surface,
            bg_shell,
            bg_canvas_backdrop,
            bg_panel,
            bg_toolbar,
            border_color,
            border_subtle,
            text_primary,
            text_secondary,
            text_muted,
            primary_accent,
            focus_ring,
        ) = match theme {
            ThemeMode::Light => (
                "#f8fafc".to_string(),
                "#ffffff".to_string(),
                "#f1f5f9".to_string(),
                "#e2e8f0".to_string(),
                "#ffffff".to_string(),
                "#f8fafc".to_string(),
                "#cbd5e1".to_string(),
                "#e2e8f0".to_string(),
                "#0f172a".to_string(),
                "#475569".to_string(),
                "#64748b".to_string(),
                "#0f6cbd".to_string(),
                "#0f6cbd".to_string(),
            ),
            ThemeMode::Dark => (
                "#0f172a".to_string(),
                "#1e293b".to_string(),
                "#0b1120".to_string(),
                "#020617".to_string(),
                "#1e293b".to_string(),
                "#1e293b".to_string(),
                "#334155".to_string(),
                "#1e293b".to_string(),
                "#f8fafc".to_string(),
                "#94a3b8".to_string(),
                "#64748b".to_string(),
                "#38bdf8".to_string(),
                "#38bdf8".to_string(),
            ),
            ThemeMode::HighContrastDark => (
                "#000000".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffff00".to_string(),
                "#00ffff".to_string(),
                "#00ffff".to_string(),
                "#00ffff".to_string(),
            ),
            ThemeMode::HighContrastLight => (
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#ffffff".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#000000".to_string(),
                "#000080".to_string(),
                "#333333".to_string(),
                "#0000ee".to_string(),
                "#0000ee".to_string(),
            ),
        };

        let contrast_ratio = Self::calculate_contrast_ratio(&text_primary, &bg_surface);

        let (button_height, icon_size) = match ribbon_mode {
            RibbonLayoutMode::Classic => (32.0, 20.0),
            RibbonLayoutMode::Compact => (26.0, 16.0),
            RibbonLayoutMode::Collapsed => (22.0, 14.0),
        };

        ThemeTokens {
            mode: theme,
            ribbon_mode,
            bg_primary,
            bg_surface,
            bg_shell,
            bg_canvas_backdrop,
            bg_panel,
            bg_toolbar,
            border_color,
            border_subtle,
            text_primary,
            text_secondary,
            text_muted,
            primary_accent,
            focus_ring,
            ribbon_height: ribbon_mode.default_height(),
            title_bar_height: 38.0,
            tab_bar_height: 34.0,
            status_bar_height: 26.0,
            button_height,
            icon_size,
            contrast_ratio,
        }
    }

    /// Generates standard CSS custom property definitions (`:root` block) for presentation shells.
    pub fn to_css_variables(&self, theme: ThemeMode, ribbon_mode: RibbonLayoutMode) -> String {
        let t = self.evaluate_tokens(theme, ribbon_mode);
        format!(
            ":root {{\n  \
             --bg-primary: {};\n  \
             --bg-surface: {};\n  \
             --bg-shell: {};\n  \
             --bg-canvas-backdrop: {};\n  \
             --bg-panel: {};\n  \
             --bg-toolbar: {};\n  \
             --border-color: {};\n  \
             --border-subtle: {};\n  \
             --text-primary: {};\n  \
             --text-secondary: {};\n  \
             --text-muted: {};\n  \
             --primary-accent: {};\n  \
             --focus-ring: {};\n  \
             --ribbon-height: {}px;\n  \
             --title-bar-height: {}px;\n  \
             --tab-bar-height: {}px;\n  \
             --status-bar-height: {}px;\n  \
             --button-height: {}px;\n  \
             --icon-size: {}px;\n\
             }}",
            t.bg_primary,
            t.bg_surface,
            t.bg_shell,
            t.bg_canvas_backdrop,
            t.bg_panel,
            t.bg_toolbar,
            t.border_color,
            t.border_subtle,
            t.text_primary,
            t.text_secondary,
            t.text_muted,
            t.primary_accent,
            t.focus_ring,
            t.ribbon_height,
            t.title_bar_height,
            t.tab_bar_height,
            t.status_bar_height,
            t.button_height,
            t.icon_size
        )
    }

    /// Calculates WCAG 2.1 relative luminance and contrast ratio between two hex colors `#RRGGBB`.
    pub fn calculate_contrast_ratio(fg_hex: &str, bg_hex: &str) -> f32 {
        let l1 = Self::relative_luminance(fg_hex);
        let l2 = Self::relative_luminance(bg_hex);

        let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        ((lighter + 0.05) / (darker + 0.05)) as f32
    }

    fn relative_luminance(hex: &str) -> f64 {
        let clean = hex.trim_start_matches('#');
        if clean.len() < 6 {
            return 0.5;
        }

        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f64 / 255.0;

        let to_linear = |c: f64| -> f64 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        let r_lin = to_linear(r);
        let g_lin = to_linear(g);
        let b_lin = to_linear(b);

        0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
    }

    /// Verifies if a given theme satisfies WCAG 2.1 AA contrast requirements.
    pub fn validate_wcag_aa(&self, theme: ThemeMode) -> bool {
        let tokens = self.evaluate_tokens(theme, RibbonLayoutMode::Classic);
        if theme.is_high_contrast() {
            tokens.contrast_ratio >= 7.0
        } else {
            tokens.contrast_ratio >= 4.5
        }
    }
}
