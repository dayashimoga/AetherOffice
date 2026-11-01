//! OpenOfficeRS Excel Engine - Cell Data Validation Engine.
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements ECMA-376 Part 1 §18.3.1.32 Data Validation models, list dropdown extraction,
//! numeric bounds checking, alert styles, and XML serialization.

use crate::grid::{cell_to_coords, parse_range, CellValue, CompressedSparsePageGrid};
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

pub fn cell_value_to_string(val: &CellValue) -> String {
    match val {
        CellValue::InlineString(s) => s.trim().to_string(),
        CellValue::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                n.to_string()
            }
        }
        CellValue::Boolean(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        CellValue::Error(e) => format!("{e}"),
        _ => String::new(),
    }
}

/// Type of data validation restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationType {
    #[default]
    Whole,
    Decimal,
    List,
    Date,
    Time,
    TextLength,
    Custom,
}

impl ValidationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Whole => "whole",
            Self::Decimal => "decimal",
            Self::List => "list",
            Self::Date => "date",
            Self::Time => "time",
            Self::TextLength => "textLength",
            Self::Custom => "custom",
        }
    }

    pub fn from_str_attr(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "whole" => Self::Whole,
            "decimal" => Self::Decimal,
            "list" => Self::List,
            "date" => Self::Date,
            "time" => Self::Time,
            "textlength" => Self::TextLength,
            "custom" => Self::Custom,
            _ => Self::Whole,
        }
    }
}

/// Comparison operator used in numeric, date, and text length validations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationOperator {
    #[default]
    Between,
    NotBetween,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
}

impl ValidationOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Between => "between",
            Self::NotBetween => "notBetween",
            Self::Equal => "equal",
            Self::NotEqual => "notEqual",
            Self::GreaterThan => "greaterThan",
            Self::LessThan => "lessThan",
            Self::GreaterEqual => "greaterThanOrEqual",
            Self::LessEqual => "lessThanOrEqual",
        }
    }

    pub fn from_str_attr(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "between" => Self::Between,
            "notbetween" => Self::NotBetween,
            "equal" => Self::Equal,
            "notequal" => Self::NotEqual,
            "greaterthan" => Self::GreaterThan,
            "lessthan" => Self::LessThan,
            "greaterthanorequal" => Self::GreaterEqual,
            "lessthanorequal" => Self::LessEqual,
            _ => Self::Between,
        }
    }
}

/// Error alert severity style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationErrorStyle {
    #[default]
    Stop,
    Warning,
    Information,
}

impl ValidationErrorStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Warning => "warning",
            Self::Information => "information",
        }
    }

    pub fn from_str_attr(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "stop" => Self::Stop,
            "warning" => Self::Warning,
            "information" => Self::Information,
            _ => Self::Stop,
        }
    }
}

/// A Data Validation definition according to ECMA-376 §18.3.1.32.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataValidation {
    pub validation_type: ValidationType,
    pub operator: ValidationOperator,
    pub allow_blank: bool,
    pub show_dropdown: bool,
    pub show_input_message: bool,
    pub prompt_title: Option<String>,
    pub prompt_message: Option<String>,
    pub show_error_alert: bool,
    pub error_style: ValidationErrorStyle,
    pub error_title: Option<String>,
    pub error_message: Option<String>,
    pub formula1: String,
    pub formula2: Option<String>,
    pub sqref: String,
}

impl Default for DataValidation {
    fn default() -> Self {
        Self {
            validation_type: ValidationType::Whole,
            operator: ValidationOperator::Between,
            allow_blank: true,
            show_dropdown: true,
            show_input_message: false,
            prompt_title: None,
            prompt_message: None,
            show_error_alert: true,
            error_style: ValidationErrorStyle::Stop,
            error_title: None,
            error_message: None,
            formula1: String::new(),
            formula2: None,
            sqref: String::new(),
        }
    }
}

/// Error payload returned on failed validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub style: ValidationErrorStyle,
    pub title: String,
    pub message: String,
}

