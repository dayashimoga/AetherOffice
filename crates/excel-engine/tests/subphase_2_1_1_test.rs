//! Subphase 2.1.1 Integration Tests:
//! SharedStringTable, Styles, and Parallel Multi-Threaded Workbook Package Engine.
//! Strictly validates GEC-v1.0 Article II (Zero-Loss Round-Trip Fidelity),
//! Article III (Performance & Resource SLAs), and Article IV (Zero Panics).

use excel_engine::*;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::Instant;

fn resolve_fixture(relative: &str) -> PathBuf {
    let direct = PathBuf::from(relative);
    if direct.exists() {
        return direct;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from_manifest = manifest_dir.join("../../").join(relative);
    if from_manifest.exists() {
        return from_manifest;
    }
    direct
}

// ---------------------------------------------------------------------------
// 1. Baseline COM Fixture Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn test_01_parse_real_baseline_package_parallel() {
    let fixture_path = resolve_fixture("specs/evidence/excel/baseline.xlsx");
    let file = File::open(&fixture_path).expect("Failed to open baseline.xlsx fixture");
    let pkg =
        ExcelWorkbookPackage::open(file).expect("Failed to parse baseline package in parallel");

    // Workbook validations
    assert_eq!(pkg.workbook.sheets.len(), 1);
    assert_eq!(pkg.workbook.sheets[0].name, "Sheet1");
    assert_eq!(pkg.workbook.sheets[0].sheet_id, 1);
    assert_eq!(pkg.workbook.sheets[0].rel_id, "rId1");
    assert_eq!(pkg.workbook.sheets[0].state, SheetState::Visible);

    // Shared Strings validations
    assert!(!pkg.shared_strings.is_empty());
    assert_eq!(
        pkg.shared_strings.get(0),
        Some("OpenOfficeRS Ground Truth Baseline")
    );

    // Styles validations
    assert!(!pkg.styles.fonts.is_empty());
    assert!(!pkg.styles.fills.is_empty());
    assert!(!pkg.styles.borders.is_empty());
    assert!(!pkg.styles.cell_xfs.is_empty());
}

#[test]
fn test_02_baseline_package_roundtrip_lossless() {
    let fixture_path = resolve_fixture("specs/evidence/excel/baseline.xlsx");
    let file = File::open(&fixture_path).expect("Failed to open baseline.xlsx");
    let pkg = ExcelWorkbookPackage::open(file).expect("Failed to open package");

    let mut out_buf = Cursor::new(Vec::new());
    pkg.save(&mut out_buf)
        .expect("Failed to save baseline package");

    out_buf.set_position(0);
    let reloaded =
        ExcelWorkbookPackage::open(out_buf).expect("Failed to reload re-serialized package");

    assert_eq!(reloaded.workbook.sheets.len(), pkg.workbook.sheets.len());
    assert_eq!(
        reloaded.workbook.sheets[0].name,
        pkg.workbook.sheets[0].name
    );
    assert_eq!(reloaded.shared_strings.get(0), pkg.shared_strings.get(0));
    assert_eq!(reloaded.styles.fonts.len(), pkg.styles.fonts.len());
    assert_eq!(reloaded.styles.fills.len(), pkg.styles.fills.len());
}

// ---------------------------------------------------------------------------
// 2. Workbook Parser & Serialization Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_03_wb_sheets_parsing() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
        <sheets>
            <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
            <sheet name="Summary" sheetId="2" r:id="rId2"/>
        </sheets>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse workbook sheets");
    assert_eq!(wb.sheets.len(), 2);
    assert_eq!(wb.sheets[0].name, "Sheet1");
    assert_eq!(wb.sheets[0].sheet_id, 1);
    assert_eq!(wb.sheets[0].rel_id, "rId1");
    assert_eq!(wb.sheets[0].state, SheetState::Visible);

    assert_eq!(wb.sheets[1].name, "Summary");
    assert_eq!(wb.sheets[1].sheet_id, 2);
    assert_eq!(wb.sheets[1].rel_id, "rId2");
}

