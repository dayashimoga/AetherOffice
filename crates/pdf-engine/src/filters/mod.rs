//! PDF Stream Filter Decompression Pipeline (ISO 32000-1 Clause 7.4).

pub mod ascii_85;
pub mod ascii_hex;
pub mod predictor;

use crate::error::PdfError;
use crate::types::PdfObject;
use flate2::read::ZlibDecoder;
use predictor::{apply_predictor, PredictorParams};
use std::collections::BTreeMap;
use std::io::Read;

pub struct FilterPipeline;

impl FilterPipeline {
    /// Decodes raw stream bytes according to `/Filter` and `/DecodeParms`.
    pub fn decode(
        raw_bytes: &[u8],
        filter_obj: Option<&PdfObject>,
        parms_obj: Option<&PdfObject>,
    ) -> Result<Vec<u8>, PdfError> {
        let filters = match filter_obj {
            None => return Ok(raw_bytes.to_vec()),
            Some(PdfObject::Name(name)) => vec![name.as_str()],
            Some(PdfObject::Array(arr)) => {
                let mut list = Vec::new();
                for item in arr {
                    if let Some(name) = item.as_name() {
                        list.push(name);
                    }
                }
                list
            }
            _ => return Ok(raw_bytes.to_vec()),
        };

        let parms_list = match parms_obj {
            Some(PdfObject::Dictionary(d)) => vec![Some(d)],
            Some(PdfObject::Array(arr)) => arr.iter().map(|item| item.as_dict()).collect(),
            _ => vec![None; filters.len()],
        };

        let mut current_data = raw_bytes.to_vec();

        for (i, filter_name) in filters.iter().enumerate() {
            let parms_dict = parms_list.get(i).copied().flatten();
            current_data = Self::apply_single_filter(filter_name, &current_data, parms_dict)?;
        }

        Ok(current_data)
    }

    fn apply_single_filter(
        name: &str,
        data: &[u8],
        parms: Option<&BTreeMap<String, PdfObject>>,
    ) -> Result<Vec<u8>, PdfError> {
        match name {
            "FlateDecode" | "Fl" => {
                let mut decoder = ZlibDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(|e| PdfError::DecompressionError(e.to_string()))?;

                if let Some(dict) = parms {
                    let pred_params = PredictorParams::from_dict(dict);
                    apply_predictor(&decompressed, &pred_params)
                } else {
                    Ok(decompressed)
                }
            }
            "ASCIIHexDecode" | "AHx" => ascii_hex::decode_ascii_hex(data),
            "ASCII85Decode" | "A85" => ascii_85::decode_ascii_85(data),
            unsupported => Err(PdfError::DecompressionError(format!(
                "Unsupported stream filter: /{}",
                unsupported
            ))),
        }
    }
}
