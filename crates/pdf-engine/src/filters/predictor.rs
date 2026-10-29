//! Predictor functions for LZW and Flate decompression (ISO 32000-1 Clause 7.4.4.4, RFC 2083).

use crate::error::PdfError;
use crate::types::PdfObject;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictorParams {
    pub predictor: u32,
    pub colors: usize,
    pub bits_per_component: usize,
    pub columns: usize,
}

impl Default for PredictorParams {
    fn default() -> Self {
        Self {
            predictor: 1, // 1 = No prediction
            colors: 1,
            bits_per_component: 8,
            columns: 1,
        }
    }
}

impl PredictorParams {
    pub fn from_dict(dict: &BTreeMap<String, PdfObject>) -> Self {
        let predictor = dict
            .get("Predictor")
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
            .max(1) as u32;

        let colors = dict
            .get("Colors")
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
            .max(1) as usize;

        let bits_per_component = dict
            .get("BitsPerComponent")
            .and_then(|v| v.as_i64())
            .unwrap_or(8)
            .max(1) as usize;

        let columns = dict
            .get("Columns")
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
            .max(1) as usize;

        Self {
            predictor,
            colors,
            bits_per_component,
            columns,
        }
    }

    #[inline]
    pub fn bytes_per_pixel(&self) -> usize {
        (self.colors * self.bits_per_component).div_ceil(8)
    }

    #[inline]
    pub fn row_bytes(&self) -> usize {
        (self.columns * self.colors * self.bits_per_component).div_ceil(8)
    }
}

/// Inverses prediction on decompressed image / stream data.
pub fn apply_predictor(data: &[u8], params: &PredictorParams) -> Result<Vec<u8>, PdfError> {
    if params.predictor <= 1 {
        return Ok(data.to_vec());
    }

    let bpp = params.bytes_per_pixel().max(1);
    let row_len = params.row_bytes().max(1);

    if params.predictor == 2 {
        // TIFF Predictor 2 (horizontal differencing)
        let mut output = data.to_vec();
        for chunk in output.chunks_mut(row_len) {
            for i in bpp..chunk.len() {
                chunk[i] = chunk[i].wrapping_add(chunk[i - bpp]);
            }
        }
        Ok(output)
    } else if (10..=15).contains(&params.predictor) {
        // PNG Predictors (Each row prefixed with 1-byte filter type)
        let stride = row_len + 1;
        let mut prev_row = vec![0u8; row_len];
        let mut output = Vec::with_capacity((data.len() / stride) * row_len);

        for chunk in data.chunks(stride) {
            if chunk.is_empty() {
                break;
            }
            let filter_type = chunk[0];
            let raw_row = &chunk[1..];
            let mut cur_row = vec![0u8; row_len];

            for (i, &b) in raw_row.iter().enumerate() {
                if i >= row_len {
                    break;
                }
                let left = if i >= bpp { cur_row[i - bpp] } else { 0 };
                let up = prev_row[i];
                let up_left = if i >= bpp { prev_row[i - bpp] } else { 0 };

                let val = match filter_type {
                    0 => b,                                                     // None
                    1 => b.wrapping_add(left),                                  // Sub
                    2 => b.wrapping_add(up),                                    // Up
                    3 => b.wrapping_add(((left as u16 + up as u16) / 2) as u8), // Average
                    4 => b.wrapping_add(paeth_predictor(left, up, up_left)),    // Paeth
                    _ => b,
                };
                cur_row[i] = val;
            }

            output.extend_from_slice(&cur_row);
            prev_row = cur_row;
        }

        Ok(output)
    } else {
        Err(PdfError::DecompressionError(format!(
            "Unsupported predictor: {}",
            params.predictor
        )))
    }
}

#[inline]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = (a as i32) + (b as i32) - (c as i32);
    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