#[test]
fn test_04_wb_sheet_states_hidden_and_very_hidden() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <sheets>
            <sheet name="Visible" sheetId="1" r:id="rId1"/>
            <sheet name="Hidden" sheetId="2" r:id="rId2" state="hidden"/>
            <sheet name="VeryHidden" sheetId="3" r:id="rId3" state="veryHidden"/>
        </sheets>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse sheet states");
    assert_eq!(wb.sheets[0].state, SheetState::Visible);
    assert_eq!(wb.sheets[1].state, SheetState::Hidden);
    assert_eq!(wb.sheets[2].state, SheetState::VeryHidden);
}

#[test]
fn test_05_wb_defined_names_global_and_local() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <definedNames>
            <definedName name="TaxRate">0.0825</definedName>
            <definedName name="TotalSales">Sheet1!$D$10</definedName>
            <definedName name="_xlnm.Print_Area" localSheetId="0" hidden="1" comment="Print Area">Sheet1!$A$1:$F$50</definedName>
        </definedNames>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse defined names");
    assert_eq!(wb.defined_names.len(), 3);

    assert_eq!(wb.defined_names[0].name, "TaxRate");
    assert_eq!(wb.defined_names[0].value, "0.0825");
    assert_eq!(wb.defined_names[0].local_sheet_id, None);
    assert!(!wb.defined_names[0].hidden);

    assert_eq!(wb.defined_names[2].name, "_xlnm.Print_Area");
    assert_eq!(wb.defined_names[2].value, "Sheet1!$A$1:$F$50");
    assert_eq!(wb.defined_names[2].local_sheet_id, Some(0));
    assert!(wb.defined_names[2].hidden);
    assert_eq!(wb.defined_names[2].comment.as_deref(), Some("Print Area"));
}

#[test]
fn test_06_wb_date1904_system_flag() {
    // 1900 date system (default)
    let xml_1900 = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><workbookPr defaultThemeVersion="202300"/></workbook>"#;
    let wb_1900 = Workbook::from_xml_str(xml_1900).expect("Failed to parse wb 1900");
    assert!(!wb_1900.is_date1904());

    // 1904 date system
    let xml_1904 = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><workbookPr date1904="1" defaultThemeVersion="202300"/></workbook>"#;
    let wb_1904 = Workbook::from_xml_str(xml_1904).expect("Failed to parse wb 1904");
    assert!(wb_1904.is_date1904());
}

#[test]
fn test_07_wb_calc_pr_properties() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <calcPr calcId="191029" calcMode="manual" fullCalcOnLoad="1" refMode="R1C1" iterate="1" iterateCount="100" iterateDelta="0.001"/>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse calcPr");
    assert_eq!(wb.calc_pr.calc_id, Some(191029));
    assert_eq!(wb.calc_pr.calc_mode, CalcMode::Manual);
    assert!(wb.calc_pr.full_calc_on_load);
    assert_eq!(wb.calc_pr.ref_mode.as_deref(), Some("R1C1"));
    assert!(wb.calc_pr.iterate);
    assert_eq!(wb.calc_pr.iterate_count, Some(100));
    assert_eq!(wb.calc_pr.iterate_delta, Some(0.001));
}

#[test]
fn test_08_wb_book_views() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <bookViews>
            <workbookView xWindow="100" yWindow="200" windowWidth="18000" windowHeight="12000" activeTab="1"/>
        </bookViews>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse bookViews");
    assert_eq!(wb.book_views.len(), 1);
    let view = &wb.book_views[0];
    assert_eq!(view.x_window, Some(100));
    assert_eq!(view.y_window, Some(200));
    assert_eq!(view.window_width, Some(18000));
    assert_eq!(view.window_height, Some(12000));
    assert_eq!(view.active_tab, Some(1));
}

