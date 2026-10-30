#![forbid(unsafe_code)]
//! High-Performance External Data Connectors & Fast Streaming Ingestion Engine.
//!
//! Provides zero-copy columnar ingestion for Delimited CSV/TSV, JSON/JSONL,
//! SQLite database tables, Apache Arrow RecordBatches, and Apache Parquet files
//! directly into `excel_engine::grid::CompressedSparsePageGrid`.
//!
//! Strictly conforms to GEC-v2.0:
//! - Pure safe Rust (`#[forbid(unsafe_code)]`).
//! - Zero panic macros (no unwrap, expect, todo, or unimplemented).
//! - Typed error handling via `ConnectorError`.
//! - High throughput ingestion SLA (< 200ms for 100k rows).

pub mod arrow;
pub mod csv;
pub mod error;
pub mod ingestion;
pub mod json;
pub mod parquet;
pub mod schema;
pub mod sqlite;
pub mod value;

pub use arrow::{ArrowColumn, ArrowDataType, ArrowField, ArrowRecordBatch, ArrowSchema};
pub use csv::{CsvReadOptions, CsvReader};
pub use error::{ConnectorError, ConnectorResult};
pub use ingestion::{IngestionOptions, IngestionPipeline, IngestionReport};
pub use json::{JsonFormat, JsonReader};
pub use parquet::{
    ParquetColumnDescriptor, ParquetEncoding, ParquetPhysicalType, ParquetReader, ParquetWriter,
};
pub use schema::{days_from_civil, DataType, Field, InferenceOptions, Schema, SchemaInferer};
pub use sqlite::{SqliteReader, SqliteTableEntry};
pub use value::DataValue;

/// Crate initialization and health check.
pub fn health_check() -> bool {
    true
}
