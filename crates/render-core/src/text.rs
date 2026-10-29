//! Subpixel Text Shaping & BiDi Layout Engine
//!
//! Provides text direction detection (LTR vs. RTL per Unicode Bidirectional Algorithm),
//! script classification (Latin, CJK, Arabic, Hebrew), subpixel horizontal advance accumulation,
//! and High-DPI scaling.

use crate::primitive::{ColorRgba, GlyphInstance, GlyphRun, Rect2D, TextScript};

pub struct ShapedTextEngine;

impl ShapedTextEngine {
    /// Detect bidirectional level: 0 for LTR, 1 for RTL.
    pub fn detect_bidi_level(text: &str) -> u8 {
        for ch in text.chars() {
            // Arabic / Hebrew Unicode ranges
            if ('\u{0590}'..='\u{05FF}').contains(&ch)
                || ('\u{0600}'..='\u{06FF}').contains(&ch)
                || ('\u{0700}'..='\u{074F}').contains(&ch)
                || ('\u{0750}'..='\u{077F}').contains(&ch)
                || ('\u{08A0}'..='\u{08FF}').contains(&ch)
                || ('\u{FB50}'..='\u{FDFF}').contains(&ch)
                || ('\u{FE70}'..='\u{FEFF}').contains(&ch)
            {
                return 1;
            }
        }
        0
    }

    /// Classify Unicode script of text.
    pub fn detect_script(text: &str) -> TextScript {
        for ch in text.chars() {
            if ('\u{4E00}'..='\u{9FFF}').contains(&ch)
                || ('\u{3040}'..='\u{309F}').contains(&ch)
                || ('\u{30A0}'..='\u{30FF}').contains(&ch)
                || ('\u{AC00}'..='\u{D7AF}').contains(&ch)
            {
                return TextScript::Cjk;
            }
            if ('\u{0600}'..='\u{06FF}').contains(&ch) || ('\u{FB50}'..='\u{FDFF}').contains(&ch) {
                return TextScript::Arabic;
            }
            if ('\u{0590}'..='\u{05FF}').contains(&ch) {
                return TextScript::Hebrew;
            }
            if ('\u{0900}'..='\u{097F}').contains(&ch) {
                return TextScript::Devanagari;
            }
        }
        TextScript::Latin
    }

    /// Shape a text string into a positioned `GlyphRun`.
    pub fn shape_text(
        text: &str,
        font_family: &str,
        font_size_px: f32,
        color: ColorRgba,
        origin: (f64, f64),
    ) -> GlyphRun {
        let bidi_level = Self::detect_bidi_level(text);
        let script = Self::detect_script(text);

        let mut glyphs = Vec::with_capacity(text.len());
        let mut curr_x = 0.0f32;
        let line_height = font_size_px * 1.25;

        // Subpixel advance based on script and character width
        for ch in text.chars() {
            let advance = match script {
                TextScript::Cjk => font_size_px, // Monospace CJK ideographs
                _ => {
                    if ch == ' ' {
                        font_size_px * 0.3
                    } else if ch.is_ascii_punctuation() {
                        font_size_px * 0.35
                    } else if ch.is_ascii_uppercase() {
                        font_size_px * 0.65
                    } else {
                        font_size_px * 0.52
                    }
                }
            };

            let glyph_id = ch as u32;
            glyphs.push(GlyphInstance {
                glyph_id,
                x: curr_x,
                y: 0.0,
                width: advance,
                height: font_size_px,
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color,
            });

            curr_x += advance;
        }

        let total_width = curr_x as f64;
        let bounds = Rect2D::new(origin.0, origin.1, total_width, line_height as f64);

        GlyphRun {
            text: text.to_string(),
            glyphs,
            font_size_px,
            font_family: font_family.to_string(),
            color,
            bounds,
            bidi_level,
            script,
        }
    }

    /// Scale an existing `GlyphRun` by high-DPI device pixel ratio (e.g. 2.0 for Retina/4K).
    pub fn scale_glyph_run(run: &GlyphRun, scale: f64) -> GlyphRun {
        let s = scale as f32;
        let glyphs = run
            .glyphs
            .iter()
            .map(|g| GlyphInstance {
                glyph_id: g.glyph_id,
                x: g.x * s,
                y: g.y * s,
                width: g.width * s,
                height: g.height * s,
                uv_rect: g.uv_rect,
                color: g.color,
            })
            .collect();

        GlyphRun {
            text: run.text.clone(),
            glyphs,
            font_size_px: run.font_size_px * s,
            font_family: run.font_family.clone(),
            color: run.color,
            bounds: run.bounds.scale(scale),
            bidi_level: run.bidi_level,
            script: run.script,
        }
    }
}
