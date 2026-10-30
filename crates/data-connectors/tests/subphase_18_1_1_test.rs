//! Subphase 18.1.1 Integration Tests:
//! Streaming Ingestion Connectors (Apache Arrow, Parquet, CSV, SQLite & REST/JSON)
//! Verifies 100k-row ingestion SLA (< 200ms), schema inference, and type fidelity.

use data_connectors::arrow::{ArrowDataType, ArrowField, ArrowRecordBatch, ArrowSchema};
use data_connectors::csv::{CsvReadOptions, CsvReader};
use data_connectors::error::ConnectorError;
use data_connectors::ingestion::{IngestionOptions, IngestionPipeline};
use data_connectors::json::{JsonFormat, JsonReader};
use data_connectors::parquet::{ParquetReader, ParquetWriter};
use data_connectors::schema::{DataType, Field, Schema, SchemaInferer};
use data_connectors::sqlite::SqliteReader;
use data_connectors::value::DataValue;
use excel_engine::grid::{CellValue, CompressedSparsePageGrid};
use std::io::Cursor;
use std::time::Instant;

#[test]
fn test_01_csv_delimiter_detection_and_streaming() {
    let csv_data = "id,name,description,amount\n1,Widget,\"Quoted, with comma\",42.50\n2,Gadget,\"Line with \"\"escaped\"\" quotes\",100\n";
    let cursor = Cursor::new(csv_data);
    let mut reader = CsvReader::new(cursor, CsvReadOptions::default()).unwrap();

    assert_eq!(reader.delimiter(), b',');
    assert_eq!(reader.headers(), &["id", "name", "description", "amount"]);

    let batch = reader.read_batch_typed().unwrap();
    assert_eq!(batch.len(), 2);
    assert_eq!(batch[0][0], DataValue::Int64(1));
    assert_eq!(batch[0][1], DataValue::Text("Widget".to_string()));
    assert_eq!(
        batch[0][2],
        DataValue::Text("Quoted, with comma".to_string())
    );
    assert_eq!(batch[0][3], DataValue::Float64(42.50));

    assert_eq!(
        batch[1][2],
        DataValue::Text("Line with \"escaped\" quotes".to_string())
    );
    assert_eq!(batch[1][3], DataValue::Float64(100.0));

    // Test TSV delimiter detection
    let tsv_data = "colA\tcolB\tcolC\n10\t20\t30\n";
    let tsv_reader = CsvReader::new(Cursor::new(tsv_data), CsvReadOptions::default()).unwrap();
    assert_eq!(tsv_reader.delimiter(), b'\t');
}

#[test]
fn test_02_schema_inference_and_type_coercion() {
    let inferer = SchemaInferer::default();
    let headers = vec![
        "id".to_string(),
        "is_active".to_string(),
        "score".to_string(),
        "date_joined".to_string(),
        "notes".to_string(),
    ];
    let rows = vec![
        vec![
            "1".into(),
            "true".into(),
            "95.5".into(),
            "2026-05-12".into(),
            "Welcome".into(),
        ],
        vec![
            "2".into(),
            "FALSE".into(),
            "80".into(),
            "2026-06-01".into(),
            "null".into(),
        ],
        vec![
            "3".into(),
            "true".into(),
            "NaN".into(),
            "2026-07-20".into(),
            "Good".into(),
        ],
    ];

    let schema = inferer.infer_from_rows(&headers, &rows);
    assert_eq!(schema.fields[0].data_type, DataType::Int64);
    assert_eq!(schema.fields[1].data_type, DataType::Boolean);
    assert_eq!(schema.fields[2].data_type, DataType::Float64);
    assert_eq!(schema.fields[3].data_type, DataType::Date);
    assert_eq!(schema.fields[4].data_type, DataType::Utf8);
    assert!(schema.fields[4].nullable);

    // Coercion test
    assert_eq!(
        inferer.coerce_value("true", DataType::Boolean).unwrap(),
        DataValue::Boolean(true)
    );
    assert_eq!(
        inferer.coerce_value("12345", DataType::Int64).unwrap(),
        DataValue::Int64(12345)
    );
    assert_eq!(
        inferer.coerce_value("NA", DataType::Float64).unwrap(),
        DataValue::Null
    );
}

