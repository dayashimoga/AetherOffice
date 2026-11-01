//! Pure safe-Rust SQLite database reader and table parser.
//! Decodes standard SQLite 3 B-Tree pages, varints, and record payloads
//! without requiring any unsafe C libraries or external bindings.

use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Field, Schema};
use crate::value::DataValue;
use std::collections::HashMap;

/// SQLite table metadata entry discovered in `sqlite_schema`.
#[derive(Debug, Clone)]
pub struct SqliteTableEntry {
    pub name: String,
    pub root_page: u32,
    pub sql: String,
    pub columns: Vec<String>,
}

/// Pure safe-Rust SQLite database reader.
pub struct SqliteReader {
    data: Vec<u8>,
    page_size: usize,
    tables: HashMap<String, SqliteTableEntry>,
}

impl SqliteReader {
    /// Opens and parses a SQLite 3 database from byte data.
    pub fn open_bytes(data: Vec<u8>) -> ConnectorResult<Self> {
        if data.len() < 100 {
            return Err(ConnectorError::Sqlite(
                "File too small for SQLite header (< 100 bytes)".to_string(),
            ));
        }

        // Verify magic string "SQLite format 3\0"
        let magic = &data[0..16];
        if magic != b"SQLite format 3\0" {
            return Err(ConnectorError::Sqlite(
                "Invalid SQLite magic header".to_string(),
            ));
        }

        let raw_page_size = u16::from_be_bytes([data[16], data[17]]);
        let page_size = if raw_page_size == 1 {
            65536
        } else {
            raw_page_size as usize
        };

        if page_size < 512 || !page_size.is_power_of_two() {
            return Err(ConnectorError::Sqlite(format!(
                "Invalid SQLite page size: {page_size}"
            )));
        }

        let mut reader = Self {
            data,
            page_size,
            tables: HashMap::new(),
        };

        reader.load_schema_catalog()?;
        Ok(reader)
    }

    /// Returns the detected page size.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Lists table names present in the database.
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Retrieves metadata for a specific table.
    pub fn get_table_entry(&self, table_name: &str) -> Option<&SqliteTableEntry> {
        self.tables.get(table_name)
    }

    /// Reads all records from the named table.
    pub fn read_table(&self, table_name: &str) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let entry = self.tables.get(table_name).ok_or_else(|| {
            ConnectorError::Sqlite(format!("Table '{table_name}' not found in database"))
        })?;

        let rows = self.read_records_from_root_page(entry.root_page)?;

        // Build schema
        let col_count = if !entry.columns.is_empty() {
            entry.columns.len()
        } else {
            rows.first().map(|r| r.len()).unwrap_or(0)
        };

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

