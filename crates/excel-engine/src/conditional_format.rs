//! OpenOfficeRS Excel Engine - Conditional Formatting & Sparklines Engine.
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements ECMA-376 Part 1 §18.3.1.18 Conditional Formatting rules (cellIs, colorScale,
//! dataBar, iconSet, top10, unique/duplicate) and OpenXML Micro-Chart Sparklines.

use crate::grid::{cell_to_coords, parse_range, CellValue};
use serde::{Deserialize, Serialize};

/// Type of conditional formatting rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionalRuleType {
    CellIs {
        operator: String,
        formula1: String,
        formula2: Option<String>,
    },
    Expression {
        formula: String,
    },
    ColorScale {
        colors: Vec<String>,
        values: Vec<f64>,
    },
    DataBar {
        color: String,
        show_value: bool,
        min_val: Option<f64>,
        max_val: Option<f64>,
    },
    IconSet {
        icon_set_name: String,
        thresholds: Vec<f64>,
    },
    Top10 {
        percent: bool,
        bottom: bool,
        rank: u32,
    },
    DuplicateValues,
    UniqueValues,
    ContainsText {
        text: String,
    },
}

/// Visual formatting style applied by a conditional formatting rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConditionalStyle {
    pub font_color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub fill_color: Option<String>,
    pub border_color: Option<String>,
}

/// Individual conditional formatting rule definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalFormattingRule {
    pub rule_type: ConditionalRuleType,
    pub priority: i32,
    pub stop_if_true: bool,
    pub style: Option<ConditionalStyle>,
}

/// Group of conditional formatting rules bounded to target cell ranges (`sqref`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConditionalFormattingCollection {
    pub sqref: String,
    pub rules: Vec<ConditionalFormattingRule>,
}

/// Evaluation result for a specific cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EvaluatedConditionalFormat {
    pub style: Option<ConditionalStyle>,
    pub data_bar_ratio: Option<f64>,
    pub data_bar_color: Option<String>,
    pub color_scale_rgb: Option<String>,
    pub icon_index: Option<usize>,
    pub icon_set_name: Option<String>,
}

/// Engine evaluating conditional formatting rules across grid cells.
pub struct ConditionalFormattingEngine;

