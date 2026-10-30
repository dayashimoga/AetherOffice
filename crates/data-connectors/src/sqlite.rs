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