        let fields = (0..col_count)
            .map(|i| {
                let name = entry
                    .columns
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("Column_{}", i + 1));
                let dtype = if col_types[i] == DataType::Null {
                    DataType::Utf8
                } else {
                    col_types[i]
                };
                Field::new(name, dtype, col_nullable[i])
            })
            .collect();

        Ok((Schema::new(fields), rows))
    }

    /// Reads schema catalog from Root Page 1 (`sqlite_schema`).
    fn load_schema_catalog(&mut self) -> ConnectorResult<()> {
        let master_rows = self.read_records_from_page(1, true)?;

        for row in master_rows {
            // Columns: 0: type, 1: name, 2: tbl_name, 3: rootpage, 4: sql
            if row.len() >= 5 {
                let entry_type = row[0].as_str().unwrap_or("");
                let name = row[1].as_str().unwrap_or("").to_string();
                let root_page = row[3].as_i64().unwrap_or(0) as u32;
                let sql = row[4].as_str().unwrap_or("").to_string();

                if entry_type == "table" && !name.starts_with("sqlite_") && root_page > 0 {
                    let columns = Self::parse_columns_from_sql(&sql);
                    self.tables.insert(
                        name.clone(),
                        SqliteTableEntry {
                            name,
                            root_page,
                            sql,
                            columns,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Parses column names from a CREATE TABLE SQL statement.
    pub fn parse_columns_from_sql(sql: &str) -> Vec<String> {
        let mut cols = Vec::new();
        if let Some(start) = sql.find('(') {
            if let Some(end) = sql.rfind(')') {
                if end > start {
                    let inner = &sql[start + 1..end];
                    for def in inner.split(',') {
                        let trimmed = def.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let upper = trimmed.to_ascii_uppercase();
                        if upper.starts_with("PRIMARY KEY")
                            || upper.starts_with("FOREIGN KEY")
                            || upper.starts_with("CONSTRAINT")
                            || upper.starts_with("CHECK")
                            || upper.starts_with("UNIQUE")
                        {
                            continue;
                        }
                        if let Some(first_word) = trimmed.split_whitespace().next() {
                            let clean_name = first_word
                                .trim_matches(|c| c == '`' || c == '"' || c == '[' || c == ']');
                            if !clean_name.is_empty() {
                                cols.push(clean_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        cols
    }

    /// Recursively traverses a B-Tree from `root_page` collecting all records.
    fn read_records_from_root_page(&self, root_page: u32) -> ConnectorResult<Vec<Vec<DataValue>>> {
        let mut all_records = Vec::new();
        let mut page_stack = vec![root_page];

        while let Some(page_num) = page_stack.pop() {
            let is_page_one = page_num == 1;
            let (page_type, _cell_count, cell_pointers, right_child) =
                self.parse_page_header(page_num, is_page_one)?;

            match page_type {
                0x0d => {
                    // Table Leaf Page
                    let page_start = (page_num as usize - 1) * self.page_size;
                    let page_data = &self.data[page_start..page_start + self.page_size];

                    for &ptr in &cell_pointers {
                        let cell_offset = ptr as usize;
                        if cell_offset < page_data.len() {
                            if let Ok(rec) = Self::parse_leaf_cell(&page_data[cell_offset..]) {
                                all_records.push(rec);
                            }
                        }
                    }
                }
                0x05 => {
                    // Table Interior Page
                    let page_start = (page_num as usize - 1) * self.page_size;
                    let page_data = &self.data[page_start..page_start + self.page_size];

                    for &ptr in &cell_pointers {
                        let cell_offset = ptr as usize;
                        if cell_offset + 4 <= page_data.len() {
                            let left_child = u32::from_be_bytes([
                                page_data[cell_offset],
                                page_data[cell_offset + 1],
                                page_data[cell_offset + 2],
                                page_data[cell_offset + 3],
                            ]);
                            page_stack.push(left_child);
                        }
                    }
                    if let Some(rchild) = right_child {
                        page_stack.push(rchild);
                    }
                }
                _ => {
                    // Other page type (e.g. index page), skip
                }
            }
        }

        Ok(all_records)
    }

    fn read_records_from_page(
        &self,
        page_num: u32,
        is_page_one: bool,
    ) -> ConnectorResult<Vec<Vec<DataValue>>> {
        let (page_type, _cell_count, cell_pointers, _) =
            self.parse_page_header(page_num, is_page_one)?;

        if page_type != 0x0d {
            return Ok(Vec::new());
        }

        let page_start = (page_num as usize - 1) * self.page_size;
        let page_data = &self.data[page_start..page_start + self.page_size];
        let mut records = Vec::with_capacity(cell_pointers.len());

        for &ptr in &cell_pointers {
            let cell_offset = ptr as usize;
            if cell_offset < page_data.len() {
                if let Ok(rec) = Self::parse_leaf_cell(&page_data[cell_offset..]) {
                    records.push(rec);
                }
            }
        }

        Ok(records)
    }

    /// Parses page header, returning `(page_type, cell_count, cell_pointers, right_child)`.
    fn parse_page_header(
        &self,
        page_num: u32,
        is_page_one: bool,
    ) -> ConnectorResult<(u8, u16, Vec<u16>, Option<u32>)> {
        let page_start = (page_num as usize - 1) * self.page_size;
        if page_start + self.page_size > self.data.len() {
            return Err(ConnectorError::Sqlite(format!(
                "Page {page_num} out of bounds"
            )));
        }

        let page_data = &self.data[page_start..page_start + self.page_size];
        let header_offset = if is_page_one { 100 } else { 0 };

        if header_offset + 8 > page_data.len() {
            return Err(ConnectorError::Sqlite("Corrupt page header".to_string()));
        }

        let page_type = page_data[header_offset];
        let cell_count =
            u16::from_be_bytes([page_data[header_offset + 3], page_data[header_offset + 4]]);

        let (cell_ptr_offset, right_child) = if page_type == 0x05 || page_type == 0x02 {
            let rchild = u32::from_be_bytes([
                page_data[header_offset + 8],
                page_data[header_offset + 9],
                page_data[header_offset + 10],
                page_data[header_offset + 11],
            ]);
            (header_offset + 12, Some(rchild))
        } else {
            (header_offset + 8, None)
        };

        let mut pointers = Vec::with_capacity(cell_count as usize);
        for i in 0..cell_count as usize {
            let ptr_idx = cell_ptr_offset + i * 2;
            if ptr_idx + 2 <= page_data.len() {
                let ptr = u16::from_be_bytes([page_data[ptr_idx], page_data[ptr_idx + 1]]);
                pointers.push(ptr);
            }
        }

        Ok((page_type, cell_count, pointers, right_child))
    }

    /// Decodes a table leaf cell into a row of `DataValue`.
    fn parse_leaf_cell(cell_data: &[u8]) -> ConnectorResult<Vec<DataValue>> {
        let (payload_size, bytes1) = Self::read_varint(cell_data)?;
        let (_rowid, bytes2) = Self::read_varint(&cell_data[bytes1..])?;
        let payload_offset = bytes1 + bytes2;

        if payload_offset > cell_data.len() {
            return Err(ConnectorError::Sqlite(
                "Cell payload offset out of bounds".to_string(),
            ));
        }

        let available_payload = cell_data.len() - payload_offset;
        let actual_len = (payload_size as usize).min(available_payload);
        let payload = &cell_data[payload_offset..payload_offset + actual_len];

        Self::parse_record(payload)
    }

    /// Decodes a SQLite record payload.
    pub fn parse_record(payload: &[u8]) -> ConnectorResult<Vec<DataValue>> {
        if payload.is_empty() {
            return Ok(Vec::new());
        }

        let (header_len, mut offset) = Self::read_varint(payload)?;
        let header_len = header_len as usize;

        if header_len > payload.len() {
            return Err(ConnectorError::Sqlite(
                "Record header length exceeds payload".to_string(),
            ));
        }

        // Read serial types
        let mut serial_types = Vec::new();
        while offset < header_len {
            let (st, read_bytes) = Self::read_varint(&payload[offset..])?;
            serial_types.push(st);
            offset += read_bytes;
        }

        // Read column values from payload body
        let mut body_offset = header_len;
        let mut values = Vec::with_capacity(serial_types.len());

        for st in serial_types {
            let (val, len) = Self::decode_serial_value(st, &payload[body_offset..])?;
            body_offset += len;
            values.push(val);
        }

        Ok(values)
    }

    /// Decodes a SQLite varint (1 to 9 bytes).
    pub fn read_varint(buf: &[u8]) -> ConnectorResult<(u64, usize)> {
        let mut result = 0u64;
        for (i, &b) in buf.iter().take(9).enumerate() {
            if i == 8 {
                result = (result << 8) | (b as u64);
                return Ok((result, 9));
            } else {
                result = (result << 7) | ((b & 0x7F) as u64);
                if (b & 0x80) == 0 {
                    return Ok((result, i + 1));
                }
            }
        }
        Err(ConnectorError::Sqlite(
            "Varint exceeds buffer length".to_string(),
        ))
    }

    /// Decodes a serial value given its serial type code.
    fn decode_serial_value(serial_type: u64, buf: &[u8]) -> ConnectorResult<(DataValue, usize)> {
        match serial_type {
            0 => Ok((DataValue::Null, 0)),
            1 => {
                if buf.is_empty() {
                    return Err(ConnectorError::Sqlite("Buffer underrun for i8".to_string()));
                }
                Ok((DataValue::Int64(buf[0] as i8 as i64), 1))
            }
            2 => {
                if buf.len() < 2 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for i16".to_string(),
                    ));
                }
                let val = i16::from_be_bytes([buf[0], buf[1]]);
                Ok((DataValue::Int64(val as i64), 2))
            }
            3 => {
                if buf.len() < 3 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for i24".to_string(),
                    ));
                }
                let is_neg = (buf[0] & 0x80) != 0;
                let sign_extend = if is_neg { 0xFF } else { 0x00 };
                let val = i32::from_be_bytes([sign_extend, buf[0], buf[1], buf[2]]);
                Ok((DataValue::Int64(val as i64), 3))
            }
            4 => {
                if buf.len() < 4 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for i32".to_string(),
                    ));
                }
                let val = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                Ok((DataValue::Int64(val as i64), 4))
            }
            5 => {
                if buf.len() < 6 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for i48".to_string(),
                    ));
                }
                let is_neg = (buf[0] & 0x80) != 0;
                let sign_extend = if is_neg { 0xFF } else { 0x00 };
                let val = i64::from_be_bytes([
                    sign_extend,
                    sign_extend,
                    buf[0],
                    buf[1],
                    buf[2],
                    buf[3],
                    buf[4],
                    buf[5],
                ]);
                Ok((DataValue::Int64(val), 6))
            }
            6 => {
                if buf.len() < 8 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for i64".to_string(),
                    ));
                }
                let val = i64::from_be_bytes([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                ]);
                Ok((DataValue::Int64(val), 8))
            }
            7 => {
                if buf.len() < 8 {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for f64".to_string(),
                    ));
                }
                let bits = u64::from_be_bytes([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                ]);
                Ok((DataValue::Float64(f64::from_bits(bits)), 8))
            }
            8 => Ok((DataValue::Int64(0), 0)),
            9 => Ok((DataValue::Int64(1), 0)),
            st if st >= 12 && st.is_multiple_of(2) => {
                let len = ((st - 12) / 2) as usize;
                if buf.len() < len {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for BLOB".to_string(),
                    ));
                }
                Ok((DataValue::Binary(buf[..len].to_vec()), len))
            }
            st if st >= 13 && !st.is_multiple_of(2) => {
                let len = ((st - 13) / 2) as usize;
                if buf.len() < len {
                    return Err(ConnectorError::Sqlite(
                        "Buffer underrun for TEXT".to_string(),
                    ));
                }
                let text = String::from_utf8_lossy(&buf[..len]).to_string();
                Ok((DataValue::Text(text), len))
            }
            other => Err(ConnectorError::Sqlite(format!(
                "Reserved or unsupported serial type: {other}"
            ))),
        }
    }
}