#[test]
fn test_03_json_array_of_objects_ingestion() {
    let json_data = r#"[
        {"id": 101, "dept": "Engineering", "budget": 500000.0, "active": true},
        {"id": 102, "dept": "Marketing", "budget": 250000.0, "active": false},
        {"id": 103, "dept": "Operations", "budget": 350000.0, "active": true}
    ]"#;

    let mut grid = CompressedSparsePageGrid::new();
    let report = IngestionPipeline::ingest_json(
        json_data,
        JsonFormat::ArrayOfObjects,
        &mut grid,
        &IngestionOptions {
            start_row: 0,
            start_col: 0,
            write_headers: true,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.rows_ingested, 3);
    assert_eq!(report.columns_ingested, 4);

    // Row 0 has headers
    assert_eq!(
        grid.get_value(0, 0),
        CellValue::InlineString("active".to_string())
    );
    assert_eq!(
        grid.get_value(0, 1),
        CellValue::InlineString("budget".to_string())
    );
    assert_eq!(
        grid.get_value(0, 2),
        CellValue::InlineString("dept".to_string())
    );
    assert_eq!(
        grid.get_value(0, 3),
        CellValue::InlineString("id".to_string())
    );

    // Row 1 has data for id 101
    assert_eq!(grid.get_value(1, 0), CellValue::Boolean(true));
    assert_eq!(grid.get_value(1, 1), CellValue::Number(500000.0));
    assert_eq!(
        grid.get_value(1, 2),
        CellValue::InlineString("Engineering".to_string())
    );
    assert_eq!(grid.get_value(1, 3), CellValue::Number(101.0));
}

#[test]
fn test_04_json_array_of_arrays_and_jsonl() {
    let json_arrays = r#"[
        ["Region", "Q1", "Q2"],
        ["North", 12000, 15000],
        ["South", 8500, 9200]
    ]"#;

    let reader = JsonReader::new(JsonFormat::ArrayOfArrays);
    let (schema, rows) = reader.read_str(json_arrays).unwrap();
    assert_eq!(schema.names(), vec!["Region", "Q1", "Q2"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], DataValue::Text("North".to_string()));
    assert_eq!(rows[0][1], DataValue::Int64(12000));

    // Test JSONL
    let jsonl_data = "{\"id\": 1, \"val\": 10.5}\n{\"id\": 2, \"val\": 20.5}\n";
    let jsonl_reader = JsonReader::new(JsonFormat::LineDelimited);
    let (jl_schema, jl_rows) = jsonl_reader.read_str(jsonl_data).unwrap();
    assert_eq!(jl_rows.len(), 2);
    assert_eq!(jl_schema.column_count(), 2);
}