impl ConditionalFormattingEngine {
    /// Checks whether `(row, col)` is covered by `sqref`.
    pub fn cell_matches_sqref(sqref: &str, row: u32, col: u32) -> bool {
        for part in sqref.split_whitespace() {
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

    /// Evaluates collection rules against a cell value and optional range numbers context.
    pub fn evaluate_cell(
        val: &CellValue,
        collection: &ConditionalFormattingCollection,
        range_numbers: &[f64],
    ) -> EvaluatedConditionalFormat {
        let mut sorted_rules = collection.rules.clone();
        sorted_rules.sort_by_key(|r| r.priority);

        let mut result = EvaluatedConditionalFormat::default();

        for rule in &sorted_rules {
            let matched = match &rule.rule_type {
                ConditionalRuleType::CellIs {
                    operator,
                    formula1,
                    formula2,
                } => Self::eval_cell_is(val, operator, formula1, formula2.as_deref()),
                ConditionalRuleType::ContainsText { text } => match val {
                    CellValue::InlineString(s) => {
                        s.to_ascii_uppercase().contains(&text.to_ascii_uppercase())
                    }
                    _ => false,
                },
                ConditionalRuleType::ColorScale { colors, values } => {
                    if let Some(n) = val.as_f64() {
                        let interpolated = Self::interpolate_color_scale(n, values, colors);
                        result.color_scale_rgb = Some(interpolated);
                        true
                    } else {
                        false
                    }
                }
                ConditionalRuleType::DataBar {
                    color,
                    min_val,
                    max_val,
                    ..
                } => {
                    if let Some(n) = val.as_f64() {
                        let min = min_val.unwrap_or_else(|| {
                            range_numbers
                                .iter()
                                .copied()
                                .fold(f64::INFINITY, f64::min)
                                .min(0.0)
                        });
                        let max = max_val.unwrap_or_else(|| {
                            range_numbers
                                .iter()
                                .copied()
                                .fold(f64::NEG_INFINITY, f64::max)
                                .max(1.0)
                        });
                        let ratio = if max > min {
                            ((n - min) / (max - min)).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        result.data_bar_ratio = Some(ratio);
                        result.data_bar_color = Some(color.clone());
                        true
                    } else {
                        false
                    }
                }
                ConditionalRuleType::IconSet {
                    icon_set_name,
                    thresholds,
                } => {
                    if let Some(n) = val.as_f64() {
                        let mut idx = 0;
                        for &th in thresholds {
                            if n >= th {
                                idx += 1;
                            }
                        }
                        result.icon_index = Some(idx);
                        result.icon_set_name = Some(icon_set_name.clone());
                        true
                    } else {
                        false
                    }
                }
                ConditionalRuleType::Top10 {
                    percent,
                    bottom,
                    rank,
                } => {
                    if let Some(n) = val.as_f64() {
                        Self::eval_top10(n, range_numbers, *percent, *bottom, *rank)
                    } else {
                        false
                    }
                }
                ConditionalRuleType::DuplicateValues => {
                    if let Some(n) = val.as_f64() {
                        range_numbers
                            .iter()
                            .filter(|&&x| (x - n).abs() < 1e-9)
                            .count()
                            > 1
                    } else {
                        false
                    }
                }
                ConditionalRuleType::UniqueValues => {
                    if let Some(n) = val.as_f64() {
                        range_numbers
                            .iter()
                            .filter(|&&x| (x - n).abs() < 1e-9)
                            .count()
                            == 1
                    } else {
                        false
                    }
                }
                ConditionalRuleType::Expression { .. } => {
                    // Custom expression truthiness
                    match val {
                        CellValue::Boolean(b) => *b,
                        CellValue::Number(n) => *n != 0.0,
                        _ => false,
                    }
                }
            };

            if matched {
                if let Some(ref st) = rule.style {
                    Self::merge_style(&mut result.style, st);
                }
                if rule.stop_if_true {
                    break;
                }
            }
        }

        result
    }

    fn eval_cell_is(val: &CellValue, op: &str, f1_str: &str, f2_str: Option<&str>) -> bool {
        let f1 = f1_str.trim().parse::<f64>().unwrap_or(0.0);
        let f2 = f2_str
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0);

        if let Some(n) = val.as_f64() {
            match op.to_ascii_lowercase().as_str() {
                "equal" => (n - f1).abs() < 1e-9,
                "notequal" => (n - f1).abs() >= 1e-9,
                "greaterthan" => n > f1,
                "lessthan" => n < f1,
                "greaterthanorequal" => n >= f1,
                "lessthanorequal" => n <= f1,
                "between" => {
                    let min = f1.min(f2);
                    let max = f1.max(f2);
                    n >= min && n <= max
                }
                "notbetween" => {
                    let min = f1.min(f2);
                    let max = f1.max(f2);
                    n < min || n > max
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn eval_top10(val: f64, range: &[f64], percent: bool, bottom: bool, rank: u32) -> bool {
        if range.is_empty() {
            return false;
        }
        let mut sorted = range.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let cutoff_count = if percent {
            ((range.len() as f64 * (rank as f64 / 100.0)).ceil() as usize).max(1)
        } else {
            (rank as usize).min(range.len())
        };

        if bottom {
            let threshold = sorted[(cutoff_count - 1).min(sorted.len() - 1)];
            val <= threshold
        } else {
            let threshold = sorted[sorted.len().saturating_sub(cutoff_count)];
            val >= threshold
        }
    }

    /// Merges rule style into accumulator.
    fn merge_style(acc: &mut Option<ConditionalStyle>, next: &ConditionalStyle) {
        if acc.is_none() {
            *acc = Some(next.clone());
            return;
        }
        if let Some(ref mut cur) = acc {
            if next.font_color.is_some() {
                cur.font_color = next.font_color.clone();
            }
            if next.bold.is_some() {
                cur.bold = next.bold;
            }
            if next.italic.is_some() {
                cur.italic = next.italic;
            }
            if next.fill_color.is_some() {
                cur.fill_color = next.fill_color.clone();
            }
            if next.border_color.is_some() {
                cur.border_color = next.border_color.clone();
            }
        }
    }

    /// Linear interpolation for 2-color and 3-color scales.
    pub fn interpolate_color_scale(val: f64, values: &[f64], hex_colors: &[String]) -> String {
        if hex_colors.is_empty() {
            return "#FFFFFF".to_string();
        }
        if hex_colors.len() == 1 || values.len() < 2 {
            return hex_colors[0].clone();
        }

        // Clamp val between min and max
        let min_val = values[0];
        let max_val = *values.last().unwrap_or(&1.0);
        let clamped = val.clamp(min_val, max_val);

        if hex_colors.len() == 2 || values.len() == 2 {
            let t = if max_val > min_val {
                (clamped - min_val) / (max_val - min_val)
            } else {
                0.0
            };
            return Self::lerp_hex(&hex_colors[0], &hex_colors[1], t);
        }

        // 3-color scale
        let mid_val = values[1];
        if clamped <= mid_val {
            let t = if mid_val > min_val {
                (clamped - min_val) / (mid_val - min_val)
            } else {
                0.0
            };
            Self::lerp_hex(&hex_colors[0], &hex_colors[1], t)
        } else {
            let t = if max_val > mid_val {
                (clamped - mid_val) / (max_val - mid_val)
            } else {
                0.0
            };
            Self::lerp_hex(&hex_colors[1], &hex_colors[2], t)
        }
    }

    fn lerp_hex(c1: &str, c2: &str, t: f64) -> String {
        let (r1, g1, b1) = Self::parse_hex_rgb(c1);
        let (r2, g2, b2) = Self::parse_hex_rgb(c2);

        let r = ((1.0 - t) * r1 as f64 + t * r2 as f64).round() as u8;
        let g = ((1.0 - t) * g1 as f64 + t * g2 as f64).round() as u8;
        let b = ((1.0 - t) * b1 as f64 + t * b2 as f64).round() as u8;

        format!("#{r:02X}{g:02X}{b:02X}")
    }

    fn parse_hex_rgb(hex: &str) -> (u8, u8, u8) {
        let clean = hex.trim_start_matches('#');
        if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255);
            (r, g, b)
        } else {
            (255, 255, 255)
        }
    }
}

// =========================================================================
// SPARKLINES ENGINE
// =========================================================================

/// Type of sparkline micro-chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SparklineType {
    #[default]
    Line,
    Column,
    Stacked, // Win/Loss
}

impl SparklineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Column => "column",
            Self::Stacked => "stacked",
        }
    }
}

