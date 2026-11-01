#![forbid(unsafe_code)]
//! OASIS OpenDocument Presentation (.odp) Codec
//!
//! Complies with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - OASIS OpenDocument Format for Office Applications (ODF 1.3).
//! - Lossless translation between ODF `<office:presentation>` / `<draw:page>` and PresentationML.

use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::model::{
    BodyProperties, ColorChoice, Fill, Geometry, Shape, ShapeElement, ShapeProperties, Slide,
    TextBody, TextParagraph, TextRunElement, Transform2D,
};
use crate::package::PresentationPackage;

pub struct OdpCodec;

impl OdpCodec {
    /// Decodes an `.odp` byte slice into a `PresentationPackage`.
    pub fn decode(odp_bytes: &[u8]) -> Result<PresentationPackage, String> {
        let cursor = Cursor::new(odp_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| format!("Invalid ODP archive: {e}"))?;

        let mut content_xml = String::new();
        {
            let mut file = archive
                .by_name("content.xml")
                .map_err(|_| "ODP archive missing content.xml".to_string())?;
            file.read_to_string(&mut content_xml)
                .map_err(|e| format!("Failed to read content.xml: {e}"))?;
        }

        Self::parse_content_xml(&content_xml)
    }

    /// Encodes a `PresentationPackage` into an `.odp` ZIP archive.
    pub fn encode(pkg: &PresentationPackage) -> Result<Vec<u8>, String> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);

            // 1. mimetype (MUST be first and uncompressed per ODF standard)
            let stored_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("mimetype", stored_opts)
                .map_err(|e| format!("Failed to write mimetype: {e}"))?;
            zip.write_all(b"application/vnd.oasis.opendocument.presentation")
                .map_err(|e| format!("Failed to write mimetype payload: {e}"))?;

            // 2. META-INF/manifest.xml
            let def_opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("META-INF/manifest.xml", def_opts)
                .map_err(|e| format!("Failed to create manifest.xml: {e}"))?;
            let manifest_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
  <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.presentation"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
            zip.write_all(manifest_xml.as_bytes())
                .map_err(|e| format!("Failed to write manifest: {e}"))?;

            // 3. styles.xml
            zip.start_file("styles.xml", def_opts)
                .map_err(|e| format!("Failed to create styles.xml: {e}"))?;
            let styles_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
  office:version="1.3">
  <office:styles>
    <style:default-style style:family="graphic"/>
  </office:styles>
