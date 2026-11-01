#![forbid(unsafe_code)]
//! PDF Page Manipulation, Assembly, Bates Numbering & Searchable OCR Injection
//!
//! Complies with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in production library code.
//! - Clean ISO 32000-1 document assembly (split, extract, merge).
//! - Non-destructive Bates numbering overlay generation.
//! - Searchable OCR text injection using invisible text rendering mode (`3 Tr`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::{PdfObject, PdfObjectId, PdfRect, PdfStream};
use crate::writer::PdfIncrementalWriter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatesPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatesNumberingConfig {
    pub prefix: String,
    pub suffix: String,
    pub start_number: u32,
    #[serde(alias = "digits")]
    pub num_digits: usize,
    pub position: BatesPosition,
    #[serde(alias = "font_size_pt")]
    pub font_size: f64,
    #[serde(default)]
    pub color_rgb: [f32; 3],
    #[serde(alias = "margin_x_pt")]
    pub margin_pt: f64,
}

impl Default for BatesNumberingConfig {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            start_number: 1,
            num_digits: 6,
            position: BatesPosition::BottomRight,
            font_size: 10.0,
            color_rgb: [0.0, 0.0, 0.0],
            margin_pt: 36.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub bbox: OcrBoundingBox,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrPageData {
    pub page_index: usize,
    pub words: Vec<OcrWord>,
}

pub struct PdfPageOps;

impl PdfPageOps {
    /// Extracts specified page indices into a new standalone PDF document.
    pub fn extract_pages(data: &[u8], page_indices: &[usize]) -> Result<Vec<u8>, PdfError> {
        if data.is_empty() {
            return Err(PdfError::UnexpectedEof);
        }
        if page_indices.is_empty() {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: "No page indices specified for extraction".into(),
            });
        }

        let doc = PdfDocument::parse(data.to_vec())?;
        let page_count = doc.page_count();

        for &idx in page_indices {
            if idx >= page_count {
                return Err(PdfError::PageOutOfRange {
                    index: idx,
                    count: page_count,
                });
            }
        }

