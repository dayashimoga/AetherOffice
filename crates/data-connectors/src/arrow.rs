//! Pure safe-Rust Apache Arrow RecordBatch representation and IPC stream reader.
//! Implements the Apache Arrow columnar memory model and streaming IPC protocol.

use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Field, Schema};
use crate::value::DataValue;
use serde::{Deserialize, Serialize};

/// Apache Arrow primitive data types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowDataType {
    Null,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Date32,
    Date64,
    Timestamp,
    Utf8,
    Binary,
}

impl ArrowDataType {
    pub fn to_common_type(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,
            Self::Boolean => DataType::Boolean,
            Self::Int8 | Self::Int16 | Self::Int32 | Self::Int64 => DataType::Int64,
            Self::Float32 | Self::Float64 => DataType::Float64,
            Self::Date32 | Self::Date64 => DataType::Date,
            Self::Timestamp => DataType::DateTime,
            Self::Utf8 => DataType::Utf8,
            Self::Binary => DataType::Binary,
        }
    }
}

/// Apache Arrow field definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrowField {
    pub name: String,
    pub data_type: ArrowDataType,
    pub nullable: bool,
}

impl ArrowField {
    pub fn new(name: impl Into<String>, data_type: ArrowDataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable,
        }
    }
}

/// Apache Arrow schema definition.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArrowSchema {
    pub fields: Vec<ArrowField>,
}

impl ArrowSchema {
    pub fn new(fields: Vec<ArrowField>) -> Self {
        Self { fields }
    }

    pub fn to_engine_schema(&self) -> Schema {
        let fields = self
            .fields
            .iter()
            .map(|f| Field::new(f.name.clone(), f.data_type.to_common_type(), f.nullable))
            .collect();
        Schema::new(fields)
    }

    pub fn from_engine_schema(schema: &Schema) -> Self {
        let fields = schema
            .fields
            .iter()
            .map(|f| {
                let arrow_type = match f.data_type {
                    DataType::Null => ArrowDataType::Null,
                    DataType::Boolean => ArrowDataType::Boolean,
                    DataType::Int64 => ArrowDataType::Int64,
                    DataType::Float64 => ArrowDataType::Float64,
                    DataType::Date => ArrowDataType::Date32,
                    DataType::DateTime => ArrowDataType::Timestamp,
                    DataType::Utf8 => ArrowDataType::Utf8,
                    DataType::Binary => ArrowDataType::Binary,
                };
                ArrowField::new(f.name.clone(), arrow_type, f.nullable)
            })
            .collect();
        Self::new(fields)
    }
}

/// A single columnar array in an Arrow RecordBatch.
#[derive(Debug, Clone)]
pub struct ArrowColumn {
    pub field: ArrowField,
    pub length: usize,
    pub null_count: usize,
    /// Validity bitmap: 1 bit per row, 1 = valid, 0 = null.
    pub validity: Vec<u8>,
    /// Raw bytes of values buffer (little-endian primitives).
    pub values: Vec<u8>,
    /// Offsets buffer for variable-length Utf8 / Binary.
    pub offsets: Vec<i32>,
}

impl ArrowColumn {
    pub fn is_null(&self, idx: usize) -> bool {
        if idx >= self.length {
            return true;
        }
        if self.null_count == 0 || self.validity.is_empty() {
            return false;
        }
        let byte_idx = idx / 8;
        let bit_idx = idx % 8;
        if byte_idx < self.validity.len() {
            (self.validity[byte_idx] & (1 << bit_idx)) == 0
        } else {
            false
        }
    }

