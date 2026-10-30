//! Schema definition, data types, and automatic type inference engine.

use crate::error::{ConnectorError, ConnectorResult};
use crate::value::DataValue;
use serde::{Deserialize, Serialize};

/// High-level data types supported in external datasets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Null,
    Boolean,
    Int64,
    Float64,
    Date,
    DateTime,
    Utf8,
    Binary,
}

impl DataType {
    /// Returns the lowest common ancestor (supertype) of two types.
    pub fn merge(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        match (self, other) {
            (Self::Null, t) | (t, Self::Null) => t,
            (Self::Int64, Self::Float64) | (Self::Float64, Self::Int64) => Self::Float64,
            (Self::Date, Self::DateTime) | (Self::DateTime, Self::Date) => Self::DateTime,
            _ => Self::Utf8,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Boolean => "Boolean",
            Self::Int64 => "Int64",
            Self::Float64 => "Float64",
            Self::Date => "Date",
            Self::DateTime => "DateTime",
            Self::Utf8 => "Utf8",
            Self::Binary => "Binary",
        }
    }
}

/// A named column field in a dataset schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, data_type: DataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable,
        }
    }
}

/// Schema representing column metadata of a tabular dataset.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn column_count(&self) -> usize {
        self.fields.len()
    }

    pub fn field_by_name(&self, name: &str) -> Option<(usize, &Field)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }

    pub fn names(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.name.clone()).collect()
    }
}

/// Options configuring schema inference and value parsing.
#[derive(Debug, Clone)]
pub struct InferenceOptions {
    pub null_values: Vec<String>,
    pub true_values: Vec<String>,
    pub false_values: Vec<String>,
    pub max_sample_rows: usize,
    pub date_formats: Vec<String>,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            null_values: vec![
                "".to_string(),
                "null".to_string(),
                "NULL".to_string(),
                "None".to_string(),
                "NA".to_string(),
                "#N/A".to_string(),
                "NaN".to_string(),
                "nil".to_string(),
            ],
            true_values: vec![
                "true".to_string(),
                "TRUE".to_string(),
                "True".to_string(),
                "t".to_string(),
                "T".to_string(),
                "yes".to_string(),
                "YES".to_string(),
            ],
            false_values: vec![
                "false".to_string(),
                "FALSE".to_string(),
                "False".to_string(),
                "f".to_string(),
                "F".to_string(),
                "no".to_string(),
                "NO".to_string(),
            ],
            max_sample_rows: 1000,
            date_formats: vec![
                "YYYY-MM-DD".to_string(),
                "YYYY/MM/DD".to_string(),
                "MM/DD/YYYY".to_string(),
                "DD/MM/YYYY".to_string(),
            ],
        }
    }
}

/// Schema inference engine.
pub struct SchemaInferer {
    options: InferenceOptions,
}

impl Default for SchemaInferer {
    fn default() -> Self {
        Self::new(InferenceOptions::default())
    }
}

impl SchemaInferer {
    pub fn new(options: InferenceOptions) -> Self {
        Self { options }
    }

    pub fn options(&self) -> &InferenceOptions {
        &self.options
    }

    /// Checks if a string representation should be treated as null.
    pub fn is_null(&self, s: &str) -> bool {
        let trimmed = s.trim();
        self.options.null_values.iter().any(|nv| nv == trimmed)
    }

    /// Tests what data type a single token best conforms to.
    pub fn infer_token_type(&self, token: &str) -> DataType {
        let trimmed = token.trim();
        if self.is_null(trimmed) {
            return DataType::Null;
        }

        // Boolean check
        if self.options.true_values.iter().any(|v| v == trimmed)
            || self.options.false_values.iter().any(|v| v == trimmed)
        {
            return DataType::Boolean;
        }

        // Integer check
        if trimmed.parse::<i64>().is_ok() {
            return DataType::Int64;
        }

        // Float check
        if trimmed.parse::<f64>().is_ok() {
            return DataType::Float64;
        }

        // Date check: YYYY-MM-DD or YYYY/MM/DD
        if Self::is_iso_date(trimmed) {
            return DataType::Date;
        }

        // DateTime check: YYYY-MM-DDTHH:MM:SS... or YYYY-MM-DD HH:MM:SS
        if Self::is_iso_datetime(trimmed) {
            return DataType::DateTime;
        }

        DataType::Utf8
    }

    /// Determines if string matches YYYY-MM-DD or YYYY/MM/DD.
    fn is_iso_date(s: &str) -> bool {
        if s.len() == 10 {
            let bytes = s.as_bytes();
            let sep1 = bytes[4];
            let sep2 = bytes[7];
            if (sep1 == b'-' && sep2 == b'-') || (sep1 == b'/' && sep2 == b'/') {
                let y = s[0..4].parse::<u32>().ok();
                let m = s[5..7].parse::<u32>().ok();
                let d = s[8..10].parse::<u32>().ok();
                if let (Some(year), Some(month), Some(day)) = (y, m, d) {
                    return year >= 1900 && (1..=12).contains(&month) && (1..=31).contains(&day);
                }
            }
        }
        false
    }

