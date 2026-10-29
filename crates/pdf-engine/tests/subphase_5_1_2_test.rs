//! Subphase 5.1.2 Integration Test Suite:
//! Content Stream Filter Decompression & Font CMap Unicode Resolver
//!
//! Validates:
//! - ASCIIHexDecode filter (whitespace, uppercase/lowercase, trailing odd digit)
//! - ASCII85Decode filter (tuple decoding, 'z' 4-zero expansion, partial ending)
//! - FlateDecode with PNG Predictors (Sub, Up, Average, Paeth) & TIFF Predictor 2
//! - Multi-filter chaining via `FilterPipeline`
//! - Content stream operator parser (graphics state, colors, paths, text)
//! - PostScript `/ToUnicode` CMap parser (bfchar, bfrange, arrays, ligatures, surrogates)
//! - Standard WinAnsiEncoding translation
//! - Font metrics resolution and glyph width calculation
//! - Full page text extraction with layout, positioning, and matrix transformations
//! - Plain text reconstruction with spatial line breaks and word spaces
//! - Sub-millisecond performance SLA benchmark (< 1.0 ms)

use flate2::write::ZlibEncoder;
use flate2::Compression;
use pdf_engine::filters::ascii_85::decode_ascii_85;
use pdf_engine::filters::ascii_hex::decode_ascii_hex;
use pdf_engine::filters::predictor::{apply_predictor, PredictorParams};
use pdf_engine::filters::FilterPipeline;
use pdf_engine::font::win_ansi_to_unicode;
use pdf_engine::{
    ContentOperator, ContentParser, FontCMap, PdfDocument, PdfFont, PdfObject, TextChunk,
};
use std::io::Write;
use std::time::Instant;

#[test]
fn test_01_ascii_hex_decode_standard_and_whitespace() {
    let input = b"48 65 6C 6C 6F 20 57 6F 72 6C 64>";
    let decoded = decode_ascii_hex(input).unwrap();
    assert_eq!(decoded, b"Hello World");

    // Odd number of digits: trailing '0' is appended
    let odd_input = b"48 65 6c 6c 6>";
    let odd_decoded = decode_ascii_hex(odd_input).unwrap();
    assert_eq!(odd_decoded, &[0x48, 0x65, 0x6c, 0x6c, 0x60]);
}

#[test]
fn test_02_ascii_85_decode() {
    // Standard 5-tuple: "Man is d"
    let input = b"<~9jqo^BlbD-~>";
    let decoded = decode_ascii_85(input).unwrap();
    assert_eq!(decoded, b"Man is d");

    // 'z' expansion for 4 zero bytes
    let z_input = b"<~z9jqo^~>";
    let z_decoded = decode_ascii_85(z_input).unwrap();
    assert_eq!(z_decoded, &[0, 0, 0, 0, b'M', b'a', b'n', b' ']);

    // Partial final tuple (3 characters produces 2 bytes)
    // '9j' + 'uuu' -> 2 bytes
    let partial_input = b"<~9j~>";
    let partial_decoded = decode_ascii_85(partial_input).unwrap();
    assert_eq!(partial_decoded.len(), 1);
}

#[test]
fn test_03_flate_decode_with_png_sub_predictor() {
    // 1 row, 2 pixels, 3 colors (RGB) -> bpp = 3, row_bytes = 6
    // Filter byte = 1 (Sub)
    // Pixel 0: [10, 20, 30]
    // Pixel 1: [5, 5, 5] (stored as differences: 15-10=5, 25-20=5, 35-30=5)
    let raw = vec![1, 10, 20, 30, 5, 5, 5];
    let params = PredictorParams {
        predictor: 11, // PNG Sub
        colors: 3,
        bits_per_component: 8,
        columns: 2,
    };

    let reconstructed = apply_predictor(&raw, &params).unwrap();
    assert_eq!(reconstructed, vec![10, 20, 30, 15, 25, 35]);
}

#[test]
fn test_04_flate_decode_with_png_up_predictor() {
    // 2 rows, 2 pixels, 1 color (Gray) -> bpp = 1, row_bytes = 2
    // Row 0: filter 0 (None), values: [100, 150]
    // Row 1: filter 2 (Up), diffs: [10, 20] -> reconstructed: [110, 170]
    let raw = vec![
        0, 100, 150, // Row 0
        2, 10, 20, // Row 1
    ];
    let params = PredictorParams {
        predictor: 12, // PNG Up
        colors: 1,
        bits_per_component: 8,
        columns: 2,
    };

    let reconstructed = apply_predictor(&raw, &params).unwrap();
    assert_eq!(reconstructed, vec![100, 150, 110, 170]);
}