/// An individual sparkline binding a data range to a target cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sparkline {
    pub data_range: String,
    pub target_cell: String,
}

/// Sparkline group definition according to OpenXML Extension List schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparklineGroup {
    pub sparkline_type: SparklineType,
    pub series_color: String,
    pub high_color: Option<String>,
    pub low_color: Option<String>,
    pub first_color: Option<String>,
    pub last_color: Option<String>,
    pub negative_color: Option<String>,
    pub show_markers: bool,
    pub show_high: bool,
    pub show_low: bool,
    pub show_first: bool,
    pub show_last: bool,
    pub show_negative: bool,
    pub sparklines: Vec<Sparkline>,
}

impl Default for SparklineGroup {
    fn default() -> Self {
        Self {
            sparkline_type: SparklineType::Line,
            series_color: "#2E75B6".to_string(),
            high_color: Some("#385623".to_string()),
            low_color: Some("#C00000".to_string()),
            first_color: None,
            last_color: None,
            negative_color: Some("#C00000".to_string()),
            show_markers: true,
            show_high: true,
            show_low: true,
            show_first: false,
            show_last: false,
            show_negative: true,
            sparklines: Vec::new(),
        }
    }
}

/// Vector renderer for high-DPI sparkline micro-charts in cell viewports.
pub struct SparklineRenderer;

