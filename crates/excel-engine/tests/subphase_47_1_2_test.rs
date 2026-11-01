//! Subphase 47.1.2 Test Suite: Data Validation, Conditional Formatting & Sparklines
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).

use excel_engine::conditional_format::{
    ConditionalFormattingCollection, ConditionalFormattingEngine, ConditionalFormattingRule,
    ConditionalRuleType, ConditionalStyle, SparklineGroup, SparklineRenderer, SparklineType,
};
use excel_engine::grid::{CellValue, CompressedSparsePageGrid};
use excel_engine::validation::{
    data_validations_to_xml, parse_data_validations_xml, DataValidation, DataValidationEngine,
    ValidationErrorStyle, ValidationOperator, ValidationType,
};

// =========================================================================
// 1. DATA VALIDATION: WHOLE & DECIMAL NUMBERS
// =========================================================================

#[test]
fn test_01_validation_whole_number_bounds() {
    let rule = DataValidation {
        validation_type: ValidationType::Whole,
        operator: ValidationOperator::Between,
        allow_blank: false,
        formula1: "10".to_string(),
        formula2: Some("50".to_string()),
        sqref: "A1:A10".to_string(),
        ..Default::default()
    };

    // Valid integer between 10 and 50
    let res_valid = DataValidationEngine::validate_cell(&CellValue::Number(25.0), &rule, None);
    assert!(res_valid.is_valid);

    // Invalid integer (< 10)
    let res_low = DataValidationEngine::validate_cell(&CellValue::Number(5.0), &rule, None);
    assert!(!res_low.is_valid);
    assert_eq!(
        res_low.error.as_ref().map(|e| e.style),
        Some(ValidationErrorStyle::Stop)
    );

    // Invalid integer (> 50)
    let res_high = DataValidationEngine::validate_cell(&CellValue::Number(60.0), &rule, None);
    assert!(!res_high.is_valid);

    // Invalid non-integer decimal (Whole type rejects 25.5)
    let res_decimal = DataValidationEngine::validate_cell(&CellValue::Number(25.5), &rule, None);
    assert!(!res_decimal.is_valid);
}

#[test]
fn test_02_validation_decimal_numbers() {
    let rule = DataValidation {
        validation_type: ValidationType::Decimal,
        operator: ValidationOperator::GreaterThan,
        allow_blank: true,
        formula1: "3.14".to_string(),
        formula2: None,
        sqref: "B1:B10".to_string(),
        ..Default::default()
    };

    assert!(DataValidationEngine::validate_cell(&CellValue::Number(5.5), &rule, None).is_valid);
    assert!(!DataValidationEngine::validate_cell(&CellValue::Number(1.2), &rule, None).is_valid);
    // Allow blank
    assert!(DataValidationEngine::validate_cell(&CellValue::Empty, &rule, None).is_valid);
}

// =========================================================================
// 2. DATA VALIDATION: LIST DROPDOWNS & TEXT LENGTH
// =========================================================================

#[test]
fn test_03_validation_list_dropdown_literal() {
    let rule = DataValidation {
        validation_type: ValidationType::List,
        allow_blank: true,
        show_dropdown: true,
        formula1: "\"Small,Medium,Large\"".to_string(),
        sqref: "C1:C100".to_string(),
        ..Default::default()
    };

    let items = DataValidationEngine::get_list_items(&rule, None);
    assert_eq!(items, vec!["Small", "Medium", "Large"]);

    assert!(
        DataValidationEngine::validate_cell(
            &CellValue::InlineString("Medium".to_string()),
            &rule,
            None
        )
        .is_valid
    );

    // Case-insensitive match
    assert!(
        DataValidationEngine::validate_cell(
            &CellValue::InlineString("large".to_string()),
            &rule,
            None
        )
        .is_valid
    );

    // Invalid option
    assert!(
        !DataValidationEngine::validate_cell(
            &CellValue::InlineString("Extra-Large".to_string()),
            &rule,
            None
        )
        .is_valid
    );
}