/// Pure safe-Rust SQLite database file writer.
/// Generates standard SQLite 3 database files with table B-Tree leaf pages and records.
pub struct SqliteWriter;

impl SqliteWriter {
    /// Writes a table dataset into a valid SQLite 3 database file.
    pub fn write_table(
        table_name: &str,
        schema: &Schema,
        rows: &[Vec<DataValue>],
    ) -> ConnectorResult<Vec<u8>> {
        let page_size: usize = 4096;
        let mut db_bytes = vec![0u8; page_size * 2]; // 2 pages: page 1 (schema) and page 2 (data)

        // 1. Write SQLite Header (100 bytes on Page 1)
        db_bytes[0..16].copy_from_slice(b"SQLite format 3\0");
        db_bytes[16..18].copy_from_slice(&(page_size as u16).to_be_bytes()); // Page size 4096
        db_bytes[18] = 1; // File format write version
        db_bytes[19] = 1; // File format read version
        db_bytes[20] = 0; // Reserved bytes per page
        db_bytes[21] = 64; // Max payload fraction
        db_bytes[22] = 32; // Min payload fraction
        db_bytes[23] = 32; // Leaf payload fraction
        db_bytes[24..28].copy_from_slice(&1u32.to_be_bytes()); // File change counter
        db_bytes[28..32].copy_from_slice(&2u32.to_be_bytes()); // Size of database in pages
        db_bytes[32..36].copy_from_slice(&0u32.to_be_bytes()); // First freelist trunk page
        db_bytes[36..40].copy_from_slice(&0u32.to_be_bytes()); // Total freelist pages
        db_bytes[40..44].copy_from_slice(&1u32.to_be_bytes()); // Schema cookie
        db_bytes[44..48].copy_from_slice(&4u32.to_be_bytes()); // Schema format number
        db_bytes[48..52].copy_from_slice(&0u32.to_be_bytes()); // Default page cache size
        db_bytes[52..56].copy_from_slice(&2u32.to_be_bytes()); // Largest root B-tree page
        db_bytes[56..60].copy_from_slice(&1u32.to_be_bytes()); // Database text encoding (1 = UTF-8)
        db_bytes[60..64].copy_from_slice(&0u32.to_be_bytes()); // User version
        db_bytes[64..68].copy_from_slice(&0u32.to_be_bytes()); // Incremental vacuum mode
        db_bytes[68..72].copy_from_slice(&0u32.to_be_bytes()); // Application ID
                                                               // 72..92: reserved
        db_bytes[92..96].copy_from_slice(&1u32.to_be_bytes()); // Version-valid-for number
        db_bytes[96..100].copy_from_slice(&3045000u32.to_be_bytes()); // SQLite version 3.45.0

        // 2. Build Schema SQL statement for page 1
        let col_defs: Vec<String> = schema
            .fields
            .iter()
            .map(|f| {
                let sql_type = match f.data_type {
                    DataType::Int64 => "INTEGER",
                    DataType::Float64 => "REAL",
                    DataType::Boolean => "INTEGER",
                    _ => "TEXT",
                };
                format!("\"{}\" {}", f.name, sql_type)
            })
            .collect();
        let create_sql = format!("CREATE TABLE \"{}\" ({})", table_name, col_defs.join(", "));

        // Record for sqlite_master: [type, name, tbl_name, rootpage, sql]
        let schema_record = vec![
            DataValue::Text("table".to_string()),
            DataValue::Text(table_name.to_string()),
            DataValue::Text(table_name.to_string()),
            DataValue::Int64(2), // root page 2
            DataValue::Text(create_sql),
        ];

        // Write Page 1 (Leaf Table page, with header starting at byte 100)
        let page1_slice = &mut db_bytes[0..page_size];
        Self::write_leaf_page(page1_slice, 100, &[schema_record], 1)?;

        // 3. Write Page 2 (Leaf Table page for data rows)
        let page2_slice = &mut db_bytes[page_size..page_size * 2];
        Self::write_leaf_page(page2_slice, 0, rows, 1)?;

        Ok(db_bytes)
    }

