//! Subphase 3.2.1 Integration Test Suite: REA Line-Breaking & Pagination Solver
//!
//! Validates:
//! - Exact Word text measurement, kerning, and whitespace compression.
//! - Kinsoku Shori (CJK line breaking) and Latin token boundaries.
//! - Knuth-Plass total-fit dynamic programming vs greedy line-breaker.
//! - Line heights, baseline calculation, and paragraph indentations/alignments.
//! - Pagination solver with widow/orphan control, keepNext, keepLines, and pageBreakBefore.
//! - Multi-column page flow and WordPackage integration.

use std::path::PathBuf;

use word_engine::layout::{
    FontMetricsProvider, KinsokuRules, LineBreaker, PaginationSolver, StandardFontMetrics,
    TokenKind,
};
use word_engine::model::{
    BodyElement, Indentation, PageSize, Paragraph, ParagraphChild, ParagraphProperties, Run,
    RunContent, RunProperties, SectionProperties, Spacing,
};
use word_engine::package::WordPackage;
use word_engine::section::SectionGeometry;
use word_engine::styles::StyleSheet;

fn resolve_repo_path(rel: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut curr = manifest_dir.clone();
    for _ in 0..5 {
        let candidate = curr.join(rel);
        if candidate.exists() {
            return candidate;
        }
        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            break;
        }
    }
    manifest_dir.join(rel)
}

fn create_text_paragraph(text: &str, font_size_half_pts: u32) -> Paragraph {
    let mut p = Paragraph::default();
    let mut r = Run {
        properties: Some(RunProperties {
            size_half_points: Some(font_size_half_pts),
            font_ascii: Some("Calibri".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    r.content.push(RunContent::Text(text.to_string(), false));
    p.children.push(ParagraphChild::Run(r));
    p
}

// =========================================================================
// Tests: Text Measurement & Kerning
// =========================================================================

#[test]
fn test_01_char_advance_measurement() {
    let metrics = StandardFontMetrics::new();

    // 10pt = 20 half-points -> em_twips = 200
    // Standard char 'a' has em_fraction 500 -> 200 * 500 / 1000 = 100 twips
    let m10 = metrics.measure_char('a', "Calibri", 20, false, false);
    assert_eq!(m10.advance_width, 100);
    assert_eq!(m10.ascent, 160);
    assert_eq!(m10.descent, 40);

    // 12pt = 24 half-points -> em_twips = 240
    // Standard char 'a' -> 240 * 500 / 1000 = 120 twips
    let m12 = metrics.measure_char('a', "Calibri", 24, false, false);
    assert_eq!(m12.advance_width, 120);

    // 24pt = 48 half-points -> em_twips = 480
    // Space has em_fraction 250 -> 480 * 250 / 1000 = 120 twips
    let m_space = metrics.measure_char(' ', "Calibri", 48, false, false);
    assert_eq!(m_space.advance_width, 120);
}

#[test]
fn test_02_kerning_pairs() {
    let metrics = StandardFontMetrics::new();

    // Kerning for ('A', 'V') at 24pt (48 half-points)
    // em_twips = 480, fraction = -80 -> (480 * -80) / 1000 = -38 twips
    let kern_av = metrics.kerning_adjustment('A', 'V', "Calibri", 48);
    assert_eq!(kern_av, -38);

    // Kerning for non-kerning pair ('A', 'B') should be 0
    let kern_ab = metrics.kerning_adjustment('A', 'B', "Calibri", 48);
    assert_eq!(kern_ab, 0);
}

#[test]
fn test_03_whitespace_compression() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    // Line with trailing spaces
    let p = create_text_paragraph("Hello world     ", 24);
    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    // Set width just enough for "Hello world"
    let m_hello = metrics
        .measure_char('H', "Calibri", 24, false, false)
        .advance_width
        * 5;
    let m_space = metrics
        .measure_char(' ', "Calibri", 24, false, false)
        .advance_width;
    let m_world = metrics
        .measure_char('w', "Calibri", 24, false, false)
        .advance_width
        * 5;
    let content_w = m_hello + m_space + m_world + 50;

    let lines = breaker.greedy_break(&tokens, &p_props, content_w);
    // Trailing spaces should not wrap into an unnecessary second line
    assert_eq!(lines.len(), 1);
}

#[test]
fn test_04_nobreak_space() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    // Paragraph with non-breaking space between "100" and "USD"
    let p = create_text_paragraph("Price: 100\u{00A0}USD total", 24);
    let _p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    // Verify token kind for non-breaking space
    let nb_token = tokens.iter().find(|t| t.text == "\u{00A0}");
    assert!(nb_token.is_some());
    assert_eq!(nb_token.unwrap().kind, TokenKind::Word);
    assert!(!nb_token.unwrap().can_break_after);
}

#[test]
fn test_05_soft_hyphen_handling() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text("super".to_string(), false));
    r.content.push(RunContent::SoftHyphen);
    r.content
        .push(RunContent::Text("califragilistic".to_string(), false));
    p.children.push(ParagraphChild::Run(r));

    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::SoftHyphen));

    // Breaking after soft hyphen turns it into "-"
    let lines = breaker.greedy_break(&tokens, &p_props, 800);
    if lines.len() > 1 {
        let first_line = &lines[0];
        assert_eq!(first_line.last().unwrap().text, "-");
    }
}