#[test]
fn test_04_validation_list_dropdown_from_grid_range() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_inline_string(0, 0, "Platinum").unwrap();
    grid.set_inline_string(1, 0, "Gold").unwrap();
    grid.set_inline_string(2, 0, "Silver").unwrap();

    let rule = DataValidation {
        validation_type: ValidationType::List,
        allow_blank: false,
        show_dropdown: true,
        formula1: "$A$1:$A$3".to_string(),
        sqref: "D1:D10".to_string(),
        ..Default::default()
    };

    let items = DataValidationEngine::get_list_items(&rule, Some(&grid));
    assert_eq!(items, vec!["Platinum", "Gold", "Silver"]);

    assert!(
        DataValidationEngine::validate_cell(
            &CellValue::InlineString("Gold".to_string()),
            &rule,
            Some(&grid)
        )
        .is_valid
    );

    assert!(
        !DataValidationEngine::validate_cell(
            &CellValue::InlineString("Bronze".to_string()),
            &rule,
            Some(&grid)
        )
        .is_valid
    );
}

#[test]
fn test_05_validation_text_length() {
    let rule = DataValidation {
        validation_type: ValidationType::TextLength,
        operator: ValidationOperator::LessEqual,
        formula1: "5".to_string(),
        sqref: "E1:E10".to_string(),
        ..Default::default()
    };

    assert!(
        DataValidationEngine::validate_cell(
            &CellValue::InlineString("Hello".to_string()),
            &rule,
            None
        )
        .is_valid
    );

    assert!(
        !DataValidationEngine::validate_cell(
            &CellValue::InlineString("Hello World".to_string()),
            &rule,
            None
        )
        .is_valid
    );
}

#[test]
fn test_06_validation_xml_roundtrip() {
    let original = vec![
        DataValidation {
            validation_type: ValidationType::Whole,
            operator: ValidationOperator::Between,
            allow_blank: true,
            show_dropdown: true,
            show_input_message: true,
            prompt_title: Some("Range Prompt".to_string()),
            prompt_message: Some("Enter integer between 1 and 100".to_string()),
            show_error_alert: true,
            error_style: ValidationErrorStyle::Stop,
            error_title: Some("Out of Range".to_string()),
            error_message: Some("Integer must be between 1 and 100".to_string()),
            formula1: "1".to_string(),
            formula2: Some("100".to_string()),
            sqref: "A1:A50".to_string(),
        },
        DataValidation {
            validation_type: ValidationType::List,
            operator: ValidationOperator::Between,
            allow_blank: false,
            show_dropdown: true,
            show_input_message: false,
            prompt_title: None,
            prompt_message: None,
            show_error_alert: true,
            error_style: ValidationErrorStyle::Warning,
            error_title: Some("Invalid Category".to_string()),
            error_message: Some("Choose from valid options".to_string()),
            formula1: "\"Tech,Finance,Health\"".to_string(),
            formula2: None,
            sqref: "B1:B20".to_string(),
        },
    ];

    let xml = data_validations_to_xml(&original);
    assert!(xml.contains("<dataValidations count=\"2\">"));
    assert!(xml.contains("type=\"whole\""));
    assert!(xml.contains("type=\"list\""));
    assert!(xml.contains("formula1"));

    let parsed = parse_data_validations_xml(&xml).expect("parse data validations xml");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].validation_type, ValidationType::Whole);
    assert_eq!(parsed[0].formula1, "1");
    assert_eq!(parsed[0].formula2.as_deref(), Some("100"));
    assert_eq!(parsed[1].validation_type, ValidationType::List);
    assert_eq!(parsed[1].formula1, "\"Tech,Finance,Health\"");
}

// =========================================================================
// 3. CONDITIONAL FORMATTING: RULES, COLOR SCALES & DATA BARS
// =========================================================================

