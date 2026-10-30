//! High-performance streaming ingestion pipeline mapping external datasets
//! directly into `excel_engine::grid::CompressedSparsePageGrid`.

use crate::arrow::ArrowRecordBatch;
use crate::csv::{CsvReadOptions, CsvReader};
use crate::error::{ConnectorError, ConnectorResult};
use crate::json::{JsonFormat, JsonReader};
use crate::parquet::ParquetReader;
use crate::schema::Schema;
use crate::sqlite::SqliteReader;
use crate::value::DataValue;
use excel_engine::grid::{CompressedSparsePageGrid, MAX_COLS, MAX_ROWS};
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::time::Instant;

/// Report detailing the execution metrics of an ingestion run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionReport {
    pub rows_ingested: usize,
    pub columns_ingested: usize,
    pub cells_written: usize,
    pub duration_ms: f64,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub schema: Schema,
}

/// Ingestion configuration options.
#[derive(Debug, Clone)]
pub struct IngestionOptions {
    pub start_row: u32,
    pub start_col: u32,
    pub write_headers: bool,
    pub batch_size: usize,
    pub max_rows: Option<usize>,
}

impl Default for IngestionOptions {
    fn default() -> Self {
        Self {
            start_row: 0,
            start_col: 0,
            write_headers: true,
            batch_size: 4096,
            max_rows: None,
        }
    }
}

/// High-throughput streaming data ingestion pipeline.
pub struct IngestionPipeline;

