//! PowerPoint Display List Compiler
//!
//! Compiles PresentationML slide canvas, background, shape hierarchies, and dynamic
//! animated/morphed scene states into GPU display lists with aspect-ratio preservation.

use crate::primitive::{
    BezierPathBatch, ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D,
};
use crate::text::ShapedTextEngine;
use ppt_engine::{MorphedSlideState, Shape, Slide};

/// Slide canvas visual definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlideCanvasConfig {
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub background_color: ColorRgba,
}

impl Default for SlideCanvasConfig {
    fn default() -> Self {
        Self {
            canvas_width: 1920.0,
            canvas_height: 1080.0,
            background_color: ColorRgba::WHITE,
        }
    }
}

pub struct PptDisplayCompiler;

impl PptDisplayCompiler {
    /// Calculate aspect-ratio centered origin and dimensions of the slide canvas within the viewport.
    pub fn compute_slide_viewport_transform(
        canvas_cfg: &SlideCanvasConfig,
        viewport: &Rect2D,
    ) -> (Rect2D, f64) {
        let canvas_aspect = canvas_cfg.canvas_width / canvas_cfg.canvas_height;
        let viewport_aspect = viewport.width / viewport.height;

        let (dest_w, dest_h) = if viewport_aspect > canvas_aspect {
            // Viewport is wider than slide: fit height
            let h = viewport.height;
            let w = h * canvas_aspect;
            (w, h)
        } else {
            // Viewport is taller than slide: fit width
            let w = viewport.width;
            let h = w / canvas_aspect;
            (w, h)
        };

        let dest_x = viewport.x + (viewport.width - dest_w) / 2.0;
        let dest_y = viewport.y + (viewport.height - dest_h) / 2.0;

        let scale = dest_w / canvas_cfg.canvas_width;
        (Rect2D::new(dest_x, dest_y, dest_w, dest_h), scale)
    }

    /// Compile a slide and optional animated morphed state into a `DisplayList`.
    pub fn compile_slide(
        _slide: &Slide,
        shapes: &[Shape],
        morphed_state: Option<&MorphedSlideState>,
        canvas_cfg: &SlideCanvasConfig,
        viewport: &Rect2D,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();
        let (slide_rect, scale) = Self::compute_slide_viewport_transform(canvas_cfg, viewport);

        // 1. Dark letterbox background for area outside slide
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            *viewport,
            ColorRgba::new(0.12, 0.12, 0.12, 1.0),
        )));

        // 2. Slide drop shadow
        let shadow_rect = Rect2D::new(
            slide_rect.x + 8.0,
            slide_rect.y + 8.0,
            slide_rect.width,
            slide_rect.height,
        );
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            shadow_rect,
            ColorRgba::new(0.0, 0.0, 0.0, 0.4),
        )));

        // 3. Slide canvas background
        display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
            slide_rect,
            canvas_cfg.background_color,
        )));

        // 4. Clip to slide canvas boundaries
        display_list.push(DisplayCommand::PushClip(slide_rect));

        // 5. If animated / morphed state is provided, evaluate morphed shapes
        if let Some(morph) = morphed_state {
            for mshape in &morph.shapes {
                let sx = slide_rect.x + mshape.transform.x * scale;
                let sy = slide_rect.y + mshape.transform.y * scale;
                let sw = mshape.transform.width * scale;
                let sh = mshape.transform.height * scale;
                let srect = Rect2D::new(sx, sy, sw, sh);

                let alpha = (mshape.opacity as f32).clamp(0.0, 1.0);
                if alpha <= 0.001 {
                    continue;
                }

                // If path is present, emit vector path
                if let Some(ref path) = mshape.geometry_path {
                    display_list.push(DisplayCommand::DrawPath(BezierPathBatch {
                        path: path.clone(),
                        fill_color: Some(ColorRgba::new(0.2, 0.4, 0.8, alpha)),
                        stroke_color: Some(ColorRgba::new(0.1, 0.2, 0.5, alpha)),
                        stroke_width: 2.0 * (scale as f32),
                        bounds: srect,
                    }));
                } else {
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::rounded(
                        srect,
                        ColorRgba::new(0.2, 0.4, 0.8, alpha),
                        4.0 * (scale as f32),
                    )));
                }

                // Render morphed text elements
                for te in &mshape.text_elements {
                    let tx = slide_rect.x + te.x * scale;
                    let ty = slide_rect.y + te.y * scale;
                    let talp = ((te.opacity * mshape.opacity) as f32).clamp(0.0, 1.0);
                    if talp > 0.001 {
                        let run = ShapedTextEngine::shape_text(
                            &te.text,
                            "Calibri",
                            te.font_size_pt as f32 * (scale as f32),
                            ColorRgba::new(1.0, 1.0, 1.0, talp),
                            (tx, ty),
                        );
                        display_list.push(DisplayCommand::DrawText(run));
                    }
                }
            }
        } else {
            // 6. Static shape tree rendering
            for s in shapes {
                if let Some(ref xfrm) = s.properties.transform {
                    let sx = slide_rect.x + (xfrm.x as f64) * scale;
                    let sy = slide_rect.y + (xfrm.y as f64) * scale;
                    let sw = (xfrm.cx as f64) * scale;
                    let sh = (xfrm.cy as f64) * scale;
                    let srect = Rect2D::new(sx, sy, sw, sh);

                    // Draw shape body
                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                        srect,
                        ColorRgba::new(0.18, 0.45, 0.72, 1.0),
                        ColorRgba::new(0.10, 0.30, 0.55, 1.0),
                        1.5 * (scale as f32),
                    )));

                    // Draw text body
                    if let Some(ref tb) = s.text_body {
                        let mut line_y = sy + 10.0 * scale;
                        for p in &tb.paragraphs {
                            for r in &p.runs {
                                if let ppt_engine::TextRunElement::Text(ref text, _) = r {
                                    let run = ShapedTextEngine::shape_text(
                                        text,
                                        "Calibri",
                                        16.0 * (scale as f32),
                                        ColorRgba::WHITE,
                                        (sx + 10.0 * scale, line_y),
                                    );
                                    display_list.push(DisplayCommand::DrawText(run));
                                    line_y += 20.0 * scale;
                                }
                            }
                        }
                    }
                }
            }
        }

        display_list.push(DisplayCommand::PopClip);
        display_list
    }
}