#[test]
fn test_07_cf_cell_is_and_style_merging() {
    let collection = ConditionalFormattingCollection {
        sqref: "A1:A10".to_string(),
        rules: vec![
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::CellIs {
                    operator: "greaterThan".to_string(),
                    formula1: "80".to_string(),
                    formula2: None,
                },
                priority: 1,
                stop_if_true: false,
                style: Some(ConditionalStyle {
                    font_color: Some("#006100".to_string()),
                    fill_color: Some("#C6EFCE".to_string()),
                    bold: Some(true),
                    ..Default::default()
                }),
            },
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::CellIs {
                    operator: "lessThan".to_string(),
                    formula1: "50".to_string(),
                    formula2: None,
                },
                priority: 2,
                stop_if_true: false,
                style: Some(ConditionalStyle {
                    font_color: Some("#9C0006".to_string()),
                    fill_color: Some("#FFC7CE".to_string()),
                    bold: Some(false),
                    ..Default::default()
                }),
            },
        ],
    };

    // Evaluates 95 -> Green highlight
    let res_high =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(95.0), &collection, &[]);
    assert_eq!(
        res_high
            .style
            .as_ref()
            .and_then(|s| s.fill_color.as_deref()),
        Some("#C6EFCE")
    );
    assert_eq!(res_high.style.as_ref().and_then(|s| s.bold), Some(true));

    // Evaluates 30 -> Red highlight
    let res_low =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(30.0), &collection, &[]);
    assert_eq!(
        res_low.style.as_ref().and_then(|s| s.fill_color.as_deref()),
        Some("#FFC7CE")
    );

    // Evaluates 65 -> No rule matches
    let res_mid =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(65.0), &collection, &[]);
    assert!(res_mid.style.is_none());
}

#[test]
fn test_08_cf_color_scale_interpolation() {
    let collection = ConditionalFormattingCollection {
        sqref: "B1:B10".to_string(),
        rules: vec![ConditionalFormattingRule {
            rule_type: ConditionalRuleType::ColorScale {
                colors: vec![
                    "#FF0000".to_string(),
                    "#FFFF00".to_string(),
                    "#00FF00".to_string(),
                ],
                values: vec![0.0, 50.0, 100.0],
            },
            priority: 1,
            stop_if_true: true,
            style: None,
        }],
    };

    // Min value -> #FF0000 (Pure Red)
    let res_min =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(0.0), &collection, &[]);
    assert_eq!(res_min.color_scale_rgb.as_deref(), Some("#FF0000"));

    // Mid value -> #FFFF00 (Pure Yellow)
    let res_mid =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(50.0), &collection, &[]);
    assert_eq!(res_mid.color_scale_rgb.as_deref(), Some("#FFFF00"));

    // Max value -> #00FF00 (Pure Green)
    let res_max =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(100.0), &collection, &[]);
    assert_eq!(res_max.color_scale_rgb.as_deref(), Some("#00FF00"));

    // 25.0 value -> Interpolar between Red (#FF0000) and Yellow (#FFFF00)
    let res_quarter =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(25.0), &collection, &[]);
    assert_eq!(res_quarter.color_scale_rgb.as_deref(), Some("#FF8000"));
}

#[test]
fn test_09_cf_data_bar_and_icon_set() {
    let data = vec![10.0, 20.0, 50.0, 100.0];

    let collection = ConditionalFormattingCollection {
        sqref: "C1:C10".to_string(),
        rules: vec![
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::DataBar {
                    color: "#638EC6".to_string(),
                    show_value: true,
                    min_val: Some(0.0),
                    max_val: Some(100.0),
                },
                priority: 1,
                stop_if_true: false,
                style: None,
            },
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::IconSet {
                    icon_set_name: "3TrafficLights1".to_string(),
                    thresholds: vec![33.0, 67.0],
                },
                priority: 2,
                stop_if_true: false,
                style: None,
            },
        ],
    };

    let res =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(50.0), &collection, &data);
    assert_eq!(res.data_bar_ratio, Some(0.5));
    assert_eq!(res.data_bar_color.as_deref(), Some("#638EC6"));
    // Icon threshold: 50.0 is >= 33.0 but < 67.0 -> index 1 (yellow light)
    assert_eq!(res.icon_index, Some(1));
    assert_eq!(res.icon_set_name.as_deref(), Some("3TrafficLights1"));
}