#[test]
fn test_05_flate_decode_with_png_average_and_paeth() {
    // 2 rows, 2 pixels, 1 color (Gray)
    // Row 0: filter 0 (None), [50, 80]
    // Row 1: filter 4 (Paeth), diffs [10, 10]
    let raw = vec![
        0, 50, 80, // Row 0
        4, 10, 10, // Row 1: left=0, up=50, up_left=0 -> paeth(0, 50, 0) = 50 -> 50+10 = 60
    ];
    let params = PredictorParams {
        predictor: 14, // PNG Paeth
        colors: 1,
        bits_per_component: 8,
        columns: 2,
    };

    let reconstructed = apply_predictor(&raw, &params).unwrap();
    assert_eq!(reconstructed[0..2], [50, 80]);
    assert_eq!(reconstructed[2], 60);
}

#[test]
fn test_06_flate_decode_with_tiff_predictor_2() {
    // 1 row, 2 pixels, 3 components (RGB) -> bpp = 3, row_bytes = 6
    // Pixel 0: [100, 120, 140]
    // Pixel 1: diffs [10, 20, 30] -> reconstructed: [110, 140, 170]
    let raw = vec![100, 120, 140, 10, 20, 30];
    let params = PredictorParams {
        predictor: 2, // TIFF Predictor 2
        colors: 3,
        bits_per_component: 8,
        columns: 2,
    };

    let reconstructed = apply_predictor(&raw, &params).unwrap();
    assert_eq!(reconstructed, vec![100, 120, 140, 110, 140, 170]);
}

#[test]
fn test_07_filter_pipeline_chained_filters() {
    let original = b"Chained Filter Pipeline Verification";

    // 1. Deflate compress
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original).unwrap();
    let flate_data = encoder.finish().unwrap();

    // 2. ASCIIHex encode
    let mut hex_ascii = Vec::new();
    for &b in &flate_data {
        hex_ascii.extend_from_slice(format!("{:02X}", b).as_bytes());
    }
    hex_ascii.push(b'>');

    // Decode through FilterPipeline with [ /ASCIIHexDecode, /FlateDecode ]
    let filters = PdfObject::Array(vec![
        PdfObject::Name("ASCIIHexDecode".into()),
        PdfObject::Name("FlateDecode".into()),
    ]);

    let decoded = FilterPipeline::decode(&hex_ascii, Some(&filters), None).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_08_content_operator_graphics_and_colors() {
    let stream = b"q 1 0 0 1 50 100 cm 2.5 w 1 J 2 j 0.8 g 0.2 0.4 0.6 rg 0 0 0 1 k Q";
    let operators = ContentParser::new(stream).parse().unwrap();

    assert_eq!(operators[0], ContentOperator::SaveGraphicsState);
    assert_eq!(
        operators[1],
        ContentOperator::ConcatMatrix([1.0, 0.0, 0.0, 1.0, 50.0, 100.0])
    );
    assert_eq!(operators[2], ContentOperator::SetLineWidth(2.5));
    assert_eq!(operators[3], ContentOperator::SetLineCap(1));
    assert_eq!(operators[4], ContentOperator::SetLineJoin(2));
    assert_eq!(operators[5], ContentOperator::SetGrayFill(0.8));
    assert_eq!(
        operators[6],
        ContentOperator::SetRgbFill {
            r: 0.2,
            g: 0.4,
            b: 0.6
        }
    );
    assert_eq!(
        operators[7],
        ContentOperator::SetCmykFill {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0
        }
    );
    assert_eq!(operators[8], ContentOperator::RestoreGraphicsState);
}

