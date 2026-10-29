//! Subphase 3.1.2 Test Suite: ECMA-376 Style Cascade, Numbering Lists & Section Geometry
//! Strictly conforms to GEC-v1.0 (Zero Panics in production code, Performance SLAs, Zero-Loss ECMA-376 Fidelity).

#![allow(clippy::field_reassign_with_default)]

use word_engine::model::*;
use word_engine::numbering::*;
use word_engine::section::*;
use word_engine::styles::*;
use word_engine::WordPackage;

fn resolve_repo_path(rel: &str) -> std::path::PathBuf {
    let direct = std::path::Path::new(rel);
    if direct.exists() {
        return direct.to_path_buf();
    }
    let from_manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel);
    if from_manifest.exists() {
        return from_manifest;
    }
    std::path::PathBuf::from(rel)
}

#[test]
fn test_01_docdefaults_resolution() {
    let mut sheet = StyleSheet::new();
    sheet.doc_defaults = DocDefaults {
        r_pr_default: Some(RunProperties {
            size_half_points: Some(24),
            font_ascii: Some("Georgia".to_string()),
            ..Default::default()
        }),
        p_pr_default: Some(ParagraphProperties {
            justification: Some("left".to_string()),
            spacing: Some(Spacing {
                before: Some(100),
                after: Some(200),
                line: Some(240),
                line_rule: Some("auto".to_string()),
            }),
            ..Default::default()
        }),
    };

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    let p_res = resolver.resolve_paragraph(&p);
    assert_eq!(p_res.justification.as_deref(), Some("left"));
    assert_eq!(p_res.spacing_before, 100);
    assert_eq!(p_res.spacing_after, 200);

    let r_res = resolver.resolve_run(&r, &p);
    assert_eq!(r_res.size_half_points, 24);
    assert_eq!(r_res.font_ascii, "Georgia");
}

#[test]
fn test_02_paragraph_style_based_on_chain() {
    let mut sheet = StyleSheet::new();

    // BaseStyle: 12pt (24 half-pts), Times New Roman, Left
    let mut base = Style::new("BaseStyle", StyleType::Paragraph);
    base.r_pr = Some(RunProperties {
        size_half_points: Some(24),
        font_ascii: Some("Times New Roman".to_string()),
        ..Default::default()
    });
    base.p_pr = Some(ParagraphProperties {
        justification: Some("left".to_string()),
        ..Default::default()
    });
    sheet.add_style(base);

    // Derived1 (based on BaseStyle): adds bold and red color, keeps font
    let mut derived1 = Style::new("Derived1", StyleType::Paragraph);
    derived1.based_on = Some("BaseStyle".to_string());
    derived1.r_pr = Some(RunProperties {
        bold: true,
        color: Some("FF0000".to_string()),
        ..Default::default()
    });
    sheet.add_style(derived1);

    // Derived2 (based on Derived1): overrides font to Arial, adds center justification
    let mut derived2 = Style::new("Derived2", StyleType::Paragraph);
    derived2.based_on = Some("Derived1".to_string());
    derived2.r_pr = Some(RunProperties {
        font_ascii: Some("Arial".to_string()),
        ..Default::default()
    });
    derived2.p_pr = Some(ParagraphProperties {
        justification: Some("center".to_string()),
        ..Default::default()
    });
    sheet.add_style(derived2);

    let resolver = StyleCascadeResolver::new(&sheet);

    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("Derived2".to_string()),
        ..Default::default()
    });
    let r = Run::default();

    let p_res = resolver.resolve_paragraph(&p);
    assert_eq!(p_res.justification.as_deref(), Some("center"));

    let r_res = resolver.resolve_run(&r, &p);
    assert_eq!(r_res.font_ascii, "Arial"); // Overridden by Derived2
    assert!(r_res.bold); // Inherited from Derived1
    assert_eq!(r_res.color.as_deref(), Some("FF0000")); // Inherited from Derived1
    assert_eq!(r_res.size_half_points, 24); // Inherited from BaseStyle
}

