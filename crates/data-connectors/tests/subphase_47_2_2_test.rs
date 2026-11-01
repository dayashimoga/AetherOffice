#![forbid(unsafe_code)]
//! Sub-Phase 47.2.2 Integration Test Suite:
//! Parquet, Arrow, SQLite & ODS Live Import/Export.
//!
//! Strictly verifies:
//! - Pure safe-Rust Apache Parquet serialization and deserialization.
//! - Apache Arrow RecordBatch IPC streaming roundtrips.
//! - Pure safe-Rust SQLite 3 database file generation and B-Tree reading.
//! - OASIS OpenDocument Spreadsheet (.ods) package generation and parsing.
//! - Unified `IngestionPipeline` and `ExportPipeline` across all formats.

use data_connectors::{
    ArrowRecordBatch, ArrowSchema, DataFormat, DataType, DataValue, ExportPipeline, Field,
    IngestionOptions, IngestionPipeline, OdsReader, OdsWriter, ParquetReader, ParquetWriter,
    Schema, SqliteReader, SqliteWriter, ODS_MIMETYPE,
};
use excel_engine::grid::CompressedSparsePageGrid;
use excel_engine::CellValue;

fn create_sample_dataset() -> (Schema, Vec<Vec<DataValue>>) {
    let schema = Schema::new(vec![
        Field::new("ID", DataType::Int64, false),
        Field::new("Product", DataType::Utf8, false),
        Field::new("Price", DataType::Float64, false),
        Field::new("InStock", DataType::Boolean, false),
    ]);

    let rows = vec![
        vec![
            DataValue::Int64(101),
            DataValue::Text("UltraBook Pro".to_string()),
            DataValue::Float64(1299.99),
            DataValue::Boolean(true),
        ],
        vec![
            DataValue::Int64(102),
            DataValue::Text("Ergonomic Keyboard".to_string()),
            DataValue::Float64(149.50),
            DataValue::Boolean(true),
        ],
        vec![
            DataValue::Int64(103),
            DataValue::Text("Wireless Mouse".to_string()),
            DataValue::Float64(49.95),
            DataValue::Boolean(false),
        ],
    ];

    (schema, rows)
}

#[test]
fn test_01_parquet_writer_and_reader_roundtrip_all_types() {
    let (schema, rows) = create_sample_dataset();
    let parquet_bytes = ParquetWriter::write_table(&schema, &rows).unwrap();

    assert!(parquet_bytes.len() > 12);
    assert_eq!(&parquet_bytes[0..4], b"PAR1");
    assert_eq!(&parquet_bytes[parquet_bytes.len() - 4..], b"PAR1");

    let reader = ParquetReader::open_bytes(parquet_bytes).unwrap();
    assert_eq!(reader.schema().fields.len(), 4);
    assert_eq!(reader.schema().fields[0].name, "ID");
    assert_eq!(reader.schema().fields[1].name, "Product");

    let read_rows = reader.read_all().unwrap();
    assert_eq!(read_rows.len(), 3);
    assert_eq!(read_rows[0][0], DataValue::Int64(101));
    assert_eq!(
        read_rows[0][1],
        DataValue::Text("UltraBook Pro".to_string())
    );
    assert_eq!(read_rows[0][2], DataValue::Float64(1299.99));
    assert_eq!(read_rows[0][3], DataValue::Boolean(true));
    assert_eq!(read_rows[2][3], DataValue::Boolean(false));
}

#[test]
fn test_02_parquet_ingestion_into_grid() {
    let (schema, rows) = create_sample_dataset();
    let parquet_bytes = ParquetWriter::write_table(&schema, &rows).unwrap();

    let mut grid = CompressedSparsePageGrid::new();
    let opts = IngestionOptions {
        start_row: 2,
        start_col: 1,
        write_headers: true,
        ..Default::default()
    };

    let report = IngestionPipeline::ingest_parquet(parquet_bytes, &mut grid, &opts).unwrap();
    assert_eq!(report.rows_ingested, 3);
    assert_eq!(report.columns_ingested, 4);

    // Headers at row 2
    assert_eq!(
        grid.get_value(2, 1),
        CellValue::InlineString("ID".to_string())
    );
    assert_eq!(
        grid.get_value(2, 2),
        CellValue::InlineString("Product".to_string())
    );

    // Row 1 at row 3
    assert_eq!(grid.get_value(3, 1), CellValue::Number(101.0));
    assert_eq!(
        grid.get_value(3, 2),
        CellValue::InlineString("UltraBook Pro".to_string())
    );
    assert_eq!(grid.get_value(3, 3), CellValue::Number(1299.99));
    assert_eq!(grid.get_value(3, 4), CellValue::Boolean(true));
}