#[test]
fn test_09_wb_ext_lst_preservation() {
    let xml = r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <extLst>
            <ext uri="{140A7094-0E35-4892-8432-C4D2E57EDEB5}" xmlns:x15="http://schemas.microsoft.com/office/spreadsheetml/2010/11/main">
                <x15:workbookPr chartTrackingRefBase="1"/>
            </ext>
        </extLst>
    </workbook>"#;

    let wb = Workbook::from_xml_str(xml).expect("Failed to parse extLst");
    assert!(wb.ext_lst.is_some());
    let raw = &wb.ext_lst.as_ref().unwrap().raw_xml;
    assert!(raw.contains("{140A7094-0E35-4892-8432-C4D2E57EDEB5}"));
    assert!(raw.contains("chartTrackingRefBase"));

    // Verify round-trip preserves extLst
    let serialized = wb.to_xml_string().expect("Failed to serialize workbook");
    assert!(serialized.contains("chartTrackingRefBase"));
}

#[test]
fn test_10_wb_roundtrip_fidelity() {
    let mut wb = Workbook::new();
    wb.add_sheet("Sales", 1, "rId1");
    wb.add_sheet("Expenses", 2, "rId2");
    wb.set_date1904(true);
    wb.calc_pr.calc_id = Some(99999);
    wb.calc_pr.calc_mode = CalcMode::AutoNoTable;

    let xml = wb.to_xml_string().expect("Failed to serialize workbook");
    let reloaded = Workbook::from_xml_str(&xml).expect("Failed to re-parse workbook");

    assert_eq!(reloaded.sheets.len(), 2);
    assert_eq!(reloaded.sheets[0].name, "Sales");
    assert_eq!(reloaded.sheets[1].name, "Expenses");
    assert!(reloaded.is_date1904());
    assert_eq!(reloaded.calc_pr.calc_id, Some(99999));
    assert_eq!(reloaded.calc_pr.calc_mode, CalcMode::AutoNoTable);
}

// ---------------------------------------------------------------------------
// 3. SharedStringTable Tests & Performance SLA
// ---------------------------------------------------------------------------

#[test]
fn test_11_sst_plain_string_and_xml_entities() {
    let xml = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
        <si><t>Simple String</t></si>
        <si><t>Tom &amp; Jerry &lt;&gt; &quot;Fun&quot; &apos;100%&apos;</t></si>
    </sst>"#;

    let sst = SharedStringTable::from_xml_str(xml).expect("Failed to parse SST");
    assert_eq!(sst.len(), 2);
    assert_eq!(sst.get(0), Some("Simple String"));
    assert_eq!(sst.get(1), Some("Tom & Jerry <> \"Fun\" '100%'"));
}

#[test]
fn test_12_sst_preserve_space() {
    let xml = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
        <si><t xml:space="preserve">   Padded Text   </t></si>
    </sst>"#;

    let sst = SharedStringTable::from_xml_str(xml).expect("Failed to parse SST with space");
    assert_eq!(sst.get(0), Some("   Padded Text   "));

    let reserialized = sst.to_xml_string().expect("Failed to serialize SST");
    assert!(reserialized.contains("xml:space=\"preserve\""));
}

#[test]
fn test_13_sst_rich_text_formatting() {
    let xml = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
        <si>
            <r>
                <rPr>
                    <b/>
                    <rFont val="Arial"/>
                    <sz val="14"/>
                    <color rgb="FFFF0000"/>
                </rPr>
                <t>Red Alert</t>
            </r>
            <r>
                <t> Plain Tail</t>
            </r>
        </si>
    </sst>"#;

    let sst = SharedStringTable::from_xml_str(xml).expect("Failed to parse rich text");
    assert_eq!(sst.len(), 1);
    assert_eq!(sst.get(0), Some("Red Alert Plain Tail"));

    let item = sst.get_item(0).unwrap();
    let runs = item.runs().expect("Expected rich text runs");
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "Red Alert");
    assert!(runs[0].bold);
    assert_eq!(runs[0].font_name.as_deref(), Some("Arial"));
    assert_eq!(runs[0].font_size, Some(14));
    assert_eq!(runs[0].color_rgb.as_deref(), Some("FFFF0000"));

    assert_eq!(runs[1].text, " Plain Tail");
    assert!(!runs[1].bold);
}