#[test]
fn test_03_character_style_based_on_chain() {
    let mut sheet = StyleSheet::new();

    let mut char_base = Style::new("CharBase", StyleType::Character);
    char_base.r_pr = Some(RunProperties {
        color: Some("0000FF".to_string()),
        underline: Some("single".to_string()),
        ..Default::default()
    });
    sheet.add_style(char_base);

    let mut char_derived = Style::new("CharDerived", StyleType::Character);
    char_derived.based_on = Some("CharBase".to_string());
    char_derived.r_pr = Some(RunProperties {
        size_half_points: Some(28),
        ..Default::default()
    });
    sheet.add_style(char_derived);

    let resolver = StyleCascadeResolver::new(&sheet);

    let p = Paragraph::default();
    let mut r = Run::default();
    let mut r_pr = RunProperties::default();
    let mut style_ref = RawXmlNode::new("w:rStyle");
    style_ref
        .attributes
        .push(("w:val".to_string(), "CharDerived".to_string()));
    r_pr.raw_elements.push(style_ref);
    r.properties = Some(r_pr);

    let r_res = resolver.resolve_run(&r, &p);
    assert_eq!(r_res.color.as_deref(), Some("0000FF"));
    assert_eq!(r_res.underline.as_deref(), Some("single"));
    assert_eq!(r_res.size_half_points, 28);
}

#[test]
fn test_04_cyclic_based_on_safety() {
    let mut sheet = StyleSheet::new();

    let mut a = Style::new("CyclicA", StyleType::Paragraph);
    a.based_on = Some("CyclicB".to_string());
    sheet.add_style(a);

    let mut b = Style::new("CyclicB", StyleType::Paragraph);
    b.based_on = Some("CyclicA".to_string());
    sheet.add_style(b);

    let resolver = StyleCascadeResolver::new(&sheet);
    let chain = resolver.get_inheritance_chain("CyclicA");

    // The chain must terminate safely without panic or infinite loop
    assert!(!chain.is_empty());
    assert!(chain.len() <= 2);
}

#[test]
fn test_05_direct_paragraph_formatting_overrides_style() {
    let mut sheet = StyleSheet::new();
    let mut s = Style::new("Normal", StyleType::Paragraph);
    s.p_pr = Some(ParagraphProperties {
        justification: Some("left".to_string()),
        spacing: Some(Spacing {
            before: Some(100),
            after: Some(100),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(s);

    let resolver = StyleCascadeResolver::new(&sheet);

    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("Normal".to_string()),
        justification: Some("right".to_string()), // Overrides left
        spacing: Some(Spacing {
            before: Some(300), // Overrides 100
            after: Some(100),
            ..Default::default()
        }),
        ..Default::default()
    });

    let res = resolver.resolve_paragraph(&p);
    assert_eq!(res.justification.as_deref(), Some("right"));
    assert_eq!(res.spacing_before, 300);
}

#[test]
fn test_06_direct_run_formatting_overrides_style() {
    let mut sheet = StyleSheet::new();
    let mut s = Style::new("Normal", StyleType::Paragraph);
    s.r_pr = Some(RunProperties {
        color: Some("333333".to_string()),
        size_half_points: Some(22),
        ..Default::default()
    });
    sheet.add_style(s);

    let resolver = StyleCascadeResolver::new(&sheet);

    let p = Paragraph::default();
    let mut r = Run::default();
    r.properties = Some(RunProperties {
        color: Some("FF00FF".to_string()), // Direct color override
        ..Default::default()
    });

    let res = resolver.resolve_run(&r, &p);
    assert_eq!(res.color.as_deref(), Some("FF00FF"));
    assert_eq!(res.size_half_points, 22); // Inherited
}

#[test]
fn test_07_toggle_property_bold_inversion() {
    let mut sheet = StyleSheet::new();
    let mut s = Style::new("BoldHeading", StyleType::Paragraph);
    s.r_pr = Some(RunProperties {
        bold: true,
        ..Default::default()
    });
    sheet.add_style(s);

    let resolver = StyleCascadeResolver::new(&sheet);

    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("BoldHeading".to_string()),
        ..Default::default()
    });

    // Run 1: No direct formatting -> inherits bold: true
    let r1 = Run::default();
    let res1 = resolver.resolve_run(&r1, &p);
    assert!(res1.bold);

    // Run 2: Direct bold toggle -> inverts to bold: false (ECMA-376 Part 1 §17.7.3)
    let mut r2 = Run::default();
    r2.properties = Some(RunProperties {
        bold: true,
        ..Default::default()
    });
    let res2 = resolver.resolve_run(&r2, &p);
    assert!(!res2.bold, "Direct bold should invert style bold to false");
}