    /// Formats a SQLite leaf page with cell pointers and cell bodies.
    fn write_leaf_page(
        page: &mut [u8],
        header_offset: usize,
        records: &[Vec<DataValue>],
        start_rowid: i64,
    ) -> ConnectorResult<()> {
        let cell_count = records.len();
        let mut cell_bytes_list = Vec::with_capacity(cell_count);

        for (idx, rec) in records.iter().enumerate() {
            let rowid = start_rowid + idx as i64;
            let payload = Self::encode_record(rec);
            let mut cell_data = Vec::new();
            Self::write_varint(&mut cell_data, payload.len() as u64);
            Self::write_varint(&mut cell_data, rowid as u64);
            cell_data.extend_from_slice(&payload);
            cell_bytes_list.push(cell_data);
        }

        // Place cells from the end of the page moving backwards
        let mut cur_content_offset = page.len();
        let mut cell_pointers = Vec::with_capacity(cell_count);

        for cell_data in &cell_bytes_list {
            if cur_content_offset < cell_data.len() {
                return Err(ConnectorError::Sqlite(
                    "Table page overflow: too many records for single page".to_string(),
                ));
            }
            cur_content_offset -= cell_data.len();
            page[cur_content_offset..cur_content_offset + cell_data.len()]
                .copy_from_slice(cell_data);
            cell_pointers.push(cur_content_offset as u16);
        }

        let cell_content_start = if cur_content_offset == 65536 {
            0
        } else {
            cur_content_offset as u16
        };

        // Write page header (8 bytes):
        // 0: page type (0x0d = table leaf)
        // 1..3: first freeblock offset (0)
        // 3..5: cell count
        // 5..7: cell content start
        // 7: fragmented bytes (0)
        page[header_offset] = 0x0d;
        page[header_offset + 1] = 0;
        page[header_offset + 2] = 0;
        page[header_offset + 3..header_offset + 5]
            .copy_from_slice(&(cell_count as u16).to_be_bytes());
        page[header_offset + 5..header_offset + 7]
            .copy_from_slice(&cell_content_start.to_be_bytes());
        page[header_offset + 7] = 0;

        // Write cell pointer array
        let ptr_start = header_offset + 8;
        for (i, &ptr) in cell_pointers.iter().enumerate() {
            let p_off = ptr_start + i * 2;
            page[p_off..p_off + 2].copy_from_slice(&ptr.to_be_bytes());
        }

        Ok(())
    }