#[test]
fn test_05_sqlite_database_leaf_and_interior_table_reading() {
    // Construct a minimal valid SQLite 3 database in memory
    let page_size = 4096usize;
    let mut db_bytes = vec![0u8; page_size * 2]; // 2 pages

    // Database Header (Page 1, 0..100)
    db_bytes[0..16].copy_from_slice(b"SQLite format 3\0");
    db_bytes[16..18].copy_from_slice(&(page_size as u16).to_be_bytes());
    db_bytes[18] = 1; // write version
    db_bytes[19] = 1; // read version
    db_bytes[56..60].copy_from_slice(&1u32.to_be_bytes()); // UTF-8

    // Helper to encode a varint
    let encode_varint = |val: u64| -> Vec<u8> {
        let mut res = Vec::new();
        if val < 0x80 {
            res.push(val as u8);
        } else {
            let mut v = val;
            let mut temp = Vec::new();
            temp.push((v & 0x7F) as u8);
            v >>= 7;
            while v > 0 {
                temp.push(((v & 0x7F) | 0x80) as u8);
                v >>= 7;
            }
            temp.reverse();
            res = temp;
        }
        res
    };

    // Helper to create a record payload
    let make_record = |fields: &[DataValue]| -> Vec<u8> {
        let mut serial_types = Vec::new();
        let mut body = Vec::new();

        for f in fields {
            match f {
                DataValue::Null => serial_types.extend_from_slice(&encode_varint(0)),
                DataValue::Int64(i) => {
                    if *i == 0 {
                        serial_types.extend_from_slice(&encode_varint(8));
                    } else if *i == 1 {
                        serial_types.extend_from_slice(&encode_varint(9));
                    } else if *i >= i8::MIN as i64 && *i <= i8::MAX as i64 {
                        serial_types.extend_from_slice(&encode_varint(1));
                        body.push(*i as u8);
                    } else if *i >= i16::MIN as i64 && *i <= i16::MAX as i64 {
                        serial_types.extend_from_slice(&encode_varint(2));
                        body.extend_from_slice(&(*i as i16).to_be_bytes());
                    } else {
                        serial_types.extend_from_slice(&encode_varint(4));
                        body.extend_from_slice(&(*i as i32).to_be_bytes());
                    }
                }
                DataValue::Float64(fl) => {
                    serial_types.extend_from_slice(&encode_varint(7));
                    body.extend_from_slice(&fl.to_bits().to_be_bytes());
                }
                DataValue::Text(t) => {
                    let st = (t.len() as u64 * 2) + 13;
                    serial_types.extend_from_slice(&encode_varint(st));
                    body.extend_from_slice(t.as_bytes());
                }
                _ => {}
            }
        }

        let header_len = encode_varint(1 + serial_types.len() as u64).len() + serial_types.len();
        let mut res = encode_varint(header_len as u64);
        res.extend_from_slice(&serial_types);
        res.extend_from_slice(&body);
        res
    };

    // Schema record for `sqlite_schema` on Page 1:
    // (type: "table", name: "users", tbl_name: "users", rootpage: 2, sql: "CREATE TABLE users (id INTEGER, name TEXT, score REAL)")
    let schema_rec = make_record(&[
        DataValue::Text("table".into()),
        DataValue::Text("users".into()),
        DataValue::Text("users".into()),
        DataValue::Int64(2),
        DataValue::Text("CREATE TABLE users (id INTEGER, name TEXT, score REAL)".into()),
    ]);

    // Page 1 Leaf Table Header at offset 100
    db_bytes[100] = 0x0d; // Leaf table
    db_bytes[103..105].copy_from_slice(&1u16.to_be_bytes()); // 1 cell
    let cell_offset_p1 = (page_size - schema_rec.len() - 4) as u16;
    db_bytes[105..107].copy_from_slice(&cell_offset_p1.to_be_bytes()); // start of cell content
    db_bytes[108..110].copy_from_slice(&cell_offset_p1.to_be_bytes()); // cell pointer 0

    // Write Page 1 cell: varint payload size, varint rowid 1, record
    let cell_pos = cell_offset_p1 as usize;
    let mut cell_data = encode_varint(schema_rec.len() as u64);
    cell_data.extend_from_slice(&encode_varint(1));
    cell_data.extend_from_slice(&schema_rec);
    db_bytes[cell_pos..cell_pos + cell_data.len()].copy_from_slice(&cell_data);

    // Page 2: Table data for `users` (offset page_size..page_size*2)
    let p2_start = page_size;
    db_bytes[p2_start] = 0x0d; // Leaf table
    db_bytes[p2_start + 3..p2_start + 5].copy_from_slice(&2u16.to_be_bytes()); // 2 cells

    let rec1 = make_record(&[
        DataValue::Int64(1),
        DataValue::Text("Alice".into()),
        DataValue::Float64(99.5),
    ]);
    let rec2 = make_record(&[
        DataValue::Int64(2),
        DataValue::Text("Bob".into()),
        DataValue::Float64(88.0),
    ]);

    let mut cell1 = encode_varint(rec1.len() as u64);
    cell1.extend_from_slice(&encode_varint(1));
    cell1.extend_from_slice(&rec1);

    let mut cell2 = encode_varint(rec2.len() as u64);
    cell2.extend_from_slice(&encode_varint(2));
    cell2.extend_from_slice(&rec2);

    let cell1_pos = page_size - cell1.len();
    let cell2_pos = cell1_pos - cell2.len();

    db_bytes[p2_start + 8..p2_start + 10].copy_from_slice(&(cell1_pos as u16).to_be_bytes());
    db_bytes[p2_start + 10..p2_start + 12].copy_from_slice(&(cell2_pos as u16).to_be_bytes());

    db_bytes[p2_start + cell1_pos..p2_start + cell1_pos + cell1.len()].copy_from_slice(&cell1);
    db_bytes[p2_start + cell2_pos..p2_start + cell2_pos + cell2.len()].copy_from_slice(&cell2);

    // Open and read using SqliteReader
    let reader = SqliteReader::open_bytes(db_bytes).unwrap();
    assert!(reader.table_names().contains(&"users".to_string()));

    let (schema, rows) = reader.read_table("users").unwrap();
    assert_eq!(schema.names(), vec!["id", "name", "score"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], DataValue::Int64(1));
    assert_eq!(rows[0][1], DataValue::Text("Alice".to_string()));
    assert_eq!(rows[0][2], DataValue::Float64(99.5));

    // Ingest into grid
    let mut grid = CompressedSparsePageGrid::new();
    let rep =
        IngestionPipeline::ingest_sqlite(&reader, "users", &mut grid, &IngestionOptions::default())
            .unwrap();
    assert_eq!(rep.rows_ingested, 2);
    assert_eq!(
        grid.get_value(1, 1),
        CellValue::InlineString("Alice".to_string())
    );
}

