//! Typed record value representation.

use excel_engine::CellValue;
use serde::{Deserialize, Serialize};

/// High-level typed data value from external data sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataValue {
    Null,
    Boolean(bool),
    Int64(i64),
    Float64(f64),
    Date(i32),
    DateTime(i64),
    Text(String),
    Binary(Vec<u8>),
}

impl DataValue {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float64(v) => Some(*v),
            Self::Int64(v) => Some(*v as f64),
            Self::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            Self::Date(d) => Some(*d as f64),
            Self::DateTime(dt) => Some(*dt as f64),
            _ => None,
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int64(v) => Some(*v),
            Self::Float64(v) => {
                if v.fract() == 0.0 && *v >= i64::MIN as f64 && *v <= i64::MAX as f64 {
                    Some(*v as i64)
                } else {
                    None
                }
            }
            Self::Boolean(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Int64(v) => Some(*v != 0),
            Self::Float64(v) => Some(*v != 0.0),
            Self::Text(s) => match s.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "t" | "y" => Some(true),
                "false" | "0" | "no" | "f" | "n" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Converts this `DataValue` into an `excel_engine::CellValue`.
    pub fn to_cell_value(&self) -> CellValue {
        match self {
            Self::Null => CellValue::Empty,
            Self::Boolean(b) => CellValue::Boolean(*b),
            Self::Int64(v) => CellValue::Number(*v as f64),
            Self::Float64(v) => CellValue::Number(*v),
            // In Excel, dates are stored as serial day numbers
            Self::Date(d) => CellValue::Number(*d as f64),
            Self::DateTime(dt) => CellValue::Number(*dt as f64 / 86400000.0),
            Self::Text(s) => CellValue::InlineString(s.clone()),
            Self::Binary(b) => CellValue::InlineString(format!("<BLOB {} bytes>", b.len())),
        }
    }

    /// Formats as string for text output.
    pub fn to_string_lossy(&self) -> String {
        match self {
            Self::Null => "".to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Int64(v) => v.to_string(),
            Self::Float64(v) => {
                if v.fract() == 0.0 && v.abs() < 1e15 {
                    format!("{:.0}", v)
                } else {
                    v.to_string()
                }
            }
            Self::Date(d) => format!("Day({d})"),
            Self::DateTime(dt) => format!("Timestamp({dt})"),
            Self::Text(s) => s.clone(),
            Self::Binary(b) => format!("<BLOB {} bytes>", b.len()),
        }
    }
}
