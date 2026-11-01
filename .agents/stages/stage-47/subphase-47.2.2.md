# Sub-Phase 47.2.2: Parquet, Arrow, SQLite & ODS Live Import/Export

## Objective
Implement industrial-grade, two-way streaming Import and Export for Apache Parquet, Apache Arrow RecordBatch / IPC stream, SQLite database tables, and OASIS OpenDocument Spreadsheet (.ods) packages in `data-connectors` and the unified presentation shell. Ensure zero data loss, high throughput (<200ms per 100k rows), clean-room provenance, and zero-panic safety invariants.

## Scope of Features

### 1. Apache Parquet Live Import & Export (`crates/data-connectors/src/parquet.rs`)
- **Parquet Ingestion (`ParquetReader`):**
  - Reads `PAR1` container headers, metadata footers, and schema descriptors.
  - Decodes PLAIN and DICTIONARY encoded column chunks.
  - Maps Parquet physical types (`Boolean`, `Int32`, `Int64`, `Float`, `Double`, `ByteArray`) to `DataValue` and `excel_engine::grid::CompressedSparsePageGrid`.
- **Parquet Serialization (`ParquetWriter`):**
  - Serializes spreadsheet rows / table datasets to valid `PAR1` container format.
  - Encodes metadata footer, schema descriptors, and column data pages.

### 2. Apache Arrow IPC Stream & RecordBatch Ingestion (`crates/data-connectors/src/arrow.rs`)
- **Arrow IPC Ingestion (`ArrowRecordBatch` / `deserialize_ipc_stream`):**
  - Decodes Arrow IPC stream messages, schema definitions, validity bitmaps, offsets, and value buffers.
  - Streaming columnar mapping into sparse page chunks.
- **Arrow IPC Serialization (`serialize_ipc_stream`):**
  - Converts spreadsheet grid tables into strongly-typed `ArrowRecordBatch` and serializes to standard Arrow IPC stream byte format.

### 3. Pure Safe-Rust SQLite Table Import & Export (`crates/data-connectors/src/sqlite.rs`)
- **SQLite Database Ingestion (`SqliteReader`):**
  - Decodes standard SQLite 3 B-Tree pages (interior `0x05` and leaf `0x0d`), varints, and record payloads without C/FFI dependencies.
  - Discovers tables from `sqlite_master` / `sqlite_schema`.
- **SQLite Database Export (`SqliteWriter`):**
  - Generates valid SQLite 3 database files (`SQLite format 3\0` 100-byte header, page size 4096).
  - Encodes `sqlite_master` table entry on root page 1.
  - Encodes table data rows into B-tree leaf page cells (varint payload sizes, rowids, record headers, serial types).

### 4. OASIS OpenDocument Spreadsheet (.ods) Codec (`crates/data-connectors/src/ods.rs`)
- **ODS Ingestion (`OdsReader`):**
  - Unzips `.ods` package, validates `mimetype` (`application/vnd.oasis.opendocument.spreadsheet`), and parses `content.xml`.
  - Extracts `<table:table>`, `<table:table-row>`, and `<table:table-cell>` elements.
  - Parses cell types (`float`, `string`, `boolean`, `date`), cell formulas (`table:formula="of:=..."`), and repeated columns (`table:number-columns-repeated`).
- **ODS Serialization (`OdsWriter`):**
  - Generates compliant OASIS ODF 1.3 `.ods` packages (uncompressed `mimetype`, `META-INF/manifest.xml`, structured `content.xml`).
  - Serializes sheet names, grid cell text, numbers, booleans, and formulas.

### 5. Unified Ingestion & Export Pipelines (`crates/data-connectors/src/ingestion.rs`)
- **Unified Pipeline API:**
  - `DataFormat`: `Parquet`, `Arrow`, `Sqlite`, `Ods`, `Csv`, `Json`.
  - `IngestionPipeline::ingest_format`: parses any supported format bytes directly into `CompressedSparsePageGrid`.
  - `ExportPipeline::export_format`: exports tabular data or grid ranges into byte buffers for the chosen `DataFormat`.

### 6. Desktop Tauri IPC & Presentation Shell Coordinator
- **Tauri Commands (`apps/unified-shell/src-tauri/src/binary_commands.rs`):**
  - `data_connector_import(format, bytes, options_json)` -> `Result<IngestionReport, String>`.
  - `data_connector_export(format, payload_json)` -> `Result<Vec<u8>, String>`.
- **Frontend Coordinator (`apps/unified-shell/src/state/CalcDataConnectorCoordinator.ts`):**
  - Unified manager for Parquet, Arrow, SQLite, and ODS import/export in the Calc presentation shell.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all Rust crates.
- Zero panics, unwrap, or expect in non-test library code.
- Clean-room implementation of ISO/IEC open document and data standards.
- 100% test pass rate across Rust workspace and frontend Vitest suites.