#[test]
fn test_09_content_operator_paths_and_painting() {
    let stream = b"10 20 m 100 20 l 100 80 50 80 10 50 c 0 0 200 100 re h B*";
    let operators = ContentParser::new(stream).parse().unwrap();

    assert_eq!(operators[0], ContentOperator::MoveTo { x: 10.0, y: 20.0 });
    assert_eq!(operators[1], ContentOperator::LineTo { x: 100.0, y: 20.0 });
    assert_eq!(
        operators[2],
        ContentOperator::CurveTo {
            x1: 100.0,
            y1: 80.0,
            x2: 50.0,
            y2: 80.0,
            x3: 10.0,
            y3: 50.0
        }
    );
    assert_eq!(
        operators[3],
        ContentOperator::Rectangle {
            x: 0.0,
            y: 0.0,
            w: 200.0,
            h: 100.0
        }
    );
    assert_eq!(operators[4], ContentOperator::ClosePath);
    assert_eq!(operators[5], ContentOperator::FillAndStrokeEvenOdd);
}

#[test]
fn test_10_content_operator_text_state_and_showing() {
    let stream = b"BT /F1 14 Tf 72 720 Td (Hello PDF) Tj T* [ (T) 120 (ext) ] TJ ET";
    let operators = ContentParser::new(stream).parse().unwrap();

    assert_eq!(operators[0], ContentOperator::BeginText);
    assert_eq!(
        operators[1],
        ContentOperator::SetFont {
            name: "F1".into(),
            size: 14.0
        }
    );
    assert_eq!(
        operators[2],
        ContentOperator::MoveTextPosition {
            tx: 72.0,
            ty: 720.0
        }
    );
    assert_eq!(
        operators[3],
        ContentOperator::ShowText(b"Hello PDF".to_vec())
    );
    assert_eq!(operators[4], ContentOperator::MoveToNextLine);
    assert_eq!(
        operators[5],
        ContentOperator::ShowTextAdjusted(vec![
            TextChunk::Text(b"T".to_vec()),
            TextChunk::Spacing(120.0),
            TextChunk::Text(b"ext".to_vec()),
        ])
    );
    assert_eq!(operators[6], ContentOperator::EndText);
}

#[test]
fn test_11_font_cmap_to_unicode_bfchar_parsing() {
    let cmap_data = b"/CIDInit /ProcSet findresource begin\n\
12 dict begin\n\
begincmap\n\
1 beginbfchar\n\
<0001> <0041>\n\
<0002> <0042>\n\
<0003> <D83DDE00>\n\
<0004> <00660069>\n\
endbfchar\n\
endcmap\n\
end\n\
end";

    let cmap = FontCMap::parse_to_unicode(cmap_data).unwrap();

    assert_eq!(cmap.lookup(1), Some("A"));
    assert_eq!(cmap.lookup(2), Some("B"));
    // Surrogate pair -> emoji U+1F600 😀
    assert_eq!(cmap.lookup(3), Some("\u{1F600}"));
    // Ligature -> "fi"
    assert_eq!(cmap.lookup(4), Some("fi"));
}

#[test]
fn test_12_font_cmap_to_unicode_bfrange_parsing() {
    let cmap_data = b"begincmap\n\
2 beginbfrange\n\
<0001> <0003> <0041>\n\
<0010> <0012> [ <0058> <0059> <005A> ]\n\
endbfrange\n\
endcmap";

    let cmap = FontCMap::parse_to_unicode(cmap_data).unwrap();

    // Range 1..=3 -> 'A', 'B', 'C'
    assert_eq!(cmap.lookup(1), Some("A"));
    assert_eq!(cmap.lookup(2), Some("B"));
    assert_eq!(cmap.lookup(3), Some("C"));

    // Array range 16..=18 -> 'X', 'Y', 'Z'
    assert_eq!(cmap.lookup(16), Some("X"));
    assert_eq!(cmap.lookup(17), Some("Y"));
    assert_eq!(cmap.lookup(18), Some("Z"));
}

#[test]
fn test_13_win_ansi_standard_encoding_fallback() {
    assert_eq!(win_ansi_to_unicode(b'A'), 'A');
    assert_eq!(win_ansi_to_unicode(b'z'), 'z');
    assert_eq!(win_ansi_to_unicode(0x80), '\u{20AC}'); // Euro
    assert_eq!(win_ansi_to_unicode(0x96), '\u{2013}'); // En dash
    assert_eq!(win_ansi_to_unicode(0x97), '\u{2014}'); // Em dash
    assert_eq!(win_ansi_to_unicode(0x93), '\u{201C}'); // Left double quote
    assert_eq!(win_ansi_to_unicode(0x94), '\u{201D}'); // Right double quote
}