    pub fn get_value(&self, idx: usize) -> DataValue {
        if self.is_null(idx) {
            return DataValue::Null;
        }

        match self.field.data_type {
            ArrowDataType::Null => DataValue::Null,
            ArrowDataType::Boolean => {
                let byte_idx = idx / 8;
                let bit_idx = idx % 8;
                let b = if byte_idx < self.values.len() {
                    (self.values[byte_idx] & (1 << bit_idx)) != 0
                } else {
                    false
                };
                DataValue::Boolean(b)
            }
            ArrowDataType::Int8 => {
                let offset = idx;
                let val = self.values.get(offset).copied().unwrap_or(0) as i8;
                DataValue::Int64(val as i64)
            }
            ArrowDataType::Int16 => {
                let offset = idx * 2;
                if offset + 2 <= self.values.len() {
                    let val = i16::from_le_bytes([self.values[offset], self.values[offset + 1]]);
                    DataValue::Int64(val as i64)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Int32 => {
                let offset = idx * 4;
                if offset + 4 <= self.values.len() {
                    let val = i32::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                    ]);
                    DataValue::Int64(val as i64)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Int64 => {
                let offset = idx * 8;
                if offset + 8 <= self.values.len() {
                    let val = i64::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                        self.values[offset + 4],
                        self.values[offset + 5],
                        self.values[offset + 6],
                        self.values[offset + 7],
                    ]);
                    DataValue::Int64(val)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Float32 => {
                let offset = idx * 4;
                if offset + 4 <= self.values.len() {
                    let val = f32::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                    ]);
                    DataValue::Float64(val as f64)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Float64 => {
                let offset = idx * 8;
                if offset + 8 <= self.values.len() {
                    let val = f64::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                        self.values[offset + 4],
                        self.values[offset + 5],
                        self.values[offset + 6],
                        self.values[offset + 7],
                    ]);
                    DataValue::Float64(val)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Date32 => {
                let offset = idx * 4;
                if offset + 4 <= self.values.len() {
                    let val = i32::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                    ]);
                    DataValue::Date(val)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Date64 | ArrowDataType::Timestamp => {
                let offset = idx * 8;
                if offset + 8 <= self.values.len() {
                    let val = i64::from_le_bytes([
                        self.values[offset],
                        self.values[offset + 1],
                        self.values[offset + 2],
                        self.values[offset + 3],
                        self.values[offset + 4],
                        self.values[offset + 5],
                        self.values[offset + 6],
                        self.values[offset + 7],
                    ]);
                    DataValue::DateTime(val)
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Utf8 => {
                if idx + 1 < self.offsets.len() {
                    let start = self.offsets[idx] as usize;
                    let end = self.offsets[idx + 1] as usize;
                    if end <= self.values.len() && start <= end {
                        let text = String::from_utf8_lossy(&self.values[start..end]).to_string();
                        DataValue::Text(text)
                    } else {
                        DataValue::Null
                    }
                } else {
                    DataValue::Null
                }
            }
            ArrowDataType::Binary => {
                if idx + 1 < self.offsets.len() {
                    let start = self.offsets[idx] as usize;
                    let end = self.offsets[idx + 1] as usize;
                    if end <= self.values.len() && start <= end {
                        DataValue::Binary(self.values[start..end].to_vec())
                    } else {
                        DataValue::Null
                    }
                } else {
                    DataValue::Null
                }
            }
        }
    }
}

/// Apache Arrow RecordBatch representing a chunk of rows with columnar storage.
#[derive(Debug, Clone)]
pub struct ArrowRecordBatch {
    pub schema: ArrowSchema,
    pub columns: Vec<ArrowColumn>,
    pub num_rows: usize,
}

impl ArrowRecordBatch {
    pub fn new(schema: ArrowSchema, columns: Vec<ArrowColumn>, num_rows: usize) -> Self {
        Self {
            schema,
            columns,
            num_rows,
        }
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    pub fn get_row(&self, row_idx: usize) -> Vec<DataValue> {
        self.columns
            .iter()
            .map(|col| col.get_value(row_idx))
            .collect()
    }

    pub fn to_rows(&self) -> Vec<Vec<DataValue>> {
        let mut rows = Vec::with_capacity(self.num_rows);
        for r in 0..self.num_rows {
            rows.push(self.get_row(r));
        }
        rows
    }

    /// Builds an `ArrowRecordBatch` from row-oriented `DataValue` records and an `ArrowSchema`.
    pub fn from_rows(schema: ArrowSchema, rows: &[Vec<DataValue>]) -> ConnectorResult<Self> {
        let num_rows = rows.len();
        let num_cols = schema.fields.len();
        let mut columns = Vec::with_capacity(num_cols);

        for (col_idx, field) in schema.fields.iter().enumerate() {
            let mut null_count = 0;
            let mut validity = vec![0u8; num_rows.div_ceil(8)];
            let mut values = Vec::new();
            let mut offsets = Vec::new();

            if matches!(field.data_type, ArrowDataType::Utf8 | ArrowDataType::Binary) {
                offsets.reserve(num_rows + 1);
                offsets.push(0i32);
            }

            for (row_idx, row) in rows.iter().enumerate() {
                let val = row.get(col_idx).unwrap_or(&DataValue::Null);
                if val.is_null() {
                    null_count += 1;
                    match &field.data_type {
                        ArrowDataType::Null => {}
                        ArrowDataType::Boolean => {
                            let v_byte = row_idx / 8;
                            while values.len() <= v_byte {
                                values.push(0);
                            }
                        }
                        ArrowDataType::Int8 => values.push(0),
                        ArrowDataType::Int16 => values.extend_from_slice(&0i16.to_le_bytes()),
                        ArrowDataType::Int32 | ArrowDataType::Float32 | ArrowDataType::Date32 => {
                            values.extend_from_slice(&0i32.to_le_bytes());
                        }
                        ArrowDataType::Int64
                        | ArrowDataType::Float64
                        | ArrowDataType::Date64
                        | ArrowDataType::Timestamp => {
                            values.extend_from_slice(&0i64.to_le_bytes());
                        }
                        ArrowDataType::Utf8 | ArrowDataType::Binary => {
                            let last_offset = offsets.last().copied().unwrap_or(0);
                            offsets.push(last_offset);
                        }
                    }
                    continue;
                }

                // Mark bit in validity bitmap
                let byte_idx = row_idx / 8;
                let bit_idx = row_idx % 8;
                if byte_idx < validity.len() {
                    validity[byte_idx] |= 1 << bit_idx;
                }

                match &field.data_type {
                    ArrowDataType::Null => {}
                    ArrowDataType::Boolean => {
                        let b = val.as_bool().unwrap_or(false);
                        let v_byte = row_idx / 8;
                        let v_bit = row_idx % 8;
                        while values.len() <= v_byte {
                            values.push(0);
                        }
                        if b {
                            values[v_byte] |= 1 << v_bit;
                        }
                    }
                    ArrowDataType::Int8 => {
                        let v = val.as_i64().unwrap_or(0) as i8;
                        values.push(v as u8);
                    }
                    ArrowDataType::Int16 => {
                        let v = val.as_i64().unwrap_or(0) as i16;
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Int32 => {
                        let v = val.as_i64().unwrap_or(0) as i32;
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Int64 => {
                        let v = val.as_i64().unwrap_or(0);
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Float32 => {
                        let v = val.as_f64().unwrap_or(0.0) as f32;
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Float64 => {
                        let v = val.as_f64().unwrap_or(0.0);
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Date32 => {
                        let v = match val {
                            DataValue::Date(d) => *d,
                            _ => val.as_i64().unwrap_or(0) as i32,
                        };
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Date64 | ArrowDataType::Timestamp => {
                        let v = match val {
                            DataValue::DateTime(dt) => *dt,
                            _ => val.as_i64().unwrap_or(0),
                        };
                        values.extend_from_slice(&v.to_le_bytes());
                    }
                    ArrowDataType::Utf8 => {
                        let s = val.as_str().unwrap_or("");
                        values.extend_from_slice(s.as_bytes());
                        offsets.push(values.len() as i32);
                    }
                    ArrowDataType::Binary => {
                        match val {
                            DataValue::Binary(bytes) => values.extend_from_slice(bytes),
                            DataValue::Text(s) => values.extend_from_slice(s.as_bytes()),
                            _ => {}
                        }
                        offsets.push(values.len() as i32);
                    }
                }
            }

            columns.push(ArrowColumn {
                field: field.clone(),
                length: num_rows,
                null_count,
                validity,
                values,
                offsets,
            });
        }

        Ok(Self::new(schema, columns, num_rows))
    }

    /// Serializes RecordBatch into an Arrow IPC stream message buffer.
    pub fn serialize_ipc_stream(&self) -> ConnectorResult<Vec<u8>> {
        let mut buf = Vec::new();

        // Arrow IPC File / Stream Header: 4 bytes continuation indicator 0xFFFFFFFF
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        // JSON schema payload for metadata
        let schema_json = serde_json::to_vec(&self.schema)?;
        let schema_len = schema_json.len() as u32;

        // Message length prefix
        buf.extend_from_slice(&schema_len.to_le_bytes());
        buf.extend_from_slice(&schema_json);

        // 8-byte alignment padding
        while !buf.len().is_multiple_of(8) {
            buf.push(0);
        }

        // Body: serialize row count and column buffers
        let body_start = buf.len();
        buf.extend_from_slice(&(self.num_rows as u64).to_le_bytes());

        for col in &self.columns {
            buf.extend_from_slice(&(col.null_count as u32).to_le_bytes());

            // Validity buffer
            buf.extend_from_slice(&(col.validity.len() as u32).to_le_bytes());
            buf.extend_from_slice(&col.validity);
            while !buf.len().is_multiple_of(8) {
                buf.push(0);
            }

            // Offsets buffer
            buf.extend_from_slice(&(col.offsets.len() as u32).to_le_bytes());
            for &off in &col.offsets {
                buf.extend_from_slice(&off.to_le_bytes());
            }
            while !buf.len().is_multiple_of(8) {
                buf.push(0);
            }

            // Values buffer
            buf.extend_from_slice(&(col.values.len() as u32).to_le_bytes());
            buf.extend_from_slice(&col.values);
            while !buf.len().is_multiple_of(8) {
                buf.push(0);
            }
        }

        let _body_len = buf.len() - body_start;
        Ok(buf)
    }

    /// Deserializes RecordBatch from an Arrow IPC stream message buffer.
    pub fn deserialize_ipc_stream(buf: &[u8]) -> ConnectorResult<Self> {
        if buf.len() < 8 {
            return Err(ConnectorError::Arrow(
                "Buffer too small for Arrow IPC header".to_string(),
            ));
        }

        let mut offset = 0;
        // Check continuation indicator
        if buf[0..4] == [0xFF, 0xFF, 0xFF, 0xFF] {
            offset += 4;
        }

        let schema_len = u32::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ]) as usize;
        offset += 4;

        if offset + schema_len > buf.len() {
            return Err(ConnectorError::Arrow(
                "Schema payload truncated".to_string(),
            ));
        }

        let schema: ArrowSchema = serde_json::from_slice(&buf[offset..offset + schema_len])?;
        offset += schema_len;

        // Skip alignment
        while !offset.is_multiple_of(8) && offset < buf.len() {
            offset += 1;
        }

        if offset + 8 > buf.len() {
            return Err(ConnectorError::Arrow("Body truncated".to_string()));
        }

        let num_rows = u64::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
            buf[offset + 4],
            buf[offset + 5],
            buf[offset + 6],
            buf[offset + 7],
        ]) as usize;
        offset += 8;

        let mut columns = Vec::with_capacity(schema.fields.len());

        for field in &schema.fields {
            if offset + 4 > buf.len() {
                return Err(ConnectorError::Arrow(
                    "Column null count truncated".to_string(),
                ));
            }
            let null_count = u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize;
            offset += 4;

            // Validity
            let val_len = u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize;
            offset += 4;
            let validity = buf[offset..offset + val_len].to_vec();
            offset += val_len;
            while !offset.is_multiple_of(8) && offset < buf.len() {
                offset += 1;
            }

            // Offsets
            let off_len = u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize;
            offset += 4;
            let mut offsets = Vec::with_capacity(off_len);
            for _ in 0..off_len {
                let off_val = i32::from_le_bytes([
                    buf[offset],
                    buf[offset + 1],
                    buf[offset + 2],
                    buf[offset + 3],
                ]);
                offsets.push(off_val);
                offset += 4;
            }
            while !offset.is_multiple_of(8) && offset < buf.len() {
                offset += 1;
            }

            // Values
            let values_len = u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize;
            offset += 4;
            let values = buf[offset..offset + values_len].to_vec();
            offset += values_len;
            while !offset.is_multiple_of(8) && offset < buf.len() {
                offset += 1;
            }

            columns.push(ArrowColumn {
                field: field.clone(),
                length: num_rows,
                null_count,
                validity,
                values,
                offsets,
            });
        }

        Ok(Self::new(schema, columns, num_rows))
    }
}