    /// Encodes a row of `DataValue` into a SQLite record payload.
    pub fn encode_record(values: &[DataValue]) -> Vec<u8> {
        let mut serial_types = Vec::new();
        let mut body_bytes = Vec::new();

        for val in values {
            match val {
                DataValue::Null => {
                    serial_types.push(0u64);
                }
                DataValue::Boolean(b) => {
                    serial_types.push(if *b { 9u64 } else { 8u64 });
                }
                DataValue::Int64(i) => {
                    if *i == 0 {
                        serial_types.push(8u64);
                    } else if *i == 1 {
                        serial_types.push(9u64);
                    } else if *i >= i8::MIN as i64 && *i <= i8::MAX as i64 {
                        serial_types.push(1u64);
                        body_bytes.push(*i as u8);
                    } else if *i >= i16::MIN as i64 && *i <= i16::MAX as i64 {
                        serial_types.push(2u64);
                        body_bytes.extend_from_slice(&(*i as i16).to_be_bytes());
                    } else if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                        serial_types.push(4u64);
                        body_bytes.extend_from_slice(&(*i as i32).to_be_bytes());
                    } else {
                        serial_types.push(6u64);
                        body_bytes.extend_from_slice(&i.to_be_bytes());
                    }
                }
                DataValue::Float64(f) => {
                    serial_types.push(7u64);
                    body_bytes.extend_from_slice(&f.to_be_bytes());
                }
                DataValue::Date(d) => {
                    serial_types.push(4u64);
                    body_bytes.extend_from_slice(&d.to_be_bytes());
                }
                DataValue::DateTime(dt) => {
                    serial_types.push(6u64);
                    body_bytes.extend_from_slice(&dt.to_be_bytes());
                }
                DataValue::Text(s) => {
                    let s_bytes = s.as_bytes();
                    let st = (s_bytes.len() as u64) * 2 + 13;
                    serial_types.push(st);
                    body_bytes.extend_from_slice(s_bytes);
                }
                DataValue::Binary(b) => {
                    let st = (b.len() as u64) * 2 + 12;
                    serial_types.push(st);
                    body_bytes.extend_from_slice(b);
                }
            }
        }

        // Calculate header size
        let mut st_varints = Vec::new();
        for st in serial_types {
            Self::write_varint(&mut st_varints, st);
        }

        let mut header_len = 1 + st_varints.len();
        if header_len >= 128 {
            header_len += 1;
        }

        let mut header_bytes = Vec::new();
        Self::write_varint(&mut header_bytes, header_len as u64);
        header_bytes.extend_from_slice(&st_varints);

        let mut payload = header_bytes;
        payload.extend_from_slice(&body_bytes);
        payload
    }

    /// Writes an integer as a SQLite variable-length integer (varint).
    pub fn write_varint(buf: &mut Vec<u8>, mut val: u64) {
        if val <= 0x7F {
            buf.push(val as u8);
            return;
        }

        let mut bytes = Vec::new();
        while val > 0 {
            bytes.push((val & 0x7F) as u8);
            val >>= 7;
        }

        for (i, b) in bytes.iter().rev().enumerate() {
            if i + 1 < bytes.len() {
                buf.push(b | 0x80);
            } else {
                buf.push(*b);
            }
        }
    }
}