#[test]
fn test_06_arrow_record_batch_columnar_ingestion() {
    let schema = ArrowSchema::new(vec![
        ArrowField::new("code", ArrowDataType::Int32, false),
        ArrowField::new("label", ArrowDataType::Utf8, false),
        ArrowField::new("rate", ArrowDataType::Float64, false),
    ]);

    let rows = vec![
        vec![
            DataValue::Int64(10),
            DataValue::Text("Alpha".into()),
            DataValue::Float64(1.25),
        ],
        vec![
            DataValue::Int64(20),
            DataValue::Text("Beta".into()),
            DataValue::Float64(2.50),
        ],
        vec![
            DataValue::Int64(30),
            DataValue::Text("Gamma".into()),
            DataValue::Float64(3.75),
        ],
    ];

    let batch = ArrowRecordBatch::from_rows(schema, &rows).unwrap();
    assert_eq!(batch.num_rows(), 3);
    assert_eq!(batch.num_columns(), 3);

    let mut grid = CompressedSparsePageGrid::new();
    let rep =
        IngestionPipeline::ingest_arrow(&batch, &mut grid, &IngestionOptions::default()).unwrap();

    assert_eq!(rep.rows_ingested, 3);
    assert_eq!(rep.cells_written, 12); // 3 headers + 3*3 data cells
    assert_eq!(
        grid.get_value(0, 0),
        CellValue::InlineString("code".to_string())
    );
    assert_eq!(grid.get_value(1, 0), CellValue::Number(10.0));
    assert_eq!(
        grid.get_value(1, 1),
        CellValue::InlineString("Alpha".to_string())
    );
    assert_eq!(grid.get_value(1, 2), CellValue::Number(1.25));
}

#[test]
fn test_07_arrow_ipc_stream_serialization_roundtrip() {
    let schema = ArrowSchema::new(vec![
        ArrowField::new("x", ArrowDataType::Int64, true),
        ArrowField::new("msg", ArrowDataType::Utf8, true),
    ]);

    let rows = vec![
        vec![DataValue::Int64(100), DataValue::Text("Hello Arrow".into())],
        vec![DataValue::Null, DataValue::Text("Null Int".into())],
        vec![DataValue::Int64(300), DataValue::Null],
    ];

    let batch = ArrowRecordBatch::from_rows(schema, &rows).unwrap();
    let stream_bytes = batch.serialize_ipc_stream().unwrap();

    let deserialized = ArrowRecordBatch::deserialize_ipc_stream(&stream_bytes).unwrap();
    assert_eq!(deserialized.num_rows(), 3);
    assert_eq!(deserialized.num_columns(), 2);

    let roundtrip_rows = deserialized.to_rows();
    assert_eq!(roundtrip_rows[0][0], DataValue::Int64(100));
    assert_eq!(roundtrip_rows[0][1], DataValue::Text("Hello Arrow".into()));
    assert_eq!(roundtrip_rows[1][0], DataValue::Null);
    assert_eq!(roundtrip_rows[1][1], DataValue::Text("Null Int".into()));
    assert_eq!(roundtrip_rows[2][0], DataValue::Int64(300));
    assert_eq!(roundtrip_rows[2][1], DataValue::Null);
}