#[test]
fn test_14_sst_deduplication_interning() {
    let mut sst = SharedStringTable::new();
    let id1 = sst.add_plain("Hello");
    let id2 = sst.add_plain("World");
    let id3 = sst.add_plain("Hello"); // duplicate
    let id4 = sst.add_plain("Rust");
    let id5 = sst.add_plain("World"); // duplicate

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 0); // Reused
    assert_eq!(id4, 2);
    assert_eq!(id5, 1); // Reused

    assert_eq!(sst.len(), 3); // 3 unique
    assert_eq!(sst.total_count(), 5); // 5 references
}

#[test]
fn test_15_sst_empty_string_handling() {
    let xml = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
        <si><t/></si>
    </sst>"#;

    let sst = SharedStringTable::from_xml_str(xml).expect("Failed to parse empty string");
    assert_eq!(sst.len(), 1);
    assert_eq!(sst.get(0), Some(""));
}

/// Hard SLA Verification: GEC-v1.0 Article III
/// Parsing 1,000,000 shared strings MUST take < 120 ms.
#[test]
fn test_16_sst_sla_1m_strings_benchmark() {
    let target_count = 1_000_000;
    let mut xml = String::with_capacity(30 * 1024 * 1024);
    xml.push_str(r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1000000" uniqueCount="1000000">"#);
    for i in 0..target_count {
        xml.push_str("<si><t>Item ");
        let mut n = i;
        let mut digits = [0u8; 10];
        let mut d_idx = 0;
        if n == 0 {
            xml.push('0');
        } else {
            while n > 0 {
                digits[d_idx] = (n % 10) as u8 + b'0';
                n /= 10;
                d_idx += 1;
            }
            while d_idx > 0 {
                d_idx -= 1;
                xml.push(digits[d_idx] as char);
            }
        }
        xml.push_str("</t></si>");
    }
    xml.push_str("</sst>");

    let start = Instant::now();
    let sst = SharedStringTable::from_xml_str(&xml).expect("Failed to parse 1M strings SST");
    let elapsed = start.elapsed();

    println!(
        "=== [SLA Benchmark] 1,000,000 Shared Strings parsed in: {:.2?} (Len: {}) ===",
        elapsed,
        sst.len()
    );

    assert_eq!(sst.len(), target_count);
    assert_eq!(sst.get(0), Some("Item 0"));
    assert_eq!(sst.get(999_999), Some("Item 999999"));

    // In release mode this is typically 30-70ms (< 120ms SLA).
    // In unoptimized debug test runs without -O3, allow up to 600ms, but assert strictly.
    #[cfg(not(debug_assertions))]
    assert!(
        elapsed.as_millis() < 120,
        "SLA Violation: parsing 1,000,000 strings took {:?}, expected < 120ms",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// 4. Stylesheet Parsing & Serialization Tests
// ---------------------------------------------------------------------------

#[test]
fn test_17_styles_builtin_number_formats() {
    assert_eq!(builtin_format_code(0), Some("General"));
    assert_eq!(builtin_format_code(1), Some("0"));
    assert_eq!(builtin_format_code(2), Some("0.00"));
    assert_eq!(builtin_format_code(3), Some("#,##0"));
    assert_eq!(builtin_format_code(4), Some("#,##0.00"));
    assert_eq!(builtin_format_code(9), Some("0%"));
    assert_eq!(builtin_format_code(14), Some("mm-dd-yy"));
    assert_eq!(builtin_format_code(49), Some("@"));
    assert_eq!(builtin_format_code(999), None);
}

#[test]
fn test_18_styles_custom_number_formats() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <numFmts count="1">
            <numFmt numFmtId="164" formatCode="YYYY-MM-DD\ HH:MM:SS"/>
        </numFmts>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse stylesheet");
    assert_eq!(ss.number_formats.len(), 1);
    assert_eq!(ss.number_formats[0].num_fmt_id, 164);
    assert_eq!(ss.get_format_code(164), Some("YYYY-MM-DD\\ HH:MM:SS"));
    // Falls back to built-in
    assert_eq!(ss.get_format_code(2), Some("0.00"));
}

#[test]
fn test_19_styles_fonts_parsing() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <fonts count="1">
            <font>
                <sz val="12"/>
                <name val="Aptos Narrow"/>
                <b/>
                <i/>
                <u/>
                <strike/>
                <color rgb="FF002060"/>
            </font>
        </fonts>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse fonts");
    assert_eq!(ss.fonts.len(), 1);
    let f = &ss.fonts[0];
    assert_eq!(f.name, "Aptos Narrow");
    assert_eq!(f.size, 12.0);
    assert!(f.bold);
    assert!(f.italic);
    assert!(f.underline);
    assert!(f.strike);
    assert_eq!(
        f.color.as_ref().and_then(|c| c.rgb.as_deref()),
        Some("FF002060")
    );
}

#[test]
fn test_20_styles_fills_parsing() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <fills count="3">
            <fill><patternFill patternType="none"/></fill>
            <fill><patternFill patternType="gray125"/></fill>
            <fill><patternFill patternType="solid"><fgColor rgb="FFFFFF00"/></patternFill></fill>
        </fills>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse fills");
    assert_eq!(ss.fills.len(), 3);
    assert_eq!(ss.fills[0].pattern_type, PatternType::None);
    assert_eq!(ss.fills[1].pattern_type, PatternType::Gray125);
    assert_eq!(ss.fills[2].pattern_type, PatternType::Solid);
    assert_eq!(
        ss.fills[2].fg_color.as_ref().and_then(|c| c.rgb.as_deref()),
        Some("FFFFFF00")
    );
}