#[test]
fn test_08_toggle_property_italic_inversion() {
    let mut sheet = StyleSheet::new();
    let mut s = Style::new("ItalicQuote", StyleType::Paragraph);
    s.r_pr = Some(RunProperties {
        italic: true,
        ..Default::default()
    });
    sheet.add_style(s);

    let resolver = StyleCascadeResolver::new(&sheet);

    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("ItalicQuote".to_string()),
        ..Default::default()
    });

    let r_normal = Run::default();
    assert!(resolver.resolve_run(&r_normal, &p).italic);

    let mut r_toggle = Run::default();
    r_toggle.properties = Some(RunProperties {
        italic: true,
        ..Default::default()
    });
    assert!(
        !resolver.resolve_run(&r_toggle, &p).italic,
        "Direct italic should invert style italic to false"
    );
}

#[test]
fn test_09_toggle_property_strike_inversion() {
    let mut sheet = StyleSheet::new();
    let mut s = Style::new("StrikethroughStyle", StyleType::Paragraph);
    s.r_pr = Some(RunProperties {
        strike: true,
        ..Default::default()
    });
    sheet.add_style(s);

    let resolver = StyleCascadeResolver::new(&sheet);

    let mut p = Paragraph::default();
    p.properties = Some(ParagraphProperties {
        style: Some("StrikethroughStyle".to_string()),
        ..Default::default()
    });

    let r_plain = Run::default();
    assert!(resolver.resolve_run(&r_plain, &p).strike);

    let mut r_toggle = Run::default();
    r_toggle.properties = Some(RunProperties {
        strike: true,
        ..Default::default()
    });
    assert!(!resolver.resolve_run(&r_toggle, &p).strike);
}