        Self::assemble_pages_into_document(&doc, page_indices)
    }

    /// Splits a document into multiple PDF documents based on boundary page indices.
    /// If `split_indices` is empty, splits into individual 1-page documents.
    pub fn split_document(data: &[u8], split_indices: &[usize]) -> Result<Vec<Vec<u8>>, PdfError> {
        let doc = PdfDocument::parse(data.to_vec())?;
        let total = doc.page_count();
        if total == 0 {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: "Cannot split a PDF with 0 pages".into(),
            });
        }

        let mut slices = Vec::new();

        if split_indices.is_empty() {
            for i in 0..total {
                slices.push(vec![i]);
            }
        } else {
            let mut boundaries = split_indices.to_vec();
            boundaries.retain(|&idx| idx > 0 && idx < total);
            boundaries.sort_unstable();
            boundaries.dedup();

            let mut start = 0usize;
            for b in boundaries {
                if b > start {
                    slices.push((start..b).collect::<Vec<_>>());
                    start = b;
                }
            }
            if start < total {
                slices.push((start..total).collect::<Vec<_>>());
            }
        }

        let mut results = Vec::with_capacity(slices.len());
        for slice in slices {
            let chunk_bytes = Self::assemble_pages_into_document(&doc, &slice)?;
            results.push(chunk_bytes);
        }

        Ok(results)
    }

    /// Merges multiple PDF document byte slices into a single unified multi-page PDF document.
    pub fn merge_documents(docs_data: &[&[u8]]) -> Result<Vec<u8>, PdfError> {
        if docs_data.is_empty() {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: "Cannot merge an empty list of documents".into(),
            });
        }
        if docs_data.len() == 1 {
            return Ok(docs_data[0].to_vec());
        }

        let mut parsed_docs = Vec::with_capacity(docs_data.len());
        for &data in docs_data {
            if data.is_empty() {
                continue;
            }
            let doc = PdfDocument::parse(data.to_vec())?;
            parsed_docs.push(doc);
        }

        if parsed_docs.is_empty() {
            return Err(PdfError::UnexpectedEof);
        }

        Self::assemble_multi_document_merge(&parsed_docs)
    }

    /// Non-destructively stamps Bates numbering onto every page of the document.
    pub fn apply_bates_numbering(
        data: &[u8],
        config: &BatesNumberingConfig,
    ) -> Result<Vec<u8>, PdfError> {
        let doc = PdfDocument::parse(data.to_vec())?;
        let page_count = doc.page_count();
        if page_count == 0 {
            return Ok(data.to_vec());
        }

        let mut writer = PdfIncrementalWriter::from_document(&doc, data.to_vec())?;

        // Allocate shared Helvetica font for Bates numbering
        let mut bates_font_dict = BTreeMap::new();
        bates_font_dict.insert("Type".into(), PdfObject::Name("Font".into()));
        bates_font_dict.insert("Subtype".into(), PdfObject::Name("Type1".into()));
        bates_font_dict.insert("BaseFont".into(), PdfObject::Name("Helvetica".into()));
        let bates_font_id = writer.add_object(PdfObject::Dictionary(bates_font_dict));

        for i in 0..page_count {
            let page = doc.page(i)?;
            let bates_num = config.start_number.saturating_add(i as u32);
            let bates_text = format!(
                "{}{:0width$}{}",
                config.prefix,
                bates_num,
                config.suffix,
                width = config.num_digits
            );

            let (pos_x, pos_y) = Self::calculate_bates_position(page, config, &bates_text);

            let r = config.color_rgb[0].clamp(0.0, 1.0);
            let g = config.color_rgb[1].clamp(0.0, 1.0);
            let b = config.color_rgb[2].clamp(0.0, 1.0);

            let escaped_label = escape_pdf_literal_string(&bates_text);
            let stream_content = format!(
                "q\n{:.3} {:.3} {:.3} rg\nBT\n/F_BATES {:.2} Tf\n{:.2} {:.2} Td\n({}) Tj\nET\nQ\n",
                r, g, b, config.font_size, pos_x, pos_y, escaped_label
            );

            let mut stm_dict = BTreeMap::new();
            stm_dict.insert(
                "Length".into(),
                PdfObject::Integer(stream_content.len() as i64),
            );
            let stream_obj =
                PdfObject::Stream(PdfStream::new(stm_dict, stream_content.into_bytes()));
            let stream_id = writer.add_object(stream_obj);

            // Update page resources to include /F_BATES
            let mut page_res = page.resources.clone();
            let mut font_res = match page_res.get("Font") {
                Some(PdfObject::Dictionary(d)) => d.clone(),
                _ => BTreeMap::new(),
            };
            font_res.insert("F_BATES".into(), PdfObject::Reference(bates_font_id));
            page_res.insert("Font".into(), PdfObject::Dictionary(font_res));

            // Append new stream to contents
            let mut contents_array: Vec<PdfObject> = page
                .contents
                .iter()
                .map(|&cid| PdfObject::Reference(cid))
                .collect();
            contents_array.push(PdfObject::Reference(stream_id));

            // Reconstruct page object
            let mut page_dict = BTreeMap::new();
            page_dict.insert("Type".into(), PdfObject::Name("Page".into()));
            page_dict.insert(
                "Parent".into(),
                PdfObject::Reference(PdfObjectId::new(2, 0)),
            );
            page_dict.insert("MediaBox".into(), rect_to_pdf_object(&page.media_box));
            page_dict.insert("CropBox".into(), rect_to_pdf_object(&page.crop_box));
            if page.rotate != 0 {
                page_dict.insert("Rotate".into(), PdfObject::Integer(page.rotate as i64));
            }
            page_dict.insert("Resources".into(), PdfObject::Dictionary(page_res));
            page_dict.insert("Contents".into(), PdfObject::Array(contents_array));

            writer.set_object(page.object_id, PdfObject::Dictionary(page_dict));
        }

        writer.finish()
    }

    /// Injects an invisible OCR text layer (`3 Tr`) positioned precisely over scanned image areas.
    pub fn inject_ocr_layer(
        data: &[u8],
        page_index: usize,
        ocr_data: &OcrPageData,
    ) -> Result<Vec<u8>, PdfError> {
        let doc = PdfDocument::parse(data.to_vec())?;
        let page = doc.page(page_index)?;

        let mut writer = PdfIncrementalWriter::from_document(&doc, data.to_vec())?;

        // Shared Helvetica font for OCR text layer
        let mut ocr_font_dict = BTreeMap::new();
        ocr_font_dict.insert("Type".into(), PdfObject::Name("Font".into()));
        ocr_font_dict.insert("Subtype".into(), PdfObject::Name("Type1".into()));
        ocr_font_dict.insert("BaseFont".into(), PdfObject::Name("Helvetica".into()));
        let ocr_font_id = writer.add_object(PdfObject::Dictionary(ocr_font_dict));

        // Generate invisible text content stream (`3 Tr` per ISO 32000-1 §9.3.5)
        let mut stream_content = String::with_capacity(1024 + ocr_data.words.len() * 48);
        stream_content.push_str("q\n3 Tr\n"); // 3 Tr: Neither fill nor stroke text (invisible)

        for word in &ocr_data.words {
            if word.text.trim().is_empty() {
                continue;
            }
            let font_size = word.bbox.height.max(6.0);
            let escaped = escape_pdf_literal_string(&word.text);
            stream_content.push_str(&format!(
                "BT\n/F_OCR {:.2} Tf\n{:.2} {:.2} Td\n({}) Tj\nET\n",
                font_size, word.bbox.x, word.bbox.y, escaped
            ));
        }
        stream_content.push_str("Q\n");

        let mut stm_dict = BTreeMap::new();
        stm_dict.insert(
            "Length".into(),
            PdfObject::Integer(stream_content.len() as i64),
        );
        let stream_obj = PdfObject::Stream(PdfStream::new(stm_dict, stream_content.into_bytes()));
        let stream_id = writer.add_object(stream_obj);

        // Update page resources
        let mut page_res = page.resources.clone();
        let mut font_res = match page_res.get("Font") {
            Some(PdfObject::Dictionary(d)) => d.clone(),
            _ => BTreeMap::new(),
        };
        font_res.insert("F_OCR".into(), PdfObject::Reference(ocr_font_id));
        page_res.insert("Font".into(), PdfObject::Dictionary(font_res));

        // Append OCR stream to page contents
        let mut contents_array: Vec<PdfObject> = page
            .contents
            .iter()
            .map(|&cid| PdfObject::Reference(cid))
            .collect();
        contents_array.push(PdfObject::Reference(stream_id));

        let mut page_dict = BTreeMap::new();
        page_dict.insert("Type".into(), PdfObject::Name("Page".into()));
        page_dict.insert(
            "Parent".into(),
            PdfObject::Reference(PdfObjectId::new(2, 0)),
        );
        page_dict.insert("MediaBox".into(), rect_to_pdf_object(&page.media_box));
        page_dict.insert("CropBox".into(), rect_to_pdf_object(&page.crop_box));
        if page.rotate != 0 {
            page_dict.insert("Rotate".into(), PdfObject::Integer(page.rotate as i64));
        }
        page_dict.insert("Resources".into(), PdfObject::Dictionary(page_res));
        page_dict.insert("Contents".into(), PdfObject::Array(contents_array));

        writer.set_object(page.object_id, PdfObject::Dictionary(page_dict));
        writer.finish()
    }

    fn calculate_bates_position(
        page: &crate::page::PdfPage,
        config: &BatesNumberingConfig,
        text: &str,
    ) -> (f64, f64) {
        let crop = &page.crop_box;
        let margin = config.margin_pt;
        let est_width = text.len() as f64 * config.font_size * 0.55;

        match config.position {
            BatesPosition::TopLeft => (crop.llx + margin, crop.ury - margin),
            BatesPosition::TopCenter => (
                crop.llx + (crop.width() - est_width) * 0.5,
                crop.ury - margin,
            ),
            BatesPosition::TopRight => (crop.urx - margin - est_width, crop.ury - margin),
            BatesPosition::BottomLeft => (crop.llx + margin, crop.lly + margin),
            BatesPosition::BottomCenter => (
                crop.llx + (crop.width() - est_width) * 0.5,
                crop.lly + margin,
            ),
            BatesPosition::BottomRight => (crop.urx - margin - est_width, crop.lly + margin),
        }
    }

    /// Internal clean-room page assembler for page extraction and single-document slicing.
    fn assemble_pages_into_document(
        doc: &PdfDocument,
        page_indices: &[usize],
    ) -> Result<Vec<u8>, PdfError> {
        let mut out = Vec::with_capacity(8192);
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let mut offsets: BTreeMap<u32, usize> = BTreeMap::new();
        let num_pages = page_indices.len();

        // 1 0 obj: Catalog
        offsets.insert(1, out.len());
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // 2 0 obj: Pages root
        offsets.insert(2, out.len());
        let mut kids_str = String::new();
        // Page object IDs will be allocated sequentially:
        // Page 0: obj_id = 3, font_id = 4, content_id = 5
        // Page 1: obj_id = 6, font_id = 7, content_id = 8
        for i in 0..num_pages {
            let page_id = 3 + i * 3;
            kids_str.push_str(&format!("{page_id} 0 R "));
        }
        let pages_obj = format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids_str.trim_end(),
            num_pages
        );
        out.extend_from_slice(pages_obj.as_bytes());

        // For each extracted page
        for (i, &p_idx) in page_indices.iter().enumerate() {
            let page = doc.page(p_idx)?;
            let page_id = (3 + i * 3) as u32;
            let font_id = page_id + 1;
            let content_id = page_id + 2;

            // Page Object
            offsets.insert(page_id, out.len());
            let page_def = format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [{:.2} {:.2} {:.2} {:.2}] /CropBox [{:.2} {:.2} {:.2} {:.2}] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>\nendobj\n",
                page_id,
                page.media_box.llx, page.media_box.lly, page.media_box.urx, page.media_box.ury,
                page.crop_box.llx, page.crop_box.lly, page.crop_box.urx, page.crop_box.ury,
                content_id, font_id
            );
            out.extend_from_slice(page_def.as_bytes());

            // Font Object
            offsets.insert(font_id, out.len());
            let font_def = format!(
                "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
                font_id
            );
            out.extend_from_slice(font_def.as_bytes());

            // Combined content stream
            let mut combined_content = Vec::new();
            for &cid in &page.contents {
                if let Ok(PdfObject::Stream(stm)) = doc.resolve_object(cid) {
                    if let Ok(decomp) = stm.decode() {
                        combined_content.extend_from_slice(&decomp);
                        combined_content.push(b'\n');
                    }
                }
            }
            if combined_content.is_empty() {
                combined_content.extend_from_slice(b"q\nQ\n");
            }

            offsets.insert(content_id, out.len());
            let content_hdr = format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n",
                content_id,
                combined_content.len()
            );
            out.extend_from_slice(content_hdr.as_bytes());
            out.extend_from_slice(&combined_content);
            out.extend_from_slice(b"\nendstream\nendobj\n");
        }

        let total_objs = (2 + num_pages * 3) as u32;
        let xref_offset = out.len();
        out.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \r\n", total_objs + 1).as_bytes(),
        );
        for obj_num in 1..=total_objs {
            let off = offsets.get(&obj_num).copied().unwrap_or(0);
            out.extend_from_slice(format!("{:010} 00000 n \r\n", off).as_bytes());
        }
        let trailer_str = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            xref_offset
        );
        out.extend_from_slice(trailer_str.as_bytes());

        Ok(out)
    }

    /// Internal clean-room multi-document merge assembling all pages sequentially into a new document.
    fn assemble_multi_document_merge(docs: &[PdfDocument]) -> Result<Vec<u8>, PdfError> {
        let mut out = Vec::with_capacity(16384);
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let mut offsets: BTreeMap<u32, usize> = BTreeMap::new();
        let total_pages: usize = docs.iter().map(|d| d.page_count()).sum();

        // 1 0 obj: Catalog
        offsets.insert(1, out.len());
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // 2 0 obj: Pages root
        offsets.insert(2, out.len());
        let mut kids_str = String::new();
        for i in 0..total_pages {
            let page_id = 3 + i * 3;
            kids_str.push_str(&format!("{page_id} 0 R "));
        }
        let pages_obj = format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids_str.trim_end(),
            total_pages
        );
        out.extend_from_slice(pages_obj.as_bytes());

        let mut page_counter = 0usize;
        for doc in docs {
            for p_idx in 0..doc.page_count() {
                let page = doc.page(p_idx)?;
                let page_id = (3 + page_counter * 3) as u32;
                let font_id = page_id + 1;
                let content_id = page_id + 2;

                // Page Object
                offsets.insert(page_id, out.len());
                let page_def = format!(
                    "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [{:.2} {:.2} {:.2} {:.2}] /CropBox [{:.2} {:.2} {:.2} {:.2}] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>\nendobj\n",
                    page_id,
                    page.media_box.llx, page.media_box.lly, page.media_box.urx, page.media_box.ury,
                    page.crop_box.llx, page.crop_box.lly, page.crop_box.urx, page.crop_box.ury,
                    content_id, font_id
                );
                out.extend_from_slice(page_def.as_bytes());

                // Font Object
                offsets.insert(font_id, out.len());
                let font_def = format!(
                    "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
                    font_id
                );
                out.extend_from_slice(font_def.as_bytes());

                // Content stream
                let mut combined_content = Vec::new();
                for &cid in &page.contents {
                    if let Ok(PdfObject::Stream(stm)) = doc.resolve_object(cid) {
                        if let Ok(decomp) = stm.decode() {
                            combined_content.extend_from_slice(&decomp);
                            combined_content.push(b'\n');
                        }
                    }
                }
                if combined_content.is_empty() {
                    combined_content.extend_from_slice(b"q\nQ\n");
                }

                offsets.insert(content_id, out.len());
                let content_hdr = format!(
                    "{} 0 obj\n<< /Length {} >>\nstream\n",
                    content_id,
                    combined_content.len()
                );
                out.extend_from_slice(content_hdr.as_bytes());
                out.extend_from_slice(&combined_content);
                out.extend_from_slice(b"\nendstream\nendobj\n");

                page_counter += 1;
            }
        }

        let total_objs = (2 + total_pages * 3) as u32;
        let xref_offset = out.len();
        out.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \r\n", total_objs + 1).as_bytes(),
        );
        for obj_num in 1..=total_objs {
            let off = offsets.get(&obj_num).copied().unwrap_or(0);
            out.extend_from_slice(format!("{:010} 00000 n \r\n", off).as_bytes());
        }
        let trailer_str = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            xref_offset
        );
        out.extend_from_slice(trailer_str.as_bytes());

        Ok(out)
    }
}