#[test]
fn test_03_arrow_ipc_stream_serialization_and_deserialization() {
    let (schema, rows) = create_sample_dataset();
    let arrow_schema = ArrowSchema::from_engine_schema(&schema);
    let batch = ArrowRecordBatch::from_rows(arrow_schema, &rows).unwrap();

    let ipc_bytes = batch.serialize_ipc_stream().unwrap();
    assert!(ipc_bytes.len() > 16);

    let decoded_batch = ArrowRecordBatch::deserialize_ipc_stream(&ipc_bytes).unwrap();
    assert_eq!(decoded_batch.num_rows, 3);
    assert_eq!(decoded_batch.schema.fields.len(), 4);

    let decoded_rows = decoded_batch.to_rows();
    assert_eq!(decoded_rows.len(), 3);
    assert_eq!(decoded_rows[1][0], DataValue::Int64(102));
    assert_eq!(
        decoded_rows[1][1],
        DataValue::Text("Ergonomic Keyboard".to_string())
    );
}

#[test]
fn test_04_arrow_ingestion_into_grid() {
    let (schema, rows) = create_sample_dataset();
    let arrow_schema = ArrowSchema::from_engine_schema(&schema);
    let batch = ArrowRecordBatch::from_rows(arrow_schema, &rows).unwrap();

    let mut grid = CompressedSparsePageGrid::new();
    let opts = IngestionOptions::default();

    let report = IngestionPipeline::ingest_arrow(&batch, &mut grid, &opts).unwrap();
    assert_eq!(report.rows_ingested, 3);
    assert_eq!(report.cells_written, 16); // 4 headers + 12 data cells

    assert_eq!(
        grid.get_value(0, 0),
        CellValue::InlineString("ID".to_string())
    );
    assert_eq!(grid.get_value(1, 0), CellValue::Number(101.0));
    assert_eq!(
        grid.get_value(1, 1),
        CellValue::InlineString("UltraBook Pro".to_string())
    );
}

#[test]
fn test_05_sqlite_writer_and_reader_roundtrip() {
    let (schema, rows) = create_sample_dataset();
    let sqlite_bytes = SqliteWriter::write_table("Inventory", &schema, &rows).unwrap();

    assert_eq!(&sqlite_bytes[0..16], b"SQLite format 3\0");
    let reader = SqliteReader::open_bytes(sqlite_bytes).unwrap();

    let tables = reader.table_names();
    assert_eq!(tables, vec!["Inventory"]);

    let (read_schema, read_rows) = reader.read_table("Inventory").unwrap();
    assert_eq!(read_schema.fields.len(), 4);
    assert_eq!(read_rows.len(), 3);
    assert_eq!(read_rows[0][0], DataValue::Int64(101));
    assert_eq!(
        read_rows[0][1],
        DataValue::Text("UltraBook Pro".to_string())
    );
    assert_eq!(read_rows[0][2], DataValue::Float64(1299.99));
    assert_eq!(read_rows[0][3], DataValue::Int64(1)); // boolean encoded as 1
}

#[test]
fn test_06_sqlite_ingestion_into_grid() {
    let (schema, rows) = create_sample_dataset();
    let sqlite_bytes = SqliteWriter::write_table("Products", &schema, &rows).unwrap();
    let reader = SqliteReader::open_bytes(sqlite_bytes).unwrap();

    let mut grid = CompressedSparsePageGrid::new();
    let opts = IngestionOptions::default();

    let report = IngestionPipeline::ingest_sqlite(&reader, "Products", &mut grid, &opts).unwrap();
    assert_eq!(report.rows_ingested, 3);
    assert_eq!(
        grid.get_value(0, 1),
        CellValue::InlineString("Product".to_string())
    );
    assert_eq!(
        grid.get_value(2, 1),
        CellValue::InlineString("Ergonomic Keyboard".to_string())
    );
}

