//! Streaming JSON and JSONL (NDJSON) tabular readers.

use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Field, Schema, SchemaInferer};
use crate::value::DataValue;
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::BufRead;

/// JSON tabular ingestion format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonFormat {
    /// Auto-detect between ArrayOfObjects, ArrayOfArrays, and LineDelimited.
    Auto,
    /// Standard JSON array of objects: `[{"a": 1}, {"a": 2}]`
    ArrayOfObjects,
    /// JSON array of row arrays: `[["a", "b"], [1, 2]]`
    ArrayOfArrays,
    /// Line-delimited JSON (JSONL / NDJSON): one JSON object per line.
    LineDelimited,
}

/// Streaming JSON reader.
pub struct JsonReader {
    format: JsonFormat,
    inferer: SchemaInferer,
}

impl Default for JsonReader {
    fn default() -> Self {
        Self {
            format: JsonFormat::Auto,
            inferer: SchemaInferer::default(),
        }
    }
}

impl JsonReader {
    pub fn new(format: JsonFormat) -> Self {
        Self {
            format,
            inferer: SchemaInferer::default(),
        }
    }

    /// Reads and parses tabular data from a JSON string or buffer.
    pub fn read_str(&self, input: &str) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok((Schema::default(), Vec::new()));
        }

        let detected_format = match self.format {
            JsonFormat::Auto => {
                if trimmed.starts_with('[') {
                    // Check if first element is object or array
                    if let Some(inner) = trimmed.strip_prefix('[') {
                        let inner_trim = inner.trim_start();
                        if inner_trim.starts_with('{') {
                            JsonFormat::ArrayOfObjects
                        } else if inner_trim.starts_with('[') {
                            JsonFormat::ArrayOfArrays
                        } else {
                            JsonFormat::ArrayOfObjects
                        }
                    } else {
                        JsonFormat::ArrayOfObjects
                    }
                } else if trimmed.starts_with('{') {
                    JsonFormat::LineDelimited
                } else {
                    return Err(ConnectorError::Json(
                        "Unsupported JSON structure (expected array or lines of objects)"
                            .to_string(),
                    ));
                }
            }
            f => f,
        };

        match detected_format {
            JsonFormat::ArrayOfObjects => self.parse_array_of_objects(trimmed),
            JsonFormat::ArrayOfArrays => self.parse_array_of_arrays(trimmed),
            JsonFormat::LineDelimited | JsonFormat::Auto => self.parse_jsonl(trimmed),
        }
    }

    /// Reads JSONL from a BufRead stream.
    pub fn read_jsonl_stream<R: BufRead>(
        &self,
        mut reader: R,
    ) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let mut lines = Vec::new();
        let mut buf = String::new();
        while reader.read_line(&mut buf)? > 0 {
            let trimmed = buf.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
            buf.clear();
        }
        self.parse_json_lines(&lines)
    }

    fn parse_array_of_objects(
        &self,
        input: &str,
    ) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let parsed: Value = serde_json::from_str(input)?;
        let arr = match parsed {
            Value::Array(a) => a,
            Value::Object(map) => vec![Value::Object(map)],
            _ => {
                return Err(ConnectorError::Json(
                    "Expected JSON array of objects".to_string(),
                ))
            }
        };

        // Collect all unique keys maintaining deterministic order
        let mut keys = BTreeSet::new();
        for item in &arr {
            if let Value::Object(map) = item {
                for k in map.keys() {
                    keys.insert(k.clone());
                }
            }
        }

        let key_list: Vec<String> = keys.into_iter().collect();
        let mut rows = Vec::with_capacity(arr.len());

        for item in arr {
            if let Value::Object(map) = item {
                let mut row = Vec::with_capacity(key_list.len());
                for k in &key_list {
                    let v = map.get(k).unwrap_or(&Value::Null);
                    row.push(self.json_value_to_data_value(v));
                }
                rows.push(row);
            }
        }

        let schema = self.infer_schema_from_values(&key_list, &rows);
        Ok((schema, rows))
    }

    fn parse_array_of_arrays(&self, input: &str) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let parsed: Value = serde_json::from_str(input)?;
        let arr = match parsed {
            Value::Array(a) => a,
            _ => {
                return Err(ConnectorError::Json(
                    "Expected JSON array of arrays".to_string(),
                ))
            }
        };

        if arr.is_empty() {
            return Ok((Schema::default(), Vec::new()));
        }

        let mut headers = Vec::new();
        let mut start_idx = 0;

        // If first row is strings, use as headers
        if let Some(Value::Array(first_row)) = arr.first() {
            let all_strings = first_row.iter().all(|v| v.is_string());
            if all_strings && arr.len() > 1 {
                for v in first_row {
                    headers.push(v.as_str().unwrap_or("").to_string());
                }
                start_idx = 1;
            } else {
                for i in 0..first_row.len() {
                    headers.push(format!("Column_{}", i + 1));
                }
            }
        }

        let mut rows = Vec::with_capacity(arr.len() - start_idx);
        for item in &arr[start_idx..] {
            if let Value::Array(row_arr) = item {
                let mut row = Vec::with_capacity(headers.len());
                for (idx, _) in headers.iter().enumerate() {
                    let v = row_arr.get(idx).unwrap_or(&Value::Null);
                    row.push(self.json_value_to_data_value(v));
                }
                rows.push(row);
            }
        }

        let schema = self.infer_schema_from_values(&headers, &rows);
        Ok((schema, rows))
    }

    fn parse_jsonl(&self, input: &str) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let lines: Vec<String> = input
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        self.parse_json_lines(&lines)
    }

    fn parse_json_lines(&self, lines: &[String]) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let mut objects = Vec::with_capacity(lines.len());
        let mut keys = BTreeSet::new();

        for line in lines {
            let val: Value = serde_json::from_str(line)?;
            if let Value::Object(map) = val {
                for k in map.keys() {
                    keys.insert(k.clone());
                }
                objects.push(map);
            }
        }

        let key_list: Vec<String> = keys.into_iter().collect();
        let mut rows = Vec::with_capacity(objects.len());

        for map in objects {
            let mut row = Vec::with_capacity(key_list.len());
            for k in &key_list {
                let v = map.get(k).unwrap_or(&Value::Null);
                row.push(self.json_value_to_data_value(v));
            }
            rows.push(row);
        }

        let schema = self.infer_schema_from_values(&key_list, &rows);
        Ok((schema, rows))
    }

    fn json_value_to_data_value(&self, val: &Value) -> DataValue {
        match val {
            Value::Null => DataValue::Null,
            Value::Bool(b) => DataValue::Boolean(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    DataValue::Int64(i)
                } else if let Some(f) = n.as_f64() {
                    DataValue::Float64(f)
                } else {
                    DataValue::Text(n.to_string())
                }
            }
            Value::String(s) => {
                let inferred = self.inferer.infer_token_type(s);
                match inferred {
                    DataType::Date | DataType::DateTime => self
                        .inferer
                        .coerce_value(s, inferred)
                        .unwrap_or(DataValue::Text(s.clone())),
                    _ => DataValue::Text(s.clone()),
                }
            }
            Value::Array(_) | Value::Object(_) => DataValue::Text(val.to_string()),
        }
    }

    fn infer_schema_from_values(&self, names: &[String], rows: &[Vec<DataValue>]) -> Schema {
        let col_count = names.len();
        let mut col_types = vec![DataType::Null; col_count];
        let mut col_nullable = vec![false; col_count];

        for row in rows.iter().take(1000) {
            for (col_idx, val) in row.iter().enumerate() {
                if col_idx >= col_count {
                    continue;
                }
                match val {
                    DataValue::Null => col_nullable[col_idx] = true,
                    DataValue::Boolean(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Boolean)
                    }
                    DataValue::Int64(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Int64)
                    }
                    DataValue::Float64(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Float64)
                    }
                    DataValue::Date(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Date)
                    }
                    DataValue::DateTime(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::DateTime)
                    }
                    DataValue::Text(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Utf8)
                    }
                    DataValue::Binary(_) => {
                        col_types[col_idx] = col_types[col_idx].merge(DataType::Binary)
                    }
                }
            }
        }

        let fields = names
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
}