    /// Determines if string matches ISO datetime YYYY-MM-DD[T ]HH:MM:SS.
    fn is_iso_datetime(s: &str) -> bool {
        if s.len() >= 19 {
            let (date_part, rest) = s.split_at(10);
            if Self::is_iso_date(date_part) {
                let sep = rest.as_bytes().first().copied().unwrap_or(0);
                if sep == b'T' || sep == b' ' {
                    let time_part = &rest[1..];
                    if time_part.len() >= 8 {
                        let h = time_part[0..2].parse::<u32>().ok();
                        let m = time_part[3..5].parse::<u32>().ok();
                        let sec = time_part[6..8].parse::<u32>().ok();
                        if let (Some(hour), Some(min), Some(second)) = (h, m, sec) {
                            return hour < 24 && min < 60 && second < 60;
                        }
                    }
                }
            }
        }
        false
    }

    /// Infers schema given column headers and sample row vectors.
    pub fn infer_from_rows(&self, headers: &[String], rows: &[Vec<String>]) -> Schema {
        let col_count = headers.len();
        let sample_limit = rows.len().min(self.options.max_sample_rows);

        let mut col_types = vec![DataType::Null; col_count];
        let mut col_nullable = vec![false; col_count];

        for row in rows.iter().take(sample_limit) {
            for (col_idx, token) in row.iter().enumerate() {
                if col_idx >= col_count {
                    continue;
                }
                let token_type = self.infer_token_type(token);
                if token_type == DataType::Null {
                    col_nullable[col_idx] = true;
                } else {
                    col_types[col_idx] = col_types[col_idx].merge(token_type);
                }
            }
        }

        let fields = headers
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let dtype = if col_types[i] == DataType::Null {
                    DataType::Utf8
                } else {
                    col_types[i]
                };
                Field::new(name, dtype, col_nullable[i])
            })
            .collect();

        Schema::new(fields)
    }

    /// Coerces a raw text token into a typed `DataValue` according to target `DataType`.
    pub fn coerce_value(&self, raw: &str, target_type: DataType) -> ConnectorResult<DataValue> {
        let trimmed = raw.trim();
        if self.is_null(trimmed) {
            return Ok(DataValue::Null);
        }

        match target_type {
            DataType::Null => Ok(DataValue::Null),
            DataType::Boolean => {
                if self.options.true_values.iter().any(|v| v == trimmed) || trimmed == "1" {
                    Ok(DataValue::Boolean(true))
                } else if self.options.false_values.iter().any(|v| v == trimmed) || trimmed == "0" {
                    Ok(DataValue::Boolean(false))
                } else {
                    Err(ConnectorError::Coercion {
                        value: raw.to_string(),
                        target_type: "Boolean".to_string(),
                    })
                }
            }
            DataType::Int64 => match trimmed.parse::<i64>() {
                Ok(v) => Ok(DataValue::Int64(v)),
                Err(_) => {
                    // Try parsing as float and checking if integer
                    if let Ok(f) = trimmed.parse::<f64>() {
                        if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                            return Ok(DataValue::Int64(f as i64));
                        }
                    }
                    Err(ConnectorError::Coercion {
                        value: raw.to_string(),
                        target_type: "Int64".to_string(),
                    })
                }
            },
            DataType::Float64 => match trimmed.parse::<f64>() {
                Ok(v) => Ok(DataValue::Float64(v)),
                Err(_) => Err(ConnectorError::Coercion {
                    value: raw.to_string(),
                    target_type: "Float64".to_string(),
                }),
            },
            DataType::Date => {
                // If it's a date YYYY-MM-DD, convert to days since 1900 or 1970
                if Self::is_iso_date(trimmed) {
                    let y = trimmed[0..4].parse::<i32>().unwrap_or(1970);
                    let m = trimmed[5..7].parse::<u32>().unwrap_or(1);
                    let d = trimmed[8..10].parse::<u32>().unwrap_or(1);
                    let days = days_from_civil(y, m, d);
                    Ok(DataValue::Date(days))
                } else {
                    // Fallback to text if parsing fails
                    Ok(DataValue::Text(raw.to_string()))
                }
            }
            DataType::DateTime => {
                // If it's ISO datetime, parse
                if Self::is_iso_datetime(trimmed) {
                    let y = trimmed[0..4].parse::<i32>().unwrap_or(1970);
                    let m = trimmed[5..7].parse::<u32>().unwrap_or(1);
                    let d = trimmed[8..10].parse::<u32>().unwrap_or(1);
                    let h = trimmed[11..13].parse::<u32>().unwrap_or(0);
                    let min = trimmed[14..16].parse::<u32>().unwrap_or(0);
                    let sec = trimmed[17..19].parse::<u32>().unwrap_or(0);
                    let days = days_from_civil(y, m, d);
                    let millis =
                        (days as i64 * 86400 + h as i64 * 3600 + min as i64 * 60 + sec as i64)
                            * 1000;
                    Ok(DataValue::DateTime(millis))
                } else {
                    Ok(DataValue::Text(raw.to_string()))
                }
            }
            DataType::Utf8 => Ok(DataValue::Text(raw.to_string())),
            DataType::Binary => Ok(DataValue::Binary(raw.as_bytes().to_vec())),
        }
    }
}

/// Computes days from Unix epoch (1970-01-01) using algorithm from Howard Hinnant.
pub fn days_from_civil(y: i32, m: u32, d: u32) -> i32 {
    let mut y = y;
    let m = m as i32;
    let d = d as i32;
    if m <= 2 {
        y -= 1;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