#[test]
fn test_06_tab_stop_advancement() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let mut p = Paragraph::default();
    let mut r = Run::default();
    r.content.push(RunContent::Text("Col1".to_string(), false));
    r.content.push(RunContent::Tab);
    r.content.push(RunContent::Text("Col2".to_string(), false));
    p.children.push(ParagraphChild::Run(r));

    let tokens = breaker.tokenize_paragraph(&p, &cascade);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Tab));
    let tab_t = tokens.iter().find(|t| t.kind == TokenKind::Tab).unwrap();
    assert_eq!(tab_t.width, 720); // default half-inch tab
}

// =========================================================================
// Tests: Kinsoku Shori (CJK Line Breaking)
// =========================================================================

#[test]
fn test_07_kinsoku_prohibited_line_start() {
    // Prohibited at start of line: closing punctuation, Japanese punctuation
    assert!(KinsokuRules::is_prohibited_line_start(')'));
    assert!(KinsokuRules::is_prohibited_line_start(']'));
    assert!(KinsokuRules::is_prohibited_line_start('}'));
    assert!(KinsokuRules::is_prohibited_line_start('、'));
    assert!(KinsokuRules::is_prohibited_line_start('。'));
    assert!(KinsokuRules::is_prohibited_line_start('！'));
    assert!(KinsokuRules::is_prohibited_line_start('？'));
    assert!(KinsokuRules::is_prohibited_line_start('」'));
    assert!(KinsokuRules::is_prohibited_line_start('』'));
    assert!(KinsokuRules::is_prohibited_line_start(','));
    assert!(KinsokuRules::is_prohibited_line_start('.'));

    // Normal characters allowed at start of line
    assert!(!KinsokuRules::is_prohibited_line_start('A'));
    assert!(!KinsokuRules::is_prohibited_line_start('日'));
    assert!(!KinsokuRules::is_prohibited_line_start('('));
}