/// Result of evaluating a cell value against a Data Validation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error: Option<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            error: None,
        }
    }

    pub fn invalid(style: ValidationErrorStyle, title: String, message: String) -> Self {
        Self {
            is_valid: false,
            error: Some(ValidationError {
                style,
                title,
                message,
            }),
        }
    }
}

/// High-performance data validation engine.
pub struct DataValidationEngine;

impl DataValidationEngine {
    /// Determines whether a given cell coordinate (row, col) is covered by the rule's `sqref`.
    pub fn applies_to_cell(rule: &DataValidation, row: u32, col: u32) -> bool {
        for part in rule.sqref.split_whitespace() {
            if part.contains(':') {
                if let Ok(((r1, c1), (r2, c2))) = parse_range(part) {
                    let r_min = r1.min(r2);
                    let r_max = r1.max(r2);
                    let c_min = c1.min(c2);
                    let c_max = c1.max(c2);
                    if row >= r_min && row <= r_max && col >= c_min && col <= c_max {
                        return true;
                    }
                }
            } else if let Ok((r, c)) = cell_to_coords(part) {
                if row == r && col == c {
                    return true;
                }
            }
        }
        false
    }

    /// Extracts available list dropdown options from formula1.
    /// Handles both literal comma-separated strings (e.g. `"Apples,Oranges,Bananas"`)
    /// and range references (e.g. `"$A$1:$A$10"`).
    pub fn get_list_items(
        rule: &DataValidation,
        grid: Option<&CompressedSparsePageGrid>,
    ) -> Vec<String> {
        if rule.validation_type != ValidationType::List {
            return Vec::new();
        }

        let f1 = rule.formula1.trim();

        // 1. Literal comma-separated list enclosed in quotes
        if f1.starts_with('"') && f1.ends_with('"') && f1.len() >= 2 {
            let inner = &f1[1..f1.len() - 1];
            return inner.split(',').map(|s| s.trim().to_string()).collect();
        }

        // 2. Unquoted comma-separated list
        if f1.contains(',') && !f1.contains(':') && !f1.starts_with('=') {
            return f1.split(',').map(|s| s.trim().to_string()).collect();
        }

        // 3. Grid cell or range reference
        let clean_ref = f1.trim_start_matches('=').replace('$', "");
        if let Some(g) = grid {
            if clean_ref.contains(':') {
                if let Ok(((r1, c1), (r2, c2))) = parse_range(&clean_ref) {
                    let mut items = Vec::new();
                    let r_min = r1.min(r2);
                    let r_max = r1.max(r2);
                    let c_min = c1.min(c2);
                    let c_max = c1.max(c2);
                    for r in r_min..=r_max {
                        for c in c_min..=c_max {
                            let val = g.get_value(r, c);
                            if !val.is_empty() {
                                items.push(cell_value_to_string(&val));
                            }
                        }
                    }
                    return items;
                }
            } else if let Ok((r, c)) = cell_to_coords(&clean_ref) {
                let val = g.get_value(r, c);
                if !val.is_empty() {
                    return vec![cell_value_to_string(&val)];
                }
            }
        }

        Vec::new()
    }