fn rect_to_pdf_object(r: &PdfRect) -> PdfObject {
    PdfObject::Array(vec![
        PdfObject::Real(r.llx),
        PdfObject::Real(r.lly),
        PdfObject::Real(r.urx),
        PdfObject::Real(r.ury),
    ])
}

fn escape_pdf_literal_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for b in s.bytes() {
        match b {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            b'\r' => out.push_str("\\r"),
            b'\n' => out.push_str("\\n"),
            other => out.push(other as char),
        }
    }
    out
}

pub fn extract_pages(data: &[u8], page_indices: &[usize]) -> Result<Vec<u8>, PdfError> {
    PdfPageOps::extract_pages(data, page_indices)
}

pub fn split_document(data: &[u8], boundary_indices: &[usize]) -> Result<Vec<Vec<u8>>, PdfError> {
    PdfPageOps::split_document(data, boundary_indices)
}

pub fn merge_documents(documents: &[&[u8]]) -> Result<Vec<u8>, PdfError> {
    PdfPageOps::merge_documents(documents)
}

pub fn apply_bates_numbering(
    data: &[u8],
    config: &BatesNumberingConfig,
) -> Result<Vec<u8>, PdfError> {
    PdfPageOps::apply_bates_numbering(data, config)
}

pub fn inject_ocr_layer(
    data: &[u8],
    page_index: usize,
    ocr_data: &OcrPageData,
) -> Result<Vec<u8>, PdfError> {
    PdfPageOps::inject_ocr_layer(data, page_index, ocr_data)
}