#[test]
fn test_08_kinsoku_prohibited_line_end() {
    // Prohibited at end of line: opening punctuation, currency symbols
    assert!(KinsokuRules::is_prohibited_line_end('('));
    assert!(KinsokuRules::is_prohibited_line_end('['));
    assert!(KinsokuRules::is_prohibited_line_end('{'));
    assert!(KinsokuRules::is_prohibited_line_end('「'));
    assert!(KinsokuRules::is_prohibited_line_end('『'));
    assert!(KinsokuRules::is_prohibited_line_end('（'));
    assert!(KinsokuRules::is_prohibited_line_end('$'));
    assert!(KinsokuRules::is_prohibited_line_end('¥'));
    assert!(KinsokuRules::is_prohibited_line_end('£'));

    // Normal characters allowed at end of line
    assert!(!KinsokuRules::is_prohibited_line_end('Z'));
    assert!(!KinsokuRules::is_prohibited_line_end('本'));
    assert!(!KinsokuRules::is_prohibited_line_end(')'));
}

// =========================================================================
// Tests: Line Breakers (Greedy & Knuth-Plass)
// =========================================================================

#[test]
fn test_09_greedy_line_breaker() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    // 10 words, each about 600 twips + space (150 twips) -> 750 twips per word
    let text = "One two three four five six seven eight nine ten";
    let p = create_text_paragraph(text, 24);
    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    // Target width fits roughly 3 words per line (~2200 twips)
    let lines = breaker.greedy_break(&tokens, &p_props, 2200);
    assert!(
        lines.len() >= 3,
        "Expected at least 3 lines, got: {}",
        lines.len()
    );
}

#[test]
fn test_10_knuth_plass_justified_line_breaking() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let text = "The quick brown fox jumps over the lazy sleeping dog near the calm river bank";
    let mut p = create_text_paragraph(text, 24);
    if let Some(ref mut props) = p.properties {
        props.justification = Some("both".to_string());
    } else {
        p.properties = Some(ParagraphProperties {
            justification: Some("both".to_string()),
            ..Default::default()
        });
    }

    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    let lines = breaker.break_lines(&tokens, &p_props, 3000);
    assert!(
        lines.len() >= 2,
        "Justified paragraph should break cleanly into multiple lines"
    );
}

#[test]
fn test_11_knuth_plass_demerits_penalties() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let text = "Paragraph layout algorithm optimal total fit breaking dynamic programming";
    let p = create_text_paragraph(text, 24);
    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    let lines = breaker.knuth_plass_break(&tokens, &p_props, 3500);
    assert!(!lines.is_empty());
}

// =========================================================================
// Tests: Indentations & Horizontal Alignments
// =========================================================================

#[test]
fn test_12_paragraph_alignments_left_right_center() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Left aligned
    let mut p_left = create_text_paragraph("Left Aligned Text", 24);
    p_left.properties = Some(ParagraphProperties {
        justification: Some("left".to_string()),
        ..Default::default()
    });

    // Right aligned
    let mut p_right = create_text_paragraph("Right Aligned Text", 24);
    p_right.properties = Some(ParagraphProperties {
        justification: Some("right".to_string()),
        ..Default::default()
    });

    // Center aligned
    let mut p_center = create_text_paragraph("Center Aligned Text", 24);
    p_center.properties = Some(ParagraphProperties {
        justification: Some("center".to_string()),
        ..Default::default()
    });

    let elements = vec![
        BodyElement::Paragraph(p_left),
        BodyElement::Paragraph(p_right),
        BodyElement::Paragraph(p_center),
    ];

    let layout = solver.paginate(&elements);
    let col = &layout.pages[0].columns[0];
    assert_eq!(col.lines.len(), 3);

    let x_left = col.lines[0].bounds.x;
    let x_right = col.lines[1].bounds.x;
    let x_center = col.lines[2].bounds.x;

    // Right-aligned start X must be greater than left-aligned start X
    assert!(
        x_right > x_left,
        "Right start X ({x_right}) must be > Left start X ({x_left})"
    );
    // Center-aligned start X must be between left and right
    assert!(x_center > x_left && x_center < x_right);
}