impl SparklineRenderer {
    /// Generates lightweight standalone SVG string for embedding directly into grid canvas.
    pub fn render_svg(group: &SparklineGroup, data: &[f64], width: f64, height: f64) -> String {
        if data.is_empty() {
            return format!("<svg width=\"{width}\" height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\"></svg>");
        }

        let padding = 3.0;
        let w = (width - padding * 2.0).max(1.0);
        let h = (height - padding * 2.0).max(1.0);

        let min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span = (max - min).max(1e-6);

        let n = data.len();

        match group.sparkline_type {
            SparklineType::Line => {
                let mut points = Vec::new();
                for (i, &val) in data.iter().enumerate() {
                    let x = if n > 1 {
                        padding + (i as f64 / (n - 1) as f64) * w
                    } else {
                        padding + w / 2.0
                    };
                    let y = padding + h - ((val - min) / span) * h;
                    points.push(format!("{x:.1},{y:.1}"));
                }
                let pts_str = points.join(" ");
                let mut svg = format!(
                    "<svg width=\"{width}\" height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\"><polyline fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\" points=\"{pts_str}\" />",
                    group.series_color
                );

                if group.show_markers {
                    for (i, &val) in data.iter().enumerate() {
                        let x = if n > 1 {
                            padding + (i as f64 / (n - 1) as f64) * w
                        } else {
                            padding + w / 2.0
                        };
                        let y = padding + h - ((val - min) / span) * h;
                        let mut fill = &group.series_color;
                        if group.show_high && (val - max).abs() < 1e-9 {
                            if let Some(ref c) = group.high_color {
                                fill = c;
                            }
                        } else if group.show_low && (val - min).abs() < 1e-9 {
                            if let Some(ref c) = group.low_color {
                                fill = c;
                            }
                        } else if group.show_negative && val < 0.0 {
                            if let Some(ref c) = group.negative_color {
                                fill = c;
                            }
                        }
                        svg.push_str(&format!(
                            "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"2.0\" fill=\"{fill}\" />"
                        ));
                    }
                }
                svg.push_str("</svg>");
                svg
            }
            SparklineType::Column => {
                let mut svg = format!("<svg width=\"{width}\" height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\">");
                let bar_width = (w / n as f64 * 0.75).max(1.0);
                let gap = (w / n as f64 * 0.25).max(0.5);

                for (i, &val) in data.iter().enumerate() {
                    let x = padding + (i as f64) * (bar_width + gap);
                    let bar_h = ((val.abs() - 0.0) / span * h).max(2.0);
                    let y = if val >= 0.0 {
                        padding + h - bar_h
                    } else {
                        padding + h
                    };

                    let mut fill = &group.series_color;
                    if group.show_negative && val < 0.0 {
                        if let Some(ref c) = group.negative_color {
                            fill = c;
                        }
                    } else if group.show_high && (val - max).abs() < 1e-9 {
                        if let Some(ref c) = group.high_color {
                            fill = c;
                        }
                    } else if group.show_low && (val - min).abs() < 1e-9 {
                        if let Some(ref c) = group.low_color {
                            fill = c;
                        }
                    }

                    svg.push_str(&format!(
                        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bar_width:.1}\" height=\"{bar_h:.1}\" fill=\"{fill}\" />"
                    ));
                }
                svg.push_str("</svg>");
                svg
            }
            SparklineType::Stacked => {
                // Win/Loss: positive = top bar, negative = bottom bar, zero = neutral line
                let mut svg = format!("<svg width=\"{width}\" height=\"{height}\" xmlns=\"http://www.w3.org/2000/svg\">");
                let bar_width = (w / n as f64 * 0.75).max(1.0);
                let gap = (w / n as f64 * 0.25).max(0.5);
                let mid_y = padding + h / 2.0;

                for (i, &val) in data.iter().enumerate() {
                    let x = padding + (i as f64) * (bar_width + gap);
                    let (y, bar_h, fill) = if val > 0.0 {
                        (padding, h / 2.0 - 1.0, &group.series_color)
                    } else if val < 0.0 {
                        let c = group.negative_color.as_ref().unwrap_or(&group.series_color);
                        (mid_y + 1.0, h / 2.0 - 1.0, c)
                    } else {
                        (mid_y - 0.5, 1.0, &group.series_color)
                    };

                    svg.push_str(&format!(
                        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bar_width:.1}\" height=\"{bar_h:.1}\" fill=\"{fill}\" />"
                    ));
                }
                svg.push_str("</svg>");
                svg
            }
        }
    }
}