impl IngestionPipeline {
    /// Ingests a pre-parsed row dataset into the grid.
    pub fn ingest_rows(
        schema: &Schema,
        rows: &[Vec<DataValue>],
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let start_time = Instant::now();
        let num_cols = schema.fields.len();
        let mut total_rows = rows.len();
        if let Some(max_r) = options.max_rows {
            total_rows = total_rows.min(max_r);
        }

        let header_offset = if options.write_headers { 1 } else { 0 };
        let end_row = options.start_row + header_offset + total_rows as u32;
        let end_col = options.start_col + num_cols as u32;

        if end_row > MAX_ROWS || end_col > MAX_COLS {
            return Err(ConnectorError::BoundsExceeded(format!(
                "Ingestion range ({}, {}) to ({}, {}) exceeds sheet bounds ({}, {})",
                options.start_row, options.start_col, end_row, end_col, MAX_ROWS, MAX_COLS
            )));
        }

        let mut cells_written = 0;

        // Write headers
        if options.write_headers {
            for (c_idx, field) in schema.fields.iter().enumerate() {
                let col = options.start_col + c_idx as u32;
                grid.set_inline_string(options.start_row, col, &field.name)?;
                cells_written += 1;
            }
        }

        // Write rows
        let data_start_row = options.start_row + header_offset;
        for (r_idx, row) in rows.iter().take(total_rows).enumerate() {
            let current_row = data_start_row + r_idx as u32;
            for (c_idx, val) in row.iter().enumerate() {
                if c_idx >= num_cols {
                    break;
                }
                let current_col = options.start_col + c_idx as u32;
                match val {
                    DataValue::Null => {}
                    DataValue::Boolean(b) => {
                        grid.set_boolean(current_row, current_col, *b)?;
                        cells_written += 1;
                    }
                    DataValue::Int64(i) => {
                        grid.set_number(current_row, current_col, *i as f64)?;
                        cells_written += 1;
                    }
                    DataValue::Float64(f) => {
                        grid.set_number(current_row, current_col, *f)?;
                        cells_written += 1;
                    }
                    DataValue::Date(d) => {
                        grid.set_number(current_row, current_col, *d as f64)?;
                        cells_written += 1;
                    }
                    DataValue::DateTime(dt) => {
                        grid.set_number(current_row, current_col, *dt as f64 / 86400000.0)?;
                        cells_written += 1;
                    }
                    DataValue::Text(s) => {
                        grid.set_inline_string(current_row, current_col, s)?;
                        cells_written += 1;
                    }
                    DataValue::Binary(b) => {
                        grid.set_inline_string(
                            current_row,
                            current_col,
                            &format!("<BLOB {} bytes>", b.len()),
                        )?;
                        cells_written += 1;
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        Ok(IngestionReport {
            rows_ingested: total_rows,
            columns_ingested: num_cols,
            cells_written,
            duration_ms: duration.as_secs_f64() * 1000.0,
            start_row: options.start_row,
            start_col: options.start_col,
            end_row,
            end_col,
            schema: schema.clone(),
        })
    }

    /// Ingests a streaming CSV/TSV source directly into the grid batch-by-batch.
    pub fn ingest_csv<R: BufRead>(
        reader: R,
        csv_options: CsvReadOptions,
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let start_time = Instant::now();
        let mut csv_reader = CsvReader::new(reader, csv_options)?;

        // Peek first batch to establish schema
        let first_batch = csv_reader.read_batch_typed()?;
        if first_batch.is_empty() {
            return Ok(IngestionReport {
                rows_ingested: 0,
                columns_ingested: 0,
                cells_written: 0,
                duration_ms: 0.0,
                start_row: options.start_row,
                start_col: options.start_col,
                end_row: options.start_row,
                end_col: options.start_col,
                schema: Schema::default(),
            });
        }

        let schema = csv_reader.schema().cloned().unwrap_or_default();
        let num_cols = schema.fields.len();
        let header_offset = if options.write_headers { 1 } else { 0 };

        let mut cells_written = 0;
        let mut total_rows = 0;

        // Write headers
        if options.write_headers {
            for (c_idx, field) in schema.fields.iter().enumerate() {
                let col = options.start_col + c_idx as u32;
                grid.set_inline_string(options.start_row, col, &field.name)?;
                cells_written += 1;
            }
        }

        let mut write_row_chunk =
            |chunk: &[Vec<DataValue>], start_r: u32| -> ConnectorResult<usize> {
                let mut written = 0;
                for (r_idx, row) in chunk.iter().enumerate() {
                    let current_row = start_r + r_idx as u32;
                    for (c_idx, val) in row.iter().enumerate() {
                        if c_idx >= num_cols {
                            break;
                        }
                        let current_col = options.start_col + c_idx as u32;
                        match val {
                            DataValue::Null => {}
                            DataValue::Boolean(b) => {
                                grid.set_boolean(current_row, current_col, *b)?;
                                written += 1;
                            }
                            DataValue::Int64(i) => {
                                grid.set_number(current_row, current_col, *i as f64)?;
                                written += 1;
                            }
                            DataValue::Float64(f) => {
                                grid.set_number(current_row, current_col, *f)?;
                                written += 1;
                            }
                            DataValue::Date(d) => {
                                grid.set_number(current_row, current_col, *d as f64)?;
                                written += 1;
                            }
                            DataValue::DateTime(dt) => {
                                grid.set_number(current_row, current_col, *dt as f64 / 86400000.0)?;
                                written += 1;
                            }
                            DataValue::Text(s) => {
                                grid.set_inline_string(current_row, current_col, s)?;
                                written += 1;
                            }
                            DataValue::Binary(b) => {
                                grid.set_inline_string(
                                    current_row,
                                    current_col,
                                    &format!("<BLOB {} bytes>", b.len()),
                                )?;
                                written += 1;
                            }
                        }
                    }
                }
                Ok(written)
            };

        // Write first batch
        let mut cur_row = options.start_row + header_offset;
        cells_written += write_row_chunk(&first_batch, cur_row)?;
        total_rows += first_batch.len();
        cur_row += first_batch.len() as u32;

        // Stream remaining batches
        while let Ok(batch) = csv_reader.read_batch_typed() {
            if batch.is_empty() {
                break;
            }
            if let Some(max_r) = options.max_rows {
                if total_rows >= max_r {
                    break;
                }
                let remaining = max_r - total_rows;
                if batch.len() > remaining {
                    cells_written += write_row_chunk(&batch[..remaining], cur_row)?;
                    total_rows += remaining;
                    cur_row += remaining as u32;
                    break;
                }
            }

            cells_written += write_row_chunk(&batch, cur_row)?;
            total_rows += batch.len();
            cur_row += batch.len() as u32;
        }

        let duration = start_time.elapsed();
        Ok(IngestionReport {
            rows_ingested: total_rows,
            columns_ingested: num_cols,
            cells_written,
            duration_ms: duration.as_secs_f64() * 1000.0,
            start_row: options.start_row,
            start_col: options.start_col,
            end_row: cur_row,
            end_col: options.start_col + num_cols as u32,
            schema,
        })
    }

    /// Ingests a JSON payload (Array of Objects, Array of Arrays, or JSONL) into the grid.
    pub fn ingest_json(
        json_data: &str,
        format: JsonFormat,
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let reader = JsonReader::new(format);
        let (schema, rows) = reader.read_str(json_data)?;
        Self::ingest_rows(&schema, &rows, grid, options)
    }

    /// Ingests a SQLite table into the grid.
    pub fn ingest_sqlite(
        sqlite_reader: &SqliteReader,
        table_name: &str,
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let (schema, rows) = sqlite_reader.read_table(table_name)?;
        Self::ingest_rows(&schema, &rows, grid, options)
    }

    /// Ingests an Apache Arrow RecordBatch into the grid.
    pub fn ingest_arrow(
        batch: &ArrowRecordBatch,
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let schema = batch.schema.to_engine_schema();
        let rows = batch.to_rows();
        Self::ingest_rows(&schema, &rows, grid, options)
    }

    /// Ingests a Parquet file buffer into the grid.
    pub fn ingest_parquet(
        parquet_bytes: Vec<u8>,
        grid: &mut CompressedSparsePageGrid,
        options: &IngestionOptions,
    ) -> ConnectorResult<IngestionReport> {
        let reader = ParquetReader::open_bytes(parquet_bytes)?;
        let rows = reader.read_all()?;
        Self::ingest_rows(reader.schema(), &rows, grid, options)
    }
}
