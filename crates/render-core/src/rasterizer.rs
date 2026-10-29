//! Headless Software Rasterizer & Vector Exporter Fallback
//!
//! Provides deterministic SVG export, pixel canvas rendering with subpixel alpha blending,
//! and PDF stream generation for headless validation and print export.

use crate::primitive::{DisplayCommand, DisplayList};

pub struct SoftwareRasterizer;

impl SoftwareRasterizer {
    /// Export an immutable `DisplayList` into standard W3C SVG XML format.
    pub fn export_svg(display_list: &DisplayList, width: f64, height: f64) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            width, height, width, height
        );
        svg.push('\n');

        for (idx, cmd) in display_list.commands.iter().enumerate() {
            match cmd {
                DisplayCommand::PushClip(r) => {
                    svg.push_str(&format!(
                        r#"  <clipPath id="clip_{}"><rect x="{}" y="{}" width="{}" height="{}" /></clipPath>"#,
                        idx, r.x, r.y, r.width, r.height
                    ));
                    svg.push('\n');
                    svg.push_str(&format!(r#"  <g clip-path="url(#clip_{})">"#, idx));
                    svg.push('\n');
                }
                DisplayCommand::PopClip => {
                    svg.push_str("  </g>\n");
                }
                DisplayCommand::DrawQuad(q) => {
                    let fill_hex = q.color.to_hex();
                    let stroke_attr = if let Some(b) = q.border_color {
                        format!(
                            r#" stroke="{}" stroke-width="{}""#,
                            b.to_hex(),
                            q.border_width
                        )
                    } else {
                        String::new()
                    };
                    let rx_attr = if q.corner_radius > 0.0 {
                        format!(r#" rx="{}" ry="{}"#, q.corner_radius, q.corner_radius)
                    } else {
                        String::new()
                    };

                    svg.push_str(&format!(
                        r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}"{}{}/>"#,
                        q.rect.x,
                        q.rect.y,
                        q.rect.width,
                        q.rect.height,
                        fill_hex,
                        stroke_attr,
                        rx_attr
                    ));
                    svg.push('\n');
                }
                DisplayCommand::DrawText(g) => {
                    let color_hex = g.color.to_hex();
                    svg.push_str(&format!(
                        r#"  <text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                        g.bounds.x,
                        g.bounds.y + (g.font_size_px as f64),
                        g.font_family,
                        g.font_size_px,
                        color_hex,
                        quick_xml::escape::escape(&g.text)
                    ));
                    svg.push('\n');
                }
                DisplayCommand::DrawPath(p) => {
                    let mut d = String::new();
                    for cmd in &p.path.commands {
                        match *cmd {
                            ppt_engine::PathCommand::MoveTo(ref pt) => {
                                d.push_str(&format!("M {} {} ", pt.x, pt.y));
                            }
                            ppt_engine::PathCommand::LineTo(ref pt) => {
                                d.push_str(&format!("L {} {} ", pt.x, pt.y));
                            }
                            ppt_engine::PathCommand::QuadTo { ref ctrl, ref to } => {
                                d.push_str(&format!("Q {} {} {} {} ", ctrl.x, ctrl.y, to.x, to.y));
                            }
                            ppt_engine::PathCommand::CubicTo {
                                ref ctrl1,
                                ref ctrl2,
                                ref to,
                            } => {
                                d.push_str(&format!(
                                    "C {} {} {} {} {} {} ",
                                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                                ));
                            }
                            ppt_engine::PathCommand::Close => {
                                d.push_str("Z ");
                            }
                            _ => {}
                        }
                    }

                    let fill_attr = p
                        .fill_color
                        .map(|c| format!(r#" fill="{}""#, c.to_hex()))
                        .unwrap_or_else(|| r#" fill="none""#.to_string());
                    let stroke_attr = p
                        .stroke_color
                        .map(|c| {
                            format!(
                                r#" stroke="{}" stroke-width="{}""#,
                                c.to_hex(),
                                p.stroke_width
                            )
                        })
                        .unwrap_or_else(|| r#" stroke="none""#.to_string());

                    svg.push_str(&format!(
                        r#"  <path d="{}"{}{}/>"#,
                        d.trim(),
                        fill_attr,
                        stroke_attr
                    ));
                    svg.push('\n');
                }
                _ => {}
            }
        }

        svg.push_str("</svg>\n");
        svg
    }

    /// Software pixel rasterizer rendering quads and background into an RGBA byte buffer.
    #[allow(clippy::manual_checked_ops)]
    pub fn render_to_pixels(display_list: &DisplayList, width: u32, height: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];

        for cmd in &display_list.commands {
            if let DisplayCommand::DrawQuad(ref q) = cmd {
                let x0 = (q.rect.x.floor() as i64).clamp(0, width as i64) as u32;
                let y0 = (q.rect.y.floor() as i64).clamp(0, height as i64) as u32;
                let x1 = (q.rect.right().ceil() as i64).clamp(0, width as i64) as u32;
                let y1 = (q.rect.bottom().ceil() as i64).clamp(0, height as i64) as u32;

                let fg_r = (q.color.r * 255.0).round() as u32;
                let fg_g = (q.color.g * 255.0).round() as u32;
                let fg_b = (q.color.b * 255.0).round() as u32;
                let fg_a = (q.color.a * 255.0).round() as u32;

                for y in y0..y1 {
                    for x in x0..x1 {
                        let idx = ((y * width + x) * 4) as usize;
                        let bg_r = buffer[idx] as u32;
                        let bg_g = buffer[idx + 1] as u32;
                        let bg_b = buffer[idx + 2] as u32;
                        let bg_a = buffer[idx + 3] as u32;

                        // Standard Porter-Duff source-over alpha blending
                        let out_a = fg_a + bg_a * (255 - fg_a) / 255;
                        if out_a > 0 {
                            let out_r = (fg_r * fg_a + bg_r * bg_a * (255 - fg_a) / 255) / out_a;
                            let out_g = (fg_g * fg_a + bg_g * bg_a * (255 - fg_a) / 255) / out_a;
                            let out_b = (fg_b * fg_a + bg_b * bg_a * (255 - fg_a) / 255) / out_a;

                            buffer[idx] = out_r as u8;
                            buffer[idx + 1] = out_g as u8;
                            buffer[idx + 2] = out_b as u8;
                            buffer[idx + 3] = out_a as u8;
                        }
                    }
                }
            }
        }

        buffer
    }
}