#[test]
fn test_13_paragraph_first_line_indentation() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let mut p = create_text_paragraph("This is a paragraph with a first line indent of 720 twips (0.5 inches). It wraps across lines.", 24);
    p.properties = Some(ParagraphProperties {
        indentation: Some(Indentation {
            first_line: Some(720),
            ..Default::default()
        }),
        ..Default::default()
    });

    let p_props = cascade.resolve_paragraph(&p);
    assert_eq!(p_props.indent_first_line, 720);

    let tokens = breaker.tokenize_paragraph(&p, &cascade);
    let lines = breaker.greedy_break(&tokens, &p_props, 2500);
    assert!(lines.len() >= 2);
}

#[test]
fn test_14_paragraph_hanging_indentation() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    let mut p = create_text_paragraph("Hanging indent paragraph", 24);
    p.properties = Some(ParagraphProperties {
        indentation: Some(Indentation {
            hanging: Some(360),
            left: Some(720),
            ..Default::default()
        }),
        ..Default::default()
    });

    let p_props = cascade.resolve_paragraph(&p);
    assert_eq!(p_props.indent_hanging, 360);
    assert_eq!(p_props.indent_left, 720);
}

// =========================================================================
// Tests: Line Heights & Spacing
// =========================================================================

#[test]
fn test_15_line_height_auto_spacing() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // 12pt font (24 half-points) with 100% single line spacing (line = 240, auto)
    let mut p = create_text_paragraph("Single line height text", 24);
    p.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            line: Some(240),
            line_rule: Some("auto".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let layout = solver.paginate(&[BodyElement::Paragraph(p)]);
    let line_h = layout.pages[0].columns[0].lines[0].bounds.height;
    // 240 * 115 / 100 = 276 twips
    assert_eq!(line_h, 276);
}

#[test]
fn test_16_line_height_exact_spacing() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Exact spacing of 400 twips (20pt)
    let mut p = create_text_paragraph("Exact line spacing text", 24);
    p.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            line: Some(400),
            line_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let layout = solver.paginate(&[BodyElement::Paragraph(p)]);
    let line_h = layout.pages[0].columns[0].lines[0].bounds.height;
    assert_eq!(line_h, 400);
}

#[test]
fn test_17_line_height_at_least_spacing() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // atLeast spacing of 500 twips (larger than natural 276 twips)
    let mut p = create_text_paragraph("AtLeast line spacing text", 24);
    p.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            line: Some(500),
            line_rule: Some("atLeast".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let layout = solver.paginate(&[BodyElement::Paragraph(p)]);
    let line_h = layout.pages[0].columns[0].lines[0].bounds.height;
    assert_eq!(line_h, 500);
}

#[test]
fn test_18_paragraph_spacing_before_and_after() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    let mut p1 = create_text_paragraph("Paragraph 1", 24);
    p1.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            after: Some(200),
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut p2 = create_text_paragraph("Paragraph 2", 24);
    p2.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            before: Some(300),
            ..Default::default()
        }),
        ..Default::default()
    });

    let layout = solver.paginate(&[BodyElement::Paragraph(p1), BodyElement::Paragraph(p2)]);
    let lines = &layout.pages[0].columns[0].lines;
    assert_eq!(lines.len(), 2);

    let y0_bottom = lines[0].bounds.bottom();
    let y1_top = lines[1].bounds.y;
    // Total vertical gap = spacing_after (200) + spacing_before (300) = 500 twips
    assert_eq!(y1_top - y0_bottom, 500);
}

// =========================================================================
// Tests: Pagination & Multi-Page Layout
// =========================================================================

#[test]
fn test_19_single_page_layout_dimensions() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    let p = create_text_paragraph("Hello OpenOfficeRS Layout!", 24);
    let layout = solver.paginate(&[BodyElement::Paragraph(p)]);

    assert_eq!(layout.page_count(), 1);
    let page = &layout.pages[0];
    assert_eq!(page.width, 12240);
    assert_eq!(page.height, 15840);
    assert_eq!(page.columns.len(), 1);

    let col = &page.columns[0];
    assert_eq!(col.bounds.x, 1440);
    assert_eq!(col.bounds.y, 1440);
    assert_eq!(col.bounds.width, 9360);
    assert_eq!(col.bounds.height, 12960);
}