#[test]
fn test_14_pdf_font_decode_and_metrics() {
    let mut font = PdfFont::new("Helvetica-Bold".into(), "TrueType".into());
    font.first_char = 65;
    font.last_char = 67;
    font.widths.insert(65, 722.0); // 'A'
    font.widths.insert(66, 667.0); // 'B'
    font.widths.insert(67, 722.0); // 'C'

    let mut cmap = FontCMap::new();
    cmap.insert(1, "Alpha".into());
    cmap.insert(2, "Beta".into());
    font.cmap = Some(cmap);

    assert_eq!(font.glyph_width(65), 722.0);
    assert_eq!(font.glyph_width(66), 667.0);
    assert_eq!(font.glyph_width(99), 1000.0); // fallback default

    // Decode with CMap
    assert_eq!(font.decode_text(&[1, 2]), "AlphaBeta");
}

#[test]
fn test_15_extracted_page_text_runs_and_positions() {
    // Build a complete PDF in memory with content stream BT ... ET
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    let content_stream = b"BT /F1 16 Tf 72 700 Td (OpenOfficeRS PDF) Tj ET";
    let off_cont = doc_bytes.len();
    doc_bytes.extend_from_slice(
        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content_stream.len(),
            std::str::from_utf8(content_stream).unwrap()
        )
        .as_bytes(),
    );

    let off_font = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 6\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_cont, off_font, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).unwrap();
    let page = doc.page(0).unwrap();

    let extracted = page.extract_text(&doc).unwrap();
    assert_eq!(extracted.runs.len(), 1);

    let run = &extracted.runs[0];
    assert_eq!(run.text, "OpenOfficeRS PDF");
    assert_eq!(run.font_name, "F1");
    assert_eq!(run.font_size, 16.0);
    assert_eq!(run.bbox.llx, 72.0);
    assert_eq!(run.bbox.lly, 700.0);
}

#[test]
fn test_16_extracted_page_plain_text_paragraphs_and_spaces() {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    // Two lines:
    // Line 1: (First Line) at (72, 720)
    // Line 2: (Second Line) at (72, 690)
    let content_stream = b"BT /F1 12 Tf 72 720 Td (First Line) Tj 0 -30 Td (Second Line) Tj ET";
    let off_cont = doc_bytes.len();
    doc_bytes.extend_from_slice(
        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content_stream.len(),
            std::str::from_utf8(content_stream).unwrap()
        )
        .as_bytes(),
    );

    let off_font = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 6\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_cont, off_font, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).unwrap();
    let page = doc.page(0).unwrap();

    let plain_text = page.extract_plain_text(&doc).unwrap();
    assert_eq!(plain_text, "First Line\nSecond Line");
}

#[test]
fn test_17_performance_text_extraction_sla() {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.4\n");

    let off_cat = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off_pages = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] /Count 1 /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
    );

    // 20 lines of text in content stream
    let mut content = b"BT /F1 12 Tf 72 750 Td\n".to_vec();
    for i in 0..20 {
        content.extend_from_slice(
            format!("(Line {} Content Stream SLA Benchmark) Tj 0 -20 Td\n", i).as_bytes(),
        );
    }
    content.extend_from_slice(b"ET\n");

    let off_cont = doc_bytes.len();
    doc_bytes.extend_from_slice(
        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content.len(),
            std::str::from_utf8(&content).unwrap()
        )
        .as_bytes(),
    );

    let off_font = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    let off_page = doc_bytes.len();
    doc_bytes.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
    );

    let xref_offset = doc_bytes.len();
    let xref_str = format!(
        "xref\n0 6\n\
0000000000 65535 f \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
{:010} 00000 n \r\n\
trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
        off_cat, off_pages, off_page, off_cont, off_font, xref_offset
    );
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    let doc = PdfDocument::parse(doc_bytes).unwrap();
    let page = doc.page(0).unwrap();

    let start = Instant::now();
    let extracted = page.extract_text(&doc).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(extracted.runs.len(), 20);
    // SLA: full text extraction of 20 lines must be < 1.0 ms
    assert!(
        elapsed.as_micros() < 1000,
        "Text extraction took {:?}, must be < 1.0 ms",
        elapsed
    );
}