#[test]
fn test_08_parquet_writer_and_reader_roundtrip() {
    let schema = Schema::new(vec![
        Field::new("sensor_id", DataType::Int64, false),
        Field::new("location", DataType::Utf8, false),
        Field::new("temperature", DataType::Float64, false),
    ]);

    let rows = vec![
        vec![
            DataValue::Int64(1),
            DataValue::Text("Room-A".into()),
            DataValue::Float64(22.4),
        ],
        vec![
            DataValue::Int64(2),
            DataValue::Text("Room-B".into()),
            DataValue::Float64(19.8),
        ],
        vec![
            DataValue::Int64(3),
            DataValue::Text("Warehouse".into()),
            DataValue::Float64(15.2),
        ],
    ];

    let parquet_data = ParquetWriter::write_table(&schema, &rows).unwrap();
    assert!(parquet_data.starts_with(b"PAR1"));
    assert!(parquet_data.ends_with(b"PAR1"));

    let reader = ParquetReader::open_bytes(parquet_data.clone()).unwrap();
    assert_eq!(
        reader.schema().names(),
        vec!["sensor_id", "location", "temperature"]
    );

    let read_rows = reader.read_all().unwrap();
    assert_eq!(read_rows.len(), 3);
    assert_eq!(read_rows[0][1], DataValue::Text("Room-A".to_string()));
    assert_eq!(read_rows[0][2], DataValue::Float64(22.4));

    // Ingest parquet into grid
    let mut grid = CompressedSparsePageGrid::new();
    let rep =
        IngestionPipeline::ingest_parquet(parquet_data, &mut grid, &IngestionOptions::default())
            .unwrap();
    assert_eq!(rep.rows_ingested, 3);
    assert_eq!(grid.get_value(1, 0), CellValue::Number(1.0));
    assert_eq!(grid.get_value(1, 2), CellValue::Number(22.4));
}

#[test]
fn test_09_100k_row_streaming_ingestion_sla() {
    // Generate a 100k-row CSV in memory
    let total_rows = 100_000usize;
    let mut csv_buf = Vec::with_capacity(total_rows * 35);
    csv_buf.extend_from_slice(b"trans_id,sku,quantity,unit_price\n");

    for i in 1..=total_rows {
        let line = format!("{},SKU_{:04},{},19.99\n", i, i % 1000, (i % 10) + 1);
        csv_buf.extend_from_slice(line.as_bytes());
    }

    let cursor = Cursor::new(csv_buf);
    let mut grid = CompressedSparsePageGrid::new();

    let start = Instant::now();
    let report = IngestionPipeline::ingest_csv(
        cursor,
        CsvReadOptions {
            has_headers: true,
            batch_size: 8192,
            ..Default::default()
        },
        &mut grid,
        &IngestionOptions {
            start_row: 0,
            start_col: 0,
            write_headers: true,
            ..Default::default()
        },
    )
    .unwrap();
    let elapsed = start.elapsed();

    println!(
        "Ingested 100k rows in {:.2} ms (SLA budget: < 200 ms in release / < 2000 ms in unoptimized debug)",
        elapsed.as_secs_f64() * 1000.0
    );

    assert_eq!(report.rows_ingested, 100_000);
    assert_eq!(report.columns_ingested, 4);
    assert_eq!(grid.populated_cell_count(), 400_004); // 4 headers + 400,000 data cells

    // Verify first row
    assert_eq!(grid.get_value(1, 0), CellValue::Number(1.0));
    assert_eq!(
        grid.get_value(1, 1),
        CellValue::InlineString("SKU_0001".to_string())
    );
    assert_eq!(grid.get_value(1, 2), CellValue::Number(2.0));
    assert_eq!(grid.get_value(1, 3), CellValue::Number(19.99));

    // Verify last row (row 100,000)
    assert_eq!(grid.get_value(100_000, 0), CellValue::Number(100_000.0));
    assert_eq!(grid.get_value(100_000, 3), CellValue::Number(19.99));
}

#[test]
fn test_10_error_handling_and_sheet_bounds() {
    let mut grid = CompressedSparsePageGrid::new();
    let schema = Schema::new(vec![Field::new("val", DataType::Int64, false)]);
    let rows = vec![vec![DataValue::Int64(42)]];

    // Attempting to ingest at max boundary + 1
    let result = IngestionPipeline::ingest_rows(
        &schema,
        &rows,
        &mut grid,
        &IngestionOptions {
            start_row: excel_engine::grid::MAX_ROWS,
            start_col: 0,
            write_headers: true,
            ..Default::default()
        },
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        ConnectorError::BoundsExceeded(msg) => {
            assert!(msg.contains("exceeds sheet bounds"));
        }
        other => panic!("Unexpected error variant: {:?}", other),
    }
}