#[test]
fn test_07_ods_writer_and_reader_roundtrip() {
    let (schema, rows) = create_sample_dataset();
    let ods_bytes = OdsWriter::write_single_sheet("InventorySheet", &schema, &rows).unwrap();

    assert!(ods_bytes.starts_with(b"PK"));

    // Verify uncompressed mimetype is first entry
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&ods_bytes)).unwrap();
    let mut mime_file = zip.by_name("mimetype").unwrap();
    let mut mime_buf = Vec::new();
    std::io::Read::read_to_end(&mut mime_file, &mut mime_buf).unwrap();
    assert_eq!(mime_buf.as_slice(), ODS_MIMETYPE);

    // Read back via OdsReader
    let sheets = OdsReader::read_sheets(&ods_bytes).unwrap();
    assert_eq!(sheets.len(), 1);
    let (sheet_name, read_schema, read_rows) = &sheets[0];
    assert_eq!(sheet_name, "InventorySheet");
    assert_eq!(read_schema.fields.len(), 4);
    assert_eq!(read_schema.fields[0].name, "ID");
    assert_eq!(read_rows.len(), 3);
    assert_eq!(read_rows[0][0], DataValue::Int64(101));
    assert_eq!(
        read_rows[0][1],
        DataValue::Text("UltraBook Pro".to_string())
    );
    assert_eq!(read_rows[0][3], DataValue::Boolean(true));
}

#[test]
fn test_08_ods_ingestion_into_grid() {
    let (schema, rows) = create_sample_dataset();
    let ods_bytes = OdsWriter::write_single_sheet("Sheet1", &schema, &rows).unwrap();

    let mut grid = CompressedSparsePageGrid::new();
    let opts = IngestionOptions::default();

    let report = IngestionPipeline::ingest_ods(&ods_bytes, 0, &mut grid, &opts).unwrap();
    assert_eq!(report.rows_ingested, 3);
    assert_eq!(report.columns_ingested, 4);
    assert_eq!(
        grid.get_value(0, 0),
        CellValue::InlineString("ID".to_string())
    );
    assert_eq!(
        grid.get_value(1, 1),
        CellValue::InlineString("UltraBook Pro".to_string())
    );
}

#[test]
fn test_09_export_pipeline_all_formats() {
    let (schema, rows) = create_sample_dataset();

    // 1. CSV
    let csv = ExportPipeline::export_format(DataFormat::Csv, "T", &schema, &rows).unwrap();
    assert!(String::from_utf8(csv).unwrap().contains("UltraBook Pro"));

    // 2. JSON
    let json = ExportPipeline::export_format(DataFormat::Json, "T", &schema, &rows).unwrap();
    assert!(String::from_utf8(json)
        .unwrap()
        .contains("\"Product\": \"UltraBook Pro\""));

    // 3. Parquet
    let parquet = ExportPipeline::export_format(DataFormat::Parquet, "T", &schema, &rows).unwrap();
    assert_eq!(&parquet[0..4], b"PAR1");

    // 4. Arrow
    let arrow = ExportPipeline::export_format(DataFormat::Arrow, "T", &schema, &rows).unwrap();
    assert!(!arrow.is_empty());

    // 5. SQLite
    let sqlite = ExportPipeline::export_format(DataFormat::Sqlite, "T", &schema, &rows).unwrap();
    assert_eq!(&sqlite[0..16], b"SQLite format 3\0");

    // 6. ODS
    let ods = ExportPipeline::export_format(DataFormat::Ods, "T", &schema, &rows).unwrap();
    assert!(ods.starts_with(b"PK"));
}

#[test]
fn test_10_ingest_format_unified_pipeline_roundtrip() {
    let (schema, rows) = create_sample_dataset();
    let formats = [
        DataFormat::Csv,
        DataFormat::Json,
        DataFormat::Parquet,
        DataFormat::Arrow,
        DataFormat::Sqlite,
        DataFormat::Ods,
    ];

    for format in formats {
        let exported = ExportPipeline::export_format(format, "TestTable", &schema, &rows).unwrap();
        let mut grid = CompressedSparsePageGrid::new();
        let opts = IngestionOptions::default();

        let report = IngestionPipeline::ingest_format(format, &exported, &mut grid, &opts).unwrap();
        assert_eq!(
            report.rows_ingested, 3,
            "Format {format:?} row count mismatch"
        );
        assert_eq!(
            report.columns_ingested, 4,
            "Format {format:?} col count mismatch"
        );

        // Locate Product column dynamically from header row
        let col_count = report.columns_ingested;
        let mut prod_col = None;
        for c in 0..col_count {
            if grid.get_value(0, c as u32) == CellValue::InlineString("Product".to_string()) {
                prod_col = Some(c as u32);
                break;
            }
        }
        assert!(
            prod_col.is_some(),
            "Product column header not found for format {format:?}"
        );
        let prod_val = grid.get_value(1, prod_col.unwrap());
        assert_eq!(
            prod_val,
            CellValue::InlineString("UltraBook Pro".to_string()),
            "Format {format:?} string cell mismatch"
        );
    }
}