    /// Validates a cell value against the specified Data Validation rule.
    pub fn validate_cell(
        val: &CellValue,
        rule: &DataValidation,
        grid: Option<&CompressedSparsePageGrid>,
    ) -> ValidationResult {
        // Handle empty cell
        if val.is_empty() {
            if rule.allow_blank {
                return ValidationResult::valid();
            } else {
                return ValidationResult::invalid(
                    rule.error_style,
                    rule.error_title
                        .clone()
                        .unwrap_or_else(|| "Validation Error".to_string()),
                    rule.error_message
                        .clone()
                        .unwrap_or_else(|| "This cell cannot be blank.".to_string()),
                );
            }
        }

        let is_valid = match rule.validation_type {
            ValidationType::List => {
                let items = Self::get_list_items(rule, grid);
                let cell_str = match val {
                    CellValue::InlineString(s) => s.trim().to_string(),
                    CellValue::Number(n) => {
                        if n.fract() == 0.0 {
                            format!("{}", *n as i64)
                        } else {
                            n.to_string()
                        }
                    }
                    CellValue::Boolean(b) => {
                        if *b {
                            "TRUE".to_string()
                        } else {
                            "FALSE".to_string()
                        }
                    }
                    other => cell_value_to_string(other),
                };
                items
                    .iter()
                    .any(|item| item.eq_ignore_ascii_case(&cell_str))
            }
            ValidationType::Whole => {
                if let Some(n) = val.as_f64() {
                    if n.fract() != 0.0 {
                        false
                    } else {
                        Self::check_numeric_bounds(
                            n,
                            rule.operator,
                            &rule.formula1,
                            rule.formula2.as_deref(),
                        )
                    }
                } else {
                    false
                }
            }
            ValidationType::Decimal => {
                if let Some(n) = val.as_f64() {
                    Self::check_numeric_bounds(
                        n,
                        rule.operator,
                        &rule.formula1,
                        rule.formula2.as_deref(),
                    )
                } else {
                    false
                }
            }
            ValidationType::TextLength => {
                let len = match val {
                    CellValue::InlineString(s) => s.chars().count() as f64,
                    CellValue::Number(n) => n.to_string().chars().count() as f64,
                    CellValue::Boolean(b) => {
                        if *b {
                            4.0
                        } else {
                            5.0
                        }
                    }
                    _ => 0.0,
                };
                Self::check_numeric_bounds(
                    len,
                    rule.operator,
                    &rule.formula1,
                    rule.formula2.as_deref(),
                )
            }
            ValidationType::Date | ValidationType::Time => {
                if let Some(n) = val.as_f64() {
                    Self::check_numeric_bounds(
                        n,
                        rule.operator,
                        &rule.formula1,
                        rule.formula2.as_deref(),
                    )
                } else {
                    false
                }
            }
            ValidationType::Custom => {
                // In custom validation, formula evaluation resulting in truthy passes
                match val {
                    CellValue::Boolean(b) => *b,
                    CellValue::Number(n) => *n != 0.0,
                    CellValue::Error(_) => false,
                    _ => true,
                }
            }
        };

        if is_valid {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid(
                rule.error_style,
                rule.error_title
                    .clone()
                    .unwrap_or_else(|| "Invalid Value".to_string()),
                rule.error_message.clone().unwrap_or_else(|| {
                    "The value you entered is not valid for this cell.".to_string()
                }),
            )
        }
    }

    /// Evaluates numeric bounds against operator and formulas.
    fn check_numeric_bounds(
        val: f64,
        operator: ValidationOperator,
        f1_str: &str,
        f2_str: Option<&str>,
    ) -> bool {
        let f1 = f1_str.trim().parse::<f64>().unwrap_or(0.0);
        let f2 = f2_str
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0);

        match operator {
            ValidationOperator::Between => {
                let min = f1.min(f2);
                let max = f1.max(f2);
                val >= min && val <= max
            }
            ValidationOperator::NotBetween => {
                let min = f1.min(f2);
                let max = f1.max(f2);
                val < min || val > max
            }
            ValidationOperator::Equal => (val - f1).abs() < 1e-9,
            ValidationOperator::NotEqual => (val - f1).abs() >= 1e-9,
            ValidationOperator::GreaterThan => val > f1,
            ValidationOperator::LessThan => val < f1,
            ValidationOperator::GreaterEqual => val >= f1,
            ValidationOperator::LessEqual => val <= f1,
        }
    }
}

