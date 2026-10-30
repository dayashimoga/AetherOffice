//! Streaming Delimited CSV/TSV reader with auto-delimiter detection and batching.

use crate::error::{ConnectorError, ConnectorResult};
use crate::schema::{DataType, Schema, SchemaInferer};
use crate::value::DataValue;
use std::io::BufRead;

/// Delimited CSV/TSV reader configuration options.
#[derive(Debug, Clone)]
pub struct CsvReadOptions {
    pub delimiter: Option<u8>,
    pub has_headers: bool,
    pub quote_char: u8,
    pub batch_size: usize,
    pub max_rows: Option<usize>,
}

impl Default for CsvReadOptions {
    fn default() -> Self {
        Self {
            delimiter: None, // Auto-detect
            has_headers: true,
            quote_char: b'"',
            batch_size: 4096,
            max_rows: None,
        }
    }
}

/// High-throughput streaming CSV parser.
pub struct CsvReader<R: BufRead> {
    reader: R,
    options: CsvReadOptions,
    delimiter: u8,
    headers: Vec<String>,
    schema: Option<Schema>,
    inferer: SchemaInferer,
    line_number: usize,
    rows_emitted: usize,
    exhausted: bool,
}

impl<R: BufRead> CsvReader<R> {
    /// Creates a new CSV reader over a buffered stream.
    pub fn new(mut reader: R, mut options: CsvReadOptions) -> ConnectorResult<Self> {
        let inferer = SchemaInferer::default();
        let mut first_line = String::new();
        let bytes_read = reader.read_line(&mut first_line)?;

        if bytes_read == 0 {
            let delimiter = options.delimiter.unwrap_or(b',');
            return Ok(Self {
                reader,
                options,
                delimiter,
                headers: Vec::new(),
                schema: None,
                inferer,
                line_number: 0,
                rows_emitted: 0,
                exhausted: true,
            });
        }

        // Auto-detect delimiter if not specified
        let delimiter = match options.delimiter {
            Some(d) => d,
            None => Self::detect_delimiter(&first_line),
        };
        options.delimiter = Some(delimiter);

        let mut headers = Vec::new();
        let line_number = 1;

        if options.has_headers {
            let parsed_headers = Self::parse_csv_record(&first_line, delimiter, options.quote_char);
            headers = parsed_headers;
        }

        Ok(Self {
            reader,
            options,
            delimiter,
            headers,
            schema: None,
            inferer,
            line_number,
            rows_emitted: 0,
            exhausted: false,
        })
    }