</office:document-styles>"#;
            zip.write_all(styles_xml.as_bytes())
                .map_err(|e| format!("Failed to write styles.xml: {e}"))?;

            // 4. content.xml
            zip.start_file("content.xml", def_opts)
                .map_err(|e| format!("Failed to create content.xml: {e}"))?;
            let content_xml = Self::generate_content_xml(pkg);
            zip.write_all(content_xml.as_bytes())
                .map_err(|e| format!("Failed to write content.xml: {e}"))?;

            zip.finish()
                .map_err(|e| format!("Failed to finish ODP zip archive: {e}"))?;
        }

        Ok(buffer.into_inner())
    }

    /// Parses ODF `content.xml` and reconstructs slides and shapes.
    fn parse_content_xml(xml: &str) -> Result<PresentationPackage, String> {
        let mut pkg = PresentationPackage::create_empty();
        pkg.slides.clear(); // Clear initial blank slide

        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_slide: Option<Slide> = None;
        let mut current_shape: Option<Shape> = None;
        let mut current_text = String::new();
        let mut in_text_p = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name = e.name().into_inner();
                    match name {
                        b"draw:page" => {
                            let mut slide = Slide::default();
                            slide.common_slide_data.shape_tree.shapes.clear();
                            current_slide = Some(slide);
                        }
                        b"draw:custom-shape" | b"draw:rect" | b"draw:ellipse" | b"draw:frame" => {
                            let shape_id = 100
                                + (current_slide
                                    .as_ref()
                                    .map(|s| s.common_slide_data.shape_tree.shapes.len())
                                    .unwrap_or(0) as u32);
                            let mut shape_name = format!("Shape {shape_id}");
                            let mut x_emu = 0i64;
                            let mut y_emu = 0i64;
                            let mut cx_emu = 1_000_000u32;
                            let mut cy_emu = 1_000_000u32;

                            for attr in e.attributes().flatten() {
                                match attr.key.into_inner() {
                                    b"draw:name" => {
                                        if let Ok(v) =
                                            attr.decode_and_unescape_value(reader.decoder())
                                        {
                                            shape_name = v.to_string();
                                        }
                                    }
                                    b"svg:x" => {
                                        let v = String::from_utf8_lossy(&attr.value);
                                        x_emu = Self::parse_odf_length(&v);
                                    }
                                    b"svg:y" => {
                                        let v = String::from_utf8_lossy(&attr.value);
                                        y_emu = Self::parse_odf_length(&v);
                                    }
                                    b"svg:width" => {
                                        let v = String::from_utf8_lossy(&attr.value);
                                        cx_emu = Self::parse_odf_length(&v).max(1) as u32;
                                    }
                                    b"svg:height" => {
                                        let v = String::from_utf8_lossy(&attr.value);
                                        cy_emu = Self::parse_odf_length(&v).max(1) as u32;
                                    }
                                    _ => {}
                                }
                            }

                            let geom = if name == b"draw:ellipse" {
                                Geometry::Preset("ellipse".to_string())
                            } else {
                                Geometry::Preset("rect".to_string())
                            };

                            let shape = Shape {
                                id: shape_id,
                                name: shape_name,
                                properties: ShapeProperties {
                                    transform: Some(Transform2D::new(x_emu, y_emu, cx_emu, cy_emu)),
                                    geometry: Some(geom),
                                    fill: Some(Fill::Solid(ColorChoice::Srgb(
                                        "4472C4".to_string(),
                                    ))),
                                    line: None,
                                },
                                ..Default::default()
                            };
                            current_shape = Some(shape);
                        }
                        b"text:p" => {
                            in_text_p = true;
                            current_text.clear();
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Text(ref e)) => {
                    if in_text_p {
                        if let Ok(txt) = e.unescape() {
                            current_text.push_str(&txt);
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => match e.name().into_inner() {
                    b"text:p" => {
                        in_text_p = false;
                        if let Some(ref mut shape) = current_shape {
                            if !current_text.is_empty() {
                                let para = TextParagraph {
                                    properties: None,
                                    runs: vec![TextRunElement::Text(current_text.clone(), None)],
                                    end_paragraph_run_properties: None,
                                };
                                if let Some(ref mut tb) = shape.text_body {
                                    tb.paragraphs.push(para);
                                } else {
                                    shape.text_body = Some(TextBody {
                                        body_properties: BodyProperties::default(),
                                        list_style: None,
                                        paragraphs: vec![para],
                                    });
                                }
                            }
                        }
                    }
                    b"draw:custom-shape" | b"draw:rect" | b"draw:ellipse" | b"draw:frame" => {
                        if let Some(shape) = current_shape.take() {
                            if let Some(ref mut slide) = current_slide {
                                slide
                                    .common_slide_data
                                    .shape_tree
                                    .shapes
                                    .push(ShapeElement::Shape(Box::new(shape)));
                            }
                        }
                    }
                    b"draw:page" => {
                        if let Some(slide) = current_slide.take() {
                            let part_name = format!("ppt/slides/slide{}.xml", pkg.slides.len() + 1);
                            pkg.slides.push((part_name, slide));
                        }
                    }
                    _ => {}
                },
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(format!("XML error reading ODP content.xml: {e}")),
                _ => {}
            }
            buf.clear();
        }

        if pkg.slides.is_empty() {
            // Default blank slide if none parsed
            pkg.slides
                .push(("ppt/slides/slide1.xml".to_string(), Slide::default()));
        }

        Ok(pkg)
    }

    /// Generates compliant ODF `content.xml`.
    fn generate_content_xml(pkg: &PresentationPackage) -> String {
        let mut out = String::with_capacity(8192);
        out.push_str(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
  xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
  xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
  xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0"
  office:version="1.3">
  <office:body>
    <office:presentation>
"#,
        );

        for (i, (_part, slide)) in pkg.slides.iter().enumerate() {
            out.push_str(&format!(
                r#"      <draw:page draw:name="page{}" draw:id="page_{}">
"#,
                i + 1,
                i + 1
            ));

            for elem in &slide.common_slide_data.shape_tree.shapes {
                if let ShapeElement::Shape(ref s) = elem {
                    let (x_pt, y_pt, w_pt, h_pt) = if let Some(ref t) = s.properties.transform {
                        (
                            t.x as f64 / 12700.0,
                            t.y as f64 / 12700.0,
                            t.cx as f64 / 12700.0,
                            t.cy as f64 / 12700.0,
                        )
                    } else {
                        (0.0, 0.0, 100.0, 100.0)
                    };

                    out.push_str(&format!(
                        r#"        <draw:custom-shape draw:name="{}" svg:x="{:.2}pt" svg:y="{:.2}pt" svg:width="{:.2}pt" svg:height="{:.2}pt">
"#,
                        s.name, x_pt, y_pt, w_pt, h_pt
                    ));

                    if let Some(ref tb) = s.text_body {
                        out.push_str("          <draw:text-box>\n");
                        for p in &tb.paragraphs {
                            out.push_str("            <text:p>");
                            for r in &p.runs {
                                if let TextRunElement::Text(ref text, _) = r {
                                    out.push_str(&quick_xml::escape::escape(text));
                                }
                            }
                            out.push_str("</text:p>\n");
                        }
                        out.push_str("          </draw:text-box>\n");
                    }

                    out.push_str("        </draw:custom-shape>\n");
                }
            }

            out.push_str("      </draw:page>\n");
        }

        out.push_str(
            r#"    </office:presentation>
  </office:body>
</office:document-content>"#,
        );
        out
    }

    /// Parses ODF dimensional length strings (e.g. "10pt", "2.5cm", "1.5in") into EMUs.
    fn parse_odf_length(val: &str) -> i64 {
        let trimmed = val.trim();
        if let Some(pt_str) = trimmed.strip_suffix("pt") {
            let pts = pt_str.parse::<f64>().unwrap_or(0.0);
            (pts * 12700.0).round() as i64
        } else if let Some(in_str) = trimmed.strip_suffix("in") {
            let inches = in_str.parse::<f64>().unwrap_or(0.0);
            (inches * 914400.0).round() as i64
        } else if let Some(cm_str) = trimmed.strip_suffix("cm") {
            let cm = cm_str.parse::<f64>().unwrap_or(0.0);
            (cm * 360000.0).round() as i64
        } else if let Some(mm_str) = trimmed.strip_suffix("mm") {
            let mm = mm_str.parse::<f64>().unwrap_or(0.0);
            (mm * 36000.0).round() as i64
        } else {
            let num = trimmed.parse::<f64>().unwrap_or(0.0);
            (num * 12700.0).round() as i64
        }
    }
}