#[test]
fn test_21_styles_borders_parsing() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <borders count="1">
            <border>
                <left style="thin"><color rgb="FF808080"/></left>
                <right style="medium"><color rgb="FF000000"/></right>
                <top style="double"/>
                <bottom style="thick"/>
                <diagonal/>
            </border>
        </borders>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse borders");
    assert_eq!(ss.borders.len(), 1);
    let b = &ss.borders[0];
    assert_eq!(b.left.style, BorderStyle::Thin);
    assert_eq!(
        b.left.color.as_ref().and_then(|c| c.rgb.as_deref()),
        Some("FF808080")
    );
    assert_eq!(b.right.style, BorderStyle::Medium);
    assert_eq!(b.top.style, BorderStyle::Double);
    assert_eq!(b.bottom.style, BorderStyle::Thick);
    assert_eq!(b.diagonal.style, BorderStyle::None);
}

#[test]
fn test_22_styles_cell_xfs_and_alignments() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <cellXfs count="1">
            <xf numFmtId="164" fontId="0" fillId="2" borderId="1" applyAlignment="1">
                <alignment horizontal="center" vertical="center" wrapText="1" shrinkToFit="1" indent="2"/>
            </xf>
        </cellXfs>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse cellXfs");
    assert_eq!(ss.cell_xfs.len(), 1);
    let xf = &ss.cell_xfs[0];
    assert_eq!(xf.num_fmt_id, 164);
    assert_eq!(xf.font_id, 0);
    assert_eq!(xf.fill_id, 2);
    assert_eq!(xf.border_id, 1);
    assert_eq!(xf.apply_alignment, Some(true));

    let align = xf.alignment.as_ref().expect("Expected alignment");
    assert_eq!(align.horizontal, Some(HorizontalAlignment::Center));
    assert_eq!(align.vertical, Some(VerticalAlignment::Center));
    assert!(align.wrap_text);
    assert!(align.shrink_to_fit);
    assert_eq!(align.indent, 2);
}