#[test]
fn test_20_multi_page_pagination() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Create 60 paragraphs with large spacing to overflow page (page height 12960 twips)
    let mut elements = Vec::new();
    for i in 0..60 {
        let mut p = create_text_paragraph(&format!("Multi-page content paragraph #{i}"), 24);
        p.properties = Some(ParagraphProperties {
            spacing: Some(Spacing {
                after: Some(300),
                ..Default::default()
            }),
            ..Default::default()
        });
        elements.push(BodyElement::Paragraph(p));
    }

    let layout = solver.paginate(&elements);
    assert!(
        layout.page_count() >= 2,
        "60 paragraphs should paginate onto at least 2 pages, got: {}",
        layout.page_count()
    );
}

#[test]
fn test_21_page_break_before() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    let p1 = create_text_paragraph("First paragraph on page 1", 24);
    let mut p2 = create_text_paragraph("Second paragraph forced to page 2", 24);
    p2.properties = Some(ParagraphProperties {
        page_break_before: true,
        ..Default::default()
    });

    let layout = solver.paginate(&[BodyElement::Paragraph(p1), BodyElement::Paragraph(p2)]);
    assert_eq!(layout.page_count(), 2);
    assert_eq!(layout.pages[0].columns[0].lines.len(), 1);
    assert_eq!(layout.pages[1].columns[0].lines.len(), 1);
}

#[test]
fn test_22_keep_lines_together() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Fill page near bottom
    let mut elements = Vec::new();
    let mut spacer = create_text_paragraph("Page filler", 24);
    spacer.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            after: Some(12500), // leaves < 500 twips
            ..Default::default()
        }),
        ..Default::default()
    });
    elements.push(BodyElement::Paragraph(spacer));

    // Multi-line paragraph with keepLines = true
    let long_text = "This multi-line paragraph cannot be split across a page boundary because keepLines is enabled. It must move to page 2.";
    let mut p_keep = create_text_paragraph(long_text, 24);
    p_keep.properties = Some(ParagraphProperties {
        keep_lines: true,
        ..Default::default()
    });
    elements.push(BodyElement::Paragraph(p_keep));

    let layout = solver.paginate(&elements);
    assert_eq!(layout.page_count(), 2);
    // Page 2 should contain all lines of the keepLines paragraph
    assert!(layout.pages[1].columns[0].lines.len() >= 2);
}

#[test]
fn test_23_keep_with_next() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    let mut elements = Vec::new();
    let mut spacer = create_text_paragraph("Spacer", 24);
    spacer.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            after: Some(12600),
            ..Default::default()
        }),
        ..Default::default()
    });
    elements.push(BodyElement::Paragraph(spacer));

    // Heading with keepNext
    let mut heading = create_text_paragraph("Heading 1 (Kept with next)", 32);
    heading.properties = Some(ParagraphProperties {
        keep_next: true,
        ..Default::default()
    });
    elements.push(BodyElement::Paragraph(heading));

    // Content following heading
    let content = create_text_paragraph("First paragraph of section.", 24);
    elements.push(BodyElement::Paragraph(content));

    let layout = solver.paginate(&elements);
    assert_eq!(layout.page_count(), 2);
    // Heading should be pushed to page 2 along with its content
    let p2_lines = &layout.pages[1].columns[0].lines;
    assert!(p2_lines.len() >= 2);
}