    /// Auto-detects delimiter by inspecting frequency of common candidates.
    pub fn detect_delimiter(sample_line: &str) -> u8 {
        let candidates = *b",\t;|";
        let mut counts = [0usize; 4];

        let mut in_quotes = false;
        for &b in sample_line.as_bytes() {
            if b == b'"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                for (idx, &cand) in candidates.iter().enumerate() {
                    if b == cand {
                        counts[idx] += 1;
                    }
                }
            }
        }

        let mut max_idx = 0;
        let mut max_count = 0;
        for (i, &cnt) in counts.iter().enumerate() {
            if cnt > max_count {
                max_count = cnt;
                max_idx = i;
            }
        }

        if max_count > 0 {
            candidates[max_idx]
        } else {
            b','
        }
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    pub fn delimiter(&self) -> u8 {
        self.delimiter
    }

    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = Some(schema);
    }

    pub fn schema(&self) -> Option<&Schema> {
        self.schema.as_ref()
    }

    /// Parses a single line/record into individual string columns, handling quotes and escapes.
    pub fn parse_csv_record(line: &str, delimiter: u8, quote_char: u8) -> Vec<String> {
        let mut fields = Vec::new();
        let mut cur_field = String::with_capacity(32);
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut in_quotes = false;

        // Strip trailing \r and \n
        let mut end = len;
        while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
            end -= 1;
        }

        while i < end {
            let b = bytes[i];
            if b == quote_char {
                if in_quotes {
                    // Check for escaped quote ""
                    if i + 1 < end && bytes[i + 1] == quote_char {
                        cur_field.push(quote_char as char);
                        i += 2;
                        continue;
                    } else {
                        in_quotes = false;
                        i += 1;
                        continue;
                    }
                } else {
                    in_quotes = true;
                    i += 1;
                    continue;
                }
            } else if b == delimiter && !in_quotes {
                fields.push(std::mem::take(&mut cur_field));
                i += 1;
                continue;
            } else {
                cur_field.push(b as char);
                i += 1;
            }
        }
        fields.push(cur_field);
        fields
    }

    /// Reads next batch of raw string rows up to `batch_size`.
    pub fn read_batch_raw(&mut self) -> ConnectorResult<Vec<Vec<String>>> {
        if self.exhausted {
            return Ok(Vec::new());
        }

        let mut batch = Vec::with_capacity(self.options.batch_size);
        let mut line_buf = String::new();

        while batch.len() < self.options.batch_size {
            if let Some(max_r) = self.options.max_rows {
                if self.rows_emitted >= max_r {
                    self.exhausted = true;
                    break;
                }
            }

            line_buf.clear();
            let bytes_read = self.reader.read_line(&mut line_buf)?;
            if bytes_read == 0 {
                self.exhausted = true;
                break;
            }
            self.line_number += 1;

            // Handle multi-line quoted fields if necessary
            let mut full_record = line_buf.clone();
            while Self::has_unclosed_quotes(&full_record, self.options.quote_char) {
                line_buf.clear();
                let more = self.reader.read_line(&mut line_buf)?;
                if more == 0 {
                    break;
                }
                self.line_number += 1;
                full_record.push_str(&line_buf);
            }

            // Skip empty trailing lines
            if full_record.trim().is_empty() {
                continue;
            }

            let record =
                Self::parse_csv_record(&full_record, self.delimiter, self.options.quote_char);
            batch.push(record);
            self.rows_emitted += 1;
        }

        Ok(batch)
    }

    fn has_unclosed_quotes(s: &str, quote_char: u8) -> bool {
        let mut count: usize = 0;
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == quote_char {
                if i + 1 < bytes.len() && bytes[i + 1] == quote_char {
                    i += 2;
                    continue;
                }
                count += 1;
            }
            i += 1;
        }
        !count.is_multiple_of(2)
    }

    /// Reads next batch of typed `DataValue` rows coerced against schema.
    pub fn read_batch_typed(&mut self) -> ConnectorResult<Vec<Vec<DataValue>>> {
        let raw_batch = self.read_batch_raw()?;
        if raw_batch.is_empty() {
            return Ok(Vec::new());
        }

        // If schema is not set, infer from first batch
        if self.schema.is_none() {
            let col_count = self
                .headers
                .len()
                .max(raw_batch.first().map(|r| r.len()).unwrap_or(0));
            let header_names: Vec<String> = if !self.headers.is_empty() {
                self.headers.clone()
            } else {
                (0..col_count)
                    .map(|i| format!("Column_{}", i + 1))
                    .collect()
            };
            let inferred = self.inferer.infer_from_rows(&header_names, &raw_batch);
            self.schema = Some(inferred);
        }

        let schema = match self.schema.as_ref() {
            Some(s) => s,
            None => {
                return Err(ConnectorError::Schema(
                    "Schema could not be resolved".to_string(),
                ))
            }
        };

        let mut typed_batch = Vec::with_capacity(raw_batch.len());
        for row in raw_batch {
            let mut typed_row = Vec::with_capacity(schema.fields.len());
            for (idx, field) in schema.fields.iter().enumerate() {
                let token = row.get(idx).map(|s| s.as_str()).unwrap_or("");
                let val = self
                    .inferer
                    .coerce_value(token, field.data_type)
                    .unwrap_or(DataValue::Text(token.to_string()));
                typed_row.push(val);
            }
            typed_batch.push(typed_row);
        }

        Ok(typed_batch)
    }

    /// Reads all records from the CSV stream at once.
    pub fn read_all_typed(&mut self) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)> {
        let mut all_rows = Vec::new();
        while let Ok(batch) = self.read_batch_typed() {
            if batch.is_empty() {
                break;
            }
            all_rows.extend(batch);
        }

        let schema = self.schema.clone().unwrap_or_else(|| {
            let names: Vec<String> = self.headers.clone();
            Schema::new(
                names
                    .into_iter()
                    .map(|n| crate::schema::Field::new(n, DataType::Utf8, true))
                    .collect(),
            )
        });

        Ok((schema, all_rows))
    }
}
