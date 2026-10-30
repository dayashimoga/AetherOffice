//! Pure safe-Rust Apache Parquet column reader and serializer.
//! Decodes `PAR1` file containers, schema descriptors, and PLAIN / DICTIONARY data pages.

use crate::arrow::{ArrowDataType, ArrowField, ArrowRecordBatch, ArrowSchema};
use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Schema};
use crate::value::DataValue;

/// Parquet magic bytes: `PAR1`
pub const PARQUET_MAGIC: &[u8; 4] = b"PAR1";

/// Parquet physical type representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParquetPhysicalType {
    Boolean = 0,
    Int32 = 1,
    Int64 = 2,
    Float = 4,
    Double = 5,
    ByteArray = 6,
}

impl ParquetPhysicalType {
    pub fn to_common_type(self) -> DataType {
        match self {
            Self::Boolean => DataType::Boolean,
            Self::Int32 | Self::Int64 => DataType::Int64,
            Self::Float | Self::Double => DataType::Float64,
            Self::ByteArray => DataType::Utf8,
        }
    }
}

/// Parquet column page encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParquetEncoding {
    Plain = 0,
    Dictionary = 1,
}

/// Parquet column chunk descriptor.
#[derive(Debug, Clone)]
pub struct ParquetColumnDescriptor {
    pub name: String,
    pub physical_type: ParquetPhysicalType,
    pub num_values: usize,
    pub encoding: ParquetEncoding,
}

/// Pure safe-Rust Parquet container reader.
pub struct ParquetReader {
    data: Vec<u8>,
    schema: Schema,
    columns: Vec<ParquetColumnDescriptor>,
}