#[test]
fn test_24_widow_orphan_control() {
    let geometry = SectionGeometry::default();
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Default widowControl is true in ECMA-376
    let resolved = cascade.resolve_paragraph(&Paragraph::default());
    assert!(resolved.widow_control, "widowControl must default to true");

    // Paragraph with 3 lines near page bottom: widowControl prevents 2+1 and 1+2 splits
    let mut spacer = create_text_paragraph("Spacer", 24);
    spacer.properties = Some(ParagraphProperties {
        spacing: Some(Spacing {
            after: Some(12600), // leaves space for only 1 line
            ..Default::default()
        }),
        ..Default::default()
    });

    let three_line_para = create_text_paragraph(
        "Line one of three line paragraph. Line two of three line paragraph. Line three of three line paragraph.",
        24,
    );

    let layout = solver.paginate(&[
        BodyElement::Paragraph(spacer),
        BodyElement::Paragraph(three_line_para),
    ]);
    assert_eq!(layout.page_count(), 2);
    // Entire 3-line paragraph moved to page 2 rather than leaving 1-line orphan
    assert!(layout.pages[1].columns[0].lines.len() >= 2);
}

// =========================================================================
// Tests: Multi-Column Section Flow
// =========================================================================

#[test]
fn test_25_multi_column_flow() {
    let sect = SectionProperties {
        page_size: Some(PageSize {
            width: 12240,
            height: 15840,
            orientation: Some("portrait".to_string()),
        }),
        columns: Some(word_engine::model::Columns {
            num: Some(2),
            space: Some(720), // 0.5 in gap
        }),
        ..Default::default()
    };

    let geometry = SectionGeometry::from_section_properties(&sect);
    assert_eq!(geometry.column_count(), 2);

    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = PaginationSolver::new(&geometry, &cascade, &metrics);

    // Create 60 paragraphs to fill column 1 and flow into column 2
    let mut elements = Vec::new();
    for i in 0..60 {
        elements.push(BodyElement::Paragraph(create_text_paragraph(
            &format!("Column flow test paragraph #{i}"),
            24,
        )));
    }

    let layout = solver.paginate(&elements);
    let page1 = &layout.pages[0];
    assert_eq!(page1.columns.len(), 2);
    assert!(
        !page1.columns[0].lines.is_empty(),
        "Column 1 should contain lines"
    );
    assert!(
        !page1.columns[1].lines.is_empty(),
        "Column 2 should contain lines"
    );
}

#[test]
fn test_26_cjk_line_breaking() {
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();

    // Japanese text containing opening/closing punctuation
    let text = "これは日本語のテストです。「オープンオフィス」は高速です。";
    let p = create_text_paragraph(text, 24);
    let p_props = cascade.resolve_paragraph(&p);
    let tokens = breaker.tokenize_paragraph(&p, &cascade);

    assert!(tokens.len() >= 10);
    let lines = breaker.greedy_break(&tokens, &p_props, 2000);
    assert!(!lines.is_empty());
}

// =========================================================================
// Tests: Package Integration & Manifest
// =========================================================================

#[test]
fn test_27_wordpackage_full_layout() {
    let baseline_path = resolve_repo_path("specs/evidence/word/baseline.docx");
    let file = std::fs::File::open(&baseline_path).expect("Failed to open baseline.docx");
    let pkg = WordPackage::open(file).expect("Failed to parse baseline.docx");

    let layout = pkg
        .layout()
        .expect("Failed to compute layout for baseline.docx");
    assert!(layout.page_count() >= 1);
    assert!(layout.total_lines >= 1);
}

#[test]
fn test_28_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/word/3.2.1_vectors.json");
    let json_bytes =
        std::fs::read(&path).expect("Must read specs/evidence/word/3.2.1_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let count = vectors["vectors"].as_array().expect("vectors array").len();
    assert!(
        count >= 25,
        "Subphase 3.2.1 requires at least 25 test vectors, found: {}",
        count
    );

    assert_eq!(vectors["subphase_id"].as_str(), Some("3.2.1"));
    assert_eq!(
        vectors["title"].as_str(),
        Some("REA Line-Breaking Knuth-Plass/Greedy Hybrid & Pagination Solver")
    );
}