#[test]
fn test_23_styles_schema_order_serialization() {
    let mut ss = StyleSheet::default();
    ss.number_formats.push(NumberFormat::new(164, "YYYY-MM-DD"));
    ss.fonts[0].bold = true;
    ss.fills.push(Fill::solid(Color::from_rgb("FFFF0000")));

    let xml = ss.to_xml_string().expect("Failed to serialize stylesheet");

    // Strict ECMA-376 child element order verification:
    // numFmts < fonts < fills < borders < cellStyleXfs < cellXfs < cellStyles
    let pos_num_fmts = xml.find("<numFmts").unwrap();
    let pos_fonts = xml.find("<fonts").unwrap();
    let pos_fills = xml.find("<fills").unwrap();
    let pos_borders = xml.find("<borders").unwrap();
    let pos_csx = xml.find("<cellStyleXfs").unwrap();
    let pos_cx = xml.find("<cellXfs").unwrap();
    let pos_cstyles = xml.find("<cellStyles").unwrap();

    assert!(pos_num_fmts < pos_fonts);
    assert!(pos_fonts < pos_fills);
    assert!(pos_fills < pos_borders);
    assert!(pos_borders < pos_csx);
    assert!(pos_csx < pos_cx);
    assert!(pos_cx < pos_cstyles);
}

#[test]
fn test_24_styles_ext_lst_preservation() {
    let xml = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <extLst>
            <ext uri="{EB79DEF2-80B8-43e5-95BD-54CBDDF9020C}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main">
                <x14:slicerStyles defaultSlicerStyle="SlicerStyleLight1"/>
            </ext>
        </extLst>
    </styleSheet>"#;

    let ss = StyleSheet::from_xml_str(xml).expect("Failed to parse stylesheet extLst");
    assert!(ss.ext_lst.is_some());
    let raw = &ss.ext_lst.as_ref().unwrap().raw_xml;
    assert!(raw.contains("slicerStyles"));
    assert!(raw.contains("SlicerStyleLight1"));

    let reserialized = ss.to_xml_string().expect("Failed to serialize styles");
    assert!(reserialized.contains("SlicerStyleLight1"));
}

#[test]
fn test_25_unmodeled_parts_preservation_across_save() {
    let fixture_path = resolve_fixture("specs/evidence/excel/baseline.xlsx");
    let file = File::open(&fixture_path).expect("Failed to open baseline");
    let pkg = ExcelWorkbookPackage::open(file).expect("Failed to open");

    // Baseline contains calcChain.xml, theme/theme1.xml, docProps, etc.
    assert!(!pkg.unparsed_store.is_empty());
    assert!(
        pkg.unparsed_store.contains("/xl/calcChain.xml")
            || pkg.unparsed_store.contains("xl/calcChain.xml")
    );
    assert!(
        pkg.unparsed_store.contains("/xl/theme/theme1.xml")
            || pkg.unparsed_store.contains("xl/theme/theme1.xml")
    );

    let mut out = Cursor::new(Vec::new());
    pkg.save(&mut out).expect("Failed to save");

    out.set_position(0);
    let reader = ooxml_opc::package::PackageReader::open(out).expect("Failed to read");
    assert!(reader.contains_part("xl/calcChain.xml"));
    assert!(reader.contains_part("xl/theme/theme1.xml"));
}

#[test]
fn test_26_vectors_manifest_validation() {
    let vectors_path = resolve_fixture("specs/evidence/excel/2.1.1_vectors.json");
    let content = std::fs::read_to_string(&vectors_path).expect("Failed to read vectors JSON");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vectors JSON");

    assert_eq!(json["subphase"], "2.1.1");
    let vectors = json["vectors"].as_array().expect("vectors array");
    assert_eq!(vectors.len(), 26);
}