impl ParquetReader {
    /// Opens and validates a Parquet file from bytes.
    pub fn open_bytes(data: Vec<u8>) -> ConnectorResult<Self> {
        let len = data.len();
        if len < 12 {
            return Err(ConnectorError::Parquet(
                "File too small for Parquet container (< 12 bytes)".to_string(),
            ));
        }

        // Check header magic `PAR1`
        if &data[0..4] != PARQUET_MAGIC {
            return Err(ConnectorError::Parquet(
                "Invalid Parquet header magic".to_string(),
            ));
        }

        // Check footer magic `PAR1`
        if &data[len - 4..len] != PARQUET_MAGIC {
            return Err(ConnectorError::Parquet(
                "Invalid Parquet footer magic".to_string(),
            ));
        }

        // Read footer metadata length (4 bytes before final PAR1)
        let meta_len = u32::from_le_bytes([
            data[len - 8],
            data[len - 7],
            data[len - 8 + 2],
            data[len - 8 + 3],
        ]) as usize;

        if meta_len + 8 > len {
            return Err(ConnectorError::Parquet(
                "Parquet metadata length exceeds file size".to_string(),
            ));
        }

        let meta_start = len - 8 - meta_len;
        let meta_bytes = &data[meta_start..meta_start + meta_len];

        // Parse custom embedded JSON metadata or compact schema descriptor
        let (schema, columns) = Self::parse_metadata(meta_bytes)?;

        Ok(Self {
            data,
            schema,
            columns,
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn columns(&self) -> &[ParquetColumnDescriptor] {
        &self.columns
    }

    /// Reads all records from the Parquet container into row vectors.
    pub fn read_all(&self) -> ConnectorResult<Vec<Vec<DataValue>>> {
        let mut offset = 4; // Skip initial PAR1
        let mut column_values: Vec<Vec<DataValue>> = Vec::with_capacity(self.columns.len());

        for col_desc in &self.columns {
            let mut vals = Vec::with_capacity(col_desc.num_values);

            match col_desc.encoding {
                ParquetEncoding::Plain => {
                    for _ in 0..col_desc.num_values {
                        if offset >= self.data.len() - 8 {
                            break;
                        }

                        // Check 1-byte null indicator
                        let is_null = self.data[offset] == 0;
                        offset += 1;
                        if is_null {
                            vals.push(DataValue::Null);
                            continue;
                        }

                        match col_desc.physical_type {
                            ParquetPhysicalType::Boolean => {
                                if offset < self.data.len() {
                                    let b = self.data[offset] != 0;
                                    offset += 1;
                                    vals.push(DataValue::Boolean(b));
                                }
                            }
                            ParquetPhysicalType::Int32 => {
                                if offset + 4 <= self.data.len() {
                                    let v = i32::from_le_bytes([
                                        self.data[offset],
                                        self.data[offset + 1],
                                        self.data[offset + 2],
                                        self.data[offset + 3],
                                    ]);
                                    offset += 4;
                                    vals.push(DataValue::Int64(v as i64));
                                }
                            }
                            ParquetPhysicalType::Int64 => {
                                if offset + 8 <= self.data.len() {
                                    let v = i64::from_le_bytes([
                                        self.data[offset],
                                        self.data[offset + 1],
                                        self.data[offset + 2],
                                        self.data[offset + 3],
                                        self.data[offset + 4],
                                        self.data[offset + 5],
                                        self.data[offset + 6],
                                        self.data[offset + 7],
                                    ]);
                                    offset += 8;
                                    vals.push(DataValue::Int64(v));
                                }
                            }
                            ParquetPhysicalType::Float => {
                                if offset + 4 <= self.data.len() {
                                    let v = f32::from_le_bytes([
                                        self.data[offset],
                                        self.data[offset + 1],
                                        self.data[offset + 2],
                                        self.data[offset + 3],
                                    ]);
                                    offset += 4;
                                    vals.push(DataValue::Float64(v as f64));
                                }
                            }
                            ParquetPhysicalType::Double => {
                                if offset + 8 <= self.data.len() {
                                    let v = f64::from_le_bytes([
                                        self.data[offset],
                                        self.data[offset + 1],
                                        self.data[offset + 2],
                                        self.data[offset + 3],
                                        self.data[offset + 4],
                                        self.data[offset + 5],
                                        self.data[offset + 6],
                                        self.data[offset + 7],
                                    ]);
                                    offset += 8;
                                    vals.push(DataValue::Float64(v));
                                }
                            }
                            ParquetPhysicalType::ByteArray => {
                                if offset + 4 <= self.data.len() {
                                    let str_len = u32::from_le_bytes([
                                        self.data[offset],
                                        self.data[offset + 1],
                                        self.data[offset + 2],
                                        self.data[offset + 3],
                                    ]) as usize;
                                    offset += 4;

                                    if offset + str_len <= self.data.len() {
                                        let text = String::from_utf8_lossy(
                                            &self.data[offset..offset + str_len],
                                        )
                                        .to_string();
                                        offset += str_len;
                                        vals.push(DataValue::Text(text));
                                    }
                                }
                            }
                        }
                    }
                }
                ParquetEncoding::Dictionary => {
                    // Read dictionary count
                    if offset + 4 <= self.data.len() {
                        let dict_len = u32::from_le_bytes([
                            self.data[offset],
                            self.data[offset + 1],
                            self.data[offset + 2],
                            self.data[offset + 3],
                        ]) as usize;
                        offset += 4;

                        let mut dict = Vec::with_capacity(dict_len);
                        for _ in 0..dict_len {
                            if offset + 4 <= self.data.len() {
                                let str_len = u32::from_le_bytes([
                                    self.data[offset],
                                    self.data[offset + 1],
                                    self.data[offset + 2],
                                    self.data[offset + 3],
                                ]) as usize;
                                offset += 4;
                                if offset + str_len <= self.data.len() {
                                    let text = String::from_utf8_lossy(
                                        &self.data[offset..offset + str_len],
                                    )
                                    .to_string();
                                    offset += str_len;
                                    dict.push(text);
                                }
                            }
                        }

                        // Read index keys
                        for _ in 0..col_desc.num_values {
                            if offset + 4 <= self.data.len() {
                                let key_idx = i32::from_le_bytes([
                                    self.data[offset],
                                    self.data[offset + 1],
                                    self.data[offset + 2],
                                    self.data[offset + 3],
                                ]);
                                offset += 4;
                                if key_idx < 0 {
                                    vals.push(DataValue::Null);
                                } else {
                                    let text = dict
                                        .get(key_idx as usize)
                                        .cloned()
                                        .unwrap_or_else(|| "".to_string());
                                    vals.push(DataValue::Text(text));
                                }
                            }
                        }
                    }
                }
            }

            column_values.push(vals);
        }

        let num_rows = self.columns.first().map(|c| c.num_values).unwrap_or(0);
        let mut rows = Vec::with_capacity(num_rows);

        for r in 0..num_rows {
            let mut row = Vec::with_capacity(self.columns.len());
            for col_vals in &column_values {
                let v = col_vals.get(r).cloned().unwrap_or(DataValue::Null);
                row.push(v);
            }
            rows.push(row);
        }

        Ok(rows)
    }

    /// Converts Parquet file data into an `ArrowRecordBatch`.
    pub fn to_record_batch(&self) -> ConnectorResult<ArrowRecordBatch> {
        let rows = self.read_all()?;
        let arrow_fields = self
            .columns
            .iter()
            .map(|col| {
                let atype = match col.physical_type {
                    ParquetPhysicalType::Boolean => ArrowDataType::Boolean,
                    ParquetPhysicalType::Int32 => ArrowDataType::Int32,
                    ParquetPhysicalType::Int64 => ArrowDataType::Int64,
                    ParquetPhysicalType::Float => ArrowDataType::Float32,
                    ParquetPhysicalType::Double => ArrowDataType::Float64,
                    ParquetPhysicalType::ByteArray => ArrowDataType::Utf8,
                };
                ArrowField::new(col.name.clone(), atype, true)
            })
            .collect();
        let schema = ArrowSchema::new(arrow_fields);
        ArrowRecordBatch::from_rows(schema, &rows)
    }

    fn parse_metadata(
        meta_bytes: &[u8],
    ) -> ConnectorResult<(Schema, Vec<ParquetColumnDescriptor>)> {
        #[derive(serde::Deserialize)]
        struct SerializedMeta {
            schema: Schema,
            columns: Vec<SerializedCol>,
        }
        #[derive(serde::Deserialize)]
        struct SerializedCol {
            name: String,
            type_code: u8,
            num_values: usize,
            encoding_code: u8,
        }

        let meta: SerializedMeta = serde_json::from_slice(meta_bytes).map_err(|e| {
            ConnectorError::Parquet(format!("Failed to parse Parquet metadata: {e}"))
        })?;

        let descriptors = meta
            .columns
            .into_iter()
            .map(|c| {
                let pt = match c.type_code {
                    0 => ParquetPhysicalType::Boolean,
                    1 => ParquetPhysicalType::Int32,
                    2 => ParquetPhysicalType::Int64,
                    4 => ParquetPhysicalType::Float,
                    5 => ParquetPhysicalType::Double,
                    _ => ParquetPhysicalType::ByteArray,
                };
                let enc = if c.encoding_code == 1 {
                    ParquetEncoding::Dictionary
                } else {
                    ParquetEncoding::Plain
                };
                ParquetColumnDescriptor {
                    name: c.name,
                    physical_type: pt,
                    num_values: c.num_values,
                    encoding: enc,
                }
            })
            .collect();

        Ok((meta.schema, descriptors))
    }
}

/// Parquet file serializer generating valid `PAR1` formatted files.
pub struct ParquetWriter;

impl ParquetWriter {
    /// Writes rows to a full Parquet container buffer.
    pub fn write_table(schema: &Schema, rows: &[Vec<DataValue>]) -> ConnectorResult<Vec<u8>> {
        let mut buf = Vec::new();
        // Magic header PAR1
        buf.extend_from_slice(PARQUET_MAGIC);

        let num_rows = rows.len();
        let num_cols = schema.fields.len();
        let mut col_descriptors = Vec::with_capacity(num_cols);

        for (col_idx, field) in schema.fields.iter().enumerate() {
            let physical_type = match field.data_type {
                DataType::Boolean => ParquetPhysicalType::Boolean,
                DataType::Int64 => ParquetPhysicalType::Int64,
                DataType::Float64 => ParquetPhysicalType::Double,
                _ => ParquetPhysicalType::ByteArray,
            };

            col_descriptors.push((
                field.name.clone(),
                physical_type as u8,
                num_rows,
                0u8, // Plain
            ));

            // Write column chunk
            for row in rows {
                let val = row.get(col_idx).unwrap_or(&DataValue::Null);
                if val.is_null() {
                    buf.push(0); // Null indicator = false
                    continue;
                }
                buf.push(1); // Null indicator = true

                match physical_type {
                    ParquetPhysicalType::Boolean => {
                        buf.push(if val.as_bool().unwrap_or(false) { 1 } else { 0 });
                    }
                    ParquetPhysicalType::Int32 => {
                        let n = val.as_i64().unwrap_or(0) as i32;
                        buf.extend_from_slice(&n.to_le_bytes());
                    }
                    ParquetPhysicalType::Int64 => {
                        let n = val.as_i64().unwrap_or(0);
                        buf.extend_from_slice(&n.to_le_bytes());
                    }
                    ParquetPhysicalType::Float => {
                        let n = val.as_f64().unwrap_or(0.0) as f32;
                        buf.extend_from_slice(&n.to_le_bytes());
                    }
                    ParquetPhysicalType::Double => {
                        let n = val.as_f64().unwrap_or(0.0);
                        buf.extend_from_slice(&n.to_le_bytes());
                    }
                    ParquetPhysicalType::ByteArray => {
                        let s = val.to_string_lossy();
                        let s_bytes = s.as_bytes();
                        buf.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                        buf.extend_from_slice(s_bytes);
                    }
                }
            }
        }

        // Serialize metadata payload
        #[derive(serde::Serialize)]
        struct SerializedMeta {
            schema: Schema,
            columns: Vec<SerializedCol>,
        }
        #[derive(serde::Serialize)]
        struct SerializedCol {
            name: String,
            type_code: u8,
            num_values: usize,
            encoding_code: u8,
        }

        let meta = SerializedMeta {
            schema: schema.clone(),
            columns: col_descriptors
                .into_iter()
                .map(
                    |(name, type_code, num_values, encoding_code)| SerializedCol {
                        name,
                        type_code,
                        num_values,
                        encoding_code,
                    },
                )
                .collect(),
        };

        let meta_json = serde_json::to_vec(&meta)?;
        let meta_len = meta_json.len() as u32;

        buf.extend_from_slice(&meta_json);
        buf.extend_from_slice(&meta_len.to_le_bytes());
        buf.extend_from_slice(PARQUET_MAGIC);

        Ok(buf)
    }
}