#[test]
fn test_10_table_style_whole_table() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("CustomTable", StyleType::Table);
    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::WholeTable),
        r_pr: Some(RunProperties {
            font_ascii: Some("Consolas".to_string()),
            size_half_points: Some(18),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    let res = resolver.resolve_table_cell_run(&r, &p, Some("CustomTable"), 1, 1, 4, 4);
    assert_eq!(res.font_ascii, "Consolas");
    assert_eq!(res.size_half_points, 18);
}

#[test]
fn test_11_table_style_first_row_override() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("GridTable", StyleType::Table);

    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::FirstRow),
        r_pr: Some(RunProperties {
            bold: true,
            color: Some("FFFFFF".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    // Row 0 (Header Row) -> Bold & White
    let header_cell_res = resolver.resolve_table_cell_run(&r, &p, Some("GridTable"), 0, 0, 5, 3);
    assert!(header_cell_res.bold);
    assert_eq!(header_cell_res.color.as_deref(), Some("FFFFFF"));

    // Row 1 (Body Row) -> Not bold
    let body_cell_res = resolver.resolve_table_cell_run(&r, &p, Some("GridTable"), 1, 0, 5, 3);
    assert!(!body_cell_res.bold);
}

#[test]
fn test_12_table_style_last_row_override() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("FinancialTable", StyleType::Table);

    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::LastRow),
        r_pr: Some(RunProperties {
            bold: true,
            underline: Some("double".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    // Total Row (row 4 out of 5)
    let total_row_res = resolver.resolve_table_cell_run(&r, &p, Some("FinancialTable"), 4, 1, 5, 2);
    assert!(total_row_res.bold);
    assert_eq!(total_row_res.underline.as_deref(), Some("double"));

    // Middle Row (row 2 out of 5)
    let mid_row_res = resolver.resolve_table_cell_run(&r, &p, Some("FinancialTable"), 2, 1, 5, 2);
    assert!(!mid_row_res.bold);
}

#[test]
fn test_13_table_style_horizontal_banding() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("BandedTable", StyleType::Table);

    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::Band1Horz),
        r_pr: Some(RunProperties {
            color: Some("111111".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    // Row 1 (Odd body row) is Band1Horz
    let row1_res = resolver.resolve_table_cell_run(&r, &p, Some("BandedTable"), 1, 0, 6, 2);
    assert_eq!(row1_res.color.as_deref(), Some("111111"));

    // Row 2 (Even body row) is not Band1Horz
    let row2_res = resolver.resolve_table_cell_run(&r, &p, Some("BandedTable"), 2, 0, 6, 2);
    assert_ne!(row2_res.color.as_deref(), Some("111111"));
}

#[test]
fn test_14_table_style_vertical_banding() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("ColBandedTable", StyleType::Table);

    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::Band1Vert),
        r_pr: Some(RunProperties {
            color: Some("222222".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    // Col 1 is Band1Vert
    let col1_res = resolver.resolve_table_cell_run(&r, &p, Some("ColBandedTable"), 1, 1, 4, 4);
    assert_eq!(col1_res.color.as_deref(), Some("222222"));

    // Col 2 is not Band1Vert
    let col2_res = resolver.resolve_table_cell_run(&r, &p, Some("ColBandedTable"), 1, 2, 4, 4);
    assert_ne!(col2_res.color.as_deref(), Some("222222"));
}

#[test]
fn test_15_table_style_corner_cells() {
    let mut sheet = StyleSheet::new();
    let mut ts = Style::new("CornerTable", StyleType::Table);

    ts.tbl_style_pr.push(TableStyleOverride {
        override_type: Some(TableStyleOverrideType::NwCell),
        r_pr: Some(RunProperties {
            bold: true,
            ..Default::default()
        }),
        ..Default::default()
    });
    sheet.add_style(ts);

    let resolver = StyleCascadeResolver::new(&sheet);
    let p = Paragraph::default();
    let r = Run::default();

    let nw_res = resolver.resolve_table_cell_run(&r, &p, Some("CornerTable"), 0, 0, 3, 3);
    assert!(nw_res.bold);

    let other_res = resolver.resolve_table_cell_run(&r, &p, Some("CornerTable"), 0, 1, 3, 3);
    assert!(!other_res.bold);
}

#[test]
fn test_16_numbering_decimal_format() {
    let fmt = NumberFormat::Decimal;
    assert_eq!(fmt.format_number(1), "1");
    assert_eq!(fmt.format_number(2), "2");
    assert_eq!(fmt.format_number(42), "42");
    assert_eq!(fmt.format_number(100), "100");
}

#[test]
fn test_17_numbering_lower_letter_format() {
    let fmt = NumberFormat::LowerLetter;
    assert_eq!(fmt.format_number(1), "a");
    assert_eq!(fmt.format_number(2), "b");
    assert_eq!(fmt.format_number(26), "z");
    assert_eq!(fmt.format_number(27), "aa");
    assert_eq!(fmt.format_number(28), "ab");
}

#[test]
fn test_18_numbering_upper_letter_format() {
    let fmt = NumberFormat::UpperLetter;
    assert_eq!(fmt.format_number(1), "A");
    assert_eq!(fmt.format_number(26), "Z");
    assert_eq!(fmt.format_number(27), "AA");
    assert_eq!(fmt.format_number(52), "AZ");
}

#[test]
fn test_19_numbering_lower_roman_format() {
    let fmt = NumberFormat::LowerRoman;
    assert_eq!(fmt.format_number(1), "i");
    assert_eq!(fmt.format_number(4), "iv");
    assert_eq!(fmt.format_number(9), "ix");
    assert_eq!(fmt.format_number(14), "xiv");
    assert_eq!(fmt.format_number(40), "xl");
    assert_eq!(fmt.format_number(90), "xc");
    assert_eq!(fmt.format_number(2026), "mmxxvi");
}

#[test]
fn test_20_numbering_upper_roman_format() {
    let fmt = NumberFormat::UpperRoman;
    assert_eq!(fmt.format_number(1), "I");
    assert_eq!(fmt.format_number(4), "IV");
    assert_eq!(fmt.format_number(9), "IX");
    assert_eq!(fmt.format_number(2026), "MMXXVI");
}

#[test]
fn test_21_numbering_bullet_format() {
    let fmt = NumberFormat::Bullet;
    assert_eq!(fmt.format_number(1), "\u{2022}");
    assert_eq!(fmt.format_number(2), "\u{2022}");
}

#[test]
fn test_22_numbering_ordinal_format() {
    let fmt = NumberFormat::Ordinal;
    assert_eq!(fmt.format_number(1), "1st");
    assert_eq!(fmt.format_number(2), "2nd");
    assert_eq!(fmt.format_number(3), "3rd");
    assert_eq!(fmt.format_number(4), "4th");
    assert_eq!(fmt.format_number(11), "11th");
    assert_eq!(fmt.format_number(21), "21st");
}

#[test]
fn test_23_numbering_cardinal_text() {
    let fmt = NumberFormat::CardinalText;
    assert_eq!(fmt.format_number(1), "one");
    assert_eq!(fmt.format_number(5), "five");
    assert_eq!(fmt.format_number(10), "ten");
}

#[test]
fn test_24_multi_level_list_template_interpolation() {
    let mut numbering = Numbering::new();
    let mut abs = AbstractNum::new(1);

    abs.add_level(Level::new(0, NumberFormat::Decimal, "%1."));
    abs.add_level(Level::new(1, NumberFormat::Decimal, "%1.%2."));
    abs.add_level(Level::new(2, NumberFormat::Decimal, "%1.%2.%3."));
    numbering.add_abstract_num(abs);

    numbering.add_num(Num::new(1, 1));

    let mut engine = NumberingEngine::new();

    // Section 1
    let item1 = engine.next_item(1, 0, &numbering).unwrap();
    assert_eq!(item1.prefix_text, "1.");

    // Section 1.1
    let item1_1 = engine.next_item(1, 1, &numbering).unwrap();
    assert_eq!(item1_1.prefix_text, "1.1.");

    // Section 1.1.1
    let item1_1_1 = engine.next_item(1, 2, &numbering).unwrap();
    assert_eq!(item1_1_1.prefix_text, "1.1.1.");
}

#[test]
fn test_25_multi_level_list_hybrid_formatting() {
    let mut numbering = Numbering::new();
    let mut abs = AbstractNum::new(2);

    abs.add_level(Level::new(0, NumberFormat::Decimal, "%1."));
    abs.add_level(Level::new(1, NumberFormat::LowerLetter, "%2)"));
    abs.add_level(Level::new(2, NumberFormat::LowerRoman, "(%3)"));
    numbering.add_abstract_num(abs);
    numbering.add_num(Num::new(2, 2));

    let mut engine = NumberingEngine::new();

    assert_eq!(
        engine.next_item(2, 0, &numbering).unwrap().prefix_text,
        "1."
    );
    assert_eq!(
        engine.next_item(2, 1, &numbering).unwrap().prefix_text,
        "a)"
    );
    assert_eq!(
        engine.next_item(2, 2, &numbering).unwrap().prefix_text,
        "(i)"
    );
    assert_eq!(
        engine.next_item(2, 2, &numbering).unwrap().prefix_text,
        "(ii)"
    );
    assert_eq!(
        engine.next_item(2, 1, &numbering).unwrap().prefix_text,
        "b)"
    );
}

#[test]
fn test_26_numbering_level_restart_on_parent_increment() {
    let mut numbering = Numbering::new();
    let mut abs = AbstractNum::new(3);

    abs.add_level(Level::new(0, NumberFormat::Decimal, "%1."));
    abs.add_level(Level::new(1, NumberFormat::LowerLetter, "%2."));
    numbering.add_abstract_num(abs);
    numbering.add_num(Num::new(3, 3));

    let mut engine = NumberingEngine::new();

    // 1.
    assert_eq!(
        engine.next_item(3, 0, &numbering).unwrap().prefix_text,
        "1."
    );
    // 1. a.
    assert_eq!(
        engine.next_item(3, 1, &numbering).unwrap().prefix_text,
        "a."
    );
    // 1. b.
    assert_eq!(
        engine.next_item(3, 1, &numbering).unwrap().prefix_text,
        "b."
    );

    // Increment level 0 -> 2. (Level 1 should restart to 'a.')
    assert_eq!(
        engine.next_item(3, 0, &numbering).unwrap().prefix_text,
        "2."
    );
    assert_eq!(
        engine.next_item(3, 1, &numbering).unwrap().prefix_text,
        "a."
    );
}

#[test]
fn test_27_numbering_level_override_start() {
    let mut numbering = Numbering::new();
    let mut abs = AbstractNum::new(4);
    abs.add_level(Level::new(0, NumberFormat::Decimal, "%1."));
    numbering.add_abstract_num(abs);

    let mut num = Num::new(4, 4);
    num.lvl_overrides.insert(
        0,
        LevelOverride {
            ilvl: 0,
            start_override: Some(10), // Starts at 10 instead of 1
            lvl: None,
        },
    );
    numbering.add_num(num);

    let mut engine = NumberingEngine::new();
    assert_eq!(
        engine.next_item(4, 0, &numbering).unwrap().prefix_text,
        "10."
    );
    assert_eq!(
        engine.next_item(4, 0, &numbering).unwrap().prefix_text,
        "11."
    );
}

#[test]
fn test_28_section_geometry_margins_and_content_box() {
    let mut sect = SectionProperties::default();
    sect.page_size = Some(PageSize {
        width: 12240,  // 8.5 inches
        height: 15840, // 11 inches
        orientation: Some("portrait".to_string()),
    });
    sect.page_margin = Some(PageMargin {
        top: 1440,    // 1 inch
        bottom: 1440, // 1 inch
        left: 1440,   // 1 inch
        right: 1440,  // 1 inch
        header: 720,
        footer: 720,
        gutter: 0,
    });

    let geom = SectionGeometry::from_section_properties(&sect);

    assert_eq!(geom.page_width, 12240);
    assert_eq!(geom.page_height, 15840);
    assert_eq!(geom.content_width(), 9360); // 12240 - 2880
    assert_eq!(geom.content_height(), 12960); // 15840 - 2880
    assert_eq!(geom.column_count(), 1);
}

#[test]
fn test_29_section_geometry_multi_column_division() {
    let mut sect = SectionProperties::default();
    sect.page_size = Some(PageSize {
        width: 12240,
        height: 15840,
        orientation: Some("portrait".to_string()),
    });
    sect.page_margin = Some(PageMargin {
        top: 1440,
        bottom: 1440,
        left: 1440,
        right: 1440,
        header: 720,
        footer: 720,
        gutter: 0,
    });
    sect.columns = Some(Columns {
        num: Some(3),
        space: Some(720), // 0.5 in gap
    });

    let geom = SectionGeometry::from_section_properties(&sect);

    assert_eq!(geom.column_count(), 3);
    // Total available: 9360. Gaps: 2 * 720 = 1440. Cols: (9360 - 1440) / 3 = 7920 / 3 = 2640 twips each.
    assert_eq!(geom.column_width(0), 2640);
    assert_eq!(geom.column_width(1), 2640);
    assert_eq!(geom.column_width(2), 2640);
}

#[test]
fn test_30_section_geometry_unit_conversions() {
    // 1440 twips = 1 inch = 72 points = 25.4 mm
    let twips = 1440;

    let pts = SectionGeometry::twips_to_points(twips);
    assert!((pts - 72.0).abs() < 1e-6);
    assert_eq!(SectionGeometry::points_to_twips(72.0), 1440);

    let inches = SectionGeometry::twips_to_inches(twips);
    assert!((inches - 1.0).abs() < 1e-6);
    assert_eq!(SectionGeometry::inches_to_twips(1.0), 1440);

    let mm = SectionGeometry::twips_to_mm(twips);
    assert!((mm - 25.4).abs() < 1e-6);
    assert_eq!(SectionGeometry::mm_to_twips(25.4), 1440);
}

#[test]
fn test_31_wordpackage_styles_roundtrip() {
    let path = resolve_repo_path("specs/evidence/word/baseline.docx");
    let pkg = WordPackage::from_file(&path).expect("Failed to open baseline.docx");

    // baseline.docx contains styles.xml
    assert!(
        pkg.styles.get_style("Normal").is_some() || !pkg.styles.styles.is_empty(),
        "baseline.docx styles must be parsed into StyleSheet"
    );

    let bytes = pkg.to_bytes().expect("Failed to serialize package");
    let reloaded = WordPackage::from_bytes(&bytes).expect("Failed to reload package");

    assert_eq!(reloaded.styles.styles.len(), pkg.styles.styles.len());
}

#[test]
fn test_32_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/word/3.1.2_vectors.json");
    let json_bytes =
        std::fs::read(&path).expect("Must read specs/evidence/word/3.1.2_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let count = vectors["vectors"].as_array().expect("vectors array").len();
    assert!(
        count >= 30,
        "Subphase 3.1.2 requires at least 30 test vectors, found: {}",
        count
    );

    assert_eq!(vectors["subphase_id"].as_str(), Some("3.1.2"));
    assert_eq!(
        vectors["title"].as_str(),
        Some("ECMA-376 Style Cascade, Numbering Lists & Section Geometry")
    );
}