/// Serializes data validation rules into compliant ECMA-376 `<dataValidations>` XML.
pub fn data_validations_to_xml(rules: &[DataValidation]) -> String {
    if rules.is_empty() {
        return String::new();
    }

    let mut xml = format!("<dataValidations count=\"{}\">", rules.len());
    for rule in rules {
        xml.push_str("<dataValidation");
        xml.push_str(&format!(" type=\"{}\"", rule.validation_type.as_str()));
        if rule.operator != ValidationOperator::Between
            && rule.validation_type != ValidationType::List
            && rule.validation_type != ValidationType::Custom
        {
            xml.push_str(&format!(" operator=\"{}\"", rule.operator.as_str()));
        }
        if !rule.allow_blank {
            xml.push_str(" allowBlank=\"0\"");
        }
        if !rule.show_dropdown {
            xml.push_str(" showDropDown=\"1\""); // Note: ECMA-376 showDropDown=1 actually suppresses dropdown!
        }
        if rule.show_input_message {
            xml.push_str(" showInputMessage=\"1\"");
            if let Some(ref title) = rule.prompt_title {
                xml.push_str(&format!(
                    " promptTitle=\"{}\"",
                    quick_xml::escape::escape(title)
                ));
            }
            if let Some(ref msg) = rule.prompt_message {
                xml.push_str(&format!(" prompt=\"{}\"", quick_xml::escape::escape(msg)));
            }
        }
        if rule.show_error_alert {
            xml.push_str(" showErrorMessage=\"1\"");
            xml.push_str(&format!(" errorStyle=\"{}\"", rule.error_style.as_str()));
            if let Some(ref title) = rule.error_title {
                xml.push_str(&format!(
                    " errorTitle=\"{}\"",
                    quick_xml::escape::escape(title)
                ));
            }
            if let Some(ref msg) = rule.error_message {
                xml.push_str(&format!(" error=\"{}\"", quick_xml::escape::escape(msg)));
            }
        }
        xml.push_str(&format!(" sqref=\"{}\">", rule.sqref));

        if !rule.formula1.is_empty() {
            xml.push_str(&format!(
                "<formula1>{}</formula1>",
                quick_xml::escape::escape(&rule.formula1)
            ));
        }
        if let Some(ref f2) = rule.formula2 {
            if !f2.is_empty() {
                xml.push_str(&format!(
                    "<formula2>{}</formula2>",
                    quick_xml::escape::escape(f2)
                ));
            }
        }
        xml.push_str("</dataValidation>");
    }
    xml.push_str("</dataValidations>");
    xml
}

/// Parses `<dataValidations>` XML into `Vec<DataValidation>`.
pub fn parse_data_validations_xml(xml: &str) -> Result<Vec<DataValidation>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut rules = Vec::new();
    let mut current_rule: Option<DataValidation> = None;
    let mut current_tag = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = tag_name.clone();

                if tag_name == "dataValidation" {
                    let mut rule = DataValidation::default();
                    for attr in e.attributes() {
                        let a = attr.map_err(EngineError::XmlAttr)?;
                        let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                        let val = a.unescape_value().unwrap_or_default().to_string();

                        match key.as_str() {
                            "type" => rule.validation_type = ValidationType::from_str_attr(&val),
                            "operator" => rule.operator = ValidationOperator::from_str_attr(&val),
                            "allowBlank" => rule.allow_blank = val != "0",
                            "showDropDown" => rule.show_dropdown = val != "1",
                            "showInputMessage" => rule.show_input_message = val == "1",
                            "promptTitle" => rule.prompt_title = Some(val),
                            "prompt" => rule.prompt_message = Some(val),
                            "showErrorMessage" => rule.show_error_alert = val == "1",
                            "errorStyle" => {
                                rule.error_style = ValidationErrorStyle::from_str_attr(&val)
                            }
                            "errorTitle" => rule.error_title = Some(val),
                            "error" => rule.error_message = Some(val),
                            "sqref" => rule.sqref = val,
                            _ => {}
                        }
                    }
                    current_rule = Some(rule);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if let Some(ref mut rule) = current_rule {
                    if current_tag == "formula1" {
                        rule.formula1 = text;
                    } else if current_tag == "formula2" {
                        rule.formula2 = Some(text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"dataValidation" {
                    if let Some(rule) = current_rule.take() {
                        rules.push(rule);
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EngineError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(rules)
}