#[test]
fn test_10_cf_top10_and_duplicates() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 50.0];

    // Top 2 items rule
    let collection = ConditionalFormattingCollection {
        sqref: "D1:D10".to_string(),
        rules: vec![
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::Top10 {
                    percent: false,
                    bottom: false,
                    rank: 2,
                },
                priority: 1,
                stop_if_true: false,
                style: Some(ConditionalStyle {
                    bold: Some(true),
                    ..Default::default()
                }),
            },
            ConditionalFormattingRule {
                rule_type: ConditionalRuleType::DuplicateValues,
                priority: 2,
                stop_if_true: false,
                style: Some(ConditionalStyle {
                    fill_color: Some("#FFFF99".to_string()),
                    ..Default::default()
                }),
            },
        ],
    };

    // 50.0 is both in top 2 and a duplicate!
    let res50 =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(50.0), &collection, &data);
    assert_eq!(res50.style.as_ref().and_then(|s| s.bold), Some(true));
    assert_eq!(
        res50.style.as_ref().and_then(|s| s.fill_color.as_deref()),
        Some("#FFFF99")
    );

    // 10.0 is not top 2 and not duplicate
    let res10 =
        ConditionalFormattingEngine::evaluate_cell(&CellValue::Number(10.0), &collection, &data);
    assert!(res10.style.is_none());
}

// =========================================================================
// 4. SPARKLINES ENGINE: LINE, COLUMN, WIN/LOSS
// =========================================================================

#[test]
fn test_11_sparkline_line_svg_generation() {
    let group = SparklineGroup {
        sparkline_type: SparklineType::Line,
        series_color: "#2E75B6".to_string(),
        show_markers: true,
        show_high: true,
        high_color: Some("#00B050".to_string()),
        show_low: true,
        low_color: Some("#C00000".to_string()),
        ..Default::default()
    };

    let data = vec![10.0, 50.0, 20.0, 80.0, 30.0];
    let svg = SparklineRenderer::render_svg(&group, &data, 120.0, 28.0);

    assert!(svg.starts_with("<svg width=\"120\" height=\"28\""));
    assert!(svg.contains("<polyline fill=\"none\" stroke=\"#2E75B6\""));
    // High point circle in green
    assert!(svg.contains("fill=\"#00B050\""));
    // Low point circle in red
    assert!(svg.contains("fill=\"#C00000\""));
}

#[test]
fn test_12_sparkline_column_and_winloss_svg() {
    let group_col = SparklineGroup {
        sparkline_type: SparklineType::Column,
        series_color: "#4472C4".to_string(),
        show_negative: true,
        negative_color: Some("#ED7D31".to_string()),
        ..Default::default()
    };

    let data_col = vec![15.0, -10.0, 25.0, -5.0];
    let svg_col = SparklineRenderer::render_svg(&group_col, &data_col, 100.0, 24.0);
    assert!(svg_col.contains("<rect"));
    // Negative bar color
    assert!(svg_col.contains("fill=\"#ED7D31\""));

    let group_winloss = SparklineGroup {
        sparkline_type: SparklineType::Stacked,
        series_color: "#70AD47".to_string(),
        negative_color: Some("#FF0000".to_string()),
        ..Default::default()
    };

    let data_winloss = vec![1.0, -1.0, 1.0, 1.0, -1.0];
    let svg_winloss = SparklineRenderer::render_svg(&group_winloss, &data_winloss, 100.0, 20.0);
    assert!(svg_winloss.contains("<rect"));
    assert!(svg_winloss.contains("fill=\"#70AD47\""));
    assert!(svg_winloss.contains("fill=\"#FF0000\""));
}
